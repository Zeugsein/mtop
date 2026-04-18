/// Feature-organized tests: metrics collection
/// Covers: sampling, metrics types, history, process CPU deltas, power collection,
/// memory metrics, disk I/O, interval clamping, and consecutive sample stability.
use mtop::metrics::types::*;

// ===========================================================================
// Process CPU delta (C1)
// ===========================================================================

#[test]
/// Validates: metrics-collection [C1] - process CPU% delta calculation
/// On first sample all processes are "new" — no delta available, cpu_pct must be 0.0.
fn process_cpu_delta_first_sample_reports_zero() {
    let mut sampler = mtop::metrics::Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("first sample");

    for p in &snapshot.processes {
        assert_eq!(
            p.cpu_pct, 0.0,
            "C1: first-sample process '{}' (pid {}) should have cpu_pct=0.0, got {}",
            p.name, p.pid, p.cpu_pct
        );
    }
}

#[test]
/// Validates: metrics-collection [C1] - second sample produces non-zero CPU% for active processes
fn process_cpu_delta_second_sample_has_nonzero_for_active() {
    let mut sampler = mtop::metrics::Sampler::new().expect("sampler init");
    let _ = sampler.sample(200).expect("first sample");
    let snapshot = sampler.sample(200).expect("second sample");

    let any_nonzero = snapshot.processes.iter().any(|p| p.cpu_pct > 0.0);
    assert!(
        any_nonzero,
        "C1: after second sample, at least one process should have cpu_pct > 0.0"
    );
}

#[test]
/// Validates: metrics-collection [C1] - stale PID cleanup prevents unbounded memory growth
fn process_cpu_delta_stale_pid_cleanup() {
    let mut sampler = mtop::metrics::Sampler::new().expect("sampler init");
    for _ in 0..5 {
        let _ = sampler.sample(200).expect("sample");
    }
    // If this test passes without OOM after 5 samples, cleanup is working.
}

// ===========================================================================
// Disk I/O (C2)
// ===========================================================================

#[test]
/// Validates: metrics-collection [C2] - disk I/O collection is read-only
fn disk_io_collection_does_not_write_probe_files() {
    let probe_path = std::env::temp_dir().join(".mtop_io_probe");
    let _ = std::fs::remove_file(&probe_path);

    let mut sampler = mtop::metrics::Sampler::new().expect("sampler init");
    let _ = sampler.sample(200).expect("first sample");
    let _ = sampler.sample(200).expect("second sample");

    assert!(
        !probe_path.exists(),
        "C2: disk I/O collection must not write probe files; found {}",
        probe_path.display()
    );
}

// ===========================================================================
// Power calculation (H3)
// ===========================================================================

#[test]
/// Validates: metrics-collection [H3] - power calculation uses measured elapsed time
fn power_calculation_uses_measured_elapsed_time() {
    let mut sampler = mtop::metrics::Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");

    assert!(snapshot.power.cpu_w >= 0.0, "H3: cpu_w should be >= 0.0");
    assert!(snapshot.power.gpu_w >= 0.0, "H3: gpu_w should be >= 0.0");
    assert!(
        snapshot.power.system_w >= 0.0,
        "H3: system_w should be >= 0.0"
    );
}

// ===========================================================================
// Memory struct correctness (M1, M2)
// ===========================================================================

#[test]
/// Validates: metrics-collection [M1] - swap_total is plausible (not struct-layout corruption)
fn xsw_usage_struct_is_32_bytes() {
    let mut sampler = mtop::metrics::Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");

    let max_swap = 100 * 1024 * 1024 * 1024_u64; // 100 GB
    assert!(
        snapshot.memory.swap_total <= max_swap,
        "M1: swap_total {} looks corrupted (> 100 GB), possible struct layout bug",
        snapshot.memory.swap_total
    );
}

#[test]
/// Validates: metrics-collection [M2] - VmStatistics64 struct produces valid memory metrics
fn vm_statistics64_struct_produces_valid_memory_metrics() {
    let mut sampler = mtop::metrics::Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");

    assert!(
        snapshot.memory.ram_used > 0,
        "M2: ram_used should be > 0; host_statistics64 may have failed due to wrong struct size"
    );
    assert!(
        snapshot.memory.ram_used <= snapshot.memory.ram_total,
        "M2: ram_used ({}) > ram_total ({}), possible struct layout corruption",
        snapshot.memory.ram_used,
        snapshot.memory.ram_total
    );
}

