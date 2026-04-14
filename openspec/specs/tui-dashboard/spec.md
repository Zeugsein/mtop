# tui-dashboard Specification

## Purpose
Define requirements for the terminal user interface dashboard displaying real-time system metrics with keyboard navigation and theming.
## Requirements
### Requirement: Multi-panel dashboard layout
The TUI SHALL display a multi-panel dashboard with sections for CPU, GPU, power, temperature, memory, network, and process list. Panels SHALL be arranged to maximize information density while remaining readable at 80x24 minimum terminal size. [T1-static]

#### Scenario: Default TUI launch
- **WHEN** the user runs `mtop` with no subcommand
- **THEN** the terminal SHALL display a dashboard with CPU panel (left), power + temperature + memory + network panels (right), and process list (bottom)

#### Scenario: Minimum terminal size
- **WHEN** the terminal is 80 columns by 24 rows
- **THEN** the dashboard SHALL render without overflow or corruption

### Requirement: CPU core visualization
The TUI SHALL display per-core CPU utilization as horizontal bar charts with percentage labels. Bars SHALL be color-coded by utilization level. Each core SHALL be labeled with its type (E for efficiency, P for performance) and index. [T1-static]

#### Scenario: Per-core bars rendering
- **WHEN** CPU metrics are available
- **THEN** each core SHALL display as a labeled bar (e.g., "E0 [████░░░░] 42%") with color coding: green < 30%, cyan 30-40%, yellow 40-60%, red > 60%

#### Scenario: Cluster summary
- **WHEN** CPU metrics are available
- **THEN** the CPU panel SHALL show aggregate CPU usage percentage and total CPU power in Watts

### Requirement: Power sparkline charts
The TUI SHALL display power metrics as sparkline history charts showing current, average, and trend over time. Individual components (CPU, GPU, ANE, DRAM) and totals (package, system) SHALL each have their own sparkline row. [T1-static]

#### Scenario: Power panel rendering
- **WHEN** power metrics are available
- **THEN** the power panel SHALL show sparkline charts for CPU, GPU, ANE, DRAM, package total, and system total, each with current value in Watts

#### Scenario: History accumulation
- **WHEN** the TUI has been running for N sample cycles
- **THEN** sparklines SHALL display up to 128 historical data points, scrolling oldest values off the left edge

### Requirement: GPU gauge display
The TUI SHALL display GPU utilization as a gauge or bar with percentage and frequency in MHz, plus power draw in Watts. [T1-static]

> Note: GPU frequency derives from IOReport state names (tech-spec/ioreport.md), not hardcoded estimates. GPU power derives from the power collector's gpu_w value.

#### Scenario: GPU panel rendering
- **WHEN** GPU metrics are available
- **THEN** the TUI SHALL show GPU utilization %, frequency in MHz, and power in Watts

### Requirement: GPU power display accuracy
The GPU power display SHALL show actual measured power from the power collector, not a hardcoded 0.0 value. [T1-static]

#### Scenario: GPU power rendering with active GPU
- **WHEN** the power collector reports GPU power of 3.5W
- **THEN** the TUI GPU panel SHALL display approximately 3.5W for power, not 0.0W

#### Scenario: GPU power rendering with idle GPU
- **WHEN** the GPU is fully idle and power is 0.0W
- **THEN** the TUI GPU panel SHALL display 0.0W (the genuine measured value)

### Requirement: Temperature display
The TUI SHALL display CPU average and GPU average temperatures in the configured unit (Celsius or Fahrenheit). [T1-static]

#### Scenario: Temperature rendering
- **WHEN** temperature metrics are available
- **THEN** the TUI SHALL show "CPU avg: XX°C  GPU avg: XX°C" (or °F if configured)

### Requirement: Memory bar display
The TUI SHALL display RAM and swap usage as horizontal bars with used/total labels in human-readable units (GB). [T1-static]

#### Scenario: Memory panel rendering
- **WHEN** memory metrics are available
- **THEN** the TUI SHALL show RAM bar (e.g., "RAM [████████░░░] 18.2/24GB") and swap bar if swap is active

### Requirement: Network rate display
The TUI SHALL display current network upload and download rates with appropriate unit scaling. [T1-static]

#### Scenario: Network panel rendering
- **WHEN** network metrics are available
- **THEN** the TUI SHALL show upload rate (up arrow) and download rate (down arrow) in auto-scaled units (B/s, KB/s, MB/s, GB/s)

### Requirement: Process list table
The TUI SHALL display a sortable table of running processes showing PID, name, CPU %, memory, and user columns. [T1-static]

#### Scenario: Default process list
- **WHEN** the TUI is displaying
- **THEN** the process list SHALL show processes sorted by CPU % descending, with columns: PID, Name, CPU%, Mem, User

#### Scenario: Sort cycling
- **WHEN** the user presses `s`
- **THEN** the sort column SHALL cycle through: CPU%, Memory, PID, Name

### Requirement: Keyboard controls
The TUI SHALL respond to keyboard input for navigation and control. [T1-static]

#### Scenario: Quit
- **WHEN** the user presses `q` or `Esc`
- **THEN** the TUI SHALL exit cleanly, restoring terminal state

#### Scenario: Process list navigation
- **WHEN** the user presses Up/Down arrow or j/k
- **THEN** the selection in the process list SHALL move accordingly

#### Scenario: Interval adjustment
- **WHEN** the user presses `+` or `-`
- **THEN** the sampling interval SHALL increase or decrease by 250ms (clamped to 100ms minimum)

#### Scenario: Theme cycling
- **WHEN** the user presses `c`
- **THEN** the color theme SHALL cycle to the next available theme

### Requirement: Color themes
The TUI SHALL support at least 3 built-in color themes. The active theme SHALL affect all UI elements consistently. [T1-static]

#### Scenario: Theme application
- **WHEN** a theme is active
- **THEN** all borders, labels, bars, graphs, and text SHALL use colors from that theme

### Requirement: Terminal resize handling
The TUI SHALL reflow its layout when the terminal is resized. [T1-static]

#### Scenario: Terminal resize
- **WHEN** the terminal dimensions change
- **THEN** the dashboard SHALL re-render to fit the new size without crashing or leaving artifacts

### Requirement: SoC info header
The TUI SHALL display a header line showing the chip model name, core configuration, and total memory. [T1-static]

#### Scenario: Header rendering
- **WHEN** the TUI starts
- **THEN** the header SHALL show e.g., "mtop — Apple M4 Pro — 10C (4E+6P) / 16GPU — 24GB"

### Requirement: Sensor unavailable distinction [I3-T1]
The TUI SHALL distinguish between "sensor unavailable" and "sensor reads zero". When sensor data is `None` or equivalent unavailable marker, the display SHALL differ from a genuine zero reading. [T1-static]

> Reference: issues-realdevice-2026-04-06.md — RD-4; challenge-iteration3.md — Gap NF-4; zero and unavailable are semantically different

#### Scenario: Sensor unavailable display
- **WHEN** a sensor (e.g., GPU temperature) returns `None` / unavailable
- **THEN** the TUI SHALL display "N/A" or equivalent placeholder text, NOT "0°C" or "0.0W"

#### Scenario: Sensor reads genuine zero
- **WHEN** a sensor returns `Some(0.0)` (genuine measured zero)
- **THEN** the TUI SHALL display "0.0W" or "0°C" as appropriate (the real measured value)

### Test Scenarios
- Unit test: render a panel with `None` sensor value, verify output contains "N/A" (not "0")
- Unit test: render a panel with `Some(0.0)` sensor value, verify output contains "0.0" (not "N/A")
- Code inspection: verify the rendering code checks for `Option::None` before formatting

### Requirement: No sparkline growth for unavailable sensors [I3-T2]
The TUI SHALL NOT render a growing sparkline when sensor data is unavailable. History SHALL NOT be pushed when the sensor value is `None` / unavailable. [T1-static]

> Reference: issues-realdevice-2026-04-06.md — RD-4; unconditionally pushing 0.0 into history makes sparklines visually "grow" with lowest bars

#### Scenario: Unavailable sensor sparkline
- **WHEN** a power sensor has been unavailable for 10 consecutive sample cycles
- **THEN** the sparkline for that sensor SHALL NOT show 10 new lowest-level bars; it SHALL remain empty or show a flat "N/A" indicator

#### Scenario: Sensor becomes available
- **WHEN** a sensor transitions from unavailable to available
- **THEN** the sparkline SHALL begin accumulating data points from the first available reading

### Test Scenarios
- Unit test: push `None` into MetricsHistory 10 times, verify the history length does not grow
- Unit test: push `Some(5.0)` after `None` values, verify only the `Some` values appear in history
- Code inspection: verify `MetricsHistory::push` skips the push when the input is `None`

### Requirement: Unavailable sensor display text [I3-T3]
The TUI SHALL display "N/A" or equivalent when a sensor returns `None` / unavailable. This applies to all metric panels: power, temperature, GPU utilization, and GPU frequency. [T1-static]

#### Scenario: Temperature N/A
- **WHEN** the temperature collector returns `None` for GPU temperature
- **THEN** the temperature panel SHALL display "GPU avg: N/A" instead of "GPU avg: 0°C"

#### Scenario: Power N/A
- **WHEN** the power collector returns `None` for GPU power
- **THEN** the power panel SHALL display "N/A" for the GPU power row, not "0.0W"

#### Scenario: GPU utilization N/A
- **WHEN** the GPU collector returns `None` for utilization
- **THEN** the GPU panel SHALL display "N/A" for utilization, not "0%"

### Test Scenarios
- Unit test: for each panel type (temperature, power, GPU), render with `None` value and verify "N/A" appears in output
- Integration test: run mtop on a system where GPU IOReport subscription fails, verify TUI shows "N/A" for GPU metrics (not 0%)

## Iteration 6: Network Panel + Process Panel [I6]

### Requirement: Network history buffers [I6-C1]
`MetricsHistory` SHALL contain `net_upload` and `net_download` history buffers. `push()` SHALL sum `tx_bytes_sec` and `rx_bytes_sec` across all non-loopback interfaces and store the aggregates.

> SHALL-C1-01, SHALL-C1-02, SHALL-C1-03

#### Scenario: Aggregate rate computation
- **WHEN** a snapshot with two interfaces (en0: tx=100, rx=200; en1: tx=50, rx=150) is pushed
- **THEN** `net_upload` receives 150.0 and `net_download` receives 350.0

