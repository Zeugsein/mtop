use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::metrics::MetricsSnapshot;
use crate::tui::{AppState, theme, braille, layout};
use crate::tui::helpers::{format_bytes_rate_compact, format_bytes_compact, is_infrastructure_interface};

pub(crate) const NET_TIERS: [(f64, &str); 7] = [
    (1_048_576.0,       "1MB/s"),
    (5_242_880.0,       "5MB/s"),
    (10_485_760.0,      "10MB/s"),
    (52_428_800.0,      "50MB/s"),
    (104_857_600.0,     "100MB/s"),
    (524_288_000.0,     "500MB/s"),
    (1_048_576_000.0,   "1GB/s"),
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
    }
    title_spans.push(Span::raw(" "));

    let tier_idx = state.history.net_tier_idx;
    let scale_label = NET_TIERS[tier_idx].1;
    let right_title = Line::from(vec![
        Span::styled(format!("100%={} ", scale_label), Style::default().fg(theme.muted)),
    ]);

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
        .title_top(right_title.alignment(ratatui::layout::Alignment::Right))
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
    let scale = NET_TIERS[tier_idx].0;

    // Minimum baseline value: ensures at least 1 braille dot renders at zero
    let baseline_floor = scale * 0.035;

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
                let is_baseline = orig_l < baseline_floor * 2.0 && orig_r < baseline_floor * 2.0;
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
                let is_baseline = orig_l < baseline_floor * 2.0 && orig_r < baseline_floor * 2.0;
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

    if state.show_detail {
        let (chart_area, right) = layout::split_type_a(content_area);

        render_symmetric_chart(f, chart_area, &download_data, &upload_data);

        // Right 25%: download on top, upload on bottom (matching chart order)
        let total_rx_bytes: u64 = display_ifaces.iter().map(|i| i.rx_bytes_total).sum();
        let total_tx_bytes: u64 = display_ifaces.iter().map(|i| i.tx_bytes_total).sum();

        let mut lines: Vec<Line> = Vec::new();

        lines.push(Line::from(Span::styled("\u{25bc} download", Style::default().fg(theme.net_download))));
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

        lines.push(Line::from(Span::styled("\u{25b2} upload", Style::default().fg(theme.net_upload))));
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

        // Active interface list with heading
        let active_ifaces: Vec<&&crate::metrics::NetInterface> = display_ifaces.iter()
            .filter(|i| i.rx_bytes_sec > 0.0 || i.tx_bytes_sec > 0.0)
            .collect();
        if !active_ifaces.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("■ active", Style::default().fg(theme.fg))));
            for iface in active_ifaces.iter().take(3) {
                lines.push(Line::from(Span::styled(
                    format!("{} ({})", iface.name, iface.iface_type),
                    Style::default().fg(theme.fg),
                )));
            }
        }
        // Pad to stabilize layout
        let shown = if active_ifaces.is_empty() { 0 } else { 2 + active_ifaces.len().min(3) };
        for _ in shown..5 {
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
    } else {
        // Full-width symmetric chart
        render_symmetric_chart(f, content_area, &download_data, &upload_data);

        // I44-F4: colored speed labels + per-half max/total on right
        if content_area.height >= 6 {
            let ul_rate = upload_data.last().copied().unwrap_or(0.0);
            let dl_rate = download_data.last().copied().unwrap_or(0.0);
            let dl_label = format!("↓ {}", format_bytes_rate_compact(dl_rate));
            let ul_label = format!("↑ {}", format_bytes_rate_compact(ul_rate));

            // Top-left: download rate colored
            f.render_widget(
                Paragraph::new(Span::styled(dl_label, Style::default().fg(theme.net_download))),
                Rect::new(content_area.x, content_area.y, content_area.width, 1),
            );
            // Bottom-left: upload rate colored
            f.render_widget(
                Paragraph::new(Span::styled(ul_label, Style::default().fg(theme.net_upload))),
                Rect::new(content_area.x, content_area.y + content_area.height - 1, content_area.width, 1),
            );

            // Total bytes from filtered interfaces
            let total_rx_bytes: u64 = display_ifaces.iter().map(|i| i.rx_bytes_total).sum();
            let total_tx_bytes: u64 = display_ifaces.iter().map(|i| i.tx_bytes_total).sum();

            // Top-right: download max + total (muted)
            let dl_right = format!("max {} total {} ", format_bytes_rate_compact(state.history.net_download_max), format_bytes_compact(total_rx_bytes as f64));
            f.render_widget(
                Paragraph::new(Line::from(Span::styled(&dl_right, Style::default().fg(theme.muted)))
                    .alignment(ratatui::layout::Alignment::Right)),
                Rect::new(content_area.x, content_area.y, content_area.width, 1),
            );
            // Bottom-right: upload max + total (muted)
            let ul_right = format!("max {} total {} ", format_bytes_rate_compact(state.history.net_upload_max), format_bytes_compact(total_tx_bytes as f64));
            f.render_widget(
                Paragraph::new(Line::from(Span::styled(&ul_right, Style::default().fg(theme.muted)))
                    .alignment(ratatui::layout::Alignment::Right)),
                Rect::new(content_area.x, content_area.y + content_area.height - 1, content_area.width, 1),
            );
        }

        // Bottom row: primary interface name
        if let Some(primary) = display_ifaces.first() {
            let fallback = format!("{} ", primary.name);
            f.render_widget(
                Paragraph::new(Line::from(Span::styled(fallback, Style::default().fg(theme.muted)))
                    .alignment(ratatui::layout::Alignment::Right)),
                Rect::new(inner.x, bottom_y, inner.width, 1),
            );
        }
    }
}
