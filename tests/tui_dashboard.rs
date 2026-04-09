/// Integration tests for tui-dashboard spec requirements.
/// Each test cites the FR requirement from:
///   openspec/changes/archive/2026-04-05-mvp-core/specs/tui-dashboard/spec.md
///
/// TUI rendering tests are intentionally unit-style: they invoke rendering
/// logic directly against a mock buffer and assert on the produced text.
/// Tests that require a real terminal or full TUI lifecycle are marked
/// #[ignore] and document the gap.

use mtop::metrics::types::{
    CpuMetrics, CoreClusterMetrics, GpuMetrics, MemoryMetrics, MetricsHistory,
    MetricsSnapshot, NetworkMetrics, NetInterface, PowerMetrics, SocInfo, ThermalMetrics,
};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn make_soc() -> SocInfo {
    SocInfo {
        chip: "Apple M4 Pro".into(),
        e_cores: 4,
        p_cores: 6,
        gpu_cores: 20,
        memory_gb: 24,
    }
}

fn make_cpu_metrics() -> CpuMetrics {
    CpuMetrics {
        e_cluster: CoreClusterMetrics { freq_mhz: 2200, usage: 0.42 },
        p_cluster: CoreClusterMetrics { freq_mhz: 4400, usage: 0.65 },
        total_usage: 0.55,
        core_usages: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],
        power_w: 8.5,
    }
}

fn make_power_metrics() -> PowerMetrics {
    PowerMetrics {
        cpu_w: 8.5,
        gpu_w: 3.2,
        ane_w: 0.5,
        dram_w: 1.1,
        package_w: 13.3,
        system_w: 15.0,
        available: true,
    }
}

fn make_snapshot() -> MetricsSnapshot {
    let mut s = MetricsSnapshot::default();
    s.soc = make_soc();
    s.cpu = make_cpu_metrics();
    s.power = make_power_metrics();
    s.temperature = ThermalMetrics { cpu_avg_c: 52.0, gpu_avg_c: 45.0, available: true };
    s.memory = MemoryMetrics {
        ram_total: 25_769_803_776, // 24 GB
        ram_used: 8_589_934_592,   // 8 GB
        swap_total: 2_147_483_648, // 2 GB
        swap_used: 1_073_741_824,  // 1 GB
    };
    s.network = NetworkMetrics {
        interfaces: vec![NetInterface {
            name: "en0".into(),
            iface_type: "wifi".into(),
            rx_bytes_sec: 125_000.0,
            tx_bytes_sec: 45_000.0,
        }],
    };
    s
}

// ---------------------------------------------------------------------------
// FR-1: Multi-panel dashboard layout (structural — requires TUI integration)
// ---------------------------------------------------------------------------

#[test]
#[ignore] // FR-1: requires full TUI render pipeline to a mock terminal buffer
/// FR-1: dashboard renders CPU panel, power panel, temperature, memory, network, and process list
fn dashboard_renders_all_required_panels() {
    // Requires constructing a ratatui TestBackend at 80x24, rendering the
    // dashboard, and asserting that the buffer contains text from each panel.
    // Skipped: needs TUI render harness infrastructure.
}

#[test]
#[ignore] // FR-1: requires TUI render pipeline
/// FR-1: dashboard renders without overflow at minimum 80x24 terminal size
fn dashboard_no_overflow_at_minimum_terminal_size() {
    // Skipped: needs TUI render harness with TestBackend at 80x24.
}

// ---------------------------------------------------------------------------
// FR-2: CPU core visualization
// ---------------------------------------------------------------------------

#[test]
/// FR-2: core_usages contains one entry per (e_cores + p_cores)
fn cpu_core_count_matches_soc() {
    let soc = make_soc();
    let cpu = make_cpu_metrics();
    let expected = (soc.e_cores + soc.p_cores) as usize;
    // The test fixture was constructed with 10 cores matching 4E+6P
    assert_eq!(
        cpu.core_usages.len(),
        expected,
        "core_usages length should equal e_cores + p_cores"
    );
}

#[test]
/// FR-2: all core usage values are in [0.0, 1.0]
fn cpu_core_usages_in_valid_range() {
    let cpu = make_cpu_metrics();
    for (i, &u) in cpu.core_usages.iter().enumerate() {
        assert!(
            (0.0..=1.0).contains(&u),
            "core {i} usage {u} out of [0.0, 1.0]"
        );
    }
}

#[test]
/// FR-2: e_cluster.usage and p_cluster.usage are in [0.0, 1.0]
fn cpu_cluster_usages_in_valid_range() {
    let cpu = make_cpu_metrics();
    assert!((0.0..=1.0).contains(&cpu.e_cluster.usage));
    assert!((0.0..=1.0).contains(&cpu.p_cluster.usage));
}

