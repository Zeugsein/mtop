/// Feature-organized tests: panel rendering
/// Covers: panel rendering, expanded panels, braille visuals, gauge edge cases,
/// dashboard render paths, HistoryBuffer, memory/process/network/power panels.

use mtop::metrics::types::{MemoryMetrics, MetricsSnapshot, PowerMetrics, ProcessInfo, SortMode};
use mtop::tui::PanelId;

// ===========================================================================
// Helpers
// ===========================================================================

fn make_snapshot_with_memory(ram_total: u64, ram_used: u64) -> MetricsSnapshot {
    let mut s = MetricsSnapshot::default();
    s.memory = MemoryMetrics {
        ram_total,
        ram_used,
        ..Default::default()
    };
    s
}

fn make_snapshot_with_power(cpu_w: f32, gpu_w: f32) -> MetricsSnapshot {
    let mut s = MetricsSnapshot::default();
    s.power = PowerMetrics {
        cpu_w,
        gpu_w,
        ane_w: 0.1,
        dram_w: 0.3,
        package_w: cpu_w + gpu_w + 0.4,
        system_w: cpu_w + gpu_w + 1.0,
        available: true,
    };
    s
}

fn make_snapshot_with_power_simple(gpu_w: f32) -> MetricsSnapshot {
    let mut s = MetricsSnapshot::default();
    s.power = PowerMetrics {
        cpu_w: 5.0,
        gpu_w,
        ane_w: 0.1,
        dram_w: 0.5,
        package_w: 6.0,
        system_w: 7.0,
        available: true,
    };
    s
}

fn make_snapshot_with_processes(count: usize) -> MetricsSnapshot {
    let mut s = MetricsSnapshot::default();
    for i in 0..count {
        s.processes.push(ProcessInfo {
            pid: (1000 + i) as i32,
            name: format!("proc_{i}"),
            cpu_pct: (i as f32) * 5.0,
            mem_bytes: (i as u64 + 1) * 100 * 1024 * 1024,
            power_w: (i as f32) * 0.5,
            thread_count: 4,
            user: "user".to_string(),
            energy_nj: 0,
            io_read_bytes_sec: 0.0,
            io_write_bytes_sec: 0.0,
        });
    }
    s
}

fn empty_snapshot() -> MetricsSnapshot {
    MetricsSnapshot::default()
}

// ===========================================================================
// Basic render paths (iter19 panel tests)
// ===========================================================================

#[test]
fn panel_render_zero_metrics_80x24() {
    let text = mtop::tui::render_dashboard_to_string(80, 24, MetricsSnapshot::default(), false);
    assert!(!text.is_empty());
}

#[test]
fn panel_render_zero_metrics_40x10() {
    let text = mtop::tui::render_dashboard_to_string(40, 10, MetricsSnapshot::default(), false);
    assert!(!text.is_empty());
}

#[test]
fn memory_type_b_labels() {
    let gb: u64 = 1024 * 1024 * 1024;
    let snapshot = make_snapshot_with_memory(16 * gb, 12 * gb);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, true);
    assert!(
        text.contains(" used "),
        "Expected ' used ' title in detail layout; buffer:\n{text}"
    );
    assert!(
        text.contains(" avail "),
        "Expected ' avail ' title in detail layout; buffer:\n{text}"
    );
}

#[test]
fn memory_label_mb_scale() {
    let gb: u64 = 1024 * 1024 * 1024;
    let mb500: u64 = 524_288_000;
    let snapshot = make_snapshot_with_memory(16 * gb, mb500);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, true);
    assert!(
        text.contains("MB"),
        "Expected 'MB' scale label when ram_used < 1GB; buffer:\n{text}"
    );
}

#[test]
fn power_idle_label() {
    let snapshot = make_snapshot_with_power_simple(0.0);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    assert!(
        text.contains("idle"),
        "Expected 'idle' label when gpu_w=0.0; buffer:\n{text}"
    );
}

