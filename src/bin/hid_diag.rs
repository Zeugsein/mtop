/// Diagnostic binary for HID temperature sensor enumeration.
/// Run: cargo run --bin hid_diag
/// Prints step-by-step what SetMatching + CopyServices finds on this hardware.
fn main() {
    unsafe { run() };
}

const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
const K_CF_NUMBER_SINT32_TYPE: i64 = 3;
const K_HID_PAGE_APPLE_VENDOR: i32 = 0xff00;
const K_HID_USAGE_APPLE_VENDOR_TEMP_SENSOR: i32 = 0x0005;

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IOHIDEventSystemClientCreate(allocator: *const libc::c_void) -> *const libc::c_void;
    fn IOHIDEventSystemClientSetMatching(client: *const libc::c_void, matching: *const libc::c_void) -> i32;
    fn IOHIDEventSystemClientCopyServices(client: *const libc::c_void) -> *const libc::c_void;
    fn IOHIDServiceClientCopyEvent(service: *const libc::c_void, event_type: i64, sub_type: i32, options: i64) -> *const libc::c_void;
    fn IOHIDEventGetFloatValue(event: *const libc::c_void, field: i64) -> f64;
    fn IOHIDServiceClientCopyProperty(service: *const libc::c_void, key: *const libc::c_void) -> *const libc::c_void;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFAllocatorDefault: *const libc::c_void;
    static kCFTypeDictionaryKeyCallBacks: libc::c_void;
    static kCFTypeDictionaryValueCallBacks: libc::c_void;
    fn CFRelease(cf: *const libc::c_void);
    fn CFArrayGetCount(array: *const libc::c_void) -> i64;
    fn CFArrayGetValueAtIndex(array: *const libc::c_void, idx: i64) -> *const libc::c_void;
    fn CFStringCreateWithCString(alloc: *const libc::c_void, c_str: *const i8, encoding: u32) -> *const libc::c_void;
    fn CFStringGetCStringPtr(the_string: *const libc::c_void, encoding: u32) -> *const i8;
    fn CFStringGetCString(the_string: *const libc::c_void, buffer: *mut i8, buffer_size: i64, encoding: u32) -> bool;
    fn CFStringGetLength(the_string: *const libc::c_void) -> i64;
    fn CFDictionaryCreate(allocator: *const libc::c_void, keys: *const *const libc::c_void, values: *const *const libc::c_void, num_values: i64, key_callbacks: *const libc::c_void, value_callbacks: *const libc::c_void) -> *const libc::c_void;
    fn CFNumberCreate(allocator: *const libc::c_void, the_type: i64, value_ptr: *const libc::c_void) -> *const libc::c_void;
}

unsafe fn cfstr(s: &str) -> *const libc::c_void {
    let c = std::ffi::CString::new(s).unwrap();
    CFStringCreateWithCString(kCFAllocatorDefault, c.as_ptr(), K_CF_STRING_ENCODING_UTF8)
}

unsafe fn from_cfstr(cf: *const libc::c_void) -> String {
    let ptr = CFStringGetCStringPtr(cf, K_CF_STRING_ENCODING_UTF8);
    if !ptr.is_null() {
        return std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned();
    }
    let len = CFStringGetLength(cf);
    let buf_size = (len * 4 + 1) as usize;
    if buf_size > 512 { return "<too long>".into(); }
    let mut buf = vec![0i8; buf_size];
    if CFStringGetCString(cf, buf.as_mut_ptr(), buf_size as i64, K_CF_STRING_ENCODING_UTF8) {
        std::ffi::CStr::from_ptr(buf.as_ptr()).to_string_lossy().into_owned()
    } else {
        "<cfstr decode failed>".into()
    }
}

