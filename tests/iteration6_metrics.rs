/// Integration tests for mtop iteration 6 metrics infrastructure.
///
/// Covers:
///   - `mtop::platform::network::speed_tier_from_baudrate`
///   - `mtop::platform::process::weighted_score`

// ---------------------------------------------------------------------------
// Speed tier from baudrate
// ---------------------------------------------------------------------------

// These tests verify that `speed_tier_from_baudrate` converts a raw link-speed
// baudrate (bits/s as reported by if_data.ifi_baudrate) to a bytes/s ceiling
// used for sparkline scaling.  The mapping is:
//   1 Gbps       → 125_000_000 B/s
//   100 Mbps     →  12_500_000 B/s
//   10 Mbps      →   1_250_000 B/s
//   0 (unknown)  →   1_250_000 B/s (conservative fallback)

#[test]
/// speed_tier_from_baudrate maps 1 Gbps to 125 MB/s ceiling
fn speed_tier_1gbps_maps_to_125_mb_per_sec() {
    use mtop::platform::network::speed_tier_from_baudrate;
    assert_eq!(speed_tier_from_baudrate(1_000_000_000), 125_000_000);
}

#[test]
/// speed_tier_from_baudrate maps 100 Mbps to 12.5 MB/s ceiling
fn speed_tier_100mbps_maps_to_12_5_mb_per_sec() {
    use mtop::platform::network::speed_tier_from_baudrate;
    assert_eq!(speed_tier_from_baudrate(100_000_000), 12_500_000);
}

#[test]
/// speed_tier_from_baudrate maps 10 Mbps to 1.25 MB/s ceiling
fn speed_tier_10mbps_maps_to_1_25_mb_per_sec() {
    use mtop::platform::network::speed_tier_from_baudrate;
    assert_eq!(speed_tier_from_baudrate(10_000_000), 1_250_000);
}

#[test]
/// speed_tier_from_baudrate falls back to 1.25 MB/s when baudrate is 0 (unknown link)
fn speed_tier_zero_baudrate_uses_fallback() {
    use mtop::platform::network::speed_tier_from_baudrate;
    assert_eq!(speed_tier_from_baudrate(0), 1_250_000);
}

// ---------------------------------------------------------------------------
// Weighted process sort score
// ---------------------------------------------------------------------------

// `weighted_score` computes a composite rank for a process so it can be sorted
// across CPU %, memory, and power dimensions simultaneously.
//
// Signature: `pub fn weighted_score(proc: &ProcessInfo, max_cpu: f32, max_mem: u64, max_power: f32) -> f64`
//
// Score = 0.5 * cpu_norm + 0.3 * mem_norm + 0.2 * power_norm
// Spike bonus: if any single normalized value > 0.9, add 0.5 to score.

#[test]
/// weighted_score: 90% CPU-only process ranks above 30% CPU + 30% memory process
fn weighted_score_high_cpu_ranks_above_split_load() {
    use mtop::platform::process::weighted_score;
    use mtop::metrics::types::ProcessInfo;

    let high_cpu = ProcessInfo {
        pid: 1,
        name: "stress".to_string(),
        cpu_pct: 0.90,
        mem_bytes: 0,
        energy_nj: 0,
        power_w: 0.0,
        user: "user".to_string(),
    };

    let split_load = ProcessInfo {
        pid: 2,
        name: "mixed".to_string(),
        cpu_pct: 0.30,
        mem_bytes: 307_200_000, // 30% of ~1 GB max
        energy_nj: 0,
        power_w: 0.0,
        user: "user".to_string(),
    };

    let max_cpu = 1.0_f32;
    let max_mem: u64 = 1_024_000_000; // ~1 GB
    let max_power = 0.0_f32; // power not relevant for this comparison

    let score_high_cpu = weighted_score(&high_cpu, max_cpu, max_mem, max_power);
    let score_split = weighted_score(&split_load, max_cpu, max_mem, max_power);

    assert!(
        score_high_cpu > score_split,
        "90% CPU process (score {score_high_cpu:.4}) should rank above \
         30% CPU + 30% mem process (score {score_split:.4})"
    );
}

#[test]
/// weighted_score: broad load across 3 dimensions outranks a single-dimension spike
/// below the 0.9 spike-bonus threshold.
///
/// The algorithm applies a +0.5 spike bonus when any single norm exceeds 0.9.
/// This test uses an 85% CPU-only process (no bonus triggered) vs a process at
/// 70% across all three dimensions, confirming multi-dimension breadth wins
/// when neither process hits the spike bonus.
fn weighted_score_multi_dimension_beats_sub_threshold_spike() {
    use mtop::platform::process::weighted_score;
    use mtop::metrics::types::ProcessInfo;

    // Process A: high CPU (85%) — below the 0.9 spike-bonus threshold, no mem/power
    // Score = 0.5 * 0.85 = 0.425
    let single_spike = ProcessInfo {
        pid: 10,
        name: "spike".to_string(),
        cpu_pct: 0.85,
        mem_bytes: 0,
        energy_nj: 0,
        power_w: 0.0,
        user: "user".to_string(),
    };

    // Process B: 70% CPU + 70% mem + 70% power
    // Score = 0.5*0.70 + 0.3*0.70 + 0.2*0.70 = 0.70
    let multi_dim = ProcessInfo {
        pid: 11,
        name: "broad".to_string(),
        cpu_pct: 0.70,
        mem_bytes: 716_800_000, // 70% of ~1 GB
        energy_nj: 0,
        power_w: 14.0, // 70% of 20 W max
        user: "user".to_string(),
    };

    let max_cpu = 1.0_f32;
    let max_mem: u64 = 1_024_000_000;
    let max_power = 20.0_f32;

    let score_spike = weighted_score(&single_spike, max_cpu, max_mem, max_power);
    let score_multi = weighted_score(&multi_dim, max_cpu, max_mem, max_power);

    assert!(
        score_multi > score_spike,
        "70% across 3 dimensions (score {score_multi:.4}) should outrank \
         85% CPU-only process (score {score_spike:.4})"
    );
}

#[test]
/// weighted_score returns exactly 0.0 when all process metrics are zero
fn weighted_score_all_zeros_returns_zero() {
    use mtop::platform::process::weighted_score;
    use mtop::metrics::types::ProcessInfo;

    let idle = ProcessInfo {
        pid: 99,
        name: "idle".to_string(),
        cpu_pct: 0.0,
        mem_bytes: 0,
        energy_nj: 0,
        power_w: 0.0,
        user: "user".to_string(),
    };

    let score = weighted_score(&idle, 1.0, 1_000_000, 20.0);

    assert_eq!(score, 0.0, "all-zero process should have score 0.0");
    assert!(!score.is_nan(), "score must not be NaN");
}

#[test]
/// weighted_score does not divide by zero when max_power is 0.0
fn weighted_score_max_power_zero_is_finite() {
    use mtop::platform::process::weighted_score;
    use mtop::metrics::types::ProcessInfo;

    let proc = ProcessInfo {
        pid: 42,
        name: "test".to_string(),
        cpu_pct: 0.50,
        mem_bytes: 512_000_000,
        energy_nj: 0,
        power_w: 5.0,
        user: "user".to_string(),
    };

    // max_power = 0.0 must not cause division by zero or NaN/infinity
    let score = weighted_score(&proc, 1.0, 1_024_000_000, 0.0);

    assert!(score.is_finite(), "score must be finite even when max_power is 0.0; got {score}");
    assert!(!score.is_nan(), "score must not be NaN when max_power is 0.0");
}