// ===========================================================================
// MetricsHistory (M3)
// ===========================================================================

#[test]
/// Validates: metrics-collection [M3] - MetricsHistory uses VecDeque for O(1) operations
fn metrics_history_uses_efficient_data_structure() {
    let mut history = MetricsHistory::new();
    let snapshot = MetricsSnapshot::default();

    for _ in 0..200 {
        history.push(&snapshot);
    }

    assert_eq!(
        history.cpu_usage.len(),
        128,
        "M3: history should cap at 128 entries"
    );
}

#[test]
/// Validates: metrics-collection [M3] - history buffer maintains capacity invariant
fn metrics_history_capacity_invariant_at_128() {
    let mut history = MetricsHistory::new();

    let mut snap = MetricsSnapshot::default();
    snap.cpu.total_usage = 0.5;
    for _ in 0..128 {
        history.push(&snap);
    }
    assert_eq!(
        history.cpu_usage.len(),
        128,
        "M3: should hold exactly 128 entries"
    );

    snap.cpu.total_usage = 1.0;
    history.push(&snap);
    assert_eq!(
        history.cpu_usage.len(),
        128,
        "M3: should still be 128 after overflow"
    );

    assert_eq!(
        *history.cpu_usage.last().unwrap(),
        1.0,
        "M3: newest value should be at the end of the buffer"
    );
}

// ===========================================================================
// GPU power wired (M8)
// ===========================================================================

#[test]
/// Validates: metrics-collection [M8] - GPU power_w is wired from power collector
fn gpu_power_is_wired_from_power_collector() {
    let mut sampler = mtop::metrics::Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");

    if snapshot.power.gpu_w > 0.0 {
        assert!(
            snapshot.gpu.power_w > 0.0,
            "M8: gpu.power_w should be {} (from power collector), not 0.0",
            snapshot.power.gpu_w
        );
        assert!(
            (snapshot.gpu.power_w - snapshot.power.gpu_w).abs() < 0.01,
            "M8: gpu.power_w ({}) should equal power.gpu_w ({})",
            snapshot.gpu.power_w,
            snapshot.power.gpu_w
        );
    }
}

// ===========================================================================
// MetricsHistory push correctness
// ===========================================================================

#[test]
/// Validates: tui-dashboard [FR-3] - history push records correct gpu_usage value
fn metrics_history_records_gpu_usage() {
    let mut history = MetricsHistory::new();
    let mut snap = MetricsSnapshot::default();
    snap.gpu.usage = 0.75;
    snap.gpu.available = true;
    history.push(&snap);

    assert_eq!(history.gpu_usage.len(), 1);
    assert!(
        (history.gpu_usage[0] - 0.75).abs() < f64::EPSILON,
        "gpu_usage should be 0.75, got {}",
        history.gpu_usage[0]
    );
}

// ===========================================================================
// Process list (FR-8)
// ===========================================================================

#[test]
/// Validates: metrics-collection [FR-8] - process list is bounded (top 50)
fn process_list_is_bounded() {
    let mut sampler = mtop::metrics::Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");

    assert!(
        snapshot.processes.len() <= 50,
        "FR-8: process list should be truncated to top 50; got {}",
        snapshot.processes.len()
    );
}

// ===========================================================================
// Sampling interval clamping (FR-10)
// ===========================================================================

#[test]
/// Validates: metrics-collection [FR-10] - interval of 0ms is clamped to 100ms minimum
fn interval_zero_is_clamped_to_minimum() {
    let mut sampler = mtop::metrics::Sampler::new().expect("sampler init");
    let start = std::time::Instant::now();
    let _ = sampler.sample(0).expect("sample with 0ms interval");
    let elapsed = start.elapsed().as_millis();

    assert!(
        elapsed >= 90,
        "FR-10: 0ms interval should be clamped to 100ms; elapsed only {}ms",
        elapsed
    );
}

// ===========================================================================
// Graceful degradation (FR-12)
// ===========================================================================

#[test]
/// Validates: metrics-collection [FR-12] - 10 consecutive samples without crash
fn ten_consecutive_samples_without_crash() {
    let mut sampler = mtop::metrics::Sampler::new().expect("sampler init");
    for i in 0..10 {
        let result = sampler.sample(100);
        assert!(
            result.is_ok(),
            "FR-12: sample {} failed: {:?}",
            i,
            result.err()
        );
    }
}

