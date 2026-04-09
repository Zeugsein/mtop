//! Keybinding handler extracted from mod.rs (iteration 8).

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use super::{PanelId, AppState, theme};

/// Process a key event and mutate AppState accordingly.
/// Returns `true` if the application should quit.
pub(crate) fn handle_key_event(key: KeyEvent, state: &mut AppState) -> bool {
    match key.code {
        KeyCode::Char('q') => return true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
        KeyCode::Esc => {
            if state.expanded_panel.is_some() {
                state.expanded_panel = None;
            } else {
                return true;
            }
        }
        KeyCode::Char('c') => {
            state.theme_idx = (state.theme_idx + 1) % theme::THEMES.len();
        }
        KeyCode::Char('1') => state.selected_panel = PanelId::Cpu,
        KeyCode::Char('2') => state.selected_panel = PanelId::Gpu,
        KeyCode::Char('3') => state.selected_panel = PanelId::MemDisk,
        KeyCode::Char('4') => state.selected_panel = PanelId::Network,
        KeyCode::Char('5') => state.selected_panel = PanelId::Power,
        KeyCode::Char('6') => state.selected_panel = PanelId::Process,
        KeyCode::Char('e') | KeyCode::Enter => {
            if state.expanded_panel == Some(state.selected_panel) {
                state.expanded_panel = None;
            } else {
                state.expanded_panel = Some(state.selected_panel);
            }
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
        KeyCode::Char('s') => {
            state.sort_mode = state.sort_mode.next();
        }
        _ => {}
    }
    false
}
