## 1. Project Setup

- [ ] 1.1 Initialize Cargo project with binary and library targets
- [ ] 1.2 Add dependencies: ratatui, crossterm, clap, core-foundation, libc, serde, serde_json, chrono
- [ ] 1.3 Set up module structure: platform/, metrics/, tui/, serve/, cli

## 2. Platform Layer (macOS FFI)

- [ ] 2.1 Implement IOReport FFI bindings: channel subscription, sampling, delta computation
- [ ] 2.2 Implement SMC FFI bindings: key reading for temperature and power rails
- [ ] 2.3 Implement Mach API wrappers: host_processor_info (CPU ticks), host_statistics64 (memory)
- [ ] 2.4 Implement sysctl wrappers: hw.memsize, vm.swapusage, hw.model, machdep.cpu
- [ ] 2.5 Implement getifaddrs wrapper for network interface byte counters
- [ ] 2.6 Implement SoC detection: chip name, core counts (E/P/GPU), total memory

## 3. Metrics Collection

- [ ] 3.1 Define Metrics, CpuMetrics, GpuMetrics, PowerMetrics, TempMetrics, MemMetrics, NetworkMetrics, DiskMetrics, ProcessInfo structs with serde Serialize
- [ ] 3.2 Implement Sampler that orchestrates collection from all platform sources
- [ ] 3.3 Implement CPU metrics: per-core utilization from tick deltas, per-cluster frequency from DVFS residency, power from IOReport Energy Model
- [ ] 3.4 Implement GPU metrics: utilization, frequency, power from IOReport
- [ ] 3.5 Implement power breakdown: CPU, GPU, ANE, DRAM, package total, system total
- [ ] 3.6 Implement temperature: SMC sensor reading with HID fallback, CPU/GPU averages
- [ ] 3.7 Implement memory: RAM total/used via host_statistics64, swap via sysctl
- [ ] 3.8 Implement network: per-interface byte counter deltas via getifaddrs
- [ ] 3.9 Implement disk I/O: read/write byte rate computation
- [ ] 3.10 Implement process list: sysctl/proc_taskinfo for PID, name, CPU%, memory, user
- [ ] 3.11 Implement MetricsHistory: rolling 128-point buffer for sparkline data
- [ ] 3.12 Implement graceful degradation: skip unavailable sensors without crashing

## 4. CLI Interface

- [ ] 4.1 Define clap CLI struct with subcommands: (default TUI), pipe, serve, debug
- [ ] 4.2 Implement global options: --interval, --color, --temp-unit, --version, --help
- [ ] 4.3 Implement pipe-specific options: --samples
- [ ] 4.4 Implement serve-specific options: --port
- [ ] 4.5 Wire subcommand routing in main()

## 5. TUI Dashboard

- [ ] 5.1 Implement terminal setup/teardown with panic hook (enter/leave alternate screen, raw mode)
- [ ] 5.2 Implement main TUI event loop: keyboard input + render cycle
- [ ] 5.3 Implement dashboard layout: CPU panel (left), power+temp+memory+network (right), process list (bottom)
- [ ] 5.4 Implement CPU core bar widget: per-core horizontal bars with E/P labels, color coding by utilization
- [ ] 5.5 Implement power sparkline widget: mini-charts for CPU, GPU, ANE, DRAM, package, system power
- [ ] 5.6 Implement GPU gauge widget: utilization bar + frequency + power
- [ ] 5.7 Implement temperature display widget: CPU avg, GPU avg in configured unit
- [ ] 5.8 Implement memory bar widget: RAM and swap usage bars with labels
- [ ] 5.9 Implement network rate display widget: upload/download with auto-scaled units
- [ ] 5.10 Implement process list table: sortable by CPU%, memory, PID, name
- [ ] 5.11 Implement SoC info header: chip name, core config, memory
- [ ] 5.12 Implement keyboard handler: q/Esc=quit, arrows/jk=navigate, s=sort, +/-=interval, c=theme
- [ ] 5.13 Implement 3 color themes with cycling
- [ ] 5.14 Implement terminal resize handling

## 6. API Server & Pipe Mode

- [ ] 6.1 Implement pipe mode: NDJSON output to stdout with --samples support
- [ ] 6.2 Implement HTTP server: raw TCP listener, parse GET path
- [ ] 6.3 Implement /json endpoint: current metrics as JSON with timestamp and soc info
- [ ] 6.4 Implement /metrics endpoint: Prometheus text format with mtop_ prefix and labels
- [ ] 6.5 Implement 404 handling for unknown routes
- [ ] 6.6 Wire server with shared metrics store (Arc<RwLock>)

## 7. Debug & Polish

- [ ] 7.1 Implement debug subcommand: print chip info, SMC keys, IOReport channels, sensor availability
- [ ] 7.2 Add interval clamping (minimum 100ms)
- [ ] 7.3 Test TUI at 80x24 minimum terminal size
- [ ] 7.4 Test pipe mode output with jq
- [ ] 7.5 Test HTTP endpoints with curl
- [ ] 7.6 Run forbidden-tokens grep against all implementation source files
