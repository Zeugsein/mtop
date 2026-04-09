pub mod braille;
pub mod gradient;
pub mod layout;
pub mod theme;

use std::io::stdout;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal,
    ExecutableCommand,
};
use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::metrics::{MetricsHistory, MetricsSnapshot, Sampler};

struct AppState {
    interval_ms: u32,
    sort_col: usize,
    process_scroll: usize,
    theme_idx: usize,
    temp_unit: String,
    history: MetricsHistory,
    snapshot: MetricsSnapshot,
}

const THEMES: &[(&str, Color, Color)] = &[
    ("default", Color::Cyan, Color::White),
    ("green", Color::Green, Color::White),
    ("blue", Color::Blue, Color::White),
];

/// Return the list of available theme names (for tests and CLI validation).
pub fn theme_names() -> Vec<&'static str> {
    THEMES.iter().map(|(name, _, _)| *name).collect()
}

const SORT_COLS: &[&str] = &["CPU%", "Mem", "PID", "Name"];

pub fn run(interval_ms: u32, color: &str, temp_unit: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut sampler = Sampler::new()?;
    let initial_theme = THEMES
        .iter()
        .position(|(name, _, _)| *name == color)
        .unwrap_or(0);
    let mut state = AppState {
        interval_ms: interval_ms.max(100),
        sort_col: 0,
        process_scroll: 0,
        theme_idx: initial_theme,
        temp_unit: temp_unit.to_string(),
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
            && let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Char('c') => {
                        state.theme_idx = (state.theme_idx + 1) % THEMES.len();
                    }
                    KeyCode::Char('s') => {
                        state.sort_col = (state.sort_col + 1) % SORT_COLS.len();
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        state.interval_ms = (state.interval_ms + 250).min(10000);
                    }
                    KeyCode::Char('-') => {
                        state.interval_ms = state.interval_ms.saturating_sub(250).max(100);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        state.process_scroll = state.process_scroll.saturating_add(1);
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        state.process_scroll = state.process_scroll.saturating_sub(1);
                    }
                    _ => {}
                }
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
    let theme = theme::default_theme();
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

    // Left column: CPU, GPU, Mem+Disk (placeholder)
    let (left_r1, left_r2, left_r3) = layout::split_column_3(page.left_column);
    draw_cpu_panel_v2(f, left_r1, s, state, theme);
    draw_gpu_panel_v2(f, left_r2, s, theme);
    draw_memory_panel(f, left_r3, s, theme.accent);

    // Right column: Network (placeholder), Power, Process list
    let (right_r1, right_r2, right_r3) = layout::split_column_3(page.right_column);
    draw_network_panel(f, right_r1, s, theme.accent);
    draw_power_panel(f, right_r2, s, state, theme.accent);
    draw_process_list(f, right_r3, s, state, theme.accent);

    // Footer (full width)
    let footer = Paragraph::new(format!(
        " q:quit  s:sort({})  c:theme  +/-:interval({}ms)  j/k:scroll ",
        SORT_COLS[state.sort_col], state.interval_ms
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
        let name: String = proc.name.chars().take(name_width).collect();

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
fn draw_gpu_panel_v2(f: &mut Frame, area: Rect, s: &MetricsSnapshot, theme: &theme::Theme) {
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
    let _sparkline_data: Vec<f64> = state_gpu_history_iter(&s.gpu).collect();
    // We don't have GPU history in this scope — use a simple approach
    // For now render a static indicator; full history requires passing MetricsHistory
    if s.gpu.available {
        let gpu_norm = s.gpu.usage as f64;
        let color = gradient::value_to_color(gpu_norm);
        let filled = (trend_area.width as f64 * gpu_norm) as u16;
        let bar: String = "▓".repeat(filled as usize);
        let empty: String = "░".repeat((trend_area.width - filled) as usize);
        let y_offset = trend_area.height / 2;
        let line = Line::from(vec![
            Span::styled(bar, Style::default().fg(color)),
            Span::styled(empty, Style::default().fg(theme.border)),
        ]);
        f.render_widget(
            Paragraph::new(line),
            Rect::new(trend_area.x, trend_area.y + y_offset, trend_area.width, 1),
        );
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

/// Placeholder: iterate GPU usage values (will use MetricsHistory in full implementation)
fn state_gpu_history_iter(_gpu: &crate::metrics::GpuMetrics) -> std::iter::Empty<f64> {
    std::iter::empty()
}

// Keep old CPU panel for backward compatibility with tests
#[allow(dead_code)]
fn draw_cpu_panel(f: &mut Frame, area: Rect, s: &MetricsSnapshot, accent: Color) {
    let block = Block::default()
        .title(" CPU ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let total_cores = s.cpu.core_usages.len();
    let e_count = s.soc.e_cores as usize;

    let mut lines = Vec::new();
    for (i, &usage) in s.cpu.core_usages.iter().enumerate() {
        let label = if i < e_count {
            format!("E{}", i)
        } else {
            format!("P{}", i - e_count)
        };

        let pct = (usage * 100.0).min(100.0);
        let bar_width = (inner.width as usize).saturating_sub(12);
        let filled = ((pct / 100.0) * bar_width as f32) as usize;
        let empty = bar_width.saturating_sub(filled);

        let bar_color = if pct > 60.0 {
            Color::Red
        } else if pct > 40.0 {
            Color::Yellow
        } else if pct > 30.0 {
            Color::Cyan
        } else {
            accent
        };

        lines.push(Line::from(vec![
            Span::styled(format!("{:>2} [", label), Style::default().fg(accent)),
            Span::styled("█".repeat(filled), Style::default().fg(bar_color)),
            Span::raw("░".repeat(empty)),
            Span::styled(format!("] {:>5.1}%", pct), Style::default().fg(Color::White)),
        ]));

        if lines.len() >= inner.height as usize {
            break;
        }
    }

    // Aggregate line
    if lines.len() < inner.height as usize && total_cores > 0 {
        lines.push(Line::from(vec![
            Span::styled(
                format!(
                    "Aggregate: {:.1}%  Power: {:.1}W",
                    s.cpu.total_usage * 100.0,
                    s.power.cpu_w
                ),
                Style::default().fg(Color::White).bold(),
            ),
        ]));
    }

    let text = Paragraph::new(lines);
    f.render_widget(text, inner);
}

fn draw_power_panel(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, accent: Color) {
    let block = Block::default()
        .title(" Power ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if !s.power.available {
        f.render_widget(
            Paragraph::new("Power sensors: N/A").style(Style::default().fg(Color::DarkGray)),
            inner,
        );
        return;
    }

    let items = [
        ("CPU ", &state.history.cpu_power, s.power.cpu_w),
        ("GPU ", &state.history.gpu_power, s.power.gpu_w),
        ("ANE ", &state.history.ane_power, s.power.ane_w),
        ("DRAM", &state.history.dram_power, s.power.dram_w),
        ("Pkg ", &state.history.package_power, s.power.package_w),
        ("Sys ", &state.history.system_power, s.power.system_w),
    ];

    let constraints: Vec<Constraint> = items.iter().map(|_| Constraint::Length(1)).collect();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (i, (label, history, value)) in items.iter().enumerate() {
        if i >= rows.len() {
            break;
        }
        let sparkline_width = rows[i].width.saturating_sub(16) as usize;
        let spark_data: Vec<u64> = history
            .iter()
            .rev()
            .take(sparkline_width)
            .rev()
            .map(|&v| (v * 100.0) as u64)
            .collect();

        let spark_str: String = spark_data
            .iter()
            .map(|&v| {
                if v == 0 {
                    ' '
                } else {
                    let idx = (v.min(700) / 100) as usize;
                    [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇'][idx.min(7)]
                }
            })
            .collect();

        let line = Line::from(vec![
            Span::styled(format!("{label} "), Style::default().fg(accent)),
            Span::styled(spark_str, Style::default().fg(power_color(*value))),
            Span::styled(format!(" {:>5.1}W", value), Style::default().fg(Color::White)),
        ]);
        f.render_widget(Paragraph::new(line), rows[i]);
    }
}

#[allow(dead_code)]
fn draw_temp_panel(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, accent: Color) {
    let block = Block::default()
        .title(" Temperature ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let text = if !s.temperature.available {
        "CPU avg: N/A    GPU avg: N/A".to_string()
    } else {
        let (cpu_t, gpu_t, unit) = if state.temp_unit == "fahrenheit" {
            (s.temperature.cpu_avg_c * 9.0 / 5.0 + 32.0, s.temperature.gpu_avg_c * 9.0 / 5.0 + 32.0, "°F")
        } else {
            (s.temperature.cpu_avg_c, s.temperature.gpu_avg_c, "°C")
        };
        format!("CPU avg: {:.0}{unit}    GPU avg: {:.0}{unit}", cpu_t, gpu_t)
    };
    f.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::White)),
        inner,
    );
}

fn draw_memory_panel(f: &mut Frame, area: Rect, s: &MetricsSnapshot, accent: Color) {
    let block = Block::default()
        .title(" Memory ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let gb = 1024.0 * 1024.0 * 1024.0;
    let ram_pct = if s.memory.ram_total > 0 {
        s.memory.ram_used as f64 / s.memory.ram_total as f64
    } else {
        0.0
    };

    let ram_label = format!(
        "RAM {:.1}/{:.0}GB",
        s.memory.ram_used as f64 / gb,
        s.memory.ram_total as f64 / gb,
    );

    let ram_gauge = Gauge::default()
        .gauge_style(Style::default().fg(accent))
        .ratio(ram_pct.min(1.0))
        .label(ram_label);

    let mem_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    f.render_widget(ram_gauge, mem_chunks[0]);

    if s.memory.swap_total > 0 {
        let swap_pct = s.memory.swap_used as f64 / s.memory.swap_total as f64;
        let swap_label = format!(
            "Swap {:.1}/{:.1}GB",
            s.memory.swap_used as f64 / gb,
            s.memory.swap_total as f64 / gb,
        );
        let swap_gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Yellow))
            .ratio(swap_pct.min(1.0))
            .label(swap_label);
        f.render_widget(swap_gauge, mem_chunks[1]);
    }
}

fn draw_network_panel(f: &mut Frame, area: Rect, s: &MetricsSnapshot, accent: Color) {
    let block = Block::default()
        .title(" Network ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let (total_rx, total_tx) = s.network.interfaces.iter().fold((0.0, 0.0), |(rx, tx), i| {
        (rx + i.rx_bytes_sec, tx + i.tx_bytes_sec)
    });

    let text = format!("↑ {}    ↓ {}", format_bytes_rate(total_tx), format_bytes_rate(total_rx));
    f.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::White)),
        inner,
    );
}