#### Scenario: Loopback excluded
- **WHEN** a snapshot includes lo0 alongside real interfaces
- **THEN** loopback bytes are excluded from the aggregate

### Requirement: Interface speed tier [I6-C2]
A tier function SHALL map `ifi_baudrate` (bits/sec) to bytes/sec scale: >=1Gbps→125M, >=100Mbps→12.5M, else→1.25M. `NetworkMetrics` SHALL expose `primary_baudrate: u64` from the highest-baudrate non-loopback interface.

> SHALL-C2-01, SHALL-C2-02

#### Scenario: Tier selection
- **WHEN** `ifi_baudrate` is 1_000_000_000
- **THEN** tier function returns 125_000_000

#### Scenario: Zero baudrate fallback
- **WHEN** all interfaces report baudrate 0
- **THEN** tier falls back to 1_250_000 (10 Mbps tier)

### Requirement: Weighted process sort [I6-C3]
`weighted_score` SHALL compute `0.5*cpu_norm + 0.3*mem_norm + 0.2*power_norm` where each dimension is normalized 0.0-1.0 against the max across all processes. A +0.5 spike bonus applies when any single dimension exceeds 0.9. Division-by-zero guards are mandatory: zero or negative max yields 0.0 for that dimension.

> SHALL-C3-01 through SHALL-C3-05

#### Scenario: Composite score
- **WHEN** process has cpu_norm=1.0, mem_norm=0.0, power_norm=0.0
- **THEN** base score is 0.5

#### Scenario: Spike bonus
- **WHEN** cpu_norm=0.95
- **THEN** score includes +0.5 spike bonus

#### Scenario: Division-by-zero guard
- **WHEN** max_cpu=0.0 and process cpu=0.0
- **THEN** cpu_norm=0.0 (not NaN); result satisfies `f64::is_finite()`

### Requirement: Network panel rendering [I6-C4]
The network panel SHALL use Type B layout (37.5/37.5/25). Upload sparkline (left, `net_upload` color) and download sparkline (middle, `net_download` color) from history. Right region shows interfaces ranked by total throughput descending, using compact rate format. Empty interface list SHALL NOT panic.

> SHALL-C4-01 through SHALL-C4-08

#### Scenario: Frame top format
- **WHEN** aggregate upload=2.5MB/s and download=47MB/s
- **THEN** frame top contains auto-scaled rate strings for both

#### Scenario: Empty interfaces
- **WHEN** interface list is empty
- **THEN** panel renders without panic; frame top shows zero rates

### Requirement: Process panel rendering [I6-C5]
The process panel SHALL display processes sorted by `weighted_score` descending. Each row SHALL show `{name}  {dot}{dot}{dot}` with three colored dots for CPU, memory, and power dimensions. Scroll offset SHALL be clamped to `processes.len().saturating_sub(1)` each frame.

> SHALL-C5-01 through SHALL-C5-06

#### Scenario: Dot colors
- **WHEN** process has cpu_norm=1.0
- **THEN** CPU dot uses high-intensity gradient color

#### Scenario: Scroll clamping
- **WHEN** scroll offset exceeds process list length
- **THEN** scroll is clamped, panel renders without panic

---

## Iteration 7: Polish + Expand/Collapse + Theme System [I7]

### Requirement: Interface display filtering [I7-W1A]
The network panel interface ranking SHALL exclude interfaces matching prefixes: `bridge`, `awdl`, `llw`, `gif`, `stf`, `XHC`, `ap`, `utun`. Filtering applies to display only; aggregate sparklines include all non-loopback interfaces.

> SHALL-W1A-01, SHALL-W1A-02, SHALL-W1A-03

#### Scenario: Infrastructure filtered
- **WHEN** interfaces are [en0, awdl0, bridge0, utun3]
- **THEN** filtered display list contains only en0

#### Scenario: All filtered
- **WHEN** only infrastructure interfaces exist
- **THEN** ranking area displays "No active interfaces"

### Requirement: Compact rate format [I7-W1C]
`format_bytes_rate_compact` SHALL produce compact output: `2.4M`, `350K`, `1.2G`, `42B`. The network interface ranking SHALL use this compact format.

> SHALL-W1C-01, SHALL-W1C-02

#### Scenario: Compact formatting
- **WHEN** rate is 2_500_000.0 bytes/sec
- **THEN** output is "2.4M"

### Requirement: Process name ellipsis [I7-W1D]
Process names exceeding the available column width SHALL be truncated and suffixed with `…` (U+2026). The truncated name + ellipsis SHALL NOT exceed the available width.

> SHALL-W1D-01, SHALL-W1D-02

#### Scenario: Long name truncated
- **WHEN** name is "Google Chrome Helper" and width is 15
- **THEN** output is "Google Chrome H…" (14 chars + ellipsis)

### Requirement: Expand/collapse state [I7-W2A]
A `PanelId` enum SHALL exist with variants: Cpu, Gpu, MemDisk, Network, Power, Process. `AppState` SHALL contain `expanded_panel: Option<PanelId>`. Expanding panel B while panel A is expanded SHALL replace A.

> SHALL-W2A-01, SHALL-W2A-02, SHALL-W2A-03

#### Scenario: Single expansion
- **WHEN** Network is expanded while Cpu is already expanded
- **THEN** `expanded_panel` = Some(Network)

### Requirement: Expanded layout [I7-W2B]
When a left-column panel (Cpu, Gpu, MemDisk) is expanded, it SHALL occupy the full left column. When a right-column panel is expanded, it SHALL occupy the full right column. The non-expanded column renders normally. No panel expanded = normal 3+3 grid.

> SHALL-W2B-01 through SHALL-W2B-04

#### Scenario: Left panel expanded
- **WHEN** expanded_panel=Some(Cpu)
- **THEN** left column has 1 panel filling all space, right column has 3 normal panels

### Requirement: Expanded panel content [I7-W2C]
Each expanded panel SHALL display additional detail: CPU→per-core bars, GPU→detailed metrics table, MemDisk→RAM/swap gauges + disk I/O, Network→per-interface stats, Power→full component breakdown, Process→full columns with scroll support. All SHALL truncate gracefully at insufficient height.

> SHALL-W2C-01 through SHALL-W2C-08

#### Scenario: Expanded process columns
- **WHEN** process panel is expanded
- **THEN** PID and user columns are visible (not present in compact view)

### Requirement: Panel keybindings [I7-W2D]
Keys `1`-`6` SHALL select panels: 1=Cpu, 2=Gpu, 3=MemDisk, 4=Network, 5=Power, 6=Process. `Enter`/`e` SHALL toggle expand/collapse. `Escape` SHALL collapse any expanded panel; if none expanded, quit. `q` SHALL always quit. Legacy `sort_col`, `SORT_COLS`, and `s` keybinding SHALL be removed.

> SHALL-W2D-01 through SHALL-W2D-06

#### Scenario: Escape modal behavior
- **WHEN** a panel is expanded and user presses Escape
- **THEN** panel collapses (does not quit)

#### Scenario: Escape without expansion
- **WHEN** no panel is expanded and user presses Escape
- **THEN** application quits

### Requirement: Unified theme array [I7-W3A]
The legacy `THEMES: &[(&str, Color, Color)]` array SHALL be replaced with `&[Theme]` using the full `Theme` struct. `theme_names()` SHALL return names from the unified array. `draw_dashboard` SHALL use `themes[state.theme_idx]`.

> SHALL-W3A-01 through SHALL-W3A-04

#### Scenario: Theme array
- **WHEN** `theme_names()` is called
- **THEN** returns at least 10 entries with "horizon" at index 0

### Requirement: Additional themes [I7-W3B]
At least 9 additional themes SHALL be defined beyond Horizon. Each theme SHALL define all `Theme` struct fields. For every theme, `net_upload` and `net_download` SHALL be distinct colors.

> SHALL-W3B-01 through SHALL-W3B-03

#### Scenario: Dracula and Nord themes
- **WHEN** theme array is accessed
- **THEN** "dracula" (bg Rgb(40,42,54)) and "nord" (bg Rgb(46,52,64)) SHALL exist

### Requirement: GPU sparkline in GPU panel [I7-W3D]
The GPU panel SHALL display a braille sparkline of `gpu_usage` history in the trend area, using `theme.gpu_accent` color, scaled to 1.0 (100% GPU usage).

> SHALL-W3D-01, SHALL-W3D-02, SHALL-W3D-03

#### Scenario: GPU sparkline rendered
- **WHEN** GPU usage history has data
- **THEN** panel renders non-empty braille sparkline

---

## Iteration 8: Module Extraction + Data Model Expansion [I8]

### Requirement: Extract expanded panel renderers [I8-W1A]
A new module `src/tui/expanded.rs` SHALL contain 6 functions: `draw_cpu_expanded`, `draw_gpu_expanded`, `draw_mem_disk_expanded`, `draw_network_expanded`, `draw_power_expanded`, `draw_process_expanded`, all `pub(crate)`. `mod.rs` SHALL NOT contain any `draw_*_expanded` function body after extraction.

> SHALL-W1A-01 through SHALL-W1A-04

#### Scenario: Extraction complete
- **WHEN** extraction is done
- **THEN** all existing tests pass; `mod.rs` contains no expanded renderer bodies

### Requirement: Extract helper functions [I8-W1B]
A new module `src/tui/helpers.rs` SHALL contain: `format_bytes_rate_compact` (pub), `truncate_with_ellipsis` (pub(crate)), `is_infrastructure_interface` (pub(crate)). No duplicate definitions in `mod.rs`.

> SHALL-W1B-01 through SHALL-W1B-04

### Requirement: Extract keybinding logic [I8-W1C]
A new module `src/tui/input.rs` SHALL contain `handle_key_event(key: KeyEvent, state: &mut AppState) -> bool` returning `true` if the app should quit. The `run()` function SHALL call this instead of inline match arms.

> SHALL-W1C-01, SHALL-W1C-02, SHALL-W1C-03

#### Scenario: Quit key
- **WHEN** `handle_key_event` receives KeyCode::Char('q')
- **THEN** returns true

### Requirement: Memory pressure fields [I8-W2A]
`MemoryMetrics` SHALL gain fields: `wired: u64`, `app: u64`, `compressed: u64` (bytes). `wired` from `wire_count * page_size`, `app` from `(internal_page_count - purgeable_count) * page_size`, `compressed` from `compressor_page_count * page_size`.

