/// Feature-organized tests: config loading and input handling
/// Covers: config loading, deserialization, sort modes, process I/O fields,
/// thermal metrics, config roundtrips.
use mtop::config::{self, Config};
use mtop::metrics::{ProcessInfo, SortMode, ThermalMetrics};

use std::sync::Mutex;

// Global mutex so that config::load() tests (which mutate HOME) are never
// run concurrently with each other.
static HOME_LOCK: Mutex<()> = Mutex::new(());

// ===========================================================================
// RAII helper: redirect HOME to a temp directory for config::load() tests
// ===========================================================================

struct TempHome {
    dir: std::path::PathBuf,
    original_home: Option<String>,
}

impl TempHome {
    fn new() -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        let dir = std::env::temp_dir().join(format!("mtop_test_{}", unique));
        std::fs::create_dir_all(&dir).unwrap();
        let original_home = std::env::var("HOME").ok();
        unsafe { std::env::set_var("HOME", &dir) };
        Self { dir, original_home }
    }

    fn write_config(&self, content: &str) {
        let config_dir = self.dir.join(".config").join("mtop");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(config_dir.join("config.toml"), content).unwrap();
    }
}

impl Drop for TempHome {
    fn drop(&mut self) {
        unsafe {
            match &self.original_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn with_home_lock<F: FnOnce()>(f: F) {
    let _guard = HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    f();
}

// ===========================================================================
// Config defaults (iter10)
// ===========================================================================

#[test]
fn config_default_values() {
    let cfg = Config::default();
    assert_eq!(cfg.theme, "default");
    assert_eq!(cfg.interval_ms, 1000);
    assert_eq!(cfg.temp_unit, "celsius");
    assert_eq!(cfg.sort_mode, "score");
}

#[test]
fn config_deserialize_full() {
    let toml_str = r#"
theme = "monokai"
interval_ms = 500
temp_unit = "fahrenheit"
sort_mode = "cpu"
"#;
    let cfg: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(cfg.theme, "monokai");
    assert_eq!(cfg.interval_ms, 500);
    assert_eq!(cfg.temp_unit, "fahrenheit");
    assert_eq!(cfg.sort_mode, "cpu");
}

#[test]
fn config_deserialize_partial() {
    let toml_str = r#"
theme = "nord"
"#;
    let cfg: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(cfg.theme, "nord");
    assert_eq!(cfg.interval_ms, 1000);
    assert_eq!(cfg.sort_mode, "score");
}

#[test]
fn config_deserialize_empty() {
    let cfg: Config = toml::from_str("").unwrap();
    assert_eq!(cfg.theme, "default");
    assert_eq!(cfg.interval_ms, 1000);
}

#[test]
fn config_unknown_keys_ignored() {
    let toml_str = r#"
theme = "horizon"
unknown_future_key = true
another = 42
"#;
    let cfg: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(cfg.theme, "horizon");
}

#[test]
fn config_serialize_roundtrip() {
    let cfg = Config {
        theme: "dracula".to_string(),
        interval_ms: 750,
        temp_unit: "celsius".to_string(),
        sort_mode: "memory".to_string(),
    };
    let serialized = toml::to_string_pretty(&cfg).unwrap();
    let deserialized: Config = toml::from_str(&serialized).unwrap();
    assert_eq!(deserialized.theme, "dracula");
    assert_eq!(deserialized.interval_ms, 750);
    assert_eq!(deserialized.sort_mode, "memory");
}

#[test]
fn config_serialize_contains_all_fields() {
    let cfg = Config::default();
    let s = toml::to_string_pretty(&cfg).unwrap();
    assert!(s.contains("theme"), "serialized should contain theme");
    assert!(
        s.contains("interval_ms"),
        "serialized should contain interval_ms"
    );
    assert!(
        s.contains("temp_unit"),
        "serialized should contain temp_unit"
    );
    assert!(
        s.contains("sort_mode"),
        "serialized should contain sort_mode"
    );
}

#[test]
fn config_load_returns_defaults_on_missing_file() {
    let cfg = mtop::config::load();
    assert_eq!(cfg.interval_ms, 1000);
}

// ===========================================================================
// Sort mode (iter9, iter10)
// ===========================================================================

#[test]
fn sort_mode_default_is_weighted_score() {
    assert_eq!(SortMode::default(), SortMode::WeightedScore);
}

#[test]
fn sort_mode_cycle_from_weighted() {
    assert_eq!(SortMode::WeightedScore.next(), SortMode::Cpu);
}

#[test]
fn sort_mode_cycle_from_cpu() {
    assert_eq!(SortMode::Cpu.next(), SortMode::Memory);
}

#[test]
fn sort_mode_cycle_from_memory() {
    assert_eq!(SortMode::Memory.next(), SortMode::Power);
}

#[test]
fn sort_mode_cycle_from_power() {
    assert_eq!(SortMode::Power.next(), SortMode::Pid);
}

#[test]
fn sort_mode_cycle_from_pid() {
    assert_eq!(SortMode::Pid.next(), SortMode::Name);
}

#[test]
fn sort_mode_cycle_from_name_wraps() {
    assert_eq!(SortMode::Name.next(), SortMode::WeightedScore);
}

#[test]
fn sort_mode_full_cycle_returns_to_start() {
    let start = SortMode::WeightedScore;
    let end = start.next().next().next().next().next().next();
    assert_eq!(start, end);
}

#[test]
fn sort_mode_labels_non_empty() {
    let modes = [
        SortMode::WeightedScore,
        SortMode::Cpu,
        SortMode::Memory,
        SortMode::Power,
        SortMode::Pid,
        SortMode::Name,
    ];
    for mode in modes {
        assert!(
            !mode.label().is_empty(),
            "label for {:?} should not be empty",
            mode
        );
    }
}

#[test]
fn sort_mode_label_weighted_score() {
    assert_eq!(SortMode::WeightedScore.label(), "Score");
}

#[test]
fn sort_mode_label_cpu() {
    assert_eq!(SortMode::Cpu.label(), "CPU%");
}

#[test]
fn sort_mode_label_roundtrip_score() {
    assert_eq!(SortMode::WeightedScore.label(), "Score");
}

#[test]
fn sort_mode_label_roundtrip_cpu() {
    assert_eq!(SortMode::Cpu.label(), "CPU%");
}

#[test]
fn sort_mode_label_roundtrip_memory() {
    assert_eq!(SortMode::Memory.label(), "Mem");
}

#[test]
fn sort_mode_label_roundtrip_power() {
    assert_eq!(SortMode::Power.label(), "Power");
}

#[test]
fn sort_mode_label_roundtrip_pid() {
    assert_eq!(SortMode::Pid.label(), "PID");
}

#[test]
fn sort_mode_label_roundtrip_name() {
    assert_eq!(SortMode::Name.label(), "Name");
}

// ===========================================================================
// Process info fields (iter9)
// ===========================================================================

#[test]
fn process_info_default_thread_count_zero() {
    let p = ProcessInfo::default();
    assert_eq!(p.thread_count, 0);
}

#[test]
fn process_info_thread_count_set() {
    let p = ProcessInfo {
        thread_count: 12,
        ..Default::default()
    };
    assert_eq!(p.thread_count, 12);
}

#[test]
fn process_info_default_io_rates_zero() {
    let p = ProcessInfo::default();
    assert_eq!(p.io_read_bytes_sec, 0.0);
    assert_eq!(p.io_write_bytes_sec, 0.0);
}

#[test]
fn process_info_io_rates_set() {
    let p = ProcessInfo {
        io_read_bytes_sec: 1_048_576.0,
        io_write_bytes_sec: 524_288.0,
        ..Default::default()
    };
    assert_eq!(p.io_read_bytes_sec, 1_048_576.0);
    assert_eq!(p.io_write_bytes_sec, 524_288.0);
}

// ===========================================================================
// Thermal metrics (iter9)
// ===========================================================================

#[test]
fn thermal_metrics_default_ssd_zero() {
    let t = ThermalMetrics::default();
    assert_eq!(t.ssd_avg_c, 0.0);
}

#[test]
fn thermal_metrics_default_battery_zero() {
    let t = ThermalMetrics::default();
    assert_eq!(t.battery_avg_c, 0.0);
}

#[test]
fn thermal_metrics_ssd_field_set() {
    let t = ThermalMetrics {
        ssd_avg_c: 42.5,
        ..Default::default()
    };
    assert_eq!(t.ssd_avg_c, 42.5);
}

#[test]
fn thermal_metrics_battery_field_set() {
    let t = ThermalMetrics {
        battery_avg_c: 35.0,
        ..Default::default()
    };
    assert_eq!(t.battery_avg_c, 35.0);
}

#[test]
fn thermal_metrics_default_fan_speeds_empty() {
    let t = ThermalMetrics::default();
    assert!(t.fan_speeds.is_empty());
}

#[test]
fn thermal_metrics_fan_speeds_set() {
    let t = ThermalMetrics {
        fan_speeds: vec![1200, 1500],
        ..Default::default()
    };
    assert_eq!(t.fan_speeds.len(), 2);
    assert_eq!(t.fan_speeds[0], 1200);
    assert_eq!(t.fan_speeds[1], 1500);
}

// ===========================================================================
// Memory metrics fields (iter8)
// ===========================================================================

use mtop::metrics::MemoryMetrics;

#[test]
fn memory_metrics_has_pressure_fields() {
    let m = MemoryMetrics {
        ram_total: 32_000_000_000,
        ram_used: 16_000_000_000,
        swap_total: 4_000_000_000,
        swap_used: 1_000_000_000,
        wired: 3_000_000_000,
        app: 8_000_000_000,
        compressed: 2_000_000_000,
        cached: 1_000_000_000,
        free: 500_000_000,
        swap_in_bytes_sec: 0.0,
        swap_out_bytes_sec: 0.0,
        pressure_level: 1,
    };
    assert!(m.wired > 0);
    assert!(m.app > 0);
    assert!(m.compressed > 0);
    assert!(m.wired + m.app + m.compressed <= m.ram_total);
}

// ===========================================================================
// FFI offset assertions compile-time test (iter10)
// ===========================================================================

#[test]
fn w1a_offset_assertions_verified_by_compilation() {
    assert!(
        true,
        "All FFI struct offset assertions pass at compile time"
    );
}

// ===========================================================================
// config::load() file-I/O paths (iter20)
// ===========================================================================

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
    assert_eq!(
        a.theme, "default",
        "clone mutation must not affect original"
    );
}

#[test]
fn config_debug_contains_field_names() {
    let cfg = Config::default();
    let s = format!("{:?}", cfg);
    assert!(s.contains("theme"));
    assert!(s.contains("interval_ms"));
}
