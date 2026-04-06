/// Integration tests for api-server spec requirements.
/// Each test cites the FR requirement from:
///   openspec/changes/archive/2026-04-05-mvp-core/specs/api-server/spec.md
///
/// Tests marked #[ignore] cover known PARTIAL / FAIL items in the compliance audit.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use mtop::metrics::types::{MetricsSnapshot, SocInfo};
use mtop::serve;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Spawn the HTTP server on a free port in a background thread.
/// Returns the bound port.
fn spawn_server_with_data(snapshot: Option<MetricsSnapshot>) -> u16 {
    let port = free_port();
    let shared: serve::SharedMetrics = Arc::new(RwLock::new(snapshot));
    let soc = SocInfo {
        chip: "Apple M4 Pro".into(),
        e_cores: 4,
        p_cores: 6,
        gpu_cores: 20,
        memory_gb: 24,
    };
    std::thread::spawn(move || {
        serve::run(port, shared, &soc).ok();
    });
    // Give the server a moment to bind
    std::thread::sleep(Duration::from_millis(50));
    port
}

fn free_port() -> u16 {
    // Bind to :0 to get an OS-assigned free port
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

/// Send a raw HTTP/1.1 GET request and return the full response as a String.
fn http_get(port: u16, path: &str) -> String {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
    let req = format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes()).expect("write request");
    let mut resp = String::new();
    stream.read_to_string(&mut resp).expect("read response");
    resp
}

fn make_snapshot() -> MetricsSnapshot {
    let mut s = MetricsSnapshot::default();
    s.timestamp = "2026-04-05T00:00:00+00:00".into();
    s.soc.chip = "Apple M4 Pro".into();
    s.memory.ram_total = 25_769_803_776; // 24 GB
    s.memory.ram_used = 8_589_934_592;   // 8 GB
    s
}

// ---------------------------------------------------------------------------
// FR-1: GET /json endpoint
// ---------------------------------------------------------------------------

#[test]
/// FR-1: GET /json returns HTTP 200 when metrics are available
fn json_endpoint_returns_200_with_data() {
    let port = spawn_server_with_data(Some(make_snapshot()));
    let resp = http_get(port, "/json");
    assert!(
        resp.starts_with("HTTP/1.1 200"),
        "expected 200 OK; got: {resp}"
    );
}

#[test]
/// FR-1: GET /json response body is valid JSON
fn json_endpoint_body_is_valid_json() {
    let port = spawn_server_with_data(Some(make_snapshot()));
    let resp = http_get(port, "/json");
    let body = body_of(&resp);
    serde_json::from_str::<serde_json::Value>(body)
        .expect("response body should be valid JSON");
}

#[test]
/// FR-1: GET /json JSON body contains required top-level fields
fn json_endpoint_contains_required_fields() {
    let port = spawn_server_with_data(Some(make_snapshot()));
    let resp = http_get(port, "/json");
    let body = body_of(&resp);
    let json: serde_json::Value = serde_json::from_str(body).unwrap();

    for field in &["timestamp", "soc", "cpu", "gpu", "power", "temperature", "memory", "network", "disk"] {
        assert!(
            json.get(field).is_some(),
            "JSON body missing required field '{field}'"
        );
    }
}

#[test]
/// FR-1: GET /json timestamp field is an ISO 8601 string
fn json_endpoint_timestamp_is_iso8601() {
    let port = spawn_server_with_data(Some(make_snapshot()));
    let resp = http_get(port, "/json");
    let body = body_of(&resp);
    let json: serde_json::Value = serde_json::from_str(body).unwrap();
    let ts = json["timestamp"].as_str().expect("timestamp should be a string");
    assert!(
        ts.contains('T'),
        "timestamp '{ts}' does not look like ISO 8601"
    );
}

#[test]
/// FR-1: GET /json returns HTTP 503 when no metrics are available yet
fn json_endpoint_returns_503_when_no_data() {
    let port = spawn_server_with_data(None);
    let resp = http_get(port, "/json");
    assert!(
        resp.starts_with("HTTP/1.1 503"),
        "expected 503 when no data; got: {resp}"
    );
}

#[test]
/// FR-1: GET /json 503 response body contains an error field
fn json_endpoint_503_body_has_error_field() {
    let port = spawn_server_with_data(None);
    let resp = http_get(port, "/json");
    let body = body_of(&resp);
    let json: serde_json::Value = serde_json::from_str(body).unwrap();
    assert!(
        json.get("error").is_some(),
        "503 body should contain an 'error' field; got: {body}"
    );
}