> SHALL-W2A-01 through SHALL-W2A-05

#### Scenario: Pressure sanity
- **WHEN** metrics are collected
- **THEN** `wired + app + compressed <= ram_total`

### Requirement: Per-interface baudrate and packets [I8-W2B]
`NetInterface` SHALL gain fields: `baudrate: u64`, `packets_in_sec: f64`, `packets_out_sec: f64`. Packet rates are delta/interval, same pattern as byte rates. First sample defaults to 0.0.

> SHALL-W2B-01 through SHALL-W2B-04

### Requirement: Interface type classification [I8-W2C]
`classify_interface(name: &str) -> &'static str` SHALL return: `en*`→"Ethernet/Wi-Fi", `utun*`→"VPN", `bridge*`→"Bridge", `awdl*`→"AirDrop", `lo*`→"Loopback", else→"Other".

> SHALL-W2C-01, SHALL-W2C-02

#### Scenario: Classification
- **WHEN** name is "en0"
- **THEN** returns "Ethernet/Wi-Fi"

---

## Iteration 9: Expanded Panel Polish + Process Detail + Thermal [I9]

### Requirement: Memory pressure stacked gauge [I9-W1A]
Expanded MemDisk panel SHALL display wired/app/compressed as a stacked horizontal gauge with proportional colored segments. Segment colors: wired=red, app=blue, compressed=yellow (theme-aware). Each segment shows its label and GB value.

> SHALL-W1A-01, SHALL-W1A-02, SHALL-W1A-03

### Requirement: Fractional baudrate formatting [I9-W1B]
`format_baudrate` SHALL use floating-point for non-round speeds (e.g., 2.5Gbps). Round speeds display as integers (e.g., 1 Gbps). Zero baudrate displays as "—".

> SHALL-W1B-01, SHALL-W1B-02, SHALL-W1B-03

#### Scenario: Fractional speed
- **WHEN** baudrate is 2_500_000_000
- **THEN** output is "2.5 Gbps"

### Requirement: Process thread count [I9-W2A]
`ProcessInfo` SHALL have `thread_count: i32` populated from `ProcTaskInfo.pti_threadnum`. Expanded Process panel SHALL display the thread count column.

> SHALL-W2A-01, SHALL-W2A-02, SHALL-W2A-03

### Requirement: Process I/O rates [I9-W2B]
`ProcessInfo` SHALL have `io_read_bytes_sec: f64` and `io_write_bytes_sec: f64` from `RusageInfoV4` fields, computed as delta rates. First sample for a PID reports 0.0. Expanded Process panel SHALL display I/O rates in compact format.

> SHALL-W2B-01 through SHALL-W2B-05

### Requirement: Process sort modes [I9-W2C]
`SortMode` enum SHALL have variants: WeightedScore, Cpu, Memory, Power, Pid, Name. `s` key SHALL cycle through sort modes. Default is WeightedScore. Current sort mode SHALL be displayed in expanded Process panel title. Compact panel also respects sort mode.

> SHALL-W2C-01 through SHALL-W2C-05

#### Scenario: Sort cycling
- **WHEN** user presses `s`
- **THEN** sort mode cycles to the next variant

### Requirement: Thermal zone mapping [I9-W3A]
`ThermalMetrics` SHALL add optional fields: `ssd_avg_c`, `battery_avg_c`. `smc_enumerate_temp_keys` SHALL classify Ts*→SSD, TB*→Battery in addition to CPU/GPU. Expanded panels SHALL show relevant thermal data.

> SHALL-W3A-01, SHALL-W3A-02, SHALL-W3A-03

### Requirement: Thermal threshold alerts [I9-W3B]
Temperature display SHALL use yellow color when >80°C (CPU) or >85°C (GPU), and red when >95°C (CPU) or >100°C (GPU). Thresholds SHALL be compile-time constants.

> SHALL-W3B-01, SHALL-W3B-02, SHALL-W3B-03

### Requirement: Fan speed display [I9-W3C]
`ThermalMetrics` SHALL add `fan_speeds: Vec<u32>` (RPM per fan) read from SMC keys F0Ac, F1Ac using fpe2 decoding. Expanded Power panel SHALL display fan RPM when available. Fanless machines show "No fans" or omit the section.

> SHALL-W3C-01 through SHALL-W3C-04

---

## Iteration 10: FFI Safety + Per-Interface History + Config Persistence [I10]

### Requirement: FFI offset assertions [I10-W1A]
Structs SHALL have compile-time `offset_of!` assertions: RusageInfoV4 (ri_diskio_bytesread@144, ri_diskio_byteswritten@152, ri_billed_energy@264), ProcTaskInfo (pti_resident_size@8, pti_threadnum@84), SmcKeyData (key@0, data8@37, bytes@48), IfData (ifi_baudrate@16, ifi_ibytes@56), VmStatistics64 (wire_count@12, compressor_page_count@104).

> SHALL-W1A-01 through SHALL-W1A-05

### Requirement: Unsafe block safety comments [I10-W1B]
All unsafe blocks SHALL have a `SAFETY:` comment explaining the invariant. No raw pointer cast SHALL target a misaligned or undersized buffer.

> SHALL-W1B-01, SHALL-W1B-02

### Requirement: Per-interface network history [I10-W2A]
`MetricsHistory` SHALL have `per_iface: HashMap<String, (HistoryBuffer, HistoryBuffer)>`. `push()` SHALL update per-interface rx/tx buffers for non-infrastructure interfaces, capped at 128 entries. Stale interfaces retain their buffers.

> SHALL-W2A-01 through SHALL-W2A-04

### Requirement: Per-interface sparklines in expanded panel [I10-W2B]
Expanded Network panel SHALL show a mini rx sparkline per active interface using that interface's own history buffer. Sparkline scale uses the interface's baudrate (or aggregate fallback).

> SHALL-W2B-01, SHALL-W2B-02, SHALL-W2B-03

### Requirement: Config file persistence [I10-W3A]
Config SHALL be read from `~/.config/mtop/config.toml`. Config struct: theme (String), interval_ms (u32), temp_unit (String), sort_mode (String). Missing or invalid config SHALL fall back to defaults silently.

> SHALL-W3A-01 through SHALL-W3A-04

### Requirement: Config save [I10-W3B]
`w` key SHALL save current theme, interval_ms, and sort_mode to config.toml. SHALL create `~/.config/mtop/` directory if needed.

> SHALL-W3B-01, SHALL-W3B-02

### Requirement: Config precedence [I10-W3C]
Config loads before CLI arg parsing. CLI args override config values. Invalid config values print a warning to stderr and use defaults.

> SHALL-W3C-01, SHALL-W3C-02, SHALL-W3C-03

---

## Iteration 11: Testability Refactor + Rendering Tests [I11]

### Requirement: AppState Default implementation [I11-W0A]
`AppState` SHALL implement `Default` with sensible test values (interval_ms=1000, theme_idx=0, sort_mode=WeightedScore, empty history/snapshot). The impl SHALL be `pub(crate)` and not cfg-gated.

> SHALL-W0A-01, SHALL-W0A-02

### Requirement: Pure data preparation functions [I11-W1]
Extraction functions SHALL exist for pure data preparation: `prepare_process_rows` (accepts process slice, sort_mode, scroll, max_visible, max values; returns `Vec<ProcessRow>`), `prepare_power_components` (returns component list with name/watts/color), `prepare_network_rows` (filters infrastructure, sorts by traffic, returns `Vec<NetworkRow>`), `prepare_memory_pressure` (computes wired/app/compressed fractions clamped 0.0-1.0).

> SHALL-W1A-01 through SHALL-W1D-03

#### Scenario: Network row filtering
- **WHEN** `prepare_network_rows` is called with mixed interfaces
- **THEN** infrastructure interfaces are excluded from results

### Requirement: TestBackend rendering tests [I11-W2]
All 6 compact panels SHALL render without panic at 80x24 and 120x40. All 6 expanded panels SHALL render without panic at both sizes. `draw_dashboard` SHALL render without panic at 80x24, 120x40, and 60x20. Content assertions: CPU panel contains "CPU", Process panel contains "Processes", Network panel contains "Network".

> SHALL-W2A-01 through SHALL-W2D-01

### Requirement: Input handler tests [I11-W3A]
Tests SHALL verify all key bindings produce correct state mutations: q→quit, Ctrl+C→quit, c→increment theme_idx, 1-6→set PanelId, e→toggle expanded, +→increase interval, -→decrease interval, j/k→scroll, s→cycle sort_mode, Esc from expanded→clear expanded, Esc without expanded→quit.

> SHALL-W3A-01 through SHALL-W3A-12

---

## Iteration 16: Visual Foundation [I16]

### Requirement: Color derivation function [I16-C1]
`derive_companion(base: Color, hue_shift_deg: f32, sat_factor: f32) -> Color` SHALL exist in `theme.rs`. It converts RGB→HSL, rotates hue by `hue_shift_deg`, multiplies saturation by `sat_factor` (clamped 0.0-1.0), converts back to RGB. For achromatic inputs (S=0), returns color with same RGB values (only sat_factor applies). GPU accent = `derive_companion(cpu_accent, 30.0, 0.9)`. Power accent = `derive_companion(mem_accent, 30.0, 0.9)`. Within each theme, derived GPU accent SHALL differ from CPU accent by at least 15° hue.

> SHALL-C1-01 through SHALL-C1-05

#### Scenario: Achromatic input
- **WHEN** input is `Color::Rgb(128,128,128)` (S=0)
- **THEN** hue rotation has no effect; only sat_factor dims the result

### Requirement: Theme palette updates [I16-C2]
HORIZON theme SHALL set: `cpu_accent` = Rgb(184,119,219), `mem_accent` = Rgb(9,247,160), `net_download` = Rgb(233,83,121). All 10 themes SHALL have manually assigned CPU, MEM, NET accent colors. GPU and Power accents SHALL be computed via `derive_companion`, NOT hardcoded. All accent colors SHALL have contrast ratio >= 3.0 against that theme's `bg`.

> SHALL-C2-01 through SHALL-C2-04

### Requirement: Adaptive border brightness [I16-C3]
`bg_luminance(theme: &Theme) -> f64` SHALL compute relative luminance of `theme.bg` per WCAG formula. `dim_color` factor SHALL be adaptive: light themes (luminance >= 0.5) use 0.35, dark themes use 0.55. All 6 panels SHALL use the adaptive factor. `BorderType::Rounded` SHALL be kept everywhere.

