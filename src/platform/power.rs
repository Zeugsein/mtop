use crate::metrics::PowerMetrics;
use crate::platform::ioreport_ffi::{self, CFDictionaryRef, CFArrayRef, IOReportFns, CFRelease};

/// Stateful power collector. Stores IOReport subscription, previous sample,
/// and timestamp to compute deltas without internal sleeps.
pub struct PowerState {
    channel: CFDictionaryRef,
    subscription: *const libc::c_void,
    prev_sample: CFDictionaryRef,
    prev_time: std::time::Instant,
}

// SAFETY: The CF objects are only accessed from the sampler thread.
unsafe impl Send for PowerState {}

impl PowerState {
    /// Create a new power state with IOReport subscription.
    /// Returns None if IOReport is unavailable.
    pub fn new() -> Option<Self> {
        let fns = ioreport_ffi::get_ioreport()?;

        unsafe {
            let group = ioreport_ffi::cfstring("Energy Model");
            let channel = (fns.copy_channels)(group, std::ptr::null(), 0, 0, 0);
            CFRelease(group as *const _);

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
                prev_time: std::time::Instant::now(),
            })
        }
    }

    /// Collect power metrics by taking a new sample and computing delta against previous.
    /// Uses actual elapsed time for nanojoules-to-watts conversion.
    pub fn collect(&mut self) -> PowerMetrics {
        let fns = match ioreport_ffi::get_ioreport() {
            Some(f) => f,
            None => return PowerMetrics::default(),
        };

        unsafe {
            let new_sample = (fns.create_samples)(self.subscription, self.channel, std::ptr::null());
            if new_sample.is_null() {
                return PowerMetrics::default();
            }

            let now = std::time::Instant::now();
            let duration_ms = now.duration_since(self.prev_time).as_secs_f64() * 1000.0;

            let delta = (fns.create_samples_delta)(self.prev_sample, new_sample, std::ptr::null());
            CFRelease(self.prev_sample as *const _);
            self.prev_sample = new_sample;
            self.prev_time = now;

            if delta.is_null() {
                return PowerMetrics::default();
            }

            let result = parse_power_delta(fns, delta, duration_ms).unwrap_or_default();
            CFRelease(delta as *const _);
            result
        }
    }
}

impl Drop for PowerState {
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
pub fn collect_power() -> PowerMetrics {
    PowerMetrics::default()
}

unsafe fn parse_power_delta(fns: &IOReportFns, delta: CFDictionaryRef, duration_ms: f64) -> Option<PowerMetrics> {
    let items_key = unsafe { ioreport_ffi::cfstring("IOReportChannels") };
    let items = unsafe { ioreport_ffi::CFDictionaryGetValue(delta, items_key as *const _) };
    unsafe { CFRelease(items_key as *const _) };

    if items.is_null() {
        return None;
    }

    let count = unsafe { ioreport_ffi::CFArrayGetCount(items as CFArrayRef) };
    if count <= 0 {
        return None;
    }

    let mut cpu_energy: i64 = 0;
    let mut gpu_energy: i64 = 0;
    let mut ane_energy: i64 = 0;
    let mut dram_energy: i64 = 0;

    for i in 0..count {
        let item = unsafe { ioreport_ffi::CFArrayGetValueAtIndex(items as CFArrayRef, i) };
        if item.is_null() {
            continue;
        }

        let name = unsafe { (fns.channel_get_channel_name)(item) };
        if name.is_null() {
            continue;
        }

        let name_str = unsafe { ioreport_ffi::cfstring_to_string(name) };
        let mut err: i32 = 0;
        let value = unsafe { (fns.simple_get_integer_value)(item, &mut err) };

        if err != 0 {
            continue;
        }

        let name_lower = name_str.to_lowercase();
        if name_lower.contains("cpu") && name_lower.contains("energy") {
            cpu_energy += value;
        } else if name_lower.contains("gpu") && name_lower.contains("energy") {
            gpu_energy += value;
        } else if name_lower.contains("ane") && name_lower.contains("energy") {
            ane_energy += value;
        } else if name_lower.contains("dram") && name_lower.contains("energy") {
            dram_energy += value;
        }
    }

    // Convert energy (nJ) to watts: W = nJ / (ms * 1e6)
    let nj_to_watts = |nj: i64| -> f32 {
        if nj <= 0 { return 0.0; }
        (nj as f64 / (duration_ms * 1_000_000.0)) as f32
    };

    let cpu_w = nj_to_watts(cpu_energy);
    let gpu_w = nj_to_watts(gpu_energy);
    let ane_w = nj_to_watts(ane_energy);
    let dram_w = nj_to_watts(dram_energy);
    let package_w = cpu_w + gpu_w + ane_w;
    let system_w = package_w + dram_w;

    Some(PowerMetrics {
        cpu_w,
        gpu_w,
        ane_w,
        dram_w,
        package_w,
        system_w,
    })
}
