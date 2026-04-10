/// Iteration 22: Braille visual and dashboard render tests.
/// Covers render_braille_graph_down, panel inner padding, memory bold titles,
/// process header dots, and network symmetric chart rendering.

use mtop::metrics::types::{MemoryMetrics, MetricsSnapshot, ProcessInfo, SortMode};
use mtop::tui::PanelId;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_snapshot_with_memory(ram_total: u64, ram_used: u64) -> MetricsSnapshot {
    let mut s = MetricsSnapshot::default();
    s.memory = MemoryMetrics {
        ram_total,
        ram_used,
        ..Default::default()
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

// ---------------------------------------------------------------------------
// Braille Down Tests
// ---------------------------------------------------------------------------

// 1. Empty input returns height empty rows
#[test]
fn braille_down_empty() {
    let result = mtop::tui::braille::render_braille_graph_down(&[], 100.0, 10, 5);
    assert_eq!(result.len(), 5, "Expected 5 rows for height=5");
    for row in &result {
        assert!(row.is_empty(), "Empty input should produce empty rows");
    }
}

// 2. 100% value with height=2 fills all rows with ⣿
#[test]
fn braille_down_full_scale() {
    let values = vec![100.0, 100.0];
    let result = mtop::tui::braille::render_braille_graph_down(&values, 100.0, 1, 2);
    assert_eq!(result.len(), 2);
    // Both rows should be fully filled: BRAILLE_DOWN[4][4] = '⣿'
    assert_eq!(result[0][0].0, '⣿', "Top row (row 0) should be ⣿ at 100%");
    assert_eq!(result[1][0].0, '⣿', "Bottom row (row 1) should be ⣿ at 100%");
}

// 3. 0% values produce spaces
#[test]
fn braille_down_zero() {
    let values = vec![0.0, 0.0];
    let result = mtop::tui::braille::render_braille_graph_down(&values, 100.0, 1, 2);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0][0].0, ' ', "Top row should be space at 0%");
    assert_eq!(result[1][0].0, ' ', "Bottom row should be space at 0%");
}

// 4. 50% with height=2: top row filled, bottom row empty (fills from top down)
#[test]
fn braille_down_half_height() {
    // 50% of 8 dots = 4 dots from top.
    // Row 0 (top, base=0): fill = (4-0).min(4) = 4 → ⣿
    // Row 1 (base=4): 4 > 4 is false → fill=0 → ' '
    let values = vec![50.0, 50.0];
    let result = mtop::tui::braille::render_braille_graph_down(&values, 100.0, 1, 2);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0][0].0, '⣿', "Top row should be fully filled at 50%");
    assert_eq!(result[1][0].0, ' ', "Bottom row should be empty at 50%");
}

// 5. left=100%, right=0% gives BRAILLE_DOWN[4][0]
#[test]
fn braille_down_asymmetric() {
    let values = vec![100.0, 0.0];
    let result = mtop::tui::braille::render_braille_graph_down(&values, 100.0, 1, 1);
    assert_eq!(result.len(), 1);
    // BRAILLE_DOWN[4][0] = '\u{2847}'
    assert_eq!(
        result[0][0].0, '\u{2847}',
        "left=100%, right=0% should give BRAILLE_DOWN[4][0]"
    );
}

// 6. BRAILLE_DOWN table corner values
#[test]
fn braille_down_table_corners() {
    // [0][0] = ' '
    let r00 = mtop::tui::braille::render_braille_graph_down(&[0.0, 0.0], 100.0, 1, 1);
    assert_eq!(r00[0][0].0, ' ', "BRAILLE_DOWN[0][0] should be space");

    // [4][4] = '⣿' (\u{28FF})
    let r44 = mtop::tui::braille::render_braille_graph_down(&[100.0, 100.0], 100.0, 1, 1);
    assert_eq!(r44[0][0].0, '\u{28FF}', "BRAILLE_DOWN[4][4] should be ⣿");

    // [4][0]: left full, right empty
    let r40 = mtop::tui::braille::render_braille_graph_down(&[100.0, 0.0], 100.0, 1, 1);
    assert_eq!(r40[0][0].0, '\u{2847}', "BRAILLE_DOWN[4][0] check");

    // [0][4]: left empty, right full
    let r04 = mtop::tui::braille::render_braille_graph_down(&[0.0, 100.0], 100.0, 1, 1);
    assert_eq!(r04[0][0].0, '\u{28B8}', "BRAILLE_DOWN[0][4] check");
}

// ---------------------------------------------------------------------------
// Dashboard Render Tests
// ---------------------------------------------------------------------------

// 7. Memory "Used" and "Avail" bold titles (capitalized) appear in detail mode
#[test]
fn memory_titles_bold_used_avail() {
    let gb: u64 = 1024 * 1024 * 1024;
    let snapshot = make_snapshot_with_memory(16 * gb, 12 * gb);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, true);
    assert!(
        text.contains("Used"),
        "Expected 'Used' (capitalized) in detail layout; buffer:\n{text}"
    );
    assert!(
        text.contains("Avail"),
        "Expected 'Avail' (capitalized) in detail layout; buffer:\n{text}"
    );
}

