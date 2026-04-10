use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::tui::{AppState, theme, layout, expanded};
use crate::tui::panels::*;

pub(crate) fn draw_dashboard(f: &mut Frame, state: &AppState) {
    let theme = &theme::THEMES[state.theme_idx];
    let s = &state.snapshot;
    let area = f.area();

    // Terminal too-small check
    if layout::terminal_too_small(area) {
        let msg = layout::too_small_message(area);
        let para = Paragraph::new(msg)
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().fg(theme.fg));
        let v = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(45),
                Constraint::Length(1),
                Constraint::Percentage(45),
            ])
            .split(area);
        f.render_widget(para, v[1]);
        return;
    }

    // Two-column page layout
    let page = layout::split_page(area);

    // Header: centered "timestamp — mtop — chip info"
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let chip_info = format!("{} ({}E+{}P+{}GPU {}GB)",
        s.soc.chip, s.soc.e_cores, s.soc.p_cores,
        s.soc.gpu_cores, s.soc.memory_gb);
    let header_text = format!("{} \u{2014} mtop \u{2014} {}", timestamp, chip_info);
    let avail = page.header.width as usize;
    let header_spans = if header_text.len() <= avail {
        vec![
            Span::styled(&timestamp, Style::default().fg(theme.muted)),
            Span::styled(" \u{2014} ", Style::default().fg(theme.muted)),
            Span::styled("mtop", Style::default().fg(theme.accent).bold()),
            Span::styled(" \u{2014} ", Style::default().fg(theme.muted)),
            Span::styled(chip_info, Style::default().fg(theme.fg)),
        ]
    } else {
        // Truncated: just "mtop — chip" or "mtop" if very narrow
        let short = format!("mtop \u{2014} {}", chip_info);
        if short.len() <= avail {
            vec![
                Span::styled("mtop", Style::default().fg(theme.accent).bold()),
                Span::styled(" \u{2014} ", Style::default().fg(theme.muted)),
                Span::styled(chip_info, Style::default().fg(theme.fg)),
            ]
        } else {
            vec![Span::styled("mtop", Style::default().fg(theme.accent).bold())]
        }
    };
    let header_line = Line::from(header_spans).alignment(ratatui::layout::Alignment::Center);
    f.render_widget(Paragraph::new(header_line), page.header);

    // Expand/collapse layout
    match state.expanded_panel {
        Some(panel) if panel.is_left_column() => {
            expanded::draw_expanded_panel(f, page.left_column, panel, s, state, theme);
            let (r1, r2, r3) = layout::split_column_3(page.right_column);
            draw_network_panel_v2(f, r1, s, state, theme);
            draw_power_panel_v2(f, r2, s, state, theme);
            draw_process_panel_v2(f, r3, s, state, theme);
        }
        Some(panel) => {
            let (l1, l2, l3) = layout::split_column_3(page.left_column);
            draw_cpu_panel_v2(f, l1, s, state, theme);
            draw_gpu_panel_v2(f, l2, s, state, theme);
            draw_mem_disk_panel_v2(f, l3, s, state, theme);
            expanded::draw_expanded_panel(f, page.right_column, panel, s, state, theme);
        }
        None => {
            let (l1, l2, l3) = layout::split_column_3(page.left_column);
            draw_cpu_panel_v2(f, l1, s, state, theme);
            draw_gpu_panel_v2(f, l2, s, state, theme);
            draw_mem_disk_panel_v2(f, l3, s, state, theme);

            let (r1, r2, r3) = layout::split_column_3(page.right_column);
            draw_network_panel_v2(f, r1, s, state, theme);
            draw_power_panel_v2(f, r2, s, state, theme);
            draw_process_panel_v2(f, r3, s, state, theme);
        }
    }

    // Footer: right-aligned keybinding hints
    let theme_name = theme::THEMES[state.theme_idx].name;
    let footer_spans = vec![
        Span::styled("[", Style::default().fg(theme.accent)),
        Span::styled("c", Style::default().fg(theme.accent).bold()),
        Span::styled("] ", Style::default().fg(theme.accent)),
        Span::styled(format!("theme({}) ", theme_name), Style::default().fg(theme.muted)),
        Span::styled("[", Style::default().fg(theme.accent)),
        Span::styled(".", Style::default().fg(theme.accent).bold()),
        Span::styled("] ", Style::default().fg(theme.accent)),
        Span::styled("detail ", Style::default().fg(theme.muted)),
        Span::styled("[", Style::default().fg(theme.accent)),
        Span::styled("?", Style::default().fg(theme.accent).bold()),
        Span::styled("] ", Style::default().fg(theme.accent)),
        Span::styled("help ", Style::default().fg(theme.muted)),
    ];
    let footer = Paragraph::new(Line::from(footer_spans).alignment(ratatui::layout::Alignment::Right));
    f.render_widget(footer, page.footer);

    // Help overlay (rendered last, on top of everything)
    if state.show_help {
        draw_help_overlay(f, area, theme);
    }
}

fn draw_help_overlay(f: &mut Frame, area: Rect, theme: &theme::Theme) {
    let overlay_w: u16 = 48.min(area.width.saturating_sub(4));
    let overlay_h: u16 = 16.min(area.height.saturating_sub(2));
    let x = (area.width.saturating_sub(overlay_w)) / 2;
    let y = (area.height.saturating_sub(overlay_h)) / 2;
    let overlay_area = Rect::new(x, y, overlay_w, overlay_h);

    // Clear area behind overlay
    f.render_widget(Clear, overlay_area);

    let border_color = theme::dim_color(theme.accent, 0.5);
    let block = Block::default()
        .title(Line::from(Span::styled(" mtop \u{2014} keyboard shortcuts ", Style::default().fg(theme.accent).bold())))
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(overlay_area);
    f.render_widget(block, overlay_area);

    let help_lines = vec![
        ("q / Esc", "quit"),
        ("h / ?", "toggle help"),
        (".", "toggle right details"),
        ("c", "cycle theme"),
        ("s", "cycle sort column"),
        ("1\u{2013}6", "expand/collapse panel"),
        ("\u{2191}/k  \u{2193}/j", "scroll process list"),
        ("+/-", "adjust interval"),
        ("w", "save config"),
    ];

    for (i, (key, desc)) in help_lines.iter().enumerate() {
        let row_y = inner.y + i as u16;
        if row_y >= inner.y + inner.height {
            break;
        }
        let line = Line::from(vec![
            Span::styled(format!("  {:<14}", key), Style::default().fg(theme.accent)),
            Span::styled(*desc, Style::default().fg(theme.fg)),
        ]);
        f.render_widget(Paragraph::new(line), Rect::new(inner.x, row_y, inner.width, 1));
    }
}