> SHALL-C3-01 through SHALL-C3-04

#### Scenario: Adaptive dim factor
- **WHEN** Horizon (dark) is active
- **THEN** dim factor is 0.55

### Requirement: Panel title redesign [I16-C4]
Panel title text SHALL use `theme.fg` with bold modifier, NOT panel accent color. Each title SHALL be prefixed with colored superscript: ¹ cpu, ² gpu, ³ mem, ⁴ net, ⁵ power, ⁶ proc — superscript colored with panel accent. Power panel cpu/gpu sub-labels retain their accent colors (exception). Frame-bottom info stays `theme.muted`.

> SHALL-C4-01 through SHALL-C4-04

#### Scenario: Title styling
- **WHEN** any panel title is rendered
- **THEN** title text uses `theme.fg` bold; superscript uses panel accent color

### Requirement: Direct panel expand via 1-6 keys [I16-C5]
Keys 1-6 SHALL directly toggle expand: same number pressed = collapse (set `expanded_panel = None`); different number = expand/switch. `e`/`Enter` still collapse for backward compat. Help overlay updates to "1-6: expand panel".

> SHALL-C5-01 through SHALL-C5-04

#### Scenario: Toggle collapse
- **WHEN** panel 1 is expanded and user presses '1'
- **THEN** `expanded_panel` = None

### Requirement: Centered header [I16-C6]
Header SHALL render centered: `YYYY-MM-DD HH:MM:SS — mtop — {chip} ({cores})`. If centered header width > terminal width, truncate chip info first. SHALL render without panic at 80-col minimum.

> SHALL-C6-01, SHALL-C6-02

### Requirement: Chart-detail gap [I16-C7]
`split_type_a` SHALL insert a 1-column gap between trend and detail areas: `[Percentage(74), Length(1), Percentage(25)]`. `split_type_b` SHALL insert 1-column gaps between all sections. Gap columns are empty.

> SHALL-C7-01, SHALL-C7-02, SHALL-C7-03

### Requirement: Braille Y-position gradient [I16-C8]
`render_braille_graph` SHALL color each row by Y-position: `value_to_color(row_idx / (height-1))`. Row 0 (bottom) = green, top row = red. Replaces data-value coloring. ALL panels using `render_braille_graph` display Y-position gradient. `render_braille_sparkline` (single-row) retains data-value coloring.

> SHALL-C8-01, SHALL-C8-02, SHALL-C8-03

#### Scenario: Row colors differ
- **WHEN** `render_braille_graph` renders with height=4
- **THEN** row 0 color != row 3 color

### Requirement: Gauge per-character gradient and layout flip [I16-C9]
`render_gauge_bar` SHALL color each filled character individually: position `i` uses `value_to_color(i / width)`. `render_compact_gauge` SHALL produce bar LEFT, percentage RIGHT (format: `■■■■■■░░░░ XX%`). Per-character gradient applies. If bar width < 3, fall back to single-color fill. Unfilled characters remain `theme.muted`.

> SHALL-C9-01 through SHALL-C9-05

#### Scenario: Gauge layout
- **WHEN** compact gauge is rendered
- **THEN** first span starts with ■, last span contains "%"

---

## Iteration 17: Panel Redesign & Detail Polish [I17]

### Requirement: Available memory history [I17-C1]
Add `mem_available: HistoryBuffer` to `MetricsHistory` tracking available fraction (0.0-1.0). `push()` computes `(ram_total - ram_used) / ram_total`. Add `swap_in_bytes_sec: f64` and `swap_out_bytes_sec: f64` to `MemoryMetrics`, computed from delta of `vm_statistics64.pageins/pageouts`.

> SHALL-C1-01 through SHALL-C1-04

#### Scenario: Available fraction
- **WHEN** total=16GB, used=12GB
- **THEN** `mem_available` latest ≈ 0.25

### Requirement: Memory panel Type B layout [I17-C2]
Memory panel SHALL use `split_type_b` when `show_detail` is true: left 37% = used braille graph, middle 37% = available braille graph, right 25% = disk detail. Label row above each sub-graph: `used X.XGB` / `avail X.XGB` in `theme.fg`. Each sub-graph has independent Y-scale (0.0-1.0 fraction). When `show_detail` false: used + available split 50/50. Title: `³mem  X.X/Y.YGB XX%`.

> SHALL-C2-01 through SHALL-C2-05

### Requirement: Memory pressure indicator [I17-C3]
Add `pressure_level: MemoryPressureLevel` (Normal/Warning/Critical) to `MemoryMetrics`. Read from `sysctl kern.memorystatus_level` (1→Normal, 2→Warning, 4→Critical). Fallback heuristic when sysctl fails. Colored dot `●` appears in title bar: green=Normal, yellow=Warning, red=Critical.

> SHALL-C3-01, SHALL-C3-02, SHALL-C3-03

### Requirement: Swap display [I17-C4]
Frame bottom left: `Swap: X.X/Y.Y GB` (only if swap_total > 0). Append I/O rates when non-zero: `in X.X/out X.X MB/s`. If both rates are 0, show only static usage.

> SHALL-C4-01, SHALL-C4-02, SHALL-C4-03

### Requirement: Detail dot styling [I17-C5]
CPU right detail dots SHALL use `•` (U+2022) instead of `●` (U+25CF). Single space between dots: `• • •`.

> SHALL-C5-01, SHALL-C5-02, SHALL-C5-03

### Requirement: Process colored dots [I17-C6]
Process panel header SHALL show colored `•` before each metric column name (cpu/mem/pow accents). Rows SHALL show colored `•` before each metric value. Near-zero metric (< 0.1% cpu, < 1MB mem, < 0.1W power) → `theme.muted` dot color.

> SHALL-C6-01, SHALL-C6-02, SHALL-C6-03

### Requirement: Network label cleanup [I17-C8]
Remove `cur:` prefix — show rate value directly. Rates display as B/s, KB/s, MB/s, GB/s (decimal). Use `total` (full word) instead of `tot`.

> SHALL-C8-01, SHALL-C8-02, SHALL-C8-03, SHALL-C8-04

#### Scenario: Label format
- **WHEN** network detail is rendered
- **THEN** detail text contains "total" not "tot"; no "cur:" prefix

### Requirement: Idle state indicators [I17-C11]
GPU panel: when all visible graph values < 0.5W → show `idle` text centered on graph area in `theme.muted`. Network panel: when all upload < 1 KB/s → `idle` per upload section; same for download. Power panel: GPU power < 0.5W → append gray `(idle)` to GPU label. Idle text does NOT replace the graph.

> SHALL-C11-01, SHALL-C11-02, SHALL-C11-03, SHALL-C11-04

### Requirement: Power sub-frames [I17-C12]
CPU and GPU sub-panels within power panel SHALL get visual `BorderType::Rounded` borders in show mode (detail view). Border color same as parent power panel frame (adaptive dim). No sub-frames in hide/no-detail mode.

> SHALL-C12-01, SHALL-C12-02, SHALL-C12-03

---

## Iteration 23: Panel Structure, Network Overhaul, Color System [I23]

### Requirement: Hide mode chart full width [I23-W0]
CPU and GPU panel hide mode trend charts SHALL use the full available inner width — no reserved empty space on the right side.

> SHALL-23-00a, SHALL-23-00b, SHALL-23-00c

#### Scenario: Full width chart
- **WHEN** CPU is in hide mode
- **THEN** chart width equals inner area width (no right gap)

### Requirement: Sub-panel bordered frames [I23-W1]
Memory panel sub-panels ("used" and "avail") SHALL have `Block::borders(Borders::ALL)` with `BorderType::Rounded` in BOTH show and hide modes. Power panel sub-panels ("cpu" and "gpu") SHALL also have bordered frames in both modes. Sub-panel border color: `dim_color(border_color, 0.8)`. Sub-panel titles display label + current value (e.g., `" used 12.3GB "`).

> SHALL-23-01a through SHALL-23-01d

### Requirement: Footer redesign [I23-W1]
Footer SHALL display keyboard shortcuts right-aligned with bracket key labels (e.g., `[c]`, `[.]`, `[?]`). Panel superscript numbers use `theme.muted` color.

> SHALL-23-03a, SHALL-23-03b

> Note: Superseded by SHALL-26-04 (iteration 26 footer reorder).

### Requirement: Sub-panel title colors [I23-W1]
All sub-panel title text SHALL use `Style::default().fg(theme.fg).bold()` — NOT accent colors. Applies to memory "used"/"avail" and power "cpu"/"gpu" titles.

> SHALL-23-04a

### Requirement: Process column layout [I23-W2]
Process table column order SHALL be: `pid`, `name`, `•cpu`, `•mem`, `•pow`, `thread`. PID is leftmost in `theme.muted`. "thread" column header uses full word. Colored dots use `gradient::value_to_color(normalized, theme)`. Header row has NO dots. Near-zero values use `theme.muted` dot.

> SHALL-23-05a through SHALL-23-05f

#### Scenario: Column order
- **WHEN** process table header is rendered
- **THEN** order is pid, name, cpu, mem, pow, thread

### Requirement: Network border color [I23-W3]
Network panel border SHALL use `theme.net_download` color (red/coral in Horizon), dimmed by `adaptive_border_dim`. Replaces previous `net_upload` (cyan) border.

> SHALL-23-06a

### Requirement: Symmetric center-baseline chart [I23-W3]
Network chart SHALL be symmetric around a center baseline: download grows upward from center (top half), upload grows downward from center (bottom half). `render_graph_downward` mirrors braille graph vertically. In show mode, right detail shows download on top and upload on bottom.

> SHALL-23-07a through SHALL-23-07d

#### Scenario: Chart layout
- **WHEN** network chart is rendered
- **THEN** download occupies top half, upload occupies bottom half

### Requirement: Stable interface list [I23-W3]
Network show mode right detail displays up to 3 interfaces in fixed positions. Empty slots padded with blank lines. Active interfaces (rx > 0 or tx > 0) use `theme.fg`, inactive use `theme.muted`. Infrastructure interfaces filtered.

> SHALL-23-08a through SHALL-23-08d

