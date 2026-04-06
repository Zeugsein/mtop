## Why

mtop is a new system monitoring tool for macOS (Apple Silicon). There is currently no single tool that provides all three presentation modes — terminal TUI, HTTP API, and menu bar widget — from a unified metrics collection core. Existing tools each excel in one area but lack the others. mtop fills this gap.

## What Changes

- New Rust binary `mtop` with 3 subcommands: TUI (default), `pipe`, `serve`
- Core metrics collection layer using macOS system APIs (IOReport, SMC, Mach, sysctl) — no sudo required
- 7 monitoring targets: CPU, memory, network, disk I/O, GPU, temperature, power consumption
- Multi-panel terminal dashboard with per-core CPU bars, power sparklines, process list
- NDJSON pipe mode for scripting integration
- HTTP server with JSON and Prometheus-format endpoints
- Menu Bar mode deferred to post-MVP

## Capabilities

### New Capabilities
- `metrics-collection`: Unified metrics gathering from macOS hardware APIs — CPU (per-core, per-cluster), GPU, power breakdown, temperature, memory, network, disk I/O, process list
- `tui-dashboard`: Terminal UI with multi-panel layout — CPU core bars, power sparklines, GPU gauge, memory bars, temperature display, sortable process table, color themes, keyboard controls
- `api-server`: HTTP server exposing metrics at /json (JSON snapshot) and /metrics (Prometheus text format), plus NDJSON pipe mode for stdout streaming
- `cli-interface`: Command-line argument parsing — subcommands (tui, pipe, serve), global options (interval, color, temp-unit, port)

### Modified Capabilities

(none — greenfield project)

## Impact

- New crate/binary: `mtop`
- Dependencies: ratatui, crossterm, clap, core-foundation, libc, serde, serde_json, chrono
- macOS-only (Apple Silicon required for full metrics; Intel Mac partial support)
- No external service dependencies — fully local, offline operation
- No sudo/root privileges required
