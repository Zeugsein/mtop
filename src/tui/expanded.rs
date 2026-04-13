//! Expanded panel renderers extracted from mod.rs (iteration 8).

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::metrics::MetricsSnapshot;
use crate::platform::network::speed_tier_from_baudrate;
use super::{AppState, PanelId, theme, braille, gauge, gradient};
use super::helpers::{format_bytes_rate, format_bytes_rate_compact, truncate_with_ellipsis, is_infrastructure_interface, format_baudrate, temp_color, sort_indices, CPU_TEMP_WARN, CPU_TEMP_CRIT, GPU_TEMP_WARN, GPU_TEMP_CRIT};


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
    let temp_col = if s.temperature.available {
        temp_color(s.temperature.cpu_avg_c, CPU_TEMP_WARN, CPU_TEMP_CRIT)
    } else {
        theme.muted
    };
    let title_spans = vec![
        Span::styled("¹ CPU  ", Style::default().fg(theme.cpu_accent).bold()),
        Span::styled(format!("{:.1}%", cpu_pct), Style::default().fg(theme.fg)),
        Span::styled(format!("  {:.1}W", s.power.cpu_w), Style::default().fg(theme.muted)),
        if s.temperature.available {
            Span::styled(format!("  {:.0}°C", s.temperature.cpu_avg_c), Style::default().fg(temp_col))
        } else {
            Span::raw("")
        },
        Span::raw(" "),
    ];

    let block = Block::default()
        .title(Line::from(title_spans))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.cpu_accent))
        .border_type(BorderType::Rounded);

    let raw_inner = block.inner(area);
    f.render_widget(block, area);
    let inner = Rect::new(raw_inner.x + 1, raw_inner.y + 1, raw_inner.width.saturating_sub(2), raw_inner.height.saturating_sub(2));

    if inner.height == 0 || inner.width == 0 { return; }

    // Top section: sparkline
    let spark_height = 2.min(inner.height);
    let sparkline_data: Vec<f64> = state.history.cpu_usage.iter().copied().collect();
    let spark_width = inner.width as usize;
    let spark = braille::render_braille_sparkline(&sparkline_data, 1.0, spark_width, theme);
    let spark_spans: Vec<Span> = spark.iter()
        .map(|&(ch, color)| Span::styled(ch.to_string(), Style::default().fg(color)))
        .collect();
    if !spark_spans.is_empty() {
        f.render_widget(Paragraph::new(Line::from(spark_spans)), Rect::new(inner.x, inner.y, inner.width, 1));
    }

    // Per-core usage bars
    let core_start_y = inner.y + spark_height;
    let available_rows = inner.height.saturating_sub(spark_height) as usize;

    // E-cluster header
    if available_rows > 0 {
        f.render_widget(
            Paragraph::new(format!("E-cluster: {:.0}% @ {}MHz", s.cpu.e_cluster.usage * 100.0, s.cpu.e_cluster.freq_mhz))
                .style(Style::default().fg(theme.cpu_accent)),
            Rect::new(inner.x, core_start_y, inner.width, 1),
        );
    }

    let bar_width = inner.width.saturating_sub(12) as usize;
    let e_count = s.soc.e_cores as usize;
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

    // P-cluster header and cores
    let p_header_y = core_start_y + 1 + e_count as u16;
    if p_header_y < inner.y + inner.height {
        f.render_widget(
            Paragraph::new(format!("P-cluster: {:.0}% @ {}MHz", s.cpu.p_cluster.usage * 100.0, s.cpu.p_cluster.freq_mhz))
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
    let temp_col = if s.temperature.available {
        temp_color(s.temperature.gpu_avg_c, GPU_TEMP_WARN, GPU_TEMP_CRIT)
    } else {
        theme.muted
    };
    let idle_suffix = if s.power.gpu_w < 0.5 { " (idle)" } else { "" };
    let title_spans = vec![
        Span::styled(format!("² GPU{}  ", idle_suffix), Style::default().fg(theme.gpu_accent).bold()),
        Span::styled(format!("{:.1}%", s.gpu.usage * 100.0), Style::default().fg(theme.fg)),
        if s.temperature.available {
            Span::styled(format!("  {:.0}°C", s.temperature.gpu_avg_c), Style::default().fg(temp_col))
        } else {
            Span::raw("")
        },
        Span::raw(" "),
    ];

    let block = Block::default()
        .title(Line::from(title_spans))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.gpu_accent))
        .border_type(BorderType::Rounded);

    let raw_inner = block.inner(area);
    f.render_widget(block, area);
    let inner = Rect::new(raw_inner.x + 1, raw_inner.y + 1, raw_inner.width.saturating_sub(2), raw_inner.height.saturating_sub(2));

    if inner.height == 0 || inner.width == 0 { return; }

    // Sparkline at top (gradient coloring like CPU)
    let sparkline_data: Vec<f64> = state.history.gpu_usage.iter().copied().collect();
    let spark = braille::render_braille_sparkline(&sparkline_data, 1.0, inner.width as usize, theme);
    let spark_spans: Vec<Span> = spark.iter()
        .map(|&(ch, color)| Span::styled(ch.to_string(), Style::default().fg(color)))
        .collect();
    if !spark_spans.is_empty() {
        f.render_widget(Paragraph::new(Line::from(spark_spans)), Rect::new(inner.x, inner.y, inner.width, 1));
    }

    // Detailed metrics table (GPU Cores and Memory lines removed per SHALL-28-12)
    let metrics = [
        format!("frequency:    {} MHz", s.gpu.freq_mhz),
        format!("usage:        {:.1}%", s.gpu.usage * 100.0),
        format!("GPU power:    {:.2} W", s.power.gpu_w),
        format!("ANE power:    {:.2} W", s.power.ane_w),
        format!("DRAM power:   {:.2} W", s.power.dram_w),
    ];

    for (i, text) in metrics.iter().enumerate() {
        let y = inner.y + 2 + i as u16;
        if y >= inner.y + inner.height || text.is_empty() { continue; }
        f.render_widget(
            Paragraph::new(text.as_str()).style(Style::default().fg(theme.fg)),
            Rect::new(inner.x, y, inner.width, 1),
        );
    }
}

