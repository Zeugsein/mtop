/// Feature-organized tests: helpers
/// Covers: format functions (bytes rate compact, baudrate), truncation, sorting,
/// temperature colors/thresholds, weighted process score.

// ===========================================================================
// Compact rate format (iter7)
// ===========================================================================

#[test]
fn compact_rate_megabytes() {
    let result = mtop::tui::format_bytes_rate_compact(2_500_000.0);
    assert_eq!(result, "2.4M/s");
}

#[test]
fn compact_rate_kilobytes() {
    let result = mtop::tui::format_bytes_rate_compact(350_000.0);
    assert!(result.ends_with("K/s"), "expected K/s suffix, got {result}");
}

#[test]
fn compact_rate_gigabytes() {
    let result = mtop::tui::format_bytes_rate_compact(1_300_000_000.0);
    assert_eq!(result, "1.2G/s");
}

#[test]
fn compact_rate_bytes() {
    assert_eq!(mtop::tui::format_bytes_rate_compact(42.0), "42B/s");
}

#[test]
fn compact_rate_zero() {
    assert_eq!(mtop::tui::format_bytes_rate_compact(0.0), "0B/s");
}

// ===========================================================================
// format_bytes_rate_compact via helpers (iter9)
// ===========================================================================

#[test]
fn format_bytes_rate_compact_bytes() {
    let result = mtop::tui::helpers::format_bytes_rate_compact(500.0);
    assert_eq!(result, "500B/s");
}

#[test]
fn format_bytes_rate_compact_kilobytes() {
    let result = mtop::tui::helpers::format_bytes_rate_compact(2048.0);
    assert_eq!(result, "2.0K/s");
}

#[test]
fn format_bytes_rate_compact_megabytes() {
    let result = mtop::tui::helpers::format_bytes_rate_compact(5_242_880.0);
    assert_eq!(result, "5.0M/s");
}

#[test]
fn format_bytes_rate_compact_gigabytes() {
    let result = mtop::tui::helpers::format_bytes_rate_compact(2_147_483_648.0);
    assert_eq!(result, "2.0G/s");
}

// ===========================================================================
// Ellipsis truncation (iter8)
// ===========================================================================

#[test]
fn truncate_long_name() {
    let result = mtop::tui::helpers::truncate_with_ellipsis("Google Chrome Helper", 15);
    assert_eq!(result.chars().count(), 15);
    assert!(result.ends_with('\u{2026}'));
}

#[test]
fn truncate_short_name_unchanged() {
    let result = mtop::tui::helpers::truncate_with_ellipsis("vim", 15);
    assert_eq!(result, "vim");
}

#[test]
fn truncate_exact_fit() {
    let result = mtop::tui::helpers::truncate_with_ellipsis("exactly15chars!", 15);
    assert_eq!(result, "exactly15chars!");
}

#[test]
fn truncate_width_zero() {
    let result = mtop::tui::helpers::truncate_with_ellipsis("anything", 0);
    assert_eq!(result, "");
}

#[test]
fn truncate_width_one() {
    let result = mtop::tui::helpers::truncate_with_ellipsis("anything", 1);
    assert_eq!(result, "\u{2026}");
}

// ===========================================================================
// Display-width-aware truncation and padding (iter19)
// ===========================================================================

use mtop::tui::helpers::{pad_to_display_width, truncate_by_display_width};

#[test]
fn helpers_truncate_cjk() {
    let input = "测试Process";
    let result = truncate_by_display_width(input, 8);
    let result_width: usize = result
        .chars()
        .map(|c| unicode_width::UnicodeWidthChar::width(c).unwrap_or(0))
        .sum();
    assert!(
        result_width <= 8,
        "Truncated CJK string display width {result_width} exceeds 8; result: '{result}'"
    );
    assert!(
        result.len() < input.len(),
        "Truncated result '{result}' should be shorter than original '{input}'"
    );
}