### Requirement: GPU idle in title bar [I23-W4]
When `power.gpu_w < 0.5`, GPU panel title SHALL show `"(idle)"` in muted style instead of usage%/freq/power. When active, title shows usage%, frequency, and power. No idle text overlay on chart area.

> SHALL-23-09a, SHALL-23-09b, SHALL-23-09c

### Requirement: btop panel colors [I23-W4]
Theme struct SHALL include `process_accent` field. Horizon panel colors match btop: cpu=#B877DB, mem=#27D796, net_download=#E95678, process=#25B2BC. Dracula matches btop values. GPU accent SHALL be the HSL hue midpoint between `cpu_accent` and `mem_accent`. Power accent SHALL be the HSL hue midpoint between `net_download` and `process_accent`.

> SHALL-23-10a through SHALL-23-10e

#### Scenario: GPU accent derivation
- **WHEN** theme is constructed
- **THEN** GPU accent hue is between cpu_accent hue and mem_accent hue (shorter arc on HSL wheel)

### Requirement: Memory formula [I23-W5]
`ram_used` SHALL be clamped: `used = used.min(ram_total)`. Memory available derived as `ram_total - ram_used`.

> SHALL-23-11b, SHALL-23-11d

> Note: Original formula (total - free) superseded by SHALL-AD-01 in iteration 25.

### Requirement: Power panel hide mode [I23-W5]
Power panel hide mode SHALL use bordered 50/50 split with cpu and gpu sub-panels, each with title showing label + watts. Graph fills inner area. GPU sub-panel title shows "(idle)" when `gpu_w < 0.5`.

> SHALL-23-12a through SHALL-23-12d

### Requirement: IOHIDEventSystem temperature fallback [I23-W6]
Temperature collection SHALL try IOHIDEventSystem as fallback when SMC returns no data. Uses `IOHIDServiceClientCopyEvent` with event type 15. Readings averaged across all matched sensors. If neither source returns data, `temperature.available` = false and UI shows "N/A".

> SHALL-23-14a through SHALL-23-14d

### Requirement: Network baseline visibility [I23-W6]
Network symmetric chart SHALL clamp minimum display value so at least 1 braille dot renders at zero. `baseline_floor = scale * 0.005`. Baseline dots use `baseline_color(theme)` (muted but visible). When network is idle (total_rx < 1024 && total_tx < 1024), panel title shows `"(idle)"` in muted style.

> SHALL-23-15a through SHALL-23-15d

> Note: Superseded by SHALL-28-03 (iteration 28 network baseline fix).

### Requirement: Infrastructure interface filter [I23-CC]
`is_infrastructure_interface()` SHALL filter: bridge*, awdl*, llw*, utun*, ipsec*, ap*, gif*, stf*, XHC*.

> SHALL-23-CC-02

---

## Iteration 25: UAT Visual Polish + Battery [I25]

### Requirement: Baseline color function [I25-UAT-01]
`baseline_color(theme: &Theme) -> Color` SHALL return a color with guaranteed visual separation from `theme.bg`. Dark themes (luminance < 0.5): brighten each muted channel by +30, capped at 255. Light themes: darken by 30, floored at 0. Network panel baseline dots SHALL use `baseline_color(theme)` instead of `theme.muted`.

> SHALL-01-01, SHALL-01-02

#### Scenario: Dark theme boost
- **WHEN** theme is Nord (dark bg, muted Rgb(76,86,106))
- **THEN** `baseline_color` returns Rgb(106,116,136) — each channel +30

### Requirement: GPU and power panel baselines [I25-UAT-02]
GPU panel sparkline SHALL apply `baseline_floor = max * 0.005` (= 0.005 since max=1.0). Power panel `render_labeled_sparkline` SHALL apply `baseline_floor = max * 0.005`. Both SHALL use `baseline_color(theme)` for near-zero baseline dots.

> SHALL-02-01, SHALL-02-02, SHALL-02-03

#### Scenario: GPU idle baseline
- **WHEN** GPU usage history is all-zero (128 samples)
- **THEN** panel renders visible baseline dots (not blank)

### Requirement: Tiered network chart scaling [I25-UAT-03]
Network chart scale SHALL be dynamically selected from visible data window maximum using tiers (bytes/sec): 1M, 10M, 100M, 1G. Tier upgrade is immediate. Tier downgrade uses hysteresis. Tier label renders at top-right of chart area in `theme.muted`.

> SHALL-03-01, SHALL-03-02, SHALL-03-03, SHALL-03-04, SHALL-03-05

> Note: Downgrade hysteresis (50%/10 samples) superseded by SHALL-26-06 (10%/128 samples).

#### Scenario: Tier selection
- **WHEN** visible window max = 500,000 bytes/sec
- **THEN** tier is 1 MB/s; label shows "1 MB/s"

### Requirement: Theme-specific gradient colors [I25-UAT-04]
`gradient::value_to_color` SHALL accept `theme: &Theme` and read from `theme.gradient_green/yellow/orange/red` instead of the module-level `STOPS` constant. `temp_to_color` SHALL also use theme gradient stops. The `STOPS` constant SHALL be removed. All 22 call sites SHALL be updated to pass theme.

> SHALL-04-01 through SHALL-04-07

#### Scenario: Theme gradient stops
- **WHEN** `value_to_color(0.0, &HORIZON)` is called
- **THEN** returns Rgb(39,215,150) (Horizon gradient_green)

#### Scenario: Per-theme stops
- **WHEN** any of the 10 themes is used
- **THEN** gradient_green values are distinct across themes

### Requirement: Battery gauge in header [I25-UAT-05]
New `platform/battery.rs` SHALL collect battery metrics via `IOPSCopyPowerSourcesInfo`, `IOPSCopyPowerSourcesList`, `IOPSGetPowerSourceDescription`. `MetricsSnapshot` SHALL gain `battery: BatteryMetrics` (charge_pct: f32, is_charging: bool, is_present: bool, is_on_ac: bool). `charge_pct` clamped to 0..=100 at collection. Header includes battery indicator right-aligned.

> SHALL-05-01 through SHALL-05-07

> Note: Gauge format superseded by SHALL-26-02.

#### Scenario: No battery
- **WHEN** `is_present == false`
- **THEN** header renders `"⚡ AC"` in muted style

### Requirement: Memory formula correction [I25-AD-01]
`ram_used` SHALL be computed as `(active + wire) * page_size`. Matches btop formula. `ram_used` SHALL be clamped to `ram_total`. Code SHALL include comment: "Matches btop formula. Excludes compressed/inactive/speculative."

> SHALL-AD-01a, SHALL-AD-01b, SHALL-AD-01c

> Supersedes SHALL-23-11a (total - free formula).

#### Scenario: Formula
- **WHEN** active=1M pages, wire=500K pages, page_size=16384
- **THEN** `ram_used = 1.5M * 16384` bytes

### Requirement: Reverse theme cycling [I25-AD-02]
A test SHALL verify: starting at index 0, pressing Capital C cycles to the last theme using `(current + THEMES.len() - 1) % THEMES.len()`.

> SHALL-AD-02a, SHALL-AD-02b

### Requirement: Remove outer panel padding [I25-UAT-07]
For panels WITHOUT sub-panels (CPU, GPU, Network, Process): `raw_inner.y + 1` SHALL change to `raw_inner.y` (remove 1-line gap). For panels WITH sub-panels (Memory, Power): same removal. All 6 panels SHALL render without panic at 80x24.

> SHALL-07-01, SHALL-07-02, SHALL-07-03, SHALL-07-04

### Requirement: Right detail column padding [I25-UAT-08]
`layout::split_type_a` gap constraint SHALL change from `Length(1)` to `Length(2)`. `layout::split_type_b` SHALL apply the same change to both gap constraints.

> SHALL-08-01, SHALL-08-02

#### Scenario: Gap width
- **WHEN** `split_type_a` on 100-wide area
- **THEN** chart=73, gap=2, detail=25

### Requirement: Header text all muted [I25-UAT-09]
ALL header text spans SHALL use `Style::default().fg(theme.muted)` — no accent color, no bold, no `theme.fg`. Applies to timestamp, "mtop", chip_info in all 3 width modes.

> SHALL-09-01, SHALL-09-02

---

## Iteration 26: Battery Gauge + Avail Graph + Footer [I26]

### Requirement: Available memory graph green hue [I26]
`render_graph_green()` SHALL render all braille dots in a single green hue from `theme.gradient_green`, varying by row: bottom rows at 0.6x brightness, top rows at 1.0x, linearly interpolated. Memory panel SHALL call `render_graph_green()` for "avail" sub-panel in BOTH show and hide modes. "used" sub-panel continues using standard `render_graph()`. Fallback base color for non-Rgb gradient_green: Rgb(80,200,120).

> SHALL-26-01a through SHALL-26-01d

#### Scenario: Green hue variance
- **WHEN** avail graph renders with Rgb gradient_green
- **THEN** bottom row color brightness ≈ 0.6x, top row ≈ 1.0x

### Requirement: Battery gauge rewrite [I26]
Battery gauge SHALL fill from RIGHT: empty cells on left (muted color), filled cells on right (gradient color). Color: `value_to_color(1.0 - pct, theme)` — full battery = green, empty = red. Filled cells have same-hue gradient: leftmost at 1.3x brightness, rightmost at 0.8x. Percentage appears exactly once after gauge. Gauge bar width: 6 characters. When charging: prepend `⚡` in muted style. When no battery: display `⚡ AC` in muted.

> SHALL-26-02a through SHALL-26-02h

> Supersedes iter25 SHALL-05-05/06/07.

#### Scenario: At 100% charge
- **WHEN** `charge_pct=100, is_present=true, is_charging=false`
- **THEN** 6 filled cells, green-ish color, percentage shows "100%" exactly once

#### Scenario: No battery
- **WHEN** `is_present=false`
- **THEN** renders "⚡ AC"

### Requirement: GPU core count removal from detail [I26]
GPU panel right detail SHALL display: ANE power (W), DRAM power (W), blank line, VRAM usage (used/total GB). SHALL NOT include GPU core count. GPU panel bottom row SHALL continue to display core count (`N cores`) in BOTH show and hide modes.

> SHALL-26-03a, SHALL-26-03b

### Requirement: Footer reorder [I26]
Footer SHALL render as a single right-aligned line: `[?] help`, `[c] theme({name})`, `[.] detail`, `[+/-] {N}ms`. All footer text uses `theme.muted`. `Alignment::Right` for entire line.

