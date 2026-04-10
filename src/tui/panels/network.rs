use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::metrics::MetricsSnapshot;
use crate::platform::network::speed_tier_from_baudrate;
use crate::tui::{AppState, theme, braille, layout};
use crate::tui::helpers::{format_bytes_rate, format_bytes_rate_compact, format_bytes_compact, is_infrastructure_interface};

/// Network panel: Type B layout (37.5% upload + 37.5% download + 25% interface ranking)
pub(crate) fn draw_network_panel_v2(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    let (total_rx, total_tx) = s.network.interfaces.iter().fold((0.0, 0.0), |(rx, tx), i| {
        (rx + i.rx_bytes_sec, tx + i.tx_bytes_sec)
    });

    let border_color = theme::dim_color(theme.net_upload, theme::adaptive_border_dim(theme));

    let title_spans = vec![
        Span::styled(format!(" {}", theme::PANEL_SUPERSCRIPTS[3]), Style::default().fg(theme.net_upload)),
        Span::styled("net  ", Style::default().fg(theme.fg).bold()),
        Span::styled(format!("\u{25b2} {}", format_bytes_rate(total_tx)), Style::default().fg(theme.fg)),
        Span::styled("  ", Style::default()),
        Span::styled(format!("\u{25bc} {}", format_bytes_rate(total_rx)), Style::default().fg(theme.fg)),
        Span::raw(" "),
    ];

    let mut sorted_ifaces: Vec<&crate::metrics::NetInterface> = s.network.interfaces.iter().collect();
    sorted_ifaces.sort_by(|a, b| {
        let a_total = a.rx_bytes_sec + a.tx_bytes_sec;
        let b_total = b.rx_bytes_sec + b.tx_bytes_sec;
        b_total.partial_cmp(&a_total).unwrap_or(std::cmp::Ordering::Equal)
    });

    let display_ifaces: Vec<&crate::metrics::NetInterface> = sorted_ifaces.into_iter()
        .filter(|i| !is_infrastructure_interface(&i.name))
        .collect();

    let block = Block::default()
        .title(Line::from(title_spans))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .border_type(ratatui::widgets::BorderType::Rounded);

    let raw_inner = block.inner(area);
    f.render_widget(block, area);

    // 1-char padding left and right inside panel frame
    let inner = Rect::new(raw_inner.x + 1, raw_inner.y, raw_inner.width.saturating_sub(2), raw_inner.height);

    if inner.height < 2 || inner.width == 0 {
        return;
    }

    // Reserve last row for bottom info
    let content_area = Rect::new(inner.x, inner.y, inner.width, inner.height.saturating_sub(1));
    let bottom_y = inner.y + inner.height.saturating_sub(1);

    let scale = speed_tier_from_baudrate(s.network.primary_baudrate) as f64;
    let upload_data: Vec<f64> = state.history.net_upload.iter().copied().collect();
    let download_data: Vec<f64> = state.history.net_download.iter().copied().collect();
    let upload_idle = state.history.net_upload.iter().all(|&v| v < 1024.0);
    let download_idle = state.history.net_download.iter().all(|&v| v < 1024.0);

    // Helper to render idle overlay centered on a graph area
    let render_idle_overlay = |f: &mut Frame, area: Rect| {
        if area.height == 0 { return; }
        let idle_text = "idle";
        let mid_y = area.y + area.height / 2;
        let idle_x = area.x + area.width.saturating_sub(idle_text.len() as u16) / 2;
        f.render_widget(
            Paragraph::new(Span::styled(idle_text, Style::default().fg(theme.muted))),
            Rect::new(idle_x, mid_y, idle_text.len() as u16, 1),
        );
    };

    // Helper to render upload graph (bottom-to-top, normal)
    let render_upload_graph = |f: &mut Frame, area: Rect, data: &[f64]| {
        let height = area.height as usize;
        let graph = braille::render_braille_graph(data, scale, area.width as usize, height);
        for (row_idx, row) in graph.iter().enumerate() {
            let y = area.y + area.height.saturating_sub(1) - row_idx as u16;
            if y < area.y { break; }
            let y_frac = row_idx as f64 / (height as f64 - 1.0).max(1.0);
            let color = crate::tui::gradient::value_to_color(y_frac);
            let spans: Vec<Span> = row.iter().map(|&(ch, _)| Span::styled(ch.to_string(), Style::default().fg(color))).collect();
            if !spans.is_empty() {
                f.render_widget(Paragraph::new(Line::from(spans)), Rect::new(area.x, y, area.width, 1));
            }
        }
    };

    // Helper to render download graph (top-to-bottom, mirrored like btop)
    let render_download_graph = |f: &mut Frame, area: Rect, data: &[f64]| {
        let height = area.height as usize;
        let graph = braille::render_braille_graph_down(data, scale, area.width as usize, height);
        for (row_idx, row) in graph.iter().enumerate() {
            let y = area.y + row_idx as u16;
            if y >= area.y + area.height { break; }
            let spans: Vec<Span> = row.iter().map(|&(ch, color)| Span::styled(ch.to_string(), Style::default().fg(color))).collect();
            if !spans.is_empty() {
                f.render_widget(Paragraph::new(Line::from(spans)), Rect::new(area.x, y, area.width, 1));
            }
        }
    };

    if state.show_detail {
        let (left, mid, right) = layout::split_type_b(content_area);

        render_upload_graph(f, left, &upload_data);
        if upload_idle { render_idle_overlay(f, left); }
        render_download_graph(f, mid, &download_data);
        if download_idle { render_idle_overlay(f, mid); }

        // Right 25%: btop-style traffic metrics + top interfaces
        let total_rx_bytes: u64 = display_ifaces.iter().map(|i| i.rx_bytes_total).sum();
        let total_tx_bytes: u64 = display_ifaces.iter().map(|i| i.tx_bytes_total).sum();

        let mut lines: Vec<Line> = Vec::new();

        lines.push(Line::from(Span::styled("\u{25b2} Upload", Style::default().fg(theme.net_upload))));
        lines.push(Line::from(Span::styled(
            format_bytes_rate_compact(total_tx),
            Style::default().fg(theme.fg),
        )));
        lines.push(Line::from(Span::styled(
            format!("max: {}", format_bytes_rate_compact(state.history.net_upload_max)),
            Style::default().fg(theme.fg),
        )));
        lines.push(Line::from(Span::styled(
            format!("total: {}", format_bytes_compact(total_tx_bytes as f64)),
            Style::default().fg(theme.fg),
        )));

        lines.push(Line::from(""));

        lines.push(Line::from(Span::styled("\u{25bc} Download", Style::default().fg(theme.net_download))));
        lines.push(Line::from(Span::styled(
            format_bytes_rate_compact(total_rx),
            Style::default().fg(theme.fg),
        )));
        lines.push(Line::from(Span::styled(
            format!("max: {}", format_bytes_rate_compact(state.history.net_download_max)),
            Style::default().fg(theme.fg),
        )));
        lines.push(Line::from(Span::styled(
            format!("total: {}", format_bytes_compact(total_rx_bytes as f64)),
            Style::default().fg(theme.fg),
        )));

        let active_ifaces: Vec<&&crate::metrics::NetInterface> = display_ifaces.iter()
            .filter(|i| i.rx_bytes_sec > 0.0 || i.tx_bytes_sec > 0.0)
            .take(3)
            .collect();
        if !active_ifaces.is_empty() {
            lines.push(Line::from(""));
            for iface in &active_ifaces {
                lines.push(Line::from(Span::styled(
                    format!("{} ({})", iface.name, iface.iface_type),
                    Style::default().fg(theme.muted),
                )));
            }
        }

        let content_height = lines.len() as u16;
        let start_y = if content_height < right.height {
            right.y + (right.height - content_height) / 2
        } else {
            right.y
        };
        for (i, line) in lines.iter().take(right.height as usize).enumerate() {
            f.render_widget(
                Paragraph::new(line.clone()),
                Rect::new(right.x, start_y + i as u16, right.width, 1),
            );
        }
        // show_detail: skip frame-bottom info (detail panel has richer content)
        return;
    } else {
        // Full-width: two graphs split 50/50
        let half_w = content_area.width / 2;
        let left = Rect::new(content_area.x, content_area.y, half_w, content_area.height);
        let mid = Rect::new(content_area.x + half_w, content_area.y, content_area.width - half_w, content_area.height);

        render_upload_graph(f, left, &upload_data);
        if upload_idle { render_idle_overlay(f, left); }
        render_download_graph(f, mid, &download_data);
        if download_idle { render_idle_overlay(f, mid); }

        // Fallback: primary interface name on bottom row
        if let Some(primary) = display_ifaces.first() {
            let fallback = format!("{} ", primary.name);
            f.render_widget(
                Paragraph::new(Line::from(Span::styled(fallback, Style::default().fg(theme.muted)))
                    .alignment(ratatui::layout::Alignment::Right)),
                Rect::new(inner.x, bottom_y, inner.width, 1),
            );
            return; // skip default bottom info
        }
    }

    // Bottom info inside panel
    let bottom_text = if let Some(primary) = display_ifaces.first() {
        format!(" {} ({}) ", primary.name, primary.iface_type)
    } else {
        " No active interfaces ".to_string()
    };
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(bottom_text, Style::default().fg(theme.muted)))),
        Rect::new(inner.x, bottom_y, inner.width, 1),
    );
}
