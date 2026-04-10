use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::metrics::MetricsSnapshot;
use crate::tui::{AppState, theme, braille, gauge, layout};
use crate::tui::helpers::format_bytes_rate;

/// Memory+Disk panel: Type A layout (75% sparkline+gauges + 25% disk detail)
pub(crate) fn draw_mem_disk_panel_v2(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    let gb = 1024.0 * 1024.0 * 1024.0;
    let ram_used_gb = s.memory.ram_used as f64 / gb;
    let ram_total_gb = s.memory.ram_total as f64 / gb;
    let ram_pct = if s.memory.ram_total > 0 {
        (s.memory.ram_used as f64 / s.memory.ram_total as f64 * 100.0) as u32
    } else {
        0
    };

    let disk_used_gb = s.disk.used_bytes as f64 / gb;
    let disk_total_gb = s.disk.total_bytes as f64 / gb;
    let disk_pct = if s.disk.total_bytes > 0 {
        (s.disk.used_bytes as f64 / s.disk.total_bytes as f64 * 100.0) as u32
    } else {
        0
    };

    let title_spans = vec![
        Span::styled(" Memory  ", Style::default().fg(theme.mem_accent).bold()),
        Span::styled(format!("{ram_used_gb:.1}/{ram_total_gb:.0} GB  {ram_pct}%"), Style::default().fg(theme.fg)),
        Span::styled("  Disk  ", Style::default().fg(theme.muted)),
        Span::styled(format!("{disk_used_gb:.0}/{disk_total_gb:.0} GB  {disk_pct}%"), Style::default().fg(theme.fg)),
        Span::raw(" "),
    ];

    let swap_used_gb = s.memory.swap_used as f64 / gb;
    let swap_total_gb = s.memory.swap_total as f64 / gb;
    let bottom_left = vec![
        Span::styled(format!(" Swap {swap_used_gb:.1}/{swap_total_gb:.1} GB"), Style::default().fg(theme.muted)),
    ];
    let bottom_right = vec![
        Span::styled(
            format!("R: {}  W: {} ", format_bytes_rate(s.disk.read_bytes_sec as f64), format_bytes_rate(s.disk.write_bytes_sec as f64)),
            Style::default().fg(theme.muted),
        ),
    ];

    let block = Block::default()
        .title(Line::from(title_spans))
        .title_bottom(Line::from(bottom_left).alignment(ratatui::layout::Alignment::Left))
        .title_bottom(Line::from(bottom_right).alignment(ratatui::layout::Alignment::Right))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .border_type(ratatui::widgets::BorderType::Rounded);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let (trend_area, detail_area) = layout::split_type_a(inner);

    // Left 75%: RAM sparkline + RAM gauge + Swap gauge
    let left_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(trend_area);

    // RAM sparkline
    let sparkline_data: Vec<f64> = state.history.mem_usage.iter().copied().collect();
    let spark_width = left_rows[0].width as usize;
    let spark = braille::render_braille_sparkline(&sparkline_data, 1.0, spark_width);
    let spark_spans: Vec<Span> = spark
        .iter()
        .map(|&(ch, color)| Span::styled(ch.to_string(), Style::default().fg(color)))
        .collect();
    if !spark_spans.is_empty() {
        let y_offset = left_rows[0].height / 2;
        let spark_rect = Rect::new(left_rows[0].x, left_rows[0].y + y_offset, left_rows[0].width, 1);
        f.render_widget(Paragraph::new(Line::from(spark_spans)), spark_rect);
    }

    // RAM gauge bar
    let ram_label = format!("{ram_used_gb:.1}/{ram_total_gb:.0} GB");
    let ram_gauge_spans = gauge::render_gauge_bar(
        s.memory.ram_used as f64, s.memory.ram_total as f64,
        left_rows[1].width.saturating_sub(16) as usize,
        &ram_label,
    );
    f.render_widget(Paragraph::new(Line::from(ram_gauge_spans)), left_rows[1]);

    // Swap gauge bar
    let swap_label = format!("{swap_used_gb:.1}/{swap_total_gb:.1} GB");
    let swap_gauge_spans = gauge::render_gauge_bar(
        s.memory.swap_used as f64, s.memory.swap_total as f64,
        left_rows[2].width.saturating_sub(16) as usize,
        &swap_label,
    );
    f.render_widget(Paragraph::new(Line::from(swap_gauge_spans)), left_rows[2]);

    // Right 25%: Disk capacity gauge + IO rates
    let right_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(detail_area);

    // Disk capacity gauge
    let disk_gauge_spans = gauge::render_compact_gauge(
        if s.disk.total_bytes > 0 { s.disk.used_bytes as f64 / s.disk.total_bytes as f64 } else { 0.0 },
        right_rows[0].width as usize,
    );
    f.render_widget(Paragraph::new(Line::from(disk_gauge_spans)), right_rows[0]);

    // IO read rate
    if right_rows.len() > 2 {
        let read_text = format!("R: {}", format_bytes_rate(s.disk.read_bytes_sec as f64));
        f.render_widget(
            Paragraph::new(read_text).style(Style::default().fg(theme.fg)),
            right_rows[2],
        );
    }

    // IO write rate
    if right_rows.len() > 3 {
        let write_text = format!("W: {}", format_bytes_rate(s.disk.write_bytes_sec as f64));
        f.render_widget(
            Paragraph::new(write_text).style(Style::default().fg(theme.fg)),
            right_rows[3],
        );
    }
}
