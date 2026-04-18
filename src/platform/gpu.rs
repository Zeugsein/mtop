use crate::metrics::GpuMetrics;
use crate::platform::ioreport_ffi::{self, CFDictionaryRef, CFArrayRef, CFStringRef, IOReportFns, CFRelease};

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
            let group = ioreport_ffi::cfstring("GPU Stats");
            let sub_group = ioreport_ffi::cfstring("GPU Performance States");

            let channel = (fns.copy_channels)(group, sub_group, 0, 0, 0);
            CFRelease(group as *const _);
            CFRelease(sub_group as *const _);

            if channel.is_null() {
                return None;
            }

            let mut desired_channels: ioreport_ffi::CFDictionaryRef = std::ptr::null();
            let subscription = (fns.create_subscription)(
                std::ptr::null(),
                channel,
                &mut desired_channels,
                0,
                std::ptr::null(),
            );

            if subscription.is_null() {
                CFRelease(channel as *const _);
                return None;
            }
            // desired_channels is an output ref we don't need; release if non-null
            if !desired_channels.is_null() {
                CFRelease(desired_channels as *const _);
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
    // Extract IOReportChannels array from the delta dictionary
    let items_key = unsafe { ioreport_ffi::cfstring("IOReportChannels") };
    let items = unsafe { ioreport_ffi::CFDictionaryGetValue(delta, items_key as *const _) };
    unsafe { CFRelease(items_key as *const _) };

    if items.is_null() {
        return None;
    }

    let channel_count = unsafe { ioreport_ffi::CFArrayGetCount(items as CFArrayRef) };
    if channel_count <= 0 {
        return None;
    }

    // Find the GPUPH channel by name
    let mut gpuph_entry: *const libc::c_void = std::ptr::null();
    for i in 0..channel_count {
        let entry = unsafe { ioreport_ffi::CFArrayGetValueAtIndex(items as CFArrayRef, i) };
        if entry.is_null() {
            continue;
        }
        let name_cf = unsafe { (fns.channel_get_channel_name)(entry as CFDictionaryRef) };
        if name_cf.is_null() {
            continue;
        }
        let name_str = unsafe { ioreport_ffi::cfstring_to_string(name_cf) };
        // Do NOT CFRelease name_cf — it is a borrowed reference (Get rule)
        if name_str.contains("GPUPH") {
            gpuph_entry = entry;
            break;
        }
    }

    if gpuph_entry.is_null() {
        return None;
    }

    let channel = gpuph_entry as CFDictionaryRef;
    let count = unsafe { (fns.state_get_count)(channel) };
    if count <= 0 {
        return None;
    }

    let mut total_residency: u64 = 0;
    let mut active_residency: u64 = 0;
    let mut weighted_freq: u64 = 0;

    for i in 0..count {
        let residency = unsafe { (fns.state_get_residency)(channel, i) };
        total_residency += residency;

        if i > 0 {
            active_residency += residency;
            // Try to read actual frequency from state name (format: "GPUPH_XXXX_YYYY")
            let freq = unsafe { get_state_freq_mhz(fns, channel, i) }
                .unwrap_or(200 + (i as u32) * 200); // fallback to linear estimate
            weighted_freq += residency * freq as u64;
        }
    }

    if total_residency == 0 {
        return Some(GpuMetrics::default());
    }

    let usage = active_residency as f32 / total_residency as f32;
    let freq_mhz = weighted_freq.checked_div(active_residency).unwrap_or(0) as u32;

    Some(GpuMetrics {
        freq_mhz,
        usage,
        power_w: 0.0,
        available: true,
    })
}

/// Extract frequency in MHz from IOReport state name.
/// State names are formatted as "GPUPH_XXXX_YYYY" where YYYY is freq in MHz.
/// The returned CFStringRef is borrowed (Get rule) — do NOT release it.
unsafe fn get_state_freq_mhz(fns: &IOReportFns, channel: CFDictionaryRef, index: i32) -> Option<u32> {
    let name_cf: CFStringRef = unsafe { (fns.state_get_name_for_index)(channel, index) };
    if name_cf.is_null() {
        return None;
    }
    let name = unsafe { ioreport_ffi::cfstring_to_string(name_cf) };
    // Do NOT CFRelease name_cf — it is a borrowed reference (Get rule)
    // State name format: "GPUPH_XXXX_YYYY" where YYYY is freq in MHz.
    // Try last 4 chars first, then search for any trailing numeric segment.
    if name.len() >= 4 && let Ok(freq) = name[name.len() - 4..].parse::<u32>() {
        return Some(freq);
    }
    // Fallback: extract last numeric segment from the name
    name.rsplit(|c: char| !c.is_ascii_digit())
        .find(|s| !s.is_empty())
        .and_then(|s| s.parse::<u32>().ok())
}