fn draw_mem_disk_expanded(f: &mut Frame, area: Rect, s: &MetricsSnapshot, _state: &AppState, theme: &theme::Theme) {
    let gb = 1024.0 * 1024.0 * 1024.0;
    let ram_used_gb = s.memory.ram_used as f64 / gb;
    let ram_total_gb = s.memory.ram_total as f64 / gb;

    let title_spans = vec![
        Span::styled("³ Memory  ", Style::default().fg(theme.mem_accent).bold()),
        Span::styled(format!("{:.1}/{:.0} GB", ram_used_gb, ram_total_gb), Style::default().fg(theme.fg)),
        Span::raw(" "),
    ];

    let block = Block::default()
        .title(Line::from(title_spans))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.mem_accent))
        .border_type(BorderType::Rounded);

    let raw_inner = block.inner(area);
    f.render_widget(block, area);
    let inner = Rect::new(raw_inner.x + 1, raw_inner.y + 1, raw_inner.width.saturating_sub(2), raw_inner.height.saturating_sub(2));

    if inner.height == 0 || inner.width == 0 { return; }

    // RAM gauge
    let bar_width = inner.width.saturating_sub(16) as usize;
    let ram_label = format!("{:.1}/{:.0} GB", ram_used_gb, ram_total_gb);
    let ram_gauge = gauge::render_gauge_bar(s.memory.ram_used as f64, s.memory.ram_total as f64, bar_width, &ram_label, theme);
    if inner.height > 1 {
        f.render_widget(Paragraph::new(Line::from(vec![
            Span::styled("RAM  ", Style::default().fg(theme.mem_accent)),
        ])), Rect::new(inner.x, inner.y, inner.width, 1));
        f.render_widget(Paragraph::new(Line::from(ram_gauge)), Rect::new(inner.x, inner.y + 1, inner.width, 1));
    }

    // Swap gauge
    let swap_used_gb = s.memory.swap_used as f64 / gb;
    let swap_total_gb = s.memory.swap_total as f64 / gb;
    let swap_label = format!("{:.1}/{:.1} GB", swap_used_gb, swap_total_gb);
    let swap_gauge = gauge::render_gauge_bar(s.memory.swap_used as f64, s.memory.swap_total as f64, bar_width, &swap_label, theme);
    if inner.height > 3 {
        f.render_widget(Paragraph::new(Line::from(vec![
            Span::styled("swap ", Style::default().fg(theme.muted)),
        ])), Rect::new(inner.x, inner.y + 3, inner.width, 1));
        f.render_widget(Paragraph::new(Line::from(swap_gauge)), Rect::new(inner.x, inner.y + 4, inner.width, 1));
    }

    // Memory pressure stacked gauge
    let mut pressure_y = inner.y + 6;
    if inner.height > 6 && (s.memory.wired > 0 || s.memory.app > 0 || s.memory.compressed > 0) {
        let wired_gb = s.memory.wired as f64 / gb;
        let app_gb = s.memory.app as f64 / gb;
        let compressed_gb = s.memory.compressed as f64 / gb;
        let total = ram_total_gb.max(0.01);

        f.render_widget(
            Paragraph::new("memory pressure").style(Style::default().fg(theme.muted)),
            Rect::new(inner.x, pressure_y, inner.width, 1),
        );
        pressure_y += 1;

        // Stacked gauge: [wired|app|compressed|free]
        let gauge_width = inner.width.saturating_sub(2) as usize;
        if pressure_y < inner.y + inner.height && gauge_width > 0 {
            let w_frac = (wired_gb / total).clamp(0.0, 1.0);
            let a_frac = (app_gb / total).clamp(0.0, 1.0);
            let c_frac = (compressed_gb / total).clamp(0.0, 1.0);
            let w_chars = (gauge_width as f64 * w_frac) as usize;
            let a_chars = (gauge_width as f64 * a_frac) as usize;
            let c_chars = (gauge_width as f64 * c_frac) as usize;
            let free_chars = gauge_width.saturating_sub(w_chars + a_chars + c_chars);

            let line = Line::from(vec![
                Span::styled("▓".repeat(w_chars), Style::default().fg(theme.cpu_accent)),
                Span::styled("▓".repeat(a_chars), Style::default().fg(theme.mem_accent)),
                Span::styled("▓".repeat(c_chars), Style::default().fg(theme.power_accent)),
                Span::styled("░".repeat(free_chars), Style::default().fg(theme.border)),
            ]);
            f.render_widget(Paragraph::new(line), Rect::new(inner.x, pressure_y, inner.width, 1));
            pressure_y += 1;
        }

        // Legend
        let pressure_items = [
            ("■ wired: ", wired_gb, theme.cpu_accent),
            ("  ■ app: ", app_gb, theme.mem_accent),
            ("  ■ compressed: ", compressed_gb, theme.power_accent),
        ];
        if pressure_y < inner.y + inner.height {
            let legend: Vec<Span> = pressure_items.iter().flat_map(|(label, val, color)| {
                vec![
                    Span::styled(*label, Style::default().fg(*color)),
                    Span::styled(format!("{:.1}G", val), Style::default().fg(theme.fg)),
                ]
            }).collect();
            f.render_widget(Paragraph::new(Line::from(legend)), Rect::new(inner.x, pressure_y, inner.width, 1));
            pressure_y += 1;
        }
    }

    // Disk info
    let disk_start = pressure_y + 1;
    let disk_used_gb = s.disk.used_bytes as f64 / gb;
    let disk_total_gb = s.disk.total_bytes as f64 / gb;
    if disk_start < inner.y + inner.height {
        let disk_metrics = [
            format!("disk: {:.0}/{:.0} GB", disk_used_gb, disk_total_gb),
            format!("read:  {}", format_bytes_rate(s.disk.read_bytes_sec as f64)),
            format!("write: {}", format_bytes_rate(s.disk.write_bytes_sec as f64)),
        ];
        for (i, text) in disk_metrics.iter().enumerate() {
            let y = disk_start + i as u16;
            if y >= inner.y + inner.height { break; }
            f.render_widget(
                Paragraph::new(text.as_str()).style(Style::default().fg(theme.fg)),
                Rect::new(inner.x, y, inner.width, 1),
            );
        }
    }
}

