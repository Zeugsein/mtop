# cli-interface Specification

## Purpose
Define requirements for command-line argument parsing, output modes (TUI, pipe, serve, debug), and configuration options.
## Requirements
### Requirement: Default TUI mode
When invoked with no subcommand, mtop SHALL launch in TUI dashboard mode. [T1-static]

#### Scenario: No arguments
- **WHEN** the user runs `mtop`
- **THEN** the TUI dashboard SHALL start with default settings (1000ms interval, theme sentinel "default" which resolves to the first theme — horizon)

### Requirement: Pipe subcommand
mtop SHALL support a `pipe` subcommand for NDJSON output to stdout. [T1-static]

#### Scenario: Pipe invocation
- **WHEN** the user runs `mtop pipe`
- **THEN** NDJSON metrics SHALL be output to stdout at the configured interval

#### Scenario: Pipe with samples
- **WHEN** the user runs `mtop pipe --samples 10`
- **THEN** exactly 10 JSON objects SHALL be output, then the process exits

### Requirement: Serve subcommand
mtop SHALL support a `serve` subcommand to start the HTTP API server. [T1-static]

#### Scenario: Serve invocation
- **WHEN** the user runs `mtop serve`
- **THEN** an HTTP server SHALL start on the default port (9090) and begin serving metrics

#### Scenario: Serve with custom port
- **WHEN** the user runs `mtop serve --port 8080`
- **THEN** the HTTP server SHALL listen on port 8080

### Requirement: Global interval option
mtop SHALL accept a `--interval` or `-i` global option specifying the metrics sampling interval in milliseconds. [T1-static]

#### Scenario: Custom interval
- **WHEN** the user runs `mtop --interval 500`
- **THEN** metrics SHALL be sampled every 500ms

#### Scenario: Default interval
- **WHEN** no interval is specified
- **THEN** the default interval SHALL be 1000ms

### Requirement: Color option
mtop SHALL accept a `--color` option to set the TUI color theme by name. [T1-static]

#### Scenario: Color selection
- **WHEN** the user runs `mtop --color blue`
- **THEN** the TUI SHALL use the blue color theme

### Requirement: Temperature unit option
mtop SHALL accept a `--temp-unit` option to set the temperature display unit. [T1-static]

#### Scenario: Fahrenheit
- **WHEN** the user runs `mtop --temp-unit fahrenheit`
- **THEN** all temperature values SHALL be displayed in Fahrenheit

#### Scenario: Default Celsius
- **WHEN** no temp-unit is specified
- **THEN** temperatures SHALL be displayed in Celsius

### Requirement: Version flag
mtop SHALL display its version when invoked with `--version` or `-V`. [T1-static]

#### Scenario: Version display
- **WHEN** the user runs `mtop --version`
- **THEN** stdout SHALL show the version string (e.g., "mtop 0.1.0") and exit

### Requirement: Help flag
mtop SHALL display usage information when invoked with `--help` or `-h`. [T1-static]

#### Scenario: Help display
- **WHEN** the user runs `mtop --help`
- **THEN** stdout SHALL show usage text including all subcommands and global options, then exit

### Requirement: Debug subcommand
mtop SHALL support a `debug` subcommand that prints raw diagnostic information about detected hardware and available sensors. [T1-static]

#### Scenario: Debug output
- **WHEN** the user runs `mtop debug`
- **THEN** stdout SHALL print chip detection info, available SMC keys, IOReport channel names, and sensor availability

### Requirement: Debug subcommand sensor enumeration
The debug subcommand SHALL enumerate available SMC keys and IOReport channel names to aid in diagnosing sensor availability and naming on different hardware. [T1-static]

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

## Non-Functional Requirements

### Resource Safety
The CLI argument parser SHALL NOT allocate unbounded memory based on user input. Argument string lengths SHALL be bounded by the OS argument limit. [T1-static]

### Performance
CLI argument parsing and subcommand dispatch SHALL complete in under 10ms. The user SHALL NOT experience visible delay between invocation and the first output (TUI frame, pipe line, or server bind message). [T2-soak]

### Correctness
All CLI flags and options documented in `--help` output SHALL be functional. No flag SHALL be documented but unimplemented, and no undocumented flag SHALL exist. [T1-static]

### Resilience
Invalid arguments SHALL produce a clear error message and a non-zero exit code. The CLI SHALL NOT panic on invalid input. [T1-static]

### Security
The CLI SHALL NOT accept or process arguments that could lead to arbitrary file writes, code execution, or privilege escalation. The `--bind` option for serve mode SHALL default to `127.0.0.1`. [T1-static]

### Longevity
New subcommands and global options SHALL be addable without breaking existing CLI invocations. Existing flag names and short aliases SHALL remain stable across minor versions. [T1-static]
