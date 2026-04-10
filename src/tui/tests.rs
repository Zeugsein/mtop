use super::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

// =========================================================================
// W0: AppState Default
// =========================================================================

#[test]
fn appstate_default_has_sensible_values() {
    let state = AppState::default();
    assert_eq!(state.interval_ms, 1000);
    assert_eq!(state.process_scroll, 0);
    assert_eq!(state.theme_idx, 0);
    assert_eq!(state.selected_panel, PanelId::Cpu);
    assert_eq!(state.expanded_panel, None);
    assert_eq!(state.sort_mode, SortMode::default());
    assert_eq!(state.temp_unit, "celsius");
}

// =========================================================================
// W1: Extracted pure logic (prepare.rs)
// =========================================================================

use crate::metrics::{ProcessInfo, NetInterface, MemoryMetrics, PowerMetrics, ThermalMetrics, SortMode as SM};
use super::prepare::*;

fn make_test_procs() -> Vec<ProcessInfo> {
    vec![
        ProcessInfo {
            pid: 1, name: "alpha".to_string(), cpu_pct: 10.0, mem_bytes: 100 * 1024 * 1024,
            power_w: 1.0, user: "root".to_string(), ..Default::default()
        },
        ProcessInfo {
            pid: 2, name: "beta".to_string(), cpu_pct: 50.0, mem_bytes: 2u64 * 1024 * 1024 * 1024,
            power_w: 5.0, user: "lume".to_string(), ..Default::default()
        },
        ProcessInfo {
            pid: 3, name: "gamma".to_string(), cpu_pct: 30.0, mem_bytes: 500 * 1024 * 1024,
            power_w: 3.0, user: "lume".to_string(), ..Default::default()
        },
    ]
}

#[test]
fn prepare_process_rows_sort_by_cpu() {
    let procs = make_test_procs();
    let rows = prepare_process_rows(&procs, SM::Cpu, 0, 10, 50.0, 2 * 1024 * 1024 * 1024, 5.0);
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].name, "beta");  // 50% CPU
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
    assert_eq!(rows[0].name, "beta");  // 2 GB
    assert_eq!(rows[1].name, "gamma"); // 500 MB
    assert_eq!(rows[2].name, "alpha"); // 100 MB
}

#[test]
fn prepare_process_rows_sort_by_power() {
    let procs = make_test_procs();
    let rows = prepare_process_rows(&procs, SM::Power, 0, 10, 50.0, 2 * 1024 * 1024 * 1024, 5.0);
    assert_eq!(rows[0].name, "beta");  // 5W
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
    assert!(rows[1].mem_display.contains("G"), "2 GB should display as G: {}", rows[1].mem_display);
    // alpha has 100 MB
    assert!(rows[0].mem_display.contains("M"), "100 MB should display as M: {}", rows[0].mem_display);
}

#[test]
fn prepare_process_rows_cpu_norm() {
    let procs = make_test_procs();
    let rows = prepare_process_rows(&procs, SM::Cpu, 0, 10, 50.0, 2 * 1024 * 1024 * 1024, 5.0);
    assert!((rows[0].cpu_norm - 1.0).abs() < 0.01, "beta (50/50) should be ~1.0");
    assert!((rows[2].cpu_norm - 0.2).abs() < 0.01, "alpha (10/50) should be ~0.2");
}

#[test]
fn prepare_network_rows_filters_infrastructure() {
    let ifaces = vec![
        NetInterface { name: "en0".to_string(), iface_type: "Ethernet".to_string(), rx_bytes_sec: 100.0, tx_bytes_sec: 200.0, ..Default::default() },
        NetInterface { name: "bridge0".to_string(), iface_type: "Bridge".to_string(), rx_bytes_sec: 10.0, tx_bytes_sec: 20.0, ..Default::default() },
        NetInterface { name: "awdl0".to_string(), iface_type: "AirDrop".to_string(), rx_bytes_sec: 5.0, tx_bytes_sec: 5.0, ..Default::default() },
        NetInterface { name: "en1".to_string(), iface_type: "Wi-Fi".to_string(), rx_bytes_sec: 500.0, tx_bytes_sec: 600.0, ..Default::default() },
    ];
    let rows = prepare_network_rows(&ifaces);
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|r| r.name == "en0" || r.name == "en1"));
}

