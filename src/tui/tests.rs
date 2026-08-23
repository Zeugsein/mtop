use super::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

// =========================================================================
// W0: AppState Default
// =========================================================================

#[test]
fn appstate_default_has_sensible_values() {
    let state = AppState::default();
    assert_eq!(state.interval_ms, 1000);
    assert_eq!(state.process_scroll, 0);
    assert_eq!(state.theme_idx, 0);
    assert_eq!(state.expanded_panel, None);
    assert_eq!(state.sort_mode, SortMode::default());
    assert_eq!(state.temp_unit, "celsius");
}

// =========================================================================
// W1: Extracted pure logic (prepare.rs)
// =========================================================================

use super::prepare::*;
use crate::metrics::{
    MemoryMetrics, NetInterface, PowerMetrics, ProcessInfo, SortMode as SM, ThermalMetrics,
};

fn make_test_procs() -> Vec<ProcessInfo> {
    vec![
        ProcessInfo {
            pid: 1,
            name: "alpha".to_string(),
            cpu_pct: 10.0,
            mem_bytes: 100 * 1024 * 1024,
            power_w: 1.0,
            user: "root".to_string(),
            ..Default::default()
        },
        ProcessInfo {
            pid: 2,
            name: "beta".to_string(),
            cpu_pct: 50.0,
            mem_bytes: 2u64 * 1024 * 1024 * 1024,
            power_w: 5.0,
            user: "lume".to_string(),
            ..Default::default()
        },
        ProcessInfo {
            pid: 3,
            name: "gamma".to_string(),
            cpu_pct: 30.0,
            mem_bytes: 500 * 1024 * 1024,
            power_w: 3.0,
            user: "lume".to_string(),
            ..Default::default()
        },
    ]
}

#[test]
fn prepare_process_rows_sort_by_cpu() {
    let procs = make_test_procs();
    let rows = prepare_process_rows(&procs, SM::Cpu, 0, 10, 50.0, 2 * 1024 * 1024 * 1024, 5.0);
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].name, "beta"); // 50% CPU
    assert_eq!(rows[1].name, "gamma"); // 30% CPU
    assert_eq!(rows[2].name, "alpha"); // 10% CPU
}

#[test]
fn prepare_process_rows_sort_by_name() {
    let procs = make_test_procs();
    let rows = prepare_process_rows(&procs, SM::Name, 0, 10, 50.0, 2 * 1024 * 1024 * 1024, 5.0);
    assert_eq!(rows[0].name, "alpha");
    assert_eq!(rows[1].name, "beta");
    assert_eq!(rows[2].name, "gamma");
}

#[test]
fn prepare_process_rows_sort_by_memory() {
    let procs = make_test_procs();
    let rows = prepare_process_rows(&procs, SM::Memory, 0, 10, 50.0, 2 * 1024 * 1024 * 1024, 5.0);
    assert_eq!(rows[0].name, "beta"); // 2 GB
    assert_eq!(rows[1].name, "gamma"); // 500 MB
    assert_eq!(rows[2].name, "alpha"); // 100 MB
}

#[test]
fn prepare_process_rows_sort_by_power() {
    let procs = make_test_procs();
    let rows = prepare_process_rows(&procs, SM::Power, 0, 10, 50.0, 2 * 1024 * 1024 * 1024, 5.0);
    assert_eq!(rows[0].name, "beta"); // 5W
    assert_eq!(rows[1].name, "gamma"); // 3W
    assert_eq!(rows[2].name, "alpha"); // 1W
}

#[test]
fn prepare_process_rows_sort_by_pid() {
    let procs = make_test_procs();
    let rows = prepare_process_rows(&procs, SM::Pid, 0, 10, 50.0, 2 * 1024 * 1024 * 1024, 5.0);
    assert_eq!(rows[0].pid, 1);
    assert_eq!(rows[1].pid, 2);
    assert_eq!(rows[2].pid, 3);
}

#[test]
fn prepare_process_rows_sort_by_weighted_score() {
    let procs = make_test_procs();
    // WeightedScore combines cpu, mem, power — beta dominates all three
    let rows = prepare_process_rows(
        &procs,
        SM::WeightedScore,
        0,
        10,
        50.0,
        2 * 1024 * 1024 * 1024,
        5.0,
    );
    assert_eq!(rows[0].name, "beta"); // highest across all dimensions
}

#[test]
fn prepare_process_rows_scroll_offset() {
    let procs = make_test_procs();
    let rows = prepare_process_rows(&procs, SM::Pid, 1, 10, 50.0, 2 * 1024 * 1024 * 1024, 5.0);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].pid, 2);
}

#[test]
fn prepare_process_rows_max_visible() {
    let procs = make_test_procs();
    let rows = prepare_process_rows(&procs, SM::Pid, 0, 2, 50.0, 2 * 1024 * 1024 * 1024, 5.0);
    assert_eq!(rows.len(), 2);
}

#[test]
fn prepare_process_rows_mem_display_gb() {
    let procs = make_test_procs();
    let rows = prepare_process_rows(&procs, SM::Pid, 0, 10, 50.0, 2 * 1024 * 1024 * 1024, 5.0);
    // beta has 2 GB
    assert!(
        rows[1].mem_display.contains("G"),
        "2 GB should display as G: {}",
        rows[1].mem_display
    );
    // alpha has 100 MB
    assert!(
        rows[0].mem_display.contains("M"),
        "100 MB should display as M: {}",
        rows[0].mem_display
    );
}

#[test]
fn prepare_process_rows_cpu_norm() {
    let procs = make_test_procs();
    let rows = prepare_process_rows(&procs, SM::Cpu, 0, 10, 50.0, 2 * 1024 * 1024 * 1024, 5.0);
    assert!(
        (rows[0].cpu_norm - 1.0).abs() < 0.01,
        "beta (50/50) should be ~1.0"
    );
    assert!(
        (rows[2].cpu_norm - 0.2).abs() < 0.01,
        "alpha (10/50) should be ~0.2"
    );
}

