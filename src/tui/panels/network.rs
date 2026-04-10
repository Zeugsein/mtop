use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::metrics::MetricsSnapshot;
use crate::tui::{AppState, theme, braille, layout};
use crate::tui::helpers::{format_bytes_rate, format_bytes_rate_compact, format_bytes_compact, is_infrastructure_interface};

/// Dynamic tier thresholds (bytes/sec) and their display labels.
pub(crate) const NET_TIERS: [(f64, &str); 4] = [
    (1_000_000.0,       "1 MB/s"),
    (10_000_000.0,      "10 MB/s"),
    (100_000_000.0,     "100 MB/s"),
    (1_000_000_000.0,   "1 GB/s"),
];


/// Network panel: Type B layout (37.5% upload + 37.5% download + 25% interface ranking)
pub(crate) fn draw_network_panel_v2(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    let (total_rx, total_tx) = s.network.interfaces.iter().fold((0.0, 0.0), |(rx, tx), i| {
        (rx + i.rx_bytes_sec, tx + i.tx_bytes_sec)
    });

    let border_color = theme::dim_color(theme.net_download, theme::adaptive_border_dim(theme));

    let net_idle = total_rx < 1024.0 && total_tx < 1024.0;

    let mut title_spans = vec![
        Span::styled(format!(" {}", theme::PANEL_SUPERSCRIPTS[3]), Style::default().fg(theme.muted)),
        Span::styled("net ", Style::default().fg(theme.fg).bold()),
    ];
    if net_idle {
        title_spans.push(Span::styled("(idle) ", Style::default().fg(theme.muted)));
    } else {
        title_spans.push(Span::styled(format!(" \u{25b2} {}", format_bytes_rate(total_tx)), Style::default().fg(theme.fg)));
        title_spans.push(Span::styled("  ", Style::default()));
        title_spans.push(Span::styled(format!("\u{25bc} {}", format_bytes_rate(total_rx)), Style::default().fg(theme.fg)));
    }
    title_spans.push(Span::raw(" "));

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

    // 1-char padding left/right, no top padding (UAT-07)
    let inner = Rect::new(raw_inner.x + 1, raw_inner.y, raw_inner.width.saturating_sub(2), raw_inner.height);

    if inner.height < 2 || inner.width == 0 {
        return;
    }

    // Reserve last row for bottom info
    let content_area = Rect::new(inner.x, inner.y, inner.width, inner.height.saturating_sub(1));
    let bottom_y = inner.y + inner.height.saturating_sub(1);

    let upload_data: Vec<f64> = state.history.net_upload.iter().copied().collect();
    let download_data: Vec<f64> = state.history.net_download.iter().copied().collect();
    let tier_idx = state.history.net_tier_idx;
    let (scale, tier_label) = (NET_TIERS[tier_idx].0, NET_TIERS[tier_idx].1);

    // Minimum baseline value: ensures at least 1 braille dot renders at zero
    let baseline_floor = scale * 0.005;

    // Helper to render graph growing upward (bottom-to-top) with muted baseline for near-zero values
    let render_graph_upward = |f: &mut Frame, area: Rect, data: &[f64]| {
        let height = area.height as usize;
        // Clamp data so zero values produce 1 dot (baseline)
        let clamped: Vec<f64> = data.iter().map(|&v| v.max(baseline_floor)).collect();
        let graph = braille::render_braille_graph(&clamped, scale, area.width as usize, height, theme);

        // Map original data visibility for per-column muted detection
        let needed = area.width as usize * 2;
        let start = data.len().saturating_sub(needed);
        let visible_orig = &data[start..];

        for (row_idx, row) in graph.iter().enumerate() {
            let y = area.y + area.height.saturating_sub(1) - row_idx as u16;
            if y < area.y { break; }
            let y_frac = row_idx as f64 / (height as f64 - 1.0).max(1.0);
            let gradient_color = crate::tui::gradient::value_to_color(y_frac, theme);

            let spans: Vec<Span> = row.iter().enumerate().map(|(col, &(ch, _))| {
                // Each braille column maps to 2 data points; out-of-bounds → muted (no data yet)
                let orig_l = visible_orig.get(col * 2).copied().unwrap_or(0.0);
                let orig_r = visible_orig.get(col * 2 + 1).copied().unwrap_or(0.0);
                let is_baseline = orig_l < 1.0 && orig_r < 1.0;
                let color = if is_baseline { theme::baseline_color(theme) } else { gradient_color };
                Span::styled(ch.to_string(), Style::default().fg(color))
            }).collect();
            if !spans.is_empty() {
                f.render_widget(Paragraph::new(Line::from(spans)), Rect::new(area.x, y, area.width, 1));
            }
        }
    };

    // Helper to render graph growing downward (top-to-bottom, mirrored) with baseline color
    let render_graph_downward = |f: &mut Frame, area: Rect, data: &[f64]| {
        let height = area.height as usize;
        let clamped: Vec<f64> = data.iter().map(|&v| v.max(baseline_floor)).collect();
        let graph = braille::render_braille_graph_down(&clamped, scale, area.width as usize, height, theme);

        let needed = area.width as usize * 2;
        let start = data.len().saturating_sub(needed);
        let visible_orig = &data[start..];

        for (row_idx, row) in graph.iter().enumerate() {
            let y = area.y + row_idx as u16;
            if y >= area.y + area.height { break; }
            let spans: Vec<Span> = row.iter().enumerate().map(|(col, &(ch, orig_color))| {
                let orig_l = visible_orig.get(col * 2).copied().unwrap_or(0.0);
                let orig_r = visible_orig.get(col * 2 + 1).copied().unwrap_or(0.0);
                let is_baseline = orig_l < 1.0 && orig_r < 1.0;
                let color = if is_baseline { theme::baseline_color(theme) } else { orig_color };
                Span::styled(ch.to_string(), Style::default().fg(color))
            }).collect();
            if !spans.is_empty() {
                f.render_widget(Paragraph::new(Line::from(spans)), Rect::new(area.x, y, area.width, 1));
            }
        }
    };

    // Symmetric center-baseline chart helper
    let render_symmetric_chart = |f: &mut Frame, chart_area: Rect, dl_data: &[f64], ul_data: &[f64]| {
        let half_h = chart_area.height / 2;
        let top_area = Rect::new(chart_area.x, chart_area.y, chart_area.width, half_h);
        let bottom_area = Rect::new(chart_area.x, chart_area.y + half_h, chart_area.width, chart_area.height - half_h);

        // Download TOP half: bars grow upward from center
        render_graph_upward(f, top_area, dl_data);

        // Upload BOTTOM half: bars grow downward from center
        render_graph_downward(f, bottom_area, ul_data);
    };

    // Render tier label at top-right of a chart area
    let render_tier_label = |f: &mut Frame, chart_area: Rect| {
        let label_len = tier_label.len() as u16;
        if chart_area.width > label_len + 1 {
            let lx = chart_area.x + chart_area.width - label_len;
            f.render_widget(
                Paragraph::new(Span::styled(tier_label, Style::default().fg(theme.muted))),
                Rect::new(lx, chart_area.y, label_len, 1),
            );
        }
    };

    if state.show_detail {
        let (chart_area, right) = layout::split_type_a(content_area);

        render_symmetric_chart(f, chart_area, &download_data, &upload_data);
        render_tier_label(f, chart_area);

        // Right 25%: download on top, upload on bottom (matching chart order)
        let total_rx_bytes: u64 = display_ifaces.iter().map(|i| i.rx_bytes_total).sum();
        let total_tx_bytes: u64 = display_ifaces.iter().map(|i| i.tx_bytes_total).sum();

        let mut lines: Vec<Line> = Vec::new();

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

        lines.push(Line::from(""));

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

        // Stable interface list: show top 3 interfaces in fixed positions
        lines.push(Line::from(""));
        let iface_count = display_ifaces.len().min(3);
        for iface in display_ifaces.iter().take(3) {
            let active = iface.rx_bytes_sec > 0.0 || iface.tx_bytes_sec > 0.0;
            let color = if active { theme.fg } else { theme.muted };
            lines.push(Line::from(Span::styled(
                format!("{} ({})", iface.name, iface.iface_type),
                Style::default().fg(color),
            )));
        }
        // Pad remaining slots to stabilize layout
        for _ in iface_count..3 {
            lines.push(Line::from(""));
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
        // Full-width symmetric chart
        render_symmetric_chart(f, content_area, &download_data, &upload_data);
        render_tier_label(f, content_area);

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
