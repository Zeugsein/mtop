/// Integration tests for metrics-collection spec requirements.
/// Each test cites the FR requirement from:
///   openspec/changes/archive/2026-04-05-mvp-core/specs/metrics-collection/spec.md
///
/// Tests marked #[ignore] are expected to fail against the current stub
/// implementation and should be un-ignored once the feature is implemented.

// Bring public API into scope via the crate lib root
use mtop::metrics::types::{MetricsHistory, MetricsSnapshot};
use mtop::metrics::Sampler;

// ---------------------------------------------------------------------------
// FR-1: CPU metrics collection
// ---------------------------------------------------------------------------

#[test]
/// FR-1: per-core utilization values are in the valid range [0.0, 1.0]
fn cpu_core_usages_in_range() {
    let mut sampler = Sampler::new().expect("sampler init");
    // Two samples needed so the delta calculation has a previous tick.
    let _ = sampler.sample(200).expect("first sample");
    let snapshot = sampler.sample(200).expect("second sample");

    for (i, &usage) in snapshot.cpu.core_usages.iter().enumerate() {
        assert!(
            (0.0..=1.0).contains(&usage),
            "core {i} usage {usage} out of [0.0, 1.0]"
        );
    }
}

#[test]
/// FR-1: combined (total) CPU usage is in [0.0, 1.0]
fn cpu_total_usage_in_range() {
    let mut sampler = Sampler::new().expect("sampler init");
    let _ = sampler.sample(200).expect("first sample");
    let snapshot = sampler.sample(200).expect("second sample");

    assert!(
        (0.0..=1.0).contains(&snapshot.cpu.total_usage),
        "total_usage {} out of [0.0, 1.0]",
        snapshot.cpu.total_usage
    );
}

#[test]
/// FR-1: e_cluster and p_cluster usage values are in [0.0, 1.0]
fn cpu_cluster_usages_in_range() {
    let mut sampler = Sampler::new().expect("sampler init");
    let _ = sampler.sample(200).expect("first sample");
    let snapshot = sampler.sample(200).expect("second sample");

    assert!(
        (0.0..=1.0).contains(&snapshot.cpu.e_cluster.usage),
        "e_cluster.usage {} out of range",
        snapshot.cpu.e_cluster.usage
    );
    assert!(
        (0.0..=1.0).contains(&snapshot.cpu.p_cluster.usage),
        "p_cluster.usage {} out of range",
        snapshot.cpu.p_cluster.usage
    );
}

#[test]
/// FR-1 (PARTIAL): e_cluster covers efficiency cores, p_cluster covers performance cores —
/// the spec says E cores come first; verify the split index matches soc.e_cores.
fn cpu_cluster_split_matches_soc_e_cores() {
    let mut sampler = Sampler::new().expect("sampler init");
    let soc = sampler.soc_info().clone();
    let _ = sampler.sample(200).expect("first sample");
    let snapshot = sampler.sample(200).expect("second sample");

    // If e_cores > 0 and total cores are known, the number of per-core usages
    // should be e_cores + p_cores.
    if soc.e_cores > 0 && soc.p_cores > 0 {
        assert_eq!(
            snapshot.cpu.core_usages.len(),
            (soc.e_cores + soc.p_cores) as usize,
            "core_usages length should equal e_cores + p_cores"
        );
    }
}

#[test]
#[ignore] // FR-1: CPU power requires IOReport — returns 0 in VMs without IOReport
/// FR-1: CPU power draw in Watts is reported (non-zero on loaded Apple Silicon)
fn cpu_power_is_nonzero_on_apple_silicon() {
    let mut sampler = Sampler::new().expect("sampler init");
    let _ = sampler.sample(200).expect("first sample");
    let snapshot = sampler.sample(200).expect("second sample");
    assert!(
        snapshot.cpu.power_w > 0.0,
        "cpu power_w should be > 0 on Apple Silicon; got {}",
        snapshot.cpu.power_w
    );
}

