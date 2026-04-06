use serde::Serialize;

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
    pub cpu_usage: Vec<f64>,
    pub gpu_usage: Vec<f64>,
    pub cpu_power: Vec<f64>,
    pub gpu_power: Vec<f64>,
    pub ane_power: Vec<f64>,
    pub dram_power: Vec<f64>,
    pub package_power: Vec<f64>,
    pub system_power: Vec<f64>,
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
            cpu_usage: Vec::new(),
            gpu_usage: Vec::new(),
            cpu_power: Vec::new(),
            gpu_power: Vec::new(),
            ane_power: Vec::new(),
            dram_power: Vec::new(),
            package_power: Vec::new(),
            system_power: Vec::new(),
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

    fn push_val(buf: &mut Vec<f64>, val: f64, max: usize) {
        buf.push(val);
        if buf.len() > max {
            buf.remove(0);
        }
    }
}
