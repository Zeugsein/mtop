use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::metrics::MetricsSnapshot;
use crate::tui::{AppState, theme, braille, gradient, layout};

/// GPU panel: Type A layout (75% braille sparkline + 25% orphan metrics)
pub(crate) fn draw_gpu_panel_v2(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    let gpu_pct = s.gpu.usage * 100.0;
    let temp_color = gradient::temp_to_color(s.temperature.gpu_avg_c);
    let temp_str = if s.temperature.available {
        format!("{}°C", s.temperature.gpu_avg_c as u32)
    } else {
        "N/A".to_string()
    };

    let title_spans = vec![
        Span::styled(" GPU  ", Style::default().fg(theme.gpu_accent).bold()),
        Span::styled(format!("{:.1}%", gpu_pct), Style::default().fg(theme.fg)),
        Span::styled(format!(" @ {}MHz", s.gpu.freq_mhz), Style::default().fg(theme.muted)),
        Span::styled(format!("  {:.1}W", s.power.gpu_w), Style::default().fg(theme.muted)),
        Span::styled(format!("  {}", temp_str), Style::default().fg(temp_color)),
        Span::raw(" "),
    ];

    let bottom_left = vec![
        Span::styled(format!(" {} cores", s.soc.gpu_cores), Style::default().fg(theme.muted)),
    ];
    let bottom_right = vec![
        Span::styled(format!("ANE {:.1}W ", s.power.ane_w), Style::default().fg(theme.muted)),
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

    // Left: GPU usage braille sparkline
    if s.gpu.available {
        let sparkline_data: Vec<f64> = state.history.gpu_usage.iter().copied().collect();
        let spark_width = trend_area.width as usize;
        let spark = braille::render_braille_sparkline(&sparkline_data, 1.0, spark_width);
        let spark_spans: Vec<Span> = spark
            .iter()
            .map(|&(ch, _)| Span::styled(ch.to_string(), Style::default().fg(theme.gpu_accent)))
            .collect();
        if !spark_spans.is_empty() {
            let y_offset = trend_area.height / 2;
            f.render_widget(
                Paragraph::new(Line::from(spark_spans)),
                Rect::new(trend_area.x, trend_area.y + y_offset, trend_area.width, 1),
            );
        }
    }

    // Right: orphan metrics
    let gb = 1024.0 * 1024.0 * 1024.0;
    let metrics = [
        format!("{} GPU cores", s.soc.gpu_cores),
        String::new(),
        format!("ANE  {:.1}W", s.power.ane_w),
        format!("DRAM {:.1}W", s.power.dram_w),
        String::new(),
        format!("Mem  {:.1}/{:.0}GB", s.memory.ram_used as f64 / gb, s.memory.ram_total as f64 / gb),
        format!("Swap {:.1}/{:.1}GB", s.memory.swap_used as f64 / gb, s.memory.swap_total as f64 / gb),
    ];

    for (i, text) in metrics.iter().enumerate() {
        let y = detail_area.y + i as u16;
        if y >= detail_area.y + detail_area.height || text.is_empty() {
            continue;
        }
        f.render_widget(
            Paragraph::new(text.as_str()).style(Style::default().fg(theme.fg)),
            Rect::new(detail_area.x, y, detail_area.width, 1),
        );
    }
}
