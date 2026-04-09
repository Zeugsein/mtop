pub mod braille;
mod expanded;
pub mod gauge;
pub mod gradient;
pub mod helpers;
mod input;
pub mod layout;
pub mod theme;

use std::io::stdout;
use std::time::Duration;

use crossterm::{
    event::{self, Event},
    terminal,
    ExecutableCommand,
};
use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::metrics::{MetricsHistory, MetricsSnapshot, Sampler};
use crate::platform::network::speed_tier_from_baudrate;

// Re-export for tests
pub use helpers::format_bytes_rate_compact;
use helpers::{format_bytes_rate, truncate_with_ellipsis, is_infrastructure_interface};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PanelId {
    Cpu,
    Gpu,
    MemDisk,
    Network,
    Power,
    Process,
}

impl PanelId {
    fn is_left_column(self) -> bool {
        matches!(self, PanelId::Cpu | PanelId::Gpu | PanelId::MemDisk)
    }
}

struct AppState {
    interval_ms: u32,
    process_scroll: usize,
    theme_idx: usize,
    selected_panel: PanelId,
    expanded_panel: Option<PanelId>,
    history: MetricsHistory,
    snapshot: MetricsSnapshot,
}

/// Return the list of available theme names (for tests and CLI validation).
pub fn theme_names() -> Vec<&'static str> {
    theme::theme_names()
}

pub fn run(interval_ms: u32, color: &str, _temp_unit: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut sampler = Sampler::new()?;
    let initial_theme = theme::THEMES
        .iter()
        .position(|t| t.name == color || (color == "default" && t.name == "horizon"))
        .unwrap_or(0);
    let mut state = AppState {
        interval_ms: interval_ms.max(100),
        process_scroll: 0,
        theme_idx: initial_theme,
        selected_panel: PanelId::Cpu,
        expanded_panel: None,
        history: MetricsHistory::new(),
        snapshot: MetricsSnapshot::default(),
    };

    // Initial sample
    state.snapshot = sampler.sample(100)?;
    state.history.push(&state.snapshot);

    terminal::enable_raw_mode()?;
    stdout().execute(terminal::EnterAlternateScreen)?;

    // Panic hook to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = terminal::disable_raw_mode();
        let _ = stdout().execute(terminal::LeaveAlternateScreen);
        original_hook(info);
    }));

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    loop {
        // Render
        terminal.draw(|f| draw_dashboard(f, &state))?;

        // Poll for input (non-blocking, with timeout = interval)
        if event::poll(Duration::from_millis(state.interval_ms as u64))?
            && let Event::Key(key) = event::read()?
                && input::handle_key_event(key, &mut state) {
                    break;
                }

        // Sample
        match sampler.sample(0) {
            // interval handled by poll timeout
            Ok(s) => {
                state.snapshot = s;
                state.history.push(&state.snapshot);
            }
            Err(e) => eprintln!("sample error: {e}"),
        }
    }

    // Cleanup
    terminal::disable_raw_mode()?;
    stdout().execute(terminal::LeaveAlternateScreen)?;

    Ok(())
}

