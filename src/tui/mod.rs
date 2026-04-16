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
    let snap = snapshot.clone();
    let mut state = AppState {
        snapshot,
        show_detail: false,
        theme_idx,
        ..AppState::default()
    };
    // Populate history with sine-wave fluctuating values for realistic sparklines
    for i in 0..50 {
        let t = i as f64 / 49.0;
        let usage = (0.05_f64 + 0.90 * (t * std::f64::consts::PI).sin()) as f32;
        let mut varied = snap.clone();
        varied.cpu.total_usage = usage.clamp(0.0, 1.0);
        varied.cpu.e_cluster.usage = (usage * 0.7).clamp(0.0, 1.0);
        varied.cpu.p_cluster.usage = (usage * 0.95).clamp(0.0, 1.0);
        state.history.push(&varied);
    }
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
    let snap = snapshot.clone();
    let mut state = AppState {
        snapshot,
        show_detail: true,
        theme_idx,
        ..AppState::default()
    };
    // Populate history with sine-wave fluctuating values for realistic sparklines
    for i in 0..50 {
        let t = i as f64 / 49.0;
        let usage = (0.05_f64 + 0.90 * (t * std::f64::consts::PI).sin()) as f32;
        let mut varied = snap.clone();
        varied.cpu.total_usage = usage.clamp(0.0, 1.0);
        varied.cpu.e_cluster.usage = (usage * 0.7).clamp(0.0, 1.0);
        varied.cpu.p_cluster.usage = (usage * 0.95).clamp(0.0, 1.0);
        state.history.push(&varied);
    }
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

