use crate::metrics::PowerMetrics;

/// Power metrics via IOReport Energy Model channel.
/// Uses dlopen/dlsym to dynamically load IOReport symbols.
/// Falls back gracefully to default values if IOReport is unavailable.
pub fn collect_power() -> PowerMetrics {
    read_power_ioreport().unwrap_or_default()
}

type CFStringRef = *const libc::c_void;
type CFDictionaryRef = *const libc::c_void;
type CFArrayRef = *const libc::c_void;

// Function pointer types for IOReport
type FnCopyChannelsInGroup = unsafe extern "C" fn(CFStringRef, CFStringRef, u64, u64, u64) -> CFDictionaryRef;
type FnCreateSubscription = unsafe extern "C" fn(*const libc::c_void, CFDictionaryRef, *mut i32, u64, *const libc::c_void) -> *const libc::c_void;
type FnCreateSamples = unsafe extern "C" fn(*const libc::c_void, CFDictionaryRef, *const libc::c_void) -> CFDictionaryRef;
type FnCreateSamplesDelta = unsafe extern "C" fn(CFDictionaryRef, CFDictionaryRef, *const libc::c_void) -> CFDictionaryRef;
type FnChannelGetChannelName = unsafe extern "C" fn(CFDictionaryRef) -> CFStringRef;
type FnSimpleGetIntegerValue = unsafe extern "C" fn(CFDictionaryRef, *mut i32) -> i64;

struct IOReportFns {
    copy_channels: FnCopyChannelsInGroup,
    create_subscription: FnCreateSubscription,
    create_samples: FnCreateSamples,
    create_samples_delta: FnCreateSamplesDelta,
    channel_get_channel_name: FnChannelGetChannelName,
    simple_get_integer_value: FnSimpleGetIntegerValue,
}

fn load_ioreport() -> Option<IOReportFns> {
    unsafe {
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
            channel_get_channel_name: sym!(b"IOReportChannelGetChannelName\0", FnChannelGetChannelName),
            simple_get_integer_value: sym!(b"IOReportSimpleGetIntegerValue\0", FnSimpleGetIntegerValue),
        })
    }
}

fn read_power_ioreport() -> Option<PowerMetrics> {
    let fns = load_ioreport()?;

    unsafe {
        let group = cfstring("Energy Model");
        let channel = (fns.copy_channels)(group, std::ptr::null(), 0, 0, 0);
        CFRelease(group as *const _);

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

        std::thread::sleep(std::time::Duration::from_millis(100));

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

        let result = parse_power_delta(&fns, delta);

        CFRelease(delta as *const _);
        CFRelease(channel as *const _);
        CFRelease(subscription as *const _);

        result
    }
}

unsafe fn parse_power_delta(fns: &IOReportFns, delta: CFDictionaryRef) -> Option<PowerMetrics> {
    // The delta is a CFDictionary containing an array of channel samples
    // Each channel has a name like "CPU Energy", "GPU Energy", etc.
    // Values are in energy units (nJ typically); divide by duration to get watts

    let items_key = unsafe { cfstring("IOReportChannels") };
    let items = unsafe { CFDictionaryGetValue(delta, items_key as *const _) };
    unsafe { CFRelease(items_key as *const _) };

    if items.is_null() {
        return None;
    }

    let count = unsafe { CFArrayGetCount(items as CFArrayRef) };
    if count <= 0 {
        return None;
    }

    let mut cpu_energy: i64 = 0;
    let mut gpu_energy: i64 = 0;
    let mut ane_energy: i64 = 0;
    let mut dram_energy: i64 = 0;

    let duration_ms: f64 = 100.0; // Our sample interval

    for i in 0..count {
        let item = unsafe { CFArrayGetValueAtIndex(items as CFArrayRef, i) };
        if item.is_null() {
            continue;
        }

        let name = unsafe { (fns.channel_get_channel_name)(item) };
        if name.is_null() {
            continue;
        }

        let name_str = unsafe { cfstring_to_string(name) };
        let mut err: i32 = 0;
        let value = unsafe { (fns.simple_get_integer_value)(item, &mut err) };

        if err != 0 {
            continue;
        }

        let name_lower = name_str.to_lowercase();
        if name_lower.contains("cpu") && name_lower.contains("energy") {
            cpu_energy += value;
        } else if name_lower.contains("gpu") && name_lower.contains("energy") {
            gpu_energy += value;
        } else if name_lower.contains("ane") && name_lower.contains("energy") {
            ane_energy += value;
        } else if name_lower.contains("dram") && name_lower.contains("energy") {
            dram_energy += value;
        }
    }

    // Convert energy (nJ) to watts: W = nJ / (ms * 1e6)
    let nj_to_watts = |nj: i64| -> f32 {
        if nj <= 0 { return 0.0; }
        (nj as f64 / (duration_ms * 1_000_000.0)) as f32
    };

    let cpu_w = nj_to_watts(cpu_energy);
    let gpu_w = nj_to_watts(gpu_energy);
    let ane_w = nj_to_watts(ane_energy);
    let dram_w = nj_to_watts(dram_energy);
    let package_w = cpu_w + gpu_w + ane_w;
    let system_w = package_w + dram_w;

    Some(PowerMetrics {
        cpu_w,
        gpu_w,
        ane_w,
        dram_w,
        package_w,
        system_w,
    })
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
    fn CFDictionaryGetValue(dict: CFDictionaryRef, key: *const libc::c_void) -> *const libc::c_void;
    fn CFArrayGetCount(array: CFArrayRef) -> i64;
    fn CFArrayGetValueAtIndex(array: CFArrayRef, idx: i64) -> *const libc::c_void;
    fn CFRelease(cf: *const libc::c_void);
}