fn draw_dashboard(f: &mut Frame, state: &AppState) {
    let theme = theme::THEMES[state.theme_idx];
    let s = &state.snapshot;
    let area = f.area();

    // Terminal too-small check
    if layout::terminal_too_small(area) {
        let msg = layout::too_small_message(area);
        let para = Paragraph::new(msg)
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().fg(theme.fg));
        // Center vertically
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

    // Header (full width)
    let header_text = format!(
        " mtop — {} — {}E+{}P — {}GPU — {}GB ",
        s.soc.chip, s.soc.e_cores, s.soc.p_cores,
        s.soc.gpu_cores, s.soc.memory_gb
    );
    let header = Paragraph::new(header_text)
        .style(Style::default().bg(theme.header_bg).fg(theme.header_fg).bold());
    f.render_widget(header, page.header);

    // Expand/collapse layout
    match state.expanded_panel {
        Some(panel) if panel.is_left_column() => {
            // Left column: expanded panel fills all 3 rows
            expanded::draw_expanded_panel(f, page.left_column, panel, s, state, theme);
            // Right column: normal 3-panel layout
            let (r1, r2, r3) = layout::split_column_3(page.right_column);
            draw_network_panel_v2(f, r1, s, state, theme);
            draw_power_panel_v2(f, r2, s, state, theme);
            draw_process_panel_v2(f, r3, s, state, theme);
        }
        Some(panel) => {
            // Left column: normal 3-panel layout
            let (l1, l2, l3) = layout::split_column_3(page.left_column);
            draw_cpu_panel_v2(f, l1, s, state, theme);
            draw_gpu_panel_v2(f, l2, s, state, theme);
            draw_mem_disk_panel_v2(f, l3, s, state, theme);
            // Right column: expanded panel fills all 3 rows
            expanded::draw_expanded_panel(f, page.right_column, panel, s, state, theme);
        }
        None => {
            // Normal 3+3 grid
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

    // Footer (full width)
    let theme_name = theme::THEMES[state.theme_idx].name;
    let footer = Paragraph::new(format!(
        " q:quit  c:theme({theme_name})  1-6:select  e:expand  +/-:interval({}ms)  j/k:scroll ",
        state.interval_ms
    ))
    .style(Style::default().fg(theme.muted));
    f.render_widget(footer, page.footer);
}

/// New CPU panel: Type A layout (75% braille sparkline + 25% process dots)
fn draw_cpu_panel_v2(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    // Frame top: CPU  45.2% @ 3376MHz  3.8W  73°C
    let cpu_pct = s.cpu.total_usage * 100.0;
    let temp_color = gradient::temp_to_color(s.temperature.cpu_avg_c);
    let temp_str = if s.temperature.available {
        format!("{}°C", s.temperature.cpu_avg_c as u32)
    } else {
        "N/A".to_string()
    };

    let title_spans = vec![
        Span::styled(" CPU  ", Style::default().fg(theme.cpu_accent).bold()),
        Span::styled(format!("{:.1}%", cpu_pct), Style::default().fg(theme.fg)),
        Span::styled(format!(" @ {}MHz", s.cpu.p_cluster.freq_mhz.max(s.cpu.e_cluster.freq_mhz)), Style::default().fg(theme.muted)),
        Span::styled(format!("  {:.1}W", s.power.cpu_w), Style::default().fg(theme.muted)),
        Span::styled(format!("  {}", temp_str), Style::default().fg(temp_color)),
        Span::raw(" "),
    ];

    let block = Block::default()
        .title(Line::from(title_spans))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .border_type(ratatui::widgets::BorderType::Rounded);

    // Frame bottom: E/P cluster info
    let bottom_spans = vec![
        Span::styled(
            format!(" E: {:.0}% @ {}MHz", s.cpu.e_cluster.usage * 100.0, s.cpu.e_cluster.freq_mhz),
            Style::default().fg(theme.muted),
        ),
    ];
    let block = block.title_bottom(Line::from(bottom_spans).alignment(ratatui::layout::Alignment::Left));

    let p_spans = vec![
        Span::styled(
            format!("P: {:.0}% @ {}MHz ", s.cpu.p_cluster.usage * 100.0, s.cpu.p_cluster.freq_mhz),
            Style::default().fg(theme.muted),
        ),
    ];
    let block = block.title_bottom(Line::from(p_spans).alignment(ratatui::layout::Alignment::Right));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split inner: 75% trend + 25% detail
    let (trend_area, detail_area) = layout::split_type_a(inner);

    // Left: braille sparkline
    let sparkline_data: Vec<f64> = state.history.cpu_usage.iter().copied().collect();
    let spark_width = trend_area.width as usize;
    let spark = braille::render_braille_sparkline(&sparkline_data, 1.0, spark_width);

    let spark_spans: Vec<Span> = spark
        .iter()
        .map(|&(ch, color)| Span::styled(ch.to_string(), Style::default().fg(color)))
        .collect();

    if !spark_spans.is_empty() {
        // Render sparkline at vertical center of trend area
        let y_offset = trend_area.height / 2;
        let spark_rect = Rect::new(trend_area.x, trend_area.y + y_offset, trend_area.width, 1);
        f.render_widget(Paragraph::new(Line::from(spark_spans)), spark_rect);
    }

    // Right: process list with colored dots (●cpu ●mem ●pow)
    let legend = Line::from(vec![
        Span::styled("●", Style::default().fg(theme.cpu_accent)),
        Span::styled("c ", Style::default().fg(theme.muted)),
        Span::styled("●", Style::default().fg(theme.mem_accent)),
        Span::styled("m ", Style::default().fg(theme.muted)),
        Span::styled("●", Style::default().fg(theme.power_accent)),
        Span::styled("p", Style::default().fg(theme.muted)),
    ]);
    f.render_widget(Paragraph::new(legend), Rect::new(detail_area.x, detail_area.y, detail_area.width, 1));

    let max_procs = (detail_area.height as usize).saturating_sub(1);
    let max_mem = s.processes.iter().map(|p| p.mem_bytes).max().unwrap_or(1).max(1);

    for (i, proc) in s.processes.iter().take(max_procs).enumerate() {
        let y = detail_area.y + 1 + i as u16;
        if y >= detail_area.y + detail_area.height {
            break;
        }

        let name_width = detail_area.width.saturating_sub(7) as usize;
        let name = truncate_with_ellipsis(&proc.name, name_width);

        let cpu_norm = (proc.cpu_pct / 100.0).clamp(0.0, 1.0) as f64;
        let mem_norm = (proc.mem_bytes as f64 / max_mem as f64).clamp(0.0, 1.0);
        let pow_norm = cpu_norm * 0.8; // approximate power from CPU usage

        let line = Line::from(vec![
            Span::styled(format!("{:<w$}", name, w = name_width), Style::default().fg(theme.fg)),
            Span::raw(" "),
            Span::styled("●", Style::default().fg(gradient::value_to_color(cpu_norm))),
            Span::styled("●", Style::default().fg(gradient::value_to_color(mem_norm))),
            Span::styled("●", Style::default().fg(gradient::value_to_color(pow_norm))),
        ]);
        f.render_widget(Paragraph::new(line), Rect::new(detail_area.x, y, detail_area.width, 1));
    }
}

/// New GPU panel: Type A layout (75% braille sparkline + 25% orphan metrics)
fn draw_gpu_panel_v2(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    // Frame top: GPU  20% @ 338MHz  0.4W  52°C
    let gpu_pct = s.gpu.usage * 100.0;
    let temp_color = gradient::temp_to_color(s.temperature.gpu_avg_c);
    let temp_str = if s.temperature.available {
        format!("{}°C", s.temperature.gpu_avg_c as u32)
    } else {
        "N/A".to_string()
    };

    let title_spans = vec![
        Span::styled(" GPU  ", Style::default().fg(theme.gpu_accent).bold()),
        Span::styled(format!("{:.1}%", gpu_pct), Style::default().fg(theme.fg)),
        Span::styled(format!(" @ {}MHz", s.gpu.freq_mhz), Style::default().fg(theme.muted)),
        Span::styled(format!("  {:.1}W", s.power.gpu_w), Style::default().fg(theme.muted)),
        Span::styled(format!("  {}", temp_str), Style::default().fg(temp_color)),
        Span::raw(" "),
    ];

    // Frame bottom: cores + ANE
    let bottom_left = vec![
        Span::styled(format!(" {} cores", s.soc.gpu_cores), Style::default().fg(theme.muted)),
    ];
    let bottom_right = vec![
        Span::styled(format!("ANE {:.1}W ", s.power.ane_w), Style::default().fg(theme.muted)),
    ];

    let block = Block::default()
        .title(Line::from(title_spans))
        .title_bottom(Line::from(bottom_left).alignment(ratatui::layout::Alignment::Left))
        .title_bottom(Line::from(bottom_right).alignment(ratatui::layout::Alignment::Right))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .border_type(ratatui::widgets::BorderType::Rounded);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let (trend_area, detail_area) = layout::split_type_a(inner);

    // Left: GPU usage braille sparkline
    if s.gpu.available {
        let sparkline_data: Vec<f64> = state.history.gpu_usage.iter().copied().collect();
        let spark_width = trend_area.width as usize;
        let spark = braille::render_braille_sparkline(&sparkline_data, 1.0, spark_width);
        let spark_spans: Vec<Span> = spark
            .iter()
            .map(|&(ch, _)| Span::styled(ch.to_string(), Style::default().fg(theme.gpu_accent)))
            .collect();
        if !spark_spans.is_empty() {
            let y_offset = trend_area.height / 2;
            f.render_widget(
                Paragraph::new(Line::from(spark_spans)),
                Rect::new(trend_area.x, trend_area.y + y_offset, trend_area.width, 1),
            );
        }
    }

    // Right: orphan metrics
    let gb = 1024.0 * 1024.0 * 1024.0;
    let metrics = [
        format!("{} GPU cores", s.soc.gpu_cores),
        String::new(),
        format!("ANE  {:.1}W", s.power.ane_w),
        format!("DRAM {:.1}W", s.power.dram_w),
        String::new(),
        format!("Mem  {:.1}/{:.0}GB", s.memory.ram_used as f64 / gb, s.memory.ram_total as f64 / gb),
        format!("Swap {:.1}/{:.1}GB", s.memory.swap_used as f64 / gb, s.memory.swap_total as f64 / gb),
    ];

    for (i, text) in metrics.iter().enumerate() {
        let y = detail_area.y + i as u16;
        if y >= detail_area.y + detail_area.height || text.is_empty() {
            continue;
        }
        f.render_widget(
            Paragraph::new(text.as_str()).style(Style::default().fg(theme.fg)),
            Rect::new(detail_area.x, y, detail_area.width, 1),
        );
    }
}

/// Memory+Disk panel: Type A layout (75% sparkline+gauges + 25% disk detail)
fn draw_mem_disk_panel_v2(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    let gb = 1024.0 * 1024.0 * 1024.0;
    let ram_used_gb = s.memory.ram_used as f64 / gb;
    let ram_total_gb = s.memory.ram_total as f64 / gb;
    let ram_pct = if s.memory.ram_total > 0 {
        (s.memory.ram_used as f64 / s.memory.ram_total as f64 * 100.0) as u32
    } else {
        0
    };

    let disk_used_gb = s.disk.used_bytes as f64 / gb;
    let disk_total_gb = s.disk.total_bytes as f64 / gb;
    let disk_pct = if s.disk.total_bytes > 0 {
        (s.disk.used_bytes as f64 / s.disk.total_bytes as f64 * 100.0) as u32
    } else {
        0
    };

    // Frame top: Memory {used}/{total} GB {pct}%  |  Disk {used}/{total} GB {pct}%
    let title_spans = vec![
        Span::styled(" Memory  ", Style::default().fg(theme.mem_accent).bold()),
        Span::styled(format!("{ram_used_gb:.1}/{ram_total_gb:.0} GB  {ram_pct}%"), Style::default().fg(theme.fg)),
        Span::styled("  Disk  ", Style::default().fg(theme.muted)),
        Span::styled(format!("{disk_used_gb:.0}/{disk_total_gb:.0} GB  {disk_pct}%"), Style::default().fg(theme.fg)),
        Span::raw(" "),
    ];

    // Frame bottom: Swap {used}/{total} GB  |  R: {r} MB/s  W: {w} MB/s
    let swap_used_gb = s.memory.swap_used as f64 / gb;
    let swap_total_gb = s.memory.swap_total as f64 / gb;
    let bottom_left = vec![
        Span::styled(format!(" Swap {swap_used_gb:.1}/{swap_total_gb:.1} GB"), Style::default().fg(theme.muted)),
    ];
    let bottom_right = vec![
        Span::styled(
            format!("R: {}  W: {} ", format_bytes_rate(s.disk.read_bytes_sec as f64), format_bytes_rate(s.disk.write_bytes_sec as f64)),
            Style::default().fg(theme.muted),
        ),
    ];

    let block = Block::default()
        .title(Line::from(title_spans))
        .title_bottom(Line::from(bottom_left).alignment(ratatui::layout::Alignment::Left))
        .title_bottom(Line::from(bottom_right).alignment(ratatui::layout::Alignment::Right))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .border_type(ratatui::widgets::BorderType::Rounded);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let (trend_area, detail_area) = layout::split_type_a(inner);

    // Left 75%: RAM sparkline + RAM gauge + Swap gauge
    let left_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // sparkline (fills remaining space)
            Constraint::Length(1), // RAM gauge
            Constraint::Length(1), // Swap gauge
        ])
        .split(trend_area);

    // RAM sparkline
    let sparkline_data: Vec<f64> = state.history.mem_usage.iter().copied().collect();
    let spark_width = left_rows[0].width as usize;
    let spark = braille::render_braille_sparkline(&sparkline_data, 1.0, spark_width);
    let spark_spans: Vec<Span> = spark
        .iter()
        .map(|&(ch, color)| Span::styled(ch.to_string(), Style::default().fg(color)))
        .collect();
    if !spark_spans.is_empty() {
        let y_offset = left_rows[0].height / 2;
        let spark_rect = Rect::new(left_rows[0].x, left_rows[0].y + y_offset, left_rows[0].width, 1);
        f.render_widget(Paragraph::new(Line::from(spark_spans)), spark_rect);
    }

    // RAM gauge bar
    let ram_label = format!("{ram_used_gb:.1}/{ram_total_gb:.0} GB");
    let ram_gauge_spans = gauge::render_gauge_bar(
        s.memory.ram_used as f64, s.memory.ram_total as f64,
        left_rows[1].width.saturating_sub(16) as usize,
        &ram_label,
    );
    f.render_widget(Paragraph::new(Line::from(ram_gauge_spans)), left_rows[1]);

    // Swap gauge bar
    let swap_label = format!("{swap_used_gb:.1}/{swap_total_gb:.1} GB");
    let swap_gauge_spans = gauge::render_gauge_bar(
        s.memory.swap_used as f64, s.memory.swap_total as f64,
        left_rows[2].width.saturating_sub(16) as usize,
        &swap_label,
    );
    f.render_widget(Paragraph::new(Line::from(swap_gauge_spans)), left_rows[2]);

    // Right 25%: Disk capacity gauge + IO rates
    let right_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Disk gauge
            Constraint::Length(1), // spacer
            Constraint::Length(1), // IO read
            Constraint::Length(1), // IO write
        ])
        .split(detail_area);

    // Disk capacity gauge
    let disk_gauge_spans = gauge::render_compact_gauge(
        if s.disk.total_bytes > 0 { s.disk.used_bytes as f64 / s.disk.total_bytes as f64 } else { 0.0 },
        right_rows[0].width as usize,
    );
    f.render_widget(Paragraph::new(Line::from(disk_gauge_spans)), right_rows[0]);

    // IO read rate
    if right_rows.len() > 2 {
        let read_text = format!("R: {}", format_bytes_rate(s.disk.read_bytes_sec as f64));
        f.render_widget(
            Paragraph::new(read_text).style(Style::default().fg(theme.fg)),
            right_rows[2],
        );
    }

    // IO write rate
    if right_rows.len() > 3 {
        let write_text = format!("W: {}", format_bytes_rate(s.disk.write_bytes_sec as f64));
        f.render_widget(
            Paragraph::new(write_text).style(Style::default().fg(theme.fg)),
            right_rows[3],
        );
    }
}

