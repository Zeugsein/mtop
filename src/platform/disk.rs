use crate::metrics::DiskMetrics;

/// Disk I/O state for delta computation between samples
pub struct DiskState {
    prev_read: u64,
    prev_write: u64,
    prev_time: std::time::Instant,
}

impl DiskState {
    pub fn new() -> Self {
        let (r, w) = read_disk_bytes();
        Self {
            prev_read: r,
            prev_write: w,
            prev_time: std::time::Instant::now(),
        }
    }

    pub fn collect(&mut self) -> DiskMetrics {
        // Touch disk to ensure IOKit counters advance between samples
        trigger_io();
        let (cur_read, cur_write) = read_disk_bytes();
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.prev_time).as_secs_f64().max(0.001);

        let read_bps = ((cur_read.saturating_sub(self.prev_read)) as f64 / dt) as u64;
        let write_bps = ((cur_write.saturating_sub(self.prev_write)) as f64 / dt) as u64;

        self.prev_read = cur_read;
        self.prev_write = cur_write;
        self.prev_time = now;

        DiskMetrics {
            read_bytes_sec: read_bps,
            write_bytes_sec: write_bps,
        }
    }
}

/// Perform a small write+sync to ensure IOKit disk counters advance.
fn trigger_io() {
    use std::io::Write;
    let path = std::env::temp_dir().join(".mtop_io_probe");
    if let Ok(mut f) = std::fs::File::create(&path) {
        let _ = f.write_all(b"probe");
        let _ = f.sync_all();
    }
    let _ = std::fs::remove_file(&path);
}

/// Read total disk bytes read/written via IOKit IOBlockStorageDriver.
/// Uses IORegistryEntryCreateCFProperty (singular) to get fresh Statistics
/// on each call, avoiding the caching behavior of CreateCFProperties.
fn read_disk_bytes() -> (u64, u64) {
    unsafe {
        let mut total_read: u64 = 0;
        let mut total_write: u64 = 0;

        let matching = IOServiceMatching(b"IOBlockStorageDriver\0".as_ptr() as *const i8);
        if matching.is_null() {
            return (0, 0);
        }

        let mut iter: u32 = 0;
        let kr = IOServiceGetMatchingServices(MASTER_PORT, matching, &mut iter);
        if kr != 0 || iter == 0 {
            return (0, 0);
        }

        let stats_key = cfstring_from_static(b"Statistics\0");
        let read_key = cfstring_from_static(b"Bytes (Read)\0");
        let write_key = cfstring_from_static(b"Bytes (Write)\0");

        loop {
            let service = IOIteratorNext(iter);
            if service == 0 {
                break;
            }

            // Use CreateCFProperty (singular) which reads the property fresh
            let stats = IORegistryEntryCreateCFProperty(
                service,
                stats_key,
                std::ptr::null(),
                0,
            );

            if !stats.is_null() {
                total_read += cf_number_value(CFDictionaryGetValue(stats as CFDictionaryRef, read_key as *const _));
                total_write += cf_number_value(CFDictionaryGetValue(stats as CFDictionaryRef, write_key as *const _));
                CFRelease(stats as *const _);
            }

            IOObjectRelease(service);
        }

        CFRelease(stats_key as *const _);
        CFRelease(read_key as *const _);
        CFRelease(write_key as *const _);
        IOObjectRelease(iter);

        (total_read, total_write)
    }
}

unsafe fn cfstring_from_static(bytes: &[u8]) -> CFStringRef {
    // bytes includes null terminator
    CFStringCreateWithCString(std::ptr::null(), bytes.as_ptr() as *const i8, 0x08000100) // kCFStringEncodingUTF8
}

unsafe fn cf_number_value(val: *const libc::c_void) -> u64 {
    if val.is_null() {
        return 0;
    }
    let mut out: i64 = 0;
    // kCFNumberSInt64Type = 4
    let ok = CFNumberGetValue(val as CFNumberRef, 4, &mut out as *mut _ as *mut libc::c_void);
    if ok {
        out as u64
    } else {
        0
    }
}

// --- IOKit and CoreFoundation FFI ---
const MASTER_PORT: u32 = 0; // kIOMasterPortDefault

type CFStringRef = *const libc::c_void;
type CFDictionaryRef = *const libc::c_void;
type CFMutableDictionaryRef = *mut libc::c_void;
type CFNumberRef = *const libc::c_void;
type CFAllocatorRef = *const libc::c_void;

#[link(name = "IOKit", kind = "framework")]
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn IOServiceMatching(name: *const i8) -> CFMutableDictionaryRef;
    fn IOServiceGetMatchingServices(
        main_port: u32,
        matching: CFMutableDictionaryRef,
        existing: *mut u32,
    ) -> i32;
    fn IOIteratorNext(iterator: u32) -> u32;
    fn IOObjectRelease(object: u32) -> i32;
    fn IORegistryEntryCreateCFProperty(
        entry: u32,
        key: CFStringRef,
        allocator: CFAllocatorRef,
        options: u32,
    ) -> *const libc::c_void;

    fn CFDictionaryGetValue(dict: CFDictionaryRef, key: *const libc::c_void) -> *const libc::c_void;
    fn CFStringCreateWithCString(
        alloc: CFAllocatorRef,
        cstr: *const i8,
        encoding: u32,
    ) -> CFStringRef;
    fn CFNumberGetValue(number: CFNumberRef, the_type: i32, value_ptr: *mut libc::c_void) -> bool;
    fn CFRelease(cf: *const libc::c_void);
}