fn draw_process_list(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, accent: Color) {
    let block = Block::default()
        .title(format!(" Processes (sort: {}) ", SORT_COLS[state.sort_col]))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent));

    let header = Row::new(vec!["PID", "Name", "CPU%", "Mem", "User"])
        .style(Style::default().fg(accent).bold());

    let mut procs = s.processes.clone();
    match state.sort_col {
        0 => procs.sort_by(|a, b| b.cpu_pct.partial_cmp(&a.cpu_pct).unwrap_or(std::cmp::Ordering::Equal)),
        1 => procs.sort_by(|a, b| b.mem_bytes.cmp(&a.mem_bytes)),
        2 => procs.sort_by(|a, b| a.pid.cmp(&b.pid)),
        3 => procs.sort_by(|a, b| a.name.cmp(&b.name)),
        _ => {}
    }

    let rows: Vec<Row> = procs
        .iter()
        .skip(state.process_scroll)
        .map(|p| {
            Row::new(vec![
                format!("{}", p.pid),
                p.name.clone(),
                format!("{:.1}", p.cpu_pct),
                format_bytes(p.mem_bytes),
                p.user.clone(),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(7),
        Constraint::Min(20),
        Constraint::Length(7),
        Constraint::Length(9),
        Constraint::Length(12),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(table, area);
}

fn power_color(watts: f32) -> Color {
    if watts > 10.0 {
        Color::Red
    } else if watts > 5.0 {
        Color::Rgb(255, 165, 0) // orange
    } else if watts > 1.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}

fn format_bytes(b: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;
    if b >= GB {
        format!("{:.1}GB", b as f64 / GB as f64)
    } else if b >= MB {
        format!("{:.0}MB", b as f64 / MB as f64)
    } else if b >= KB {
        format!("{:.0}KB", b as f64 / KB as f64)
    } else {
        format!("{b}B")
    }
}

fn format_bytes_rate(b: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    if b >= GB {
        format!("{:.1} GB/s", b / GB)
    } else if b >= MB {
        format!("{:.1} MB/s", b / MB)
    } else if b >= KB {
        format!("{:.1} KB/s", b / KB)
    } else {
        format!("{:.0} B/s", b)
    }
}
