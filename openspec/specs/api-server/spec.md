# api-server Specification

## Purpose
Define requirements for the HTTP API server exposing system metrics in JSON and Prometheus formats.
## Requirements
### Requirement: JSON metrics endpoint
The HTTP server SHALL expose a `GET /json` endpoint that returns the current metrics snapshot as a JSON object with timestamp, soc info, and all metric categories.

#### Scenario: JSON endpoint response
- **WHEN** a client sends `GET /json` to the server
- **THEN** the server SHALL respond with HTTP 200 and a JSON body containing: timestamp (ISO 8601), soc (chip info), cpu, gpu, power, temperature, memory, network, and disk metric groups

#### Scenario: No metrics available yet
- **WHEN** a client requests `/json` before the first sample completes
- **THEN** the server SHALL respond with HTTP 503 and a JSON error message

### Requirement: Prometheus metrics endpoint
The HTTP server SHALL expose a `GET /metrics` endpoint that returns metrics in Prometheus text exposition format with `mtop_` prefix.

#### Scenario: Prometheus endpoint response
- **WHEN** a client sends `GET /metrics`
- **THEN** the server SHALL respond with Content-Type `text/plain` and Prometheus-formatted gauges including: mtop_cpu_usage_ratio, mtop_cpu_freq_mhz, mtop_gpu_usage_ratio, mtop_power_watts, mtop_temperature_celsius, mtop_memory_bytes, mtop_network_bytes_per_second

#### Scenario: Prometheus labels
- **WHEN** a metric has multiple instances (e.g., power components, network interfaces)
- **THEN** each instance SHALL have distinguishing labels (e.g., `component="cpu"`, `interface="en0"`, `direction="rx"`)

### Requirement: Server port configuration
The HTTP server SHALL listen on a configurable port, defaulting to 9090, bound to 127.0.0.1 by default.

#### Scenario: Default port
- **WHEN** the user runs `mtop serve` with no port option
- **THEN** the server SHALL listen on 127.0.0.1:9090

#### Scenario: Custom port
- **WHEN** the user runs `mtop serve --port 8080`
- **THEN** the server SHALL listen on 127.0.0.1:8080

### Requirement: NDJSON pipe mode
The pipe subcommand SHALL output one JSON object per line to stdout at the configured sampling interval.

#### Scenario: Pipe output format
- **WHEN** the user runs `mtop pipe`
- **THEN** stdout SHALL receive one complete JSON object per line (newline-delimited), using the same schema as the `/json` endpoint

#### Scenario: Sample count limit
- **WHEN** the user runs `mtop pipe --samples 5`
- **THEN** exactly 5 JSON lines SHALL be output, then the process SHALL exit with code 0

#### Scenario: Infinite pipe
- **WHEN** the user runs `mtop pipe` with no --samples flag (or --samples 0)
- **THEN** output SHALL continue indefinitely until the process is terminated

### Requirement: Unknown route handling
The HTTP server SHALL return HTTP 404 for any path other than `/json` and `/metrics`.

#### Scenario: Unknown path
- **WHEN** a client sends `GET /foo`
- **THEN** the server SHALL respond with HTTP 404

### Requirement: HTTP connection limit [H5]
The HTTP server SHALL limit concurrent connections to a maximum of 64. Connections beyond the limit SHALL be rejected or queued, not spawn unbounded threads.

#### Scenario: Connection limit enforcement
- **WHEN** 64 clients are connected simultaneously and a 65th client attempts to connect
- **THEN** the server SHALL either reject the connection with an appropriate error or queue it until a slot is available, not spawn a 65th handler thread

#### Scenario: Normal operation under limit
- **WHEN** fewer than 64 clients are connected
- **THEN** each connection SHALL be handled normally with no artificial delay

### Requirement: Prometheus label value escaping [M6]
Prometheus label values SHALL escape backslash, double-quote, and newline characters per the Prometheus exposition format specification.

> Reference: tech-spec/prometheus.md — "Label Rules" section; escape_label_value: replace \ with \\, " with \", newline with \n

#### Scenario: Label value with special characters
- **WHEN** a label value contains a backslash, double-quote, or newline
- **THEN** the output SHALL contain the escaped form (\\, \", \n) in the label value string

#### Scenario: Normal label values
- **WHEN** label values contain only alphanumeric characters and common symbols
- **THEN** the values SHALL be output unchanged (escaping has no effect)

### Requirement: Pipe mode sample counter overflow safety
The pipe mode sample counter SHALL use u64 to prevent overflow. At 1 sample/second, u64 provides over 584 billion years of operation.

#### Scenario: Long-running pipe session
- **WHEN** the pipe mode has emitted more than 2^32 samples
- **THEN** the sample counter SHALL continue incrementing correctly without overflow or wrap-around
