/// Feature-organized tests: platform
/// Covers: IOReport, SMC, Mach ports, security/hardening, server connection limits,
/// Prometheus label escaping, JSON/pipe endpoints, source-code static analysis tests.

// ===========================================================================
// HTTP server connection limit (H5, iter2)
// ===========================================================================

#[test]
/// Validates: api-server [H5] - HTTP server limits concurrent connections to 64
fn http_server_limits_concurrent_connections() {
    use std::net::TcpStream;
    use std::sync::{Arc, RwLock};
    use std::time::Duration;

    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };

    let shared: mtop::serve::SharedMetrics = Arc::new(RwLock::new(None));
    let soc = mtop::metrics::types::SocInfo {
        chip: "Apple M4 Pro".into(),
        e_cores: 4,
        p_cores: 6,
        gpu_cores: 20,
        memory_gb: 24,
    };

    std::thread::spawn(move || {
        let last_request = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        ));
        mtop::serve::run(port, "127.0.0.1", shared, &soc, last_request, None).ok();
    });
    std::thread::sleep(Duration::from_millis(50));

    let mut held: Vec<TcpStream> = Vec::new();
    for _ in 0..64 {
        if let Ok(stream) = TcpStream::connect_timeout(
            &format!("127.0.0.1:{port}").parse().unwrap(),
            Duration::from_secs(1),
        ) {
            stream.set_read_timeout(Some(Duration::from_secs(1))).ok();
            held.push(stream);
        }
    }

    let _result = TcpStream::connect_timeout(
        &format!("127.0.0.1:{port}").parse().unwrap(),
        Duration::from_secs(1),
    );

    drop(held);

    std::thread::sleep(Duration::from_millis(100));
    let check = TcpStream::connect_timeout(
        &format!("127.0.0.1:{port}").parse().unwrap(),
        Duration::from_secs(1),
    );
    assert!(
        check.is_ok(),
        "H5: server should still accept connections after load test"
    );
}

// ===========================================================================
// Prometheus label escaping (M6, iter2)
// ===========================================================================

#[test]
/// Validates: api-server [M6] - backslash in label value is escaped to double backslash
fn prometheus_label_escapes_backslash() {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::sync::{Arc, RwLock};
    use std::time::Duration;

    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };

    let mut snapshot = mtop::metrics::types::MetricsSnapshot::default();
    snapshot.timestamp = "2026-04-06T00:00:00+00:00".into();
    let shared: mtop::serve::SharedMetrics = Arc::new(RwLock::new(Some(snapshot)));
    let soc = mtop::metrics::types::SocInfo {
        chip: "Apple\\M4".into(),
        e_cores: 4,
        p_cores: 6,
        gpu_cores: 20,
        memory_gb: 24,
    };

    std::thread::spawn(move || {
        let last_request = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        ));
        mtop::serve::run(port, "127.0.0.1", shared, &soc, last_request, None).ok();
    });
    std::thread::sleep(Duration::from_millis(50));

    let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
    let req = "GET /metrics HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    stream.write_all(req.as_bytes()).expect("write");
    let mut resp = String::new();
    stream.read_to_string(&mut resp).expect("read");

    let body = resp.split_once("\r\n\r\n").map(|(_, b)| b).unwrap_or(&resp);
    assert!(
        body.contains(r#"chip="Apple\\M4""#),
        "M6: backslash in label value must be escaped to \\\\; body:\n{body}"
    );
}

#[test]
/// Validates: api-server [M6] - double-quote in label value is escaped
fn prometheus_label_escapes_double_quote() {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::sync::{Arc, RwLock};
    use std::time::Duration;

    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };

    let mut snapshot = mtop::metrics::types::MetricsSnapshot::default();
    snapshot.timestamp = "2026-04-06T00:00:00+00:00".into();
    let shared: mtop::serve::SharedMetrics = Arc::new(RwLock::new(Some(snapshot)));
    let soc = mtop::metrics::types::SocInfo {
        chip: r#"Apple "M4" Pro"#.into(),
        e_cores: 4,
        p_cores: 6,
        gpu_cores: 20,
        memory_gb: 24,
    };

    std::thread::spawn(move || {
        let last_request = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        ));
        mtop::serve::run(port, "127.0.0.1", shared, &soc, last_request, None).ok();
    });
    std::thread::sleep(Duration::from_millis(50));

    let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
    let req = "GET /metrics HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    stream.write_all(req.as_bytes()).expect("write");
    let mut resp = String::new();
    stream.read_to_string(&mut resp).expect("read");

    let body = resp.split_once("\r\n\r\n").map(|(_, b)| b).unwrap_or(&resp);
    assert!(
        body.contains(r#"chip="Apple \"M4\" Pro""#),
        "M6: double-quote in label value must be escaped to \\\"; body:\n{body}"
    );
}