#[test]
fn prepare_network_rows_filters_infrastructure() {
    let ifaces = vec![
        NetInterface {
            name: "en0".to_string(),
            iface_type: "Ethernet".to_string(),
            rx_bytes_sec: 100.0,
            tx_bytes_sec: 200.0,
            ..Default::default()
        },
        NetInterface {
            name: "bridge0".to_string(),
            iface_type: "Bridge".to_string(),
            rx_bytes_sec: 10.0,
            tx_bytes_sec: 20.0,
            ..Default::default()
        },
        NetInterface {
            name: "awdl0".to_string(),
            iface_type: "AirDrop".to_string(),
            rx_bytes_sec: 5.0,
            tx_bytes_sec: 5.0,
            ..Default::default()
        },
        NetInterface {
            name: "en1".to_string(),
            iface_type: "Wi-Fi".to_string(),
            rx_bytes_sec: 500.0,
            tx_bytes_sec: 600.0,
            ..Default::default()
        },
    ];
    let rows = prepare_network_rows(&ifaces);
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|r| r.name == "en0" || r.name == "en1"));
}

#[test]
fn prepare_network_rows_sorted_by_total_traffic() {
    let ifaces = vec![
        NetInterface {
            name: "en0".to_string(),
            rx_bytes_sec: 100.0,
            tx_bytes_sec: 200.0,
            ..Default::default()
        },
        NetInterface {
            name: "en1".to_string(),
            rx_bytes_sec: 500.0,
            tx_bytes_sec: 600.0,
            ..Default::default()
        },
    ];
    let rows = prepare_network_rows(&ifaces);
    assert_eq!(rows[0].name, "en1"); // 1100 total
    assert_eq!(rows[1].name, "en0"); // 300 total
}

#[test]
fn prepare_network_rows_empty_input() {
    let rows = prepare_network_rows(&[]);
    assert!(rows.is_empty());
}

#[test]
fn prepare_memory_pressure_fractions() {
    let mem = MemoryMetrics {
        ram_total: 16 * 1024 * 1024 * 1024,
        ram_used: 12 * 1024 * 1024 * 1024,
        wired: 4 * 1024 * 1024 * 1024,
        app: 6 * 1024 * 1024 * 1024,
        compressed: 2 * 1024 * 1024 * 1024,
        ..Default::default()
    };
    let p = prepare_memory_pressure(&mem, 16.0);
    assert!(
        (p.wired_frac - 0.25).abs() < 0.01,
        "wired 4/16 = 0.25: {}",
        p.wired_frac
    );
    assert!(
        (p.app_frac - 0.375).abs() < 0.01,
        "app 6/16 = 0.375: {}",
        p.app_frac
    );
    assert!(
        (p.compressed_frac - 0.125).abs() < 0.01,
        "compressed 2/16 = 0.125: {}",
        p.compressed_frac
    );
    assert!(p.wired_frac + p.app_frac + p.compressed_frac <= 1.0);
}

#[test]
fn prepare_memory_pressure_zero_total() {
    let mem = MemoryMetrics {
        wired: 1024,
        app: 2048,
        compressed: 512,
        ..Default::default()
    };
    let p = prepare_memory_pressure(&mem, 0.0);
    // Should clamp to 1.0 max, not panic
    assert!(p.wired_frac <= 1.0);
    assert!(p.app_frac <= 1.0);
    assert!(p.compressed_frac <= 1.0);
}

#[test]
fn prepare_power_components_has_six_entries() {
    let power = PowerMetrics {
        cpu_w: 5.0,
        gpu_w: 3.0,
        ane_w: 0.5,
        dram_w: 1.0,
        system_w: 2.0,
        package_w: 10.0,
        available: true,
    };
    let thermal = ThermalMetrics {
        fan_speeds: vec![2000, 3000],
        ..Default::default()
    };
    let (components, fans) = prepare_power_components(&power, &thermal);
    assert_eq!(components.len(), 6);
    assert_eq!(components[0].name, "CPU");
    assert_eq!(components[0].watts, 5.0);
    assert_eq!(fans, vec![2000, 3000]);
}

// =========================================================================
// W2: TestBackend rendering tests
// =========================================================================

fn render_dashboard_at_size(width: u16, height: u16) {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let state = AppState::default();
    terminal.draw(|f| draw_dashboard(f, &state)).unwrap();
}

#[test]
fn dashboard_renders_80x24() {
    render_dashboard_at_size(80, 24);
}

#[test]
fn dashboard_renders_120x40() {
    render_dashboard_at_size(120, 40);
}

#[test]
fn dashboard_renders_60x20() {
    render_dashboard_at_size(60, 20);
}

#[test]
fn dashboard_renders_minimum_40x10() {
    render_dashboard_at_size(40, 10);
}

fn render_expanded_panel_at_size(panel: PanelId, width: u16, height: u16) {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = AppState::default();
    state.expanded_panel = Some(panel);
    terminal.draw(|f| draw_dashboard(f, &state)).unwrap();
}

#[test]
fn expanded_cpu_80x24() {
    render_expanded_panel_at_size(PanelId::Cpu, 80, 24);
}
#[test]
fn expanded_gpu_80x24() {
    render_expanded_panel_at_size(PanelId::Gpu, 80, 24);
}
#[test]
fn expanded_memdisk_80x24() {
    render_expanded_panel_at_size(PanelId::MemDisk, 80, 24);
}
#[test]
fn expanded_network_80x24() {
    render_expanded_panel_at_size(PanelId::Network, 80, 24);
}
#[test]
fn expanded_power_80x24() {
    render_expanded_panel_at_size(PanelId::Power, 80, 24);
}
#[test]
fn expanded_process_80x24() {
    render_expanded_panel_at_size(PanelId::Process, 80, 24);
}

#[test]
fn expanded_cpu_120x40() {
    render_expanded_panel_at_size(PanelId::Cpu, 120, 40);
}
#[test]
fn expanded_gpu_120x40() {
    render_expanded_panel_at_size(PanelId::Gpu, 120, 40);
}
#[test]
fn expanded_memdisk_120x40() {
    render_expanded_panel_at_size(PanelId::MemDisk, 120, 40);
}
#[test]
fn expanded_network_120x40() {
    render_expanded_panel_at_size(PanelId::Network, 120, 40);
}
#[test]
fn expanded_power_120x40() {
    render_expanded_panel_at_size(PanelId::Power, 120, 40);
}
#[test]
fn expanded_process_120x40() {
    render_expanded_panel_at_size(PanelId::Process, 120, 40);
}

fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
    let buf = terminal.backend().buffer();
    let mut text = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            let cell = &buf[(x, y)];
            text.push_str(cell.symbol());
        }
        text.push('\n');
    }
    text
}