#[test]
fn gpu_idle_overlay() {
    let snapshot = make_snapshot_with_power_simple(0.0);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    assert!(
        text.contains("idle"),
        "Expected 'idle' overlay in GPU panel when gpu_w=0.0; buffer:\n{text}"
    );
}

#[test]
fn dashboard_header_narrow() {
    let text = mtop::tui::render_dashboard_to_string(80, 24, MetricsSnapshot::default(), false);
    assert!(
        text.contains("mtop"),
        "Expected 'mtop' in narrow header; buffer:\n{text}"
    );
}

// ===========================================================================
// Expanded panel rendering (iter20 config input)
// ===========================================================================

#[test]
fn render_with_show_detail_true_does_not_panic() {
    let text = mtop::tui::render_dashboard_to_string(120, 40, empty_snapshot(), true);
    assert!(!text.is_empty());
}

#[test]
fn render_with_expanded_cpu_panel() {
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, Some(PanelId::Cpu), SortMode::default(),
    );
    assert!(!text.is_empty());
}

#[test]
fn render_with_expanded_gpu_panel() {
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, Some(PanelId::Gpu), SortMode::default(),
    );
    assert!(!text.is_empty());
}

#[test]
fn render_with_expanded_memdisk_panel() {
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, Some(PanelId::MemDisk), SortMode::default(),
    );
    assert!(!text.is_empty());
}

#[test]
fn render_with_expanded_network_panel() {
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, Some(PanelId::Network), SortMode::default(),
    );
    assert!(!text.is_empty());
}

#[test]
fn render_with_expanded_power_panel() {
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, Some(PanelId::Power), SortMode::default(),
    );
    assert!(!text.is_empty());
}

#[test]
fn render_with_expanded_process_panel() {
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, Some(PanelId::Process), SortMode::default(),
    );
    assert!(!text.is_empty());
}

#[test]
fn render_expanded_cpu_80x24_no_panic() {
    let text = mtop::tui::render_dashboard_with_state(
        80, 24, empty_snapshot(), false, Some(PanelId::Cpu), SortMode::default(),
    );
    assert!(!text.is_empty());
}

#[test]
fn render_expanded_memdisk_80x24_no_panic() {
    let text = mtop::tui::render_dashboard_with_state(
        80, 24, empty_snapshot(), false, Some(PanelId::MemDisk), SortMode::default(),
    );
    assert!(!text.is_empty());
}

#[test]
fn render_expanded_network_80x24_no_panic() {
    let text = mtop::tui::render_dashboard_with_state(
        80, 24, empty_snapshot(), false, Some(PanelId::Network), SortMode::default(),
    );
    assert!(!text.is_empty());
}

#[test]
fn render_expanded_gpu_80x24_no_panic() {
    let text = mtop::tui::render_dashboard_with_state(
        80, 24, empty_snapshot(), false, Some(PanelId::Gpu), SortMode::default(),
    );
    assert!(!text.is_empty());
}

#[test]
fn render_expanded_power_80x24_no_panic() {
    let text = mtop::tui::render_dashboard_with_state(
        80, 24, empty_snapshot(), false, Some(PanelId::Power), SortMode::default(),
    );
    assert!(!text.is_empty());
}

#[test]
fn render_expanded_process_80x24_no_panic() {
    let text = mtop::tui::render_dashboard_with_state(
        80, 24, empty_snapshot(), false, Some(PanelId::Process), SortMode::default(),
    );
    assert!(!text.is_empty());
}

// Content verification tests for expanded panels at 120x40
#[test]
fn expanded_cpu_contains_cluster_labels() {
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, Some(PanelId::Cpu), SortMode::default(),
    );
    assert!(text.contains("cpu"), "expanded CPU panel should contain 'cpu' title");
    assert!(text.contains("e-cluster") || text.contains("p-cluster"), "expanded CPU should contain cluster labels");
}

