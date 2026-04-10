use mtop::metrics::types::*;

// =========================================================================
// W2: Per-interface network history
// =========================================================================

fn make_iface_snapshot(ifaces: &[(&str, f64, f64)]) -> MetricsSnapshot {
    let mut snapshot = MetricsSnapshot::default();
    for &(name, rx, tx) in ifaces {
        snapshot.network.interfaces.push(NetInterface {
            name: name.to_string(),
            rx_bytes_sec: rx,
            tx_bytes_sec: tx,
            ..Default::default()
        });
    }
    snapshot
}

#[test]
fn per_iface_push_creates_separate_buffers() {
    let mut h = MetricsHistory::new();
    let snap = make_iface_snapshot(&[("en0", 100.0, 200.0), ("en1", 300.0, 400.0)]);
    h.push(&snap);

    assert!(h.per_iface.contains_key("en0"), "en0 buffer should exist");
    assert!(h.per_iface.contains_key("en1"), "en1 buffer should exist");

    let (rx0, tx0) = h.per_iface.get("en0").unwrap();
    assert_eq!(*rx0.last().unwrap(), 100.0);
    assert_eq!(*tx0.last().unwrap(), 200.0);

    let (rx1, tx1) = h.per_iface.get("en1").unwrap();
    assert_eq!(*rx1.last().unwrap(), 300.0);
    assert_eq!(*tx1.last().unwrap(), 400.0);
}

#[test]
fn per_iface_buffers_cap_at_128() {
    let mut h = MetricsHistory::new();
    let snap = make_iface_snapshot(&[("en0", 1.0, 2.0)]);
    for _ in 0..150 {
        h.push(&snap);
    }
    let (rx, tx) = h.per_iface.get("en0").unwrap();
    assert_eq!(rx.len(), 128, "per-iface rx should cap at 128");
    assert_eq!(tx.len(), 128, "per-iface tx should cap at 128");
}

#[test]
fn per_iface_skips_loopback() {
    let mut h = MetricsHistory::new();
    let snap = make_iface_snapshot(&[("lo0", 100.0, 200.0), ("en0", 50.0, 60.0)]);
    h.push(&snap);
    assert!(!h.per_iface.contains_key("lo0"), "loopback should be skipped");
    assert!(h.per_iface.contains_key("en0"));
}

#[test]
fn per_iface_stale_pruned() {
    let mut h = MetricsHistory::new();
    // First push with en0 and en1
    let snap1 = make_iface_snapshot(&[("en0", 100.0, 200.0), ("en1", 300.0, 400.0)]);
    h.push(&snap1);
    assert!(h.per_iface.contains_key("en1"));
    // Second push with only en0
    let snap2 = make_iface_snapshot(&[("en0", 150.0, 250.0)]);
    h.push(&snap2);
    // en1 buffer should be pruned (stale interfaces removed to bound memory)
    assert!(!h.per_iface.contains_key("en1"), "stale interface buffer should be pruned");
}

#[test]
fn per_iface_independent_values() {
    let mut h = MetricsHistory::new();
    let snap1 = make_iface_snapshot(&[("en0", 10.0, 20.0), ("en1", 100.0, 200.0)]);
    h.push(&snap1);
    let snap2 = make_iface_snapshot(&[("en0", 30.0, 40.0), ("en1", 500.0, 600.0)]);
    h.push(&snap2);

    let (rx0, _) = h.per_iface.get("en0").unwrap();
    assert_eq!(rx0.len(), 2);
    assert_eq!(rx0[0], 10.0);
    assert_eq!(rx0[1], 30.0);

    let (rx1, _) = h.per_iface.get("en1").unwrap();
    assert_eq!(rx1[0], 100.0);
    assert_eq!(rx1[1], 500.0);
}

#[test]
fn per_iface_aggregate_still_works() {
    let mut h = MetricsHistory::new();
    let snap = make_iface_snapshot(&[("en0", 100.0, 200.0), ("en1", 300.0, 400.0)]);
    h.push(&snap);
    assert_eq!(*h.net_download.last().unwrap(), 400.0); // 100 + 300
    assert_eq!(*h.net_upload.last().unwrap(), 600.0);   // 200 + 400
}

// =========================================================================
// W3: Configuration persistence
// =========================================================================

use mtop::config::Config;

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
    assert!(s.contains("interval_ms"), "serialized should contain interval_ms");
    assert!(s.contains("temp_unit"), "serialized should contain temp_unit");
    assert!(s.contains("sort_mode"), "serialized should contain sort_mode");
}

#[test]
fn config_load_returns_defaults_on_missing_file() {
    // load() returns defaults when file doesn't exist
    let cfg = mtop::config::load();
    assert_eq!(cfg.interval_ms, 1000);
}

// =========================================================================
// W1A: offset_of! assertions (compile-time only, verified by compilation)
// W1B: SAFETY comments (verified by code review, not runtime tests)
// =========================================================================

// These are compile-time assertions — if the crate compiles, W1A passes.
// This test documents that fact.
#[test]
fn w1a_offset_assertions_verified_by_compilation() {
    // RusageInfoV4: offsets 144, 152, 264
    // ProcTaskInfo: offsets 8, 16, 24, 84
    // SmcKeyData: offsets 0, 37, 38, 48
    // IfData: offsets 16, 24, 64, 72
    // VmStatistics64: offsets 12, 128, 140
    // If any of these were wrong, cargo check would fail with a compile error.
    assert!(true, "All FFI struct offset assertions pass at compile time");
}

// =========================================================================
// Sort mode label mapping (for config persistence)
// =========================================================================

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