unsafe fn run() {
    println!("=== HID temperature diagnostic ===");

    // Phase 1: no filter — enumerate all services
    println!("\n--- Phase 1: no filter (all services) ---");
    let client_all = IOHIDEventSystemClientCreate(kCFAllocatorDefault);
    if client_all.is_null() { println!("CLIENT CREATE FAILED"); return; }
    let services_all = IOHIDEventSystemClientCopyServices(client_all);
    if services_all.is_null() {
        println!("CopyServices (no filter) returned null");
    } else {
        let n = CFArrayGetCount(services_all);
        println!("CopyServices (no filter): {} services", n);
        let mut temp_event_count = 0i64;
        for i in 0..n {
            let sc = CFArrayGetValueAtIndex(services_all, i);
            if sc.is_null() { continue; }
            let ev = IOHIDServiceClientCopyEvent(sc, 15, 0, 0);
            if !ev.is_null() {
                let t = IOHIDEventGetFloatValue(ev, 15 << 16);
                CFRelease(ev);
                let name_cf = IOHIDServiceClientCopyProperty(sc, cfstr("Product"));
                let name = if name_cf.is_null() { "<no Product>".into() } else {
                    let s = from_cfstr(name_cf);
                    CFRelease(name_cf);
                    s
                };
                println!("  [{}] temp={:.1}°C  name={}", i, t, name);
                temp_event_count += 1;
            }
        }
        println!("  total services with temp events: {}", temp_event_count);
        CFRelease(services_all);
    }
    CFRelease(client_all);

    // Phase 2: with Apple Vendor temperature sensor filter
    println!("\n--- Phase 2: with SetMatching filter ---");
    let client = IOHIDEventSystemClientCreate(kCFAllocatorDefault);
    if client.is_null() { println!("CLIENT CREATE FAILED"); return; }

    let key_page = cfstr("PrimaryUsagePage");
    let key_usage = cfstr("PrimaryUsage");
    let val_page = CFNumberCreate(kCFAllocatorDefault, K_CF_NUMBER_SINT32_TYPE, &K_HID_PAGE_APPLE_VENDOR as *const i32 as *const libc::c_void);
    let val_usage = CFNumberCreate(kCFAllocatorDefault, K_CF_NUMBER_SINT32_TYPE, &K_HID_USAGE_APPLE_VENDOR_TEMP_SENSOR as *const i32 as *const libc::c_void);
    let keys: [*const libc::c_void; 2] = [key_page, key_usage];
    let vals: [*const libc::c_void; 2] = [val_page, val_usage];
    let dict = CFDictionaryCreate(kCFAllocatorDefault, keys.as_ptr(), vals.as_ptr(), 2, &kCFTypeDictionaryKeyCallBacks, &kCFTypeDictionaryValueCallBacks);
    CFRelease(key_page); CFRelease(key_usage); CFRelease(val_page); CFRelease(val_usage);

    if dict.is_null() {
        println!("CFDictionaryCreate FAILED");
        CFRelease(client);
        return;
    }

    let ret = IOHIDEventSystemClientSetMatching(client, dict);
    CFRelease(dict);
    println!("SetMatching returned: {}", ret);

    let services = IOHIDEventSystemClientCopyServices(client);
    if services.is_null() {
        println!("CopyServices (filtered) returned null");
        CFRelease(client);
        return;
    }

    let count = CFArrayGetCount(services);
    println!("CopyServices (filtered): {} services", count);

    let product_key = cfstr("Product");
    for i in 0..count {
        let sc = CFArrayGetValueAtIndex(services, i);
        if sc.is_null() { println!("  [{}] null service", i); continue; }

        let name_cf = IOHIDServiceClientCopyProperty(sc, product_key);
        let name = if name_cf.is_null() { "<no Product>".into() } else {
            let s = from_cfstr(name_cf);
            CFRelease(name_cf);
            s
        };

        let ev = IOHIDServiceClientCopyEvent(sc, 15, 0, 0);
        if ev.is_null() {
            println!("  [{}] name={:?}  event=null", i, name);
        } else {
            let t = IOHIDEventGetFloatValue(ev, 15 << 16);
            CFRelease(ev);
            println!("  [{}] name={:?}  temp={:.1}°C", i, name, t);
        }
    }

    CFRelease(product_key);
    CFRelease(services);
    CFRelease(client);

    println!("\n=== done ===");
}
