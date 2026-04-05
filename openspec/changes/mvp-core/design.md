## Context

mtop is a greenfield macOS system monitor targeting Apple Silicon. It collects hardware metrics (CPU, GPU, power, temperature, memory, network, disk) without sudo and presents them via three modes: TUI dashboard, JSON pipe, and HTTP API. This is the MVP — first working version with core functionality.

The project uses a cleanroom development methodology. All implementation derives from these formal specs, not from any reference project source code.

## Goals / Non-Goals

**Goals:**
- Collect all 7 metric categories from macOS APIs without elevated privileges
- Render a responsive multi-panel TUI dashboard using ratatui
- Expose metrics via NDJSON pipe and HTTP endpoints (JSON + Prometheus)
- Single statically-linked Rust binary, installable via cargo or Homebrew
- Support all Apple Silicon variants (M1–M5, base/Pro/Max)
- Graceful degradation when sensors are unavailable

**Non-Goals:**
- Menu Bar mode (requires native macOS GUI, deferred)
- Cross-platform support (macOS only for MVP)
- Ultra (dual-die) chip support (single-die first)
- Per-process energy attribution
- Mouse support in TUI
- In-app configuration menus
- Battery detailed monitoring (voltage, amperage, time remaining)
- Peripheral monitoring (USB, Bluetooth, WiFi power)

## Decisions

### D1: Language — Rust
**Choice**: Rust with `ratatui` for TUI
**Rationale**: Best performance/safety balance for system-level monitoring. Zero-cost abstractions keep CPU overhead minimal. `core-foundation` crate provides mature CoreFoundation FFI. Single binary distribution.
**Alternatives considered**: Go (gopsutil ecosystem is good but CGo overhead for IOReport), Swift (natural for macOS but TUI ecosystem is immature), C++ (unsafe, no modern package management)

### D2: Architecture — layered with shared metrics store
**Choice**: 4-layer architecture: Platform → Collection → Store → Presentation
**Rationale**: Separating platform FFI from business logic from presentation allows each mode (TUI/API/pipe) to read from the same store without duplication. Also enables future menu bar mode to share the same core.
**Store design**: `Arc<RwLock<MetricsSnapshot>>` with rolling history buffer (128 points) for sparklines.

### D3: Threading — collection thread + presentation thread
**Choice**: Dedicated collection thread updates shared store at interval. TUI runs in main thread. HTTP server spawns its own thread.
**Rationale**: Decouples collection latency from rendering. If a sensor is slow, the TUI doesn't freeze. Simpler than per-source threads (macpow approach) for MVP.

### D4: macOS API strategy — IOReport + SMC + Mach
**Choice**: Three primary data sources via FFI:
- **IOReport**: CPU/GPU frequencies (DVFS residency), power (Energy Model)
- **SMC**: Temperature sensors, system power rails
- **Mach API**: CPU utilization ticks (`host_processor_info`), memory stats (`host_statistics64`)
- **sysctl**: Hardware info, swap, system metadata
- **getifaddrs**: Network interface byte counters

**Rationale**: This combination covers all 7 metric categories without sudo. IOReport is the key private API for Apple Silicon-specific data.

### D5: TUI layout — single fixed layout for MVP
**Choice**: One well-designed multi-panel layout (CPU left, Power+Temp+Memory+Network right, Process list bottom)
**Rationale**: Focus on getting one great layout rather than many mediocre ones. Add layout presets in v0.2+.

### D6: HTTP server — minimal raw TCP
**Choice**: Lightweight HTTP handler (parse GET path, return response), no framework
**Rationale**: Only need 2-3 endpoints. Adding a full HTTP framework (actix, axum) is overkill and bloats the binary.

### D7: JSON schema — flat with nested groups
**Choice**: JSON output groups metrics by category (cpu, gpu, power, temperature, memory, network, disk) with a top-level soc info block and timestamp.
**Rationale**: Easy to parse with jq, each category is independently useful, extensible without breaking changes.

### D8: Process list — via sysctl/proc_taskinfo
**Choice**: Read process info via sysctl and proc_taskinfo Mach calls
**Rationale**: No shelling out to `ps`. Direct API access is faster and more reliable.

## Risks / Trade-offs

- **[Private API instability]** → IOReport API may change between macOS versions. Mitigation: version-detect at startup, graceful fallback if API shape changes.
- **[No sudo = limited sensors]** → Some SMC keys may be inaccessible without root. Mitigation: skip unavailable sensors, show "N/A" in TUI.
- **[Single-die only]** → Ultra chips have dual-die IOReport prefixes. Mitigation: defer Ultra support, document limitation.
- **[No GPU on Intel Mac]** → IOReport Energy Model is Apple Silicon only. Mitigation: detect arch at startup, disable GPU/power sections on Intel.
- **[TUI terminal compatibility]** → Not all terminals support 24-bit color or Braille characters. Mitigation: detect capabilities, fallback to 256-color and block characters.

## Open Questions

1. Should the Prometheus metric prefix be `mtop_` or configurable?
2. Config file format — should MVP support a config file, or CLI flags only?
3. Process list update frequency — same as metrics interval, or independent?