fn draw_network_expanded(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    let (total_rx, total_tx) = s.network.interfaces.iter().fold((0.0, 0.0), |(rx, tx), i| {
        (rx + i.rx_bytes_sec, tx + i.tx_bytes_sec)
    });

    let title_spans = vec![
        Span::styled("⁴ Network  ", Style::default().fg(theme.net_upload).bold()),
        Span::styled(format!("↑ {}", format_bytes_rate(total_tx)), Style::default().fg(theme.net_upload)),
        Span::styled("  ", Style::default()),
        Span::styled(format!("↓ {}", format_bytes_rate(total_rx)), Style::default().fg(theme.net_download)),
        Span::raw(" "),
    ];

    let block = Block::default()
        .title(Line::from(title_spans))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.net_download))
        .border_type(BorderType::Rounded);

    let raw_inner = block.inner(area);
    f.render_widget(block, area);
    let inner = Rect::new(raw_inner.x + 1, raw_inner.y + 1, raw_inner.width.saturating_sub(2), raw_inner.height.saturating_sub(2));

    if inner.height == 0 || inner.width == 0 { return; }

    let scale = speed_tier_from_baudrate(s.network.primary_baudrate) as f64;

    // Upload sparkline (gradient colors from braille renderer)
    let upload_data: Vec<f64> = state.history.net_upload.iter().copied().collect();
    let spark = braille::render_braille_sparkline(&upload_data, scale, inner.width as usize, theme);
    let spans: Vec<Span> = spark.iter()
        .map(|&(ch, color)| Span::styled(ch.to_string(), Style::default().fg(color)))
        .collect();
    if !spans.is_empty() {
        f.render_widget(Paragraph::new(Line::from(vec![
            Span::styled("upload ", Style::default().fg(theme.net_upload)),
        ])), Rect::new(inner.x, inner.y, inner.width, 1));
        f.render_widget(Paragraph::new(Line::from(spans)), Rect::new(inner.x, inner.y + 1, inner.width, 1));
    }

    // Download sparkline (gradient colors from braille renderer)
    let download_data: Vec<f64> = state.history.net_download.iter().copied().collect();
    let spark = braille::render_braille_sparkline(&download_data, scale, inner.width as usize, theme);
    let spans: Vec<Span> = spark.iter()
        .map(|&(ch, color)| Span::styled(ch.to_string(), Style::default().fg(color)))
        .collect();
    if !spans.is_empty() && inner.height > 3 {
        f.render_widget(Paragraph::new(Line::from(vec![
            Span::styled("download ", Style::default().fg(theme.net_download)),
        ])), Rect::new(inner.x, inner.y + 3, inner.width, 1));
        f.render_widget(Paragraph::new(Line::from(spans)), Rect::new(inner.x, inner.y + 4, inner.width, 1));
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

    let header_y = inner.y.saturating_add(6);
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
                let iface_scale = if iface.baudrate > 0 {
                    speed_tier_from_baudrate(iface.baudrate) as f64
                } else {
                    scale
                };
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
    let total_w = s.power.package_w.max(s.power.cpu_w + s.power.gpu_w + s.power.ane_w + s.power.dram_w);

    let title_spans = vec![
        Span::styled("⁵ Power  ", Style::default().fg(theme.power_accent).bold()),
        Span::styled(format!("{:.1}W total", total_w), Style::default().fg(theme.fg)),
        Span::raw(" "),
    ];

    let block = Block::default()
        .title(Line::from(title_spans))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.power_accent))
        .border_type(BorderType::Rounded);

    let raw_inner = block.inner(area);
    f.render_widget(block, area);
    let inner = Rect::new(raw_inner.x + 1, raw_inner.y + 1, raw_inner.width.saturating_sub(2), raw_inner.height.saturating_sub(2));

    if inner.height == 0 || inner.width == 0 { return; }

    // CPU power sparkline
    let cpu_tdp = s.soc.cpu_tdp_w() as f64;
    let cpu_data: Vec<f64> = state.history.cpu_power.iter().copied().collect();
    let spark = braille::render_braille_sparkline(&cpu_data, cpu_tdp, inner.width as usize, theme);
    let spans: Vec<Span> = spark.iter()
        .map(|&(ch, color)| Span::styled(ch.to_string(), Style::default().fg(color)))
        .collect();
    f.render_widget(
        Paragraph::new("cpu power").style(Style::default().fg(theme.cpu_accent)),
        Rect::new(inner.x, inner.y, inner.width, 1),
    );
    if !spans.is_empty() && inner.height > 1 {
        f.render_widget(Paragraph::new(Line::from(spans)), Rect::new(inner.x, inner.y + 1, inner.width, 1));
    }

    // GPU power sparkline
    let gpu_tdp = s.soc.gpu_tdp_w() as f64;
    let gpu_data: Vec<f64> = state.history.gpu_power.iter().copied().collect();
    let spark = braille::render_braille_sparkline(&gpu_data, gpu_tdp, inner.width as usize, theme);
    let spans: Vec<Span> = spark.iter()
        .map(|&(ch, color)| Span::styled(ch.to_string(), Style::default().fg(color)))
        .collect();
    if inner.height > 3 {
        f.render_widget(
            Paragraph::new("gpu power").style(Style::default().fg(theme.gpu_accent)),
            Rect::new(inner.x, inner.y + 3, inner.width, 1),
        );
        if !spans.is_empty() {
            f.render_widget(Paragraph::new(Line::from(spans)), Rect::new(inner.x, inner.y + 4, inner.width, 1));
        }
    }

    // Component breakdown
    let components = [
        ("CPU", s.power.cpu_w, theme.cpu_accent),
        ("GPU", s.power.gpu_w, theme.gpu_accent),
        ("ANE", s.power.ane_w, theme.power_accent),
        ("DRAM", s.power.dram_w, theme.mem_accent),
        ("system", s.power.system_w, theme.muted),
        ("package", s.power.package_w, theme.fg),
    ];

    if inner.height > 6 {
        f.render_widget(
            Paragraph::new("component breakdown").style(Style::default().fg(theme.muted)),
            Rect::new(inner.x, inner.y + 6, inner.width, 1),
        );
    }

    let bar_width = inner.width.saturating_sub(18) as usize;
    let max_component = components.iter().map(|(_, w, _)| *w).fold(0.0f32, f32::max).max(0.01);

    for (i, (name, watts, color)) in components.iter().enumerate() {
        let y = inner.y + 7 + i as u16;
        if y >= inner.y + inner.height { break; }

        let norm = (*watts / max_component).clamp(0.0, 1.0);
        let filled = (bar_width as f32 * norm) as usize;
        let empty = bar_width.saturating_sub(filled);

        let line = Line::from(vec![
            Span::styled(format!("{:<7}", name), Style::default().fg(*color)),
            Span::styled("▓".repeat(filled), Style::default().fg(*color)),
            Span::styled("░".repeat(empty), Style::default().fg(theme.border)),
            Span::styled(format!(" {:.2}W", watts), Style::default().fg(theme.fg)),
        ]);
        f.render_widget(Paragraph::new(line), Rect::new(inner.x, y, inner.width, 1));
    }

    // Fan speeds
    let mut fan_y = inner.y.saturating_add(14);
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
        .filter(|p| p.power_w > 0.0)
        .collect();
    procs_by_power.sort_by(|a, b| b.power_w.partial_cmp(&a.power_w).unwrap_or(std::cmp::Ordering::Equal));

    let proc_start_y = fan_y.saturating_add(1);
    if proc_start_y < inner.y.saturating_add(inner.height) {
        f.render_widget(
            Paragraph::new("top processes by power").style(Style::default().fg(theme.muted)),
            Rect::new(inner.x, proc_start_y, inner.width, 1),
        );
    }

    for (i, proc) in procs_by_power.iter().take(10).enumerate() {
        let y = proc_start_y.saturating_add(1).saturating_add(i as u16);
        if y >= inner.y.saturating_add(inner.height) { break; }
        let line = Line::from(vec![
            Span::styled(format!("{:<20}", truncate_with_ellipsis(&proc.name, 20)), Style::default().fg(theme.fg)),
            Span::styled(format!(" {:.2}W", proc.power_w), Style::default().fg(theme.power_accent)),
        ]);
        f.render_widget(Paragraph::new(line), Rect::new(inner.x, y, inner.width, 1));
    }
}

