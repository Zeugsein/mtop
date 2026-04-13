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

#[cfg(test)]
mod tests;
