use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::metrics::MetricsSnapshot;
use crate::tui::{AppState, theme, gauge, layout};
use crate::tui::helpers::format_bytes_rate;
use super::cpu::render_graph;

/// Memory+Disk panel: Type A layout (75% multi-row braille graph + 25% disk detail)
pub(crate) fn draw_mem_disk_panel_v2(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    let gb = 1024.0 * 1024.0 * 1024.0;
    let ram_used_gb = s.memory.ram_used as f64 / gb;
    let ram_total_gb = s.memory.ram_total as f64 / gb;
    let ram_pct = if s.memory.ram_total > 0 {
        (s.memory.ram_used as f64 / s.memory.ram_total as f64 * 100.0) as u32
    } else {
        0
    };

    let border_color = theme::dim_color(theme.mem_accent, 0.4);

    let title_spans = vec![
        Span::styled(" mem  ", Style::default().fg(theme.mem_accent).bold()),
        Span::styled(format!("{ram_used_gb:.1}/{ram_total_gb:.0}GB {ram_pct}%"), Style::default().fg(theme.fg)),
        Span::raw(" "),
    ];

    let block = Block::default()
        .title(Line::from(title_spans))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .border_type(ratatui::widgets::BorderType::Rounded);

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 2 {
        return;
    }

    // Reserve last row for bottom info
    let content_area = Rect::new(inner.x, inner.y, inner.width, inner.height.saturating_sub(1));
    let bottom_y = inner.y + inner.height.saturating_sub(1);

    let sparkline_data: Vec<f64> = state.history.mem_usage.iter().copied().collect();

    if state.show_detail {
        let (trend_area, detail_area) = layout::split_type_a(content_area);

        // Left: Memory usage multi-row braille graph
        render_graph(f, trend_area, &sparkline_data, 1.0, theme.mem_accent);

        // Right: Disk detail (vertically centered)
        let disk_used_gb = s.disk.used_bytes as f64 / gb;
        let disk_total_gb = s.disk.total_bytes as f64 / gb;
        let disk_fraction = if s.disk.total_bytes > 0 {
            s.disk.used_bytes as f64 / s.disk.total_bytes as f64
        } else {
            0.0
        };

        let detail_lines: Vec<Line> = vec![
            Line::from(Span::styled("Disk", Style::default().fg(theme.fg).bold())),
            Line::from(gauge::render_compact_gauge(disk_fraction, detail_area.width as usize, theme)),
            Line::from(Span::styled(
                format!("{disk_used_gb:.0}/{disk_total_gb:.0} GB"),
                Style::default().fg(theme.fg),
            )),
            Line::from(Span::styled(
                format!("R: {}", format_bytes_rate(s.disk.read_bytes_sec as f64)),
                Style::default().fg(theme.fg),
            )),
            Line::from(Span::styled(
                format!("W: {}", format_bytes_rate(s.disk.write_bytes_sec as f64)),
                Style::default().fg(theme.fg),
            )),
        ];

        let content_h = detail_lines.len().min(detail_area.height as usize);
        let y_offset = (detail_area.height as usize).saturating_sub(content_h) / 2;

        for (i, line) in detail_lines.iter().enumerate().take(content_h) {
            let y = detail_area.y + y_offset as u16 + i as u16;
            if y >= detail_area.y + detail_area.height {
                break;
            }
            f.render_widget(
                Paragraph::new(line.clone()),
                Rect::new(detail_area.x, y, detail_area.width, 1),
            );
        }
    } else {
        // Full-width graph, fallback disk info on bottom row
        render_graph(f, content_area, &sparkline_data, 1.0, theme.mem_accent);

        let disk_used_gb = s.disk.used_bytes as f64 / gb;
        let disk_total_gb = s.disk.total_bytes as f64 / gb;
        let disk_pct = if s.disk.total_bytes > 0 {
            (s.disk.used_bytes as f64 / s.disk.total_bytes as f64 * 100.0) as u32
        } else {
            0
        };
        let fallback = format!("disk: {disk_pct}% {disk_used_gb:.0}/{disk_total_gb:.0}GB ");
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(fallback, Style::default().fg(theme.muted)))
                .alignment(ratatui::layout::Alignment::Right)),
            Rect::new(inner.x, bottom_y, inner.width, 1),
        );
        return; // skip default bottom info
    }

    // Bottom info inside panel
    let swap_used_gb = s.memory.swap_used as f64 / gb;
    let swap_total_gb = s.memory.swap_total as f64 / gb;
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!(" Swap {swap_used_gb:.1}/{swap_total_gb:.1} GB"),
            Style::default().fg(theme.muted),
        ))),
        Rect::new(inner.x, bottom_y, inner.width, 1),
    );
}
