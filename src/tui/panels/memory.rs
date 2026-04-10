use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::metrics::MetricsSnapshot;
use crate::tui::{AppState, theme, gauge, layout};
use crate::tui::helpers::{format_bytes_rate, format_bytes_rate_compact};
use super::cpu::render_graph;

/// Memory+Disk panel: Type B layout when detail, 50/50 when not
pub(crate) fn draw_mem_disk_panel_v2(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    let gb = 1024.0 * 1024.0 * 1024.0;
    let ram_used_gb = s.memory.ram_used as f64 / gb;
    let ram_total_gb = s.memory.ram_total as f64 / gb;
    let ram_pct = if s.memory.ram_total > 0 {
        (s.memory.ram_used as f64 / s.memory.ram_total as f64 * 100.0) as u32
    } else {
        0
    };

    let border_color = theme::dim_color(theme.mem_accent, theme::adaptive_border_dim(theme));

    // Memory pressure colored dot (from theme)
    let pressure_dot_color = match s.memory.pressure_level {
        2 => theme.pressure_warn,
        4 => theme.pressure_critical,
        _ => theme.pressure_normal,
    };

    let title_spans = vec![
        Span::styled(format!(" {}", theme::PANEL_SUPERSCRIPTS[2]), Style::default().fg(theme.muted)),
        Span::styled("mem  ", Style::default().fg(theme.fg).bold()),
        Span::styled(format!("{ram_used_gb:.1}/{ram_total_gb:.0}GB {ram_pct}%"), Style::default().fg(theme.fg)),
        Span::styled(" \u{25cf}", Style::default().fg(pressure_dot_color)),
        Span::raw(" "),
    ];

    let block = Block::default()
        .title(Line::from(title_spans))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .border_type(ratatui::widgets::BorderType::Rounded);

    let raw_inner = block.inner(area);
    f.render_widget(block, area);

    // 1-char padding left/right + 1-line top padding
    let inner = Rect::new(raw_inner.x + 1, raw_inner.y + 1, raw_inner.width.saturating_sub(2), raw_inner.height.saturating_sub(1));

    if inner.height < 2 || inner.width == 0 {
        return;
    }

    // Reserve last row for bottom info
    let content_area = Rect::new(inner.x, inner.y, inner.width, inner.height.saturating_sub(1));
    let bottom_y = inner.y + inner.height.saturating_sub(1);

    let sparkline_data: Vec<f64> = state.history.mem_usage.iter().copied().collect();
    let available_data: Vec<f64> = state.history.mem_available.iter().copied().collect();

    // Compute available bytes/GB for labels (single source of truth)
    let ram_avail_bytes = s.memory.ram_total.saturating_sub(s.memory.ram_used) as f64;
    let ram_avail_gb = if s.memory.ram_total > 0 { ram_avail_bytes / gb } else { 0.0 };

    let sub_border_color = theme::dim_color(border_color, 0.8);
    let mb = 1024.0 * 1024.0;

    // Value strings for sub-panel titles (same calculation in both modes)
    let used_value_str = if ram_used_gb >= 1.0 {
        format!("{ram_used_gb:.1}GB")
    } else {
        format!("{:.0}MB", s.memory.ram_used as f64 / mb)
    };
    let avail_value_str = if ram_avail_gb >= 1.0 {
        format!("{ram_avail_gb:.1}GB")
    } else {
        format!("{:.0}MB", ram_avail_bytes / mb)
    };

    if state.show_detail {
        let (left, mid, right) = layout::split_type_b(content_area);

        // Left: "Used" sub-frame with bordered frame
        if left.height > 0 {
            let used_block = Block::default()
                .title(Line::from(vec![
                    Span::styled(" Used ", Style::default().fg(theme.fg).bold()),
                    Span::styled(used_value_str.clone(), Style::default().fg(theme.fg)),
                    Span::raw(" "),
                ]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(sub_border_color));
            let used_inner = used_block.inner(left);
            f.render_widget(used_block, left);
            if used_inner.height > 0 {
                render_graph(f, used_inner, &sparkline_data, 1.0);
            }
        }

        // Mid: "Avail" sub-frame with bordered frame
        if mid.height > 0 {
            let avail_block = Block::default()
                .title(Line::from(vec![
                    Span::styled(" Avail ", Style::default().fg(theme.fg).bold()),
                    Span::styled(avail_value_str.clone(), Style::default().fg(theme.fg)),
                    Span::raw(" "),
                ]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(sub_border_color));
            let avail_inner = avail_block.inner(mid);
            f.render_widget(avail_block, mid);
            if avail_inner.height > 0 {
                render_graph(f, avail_inner, &available_data, 1.0);
            }
        }

        // Right 25%: Disk detail (vertically centered)
        let disk_used_gb = s.disk.used_bytes as f64 / gb;
        let disk_total_gb = s.disk.total_bytes as f64 / gb;
        let disk_fraction = if s.disk.total_bytes > 0 {
            s.disk.used_bytes as f64 / s.disk.total_bytes as f64
        } else {
            0.0
        };

        let detail_lines: Vec<Line> = vec![
            Line::from(Span::styled("Disk", Style::default().fg(theme.fg).bold())),
            Line::from(gauge::render_compact_gauge(disk_fraction, right.width as usize, theme)),
            Line::from(Span::styled(
                format!("{disk_used_gb:.0}/{disk_total_gb:.0} GB"),
                Style::default().fg(theme.fg),
            )),
            Line::from(Span::raw("")),
            Line::from(Span::styled(
                format!("read  {}", format_bytes_rate(s.disk.read_bytes_sec as f64)),
                Style::default().fg(theme.fg),
            )),
            Line::from(Span::styled(
                format!("write {}", format_bytes_rate(s.disk.write_bytes_sec as f64)),
                Style::default().fg(theme.fg),
            )),
        ];

        let content_h = detail_lines.len().min(right.height as usize);
        let y_offset = (right.height as usize).saturating_sub(content_h) / 2;

        for (i, line) in detail_lines.iter().enumerate().take(content_h) {
            let y = right.y + y_offset as u16 + i as u16;
            if y >= right.y + right.height {
                break;
            }
            f.render_widget(
                Paragraph::new(line.clone()),
                Rect::new(right.x, y, right.width, 1),
            );
        }
    } else {
        // 50/50 split: used + available graphs with sub-panel borders
        let half_w = content_area.width / 2;
        let left = Rect::new(content_area.x, content_area.y, half_w, content_area.height);
        let mid = Rect::new(content_area.x + half_w, content_area.y, content_area.width - half_w, content_area.height);

        let used_block = Block::default()
            .title(Line::from(vec![
                Span::styled(" Used ", Style::default().fg(theme.fg).bold()),
                Span::styled(used_value_str, Style::default().fg(theme.fg)),
                Span::raw(" "),
            ]))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(sub_border_color));
        let used_inner = used_block.inner(left);
        f.render_widget(used_block, left);
        if used_inner.height > 0 {
            render_graph(f, used_inner, &sparkline_data, 1.0);
        }

        let avail_block = Block::default()
            .title(Line::from(vec![
                Span::styled(" Avail ", Style::default().fg(theme.fg).bold()),
                Span::styled(avail_value_str, Style::default().fg(theme.fg)),
                Span::raw(" "),
            ]))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(sub_border_color));
        let avail_inner = avail_block.inner(mid);
        f.render_widget(avail_block, mid);
        if avail_inner.height > 0 {
            render_graph(f, avail_inner, &available_data, 1.0);
        }

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

    // Bottom info: Swap (only show if swap is configured)
    if s.memory.swap_total == 0 {
        return;
    }
    let swap_used_gb = s.memory.swap_used as f64 / gb;
    let swap_total_gb = s.memory.swap_total as f64 / gb;
    let swap_in = s.memory.swap_in_bytes_sec;
    let swap_out = s.memory.swap_out_bytes_sec;

    let swap_text = if swap_in == 0.0 && swap_out == 0.0 {
        format!(" Swap: {swap_used_gb:.1}/{swap_total_gb:.1} GB")
    } else {
        format!(
            " Swap: {swap_used_gb:.1}/{swap_total_gb:.1} GB  in {}/out {}",
            format_bytes_rate_compact(swap_in),
            format_bytes_rate_compact(swap_out),
        )
    };

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(swap_text, Style::default().fg(theme.muted)))),
        Rect::new(inner.x, bottom_y, inner.width, 1),
    );
}
