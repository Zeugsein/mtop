use crate::metrics::GpuMetrics;
use std::sync::OnceLock;

/// GPU metrics via IOReport GPU Performance States channel.
/// Uses dlopen/dlsym to dynamically load IOReport symbols.
/// Falls back gracefully to default values if IOReport is unavailable.
pub fn collect_gpu() -> GpuMetrics {
    read_gpu_ioreport().unwrap_or_default()
}

static IOREPORT_FNS: OnceLock<Option<IOReportFns>> = OnceLock::new();

type CFStringRef = *const libc::c_void;
type CFDictionaryRef = *const libc::c_void;

// Function pointer types for IOReport
type FnCopyChannelsInGroup = unsafe extern "C" fn(CFStringRef, CFStringRef, u64, u64, u64) -> CFDictionaryRef;
type FnCreateSubscription = unsafe extern "C" fn(*const libc::c_void, CFDictionaryRef, *mut i32, u64, *const libc::c_void) -> *const libc::c_void;
type FnCreateSamples = unsafe extern "C" fn(*const libc::c_void, CFDictionaryRef, *const libc::c_void) -> CFDictionaryRef;
type FnCreateSamplesDelta = unsafe extern "C" fn(CFDictionaryRef, CFDictionaryRef, *const libc::c_void) -> CFDictionaryRef;
type FnStateGetCount = unsafe extern "C" fn(CFDictionaryRef) -> i32;
type FnStateGetResidency = unsafe extern "C" fn(CFDictionaryRef, i32) -> u64;
type FnStateGetNameForIndex = unsafe extern "C" fn(CFDictionaryRef, i32) -> CFStringRef;

struct IOReportFns {
    copy_channels: FnCopyChannelsInGroup,
    create_subscription: FnCreateSubscription,
    create_samples: FnCreateSamples,
    create_samples_delta: FnCreateSamplesDelta,
    state_get_count: FnStateGetCount,
    state_get_residency: FnStateGetResidency,
    state_get_name_for_index: FnStateGetNameForIndex,
}

// SAFETY: IOReportFns only holds function pointers from a shared library,
// which are valid for the lifetime of the process and safe to call from any thread.
unsafe impl Send for IOReportFns {}
unsafe impl Sync for IOReportFns {}

fn get_ioreport() -> Option<&'static IOReportFns> {
    IOREPORT_FNS.get_or_init(load_ioreport).as_ref()
}

fn load_ioreport() -> Option<IOReportFns> {
    unsafe {
        // Try multiple possible library paths
        let paths = [
            c"/System/Library/PrivateFrameworks/IOReport.framework/IOReport".as_ptr(),
            c"/usr/lib/libIOReport.dylib".as_ptr(),
        ];

        let mut handle: *mut libc::c_void = std::ptr::null_mut();
        for path in &paths {
            handle = libc::dlopen(*path, libc::RTLD_LAZY);
            if !handle.is_null() {
                break;
            }
        }

        if handle.is_null() {
            return None;
        }

        macro_rules! sym {
            ($name:literal, $ty:ty) => {{
                let p = libc::dlsym(handle, $name.as_ptr() as *const i8);
                if p.is_null() { return None; }
                std::mem::transmute::<*mut libc::c_void, $ty>(p)
            }};
        }

        Some(IOReportFns {
            copy_channels: sym!(b"IOReportCopyChannelsInGroup\0", FnCopyChannelsInGroup),
            create_subscription: sym!(b"IOReportCreateSubscription\0", FnCreateSubscription),
            create_samples: sym!(b"IOReportCreateSamples\0", FnCreateSamples),
            create_samples_delta: sym!(b"IOReportCreateSamplesDelta\0", FnCreateSamplesDelta),
            state_get_count: sym!(b"IOReportStateGetCount\0", FnStateGetCount),
            state_get_residency: sym!(b"IOReportStateGetResidency\0", FnStateGetResidency),
            state_get_name_for_index: sym!(b"IOReportStateGetNameForIndex\0", FnStateGetNameForIndex),
        })
    }
}

