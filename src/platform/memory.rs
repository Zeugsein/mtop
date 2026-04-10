use crate::metrics::MemoryMetrics;

/// Stateful memory collector — tracks previous swapins/swapouts for I/O rate calculation.
pub struct MemoryState {
    prev_swapins: u64,
    prev_swapouts: u64,
    prev_time: std::time::Instant,
}

impl Default for MemoryState {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryState {
    pub fn new() -> Self {
        Self {
            prev_swapins: 0,
            prev_swapouts: 0,
            prev_time: std::time::Instant::now(),
        }
    }

    pub fn collect(&mut self, host: u32) -> MemoryMetrics {
        let mut metrics = collect_memory(host);

        // Swap I/O rates from VmStatistics64 swapins/swapouts delta
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.prev_time).as_secs_f64().max(0.001);

        let (swapins, swapouts) = get_swap_page_counts(host);
        let page_size = get_page_size();

        if self.prev_swapins > 0 || self.prev_swapouts > 0 {
            // Not first sample
            metrics.swap_in_bytes_sec = swapins.saturating_sub(self.prev_swapins) as f64 * page_size as f64 / dt;
            metrics.swap_out_bytes_sec = swapouts.saturating_sub(self.prev_swapouts) as f64 * page_size as f64 / dt;
        }

        self.prev_swapins = swapins;
        self.prev_swapouts = swapouts;
        self.prev_time = now;

        // Memory pressure level via sysctl
        metrics.pressure_level = get_memory_pressure_level();

        metrics
    }
}

pub fn collect_memory(host: u32) -> MemoryMetrics {
    let ram_total = sysctl_u64("hw.memsize").unwrap_or(0);

    // Get VM statistics via Mach API
    // SAFETY: VmStatistics64 is repr(C) with compile-time offset assertions.
    // host_statistics64 writes into vm_stat; count is set to the struct size in i32 units.
    // sysconf(_SC_PAGESIZE) returns the system page size (always positive on macOS).
    let (ram_used, swap_total, swap_used, wired, app, compressed) = unsafe {
        let mut vm_stat: VmStatistics64 = std::mem::zeroed();
        let mut count = (std::mem::size_of::<VmStatistics64>() / std::mem::size_of::<i32>()) as u32;

        let ret = host_statistics64(
            host,
            HOST_VM_INFO64,
            &mut vm_stat as *mut _ as *mut i32,
            &mut count,
        );

        let raw = libc::sysconf(libc::_SC_PAGESIZE);
        let page_size = if raw <= 0 { 16384u64 } else { raw as u64 };

        let (ram_used, wired, app, compressed) = if ret == 0 {
            let used = (vm_stat.active_count as u64
                + vm_stat.inactive_count as u64
                + vm_stat.wire_count as u64
                + vm_stat.compressor_page_count as u64)
                * page_size;
            let wired = vm_stat.wire_count as u64 * page_size;
            let app = (vm_stat.internal_page_count as u64)
                .saturating_sub(vm_stat.purgeable_count as u64)
                * page_size;
            let compressed = vm_stat.compressor_page_count as u64 * page_size;
            (used, wired, app, compressed)
        } else {
            (0, 0, 0, 0)
        };

        // Swap via sysctl
        let swap = get_swap_usage();

        (ram_used, swap.0, swap.1, wired, app, compressed)
    };

    MemoryMetrics {
        ram_total,
        ram_used,
        swap_total,
        swap_used,
        wired,
        app,
        compressed,
        swap_in_bytes_sec: 0.0,
        swap_out_bytes_sec: 0.0,
        pressure_level: 1,
    }
}

fn sysctl_u64(name: &str) -> Option<u64> {
    let cname = std::ffi::CString::new(name).ok()?;
    let mut val: u64 = 0;
    let mut size = std::mem::size_of::<u64>() as libc::size_t;
    // SAFETY: cname is a valid NUL-terminated C string; val is a properly-sized u64 buffer.
    unsafe {
        let ret = libc::sysctlbyname(
            cname.as_ptr(),
            &mut val as *mut u64 as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        );
        if ret == 0 { Some(val) } else { None }
    }
}

