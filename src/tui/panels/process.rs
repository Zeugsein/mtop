use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::metrics::MetricsSnapshot;
use crate::tui::{AppState, theme, gradient};
use crate::tui::helpers::{truncate_with_ellipsis, sort_indices};

/// Process panel: sorted process list with color indicator dots
pub(crate) fn draw_process_panel_v2(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" Processes ", Style::default().fg(theme.fg).bold()),
            Span::styled(format!("({})", state.sort_mode.label()), Style::default().fg(theme.muted)),
            Span::raw(" "),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    // Legend row
    let legend = Line::from(vec![
        Span::styled("●", Style::default().fg(theme.cpu_accent)),
        Span::styled("c ", Style::default().fg(theme.muted)),
        Span::styled("●", Style::default().fg(theme.mem_accent)),
        Span::styled("m ", Style::default().fg(theme.muted)),
        Span::styled("●", Style::default().fg(theme.power_accent)),
        Span::styled("p", Style::default().fg(theme.muted)),
    ]);
    f.render_widget(Paragraph::new(legend), Rect::new(inner.x, inner.y, inner.width, 1));

    // Sort processes using current sort mode
    let procs = &s.processes;
    let max_cpu = procs.iter().map(|p| p.cpu_pct).fold(0.0f32, f32::max);
    let max_mem = procs.iter().map(|p| p.mem_bytes).max().unwrap_or(1).max(1);
    let max_power = procs.iter().map(|p| p.power_w).fold(0.0f32, f32::max);

    let mut indices: Vec<usize> = (0..procs.len()).collect();
    sort_indices(&mut indices, procs, state.sort_mode, max_cpu, max_mem, max_power);

    // Empty state
    if indices.is_empty() {
        let y = inner.y + 1;
        if y < inner.y + inner.height {
            let line = Line::from(Span::styled("No processes", Style::default().fg(theme.muted)));
            f.render_widget(Paragraph::new(line), Rect::new(inner.x, y, inner.width, 1));
        }
        return;
    }

    // Scroll support
    let scroll = state.process_scroll.min(indices.len().saturating_sub(1));
    let max_visible = inner.height.saturating_sub(1) as usize;

    let name_width = inner.width.saturating_sub(7) as usize;

    for (i, &idx) in indices.iter().skip(scroll).take(max_visible).enumerate() {
        let proc = &procs[idx];
        let y = inner.y + 1 + i as u16;
        if y >= inner.y + inner.height {
            break;
        }

        let name = truncate_with_ellipsis(&proc.name, name_width);

        let cpu_norm = if max_cpu > 0.0 {
            (proc.cpu_pct / max_cpu).clamp(0.0, 1.0) as f64
        } else {
            0.0
        };
        let mem_norm = (proc.mem_bytes as f64 / max_mem as f64).clamp(0.0, 1.0);
        let power_norm = if max_power > 0.0 {
            (proc.power_w / max_power).clamp(0.0, 1.0) as f64
        } else {
            0.0
        };

        let line = Line::from(vec![
            Span::styled(format!("{:<w$}", name, w = name_width), Style::default().fg(theme.fg)),
            Span::raw(" "),
            Span::styled("●", Style::default().fg(gradient::value_to_color(cpu_norm))),
            Span::styled("●", Style::default().fg(gradient::value_to_color(mem_norm))),
            Span::styled("●", Style::default().fg(gradient::value_to_color(power_norm))),
        ]);
        f.render_widget(Paragraph::new(line), Rect::new(inner.x, y, inner.width, 1));
    }
}
