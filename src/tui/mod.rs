pub mod braille;
mod dashboard;
mod expanded;
pub mod gauge;
pub mod gradient;
pub mod helpers;
mod input;
pub mod layout;
mod panels;
#[allow(dead_code)]
pub(crate) mod prepare;
pub mod theme;

use std::io::stdout;
use std::time::Duration;

use crossterm::{
    event::{self, Event},
    terminal,
    ExecutableCommand,
};
use ratatui::prelude::*;

use crate::metrics::{MetricsHistory, MetricsSnapshot, Sampler, SortMode};

// Re-export for tests
pub use helpers::format_bytes_rate_compact;

/// Public test helper: render the dashboard onto a TestBackend and return
/// the flattened buffer text. Used by integration tests in tests/.
pub fn render_dashboard_to_string(width: u16, height: u16, snapshot: MetricsSnapshot, show_detail: bool) -> String {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let state = AppState {
        snapshot,
        show_detail,
        ..AppState::default()
    };
    terminal.draw(|f| draw_dashboard(f, &state)).unwrap();
    let buf = terminal.backend().buffer().clone();
    let mut text = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            text.push_str(buf[(x, y)].symbol());
        }
        text.push('\n');
    }
    text
}

/// Extended test helper: render the dashboard with configurable AppState fields.
pub fn render_dashboard_with_state(
    width: u16,
    height: u16,
    snapshot: MetricsSnapshot,
    show_detail: bool,
    expanded_panel: Option<PanelId>,
    sort_mode: SortMode,
) -> String {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let state = AppState {
        snapshot,
        show_detail,
        expanded_panel,
        sort_mode,
        ..AppState::default()
    };
    terminal.draw(|f| draw_dashboard(f, &state)).unwrap();
    let buf = terminal.backend().buffer().clone();
    let mut text = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            text.push_str(buf[(x, y)].symbol());
        }
        text.push('\n');
    }
    text
}

/// Test helper: render the CPU panel in compact mode (show_detail=false) to a string.
pub fn render_cpu_panel_compact_to_string(width: u16, height: u16, snapshot: MetricsSnapshot, theme_idx: usize) -> String {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let state = AppState {
        snapshot,
        show_detail: false,
        theme_idx,
        ..AppState::default()
    };
    let theme = &theme::THEMES[theme_idx.min(theme::THEMES.len() - 1)];
    terminal.draw(|f| panels::draw_cpu_panel_v2(f, f.area(), &state.snapshot, &state, theme)).unwrap();
    let buf = terminal.backend().buffer().clone();
    let mut text = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            text.push_str(buf[(x, y)].symbol());
        }
        text.push('\n');
    }
    text
}

/// Test helper: render the CPU panel in expanded mode (show_detail=true) to a string.
pub fn render_cpu_panel_expanded_to_string(width: u16, height: u16, snapshot: MetricsSnapshot, theme_idx: usize) -> String {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let state = AppState {
        snapshot,
        show_detail: true,
        theme_idx,
        ..AppState::default()
    };
    let theme = &theme::THEMES[theme_idx.min(theme::THEMES.len() - 1)];
    terminal.draw(|f| panels::draw_cpu_panel_v2(f, f.area(), &state.snapshot, &state, theme)).unwrap();
    let buf = terminal.backend().buffer().clone();
    let mut text = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            text.push_str(buf[(x, y)].symbol());
        }
        text.push('\n');
    }
    text
}