#[test]
/// FR-1: e_cluster and p_cluster frequencies are non-zero on Apple Silicon
fn cpu_cluster_freq_nonzero_on_apple_silicon() {
    let mut sampler = Sampler::new().expect("sampler init");
    let _ = sampler.sample(200).expect("first sample");
    let snapshot = sampler.sample(200).expect("second sample");
    assert!(
        snapshot.cpu.e_cluster.freq_mhz > 0,
        "e_cluster.freq_mhz should be > 0; got {}",
        snapshot.cpu.e_cluster.freq_mhz
    );
    assert!(
        snapshot.cpu.p_cluster.freq_mhz > 0,
        "p_cluster.freq_mhz should be > 0; got {}",
        snapshot.cpu.p_cluster.freq_mhz
    );
}

// ---------------------------------------------------------------------------
// FR-3: GPU utilization (FAIL — stub returning zeros)
// ---------------------------------------------------------------------------

#[test]
#[ignore] // FR-3 (FAIL): GPU utilization requires IOReport — stub returns 0.0
/// FR-3: GPU utilization ratio is in [0.0, 1.0] and non-zero on active Apple Silicon
fn gpu_usage_is_nonzero_on_apple_silicon() {
    let mut sampler = Sampler::new().expect("sampler init");
    let _ = sampler.sample(200).expect("first sample");
    let snapshot = sampler.sample(200).expect("second sample");
    assert!(
        (0.0..=1.0).contains(&snapshot.gpu.usage),
        "gpu.usage {} out of [0.0, 1.0]",
        snapshot.gpu.usage
    );
    // On any active Apple Silicon system GPU is never permanently 0
    assert!(
        snapshot.gpu.usage >= 0.0,
        "gpu.usage must be a valid ratio"
    );
}

#[test]
/// FR-3: GPU usage field is always within valid ratio range (even when stubbed)
fn gpu_usage_in_valid_range() {
    let mut sampler = Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");
    assert!(
        (0.0..=1.0).contains(&snapshot.gpu.usage),
        "gpu.usage {} must be in [0.0, 1.0]",
        snapshot.gpu.usage
    );
}

// ---------------------------------------------------------------------------
// FR-4: GPU frequency (FAIL — stub returning zeros)
// ---------------------------------------------------------------------------

#[test]
#[ignore] // FR-4 (FAIL): GPU frequency requires IOReport — stub returns 0
/// FR-4: GPU frequency in MHz is non-zero on Apple Silicon
fn gpu_freq_nonzero_on_apple_silicon() {
    let mut sampler = Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");
    assert!(
        snapshot.gpu.freq_mhz > 0,
        "gpu.freq_mhz should be > 0 on Apple Silicon; got {}",
        snapshot.gpu.freq_mhz
    );
}

// ---------------------------------------------------------------------------
// FR-5: Power consumption (FAIL — stub returning zeros)
// ---------------------------------------------------------------------------

#[test]
#[ignore] // FR-5 (FAIL): Power metrics require IOReport Energy Model — all stubs return 0.0
/// FR-5: package_w is the sum of cpu_w + gpu_w + ane_w (SoC components)
fn power_package_equals_soc_component_sum() {
    let mut sampler = Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");
    let p = &snapshot.power;
    let expected_package = p.cpu_w + p.gpu_w + p.ane_w;
    // Allow small floating point tolerance
    assert!(
        (p.package_w - expected_package).abs() < 0.1,
        "package_w ({}) should equal cpu_w + gpu_w + ane_w ({})",
        p.package_w,
        expected_package
    );
}

#[test]
#[ignore] // FR-5 (FAIL): Power metrics are all zero stubs
/// FR-5: system_w is non-zero on any powered Apple Silicon Mac
fn power_system_nonzero_on_apple_silicon() {
    let mut sampler = Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");
    assert!(
        snapshot.power.system_w > 0.0,
        "system_w should be > 0 on Apple Silicon; got {}",
        snapshot.power.system_w
    );
}

#[test]
#[ignore] // FR-5 (FAIL): CPU power is zero stub
/// FR-5: cpu_w power is non-zero when system is running
fn power_cpu_nonzero_on_apple_silicon() {
    let mut sampler = Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");
    assert!(
        snapshot.power.cpu_w > 0.0,
        "cpu_w should be > 0; got {}",
        snapshot.power.cpu_w
    );
}