#[test]
fn dashboard_contains_cpu_text() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    let state = AppState::default();
    terminal.draw(|f| draw_dashboard(f, &state)).unwrap();
    let text = buffer_text(&terminal);
    assert!(text.contains("cpu"), "Dashboard should contain 'cpu' text");
}

#[test]
fn dashboard_contains_gpu_text() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    let state = AppState::default();
    terminal.draw(|f| draw_dashboard(f, &state)).unwrap();
    let text = buffer_text(&terminal);
    assert!(text.contains("GPU"), "Dashboard should contain 'GPU' text");
}

#[test]
fn dashboard_contains_network_text() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    let state = AppState::default();
    terminal.draw(|f| draw_dashboard(f, &state)).unwrap();
    let text = buffer_text(&terminal);
    assert!(text.contains("net"), "Dashboard should contain 'net' text");
}

#[test]
fn dashboard_contains_process_text() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    let state = AppState::default();
    terminal.draw(|f| draw_dashboard(f, &state)).unwrap();
    let text = buffer_text(&terminal);
    assert!(
        text.contains("proc"),
        "Dashboard should contain 'proc' text"
    );
}

#[test]
fn dashboard_contains_footer() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    let state = AppState::default();
    terminal.draw(|f| draw_dashboard(f, &state)).unwrap();
    let text = buffer_text(&terminal);
    assert!(
        text.contains("help"),
        "Dashboard should contain footer with help keybinding"
    );
}

#[test]
fn dashboard_contains_mtop_header() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    let state = AppState::default();
    terminal.draw(|f| draw_dashboard(f, &state)).unwrap();
    let text = buffer_text(&terminal);
    assert!(
        text.contains("mtop"),
        "Dashboard should contain 'mtop' in header"
    );
}

// =========================================================================
// W3: Input handler tests
// =========================================================================

fn make_key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::empty())
}

fn make_key_ctrl(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::CONTROL)
}

#[test]
fn key_q_quits() {
    let mut state = AppState::default();
    assert!(input::handle_key_event(
        make_key(KeyCode::Char('q')),
        &mut state
    ));
}

#[test]
fn key_ctrl_c_quits() {
    let mut state = AppState::default();
    assert!(input::handle_key_event(
        make_key_ctrl(KeyCode::Char('c')),
        &mut state
    ));
}

#[test]
fn key_esc_without_expanded_quits() {
    let mut state = AppState::default();
    assert!(input::handle_key_event(make_key(KeyCode::Esc), &mut state));
}

#[test]
fn key_esc_from_expanded_collapses() {
    let mut state = AppState::default();
    state.expanded_panel = Some(PanelId::Cpu);
    let quit = input::handle_key_event(make_key(KeyCode::Esc), &mut state);
    assert!(!quit, "Esc from expanded should not quit");
    assert_eq!(state.expanded_panel, None, "expanded should be cleared");
}

#[test]
fn key_c_cycles_theme() {
    let mut state = AppState::default();
    assert_eq!(state.theme_idx, 0);
    input::handle_key_event(make_key(KeyCode::Char('c')), &mut state);
    assert_eq!(state.theme_idx, 1);
}

#[test]
fn key_c_wraps_theme() {
    let mut state = AppState::default();
    state.theme_idx = theme::THEMES.len() - 1;
    input::handle_key_event(make_key(KeyCode::Char('c')), &mut state);
    assert_eq!(state.theme_idx, 0, "theme should wrap around");
}

#[test]
fn key_1_expands_cpu() {
    let mut state = AppState::default();
    input::handle_key_event(make_key(KeyCode::Char('1')), &mut state);
    assert_eq!(state.expanded_panel, Some(PanelId::Cpu));
}

#[test]
fn key_2_expands_gpu() {
    let mut state = AppState::default();
    input::handle_key_event(make_key(KeyCode::Char('2')), &mut state);
    assert_eq!(state.expanded_panel, Some(PanelId::Gpu));
}

#[test]
fn key_3_expands_memdisk() {
    let mut state = AppState::default();
    input::handle_key_event(make_key(KeyCode::Char('3')), &mut state);
    assert_eq!(state.expanded_panel, Some(PanelId::MemDisk));
}

#[test]
fn key_4_expands_network() {
    let mut state = AppState::default();
    input::handle_key_event(make_key(KeyCode::Char('4')), &mut state);
    assert_eq!(state.expanded_panel, Some(PanelId::Network));
}

#[test]
fn key_5_expands_power() {
    let mut state = AppState::default();
    input::handle_key_event(make_key(KeyCode::Char('5')), &mut state);
    assert_eq!(state.expanded_panel, Some(PanelId::Power));
}

#[test]
fn key_6_expands_process() {
    let mut state = AppState::default();
    input::handle_key_event(make_key(KeyCode::Char('6')), &mut state);
    assert_eq!(state.expanded_panel, Some(PanelId::Process));
}

#[test]
fn key_1_toggles_expand_collapse() {
    let mut state = AppState::default();
    input::handle_key_event(make_key(KeyCode::Char('1')), &mut state);
    assert_eq!(state.expanded_panel, Some(PanelId::Cpu));
    // Same key again collapses
    input::handle_key_event(make_key(KeyCode::Char('1')), &mut state);
    assert_eq!(state.expanded_panel, None);
}

#[test]
fn key_switches_expanded_panel() {
    let mut state = AppState::default();
    input::handle_key_event(make_key(KeyCode::Char('1')), &mut state);
    assert_eq!(state.expanded_panel, Some(PanelId::Cpu));
    // Different key switches
    input::handle_key_event(make_key(KeyCode::Char('4')), &mut state);
    assert_eq!(state.expanded_panel, Some(PanelId::Network));
}

#[test]
fn key_e_collapses_expanded() {
    let mut state = AppState::default();
    state.expanded_panel = Some(PanelId::Gpu);
    input::handle_key_event(make_key(KeyCode::Char('e')), &mut state);
    assert_eq!(state.expanded_panel, None);
}

#[test]
fn key_plus_increases_interval() {
    let mut state = AppState::default();
    assert_eq!(state.interval_ms, 1000);
    input::handle_key_event(make_key(KeyCode::Char('+')), &mut state);
    assert_eq!(state.interval_ms, 1500);
}