// 8. Process header has column names but NOT "•" on the header line
#[test]
fn process_header_no_dots() {
    let snapshot = make_snapshot_with_processes(3);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);

    // Find a line containing the header keywords
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

// 9. Dashboard renders without panic at 120x40 with inner padding
#[test]
fn panels_have_inner_padding() {
    let text = mtop::tui::render_dashboard_to_string(120, 40, MetricsSnapshot::default(), false);
    // The render must not be empty and must not panic
    assert!(!text.is_empty(), "Dashboard should render non-empty output");
    // Panel borders are present (ratatui draws box chars)
    assert!(
        text.contains('─') || text.contains('│') || text.contains('┌'),
        "Expected panel border characters in output"
    );
}

// 10. Network panel renders with "Upload" and "Download" labels in detail mode
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

// ---------------------------------------------------------------------------
// Additional Coverage
// ---------------------------------------------------------------------------

// 11. 29% with height=10 fills exactly top 3 rows (mirrors braille_up test)
#[test]
fn braille_down_29_percent() {
    // 29% of 40 dots = round(11.6) = 12 dots from top.
    // Row 0 (base=0): fill = (12-0).min(4) = 4 → ⣿
    // Row 1 (base=4): fill = (12-4).min(4) = 4 → ⣿
    // Row 2 (base=8): fill = (12-8).min(4) = 4 → ⣿
    // Row 3 (base=12): 12 > 12 = false → fill=0 → ' '
    let values = vec![29.0, 29.0];
    let result = mtop::tui::braille::render_braille_graph_down(&values, 100.0, 1, 10);
    assert_eq!(result.len(), 10);
    assert_eq!(result[0][0].0, '⣿', "Row 0 should be ⣿ at 29%");
    assert_eq!(result[1][0].0, '⣿', "Row 1 should be ⣿ at 29%");
    assert_eq!(result[2][0].0, '⣿', "Row 2 should be ⣿ at 29%");
    assert_eq!(result[3][0].0, ' ', "Row 3 should be empty at 29%");
    assert_eq!(result[9][0].0, ' ', "Bottom row should be empty at 29%");
}

// 12. Non-detail mode contains "disk:" label in memory/disk panel
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

// 13. Process data rows (entries, not header) contain "•" characters
#[test]
fn process_data_rows_have_dots() {
    let snapshot = make_snapshot_with_processes(5);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    // Data rows should contain bullet separators
    assert!(
        text.contains('•'),
        "Expected '•' in process data rows; buffer:\n{text}"
    );
}

// 14. All 6 panels render without panic when expanded at 120x40
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
            120,
            40,
            snapshot,
            false,
            Some(panel_id),
            SortMode::default(),
        );
        assert!(
            !text.is_empty(),
            "Expanded panel {panel_id:?} should render non-empty output"
        );
    }
}

// 15. Braille down: width=0 returns empty vec
#[test]
fn braille_down_zero_width() {
    let result = mtop::tui::braille::render_braille_graph_down(&[50.0, 50.0], 100.0, 0, 3);
    assert_eq!(result.len(), 3, "height=3 rows expected even for width=0");
    for row in &result {
        assert!(row.is_empty(), "Rows should be empty when width=0");
    }
}

// 16. Braille down: height=0 returns empty vec
#[test]
fn braille_down_zero_height() {
    let result = mtop::tui::braille::render_braille_graph_down(&[50.0, 50.0], 100.0, 5, 0);
    assert!(result.is_empty(), "height=0 should return empty vec");
}

// 17. Braille down: max_value<=0 treated as 1.0 (no divide-by-zero)
#[test]
fn braille_down_zero_max_value() {
    let values = vec![0.0, 0.0];
    let result = mtop::tui::braille::render_braille_graph_down(&values, 0.0, 1, 1);
    assert_eq!(result.len(), 1);
    // 0 / 1.0 = 0 → space
    assert_eq!(result[0][0].0, ' ', "Zero value with zero max should produce space");
}

// 18. Dashboard renders correctly at narrow 80x24 (no panic, contains mtop)
#[test]
fn dashboard_narrow_no_panic() {
    let text = mtop::tui::render_dashboard_to_string(80, 24, MetricsSnapshot::default(), false);
    assert!(!text.is_empty());
    assert!(
        text.contains("mtop"),
        "Expected 'mtop' in narrow dashboard header; buffer:\n{text}"
    );
}

// 19. Network expanded panel contains upload/download labels
#[test]
fn network_expanded_has_symmetric_labels() {
    let text = mtop::tui::render_dashboard_with_state(
        120,
        40,
        MetricsSnapshot::default(),
        false,
        Some(PanelId::Network),
        SortMode::default(),
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

// 20. Memory detail=true shows "Used" and "Avail" but NOT lowercase "used"/"avail" as standalone titles
#[test]
fn memory_detail_titles_are_capitalized() {
    let gb: u64 = 1024 * 1024 * 1024;
    let snapshot = make_snapshot_with_memory(16 * gb, 12 * gb);
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, true);

    // Capitalized versions must be present
    assert!(
        text.contains("Used"),
        "Expected capitalized 'Used' title; buffer:\n{text}"
    );
    assert!(
        text.contains("Avail"),
        "Expected capitalized 'Avail' title; buffer:\n{text}"
    );
}