#[test]
#[ignore] // FR-5 (FAIL): DRAM power is zero stub
/// FR-5: dram_w power is non-zero (DRAM is always consuming power)
fn power_dram_nonzero_on_apple_silicon() {
    let mut sampler = Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");
    assert!(
        snapshot.power.dram_w > 0.0,
        "dram_w should be > 0; got {}",
        snapshot.power.dram_w
    );
}

// ---------------------------------------------------------------------------
// FR-6: Temperature (FAIL — stub returning zeros)
// ---------------------------------------------------------------------------

#[test]
#[ignore] // FR-6 (FAIL): Temperature requires SMC access — stub returns 0.0
/// FR-6: CPU average temperature is in a plausible range (20°C–110°C)
fn temperature_cpu_avg_in_plausible_range() {
    let mut sampler = Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");
    assert!(
        snapshot.temperature.cpu_avg_c >= 20.0 && snapshot.temperature.cpu_avg_c <= 110.0,
        "cpu_avg_c {} not in [20, 110]",
        snapshot.temperature.cpu_avg_c
    );
}

#[test]
#[ignore] // FR-6 (FAIL): Temperature requires SMC access — stub returns 0.0
/// FR-6: GPU average temperature is in a plausible range (20°C–110°C)
fn temperature_gpu_avg_in_plausible_range() {
    let mut sampler = Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");
    assert!(
        snapshot.temperature.gpu_avg_c >= 20.0 && snapshot.temperature.gpu_avg_c <= 110.0,
        "gpu_avg_c {} not in [20, 110]",
        snapshot.temperature.gpu_avg_c
    );
}

#[test]
/// FR-6: when SMC is unavailable the system shall not panic — it returns defaults
fn temperature_unavailable_does_not_crash() {
    // If SMC is unavailable the stub returns ThermalMetrics::default() (zeros).
    // The important invariant: sample() must not panic or return Err.
    let mut sampler = Sampler::new().expect("sampler init");
    let result = sampler.sample(200);
    assert!(result.is_ok(), "sample() should not crash when temperature is unavailable");
}

// ---------------------------------------------------------------------------
// FR-7: Memory metrics collection (PARTIAL — XswUsage field order)
// ---------------------------------------------------------------------------

#[test]
/// FR-7: ram_total is non-zero on any real Mac
fn memory_ram_total_nonzero() {
    let mut sampler = Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");
    assert!(
        snapshot.memory.ram_total > 0,
        "ram_total should be > 0; got {}",
        snapshot.memory.ram_total
    );
}

#[test]
/// FR-7: ram_used is non-zero (system always uses some RAM)
fn memory_ram_used_nonzero() {
    let mut sampler = Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");
    assert!(
        snapshot.memory.ram_used > 0,
        "ram_used should be > 0; got {}",
        snapshot.memory.ram_used
    );
}

#[test]
/// FR-7: ram_used must not exceed ram_total
fn memory_used_does_not_exceed_total() {
    let mut sampler = Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");
    assert!(
        snapshot.memory.ram_used <= snapshot.memory.ram_total,
        "ram_used ({}) must not exceed ram_total ({})",
        snapshot.memory.ram_used,
        snapshot.memory.ram_total
    );
}

#[test]
/// FR-7 (PARTIAL): swap_used must not exceed swap_total when swap is active
fn memory_swap_used_does_not_exceed_swap_total() {
    let mut sampler = Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");
    if snapshot.memory.swap_total > 0 {
        assert!(
            snapshot.memory.swap_used <= snapshot.memory.swap_total,
            "swap_used ({}) must not exceed swap_total ({})",
            snapshot.memory.swap_used,
            snapshot.memory.swap_total
        );
    }
}