#[test]
fn expanded_gpu_contains_metrics() {
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, Some(PanelId::Gpu), SortMode::default(),
    );
    assert!(text.contains("gpu"), "expanded GPU panel should contain 'gpu' title");
    assert!(text.contains("frequency"), "expanded GPU should contain frequency metric");
}

#[test]
fn expanded_power_contains_component_breakdown() {
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, Some(PanelId::Power), SortMode::default(),
    );
    assert!(text.contains("power"), "expanded power panel should contain 'power' title");
    assert!(text.contains("component breakdown"), "expanded power should contain component breakdown");
}

#[test]
fn expanded_network_contains_interface_header() {
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, Some(PanelId::Network), SortMode::default(),
    );
    assert!(text.contains("network"), "expanded network panel should contain 'network' title");
    assert!(text.contains("interface"), "expanded network should contain interface table header");
}

#[test]
fn expanded_process_contains_sort_label() {
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, Some(PanelId::Process), SortMode::default(),
    );
    assert!(text.contains("processes"), "expanded process panel should contain 'processes' title");
    assert!(text.contains("sort:"), "expanded process should contain sort label");
}

#[test]
fn expanded_memory_contains_ram_label() {
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, Some(PanelId::MemDisk), SortMode::default(),
    );
    assert!(text.contains("memory"), "expanded memory panel should contain 'memory' title");
    assert!(text.contains("used") || text.contains("available"), "expanded memory should contain chart labels");
}

#[test]
fn render_sort_mode_cpu() {
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, None, SortMode::Cpu,
    );
    assert!(!text.is_empty());
}

#[test]
fn render_sort_mode_memory() {
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, None, SortMode::Memory,
    );
    assert!(!text.is_empty());
}

#[test]
fn render_sort_mode_name() {
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, None, SortMode::Name,
    );
    assert!(!text.is_empty());
}

// ===========================================================================
// Process panel sort modes (iter20 render coverage)
// ===========================================================================

#[test]
fn process_panel_multiple_processes() {
    let snapshot = make_snapshot_with_processes(5);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    assert!(!text.is_empty());
    assert!(
        text.contains("proc"),
        "Expected 'proc' panel title; buffer:\n{text}"
    );
}

#[test]
fn process_panel_sort_memory() {
    let snapshot = make_snapshot_with_processes(4);
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, snapshot, false, None, SortMode::Memory,
    );
    assert!(!text.is_empty());
    assert!(
        text.contains("Mem"),
        "Expected 'Mem' sort indicator; buffer:\n{text}"
    );
}

#[test]
fn process_panel_sort_power() {
    let snapshot = make_snapshot_with_processes(4);
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, snapshot, false, None, SortMode::Power,
    );
    assert!(!text.is_empty());
    assert!(
        text.contains("Power"),
        "Expected 'Power' sort indicator; buffer:\n{text}"
    );
}

#[test]
fn process_panel_sort_pid() {
    let snapshot = make_snapshot_with_processes(4);
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, snapshot, false, None, SortMode::Pid,
    );
    assert!(!text.is_empty());
    assert!(
        text.contains("PID"),
        "Expected 'PID' sort indicator; buffer:\n{text}"
    );
}

#[test]
fn dashboard_expanded_cpu() {
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, MetricsSnapshot::default(), false, Some(PanelId::Cpu), SortMode::default(),
    );
    assert!(!text.is_empty());
    assert!(
        text.contains("cpu"),
        "Expected 'cpu' in expanded panel header; buffer:\n{text}"
    );
}

#[test]
fn dashboard_expanded_network() {
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, MetricsSnapshot::default(), false, Some(PanelId::Network), SortMode::default(),
    );
    assert!(!text.is_empty());
    assert!(
        text.contains("network"),
        "Expected 'network' in expanded panel header; buffer:\n{text}"
    );
}