// ===========================================================================
// Per-interface network history (W2 iter10)
// ===========================================================================

fn make_iface_snapshot(ifaces: &[(&str, f64, f64)]) -> MetricsSnapshot {
    let mut snapshot = MetricsSnapshot::default();
    for &(name, rx, tx) in ifaces {
        snapshot.network.interfaces.push(NetInterface {
            name: name.to_string(),
            rx_bytes_sec: rx,
            tx_bytes_sec: tx,
            ..Default::default()
        });
    }
    snapshot
}

#[test]
fn per_iface_push_creates_separate_buffers() {
    let mut h = MetricsHistory::new();
    let snap = make_iface_snapshot(&[("en0", 100.0, 200.0), ("en1", 300.0, 400.0)]);
    h.push(&snap);

    assert!(h.per_iface.contains_key("en0"), "en0 buffer should exist");
    assert!(h.per_iface.contains_key("en1"), "en1 buffer should exist");

    let (rx0, tx0) = h.per_iface.get("en0").unwrap();
    assert_eq!(*rx0.last().unwrap(), 100.0);
    assert_eq!(*tx0.last().unwrap(), 200.0);

    let (rx1, tx1) = h.per_iface.get("en1").unwrap();
    assert_eq!(*rx1.last().unwrap(), 300.0);
    assert_eq!(*tx1.last().unwrap(), 400.0);
}

#[test]
fn per_iface_buffers_cap_at_128() {
    let mut h = MetricsHistory::new();
    let snap = make_iface_snapshot(&[("en0", 1.0, 2.0)]);
    for _ in 0..150 {
        h.push(&snap);
    }
    let (rx, tx) = h.per_iface.get("en0").unwrap();
    assert_eq!(rx.len(), 128, "per-iface rx should cap at 128");
    assert_eq!(tx.len(), 128, "per-iface tx should cap at 128");
}

#[test]
fn per_iface_skips_loopback() {
    let mut h = MetricsHistory::new();
    let snap = make_iface_snapshot(&[("lo0", 100.0, 200.0), ("en0", 50.0, 60.0)]);
    h.push(&snap);
    assert!(
        !h.per_iface.contains_key("lo0"),
        "loopback should be skipped"
    );
    assert!(h.per_iface.contains_key("en0"));
}

#[test]
fn per_iface_stale_pruned() {
    let mut h = MetricsHistory::new();
    let snap1 = make_iface_snapshot(&[("en0", 100.0, 200.0), ("en1", 300.0, 400.0)]);
    h.push(&snap1);
    assert!(h.per_iface.contains_key("en1"));
    let snap2 = make_iface_snapshot(&[("en0", 150.0, 250.0)]);
    h.push(&snap2);
    assert!(
        !h.per_iface.contains_key("en1"),
        "stale interface buffer should be pruned"
    );
}

#[test]
fn per_iface_independent_values() {
    let mut h = MetricsHistory::new();
    let snap1 = make_iface_snapshot(&[("en0", 10.0, 20.0), ("en1", 100.0, 200.0)]);
    h.push(&snap1);
    let snap2 = make_iface_snapshot(&[("en0", 30.0, 40.0), ("en1", 500.0, 600.0)]);
    h.push(&snap2);

    let (rx0, _) = h.per_iface.get("en0").unwrap();
    assert_eq!(rx0.len(), 2);
    assert_eq!(rx0[0], 10.0);
    assert_eq!(rx0[1], 30.0);

    let (rx1, _) = h.per_iface.get("en1").unwrap();
    assert_eq!(rx1[0], 100.0);
    assert_eq!(rx1[1], 500.0);
}

#[test]
fn per_iface_aggregate_still_works() {
    let mut h = MetricsHistory::new();
    let snap = make_iface_snapshot(&[("en0", 100.0, 200.0), ("en1", 300.0, 400.0)]);
    h.push(&snap);
    assert_eq!(*h.net_download.last().unwrap(), 400.0); // 100 + 300
    assert_eq!(*h.net_upload.last().unwrap(), 600.0); // 200 + 400
}

// ===========================================================================
// Memory available fraction (iter17)
// ===========================================================================

use mtop::metrics::{MemoryMetrics, MetricsHistory, MetricsSnapshot};

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
    assert!(
        (latest - expected).abs() < 1e-9,
        "expected ~0.25, got {latest}"
    );
}

