# api-server Specification

## Purpose
Define requirements for the HTTP API server exposing system metrics in JSON and Prometheus formats.
## Requirements
### Requirement: JSON metrics endpoint
The HTTP server SHALL expose a `GET /json` endpoint that returns the current metrics snapshot as a JSON object with timestamp, soc info, and all metric categories. [T1-static]

#### Scenario: JSON endpoint response
- **WHEN** a client sends `GET /json` to the server
- **THEN** the server SHALL respond with HTTP 200 and a JSON body containing: timestamp (ISO 8601), soc (chip info), cpu, gpu, power, temperature, memory, network, and disk metric groups

#### Scenario: No metrics available yet
- **WHEN** a client requests `/json` before the first sample completes
- **THEN** the server SHALL respond with HTTP 503 and a JSON error message

### Requirement: Prometheus metrics endpoint
The HTTP server SHALL expose a `GET /metrics` endpoint that returns metrics in Prometheus text exposition format with `mtop_` prefix. [T1-static]

#### Scenario: Prometheus endpoint response
- **WHEN** a client sends `GET /metrics`
- **THEN** the server SHALL respond with Content-Type `text/plain` and Prometheus-formatted gauges including: mtop_cpu_usage_ratio, mtop_cpu_freq_mhz, mtop_gpu_usage_ratio, mtop_power_watts, mtop_temperature_celsius, mtop_memory_bytes, mtop_network_bytes_per_second

#### Scenario: Prometheus labels
- **WHEN** a metric has multiple instances (e.g., power components, network interfaces)
- **THEN** each instance SHALL have distinguishing labels (e.g., `component="cpu"`, `interface="en0"`, `direction="rx"`)

### Requirement: Server port configuration
The HTTP server SHALL listen on a configurable port, defaulting to 9090, bound to 127.0.0.1 by default. [T1-static]

#### Scenario: Default port
- **WHEN** the user runs `mtop serve` with no port option
- **THEN** the server SHALL listen on 127.0.0.1:9090

#### Scenario: Custom port
- **WHEN** the user runs `mtop serve --port 8080`
- **THEN** the server SHALL listen on 127.0.0.1:8080

### Requirement: NDJSON pipe mode
The pipe subcommand SHALL output one JSON object per line to stdout at the configured sampling interval. [T1-static]

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
The HTTP server SHALL return HTTP 404 for any path other than `/json` and `/metrics`. [T1-static]

#### Scenario: Unknown path
- **WHEN** a client sends `GET /foo`
- **THEN** the server SHALL respond with HTTP 404

### Requirement: HTTP connection limit [H5]
The HTTP server SHALL limit concurrent connections to a maximum of 64. Connections beyond the limit SHALL be rejected or queued, not spawn unbounded threads. [T1-static]

#### Scenario: Connection limit enforcement
- **WHEN** 64 clients are connected simultaneously and a 65th client attempts to connect
- **THEN** the server SHALL either reject the connection with an appropriate error or queue it until a slot is available, not spawn a 65th handler thread

#### Scenario: Normal operation under limit
- **WHEN** fewer than 64 clients are connected
- **THEN** each connection SHALL be handled normally with no artificial delay

### Requirement: Prometheus label value escaping [M6]
Prometheus label values SHALL escape backslash, double-quote, and newline characters per the Prometheus exposition format specification. [T1-static]

> Reference: tech-spec/prometheus.md — "Label Rules" section; escape_label_value: replace \ with \\, " with \", newline with \n

