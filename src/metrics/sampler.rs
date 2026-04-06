use crate::metrics::types::*;
use crate::platform;

pub struct Sampler {
    soc: SocInfo,
    host_port: u32,
    cpu_ticks: Vec<(u64, u64)>,
    net_state: platform::network::NetworkState,
    disk_state: platform::disk::DiskState,
    proc_cpu_state: platform::process::ProcessCpuState,
}

impl Sampler {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let soc = platform::soc::detect_soc();
        let host_port = unsafe { mach_host_self() };
        Ok(Self {
            soc,
            host_port,
            cpu_ticks: Vec::new(),
            net_state: platform::network::NetworkState::new(),
            disk_state: platform::disk::DiskState::new(),
            proc_cpu_state: platform::process::ProcessCpuState::new(),
        })
    }

    pub fn soc_info(&self) -> &SocInfo {
        &self.soc
    }

    pub fn sample(&mut self, interval_ms: u32) -> Result<MetricsSnapshot, Box<dyn std::error::Error>> {
        let interval = interval_ms.max(100);

        // Sleep for the interval
        std::thread::sleep(std::time::Duration::from_millis(interval as u64));

        let mut cpu = platform::cpu::collect_cpu(self.host_port, &mut self.cpu_ticks, self.soc.e_cores, self.soc.p_cores);
        let mut gpu = platform::gpu::collect_gpu();
        let power = platform::power::collect_power();
        let temperature = platform::temperature::collect_temperature();
        let memory = platform::memory::collect_memory(self.host_port);
        let network = self.net_state.collect();
        let processes = platform::process::collect_processes(&mut self.proc_cpu_state);

        // Cross-reference: CPU and GPU power from power module
        cpu.power_w = power.cpu_w;
        gpu.power_w = power.gpu_w;

        // CPU frequencies from sysctl (perflevel nominal frequencies as fallback)
        if cpu.e_cluster.freq_mhz == 0 {
            cpu.e_cluster.freq_mhz = platform::cpu::sysctl_cpu_freq(1); // perflevel1 = E-cores
        }
        if cpu.p_cluster.freq_mhz == 0 {
            cpu.p_cluster.freq_mhz = platform::cpu::sysctl_cpu_freq(0); // perflevel0 = P-cores
        }

        let timestamp = chrono::Utc::now().to_rfc3339();

        Ok(MetricsSnapshot {
            timestamp,
            soc: self.soc.clone(),
            cpu,
            gpu,
            power,
            temperature,
            memory,
            network,
            disk: self.disk_state.collect(),
            processes,
        })
    }

    #[allow(dead_code)]
    pub fn host_port(&self) -> u32 {
        self.host_port
    }

    pub fn debug_info(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("SoC: {}\n", self.soc.chip));
        out.push_str(&format!("E-cores: {}\n", self.soc.e_cores));
        out.push_str(&format!("P-cores: {}\n", self.soc.p_cores));
        out.push_str(&format!("GPU cores: {} (estimated)\n", self.soc.gpu_cores));
        out.push_str(&format!("Memory: {} GB\n", self.soc.memory_gb));
        out.push_str("\nIOReport FFI active for GPU, power, and temperature metrics.\n");
        out
    }
}

unsafe extern "C" {
    fn mach_host_self() -> u32;
}
