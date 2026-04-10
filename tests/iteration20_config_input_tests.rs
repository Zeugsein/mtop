/// Iteration 20: Coverage tests for config::load() paths and public tui rendering.
/// Input handler branch tests live in src/tui/tests.rs (where AppState is accessible).
use mtop::config::{self, Config};
use mtop::metrics::{MetricsSnapshot, SortMode};
use mtop::tui::{self, PanelId};

use std::sync::Mutex;

// Global mutex so that config::load() tests (which mutate HOME) are never
// run concurrently with each other. Cargo runs integration test binaries
// multi-threaded by default; without this the HOME env var races.
static HOME_LOCK: Mutex<()> = Mutex::new(());

// =========================================================================
// RAII helper: redirect HOME to a temp directory for config::load() tests
// =========================================================================

struct TempHome {
    dir: std::path::PathBuf,
    original_home: Option<String>,
}

impl TempHome {
    fn new() -> Self {
        // Use nanoseconds for a unique-enough dir name per call.
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        let dir = std::env::temp_dir().join(format!("mtop_test_{}", unique));
        std::fs::create_dir_all(&dir).unwrap();
        let original_home = std::env::var("HOME").ok();
        // config_path() resolves $HOME/.config/mtop/config.toml
        unsafe { std::env::set_var("HOME", &dir) };
        Self { dir, original_home }
    }

    /// Write content into $HOME/.config/mtop/config.toml
    fn write_config(&self, content: &str) {
        let config_dir = self.dir.join(".config").join("mtop");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(config_dir.join("config.toml"), content).unwrap();
    }
}

impl Drop for TempHome {
    fn drop(&mut self) {
        // Restore HOME before releasing the dir so any concurrent reader
        // of HOME still sees the original value.
        unsafe {
            match &self.original_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Run a closure with exclusive access to the HOME environment variable.
fn with_home_lock<F: FnOnce()>(f: F) {
    let _guard = HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    f();
}

// =========================================================================
// config::load() — file-I/O paths not covered by src/config.rs unit tests
// =========================================================================

#[test]
fn config_load_valid_toml_file() {
    with_home_lock(|| {
        let home = TempHome::new();
        home.write_config(
            r#"
theme = "monokai"
interval_ms = 500
temp_unit = "fahrenheit"
sort_mode = "cpu"
"#,
        );
        let cfg = config::load();
        assert_eq!(cfg.theme, "monokai");
        assert_eq!(cfg.interval_ms, 500);
        assert_eq!(cfg.temp_unit, "fahrenheit");
        assert_eq!(cfg.sort_mode, "cpu");
    });
}

#[test]
fn config_load_invalid_toml_returns_defaults() {
    with_home_lock(|| {
        let home = TempHome::new();
        home.write_config("this is ][[ not valid toml at all");
        let cfg = config::load();
        assert_eq!(cfg.theme, "default");
        assert_eq!(cfg.interval_ms, 1000);
        assert_eq!(cfg.temp_unit, "celsius");
        assert_eq!(cfg.sort_mode, "score");
    });
}

#[test]
fn config_load_nonexistent_file_returns_defaults() {
    with_home_lock(|| {
        let _home = TempHome::new();
        // No config file created — load() must return defaults without panicking.
        let cfg = config::load();
        assert_eq!(cfg.theme, "default");
        assert_eq!(cfg.interval_ms, 1000);
        assert_eq!(cfg.temp_unit, "celsius");
        assert_eq!(cfg.sort_mode, "score");
    });
}

#[test]
fn config_load_empty_file_returns_defaults() {
    with_home_lock(|| {
        let home = TempHome::new();
        home.write_config("");
        let cfg = config::load();
        assert_eq!(cfg.theme, "default");
        assert_eq!(cfg.interval_ms, 1000);
    });
}

#[test]
fn config_load_partial_toml_fills_missing_with_defaults() {
    with_home_lock(|| {
        let home = TempHome::new();
        home.write_config(r#"theme = "nord""#);
        let cfg = config::load();
        assert_eq!(cfg.theme, "nord");
        assert_eq!(cfg.interval_ms, 1000);
        assert_eq!(cfg.sort_mode, "score");
    });
}

// =========================================================================
// Config struct — serialization round-trip (integration-level)
// =========================================================================

#[test]
fn config_serialized_roundtrip() {
    let original = Config {
        theme: "dracula".to_string(),
        interval_ms: 333,
        temp_unit: "fahrenheit".to_string(),
        sort_mode: "memory".to_string(),
    };
    let toml_str = toml::to_string_pretty(&original).unwrap();
    let restored: Config = toml::from_str(&toml_str).unwrap();
    assert_eq!(restored.theme, original.theme);
    assert_eq!(restored.interval_ms, original.interval_ms);
    assert_eq!(restored.temp_unit, original.temp_unit);
    assert_eq!(restored.sort_mode, original.sort_mode);
}

#[test]
fn config_clone_is_independent() {
    let a = Config::default();
    let mut b = a.clone();
    b.theme = "custom".to_string();
    assert_eq!(a.theme, "default", "clone mutation must not affect original");
}

#[test]
fn config_debug_contains_field_names() {
    let cfg = Config::default();
    let s = format!("{:?}", cfg);
    assert!(s.contains("theme"));
    assert!(s.contains("interval_ms"));
}

// =========================================================================
// Rendering with expanded panels — exercises branches in dashboard rendering
// =========================================================================

fn empty_snapshot() -> MetricsSnapshot {
    MetricsSnapshot::default()
}

#[test]
fn render_with_show_detail_true_does_not_panic() {
    let text = tui::render_dashboard_to_string(120, 40, empty_snapshot(), true);
    assert!(!text.is_empty());
}

#[test]
fn render_with_expanded_cpu_panel() {
    let text = tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, Some(PanelId::Cpu), SortMode::default(),
    );
    assert!(!text.is_empty());
}

#[test]
fn render_with_expanded_gpu_panel() {
    let text = tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, Some(PanelId::Gpu), SortMode::default(),
    );
    assert!(!text.is_empty());
}

#[test]
fn render_with_expanded_memdisk_panel() {
    let text = tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, Some(PanelId::MemDisk), SortMode::default(),
    );
    assert!(!text.is_empty());
}

#[test]
fn render_with_expanded_network_panel() {
    let text = tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, Some(PanelId::Network), SortMode::default(),
    );
    assert!(!text.is_empty());
}

#[test]
fn render_with_expanded_power_panel() {
    let text = tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, Some(PanelId::Power), SortMode::default(),
    );
    assert!(!text.is_empty());
}

#[test]
fn render_with_expanded_process_panel() {
    let text = tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, Some(PanelId::Process), SortMode::default(),
    );
    assert!(!text.is_empty());
}

#[test]
fn render_sort_mode_cpu() {
    let text = tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, None, SortMode::Cpu,
    );
    assert!(!text.is_empty());
}

#[test]
fn render_sort_mode_memory() {
    let text = tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, None, SortMode::Memory,
    );
    assert!(!text.is_empty());
}

#[test]
fn render_sort_mode_name() {
    let text = tui::render_dashboard_with_state(
        120, 40, empty_snapshot(), false, None, SortMode::Name,
    );
    assert!(!text.is_empty());
}
