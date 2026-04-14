//! Expanded panel renderers extracted from mod.rs (iteration 8).

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::metrics::MetricsSnapshot;
use super::{AppState, PanelId, theme, braille, gauge, gradient};
use super::helpers::{format_bytes_rate_compact, truncate_with_ellipsis, truncate_by_display_width, pad_to_display_width, is_infrastructure_interface, format_baudrate, sort_indices};
use super::panels::render_graph_with_baseline;
use super::panels::{COL_PID, COL_CPU, COL_MEM, COL_POW, COL_THR, COL_FIXED_TOTAL};


pub(crate) fn draw_expanded_panel(f: &mut Frame, area: Rect, panel: PanelId, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    match panel {
        PanelId::Cpu => draw_cpu_expanded(f, area, s, state, theme),
        PanelId::Gpu => draw_gpu_expanded(f, area, s, state, theme),
        PanelId::MemDisk => draw_mem_disk_expanded(f, area, s, state, theme),
        PanelId::Network => draw_network_expanded(f, area, s, state, theme),
        PanelId::Power => draw_power_expanded(f, area, s, state, theme),
        PanelId::Process => draw_process_expanded(f, area, s, state, theme),
    }
}

fn draw_cpu_expanded(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    let cpu_pct = s.cpu.total_usage * 100.0;
    // F5: use gradient::temp_to_color matching non-expanded
    let temp_col = gradient::temp_to_color(s.temperature.cpu_avg_c, theme);
    // F4: show "N/A" when temp unavailable, matching non-expanded
    let temp_str = if s.temperature.available {
        format!("{:.0}°C", s.temperature.cpu_avg_c)
    } else {
        "N/A".to_string()
    };
    let temp_display_color = if s.temperature.available { temp_col } else { theme.muted };

    // F1: add frequency; F6: superscript as separate muted span
    let max_freq = s.cpu.p_cluster.freq_mhz.max(s.cpu.e_cluster.freq_mhz);
    let title_spans = vec![
        Span::styled(format!(" {}", theme::PANEL_SUPERSCRIPTS[0]), Style::default().fg(theme.muted)),
        Span::styled("cpu  ", Style::default().fg(theme.cpu_accent).bold()),
        Span::styled(format!("{:.1}%", cpu_pct), Style::default().fg(theme.fg)),
        Span::styled(format!(" @ {}MHz", max_freq), Style::default().fg(theme.muted)),
        Span::styled(format!("  {:.1}W", s.power.cpu_w), Style::default().fg(theme.muted)),
        Span::styled(format!("  {}", temp_str), Style::default().fg(temp_display_color)),
        Span::raw(" "),
    ];

    let block = Block::default()
        .title(Line::from(title_spans))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.cpu_accent))
        .border_type(BorderType::Rounded);

    let raw_inner = block.inner(area);
    f.render_widget(block, area);
    let inner = Rect::new(raw_inner.x + 1, raw_inner.y, raw_inner.width.saturating_sub(2), raw_inner.height);

    if inner.height == 0 || inner.width == 0 { return; }

    // Count how many rows the core bars need (including margins)
    let e_count = s.soc.e_cores as usize;
    let p_count = s.cpu.core_usages.len().saturating_sub(e_count);
    // e-header + e-cores + margin + p-header + p-cores + margin(chart→e)
    let core_rows_needed = 1 + e_count + 1 + 1 + p_count + 1;

    // F2: cap chart height at 20 rows; F3: distribute space vertically
    let chart_height = (inner.height.saturating_sub(core_rows_needed as u16)).clamp(3, 20);

    // F3: vertically center the content block (chart + cores)
    let total_content = chart_height + core_rows_needed as u16;
    let top_pad = (inner.height.saturating_sub(total_content)) / 2;

    let chart_y = inner.y + top_pad;
    let chart_area = Rect::new(inner.x, chart_y, inner.width, chart_height);
    let sparkline_data: Vec<f64> = state.history.cpu_usage.iter().copied().collect();
    render_graph_with_baseline(f, chart_area, &sparkline_data, 1.0, theme);

    // I42-F1a: percentage overlay at chart top-left
    let pct_label = format!("{:.0}% ", cpu_pct);
    f.render_widget(
        Paragraph::new(Span::styled(&pct_label, Style::default().fg(theme.muted))),
        Rect::new(chart_area.x + 1, chart_area.y, pct_label.len() as u16, 1),
    );

    // Per-core usage bars — I42-F1b: 1-row margin between chart and e-cluster
    let core_start_y = chart_y + chart_height + 1;

    // E-cluster header
    if core_start_y < inner.y + inner.height {
        f.render_widget(
            Paragraph::new(format!("e-cluster: {:.0}% @ {}MHz", s.cpu.e_cluster.usage * 100.0, s.cpu.e_cluster.freq_mhz))
                .style(Style::default().fg(theme.cpu_accent)),
            Rect::new(inner.x, core_start_y, inner.width, 1),
        );
    }

    let bar_width = inner.width.saturating_sub(12) as usize;
    for (i, &usage) in s.cpu.core_usages.iter().take(e_count).enumerate() {
        let y = core_start_y + 1 + i as u16;
        if y >= inner.y + inner.height { break; }

        let norm = usage.clamp(0.0, 1.0);
        let filled = (bar_width as f32 * norm) as usize;
        let empty = bar_width.saturating_sub(filled);
        let color = gradient::value_to_color(norm as f64, theme);

        let line = Line::from(vec![
            Span::styled(format!("core {:>2} ",i), Style::default().fg(theme.muted)),
            Span::styled("▓".repeat(filled), Style::default().fg(color)),
            Span::styled("░".repeat(empty), Style::default().fg(theme.border)),
            Span::styled(format!(" {:>5.1}%", usage * 100.0), Style::default().fg(theme.fg)),
        ]);
        f.render_widget(Paragraph::new(line), Rect::new(inner.x, y, inner.width, 1));
    }

    // I42-F1b: 1-row margin between e-cores and p-cluster
    let p_header_y = core_start_y + 1 + e_count as u16 + 1;
    if p_header_y < inner.y + inner.height {
        f.render_widget(
            Paragraph::new(format!("p-cluster: {:.0}% @ {}MHz", s.cpu.p_cluster.usage * 100.0, s.cpu.p_cluster.freq_mhz))
                .style(Style::default().fg(theme.cpu_accent)),
            Rect::new(inner.x, p_header_y, inner.width, 1),
        );
    }
    for (i, &usage) in s.cpu.core_usages.iter().skip(e_count).enumerate() {
        let y = p_header_y + 1 + i as u16;
        if y >= inner.y + inner.height { break; }

        let norm = usage.clamp(0.0, 1.0);
        let filled = (bar_width as f32 * norm) as usize;
        let empty = bar_width.saturating_sub(filled);
        let color = gradient::value_to_color(norm as f64, theme);

        let line = Line::from(vec![
            Span::styled(format!("core {:>2} ",e_count + i), Style::default().fg(theme.muted)),
            Span::styled("▓".repeat(filled), Style::default().fg(color)),
            Span::styled("░".repeat(empty), Style::default().fg(theme.border)),
            Span::styled(format!(" {:>5.1}%", usage * 100.0), Style::default().fg(theme.fg)),
        ]);
        f.render_widget(Paragraph::new(line), Rect::new(inner.x, y, inner.width, 1));
    }
}

