use crate::metrics::types::*;
use crate::platform;

pub struct Sampler {
    soc: SocInfo,
    host_port: u32,
    cpu_ticks: Vec<(u64, u64)>,
    net_state: platform::network::NetworkState,
    mem_state: platform::memory::MemoryState,
    disk_state: platform::disk::DiskState,
    proc_cpu_state: platform::process::ProcessCpuState,
    gpu_state: Option<platform::gpu::GpuState>,
    power_state: Option<platform::power::PowerState>,
    temp_state: Option<platform::temperature::TemperatureState>,
}

impl Sampler {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let soc = platform::soc::detect_soc();
        // SAFETY: mach_host_self() returns the host port for the current task; always succeeds.
        let host_port = unsafe { mach_host_self() };
        Ok(Self {
            soc,
            host_port,
            cpu_ticks: Vec::new(),
            net_state: platform::network::NetworkState::new(),
            mem_state: platform::memory::MemoryState::new(),
            disk_state: platform::disk::DiskState::new(),
            proc_cpu_state: platform::process::ProcessCpuState::new(),
            gpu_state: platform::gpu::GpuState::new(),
            power_state: platform::power::PowerState::new(),
            temp_state: platform::temperature::TemperatureState::new(),
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
        let mut gpu = match &mut self.gpu_state {
            Some(state) => state.collect(),
            None => platform::gpu::collect_gpu(),
        };
        let power = match &mut self.power_state {
            Some(state) => state.collect(),
            None => platform::power::collect_power(),
        };
        let temperature = match &self.temp_state {
            Some(state) => state.collect(),
            None => platform::temperature::collect_temperature(),
        };
        let memory = self.mem_state.collect(self.host_port);
        let network = self.net_state.collect();
        let processes = platform::process::collect_processes(&mut self.proc_cpu_state);
        let battery = platform::battery::collect_battery();

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
            battery,
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

        out.push_str("\nSensor Status:\n");
        out.push_str(&format!("  GPU (IOReport): {}\n",
            if self.gpu_state.is_some() { "active" } else { "unavailable" }));
        out.push_str(&format!("  Power (IOReport): {}\n",
            if self.power_state.is_some() { "active" } else { "unavailable" }));
        out.push_str(&format!("  Temperature (SMC): {}\n",
            if self.temp_state.is_some() { "active" } else { "unavailable" }));

        if platform::ioreport_ffi::get_ioreport().is_some() {
            out.push_str("\nIOReport Channels:\n");
            out.push_str("  GPU group: \"GPU Stats\" / \"GPU Performance States\"\n");
            out.push_str("  Power group: \"Energy Model\"\n");
        }

        // Live SMC key enumeration
        if let Some(ref temp) = self.temp_state {
            let (cpu_keys, gpu_keys, ssd_keys, battery_keys) = platform::temperature::smc_enumerate_temp_keys(temp.conn());
            if !cpu_keys.is_empty() || !gpu_keys.is_empty() {
                out.push_str("\nSMC Keys (discovered):\n");
                out.push_str(&format!("  CPU: {}\n", if cpu_keys.is_empty() { "none".to_string() } else { cpu_keys.join(", ") }));
                out.push_str(&format!("  GPU: {}\n", if gpu_keys.is_empty() { "none".to_string() } else { gpu_keys.join(", ") }));
                out.push_str(&format!("  SSD: {}\n", if ssd_keys.is_empty() { "none".to_string() } else { ssd_keys.join(", ") }));
                out.push_str(&format!("  Battery: {}\n", if battery_keys.is_empty() { "none".to_string() } else { battery_keys.join(", ") }));
            } else {
                out.push_str("\nSMC Keys (static fallback):\n");
                out.push_str("  CPU: TC0P, TC0C, TC1C, TC2C, TC0F, Tp09, Tp0T, Tp01, Tp02, Te01, Te02\n");
                out.push_str("  GPU: TG0P, TG0D, TG1D, Tg05, Tg0f, Tg0j\n");
            }
        } else {
            out.push_str("\nSMC: unavailable\n");
        }

        out
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        // SAFETY: host_port was obtained from mach_host_self() in new(); deallocating it
        // releases the send right. mach_task_self() always returns the current task port.
        unsafe {
            mach_port_deallocate(mach_task_self(), self.host_port);
        }
    }
}

unsafe extern "C" {
    fn mach_host_self() -> u32;
    fn mach_task_self() -> u32;
    fn mach_port_deallocate(task: u32, name: u32) -> i32;
}