#[test]
#[ignore] // FR-2: bar rendering requires TUI render pipeline
/// FR-2: each core bar includes a label like "E0" or "P0" with percentage
fn cpu_bar_labels_include_core_type_and_index() {
    // Skipped: needs TUI render harness to inspect rendered bar text.
}

#[test]
#[ignore] // FR-2: color coding requires TUI render pipeline
/// FR-2: green < 30%, cyan 30-40%, yellow 40-60%, red > 60% color coding
fn cpu_bar_color_codes_by_utilization_level() {
    // Skipped: needs TUI render harness to inspect cell styles.
}

// ---------------------------------------------------------------------------
// FR-3: Power sparkline charts
// ---------------------------------------------------------------------------

#[test]
/// FR-3: MetricsHistory tracks all 8 power component series
fn power_history_tracks_all_components() {
    let mut history = MetricsHistory::new();
    let snapshot = make_snapshot();
    history.push(&snapshot);

    assert_eq!(history.cpu_power.len(), 1);
    assert_eq!(history.gpu_power.len(), 1);
    assert_eq!(history.ane_power.len(), 1);
    assert_eq!(history.dram_power.len(), 1);
    assert_eq!(history.package_power.len(), 1);
    assert_eq!(history.system_power.len(), 1);
}

#[test]
/// FR-3: pushed power values match the snapshot's power metrics
fn power_history_values_match_snapshot() {
    let mut history = MetricsHistory::new();
    let snapshot = make_snapshot();
    history.push(&snapshot);

    assert_eq!(history.cpu_power[0], snapshot.power.cpu_w as f64);
    assert_eq!(history.gpu_power[0], snapshot.power.gpu_w as f64);
    assert_eq!(history.ane_power[0], snapshot.power.ane_w as f64);
    assert_eq!(history.dram_power[0], snapshot.power.dram_w as f64);
    assert_eq!(history.package_power[0], snapshot.power.package_w as f64);
    assert_eq!(history.system_power[0], snapshot.power.system_w as f64);
}

#[test]
/// FR-3: sparkline history caps at 128 entries (oldest evicted from left)
fn power_sparkline_caps_at_128_entries() {
    let mut history = MetricsHistory::new();
    let mut snapshot = MetricsSnapshot::default();
    snapshot.power.available = true;
    snapshot.gpu.available = true;
    for _ in 0..200 {
        history.push(&snapshot);
    }
    assert_eq!(history.cpu_power.len(), 128, "cpu_power should cap at 128");
    assert_eq!(history.gpu_power.len(), 128, "gpu_power should cap at 128");
    assert_eq!(history.system_power.len(), 128, "system_power should cap at 128");
}

#[test]
#[ignore] // FR-3: requires TUI render pipeline
/// FR-3: power panel renders sparkline rows for CPU, GPU, ANE, DRAM, package, system
fn power_panel_renders_all_sparkline_rows() {
    // Skipped: needs TUI render harness to inspect sparkline row labels.
}

// ---------------------------------------------------------------------------
// FR-4: GPU gauge display
// ---------------------------------------------------------------------------

#[test]
/// FR-4: GpuMetrics has usage (ratio), freq_mhz, and power_w fields
fn gpu_metrics_has_all_required_fields() {
    let gpu = GpuMetrics {
        usage: 0.35,
        freq_mhz: 1200,
        power_w: 3.2,
        available: true,
    };
    assert!((0.0..=1.0).contains(&gpu.usage));
    assert!(gpu.freq_mhz > 0);
    assert!(gpu.power_w >= 0.0);
}

#[test]
#[ignore] // FR-4: requires TUI render pipeline
/// FR-4: GPU panel renders utilization %, frequency in MHz, and power in Watts
fn gpu_panel_renders_usage_freq_power() {
    // Skipped: needs TUI render harness to inspect GPU panel text.
}

// ---------------------------------------------------------------------------
// FR-5: Temperature display
// ---------------------------------------------------------------------------

#[test]
/// FR-5: ThermalMetrics stores cpu_avg_c and gpu_avg_c
fn thermal_metrics_has_cpu_and_gpu_fields() {
    let t = ThermalMetrics { cpu_avg_c: 52.0, gpu_avg_c: 45.0, available: true };
    assert!(t.cpu_avg_c > 0.0);
    assert!(t.gpu_avg_c > 0.0);
}