fn draw_gpu_expanded(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    // F5/F13: use gradient::temp_to_color matching non-expanded
    let temp_col = gradient::temp_to_color(s.temperature.gpu_avg_c, theme);
    let temp_str = if s.temperature.available {
        format!("{}°C", s.temperature.gpu_avg_c as u32)
    } else {
        "N/A".to_string()
    };
    let temp_display_color = if s.temperature.available { temp_col } else { theme.muted };

    let gpu_idle = s.power.gpu_w < 0.5;

    // F6: superscript as separate muted span; F7: add freq+power; F8: no % when idle
    let mut title_spans = vec![
        Span::styled(format!(" {}", theme::PANEL_SUPERSCRIPTS[1]), Style::default().fg(theme.muted)),
        Span::styled("gpu ", Style::default().fg(theme.gpu_accent).bold()),
    ];
    if gpu_idle {
        title_spans.push(Span::styled("(idle) ", Style::default().fg(theme.muted)));
    } else {
        title_spans.push(Span::styled(format!("{:.1}%", s.gpu.usage * 100.0), Style::default().fg(theme.fg)));
        title_spans.push(Span::styled(format!(" @ {}MHz", s.gpu.freq_mhz), Style::default().fg(theme.muted)));
        title_spans.push(Span::styled(format!("  {:.1}W", s.power.gpu_w), Style::default().fg(theme.muted)));
    }
    title_spans.push(Span::styled(format!("  {}", temp_str), Style::default().fg(temp_display_color)));
    title_spans.push(Span::raw(" "));

    let block = Block::default()
        .title(Line::from(title_spans))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.gpu_accent))
        .border_type(BorderType::Rounded);

    let raw_inner = block.inner(area);
    f.render_widget(block, area);
    let inner = Rect::new(raw_inner.x + 1, raw_inner.y, raw_inner.width.saturating_sub(2), raw_inner.height);

    if inner.height == 0 || inner.width == 0 { return; }

    // F2-equivalent: cap chart height at 20 rows
    let chart_height = (inner.height * 7 / 10).clamp(3, 20);
    let chart_area = Rect::new(inner.x, inner.y, inner.width, chart_height);
    let sparkline_data: Vec<f64> = state.history.gpu_usage.iter().copied().collect();
    // F11: use render_graph_with_baseline matching non-expanded
    render_graph_with_baseline(f, chart_area, &sparkline_data, 1.0, theme);

    // I42-F2: percentage overlay at chart top-left (suppress when idle)
    if !gpu_idle {
        let gpu_pct_label = format!("{:.0}% ", s.gpu.usage * 100.0);
        f.render_widget(
            Paragraph::new(Span::styled(&gpu_pct_label, Style::default().fg(theme.muted))),
            Rect::new(chart_area.x + 1, chart_area.y, gpu_pct_label.len() as u16, 1),
        );
    }

    // F9: 1-row margin between chart and metrics
    let metrics_y = inner.y + chart_height + 1;

    let gb = 1024.0 * 1024.0 * 1024.0;
    // F10: add VRAM; F12: use {:.1}W precision (usage removed — shown in overlay)
    let metrics = [
        format!("frequency:    {} MHz", s.gpu.freq_mhz),
        format!("GPU power:    {:.1}W", s.power.gpu_w),
        format!("ANE power:    {:.1}W", s.power.ane_w),
        format!("DRAM power:   {:.1}W", s.power.dram_w),
        format!("VRAM:         {:.1}/{:.0}GB", s.memory.ram_used as f64 / gb, s.memory.ram_total as f64 / gb),
    ];

    for (i, text) in metrics.iter().enumerate() {
        let y = metrics_y + i as u16;
        if y >= inner.y + inner.height { break; }
        f.render_widget(
            Paragraph::new(text.as_str()).style(Style::default().fg(theme.fg)),
            Rect::new(inner.x, y, inner.width, 1),
        );
    }
}

