use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::metrics::MetricsSnapshot;
use crate::tui::{AppState, theme, braille, gradient, layout};
use crate::tui::helpers::truncate_with_ellipsis;

/// CPU panel: Type A layout (75% braille sparkline + 25% process dots)
pub(crate) fn draw_cpu_panel_v2(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    let cpu_pct = s.cpu.total_usage * 100.0;
    let temp_color = gradient::temp_to_color(s.temperature.cpu_avg_c);
    let temp_str = if s.temperature.available {
        format!("{}°C", s.temperature.cpu_avg_c as u32)
    } else {
        "N/A".to_string()
    };

    let title_spans = vec![
        Span::styled(" CPU  ", Style::default().fg(theme.cpu_accent).bold()),
        Span::styled(format!("{:.1}%", cpu_pct), Style::default().fg(theme.fg)),
        Span::styled(format!(" @ {}MHz", s.cpu.p_cluster.freq_mhz.max(s.cpu.e_cluster.freq_mhz)), Style::default().fg(theme.muted)),
        Span::styled(format!("  {:.1}W", s.power.cpu_w), Style::default().fg(theme.muted)),
        Span::styled(format!("  {}", temp_str), Style::default().fg(temp_color)),
        Span::raw(" "),
    ];

    let block = Block::default()
        .title(Line::from(title_spans))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .border_type(ratatui::widgets::BorderType::Rounded);

    let bottom_spans = vec![
        Span::styled(
            format!(" E: {:.0}% @ {}MHz", s.cpu.e_cluster.usage * 100.0, s.cpu.e_cluster.freq_mhz),
            Style::default().fg(theme.muted),
        ),
    ];
    let block = block.title_bottom(Line::from(bottom_spans).alignment(ratatui::layout::Alignment::Left));

    let p_spans = vec![
        Span::styled(
            format!("P: {:.0}% @ {}MHz ", s.cpu.p_cluster.usage * 100.0, s.cpu.p_cluster.freq_mhz),
            Style::default().fg(theme.muted),
        ),
    ];
    let block = block.title_bottom(Line::from(p_spans).alignment(ratatui::layout::Alignment::Right));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let (trend_area, detail_area) = layout::split_type_a(inner);

    // Left: braille sparkline
    let sparkline_data: Vec<f64> = state.history.cpu_usage.iter().copied().collect();
    let spark_width = trend_area.width as usize;
    let spark = braille::render_braille_sparkline(&sparkline_data, 1.0, spark_width);

    let spark_spans: Vec<Span> = spark
        .iter()
        .map(|&(ch, color)| Span::styled(ch.to_string(), Style::default().fg(color)))
        .collect();

    if !spark_spans.is_empty() {
        let y_offset = trend_area.height / 2;
        let spark_rect = Rect::new(trend_area.x, trend_area.y + y_offset, trend_area.width, 1);
        f.render_widget(Paragraph::new(Line::from(spark_spans)), spark_rect);
    }

    // Right: process list with colored dots
    let legend = Line::from(vec![
        Span::styled("●", Style::default().fg(theme.cpu_accent)),
        Span::styled("c ", Style::default().fg(theme.muted)),
        Span::styled("●", Style::default().fg(theme.mem_accent)),
        Span::styled("m ", Style::default().fg(theme.muted)),
        Span::styled("●", Style::default().fg(theme.power_accent)),
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

        let name_width = detail_area.width.saturating_sub(7) as usize;
        let name = truncate_with_ellipsis(&proc.name, name_width);

        let cpu_norm = (proc.cpu_pct / 100.0).clamp(0.0, 1.0) as f64;
        let mem_norm = (proc.mem_bytes as f64 / max_mem as f64).clamp(0.0, 1.0);
        let pow_norm = cpu_norm * 0.8;

        let line = Line::from(vec![
            Span::styled(format!("{:<w$}", name, w = name_width), Style::default().fg(theme.fg)),
            Span::raw(" "),
            Span::styled("●", Style::default().fg(gradient::value_to_color(cpu_norm))),
            Span::styled("●", Style::default().fg(gradient::value_to_color(mem_norm))),
            Span::styled("●", Style::default().fg(gradient::value_to_color(pow_norm))),
        ]);
        f.render_widget(Paragraph::new(line), Rect::new(detail_area.x, y, detail_area.width, 1));
    }
}