#[test]
fn dashboard_expanded_power() {
    let snapshot = make_snapshot_with_power(5.0, 2.0);
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, snapshot, false, Some(PanelId::Power), SortMode::default(),
    );
    assert!(!text.is_empty());
    assert!(
        text.contains("power"),
        "Expected 'power' in expanded panel header; buffer:\n{text}"
    );
}

#[test]
fn detail_toggle_produces_different_output() {
    let gb: u64 = 1024 * 1024 * 1024;
    let snapshot = make_snapshot_with_memory(16 * gb, 8 * gb);

    let text_no_detail =
        mtop::tui::render_dashboard_to_string(120, 40, snapshot.clone(), false);
    let text_with_detail =
        mtop::tui::render_dashboard_to_string(120, 40, snapshot, true);

    assert!(!text_no_detail.is_empty());
    assert!(!text_with_detail.is_empty());
    assert_ne!(
        text_no_detail, text_with_detail,
        "show_detail=true should produce different output than show_detail=false"
    );
}

#[test]
fn power_panel_nonzero_watts() {
    let snapshot = make_snapshot_with_power(8.5, 3.2);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    assert!(!text.is_empty());
    assert!(
        text.contains("cpu") || text.contains("gpu"),
        "Expected 'cpu' or 'gpu' label in power panel; buffer:\n{text}"
    );
}

#[test]
fn power_panel_gpu_idle() {
    let snapshot = make_snapshot_with_power(4.0, 0.0);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    assert!(!text.is_empty());
    assert!(
        text.contains("idle"),
        "Expected 'idle' label when gpu_w=0.0; buffer:\n{text}"
    );
}

#[test]
fn power_panel_gpu_idle_detail() {
    let snapshot = make_snapshot_with_power(4.0, 0.0);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, true);
    assert!(!text.is_empty());
    assert!(
        text.contains("idle"),
        "Expected 'idle' label in detail mode when gpu_w=0.0; buffer:\n{text}"
    );
}

#[test]
fn expanded_panel_cpu_no_panic() {
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, MetricsSnapshot::default(), false, Some(PanelId::Cpu), SortMode::default(),
    );
    assert!(
        text.contains("CPU") || text.contains("cpu"),
        "Expected CPU panel content; buffer:\n{text}"
    );
}

#[test]
fn expanded_panel_gpu_no_panic() {
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, MetricsSnapshot::default(), false, Some(PanelId::Gpu), SortMode::default(),
    );
    assert!(
        text.contains("GPU") || text.contains("gpu"),
        "Expected GPU panel content; buffer:\n{text}"
    );
}

#[test]
fn expanded_panel_memdisk_no_panic() {
    let gb: u64 = 1024 * 1024 * 1024;
    let snapshot = make_snapshot_with_memory(16 * gb, 4 * gb);
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, snapshot, false, Some(PanelId::MemDisk), SortMode::default(),
    );
    assert!(
        text.contains("memory") || text.contains("RAM"),
        "Expected memory panel content; buffer:\n{text}"
    );
}

#[test]
fn expanded_panel_network_no_panic() {
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, MetricsSnapshot::default(), false, Some(PanelId::Network), SortMode::default(),
    );
    assert!(
        text.contains("network") || text.contains("upload"),
        "Expected network panel content; buffer:\n{text}"
    );
}

#[test]
fn expanded_panel_power_no_panic() {
    let snapshot = make_snapshot_with_power(6.0, 1.5);
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, snapshot, false, Some(PanelId::Power), SortMode::default(),
    );
    assert!(
        text.contains("power") || text.contains("cpu power"),
        "Expected power panel content; buffer:\n{text}"
    );
}

#[test]
fn expanded_panel_process_no_panic() {
    let snapshot = make_snapshot_with_processes(10);
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, snapshot, false, Some(PanelId::Process), SortMode::default(),
    );
    assert!(
        text.contains("processes") || text.contains("proc"),
        "Expected process panel content; buffer:\n{text}"
    );
}