> SHALL-26-04a through SHALL-26-04d

> Supersedes iter23 SHALL-23-03 and iter25 SHALL-AD-03.

#### Scenario: Footer content
- **WHEN** footer is rendered
- **THEN** line contains all 4 items in correct left-to-right order

### Requirement: Interval preset ladder [I26]
`+`/`=` keys advance `interval_ms` to next higher preset: [100, 250, 500, 750, 1000, 1500, 2000, 3000, 5000, 10000]. `-` key moves to next lower preset. 1000ms is always reachable.

> SHALL-26-05a through SHALL-26-05d

#### Scenario: Preset steps
- **WHEN** interval is 1000ms and user presses `+`
- **THEN** interval becomes 1500ms

### Requirement: Net tier hysteresis full chart window [I26]
Network tier upgrade SHALL be instant. Tier downgrade requires `max_len` (128) consecutive samples where max(upload+download history) < 10% of current tier threshold. `net_tier_hold` resets to 0 if any sample exceeds the threshold. Tiers: [1M, 10M, 100M, 1G] bytes/sec.

> SHALL-26-06a through SHALL-26-06e

> Supersedes iter25 SHALL-03-03 (50%/10 samples).

#### Scenario: Downgrade hysteresis
- **WHEN** 127 samples below threshold
- **THEN** still at current tier; 128th sample triggers downgrade

### Requirement: Memory swap in hide mode [I26]
Memory panel hide mode bottom row SHALL display swap info left-aligned (`Swap: X.X/Y.YGB`) if swap_total > 0, and disk info right-aligned. Both visible without overlap. When no swap configured, only disk info shown right-aligned. Show mode bottom row continues showing swap only.

> SHALL-26-07a through SHALL-26-07e

### Requirement: Battery lightning on AC power [I28-W0]
The battery gauge SHALL display the ⚡ symbol whenever `is_on_ac` is true, regardless of whether `is_charging` is true or false.

> SHALL-28-01

#### Scenario: AC power connected, not charging
- **WHEN** `is_on_ac=true, is_charging=false, charge_pct=100`
- **THEN** the battery display SHALL show `⚡ [gauge] 100%`

#### Scenario: AC power connected, charging
- **WHEN** `is_on_ac=true, is_charging=true, charge_pct=62`
- **THEN** the ⚡ symbol SHALL be present

#### Scenario: On battery
- **WHEN** `is_on_ac=false`
- **THEN** no ⚡ symbol SHALL be shown

### Requirement: Dynamic history buffer [I28-W1]
History buffer capacity SHALL be dynamically sized to `terminal_width * 2` samples, ensuring braille charts can fill the full available width (each braille char = 2 data points). Minimum buffer capacity SHALL be 128 samples.

> SHALL-28-02. Supersedes fixed 128-sample buffer.

#### Scenario: Buffer sizing
- **WHEN** terminal width is 120 columns
- **THEN** buffer capacity SHALL be >= 240

#### Scenario: Buffer growth preserves data
- **WHEN** buffer grows from 128 to 240
- **THEN** existing 128 samples SHALL be preserved

#### Scenario: Buffer never shrinks
- **WHEN** terminal shrinks from 120 to 80 columns
- **THEN** buffer capacity SHALL stay at 240 (no shrink)

#### Scenario: All buffers resize together
- **WHEN** terminal is resized
- **THEN** all history buffers SHALL have the same capacity: cpu_usage, gpu_usage, cpu_power, gpu_power, net_upload, net_download, per-interface buffers, memory_used, memory_avail

#### Scenario: Minimum buffer
- **WHEN** terminal width is 30 columns
- **THEN** buffer capacity SHALL be 128 (minimum)

### Requirement: Network baseline fix [I28-W1]
Network baseline floor SHALL be calculated as `scale / (chart_height_rows * 4 * 2)` where `chart_height_rows` is the actual braille chart height in terminal rows. The `is_baseline` check SHALL compare against `baseline_floor`, not a hardcoded literal.

> SHALL-28-03. Supersedes SHALL-23-15a (`baseline_floor = scale * 0.005`).

#### Scenario: Baseline produces visible dot
- **WHEN** chart is 4 rows with 10MB/s scale
- **THEN** baseline_floor SHALL be >= 312500 (produces 1 braille dot)

#### Scenario: Baseline check uses floor
- **WHEN** checking if a value is baseline
- **THEN** `is_baseline` SHALL use `baseline_floor` threshold, not hardcoded `1.0`

#### Scenario: Idle network shows baseline
- **WHEN** network is idle (all values near zero)
- **THEN** the chart SHALL show visible baseline dots using `baseline_color(theme)`

### Requirement: Network scale label in title [I28-W1]
Network panel title SHALL include the current scale tier in parentheses using the `100%=` format: e.g., `⁴ Network (100%=10MB/s)`. Scale label text SHALL use `theme.muted` color.

> SHALL-28-04. Supersedes previous top-right muted scale display.

#### Scenario: Scale label in title
- **WHEN** current tier is 10MB/s
- **THEN** network title SHALL contain `(100%=10MB/s)`

#### Scenario: Scale label color
- **WHEN** scale label is rendered
- **THEN** the `(100%=XMB/s)` portion SHALL use `theme.muted` color

### Requirement: Network tier hysteresis [I28-W1]
Network scale tiers SHALL be: 1, 5, 10, 50, 100, 500, 1000 MB/s. Upward tier jump SHALL be delayed: traffic must exceed the current tier for 10 consecutive samples before the tier jumps. Downward tier jump retains existing behavior (full buffer window).

> SHALL-28-05. Supersedes SHALL-26-06 tier set [1, 10, 100, 1000].

#### Scenario: Tier set
- **WHEN** network scaling is active
- **THEN** tier set SHALL be [1, 5, 10, 50, 100, 500, 1000] MB/s

#### Scenario: Delayed upward jump
- **WHEN** 9 samples exceed current tier
- **THEN** no jump occurs; 10th consecutive sample above tier triggers jump to smallest tier containing the buffer-wide maximum

#### Scenario: Hold counter reset
- **WHEN** 8 samples above tier, 1 below, then 8 above
- **THEN** counter resets on the below-threshold sample, no jump

#### Scenario: Silent cap during delay
- **WHEN** values exceed current tier during the 10-sample delay
- **THEN** values SHALL be capped at 100% of current tier

### Requirement: Network upload/download overlay labels [I28-W1]
Network hide mode chart SHALL render upload rate overlay at top-left (`↑ XX.X MB/s`) and download rate overlay at bottom-left (`↓ XX.X MB/s`) in `theme.muted` color. Overlays render on top of braille chart data.

> SHALL-28-06

#### Scenario: Overlay position
- **WHEN** chart area height >= 6 rows
- **THEN** upload overlay at top-left, download overlay at bottom-left

#### Scenario: Small chart suppression
- **WHEN** chart area height < 6 rows
- **THEN** no overlay labels SHALL render

### Requirement: Filter zero-watt processes [I28-W2]
Power panel show mode detail process list SHALL filter out processes with `power_w` that rounds to 0.0W (i.e., `power_w < 0.05`).

> SHALL-28-07

#### Scenario: All processes zero
- **WHEN** all processes are at 0.0W
- **THEN** process list area SHALL be empty (blank space, no header, no rows)

#### Scenario: Some processes qualify
- **WHEN** 2 processes at 1.5W and 8 at 0.0W
- **THEN** only the 2 qualifying processes SHALL be shown, sorted by power descending

### Requirement: Help popup frame color [I28-W3]
Help popup border/frame SHALL use `theme.fg` color with `BorderType::Rounded`.

> SHALL-28-08

#### Scenario: Help popup styling
- **WHEN** help popup is displayed
- **THEN** border color SHALL be `theme.fg` and border type SHALL be `Rounded`

### Requirement: Remove expanded tag from titles [I28-W4]
All 6 expanded panel titles SHALL NOT contain the text `[expanded]`. All SHALL use the superscript number prefix consistent with hide/show mode.

> SHALL-28-09

#### Scenario: Expanded title format
- **WHEN** any panel is in expanded mode
- **THEN** title SHALL NOT contain `[expanded]` and SHALL start with superscript number (¹-⁶)

### Requirement: Footer expand hint [I28-W4]
Footer SHALL include `[1-6] expand` hint positioned after the `[.] detail` hint. Footer SHALL not overflow at 80 columns.

> SHALL-28-10

#### Scenario: Footer content
- **WHEN** footer is rendered
- **THEN** footer SHALL contain `[1-6] expand` after `[.] detail`

### Requirement: CPU expanded polish [I28-W4]
CPU expanded panel SHALL show both E-cluster and P-cluster sections with headers, matching the show mode detail layout.

> SHALL-28-11

#### Scenario: CPU expanded clusters
- **WHEN** CPU panel is expanded
- **THEN** both "E-cluster" and "P-cluster" headers SHALL be visible with cores grouped under respective headers

### Requirement: GPU expanded polish [I28-W4]
GPU expanded panel SHALL NOT display "GPU Cores: N" or system RAM "Memory Used/Total". GPU expanded title SHALL show "(idle)" in muted style when `power.gpu_w < 0.5`.

> SHALL-28-12

#### Scenario: GPU expanded cleanup
- **WHEN** GPU panel is expanded
- **THEN** "GPU Cores" and "Memory Used/Total" SHALL NOT appear

#### Scenario: GPU idle in expanded
- **WHEN** `power.gpu_w < 0.5` in expanded view
- **THEN** title SHALL contain "(idle)" in muted style

### Requirement: Memory expanded theme colors [I28-W4]
Memory pressure stacked gauge SHALL use theme accent colors: Wired = `theme.cpu_accent`, App = `theme.mem_accent`, Compressed = `theme.power_accent`. No hardcoded `Color::Red`, `Color::Blue`, or `Color::Yellow`.

> SHALL-28-13

#### Scenario: Pressure gauge colors
- **WHEN** memory pressure gauge is rendered in expanded mode
- **THEN** Wired uses `theme.cpu_accent`, App uses `theme.mem_accent`, Compressed uses `theme.power_accent`

### Requirement: Network expanded polish [I28-W4]
Network expanded border SHALL use `theme.net_download` color. Network expanded sparklines SHALL use gradient coloring via `value_to_color`.

> SHALL-28-14

#### Scenario: Network expanded theming
- **WHEN** network panel is expanded
- **THEN** border SHALL use `theme.net_download` and sparklines SHALL use gradient colors