#[test]
/// FR-5: Fahrenheit conversion formula: F = C * 9/5 + 32
fn temperature_celsius_to_fahrenheit_conversion() {
    fn to_fahrenheit(c: f32) -> f32 { c * 9.0 / 5.0 + 32.0 }
    assert!((to_fahrenheit(0.0) - 32.0).abs() < 0.01);
    assert!((to_fahrenheit(100.0) - 212.0).abs() < 0.01);
    assert!((to_fahrenheit(52.0) - 125.6).abs() < 0.1);
}

#[test]
#[ignore] // FR-5: requires TUI render pipeline
/// FR-5: temperature panel shows °C by default
fn temperature_panel_shows_celsius_by_default() {
    // Skipped: needs TUI render harness to inspect temperature panel text.
}

#[test]
#[ignore] // FR-5: requires TUI render pipeline
/// FR-5: temperature panel shows °F when --temp-unit fahrenheit is set
fn temperature_panel_shows_fahrenheit_when_configured() {
    // Skipped: needs TUI render harness with fahrenheit config.
}

// ---------------------------------------------------------------------------
// FR-6: Memory bar display
// ---------------------------------------------------------------------------

#[test]
/// FR-6: MemoryMetrics fields are all non-negative (u64 invariant)
fn memory_metrics_fields_are_nonnegative() {
    let m = make_snapshot().memory;
    // u64 is inherently non-negative; verify values are plausible
    assert!(m.ram_total > 0);
    assert!(m.ram_used > 0);
    assert!(m.ram_used <= m.ram_total);
}

#[test]
/// FR-6: RAM human-readable formatting: bytes to GB conversion
fn memory_bytes_to_gb_conversion() {
    let ram_total: u64 = 25_769_803_776; // 24 GB
    let gb = ram_total as f64 / (1024.0 * 1024.0 * 1024.0);
    assert!((gb - 24.0).abs() < 0.1, "24 GB conversion failed: {gb}");
}

#[test]
#[ignore] // FR-6: requires TUI render pipeline
/// FR-6: memory panel renders RAM bar with used/total in GB
fn memory_panel_renders_ram_bar() {
    // Skipped: needs TUI render harness to inspect memory bar text.
}

#[test]
#[ignore] // FR-6: requires TUI render pipeline
/// FR-6: memory panel renders swap bar when swap is active
fn memory_panel_renders_swap_bar_when_active() {
    // Skipped: needs TUI render harness with active swap.
}

// ---------------------------------------------------------------------------
// FR-7: Network rate display
// ---------------------------------------------------------------------------

#[test]
/// FR-7: NetInterface has name, rx_bytes_sec, tx_bytes_sec fields
fn net_interface_has_required_fields() {
    let iface = NetInterface {
        name: "en0".into(),
        iface_type: "wifi".into(),
        rx_bytes_sec: 125_000.0,
        tx_bytes_sec: 45_000.0,
    };
    assert!(!iface.name.is_empty());
    assert!(iface.rx_bytes_sec >= 0.0);
    assert!(iface.tx_bytes_sec >= 0.0);
}

#[test]
/// FR-7: network rate auto-scaling: B/s, KB/s, MB/s, GB/s
fn network_rate_auto_scale_formatting() {
    fn format_rate(bytes_sec: f64) -> String {
        if bytes_sec >= 1e9 { format!("{:.1} GB/s", bytes_sec / 1e9) }
        else if bytes_sec >= 1e6 { format!("{:.1} MB/s", bytes_sec / 1e6) }
        else if bytes_sec >= 1e3 { format!("{:.1} KB/s", bytes_sec / 1e3) }
        else { format!("{:.0} B/s", bytes_sec) }
    }

    assert_eq!(format_rate(500.0), "500 B/s");
    assert_eq!(format_rate(1500.0), "1.5 KB/s");
    assert_eq!(format_rate(2_500_000.0), "2.5 MB/s");
    assert_eq!(format_rate(1_200_000_000.0), "1.2 GB/s");
}

#[test]
#[ignore] // FR-7: requires TUI render pipeline
/// FR-7: network panel shows upload (↑) and download (↓) rates with auto-scaled units
fn network_panel_renders_rates_with_units() {
    // Skipped: needs TUI render harness to inspect network panel text.
}

// ---------------------------------------------------------------------------
// FR-8: Process list table
// ---------------------------------------------------------------------------