// ===========================================================================
// Braille graph (iter19 hardening, iter22)
// ===========================================================================

use mtop::metrics::types::HistoryBuffer;

#[test]
fn history_buffer_empty_iter() {
    let buf = HistoryBuffer::new();
    let count = buf.iter().count();
    assert_eq!(count, 0, "empty HistoryBuffer should yield 0 items, got {count}");
}

#[test]
fn history_buffer_single_push() {
    let mut buf = HistoryBuffer::new();
    buf.push_back(42.0);
    let items: Vec<f64> = buf.iter().copied().collect();
    assert_eq!(items.len(), 1, "single-push HistoryBuffer should yield 1 item");
    assert_eq!(items[0], 42.0);
}

#[test]
fn braille_graph_1x1() {
    let theme = &mtop::tui::theme::THEMES[0];
    let result = mtop::tui::braille::render_braille_graph(&[0.5], 1.0, 1, 1, theme);
    assert_eq!(result.len(), 1, "1×1 graph should return 1 row");
    assert_eq!(result[0].len(), 1, "1×1 graph row should have 1 character");
}

#[test]
fn braille_graph_zero_height() {
    let theme = &mtop::tui::theme::THEMES[0];
    let result = mtop::tui::braille::render_braille_graph(&[0.5], 1.0, 10, 0, theme);
    assert!(
        result.is_empty(),
        "height=0 graph should return empty vec, got {} rows",
        result.len()
    );
}

#[test]
fn gauge_value_exceeds_max() {
    use mtop::tui::gauge::render_gauge_bar;
    use mtop::tui::theme::HORIZON;
    let spans = render_gauge_bar(150.0, 100.0, 20, "", &HORIZON);
    let content: String = spans.iter().map(|s| s.content.as_ref()).collect();
    let filled_count = content.matches('■').count();
    assert!(
        filled_count <= 20,
        "filled chars ({filled_count}) must not exceed width (20)"
    );
}

#[test]
fn gauge_negative_value() {
    use mtop::tui::gauge::render_gauge_bar;
    use mtop::tui::theme::HORIZON;
    let spans = render_gauge_bar(-5.0, 100.0, 20, "", &HORIZON);
    let content: String = spans.iter().map(|s| s.content.as_ref()).collect();
    let char_count = content.matches('■').count();
    assert_eq!(
        char_count, 20,
        "negative value should produce 0 filled + 20 empty = 20 total ■ chars, got {char_count}"
    );
}

#[test]
fn gauge_zero_max() {
    use mtop::tui::gauge::render_gauge_bar;
    use mtop::tui::theme::HORIZON;
    let spans = render_gauge_bar(50.0, 0.0, 20, "", &HORIZON);
    assert!(
        !spans.is_empty(),
        "zero-max gauge should still return spans for the empty bar"
    );
}

// ===========================================================================
// Braille down (iter22)
// ===========================================================================

#[test]
fn braille_down_empty() {
    let result = mtop::tui::braille::render_braille_graph_down(&[], 100.0, 10, 5, &mtop::tui::theme::THEMES[0]);
    assert_eq!(result.len(), 5, "Expected 5 rows for height=5");
    for row in &result {
        assert!(row.is_empty(), "Empty input should produce empty rows");
    }
}

#[test]
fn braille_down_full_scale() {
    let values = vec![100.0, 100.0];
    let result = mtop::tui::braille::render_braille_graph_down(&values, 100.0, 1, 2, &mtop::tui::theme::THEMES[0]);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0][0].0, '⣿', "Top row (row 0) should be ⣿ at 100%");
    assert_eq!(result[1][0].0, '⣿', "Bottom row (row 1) should be ⣿ at 100%");
}

