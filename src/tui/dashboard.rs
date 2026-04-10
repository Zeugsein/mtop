use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::tui::{AppState, theme, layout, expanded};
use crate::tui::panels::*;

pub(crate) fn draw_dashboard(f: &mut Frame, state: &AppState) {
    let theme = theme::THEMES[state.theme_idx];
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

    // Header: plain white text, no background color
    // Format: mtop — {chip} ({E}E+{P}P+{GPU}GPU {RAM}GB)  centered
    let header_text = format!(
        "mtop \u{2014} {} ({}E+{}P+{}GPU {}GB)",
        s.soc.chip, s.soc.e_cores, s.soc.p_cores,
        s.soc.gpu_cores, s.soc.memory_gb
    );
    let header = Paragraph::new(header_text)
        .alignment(ratatui::layout::Alignment::Center)
        .style(Style::default().fg(theme.fg));
    f.render_widget(header, page.header);

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

    // Footer: minimal, just theme name and interval
    let theme_name = theme::THEMES[state.theme_idx].name;
    let footer = Paragraph::new(format!(
        " q:quit  c:theme({theme_name})  1-6:expand  +/-:interval({}ms)  j/k:scroll  s:sort ",
        state.interval_ms
    ))
    .style(Style::default().fg(theme.muted));
    f.render_widget(footer, page.footer);
}