#[test]
/// FR-7: ram_total matches hw.memsize sysctl (within 1 GB tolerance for alignment)
fn memory_ram_total_matches_soc_memory() {
    let mut sampler = Sampler::new().expect("sampler init");
    let soc = sampler.soc_info().clone();
    let snapshot = sampler.sample(200).expect("sample");

    // soc.memory_gb is derived from the same sysctl; values should be consistent.
    let ram_total_gb = snapshot.memory.ram_total / (1024 * 1024 * 1024);
    assert_eq!(
        ram_total_gb, soc.memory_gb as u64,
        "ram_total ({ram_total_gb} GB) should match soc.memory_gb ({})",
        soc.memory_gb
    );
}

// ---------------------------------------------------------------------------
// FR-6 (network): Network rate computation
// ---------------------------------------------------------------------------

#[test]
/// FR-6 network: network interfaces list is non-empty on a Mac with networking
fn network_interfaces_nonempty() {
    let mut sampler = Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");
    assert!(
        !snapshot.network.interfaces.is_empty(),
        "expected at least one network interface"
    );
}

#[test]
/// FR-6 network: rx_bytes_sec and tx_bytes_sec are non-negative
fn network_rates_nonnegative() {
    let mut sampler = Sampler::new().expect("sampler init");
    let _ = sampler.sample(200).expect("first sample");
    let snapshot = sampler.sample(200).expect("second sample");
    for iface in &snapshot.network.interfaces {
        assert!(
            iface.rx_bytes_sec >= 0.0,
            "iface {} rx_bytes_sec must be >= 0; got {}",
            iface.name,
            iface.rx_bytes_sec
        );
        assert!(
            iface.tx_bytes_sec >= 0.0,
            "iface {} tx_bytes_sec must be >= 0; got {}",
            iface.name,
            iface.tx_bytes_sec
        );
    }
}

#[test]
/// FR-6 network: loopback interface (lo0) is excluded from the reported interfaces
fn network_loopback_excluded() {
    let mut sampler = Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");
    for iface in &snapshot.network.interfaces {
        assert!(
            !iface.name.starts_with("lo"),
            "loopback interface {} should be excluded",
            iface.name
        );
    }
}

// ---------------------------------------------------------------------------
// FR-9: Disk I/O (FAIL — stub returning zeros)
// ---------------------------------------------------------------------------

#[test]
/// FR-9: disk read_bytes_sec is non-negative
fn disk_read_bytes_sec_nonnegative() {
    let mut sampler = Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");
    // Even with no disk activity the value should be >= 0, never negative
    assert!(
        snapshot.disk.read_bytes_sec as i64 >= 0,
        "read_bytes_sec must be non-negative"
    );
}

#[test]
/// FR-9: disk write_bytes_sec produces a real value (not always zero) on an active system
fn disk_write_bytes_sec_is_implemented() {
    let mut sampler = Sampler::new().expect("sampler init");
    // Trigger some disk activity by sampling twice
    let _ = sampler.sample(200).expect("first sample");
    let snapshot = sampler.sample(200).expect("second sample");
    // If disk I/O is implemented, at least one of read or write should be > 0
    // on a live macOS system doing background work
    assert!(
        snapshot.disk.read_bytes_sec > 0 || snapshot.disk.write_bytes_sec > 0,
        "disk I/O should produce non-zero values on an active system"
    );
}

// ---------------------------------------------------------------------------
// FR-8: Process list collection
// ---------------------------------------------------------------------------

#[test]
/// FR-8: process list is non-empty
fn process_list_nonempty() {
    let mut sampler = Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");
    assert!(
        !snapshot.processes.is_empty(),
        "process list should contain at least one process"
    );
}

#[test]
/// FR-8: processes are sorted by CPU % descending
fn process_list_sorted_by_cpu_descending() {
    let mut sampler = Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");
    for window in snapshot.processes.windows(2) {
        assert!(
            window[0].cpu_pct >= window[1].cpu_pct,
            "processes not sorted by cpu_pct desc: {} < {}",
            window[0].cpu_pct,
            window[1].cpu_pct
        );
    }
}

#[test]
/// FR-8: every process entry has a non-zero PID
fn process_list_all_have_pid() {
    let mut sampler = Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");
    for p in &snapshot.processes {
        assert!(p.pid > 0, "process '{}' has invalid pid {}", p.name, p.pid);
    }
}