/// Run the interactive story browser in the current terminal.
/// Stories: cpu panel (compact/expanded) × (dark/light themes).
/// Navigate: n/→ next, p/← prev, q quit.
pub fn run_stories() -> Result<(), Box<dyn std::error::Error>> {
    use crossterm::{event::{self, Event, KeyCode}, terminal, ExecutableCommand};
    use ratatui::backend::CrosstermBackend;
    use ratatui::Terminal;
    use std::io::stdout;
    use crate::metrics::MetricsSnapshot;

    // Build fixture snapshot (same as insta tests) — realistic M3 Pro-like data
    let mut snapshot = MetricsSnapshot::default();
    snapshot.soc.chip = "Apple M3 Pro".to_string();
    snapshot.soc.e_cores = 4;
    snapshot.soc.p_cores = 6;
    snapshot.soc.gpu_cores = 18;
    snapshot.soc.memory_gb = 18;
    snapshot.cpu.total_usage = 0.42;
    snapshot.cpu.e_cluster.freq_mhz = 1200;
    snapshot.cpu.e_cluster.usage = 0.24;
    snapshot.cpu.p_cluster.freq_mhz = 3400;
    snapshot.cpu.p_cluster.usage = 0.67;
    // 4 e-cores + 6 p-cores
    snapshot.cpu.core_usages = vec![0.12, 0.45, 0.08, 0.31, 0.78, 0.55, 0.92, 0.43, 0.61, 0.39];
    snapshot.power.cpu_w = 8.5;
    snapshot.power.gpu_w = 3.2;
    snapshot.power.ane_w = 0.8;
    snapshot.power.dram_w = 1.5;
    snapshot.power.package_w = 14.0;
    snapshot.power.system_w = 16.5;
    snapshot.power.available = true;
    snapshot.temperature.cpu_avg_c = 55.0;
    snapshot.temperature.gpu_avg_c = 48.0;
    snapshot.temperature.available = true;
    snapshot.temperature.fan_speeds = vec![1200];

    let dark_idx = 0usize;
    let light_idx = theme::THEMES.iter().position(|t| t.name == "solarized-light").unwrap_or(0);

    let stories: &[(&str, bool, usize)] = &[
        ("cpu compact — dark",    false, dark_idx),
        ("cpu compact — light",   false, light_idx),
        ("cpu expanded — dark",   true,  dark_idx),
        ("cpu expanded — light",  true,  light_idx),
    ];

    let total = stories.len();
    let mut current = 0usize;

    terminal::enable_raw_mode()?;
    stdout().execute(terminal::EnterAlternateScreen)?;

    // Panic hook: restore terminal on crash (mirrors run())
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = terminal::disable_raw_mode();
        let _ = std::io::stdout().execute(terminal::LeaveAlternateScreen);
        original_hook(info);
    }));

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    loop {
        let (name, show_detail, theme_idx) = stories[current];
        let th = &theme::THEMES[theme_idx];
        let state = AppState {
            snapshot: snapshot.clone(),
            show_detail,
            theme_idx,
            ..AppState::default()
        };

        terminal.draw(|f| {
            let full = f.area();
            // Header bar (1 row)
            let header_area = ratatui::layout::Rect::new(full.x, full.y, full.width, 1);
            let panel_area  = ratatui::layout::Rect::new(full.x, full.y + 1, full.width, full.height.saturating_sub(1));

            let header_text = format!(
                " story {}/{} — {}  [n/→] next  [p/←] prev  [q] quit",
                current + 1, total, name
            );
            f.render_widget(
                ratatui::widgets::Paragraph::new(header_text)
                    .style(ratatui::style::Style::default().fg(th.muted)),
                header_area,
            );

            panels::draw_cpu_panel_v2(f, panel_area, &state.snapshot, &state, th);
        })?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    KeyCode::Char('n') | KeyCode::Right => current = (current + 1) % total,
                    KeyCode::Char('p') | KeyCode::Left  => current = (current + total - 1) % total,
                    _ => {}
                }
            }
        }
    }

    terminal::disable_raw_mode()?;
    stdout().execute(terminal::LeaveAlternateScreen)?;
    Ok(())
}

use dashboard::draw_dashboard;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelId {
    Cpu,
    Gpu,
    MemDisk,
    Network,
    Power,
    Process,
}

impl PanelId {
    pub(crate) fn is_left_column(self) -> bool {
        matches!(self, PanelId::Cpu | PanelId::Gpu | PanelId::MemDisk)
    }
}

pub(crate) struct AppState {
    pub(crate) interval_ms: u32,
    pub(crate) process_scroll: usize,
    pub(crate) theme_idx: usize,
    pub expanded_panel: Option<PanelId>,
    pub(crate) sort_mode: SortMode,
    pub(crate) temp_unit: String,
    pub(crate) show_detail: bool,
    pub(crate) show_help: bool,
    pub(crate) history: MetricsHistory,
    pub(crate) snapshot: MetricsSnapshot,
    // I44-F5: process selection and signal confirmation
    pub(crate) process_selected: Option<usize>,
    pub(crate) pending_signal: Option<(i32, String, i32)>, // (pid, process_name, signal)
    // I45-F5: process name filter in expanded mode
    pub(crate) process_filter: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            interval_ms: 1000,
            process_scroll: 0,
            theme_idx: 0,
            expanded_panel: None,
            sort_mode: SortMode::default(),
            temp_unit: "celsius".to_string(),
            show_detail: false,
            show_help: false,
            history: MetricsHistory::new(),
            snapshot: MetricsSnapshot::default(),
            process_selected: None,
            pending_signal: None,
            process_filter: None,
        }
    }
}

/// Return the list of available theme names (for tests and CLI validation).
pub fn theme_names() -> Vec<&'static str> {
    theme::theme_names()
}

pub fn run(interval_ms: u32, color: &str, temp_unit: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut sampler = Sampler::new()?;
    let initial_theme = theme::THEMES
        .iter()
        .position(|t| t.name == color || (color == "default" && t.name == "horizon"))
        .unwrap_or(0);
    let mut state = AppState {
        interval_ms: interval_ms.max(100),
        process_scroll: 0,
        theme_idx: initial_theme,
        expanded_panel: None,
        sort_mode: SortMode::default(),
        temp_unit: temp_unit.to_string(),
        show_detail: false,
        show_help: false,
        history: MetricsHistory::new(),
        snapshot: MetricsSnapshot::default(),
        process_selected: None,
        pending_signal: None,
        process_filter: None,
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
        // Resize history buffers to terminal width
        state.history.resize_buffers(terminal.size()?.width);

        // Render
        terminal.draw(|f| draw_dashboard(f, &state))?;

        // Poll for input (non-blocking, with timeout = interval)
        let mut should_quit = false;
        if event::poll(Duration::from_millis(state.interval_ms as u64))? {
            if let Event::Key(key) = event::read()? {
                should_quit = input::handle_key_event(key, &mut state);
            }
            // I45-F4: drain queued events to coalesce rapid input (debounce)
            while !should_quit && event::poll(Duration::ZERO)? {
                if let Event::Key(key) = event::read()? {
                    should_quit = input::handle_key_event(key, &mut state);
                }
            }
        }
        if should_quit { break; }

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

#[cfg(test)]
mod tests;
