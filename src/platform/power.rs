use crate::metrics::PowerMetrics;

/// Power metrics placeholder — requires IOReport Energy Model for real data.
/// For MVP, we provide stub values. IOReport integration will be added
/// incrementally as it requires complex CoreFoundation FFI.
pub fn collect_power() -> PowerMetrics {
    // TODO: Implement IOReport Energy Model reading
    // This requires:
    // 1. IOReportCopyChannelsInGroup for "Energy Model" group
    // 2. IOReportCreateSubscription + IOReportCreateSamplesDelta
    // 3. Parsing channel values for CPU/GPU/ANE/DRAM power
    PowerMetrics::default()
}
