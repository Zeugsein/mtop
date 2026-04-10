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

    let title_spans = vec![
        Span::styled(" Network  ", Style::default().fg(theme.net_upload).bold()),
        Span::styled(format!("↑ {}", format_bytes_rate(total_tx)), Style::default().fg(theme.net_upload)),
        Span::styled("  ", Style::default()),
        Span::styled(format!("↓ {}", format_bytes_rate(total_rx)), Style::default().fg(theme.net_download)),
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

    let bottom_text = if let Some(primary) = display_ifaces.first() {
        format!(" {} ({}) ", primary.name, primary.iface_type)
    } else {
        " No active interfaces ".to_string()
    };
    let bottom_spans = vec![
        Span::styled(bottom_text, Style::default().fg(theme.muted)),
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

    let scale = speed_tier_from_baudrate(s.network.primary_baudrate) as f64;

    // Left 37.5%: Upload braille sparkline
    let upload_data: Vec<f64> = state.history.net_upload.iter().copied().collect();
    let upload_spark = braille::render_braille_sparkline(&upload_data, scale, left.width as usize);
    let upload_spans: Vec<Span> = upload_spark
        .iter()
        .map(|&(ch, _)| Span::styled(ch.to_string(), Style::default().fg(theme.net_upload)))
        .collect();
    if !upload_spans.is_empty() {
        let y_offset = left.height / 2;
        f.render_widget(
            Paragraph::new(Line::from(upload_spans)),
            Rect::new(left.x, left.y + y_offset, left.width, 1),
        );
    }

    // Middle 37.5%: Download braille sparkline
    let download_data: Vec<f64> = state.history.net_download.iter().copied().collect();
    let download_spark = braille::render_braille_sparkline(&download_data, scale, mid.width as usize);
    let download_spans: Vec<Span> = download_spark
        .iter()
        .map(|&(ch, _)| Span::styled(ch.to_string(), Style::default().fg(theme.net_download)))
        .collect();
    if !download_spans.is_empty() {
        let y_offset = mid.height / 2;
        f.render_widget(
            Paragraph::new(Line::from(download_spans)),
            Rect::new(mid.x, mid.y + y_offset, mid.width, 1),
        );
    }

    // Right 25%: Interface ranking by throughput
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
                    Style::default().fg(theme.net_upload),
                ),
                Span::styled(
                    format!("  ↓{}", format_bytes_rate_compact(iface.rx_bytes_sec)),
                    Style::default().fg(theme.net_download),
                ),
            ]);
            f.render_widget(
                Paragraph::new(line),
                Rect::new(right.x, y, right.width, 1),
            );
        }
    }
}