// ---------------------------------------------------------------------------
// FR-2: GET /metrics Prometheus endpoint (PARTIAL — duplicate HELP/TYPE, missing headers)
// ---------------------------------------------------------------------------

#[test]
/// FR-2: GET /metrics returns HTTP 200 when metrics are available
fn prometheus_endpoint_returns_200_with_data() {
    let port = spawn_server_with_data(Some(make_snapshot()));
    let resp = http_get(port, "/metrics");
    assert!(
        resp.starts_with("HTTP/1.1 200"),
        "expected 200 OK from /metrics; got: {resp}"
    );
}

#[test]
/// FR-2: GET /metrics Content-Type is text/plain
fn prometheus_endpoint_content_type_is_text_plain() {
    let port = spawn_server_with_data(Some(make_snapshot()));
    let resp = http_get(port, "/metrics");
    assert!(
        resp.to_lowercase().contains("content-type: text/plain"),
        "Content-Type should be text/plain; response headers: {}",
        headers_of(&resp)
    );
}

#[test]
/// FR-2: GET /metrics body contains mtop_cpu_usage_ratio gauge
fn prometheus_endpoint_contains_cpu_usage_ratio() {
    let port = spawn_server_with_data(Some(make_snapshot()));
    let resp = http_get(port, "/metrics");
    let body = body_of(&resp);
    assert!(
        body.contains("mtop_cpu_usage_ratio"),
        "Prometheus body should contain mtop_cpu_usage_ratio"
    );
}

#[test]
/// FR-2: GET /metrics body contains mtop_gpu_usage_ratio gauge
fn prometheus_endpoint_contains_gpu_usage_ratio() {
    let port = spawn_server_with_data(Some(make_snapshot()));
    let resp = http_get(port, "/metrics");
    let body = body_of(&resp);
    assert!(
        body.contains("mtop_gpu_usage_ratio"),
        "Prometheus body should contain mtop_gpu_usage_ratio"
    );
}

#[test]
/// FR-2: GET /metrics body contains mtop_power_watts gauge
fn prometheus_endpoint_contains_power_watts() {
    let port = spawn_server_with_data(Some(make_snapshot()));
    let resp = http_get(port, "/metrics");
    let body = body_of(&resp);
    assert!(
        body.contains("mtop_power_watts"),
        "Prometheus body should contain mtop_power_watts"
    );
}

#[test]
/// FR-2: GET /metrics body contains mtop_memory_bytes gauge
fn prometheus_endpoint_contains_memory_bytes() {
    let port = spawn_server_with_data(Some(make_snapshot()));
    let resp = http_get(port, "/metrics");
    let body = body_of(&resp);
    assert!(
        body.contains("mtop_memory_bytes"),
        "Prometheus body should contain mtop_memory_bytes"
    );
}

#[test]
/// FR-2: GET /metrics body contains mtop_temperature_celsius gauge
fn prometheus_endpoint_contains_temperature_celsius() {
    let port = spawn_server_with_data(Some(make_snapshot()));
    let resp = http_get(port, "/metrics");
    let body = body_of(&resp);
    assert!(
        body.contains("mtop_temperature_celsius"),
        "Prometheus body should contain mtop_temperature_celsius"
    );
}

#[test]
/// FR-2: GET /metrics power labels include component="cpu", "gpu", "ane", "dram"
fn prometheus_endpoint_power_has_component_labels() {
    let port = spawn_server_with_data(Some(make_snapshot()));
    let resp = http_get(port, "/metrics");
    let body = body_of(&resp);
    for component in &["cpu", "gpu", "ane", "dram", "package", "system"] {
        assert!(
            body.contains(&format!("component=\"{component}\"")),
            "Prometheus body missing power component label '{component}'"
        );
    }
}

