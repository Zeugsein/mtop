use crate::metrics::CpuMetrics;

/// Read per-CPU utilization ticks via Mach host_processor_info
pub fn collect_cpu(prev_ticks: &mut Vec<(u64, u64)>, e_cores: u32, _p_cores: u32) -> CpuMetrics {
    let mut metrics = CpuMetrics::default();

    unsafe {
        let host = libc::mach_host_self();
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
            libc::mach_task_self(),
            info as libc::vm_address_t,
            (msg_count as usize * std::mem::size_of::<i32>()) as libc::vm_size_t,
        );
    }

    metrics
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
}
