use crate::metrics::{MetricsSnapshot, SocInfo};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};

pub type SharedMetrics = Arc<RwLock<Option<MetricsSnapshot>>>;

pub fn run(port: u16, bind: &str, shared: SharedMetrics, soc: &SocInfo) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{bind}:{port}");
    let listener = TcpListener::bind(&addr)?;
    eprintln!("mtop serve listening on http://{addr}");
    eprintln!("  GET /json    — JSON metrics snapshot");
    eprintln!("  GET /metrics — Prometheus text format");

    let soc = soc.clone();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let shared = Arc::clone(&shared);
                let soc = soc.clone();
                std::thread::spawn(move || {
                    process_request(stream, &shared, &soc);
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

fn to_prometheus(m: &MetricsSnapshot, soc: &SocInfo) -> String {
    let chip = &soc.chip;
    let l = format!(r#"chip="{chip}""#);

    let mut out = String::new();

    macro_rules! gauge {
        ($name:literal, $help:literal, $value:expr) => {
            out.push_str(&format!(
                "# HELP {0} {1}\n# TYPE {0} gauge\n{0}{{{l}}} {2}\n\n",
                $name, $help, $value
            ));
        };
        ($name:literal, $help:literal, $value:expr, $labels:expr) => {
            out.push_str(&format!(
                "# HELP {0} {1}\n# TYPE {0} gauge\n{0}{{{l},{3}}} {2}\n\n",
                $name, $help, $value, $labels
            ));
        };
    }

    // CPU
    gauge!("mtop_cpu_usage_ratio", "Combined CPU utilization (0-1)", m.cpu.total_usage);
    gauge!("mtop_cpu_usage_ratio", "E-cluster CPU utilization", m.cpu.e_cluster.usage, r#"cluster="efficiency""#);
    gauge!("mtop_cpu_usage_ratio", "P-cluster CPU utilization", m.cpu.p_cluster.usage, r#"cluster="performance""#);
    gauge!("mtop_cpu_freq_mhz", "E-cluster frequency MHz", m.cpu.e_cluster.freq_mhz, r#"cluster="efficiency""#);
    gauge!("mtop_cpu_freq_mhz", "P-cluster frequency MHz", m.cpu.p_cluster.freq_mhz, r#"cluster="performance""#);

    // GPU
    gauge!("mtop_gpu_usage_ratio", "GPU utilization (0-1)", m.gpu.usage);
    gauge!("mtop_gpu_freq_mhz", "GPU frequency MHz", m.gpu.freq_mhz);

    // Power
    gauge!("mtop_power_watts", "CPU power", m.power.cpu_w, r#"component="cpu""#);
    gauge!("mtop_power_watts", "GPU power", m.power.gpu_w, r#"component="gpu""#);
    gauge!("mtop_power_watts", "ANE power", m.power.ane_w, r#"component="ane""#);
    gauge!("mtop_power_watts", "DRAM power", m.power.dram_w, r#"component="dram""#);
    gauge!("mtop_power_watts", "Package power", m.power.package_w, r#"component="package""#);
    gauge!("mtop_power_watts", "System power", m.power.system_w, r#"component="system""#);

    // Temperature
    gauge!("mtop_temperature_celsius", "CPU avg temp", m.temperature.cpu_avg_c, r#"sensor="cpu_avg""#);
    gauge!("mtop_temperature_celsius", "GPU avg temp", m.temperature.gpu_avg_c, r#"sensor="gpu_avg""#);

    // Memory
    gauge!("mtop_memory_bytes", "RAM total", m.memory.ram_total, r#"type="ram_total""#);
    gauge!("mtop_memory_bytes", "RAM used", m.memory.ram_used, r#"type="ram_used""#);
    gauge!("mtop_memory_bytes", "Swap total", m.memory.swap_total, r#"type="swap_total""#);
    gauge!("mtop_memory_bytes", "Swap used", m.memory.swap_used, r#"type="swap_used""#);

    // Network
    for iface in &m.network.interfaces {
        let il = format!(r#"interface="{}",direction="rx""#, iface.name);
        out.push_str(&format!(
            "mtop_network_bytes_per_second{{{l},{il}}} {}\n",
            iface.rx_bytes_sec
        ));
        let il = format!(r#"interface="{}",direction="tx""#, iface.name);
        out.push_str(&format!(
            "mtop_network_bytes_per_second{{{l},{il}}} {}\n",
            iface.tx_bytes_sec
        ));
    }

    out
}
