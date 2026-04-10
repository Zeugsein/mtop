/// Iteration 20: Extended render coverage tests.
/// Exercises process panel, expanded panel layout, detail toggle, power panel,
/// and all 6 expanded-panel paths using render_dashboard_to_string /
/// render_dashboard_with_state.

use mtop::metrics::types::{
    MemoryMetrics, MetricsSnapshot, PowerMetrics, ProcessInfo, SortMode,
};
use mtop::tui::PanelId;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

fn make_snapshot_with_memory(ram_total: u64, ram_used: u64) -> MetricsSnapshot {
    let mut s = MetricsSnapshot::default();
    s.memory = MemoryMetrics {
        ram_total,
        ram_used,
        ..Default::default()
    };
    s
}

// ---------------------------------------------------------------------------
// 1. Process panel: render at 120x40 with multiple processes
// ---------------------------------------------------------------------------

#[test]
fn process_panel_multiple_processes() {
    let snapshot = make_snapshot_with_processes(5);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    assert!(!text.is_empty());
    // The process panel title includes "proc"
    assert!(
        text.contains("proc"),
        "Expected 'proc' panel title; buffer:\n{text}"
    );
}

// ---------------------------------------------------------------------------
// 2. Process panel: sort mode = Memory
// ---------------------------------------------------------------------------

#[test]
fn process_panel_sort_memory() {
    let snapshot = make_snapshot_with_processes(4);
    let text = mtop::tui::render_dashboard_with_state(
        120,
        40,
        snapshot,
        false,
        None,
        SortMode::Memory,
    );
    assert!(!text.is_empty());
    assert!(
        text.contains("Mem"),
        "Expected 'Mem' sort indicator; buffer:\n{text}"
    );
}

// ---------------------------------------------------------------------------
// 3. Process panel: sort mode = Power
// ---------------------------------------------------------------------------

#[test]
fn process_panel_sort_power() {
    let snapshot = make_snapshot_with_processes(4);
    let text = mtop::tui::render_dashboard_with_state(
        120,
        40,
        snapshot,
        false,
        None,
        SortMode::Power,
    );
    assert!(!text.is_empty());
    assert!(
        text.contains("Power"),
        "Expected 'Power' sort indicator; buffer:\n{text}"
    );
}

// ---------------------------------------------------------------------------
// 4. Process panel: sort mode = Pid
// ---------------------------------------------------------------------------

#[test]
fn process_panel_sort_pid() {
    let snapshot = make_snapshot_with_processes(4);
    let text = mtop::tui::render_dashboard_with_state(
        120,
        40,
        snapshot,
        false,
        None,
        SortMode::Pid,
    );
    assert!(!text.is_empty());
    assert!(
        text.contains("PID"),
        "Expected 'PID' sort indicator; buffer:\n{text}"
    );
}

// ---------------------------------------------------------------------------
// 5. Dashboard expanded: CPU expanded (left column panel)
// ---------------------------------------------------------------------------

#[test]
fn dashboard_expanded_cpu() {
    let text = mtop::tui::render_dashboard_with_state(
        120,
        40,
        MetricsSnapshot::default(),
        false,
        Some(PanelId::Cpu),
        SortMode::default(),
    );
    assert!(!text.is_empty());
    assert!(
        text.contains("CPU"),
        "Expected 'CPU' in expanded panel header; buffer:\n{text}"
    );
}

// ---------------------------------------------------------------------------
// 6. Dashboard expanded: Network (right column panel)
// ---------------------------------------------------------------------------

#[test]
fn dashboard_expanded_network() {
    let text = mtop::tui::render_dashboard_with_state(
        120,
        40,
        MetricsSnapshot::default(),
        false,
        Some(PanelId::Network),
        SortMode::default(),
    );
    assert!(!text.is_empty());
    assert!(
        text.contains("Network"),
        "Expected 'Network' in expanded panel header; buffer:\n{text}"
    );
}

