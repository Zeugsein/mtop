use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::metrics::MetricsSnapshot;
use crate::platform::network::speed_tier_from_baudrate;
use crate::tui::{AppState, theme, braille, layout};
use crate::tui::helpers::{format_bytes_rate, format_bytes_rate_compact, is_infrastructure_interface};

/// Network panel: Type B layout (37.5% upload + 37.5% download + 25% interface ranking)
pub(crate) fn draw_network_panel_v2(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    let (total_rx, total_tx) = s.network.interfaces.iter().fold((0.0, 0.0), |(rx, tx), i| {
        (rx + i.rx_bytes_sec, tx + i.tx_bytes_sec)
    });

    let border_color = theme::dim_color(theme.net_upload, 0.4);

    let title_spans = vec![
        Span::styled(" net  ", Style::default().fg(theme.net_upload).bold()),
        Span::styled(format!("↑ {}", format_bytes_rate(total_tx)), Style::default().fg(theme.fg)),
        Span::styled("  ", Style::default()),
        Span::styled(format!("↓ {}", format_bytes_rate(total_rx)), Style::default().fg(theme.fg)),
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

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 2 {
        return;
    }

    // Reserve last row for bottom info
    let content_area = Rect::new(inner.x, inner.y, inner.width, inner.height.saturating_sub(1));
    let bottom_y = inner.y + inner.height.saturating_sub(1);

    let (left, mid, right) = layout::split_type_b(content_area);

    let scale = speed_tier_from_baudrate(s.network.primary_baudrate) as f64;

    // Left 37.5%: Upload multi-row braille graph
    let upload_data: Vec<f64> = state.history.net_upload.iter().copied().collect();
    let graph = braille::render_braille_graph(&upload_data, scale, left.width as usize, left.height as usize);
    for (row_idx, row) in graph.iter().enumerate() {
        let y = left.y + left.height.saturating_sub(1) - row_idx as u16;
        if y < left.y { break; }
        let spans: Vec<Span> = row.iter().map(|&(ch, _)| Span::styled(ch.to_string(), Style::default().fg(theme.net_upload))).collect();
        if !spans.is_empty() {
            f.render_widget(Paragraph::new(Line::from(spans)), Rect::new(left.x, y, left.width, 1));
        }
    }

    // Middle 37.5%: Download multi-row braille graph
    let download_data: Vec<f64> = state.history.net_download.iter().copied().collect();
    let graph = braille::render_braille_graph(&download_data, scale, mid.width as usize, mid.height as usize);
    for (row_idx, row) in graph.iter().enumerate() {
        let y = mid.y + mid.height.saturating_sub(1) - row_idx as u16;
        if y < mid.y { break; }
        let spans: Vec<Span> = row.iter().map(|&(ch, _)| Span::styled(ch.to_string(), Style::default().fg(theme.net_download))).collect();
        if !spans.is_empty() {
            f.render_widget(Paragraph::new(Line::from(spans)), Rect::new(mid.x, y, mid.width, 1));
        }
    }

    // Right 25%: Interface ranking (white text)
    if display_ifaces.is_empty() {
        f.render_widget(
            Paragraph::new("No interfaces").style(Style::default().fg(theme.muted)),
            right,
        );
    } else {
        let max_rows = right.height as usize;
        for (i, iface) in display_ifaces.iter().take(max_rows).enumerate() {
            let y = right.y + i as u16;
            if y >= right.y + right.height {
                break;
            }
            let line = Line::from(vec![
                Span::styled(&*iface.name, Style::default().fg(theme.fg)),
                Span::styled(
                    format!("  ↑{}", format_bytes_rate_compact(iface.tx_bytes_sec)),
                    Style::default().fg(theme.fg),
                ),
                Span::styled(
                    format!("  ↓{}", format_bytes_rate_compact(iface.rx_bytes_sec)),
                    Style::default().fg(theme.fg),
                ),
            ]);
            f.render_widget(
                Paragraph::new(line),
                Rect::new(right.x, y, right.width, 1),
            );
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