fn read_gpu_ioreport() -> Option<GpuMetrics> {
    let fns = get_ioreport()?;

    unsafe {
        let group = cfstring("GPU");
        let sub_group = cfstring("GPU Performance States");

        let channel = (fns.copy_channels)(group, sub_group, 0, 0, 0);
        CFRelease(group as *const _);
        CFRelease(sub_group as *const _);

        if channel.is_null() {
            return None;
        }

        let mut sub_err: i32 = 0;
        let subscription = (fns.create_subscription)(
            std::ptr::null(),
            channel,
            &mut sub_err,
            0,
            std::ptr::null(),
        );

        if subscription.is_null() || sub_err != 0 {
            CFRelease(channel as *const _);
            return None;
        }

        let sample1 = (fns.create_samples)(subscription, channel, std::ptr::null());
        if sample1.is_null() {
            CFRelease(channel as *const _);
            CFRelease(subscription as *const _);
            return None;
        }

        // Delta measurement requires a time gap between initial and final sample.
        // Without this sleep the two samples would be nearly identical, producing
        // zero or wildly noisy residency deltas.
        std::thread::sleep(std::time::Duration::from_millis(50));

        let sample2 = (fns.create_samples)(subscription, channel, std::ptr::null());
        if sample2.is_null() {
            CFRelease(sample1 as *const _);
            CFRelease(channel as *const _);
            CFRelease(subscription as *const _);
            return None;
        }

        let delta = (fns.create_samples_delta)(sample1, sample2, std::ptr::null());
        CFRelease(sample1 as *const _);
        CFRelease(sample2 as *const _);

        if delta.is_null() {
            CFRelease(channel as *const _);
            CFRelease(subscription as *const _);
            return None;
        }

        let result = parse_gpu_delta(fns, delta);

        CFRelease(delta as *const _);
        CFRelease(channel as *const _);
        CFRelease(subscription as *const _);

        result
    }
}

unsafe fn parse_gpu_delta(fns: &IOReportFns, delta: CFDictionaryRef) -> Option<GpuMetrics> {
    let count = unsafe { (fns.state_get_count)(delta) };
    if count <= 0 {
        return None;
    }

    let mut total_residency: u64 = 0;
    let mut active_residency: u64 = 0;
    let mut weighted_freq: u64 = 0;

    for i in 0..count {
        let residency = unsafe { (fns.state_get_residency)(delta, i) };
        total_residency += residency;

        if i > 0 {
            active_residency += residency;
            // Try to read actual frequency from state name (format: "GPUPH_XXXX_YYYY")
            let freq = unsafe { get_state_freq_mhz(fns, delta, i) }
                .unwrap_or(200 + (i as u32) * 200); // fallback to linear estimate
            weighted_freq += residency * freq as u64;
        }
    }

    if total_residency == 0 {
        return Some(GpuMetrics::default());
    }

    let usage = active_residency as f32 / total_residency as f32;
    let freq_mhz = if active_residency > 0 {
        (weighted_freq / active_residency) as u32
    } else {
        0
    };

    Some(GpuMetrics {
        freq_mhz,
        usage,
        power_w: 0.0,
    })
}

/// Extract frequency in MHz from IOReport state name.
/// State names are formatted as "GPUPH_XXXX_YYYY" where YYYY is freq in MHz.
/// The returned CFStringRef is borrowed (Get rule) — do NOT release it.
unsafe fn get_state_freq_mhz(fns: &IOReportFns, delta: CFDictionaryRef, index: i32) -> Option<u32> {
    let name_cf = unsafe { (fns.state_get_name_for_index)(delta, index) };
    if name_cf.is_null() {
        return None;
    }
    let name = unsafe { cfstring_to_string(name_cf) };
    // Do NOT CFRelease name_cf — it is a borrowed reference (Get rule)
    if name.len() >= 4 {
        name[name.len() - 4..].parse::<u32>().ok()
    } else {
        None
    }
}

unsafe fn cfstring(s: &str) -> CFStringRef {
    let cstr = std::ffi::CString::new(s).unwrap_or_default();
    unsafe { CFStringCreateWithCString(std::ptr::null(), cstr.as_ptr(), 0x08000100) }
}

unsafe fn cfstring_to_string(cf: CFStringRef) -> String {
    let len = unsafe { CFStringGetLength(cf) };
    if len <= 0 {
        return String::new();
    }
    let max_size = unsafe { CFStringGetMaximumSizeForEncoding(len, 0x08000100) } + 1;
    let mut buf = vec![0u8; max_size as usize];
    let ok = unsafe { CFStringGetCString(cf, buf.as_mut_ptr() as *mut i8, max_size, 0x08000100) };
    if ok {
        let cstr = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr() as *const i8) };
        cstr.to_string_lossy().to_string()
    } else {
        String::new()
    }
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFStringCreateWithCString(
        alloc: *const libc::c_void,
        cstr: *const i8,
        encoding: u32,
    ) -> CFStringRef;
    fn CFStringGetLength(cf: CFStringRef) -> i64;
    fn CFStringGetMaximumSizeForEncoding(length: i64, encoding: u32) -> i64;
    fn CFStringGetCString(cf: CFStringRef, buffer: *mut i8, max_size: i64, encoding: u32) -> bool;
    fn CFRelease(cf: *const libc::c_void);
}
