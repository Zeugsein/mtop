use crate::metrics::ThermalMetrics;

/// Temperature metrics placeholder — requires SMC access for real data.
/// SMC reading requires IOKit service matching and structured key reads.
/// For MVP, we provide stub values.
pub fn collect_temperature() -> ThermalMetrics {
    // TODO: Implement SMC temperature reading
    // This requires:
    // 1. IOServiceOpen to AppleSMC service
    // 2. IOConnectCallStructMethod to read temperature keys
    // 3. Decoding SP78 fixed-point format
    // 4. Averaging across CPU/GPU sensor keys
    ThermalMetrics::default()
}
