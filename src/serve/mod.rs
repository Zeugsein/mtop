use crate::metrics::{MetricsSnapshot, SocInfo};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

pub type SharedMetrics = Arc<RwLock<Option<MetricsSnapshot>>>;

const MAX_CONNECTIONS: usize = 64;

pub fn run(port: u16, bind: &str, shared: SharedMetrics, soc: &SocInfo) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{bind}:{port}");
    let listener = TcpListener::bind(&addr)?;
    eprintln!("mtop serve listening on http://{addr}");
    eprintln!("  GET /json    — JSON metrics snapshot");
    eprintln!("  GET /metrics — Prometheus text format");

    let soc = soc.clone();
    let active = Arc::new(AtomicUsize::new(0));

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
                let shared = Arc::clone(&shared);
                let soc = soc.clone();
                let active = Arc::clone(&active);
                std::thread::spawn(move || {
                    process_request(stream, &shared, &soc);
                    active.fetch_sub(1, Ordering::Release);
                });
            }
            Err(e) => eprintln!("connection error: {e}"),
        }
    }

    Ok(())
}

fn process_request(mut stream: TcpStream, shared: &SharedMetrics, soc: &SocInfo) {
    let path = match read_path(&mut stream) {
        Some(p) => p,
        None => return,
    };

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

fn read_path(stream: &mut TcpStream) -> Option<String> {
    stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).ok()?;
    let mut buf = [0u8; 2048];
    let n = stream.read(&mut buf).ok()?;
    let text = std::str::from_utf8(&buf[..n]).ok()?;
    let path = text.lines().next()?.split_whitespace().nth(1)?;
    Some(path.split('?').next().unwrap_or(path).to_string())
}

fn write_response(stream: &mut TcpStream, status: u16, content_type: &str, body: &str) {
    let status_text = match status {
        200 => "OK",
        404 => "Not Found",
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
