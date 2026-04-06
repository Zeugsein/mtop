use crate::metrics::GpuMetrics;
use crate::platform::ioreport_ffi::{self, CFDictionaryRef, CFStringRef, IOReportFns, CFRelease};

/// Stateful GPU collector. Stores IOReport subscription and previous sample
/// to compute deltas without internal sleeps.
pub struct GpuState {
    channel: CFDictionaryRef,
    subscription: *const libc::c_void,
    prev_sample: CFDictionaryRef,
}

// SAFETY: The CF objects are only accessed from the sampler thread.
unsafe impl Send for GpuState {}

impl GpuState {
    /// Create a new GPU state with IOReport subscription.
    /// Returns None if IOReport is unavailable.
    pub fn new() -> Option<Self> {
        let fns = ioreport_ffi::get_ioreport()?;

        unsafe {
            let group = ioreport_ffi::cfstring("GPU");
            let sub_group = ioreport_ffi::cfstring("GPU Performance States");

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

            let prev_sample = (fns.create_samples)(subscription, channel, std::ptr::null());
            if prev_sample.is_null() {
                CFRelease(channel as *const _);
                CFRelease(subscription as *const _);
                return None;
            }

            Some(Self {
                channel,
                subscription,
                prev_sample,
            })
        }
    }

    /// Collect GPU metrics by taking a new sample and computing delta against previous.
    /// First call after new() returns default metrics (no time gap yet for meaningful delta).
    pub fn collect(&mut self) -> GpuMetrics {
        let fns = match ioreport_ffi::get_ioreport() {
            Some(f) => f,
            None => return GpuMetrics::default(),
        };

        unsafe {
            let new_sample = (fns.create_samples)(self.subscription, self.channel, std::ptr::null());
            if new_sample.is_null() {
                return GpuMetrics::default();
            }

            let delta = (fns.create_samples_delta)(self.prev_sample, new_sample, std::ptr::null());
            CFRelease(self.prev_sample as *const _);
            self.prev_sample = new_sample;

            if delta.is_null() {
                return GpuMetrics::default();
            }

            let result = parse_gpu_delta(fns, delta).unwrap_or_default();
            CFRelease(delta as *const _);
            result
        }
    }
}

impl Drop for GpuState {
    fn drop(&mut self) {
        unsafe {
            if !self.prev_sample.is_null() {
                CFRelease(self.prev_sample as *const _);
            }
            if !self.channel.is_null() {
                CFRelease(self.channel as *const _);
            }
            if !self.subscription.is_null() {
                CFRelease(self.subscription as *const _);
            }
        }
    }
}

/// Fallback for when IOReport is unavailable — returns default metrics.
pub fn collect_gpu() -> GpuMetrics {
    GpuMetrics::default()
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
    let name_cf: CFStringRef = unsafe { (fns.state_get_name_for_index)(delta, index) };
    if name_cf.is_null() {
        return None;
    }
    let name = unsafe { ioreport_ffi::cfstring_to_string(name_cf) };
    // Do NOT CFRelease name_cf — it is a borrowed reference (Get rule)
    if name.len() >= 4 {
        name[name.len() - 4..].parse::<u32>().ok()
    } else {
        None
    }
}
