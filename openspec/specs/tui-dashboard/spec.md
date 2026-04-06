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