### Requirement: Process expanded polish [I28-W4]
Process expanded border SHALL use `theme.process_accent`. Process expanded SHALL display gradient-colored dots (•) before cpu, mem, and power values. Header SHALL use "thread" (not "Thr"). Mem and power value columns SHALL use gradient coloring.

> SHALL-28-15

#### Scenario: Process expanded styling
- **WHEN** process panel is expanded
- **THEN** border uses `theme.process_accent`, header contains "thread", rows contain gradient "•" dots, power values use gradient color

## Non-Functional Requirements

### Resource Safety
The TUI SHALL restore terminal state (raw mode, alternate screen, cursor visibility) on exit via all exit paths: normal quit (`q`/`Esc`), SIGINT/SIGTERM, and panic. [T1-static]

### Performance
The TUI render cycle SHALL complete in under 16ms (60fps budget) to avoid visible flicker or input lag. [T2-soak]

### Correctness
All displayed numeric values SHALL match the underlying `MetricsSnapshot` data within rounding tolerance. The TUI SHALL NOT apply its own scaling or unit conversion beyond display formatting (e.g., bytes to GB). [T1-static]

### Resilience
The TUI SHALL NOT crash or hang when receiving malformed or partial metrics data. Missing fields in `MetricsSnapshot` SHALL result in "N/A" display, not a panic. [T1-static]

### Security
The TUI SHALL NOT write any data to disk during normal operation. No log files, temp files, or state files SHALL be created. [T1-static]

### Longevity
The TUI layout SHALL gracefully handle new metric categories added in future versions by reserving space or using a scrollable panel area. New metrics SHALL be addable without restructuring the entire layout. [T3-long]

## Iteration 29: Network & Power Polish, Expanded Padding, Label Casing [I29]

### Requirement: Network panel title cleanup [I29-W0]
Network panel title SHALL NOT contain ↑/↓ rate text when active; format SHALL be `⁴ net (100%=XMB/s)`. When idle, title SHALL show `(idle)`.

> SHALL-29-01a, SHALL-29-01b

### Requirement: Remove top-right scale label from chart area [I29-W0]
`render_tier_label` function and all call sites SHALL be removed from network.rs. Scale is shown exclusively in the frame title; no scale label SHALL render inside the chart area.

> SHALL-29-02a, SHALL-29-02b

### Requirement: Fix overlay label swap [I29-W0]
Top-left overlay SHALL show download rate (`↓ XX.X MB/s`, matching chart top half). Bottom-left overlay SHALL show upload rate (`↑ XX.X MB/s`, matching chart bottom half).

> SHALL-29-03a, SHALL-29-03b

### Requirement: Rate suffix in compact formatter [I29-W1]
`format_bytes_rate_compact` SHALL append `/s` to its output (e.g., `"2.4M/s"`, `"350K/s"`, `"42B/s"`). All rate displays using this function SHALL show `/s` consistently.

> SHALL-29-04a, SHALL-29-04b

### Requirement: Active interfaces heading in network right detail [I29-W1]
Network show mode right detail SHALL display `■ active` heading (with `theme.fg` dot color) before the interface list. When no interface has activity (all rx=0 and tx=0), the entire interface section SHALL be hidden. Only interfaces with rx > 0 or tx > 0 SHALL appear.

> SHALL-29-05a through SHALL-29-05c

### Requirement: Power panel empty process list overlay [I29-W2]
When `procs_by_power` is empty (all filtered by 0.05W threshold) in show mode right detail, render vertically and horizontally centered muted "idle" text using `theme.muted` in the right area.

> SHALL-29-06a, SHALL-29-06b

### Requirement: 1-character padding in all expanded panels [I29-W3]
All 6 expanded panel renderers SHALL apply 1-character padding on all 4 sides between the panel border and content. All expanded panels SHALL render without panic at 80x24 minimum terminal size after padding is applied.

> SHALL-29-07a through SHALL-29-07c

### Requirement: Lowercase label audit [I29-W4]
All display labels SHALL be lowercase. Only uppercase allowed: units (MB, GB, MHz, RPM, W) and acronyms (ANE, VRAM, DRAM, GPU, CPU). Violations in expanded.rs, network.rs, and power.rs SHALL be corrected (e.g., "Frequency:" → "frequency:", "Memory Pressure" → "memory pressure", "Component Breakdown" → "component breakdown", "Top Processes by Power" → "top processes by power").

> SHALL-29-08a through SHALL-29-08d

### Requirement: Scale label at top-right of network frame border [I29-UAT]
Network panel frame SHALL display the scale label (`100%=XMB/s`) at the top-right of the border, right-aligned, using `theme.muted` color. Label SHALL NOT have parentheses.

> SHALL-29-UAT-01a through SHALL-29-UAT-01c

### Requirement: Active heading symbol [I29-UAT]
Network right detail active heading SHALL use `■` (square box) prefix, not `•` (dot).

> SHALL-29-UAT-02a

### Requirement: Power idle text centering [I29-UAT]
Power panel "idle" text SHALL be both vertically and horizontally centered in the right detail area using `Alignment::Center`.

> SHALL-29-UAT-03a

### Requirement: Casing rules for titles vs body content [I29-UAT]
Panel titles, sub-panel titles, and table headers SHALL always use lowercase, even for acronyms (e.g., `cpu`, `gpu`, `name`, `mem`, `power`). Acronyms in body content and gauge labels SHALL use uppercase (e.g., `GPU power:`, `ANE`, `DRAM`, `VRAM`, `RAM`). All non-acronym words in body content SHALL use lowercase, never capitalizing the first letter.

> SHALL-29-UAT-04a through SHALL-29-UAT-04c

### Cross-cutting: Rendering-only, no data model changes [I29-CC]
All iteration 29 changes are rendering-only. No changes to MetricsHistory, MetricsSnapshot, or AppState structs. All changes SHALL render without panic at 80x24 terminal size.

> SHALL-29-CC-01, SHALL-29-CC-02, SHALL-29-UAT-CC-01

---

## Iteration 31: Panel Polish & Hide-Mode Completeness [I31]

### Requirement: Power panel show-mode title consistency [I31-F1]
In show_detail mode, the power panel's cpu and gpu sub-frames SHALL display their labels in the sub-frame `.title()` using `theme.fg`, not as colored body text. This matches hide-mode convention.

> SHALL-31-01a, SHALL-31-01b

### Requirement: GPU hide-mode DRAM wattage [I31-F2]
The GPU panel's hide-mode bottom row SHALL display DRAM wattage alongside ANE wattage.

> SHALL-31-02a

### Requirement: Memory hide-mode disk I/O rates [I31-F3]
The memory panel's hide-mode bottom row SHALL display disk read and write I/O rates using compact format.

> SHALL-31-03a

### Requirement: Network hide-mode session max rates [I31-F4]
The network panel's hide-mode bottom row SHALL display session max download and upload rates using compact format.

> SHALL-31-04a

### Requirement: Expanded panel padding fix [I31-F5]
All 6 expanded panels SHALL use `raw_inner.y` (no extra top padding) and `raw_inner.height` (no height reduction), matching hide-mode convention. Left/right 1-char padding SHALL be preserved.

> SHALL-31-05a, SHALL-31-05b

### Requirement: Expanded network dynamic tier [I31-F6]
The expanded network panel SHALL use `state.history.net_tier_idx` for all sparkline scales, not `speed_tier_from_baudrate`.

> SHALL-31-06a

### Requirement: Network tier immediate upgrade [I31-F7]
When max network value exceeds the current tier, the tier SHALL immediately jump to the appropriate tier (no 10-sample delay). Downgrade delay SHALL be preserved unchanged.

> SHALL-31-07a, SHALL-31-07b

### Requirement: CPU expanded multi-row braille chart [I32-F1]
The CPU expanded panel SHALL render a multi-row braille chart of cpu_usage history using `render_graph`, occupying ~30% of panel height (minimum 3 rows), replacing the single-row sparkline.

> SHALL-32-01a, SHALL-32-01b

### Requirement: Memory expanded usage trend chart [I32-F2]
The memory expanded panel SHALL render a multi-row braille chart of mem_usage history at the top of the panel before gauges, occupying ~25% of panel height (minimum 2 rows). Pressure and disk sections SHALL be offset below the chart and gauges.

> SHALL-32-02a, SHALL-32-02b, SHALL-32-02c

### Requirement: Network expanded symmetric chart [I32-F3]
The network expanded panel SHALL render a symmetric center-baseline chart (download top half growing upward, upload bottom half growing downward) with baseline coloring, replacing separate single-row sparklines. The chart SHALL occupy ~50% of panel height, with per-interface table below.

> SHALL-32-03a, SHALL-32-03b, SHALL-32-03c

### Requirement: GPU expanded multi-row braille chart [I33-F1]
The GPU expanded panel SHALL render a multi-row braille chart of gpu_usage history using `render_graph`, occupying ~30% of panel height (minimum 3 rows), replacing the single-row sparkline.

> SHALL-33-01a, SHALL-33-01b

### Requirement: Power expanded multi-row braille charts [I33-F2]
The power expanded panel SHALL render multi-row braille charts for both cpu_power and gpu_power histories, each ~20% of panel height (minimum 2 rows), replacing single-row sparklines. Component breakdown and fan/process sections SHALL be positioned below both charts.

> SHALL-33-02a, SHALL-33-02b, SHALL-33-02c

### Requirement: Process expanded sort indicator and wider name [I33-F3]
The process expanded panel SHALL display sort mode in the header, use a 24-character name column (wider than hide-mode), and include a thread count column.

> SHALL-33-03a, SHALL-33-03b, SHALL-33-03c

### Requirement: Remove unused re-export [I34-F1]
The `render_graph_with_baseline` re-export from `panels/mod.rs` SHALL be removed to eliminate compiler warning.

> SHALL-34-05a

### Requirement: 80x24 expanded panel test coverage [I34-F2]
All 6 expanded panels SHALL have 80x24 render-without-panic tests.

> SHALL-34-F2

### Requirement: Expanded panel content verification [I34-F3]
Expanded panel render tests SHALL verify content presence (titles, labels, metrics) beyond empty-string checks.

> SHALL-34-F3

### Requirement: Clippy clean build [I35-F1]
The codebase SHALL produce zero clippy warnings.

> SHALL-35-01a

