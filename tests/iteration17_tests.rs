/// Integration tests for mtop iteration 17: Memory metrics, pressure level,
/// swap I/O, idle thresholds, and network/process label checks.

use mtop::metrics::{MetricsHistory, MetricsSnapshot, MemoryMetrics};

// ---------------------------------------------------------------------------
// Memory available fraction
// ---------------------------------------------------------------------------

#[test]
fn mem_available_fraction_correct() {
    let gb: u64 = 1024 * 1024 * 1024;
    let mut history = MetricsHistory::new();
    let mut snapshot = MetricsSnapshot::default();
    snapshot.memory.ram_total = 16 * gb;
    snapshot.memory.ram_used = 12 * gb;
    history.push(&snapshot);

    let latest = history.mem_available.last().copied().unwrap_or(0.0);
    let expected = 4.0 / 16.0; // 0.25
    assert!((latest - expected).abs() < 1e-9, "expected ~0.25, got {latest}");
}

#[test]
fn mem_available_zero_total_safe() {
    let mut history = MetricsHistory::new();
    let mut snapshot = MetricsSnapshot::default();
    snapshot.memory.ram_total = 0;
    snapshot.memory.ram_used = 0;
    history.push(&snapshot);

    // When total == 0, nothing is pushed — last() returns None
    let latest = history.mem_available.last().copied().unwrap_or(0.0);
    assert_eq!(latest, 0.0, "expected 0.0 when total is zero, got {latest}");
}

// ---------------------------------------------------------------------------
// MemoryMetrics fields
// ---------------------------------------------------------------------------

#[test]
fn pressure_level_field_exists() {
    let m = MemoryMetrics {
        ram_total: 0,
        ram_used: 0,
        swap_total: 0,
        swap_used: 0,
        wired: 0,
        app: 0,
        compressed: 0,
        swap_in_bytes_sec: 0.0,
        swap_out_bytes_sec: 0.0,
        pressure_level: 2,
    };
    assert_eq!(m.pressure_level, 2);
}

#[test]
fn swap_io_fields_default_zero() {
    let m = MemoryMetrics::default();
    assert_eq!(m.swap_in_bytes_sec, 0.0);
    assert_eq!(m.swap_out_bytes_sec, 0.0);
}

// ---------------------------------------------------------------------------
// Idle thresholds (verify constants in source files)
// ---------------------------------------------------------------------------

#[test]
fn idle_threshold_gpu() {
    // power.rs uses `s.power.gpu_w < 0.5` as the GPU idle threshold
    let power_src = include_str!("../src/tui/panels/power.rs");
    assert!(
        power_src.contains("< 0.5"),
        "expected GPU idle threshold `< 0.5` in power.rs"
    );
}

#[test]
fn idle_threshold_network() {
    // network.rs uses `< 1024.0` as the per-history-value idle threshold
    let net_src = include_str!("../src/tui/panels/network.rs");
    assert!(
        net_src.contains("< 1024.0"),
        "expected network idle threshold `< 1024.0` in network.rs"
    );
}

// ---------------------------------------------------------------------------
// Process dot near-zero thresholds
// ---------------------------------------------------------------------------

#[test]
fn process_dot_near_zero_threshold() {
    let proc_src = include_str!("../src/tui/panels/process.rs");
    // cpu < 0.1
    assert!(proc_src.contains("cpu_pct < 0.1"), "expected `cpu_pct < 0.1` in process.rs");
    // mem < 1MB (1_048_576 bytes)
    assert!(proc_src.contains("1_048_576"), "expected `1_048_576` (1MB) threshold in process.rs");
    // power < 0.1W
    assert!(proc_src.contains("power_w < 0.1"), "expected `power_w < 0.1` in process.rs");
}

// ---------------------------------------------------------------------------
// Network label checks
// ---------------------------------------------------------------------------

#[test]
fn network_label_no_cur_prefix() {
    let net_src = include_str!("../src/tui/panels/network.rs");
    assert!(
        !net_src.contains("cur:"),
        "network.rs should not contain old `cur:` label prefix"
    );
}

#[test]
fn network_label_uses_total() {
    let net_src = include_str!("../src/tui/panels/network.rs");
    assert!(
        net_src.contains("total"),
        "network.rs should use `total` label (not `tot:`)"
    );
    assert!(
        !net_src.contains("tot:"),
        "network.rs should not contain old `tot:` label"
    );
}

// ---------------------------------------------------------------------------
// Memory panel swap guard
// ---------------------------------------------------------------------------

#[test]
fn memory_panel_swap_guard() {
    let mem_src = include_str!("../src/tui/panels/memory.rs");
    assert!(
        mem_src.contains("swap_total == 0"),
        "memory panel should guard on `swap_total == 0` before showing swap info"
    );
}
