use mtop::tui::{
    render_cpu_panel_compact_to_string,
    render_cpu_panel_expanded_to_string,
    render_gpu_panel_to_string,
    render_mem_panel_to_string,
    render_network_panel_to_string,
    render_process_panel_to_string,
    story_cpu_normal_fixture,
    story_cpu_stress_fixture,
    story_gpu_active_fixture,
    story_mem_near_full_fixture,
    story_network_active_fixture,
    story_process_populated_fixture,
};

fn dark_theme_idx() -> usize {
    0 // horizon is index 0
}

#[test]
fn cpu_compact_normal() {
    let output = render_cpu_panel_compact_to_string(80, 12, story_cpu_normal_fixture(), dark_theme_idx());
    insta::assert_snapshot!(output);
}

#[test]
fn cpu_show_normal() {
    let output = render_cpu_panel_expanded_to_string(80, 40, story_cpu_normal_fixture(), dark_theme_idx());
    insta::assert_snapshot!(output);
}

#[test]
fn cpu_expanded_stress() {
    let output = render_cpu_panel_expanded_to_string(80, 40, story_cpu_stress_fixture(), dark_theme_idx());
    insta::assert_snapshot!(output);
}

#[test]
fn gpu_active() {
    let output = render_gpu_panel_to_string(80, 40, story_gpu_active_fixture(), dark_theme_idx());
    insta::assert_snapshot!(output);
}

#[test]
fn mem_near_full() {
    let output = render_mem_panel_to_string(80, 40, story_mem_near_full_fixture(), dark_theme_idx());
    insta::assert_snapshot!(output);
}

#[test]
fn network_active() {
    let output = render_network_panel_to_string(80, 40, story_network_active_fixture(), dark_theme_idx());
    insta::assert_snapshot!(output);
}

#[test]
fn process_populated() {
    let output = render_process_panel_to_string(80, 40, story_process_populated_fixture(), dark_theme_idx());
    insta::assert_snapshot!(output);
}
