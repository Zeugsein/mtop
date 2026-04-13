/// Integration tests for mtop iteration 9: Expanded panel polish,
/// process detail expansion, thermal integration.

use mtop::metrics::{ProcessInfo, ThermalMetrics, SortMode};

// ---------------------------------------------------------------------------
// W1B: Fractional baudrate formatting
// ---------------------------------------------------------------------------

#[test]
fn format_baudrate_round_gbps() {
    let result = mtop::tui::helpers::format_baudrate(1_000_000_000);
    assert_eq!(result, "1 Gbps");
}

#[test]
fn format_baudrate_fractional_gbps() {
    let result = mtop::tui::helpers::format_baudrate(2_500_000_000);
    assert_eq!(result, "2.5 Gbps");
}

#[test]
fn format_baudrate_round_mbps() {
    let result = mtop::tui::helpers::format_baudrate(100_000_000);
    assert_eq!(result, "100 Mbps");
}

#[test]
fn format_baudrate_fractional_mbps() {
    let result = mtop::tui::helpers::format_baudrate(54_500_000);
    assert_eq!(result, "54.5 Mbps");
}

#[test]
fn format_baudrate_zero_returns_dash() {
    let result = mtop::tui::helpers::format_baudrate(0);
    assert_eq!(result, "—");
}

#[test]
fn format_baudrate_kbps() {
    let result = mtop::tui::helpers::format_baudrate(56_000);
    assert_eq!(result, "56 Kbps");
}

// ---------------------------------------------------------------------------
// W2A: Thread count field
// ---------------------------------------------------------------------------

#[test]
fn process_info_default_thread_count_zero() {
    let p = ProcessInfo::default();
    assert_eq!(p.thread_count, 0);
}

#[test]
fn process_info_thread_count_set() {
    let p = ProcessInfo {
        thread_count: 12,
        ..Default::default()
    };
    assert_eq!(p.thread_count, 12);
}

// ---------------------------------------------------------------------------
// W2B: Process I/O rate fields
// ---------------------------------------------------------------------------

#[test]
fn process_info_default_io_rates_zero() {
    let p = ProcessInfo::default();
    assert_eq!(p.io_read_bytes_sec, 0.0);
    assert_eq!(p.io_write_bytes_sec, 0.0);
}

#[test]
fn process_info_io_rates_set() {
    let p = ProcessInfo {
        io_read_bytes_sec: 1_048_576.0,
        io_write_bytes_sec: 524_288.0,
        ..Default::default()
    };
    assert_eq!(p.io_read_bytes_sec, 1_048_576.0);
    assert_eq!(p.io_write_bytes_sec, 524_288.0);
}

// ---------------------------------------------------------------------------
// W2C: Sort mode cycling
// ---------------------------------------------------------------------------

#[test]
fn sort_mode_default_is_weighted_score() {
    assert_eq!(SortMode::default(), SortMode::WeightedScore);
}

#[test]
fn sort_mode_cycle_from_weighted() {
    assert_eq!(SortMode::WeightedScore.next(), SortMode::Cpu);
}

#[test]
fn sort_mode_cycle_from_cpu() {
    assert_eq!(SortMode::Cpu.next(), SortMode::Memory);
}

#[test]
fn sort_mode_cycle_from_memory() {
    assert_eq!(SortMode::Memory.next(), SortMode::Power);
}

#[test]
fn sort_mode_cycle_from_power() {
    assert_eq!(SortMode::Power.next(), SortMode::Pid);
}

#[test]
fn sort_mode_cycle_from_pid() {
    assert_eq!(SortMode::Pid.next(), SortMode::Name);
}

#[test]
fn sort_mode_cycle_from_name_wraps() {
    assert_eq!(SortMode::Name.next(), SortMode::WeightedScore);
}

#[test]
fn sort_mode_full_cycle_returns_to_start() {
    let start = SortMode::WeightedScore;
    let end = start.next().next().next().next().next().next();
    assert_eq!(start, end);
}

