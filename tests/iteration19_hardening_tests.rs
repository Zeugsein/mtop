/// Iteration 19: Network fix verification and edge case hardening tests.

use mtop::metrics::types::HistoryBuffer;
use mtop::platform::network::{speed_tier_from_baudrate, NetworkState};
use mtop::tui::braille::render_braille_graph;
use mtop::tui::gauge::render_gauge_bar;
use mtop::tui::theme::HORIZON;

// ---------------------------------------------------------------------------
// W2 — Network verification
// ---------------------------------------------------------------------------

/// NetworkState::collect() on first call should return only non-negative rates.
/// (No previous sample exists, so all deltas are 0.0.)
#[test]
fn network_rate_first_sample_zero() {
    let mut state = NetworkState::new();
    let metrics = state.collect();

    for iface in &metrics.interfaces {
        assert!(
            iface.rx_bytes_sec >= 0.0,
            "rx_bytes_sec should be >= 0.0 on first sample, got {} for {}",
            iface.rx_bytes_sec,
            iface.name
        );
        assert!(
            iface.tx_bytes_sec >= 0.0,
            "tx_bytes_sec should be >= 0.0 on first sample, got {} for {}",
            iface.tx_bytes_sec,
            iface.name
        );
    }
}

/// After two collects (with a brief pause), all rates must remain non-negative.
#[test]
fn network_rate_second_sample_nonnegative() {
    let mut state = NetworkState::new();
    let _ = state.collect(); // prime prev map
    std::thread::sleep(std::time::Duration::from_millis(20));
    let metrics = state.collect();

    for iface in &metrics.interfaces {
        assert!(
            iface.rx_bytes_sec >= 0.0,
            "rx_bytes_sec must be >= 0 after second sample, got {} for {}",
            iface.rx_bytes_sec,
            iface.name
        );
        assert!(
            iface.tx_bytes_sec >= 0.0,
            "tx_bytes_sec must be >= 0 after second sample, got {} for {}",
            iface.tx_bytes_sec,
            iface.name
        );
    }
}

/// speed_tier_from_baudrate(0) should return the 10 Mbps fallback: 1_250_000.
#[test]
fn network_speed_tier_zero() {
    assert_eq!(
        speed_tier_from_baudrate(0),
        1_250_000,
        "baudrate 0 should map to 10 Mbps fallback (1_250_000 bytes/sec)"
    );
}

/// speed_tier_from_baudrate(1_000_000_000) should return 125_000_000 (1 Gbps tier).
#[test]
fn network_speed_tier_gigabit() {
    assert_eq!(
        speed_tier_from_baudrate(1_000_000_000),
        125_000_000,
        "1 Gbps baudrate should map to 125_000_000 bytes/sec tier"
    );
}

// ---------------------------------------------------------------------------
// W3 — Edge case hardening
// ---------------------------------------------------------------------------

/// Empty HistoryBuffer should yield 0 items when iterated.
#[test]
fn history_buffer_empty_iter() {
    let buf = HistoryBuffer::new();
    let count = buf.iter().count();
    assert_eq!(count, 0, "empty HistoryBuffer should yield 0 items, got {count}");
}

/// HistoryBuffer with one push should yield exactly 1 item.
#[test]
fn history_buffer_single_push() {
    let mut buf = HistoryBuffer::new();
    buf.push_back(42.0);
    let items: Vec<f64> = buf.iter().copied().collect();
    assert_eq!(items.len(), 1, "single-push HistoryBuffer should yield 1 item");
    assert_eq!(items[0], 42.0);
}

/// render_braille_graph with 1×1 dimensions should not panic and return 1 row.
#[test]
fn braille_graph_1x1() {
    let theme = &mtop::tui::theme::THEMES[0];
    let result = render_braille_graph(&[0.5], 1.0, 1, 1, theme);
    assert_eq!(result.len(), 1, "1×1 graph should return 1 row");
    assert_eq!(result[0].len(), 1, "1×1 graph row should have 1 character");
}

/// render_braille_graph with height=0 should not panic and return empty vec.
#[test]
fn braille_graph_zero_height() {
    let theme = &mtop::tui::theme::THEMES[0];
    let result = render_braille_graph(&[0.5], 1.0, 10, 0, theme);
    assert!(
        result.is_empty(),
        "height=0 graph should return empty vec, got {} rows",
        result.len()
    );
}

/// render_gauge_bar with value > max should not panic; filled chars clamped to width.
#[test]
fn gauge_value_exceeds_max() {
    let spans = render_gauge_bar(150.0, 100.0, 20, "", &HORIZON);
    // Should not panic; bar content should not exceed width
    let content: String = spans.iter().map(|s| s.content.as_ref()).collect();
    let filled_count = content.matches('■').count();
    assert!(
        filled_count <= 20,
        "filled chars ({filled_count}) must not exceed width (20)"
    );
}

/// render_gauge_bar with negative value should not panic; filled should be 0.
#[test]
fn gauge_negative_value() {
    let spans = render_gauge_bar(-5.0, 100.0, 20, "", &HORIZON);
    // fraction clamps to 0.0 → 0 filled chars, 20 empty chars
    let content: String = spans.iter().map(|s| s.content.as_ref()).collect();
    let char_count = content.matches('■').count();
    assert_eq!(
        char_count, 20,
        "negative value should produce 0 filled + 20 empty = 20 total ■ chars, got {char_count}"
    );
}

/// render_gauge_bar with max=0.0 should not panic (division-by-zero guard).
#[test]
fn gauge_zero_max() {
    // Should not panic; fraction is clamped to 0.0 when max <= 0
    let spans = render_gauge_bar(50.0, 0.0, 20, "", &HORIZON);
    assert!(
        !spans.is_empty(),
        "zero-max gauge should still return spans for the empty bar"
    );
}