fn draw_mem_disk_expanded(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    let gb = 1024.0 * 1024.0 * 1024.0;
    let ram_used_gb = s.memory.ram_used as f64 / gb;
    let ram_total_gb = s.memory.ram_total as f64 / gb;

    let ram_pct = if s.memory.ram_total > 0 {
        (s.memory.ram_used as f64 / s.memory.ram_total as f64 * 100.0) as u32
    } else { 0 };
    let pressure_dot_color = match s.memory.pressure_level {
        2 => theme.pressure_warn,
        4 => theme.pressure_critical,
        _ => theme.pressure_normal,
    };
    let title_spans = vec![
        Span::styled(format!(" {}", theme::PANEL_SUPERSCRIPTS[2]), Style::default().fg(theme.muted)),
        Span::styled("mem  ", Style::default().fg(theme.mem_accent).bold()),
        Span::styled(format!("{:.1}/{:.0}GB {ram_pct}%", ram_used_gb, ram_total_gb), Style::default().fg(theme.fg)),
        Span::styled(" \u{25cf}", Style::default().fg(pressure_dot_color)),
        Span::raw(" "),
    ];

    let block = Block::default()
        .title(Line::from(title_spans))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.mem_accent))
        .border_type(BorderType::Rounded);

    let raw_inner = block.inner(area);
    f.render_widget(block, area);
    let inner = Rect::new(raw_inner.x + 1, raw_inner.y, raw_inner.width.saturating_sub(2), raw_inner.height);

    if inner.height == 0 || inner.width == 0 { return; }

    // === MEMORY GROUP ===
    let mut y_cursor = inner.y;

    // F1a: Memory usage gauge at top
    let ram_fraction = if s.memory.ram_total > 0 {
        s.memory.ram_used as f64 / s.memory.ram_total as f64
    } else {
        0.0
    };
    let gauge_spans = gauge::render_compact_gauge(ram_fraction, inner.width as usize, theme);
    f.render_widget(Paragraph::new(Line::from(gauge_spans)), Rect::new(inner.x, y_cursor, inner.width, 1));
    y_cursor += 1;

    // F1b: 2×2 chart grid (used, available, cached, free)
    let remaining = inner.y + inner.height - y_cursor;
    let mem_chart_h = (remaining * 40 / 100).max(4);
    let row1_h = mem_chart_h / 2;
    let row2_h = mem_chart_h - row1_h;
    let half_w = inner.width / 2;

    let sub_border_color = theme::dim_color(theme.mem_accent, 0.5);

    let tl = Rect::new(inner.x, y_cursor, half_w, row1_h);
    let tr = Rect::new(inner.x + half_w, y_cursor, inner.width - half_w, row1_h);
    let bl = Rect::new(inner.x, y_cursor + row1_h, half_w, row2_h);
    let br = Rect::new(inner.x + half_w, y_cursor + row1_h, inner.width - half_w, row2_h);

    let render_sub_chart = |f: &mut Frame, area: Rect, title: &str, value: String, data: &[f64], green: bool| {
        if area.height == 0 || area.width == 0 { return; }
        let block = Block::default()
            .title(Line::from(vec![
                Span::styled(format!(" {title} "), Style::default().fg(theme.fg).bold()),
                Span::styled(value, Style::default().fg(theme.fg)),
                Span::raw(" "),
            ]))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(sub_border_color));
        let chart_inner = block.inner(area);
        f.render_widget(block, area);
        if chart_inner.height > 0 && chart_inner.width > 0 {
            if green {
                super::panels::render_graph_green(f, chart_inner, data, 1.0, theme);
            } else {
                super::panels::render_graph(f, chart_inner, data, 1.0, theme);
            }
        }
    };

    let used_data: Vec<f64> = state.history.mem_usage.iter().copied().collect();
    let avail_data: Vec<f64> = state.history.mem_available.iter().copied().collect();
    let cached_data: Vec<f64> = state.history.mem_cached.iter().copied().collect();
    let free_data: Vec<f64> = state.history.mem_free.iter().copied().collect();

    let fmt_mem = |bytes: u64| -> String {
        let gb_val = bytes as f64 / gb;
        if gb_val >= 1.0 { format!("{gb_val:.1}GB") }
        else { format!("{:.0}MB", bytes as f64 / (1024.0 * 1024.0)) }
    };

    render_sub_chart(f, tl, "used", fmt_mem(s.memory.ram_used), &used_data, false);
    render_sub_chart(f, tr, "available", fmt_mem(s.memory.ram_total.saturating_sub(s.memory.ram_used)), &avail_data, true);
    render_sub_chart(f, bl, "cached", fmt_mem(s.memory.cached), &cached_data, false);
    render_sub_chart(f, br, "free", fmt_mem(s.memory.free), &free_data, true);
    y_cursor += mem_chart_h;

    // F1c: compressed + swap text (with I/O rates)
    if y_cursor < inner.y + inner.height {
        let compressed_gb = s.memory.compressed as f64 / gb;
        let swap_used_gb = s.memory.swap_used as f64 / gb;
        let swap_total_gb = s.memory.swap_total as f64 / gb;
        let mut info_spans: Vec<Span> = vec![
            Span::styled(format!(" compressed: {compressed_gb:.1}GB"), Style::default().fg(theme.muted)),
        ];
        if s.memory.swap_total > 0 {
            let mut swap_text = format!("  swap: {swap_used_gb:.1}/{swap_total_gb:.1}GB");
            if s.memory.swap_in_bytes_sec > 0.0 || s.memory.swap_out_bytes_sec > 0.0 {
                swap_text.push_str(&format!(
                    "  in:{} out:{}",
                    format_bytes_rate_compact(s.memory.swap_in_bytes_sec),
                    format_bytes_rate_compact(s.memory.swap_out_bytes_sec),
                ));
            }
            info_spans.push(Span::styled(swap_text, Style::default().fg(theme.muted)));
        }
        f.render_widget(Paragraph::new(Line::from(info_spans)), Rect::new(inner.x, y_cursor, inner.width, 1));
        y_cursor += 1;
    }

    // F1d: Visual separator between mem and disk groups
    if y_cursor < inner.y + inner.height {
        let sep = "─".repeat(inner.width as usize);
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(sep, Style::default().fg(theme.muted)))),
            Rect::new(inner.x, y_cursor, inner.width, 1),
        );
        y_cursor += 1;
    }

    // === DISK GROUP ===

    // F1e: Disk title + full-width gauge + size label
    let disk_used_gb = s.disk.used_bytes as f64 / gb;
    let disk_total_gb = s.disk.total_bytes as f64 / gb;
    let disk_fraction = if s.disk.total_bytes > 0 {
        s.disk.used_bytes as f64 / s.disk.total_bytes as f64
    } else {
        0.0
    };

    if y_cursor < inner.y + inner.height {
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(" disk", Style::default().fg(theme.fg).bold()))),
            Rect::new(inner.x, y_cursor, inner.width, 1),
        );
        y_cursor += 1;
    }

    if y_cursor < inner.y + inner.height {
        let disk_gauge_spans = gauge::render_compact_gauge(disk_fraction, inner.width as usize, theme);
        f.render_widget(Paragraph::new(Line::from(disk_gauge_spans)), Rect::new(inner.x, y_cursor, inner.width, 1));
        y_cursor += 1;
    }

    if y_cursor < inner.y + inner.height {
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!(" {disk_used_gb:.0}/{disk_total_gb:.0} GB"),
                Style::default().fg(theme.fg),
            ))),
            Rect::new(inner.x, y_cursor, inner.width, 1),
        );
        y_cursor += 1;
    }

    // I42-F3d: 1-row margin between disk gauge section and chart
    if y_cursor < inner.y + inner.height {
        y_cursor += 1;
    }

    // Disk symmetric chart — write ↓ top half (grows UPWARD from center),
    // read ↑ bottom half (grows DOWNWARD from center) — symmetric like network panel
    let remaining_h = (inner.y + inner.height).saturating_sub(y_cursor);
    if remaining_h >= 3 {
        let disk_chart_h = remaining_h;
        let disk_half_h = disk_chart_h / 2;

        let read_data: Vec<f64> = state.history.disk_read.iter().copied().collect();
        let write_data: Vec<f64> = state.history.disk_write.iter().copied().collect();

        let max_disk = read_data.iter().chain(write_data.iter()).copied().fold(0.0f64, f64::max).max(1024.0);
        let disk_scale = max_disk * 1.2;
        // I42-F3a: baseline floor matching network (0.035 not 0.005)
        let disk_baseline = disk_scale * 0.035;

        // Write ↓ top half (grows UPWARD from center baseline — like download in net)
        if disk_half_h > 0 {
            let top_area = Rect::new(inner.x, y_cursor, inner.width, disk_half_h);
            let clamped: Vec<f64> = write_data.iter().map(|&v| v.max(disk_baseline)).collect();
            let graph = braille::render_braille_graph(&clamped, disk_scale, inner.width as usize, disk_half_h as usize, theme);
            let needed = inner.width as usize * 2;
            let start = write_data.len().saturating_sub(needed);
            let visible_orig = &write_data[start..];
            for (row_idx, row) in graph.iter().enumerate() {
                let y = top_area.y + top_area.height.saturating_sub(1) - row_idx as u16;
                if y < top_area.y { break; }
                let y_frac = row_idx as f64 / (disk_half_h as f64 - 1.0).max(1.0);
                let gradient_color = super::gradient::value_to_color(y_frac, theme);
                let spans: Vec<Span> = row.iter().enumerate().map(|(col, &(ch, _))| {
                    let orig_l = visible_orig.get(col * 2).copied().unwrap_or(0.0);
                    let orig_r = visible_orig.get(col * 2 + 1).copied().unwrap_or(0.0);
                    let is_baseline = orig_l < disk_baseline * 2.0 && orig_r < disk_baseline * 2.0;
                    let color = if is_baseline { theme::baseline_color(theme) } else { gradient_color };
                    Span::styled(ch.to_string(), Style::default().fg(color))
                }).collect();
                if !spans.is_empty() {
                    f.render_widget(Paragraph::new(Line::from(spans)), Rect::new(top_area.x, y, top_area.width, 1));
                }
            }

            // I42-F3b: space between arrow and word; I42-F3c: positional color (write=top=download)
            let write_rate = format_bytes_rate_compact(s.disk.write_bytes_sec as f64);
            let write_label = format!(" ↓ write {write_rate} ");
            let label_w = unicode_width::UnicodeWidthStr::width(write_label.as_str()).min(inner.width as usize);
            if label_w > 0 && top_area.height > 0 {
                f.render_widget(
                    Paragraph::new(Line::from(Span::styled(write_label, Style::default().fg(theme.net_download)))),
                    Rect::new(top_area.x, top_area.y, label_w as u16, 1),
                );
            }
        }

        // Muted baseline at center
        let baseline_y = y_cursor + disk_half_h;
        if baseline_y < inner.y + inner.height {
            let baseline_chars: Vec<Span> = (0..inner.width).map(|_| {
                Span::styled("─", Style::default().fg(theme::baseline_color(theme)))
            }).collect();
            f.render_widget(Paragraph::new(Line::from(baseline_chars)), Rect::new(inner.x, baseline_y, inner.width, 1));
        }

        // Read ↑ bottom half (grows DOWNWARD from center baseline — like upload in net)
        let bottom_start = baseline_y + 1;
        let bottom_disk_h = (inner.y + inner.height).saturating_sub(bottom_start);
        if bottom_disk_h > 0 && bottom_start < inner.y + inner.height {
            let bottom_area = Rect::new(inner.x, bottom_start, inner.width, bottom_disk_h);
            let clamped: Vec<f64> = read_data.iter().map(|&v| v.max(disk_baseline)).collect();
            let graph = braille::render_braille_graph_down(&clamped, disk_scale, inner.width as usize, bottom_disk_h as usize, theme);
            let needed = inner.width as usize * 2;
            let start = read_data.len().saturating_sub(needed);
            let visible_orig = &read_data[start..];
            for (row_idx, row) in graph.iter().enumerate() {
                let y = bottom_area.y + row_idx as u16;
                if y >= bottom_area.y + bottom_area.height { break; }
                let y_frac = row_idx as f64 / (bottom_disk_h as f64 - 1.0).max(1.0);
                let gradient_color = super::gradient::value_to_color(y_frac, theme);
                let spans: Vec<Span> = row.iter().enumerate().map(|(col, &(ch, _))| {
                    let orig_l = visible_orig.get(col * 2).copied().unwrap_or(0.0);
                    let orig_r = visible_orig.get(col * 2 + 1).copied().unwrap_or(0.0);
                    let is_baseline = orig_l < disk_baseline * 2.0 && orig_r < disk_baseline * 2.0;
                    let color = if is_baseline { theme::baseline_color(theme) } else { gradient_color };
                    Span::styled(ch.to_string(), Style::default().fg(color))
                }).collect();
                if !spans.is_empty() {
                    f.render_widget(Paragraph::new(Line::from(spans)), Rect::new(bottom_area.x, y, bottom_area.width, 1));
                }
            }

            // I42-F3b: space between arrow and word; I42-F3c: positional color (read=bottom=upload)
            let read_rate = format_bytes_rate_compact(s.disk.read_bytes_sec as f64);
            let read_label = format!(" ↑ read {read_rate} ");
            let label_w = unicode_width::UnicodeWidthStr::width(read_label.as_str()).min(inner.width as usize);
            let label_y = bottom_area.y + bottom_area.height.saturating_sub(1);
            if label_w > 0 && bottom_area.height > 0 {
                f.render_widget(
                    Paragraph::new(Line::from(Span::styled(read_label, Style::default().fg(theme.net_upload)))),
                    Rect::new(bottom_area.x, label_y, label_w as u16, 1),
                );
            }
        }
    }
}

