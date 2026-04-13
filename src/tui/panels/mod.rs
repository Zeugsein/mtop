mod cpu;
mod gpu;
mod memory;
mod network;
mod power;
mod process;

pub(crate) use cpu::draw_cpu_panel_v2;
pub(crate) use cpu::render_graph;
pub(crate) use gpu::draw_gpu_panel_v2;
pub(crate) use memory::draw_mem_disk_panel_v2;
pub(crate) use network::draw_network_panel_v2;
pub(crate) use network::NET_TIERS;
pub(crate) use power::draw_power_panel_v2;
pub(crate) use process::draw_process_panel_v2;