#[test]
fn key_plus_caps_at_10000() {
    let mut state = AppState::default();
    state.interval_ms = 10000;
    input::handle_key_event(make_key(KeyCode::Char('+')), &mut state);
    assert_eq!(state.interval_ms, 10000);
}

#[test]
fn key_minus_decreases_interval() {
    let mut state = AppState::default();
    input::handle_key_event(make_key(KeyCode::Char('-')), &mut state);
    assert_eq!(state.interval_ms, 750); // 1000 → prev preset 750
}

#[test]
fn key_minus_floors_at_100() {
    let mut state = AppState::default();
    state.interval_ms = 100;
    input::handle_key_event(make_key(KeyCode::Char('-')), &mut state);
    assert_eq!(state.interval_ms, 100);
}

#[test]
fn key_j_selects_next_in_expanded_process() {
    let mut state = make_process_state(make_test_procs());
    state.sort_mode = SM::Pid;
    state.process_selected = Some(0);
    input::handle_key_event(make_key(KeyCode::Char('j')), &mut state);
    assert_eq!(state.process_selected, Some(1));
    assert_eq!(state.selected_pid, Some(2));
}

#[test]
fn key_j_noop_outside_expanded() {
    let mut state = AppState::default();
    let scroll_before = state.process_scroll;
    input::handle_key_event(make_key(KeyCode::Char('j')), &mut state);
    assert_eq!(state.process_scroll, scroll_before);
}

#[test]
fn key_k_at_zero_stays_zero() {
    let mut state = AppState::default();
    input::handle_key_event(make_key(KeyCode::Char('k')), &mut state);
    assert_eq!(state.process_scroll, 0);
}

#[test]
fn key_s_cycles_sort_mode() {
    let mut state = AppState::default();
    assert_eq!(state.sort_mode, SM::WeightedScore);
    input::handle_key_event(make_key(KeyCode::Char('s')), &mut state);
    assert_eq!(state.sort_mode, SM::Cpu);
    input::handle_key_event(make_key(KeyCode::Char('s')), &mut state);
    assert_eq!(state.sort_mode, SM::Memory);
}

#[test]
fn key_enter_collapses_expanded() {
    let mut state = AppState::default();
    state.expanded_panel = Some(PanelId::Cpu);
    input::handle_key_event(make_key(KeyCode::Enter), &mut state);
    assert_eq!(state.expanded_panel, None);
}

#[test]
fn unknown_key_does_nothing() {
    let mut state = AppState::default();
    let quit = input::handle_key_event(make_key(KeyCode::Char('z')), &mut state);
    assert!(!quit);
    assert_eq!(state.theme_idx, 0);
    assert_eq!(state.process_scroll, 0);
}

// =========================================================================
// W4: Uncovered input branches (iteration 20)
// =========================================================================

#[test]
fn key_dot_toggles_show_detail_on() {
    let mut state = AppState::default();
    assert!(!state.show_detail);
    input::handle_key_event(make_key(KeyCode::Char('.')), &mut state);
    assert!(state.show_detail, "'.' should enable show_detail");
}

#[test]
fn key_dot_toggles_show_detail_off() {
    let mut state = AppState::default();
    state.show_detail = true;
    input::handle_key_event(make_key(KeyCode::Char('.')), &mut state);
    assert!(!state.show_detail, "second '.' should disable show_detail");
}

#[test]
fn key_h_toggles_help_on() {
    let mut state = AppState::default();
    assert!(!state.show_help);
    input::handle_key_event(make_key(KeyCode::Char('h')), &mut state);
    assert!(state.show_help, "'h' should enable show_help");
}

#[test]
fn key_h_toggles_help_off() {
    let mut state = AppState::default();
    state.show_help = true;
    input::handle_key_event(make_key(KeyCode::Char('h')), &mut state);
    assert!(!state.show_help, "second 'h' should disable show_help");
}

#[test]
fn key_question_toggles_help_on() {
    let mut state = AppState::default();
    input::handle_key_event(make_key(KeyCode::Char('?')), &mut state);
    assert!(state.show_help, "'?' should enable show_help");
}

#[test]
fn key_esc_clears_help_without_quitting() {
    let mut state = AppState::default();
    state.show_help = true;
    let quit = input::handle_key_event(make_key(KeyCode::Esc), &mut state);
    assert!(!quit, "Esc with show_help=true should not quit");
    assert!(!state.show_help, "Esc should clear show_help");
}

#[test]
fn key_arrow_down_scrolls_process() {
    let mut state = AppState::default();
    assert_eq!(state.process_scroll, 0);
    input::handle_key_event(make_key(KeyCode::Down), &mut state);
    assert_eq!(state.process_scroll, 1);
}

#[test]
fn key_arrow_up_scrolls_process() {
    let mut state = AppState::default();
    state.process_scroll = 3;
    input::handle_key_event(make_key(KeyCode::Up), &mut state);
    assert_eq!(state.process_scroll, 2);
}

#[test]
fn key_arrow_up_at_zero_stays_zero() {
    let mut state = AppState::default();
    input::handle_key_event(make_key(KeyCode::Up), &mut state);
    assert_eq!(
        state.process_scroll, 0,
        "Up at scroll=0 should saturate at 0"
    );
}

#[test]
fn key_equals_increases_interval() {
    let mut state = AppState::default();
    assert_eq!(state.interval_ms, 1000);
    input::handle_key_event(make_key(KeyCode::Char('=')), &mut state);
    assert_eq!(state.interval_ms, 1500, "'=' should be an alias for '+'");
}

#[test]
fn key_plus_at_max_stays_at_10000() {
    let mut state = AppState::default();
    state.interval_ms = 10000;
    input::handle_key_event(make_key(KeyCode::Char('+')), &mut state);
    assert_eq!(state.interval_ms, 10000, "interval should not exceed 10000");
}

#[test]
fn key_minus_at_minimum_stays_at_100() {
    let mut state = AppState::default();
    state.interval_ms = 100;
    input::handle_key_event(make_key(KeyCode::Char('-')), &mut state);
    assert_eq!(state.interval_ms, 100, "interval should not go below 100");
}

// =========================================================================
// Reverse theme cycling (Shift+C)
// =========================================================================

