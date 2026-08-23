use ratatui::prelude::*;
use ratatui::widgets::*;

use super::cpu::{render_graph, render_graph_green};
use crate::metrics::MetricsSnapshot;
use crate::tui::helpers::{format_bytes_rate, format_bytes_rate_compact};
use crate::tui::{AppState, gauge, layout, theme};

/// Memory+Disk panel: Type B layout when detail, 50/50 when not
pub(crate) fn draw_mem_disk_panel_v2(
    f: &mut Frame,
    area: Rect,
    s: &MetricsSnapshot,
    state: &AppState,
    theme: &theme::Theme,
) {
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

    let title_capacity = area.width.saturating_sub(2) as usize;
    let mut swap_text = (s.memory.swap_total > 0).then(|| {
        let swap_used_gb = s.memory.swap_used as f64 / gb;
        let swap_total_gb = s.memory.swap_total as f64 / gb;
        let mut text = format!(" swap: {swap_used_gb:.1}/{swap_total_gb:.1}GB");
        if state.show_detail
            && (s.memory.swap_in_bytes_sec > 0.0 || s.memory.swap_out_bytes_sec > 0.0)
        {
            text.push_str(&format!(
                " in:{} out:{}",
                format_bytes_rate_compact(s.memory.swap_in_bytes_sec),
                format_bytes_rate_compact(s.memory.swap_out_bytes_sec),
            ));
        }
        text.push(' ');
        text
    });

    // Keep the right-aligned swap title complete at the supported narrow
    // dashboard width by compacting its precision and labels before the left
    // memory title yields detail.
    if swap_text.as_ref().map_or(0, String::len) + 6 > title_capacity {
        swap_text = swap_text.map(|_| {
            let swap_used_gb = s.memory.swap_used as f64 / gb;
            let swap_total_gb = s.memory.swap_total as f64 / gb;
            let mut text = format!(" {}", format_swap_usage_title(swap_used_gb, swap_total_gb));
            if state.show_detail
                && (s.memory.swap_in_bytes_sec > 0.0 || s.memory.swap_out_bytes_sec > 0.0)
            {
                let swap_in = format_swap_rate_title(s.memory.swap_in_bytes_sec);
                let swap_out = format_swap_rate_title(s.memory.swap_out_bytes_sec);
                text.push_str(&format!(" in:{swap_in} out:{swap_out}"));
            }
            text.push(' ');
            if text.len() > title_capacity {
                text.remove(0);
            }
            text
        });
    }

    let left_capacity = title_capacity.saturating_sub(swap_text.as_ref().map_or(0, String::len));
    let full_memory_text = format!("{ram_used_gb:.1}/{ram_total_gb:.0}GB {ram_pct}%");
    let compact_memory_text = format!("{ram_used_gb:.0}/{ram_total_gb:.0}GB {ram_pct}%");
    let percent_text = format!("{ram_pct}%");
    let (mem_label, memory_text, show_pressure) =
        if 2 + 5 + full_memory_text.len() + 3 <= left_capacity {
            ("mem  ", full_memory_text, true)
        } else if 2 + 4 + compact_memory_text.len() + 3 <= left_capacity {
            ("mem ", compact_memory_text, true)
        } else if 2 + 4 + percent_text.len() + 3 <= left_capacity {
            ("mem ", percent_text, true)
        } else if 2 + 4 <= left_capacity {
            ("mem ", String::new(), false)
        } else {
            ("", String::new(), false)
        };

    let left_title = (left_capacity >= 2).then(|| {
        let mut spans = vec![
            Span::styled(
                format!(" {}", theme::PANEL_SUPERSCRIPTS[2]),
                Style::default().fg(theme.muted),
            ),
            Span::styled(mem_label, Style::default().fg(theme.fg).bold()),
            Span::styled(memory_text, Style::default().fg(theme.fg)),
        ];
        if show_pressure {
            spans.push(Span::styled(
                " \u{25cf}",
                Style::default().fg(pressure_dot_color),
            ));
            spans.push(Span::raw(" "));
        }
        Line::from(spans)
    });

    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .border_type(ratatui::widgets::BorderType::Rounded);
    if let Some(left_title) = left_title {
        block = block.title(left_title);
    }
    if let Some(swap_text) = swap_text {
        block = block.title_top(
            Line::from(Span::styled(swap_text, Style::default().fg(theme.muted)))
                .alignment(Alignment::Right),
        );
    }

    let raw_inner = block.inner(area);
    f.render_widget(block, area);

    // 1-char padding left/right, no top padding (UAT-07)
    let inner = Rect::new(
        raw_inner.x + 1,
        raw_inner.y,
        raw_inner.width.saturating_sub(2),
        raw_inner.height,
    );

    if inner.height < 2 || inner.width == 0 {
        return;
    }

    // Hidden mode reserves its last row for the compact disk-capacity signal.
    // Detail mode reclaims the old swap-only row after swap moves to the title.
    let bottom_rows = usize::from(!state.show_detail) as u16;
    let content_area = Rect::new(
        inner.x,
        inner.y,
        inner.width,
        inner.height.saturating_sub(bottom_rows),
    );
    let bottom_y = inner.y + inner.height.saturating_sub(1);

    let sparkline_data: Vec<f64> = state.history.mem_usage.iter().copied().collect();
    let available_data: Vec<f64> = state.history.mem_available.iter().copied().collect();

    // Compute available bytes/GB for labels (single source of truth)
    let ram_avail_bytes = s.memory.ram_total.saturating_sub(s.memory.ram_used) as f64;
    let ram_avail_gb = if s.memory.ram_total > 0 {
        ram_avail_bytes / gb
    } else {
        0.0
    };

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
                    Span::styled(" used ", Style::default().fg(theme.fg).bold()),
                    Span::styled(used_value_str.clone(), Style::default().fg(theme.fg)),
                    Span::raw(" "),
                ]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(sub_border_color));
            let used_inner = used_block.inner(left);
            f.render_widget(used_block, left);
            if used_inner.height > 0 {
                render_graph(f, used_inner, &sparkline_data, 1.0, theme);
            }
        }

        // Mid: "avail" sub-frame with bordered frame
        if mid.height > 0 {
            let avail_block = Block::default()
                .title(Line::from(vec![
                    Span::styled(" avail ", Style::default().fg(theme.fg).bold()),
                    Span::styled(avail_value_str.clone(), Style::default().fg(theme.fg)),
                    Span::raw(" "),
                ]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(sub_border_color));
            let avail_inner = avail_block.inner(mid);
            f.render_widget(avail_block, mid);
            if avail_inner.height > 0 {
                render_graph_green(f, avail_inner, &available_data, 1.0, theme);
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
            Line::from(Span::styled("disk", Style::default().fg(theme.fg).bold())),
            Line::from(gauge::render_compact_gauge(
                disk_fraction,
                right.width as usize,
                theme,
            )),
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
        let mid = Rect::new(
            content_area.x + half_w,
            content_area.y,
            content_area.width - half_w,
            content_area.height,
        );

        let used_block = Block::default()
            .title(Line::from(vec![
                Span::styled(" used ", Style::default().fg(theme.fg).bold()),
                Span::styled(used_value_str, Style::default().fg(theme.fg)),
                Span::raw(" "),
            ]))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(sub_border_color));
        let used_inner = used_block.inner(left);
        f.render_widget(used_block, left);
        if used_inner.height > 0 {
            render_graph(f, used_inner, &sparkline_data, 1.0, theme);
        }

        let avail_block = Block::default()
            .title(Line::from(vec![
                Span::styled(" avail ", Style::default().fg(theme.fg).bold()),
                Span::styled(avail_value_str, Style::default().fg(theme.fg)),
                Span::raw(" "),
            ]))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(sub_border_color));
        let avail_inner = avail_block.inner(mid);
        f.render_widget(avail_block, mid);
        if avail_inner.height > 0 {
            render_graph_green(f, avail_inner, &available_data, 1.0, theme);
        }

        render_compact_disk_row(f, Rect::new(inner.x, bottom_y, inner.width, 1), s, theme);
    }
}

