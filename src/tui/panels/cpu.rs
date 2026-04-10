use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::metrics::MetricsSnapshot;
use crate::tui::{AppState, theme, braille, gradient, layout};
use crate::tui::helpers::truncate_by_display_width;

/// Helper to render a multi-row braille graph into a frame area.
pub(crate) fn render_graph(f: &mut Frame, area: Rect, data: &[f64], max: f64, theme: &theme::Theme) {
    let graph = braille::render_braille_graph(data, max, area.width as usize, area.height as usize, theme);
    for (row_idx, row) in graph.iter().enumerate() {
        let y = area.y + area.height.saturating_sub(1) - row_idx as u16;
        if y < area.y {
            break;
        }
        let spans: Vec<Span> = row
            .iter()
            .map(|&(ch, color)| Span::styled(ch.to_string(), Style::default().fg(color)))
            .collect();
        if !spans.is_empty() {
            f.render_widget(Paragraph::new(Line::from(spans)), Rect::new(area.x, y, area.width, 1));
        }
    }
}

/// Render a braille graph with baseline dots: near-zero values are clamped to a floor
/// so at least 1 braille dot renders, colored with baseline_color instead of the gradient.
pub(crate) fn render_graph_with_baseline(f: &mut Frame, area: Rect, data: &[f64], max: f64, theme: &theme::Theme) {
    let baseline_floor = max * 0.005;
    let clamped: Vec<f64> = data.iter().map(|&v| v.max(baseline_floor)).collect();
    let graph = braille::render_braille_graph(&clamped, max, area.width as usize, area.height as usize, theme);

    let needed = area.width as usize * 2;
    let start = data.len().saturating_sub(needed);
    let visible_orig = &data[start..];

    for (row_idx, row) in graph.iter().enumerate() {
        let y = area.y + area.height.saturating_sub(1) - row_idx as u16;
        if y < area.y {
            break;
        }
        let spans: Vec<Span> = row.iter().enumerate().map(|(col, &(ch, orig_color))| {
            let orig_l = visible_orig.get(col * 2).copied().unwrap_or(0.0);
            let orig_r = visible_orig.get(col * 2 + 1).copied().unwrap_or(0.0);
            let is_baseline = orig_l < baseline_floor * 2.0 && orig_r < baseline_floor * 2.0;
            let color = if is_baseline { theme::baseline_color(theme) } else { orig_color };
            Span::styled(ch.to_string(), Style::default().fg(color))
        }).collect();
        if !spans.is_empty() {
            f.render_widget(Paragraph::new(Line::from(spans)), Rect::new(area.x, y, area.width, 1));
        }
    }
}

/// CPU panel: Type A layout (75% multi-row braille graph + 25% process dots)
pub(crate) fn draw_cpu_panel_v2(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    let cpu_pct = s.cpu.total_usage * 100.0;
    let temp_color = gradient::temp_to_color(s.temperature.cpu_avg_c, theme);
    let temp_str = if s.temperature.available {
        format!("{}°C", s.temperature.cpu_avg_c as u32)
    } else {
        "N/A".to_string()
    };

    let border_color = theme::dim_color(theme.cpu_accent, theme::adaptive_border_dim(theme));

    let title_spans = vec![
        Span::styled(format!(" {}", theme::PANEL_SUPERSCRIPTS[0]), Style::default().fg(theme.muted)),
        Span::styled("cpu  ", Style::default().fg(theme.fg).bold()),
        Span::styled(format!("{:.1}%", cpu_pct), Style::default().fg(theme.fg)),
        Span::styled(format!(" @ {}MHz", s.cpu.p_cluster.freq_mhz.max(s.cpu.e_cluster.freq_mhz)), Style::default().fg(theme.muted)),
        Span::styled(format!("  {:.1}W", s.power.cpu_w), Style::default().fg(theme.muted)),
        Span::styled(format!("  {}", temp_str), Style::default().fg(temp_color)),
        Span::raw(" "),
    ];

    let block = Block::default()
        .title(Line::from(title_spans))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .border_type(ratatui::widgets::BorderType::Rounded);

    let raw_inner = block.inner(area);
    f.render_widget(block, area);

    // 1-char padding left/right, no top padding (UAT-07)
    let inner = Rect::new(raw_inner.x + 1, raw_inner.y, raw_inner.width.saturating_sub(2), raw_inner.height);

    if inner.height < 2 || inner.width == 0 {
        return;
    }

    // Reserve last row for bottom info (inside panel)
    let content_area = Rect::new(inner.x, inner.y, inner.width, inner.height.saturating_sub(1));
    let bottom_y = inner.y + inner.height.saturating_sub(1);

    let sparkline_data: Vec<f64> = state.history.cpu_usage.iter().copied().collect();

    if state.show_detail {
        let (trend_area, detail_area) = layout::split_type_a(content_area);

        // Left: multi-row braille graph
        render_graph(f, trend_area, &sparkline_data, 1.0, theme);

        // Right: process list with dots — c/m/p labels aligned above dot columns
        let name_width = detail_area.width.saturating_sub(7) as usize;
        let legend = Line::from(vec![
            Span::raw(format!("{:>w$}", "", w = name_width + 2)),
            Span::styled("c", Style::default().fg(theme.muted)),
            Span::raw(" "),
            Span::styled("m", Style::default().fg(theme.muted)),
            Span::raw(" "),
            Span::styled("p", Style::default().fg(theme.muted)),
        ]);
        f.render_widget(Paragraph::new(legend), Rect::new(detail_area.x, detail_area.y, detail_area.width, 1));

        let max_procs = (detail_area.height as usize).saturating_sub(1);
        let max_mem = s.processes.iter().map(|p| p.mem_bytes).max().unwrap_or(1).max(1);

        for (i, proc) in s.processes.iter().take(max_procs).enumerate() {
            let y = detail_area.y + 1 + i as u16;
            if y >= detail_area.y + detail_area.height {
                break;
            }

            let name = truncate_by_display_width(&proc.name, name_width);

            let cpu_norm = (proc.cpu_pct / 100.0).clamp(0.0, 1.0) as f64;
            let mem_norm = (proc.mem_bytes as f64 / max_mem as f64).clamp(0.0, 1.0);
            let pow_norm = cpu_norm * 0.8;

            let line = Line::from(vec![
                Span::styled(format!("{:<w$}", name, w = name_width), Style::default().fg(theme.fg)),
                Span::raw("  "),
                Span::styled("•", Style::default().fg(gradient::value_to_color(cpu_norm, theme))),
                Span::raw(" "),
                Span::styled("•", Style::default().fg(gradient::value_to_color(mem_norm, theme))),
                Span::raw(" "),
                Span::styled("•", Style::default().fg(gradient::value_to_color(pow_norm, theme))),
            ]);
            f.render_widget(Paragraph::new(line), Rect::new(detail_area.x, y, detail_area.width, 1));
        }
    } else {
        // Full-width graph, no right detail
        render_graph(f, content_area, &sparkline_data, 1.0, theme);
    }

    // Bottom info: E and P both left-aligned
    let bottom_text = format!(
        " E: {:.0}% @ {}MHz  P: {:.0}% @ {}MHz",
        s.cpu.e_cluster.usage * 100.0, s.cpu.e_cluster.freq_mhz,
        s.cpu.p_cluster.usage * 100.0, s.cpu.p_cluster.freq_mhz,
    );
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(bottom_text, Style::default().fg(theme.muted)))),
        Rect::new(inner.x, bottom_y, inner.width, 1),
    );
}
