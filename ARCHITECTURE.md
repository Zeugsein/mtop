# Architecture

mtop follows a 4-layer architecture:

1. **Platform** (`src/platform/`) — macOS-specific data collection using Mach APIs, sysctl, IOKit, IOReport (dynamic), and SMC. Each subsystem (cpu, gpu, power, temperature, memory, network, disk, process, soc) is a separate module.

2. **Metrics** (`src/metrics/`) — Type definitions (`types.rs`) and the `Sampler` that orchestrates platform collectors at a configurable interval. Produces `MetricsSnapshot` structs. Includes history buffers for sparkline rendering.

3. **TUI** (`src/tui/`) — Terminal dashboard built with ratatui/crossterm. Panels for CPU bars, GPU gauge, power sparklines, temperature, memory gauges, network sparklines with interface ranking, and a weighted-score sortable process table. Supports expand/collapse, 10+ themes, braille-resolution sparklines, gradient coloring, and detail mode.

4. **Serve** (`src/serve/`) — Minimal HTTP server exposing JSON and Prometheus endpoints.

---

## Known limitations

- GPU utilization, power, and frequency require the IOReport private framework. On VMs or sandboxed environments without IOReport, these metrics gracefully degrade to N/A.
- Temperature requires SMC access via the AppleSMC IOKit service. Returns N/A when SMC is unavailable.
- CPU frequency falls back to nominal estimates from the chip model name when sysctl frequency data is not available.
- Process CPU percentage uses a heuristic based on running thread count rather than precise per-process CPU time deltas.
