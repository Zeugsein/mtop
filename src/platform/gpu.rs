use crate::metrics::GpuMetrics;

/// GPU metrics placeholder — requires IOReport for real data.
/// For MVP, we provide stub values. IOReport integration is complex
/// and will be added incrementally.
pub fn collect_gpu() -> GpuMetrics {
    // TODO: Implement IOReport-based GPU frequency and utilization reading
    // This requires subscribing to IOReport channels for GPU Performance States
    // and computing weighted averages from DVFS residency data
    GpuMetrics::default()
}