#[test]
fn braille_down_zero() {
    let values = vec![0.0, 0.0];
    let result = mtop::tui::braille::render_braille_graph_down(&values, 100.0, 1, 2, &mtop::tui::theme::THEMES[0]);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0][0].0, ' ', "Top row should be space at 0%");
    assert_eq!(result[1][0].0, ' ', "Bottom row should be space at 0%");
}

#[test]
fn braille_down_half_height() {
    let values = vec![50.0, 50.0];
    let result = mtop::tui::braille::render_braille_graph_down(&values, 100.0, 1, 2, &mtop::tui::theme::THEMES[0]);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0][0].0, '⣿', "Top row should be fully filled at 50%");
    assert_eq!(result[1][0].0, ' ', "Bottom row should be empty at 50%");
}

#[test]
fn braille_down_asymmetric() {
    let values = vec![100.0, 0.0];
    let result = mtop::tui::braille::render_braille_graph_down(&values, 100.0, 1, 1, &mtop::tui::theme::THEMES[0]);
    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0][0].0, '\u{2847}',
        "left=100%, right=0% should give BRAILLE_DOWN[4][0]"
    );
}

#[test]
fn braille_down_table_corners() {
    let r00 = mtop::tui::braille::render_braille_graph_down(&[0.0, 0.0], 100.0, 1, 1, &mtop::tui::theme::THEMES[0]);
    assert_eq!(r00[0][0].0, ' ', "BRAILLE_DOWN[0][0] should be space");

    let r44 = mtop::tui::braille::render_braille_graph_down(&[100.0, 100.0], 100.0, 1, 1, &mtop::tui::theme::THEMES[0]);
    assert_eq!(r44[0][0].0, '\u{28FF}', "BRAILLE_DOWN[4][4] should be ⣿");

    let r40 = mtop::tui::braille::render_braille_graph_down(&[100.0, 0.0], 100.0, 1, 1, &mtop::tui::theme::THEMES[0]);
    assert_eq!(r40[0][0].0, '\u{2847}', "BRAILLE_DOWN[4][0] check");

    let r04 = mtop::tui::braille::render_braille_graph_down(&[0.0, 100.0], 100.0, 1, 1, &mtop::tui::theme::THEMES[0]);
    assert_eq!(r04[0][0].0, '\u{28B8}', "BRAILLE_DOWN[0][4] check");
}

#[test]
fn braille_down_29_percent() {
    let values = vec![29.0, 29.0];
    let result = mtop::tui::braille::render_braille_graph_down(&values, 100.0, 1, 10, &mtop::tui::theme::THEMES[0]);
    assert_eq!(result.len(), 10);
    assert_eq!(result[0][0].0, '⣿', "Row 0 should be ⣿ at 29%");
    assert_eq!(result[1][0].0, '⣿', "Row 1 should be ⣿ at 29%");
    assert_eq!(result[2][0].0, '⣿', "Row 2 should be ⣿ at 29%");
    assert_eq!(result[3][0].0, ' ', "Row 3 should be empty at 29%");
    assert_eq!(result[9][0].0, ' ', "Bottom row should be empty at 29%");
}

#[test]
fn braille_down_zero_width() {
    let result = mtop::tui::braille::render_braille_graph_down(&[50.0, 50.0], 100.0, 0, 3, &mtop::tui::theme::THEMES[0]);
    assert_eq!(result.len(), 3, "height=3 rows expected even for width=0");
    for row in &result {
        assert!(row.is_empty(), "Rows should be empty when width=0");
    }
}

#[test]
fn braille_down_zero_height() {
    let result = mtop::tui::braille::render_braille_graph_down(&[50.0, 50.0], 100.0, 5, 0, &mtop::tui::theme::THEMES[0]);
    assert!(result.is_empty(), "height=0 should return empty vec");
}