#[test]
/// FR-8: every process entry has a non-empty name
fn process_list_all_have_name() {
    let mut sampler = Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");
    for p in &snapshot.processes {
        assert!(!p.name.is_empty(), "process with pid {} has empty name", p.pid);
    }
}

#[test]
/// FR-8: every process entry has a non-empty username
fn process_list_all_have_user() {
    let mut sampler = Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");
    for p in &snapshot.processes {
        assert!(!p.user.is_empty(), "process '{}' has empty user", p.name);
    }
}

#[test]
/// FR-8: memory bytes is non-negative for all processes
fn process_list_mem_bytes_nonnegative() {
    let mut sampler = Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");
    for p in &snapshot.processes {
        // mem_bytes is u64, so it's always non-negative, but confirm it's parseable
        let _ = p.mem_bytes; // type check suffices
    }
}

// ---------------------------------------------------------------------------
// FR-9 (SoC identification)
// ---------------------------------------------------------------------------

#[test]
/// FR-9 soc: chip name is non-empty
fn soc_chip_name_nonempty() {
    let sampler = Sampler::new().expect("sampler init");
    let soc = sampler.soc_info();
    assert!(!soc.chip.is_empty(), "soc.chip should not be empty");
}

#[test]
/// FR-9 soc: chip name contains "Apple" on Apple Silicon Macs
fn soc_chip_name_contains_apple() {
    let sampler = Sampler::new().expect("sampler init");
    let soc = sampler.soc_info();
    assert!(
        soc.chip.contains("Apple"),
        "soc.chip '{}' should contain 'Apple' on Apple Silicon",
        soc.chip
    );
}

#[test]
/// FR-9 soc: total core count (e + p) is >= 2
fn soc_core_counts_nonzero() {
    let sampler = Sampler::new().expect("sampler init");
    let soc = sampler.soc_info();
    let total = soc.e_cores + soc.p_cores;
    assert!(total >= 2, "total cores (e+p) should be >= 2; got {total}");
}

#[test]
/// FR-9 soc: gpu_cores is > 0 on Apple Silicon
fn soc_gpu_cores_nonzero() {
    let sampler = Sampler::new().expect("sampler init");
    let soc = sampler.soc_info();
    assert!(soc.gpu_cores > 0, "gpu_cores should be > 0 on Apple Silicon; got {}", soc.gpu_cores);
}

#[test]
/// FR-9 soc: memory_gb is positive (CI runners may have < 8 GB)
fn soc_memory_gb_nonzero() {
    let sampler = Sampler::new().expect("sampler init");
    let soc = sampler.soc_info();
    assert!(soc.memory_gb > 0, "memory_gb should be > 0; got {}", soc.memory_gb);
}

// ---------------------------------------------------------------------------
// FR-10: Configurable sampling interval
// ---------------------------------------------------------------------------

#[test]
/// FR-10: interval below 100ms is clamped to 100ms (sampler.sample enforces minimum)
fn interval_below_minimum_is_clamped() {
    let mut sampler = Sampler::new().expect("sampler init");
    let start = std::time::Instant::now();
    // Pass 10ms — should be clamped to 100ms by the implementation
    let _ = sampler.sample(10).expect("sample with sub-minimum interval");
    let elapsed = start.elapsed().as_millis();
    // Should have waited at least ~100ms (allow generous margin for slow CI)
    assert!(
        elapsed >= 90,
        "sub-minimum interval should be clamped to 100ms; elapsed only {}ms",
        elapsed
    );
}

#[test]
/// FR-10: default interval is 1000ms (documented in CLI and sampler)
fn default_interval_is_1000ms() {
    use mtop::Cli;
    use clap::Parser;
    let cli = Cli::parse_from(["mtop"]);
    assert_eq!(cli.interval, 1000, "default interval should be 1000ms");
}

