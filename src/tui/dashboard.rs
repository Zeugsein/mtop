use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::tui::{AppState, theme, gradient, layout, expanded};
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
    // All header text uses muted styling (UAT-09)
    let muted_style = Style::default().fg(theme.muted);
    let header_spans = if header_text.len() <= avail {
        vec![
            Span::styled(&timestamp, muted_style),
            Span::styled(" \u{2014} ", muted_style),
            Span::styled("mtop", muted_style),
            Span::styled(" \u{2014} ", muted_style),
            Span::styled(chip_info, muted_style),
        ]
    } else {
        // Truncated: just "mtop — chip" or "mtop" if very narrow
        let short = format!("mtop \u{2014} {}", chip_info);
        if short.len() <= avail {
            vec![
                Span::styled("mtop", muted_style),
                Span::styled(" \u{2014} ", muted_style),
                Span::styled(chip_info, muted_style),
            ]
        } else {
            vec![Span::styled("mtop", muted_style)]
        }
    };
    let header_line = Line::from(header_spans).alignment(ratatui::layout::Alignment::Center);
    f.render_widget(Paragraph::new(header_line), page.header);

    // Battery gauge right-aligned in header
    // Grows from right, greener=full, redder=empty, same-hue gradient (lighter left, darker right)
    let bat = &s.battery;
    let bat_spans: Vec<Span> = if !bat.is_present {
        vec![Span::styled("\u{26a1} AC ", muted_style)]
    } else {
        let pct = bat.charge_pct as f64 / 100.0;
        let base_color = gradient::value_to_color(1.0 - pct, theme); // green=full, red=empty
        let mut spans = Vec::new();
        if bat.is_on_ac {
            spans.push(Span::styled("\u{26a1}", muted_style));
        }
        // 6-char gauge bar, fills from right
        let bar_width: usize = 6;
        let filled = (pct * bar_width as f64).round() as usize;
        let filled = filled.min(bar_width);
        let empty = bar_width - filled;
        // Empty portion (left side)
        if empty > 0 {
            spans.push(Span::styled(
                "\u{25a0}".repeat(empty),
                Style::default().fg(theme.muted),
            ));
        }
        // Filled portion (right side): lighter on left → darker on right
        if filled > 0 {
            let (br, bg, bb) = match base_color {
                Color::Rgb(r, g, b) => (r, g, b),
                _ => (100, 200, 100),
            };
            for i in 0..filled {
                let t = if filled > 1 { i as f64 / (filled - 1) as f64 } else { 1.0 };
                // Lighter (t=0) to darker (t=1): scale brightness from 1.3x down to 0.8x
                let scale = 1.3 - 0.5 * t;
                let r = (br as f64 * scale).round().min(255.0) as u8;
                let g = (bg as f64 * scale).round().min(255.0) as u8;
                let b = (bb as f64 * scale).round().min(255.0) as u8;
                spans.push(Span::styled(
                    "\u{25a0}".to_string(),
                    Style::default().fg(Color::Rgb(r, g, b)),
                ));
            }
        }
        spans.push(Span::styled(format!("{:.0}% ", bat.charge_pct), Style::default().fg(base_color)));
        spans
    };
    let bat_width: u16 = bat_spans.iter().map(|s| unicode_width::UnicodeWidthStr::width(&*s.content) as u16).sum();
    if page.header.width > bat_width {
        let bat_x = page.header.x + page.header.width - bat_width;
        f.render_widget(
            Paragraph::new(Line::from(bat_spans)),
            Rect::new(bat_x, page.header.y, bat_width, 1),
        );
    }

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

    // Footer: all right-aligned, help leftmost, interval rightmost
    let footer_muted = Style::default().fg(theme.muted);
    let theme_name = theme::THEMES[state.theme_idx].name;

    let footer_spans = vec![
        Span::styled("[?] help ", footer_muted),
        Span::styled(format!("[c] theme({}) ", theme_name), footer_muted),
        Span::styled("[.] detail ", footer_muted),
        Span::styled("[1-6] expand ", footer_muted),
        Span::styled(format!("[+/-] {}ms ", state.interval_ms), footer_muted),
    ];
    f.render_widget(
        Paragraph::new(Line::from(footer_spans).alignment(ratatui::layout::Alignment::Right)),
        page.footer,
    );

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

    let block = Block::default()
        .title(Line::from(Span::styled(" mtop \u{2014} keyboard shortcuts ", Style::default().fg(theme.accent).bold())))
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(theme.fg))
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
