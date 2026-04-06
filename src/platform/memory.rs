use crate::metrics::MemoryMetrics;

pub fn collect_memory(host: u32) -> MemoryMetrics {
    let ram_total = sysctl_u64("hw.memsize").unwrap_or(0);

    // Get VM statistics via Mach API
    let (ram_used, swap_total, swap_used) = unsafe {
        let mut vm_stat: VmStatistics64 = std::mem::zeroed();
        let mut count = (std::mem::size_of::<VmStatistics64>() / std::mem::size_of::<i32>()) as u32;

        let ret = host_statistics64(
            host,
            HOST_VM_INFO64,
            &mut vm_stat as *mut _ as *mut i32,
            &mut count,
        );

        let ram_used = if ret == 0 {
            let page_size = libc::sysconf(libc::_SC_PAGESIZE) as u64;
            (vm_stat.active_count as u64
                + vm_stat.inactive_count as u64
                + vm_stat.wire_count as u64
                + vm_stat.compressor_page_count as u64)
                * page_size
        } else {
            0
        };

        // Swap via sysctl
        let swap = get_swap_usage();

        (ram_used, swap.0, swap.1)
    };

    MemoryMetrics {
        ram_total,
        ram_used,
        swap_total,
        swap_used,
    }
}

fn sysctl_u64(name: &str) -> Option<u64> {
    let cname = std::ffi::CString::new(name).ok()?;
    let mut val: u64 = 0;
    let mut size = std::mem::size_of::<u64>() as libc::size_t;
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
    _padding: i32,
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

const HOST_VM_INFO64: i32 = 4;

unsafe extern "C" {
    fn host_statistics64(host: u32, flavor: i32, info: *mut i32, count: *mut u32) -> i32;
}
