# cli-interface Specification

## Purpose
Define requirements for command-line argument parsing, output modes (TUI, pipe, serve, debug), and configuration options.
## Requirements
### Requirement: Default TUI mode
When invoked with no subcommand, mtop SHALL launch in TUI dashboard mode.

#### Scenario: No arguments
- **WHEN** the user runs `mtop`
- **THEN** the TUI dashboard SHALL start with default settings (1000ms interval, default theme)

### Requirement: Pipe subcommand
mtop SHALL support a `pipe` subcommand for NDJSON output to stdout.

#### Scenario: Pipe invocation
- **WHEN** the user runs `mtop pipe`
- **THEN** NDJSON metrics SHALL be output to stdout at the configured interval

#### Scenario: Pipe with samples
- **WHEN** the user runs `mtop pipe --samples 10`
- **THEN** exactly 10 JSON objects SHALL be output, then the process exits

### Requirement: Serve subcommand
mtop SHALL support a `serve` subcommand to start the HTTP API server.

#### Scenario: Serve invocation
- **WHEN** the user runs `mtop serve`
- **THEN** an HTTP server SHALL start on the default port (9090) and begin serving metrics

#### Scenario: Serve with custom port
- **WHEN** the user runs `mtop serve --port 8080`
- **THEN** the HTTP server SHALL listen on port 8080

### Requirement: Global interval option
mtop SHALL accept a `--interval` or `-i` global option specifying the metrics sampling interval in milliseconds.

#### Scenario: Custom interval
- **WHEN** the user runs `mtop --interval 500`
- **THEN** metrics SHALL be sampled every 500ms

#### Scenario: Default interval
- **WHEN** no interval is specified
- **THEN** the default interval SHALL be 1000ms

### Requirement: Color option
mtop SHALL accept a `--color` option to set the TUI color theme by name.

#### Scenario: Color selection
- **WHEN** the user runs `mtop --color blue`
- **THEN** the TUI SHALL use the blue color theme

### Requirement: Temperature unit option
mtop SHALL accept a `--temp-unit` option to set the temperature display unit.

#### Scenario: Fahrenheit
- **WHEN** the user runs `mtop --temp-unit fahrenheit`
- **THEN** all temperature values SHALL be displayed in Fahrenheit

#### Scenario: Default Celsius
- **WHEN** no temp-unit is specified
- **THEN** temperatures SHALL be displayed in Celsius

### Requirement: Version flag
mtop SHALL display its version when invoked with `--version` or `-V`.

#### Scenario: Version display
- **WHEN** the user runs `mtop --version`
- **THEN** stdout SHALL show the version string (e.g., "mtop 0.1.0") and exit

### Requirement: Help flag
mtop SHALL display usage information when invoked with `--help` or `-h`.

#### Scenario: Help display
- **WHEN** the user runs `mtop --help`
- **THEN** stdout SHALL show usage text including all subcommands and global options, then exit

### Requirement: Debug subcommand
mtop SHALL support a `debug` subcommand that prints raw diagnostic information about detected hardware and available sensors.

#### Scenario: Debug output
- **WHEN** the user runs `mtop debug`
- **THEN** stdout SHALL print chip detection info, available SMC keys, IOReport channel names, and sensor availability

### Requirement: Debug subcommand sensor enumeration
The debug subcommand SHALL enumerate available SMC keys and IOReport channel names to aid in diagnosing sensor availability and naming on different hardware.

> Reference: tech-spec/smc.md — SMC key enumeration via SEARCH_KEY; tech-spec/ioreport.md — IOReportCopyChannelsInGroup for channel discovery

#### Scenario: SMC key listing
- **WHEN** the user runs `mtop debug` on a system with SMC access
- **THEN** the output SHALL include a list of discovered SMC temperature keys (e.g., TC0P, TG0P) with their current values

#### Scenario: IOReport channel listing
- **WHEN** the user runs `mtop debug` on an Apple Silicon Mac
- **THEN** the output SHALL include IOReport channel group names and subgroup names for "Energy Model" and "GPU Performance States"

#### Scenario: Debug on unsupported hardware
- **WHEN** the user runs `mtop debug` on a system without SMC or IOReport
- **THEN** the output SHALL indicate which sensors are unavailable rather than crashing