/// Power panel: Type B layout (37.5% CPU sparkline + 37.5% GPU sparkline + 25% per-process energy)
fn draw_power_panel_v2(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    if !s.power.available {
        let block = Block::default()
            .title(" Power ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .border_type(ratatui::widgets::BorderType::Rounded);
        let inner = block.inner(area);
        f.render_widget(block, area);
        f.render_widget(
            Paragraph::new("Power sensors: N/A").style(Style::default().fg(theme.muted)),
            inner,
        );
        return;
    }

    // Frame top: Power  CPU {w}W  GPU {w}W
    let title_spans = vec![
        Span::styled(" Power  ", Style::default().fg(theme.power_accent).bold()),
        Span::styled(format!("CPU {:.1}W", s.power.cpu_w), Style::default().fg(theme.cpu_accent)),
        Span::styled("  ", Style::default()),
        Span::styled(format!("GPU {:.1}W", s.power.gpu_w), Style::default().fg(theme.gpu_accent)),
        Span::raw(" "),
    ];

    // Frame bottom: Total {w}W  Avg {w}W  Max {w}W
    let total_w = s.power.package_w.max(s.power.cpu_w + s.power.gpu_w + s.power.ane_w + s.power.dram_w);
    let avg_w = if !state.history.package_power.is_empty() {
        let sum: f64 = state.history.package_power.iter().sum();
        sum / state.history.package_power.len() as f64
    } else {
        total_w as f64
    };
    let max_w = state.history.package_power.iter().copied().fold(0.0_f64, f64::max);

    let bottom_spans = vec![
        Span::styled(
            format!(" Total {total_w:.1}W  Avg {avg_w:.1}W  Max {max_w:.1}W "),
            Style::default().fg(theme.muted),
        ),
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

    // Left 37.5%: CPU power sparkline
    let cpu_tdp = s.soc.cpu_tdp_w() as f64;
    let cpu_power_data: Vec<f64> = state.history.cpu_power.iter().copied().collect();
    let cpu_spark = braille::render_braille_sparkline(&cpu_power_data, cpu_tdp, left.width as usize);
    let cpu_spark_spans: Vec<Span> = cpu_spark
        .iter()
        .map(|&(ch, color)| Span::styled(ch.to_string(), Style::default().fg(color)))
        .collect();
    if !cpu_spark_spans.is_empty() {
        let y_offset = left.height / 2;
        f.render_widget(
            Paragraph::new(Line::from(cpu_spark_spans)),
            Rect::new(left.x, left.y + y_offset, left.width, 1),
        );
    }
    // Label
    f.render_widget(
        Paragraph::new("CPU").style(Style::default().fg(theme.cpu_accent)),
        Rect::new(left.x, left.y, left.width, 1),
    );

    // Middle 37.5%: GPU power sparkline
    let gpu_tdp = s.soc.gpu_tdp_w() as f64;
    let gpu_power_data: Vec<f64> = state.history.gpu_power.iter().copied().collect();
    let gpu_spark = braille::render_braille_sparkline(&gpu_power_data, gpu_tdp, mid.width as usize);
    let gpu_spark_spans: Vec<Span> = gpu_spark
        .iter()
        .map(|&(ch, color)| Span::styled(ch.to_string(), Style::default().fg(color)))
        .collect();
    if !gpu_spark_spans.is_empty() {
        let y_offset = mid.height / 2;
        f.render_widget(
            Paragraph::new(Line::from(gpu_spark_spans)),
            Rect::new(mid.x, mid.y + y_offset, mid.width, 1),
        );
    }
    // Label
    f.render_widget(
        Paragraph::new("GPU").style(Style::default().fg(theme.gpu_accent)),
        Rect::new(mid.x, mid.y, mid.width, 1),
    );

    // Right 25%: Per-process energy ranking
    let mut procs_by_power: Vec<&crate::metrics::ProcessInfo> = s.processes.iter()
        .filter(|p| p.power_w > 0.0)
        .collect();
    procs_by_power.sort_by(|a, b| b.power_w.partial_cmp(&a.power_w).unwrap_or(std::cmp::Ordering::Equal));

    let max_power = procs_by_power.first().map(|p| p.power_w).unwrap_or(1.0).max(0.01);
    let max_rows = right.height.saturating_sub(1) as usize; // leave 1 row for footer note

    for (i, proc) in procs_by_power.iter().take(max_rows).enumerate() {
        let y = right.y + i as u16;
        if y >= right.y + right.height.saturating_sub(1) {
            break;
        }

        let name_width = right.width.saturating_sub(8) as usize;
        let name = truncate_with_ellipsis(&proc.name, name_width);
        let power_norm = (proc.power_w / max_power).clamp(0.0, 1.0) as f64;

        let line = Line::from(vec![
            Span::styled(format!("{:<w$}", name, w = name_width), Style::default().fg(theme.fg)),
            Span::raw(" "),
            Span::styled("●", Style::default().fg(gradient::value_to_color(power_norm))),
            Span::styled(format!("{:.1}W", proc.power_w), Style::default().fg(theme.muted)),
        ]);
        f.render_widget(Paragraph::new(line), Rect::new(right.x, y, right.width, 1));
    }

    // Footer note
    if right.height > 1 {
        let note_y = right.y + right.height - 1;
        f.render_widget(
            Paragraph::new("(user procs)").style(Style::default().fg(theme.muted)),
            Rect::new(right.x, note_y, right.width, 1),
        );
    }
}


fn draw_network_panel_v2(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    // Aggregate upload/download rates
    let (total_rx, total_tx) = s.network.interfaces.iter().fold((0.0, 0.0), |(rx, tx), i| {
        (rx + i.rx_bytes_sec, tx + i.tx_bytes_sec)
    });

    // Frame top: " Network  ↑ {upload}  ↓ {download} "
    let title_spans = vec![
        Span::styled(" Network  ", Style::default().fg(theme.net_upload).bold()),
        Span::styled(format!("↑ {}", format_bytes_rate(total_tx)), Style::default().fg(theme.net_upload)),
        Span::styled("  ", Style::default()),
        Span::styled(format!("↓ {}", format_bytes_rate(total_rx)), Style::default().fg(theme.net_download)),
        Span::raw(" "),
    ];

    // Frame bottom: primary interface name or "No active interfaces"
    let mut sorted_ifaces: Vec<&crate::metrics::NetInterface> = s.network.interfaces.iter().collect();
    sorted_ifaces.sort_by(|a, b| {
        let a_total = a.rx_bytes_sec + a.tx_bytes_sec;
        let b_total = b.rx_bytes_sec + b.tx_bytes_sec;
        b_total.partial_cmp(&a_total).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Filter infrastructure interfaces for display ranking
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

fn draw_process_panel_v2(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, theme: &theme::Theme) {
    use crate::platform::process::weighted_score;

    let block = Block::default()
        .title(" Processes (weighted) ")
        .title_style(Style::default().fg(theme.fg).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    // Legend row
    let legend = Line::from(vec![
        Span::styled("●", Style::default().fg(theme.cpu_accent)),
        Span::styled("c ", Style::default().fg(theme.muted)),
        Span::styled("●", Style::default().fg(theme.mem_accent)),
        Span::styled("m ", Style::default().fg(theme.muted)),
        Span::styled("●", Style::default().fg(theme.power_accent)),
        Span::styled("p", Style::default().fg(theme.muted)),
    ]);
    f.render_widget(Paragraph::new(legend), Rect::new(inner.x, inner.y, inner.width, 1));

    // Sort processes by weighted_score descending (index-based to avoid clone)
    let procs = &s.processes;
    let max_cpu = procs.iter().map(|p| p.cpu_pct).fold(0.0f32, f32::max);
    let max_mem = procs.iter().map(|p| p.mem_bytes).max().unwrap_or(1).max(1);
    let max_power = procs.iter().map(|p| p.power_w).fold(0.0f32, f32::max);

    let mut indices: Vec<usize> = (0..procs.len()).collect();
    indices.sort_by(|&a, &b| {
        let sa = weighted_score(&procs[a], max_cpu, max_mem, max_power);
        let sb = weighted_score(&procs[b], max_cpu, max_mem, max_power);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Empty state
    if indices.is_empty() {
        let y = inner.y + 1;
        if y < inner.y + inner.height {
            let line = Line::from(Span::styled("No processes", Style::default().fg(theme.muted)));
            f.render_widget(Paragraph::new(line), Rect::new(inner.x, y, inner.width, 1));
        }
        return;
    }

    // Scroll support
    let scroll = state.process_scroll.min(indices.len().saturating_sub(1));
    let max_visible = inner.height.saturating_sub(1) as usize;

    let name_width = inner.width.saturating_sub(7) as usize;

    for (i, &idx) in indices.iter().skip(scroll).take(max_visible).enumerate() {
        let proc = &procs[idx];
        let y = inner.y + 1 + i as u16;
        if y >= inner.y + inner.height {
            break;
        }

        let name = truncate_with_ellipsis(&proc.name, name_width);

        let cpu_norm = if max_cpu > 0.0 {
            (proc.cpu_pct / max_cpu).clamp(0.0, 1.0) as f64
        } else {
            0.0
        };
        let mem_norm = (proc.mem_bytes as f64 / max_mem as f64).clamp(0.0, 1.0);
        let power_norm = if max_power > 0.0 {
            (proc.power_w / max_power).clamp(0.0, 1.0) as f64
        } else {
            0.0
        };

        let line = Line::from(vec![
            Span::styled(format!("{:<w$}", name, w = name_width), Style::default().fg(theme.fg)),
            Span::raw(" "),
            Span::styled("●", Style::default().fg(gradient::value_to_color(cpu_norm))),
            Span::styled("●", Style::default().fg(gradient::value_to_color(mem_norm))),
            Span::styled("●", Style::default().fg(gradient::value_to_color(power_norm))),
        ]);
        f.render_widget(Paragraph::new(line), Rect::new(inner.x, y, inner.width, 1));
    }
}