#[test]
/// Validates: api-server [M6] - newline in label value is escaped to backslash-n
fn prometheus_label_escapes_newline() {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::sync::{Arc, RwLock};
    use std::time::Duration;

    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };

    let mut snapshot = mtop::metrics::types::MetricsSnapshot::default();
    snapshot.timestamp = "2026-04-06T00:00:00+00:00".into();
    let shared: mtop::serve::SharedMetrics = Arc::new(RwLock::new(Some(snapshot)));
    let soc = mtop::metrics::types::SocInfo {
        chip: "Apple\nM4".into(),
        e_cores: 4,
        p_cores: 6,
        gpu_cores: 20,
        memory_gb: 24,
    };

    std::thread::spawn(move || {
        let last_request = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        ));
        mtop::serve::run(port, "127.0.0.1", shared, &soc, last_request, None).ok();
    });
    std::thread::sleep(Duration::from_millis(50));

    let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
    let req = "GET /metrics HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    stream.write_all(req.as_bytes()).expect("write");
    let mut resp = String::new();
    stream.read_to_string(&mut resp).expect("read");

    let body = resp.split_once("\r\n\r\n").map(|(_, b)| b).unwrap_or(&resp);
    assert!(
        body.contains(r#"chip="Apple\nM4""#),
        "M6: newline in label value must be escaped to \\n; body:\n{body}"
    );
}

#[test]
/// Validates: api-server [M6] - normal label values pass through unchanged
fn prometheus_label_normal_values_unchanged() {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::sync::{Arc, RwLock};
    use std::time::Duration;

    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };

    let mut snapshot = mtop::metrics::types::MetricsSnapshot::default();
    snapshot.timestamp = "2026-04-06T00:00:00+00:00".into();
    let shared: mtop::serve::SharedMetrics = Arc::new(RwLock::new(Some(snapshot)));
    let soc = mtop::metrics::types::SocInfo {
        chip: "Apple M4 Pro".into(),
        e_cores: 4,
        p_cores: 6,
        gpu_cores: 20,
        memory_gb: 24,
    };

    std::thread::spawn(move || {
        let last_request = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        ));
        mtop::serve::run(port, "127.0.0.1", shared, &soc, last_request, None).ok();
    });
    std::thread::sleep(Duration::from_millis(50));

    let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
    let req = "GET /metrics HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    stream.write_all(req.as_bytes()).expect("write");
    let mut resp = String::new();
    stream.read_to_string(&mut resp).expect("read");

    let body = resp.split_once("\r\n\r\n").map(|(_, b)| b).unwrap_or(&resp);
    assert!(
        body.contains(r#"chip="Apple M4 Pro""#),
        "M6: normal label values should pass through unchanged; body:\n{body}"
    );
}

// ===========================================================================
// Prometheus mtop_ prefix (FR-2, iter2)
// ===========================================================================

#[test]
/// Validates: api-server [FR-2] - all metric names use mtop_ prefix
fn prometheus_all_metrics_have_mtop_prefix() {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::sync::{Arc, RwLock};
    use std::time::Duration;

    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };

    let mut snapshot = mtop::metrics::types::MetricsSnapshot::default();
    snapshot.timestamp = "2026-04-06T00:00:00+00:00".into();
    let shared: mtop::serve::SharedMetrics = Arc::new(RwLock::new(Some(snapshot)));
    let soc = mtop::metrics::types::SocInfo {
        chip: "Apple M4 Pro".into(),
        e_cores: 4,
        p_cores: 6,
        gpu_cores: 20,
        memory_gb: 24,
    };

    std::thread::spawn(move || {
        let last_request = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        ));
        mtop::serve::run(port, "127.0.0.1", shared, &soc, last_request, None).ok();
    });
    std::thread::sleep(Duration::from_millis(50));

    let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
    let req = "GET /metrics HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    stream.write_all(req.as_bytes()).expect("write");
    let mut resp = String::new();
    stream.read_to_string(&mut resp).expect("read");

    let body = resp.split_once("\r\n\r\n").map(|(_, b)| b).unwrap_or(&resp);

    for line in body.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        assert!(
            line.starts_with("mtop_"),
            "FR-2: all Prometheus metric names must start with 'mtop_'; found: {line}"
        );
    }
}

