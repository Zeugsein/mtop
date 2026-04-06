# tui-dashboard Specification

## Purpose
Define requirements for the terminal user interface dashboard displaying real-time system metrics with keyboard navigation and theming.
## Requirements
### Requirement: Multi-panel dashboard layout
The TUI SHALL display a multi-panel dashboard with sections for CPU, GPU, power, temperature, memory, network, and process list. Panels SHALL be arranged to maximize information density while remaining readable at 80x24 minimum terminal size.

#### Scenario: Default TUI launch
- **WHEN** the user runs `mtop` with no subcommand
- **THEN** the terminal SHALL display a dashboard with CPU panel (left), power + temperature + memory + network panels (right), and process list (bottom)

#### Scenario: Minimum terminal size
- **WHEN** the terminal is 80 columns by 24 rows
- **THEN** the dashboard SHALL render without overflow or corruption

### Requirement: CPU core visualization
The TUI SHALL display per-core CPU utilization as horizontal bar charts with percentage labels. Bars SHALL be color-coded by utilization level. Each core SHALL be labeled with its type (E for efficiency, P for performance) and index.

#### Scenario: Per-core bars rendering
- **WHEN** CPU metrics are available
- **THEN** each core SHALL display as a labeled bar (e.g., "E0 [████░░░░] 42%") with color coding: green < 30%, cyan 30-40%, yellow 40-60%, red > 60%

#### Scenario: Cluster summary
- **WHEN** CPU metrics are available
- **THEN** the CPU panel SHALL show aggregate CPU usage percentage and total CPU power in Watts

### Requirement: Power sparkline charts
The TUI SHALL display power metrics as sparkline history charts showing current, average, and trend over time. Individual components (CPU, GPU, ANE, DRAM) and totals (package, system) SHALL each have their own sparkline row.

#### Scenario: Power panel rendering
- **WHEN** power metrics are available
- **THEN** the power panel SHALL show sparkline charts for CPU, GPU, ANE, DRAM, package total, and system total, each with current value in Watts

#### Scenario: History accumulation
- **WHEN** the TUI has been running for N sample cycles
- **THEN** sparklines SHALL display up to 128 historical data points, scrolling oldest values off the left edge

### Requirement: GPU gauge display
The TUI SHALL display GPU utilization as a gauge or bar with percentage and frequency in MHz, plus power draw in Watts.

#### Scenario: GPU panel rendering
- **WHEN** GPU metrics are available
- **THEN** the TUI SHALL show GPU utilization %, frequency in MHz, and power in Watts

### Requirement: Temperature display
The TUI SHALL display CPU average and GPU average temperatures in the configured unit (Celsius or Fahrenheit).

#### Scenario: Temperature rendering
- **WHEN** temperature metrics are available
- **THEN** the TUI SHALL show "CPU avg: XX°C  GPU avg: XX°C" (or °F if configured)

### Requirement: Memory bar display
The TUI SHALL display RAM and swap usage as horizontal bars with used/total labels in human-readable units (GB).

#### Scenario: Memory panel rendering
- **WHEN** memory metrics are available
- **THEN** the TUI SHALL show RAM bar (e.g., "RAM [████████░░░] 18.2/24GB") and swap bar if swap is active

### Requirement: Network rate display
The TUI SHALL display current network upload and download rates with appropriate unit scaling.

#### Scenario: Network panel rendering
- **WHEN** network metrics are available
- **THEN** the TUI SHALL show upload rate (↑) and download rate (↓) in auto-scaled units (B/s, KB/s, MB/s, GB/s)

### Requirement: Process list table
The TUI SHALL display a sortable table of running processes showing PID, name, CPU %, memory, and user columns.

#### Scenario: Default process list
- **WHEN** the TUI is displaying
- **THEN** the process list SHALL show processes sorted by CPU % descending, with columns: PID, Name, CPU%, Mem, User

#### Scenario: Sort cycling
- **WHEN** the user presses `s`
- **THEN** the sort column SHALL cycle through: CPU%, Memory, PID, Name

### Requirement: Keyboard controls
The TUI SHALL respond to keyboard input for navigation and control.

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
The TUI SHALL support at least 3 built-in color themes. The active theme SHALL affect all UI elements consistently.

#### Scenario: Theme application
- **WHEN** a theme is active
- **THEN** all borders, labels, bars, graphs, and text SHALL use colors from that theme

### Requirement: Terminal resize handling
The TUI SHALL reflow its layout when the terminal is resized.

#### Scenario: Terminal resize
- **WHEN** the terminal dimensions change
- **THEN** the dashboard SHALL re-render to fit the new size without crashing or leaving artifacts

### Requirement: SoC info header
The TUI SHALL display a header line showing the chip model name, core configuration, and total memory.

#### Scenario: Header rendering
- **WHEN** the TUI starts
- **THEN** the header SHALL show e.g., "mtop — Apple M4 Pro — 10C (4E+6P) / 16GPU — 24GB"