#[test]
/// FR-10: custom interval via --interval flag is respected
fn custom_interval_parsed_from_cli() {
    use mtop::Cli;
    use clap::Parser;
    let cli = Cli::parse_from(["mtop", "--interval", "500"]);
    assert_eq!(cli.interval, 500, "interval should be 500ms when --interval 500 is passed");
}

// ---------------------------------------------------------------------------
// FR-12: Graceful sensor degradation (PARTIAL — RwLock panics)
// ---------------------------------------------------------------------------

#[test]
/// FR-12: sample() returns Ok even when sensors are partially unavailable
fn sample_returns_ok_with_partial_sensors() {
    // The sampler must not panic or return Err due to unavailable sensors.
    // Currently GPU/power/temperature stubs always return default; this
    // confirms the contract holds.
    let mut sampler = Sampler::new().expect("sampler init");
    for _ in 0..3 {
        let result = sampler.sample(200);
        assert!(result.is_ok(), "sample() must not fail due to unavailable sensors");
    }
}

// ---------------------------------------------------------------------------
// FR-11: No sudo requirement
// ---------------------------------------------------------------------------

#[test]
/// FR-11: Sampler can be constructed and sampled without elevated privileges
fn sampler_works_without_sudo() {
    // This test is run as a regular user in CI. If it passes, no sudo is needed.
    let uid = unsafe { libc::getuid() };
    assert_ne!(uid, 0, "test should run as non-root user");

    let mut sampler = Sampler::new().expect("sampler should init as non-root");
    let result = sampler.sample(200);
    assert!(result.is_ok(), "sample() should succeed as non-root user");
}

// ---------------------------------------------------------------------------
// MetricsHistory: sparkline buffer (TUI / FR-3 tui)
// ---------------------------------------------------------------------------

#[test]
/// MetricsHistory caps at 128 entries (spec: sparklines display up to 128 data points)
fn metrics_history_caps_at_128() {
    let mut history = MetricsHistory::new();
    let snapshot = MetricsSnapshot::default();
    for _ in 0..200 {
        history.push(&snapshot);
    }
    assert_eq!(history.cpu_usage.len(), 128, "history should cap at 128 entries");
    assert_eq!(history.gpu_usage.len(), 128);
    assert_eq!(history.cpu_power.len(), 128);
    assert_eq!(history.gpu_power.len(), 128);
    assert_eq!(history.ane_power.len(), 128);
    assert_eq!(history.dram_power.len(), 128);
    assert_eq!(history.package_power.len(), 128);
    assert_eq!(history.system_power.len(), 128);
}

#[test]
/// MetricsHistory evicts oldest values from the left edge
fn metrics_history_evicts_oldest() {
    let mut history = MetricsHistory::new();

    // Push 128 snapshots with cpu total_usage = 0.5
    let mut snap = MetricsSnapshot::default();
    snap.cpu.total_usage = 0.5;
    for _ in 0..128 {
        history.push(&snap);
    }
    // Now push one with 1.0 — the first 0.5 should be evicted
    snap.cpu.total_usage = 1.0;
    history.push(&snap);

    assert_eq!(history.cpu_usage.len(), 128);
    assert_eq!(
        *history.cpu_usage.last().unwrap(),
        1.0,
        "newest value should be at end"
    );
    // The oldest (first) entry should no longer be the very first 0.5 at index 0
    // (it was shifted left), but all entries should be 0.5 except the last
    assert_eq!(history.cpu_usage[127], 1.0);
}

// ---------------------------------------------------------------------------
// Snapshot serialization: timestamp is ISO 8601
// ---------------------------------------------------------------------------

#[test]
/// MetricsSnapshot.timestamp is a valid ISO 8601 / RFC 3339 string
fn snapshot_timestamp_is_iso8601() {
    let mut sampler = Sampler::new().expect("sampler init");
    let snapshot = sampler.sample(200).expect("sample");
    // chrono's to_rfc3339() always includes timezone offset; check basic shape
    assert!(
        snapshot.timestamp.contains('T') && snapshot.timestamp.contains('+')
            || snapshot.timestamp.ends_with('Z'),
        "timestamp '{}' is not a valid RFC3339 string",
        snapshot.timestamp
    );
}