/// Reverse cycle from index 0 wraps to last theme. (ref: SHALL-AD-02a)
#[test]
fn theme_reverse_cycle_wraps_to_last() {
    let mut state = AppState::default();
    state.theme_idx = 0;
    input::handle_key_event(make_key(KeyCode::Char('C')), &mut state);
    assert_eq!(
        state.theme_idx,
        theme::THEMES.len() - 1,
        "reverse cycling from 0 should wrap to last theme"
    );
}

/// Reverse cycle from index 5 decrements to 4. (ref: SHALL-AD-02b)
#[test]
fn theme_reverse_cycle_decrements() {
    let mut state = AppState::default();
    state.theme_idx = 5;
    input::handle_key_event(make_key(KeyCode::Char('C')), &mut state);
    assert_eq!(state.theme_idx, 4, "reverse cycling from 5 should go to 4");
}

/// Forward then reverse theme cycling round-trips: 0 -> 1 -> 0. (ref: SHALL-AD-02c)
#[test]
fn theme_forward_reverse_roundtrip() {
    let mut state = AppState::default();
    state.theme_idx = 0;
    input::handle_key_event(make_key(KeyCode::Char('c')), &mut state);
    assert_eq!(state.theme_idx, 1, "forward from 0 -> 1");
    input::handle_key_event(make_key(KeyCode::Char('C')), &mut state);
    assert_eq!(state.theme_idx, 0, "reverse from 1 -> 0 (round-trip)");
}

// =========================================================================
// Interval preset ladder (+/-/= keys)
// =========================================================================

/// Interval 1000ms + key -> 1500ms. (ref: SHALL-26-05a)
#[test]
fn interval_1000_plus_becomes_1500() {
    let mut state = AppState::default();
    state.interval_ms = 1000;
    input::handle_key_event(make_key(KeyCode::Char('+')), &mut state);
    assert_eq!(state.interval_ms, 1500);
}

/// Interval 1000ms - key -> 750ms. (ref: SHALL-26-05b)
#[test]
fn interval_1000_minus_becomes_750() {
    let mut state = AppState::default();
    state.interval_ms = 1000;
    input::handle_key_event(make_key(KeyCode::Char('-')), &mut state);
    assert_eq!(state.interval_ms, 750);
}

/// Interval caps at 10000ms maximum. (ref: SHALL-26-05c)
#[test]
fn interval_caps_at_10000() {
    let mut state = AppState::default();
    state.interval_ms = 10000;
    input::handle_key_event(make_key(KeyCode::Char('+')), &mut state);
    assert_eq!(state.interval_ms, 10000);
}

/// Interval floors at 100ms minimum. (ref: SHALL-26-05d)
#[test]
fn interval_floors_at_100() {
    let mut state = AppState::default();
    state.interval_ms = 100;
    input::handle_key_event(make_key(KeyCode::Char('-')), &mut state);
    assert_eq!(state.interval_ms, 100);
}

/// Interval 750ms + key -> 1000ms (1000 always reachable). (ref: SHALL-26-05e)
#[test]
fn interval_750_plus_reaches_1000() {
    let mut state = AppState::default();
    state.interval_ms = 750;
    input::handle_key_event(make_key(KeyCode::Char('+')), &mut state);
    assert_eq!(state.interval_ms, 1000);
}

/// Interval 1500ms - key -> 1000ms (1000 always reachable). (ref: SHALL-26-05f)
#[test]
fn interval_1500_minus_reaches_1000() {
    let mut state = AppState::default();
    state.interval_ms = 1500;
    input::handle_key_event(make_key(KeyCode::Char('-')), &mut state);
    assert_eq!(state.interval_ms, 1000);
}

/// '=' key behaves same as '+'. (ref: SHALL-26-05g)
#[test]
fn interval_equals_same_as_plus() {
    let mut state = AppState::default();
    state.interval_ms = 1000;
    input::handle_key_event(make_key(KeyCode::Char('=')), &mut state);
    assert_eq!(state.interval_ms, 1500, "'=' should behave like '+'");
}

// =========================================================================
// I58: pid-keyed process selection stability
// =========================================================================

/// Build a Process-expanded AppState with `procs` loaded into the snapshot.
fn make_process_state(procs: Vec<ProcessInfo>) -> AppState {
    let mut state = AppState::default();
    state.expanded_panel = Some(PanelId::Process);
    state.snapshot.processes = procs;
    state
}

/// Test fixture where CPU-sort and Memory-sort orders genuinely diverge, so
/// that a test which selects by pid can distinguish real pid-key resolution
/// from cursor-fallback (which would return whatever pid happens to occupy
/// the same cursor row after the reordering).
///
/// Layout:
///   pid 100 "a" — high cpu (90%), low mem  → CPU rank 0, Memory rank 2
///   pid 200 "b" — low cpu (5%), high mem   → CPU rank 2, Memory rank 0
///   pid 300 "c" — medium both              → CPU rank 1, Memory rank 1
fn make_divergent_test_procs() -> Vec<ProcessInfo> {
    vec![
        ProcessInfo {
            pid: 100,
            name: "a".to_string(),
            cpu_pct: 90.0,
            mem_bytes: 10 * 1024 * 1024,
            power_w: 0.5,
            user: "u".to_string(),
            ..Default::default()
        },
        ProcessInfo {
            pid: 200,
            name: "b".to_string(),
            cpu_pct: 5.0,
            mem_bytes: 4u64 * 1024 * 1024 * 1024,
            power_w: 0.5,
            user: "u".to_string(),
            ..Default::default()
        },
        ProcessInfo {
            pid: 300,
            name: "c".to_string(),
            cpu_pct: 40.0,
            mem_bytes: 500 * 1024 * 1024,
            power_w: 0.5,
            user: "u".to_string(),
            ..Default::default()
        },
    ]
}

/// I58-F1c: pressing `↓` populates `selected_pid` from the row now under the cursor.
#[test]
fn i58_nav_down_anchors_selected_pid() {
    let mut state = make_process_state(make_test_procs());
    // WeightedScore sort → order beta, gamma, alpha (2, 3, 1) roughly.
    // Preserved unwrap_or(0).saturating_add(1) means first ↓ lands on row 1.
    input::handle_key_event(make_key(KeyCode::Down), &mut state);
    assert_eq!(state.process_selected, Some(1));
    assert!(
        state.selected_pid.is_some(),
        "selected_pid should be set after nav"
    );
}

