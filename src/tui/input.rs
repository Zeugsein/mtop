//! Keybinding handler extracted from mod.rs (iteration 8).

use super::{AppState, PanelId, theme};
use crate::config;
use crate::tui::helpers::sort_indices;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn toggle_expand(state: &mut AppState, panel: PanelId) {
    if state.expanded_panel == Some(panel) {
        state.expanded_panel = None;
        // I44-F5f: reset selection on panel close
        state.process_selected = None;
        state.pending_signal = None;
        // I45-F5a: reset filter on close
        state.process_filter = None;
    } else {
        state.expanded_panel = Some(panel);
        state.process_selected = None;
        state.pending_signal = None;
        state.process_filter = None;
    }
}

/// Process a key event and mutate AppState accordingly.
/// Returns `true` if the application should quit.
pub(crate) fn handle_key_event(key: KeyEvent, state: &mut AppState) -> bool {
    // I44-F5d: confirmation dialog intercepts all keys when active
    if let Some((pid, _, signal)) = state.pending_signal.take() {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                // Send the signal
                unsafe {
                    libc::kill(pid, signal);
                }
            }
            _ => {} // Any other key cancels
        }
        return false;
    }

    // I44-F5a: process expanded mode intercepts ↑/↓/j/k/t/f
    if state.expanded_panel == Some(PanelId::Process) {
        // I45-F5b: filter input mode — intercepts all printable chars when active
        if let Some(ref mut filter) = state.process_filter {
            match key.code {
                KeyCode::Esc => {
                    // Clear filter and exit filter mode (don't close panel)
                    state.process_filter = None;
                    state.process_selected = Some(0);
                    return false;
                }
                KeyCode::Backspace => {
                    filter.pop();
                    if filter.is_empty() {
                        state.process_filter = None;
                    }
                    state.process_selected = Some(0);
                    return false;
                }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    filter.push(c);
                    state.process_selected = Some(0);
                    return false;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let sel = state.process_selected.unwrap_or(0);
                    state.process_selected = Some(sel.saturating_add(1));
                    return false;
                }
                KeyCode::Up => {
                    let sel = state.process_selected.unwrap_or(0);
                    state.process_selected = Some(sel.saturating_sub(1));
                    return false;
                }
                _ => {} // Fall through to global handlers
            }
        }

        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                let sel = state.process_selected.unwrap_or(0);
                state.process_selected = Some(sel.saturating_add(1));
                // Clamping happens in draw_process_expanded against actual list length
                return false;
            }
            KeyCode::Up => {
                let sel = state.process_selected.unwrap_or(0);
                state.process_selected = Some(sel.saturating_sub(1));
                return false;
            }
            KeyCode::Char('t') => {
                if let Some((pid, name)) = resolve_selected_process(state) {
                    state.pending_signal = Some((pid, name, libc::SIGTERM));
                }
                return false;
            }
            KeyCode::Char('k') => {
                if let Some((pid, name)) = resolve_selected_process(state) {
                    state.pending_signal = Some((pid, name, libc::SIGKILL));
                }
                return false;
            }
            // I45-F5a: 'f' enters filter mode
            KeyCode::Char('f') => {
                state.process_filter = Some(String::new());
                state.process_selected = Some(0);
                return false;
            }
            _ => {} // Fall through to global handlers
        }
    }

    match key.code {
        KeyCode::Char('q') => return true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
        KeyCode::Esc => {
            if state.show_help {
                state.show_help = false;
            } else if state.expanded_panel.is_some() {
                state.process_selected = None;
                state.pending_signal = None;
                state.process_filter = None;
                state.expanded_panel = None;
            } else {
                return true;
            }
        }
        KeyCode::Char('c') => {
            state.theme_idx = (state.theme_idx + 1) % theme::THEMES.len();
        }
        KeyCode::Char('C') => {
            let len = theme::THEMES.len();
            state.theme_idx = (state.theme_idx + len - 1) % len;
        }
        KeyCode::Char('1') => toggle_expand(state, PanelId::Cpu),
        KeyCode::Char('2') => toggle_expand(state, PanelId::Gpu),
        KeyCode::Char('3') => toggle_expand(state, PanelId::MemDisk),
        KeyCode::Char('4') => toggle_expand(state, PanelId::Network),
        KeyCode::Char('5') => toggle_expand(state, PanelId::Power),
        KeyCode::Char('6') => toggle_expand(state, PanelId::Process),
        KeyCode::Char('e') | KeyCode::Enter => {
            state.process_selected = None;
            state.pending_signal = None;
            state.expanded_panel = None;
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            const PRESETS: [u32; 10] = [100, 250, 500, 750, 1000, 1500, 2000, 3000, 5000, 10000];
            state.interval_ms = PRESETS
                .iter()
                .copied()
                .find(|&v| v > state.interval_ms)
                .unwrap_or(10000);
        }
        KeyCode::Char('-') => {
            const PRESETS: [u32; 10] = [100, 250, 500, 750, 1000, 1500, 2000, 3000, 5000, 10000];
            state.interval_ms = PRESETS
                .iter()
                .copied()
                .rev()
                .find(|&v| v < state.interval_ms)
                .unwrap_or(100);
        }
        // I45-F3: j/k removed from global scroll (reserved for process-expand nav)
        KeyCode::Down => {
            state.process_scroll = state.process_scroll.saturating_add(1);
        }
        KeyCode::Up => {
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

/// Resolve the currently selected process (by display-order index) to (pid, name).
/// I45-F5c: operates on filtered list when process_filter is active.
fn resolve_selected_process(state: &AppState) -> Option<(i32, String)> {
    let sel = state.process_selected?;
    let procs = &state.snapshot.processes;
    if procs.is_empty() {
        return None;
    }

    let max_cpu = procs.iter().map(|p| p.cpu_pct).fold(0.0f32, f32::max);
    let max_mem = procs.iter().map(|p| p.mem_bytes).max().unwrap_or(1).max(1);
    let max_power = procs.iter().map(|p| p.power_w).fold(0.0f32, f32::max);

    let mut indices: Vec<usize> = (0..procs.len()).collect();
    sort_indices(
        &mut indices,
        procs,
        state.sort_mode,
        max_cpu,
        max_mem,
        max_power,
    );

    // I45-F5c: apply filter to sorted indices
    if let Some(ref filter) = state.process_filter
        && !filter.is_empty()
    {
        let filter_lower = filter.to_lowercase();
        indices.retain(|&idx| procs[idx].name.to_lowercase().contains(&filter_lower));
    }

    let scroll = state.process_scroll.min(indices.len().saturating_sub(1));
    let display_idx = scroll + sel;
    indices
        .get(display_idx)
        .map(|&idx| (procs[idx].pid, procs[idx].name.clone()))
}
