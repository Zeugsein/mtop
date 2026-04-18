use crate::metrics::BatteryMetrics;

type CFTypeRef = *const libc::c_void;
type CFStringRef = *const libc::c_void;
type CFArrayRef = *const libc::c_void;
type CFDictionaryRef = *const libc::c_void;
type CFBooleanRef = *const libc::c_void;
type CFNumberRef = *const libc::c_void;
type CFAllocatorRef = *const libc::c_void;

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IOPSCopyPowerSourcesInfo() -> CFTypeRef;
    fn IOPSCopyPowerSourcesList(blob: CFTypeRef) -> CFArrayRef;
    fn IOPSGetPowerSourceDescription(blob: CFTypeRef, ps: CFTypeRef) -> CFDictionaryRef;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFArrayGetCount(array: CFArrayRef) -> i64;
    fn CFArrayGetValueAtIndex(array: CFArrayRef, idx: i64) -> CFTypeRef;
    fn CFDictionaryGetValue(dict: CFDictionaryRef, key: *const libc::c_void)
    -> *const libc::c_void;
    fn CFStringCreateWithCString(
        alloc: CFAllocatorRef,
        cstr: *const i8,
        encoding: u32,
    ) -> CFStringRef;
    fn CFStringCompare(a: CFStringRef, b: CFStringRef, flags: u64) -> i32;
    fn CFNumberGetValue(number: CFNumberRef, the_type: i32, value_ptr: *mut libc::c_void) -> bool;
    fn CFBooleanGetValue(boolean: CFBooleanRef) -> bool;
    fn CFRelease(cf: *const libc::c_void);
}

const K_CF_STRING_ENCODING_UTF8: u32 = 0x08000100;
const K_CF_NUMBER_INT_TYPE: i32 = 9; // kCFNumberIntType

fn cfstr(s: &str) -> CFStringRef {
    let cstr = std::ffi::CString::new(s).unwrap();
    unsafe { CFStringCreateWithCString(std::ptr::null(), cstr.as_ptr(), K_CF_STRING_ENCODING_UTF8) }
}

fn dict_get_int(dict: CFDictionaryRef, key: &str) -> Option<i32> {
    let cf_key = cfstr(key);
    let val = unsafe { CFDictionaryGetValue(dict, cf_key) };
    unsafe { CFRelease(cf_key) };
    if val.is_null() {
        return None;
    }
    let mut result: i32 = 0;
    let ok = unsafe {
        CFNumberGetValue(
            val as CFNumberRef,
            K_CF_NUMBER_INT_TYPE,
            &mut result as *mut i32 as *mut libc::c_void,
        )
    };
    if ok { Some(result) } else { None }
}

fn dict_get_bool(dict: CFDictionaryRef, key: &str) -> Option<bool> {
    let cf_key = cfstr(key);
    let val = unsafe { CFDictionaryGetValue(dict, cf_key) };
    unsafe { CFRelease(cf_key) };
    if val.is_null() {
        return None;
    }
    Some(unsafe { CFBooleanGetValue(val as CFBooleanRef) })
}

fn dict_get_string_eq(dict: CFDictionaryRef, key: &str, expected: &str) -> bool {
    let cf_key = cfstr(key);
    let val = unsafe { CFDictionaryGetValue(dict, cf_key) };
    unsafe { CFRelease(cf_key) };
    if val.is_null() {
        return false;
    }
    let cf_expected = cfstr(expected);
    let cmp = unsafe { CFStringCompare(val as CFStringRef, cf_expected, 0) };
    unsafe { CFRelease(cf_expected) };
    cmp == 0
}

/// Collect battery metrics via IOPSCopyPowerSourcesInfo.
pub fn collect_battery() -> BatteryMetrics {
    unsafe {
        let blob = IOPSCopyPowerSourcesInfo();
        if blob.is_null() {
            return BatteryMetrics::default();
        }
        let sources = IOPSCopyPowerSourcesList(blob);
        if sources.is_null() {
            CFRelease(blob);
            return BatteryMetrics::default();
        }

        let count = CFArrayGetCount(sources);
        if count == 0 {
            CFRelease(sources);
            CFRelease(blob);
            return BatteryMetrics::default();
        }

        // Use the first power source (internal battery)
        let ps = CFArrayGetValueAtIndex(sources, 0);
        let desc = IOPSGetPowerSourceDescription(blob, ps);
        if desc.is_null() {
            CFRelease(sources);
            CFRelease(blob);
            return BatteryMetrics::default();
        }

        let is_present = dict_get_bool(desc, "Is Present").unwrap_or(false);
        if !is_present {
            CFRelease(sources);
            CFRelease(blob);
            return BatteryMetrics {
                is_present: false,
                ..Default::default()
            };
        }

        let current_capacity = dict_get_int(desc, "Current Capacity").unwrap_or(0);
        let max_capacity = dict_get_int(desc, "Max Capacity").unwrap_or(100);
        let charge_pct = if max_capacity > 0 {
            ((current_capacity as f32 / max_capacity as f32) * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        let is_charging = dict_get_string_eq(desc, "Power Source State", "AC Power")
            && !dict_get_bool(desc, "Is Charged").unwrap_or(false);

        let is_on_ac = dict_get_string_eq(desc, "Power Source State", "AC Power");

        CFRelease(sources);
        CFRelease(blob);

        BatteryMetrics {
            is_present: true,
            charge_pct,
            is_charging,
            is_on_ac,
        }
    }
}