fn get_swap_usage() -> (u64, u64) {
    let name = match std::ffi::CString::new("vm.swapusage") {
        Ok(n) => n,
        Err(_) => return (0, 0),
    };
    // SAFETY: XswUsage is repr(C); sysctlbyname writes exactly sizeof(xsw_usage) bytes.
    let mut swap: XswUsage = unsafe { std::mem::zeroed() };
    let mut size = std::mem::size_of::<XswUsage>() as libc::size_t;
    unsafe {
        let ret = libc::sysctlbyname(
            name.as_ptr(),
            &mut swap as *mut _ as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        );
        if ret == 0 {
            (swap.xsu_total, swap.xsu_used)
        } else {
            (0, 0)
        }
    }
}

#[repr(C)]
struct XswUsage {
    xsu_total: u64,
    xsu_avail: u64,
    xsu_used: u64,
    xsu_encrypted: i32,
    xsu_pagesize: i32,
}

#[repr(C)]
#[derive(Default)]
struct VmStatistics64 {
    free_count: u32,
    active_count: u32,
    inactive_count: u32,
    wire_count: u32,
    zero_fill_count: u64,
    reactivations: u64,
    pageins: u64,
    pageouts: u64,
    faults: u64,
    cow_faults: u64,
    lookups: u64,
    hits: u64,
    purges: u64,
    purgeable_count: u32,
    speculative_count: u32,
    decompressions: u64,
    compressions: u64,
    swapins: u64,
    swapouts: u64,
    compressor_page_count: u32,
    throttled_count: u32,
    external_page_count: u32,
    internal_page_count: u32,
    total_uncompressed_pages_in_compressor: u64,
    swapped_count: u64,
}

// Compile-time assertions: VmStatistics64 field offsets (from macOS mach/vm_statistics.h).
// 4 × u32 (0-15), then u64 fields with alignment, then mixed u32/u64 tail.
const _: () = assert!(std::mem::offset_of!(VmStatistics64, wire_count) == 12);
const _: () = assert!(std::mem::offset_of!(VmStatistics64, compressor_page_count) == 128);
const _: () = assert!(std::mem::offset_of!(VmStatistics64, internal_page_count) == 140);

const HOST_VM_INFO64: i32 = 4;

fn get_page_size() -> u64 {
    // SAFETY: sysconf always succeeds for _SC_PAGESIZE on macOS.
    let raw = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    if raw <= 0 { 16384 } else { raw as u64 }
}

/// Read cumulative swap page counts from vm_statistics64.
fn get_swap_page_counts(host: u32) -> (u64, u64) {
    // SAFETY: same pattern as collect_memory — zeroed struct, host_statistics64 fills it.
    unsafe {
        let mut vm_stat: VmStatistics64 = std::mem::zeroed();
        let mut count = (std::mem::size_of::<VmStatistics64>() / std::mem::size_of::<i32>()) as u32;
        let ret = host_statistics64(
            host,
            HOST_VM_INFO64,
            &mut vm_stat as *mut _ as *mut i32,
            &mut count,
        );
        if ret == 0 {
            (vm_stat.swapins, vm_stat.swapouts)
        } else {
            (0, 0)
        }
    }
}

/// Read macOS memory pressure level via sysctl.
/// Returns: 1=normal, 2=warning, 4=critical. Defaults to 1 on failure.
fn get_memory_pressure_level() -> u8 {
    let name = match std::ffi::CString::new("kern.memorystatus_vm_pressure_level") {
        Ok(n) => n,
        Err(_) => return 1,
    };
    let mut val: i32 = 0;
    let mut size = std::mem::size_of::<i32>() as libc::size_t;
    // SAFETY: name is a valid C string; val is a properly-sized i32 buffer.
    let ret = unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            &mut val as *mut i32 as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret == 0 {
        match val {
            1 => 1, // normal
            2 => 2, // warning
            4 => 4, // critical
            _ => 1, // unknown → normal
        }
    } else {
        1 // fallback: normal
    }
}

unsafe extern "C" {
    fn host_statistics64(host: u32, flavor: i32, info: *mut i32, count: *mut u32) -> i32;
}
