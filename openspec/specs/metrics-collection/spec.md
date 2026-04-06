# metrics-collection Specification

## Purpose
Define requirements for collecting macOS system metrics (CPU, GPU, memory, disk, network, power, temperature, process) via native APIs without requiring elevated privileges.
## Requirements
### Requirement: CPU metrics collection
The system SHALL collect CPU metrics from macOS system APIs without requiring sudo privileges. Metrics SHALL include per-core utilization percentage, per-cluster (efficiency/performance) aggregate utilization and frequency in MHz, combined weighted CPU utilization, and CPU power draw in Watts.

#### Scenario: CPU utilization sampling
- **WHEN** the metrics collector runs a sample cycle
- **THEN** it SHALL produce per-core utilization values (0.0–1.0) for all cores, cluster-level frequency (MHz) and usage ratio for both E and P clusters, and a combined CPU usage ratio weighted by core count

#### Scenario: CPU power measurement
- **WHEN** the metrics collector reads power data
- **THEN** it SHALL report CPU power consumption in Watts from the IOReport Energy Model

### Requirement: GPU metrics collection
The system SHALL collect GPU metrics including utilization ratio (0.0–1.0), frequency in MHz, and power draw in Watts.

#### Scenario: GPU metrics sampling
- **WHEN** the metrics collector runs a sample cycle on an Apple Silicon Mac
- **THEN** it SHALL produce GPU utilization ratio, GPU frequency in MHz, and GPU power in Watts

### Requirement: Power metrics collection
The system SHALL collect power breakdown metrics including CPU, GPU, ANE (Neural Engine), DRAM, package total, and system total power in Watts.

#### Scenario: Power breakdown
- **WHEN** the metrics collector reads power data
- **THEN** it SHALL report individual power values for CPU, GPU, ANE, DRAM, package (sum of SoC components), and system total, all in Watts

### Requirement: Temperature metrics collection
The system SHALL collect temperature readings for CPU (average across sensors) and GPU in degrees Celsius.

#### Scenario: Temperature with SMC available
- **WHEN** SMC temperature sensors are accessible
- **THEN** it SHALL report CPU average temperature and GPU temperature in Celsius

#### Scenario: Temperature with SMC unavailable
- **WHEN** SMC temperature sensors are not accessible
- **THEN** it SHALL attempt HID sensor fallback, and if that also fails, report the metric as unavailable rather than crashing

### Requirement: Memory metrics collection
The system SHALL collect RAM total, RAM used, swap total, and swap used in bytes.

#### Scenario: Memory sampling
- **WHEN** the metrics collector runs a sample cycle
- **THEN** it SHALL report RAM total and used (active + inactive + wired + compressor pages) and swap total and used, all in bytes

### Requirement: Network metrics collection
The system SHALL collect per-network-interface byte counters and compute upload/download rates in bytes per second.

#### Scenario: Network rate computation
- **WHEN** two consecutive samples are collected at interval T
- **THEN** it SHALL compute per-interface rx_bytes_per_sec and tx_bytes_per_sec as (current - previous) / T

### Requirement: Disk I/O metrics collection
The system SHALL collect disk read and write rates in bytes per second.

#### Scenario: Disk I/O sampling
- **WHEN** the metrics collector runs a sample cycle
- **THEN** it SHALL report disk read_bytes_per_sec and write_bytes_per_sec computed from counter deltas

### Requirement: Process list collection
The system SHALL collect a list of running processes with PID, name, CPU usage percentage, memory usage in bytes, and username.

#### Scenario: Process list sampling
- **WHEN** the metrics collector runs a sample cycle
- **THEN** it SHALL produce a list of processes sorted by CPU usage (descending) with PID, name, CPU %, memory bytes, and username

### Requirement: SoC identification
The system SHALL detect and report the Apple Silicon chip model, including core counts (efficiency, performance, GPU).

#### Scenario: Chip detection
- **WHEN** the application starts
- **THEN** it SHALL identify the chip name (e.g., "Apple M4 Pro"), efficiency core count, performance core count, GPU core count, and total memory

### Requirement: Configurable sampling interval
The system SHALL support a configurable metrics sampling interval with a default of 1000ms and a minimum of 100ms.

#### Scenario: Custom interval
- **WHEN** the user specifies --interval 500
- **THEN** the collector SHALL sample metrics every 500ms

#### Scenario: Minimum interval enforcement
- **WHEN** the user specifies an interval below 100ms
- **THEN** the system SHALL clamp the interval to 100ms

### Requirement: Graceful sensor degradation
The system SHALL continue operating when individual sensors or data sources are unavailable, reporting those metrics as absent rather than crashing.

#### Scenario: Unavailable sensor
- **WHEN** a temperature sensor fails to read
- **THEN** the system SHALL skip that sensor and continue collecting all other metrics without error

### Requirement: No sudo requirement
The system SHALL operate entirely with user-level privileges. No metric collection SHALL require root or sudo.

#### Scenario: Normal user execution
- **WHEN** a non-root user runs mtop
- **THEN** all metrics SHALL be collected successfully (assuming Apple Silicon hardware)

