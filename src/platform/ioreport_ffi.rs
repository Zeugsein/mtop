//! Shared IOReport FFI — single dlopen for both gpu.rs and power.rs.

use std::sync::OnceLock;

pub type CFStringRef = *const libc::c_void;
pub type CFDictionaryRef = *const libc::c_void;
pub type CFArrayRef = *const libc::c_void;

// Function pointer types for IOReport
pub type FnCopyChannelsInGroup = unsafe extern "C" fn(CFStringRef, CFStringRef, u64, u64, u64) -> CFDictionaryRef;
pub type FnCreateSubscription = unsafe extern "C" fn(*const libc::c_void, CFDictionaryRef, *mut i32, u64, *const libc::c_void) -> *const libc::c_void;
pub type FnCreateSamples = unsafe extern "C" fn(*const libc::c_void, CFDictionaryRef, *const libc::c_void) -> CFDictionaryRef;
pub type FnCreateSamplesDelta = unsafe extern "C" fn(CFDictionaryRef, CFDictionaryRef, *const libc::c_void) -> CFDictionaryRef;
pub type FnChannelGetChannelName = unsafe extern "C" fn(CFDictionaryRef) -> CFStringRef;
pub type FnSimpleGetIntegerValue = unsafe extern "C" fn(CFDictionaryRef, *mut i32) -> i64;
pub type FnStateGetCount = unsafe extern "C" fn(CFDictionaryRef) -> i32;
pub type FnStateGetResidency = unsafe extern "C" fn(CFDictionaryRef, i32) -> u64;
pub type FnStateGetNameForIndex = unsafe extern "C" fn(CFDictionaryRef, i32) -> CFStringRef;
pub type FnChannelGetUnitLabel = unsafe extern "C" fn(CFDictionaryRef) -> CFStringRef;

pub struct IOReportFns {
    pub copy_channels: FnCopyChannelsInGroup,
    pub create_subscription: FnCreateSubscription,
    pub create_samples: FnCreateSamples,
    pub create_samples_delta: FnCreateSamplesDelta,
    pub channel_get_channel_name: FnChannelGetChannelName,
    pub simple_get_integer_value: FnSimpleGetIntegerValue,
    pub state_get_count: FnStateGetCount,
    pub state_get_residency: FnStateGetResidency,
    pub state_get_name_for_index: FnStateGetNameForIndex,
    pub channel_get_unit_label: FnChannelGetUnitLabel,
}

// SAFETY: IOReportFns only holds function pointers from a shared library,
// which are valid for the lifetime of the process and safe to call from any thread.
unsafe impl Send for IOReportFns {}
unsafe impl Sync for IOReportFns {}

static IOREPORT_FNS: OnceLock<Option<IOReportFns>> = OnceLock::new();

pub fn get_ioreport() -> Option<&'static IOReportFns> {
    IOREPORT_FNS.get_or_init(load_ioreport).as_ref()
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
            state_get_count: sym!(b"IOReportStateGetCount\0", FnStateGetCount),
            state_get_residency: sym!(b"IOReportStateGetResidency\0", FnStateGetResidency),
            state_get_name_for_index: sym!(b"IOReportStateGetNameForIndex\0", FnStateGetNameForIndex),
            channel_get_unit_label: sym!(b"IOReportChannelGetUnitLabel\0", FnChannelGetUnitLabel),
        })
    }
}

// --- Shared CoreFoundation helpers ---

/// # Safety
/// Caller must ensure the CoreFoundation framework is loaded.
pub unsafe fn cfstring(s: &str) -> CFStringRef {
    let cstr = std::ffi::CString::new(s).unwrap_or_default();
    unsafe { CFStringCreateWithCString(std::ptr::null(), cstr.as_ptr(), 0x08000100) }
}

/// # Safety
/// `cf` must be a valid CFStringRef or null (returns empty string if null-length).
pub unsafe fn cfstring_to_string(cf: CFStringRef) -> String {
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
    pub fn CFStringCreateWithCString(
        alloc: *const libc::c_void,
        cstr: *const i8,
        encoding: u32,
    ) -> CFStringRef;
    pub fn CFStringGetLength(cf: CFStringRef) -> i64;
    pub fn CFStringGetMaximumSizeForEncoding(length: i64, encoding: u32) -> i64;
    pub fn CFStringGetCString(cf: CFStringRef, buffer: *mut i8, max_size: i64, encoding: u32) -> bool;
    pub fn CFDictionaryGetValue(dict: CFDictionaryRef, key: *const libc::c_void) -> *const libc::c_void;
    pub fn CFArrayGetCount(array: CFArrayRef) -> i64;
    pub fn CFArrayGetValueAtIndex(array: CFArrayRef, idx: i64) -> *const libc::c_void;
    pub fn CFRelease(cf: *const libc::c_void);
}