/// Test helper: render the GPU panel (show_detail=true) to a string.
pub fn render_gpu_panel_to_string(width: u16, height: u16, snapshot: MetricsSnapshot, theme_idx: usize) -> String {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let snap = snapshot.clone();
    let mut state = AppState {
        snapshot,
        show_detail: true,
        theme_idx,
        ..AppState::default()
    };
    for i in 0..50 {
        let t = i as f64 / 49.0;
        let usage = (0.05_f64 + 0.85 * (t * std::f64::consts::PI).sin()) as f32;
        let mut varied = snap.clone();
        varied.gpu.usage = usage.clamp(0.0, 1.0);
        varied.cpu.total_usage = (usage * 0.4).clamp(0.0, 1.0);
        state.history.push(&varied);
    }
    let theme = &theme::THEMES[theme_idx.min(theme::THEMES.len() - 1)];
    terminal.draw(|f| panels::draw_gpu_panel_v2(f, f.area(), &state.snapshot, &state, theme)).unwrap();
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

/// Test helper: render the memory/disk panel (show_detail=true) to a string.
pub fn render_mem_panel_to_string(width: u16, height: u16, snapshot: MetricsSnapshot, theme_idx: usize) -> String {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let snap = snapshot.clone();
    let mut state = AppState {
        snapshot,
        show_detail: true,
        theme_idx,
        ..AppState::default()
    };
    for i in 0..50 {
        let t = i as f64 / 49.0;
        let swap_rate = 512.0 * 1024.0 * (t * std::f64::consts::PI).sin().abs();
        let mut varied = snap.clone();
        varied.memory.swap_in_bytes_sec = swap_rate;
        varied.cpu.total_usage = (0.1 + 0.3 * (t * std::f64::consts::PI).sin()) as f32;
        state.history.push(&varied);
    }
    let theme = &theme::THEMES[theme_idx.min(theme::THEMES.len() - 1)];
    terminal.draw(|f| panels::draw_mem_disk_panel_v2(f, f.area(), &state.snapshot, &state, theme)).unwrap();
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

/// Test helper: render the power panel (show_detail=true) to a string.
pub fn render_power_panel_to_string(width: u16, height: u16, snapshot: MetricsSnapshot, theme_idx: usize) -> String {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let snap = snapshot.clone();
    let mut state = AppState {
        snapshot,
        show_detail: true,
        theme_idx,
        ..AppState::default()
    };
    for i in 0..50 {
        let t = i as f64 / 49.0;
        let factor = (0.10_f64 + 0.85 * (t * std::f64::consts::PI).sin()) as f32;
        let mut varied = snap.clone();
        varied.cpu.total_usage = (factor * 0.8).clamp(0.0, 1.0);
        varied.power.cpu_w = snap.power.cpu_w * factor;
        state.history.push(&varied);
    }
    let theme = &theme::THEMES[theme_idx.min(theme::THEMES.len() - 1)];
    terminal.draw(|f| panels::draw_power_panel_v2(f, f.area(), &state.snapshot, &state, theme)).unwrap();
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

/// Test helper: render the network panel (show_detail=true) to a string.
pub fn render_network_panel_to_string(width: u16, height: u16, snapshot: MetricsSnapshot, theme_idx: usize) -> String {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let snap = snapshot.clone();
    let mut state = AppState {
        snapshot,
        show_detail: true,
        theme_idx,
        ..AppState::default()
    };
    for i in 0..50 {
        let t = i as f64 / 49.0;
        let factor = 0.05 + 0.90 * (t * std::f64::consts::PI).sin().abs();
        let mut varied = snap.clone();
        for iface in &mut varied.network.interfaces {
            iface.rx_bytes_sec *= factor;
            iface.tx_bytes_sec *= factor;
        }
        varied.cpu.total_usage = (factor * 0.3) as f32;
        state.history.push(&varied);
    }
    let theme = &theme::THEMES[theme_idx.min(theme::THEMES.len() - 1)];
    terminal.draw(|f| panels::draw_network_panel_v2(f, f.area(), &state.snapshot, &state, theme)).unwrap();
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

/// Test helper: render the process panel (show_detail=true) to a string.
pub fn render_process_panel_to_string(width: u16, height: u16, snapshot: MetricsSnapshot, theme_idx: usize) -> String {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let snap = snapshot.clone();
    let mut state = AppState {
        snapshot,
        show_detail: true,
        theme_idx,
        ..AppState::default()
    };
    for i in 0..50 {
        let t = i as f64 / 49.0;
        let usage = (0.05_f64 + 0.90 * (t * std::f64::consts::PI).sin()) as f32;
        let mut varied = snap.clone();
        varied.cpu.total_usage = usage.clamp(0.0, 1.0);
        state.history.push(&varied);
    }
    let theme = &theme::THEMES[theme_idx.min(theme::THEMES.len() - 1)];
    terminal.draw(|f| panels::draw_process_panel_v2(f, f.area(), &state.snapshot, &state, theme)).unwrap();
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

/// Story fixture: cpu panel, normal load (M3 Pro, 42% total)
pub fn story_cpu_normal_fixture() -> MetricsSnapshot {
    let mut s = MetricsSnapshot::default();
    s.soc.chip = "Apple M3 Pro".to_string();
    s.soc.e_cores = 4; s.soc.p_cores = 6; s.soc.gpu_cores = 18; s.soc.memory_gb = 18;
    s.cpu.total_usage = 0.42;
    s.cpu.e_cluster.freq_mhz = 1200; s.cpu.e_cluster.usage = 0.24;
    s.cpu.p_cluster.freq_mhz = 3400; s.cpu.p_cluster.usage = 0.67;
    s.cpu.core_usages = vec![0.12, 0.45, 0.08, 0.31, 0.78, 0.55, 0.92, 0.43, 0.61, 0.39];
    s.power.cpu_w = 8.5; s.power.gpu_w = 3.2; s.power.ane_w = 0.8; s.power.dram_w = 1.5;
    s.power.package_w = 14.0; s.power.system_w = 16.5; s.power.available = true;
    s.temperature.cpu_avg_c = 55.0; s.temperature.gpu_avg_c = 48.0;
    s.temperature.available = true; s.temperature.fan_speeds = vec![1200];
    s
}

/// Story fixture: gpu panel, active usage
pub fn story_gpu_active_fixture() -> MetricsSnapshot {
    let mut s = story_cpu_normal_fixture();
    s.gpu.usage = 0.73; s.gpu.freq_mhz = 1400; s.gpu.available = true;
    s.power.gpu_w = 12.5; s.power.ane_w = 1.2; s.power.dram_w = 2.3;
    s.power.cpu_w = 4.5; s.power.package_w = 20.5; s.power.system_w = 25.0;
    s.temperature.gpu_avg_c = 68.0; s.temperature.cpu_avg_c = 52.0;
    s
}

/// Story fixture: mem/disk panel, near full with critical pressure
pub fn story_mem_near_full_fixture() -> MetricsSnapshot {
    let mut s = story_cpu_normal_fixture();
    let gb: u64 = 1024 * 1024 * 1024;
    s.memory.ram_total = 18 * gb;
    s.memory.ram_used = 17 * gb;
    s.memory.wired = 4 * gb;
    s.memory.app = 10 * gb;
    s.memory.compressed = 2 * gb;
    s.memory.cached = gb;
    s.memory.free = 256 * 1024 * 1024;
    s.memory.pressure_level = 4;
    s.memory.swap_total = 4 * gb; s.memory.swap_used = 2 * gb;
    s.memory.swap_in_bytes_sec = 512.0 * 1024.0; s.memory.swap_out_bytes_sec = 256.0 * 1024.0;
    s
}

/// Story fixture: network panel, active traffic on 3 interfaces
pub fn story_network_active_fixture() -> MetricsSnapshot {
    use crate::metrics::NetInterface;
    let mut s = story_cpu_normal_fixture();
    s.network.interfaces = vec![
        NetInterface {
            name: "en0".to_string(),
            iface_type: "ethernet".to_string(),
            rx_bytes_sec: 8_500_000.0,
            tx_bytes_sec: 2_200_000.0,
            baudrate: 1_000_000_000,
            rx_bytes_total: 2_500_000_000,
            tx_bytes_total: 800_000_000,
            ..Default::default()
        },
        NetInterface {
            name: "en1".to_string(),
            iface_type: "wifi".to_string(),
            rx_bytes_sec: 1_200_000.0,
            tx_bytes_sec: 450_000.0,
            baudrate: 600_000_000,
            rx_bytes_total: 900_000_000,
            tx_bytes_total: 300_000_000,
            ..Default::default()
        },
        NetInterface {
            name: "utun3".to_string(),
            iface_type: "vpn".to_string(),
            rx_bytes_sec: 85_000.0,
            tx_bytes_sec: 12_000.0,
            baudrate: 10_000_000,
            rx_bytes_total: 50_000_000,
            tx_bytes_total: 15_000_000,
            ..Default::default()
        },
    ];
    s
}

/// Story fixture: process panel, 10 populated processes
pub fn story_process_populated_fixture() -> MetricsSnapshot {
    use crate::metrics::ProcessInfo;
    let mut s = story_cpu_normal_fixture();
    s.processes = vec![
        ProcessInfo { pid: 1,    name: "kernel_task".to_string(), cpu_pct: 5.2,  mem_bytes: 2_048_000_000, power_w: 0.0, user: "root".to_string(),          thread_count: 512, io_read_bytes_sec: 1_000_000.0, io_write_bytes_sec: 500_000.0, ..Default::default() },
        ProcessInfo { pid: 312,  name: "WindowServer".to_string(), cpu_pct: 12.4, mem_bytes: 512_000_000,  power_w: 1.8, user: "_windowserver".to_string(),  thread_count: 18,  io_read_bytes_sec: 250_000.0,   io_write_bytes_sec: 100_000.0, ..Default::default() },
        ProcessInfo { pid: 891,  name: "Safari".to_string(),       cpu_pct: 8.7,  mem_bytes: 350_000_000,  power_w: 0.9, user: "lume".to_string(),           thread_count: 32,  io_read_bytes_sec: 80_000.0,    io_write_bytes_sec: 20_000.0,  ..Default::default() },
        ProcessInfo { pid: 1204, name: "Xcode".to_string(),        cpu_pct: 45.3, mem_bytes: 1_800_000_000, power_w: 6.2, user: "lume".to_string(),          thread_count: 64,  ..Default::default() },
        ProcessInfo { pid: 2341, name: "mtop".to_string(),         cpu_pct: 3.1,  mem_bytes: 8_500_000,    power_w: 0.2, user: "lume".to_string(),           thread_count: 4,   ..Default::default() },
        ProcessInfo { pid: 445,  name: "Finder".to_string(),       cpu_pct: 0.8,  mem_bytes: 95_000_000,   power_w: 0.1, user: "lume".to_string(),           thread_count: 8,   ..Default::default() },
        ProcessInfo { pid: 678,  name: "Terminal".to_string(),     cpu_pct: 1.4,  mem_bytes: 45_000_000,   power_w: 0.05, user: "lume".to_string(),          thread_count: 6,   ..Default::default() },
        ProcessInfo { pid: 2890, name: "cargo".to_string(),        cpu_pct: 88.5, mem_bytes: 420_000_000,  power_w: 9.8, user: "lume".to_string(),           thread_count: 16,  ..Default::default() },
        ProcessInfo { pid: 334,  name: "coreaudiod".to_string(),   cpu_pct: 0.3,  mem_bytes: 22_000_000,   power_w: 0.03, user: "_coreaudio".to_string(),    thread_count: 4,   ..Default::default() },
        ProcessInfo { pid: 512,  name: "mds_stores".to_string(),   cpu_pct: 15.2, mem_bytes: 180_000_000,  power_w: 1.1, user: "root".to_string(),           thread_count: 8,   ..Default::default() },
    ];
    s
}

/// Run the interactive story browser in the current terminal.
/// Stories: all panels with meaningful fixture data.
/// Navigate: n/→ next, p/← prev, q quit.
pub fn run_stories() -> Result<(), Box<dyn std::error::Error>> {
    use crossterm::{event::{self, Event, KeyCode}, terminal, ExecutableCommand};
    use ratatui::backend::CrosstermBackend;
    use ratatui::Terminal;
    use std::io::stdout;

    type DrawFn = fn(&mut ratatui::Frame, ratatui::layout::Rect, &MetricsSnapshot, &AppState, &theme::Theme);

    struct Story {
        name: &'static str,
        snapshot: MetricsSnapshot,
        show_detail: bool,
        theme_idx: usize,
        draw_fn: DrawFn,
    }

    let dark = 0usize;
    let stories: Vec<Story> = vec![
        Story { name: "cpu compact — normal",  snapshot: story_cpu_normal_fixture(),      show_detail: false, theme_idx: dark, draw_fn: panels::draw_cpu_panel_v2 },
        Story { name: "cpu show — normal",     snapshot: story_cpu_normal_fixture(),      show_detail: true,  theme_idx: dark, draw_fn: panels::draw_cpu_panel_v2 },
        Story { name: "gpu active",            snapshot: story_gpu_active_fixture(),      show_detail: true,  theme_idx: dark, draw_fn: panels::draw_gpu_panel_v2 },
        Story { name: "mem near-full",         snapshot: story_mem_near_full_fixture(),   show_detail: true,  theme_idx: dark, draw_fn: panels::draw_mem_disk_panel_v2 },
        Story { name: "network active",        snapshot: story_network_active_fixture(),  show_detail: true,  theme_idx: dark, draw_fn: panels::draw_network_panel_v2 },
        Story { name: "process populated",     snapshot: story_process_populated_fixture(), show_detail: true, theme_idx: dark, draw_fn: panels::draw_process_panel_v2 },
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
        let story = &stories[current];
        let th = &theme::THEMES[story.theme_idx];
        let snap = story.snapshot.clone();
        let mut state = AppState {
            snapshot: story.snapshot.clone(),
            show_detail: story.show_detail,
            theme_idx: story.theme_idx,
            ..AppState::default()
        };
        // Populate history with sine-wave fluctuating values for realistic sparklines
        for i in 0..50 {
            let t = i as f64 / 49.0;
            let usage = (0.05_f64 + 0.90 * (t * std::f64::consts::PI).sin()) as f32;
            let mut varied = snap.clone();
            varied.cpu.total_usage = usage.clamp(0.0, 1.0);
            state.history.push(&varied);
        }

        let name = story.name;
        let draw_fn = story.draw_fn;
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

            draw_fn(f, panel_area, &state.snapshot, &state, th);
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
