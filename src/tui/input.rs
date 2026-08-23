//! Keybinding handler extracted from mod.rs (iteration 8).

use super::{AppState, PanelId, theme};
use crate::config;
use crate::tui::helpers::{effective_selection_row, sorted_filtered_indices};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn toggle_expand(state: &mut AppState, panel: PanelId) {
    if state.expanded_panel == Some(panel) {
        state.expanded_panel = None;
        // I44-F5f: reset selection on panel close
        state.process_selected = None;
        state.pending_signal = None;
        // I45-F5a: reset filter on close
        state.process_filter = None;
        state.process_filter_editing = false;
        // I58-F1e: clear stable pid selection alongside the cursor
        state.selected_pid = None;
    } else {
        state.expanded_panel = Some(panel);
        state.process_selected = None;
        state.pending_signal = None;
        state.process_filter = None;
        state.process_filter_editing = false;
        state.selected_pid = None;
    }
}

/// Move the process-panel cursor by `delta` (positive = down, negative = up),
/// then re-anchor `selected_pid` from the current sorted+filtered view.
/// I58-F1c. Preserves the pre-existing `unwrap_or(0).saturating_add/sub(1)`
/// semantics from a `None` cursor to keep SHALL-44-F5a tests green.
fn move_process_cursor(state: &mut AppState, delta: i32) {
    let cur = state.process_selected.unwrap_or(0);
    let requested_cursor = if delta >= 0 {
        cur.saturating_add(delta as usize)
    } else {
        cur.saturating_sub(delta.unsigned_abs() as usize)
    };

    let indices = sorted_filtered_indices(
        &state.snapshot.processes,
        state.sort_mode,
        state.process_filter.as_deref(),
    );
    if indices.is_empty() {
        // I59-F1c: navigation cannot establish a target in an empty view.
        // Clear the positional cursor but preserve an existing missing pid so
        // later repopulation cannot silently re-enable cursor fallback.
        state.process_selected = None;
        return;
    }

    // I59-F1c: every navigation on a non-empty view produces a valid,
    // pid-backed selection, including after list shrinkage.
    let new_cursor = requested_cursor.min(indices.len() - 1);
    state.process_selected = Some(new_cursor);
    state.selected_pid = Some(state.snapshot.processes[indices[new_cursor]].pid);
}

/// Process a key event and mutate AppState accordingly.
/// Returns `true` if the application should quit.
pub(crate) fn handle_key_event(key: KeyEvent, state: &mut AppState) -> bool {
    // I59-F1b: discard an invalidated modal before dispatch so its first
    // recovery key is handled as navigation rather than swallowed as cancel.
    invalidate_stale_pending_signal(state);

    // I44-F5d: confirmation dialog intercepts all keys when active
    if let Some((pid, name, signal)) = state.pending_signal.take() {
        // I59-F1b: the confirmation is valid only while the same process
        // remains the effective selection.
        match key.code {
            KeyCode::Char('y' | 'Y') if pending_signal_is_current(state, pid, &name) => unsafe {
                libc::kill(pid, signal);
            },
            _ => {} // Any other key cancels
        }
        return false;
    }

    // I44-F5a: process expanded mode intercepts ↑/↓/j/k/t/f
    if state.expanded_panel == Some(PanelId::Process) {
        // I61-F1b: explicit filter-editing mode. The active query remains
        // available after Enter so normal action keys can use the filtered view.
        if state.process_filter_editing {
            match key.code {
                KeyCode::Esc => {
                    // Clear filter and exit filter mode (don't close panel)
                    state.process_filter = None;
                    state.process_filter_editing = false;
                    state.process_selected = Some(0);
                    state.selected_pid = None;
                    return false;
                }
                KeyCode::Enter => {
                    state.process_filter_editing = false;
                    if state.process_filter.as_ref().is_some_and(String::is_empty) {
                        state.process_filter = None;
                    }
                    state.process_selected = Some(0);
                    state.selected_pid = None;
                    return false;
                }
                KeyCode::Backspace => {
                    state.process_filter.get_or_insert_default().pop();
                    state.process_selected = Some(0);
                    state.selected_pid = None;
                    return false;
                }
                KeyCode::Down => {
                    move_process_cursor(state, 1);
                    return false;
                }
                KeyCode::Up => {
                    move_process_cursor(state, -1);
                    return false;
                }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    state.process_filter.get_or_insert_default().push(c);
                    state.process_selected = Some(0);
                    state.selected_pid = None;
                    return false;
                }
                _ => {} // Fall through to global handlers
            }
        }

        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                move_process_cursor(state, 1);
                return false;
            }
            KeyCode::Up => {
                move_process_cursor(state, -1);
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
                state.process_filter.get_or_insert_default();
                state.process_filter_editing = true;
                state.process_selected = Some(0);
                state.selected_pid = None;
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
                state.process_filter_editing = false;
                state.selected_pid = None;
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
            state.process_filter_editing = false;
            state.selected_pid = None;
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
            // I58-F1a: sort-mode change intentionally preserves selected_pid.
            // The whole point of the pid-key is that the highlight tracks the
            // process across re-orderings — sort mode is just another kind of
            // re-ordering.
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

/// Revalidate a queued destructive action against the current visible
/// selection immediately before delivery. I59-F1b.
pub(crate) fn pending_signal_is_current(state: &AppState, pid: i32, name: &str) -> bool {
    resolve_selected_process(state)
        .is_some_and(|(current_pid, current_name)| current_pid == pid && current_name == name)
}

/// Permanently cancel a queued signal as soon as its target ceases to be the
/// current effective selection. Once cleared, later pid/name reuse cannot
/// revive the old confirmation. I59-F1b.
pub(crate) fn invalidate_stale_pending_signal(state: &mut AppState) {
    let stale = state
        .pending_signal
        .as_ref()
        .is_some_and(|(pid, name, _)| !pending_signal_is_current(state, *pid, name));
    if stale {
        state.pending_signal = None;
    }
}

/// Resolve the currently selected process to `(pid, name)` via the unified
/// I58-F1b helper. I59-F1a fails closed when an anchored pid is absent;
/// cursor fallback remains available only when no pid has been anchored.
/// Highlight (in expanded.rs) resolves through the same helper, so the pid
/// this returns and the pid the user sees highlighted are always the same.
/// I45-F5c is preserved: filter narrows the view before resolution.
pub(crate) fn resolve_selected_process(state: &AppState) -> Option<(i32, String)> {
    let procs = &state.snapshot.processes;
    if procs.is_empty() {
        return None;
    }
    let indices = sorted_filtered_indices(procs, state.sort_mode, state.process_filter.as_deref());
    let row = effective_selection_row(procs, &indices, state.selected_pid, state.process_selected)?;
    // Row is already clamped by effective_selection_row; skip scroll offset
    // for kill target — the target is the visible highlighted row, not a
    // scroll-window-relative offset. This matches the highlight in expanded.rs.
    let idx = *indices.get(row)?;
    Some((procs[idx].pid, procs[idx].name.clone()))
}