#[test]
fn prepare_network_rows_sorted_by_total_traffic() {
    let ifaces = vec![
        NetInterface { name: "en0".to_string(), rx_bytes_sec: 100.0, tx_bytes_sec: 200.0, ..Default::default() },
        NetInterface { name: "en1".to_string(), rx_bytes_sec: 500.0, tx_bytes_sec: 600.0, ..Default::default() },
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
    assert!((p.wired_frac - 0.25).abs() < 0.01, "wired 4/16 = 0.25: {}", p.wired_frac);
    assert!((p.app_frac - 0.375).abs() < 0.01, "app 6/16 = 0.375: {}", p.app_frac);
    assert!((p.compressed_frac - 0.125).abs() < 0.01, "compressed 2/16 = 0.125: {}", p.compressed_frac);
    assert!(p.wired_frac + p.app_frac + p.compressed_frac <= 1.0);
}

#[test]
fn prepare_memory_pressure_zero_total() {
    let mem = MemoryMetrics { wired: 1024, app: 2048, compressed: 512, ..Default::default() };
    let p = prepare_memory_pressure(&mem, 0.0);
    // Should clamp to 1.0 max, not panic
    assert!(p.wired_frac <= 1.0);
    assert!(p.app_frac <= 1.0);
    assert!(p.compressed_frac <= 1.0);
}

#[test]
fn prepare_power_components_has_six_entries() {
    let power = PowerMetrics { cpu_w: 5.0, gpu_w: 3.0, ane_w: 0.5, dram_w: 1.0, system_w: 2.0, package_w: 10.0, available: true };
    let thermal = ThermalMetrics { fan_speeds: vec![2000, 3000], ..Default::default() };
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
fn expanded_cpu_80x24() { render_expanded_panel_at_size(PanelId::Cpu, 80, 24); }
#[test]
fn expanded_gpu_80x24() { render_expanded_panel_at_size(PanelId::Gpu, 80, 24); }
#[test]
fn expanded_memdisk_80x24() { render_expanded_panel_at_size(PanelId::MemDisk, 80, 24); }
#[test]
fn expanded_network_80x24() { render_expanded_panel_at_size(PanelId::Network, 80, 24); }
#[test]
fn expanded_power_80x24() { render_expanded_panel_at_size(PanelId::Power, 80, 24); }
#[test]
fn expanded_process_80x24() { render_expanded_panel_at_size(PanelId::Process, 80, 24); }

#[test]
fn expanded_cpu_120x40() { render_expanded_panel_at_size(PanelId::Cpu, 120, 40); }
#[test]
fn expanded_gpu_120x40() { render_expanded_panel_at_size(PanelId::Gpu, 120, 40); }
#[test]
fn expanded_memdisk_120x40() { render_expanded_panel_at_size(PanelId::MemDisk, 120, 40); }
#[test]
fn expanded_network_120x40() { render_expanded_panel_at_size(PanelId::Network, 120, 40); }
#[test]
fn expanded_power_120x40() { render_expanded_panel_at_size(PanelId::Power, 120, 40); }
#[test]
fn expanded_process_120x40() { render_expanded_panel_at_size(PanelId::Process, 120, 40); }

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
    assert!(text.contains("CPU"), "Dashboard should contain 'CPU' text");
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
    assert!(text.contains("Network"), "Dashboard should contain 'Network' text");
}

#[test]
fn dashboard_contains_process_text() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    let state = AppState::default();
    terminal.draw(|f| draw_dashboard(f, &state)).unwrap();
    let text = buffer_text(&terminal);
    assert!(text.contains("Process"), "Dashboard should contain 'Process' text");
}

#[test]
fn dashboard_contains_footer() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    let state = AppState::default();
    terminal.draw(|f| draw_dashboard(f, &state)).unwrap();
    let text = buffer_text(&terminal);
    assert!(text.contains("q:quit"), "Dashboard should contain footer with q:quit");
}

#[test]
fn dashboard_contains_mtop_header() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    let state = AppState::default();
    terminal.draw(|f| draw_dashboard(f, &state)).unwrap();
    let text = buffer_text(&terminal);
    assert!(text.contains("mtop"), "Dashboard should contain 'mtop' in header");
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
    assert!(input::handle_key_event(make_key(KeyCode::Char('q')), &mut state));
}