/// I58-F1a + F1b: after a snapshot refresh, `selected_pid` still resolves to
/// the same pid even when the tracked process has moved to a different visual
/// row due to score changes. Uses divergent fixture data so cursor-fallback
/// and pid-key produce DIFFERENT answers — a broken pid-key would fail this.
#[test]
fn i58_selection_survives_snapshot_reorder() {
    let mut state = make_process_state(make_divergent_test_procs());
    state.sort_mode = SM::Cpu;
    // CPU order at row 0 = pid 100 "a" (90% cpu), row 1 = pid 300 "c" (40%),
    // row 2 = pid 200 "b" (5%). Cursor to row 1 → anchors pid 300.
    input::handle_key_event(make_key(KeyCode::Down), &mut state);
    let tracked_pid = state.selected_pid.expect("nav must anchor pid");
    assert_eq!(tracked_pid, 300, "row 1 under CPU sort = pid 300 'c'");

    // Simulate a snapshot refresh that mutates cpu_pct so the CPU-sort order
    // changes: pid 300 drops to row 2, pid 100 stays at row 0, pid 200 moves
    // to row 1. Cursor-fallback at row 1 would now return pid 200.
    for p in state.snapshot.processes.iter_mut() {
        if p.pid == 300 {
            p.cpu_pct = 1.0; // tanks to the bottom
        } else if p.pid == 200 {
            p.cpu_pct = 60.0; // rises above 300
        }
    }

    let (resolved_pid, _) = input::resolve_selected_process(&state)
        .expect("selection must still resolve after snapshot mutation");
    assert_eq!(
        resolved_pid, tracked_pid,
        "pid-key must track pid 300 to its new row 2, not fallback to cursor row 1 (which now = pid 200)"
    );
}

/// I58-F1a + F1b: sort-mode change is another kind of re-ordering — pid
/// selection MUST persist across it. Divergent fixture ensures the cursor row
/// under the new sort maps to a DIFFERENT pid, so cursor-fallback would fail.
#[test]
fn i58_selection_survives_sort_mode_change() {
    let mut state = make_process_state(make_divergent_test_procs());
    state.sort_mode = SM::Cpu;
    // CPU row 0 = pid 100 "a" (90% cpu). Cursor stays at Some(0) initially;
    // first `↓` moves to row 1 = pid 300 "c" (semantics preserved from I44-F5a).
    // But we want row 0. Use `Up` from initial None to stay at 0? Actually
    // `Up` with None cursor uses `unwrap_or(0).saturating_sub(1)` = 0. That
    // resolves row 0 = pid 100.
    input::handle_key_event(make_key(KeyCode::Up), &mut state);
    let tracked_pid = state.selected_pid.expect("nav must anchor pid");
    assert_eq!(tracked_pid, 100, "row 0 under CPU sort = pid 100 'a'");

    // Cycle sort: WeightedScore → Cpu → Memory. Two 's' presses from Cpu
    // land on Memory. Under Memory sort, pid 200 is at row 0, pid 300 at
    // row 1, pid 100 (our tracked pid) at row 2. Cursor is still 0, so
    // cursor-fallback would return pid 200 — WRONG.
    input::handle_key_event(make_key(KeyCode::Char('s')), &mut state); // Cpu → Memory
    assert_eq!(state.sort_mode, SM::Memory);

    let (resolved_pid, _) = input::resolve_selected_process(&state)
        .expect("selection must still resolve after sort-mode change");
    assert_eq!(
        resolved_pid, tracked_pid,
        "pid-key must track pid 100 to its new row under Memory sort, not fallback to cursor row 0 (pid 200)"
    );
}

/// I58-F1f: after a snapshot reorder, pressing `t` produces a SIGTERM
/// `pending_signal` targeting the originally-tracked pid — NOT a neighbor
/// that happens to now occupy the same cursor row. This is the end-to-end
/// bug-fix guarantee (highlight-agrees-with-kill after score jitter).
#[test]
fn i58_sigterm_targets_tracked_pid_after_reorder() {
    let mut state = make_process_state(make_divergent_test_procs());
    state.sort_mode = SM::Cpu;
    // Cursor to row 1 = pid 300 "c".
    input::handle_key_event(make_key(KeyCode::Down), &mut state);
    let tracked_pid = state.selected_pid.expect("nav must anchor pid");
    assert_eq!(tracked_pid, 300);

    // Snapshot refresh: pid 300 drops out of row 1, pid 200 takes its place.
    for p in state.snapshot.processes.iter_mut() {
        if p.pid == 300 {
            p.cpu_pct = 1.0;
        } else if p.pid == 200 {
            p.cpu_pct = 60.0;
        }
    }

    // User presses `t`.
    input::handle_key_event(make_key(KeyCode::Char('t')), &mut state);

    let (signal_pid, _name, signal) = state
        .pending_signal
        .expect("pressing t must construct a pending_signal");
    assert_eq!(signal, libc::SIGTERM);
    assert_eq!(
        signal_pid, tracked_pid,
        "SIGTERM target must be the tracked pid 300, not the neighbor pid 200 that took row 1"
    );
}

/// I59-F1a + F1b: when the tracked pid disappears, neither the effective
/// selection nor either signal key may retarget the cursor neighbor.
#[test]
fn i59_missing_tracked_pid_disables_term_and_kill() {
    for key in [KeyCode::Char('t'), KeyCode::Char('k')] {
        let mut state = make_process_state(make_test_procs());
        state.sort_mode = SM::Pid; // order = pid 1, 2, 3
        input::handle_key_event(make_key(KeyCode::Down), &mut state); // cursor→1, pid=2
        assert_eq!(state.selected_pid, Some(2));

        // Simulate pid 2 exiting between ticks. Cursor row 1 now holds pid 3,
        // but the user never selected pid 3.
        state.snapshot.processes.retain(|p| p.pid != 2);

        let indices = helpers::sorted_filtered_indices(
            &state.snapshot.processes,
            state.sort_mode,
            state.process_filter.as_deref(),
        );
        assert_eq!(
            helpers::effective_selection_row(
                &state.snapshot.processes,
                &indices,
                state.selected_pid,
                state.process_selected,
            ),
            None,
            "the renderer-facing helper must expose no highlighted row"
        );
        assert!(input::resolve_selected_process(&state).is_none());
        input::handle_key_event(make_key(key), &mut state);
        assert!(
            state.pending_signal.is_none(),
            "a missing tracked pid must not retarget the cursor neighbor"
        );
    }
}

