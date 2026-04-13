/// Integration tests for mtop iteration 7 Workstream 1 (Polish).

// W1C: Compact rate format (updated iteration 29: /s suffix added)
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
