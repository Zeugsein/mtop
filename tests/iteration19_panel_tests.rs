/// Iteration 19: Panel rendering tests using ratatui TestBackend.
/// Exercises TUI panel code paths for memory, power, GPU panels and helpers.

use mtop::metrics::types::{MemoryMetrics, MetricsSnapshot, PowerMetrics};
use mtop::tui::helpers::{truncate_by_display_width, pad_to_display_width};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn make_snapshot_with_memory(ram_total: u64, ram_used: u64) -> MetricsSnapshot {
    let mut s = MetricsSnapshot::default();
    s.memory = MemoryMetrics {
        ram_total,
        ram_used,
        ..Default::default()
    };
    s
}

fn make_snapshot_with_power(gpu_w: f32) -> MetricsSnapshot {
    let mut s = MetricsSnapshot::default();
    s.power = PowerMetrics {
        cpu_w: 5.0,
        gpu_w,
        ane_w: 0.1,
        dram_w: 0.5,
        package_w: 6.0,
        system_w: 7.0,
        available: true,
    };
    s
}

// ---------------------------------------------------------------------------
// 1. panel_render_zero_metrics_80x24
// ---------------------------------------------------------------------------

#[test]
fn panel_render_zero_metrics_80x24() {
    // Default MetricsSnapshot (all zeros) — must not panic.
    let text = mtop::tui::render_dashboard_to_string(80, 24, MetricsSnapshot::default(), false);
    assert!(!text.is_empty());
}

// ---------------------------------------------------------------------------
// 2. panel_render_zero_metrics_40x10
// ---------------------------------------------------------------------------

#[test]
fn panel_render_zero_metrics_40x10() {
    // Minimum size triggers header truncation path.
    let text = mtop::tui::render_dashboard_to_string(40, 10, MetricsSnapshot::default(), false);
    assert!(!text.is_empty());
}

// ---------------------------------------------------------------------------
// 3. memory_type_b_labels
// ---------------------------------------------------------------------------

#[test]
fn memory_type_b_labels() {
    // 120x40 with show_detail=true, 16GB total / 12GB used.
    let gb: u64 = 1024 * 1024 * 1024;
    let snapshot = make_snapshot_with_memory(16 * gb, 12 * gb);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, true);
    assert!(
        text.contains(" used "),
        "Expected ' used ' title in detail layout; buffer:\n{text}"
    );
    assert!(
        text.contains(" avail "),
        "Expected ' avail ' title in detail layout; buffer:\n{text}"
    );
}

// ---------------------------------------------------------------------------
// 4. memory_label_mb_scale
// ---------------------------------------------------------------------------

#[test]
fn memory_label_mb_scale() {
    // ram_used < 1GB (500 MB) — label should display "MB".
    let gb: u64 = 1024 * 1024 * 1024;
    let mb500: u64 = 524_288_000; // 500 MB in bytes
    let snapshot = make_snapshot_with_memory(16 * gb, mb500);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, true);
    assert!(
        text.contains("MB"),
        "Expected 'MB' scale label when ram_used < 1GB; buffer:\n{text}"
    );
}

// ---------------------------------------------------------------------------
// 5. power_idle_label
// ---------------------------------------------------------------------------

#[test]
fn power_idle_label() {
    // gpu_w=0.0 (below 0.5W threshold) — power panel should show "(idle)".
    let snapshot = make_snapshot_with_power(0.0);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    assert!(
        text.contains("idle"),
        "Expected 'idle' label when gpu_w=0.0; buffer:\n{text}"
    );
}

// ---------------------------------------------------------------------------
// 6. gpu_idle_overlay
// ---------------------------------------------------------------------------

#[test]
fn gpu_idle_overlay() {
    // gpu_w=0.0 — GPU panel renders "idle" overlay on the graph.
    let snapshot = make_snapshot_with_power(0.0);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    assert!(
        text.contains("idle"),
        "Expected 'idle' overlay in GPU panel when gpu_w=0.0; buffer:\n{text}"
    );
}

// ---------------------------------------------------------------------------
// 7. dashboard_header_narrow
// ---------------------------------------------------------------------------

#[test]
fn dashboard_header_narrow() {
    // At 80x24 (minimum valid size) the full header
    // "YYYY-MM-DD HH:MM:SS — mtop — <chip info>" doesn't fit in the header
    // area (half width), so the truncation path runs and shows "mtop" without
    // the full timestamp.
    let text = mtop::tui::render_dashboard_to_string(80, 24, MetricsSnapshot::default(), false);
    assert!(
        text.contains("mtop"),
        "Expected 'mtop' in narrow header; buffer:\n{text}"
    );
    // A full timestamp is 19 chars ("YYYY-MM-DD HH:MM:SS"). At 80 wide the
    // header area is ~40 cols, so the full header is truncated — we verify
    // "mtop" appears (the short form always renders it).
}

// ---------------------------------------------------------------------------
// 8. helpers_truncate_cjk
// ---------------------------------------------------------------------------

#[test]
fn helpers_truncate_cjk() {
    // "测试Process" — 测(2) + 试(2) + Process(7) = 11 display cols.
    // Truncated to width 8 should not exceed 8 display columns.
    let input = "测试Process";
    let result = truncate_by_display_width(input, 8);
    let result_width: usize = result.chars()
        .map(|c| unicode_width::UnicodeWidthChar::width(c).unwrap_or(0))
        .sum();
    assert!(
        result_width <= 8,
        "Truncated CJK string display width {result_width} exceeds 8; result: '{result}'"
    );
    // Result should be shorter than the original
    assert!(
        result.len() < input.len(),
        "Truncated result '{result}' should be shorter than original '{input}'"
    );
}

// ---------------------------------------------------------------------------
// 9. helpers_pad_cjk
// ---------------------------------------------------------------------------

#[test]
fn helpers_pad_cjk() {
    // "测试" has display width 4; pad to 8 — result should have display width 8.
    let input = "测试";
    let result = pad_to_display_width(input, 8);
    let result_width: usize = result.chars()
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

// ---------------------------------------------------------------------------
// 10. helpers_truncate_exact
// ---------------------------------------------------------------------------

#[test]
fn helpers_truncate_exact() {
    // "hello" is exactly 5 chars / 5 display cols — truncating to 5 returns unchanged.
    let result = truncate_by_display_width("hello", 5);
    assert_eq!(result, "hello", "Exact-width string should be returned unchanged");
}

// ---------------------------------------------------------------------------
// 11. helpers_truncate_narrow
// ---------------------------------------------------------------------------

#[test]
fn helpers_truncate_narrow() {
    // "hello_world" (11 chars) truncated to width 3 — result must fit in 3 cols.
    let result = truncate_by_display_width("hello_world", 3);
    let result_width: usize = result.chars()
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