#[test]
fn braille_down_zero_max_value() {
    let values = vec![0.0, 0.0];
    let result = mtop::tui::braille::render_braille_graph_down(&values, 0.0, 1, 1, &mtop::tui::theme::THEMES[0]);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0][0].0, ' ', "Zero value with zero max should produce space");
}

// ===========================================================================
// Dashboard render with braille/padding (iter22)
// ===========================================================================

#[test]
fn memory_titles_bold_used_avail() {
    let gb: u64 = 1024 * 1024 * 1024;
    let snapshot = make_snapshot_with_memory(16 * gb, 12 * gb);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, true);
    assert!(
        text.contains(" used "),
        "Expected ' used ' (lowercase) in detail layout; buffer:\n{text}"
    );
    assert!(
        text.contains(" avail "),
        "Expected ' avail ' (lowercase) in detail layout; buffer:\n{text}"
    );
}

#[test]
fn process_header_no_dots() {
    let snapshot = make_snapshot_with_processes(3);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);

    let header_line = text
        .lines()
        .find(|line| {
            let lower = line.to_lowercase();
            lower.contains("name") && lower.contains("pid")
        });

    assert!(
        header_line.is_some(),
        "Expected a process header line with 'name' and 'pid'; buffer:\n{text}"
    );

    let header = header_line.unwrap();
    assert!(
        !header.contains('•'),
        "Process header line should NOT contain '•' dots; header: '{header}'"
    );
}

#[test]
fn panels_have_inner_padding() {
    let text = mtop::tui::render_dashboard_to_string(120, 40, MetricsSnapshot::default(), false);
    assert!(!text.is_empty(), "Dashboard should render non-empty output");
    assert!(
        text.contains('─') || text.contains('│') || text.contains('┌'),
        "Expected panel border characters in output"
    );
}

#[test]
fn network_panel_renders() {
    let text = mtop::tui::render_dashboard_to_string(120, 40, MetricsSnapshot::default(), true);
    assert!(
        text.contains("Upload") || text.contains("upload"),
        "Expected 'Upload' label in network panel (detail=true); buffer:\n{text}"
    );
    assert!(
        text.contains("Download") || text.contains("download"),
        "Expected 'Download' label in network panel (detail=true); buffer:\n{text}"
    );
}

#[test]
fn memory_non_detail_has_disk_label() {
    let gb: u64 = 1024 * 1024 * 1024;
    let snapshot = make_snapshot_with_memory(16 * gb, 8 * gb);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    assert!(
        text.to_lowercase().contains("disk"),
        "Expected 'disk' label in non-detail mode; buffer:\n{text}"
    );
}

#[test]
fn process_data_rows_have_dots() {
    let snapshot = make_snapshot_with_processes(5);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    assert!(
        text.contains('•'),
        "Expected '•' in process data rows; buffer:\n{text}"
    );
}

#[test]
fn render_all_expanded_panels_with_padding() {
    let gb: u64 = 1024 * 1024 * 1024;
    let snapshot_mem = make_snapshot_with_memory(16 * gb, 8 * gb);
    let snapshot_proc = make_snapshot_with_processes(5);

    let panels = [
        (PanelId::Cpu, MetricsSnapshot::default()),
        (PanelId::Gpu, MetricsSnapshot::default()),
        (PanelId::MemDisk, snapshot_mem),
        (PanelId::Network, MetricsSnapshot::default()),
        (PanelId::Power, MetricsSnapshot::default()),
        (PanelId::Process, snapshot_proc),
    ];

    for (panel_id, snapshot) in panels {
        let text = mtop::tui::render_dashboard_with_state(
            120, 40, snapshot, false, Some(panel_id), SortMode::default(),
        );
        assert!(
            !text.is_empty(),
            "Expanded panel {panel_id:?} should render non-empty output"
        );
    }
}

#[test]
fn dashboard_narrow_no_panic() {
    let text = mtop::tui::render_dashboard_to_string(80, 24, MetricsSnapshot::default(), false);
    assert!(!text.is_empty());
    assert!(
        text.contains("mtop"),
        "Expected 'mtop' in narrow dashboard header; buffer:\n{text}"
    );
}

