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
        if event::poll(Duration::from_millis(state.interval_ms as u64))? {
            if let Event::Key(key) = event::read()? {
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
    let (_, accent, _) = THEMES[state.theme_idx];
    let s = &state.snapshot;

    let area = f.area();

    // Main layout: header (1) + body + process list
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // header
            Constraint::Min(12),   // body panels
            Constraint::Length(12), // process list
            Constraint::Length(1),  // footer
        ])
        .split(area);

    // Header
    let header_text = format!(
        " mtop — {} — {}C ({}E+{}P) / {}GPU — {}GB ",
        s.soc.chip, s.soc.e_cores + s.soc.p_cores, s.soc.e_cores, s.soc.p_cores,
        s.soc.gpu_cores, s.soc.memory_gb
    );
    let header = Paragraph::new(header_text)
        .style(Style::default().bg(accent).fg(Color::Black).bold());
    f.render_widget(header, main_chunks[0]);

    // Body: CPU left, info panels right
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[1]);

    // CPU panel (left)
    draw_cpu_panel(f, body_chunks[0], s, accent);

    // Right side: power, temp, memory, network stacked
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),    // power
            Constraint::Length(3), // temperature
            Constraint::Length(4), // memory
            Constraint::Length(3), // network
        ])
        .split(body_chunks[1]);

    draw_power_panel(f, right_chunks[0], s, state, accent);
    draw_temp_panel(f, right_chunks[1], s, state, accent);
    draw_memory_panel(f, right_chunks[2], s, accent);
    draw_network_panel(f, right_chunks[3], s, accent);

    // Process list
    draw_process_list(f, main_chunks[2], s, state, accent);

    // Footer
    let footer = Paragraph::new(format!(
        " q:quit  s:sort({})  c:theme  +/-:interval({}ms)  j/k:scroll ",
        SORT_COLS[state.sort_col], state.interval_ms
    ))
    .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, main_chunks[3]);
}

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
                let idx = (v.min(700) / 100) as usize;
                ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'][idx.min(7)]
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

fn draw_temp_panel(f: &mut Frame, area: Rect, s: &MetricsSnapshot, state: &AppState, accent: Color) {
    let block = Block::default()
        .title(" Temperature ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let (cpu_t, gpu_t, unit) = if state.temp_unit == "fahrenheit" {
        (s.temperature.cpu_avg_c * 9.0 / 5.0 + 32.0, s.temperature.gpu_avg_c * 9.0 / 5.0 + 32.0, "°F")
    } else {
        (s.temperature.cpu_avg_c, s.temperature.gpu_avg_c, "°C")
    };

    let text = format!("CPU avg: {:.0}{unit}    GPU avg: {:.0}{unit}", cpu_t, gpu_t);
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
