use serde::Serialize;
use std::collections::VecDeque;

/// O(1) ring buffer wrapping VecDeque, exposing Vec-compatible interface.
pub struct HistoryBuffer(VecDeque<f64>);

impl HistoryBuffer {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    pub fn push_back(&mut self, val: f64) {
        self.0.push_back(val);
    }

    pub fn pop_front(&mut self) -> Option<f64> {
        self.0.pop_front()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Equivalent to Vec::last() — returns the back element.
    pub fn last(&self) -> Option<&f64> {
        self.0.back()
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, f64> {
        self.0.iter()
    }
}

impl Default for HistoryBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Index<usize> for HistoryBuffer {
    type Output = f64;
    fn index(&self, index: usize) -> &f64 {
        &self.0[index]
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SocInfo {
    pub chip: String,
    pub e_cores: u32,
    pub p_cores: u32,
    pub gpu_cores: u32,
    pub memory_gb: u32,
}

impl SocInfo {
    /// Estimated CPU TDP in watts for sparkline scaling.
    pub fn cpu_tdp_w(&self) -> f32 {
        estimate_cpu_tdp(&self.chip)
    }

    /// Estimated GPU TDP in watts for sparkline scaling.
    pub fn gpu_tdp_w(&self) -> f32 {
        estimate_gpu_tdp(&self.chip)
    }
}

#[allow(clippy::if_same_then_else)]
fn estimate_cpu_tdp(chip: &str) -> f32 {
    let lower = chip.to_lowercase();
    if lower.contains("ultra") {
        if lower.contains("m4") { 60.0 }
        else if lower.contains("m3") { 50.0 }
        else if lower.contains("m2") { 50.0 }
        else { 40.0 } // M1 Ultra
    } else if lower.contains("max") {
        if lower.contains("m4") { 30.0 }
        else if lower.contains("m3") { 25.0 }
        else if lower.contains("m2") { 20.0 }
        else { 20.0 } // M1 Max
    } else if lower.contains("pro") {
        if lower.contains("m4") { 25.0 }
        else if lower.contains("m3") { 20.0 }
        else if lower.contains("m2") { 20.0 }
        else { 20.0 } // M1 Pro
    } else {
        // Base chips (M1, M2, M3, M4)
        if lower.contains("m4") { 12.0 }
        else if lower.contains("m3") { 10.0 }
        else if lower.contains("m2") { 10.0 }
        else if lower.contains("m1") { 10.0 }
        else { 30.0 } // Unknown — conservative fallback
    }
}

#[allow(clippy::if_same_then_else)]
fn estimate_gpu_tdp(chip: &str) -> f32 {
    let lower = chip.to_lowercase();
    if lower.contains("ultra") {
        if lower.contains("m4") { 100.0 }
        else if lower.contains("m3") { 90.0 }
        else if lower.contains("m2") { 76.0 }
        else { 64.0 }
    } else if lower.contains("max") {
        if lower.contains("m4") { 50.0 }
        else if lower.contains("m3") { 45.0 }
        else if lower.contains("m2") { 40.0 }
        else { 32.0 }
    } else if lower.contains("pro") {
        if lower.contains("m4") { 20.0 }
        else if lower.contains("m3") { 15.0 }
        else if lower.contains("m2") { 15.0 }
        else { 12.0 }
    } else {
        if lower.contains("m4") { 10.0 }
        else if lower.contains("m3") { 10.0 }
        else if lower.contains("m2") { 8.0 }
        else if lower.contains("m1") { 8.0 }
        else { 30.0 } // Unknown — conservative fallback
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CoreClusterMetrics {
    pub freq_mhz: u32,
    pub usage: f32,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CpuMetrics {
    pub e_cluster: CoreClusterMetrics,
    pub p_cluster: CoreClusterMetrics,
    pub total_usage: f32,
    pub core_usages: Vec<f32>,
    pub power_w: f32,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GpuMetrics {
    pub freq_mhz: u32,
    pub usage: f32,
    pub power_w: f32,
    pub available: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct PowerMetrics {
    pub cpu_w: f32,
    pub gpu_w: f32,
    pub ane_w: f32,
    pub dram_w: f32,
    pub package_w: f32,
    pub system_w: f32,
    pub available: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ThermalMetrics {
    pub cpu_avg_c: f32,
    pub gpu_avg_c: f32,
    pub ssd_avg_c: f32,
    pub battery_avg_c: f32,
    pub fan_speeds: Vec<u32>,
    pub available: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct MemoryMetrics {
    pub ram_total: u64,
    pub ram_used: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub wired: u64,
    pub app: u64,
    pub compressed: u64,
    pub swap_in_bytes_sec: f64,
    pub swap_out_bytes_sec: f64,
    /// Memory pressure level: 1=normal, 2=warning, 4=critical
    pub pressure_level: u8,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct NetInterface {
    pub name: String,
    pub iface_type: String,
    pub rx_bytes_sec: f64,
    pub tx_bytes_sec: f64,
    pub baudrate: u64,
    pub packets_in_sec: f64,
    pub packets_out_sec: f64,
    pub rx_bytes_total: u64,
    pub tx_bytes_total: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct NetworkMetrics {
    pub interfaces: Vec<NetInterface>,
    pub primary_baudrate: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DiskMetrics {
    pub read_bytes_sec: u64,
    pub write_bytes_sec: u64,
    pub total_bytes: u64,
    pub used_bytes: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub enum SortMode {
    #[default]
    WeightedScore,
    Cpu,
    Memory,
    Power,
    Pid,
    Name,
}

impl SortMode {
    pub fn next(self) -> Self {
        match self {
            SortMode::WeightedScore => SortMode::Cpu,
            SortMode::Cpu => SortMode::Memory,
            SortMode::Memory => SortMode::Power,
            SortMode::Power => SortMode::Pid,
            SortMode::Pid => SortMode::Name,
            SortMode::Name => SortMode::WeightedScore,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SortMode::WeightedScore => "Score",
            SortMode::Cpu => "CPU%",
            SortMode::Memory => "Mem",
            SortMode::Power => "Power",
            SortMode::Pid => "PID",
            SortMode::Name => "Name",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ProcessInfo {
    pub pid: i32,
    pub name: String,
    pub cpu_pct: f32,
    pub mem_bytes: u64,
    pub energy_nj: u64,
    pub power_w: f32,
    pub user: String,
    pub thread_count: i32,
    pub io_read_bytes_sec: f64,
    pub io_write_bytes_sec: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct MetricsSnapshot {
    pub timestamp: String,
    pub soc: SocInfo,
    pub cpu: CpuMetrics,
    pub gpu: GpuMetrics,
    pub power: PowerMetrics,
    pub temperature: ThermalMetrics,
    pub memory: MemoryMetrics,
    pub network: NetworkMetrics,
    pub disk: DiskMetrics,
    pub processes: Vec<ProcessInfo>,
}

/// Rolling history buffer for sparkline data
pub struct MetricsHistory {
    pub cpu_usage: HistoryBuffer,
    pub gpu_usage: HistoryBuffer,
    pub cpu_power: HistoryBuffer,
    pub gpu_power: HistoryBuffer,
    pub ane_power: HistoryBuffer,
    pub dram_power: HistoryBuffer,
    pub package_power: HistoryBuffer,
    pub system_power: HistoryBuffer,
    pub mem_usage: HistoryBuffer,
    pub mem_available: HistoryBuffer,
    pub net_upload: HistoryBuffer,
    pub net_download: HistoryBuffer,
    /// Per-interface rx/tx history for per-interface sparklines
    pub per_iface: std::collections::HashMap<String, (HistoryBuffer, HistoryBuffer)>,
    /// Session maximum upload rate (bytes/sec)
    pub net_upload_max: f64,
    /// Session maximum download rate (bytes/sec)
    pub net_download_max: f64,
    max_len: usize,
}

impl Default for MetricsHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsHistory {
    pub fn new() -> Self {
        Self {
            cpu_usage: HistoryBuffer::new(),
            gpu_usage: HistoryBuffer::new(),
            cpu_power: HistoryBuffer::new(),
            gpu_power: HistoryBuffer::new(),
            ane_power: HistoryBuffer::new(),
            dram_power: HistoryBuffer::new(),
            package_power: HistoryBuffer::new(),
            system_power: HistoryBuffer::new(),
            mem_usage: HistoryBuffer::new(),
            mem_available: HistoryBuffer::new(),
            net_upload: HistoryBuffer::new(),
            net_download: HistoryBuffer::new(),
            per_iface: std::collections::HashMap::new(),
            net_upload_max: 0.0,
            net_download_max: 0.0,
            max_len: 128,
        }
    }

    pub fn push(&mut self, snapshot: &MetricsSnapshot) {
        Self::push_val(&mut self.cpu_usage, snapshot.cpu.total_usage as f64, self.max_len);
        if snapshot.gpu.available {
            Self::push_val(&mut self.gpu_usage, snapshot.gpu.usage as f64, self.max_len);
        }
        if snapshot.power.available {
            Self::push_val(&mut self.cpu_power, snapshot.power.cpu_w as f64, self.max_len);
            Self::push_val(&mut self.gpu_power, snapshot.power.gpu_w as f64, self.max_len);
            Self::push_val(&mut self.ane_power, snapshot.power.ane_w as f64, self.max_len);
            Self::push_val(&mut self.dram_power, snapshot.power.dram_w as f64, self.max_len);
            Self::push_val(&mut self.package_power, snapshot.power.package_w as f64, self.max_len);
            Self::push_val(&mut self.system_power, snapshot.power.system_w as f64, self.max_len);
        }
        // Memory usage and available as fractions (0.0 to 1.0)
        if snapshot.memory.ram_total > 0 {
            let total = snapshot.memory.ram_total as f64;
            Self::push_val(
                &mut self.mem_usage,
                snapshot.memory.ram_used as f64 / total,
                self.max_len,
            );
            let available = snapshot.memory.ram_total.saturating_sub(snapshot.memory.ram_used) as f64;
            Self::push_val(
                &mut self.mem_available,
                available / total,
                self.max_len,
            );
        }
        // Network aggregate rates (sum across all interfaces)
        let total_upload: f64 = snapshot.network.interfaces.iter()
            .map(|i| i.tx_bytes_sec)
            .sum();
        let total_download: f64 = snapshot.network.interfaces.iter()
            .map(|i| i.rx_bytes_sec)
            .sum();
        Self::push_val(&mut self.net_upload, total_upload, self.max_len);
        Self::push_val(&mut self.net_download, total_download, self.max_len);
        if total_upload > self.net_upload_max {
            self.net_upload_max = total_upload;
        }
        if total_download > self.net_download_max {
            self.net_download_max = total_download;
        }

        // Per-interface history (skip loopback)
        for iface in &snapshot.network.interfaces {
            if iface.name.starts_with("lo") {
                continue;
            }
            let (rx_buf, tx_buf) = self.per_iface
                .entry(iface.name.clone())
                .or_insert_with(|| (HistoryBuffer::new(), HistoryBuffer::new()));
            Self::push_val(rx_buf, iface.rx_bytes_sec, self.max_len);
            Self::push_val(tx_buf, iface.tx_bytes_sec, self.max_len);
        }
        // Prune interfaces not seen in this snapshot to bound memory
        self.per_iface.retain(|name, _| {
            snapshot.network.interfaces.iter().any(|i| i.name == *name)
        });
    }

    fn push_val(buf: &mut HistoryBuffer, val: f64, max: usize) {
        buf.push_back(val);
        if buf.len() > max {
            buf.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Network history buffer (iteration 6)
    // -------------------------------------------------------------------------

    fn make_net_snapshot(tx_en0: f64, rx_en0: f64, tx_en1: f64, rx_en1: f64) -> MetricsSnapshot {
        let mut snapshot = MetricsSnapshot::default();
        snapshot.network.interfaces.push(NetInterface {
            name: "en0".to_string(),
            iface_type: "ethernet".to_string(),
            rx_bytes_sec: rx_en0,
            tx_bytes_sec: tx_en0,
            ..Default::default()
        });
        snapshot.network.interfaces.push(NetInterface {
            name: "en1".to_string(),
            iface_type: "ethernet".to_string(),
            rx_bytes_sec: rx_en1,
            tx_bytes_sec: tx_en1,
            ..Default::default()
        });
        snapshot
    }

    #[test]
    /// Pushing 130 aggregate samples caps net_upload and net_download at 128 entries.
    fn net_history_caps_at_128_after_130_pushes() {
        let mut history = MetricsHistory::new();
        let snapshot = make_net_snapshot(500.0, 1_000.0, 300.0, 2_000.0);

        for _ in 0..130 {
            history.push(&snapshot);
        }

        assert_eq!(
            history.net_upload.len(),
            128,
            "net_upload buffer should cap at 128; got {}",
            history.net_upload.len()
        );
        assert_eq!(
            history.net_download.len(),
            128,
            "net_download buffer should cap at 128; got {}",
            history.net_download.len()
        );
    }

    #[test]
    /// First push produces correct aggregate: upload = sum of tx_bytes_sec,
    /// download = sum of rx_bytes_sec across all interfaces.
    fn net_history_first_sample_aggregates_interfaces() {
        let mut history = MetricsHistory::new();
        // en0: tx=500, rx=1000 | en1: tx=300, rx=2000
        let snapshot = make_net_snapshot(500.0, 1_000.0, 300.0, 2_000.0);

        history.push(&snapshot);

        let upload = *history.net_upload.last().expect("net_upload should have one entry");
        let download = *history.net_download.last().expect("net_download should have one entry");

        assert_eq!(
            upload, 800.0,
            "aggregate upload (tx) should be 500 + 300 = 800; got {upload}"
        );
        assert_eq!(
            download, 3_000.0,
            "aggregate download (rx) should be 1000 + 2000 = 3000; got {download}"
        );
    }

    // -------------------------------------------------------------------------
    // Existing TDP tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_cpu_tdp_m4_pro() {
        assert_eq!(estimate_cpu_tdp("Apple M4 Pro"), 25.0);
    }

    #[test]
    fn test_gpu_tdp_m1() {
        assert_eq!(estimate_gpu_tdp("Apple M1"), 8.0);
    }

    #[test]
    fn test_tdp_unknown_chip() {
        assert_eq!(estimate_cpu_tdp("Intel Core i9"), 30.0);
        assert_eq!(estimate_gpu_tdp("Intel Core i9"), 30.0);
    }

    #[test]
    fn test_soc_info_tdp_methods() {
        let soc = SocInfo {
            chip: "Apple M3 Max".to_string(),
            e_cores: 4, p_cores: 12, gpu_cores: 40, memory_gb: 36,
        };
        assert_eq!(soc.cpu_tdp_w(), 25.0);
        assert_eq!(soc.gpu_tdp_w(), 45.0);
    }
}
