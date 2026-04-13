use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::metrics::MetricsSnapshot;
use crate::tui::{AppState, theme, gradient, layout};
use super::cpu::render_graph_with_baseline;

/// GPU panel: Type A layout (75% multi-row braille graph + 25% orphan metrics)
pub(crate) fn draw_gpu_panel_v2(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    let gpu_pct = s.gpu.usage * 100.0;
    let temp_color = gradient::temp_to_color(s.temperature.gpu_avg_c, theme);
    let temp_str = if s.temperature.available {
        format!("{}°C", s.temperature.gpu_avg_c as u32)
    } else {
        "N/A".to_string()
    };

    let border_color = theme::dim_color(theme.gpu_accent, theme::adaptive_border_dim(theme));

    let gpu_idle = s.power.gpu_w < 0.5;

    let mut title_spans = vec![
        Span::styled(format!(" {}", theme::PANEL_SUPERSCRIPTS[1]), Style::default().fg(theme.muted)),
        Span::styled("gpu ", Style::default().fg(theme.fg).bold()),
    ];
    if gpu_idle {
        title_spans.push(Span::styled("(idle) ", Style::default().fg(theme.muted)));
    } else {
        title_spans.push(Span::styled(format!("{:.1}%", gpu_pct), Style::default().fg(theme.fg)));
        title_spans.push(Span::styled(format!(" @ {}MHz", s.gpu.freq_mhz), Style::default().fg(theme.muted)));
        title_spans.push(Span::styled(format!("  {:.1}W", s.power.gpu_w), Style::default().fg(theme.muted)));
    }
    title_spans.push(Span::styled(format!("  {}", temp_str), Style::default().fg(temp_color)));
    title_spans.push(Span::raw(" "));

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

    // Reserve last row for bottom info
    let content_area = Rect::new(inner.x, inner.y, inner.width, inner.height.saturating_sub(1));
    let bottom_y = inner.y + inner.height.saturating_sub(1);

    if state.show_detail {
        let (trend_area, detail_area) = layout::split_type_a(content_area);

        // Left: GPU usage multi-row braille graph
        if s.gpu.available {
            let sparkline_data: Vec<f64> = state.history.gpu_usage.iter().copied().collect();
            render_graph_with_baseline(f, trend_area, &sparkline_data, 1.0, theme);
        }

        // Right: orphan metrics (vertically centered, white text)
        let gb = 1024.0 * 1024.0 * 1024.0;
        let metrics: Vec<String> = vec![
            format!("ane  {:.1}W", s.power.ane_w),
            format!("dram {:.1}W", s.power.dram_w),
            String::new(),
            format!("vram {:.1}/{:.0}GB", s.memory.ram_used as f64 / gb, s.memory.ram_total as f64 / gb),
        ];

        let content_lines = metrics.iter().filter(|m| !m.is_empty()).count() + metrics.iter().filter(|m| m.is_empty()).count();
        let content_lines = content_lines.min(detail_area.height as usize);
        let y_offset = (detail_area.height as usize).saturating_sub(content_lines) / 2;

        for (i, text) in metrics.iter().enumerate() {
            let y = detail_area.y + y_offset as u16 + i as u16;
            if y >= detail_area.y + detail_area.height || text.is_empty() {
                if text.is_empty() {
                    continue;
                }
                break;
            }
            f.render_widget(
                Paragraph::new(text.as_str()).style(Style::default().fg(theme.fg)),
                Rect::new(detail_area.x, y, detail_area.width, 1),
            );
        }
    } else {
        // Full-width graph, no right detail
        if s.gpu.available {
            let sparkline_data: Vec<f64> = state.history.gpu_usage.iter().copied().collect();
            render_graph_with_baseline(f, content_area, &sparkline_data, 1.0, theme);
        }

    }

    // Bottom info inside panel
    let bottom_left = Span::styled(
        format!(" {} cores", s.soc.gpu_cores),
        Style::default().fg(theme.muted),
    );
    let bottom_right = Span::styled(
        format!("ane {:.1}W ", s.power.ane_w),
        Style::default().fg(theme.muted),
    );
    f.render_widget(
        Paragraph::new(Line::from(bottom_left)),
        Rect::new(inner.x, bottom_y, inner.width / 2, 1),
    );
    f.render_widget(
        Paragraph::new(Line::from(bottom_right).alignment(ratatui::layout::Alignment::Right)),
        Rect::new(inner.x, bottom_y, inner.width, 1),
    );
}
