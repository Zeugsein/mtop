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
    let mut last_unit = "nJ".to_string();

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

        if name_str.ends_with("CPU Energy") {
            cpu_energy += value;
        } else if name_str == "GPU Energy" {
            gpu_energy += value;
        } else if name_str.starts_with("ANE") {
            ane_energy += value;
        } else if name_str.starts_with("DRAM") {
            dram_energy += value;
        } else {
            continue;
        }

        // Read unit label dynamically for this channel
        let unit_cf = unsafe { (fns.channel_get_unit_label)(item) };
        let unit_str = if !unit_cf.is_null() {
            unsafe { ioreport_ffi::cfstring_to_string(unit_cf) }
            // Do NOT CFRelease unit_cf — it is a borrowed reference (Get rule)
        } else {
            "nJ".to_string() // default fallback
        };

        // Track unit per-channel; store for conversion below
        // For simplicity, use last-seen unit (all channels in Energy Model use same unit)
        last_unit = unit_str;
    }

    let duration_s = duration_ms / 1000.0;
    if duration_s < 1e-6 {
        return Some(PowerMetrics { available: true, ..Default::default() });
    }
    // Convert energy to watts based on unit
    let energy_to_watts = |energy: i64| -> f32 {
        if energy <= 0 { return 0.0; }
        let joules = match last_unit.as_str() {
            "uJ" => energy as f64 / 1e6,
            "mJ" => energy as f64 / 1e3,
            _ => energy as f64 / 1e9, // nJ default
        };
        (joules / duration_s) as f32
    };

    let cpu_w = energy_to_watts(cpu_energy);
    let gpu_w = energy_to_watts(gpu_energy);
    let ane_w = energy_to_watts(ane_energy);
    let dram_w = energy_to_watts(dram_energy);
    let package_w = cpu_w + gpu_w + ane_w;
    let system_w = package_w + dram_w;

    Some(PowerMetrics {
        cpu_w,
        gpu_w,
        ane_w,
        dram_w,
        package_w,
        system_w,
        available: true,
    })
}