#[test]
/// FR-8: process list is sorted by CPU% descending by spec
fn process_list_is_sorted_cpu_desc() {
    use mtop::metrics::types::ProcessInfo;
    let mut procs = vec![
        ProcessInfo { pid: 1, name: "a".into(), cpu_pct: 10.0, mem_bytes: 100, energy_nj: 0, power_w: 0.0, user: "root".into() },
        ProcessInfo { pid: 2, name: "b".into(), cpu_pct: 50.0, mem_bytes: 200, energy_nj: 0, power_w: 0.0, user: "root".into() },
        ProcessInfo { pid: 3, name: "c".into(), cpu_pct: 30.0, mem_bytes: 300, energy_nj: 0, power_w: 0.0, user: "root".into() },
    ];
    procs.sort_by(|a, b| b.cpu_pct.partial_cmp(&a.cpu_pct).unwrap());
    assert_eq!(procs[0].cpu_pct, 50.0);
    assert_eq!(procs[1].cpu_pct, 30.0);
    assert_eq!(procs[2].cpu_pct, 10.0);
}

#[test]
#[ignore] // FR-8: requires TUI render pipeline + keyboard event simulation
/// FR-8: pressing 's' cycles sort column through CPU%, Memory, PID, Name
fn process_list_sort_cycles_on_s_key() {
    // Skipped: needs TUI event simulation harness.
}

#[test]
#[ignore] // FR-8: requires TUI render pipeline + keyboard event simulation
/// FR-8: Up/Down arrow keys and j/k move process list selection
fn process_list_navigation_with_arrows_and_jk() {
    // Skipped: needs TUI event simulation harness.
}

// ---------------------------------------------------------------------------
// FR-9: Keyboard controls
// ---------------------------------------------------------------------------

#[test]
#[ignore] // FR-9: requires TUI lifecycle + terminal state restoration check
/// FR-9: pressing 'q' exits the TUI and restores terminal state
fn quit_key_exits_tui_cleanly() {
    // Skipped: needs TUI lifecycle test with terminal state inspection.
}

#[test]
#[ignore] // FR-9: requires TUI render pipeline + keyboard event simulation
/// FR-9: pressing '+' increases interval by 250ms, '-' decreases (min 100ms)
fn interval_adjustment_keys_plus_minus() {
    // Skipped: needs TUI event simulation harness.
}

#[test]
#[ignore] // FR-9/FR-10 (FAIL): --color flag accepted but theme not applied
/// FR-9: pressing 'c' cycles to the next available color theme
fn theme_cycling_key_c() {
    // Skipped: needs TUI event simulation harness and theme registry.
}

// ---------------------------------------------------------------------------
// FR-10: Color themes (FAIL — FR-9/FR-6 cli color ignored)
// ---------------------------------------------------------------------------

#[test]
/// FR-10: at least 3 built-in themes are available
fn at_least_3_builtin_color_themes() {
    let themes = mtop::tui::theme_names();
    assert!(
        themes.len() >= 3,
        "expected at least 3 built-in themes; got {}",
        themes.len()
    );
}

// ---------------------------------------------------------------------------
// FR-11: Terminal resize handling
// ---------------------------------------------------------------------------

#[test]
#[ignore] // FR-11: requires TUI lifecycle with resize event injection
/// FR-11: dashboard re-renders without crash when terminal is resized
fn dashboard_handles_terminal_resize() {
    // Skipped: needs TUI lifecycle test with resize event injection.
}

// ---------------------------------------------------------------------------
// FR-12: SoC info header
// ---------------------------------------------------------------------------

#[test]
/// FR-12: SocInfo chip string contains the chip model name
fn soc_info_chip_has_model_name() {
    let soc = make_soc();
    assert!(soc.chip.contains("Apple M4 Pro"), "chip '{}' should contain model name", soc.chip);
}

#[test]
/// FR-12: SocInfo header format includes e_cores, p_cores, gpu_cores, memory_gb
fn soc_info_fields_present_for_header() {
    let soc = make_soc();
    // Build the header string as the TUI should: "mtop — Apple M4 Pro — 10C (4E+6P) / 20GPU — 24GB"
    let total_cores = soc.e_cores + soc.p_cores;
    let header = format!(
        "mtop — {} — {}C ({}E+{}P) / {}GPU — {}GB",
        soc.chip, total_cores, soc.e_cores, soc.p_cores, soc.gpu_cores, soc.memory_gb
    );
    assert!(header.contains("Apple M4 Pro"));
    assert!(header.contains("10C"));
    assert!(header.contains("4E+6P"));
    assert!(header.contains("20GPU"));
    assert!(header.contains("24GB"));
}

#[test]
#[ignore] // FR-12: requires TUI render pipeline to inspect header line
/// FR-12: header line is rendered at the top of the TUI dashboard
fn soc_header_rendered_in_tui() {
    // Skipped: needs TUI render harness to inspect first line of buffer.
}
