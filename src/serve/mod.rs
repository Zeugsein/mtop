use crate::metrics::{MetricsSnapshot, SocInfo};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{IpAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use subtle::ConstantTimeEq;

pub type SharedMetrics = Arc<RwLock<Option<MetricsSnapshot>>>;

const MAX_CONNECTIONS: usize = 64;
const MAX_PER_IP: usize = 8;

pub fn run(
    port: u16,
    bind: &str,
    shared: SharedMetrics,
    soc: &SocInfo,
    last_request: Arc<AtomicU64>,
    token: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{bind}:{port}");
    let listener = TcpListener::bind(&addr)?;
    eprintln!("mtop serve listening on http://{addr}");
    eprintln!("  GET /json    — JSON metrics snapshot");
    eprintln!("  GET /metrics — Prometheus text format");

    let soc = soc.clone();
    let active = Arc::new(AtomicUsize::new(0));
    let per_ip: Arc<Mutex<HashMap<IpAddr, usize>>> = Arc::new(Mutex::new(HashMap::new()));
    let token = Arc::new(token);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let prev = active.fetch_add(1, Ordering::AcqRel);
                if prev >= MAX_CONNECTIONS {
                    active.fetch_sub(1, Ordering::Release);
                    let mut s = stream;
                    write_response(&mut s, 503, "text/plain", "too many connections\n");
                    continue;
                }

                let peer_ip = match stream.peer_addr() {
                    Ok(addr) => addr.ip(),
                    Err(_) => {
                        active.fetch_sub(1, Ordering::Release);
                        continue;
                    }
                };

                // Atomically check and increment per-IP count (single lock scope)
                {
                    let mut counts = per_ip.lock().unwrap_or_else(|e| e.into_inner());
                    let count = counts.entry(peer_ip).or_insert(0);
                    if *count >= MAX_PER_IP {
                        active.fetch_sub(1, Ordering::Release);
                        let mut s = stream;
                        write_response(&mut s, 429, "text/plain", "too many connections from your IP\n");
                        continue;
                    }
                    *count += 1;
                }

                let shared = Arc::clone(&shared);
                let soc = soc.clone();
                let active = Arc::clone(&active);
                let per_ip = Arc::clone(&per_ip);
                let last_request = Arc::clone(&last_request);
                let token = Arc::clone(&token);
                std::thread::spawn(move || {
                    process_request(stream, &shared, &soc, &last_request, &token);
                    active.fetch_sub(1, Ordering::Release);
                    let mut counts = per_ip.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(c) = counts.get_mut(&peer_ip) {
                        *c = c.saturating_sub(1);
                        if *c == 0 {
                            counts.remove(&peer_ip);
                        }
                    }
                });
            }
            Err(e) => eprintln!("connection error: {e}"),
        }
    }

    Ok(())
}