#[test]
fn helpers_pad_cjk() {
    let input = "测试";
    let result = pad_to_display_width(input, 8);
    let result_width: usize = result
        .chars()
        .map(|c| unicode_width::UnicodeWidthChar::width(c).unwrap_or(0))
        .sum();
    assert_eq!(
        result_width, 8,
        "Padded CJK string should have display width 8; got {result_width}; result: '{result}'"
    );
    assert!(
        result.starts_with("测试"),
        "Padded result should start with original; got: '{result}'"
    );
}

#[test]
fn helpers_truncate_exact() {
    let result = truncate_by_display_width("hello", 5);
    assert_eq!(
        result, "hello",
        "Exact-width string should be returned unchanged"
    );
}

#[test]
fn helpers_truncate_narrow() {
    let result = truncate_by_display_width("hello_world", 3);
    let result_width: usize = result
        .chars()
        .map(|c| unicode_width::UnicodeWidthChar::width(c).unwrap_or(0))
        .sum();
    assert!(
        result_width <= 3,
        "Truncated result display width {result_width} should be <= 3; result: '{result}'"
    );
    assert!(
        !result.is_empty(),
        "Truncated result should not be empty for width 3"
    );
}

// ===========================================================================
// Weighted process sort score (iter6)
// ===========================================================================

#[test]
/// weighted_score: 90% CPU-only process ranks above 30% CPU + 30% memory process
fn weighted_score_high_cpu_ranks_above_split_load() {
    use mtop::metrics::types::ProcessInfo;
    use mtop::platform::process::weighted_score;

    let high_cpu = ProcessInfo {
        pid: 1,
        name: "stress".to_string(),
        cpu_pct: 0.90,
        mem_bytes: 0,
        energy_nj: 0,
        power_w: 0.0,
        user: "user".to_string(),
        ..Default::default()
    };

    let split_load = ProcessInfo {
        pid: 2,
        name: "mixed".to_string(),
        cpu_pct: 0.30,
        mem_bytes: 307_200_000,
        energy_nj: 0,
        power_w: 0.0,
        user: "user".to_string(),
        ..Default::default()
    };

    let max_cpu = 1.0_f32;
    let max_mem: u64 = 1_024_000_000;
    let max_power = 0.0_f32;

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
fn weighted_score_multi_dimension_beats_sub_threshold_spike() {
    use mtop::metrics::types::ProcessInfo;
    use mtop::platform::process::weighted_score;

    let single_spike = ProcessInfo {
        pid: 10,
        name: "spike".to_string(),
        cpu_pct: 0.85,
        mem_bytes: 0,
        energy_nj: 0,
        power_w: 0.0,
        user: "user".to_string(),
        ..Default::default()
    };

    let multi_dim = ProcessInfo {
        pid: 11,
        name: "broad".to_string(),
        cpu_pct: 0.70,
        mem_bytes: 716_800_000,
        energy_nj: 0,
        power_w: 14.0,
        user: "user".to_string(),
        ..Default::default()
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
    use mtop::metrics::types::ProcessInfo;
    use mtop::platform::process::weighted_score;

    let idle = ProcessInfo {
        pid: 99,
        name: "idle".to_string(),
        cpu_pct: 0.0,
        mem_bytes: 0,
        energy_nj: 0,
        power_w: 0.0,
        user: "user".to_string(),
        ..Default::default()
    };

    let score = weighted_score(&idle, 1.0, 1_000_000, 20.0);

    assert_eq!(score, 0.0, "all-zero process should have score 0.0");
    assert!(!score.is_nan(), "score must not be NaN");
}

#[test]
/// weighted_score does not divide by zero when max_power is 0.0
fn weighted_score_max_power_zero_is_finite() {
    use mtop::metrics::types::ProcessInfo;
    use mtop::platform::process::weighted_score;

    let proc = ProcessInfo {
        pid: 42,
        name: "test".to_string(),
        cpu_pct: 0.50,
        mem_bytes: 512_000_000,
        energy_nj: 0,
        power_w: 5.0,
        user: "user".to_string(),
        ..Default::default()
    };

    let score = weighted_score(&proc, 1.0, 1_024_000_000, 0.0);

    assert!(
        score.is_finite(),
        "score must be finite even when max_power is 0.0; got {score}"
    );
    assert!(
        !score.is_nan(),
        "score must not be NaN when max_power is 0.0"
    );
}
