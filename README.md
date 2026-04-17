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
mtop --interval 500 --color dracula --temp-unit fahrenheit
```

Launches an interactive terminal dashboard with a 6-panel layout: CPU (per-core bars), GPU (utilization gauge + frequency), memory/disk, network (sparklines + interface list), power (component sparklines), and a sortable process list.

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

## Panel layout

The dashboard arranges 6 panels in a two-column grid:

| Left column     | Right column    |
|-----------------|-----------------|
| CPU cores       | Network         |
| GPU             | Power           |
| Memory / Disk   | Process list    |

Each panel can be expanded to fill its column, showing additional detail (per-core breakdowns, full component tables, disk I/O, per-interface stats, etc.).

## Key bindings

| Key          | Action                                 |
|--------------|----------------------------------------|
| `q`          | Quit                                   |
| `Esc`        | Collapse expanded panel, or quit       |
| `1`-`6`      | Select panel (CPU, GPU, Mem, Net, Power, Process) |
| `e` / `Enter`| Toggle expand/collapse selected panel  |
| `s`          | Cycle process sort mode                |
| `w`          | Save current theme, interval, and sort to config |
| `c`          | Cycle color theme forward              |
| `C`          | Cycle color theme backward             |
| `j` / `Down` | Scroll process list down               |
| `k` / `Up`   | Scroll process list up                 |
| `t`          | Send SIGTERM to selected process       |
| `f`          | Filter process list by name            |
| `y` / `n`    | Confirm / cancel pending signal        |
| `+` / `-`    | Increase / decrease sample interval    |
| `.`          | Toggle detail mode                     |
| `h` / `?`    | Toggle help overlay                    |

Process sort modes cycle through: Score, CPU%, Memory, Power, PID, Name.

## Themes

mtop ships with 10+ built-in color themes. Cycle with `c`/`C`. Set a default in `~/.config/mtop/config.toml`:

```toml
theme = "dracula"
```

## Architecture

mtop follows a 4-layer architecture:

1. **Platform** (`src/platform/`) -- macOS-specific data collection using Mach APIs, sysctl, IOKit, IOReport (dynamic), and SMC. Each subsystem (cpu, gpu, power, temperature, memory, network, disk, process, soc) is a separate module.

2. **Metrics** (`src/metrics/`) -- Type definitions (`types.rs`) and the `Sampler` that orchestrates platform collectors at a configurable interval. Produces `MetricsSnapshot` structs. Includes history buffers for sparkline rendering.

3. **TUI** (`src/tui/`) -- Terminal dashboard built with ratatui/crossterm. Panels for CPU bars, GPU gauge, power sparklines, temperature, memory gauges, network sparklines with interface ranking, and a weighted-score sortable process table. Supports expand/collapse, 10+ themes, braille-resolution sparklines, gradient coloring, and detail mode.

4. **Serve** (`src/serve/`) -- Minimal HTTP server exposing JSON and Prometheus endpoints.

## Configuration

mtop reads `~/.config/mtop/config.toml` on startup. CLI flags override config values.

```toml
interval_ms = 1000
theme = "horizon"
temp_unit = "celsius"
```

## Known limitations

- GPU utilization, power, and frequency require the IOReport private framework. On VMs or sandboxed environments without IOReport, these metrics gracefully degrade to N/A.
- Temperature requires SMC access via the AppleSMC IOKit service. Returns N/A when SMC is unavailable.
- CPU frequency falls back to nominal estimates from the chip model name when sysctl frequency data is not available.
- Process CPU percentage uses a heuristic based on running thread count rather than precise per-process CPU time deltas.

## License

MIT