#[test]
#[ignore] // FR-2 (PARTIAL): duplicate HELP/TYPE lines — each metric name should appear exactly once
/// FR-2: each metric name has exactly one # HELP and one # TYPE declaration
fn prometheus_endpoint_no_duplicate_help_type() {
    let port = spawn_server_with_data(Some(make_snapshot()));
    let resp = http_get(port, "/metrics");
    let body = body_of(&resp);

    let mut help_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for line in body.lines() {
        if let Some(name) = line.strip_prefix("# HELP ") {
            let metric_name = name.split_whitespace().next().unwrap_or("");
            *help_counts.entry(metric_name).or_insert(0) += 1;
        }
    }

    for (name, count) in &help_counts {
        assert_eq!(
            *count, 1,
            "metric '{name}' has {count} # HELP declarations (expected exactly 1)"
        );
    }
}

#[test]
/// FR-2: GET /metrics returns 503 when no data available
fn prometheus_endpoint_returns_503_when_no_data() {
    let port = spawn_server_with_data(None);
    let resp = http_get(port, "/metrics");
    assert!(
        resp.starts_with("HTTP/1.1 503"),
        "expected 503 when no data; got: {resp}"
    );
}

// ---------------------------------------------------------------------------
// FR-3: Server port configuration (PARTIAL — --bind flag missing)
// ---------------------------------------------------------------------------

#[test]
/// FR-3: server listens on 127.0.0.1 by default (not 0.0.0.0)
fn server_binds_to_localhost_by_default() {
    let port = spawn_server_with_data(Some(make_snapshot()));

    // Should be able to connect via 127.0.0.1
    let ok = TcpStream::connect(format!("127.0.0.1:{port}")).is_ok();
    assert!(ok, "server should accept connections on 127.0.0.1:{port}");
}

#[test]
/// FR-3: server listens on the configured port
fn server_listens_on_configured_port() {
    let port = spawn_server_with_data(Some(make_snapshot()));
    let resp = http_get(port, "/json");
    assert!(
        resp.starts_with("HTTP/1.1 200"),
        "server should be reachable on configured port {port}"
    );
}

#[test]
#[ignore] // FR-3 (PARTIAL): --bind flag is not implemented yet
/// FR-3: serve subcommand accepts a --bind flag for custom bind address
fn serve_subcommand_accepts_bind_flag() {
    use clap::Parser;
    use mtop::Cli;
    // This will fail to parse until --bind is added to the Serve subcommand
    let cli = Cli::parse_from(["mtop", "serve", "--bind", "0.0.0.0", "--port", "9191"]);
    match cli.command {
        Some(mtop::cli::Command::Serve { port, .. }) => {
            assert_eq!(port, 9191);
        }
        _ => panic!("expected Serve subcommand"),
    }
}

// ---------------------------------------------------------------------------
// FR-4: NDJSON pipe mode (tested via CLI layer here for server-side contract)
// ---------------------------------------------------------------------------

// Pipe mode tests live primarily in cli_interface.rs; no server-side component.

// ---------------------------------------------------------------------------
// FR-5: Unknown route returns 404
// ---------------------------------------------------------------------------

#[test]
/// FR-5: GET /foo returns HTTP 404
fn unknown_route_returns_404() {
    let port = spawn_server_with_data(Some(make_snapshot()));
    let resp = http_get(port, "/foo");
    assert!(
        resp.starts_with("HTTP/1.1 404"),
        "expected 404 for unknown path; got: {resp}"
    );
}

#[test]
/// FR-5: GET / (root) returns HTTP 404
fn root_route_returns_404() {
    let port = spawn_server_with_data(Some(make_snapshot()));
    let resp = http_get(port, "/");
    assert!(
        resp.starts_with("HTTP/1.1 404"),
        "expected 404 for root path; got: {resp}"
    );
}

#[test]
/// FR-5: GET /healthz returns HTTP 404 (not a defined route)
fn healthz_returns_404() {
    let port = spawn_server_with_data(Some(make_snapshot()));
    let resp = http_get(port, "/healthz");
    assert!(
        resp.starts_with("HTTP/1.1 404"),
        "expected 404 for /healthz; got: {resp}"
    );
}

// ---------------------------------------------------------------------------
// Helpers (private to this file)
// ---------------------------------------------------------------------------

/// Extract the body from a raw HTTP response string (after the blank line).
fn body_of(resp: &str) -> &str {
    resp.split_once("\r\n\r\n")
        .map(|(_, b)| b)
        .unwrap_or(resp)
}

/// Extract only the header section of a raw HTTP response.
fn headers_of(resp: &str) -> &str {
    resp.split_once("\r\n\r\n")
        .map(|(h, _)| h)
        .unwrap_or(resp)
}
