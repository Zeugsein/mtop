use crate::metrics::CpuMetrics;

/// Read per-CPU utilization ticks via Mach host_processor_info
pub fn collect_cpu(host: u32, prev_ticks: &mut Vec<(u64, u64)>, e_cores: u32, _p_cores: u32) -> CpuMetrics {
    let mut metrics = CpuMetrics::default();

    unsafe {
        let mut count: u32 = 0;
        let mut info: *mut i32 = std::ptr::null_mut();
        let mut msg_count: u32 = 0;

        let ret = host_processor_info(
            host,
            PROCESSOR_CPU_LOAD_INFO,
            &mut count,
            &mut info as *mut *mut i32 as *mut *mut _,
            &mut msg_count,
        );

        if ret != 0 || info.is_null() {
            return metrics;
        }

        let ncpu = count as usize;
        let mut core_usages = Vec::with_capacity(ncpu);
        let mut new_ticks = Vec::with_capacity(ncpu);

        for i in 0..ncpu {
            let base = info.add(i * CPU_STATE_MAX as usize);
            let user = *base.add(CPU_STATE_USER as usize) as u64;
            let system = *base.add(CPU_STATE_SYSTEM as usize) as u64;
            let idle = *base.add(CPU_STATE_IDLE as usize) as u64;
            let nice = *base.add(CPU_STATE_NICE as usize) as u64;

            let active = user + system + nice;
            let total = active + idle;

            new_ticks.push((active, total));

            if i < prev_ticks.len() {
                let (prev_active, prev_total) = prev_ticks[i];
                let d_active = active.saturating_sub(prev_active) as f32;
                let d_total = total.saturating_sub(prev_total) as f32;
                let usage = if d_total > 0.0 { d_active / d_total } else { 0.0 };
                core_usages.push(usage);
            } else {
                core_usages.push(0.0);
            }
        }

        *prev_ticks = new_ticks;

        // Split into E and P cluster usages
        let e_count = e_cores as usize;
        let e_usage: f32 = if e_count > 0 && e_count <= core_usages.len() {
            core_usages[..e_count].iter().sum::<f32>() / e_count as f32
        } else {
            0.0
        };
        let p_usage: f32 = if e_count < core_usages.len() {
            let p_slice = &core_usages[e_count..];
            if !p_slice.is_empty() { p_slice.iter().sum::<f32>() / p_slice.len() as f32 } else { 0.0 }
        } else {
            0.0
        };

        let total_usage = if !core_usages.is_empty() {
            core_usages.iter().sum::<f32>() / core_usages.len() as f32
        } else {
            0.0
        };

        metrics.e_cluster.usage = e_usage;
        metrics.p_cluster.usage = p_usage;
        metrics.total_usage = total_usage;
        metrics.core_usages = core_usages;

        // Deallocate
        libc::vm_deallocate(
            mach_task_self(),
            info as libc::vm_address_t,
            (msg_count as usize * std::mem::size_of::<i32>()) as libc::vm_size_t,
        );
    }

    metrics
}

/// Read CPU frequency from sysctl for a given perflevel (0=P-cores, 1=E-cores).
/// Returns the nominal frequency in MHz, or 0 if unavailable.
pub fn sysctl_cpu_freq(perflevel: u32) -> u32 {
    let name = format!("hw.perflevel{}.cpuspeeds", perflevel);
    let cname = match std::ffi::CString::new(name) {
        Ok(c) => c,
        Err(_) => return 0,
    };

    // Try cpuspeeds first (array of frequencies)
    let mut val: u64 = 0;
    let mut size = std::mem::size_of::<u64>() as libc::size_t;
    let ret = unsafe {
        libc::sysctlbyname(
            cname.as_ptr(),
            &mut val as *mut u64 as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret == 0 && val > 0 {
        return (val / 1_000_000) as u32; // Hz to MHz
    }

    // Fallback: try hw.cpufrequency
    let fallback = std::ffi::CString::new("hw.cpufrequency").unwrap_or_default();
    let mut freq: u64 = 0;
    let mut fsize = std::mem::size_of::<u64>() as libc::size_t;
    let ret = unsafe {
        libc::sysctlbyname(
            fallback.as_ptr(),
            &mut freq as *mut u64 as *mut libc::c_void,
            &mut fsize,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret == 0 && freq > 0 {
        return (freq / 1_000_000) as u32;
    }

    // Last fallback: estimate from chip name
    let chip_name = std::ffi::CString::new("machdep.cpu.brand_string").unwrap_or_default();
    let mut size: libc::size_t = 0;
    unsafe {
        libc::sysctlbyname(chip_name.as_ptr(), std::ptr::null_mut(), &mut size, std::ptr::null_mut(), 0);
        if size > 0 {
            let mut buf = vec![0u8; size];
            libc::sysctlbyname(
                chip_name.as_ptr(),
                buf.as_mut_ptr() as *mut libc::c_void,
                &mut size,
                std::ptr::null_mut(),
                0,
            );
            let name = String::from_utf8_lossy(&buf).to_lowercase();
            // Estimate nominal frequencies for known Apple Silicon chips
            if perflevel == 0 {
                // P-cores
                if name.contains("m4") { return 4400; }
                if name.contains("m3") { return 4050; }
                if name.contains("m2") { return 3490; }
                if name.contains("m1") { return 3200; }
            } else {
                // E-cores
                if name.contains("m4") { return 2800; }
                if name.contains("m3") { return 2750; }
                if name.contains("m2") { return 2420; }
                if name.contains("m1") { return 2064; }
            }
        }
    }

    0
}

// Mach API constants and extern
const PROCESSOR_CPU_LOAD_INFO: i32 = 2;
const CPU_STATE_USER: i32 = 0;
const CPU_STATE_SYSTEM: i32 = 1;
const CPU_STATE_IDLE: i32 = 2;
const CPU_STATE_NICE: i32 = 3;
const CPU_STATE_MAX: i32 = 4;

unsafe extern "C" {
    fn host_processor_info(
        host: u32,
        flavor: i32,
        out_count: *mut u32,
        out_info: *mut *mut u8,
        out_info_count: *mut u32,
    ) -> i32;
    fn mach_task_self() -> u32;
}
