use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::metrics::MetricsSnapshot;
use crate::tui::{AppState, theme, braille, layout};
use crate::tui::helpers::{truncate_by_display_width, pad_to_display_width};

/// Power panel: Type B layout (37.5% CPU sparkline + 37.5% GPU sparkline + 25% per-process energy)
pub(crate) fn draw_power_panel_v2(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    let border_color = theme::dim_color(theme.power_accent, theme::adaptive_border_dim(theme));

    if !s.power.available {
        let block = Block::default()
            .title(Line::from(vec![
                Span::styled(format!(" {}", theme::PANEL_SUPERSCRIPTS[4]), Style::default().fg(theme.muted)),
                Span::styled("power ", Style::default().fg(theme.fg).bold()),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .border_type(ratatui::widgets::BorderType::Rounded);
        let raw_inner = block.inner(area);
        f.render_widget(block, area);
        let inner = Rect::new(raw_inner.x + 1, raw_inner.y + 1, raw_inner.width.saturating_sub(2), raw_inner.height.saturating_sub(1));
        f.render_widget(
            Paragraph::new("Power sensors: N/A").style(Style::default().fg(theme.muted)),
            inner,
        );
        return;
    }

    let title_spans = vec![
        Span::styled(format!(" {}", theme::PANEL_SUPERSCRIPTS[4]), Style::default().fg(theme.muted)),
        Span::styled("power ", Style::default().fg(theme.fg).bold()),
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

    let cpu_tdp = s.soc.cpu_tdp_w() as f64;
    let gpu_tdp = s.soc.gpu_tdp_w() as f64;
    let cpu_power_data: Vec<f64> = state.history.cpu_power.iter().copied().collect();
    let gpu_power_data: Vec<f64> = state.history.gpu_power.iter().copied().collect();

    let gpu_idle = s.power.gpu_w < 0.5;

    // Helper to render a labeled power sparkline into an area
    let render_labeled_sparkline = |f: &mut Frame, area: Rect, label: &str, watts: f32, data: &[f64], max: f64, label_color: Color, show_idle: bool| {
        let graph_area = if label.is_empty() {
            // No label line — graph fills entire area (title bar has info)
            area
        } else {
            let mut spans = vec![
                Span::styled(format!("{label} "), Style::default().fg(label_color)),
                Span::styled(format!("{watts:.1}W"), Style::default().fg(theme.fg)),
            ];
            if show_idle {
                spans.push(Span::styled(" (idle)", Style::default().fg(theme.muted)));
            }
            f.render_widget(
                Paragraph::new(Line::from(spans)),
                Rect::new(area.x, area.y, area.width, 1),
            );
            if area.height <= 1 { return; }
            Rect::new(area.x, area.y + 1, area.width, area.height - 1)
        };
        let graph = braille::render_braille_graph(data, max, graph_area.width as usize, graph_area.height as usize);
        for (row_idx, row) in graph.iter().enumerate() {
            let y = graph_area.y + graph_area.height.saturating_sub(1) - row_idx as u16;
            if y < graph_area.y { break; }
            let spans: Vec<Span> = row.iter().map(|&(ch, color)| Span::styled(ch.to_string(), Style::default().fg(color))).collect();
            if !spans.is_empty() {
                f.render_widget(Paragraph::new(Line::from(spans)), Rect::new(graph_area.x, y, graph_area.width, 1));
            }
        }
    };

    if state.show_detail {
        let (left, mid, right) = layout::split_type_b(content_area);

        // Sub-frame borders for CPU and GPU areas
        let sub_border_color = theme::dim_color(border_color, 0.8);
        let cpu_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(sub_border_color));
        let gpu_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(sub_border_color));
        let cpu_inner = cpu_block.inner(left);
        let gpu_inner = gpu_block.inner(mid);
        f.render_widget(cpu_block, left);
        f.render_widget(gpu_block, mid);

        render_labeled_sparkline(f, cpu_inner, "cpu", s.power.cpu_w, &cpu_power_data, cpu_tdp, theme.cpu_accent, false);
        render_labeled_sparkline(f, gpu_inner, "gpu", s.power.gpu_w, &gpu_power_data, gpu_tdp, theme.gpu_accent, gpu_idle);

        // Right 25%: Per-process energy ranking (white text)
        let mut procs_by_power: Vec<&crate::metrics::ProcessInfo> = s.processes.iter()
            .filter(|p| p.power_w > 0.0)
            .collect();
        procs_by_power.sort_by(|a, b| b.power_w.partial_cmp(&a.power_w).unwrap_or(std::cmp::Ordering::Equal));

        let max_rows = right.height as usize;

        for (i, proc) in procs_by_power.iter().take(max_rows).enumerate() {
            let y = right.y + i as u16;
            if y >= right.y + right.height {
                break;
            }

            let name_width = right.width.saturating_sub(6) as usize;
            let name = truncate_by_display_width(&proc.name, name_width);

            let line = Line::from(vec![
                Span::styled(pad_to_display_width(&name, name_width), Style::default().fg(theme.fg)),
                Span::raw("  "),
                Span::styled(format!("{:.1}W", proc.power_w), Style::default().fg(theme.fg)),
            ]);
            f.render_widget(Paragraph::new(line), Rect::new(right.x, y, right.width, 1));
        }
    } else {
        // 50/50 split: bordered cpu + gpu sub-panels (no detail text)
        let half_w = content_area.width / 2;
        let left = Rect::new(content_area.x, content_area.y, half_w, content_area.height);
        let mid = Rect::new(content_area.x + half_w, content_area.y, content_area.width - half_w, content_area.height);

        let sub_border_color = theme::dim_color(border_color, 0.8);
        let cpu_block = Block::default()
            .title(Line::from(vec![
                Span::styled(" cpu ", Style::default().fg(theme.fg).bold()),
                Span::styled(format!("{:.1}W", s.power.cpu_w), Style::default().fg(theme.fg)),
                Span::raw(" "),
            ]))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(sub_border_color));
        let cpu_inner = cpu_block.inner(left);
        f.render_widget(cpu_block, left);
        render_labeled_sparkline(f, cpu_inner, "", s.power.cpu_w, &cpu_power_data, cpu_tdp, theme.cpu_accent, false);

        let gpu_block = Block::default()
            .title(Line::from(vec![
                Span::styled(" gpu ", Style::default().fg(theme.fg).bold()),
                Span::styled(format!("{:.1}W", s.power.gpu_w), Style::default().fg(theme.fg)),
                if gpu_idle { Span::styled(" (idle)", Style::default().fg(theme.muted)) } else { Span::raw("") },
                Span::raw(" "),
            ]))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(sub_border_color));
        let gpu_inner = gpu_block.inner(mid);
        f.render_widget(gpu_block, mid);
        render_labeled_sparkline(f, gpu_inner, "", s.power.gpu_w, &gpu_power_data, gpu_tdp, theme.gpu_accent, false);
    }

    // Bottom info inside panel
    let total_w = s.power.package_w.max(s.power.cpu_w + s.power.gpu_w + s.power.ane_w + s.power.dram_w);
    let avg_w = if !state.history.package_power.is_empty() {
        let sum: f64 = state.history.package_power.iter().sum();
        sum / state.history.package_power.len() as f64
    } else {
        total_w as f64
    };
    let max_w = state.history.package_power.iter().copied().fold(0.0_f64, f64::max);
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!(" Total {total_w:.1}W  Avg {avg_w:.1}W  Max {max_w:.1}W"),
            Style::default().fg(theme.muted),
        ))),
        Rect::new(inner.x, bottom_y, inner.width, 1),
    );
}