fn process_request(
    mut stream: TcpStream,
    shared: &SharedMetrics,
    soc: &SocInfo,
    last_request: &Arc<AtomicU64>,
    token: &Arc<Option<String>>,
) {
    // Update last-request timestamp (S4)
    last_request.store(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        Ordering::Relaxed,
    );

    let (path, auth_header) = match read_path_and_auth(&mut stream) {
        Some(p) => p,
        None => return,
    };

    // Bearer token check (S5)
    if let Some(ref expected) = **token {
        let provided = auth_header
            .as_deref()
            .and_then(|h| h.strip_prefix("Bearer "))
            .unwrap_or("");
        let expected_bytes = expected.as_bytes();
        let provided_bytes = provided.as_bytes();
        // Constant-time compare; if lengths differ pad to avoid short-circuit
        let lengths_match = provided_bytes.len() == expected_bytes.len();
        let dummy = expected_bytes; // same length as expected for dummy compare
        let compare_against = if lengths_match { provided_bytes } else { dummy };
        let ok = compare_against.ct_eq(expected_bytes).unwrap_u8() == 1 && lengths_match;
        if !ok {
            let body = b"";
            let _ = stream.write_all(
                format!(
                    "HTTP/1.1 401 Unauthorized\r\nWWW-Authenticate: Bearer\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                )
                .as_bytes(),
            );
            let _ = body; // suppress unused warning
            return;
        }
    }

    let metrics = match shared.read() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    };

    match path.as_str() {
        "/json" => {
            match metrics {
                Some(m) => {
                    let body = serde_json::to_string_pretty(&m).unwrap_or_default();
                    write_response(&mut stream, 200, "application/json", &body);
                }
                None => {
                    write_response(&mut stream, 503, "application/json", r#"{"error":"no data yet"}"#);
                }
            }
        }
        "/metrics" => {
            match metrics {
                Some(m) => {
                    let body = to_prometheus(&m, soc);
                    write_response(&mut stream, 200, "text/plain; version=0.0.4", &body);
                }
                None => {
                    write_response(&mut stream, 503, "text/plain", "# no data yet\n");
                }
            }
        }
        _ => {
            write_response(&mut stream, 404, "text/plain", "not found\n");
        }
    }
}

/// Read the HTTP request path and Authorization header from the stream.
/// NOTE: Single-read design is an accepted residual Slowloris risk.
/// Primary defense is the per-IP connection limit (MAX_PER_IP) + 2s timeout.
/// A slow sender can hold one connection for up to 2s before timeout fires.
/// With MAX_PER_IP=8, an attacker can occupy at most 8 slots per IP.
fn read_path_and_auth(stream: &mut TcpStream) -> Option<(String, Option<String>)> {
    stream.set_read_timeout(Some(std::time::Duration::from_secs(2))).ok()?;
    stream.set_write_timeout(Some(std::time::Duration::from_secs(2))).ok()?;
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf).ok()?;
    let text = std::str::from_utf8(&buf[..n]).ok()?;
    let mut lines = text.lines();
    let request_line = lines.next()?;
    let path = request_line.split_whitespace().nth(1)?;
    let path = path.split('?').next().unwrap_or(path).to_string();

    // Scan remaining lines for Authorization header
    let mut auth: Option<String> = None;
    for line in lines {
        if line.is_empty() {
            break; // end of headers
        }
        let lower = line.to_lowercase();
        if lower.starts_with("authorization:") {
            auth = Some(line[14..].trim().to_string());
        }
    }

    Some((path, auth))
}

