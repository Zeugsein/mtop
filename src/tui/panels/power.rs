use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::metrics::MetricsSnapshot;
use crate::tui::{AppState, theme, braille, gradient, layout};
use crate::tui::helpers::truncate_with_ellipsis;

/// Power panel: Type B layout (37.5% CPU sparkline + 37.5% GPU sparkline + 25% per-process energy)
pub(crate) fn draw_power_panel_v2(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    if !s.power.available {
        let block = Block::default()
            .title(" Power ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
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
        Span::styled(" Power  ", Style::default().fg(theme.power_accent).bold()),
        Span::styled(format!("CPU {:.1}W", s.power.cpu_w), Style::default().fg(theme.cpu_accent)),
        Span::styled("  ", Style::default()),
        Span::styled(format!("GPU {:.1}W", s.power.gpu_w), Style::default().fg(theme.gpu_accent)),
        Span::raw(" "),
    ];

    let total_w = s.power.package_w.max(s.power.cpu_w + s.power.gpu_w + s.power.ane_w + s.power.dram_w);
    let avg_w = if !state.history.package_power.is_empty() {
        let sum: f64 = state.history.package_power.iter().sum();
        sum / state.history.package_power.len() as f64
    } else {
        total_w as f64
    };
    let max_w = state.history.package_power.iter().copied().fold(0.0_f64, f64::max);

    let bottom_spans = vec![
        Span::styled(
            format!(" Total {total_w:.1}W  Avg {avg_w:.1}W  Max {max_w:.1}W "),
            Style::default().fg(theme.muted),
        ),
    ];

    let block = Block::default()
        .title(Line::from(title_spans))
        .title_bottom(Line::from(bottom_spans).alignment(ratatui::layout::Alignment::Left))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .border_type(ratatui::widgets::BorderType::Rounded);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let (left, mid, right) = layout::split_type_b(inner);

    // Left 37.5%: CPU power sparkline
    let cpu_tdp = s.soc.cpu_tdp_w() as f64;
    let cpu_power_data: Vec<f64> = state.history.cpu_power.iter().copied().collect();
    let cpu_spark = braille::render_braille_sparkline(&cpu_power_data, cpu_tdp, left.width as usize);
    let cpu_spark_spans: Vec<Span> = cpu_spark
        .iter()
        .map(|&(ch, color)| Span::styled(ch.to_string(), Style::default().fg(color)))
        .collect();
    if !cpu_spark_spans.is_empty() {
        let y_offset = left.height / 2;
        f.render_widget(
            Paragraph::new(Line::from(cpu_spark_spans)),
            Rect::new(left.x, left.y + y_offset, left.width, 1),
        );
    }
    f.render_widget(
        Paragraph::new("CPU").style(Style::default().fg(theme.cpu_accent)),
        Rect::new(left.x, left.y, left.width, 1),
    );

    // Middle 37.5%: GPU power sparkline
    let gpu_tdp = s.soc.gpu_tdp_w() as f64;
    let gpu_power_data: Vec<f64> = state.history.gpu_power.iter().copied().collect();
    let gpu_spark = braille::render_braille_sparkline(&gpu_power_data, gpu_tdp, mid.width as usize);
    let gpu_spark_spans: Vec<Span> = gpu_spark
        .iter()
        .map(|&(ch, color)| Span::styled(ch.to_string(), Style::default().fg(color)))
        .collect();
    if !gpu_spark_spans.is_empty() {
        let y_offset = mid.height / 2;
        f.render_widget(
            Paragraph::new(Line::from(gpu_spark_spans)),
            Rect::new(mid.x, mid.y + y_offset, mid.width, 1),
        );
    }
    f.render_widget(
        Paragraph::new("GPU").style(Style::default().fg(theme.gpu_accent)),
        Rect::new(mid.x, mid.y, mid.width, 1),
    );

    // Right 25%: Per-process energy ranking
    let mut procs_by_power: Vec<&crate::metrics::ProcessInfo> = s.processes.iter()
        .filter(|p| p.power_w > 0.0)
        .collect();
    procs_by_power.sort_by(|a, b| b.power_w.partial_cmp(&a.power_w).unwrap_or(std::cmp::Ordering::Equal));

    let max_power = procs_by_power.first().map(|p| p.power_w).unwrap_or(1.0).max(0.01);
    let max_rows = right.height.saturating_sub(1) as usize;

    for (i, proc) in procs_by_power.iter().take(max_rows).enumerate() {
        let y = right.y + i as u16;
        if y >= right.y + right.height.saturating_sub(1) {
            break;
        }

        let name_width = right.width.saturating_sub(8) as usize;
        let name = truncate_with_ellipsis(&proc.name, name_width);
        let power_norm = (proc.power_w / max_power).clamp(0.0, 1.0) as f64;

        let line = Line::from(vec![
            Span::styled(format!("{:<w$}", name, w = name_width), Style::default().fg(theme.fg)),
            Span::raw(" "),
            Span::styled("●", Style::default().fg(gradient::value_to_color(power_norm))),
            Span::styled(format!("{:.1}W", proc.power_w), Style::default().fg(theme.muted)),
        ]);
        f.render_widget(Paragraph::new(line), Rect::new(right.x, y, right.width, 1));
    }

    if right.height > 1 {
        let note_y = right.y + right.height - 1;
        f.render_widget(
            Paragraph::new("(user procs)").style(Style::default().fg(theme.muted)),
            Rect::new(right.x, note_y, right.width, 1),
        );
    }
}