### Requirement: Network expanded chart color symmetry [I35-F2]
The expanded network symmetric chart SHALL use the same `gradient::value_to_color` coloring strategy for both download (top half) and upload (bottom half), ensuring visual symmetry.

> SHALL-35-02a

### Requirement: Memory expanded layout — mem gauge at top [I36-F1a]
The expanded memory panel SHALL render a full-width memory usage gauge (`render_compact_gauge`) at the top of the memory group, showing overall RAM usage percentage before the chart grid.

> SHALL-36-F1a

### Requirement: Memory expanded layout — 2×2 chart grid [I36-F1b]
The expanded memory panel SHALL render a 2×2 grid of braille sparkline charts below the memory gauge: used (top-left), available (top-right), cached (bottom-left), free (bottom-right). Each sub-chart SHALL have its own titled border and current value label.

> SHALL-36-F1b

### Requirement: Memory expanded layout — compressed and swap text [I36-F1c]
Compressed memory and swap SHALL appear as text metrics below the 2×2 chart grid within the memory group (e.g. `compressed: 2.1GB  swap: 0.0/4.0GB`).

> SHALL-36-F1c

### Requirement: Memory expanded layout — visual separator [I36-F1d]
A horizontal visual separator line SHALL divide the memory group from the disk group in the expanded memory panel.

> SHALL-36-F1d

### Requirement: Disk expanded layout — disk gauge at top of disk group [I36-F1e]
The disk group SHALL begin with a `disk` title, a full-width disk usage gauge (`render_compact_gauge`) showing disk usage percentage, and a size label (e.g. `240/500 GB`), matching the style of the non-expanded right-detail disk gauge.

> SHALL-36-F1e

### Requirement: Disk expanded layout — symmetric chart direction [I37-F1]
The disk symmetric chart SHALL be symmetric around a center baseline, matching the network panel pattern: write ↓ on the top half grows UPWARD from the center baseline (`render_braille_graph`), read ↑ on the bottom half grows DOWNWARD from the center baseline (`render_braille_graph_down`). A muted baseline SHALL be rendered at the midpoint between halves.

> SHALL-37-F1

### Requirement: Disk expanded layout — arrow label convention [I36-F1g]
The disk symmetric chart SHALL label write with ↓ and read with ↑, consistent with the network panel convention where download=↓ and upload=↑. Write ↓ (data to disk, like download) occupies the top half; read ↑ (data from disk, like upload) occupies the bottom half.

> SHALL-36-F1g

### Requirement: Expanded panel chart height parity [I36-F2]
All expanded panels SHALL allocate chart height proportional to the non-expanded (hide-mode) layout (~70% for CPU/GPU, ~45% for memory 2×2 grid, ~60% for network symmetric chart) so expanded charts are not significantly shorter than their non-expanded counterparts.

> SHALL-36-F2

### Requirement: Memory metrics cached and free fields [I36-F3]
MemoryMetrics SHALL include `cached` (inactive + purgeable pages × page_size) and `free` (free_count × page_size) fields populated from macOS vm_statistics64, with corresponding history buffers for sparkline rendering.

> SHALL-36-F3

### Requirement: Disk I/O history buffers [I36-F4]
MetricsHistory SHALL include `disk_read` and `disk_write` history buffers tracking per-sample disk read/write bytes-per-second for use in disk sparkline charts.

> SHALL-36-F4

### Requirement: Disk chart labels with rate values [I37-F3]
The disk symmetric chart write label SHALL include the current write rate (e.g. `↓write 1.2M/s`) and the read label SHALL include the current read rate (e.g. `↑read 500K/s`). The read label SHALL be positioned at the BOTTOM of the chart, not at the midline.

> SHALL-37-F3

### Requirement: Expanded panel naming consistency [I37-F5]
All expanded panel titles SHALL use the same short names as non-expanded panels: `mem` (not `memory`), `net` (not `network`), `proc` (not `processes`).

> SHALL-37-F5

### Requirement: Memory expanded title parity [I37-F4]
The expanded memory panel title SHALL include usage percentage and pressure dot, matching the non-expanded title format.

> SHALL-37-F4

### Requirement: Memory sub-chart adaptive units [I37-F6a]
Memory sub-chart value labels SHALL adapt to MB when the value is less than 1GB, matching non-expanded behavior.

> SHALL-37-F6a

### Requirement: Swap I/O rates in expanded [I37-F6b]
The expanded memory panel swap text SHALL include swap I/O rates (in/out bytes/sec) when available, matching non-expanded detail mode.

> SHALL-37-F6b

## Iteration 38 — CPU + GPU expanded panel parity

### Requirement: CPU expanded title frequency [I38-F1]
The expanded CPU panel title SHALL include `@ {MHz}MHz` showing the max cluster frequency (`p_cluster.freq_mhz.max(e_cluster.freq_mhz)`), matching non-expanded.

> SHALL-38-F1

### Requirement: CPU expanded chart height cap [I38-F2]
The expanded CPU panel chart height SHALL be capped at a maximum of 20 rows to prevent excessive vertical stretch in full-screen mode.

> SHALL-38-F2

### Requirement: CPU expanded vertical centering [I38-F3]
The expanded CPU panel SHALL vertically distribute content so chart and core bars do not cluster at the top with empty space below.

> SHALL-38-F3

### Requirement: CPU expanded temp N/A [I38-F4]
The expanded CPU panel SHALL display `"N/A"` in muted color when temperature is unavailable, matching non-expanded behavior.

> SHALL-38-F4

### Requirement: CPU expanded temp color function [I38-F5]
The expanded CPU panel SHALL use `gradient::temp_to_color()` for temperature coloring, matching non-expanded (not `helpers::temp_color()`).

> SHALL-38-F5

### Requirement: Expanded superscript styling [I38-F6]
All expanded panel superscripts SHALL be rendered as a separate muted-colored span using `PANEL_SUPERSCRIPTS[n]`, matching non-expanded pattern (not hardcoded inside the accent title string).

> SHALL-38-F6

### Requirement: GPU expanded title frequency and power [I38-F7]
The expanded GPU panel title SHALL include `@ {MHz}MHz  {:.1}W` when active, matching non-expanded.

> SHALL-38-F7

### Requirement: GPU expanded idle no percentage [I38-F8]
The expanded GPU panel SHALL display `"(idle)"` without percentage when GPU power < 0.5W, matching non-expanded.

> SHALL-38-F8

### Requirement: GPU expanded chart-metrics margin [I38-F9]
The expanded GPU panel SHALL include a 1-row margin between the braille chart and the metrics table.

> SHALL-38-F9

### Requirement: GPU expanded VRAM metric [I38-F10]
The expanded GPU panel metrics table SHALL include a VRAM line showing `"VRAM {used}/{total}GB"`, matching non-expanded.

> SHALL-38-F10

### Requirement: GPU expanded baseline graph renderer [I38-F11]
The expanded GPU panel SHALL use `render_graph_with_baseline` (not plain `render_graph`), matching non-expanded baseline dot behavior.

> SHALL-38-F11

### Requirement: GPU expanded power precision [I38-F12]
The expanded GPU panel power display SHALL use `{:.1}W` precision, matching non-expanded (not `{:.2} W`).

> SHALL-38-F12

### Requirement: GPU expanded temp N/A [I38-F13]
The expanded GPU panel SHALL display `"N/A"` in muted color when temperature is unavailable, matching non-expanded.

> SHALL-38-F13

## Iteration 39 — Network expanded panel parity

### Requirement: Network expanded border dimming [I39-F1]
The expanded network panel border SHALL use `dim_color(theme.net_download, adaptive_border_dim(theme))`, matching non-expanded.

> SHALL-39-F1

### Requirement: Network expanded scale label [I39-F2]
The expanded network panel SHALL display the current tier scale label (`100%={scale}`) as a right-aligned title, matching non-expanded.

> SHALL-39-F2

### Requirement: Network expanded idle detection [I39-F3]
The expanded network panel SHALL display `"(idle)"` in the title when both rx and tx are below 1024 bytes/sec, matching non-expanded.

> SHALL-39-F3

### Requirement: Network expanded baseline floor [I39-F4]
The expanded network panel baseline floor SHALL use `scale * 0.035`, matching non-expanded (not `scale * 0.005`).

> SHALL-39-F4

### Requirement: Network expanded title rate format [I39-F5]
The expanded network panel title rates SHALL use `format_bytes_rate_compact`, matching the compact style used throughout the UI.

> SHALL-39-F5

### Requirement: Network expanded max rates [I39-F6]
The expanded network panel SHALL display max download and upload rates, matching non-expanded bottom bar pattern.

> SHALL-39-F6

### Requirement: Network expanded chart height cap [I39-F7]
The expanded network panel symmetric chart height SHALL be capped at 20 rows maximum, matching the CPU/GPU cap for very tall terminals.

> SHALL-39-F7

## Iteration 40 — Power expanded panel parity

### Requirement: Power expanded available guard [I40-F1]
The expanded power panel SHALL display `"power sensors: N/A"` when `!s.power.available`, matching non-expanded.

> SHALL-40-F1

### Requirement: Power expanded baseline graph [I40-F2]
The expanded power panel charts SHALL use `render_graph_with_baseline` (not plain `render_graph`), matching non-expanded baseline dot behavior.

> SHALL-40-F2

### Requirement: Power expanded border dimming [I40-F3]
The expanded power panel border SHALL use `dim_color(theme.power_accent, adaptive_border_dim(theme))`, matching non-expanded.

> SHALL-40-F3

### Requirement: Power expanded precision [I40-F4]
The expanded power panel component breakdown SHALL use `{:.1}W` precision, matching non-expanded (not `{:.2}W`).

> SHALL-40-F4

### Requirement: Power expanded process width [I40-F5]
The expanded power panel process list name width SHALL be dynamically computed from available width, matching non-expanded pattern.

> SHALL-40-F5

### Requirement: Power expanded section padding [I40-F6]
The expanded power panel SHALL include 1-row padding between chart sections and between breakdown and process list.

> SHALL-40-F6

### Requirement: Power expanded avg/max [I40-F7]
The expanded power panel SHALL display avg and max power alongside total, matching non-expanded bottom bar pattern.

> SHALL-40-F7

### Requirement: Power expanded chart height cap [I40-F8]
The expanded power panel CPU and GPU chart heights SHALL each be capped at 10 rows maximum.

> SHALL-40-F8
