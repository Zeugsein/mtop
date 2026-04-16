use mtop::metrics::MetricsSnapshot;
use mtop::tui::{render_cpu_panel_compact_to_string, render_cpu_panel_expanded_to_string};
use mtop::tui::theme::THEMES;

fn dark_theme_idx() -> usize {
    0 // horizon is index 0
}

fn light_theme_idx() -> usize {
    THEMES.iter().position(|t| t.name == "solarized-light").unwrap_or(0)
}

fn cpu_fixture() -> MetricsSnapshot {
    let mut s = MetricsSnapshot::default();
    s.soc.chip = "Apple M3 Pro".to_string();
    s.soc.e_cores = 4;
    s.soc.p_cores = 6;
    s.soc.gpu_cores = 18;
    s.soc.memory_gb = 18;
    s.cpu.total_usage = 0.42;
    s.cpu.e_cluster.freq_mhz = 1200;
    s.cpu.e_cluster.usage = 0.24;
    s.cpu.p_cluster.freq_mhz = 3400;
    s.cpu.p_cluster.usage = 0.67;
    s.cpu.core_usages = vec![0.12, 0.45, 0.08, 0.31, 0.78, 0.55, 0.92, 0.43, 0.61, 0.39];
    s.power.cpu_w = 8.5;
    s.power.gpu_w = 3.2;
    s.power.ane_w = 0.8;
    s.power.dram_w = 1.5;
    s.power.package_w = 14.0;
    s.power.system_w = 16.5;
    s.power.available = true;
    s.temperature.cpu_avg_c = 55.0;
    s.temperature.gpu_avg_c = 48.0;
    s.temperature.available = true;
    s.temperature.fan_speeds = vec![1200];
    s
}

#[test]
fn cpu_panel_compact_dark() {
    let output = render_cpu_panel_compact_to_string(80, 12, cpu_fixture(), dark_theme_idx());
    insta::assert_snapshot!(output);
}

#[test]
fn cpu_panel_compact_light() {
    let output = render_cpu_panel_compact_to_string(80, 12, cpu_fixture(), light_theme_idx());
    insta::assert_snapshot!(output);
}

#[test]
fn cpu_panel_expanded_dark() {
    let output = render_cpu_panel_expanded_to_string(80, 40, cpu_fixture(), dark_theme_idx());
    insta::assert_snapshot!(output);
}

#[test]
fn cpu_panel_expanded_light() {
    let output = render_cpu_panel_expanded_to_string(80, 40, cpu_fixture(), light_theme_idx());
    insta::assert_snapshot!(output);
}