/// Bound a narrow frame-title capacity to at most three digits per value,
/// promoting both values together so the swap status remains complete.
fn format_swap_usage_title(used_gb: f64, total_gb: f64) -> String {
    const UNITS: [&str; 4] = ["GB", "TB", "PB", "EB"];
    let mut unit_index = 0;
    let mut divisor = 1.0;
    while unit_index + 1 < UNITS.len() && total_gb / divisor >= 999.5 {
        unit_index += 1;
        divisor *= 1024.0;
    }
    format!(
        "swap:{:.0}/{:.0}{}",
        used_gb / divisor,
        total_gb / divisor,
        UNITS[unit_index]
    )
}

/// Bound a narrow frame-title rate to at most `999X/s`, promoting units near
/// the four-digit boundary so two complete `in:`/`out:` fields can coexist.
fn format_swap_rate_title(bytes_per_sec: f64) -> String {
    const UNITS: [&str; 7] = ["B/s", "K/s", "M/s", "G/s", "T/s", "P/s", "E/s"];
    let mut unit_index = 0;
    let mut divisor = 1.0;
    while unit_index + 1 < UNITS.len() && bytes_per_sec / divisor >= 999.5 {
        unit_index += 1;
        divisor *= 1024.0;
    }
    format!("{:.0}{}", bytes_per_sec / divisor, UNITS[unit_index])
}

