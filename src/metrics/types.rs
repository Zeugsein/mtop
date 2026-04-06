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
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct PowerMetrics {
    pub cpu_w: f32,
    pub gpu_w: f32,
    pub ane_w: f32,
    pub dram_w: f32,
    pub package_w: f32,
    pub system_w: f32,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ThermalMetrics {
    pub cpu_avg_c: f32,
    pub gpu_avg_c: f32,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct MemoryMetrics {
    pub ram_total: u64,
    pub ram_used: u64,
    pub swap_total: u64,
    pub swap_used: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct NetInterface {
    pub name: String,
    pub iface_type: String,
    pub rx_bytes_sec: f64,
    pub tx_bytes_sec: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct NetworkMetrics {
    pub interfaces: Vec<NetInterface>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DiskMetrics {
    pub read_bytes_sec: u64,
    pub write_bytes_sec: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ProcessInfo {
    pub pid: i32,
    pub name: String,
    pub cpu_pct: f32,
    pub mem_bytes: u64,
    pub user: String,
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
            max_len: 128,
        }
    }

    pub fn push(&mut self, snapshot: &MetricsSnapshot) {
        Self::push_val(&mut self.cpu_usage, snapshot.cpu.total_usage as f64, self.max_len);
        Self::push_val(&mut self.gpu_usage, snapshot.gpu.usage as f64, self.max_len);
        Self::push_val(&mut self.cpu_power, snapshot.power.cpu_w as f64, self.max_len);
        Self::push_val(&mut self.gpu_power, snapshot.power.gpu_w as f64, self.max_len);
        Self::push_val(&mut self.ane_power, snapshot.power.ane_w as f64, self.max_len);
        Self::push_val(&mut self.dram_power, snapshot.power.dram_w as f64, self.max_len);
        Self::push_val(&mut self.package_power, snapshot.power.package_w as f64, self.max_len);
        Self::push_val(&mut self.system_power, snapshot.power.system_w as f64, self.max_len);
    }

    fn push_val(buf: &mut HistoryBuffer, val: f64, max: usize) {
        buf.push_back(val);
        if buf.len() > max {
            buf.pop_front();
        }
    }
}