fn draw_network_expanded(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    let (total_rx, total_tx) = s.network.interfaces.iter().fold((0.0, 0.0), |(rx, tx), i| {
        (rx + i.rx_bytes_sec, tx + i.tx_bytes_sec)
    });

    // F1: dim border matching non-expanded
    let border_color = theme::dim_color(theme.net_download, theme::adaptive_border_dim(theme));

    // F3: idle detection matching non-expanded
    let net_idle = total_rx < 1024.0 && total_tx < 1024.0;

    let mut title_spans = vec![
        Span::styled(format!(" {}", theme::PANEL_SUPERSCRIPTS[3]), Style::default().fg(theme.muted)),
        Span::styled("net ", Style::default().fg(theme.net_upload).bold()),
    ];
    if net_idle {
        title_spans.push(Span::styled("(idle) ", Style::default().fg(theme.muted)));
    } else {
        // F5: use format_bytes_rate_compact matching UI style
        title_spans.push(Span::styled(format!("↑ {}", format_bytes_rate_compact(total_tx)), Style::default().fg(theme.net_upload)));
        title_spans.push(Span::styled("  ", Style::default()));
        title_spans.push(Span::styled(format!("↓ {}", format_bytes_rate_compact(total_rx)), Style::default().fg(theme.net_download)));
    }
    title_spans.push(Span::raw(" "));

    // F2: scale label matching non-expanded
    let tier_idx = state.history.net_tier_idx;
    let scale_label = super::panels::NET_TIERS[tier_idx].1;
    let right_title = Line::from(vec![
        Span::styled(format!("100%={} ", scale_label), Style::default().fg(theme.muted)),
    ]);

    let block = Block::default()
        .title(Line::from(title_spans))
        .title_top(right_title.alignment(ratatui::layout::Alignment::Right))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .border_type(BorderType::Rounded);

    let raw_inner = block.inner(area);
    f.render_widget(block, area);
    let inner = Rect::new(raw_inner.x + 1, raw_inner.y, raw_inner.width.saturating_sub(2), raw_inner.height);

    if inner.height == 0 || inner.width == 0 { return; }

    let scale = super::panels::NET_TIERS[tier_idx].0;
    // F4: baseline floor matching non-expanded (0.035 not 0.005)
    let baseline_floor = scale * 0.035;

    // F7: Symmetric center-baseline chart (~60% of panel height, capped at 20 rows)
    let chart_height = (inner.height * 6 / 10).clamp(4, 20);
    let half_h = chart_height / 2;

    let download_data: Vec<f64> = state.history.net_download.iter().copied().collect();
    let upload_data: Vec<f64> = state.history.net_upload.iter().copied().collect();

    // Download TOP half: bars grow upward from center
    if half_h > 0 {
        let top_area = Rect::new(inner.x, inner.y, inner.width, half_h);
        let clamped_dl: Vec<f64> = download_data.iter().map(|&v| v.max(baseline_floor)).collect();
        let graph = braille::render_braille_graph(&clamped_dl, scale, inner.width as usize, half_h as usize, theme);
        let needed = inner.width as usize * 2;
        let start = download_data.len().saturating_sub(needed);
        let visible_orig = &download_data[start..];
        for (row_idx, row) in graph.iter().enumerate() {
            let y = top_area.y + top_area.height.saturating_sub(1) - row_idx as u16;
            if y < top_area.y { break; }
            let y_frac = row_idx as f64 / (half_h as f64 - 1.0).max(1.0);
            let gradient_color = super::gradient::value_to_color(y_frac, theme);
            let spans: Vec<Span> = row.iter().enumerate().map(|(col, &(ch, _))| {
                let orig_l = visible_orig.get(col * 2).copied().unwrap_or(0.0);
                let orig_r = visible_orig.get(col * 2 + 1).copied().unwrap_or(0.0);
                let is_baseline = orig_l < baseline_floor * 2.0 && orig_r < baseline_floor * 2.0;
                let color = if is_baseline { theme::baseline_color(theme) } else { gradient_color };
                Span::styled(ch.to_string(), Style::default().fg(color))
            }).collect();
            if !spans.is_empty() {
                f.render_widget(Paragraph::new(Line::from(spans)), Rect::new(top_area.x, y, top_area.width, 1));
            }
        }
    }

    // Upload BOTTOM half: bars grow downward from center
    let bottom_h = chart_height - half_h;
    if bottom_h > 0 {
        let bottom_area = Rect::new(inner.x, inner.y + half_h, inner.width, bottom_h);
        let clamped_ul: Vec<f64> = upload_data.iter().map(|&v| v.max(baseline_floor)).collect();
        let graph = braille::render_braille_graph_down(&clamped_ul, scale, inner.width as usize, bottom_h as usize, theme);
        let needed = inner.width as usize * 2;
        let start = upload_data.len().saturating_sub(needed);
        let visible_orig = &upload_data[start..];
        for (row_idx, row) in graph.iter().enumerate() {
            let y = bottom_area.y + row_idx as u16;
            if y >= bottom_area.y + bottom_area.height { break; }
            let y_frac = row_idx as f64 / (bottom_h as f64 - 1.0).max(1.0);
            let gradient_color = super::gradient::value_to_color(y_frac, theme);
            let spans: Vec<Span> = row.iter().enumerate().map(|(col, &(ch, _))| {
                let orig_l = visible_orig.get(col * 2).copied().unwrap_or(0.0);
                let orig_r = visible_orig.get(col * 2 + 1).copied().unwrap_or(0.0);
                let is_baseline = orig_l < baseline_floor * 2.0 && orig_r < baseline_floor * 2.0;
                let color = if is_baseline { theme::baseline_color(theme) } else { gradient_color };
                Span::styled(ch.to_string(), Style::default().fg(color))
            }).collect();
            if !spans.is_empty() {
                f.render_widget(Paragraph::new(Line::from(spans)), Rect::new(bottom_area.x, y, bottom_area.width, 1));
            }
        }
    }

    // Per-interface detailed stats (filter infrastructure interfaces consistently)
    let mut sorted_ifaces: Vec<&crate::metrics::NetInterface> = s.network.interfaces.iter()
        .filter(|i| !is_infrastructure_interface(&i.name))
        .collect();
    sorted_ifaces.sort_by(|a, b| {
        let a_total = a.rx_bytes_sec + a.tx_bytes_sec;
        let b_total = b.rx_bytes_sec + b.tx_bytes_sec;
        b_total.partial_cmp(&a_total).unwrap_or(std::cmp::Ordering::Equal)
    });

    // F6: max rates display matching non-expanded bottom bar
    let max_y = inner.y.saturating_add(chart_height);
    if max_y < inner.y.saturating_add(inner.height) {
        let max_text = format!(
            " max ↓{}  ↑{}",
            format_bytes_rate_compact(state.history.net_download_max),
            format_bytes_rate_compact(state.history.net_upload_max),
        );
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(max_text, Style::default().fg(theme.muted)))),
            Rect::new(inner.x, max_y, inner.width, 1),
        );
    }

    let header_y = max_y.saturating_add(1);
    if header_y < inner.y.saturating_add(inner.height) {
        let hdr = Line::from(vec![
            Span::styled(format!("{:<10} {:>14} {:>10} {:>10} {:>8} {:>8}", "interface", "type", "baudrate", "upload", "download", "pkt in"), Style::default().fg(theme.muted)),
        ]);
        f.render_widget(Paragraph::new(hdr), Rect::new(inner.x, header_y, inner.width, 1));
    }

    let mut cur_y = header_y.saturating_add(1);
    for iface in &sorted_ifaces {
        if cur_y >= inner.y.saturating_add(inner.height) { break; }
        let line = Line::from(vec![
            Span::styled(format!("{:<10}", iface.name), Style::default().fg(theme.fg)),
            Span::styled(format!(" {:>14}", iface.iface_type), Style::default().fg(theme.muted)),
            Span::styled(format!(" {:>10}", format_baudrate(iface.baudrate)), Style::default().fg(theme.muted)),
            Span::styled(format!("  ↑{:>8}", format_bytes_rate_compact(iface.tx_bytes_sec)), Style::default().fg(theme.net_upload)),
            Span::styled(format!("  ↓{:>8}", format_bytes_rate_compact(iface.rx_bytes_sec)), Style::default().fg(theme.net_download)),
            Span::styled(format!(" {:>7.0}", iface.packets_in_sec), Style::default().fg(theme.muted)),
        ]);
        f.render_widget(Paragraph::new(line), Rect::new(inner.x, cur_y, inner.width, 1));
        cur_y += 1;

        // Per-interface rx sparkline using interface-specific history
        if cur_y < inner.y.saturating_add(inner.height) {
            if let Some((rx_buf, _)) = state.history.per_iface.get(&iface.name) {
                let iface_scale = scale;
                let rx_data: Vec<f64> = rx_buf.iter().copied().collect();
                let spark = braille::render_braille_sparkline(&rx_data, iface_scale, inner.width as usize, theme);
                let spark_spans: Vec<Span> = spark.iter()
                    .map(|&(ch, color)| Span::styled(ch.to_string(), Style::default().fg(color)))
                    .collect();
                if !spark_spans.is_empty() {
                    f.render_widget(Paragraph::new(Line::from(spark_spans)), Rect::new(inner.x, cur_y, inner.width, 1));
                }
            }
            cur_y += 1;
        }
    }
}