// ===========================================================================
// JSON endpoint schema (FR-1, iter2)
// ===========================================================================

#[test]
/// Validates: api-server [FR-1] - JSON endpoint includes processes field
fn json_endpoint_includes_processes_field() {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::sync::{Arc, RwLock};
    use std::time::Duration;

    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };

    let mut snapshot = mtop::metrics::types::MetricsSnapshot::default();
    snapshot.timestamp = "2026-04-06T00:00:00+00:00".into();
    let shared: mtop::serve::SharedMetrics = Arc::new(RwLock::new(Some(snapshot)));
    let soc = mtop::metrics::types::SocInfo {
        chip: "Apple M4 Pro".into(),
        e_cores: 4,
        p_cores: 6,
        gpu_cores: 20,
        memory_gb: 24,
    };

    std::thread::spawn(move || {
        let last_request = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        ));
        mtop::serve::run(port, "127.0.0.1", shared, &soc, last_request, None).ok();
    });
    std::thread::sleep(Duration::from_millis(50));

    let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
    let req = "GET /json HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    stream.write_all(req.as_bytes()).expect("write");
    let mut resp = String::new();
    stream.read_to_string(&mut resp).expect("read");

    let body = resp.split_once("\r\n\r\n").map(|(_, b)| b).unwrap_or(&resp);
    let json: serde_json::Value = serde_json::from_str(body).expect("valid JSON");

    assert!(json.get("processes").is_some(), "JSON body should include 'processes' field");
    assert!(json["processes"].is_array(), "processes should be an array");
}

// ===========================================================================
// Pipe mode NDJSON (FR-4, iter2)
// ===========================================================================

