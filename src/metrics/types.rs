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
    pub available: bool,
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
    pub total_bytes: u64,
    pub used_bytes: u64,
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
        // Memory usage as fraction (0.0 to 1.0)
        if snapshot.memory.ram_total > 0 {
            Self::push_val(
                &mut self.mem_usage,
                snapshot.memory.ram_used as f64 / snapshot.memory.ram_total as f64,
                self.max_len,
            );
        }
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
