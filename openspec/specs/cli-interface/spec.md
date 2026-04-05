# cli-interface Specification

## Purpose
TBD - created by archiving change mvp-core. Update Purpose after archive.
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

