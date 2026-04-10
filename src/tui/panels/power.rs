use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::metrics::MetricsSnapshot;
use crate::tui::{AppState, theme, braille, layout};
use crate::tui::helpers::truncate_with_ellipsis;

/// Power panel: Type B layout (37.5% CPU sparkline + 37.5% GPU sparkline + 25% per-process energy)
pub(crate) fn draw_power_panel_v2(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    let border_color = theme::dim_color(theme.power_accent, 0.4);

    if !s.power.available {
        let block = Block::default()
            .title(" power ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .border_type(ratatui::widgets::BorderType::Rounded);
        let inner = block.inner(area);
        f.render_widget(block, area);
        f.render_widget(
            Paragraph::new("Power sensors: N/A").style(Style::default().fg(theme.muted)),
            inner,
        );
        return;
    }

    let title_spans = vec![
        Span::styled(" power ", Style::default().fg(theme.power_accent).bold()),
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

    let (left, mid, right) = layout::split_type_b(content_area);

    // Left 37.5%: CPU power sparkline with label inside panel
    let cpu_tdp = s.soc.cpu_tdp_w() as f64;
    let cpu_power_data: Vec<f64> = state.history.cpu_power.iter().copied().collect();

    // Label: "cpu X.XW" at top of sparkline area
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("cpu ", Style::default().fg(theme.cpu_accent)),
            Span::styled(format!("{:.1}W", s.power.cpu_w), Style::default().fg(theme.fg)),
        ])),
        Rect::new(left.x, left.y, left.width, 1),
    );
    // Graph below label
    if left.height > 1 {
        let graph_area = Rect::new(left.x, left.y + 1, left.width, left.height - 1);
        let graph = braille::render_braille_graph(&cpu_power_data, cpu_tdp, graph_area.width as usize, graph_area.height as usize);
        for (row_idx, row) in graph.iter().enumerate() {
            let y = graph_area.y + graph_area.height.saturating_sub(1) - row_idx as u16;
            if y < graph_area.y { break; }
            let spans: Vec<Span> = row.iter().map(|&(ch, color)| Span::styled(ch.to_string(), Style::default().fg(color))).collect();
            if !spans.is_empty() {
                f.render_widget(Paragraph::new(Line::from(spans)), Rect::new(graph_area.x, y, graph_area.width, 1));
            }
        }
    }

    // Middle 37.5%: GPU power sparkline with label
    let gpu_tdp = s.soc.gpu_tdp_w() as f64;
    let gpu_power_data: Vec<f64> = state.history.gpu_power.iter().copied().collect();

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("gpu ", Style::default().fg(theme.gpu_accent)),
            Span::styled(format!("{:.1}W", s.power.gpu_w), Style::default().fg(theme.fg)),
        ])),
        Rect::new(mid.x, mid.y, mid.width, 1),
    );
    if mid.height > 1 {
        let graph_area = Rect::new(mid.x, mid.y + 1, mid.width, mid.height - 1);
        let graph = braille::render_braille_graph(&gpu_power_data, gpu_tdp, graph_area.width as usize, graph_area.height as usize);
        for (row_idx, row) in graph.iter().enumerate() {
            let y = graph_area.y + graph_area.height.saturating_sub(1) - row_idx as u16;
            if y < graph_area.y { break; }
            let spans: Vec<Span> = row.iter().map(|&(ch, color)| Span::styled(ch.to_string(), Style::default().fg(color))).collect();
            if !spans.is_empty() {
                f.render_widget(Paragraph::new(Line::from(spans)), Rect::new(graph_area.x, y, graph_area.width, 1));
            }
        }
    }

    // Right 25%: Per-process energy ranking (white text)
    let mut procs_by_power: Vec<&crate::metrics::ProcessInfo> = s.processes.iter()
        .filter(|p| p.power_w > 0.0)
        .collect();
    procs_by_power.sort_by(|a, b| b.power_w.partial_cmp(&a.power_w).unwrap_or(std::cmp::Ordering::Equal));

    let max_rows = right.height as usize;

    for (i, proc) in procs_by_power.iter().take(max_rows) .enumerate() {
        let y = right.y + i as u16;
        if y >= right.y + right.height {
            break;
        }

        let name_width = right.width.saturating_sub(6) as usize;
        let name = truncate_with_ellipsis(&proc.name, name_width);

        let line = Line::from(vec![
            Span::styled(format!("{:<w$}", name, w = name_width), Style::default().fg(theme.fg)),
            Span::raw(" "),
            Span::styled(format!("{:.1}W", proc.power_w), Style::default().fg(theme.fg)),
        ]);
        f.render_widget(Paragraph::new(line), Rect::new(right.x, y, right.width, 1));
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