fn draw_power_expanded(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    // F3: dim border matching non-expanded
    let border_color = theme::dim_color(theme.power_accent, theme::adaptive_border_dim(theme));

    // F1: available guard matching non-expanded
    if !s.power.available {
        let block = Block::default()
            .title(Line::from(vec![
                Span::styled(format!(" {}", theme::PANEL_SUPERSCRIPTS[4]), Style::default().fg(theme.muted)),
                Span::styled("power ", Style::default().fg(theme.fg).bold()),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .border_type(BorderType::Rounded);
        let raw_inner = block.inner(area);
        f.render_widget(block, area);
        let inner = Rect::new(raw_inner.x + 1, raw_inner.y, raw_inner.width.saturating_sub(2), raw_inner.height);
        f.render_widget(
            Paragraph::new("power sensors: N/A").style(Style::default().fg(theme.muted)),
            inner,
        );
        return;
    }

    let total_w = s.power.package_w.max(s.power.cpu_w + s.power.gpu_w + s.power.ane_w + s.power.dram_w);
    // F7: avg/max matching non-expanded bottom bar
    let avg_w = if !state.history.package_power.is_empty() {
        let sum: f64 = state.history.package_power.iter().sum();
        sum / state.history.package_power.len() as f64
    } else {
        total_w as f64
    };
    let max_w = state.history.package_power.iter().copied().fold(0.0_f64, f64::max);

    let title_spans = vec![
        Span::styled(format!(" {}", theme::PANEL_SUPERSCRIPTS[4]), Style::default().fg(theme.muted)),
        Span::styled("power  ", Style::default().fg(theme.power_accent).bold()),
        Span::styled(format!("{:.1}W total  avg {:.1}W  max {:.1}W", total_w, avg_w, max_w), Style::default().fg(theme.fg)),
        Span::raw(" "),
    ];

    let block = Block::default()
        .title(Line::from(title_spans))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .border_type(BorderType::Rounded);

    let raw_inner = block.inner(area);
    f.render_widget(block, area);
    let inner = Rect::new(raw_inner.x + 1, raw_inner.y, raw_inner.width.saturating_sub(2), raw_inner.height);

    if inner.height == 0 || inner.width == 0 { return; }

    // F8: CPU power multi-row chart (~30% height, capped at 10 rows)
    let cpu_chart_h = (inner.height * 3 / 10).clamp(2, 10);
    let cpu_tdp = s.soc.cpu_tdp_w() as f64;
    let cpu_data: Vec<f64> = state.history.cpu_power.iter().copied().collect();
    f.render_widget(
        Paragraph::new("cpu power").style(Style::default().fg(theme.cpu_accent)),
        Rect::new(inner.x, inner.y, inner.width, 1),
    );
    let cpu_chart_area = Rect::new(inner.x, inner.y + 1, inner.width, cpu_chart_h);
    // F2: use render_graph_with_baseline matching non-expanded
    render_graph_with_baseline(f, cpu_chart_area, &cpu_data, cpu_tdp, theme);

    // F6: 1-row padding between CPU and GPU charts
    let gpu_chart_start = inner.y + 1 + cpu_chart_h + 1;
    // F8: GPU chart capped at 10 rows
    let gpu_chart_h = (inner.height * 3 / 10).clamp(2, 10).min(inner.height.saturating_sub(2 + cpu_chart_h + 1));
    let gpu_tdp = s.soc.gpu_tdp_w() as f64;
    let gpu_data: Vec<f64> = state.history.gpu_power.iter().copied().collect();
    if gpu_chart_start < inner.y + inner.height {
        f.render_widget(
            Paragraph::new("gpu power").style(Style::default().fg(theme.gpu_accent)),
            Rect::new(inner.x, gpu_chart_start, inner.width, 1),
        );
        if gpu_chart_h > 0 && gpu_chart_start + 1 < inner.y + inner.height {
            let gpu_chart_area = Rect::new(inner.x, gpu_chart_start + 1, inner.width, gpu_chart_h);
            render_graph_with_baseline(f, gpu_chart_area, &gpu_data, gpu_tdp, theme);
        }
    }

    // F6: 1-row padding before breakdown
    let breakdown_y = gpu_chart_start + 1 + gpu_chart_h + 1;
    let components = [
        ("CPU", s.power.cpu_w, theme.cpu_accent),
        ("GPU", s.power.gpu_w, theme.gpu_accent),
        ("ANE", s.power.ane_w, theme.power_accent),
        ("DRAM", s.power.dram_w, theme.mem_accent),
        ("system", s.power.system_w, theme.muted),
        ("package", s.power.package_w, theme.fg),
    ];

    if breakdown_y < inner.y + inner.height {
        f.render_widget(
            Paragraph::new("component breakdown").style(Style::default().fg(theme.muted)),
            Rect::new(inner.x, breakdown_y, inner.width, 1),
        );
    }

    let bar_width = inner.width.saturating_sub(18) as usize;
    let max_component = components.iter().map(|(_, w, _)| *w).fold(0.0f32, f32::max).max(0.01);

    for (i, (name, watts, color)) in components.iter().enumerate() {
        let y = breakdown_y + 1 + i as u16;
        if y >= inner.y + inner.height { break; }

        let norm = (*watts / max_component).clamp(0.0, 1.0);
        let filled = (bar_width as f32 * norm) as usize;
        let empty = bar_width.saturating_sub(filled);

        // F4: use {:.1}W precision matching non-expanded
        let line = Line::from(vec![
            Span::styled(format!("{:<7}", name), Style::default().fg(*color)),
            Span::styled("▓".repeat(filled), Style::default().fg(*color)),
            Span::styled("░".repeat(empty), Style::default().fg(theme.border)),
            Span::styled(format!(" {:.1}W", watts), Style::default().fg(theme.fg)),
        ]);
        f.render_widget(Paragraph::new(line), Rect::new(inner.x, y, inner.width, 1));
    }

    // Fan speeds
    // F6: 1-row padding before fans
    let mut fan_y = breakdown_y + 1 + components.len() as u16 + 1;
    if !s.temperature.fan_speeds.is_empty() && fan_y < inner.y.saturating_add(inner.height) {
        let fan_text: String = s.temperature.fan_speeds.iter().enumerate()
            .map(|(i, rpm)| format!("fan {}: {} RPM", i, rpm))
            .collect::<Vec<_>>()
            .join("  ");
        f.render_widget(
            Paragraph::new(fan_text).style(Style::default().fg(theme.muted)),
            Rect::new(inner.x, fan_y, inner.width, 1),
        );
        fan_y += 1;
    }

    // Per-process energy ranking
    let mut procs_by_power: Vec<&crate::metrics::ProcessInfo> = s.processes.iter()
        .filter(|p| p.power_w >= 0.05)
        .collect();
    procs_by_power.sort_by(|a, b| b.power_w.partial_cmp(&a.power_w).unwrap_or(std::cmp::Ordering::Equal));

    // F6: 1-row padding before process list
    let proc_start_y = fan_y.saturating_add(1);
    if proc_start_y < inner.y.saturating_add(inner.height) {
        f.render_widget(
            Paragraph::new("top processes by power").style(Style::default().fg(theme.muted)),
            Rect::new(inner.x, proc_start_y, inner.width, 1),
        );
    }

    // F5: dynamic name width from available space
    let name_width = inner.width.saturating_sub(8) as usize;

    for (i, proc) in procs_by_power.iter().take(10).enumerate() {
        let y = proc_start_y.saturating_add(1).saturating_add(i as u16);
        if y >= inner.y.saturating_add(inner.height) { break; }
        // F4: use {:.1}W precision
        let line = Line::from(vec![
            Span::styled(format!("{:<w$}", truncate_with_ellipsis(&proc.name, name_width), w = name_width), Style::default().fg(theme.fg)),
            Span::styled(format!("  {:.1}W", proc.power_w), Style::default().fg(theme.power_accent)),
        ]);
        f.render_widget(Paragraph::new(line), Rect::new(inner.x, y, inner.width, 1));
    }
}

fn draw_process_expanded(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    // F6: dim border matching non-expanded
    let border_color = theme::dim_color(theme.process_accent, theme::adaptive_border_dim(theme));

    // F7: sort indicator moved to bottom-right, removed from title
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
    let inner = Rect::new(raw_inner.x + 1, raw_inner.y, raw_inner.width.saturating_sub(2), raw_inner.height);

    if inner.width == 0 || inner.height < 3 { return; }

    let gb = 1024.0 * 1024.0 * 1024.0;
    let mb = 1024.0 * 1024.0;

    // F8: expanded-specific column widths (adds io r, io w, user to base)
    let col_io: usize = 7;
    let col_user: usize = 9; // " " + 8 chars
    let expanded_fixed = COL_FIXED_TOTAL + col_io + col_io + col_user;
    // F4: dynamic name width
    let panel_width = inner.width as usize;
    let name_width = panel_width.saturating_sub(expanded_fixed).max(4);

    // F1: pid first; F2: all headers muted
    let header = Line::from(vec![
        Span::styled(format!("{:<w$}", "pid", w = COL_PID), Style::default().fg(theme.muted)),
        Span::styled(" ", Style::default()),
        Span::styled(pad_to_display_width("name", name_width), Style::default().fg(theme.muted)),
        Span::styled(format!("{:>w$}", "cpu", w = COL_CPU + 2), Style::default().fg(theme.muted)),
        Span::styled(format!("{:>w$}", "mem", w = COL_MEM + 2), Style::default().fg(theme.muted)),
        Span::styled(format!("{:>w$}", "pow", w = COL_POW + 2), Style::default().fg(theme.muted)),
        Span::styled(format!("{:>w$}", "thread", w = COL_THR), Style::default().fg(theme.muted)),
        Span::styled(format!("{:>w$}", "io r", w = col_io), Style::default().fg(theme.muted)),
        Span::styled(format!("{:>w$}", "io w", w = col_io), Style::default().fg(theme.muted)),
        Span::styled(format!(" {:<8}", "user"), Style::default().fg(theme.muted)),
    ]);
    f.render_widget(Paragraph::new(header), Rect::new(inner.x, inner.y, inner.width, 1));

    let procs = &s.processes;
    let max_cpu = procs.iter().map(|p| p.cpu_pct).fold(0.0f32, f32::max);
    let max_mem = procs.iter().map(|p| p.mem_bytes).max().unwrap_or(1).max(1);
    let max_power = procs.iter().map(|p| p.power_w).fold(0.0f32, f32::max);

    let mut indices: Vec<usize> = (0..procs.len()).collect();
    sort_indices(&mut indices, procs, state.sort_mode, max_cpu, max_mem, max_power);

    if indices.is_empty() {
        if inner.height > 1 {
            f.render_widget(
                Paragraph::new("no processes").style(Style::default().fg(theme.muted)),
                Rect::new(inner.x, inner.y + 1, inner.width, 1),
            );
        }
        return;
    }

    // Reserve header + sort indicator rows
    let scroll = state.process_scroll.min(indices.len().saturating_sub(1));
    let max_visible = inner.height.saturating_sub(2) as usize;

    for (i, &idx) in indices.iter().skip(scroll).take(max_visible).enumerate() {
        let y = inner.y + 1 + i as u16;
        if y >= inner.y + inner.height.saturating_sub(1) { break; }

        let proc = &procs[idx];

        // F3: CJK-aware name truncation and padding
        let name_trunc = truncate_by_display_width(&proc.name, name_width);
        let name_padded = pad_to_display_width(&name_trunc, name_width);

        let mem_display = if proc.mem_bytes as f64 >= gb {
            format!("{:.1}G", proc.mem_bytes as f64 / gb)
        } else {
            format!("{:.0}M", proc.mem_bytes as f64 / mb)
        };

        let cpu_norm = if max_cpu > 0.0 { (proc.cpu_pct / max_cpu).clamp(0.0, 1.0) as f64 } else { 0.0 };
        let mem_norm = if max_mem > 0 { (proc.mem_bytes as f64 / max_mem as f64).clamp(0.0, 1.0) } else { 0.0 };
        let pwr_norm = if max_power > 0.0 { (proc.power_w / max_power).clamp(0.0, 1.0) as f64 } else { 0.0 };

        // F5: dot thresholds matching non-expanded
        let cpu_dot_color = if proc.cpu_pct < 0.1 { theme.muted } else { gradient::value_to_color(cpu_norm, theme) };
        let mem_dot_color = if proc.mem_bytes < 1_048_576 { theme.muted } else { gradient::value_to_color(mem_norm, theme) };
        let pow_dot_color = if proc.power_w < 0.1 { theme.muted } else { gradient::value_to_color(pwr_norm, theme) };

        // F1: pid first; F9: use \u{2022} consistently
        let line = Line::from(vec![
            Span::styled(format!("{:<w$}", proc.pid, w = COL_PID), Style::default().fg(theme.muted)),
            Span::styled(" ", Style::default()),
            Span::styled(name_padded, Style::default().fg(theme.fg)),
            Span::styled(" \u{2022}", Style::default().fg(cpu_dot_color)),
            Span::styled(format!("{:>w$.1}", proc.cpu_pct, w = COL_CPU), Style::default().fg(theme.fg)),
            Span::styled(" \u{2022}", Style::default().fg(mem_dot_color)),
            Span::styled(format!("{:>w$}", mem_display, w = COL_MEM), Style::default().fg(theme.fg)),
            Span::styled(" \u{2022}", Style::default().fg(pow_dot_color)),
            Span::styled(format!("{:>w$.1}", proc.power_w, w = COL_POW), Style::default().fg(theme.fg)),
            Span::styled(format!("{:>w$}", proc.thread_count, w = COL_THR), Style::default().fg(theme.fg)),
            Span::styled(format!("{:>w$}", format_bytes_rate_compact(proc.io_read_bytes_sec), w = col_io), Style::default().fg(theme.muted)),
            Span::styled(format!("{:>w$}", format_bytes_rate_compact(proc.io_write_bytes_sec), w = col_io), Style::default().fg(theme.muted)),
            Span::styled(format!(" {:<8}", truncate_with_ellipsis(&proc.user, 8)), Style::default().fg(theme.muted)),
        ]);
        f.render_widget(Paragraph::new(line), Rect::new(inner.x, y, inner.width, 1));
    }

    // F7: sort indicator at bottom-right matching non-expanded
    let sort_y = inner.y + inner.height.saturating_sub(1);
    let sort_text = format!("sort: {} \u{2193}", state.sort_mode.label());
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(sort_text, Style::default().fg(theme.muted)))
            .alignment(ratatui::layout::Alignment::Right)),
        Rect::new(inner.x, sort_y, inner.width, 1),
    );
}