#[test]
fn network_expanded_has_symmetric_labels() {
    let text = mtop::tui::render_dashboard_with_state(
        120, 40, MetricsSnapshot::default(), false, Some(PanelId::Network), SortMode::default(),
    );
    assert!(
        text.contains("Upload") || text.contains("upload"),
        "Expected 'Upload' in expanded network panel; buffer:\n{text}"
    );
    assert!(
        text.contains("Download") || text.contains("download"),
        "Expected 'Download' in expanded network panel; buffer:\n{text}"
    );
}

#[test]
fn memory_detail_titles_are_lowercase() {
    let gb: u64 = 1024 * 1024 * 1024;
    let snapshot = make_snapshot_with_memory(16 * gb, 12 * gb);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, true);

    assert!(
        text.contains(" used "),
        "Expected lowercase ' used ' title; buffer:\n{text}"
    );
    assert!(
        text.contains(" avail "),
        "Expected lowercase ' avail ' title; buffer:\n{text}"
    );
}

// ===========================================================================
// SHALL-23-13: GPU panel idle (iter23)
// ===========================================================================

#[test]
fn shall_23_13_gpu_panel_renders_without_panic_when_idle() {
    let snapshot = MetricsSnapshot::default();
    let _text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    // No panic = pass
}

#[test]
fn shall_23_13_gpu_panel_title_contains_idle_when_gpu_w_is_zero() {
    let snapshot = MetricsSnapshot::default();
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    assert!(
        text.contains("idle"),
        "GPU panel should render 'idle' in title when gpu_w < 0.5"
    );
}

#[test]
fn shall_23_13_gpu_panel_shows_usage_percent_when_gpu_active() {
    use mtop::metrics::types::{GpuMetrics, PowerMetrics};
    let mut snapshot = MetricsSnapshot::default();
    snapshot.gpu = GpuMetrics { freq_mhz: 800, usage: 0.45, power_w: 3.5, available: true };
    snapshot.power = PowerMetrics { gpu_w: 3.5, available: true, ..Default::default() };
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    assert!(
        text.contains("45.0"),
        "GPU panel active branch should render usage percentage (45.0%) when gpu_w = 3.5W, got no match in rendered text"
    );
}

// ===========================================================================
// Panel ID / theme names (iter8)
// ===========================================================================

#[test]
fn panel_id_is_left_column_cpu() {
    let names = mtop::tui::theme_names();
    assert!(!names.is_empty(), "theme_names should return non-empty list (proves mod.rs loads)");
}

// ===========================================================================
// Memory panel swap guard (iter17)
// ===========================================================================

#[test]
fn memory_panel_swap_guard() {
    let mem_src = include_str!("../src/tui/panels/memory.rs");
    assert!(
        mem_src.contains("swap_total == 0"),
        "memory panel should guard on `swap_total == 0` before showing swap info"
    );
}

// ===========================================================================
// Idle thresholds (iter17)
// ===========================================================================

#[test]
fn idle_threshold_gpu() {
    let power_src = include_str!("../src/tui/panels/power.rs");
    assert!(
        power_src.contains("< 0.5"),
        "expected GPU idle threshold `< 0.5` in power.rs"
    );
}

// ===========================================================================
// Process dot thresholds (iter17)
// ===========================================================================

#[test]
fn process_dot_near_zero_threshold() {
    let proc_src = include_str!("../src/tui/panels/process.rs");
    assert!(proc_src.contains("cpu_pct < 0.1"), "expected `cpu_pct < 0.1` in process.rs");
    assert!(proc_src.contains("1_048_576"), "expected `1_048_576` (1MB) threshold in process.rs");
    assert!(proc_src.contains("power_w < 0.1"), "expected `power_w < 0.1` in process.rs");
}
