# mtop

A real-time system monitor for macOS on Apple Silicon. Displays CPU, GPU, power, temperature, memory, network, disk, and process metrics via a terminal dashboard, HTTP API, or streaming JSON pipe.

## Build

Requires Rust (stable) and macOS on Apple Silicon.

```sh
cargo build --release
```

## Usage

mtop has four operating modes:

### TUI dashboard (default)

```sh
mtop
mtop --interval 500 --color green --temp-unit fahrenheit
```

Interactive keyboard controls: `q` quit, `c` cycle theme, `s` cycle sort, `+`/`-` adjust interval, `j`/`k` scroll process list.

### Pipe mode (NDJSON)

```sh
mtop pipe --samples 10
mtop pipe  # infinite stream
```

Emits one JSON object per line per sample interval.

### HTTP API server

```sh
mtop serve --port 9090 --bind 127.0.0.1
```

Endpoints:
- `GET /json` -- full metrics snapshot as JSON
- `GET /metrics` -- Prometheus text exposition format

### Debug

```sh
mtop debug
```

Prints SoC detection info and diagnostic details.

## Architecture

mtop follows a 4-layer architecture:

1. **Platform** (`src/platform/`) -- macOS-specific data collection using Mach APIs, sysctl, IOKit, IOReport (dynamic), and SMC. Each subsystem (cpu, gpu, power, temperature, memory, network, disk, process, soc) is a separate module.

2. **Metrics** (`src/metrics/`) -- Type definitions (`types.rs`) and the `Sampler` that orchestrates platform collectors at a configurable interval. Produces `MetricsSnapshot` structs.

3. **TUI** (`src/tui/`) -- Terminal dashboard built with ratatui/crossterm. Renders CPU bars, power sparklines, temperature, memory gauges, network rates, and a sortable process table.

4. **Serve** (`src/serve/`) -- Minimal HTTP server exposing JSON and Prometheus endpoints.

## Known Limitations

- GPU utilization, power, and frequency require the IOReport private framework. On VMs or sandboxed environments without IOReport, these metrics gracefully degrade to zero.
- Temperature requires SMC access via the AppleSMC IOKit service. Returns zero when SMC is unavailable.
- CPU frequency falls back to nominal estimates from the chip model name when sysctl frequency data is not available.
- Process CPU percentage uses a heuristic based on running thread count rather than precise per-process CPU time deltas.

## License

MIT