#### Scenario: Label value with special characters
- **WHEN** a label value contains a backslash, double-quote, or newline
- **THEN** the output SHALL contain the escaped form (\\, \", \n) in the label value string

#### Scenario: Normal label values
- **WHEN** label values contain only alphanumeric characters and common symbols
- **THEN** the values SHALL be output unchanged (escaping has no effect)

### Requirement: Pipe mode sample counter overflow safety
The pipe mode sample counter SHALL use u64 to prevent overflow. At 1 sample/second, u64 provides over 584 billion years of operation. [T1-static]

#### Scenario: Long-running pipe session
- **WHEN** the pipe mode has emitted more than 2^32 samples
- **THEN** the sample counter SHALL continue incrementing correctly without overflow or wrap-around

### Requirement: Read timeout [I3-S1]
The HTTP server SHALL use a read timeout of no more than 2 seconds per connection. [T1-static]

> Reference: challenge-iteration3.md — Challenge 3; reduces Slowloris attack window from 5s to 2s per connection slot

#### Scenario: Slow client timeout
- **WHEN** a connected client does not send a complete HTTP request within 2 seconds
- **THEN** the server SHALL close the connection

#### Scenario: Normal client within timeout
- **WHEN** a client sends a complete HTTP request within 2 seconds
- **THEN** the server SHALL process and respond normally

### Test Scenarios
- Unit test: verify `set_read_timeout` is called with `Duration::from_secs(2)` or less on accepted connections
- Integration test: open a TCP connection, send 1 byte, wait 3 seconds, verify the server has closed the connection
- Integration test: open a TCP connection, send a valid `GET /json` immediately, verify a successful response

### Requirement: Per-IP connection limit [I3-S2]
The HTTP server SHALL limit concurrent connections per IP address. Connections from a single IP exceeding the limit SHALL be rejected with an appropriate HTTP error (e.g., 429 Too Many Requests). [T1-static]

> Reference: challenge-iteration3.md — Challenge 3; per-IP limit prevents a single attacker from exhausting all 64 connection slots

#### Scenario: Per-IP limit enforcement
- **WHEN** a single IP address has reached the per-IP connection limit and attempts another connection
- **THEN** the server SHALL reject the excess connection with an HTTP error response

#### Scenario: Multiple IPs within limits
- **WHEN** multiple different IP addresses each have connections below the per-IP limit
- **THEN** all connections SHALL be handled normally

#### Scenario: Connection slot release
- **WHEN** a client from a rate-limited IP disconnects
- **THEN** the per-IP counter SHALL decrement, allowing a new connection from that IP

### Test Scenarios
- Unit test: verify a per-IP connection tracking data structure exists (e.g., `HashMap<IpAddr, usize>`) and is updated on accept/disconnect
- Integration test: open N+1 connections from the same IP (where N is the limit), verify the (N+1)th is rejected
- Integration test: verify connections from different IPs are accepted independently

### Requirement: No sensitive information exposure [I3-S3]
The HTTP server SHALL NOT expose sensitive system information beyond metrics data. Response headers SHALL NOT include server software identification. Error messages SHALL NOT include internal paths, stack traces, or implementation details. [T1-static]

> Reference: challenge-iteration3.md — Challenge 1; information disclosure risks when bound to non-loopback

#### Scenario: Error response content
- **WHEN** the server encounters an internal error
- **THEN** the error response SHALL contain a generic error message without internal paths or stack traces

#### Scenario: Response headers
- **WHEN** the server sends any response
- **THEN** no `Server` header or equivalent software identification header SHALL be present

### Test Scenarios
- Code inspection: verify no `Server:` header is set in response writing code
- Unit test: trigger error conditions (invalid request, 503 no data) and verify response bodies contain no file paths, line numbers, or Rust panic messages

## Non-Functional Requirements

### Resource Safety
The HTTP server SHALL release all per-connection resources (TCP stream, thread/task, per-IP counter slot) when a connection closes, whether by client disconnect, timeout, or error. No resource SHALL accumulate across connection lifecycles. [T2-soak]

### Performance
The server SHALL respond to a valid `GET /json` or `GET /metrics` request within 50ms under normal load (fewer than 10 concurrent connections). [T2-soak]

### Correctness
The JSON response body SHALL be valid JSON parseable by any RFC 8259-compliant parser. The Prometheus response body SHALL conform to the Prometheus text exposition format specification. [T1-static]

### Resilience
The server SHALL continue serving requests after any single connection error (malformed request, client disconnect, timeout). A failing connection SHALL NOT affect other connections or the main accept loop. [T1-static]

### Security
The server SHALL bind to `127.0.0.1` by default. When bound to a non-loopback address, the server SHALL still enforce connection limits and timeouts. The server SHALL NOT require or accept authentication credentials (no auth surface to attack). [T1-static]

A write timeout of no more than 2 seconds SHALL be set on each connection to prevent response-phase Slowloris (client not reading from socket, causing write buffer to fill and `write_all` to block). [T1-static]

### Longevity
The server endpoint paths (`/json`, `/metrics`) and response schemas SHALL remain stable across minor versions. New metric fields MAY be added to JSON responses but existing fields SHALL NOT be removed or renamed without a major version bump. [T1-static]