/// Render a responsive, disk-only compact row. Capacity and its gradient gauge
/// win width over throughput so near-full storage remains visible at the
/// supported 80-column dashboard size.
fn render_compact_disk_row(f: &mut Frame, area: Rect, s: &MetricsSnapshot, theme: &theme::Theme) {
    if area.width == 0 {
        return;
    }

    const MIN_GAUGE_WIDTH: usize = 4;
    const GAP: usize = 2;

    let gb = 1024.0 * 1024.0 * 1024.0;
    let fraction = if s.disk.total_bytes > 0 {
        (s.disk.used_bytes as f64 / s.disk.total_bytes as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let pct = (fraction * 100.0).round() as u32;
    let used_gb = s.disk.used_bytes as f64 / gb;
    let total_gb = s.disk.total_bytes as f64 / gb;
    let full_label = format!("disk: {pct}% {used_gb:.0}/{total_gb:.0}GB ");
    let short_label = format!("disk: {pct}% ");
    let rates = format!(
        "r:{} w:{}",
        format_bytes_rate_compact(s.disk.read_bytes_sec as f64),
        format_bytes_rate_compact(s.disk.write_bytes_sec as f64),
    );

    let total_width = area.width as usize;
    let show_rates = total_width
        >= full_label
            .len()
            .saturating_add(MIN_GAUGE_WIDTH + GAP + rates.len());
    let left_width = if show_rates {
        total_width - GAP - rates.len()
    } else {
        total_width
    };

    let label = if left_width >= full_label.len() + MIN_GAUGE_WIDTH {
        full_label
    } else if left_width > short_label.len() {
        short_label
    } else {
        String::new()
    };
    let gauge_width = left_width.saturating_sub(label.len());
    let mut left_spans = vec![Span::styled(label, Style::default().fg(theme.fg))];
    left_spans.extend(gauge::render_gauge_bar(
        fraction,
        1.0,
        gauge_width,
        "",
        theme,
    ));
    f.render_widget(
        Paragraph::new(Line::from(left_spans)),
        Rect::new(area.x, area.y, left_width as u16, 1),
    );

    if show_rates {
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                rates,
                Style::default().fg(theme.muted),
            )))
            .alignment(Alignment::Right),
            Rect::new(
                area.x + left_width as u16 + GAP as u16,
                area.y,
                (total_width - left_width - GAP) as u16,
                1,
            ),
        );
    }
}