/// I58-F1e: `Esc` from expanded panel resets BOTH `process_selected` and
/// `selected_pid` — a stale pid must not survive panel close.
#[test]
fn i58_esc_from_expanded_resets_selected_pid() {
    let mut state = make_process_state(make_test_procs());
    input::handle_key_event(make_key(KeyCode::Down), &mut state);
    assert!(state.selected_pid.is_some());

    input::handle_key_event(make_key(KeyCode::Esc), &mut state);
    assert_eq!(state.expanded_panel, None);
    assert_eq!(state.process_selected, None);
    assert_eq!(
        state.selected_pid, None,
        "Esc close must clear selected_pid too"
    );
}

/// I58-F1e: `e` collapse resets both cursor and pid.
#[test]
fn i58_e_key_resets_selected_pid() {
    let mut state = make_process_state(make_test_procs());
    input::handle_key_event(make_key(KeyCode::Down), &mut state);
    assert!(state.selected_pid.is_some());

    input::handle_key_event(make_key(KeyCode::Char('e')), &mut state);
    assert_eq!(state.selected_pid, None);
}

/// I58-F1e: entering filter mode (`f`) resets both, since row 0 of the
/// (about-to-be-filtered) view is a fresh anchor.
#[test]
fn i58_filter_enter_resets_selected_pid() {
    let mut state = make_process_state(make_test_procs());
    input::handle_key_event(make_key(KeyCode::Down), &mut state);
    assert!(state.selected_pid.is_some());

    input::handle_key_event(make_key(KeyCode::Char('f')), &mut state);
    assert_eq!(state.process_filter.as_deref(), Some(""));
    assert_eq!(state.process_selected, Some(0));
    assert_eq!(
        state.selected_pid, None,
        "entering filter mode must clear selected_pid"
    );
}

/// I58-F1b + F1f: `resolve_selected_process` returns None when there is no
/// selection (fresh panel open, user hasn't navigated).
#[test]
fn i58_resolve_none_before_first_nav() {
    let state = make_process_state(make_test_procs());
    assert_eq!(state.process_selected, None);
    assert_eq!(state.selected_pid, None);
    assert!(input::resolve_selected_process(&state).is_none());
}

/// I59-F1a: a filter that excludes an anchored pid produces no effective
/// selection rather than retargeting the only visible row.
#[test]
fn i59_selection_clears_when_filter_hides_pid() {
    let mut state = make_process_state(make_test_procs());
    state.sort_mode = SM::Pid;
    input::handle_key_event(make_key(KeyCode::Down), &mut state); // pid 2 = "beta"
    assert_eq!(state.selected_pid, Some(2));

    // Filter to names starting with "g" — only "gamma" (pid 3) remains.
    state.process_filter = Some("gamma".to_string());

    assert!(input::resolve_selected_process(&state).is_none());
}

/// I59-F1a: cursor fallback remains available when no pid has been anchored.
#[test]
fn i59_unanchored_cursor_still_resolves() {
    let mut state = make_process_state(make_test_procs());
    state.sort_mode = SM::Pid;
    state.process_selected = Some(1);
    state.selected_pid = None;

    let (resolved_pid, _) =
        input::resolve_selected_process(&state).expect("unanchored cursor should resolve");
    assert_eq!(resolved_pid, 2);
}

/// I59-F1c: explicit navigation after pid disappearance anchors a new target.
#[test]
fn i59_navigation_restores_selection_after_pid_disappears() {
    let mut state = make_process_state(make_test_procs());
    state.sort_mode = SM::Pid;
    input::handle_key_event(make_key(KeyCode::Down), &mut state); // cursor→1, pid=2
    state.snapshot.processes.retain(|p| p.pid != 2);
    assert!(input::resolve_selected_process(&state).is_none());

    input::handle_key_event(make_key(KeyCode::Up), &mut state); // cursor→0, pid=1
    assert_eq!(state.selected_pid, Some(1));
    input::handle_key_event(make_key(KeyCode::Char('t')), &mut state);
    assert_eq!(
        state.pending_signal.map(|(pid, _, signal)| (pid, signal)),
        Some((1, libc::SIGTERM))
    );
}

/// I59-F1c: boundary navigation after list shrink clamps to a visible row and
/// immediately anchors its pid rather than re-enabling cursor-only targeting.
#[test]
fn i59_boundary_navigation_clamps_and_anchors_pid() {
    let mut state = make_process_state(make_test_procs());
    state.sort_mode = SM::Pid;
    input::handle_key_event(make_key(KeyCode::Down), &mut state); // row 1, pid 2
    input::handle_key_event(make_key(KeyCode::Down), &mut state); // row 2, pid 3
    assert_eq!(state.selected_pid, Some(3));

    state.snapshot.processes.retain(|p| p.pid != 3);
    input::handle_key_event(make_key(KeyCode::Down), &mut state);

    assert_eq!(state.process_selected, Some(1));
    assert_eq!(state.selected_pid, Some(2));
    input::handle_key_event(make_key(KeyCode::Char('k')), &mut state);
    assert_eq!(
        state.pending_signal.map(|(pid, _, signal)| (pid, signal)),
        Some((2, libc::SIGKILL))
    );
}

/// I59-F1c: navigation on an empty view cannot create a cursor fallback when
/// the list later repopulates; a new non-empty navigation is required.
#[test]
fn i59_empty_view_navigation_stays_fail_closed_after_repopulation() {
    let mut state = make_process_state(make_test_procs());
    state.sort_mode = SM::Pid;
    input::handle_key_event(make_key(KeyCode::Down), &mut state); // row 1, pid 2
    state.snapshot.processes.clear();

    input::handle_key_event(make_key(KeyCode::Down), &mut state);
    assert_eq!(state.process_selected, None);
    assert_eq!(state.selected_pid, Some(2));

    state.snapshot.processes = vec![ProcessInfo {
        pid: 4,
        name: "delta".to_string(),
        user: "u".to_string(),
        ..Default::default()
    }];
    assert!(input::resolve_selected_process(&state).is_none());

    input::handle_key_event(make_key(KeyCode::Up), &mut state);
    assert_eq!(state.process_selected, Some(0));
    assert_eq!(state.selected_pid, Some(4));
}