// ---------------------------------------------------------------------------
// 7. Dashboard expanded: Power (right column panel)
// ---------------------------------------------------------------------------

#[test]
fn dashboard_expanded_power() {
    let snapshot = make_snapshot_with_power(5.0, 2.0);
    let text = mtop::tui::render_dashboard_with_state(
        120,
        40,
        snapshot,
        false,
        Some(PanelId::Power),
        SortMode::default(),
    );
    assert!(!text.is_empty());
    assert!(
        text.contains("Power"),
        "Expected 'Power' in expanded panel header; buffer:\n{text}"
    );
}

// ---------------------------------------------------------------------------
// 8. Detail toggle: show_detail=true vs show_detail=false produce different output
// ---------------------------------------------------------------------------

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
    // detail mode changes the layout — the two renders must differ
    assert_ne!(
        text_no_detail, text_with_detail,
        "show_detail=true should produce different output than show_detail=false"
    );
}

// ---------------------------------------------------------------------------
// 9. Power panel: non-zero cpu_w and gpu_w — tests sparkline label paths
// ---------------------------------------------------------------------------

#[test]
fn power_panel_nonzero_watts() {
    let snapshot = make_snapshot_with_power(8.5, 3.2);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    assert!(!text.is_empty());
    // cpu and gpu labels should appear in the power panel
    assert!(
        text.contains("cpu") || text.contains("gpu"),
        "Expected 'cpu' or 'gpu' label in power panel; buffer:\n{text}"
    );
}

// ---------------------------------------------------------------------------
// 10. Power panel: gpu_w=0.0 — tests the idle path
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// 11. Power panel: gpu_w=0.0 with show_detail=true — sub-frame borders path
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// 12–17. Expanded panels: all 6 panels at 120x40 — no panics + expected text
// ---------------------------------------------------------------------------

#[test]
fn expanded_panel_cpu_no_panic() {
    let text = mtop::tui::render_dashboard_with_state(
        120,
        40,
        MetricsSnapshot::default(),
        false,
        Some(PanelId::Cpu),
        SortMode::default(),
    );
    assert!(
        text.contains("CPU") || text.contains("cpu"),
        "Expected CPU panel content; buffer:\n{text}"
    );
}

#[test]
fn expanded_panel_gpu_no_panic() {
    let text = mtop::tui::render_dashboard_with_state(
        120,
        40,
        MetricsSnapshot::default(),
        false,
        Some(PanelId::Gpu),
        SortMode::default(),
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
        120,
        40,
        snapshot,
        false,
        Some(PanelId::MemDisk),
        SortMode::default(),
    );
    assert!(
        text.contains("Memory") || text.contains("RAM"),
        "Expected memory panel content; buffer:\n{text}"
    );
}

#[test]
fn expanded_panel_network_no_panic() {
    let text = mtop::tui::render_dashboard_with_state(
        120,
        40,
        MetricsSnapshot::default(),
        false,
        Some(PanelId::Network),
        SortMode::default(),
    );
    assert!(
        text.contains("Network") || text.contains("Upload"),
        "Expected network panel content; buffer:\n{text}"
    );
}

#[test]
fn expanded_panel_power_no_panic() {
    let snapshot = make_snapshot_with_power(6.0, 1.5);
    let text = mtop::tui::render_dashboard_with_state(
        120,
        40,
        snapshot,
        false,
        Some(PanelId::Power),
        SortMode::default(),
    );
    assert!(
        text.contains("Power") || text.contains("CPU Power"),
        "Expected power panel content; buffer:\n{text}"
    );
}

#[test]
fn expanded_panel_process_no_panic() {
    let snapshot = make_snapshot_with_processes(10);
    let text = mtop::tui::render_dashboard_with_state(
        120,
        40,
        snapshot,
        false,
        Some(PanelId::Process),
        SortMode::default(),
    );
    assert!(
        text.contains("Processes") || text.contains("proc"),
        "Expected process panel content; buffer:\n{text}"
    );
}