#[test]
fn sort_mode_labels_non_empty() {
    let modes = [
        SortMode::WeightedScore, SortMode::Cpu, SortMode::Memory,
        SortMode::Power, SortMode::Pid, SortMode::Name,
    ];
    for mode in modes {
        assert!(!mode.label().is_empty(), "label for {:?} should not be empty", mode);
    }
}

#[test]
fn sort_mode_label_weighted_score() {
    assert_eq!(SortMode::WeightedScore.label(), "Score");
}

#[test]
fn sort_mode_label_cpu() {
    assert_eq!(SortMode::Cpu.label(), "CPU%");
}

// ---------------------------------------------------------------------------
// W3A: Thermal zone mapping — new fields
// ---------------------------------------------------------------------------

#[test]
fn thermal_metrics_default_ssd_zero() {
    let t = ThermalMetrics::default();
    assert_eq!(t.ssd_avg_c, 0.0);
}

#[test]
fn thermal_metrics_default_battery_zero() {
    let t = ThermalMetrics::default();
    assert_eq!(t.battery_avg_c, 0.0);
}

#[test]
fn thermal_metrics_ssd_field_set() {
    let t = ThermalMetrics {
        ssd_avg_c: 42.5,
        ..Default::default()
    };
    assert_eq!(t.ssd_avg_c, 42.5);
}

#[test]
fn thermal_metrics_battery_field_set() {
    let t = ThermalMetrics {
        battery_avg_c: 35.0,
        ..Default::default()
    };
    assert_eq!(t.battery_avg_c, 35.0);
}

// ---------------------------------------------------------------------------
// W3B: Thermal threshold alerts
// ---------------------------------------------------------------------------

#[test]
fn temp_color_normal_is_green() {
    let color = mtop::tui::helpers::temp_color(60.0, 80.0, 95.0);
    assert_eq!(color, ratatui::style::Color::Green);
}

#[test]
fn temp_color_warn_is_yellow() {
    let color = mtop::tui::helpers::temp_color(85.0, 80.0, 95.0);
    assert_eq!(color, ratatui::style::Color::Yellow);
}

#[test]
fn temp_color_critical_is_red() {
    let color = mtop::tui::helpers::temp_color(96.0, 80.0, 95.0);
    assert_eq!(color, ratatui::style::Color::Red);
}

#[test]
fn temp_color_exact_warn_threshold() {
    let color = mtop::tui::helpers::temp_color(80.0, 80.0, 95.0);
    assert_eq!(color, ratatui::style::Color::Yellow);
}

#[test]
fn temp_color_exact_crit_threshold() {
    let color = mtop::tui::helpers::temp_color(95.0, 80.0, 95.0);
    assert_eq!(color, ratatui::style::Color::Red);
}

#[test]
fn cpu_temp_thresholds_are_correct() {
    assert_eq!(mtop::tui::helpers::CPU_TEMP_WARN, 80.0);
    assert_eq!(mtop::tui::helpers::CPU_TEMP_CRIT, 95.0);
}

#[test]
fn gpu_temp_thresholds_are_correct() {
    assert_eq!(mtop::tui::helpers::GPU_TEMP_WARN, 85.0);
    assert_eq!(mtop::tui::helpers::GPU_TEMP_CRIT, 100.0);
}

// ---------------------------------------------------------------------------
// W3C: Fan speed field
// ---------------------------------------------------------------------------

#[test]
fn thermal_metrics_default_fan_speeds_empty() {
    let t = ThermalMetrics::default();
    assert!(t.fan_speeds.is_empty());
}

#[test]
fn thermal_metrics_fan_speeds_set() {
    let t = ThermalMetrics {
        fan_speeds: vec![1200, 1500],
        ..Default::default()
    };
    assert_eq!(t.fan_speeds.len(), 2);
    assert_eq!(t.fan_speeds[0], 1200);
    assert_eq!(t.fan_speeds[1], 1500);
}

// ---------------------------------------------------------------------------
// Format bytes rate compact (coverage for new I/O display)
// ---------------------------------------------------------------------------

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