#[test]
fn key_ctrl_c_quits() {
    let mut state = AppState::default();
    assert!(input::handle_key_event(make_key_ctrl(KeyCode::Char('c')), &mut state));
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
fn key_1_selects_cpu() {
    let mut state = AppState::default();
    state.selected_panel = PanelId::Network;
    input::handle_key_event(make_key(KeyCode::Char('1')), &mut state);
    assert_eq!(state.selected_panel, PanelId::Cpu);
}

#[test]
fn key_2_selects_gpu() {
    let mut state = AppState::default();
    input::handle_key_event(make_key(KeyCode::Char('2')), &mut state);
    assert_eq!(state.selected_panel, PanelId::Gpu);
}

#[test]
fn key_3_selects_memdisk() {
    let mut state = AppState::default();
    input::handle_key_event(make_key(KeyCode::Char('3')), &mut state);
    assert_eq!(state.selected_panel, PanelId::MemDisk);
}

#[test]
fn key_4_selects_network() {
    let mut state = AppState::default();
    input::handle_key_event(make_key(KeyCode::Char('4')), &mut state);
    assert_eq!(state.selected_panel, PanelId::Network);
}

#[test]
fn key_5_selects_power() {
    let mut state = AppState::default();
    input::handle_key_event(make_key(KeyCode::Char('5')), &mut state);
    assert_eq!(state.selected_panel, PanelId::Power);
}

#[test]
fn key_6_selects_process() {
    let mut state = AppState::default();
    input::handle_key_event(make_key(KeyCode::Char('6')), &mut state);
    assert_eq!(state.selected_panel, PanelId::Process);
}

#[test]
fn key_e_toggles_expand() {
    let mut state = AppState::default();
    state.selected_panel = PanelId::Gpu;
    input::handle_key_event(make_key(KeyCode::Char('e')), &mut state);
    assert_eq!(state.expanded_panel, Some(PanelId::Gpu));
    // Toggle again to collapse
    input::handle_key_event(make_key(KeyCode::Char('e')), &mut state);
    assert_eq!(state.expanded_panel, None);
}

#[test]
fn key_plus_increases_interval() {
    let mut state = AppState::default();
    assert_eq!(state.interval_ms, 1000);
    input::handle_key_event(make_key(KeyCode::Char('+')), &mut state);
    assert_eq!(state.interval_ms, 1250);
}

#[test]
fn key_plus_caps_at_10000() {
    let mut state = AppState::default();
    state.interval_ms = 9900;
    input::handle_key_event(make_key(KeyCode::Char('+')), &mut state);
    assert_eq!(state.interval_ms, 10000);
}

#[test]
fn key_minus_decreases_interval() {
    let mut state = AppState::default();
    input::handle_key_event(make_key(KeyCode::Char('-')), &mut state);
    assert_eq!(state.interval_ms, 750);
}

#[test]
fn key_minus_floors_at_100() {
    let mut state = AppState::default();
    state.interval_ms = 200;
    input::handle_key_event(make_key(KeyCode::Char('-')), &mut state);
    assert_eq!(state.interval_ms, 100);
}

#[test]
fn key_j_scrolls_down() {
    let mut state = AppState::default();
    input::handle_key_event(make_key(KeyCode::Char('j')), &mut state);
    assert_eq!(state.process_scroll, 1);
}

#[test]
fn key_k_scrolls_up() {
    let mut state = AppState::default();
    state.process_scroll = 5;
    input::handle_key_event(make_key(KeyCode::Char('k')), &mut state);
    assert_eq!(state.process_scroll, 4);
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
fn key_enter_toggles_expand() {
    let mut state = AppState::default();
    input::handle_key_event(make_key(KeyCode::Enter), &mut state);
    assert_eq!(state.expanded_panel, Some(PanelId::Cpu));
}

#[test]
fn unknown_key_does_nothing() {
    let mut state = AppState::default();
    let quit = input::handle_key_event(make_key(KeyCode::Char('z')), &mut state);
    assert!(!quit);
    assert_eq!(state.theme_idx, 0);
    assert_eq!(state.process_scroll, 0);
}