fn write_response(stream: &mut TcpStream, status: u16, content_type: &str, body: &str) {
    let status_text = match status {
        200 => "OK",
        401 => "Unauthorized",
        404 => "Not Found",
        429 => "Too Many Requests",
        503 => "Service Unavailable",
        _ => "OK",
    };
    let _ = stream.write_all(
        format!(
            "HTTP/1.1 {status} {status_text}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .as_bytes(),
    );
}

fn escape_label_value(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn to_prometheus(m: &MetricsSnapshot, soc: &SocInfo) -> String {
    let chip = escape_label_value(&soc.chip);
    let l = format!(r#"chip="{chip}""#);

    let mut out = String::new();

    // CPU usage ratio
    out.push_str("# HELP mtop_cpu_usage_ratio CPU utilization ratio (0-1)\n");
    out.push_str("# TYPE mtop_cpu_usage_ratio gauge\n");
    out.push_str(&format!("mtop_cpu_usage_ratio{{{l}}} {}\n", m.cpu.total_usage));
    out.push_str(&format!("mtop_cpu_usage_ratio{{{l},cluster=\"efficiency\"}} {}\n", m.cpu.e_cluster.usage));
    out.push_str(&format!("mtop_cpu_usage_ratio{{{l},cluster=\"performance\"}} {}\n", m.cpu.p_cluster.usage));
    out.push('\n');

    // CPU frequency
    out.push_str("# HELP mtop_cpu_freq_mhz CPU cluster frequency in MHz\n");
    out.push_str("# TYPE mtop_cpu_freq_mhz gauge\n");
    out.push_str(&format!("mtop_cpu_freq_mhz{{{l},cluster=\"efficiency\"}} {}\n", m.cpu.e_cluster.freq_mhz));
    out.push_str(&format!("mtop_cpu_freq_mhz{{{l},cluster=\"performance\"}} {}\n", m.cpu.p_cluster.freq_mhz));
    out.push('\n');

    // GPU
    out.push_str("# HELP mtop_gpu_usage_ratio GPU utilization ratio (0-1)\n");
    out.push_str("# TYPE mtop_gpu_usage_ratio gauge\n");
    out.push_str(&format!("mtop_gpu_usage_ratio{{{l}}} {}\n", m.gpu.usage));
    out.push('\n');

    out.push_str("# HELP mtop_gpu_freq_mhz GPU frequency in MHz\n");
    out.push_str("# TYPE mtop_gpu_freq_mhz gauge\n");
    out.push_str(&format!("mtop_gpu_freq_mhz{{{l}}} {}\n", m.gpu.freq_mhz));
    out.push('\n');

    // Power
    out.push_str("# HELP mtop_power_watts Power consumption in watts\n");
    out.push_str("# TYPE mtop_power_watts gauge\n");
    out.push_str(&format!("mtop_power_watts{{{l},component=\"cpu\"}} {}\n", m.power.cpu_w));
    out.push_str(&format!("mtop_power_watts{{{l},component=\"gpu\"}} {}\n", m.power.gpu_w));
    out.push_str(&format!("mtop_power_watts{{{l},component=\"ane\"}} {}\n", m.power.ane_w));
    out.push_str(&format!("mtop_power_watts{{{l},component=\"dram\"}} {}\n", m.power.dram_w));
    out.push_str(&format!("mtop_power_watts{{{l},component=\"package\"}} {}\n", m.power.package_w));
    out.push_str(&format!("mtop_power_watts{{{l},component=\"system\"}} {}\n", m.power.system_w));
    out.push('\n');

    // Temperature
    out.push_str("# HELP mtop_temperature_celsius Temperature in degrees Celsius\n");
    out.push_str("# TYPE mtop_temperature_celsius gauge\n");
    out.push_str(&format!("mtop_temperature_celsius{{{l},sensor=\"cpu_avg\"}} {}\n", m.temperature.cpu_avg_c));
    out.push_str(&format!("mtop_temperature_celsius{{{l},sensor=\"gpu_avg\"}} {}\n", m.temperature.gpu_avg_c));
    out.push('\n');

    // Memory
    out.push_str("# HELP mtop_memory_bytes Memory in bytes\n");
    out.push_str("# TYPE mtop_memory_bytes gauge\n");
    out.push_str(&format!("mtop_memory_bytes{{{l},type=\"ram_total\"}} {}\n", m.memory.ram_total));
    out.push_str(&format!("mtop_memory_bytes{{{l},type=\"ram_used\"}} {}\n", m.memory.ram_used));
    out.push_str(&format!("mtop_memory_bytes{{{l},type=\"swap_total\"}} {}\n", m.memory.swap_total));
    out.push_str(&format!("mtop_memory_bytes{{{l},type=\"swap_used\"}} {}\n", m.memory.swap_used));
    out.push('\n');

    // Network
    out.push_str("# HELP mtop_network_bytes_per_second Network throughput in bytes per second\n");
    out.push_str("# TYPE mtop_network_bytes_per_second gauge\n");
    for iface in &m.network.interfaces {
        let iname = escape_label_value(&iface.name);
        out.push_str(&format!(
            "mtop_network_bytes_per_second{{{l},interface=\"{iname}\",direction=\"rx\"}} {}\n",
            iface.rx_bytes_sec
        ));
        out.push_str(&format!(
            "mtop_network_bytes_per_second{{{l},interface=\"{iname}\",direction=\"tx\"}} {}\n",
            iface.tx_bytes_sec
        ));
    }
    out.push('\n');

    out
}
