use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::metrics::MetricsSnapshot;
use crate::tui::{AppState, theme, gradient};
use crate::tui::helpers::{truncate_by_display_width, pad_to_display_width, sort_indices};

// Fixed column widths for numeric columns
const COL_PID: usize = 6;
const COL_CPU: usize = 5;
const COL_MEM: usize = 5;
const COL_POW: usize = 5;
const COL_THR: usize = 7;
// +5 for spaces, +3 for colored dots before cpu/mem/pow columns
const COL_FIXED_TOTAL: usize = COL_PID + COL_CPU + COL_MEM + COL_POW + COL_THR + 5 + 3;

/// Process panel: sorted process table with fixed-position columns
pub(crate) fn draw_process_panel_v2(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    let border_color = theme::dim_color(theme.process_accent, theme::adaptive_border_dim(theme));

    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(format!(" {}", theme::PANEL_SUPERSCRIPTS[5]), Style::default().fg(theme.muted)),
            Span::styled("proc ", Style::default().fg(theme.fg).bold()),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let raw_inner = block.inner(area);
    f.render_widget(block, area);

    // 1-char padding left/right, no top padding (UAT-07)
    let inner = Rect::new(
        raw_inner.x + 1,
        raw_inner.y,
        raw_inner.width.saturating_sub(2),
        raw_inner.height,
    );

    if inner.width == 0 || inner.height < 3 {
        return;
    }

    let panel_width = inner.width as usize;
    let name_width = panel_width.saturating_sub(COL_FIXED_TOTAL).max(4);

    // Header row: pid first, no dots on headers
    let header = Line::from(vec![
        Span::styled(format!("{:<w$}", "pid", w = COL_PID), Style::default().fg(theme.muted)),
        Span::styled(" ", Style::default()),
        Span::styled(
            pad_to_display_width("name", name_width),
            Style::default().fg(theme.muted),
        ),
        Span::styled(format!("{:>w$}", "cpu", w = COL_CPU + 2), Style::default().fg(theme.muted)),
        Span::styled(format!("{:>w$}", "mem", w = COL_MEM + 2), Style::default().fg(theme.muted)),
        Span::styled(format!("{:>w$}", "pow", w = COL_POW + 2), Style::default().fg(theme.muted)),
        Span::styled(format!("{:>w$}", "thread", w = COL_THR), Style::default().fg(theme.muted)),
    ]);
    f.render_widget(Paragraph::new(header), Rect::new(inner.x, inner.y, inner.width, 1));

    // Sort processes
    let procs = &s.processes;
    let max_cpu = procs.iter().map(|p| p.cpu_pct).fold(0.0f32, f32::max);
    let max_mem = procs.iter().map(|p| p.mem_bytes).max().unwrap_or(1).max(1);
    let max_power = procs.iter().map(|p| p.power_w).fold(0.0f32, f32::max);

    let mut indices: Vec<usize> = (0..procs.len()).collect();
    sort_indices(&mut indices, procs, state.sort_mode, max_cpu, max_mem, max_power);

    if indices.is_empty() {
        let y = inner.y + 1;
        if y < inner.y + inner.height {
            f.render_widget(
                Paragraph::new("No processes").style(Style::default().fg(theme.muted)),
                Rect::new(inner.x, y, inner.width, 1),
            );
        }
        return;
    }

    // Scroll support — reserve 1 row for header, 1 for sort indicator
    let scroll = state.process_scroll.min(indices.len().saturating_sub(1));
    let max_visible = inner.height.saturating_sub(2) as usize; // header + sort line

    let gb = 1024.0 * 1024.0 * 1024.0;
    let mb = 1024.0 * 1024.0;

    for (i, &idx) in indices.iter().skip(scroll).take(max_visible).enumerate() {
        let proc = &procs[idx];
        let y = inner.y + 1 + i as u16;
        if y >= inner.y + inner.height.saturating_sub(1) {
            break;
        }

        // Name: CJK-aware truncation and padding
        let name_trunc = truncate_by_display_width(&proc.name, name_width);
        let name_padded = pad_to_display_width(&name_trunc, name_width);

        // Memory display
        let mem_str = if proc.mem_bytes as f64 >= gb {
            format!("{:.1}G", proc.mem_bytes as f64 / gb)
        } else {
            format!("{:.0}M", proc.mem_bytes as f64 / mb)
        };

        // Gradient dot colors (matching CPU panel chart colors)
        let cpu_norm = if max_cpu > 0.0 { (proc.cpu_pct / max_cpu).clamp(0.0, 1.0) as f64 } else { 0.0 };
        let mem_norm = if max_mem > 0 { (proc.mem_bytes as f64 / max_mem as f64).clamp(0.0, 1.0) } else { 0.0 };
        let pow_norm = if max_power > 0.0 { (proc.power_w / max_power).clamp(0.0, 1.0) as f64 } else { 0.0 };

        let cpu_dot_color = if proc.cpu_pct < 0.1 { theme.muted } else { gradient::value_to_color(cpu_norm, theme) };
        let mem_dot_color = if proc.mem_bytes < 1_048_576 { theme.muted } else { gradient::value_to_color(mem_norm, theme) };
        let pow_dot_color = if proc.power_w < 0.1 { theme.muted } else { gradient::value_to_color(pow_norm, theme) };

        let line = Line::from(vec![
            Span::styled(format!("{:<w$}", proc.pid, w = COL_PID), Style::default().fg(theme.muted)),
            Span::styled(" ", Style::default()),
            Span::styled(name_padded, Style::default().fg(theme.fg)),
            Span::styled(" \u{2022}", Style::default().fg(cpu_dot_color)),
            Span::styled(format!("{:>w$.1}", proc.cpu_pct, w = COL_CPU), Style::default().fg(theme.fg)),
            Span::styled(" \u{2022}", Style::default().fg(mem_dot_color)),
            Span::styled(format!("{:>w$}", mem_str, w = COL_MEM), Style::default().fg(theme.fg)),
            Span::styled(" \u{2022}", Style::default().fg(pow_dot_color)),
            Span::styled(format!("{:>w$.1}", proc.power_w, w = COL_POW), Style::default().fg(theme.fg)),
            Span::styled(format!("{:>w$}", proc.thread_count, w = COL_THR), Style::default().fg(theme.fg)),
        ]);
        f.render_widget(Paragraph::new(line), Rect::new(inner.x, y, inner.width, 1));
    }

    // Sort indicator at bottom
    let sort_y = inner.y + inner.height.saturating_sub(1);
    let sort_text = format!("sort: {} \u{2193}", state.sort_mode.label());
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(sort_text, Style::default().fg(theme.muted)))
            .alignment(ratatui::layout::Alignment::Right)),
        Rect::new(inner.x, sort_y, inner.width, 1),
    );
}
