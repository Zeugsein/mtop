use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Minimum terminal dimensions required by mtop.
pub const MIN_COLS: u16 = 80;
pub const MIN_ROWS: u16 = 24;

/// Check if the terminal is large enough for the dashboard.
pub fn terminal_too_small(area: Rect) -> bool {
    area.width < MIN_COLS || area.height < MIN_ROWS
}

/// Generate the "terminal too small" message for display.
pub fn too_small_message(area: Rect) -> String {
    format!(
        "Terminal too small (need {}×{}, got {}×{})",
        MIN_COLS, MIN_ROWS, area.width, area.height
    )
}

/// Split area into a Type A panel layout: 75% trend + 25% detail.
pub fn split_type_a(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(area);
    (chunks[0], chunks[1])
}

/// Split area into a Type B panel layout: 37.5% + 37.5% + 25% (approximated as 38/37/25).
pub fn split_type_b(area: Rect) -> (Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(38),
            Constraint::Percentage(37),
            Constraint::Percentage(25),
        ])
        .split(area);
    (chunks[0], chunks[1], chunks[2])
}

/// Main page layout: header + two-column body + footer.
/// Returns (header, left_column, right_column, footer).
pub fn split_page(area: Rect) -> PageLayout {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // header
            Constraint::Min(18),   // body (two columns)
            Constraint::Length(1), // footer
        ])
        .split(area);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main[1]);

    PageLayout {
        header: main[0],
        left_column: columns[0],
        right_column: columns[1],
        footer: main[2],
    }
}

/// Split a column into 3 equal panel rows.
pub fn split_column_3(area: Rect) -> (Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(area);
    (chunks[0], chunks[1], chunks[2])
}

/// Page layout areas returned by split_page.
pub struct PageLayout {
    pub header: Rect,
    pub left_column: Rect,
    pub right_column: Rect,
    pub footer: Rect,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(w: u16, h: u16) -> Rect {
        Rect::new(0, 0, w, h)
    }

    #[test]
    fn test_terminal_too_small_below_minimum() {
        assert!(terminal_too_small(rect(79, 24)));
        assert!(terminal_too_small(rect(80, 23)));
        assert!(terminal_too_small(rect(40, 10)));
    }

    #[test]
    fn test_terminal_ok_at_minimum() {
        assert!(!terminal_too_small(rect(80, 24)));
        assert!(!terminal_too_small(rect(120, 40)));
        assert!(!terminal_too_small(rect(200, 60)));
    }

    #[test]
    fn test_too_small_message_format() {
        let msg = too_small_message(rect(60, 20));
        assert!(msg.contains("60"));
        assert!(msg.contains("20"));
        assert!(msg.contains("80"));
        assert!(msg.contains("24"));
    }

    #[test]
    fn test_type_a_split_proportions() {
        let area = rect(100, 20);
        let (trend, detail) = split_type_a(area);
        // 75% of 100 = 75, 25% = 25
        assert_eq!(trend.width, 75);
        assert_eq!(detail.width, 25);
        assert_eq!(trend.height, area.height);
        assert_eq!(detail.height, area.height);
    }

    #[test]
    fn test_type_b_split_proportions() {
        let area = rect(100, 20);
        let (t1, t2, detail) = split_type_b(area);
        // 38% + 37% + 25% = 100
        assert_eq!(t1.width, 38);
        assert_eq!(t2.width, 37);
        assert_eq!(detail.width, 25);
    }

    #[test]
    fn test_page_layout_header_full_width() {
        let area = rect(160, 40);
        let page = split_page(area);
        assert_eq!(page.header.width, 160);
        assert_eq!(page.header.height, 1);
        assert_eq!(page.footer.width, 160);
        assert_eq!(page.footer.height, 1);
    }

    #[test]
    fn test_page_layout_columns_split_evenly() {
        let area = rect(160, 40);
        let page = split_page(area);
        assert_eq!(page.left_column.width, 80);
        assert_eq!(page.right_column.width, 80);
    }

    #[test]
    fn test_column_3_split() {
        let area = rect(80, 30);
        let (r1, r2, r3) = split_column_3(area);
        assert_eq!(r1.width, 80);
        assert_eq!(r2.width, 80);
        assert_eq!(r3.width, 80);
        // Each row gets ~10 lines (30/3)
        assert_eq!(r1.height, 10);
        assert_eq!(r2.height, 10);
        assert_eq!(r3.height, 10);
    }
}