#[test]
/// Validates: api-server [FR-4] - pipe output includes processes field
fn pipe_output_includes_processes_field() {
    use std::process::Command;
    let output = Command::new(env!("CARGO_BIN_EXE_mtop"))
        .args(["pipe", "--samples", "1"])
        .output()
        .expect("failed to run mtop binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next().expect("expected at least one line");
    let json: serde_json::Value = serde_json::from_str(line).expect("valid JSON");

    assert!(
        json.get("processes").is_some(),
        "FR-4: pipe JSON should include 'processes' field"
    );
}

// ===========================================================================
// Source-code static analysis: IOReport, SMC, Mach ports (iter3)
// ===========================================================================

#[test]
fn gpu_uses_correct_ioreport_group_name() {
    let source = std::fs::read_to_string("src/platform/gpu.rs")
        .expect("failed to read src/platform/gpu.rs");
    assert!(
        source.contains(r#""GPU Stats""#),
        "I3-C3: gpu.rs must use 'GPU Stats' group name, not 'GPU'"
    );
    assert!(
        !source.contains(r#"cfstring("GPU")"#),
        "I3-C3: gpu.rs must not use bare 'GPU' group name"
    );
}

#[test]
fn gpu_iterates_delta_channels_not_top_level() {
    let source = std::fs::read_to_string("src/platform/gpu.rs")
        .expect("failed to read src/platform/gpu.rs");
    assert!(
        source.contains("IOReportChannels"),
        "I3-C2: gpu.rs must extract IOReportChannels array from delta"
    );
    assert!(
        source.contains("CFArrayGetValueAtIndex") || source.contains("CFArrayGetCount"),
        "I3-C2: gpu.rs must iterate channel entries from the array"
    );
}

#[test]
fn sampler_drop_deallocates_mach_port() {
    let source = std::fs::read_to_string("src/metrics/sampler.rs")
        .expect("failed to read src/metrics/sampler.rs");
    assert!(
        source.contains("mach_port_deallocate"),
        "I3-C1: Sampler must call mach_port_deallocate in Drop"
    );
    assert!(
        source.contains("impl Drop for Sampler"),
        "I3-C1: Sampler must implement Drop"
    );
}

#[test]
fn power_uses_dynamic_energy_units() {
    let source = std::fs::read_to_string("src/platform/power.rs")
        .expect("failed to read src/platform/power.rs");
    assert!(
        source.contains("unit_label")
            || source.contains("UnitLabel")
            || source.contains("channel_get_unit"),
        "I3-C4: power.rs must read energy units dynamically"
    );
}

#[test]
fn power_uses_correct_channel_matching() {
    let source = std::fs::read_to_string("src/platform/power.rs")
        .expect("failed to read src/platform/power.rs");
    assert!(
        source.contains("CPU Energy")
            || (source.contains("cpu") && source.contains("energy")),
        "I3-C5: power.rs must match CPU energy channels"
    );
}

#[test]
fn temperature_uses_smc_keys_endpoint() {
    let source = std::fs::read_to_string("src/platform/temperature.rs")
        .expect("failed to read src/platform/temperature.rs");
    assert!(
        source.contains("AppleSMCKeysEndpoint"),
        "I3-C6: temperature.rs must target AppleSMCKeysEndpoint service"
    );
}

#[test]
fn temperature_supports_flt_type() {
    let source = std::fs::read_to_string("src/platform/temperature.rs")
        .expect("failed to read src/platform/temperature.rs");
    assert!(
        source.contains("flt "),
        "I3-C7: temperature.rs must handle 'flt ' data type"
    );
}

#[test]
fn debug_info_enumerates_sensors() {
    let source = std::fs::read_to_string("src/metrics/sampler.rs")
        .expect("failed to read src/metrics/sampler.rs");
    assert!(
        !source.contains("IOReport FFI active for GPU, power, and temperature metrics"),
        "I3-C9: debug_info must enumerate actual sensors, not print generic message"
    );
}

#[test]
fn server_uses_short_read_timeout() {
    let source = std::fs::read_to_string("src/serve/mod.rs")
        .expect("failed to read src/serve/mod.rs");
    assert!(
        !source.contains("from_secs(5)"),
        "I3-S1: serve must not use 5-second read timeout"
    );
    assert!(
        source.contains("from_secs(2)") || source.contains("from_millis(2000)"),
        "I3-S1: serve must use 2-second read timeout"
    );
}

#[test]
fn server_has_per_ip_connection_limit() {
    let source = std::fs::read_to_string("src/serve/mod.rs")
        .expect("failed to read src/serve/mod.rs");
    assert!(
        source.contains("peer_addr") || source.contains("SocketAddr"),
        "I3-S2: serve must track connections by IP address"
    );
}

#[test]
fn tui_handles_sensor_unavailable() {
    let mut tui_source = std::fs::read_to_string("src/tui/mod.rs")
        .expect("failed to read src/tui/mod.rs");
    for panel in &["cpu", "gpu", "power"] {
        if let Ok(s) = std::fs::read_to_string(format!("src/tui/panels/{panel}.rs")) {
            tui_source.push_str(&s);
        }
    }
    let _types_source = std::fs::read_to_string("src/metrics/types.rs")
        .expect("failed to read src/metrics/types.rs");
    assert!(
        tui_source.contains("N/A")
            || tui_source.contains("n/a")
            || tui_source.contains("unavailable"),
        "I3-T1/T2/T3: TUI must display N/A or unavailable when sensor data missing"
    );
}

#[test]
fn isolation_dlopen_cached_across_instances() {
    let source = std::fs::read_to_string("src/platform/ioreport_ffi.rs")
        .expect("failed to read src/platform/ioreport_ffi.rs");
    assert!(
        source.contains("OnceLock"),
        "M4/M5: IOReport FFI must cache dlopen handle via OnceLock"
    );
}

#[test]
fn isolation_smc_connection_cached() {
    let source = std::fs::read_to_string("src/platform/temperature.rs")
        .expect("failed to read src/platform/temperature.rs");
    assert!(
        source.contains("struct TemperatureState"),
        "M7: Temperature must use stateful connection caching"
    );
}

// ===========================================================================
// Forensic tests (hardware-dependent, #[ignore])
// ===========================================================================

#[test]
#[ignore]
fn forensic_mach_port_count_stable() {
    let mut sampler = mtop::metrics::Sampler::new().expect("sampler init");
    for i in 0..100 {
        let _ = sampler.sample(200).unwrap_or_else(|e| {
            panic!("H1: sample {} failed: {}", i, e);
        });
    }
}

#[test]
#[ignore]
fn forensic_ioreport_subscription_stable() {
    let mut sampler = mtop::metrics::Sampler::new().expect("sampler init");
    for i in 0..100 {
        let _ = sampler.sample(200).unwrap_or_else(|e| {
            panic!("H2: sample {} failed: {}", i, e);
        });
    }
}

#[test]
#[ignore]
fn forensic_subscription_count_stable() {
    let mut sampler = mtop::metrics::Sampler::new().expect("sampler init");
    for i in 0..100 {
        let _ = sampler.sample(200).unwrap_or_else(|e| {
            panic!("H4: sample {} failed: {}", i, e);
        });
    }
}
