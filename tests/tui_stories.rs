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
    s.cpu.total_usage = 0.42;
    s.cpu.e_cluster.freq_mhz = 1200;
    s.cpu.p_cluster.freq_mhz = 3400;
    s.temperature.cpu_avg_c = 55.0;
    s.temperature.available = true;
    s.power.cpu_w = 8.5;
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