/// I59-F1c + I58-F1c: `j` remains navigation, rather than filter text, while
/// filter-input mode is active and anchors a replacement after pid loss.
#[test]
fn i59_filter_input_j_navigation_anchors_visible_pid() {
    let mut state = make_process_state(make_test_procs());
    state.sort_mode = SM::Pid;
    input::handle_key_event(make_key(KeyCode::Down), &mut state); // row 1, pid 2
    state.process_filter = Some(String::new());
    state.snapshot.processes.retain(|p| p.pid != 2);

    input::handle_key_event(make_key(KeyCode::Char('j')), &mut state);

    assert_eq!(state.process_filter.as_deref(), Some(""));
    assert_eq!(state.process_selected, Some(1));
    assert_eq!(state.selected_pid, Some(3));
}

/// I59-F1c + I45-F5b: after filter-mode navigation restores a selection,
/// printable action keys remain filter text and cannot queue a signal.
#[test]
fn i59_filter_input_action_keys_remain_text_after_navigation() {
    for key in ['t', 'k'] {
        let mut state = make_process_state(make_test_procs());
        state.sort_mode = SM::Pid;
        input::handle_key_event(make_key(KeyCode::Down), &mut state); // row 1, pid 2
        state.process_filter = Some(String::new());
        state.snapshot.processes.retain(|p| p.pid != 2);

        input::handle_key_event(make_key(KeyCode::Char('j')), &mut state);
        assert_eq!(state.selected_pid, Some(3));

        input::handle_key_event(make_key(KeyCode::Char(key)), &mut state);

        let expected = key.to_string();
        assert_eq!(state.process_filter.as_deref(), Some(expected.as_str()));
        assert_eq!(state.selected_pid, None);
        assert_eq!(state.pending_signal, None);
    }
}

/// I59-F1b: a confirmation queued for a process that has disappeared is
/// cancelled for both TERM and KILL. The child is owned by this test so a
/// regression cannot signal an unrelated process.
#[test]
fn i59_confirmation_revalidates_missing_pid() {
    use std::process::Command;
    use std::thread;
    use std::time::Duration;

    for key in [KeyCode::Char('t'), KeyCode::Char('k')] {
        let mut child = Command::new("/bin/sleep")
            .arg("30")
            .spawn()
            .expect("spawn owned signal-test child");
        let pid = child.id() as i32;
        let mut state = make_process_state(vec![ProcessInfo {
            pid,
            name: "mtop-i59-signal-test".to_string(),
            user: "u".to_string(),
            ..Default::default()
        }]);
        state.process_selected = Some(0);
        state.selected_pid = Some(pid);

        input::handle_key_event(make_key(key), &mut state);
        assert!(state.pending_signal.is_some());
        state.snapshot.processes.clear();
        assert!(!input::pending_signal_is_current(
            &state,
            pid,
            "mtop-i59-signal-test"
        ));

        input::handle_key_event(make_key(KeyCode::Char('y')), &mut state);
        thread::sleep(Duration::from_millis(50));
        let survived = child.try_wait().expect("query signal-test child").is_none();
        let _ = child.kill();
        let _ = child.wait();
        assert!(survived, "stale confirmation must not signal the child");
    }
}

/// I59-F1b: an invalidated confirmation is not presented as an actionable
/// target after the selected pid disappears.
#[test]
fn i59_stale_confirmation_is_not_rendered() {
    let mut state = make_process_state(make_test_procs());
    state.sort_mode = SM::Pid;
    state.process_selected = Some(0);
    state.selected_pid = Some(1);
    input::handle_key_event(make_key(KeyCode::Char('t')), &mut state);

    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| draw_dashboard(f, &state)).unwrap();
    assert!(buffer_text(&terminal).contains("send SIGTERM to alpha (1)?"));

    state.snapshot.processes.retain(|p| p.pid != 1);
    terminal.draw(|f| draw_dashboard(f, &state)).unwrap();
    let text = buffer_text(&terminal);
    assert!(!text.contains("send SIGTERM"));
    assert!(text.contains("[t] term"));
}

/// I59-F1b: disappearance permanently cancels the confirmation, so later
/// reuse of the same pid and name cannot revive or deliver it.
#[test]
fn i59_confirmation_stays_cancelled_after_pid_name_reuse() {
    use std::process::Command;
    use std::thread;
    use std::time::Duration;

    for key in [KeyCode::Char('t'), KeyCode::Char('k')] {
        let mut child = Command::new("/bin/sleep")
            .arg("30")
            .spawn()
            .expect("spawn owned reuse-test child");
        let pid = child.id() as i32;
        let proc = ProcessInfo {
            pid,
            name: "mtop-i59-reuse-test".to_string(),
            user: "u".to_string(),
            ..Default::default()
        };
        let mut state = make_process_state(vec![proc.clone()]);
        state.process_selected = Some(0);
        state.selected_pid = Some(pid);
        input::handle_key_event(make_key(key), &mut state);
        assert!(state.pending_signal.is_some());

        state.snapshot.processes.clear();
        input::invalidate_stale_pending_signal(&mut state);
        assert!(state.pending_signal.is_none());
        state.snapshot.processes.push(proc);
        input::handle_key_event(make_key(KeyCode::Char('y')), &mut state);

        thread::sleep(Duration::from_millis(50));
        let survived = child.try_wait().expect("query reuse-test child").is_none();
        let _ = child.kill();
        let _ = child.wait();
        assert!(survived, "pid/name reuse must not revive a stale action");
    }
}

/// I59-F1b + F1c: once a hidden stale modal is invalid, the first navigation
/// key cancels it and anchors a replacement in the same dispatch.
#[test]
fn i59_first_navigation_after_stale_confirmation_recovers_selection() {
    let mut state = make_process_state(make_test_procs());
    state.sort_mode = SM::Pid;
    input::handle_key_event(make_key(KeyCode::Down), &mut state); // row 1, pid 2
    input::handle_key_event(make_key(KeyCode::Char('t')), &mut state);
    assert!(state.pending_signal.is_some());
    state.snapshot.processes.retain(|p| p.pid != 2);

    input::handle_key_event(make_key(KeyCode::Up), &mut state);

    assert!(state.pending_signal.is_none());
    assert_eq!(state.process_selected, Some(0));
    assert_eq!(state.selected_pid, Some(1));
}
