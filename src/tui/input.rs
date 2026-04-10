//! Keybinding handler extracted from mod.rs (iteration 8).

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use super::{PanelId, AppState, theme};
use crate::config;

fn toggle_expand(state: &mut AppState, panel: PanelId) {
    if state.expanded_panel == Some(panel) {
        state.expanded_panel = None;
    } else {
        state.expanded_panel = Some(panel);
    }
}

/// Process a key event and mutate AppState accordingly.
/// Returns `true` if the application should quit.
pub(crate) fn handle_key_event(key: KeyEvent, state: &mut AppState) -> bool {
    match key.code {
        KeyCode::Char('q') => return true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
        KeyCode::Esc => {
            if state.show_help {
                state.show_help = false;
            } else if state.expanded_panel.is_some() {
                state.expanded_panel = None;
            } else {
                return true;
            }
        }
        KeyCode::Char('c') => {
            state.theme_idx = (state.theme_idx + 1) % theme::THEMES.len();
        }
        KeyCode::Char('1') => toggle_expand(state, PanelId::Cpu),
        KeyCode::Char('2') => toggle_expand(state, PanelId::Gpu),
        KeyCode::Char('3') => toggle_expand(state, PanelId::MemDisk),
        KeyCode::Char('4') => toggle_expand(state, PanelId::Network),
        KeyCode::Char('5') => toggle_expand(state, PanelId::Power),
        KeyCode::Char('6') => toggle_expand(state, PanelId::Process),
        KeyCode::Char('e') | KeyCode::Enter => {
            state.expanded_panel = None;
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
        KeyCode::Char('.') => {
            state.show_detail = !state.show_detail;
        }
        KeyCode::Char('h') | KeyCode::Char('?') => {
            state.show_help = !state.show_help;
        }
        KeyCode::Char('s') => {
            state.sort_mode = state.sort_mode.next();
        }
        KeyCode::Char('w') => {
            let theme_name = theme::THEMES[state.theme_idx].name;
            let sort_label = match state.sort_mode {
                crate::metrics::SortMode::WeightedScore => "score",
                crate::metrics::SortMode::Cpu => "cpu",
                crate::metrics::SortMode::Memory => "memory",
                crate::metrics::SortMode::Power => "power",
                crate::metrics::SortMode::Pid => "pid",
                crate::metrics::SortMode::Name => "name",
            };
            let cfg = config::Config {
                theme: theme_name.to_string(),
                interval_ms: state.interval_ms,
                temp_unit: state.temp_unit.clone(),
                sort_mode: sort_label.to_string(),
            };
            let _ = config::save(&cfg); // best-effort save
        }
        _ => {}
    }
    false
}