fn draw_process_expanded(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    let sort_label = state.sort_mode.label();
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled("⁶ Processes  ", Style::default().fg(theme.fg).bold()),
            Span::styled(format!(" sort: {} ", sort_label), Style::default().fg(theme.muted)),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.process_accent));

    let raw_inner = block.inner(area);
    f.render_widget(block, area);
    let inner = Rect::new(raw_inner.x + 1, raw_inner.y + 1, raw_inner.width.saturating_sub(2), raw_inner.height.saturating_sub(2));

    if inner.width == 0 || inner.height == 0 { return; }

    let gb = 1024.0 * 1024.0 * 1024.0;
    let mb = 1024.0 * 1024.0;

    // Header row with thread count and I/O
    let header = Line::from(vec![
        Span::styled(format!("{:<18}", "name"), Style::default().fg(theme.muted).bold()),
        Span::styled(format!("{:>6}", "cpu%"), Style::default().fg(theme.cpu_accent).bold()),
        Span::styled(format!("{:>8}", "mem"), Style::default().fg(theme.mem_accent).bold()),
        Span::styled(format!("{:>7}", "power"), Style::default().fg(theme.power_accent).bold()),
        Span::styled(format!("{:>7}", "thread"), Style::default().fg(theme.muted).bold()),
        Span::styled(format!("{:>7}", "IO R"), Style::default().fg(theme.muted).bold()),
        Span::styled(format!("{:>7}", "IO W"), Style::default().fg(theme.muted).bold()),
        Span::styled(format!("{:>7}", "PID"), Style::default().fg(theme.muted).bold()),
        Span::styled(format!(" {:<8}", "user"), Style::default().fg(theme.muted).bold()),
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

    let scroll = state.process_scroll.min(indices.len().saturating_sub(1));
    let max_visible = inner.height.saturating_sub(1) as usize;

    for (i, &idx) in indices.iter().skip(scroll).take(max_visible).enumerate() {
        let y = inner.y + 1 + i as u16;
        if y >= inner.y + inner.height { break; }

        let proc = &procs[idx];
        let mem_display = if proc.mem_bytes as f64 >= gb {
            format!("{:.1}G", proc.mem_bytes as f64 / gb)
        } else {
            format!("{:.0}M", proc.mem_bytes as f64 / mb)
        };

        let cpu_norm = if max_cpu > 0.0 { (proc.cpu_pct / max_cpu).clamp(0.0, 1.0) as f64 } else { 0.0 };
        let mem_norm = if max_mem > 0 { (proc.mem_bytes as f64 / max_mem as f64).clamp(0.0, 1.0) } else { 0.0 };
        let pwr_norm = if max_power > 0.0 { (proc.power_w / max_power).clamp(0.0, 1.0) as f64 } else { 0.0 };
        let cpu_color = gradient::value_to_color(cpu_norm, theme);
        let mem_color = gradient::value_to_color(mem_norm, theme);
        let pwr_color = gradient::value_to_color(pwr_norm, theme);

        let line = Line::from(vec![
            Span::styled(format!("{:<18}", truncate_with_ellipsis(&proc.name, 18)), Style::default().fg(theme.fg)),
            Span::styled("•", Style::default().fg(cpu_color)),
            Span::styled(format!("{:>4.1}%", proc.cpu_pct), Style::default().fg(cpu_color)),
            Span::styled("•", Style::default().fg(mem_color)),
            Span::styled(format!("{:>7}", mem_display), Style::default().fg(mem_color)),
            Span::styled("•", Style::default().fg(pwr_color)),
            Span::styled(format!("{:>5.1}W", proc.power_w), Style::default().fg(pwr_color)),
            Span::styled(format!("{:>7}", proc.thread_count), Style::default().fg(theme.muted)),
            Span::styled(format!("{:>7}", format_bytes_rate_compact(proc.io_read_bytes_sec)), Style::default().fg(theme.muted)),
            Span::styled(format!("{:>7}", format_bytes_rate_compact(proc.io_write_bytes_sec)), Style::default().fg(theme.muted)),
            Span::styled(format!("{:>7}", proc.pid), Style::default().fg(theme.muted)),
            Span::styled(format!(" {:<8}", truncate_with_ellipsis(&proc.user, 8)), Style::default().fg(theme.muted)),
        ]);
        f.render_widget(Paragraph::new(line), Rect::new(inner.x, y, inner.width, 1));
    }
}
