//! Pure data-preparation functions extracted from draw_* for testability.

use crate::metrics::{
    MemoryMetrics, NetInterface, PowerMetrics, ProcessInfo, SortMode, ThermalMetrics,
};

use super::helpers::is_infrastructure_interface;

/// A prepared row for the process table.
#[derive(Debug, Clone)]
pub(crate) struct ProcessRow {
    pub name: String,
    pub cpu_pct: f32,
    pub mem_display: String,
    pub power_w: f32,
    pub thread_count: i32,
    pub io_read: f64,
    pub io_write: f64,
    pub pid: i32,
    pub user: String,
    pub cpu_norm: f64,
}

/// A prepared component for the power breakdown.
#[derive(Debug, Clone)]
pub(crate) struct PowerComponent {
    pub name: &'static str,
    pub watts: f32,
}

/// A prepared row for the network interface table.
#[derive(Debug, Clone)]
pub(crate) struct NetworkRow {
    pub name: String,
    pub iface_type: String,
    pub baudrate: u64,
    pub tx_bytes_sec: f64,
    pub rx_bytes_sec: f64,
    pub packets_in_sec: f64,
}

/// Prepared memory pressure data for the stacked gauge.
#[derive(Debug, Clone)]
pub(crate) struct MemoryPressure {
    pub wired_frac: f64,
    pub app_frac: f64,
    pub compressed_frac: f64,
    pub wired_gb: f64,
    pub app_gb: f64,
    pub compressed_gb: f64,
}

const GB: f64 = 1024.0 * 1024.0 * 1024.0;
const MB: f64 = 1024.0 * 1024.0;

/// Prepare process rows: sort, scroll, truncate, format memory.
pub(crate) fn prepare_process_rows(
    procs: &[ProcessInfo],
    sort_mode: SortMode,
    scroll: usize,
    max_visible: usize,
    max_cpu: f32,
    max_mem: u64,
    max_power: f32,
) -> Vec<ProcessRow> {
    let mut indices: Vec<usize> = (0..procs.len()).collect();
    super::helpers::sort_indices(&mut indices, procs, sort_mode, max_cpu, max_mem, max_power);

    let scroll = scroll.min(indices.len().saturating_sub(1));

    indices
        .iter()
        .skip(scroll)
        .take(max_visible)
        .map(|&idx| {
            let p = &procs[idx];
            let mem_display = if p.mem_bytes as f64 >= GB {
                format!("{:.1}G", p.mem_bytes as f64 / GB)
            } else {
                format!("{:.0}M", p.mem_bytes as f64 / MB)
            };
            let cpu_norm = if max_cpu > 0.0 {
                (p.cpu_pct / max_cpu).clamp(0.0, 1.0) as f64
            } else {
                0.0
            };
            ProcessRow {
                name: p.name.clone(),
                cpu_pct: p.cpu_pct,
                mem_display,
                power_w: p.power_w,
                thread_count: p.thread_count,
                io_read: p.io_read_bytes_sec,
                io_write: p.io_write_bytes_sec,
                pid: p.pid,
                user: p.user.clone(),
                cpu_norm,
            }
        })
        .collect()
}

/// Prepare power component breakdown list.
pub(crate) fn prepare_power_components(
    power: &PowerMetrics,
    temperature: &ThermalMetrics,
) -> (Vec<PowerComponent>, Vec<u32>) {
    let components = vec![
        PowerComponent {
            name: "CPU",
            watts: power.cpu_w,
        },
        PowerComponent {
            name: "GPU",
            watts: power.gpu_w,
        },
        PowerComponent {
            name: "ANE",
            watts: power.ane_w,
        },
        PowerComponent {
            name: "DRAM",
            watts: power.dram_w,
        },
        PowerComponent {
            name: "system",
            watts: power.system_w,
        },
        PowerComponent {
            name: "package",
            watts: power.package_w,
        },
    ];
    (components, temperature.fan_speeds.clone())
}

/// Prepare network rows: filter infrastructure, sort by total traffic descending.
pub(crate) fn prepare_network_rows(interfaces: &[NetInterface]) -> Vec<NetworkRow> {
    let mut rows: Vec<NetworkRow> = interfaces
        .iter()
        .filter(|i| !is_infrastructure_interface(&i.name))
        .map(|i| NetworkRow {
            name: i.name.clone(),
            iface_type: i.iface_type.clone(),
            baudrate: i.baudrate,
            tx_bytes_sec: i.tx_bytes_sec,
            rx_bytes_sec: i.rx_bytes_sec,
            packets_in_sec: i.packets_in_sec,
        })
        .collect();

    rows.sort_by(|a, b| {
        let a_total = a.rx_bytes_sec + a.tx_bytes_sec;
        let b_total = b.rx_bytes_sec + b.tx_bytes_sec;
        b_total
            .partial_cmp(&a_total)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    rows
}

/// Prepare memory pressure fractions and absolute values.
pub(crate) fn prepare_memory_pressure(memory: &MemoryMetrics, ram_total_gb: f64) -> MemoryPressure {
    let total = ram_total_gb.max(0.01);
    let wired_gb = memory.wired as f64 / GB;
    let app_gb = memory.app as f64 / GB;
    let compressed_gb = memory.compressed as f64 / GB;

    MemoryPressure {
        wired_frac: (wired_gb / total).clamp(0.0, 1.0),
        app_frac: (app_gb / total).clamp(0.0, 1.0),
        compressed_frac: (compressed_gb / total).clamp(0.0, 1.0),
        wired_gb,
        app_gb,
        compressed_gb,
    }
}