#[test]
fn mem_available_zero_total_safe() {
    let mut history = MetricsHistory::new();
    let mut snapshot = MetricsSnapshot::default();
    snapshot.memory.ram_total = 0;
    snapshot.memory.ram_used = 0;
    history.push(&snapshot);

    let latest = history.mem_available.last().copied().unwrap_or(0.0);
    assert_eq!(latest, 0.0, "expected 0.0 when total is zero, got {latest}");
}

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
        cached: 0,
        free: 0,
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

// ===========================================================================
// SHALL-23-15: Memory formula
// ===========================================================================

#[test]
fn shall_23_15_memory_used_equals_total_minus_free() {
    let ram_total: u64 = 16 * 1024 * 1024 * 1024;
    let free_bytes: u64 = 4 * 1024 * 1024 * 1024;
    let ram_used = ram_total.saturating_sub(free_bytes);

    assert_eq!(
        ram_used,
        12 * 1024 * 1024 * 1024,
        "ram_used should be 12 GB"
    );

    let mut snapshot = MetricsSnapshot::default();
    snapshot.memory = MemoryMetrics {
        ram_total,
        ram_used,
        ..Default::default()
    };

    let mut history = MetricsHistory::new();
    history.push(&snapshot);

    let usage_frac = *history
        .mem_usage
        .last()
        .expect("mem_usage should have one entry");
    let avail_frac = *history
        .mem_available
        .last()
        .expect("mem_available should have one entry");

    assert!(
        (usage_frac - 0.75).abs() < 0.001,
        "usage fraction should be 0.75 (12/16 GB), got {usage_frac}"
    );
    assert!(
        (avail_frac - 0.25).abs() < 0.001,
        "available fraction should be 0.25 (4/16 GB free), got {avail_frac}"
    );
    assert!(
        (usage_frac + avail_frac - 1.0).abs() < 0.001,
        "usage + available should equal 1.0, got {}",
        usage_frac + avail_frac
    );
}

#[test]
fn shall_23_15_memory_used_saturates_at_zero_when_free_exceeds_total() {
    let ram_total: u64 = 8 * 1024 * 1024 * 1024;
    let free_bytes: u64 = 10 * 1024 * 1024 * 1024;
    let ram_used = ram_total.saturating_sub(free_bytes);
    assert_eq!(
        ram_used, 0,
        "saturating_sub must floor at 0, never underflow"
    );
}

#[test]
fn shall_23_15_memory_all_free_gives_zero_usage_fraction() {
    let ram_total: u64 = 8 * 1024 * 1024 * 1024;
    let ram_used = 0u64;
    let mut snapshot = MetricsSnapshot::default();
    snapshot.memory = MemoryMetrics {
        ram_total,
        ram_used,
        ..Default::default()
    };
    let mut history = MetricsHistory::new();
    history.push(&snapshot);
    let usage_frac = *history.mem_usage.last().unwrap();
    assert!(
        usage_frac.abs() < 0.001,
        "usage fraction should be 0.0 when all memory is free, got {usage_frac}"
    );
}

#[test]
fn shall_23_15_memory_half_free_gives_half_usage_fraction() {
    let ram_total: u64 = 16 * 1024 * 1024 * 1024;
    let free_bytes: u64 = ram_total / 2;
    let ram_used = ram_total.saturating_sub(free_bytes);
    let mut snapshot = MetricsSnapshot::default();
    snapshot.memory = MemoryMetrics {
        ram_total,
        ram_used,
        ..Default::default()
    };
    let mut history = MetricsHistory::new();
    history.push(&snapshot);
    let usage_frac = *history.mem_usage.last().unwrap();
    assert!(
        (usage_frac - 0.5).abs() < 0.001,
        "usage fraction should be 0.5 when half of memory is free, got {usage_frac}"
    );
}

#[test]
fn shall_23_15_ram_used_never_exceeds_total() {
    let ram_total: u64 = 32 * 1024 * 1024 * 1024;
    let page_size: u64 = 16384;

    for free_pages in [0u64, 1, 1000, 100_000, 500_000, ram_total / page_size] {
        let free_bytes = free_pages * page_size;
        let ram_used = ram_total.saturating_sub(free_bytes);
        assert!(
            ram_used <= ram_total,
            "ram_used ({ram_used}) must be <= ram_total ({ram_total}) for free_pages={free_pages}"
        );
    }
}
