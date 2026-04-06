# metrics-collection Specification

## Purpose
Define requirements for collecting macOS system metrics (CPU, GPU, memory, disk, network, power, temperature, process) via native APIs without requiring elevated privileges.
## Requirements
### Requirement: CPU metrics collection
The system SHALL collect CPU metrics from macOS system APIs without requiring sudo privileges. Metrics SHALL include per-core utilization percentage, per-cluster (efficiency/performance) aggregate utilization and frequency in MHz, combined weighted CPU utilization, and CPU power draw in Watts.

> Reference: tech-spec/mach-host.md — host_processor_info format, E-core first ordering in per-CPU array

#### Scenario: CPU utilization sampling
- **WHEN** the metrics collector runs a sample cycle
- **THEN** it SHALL produce per-core utilization values (0.0–1.0) for all cores, cluster-level frequency (MHz) and usage ratio for both E and P clusters, and a combined CPU usage ratio weighted by core count

#### Scenario: CPU power measurement
- **WHEN** the metrics collector reads power data
- **THEN** it SHALL report CPU power consumption in Watts from the IOReport Energy Model

### Requirement: GPU metrics collection
The system SHALL collect GPU metrics including utilization ratio (0.0–1.0), frequency in MHz, and power draw in Watts.

> Reference: tech-spec/ioreport.md — IOReport GPU Performance States channel group, IOReportStateGetNameForIndex for frequency

#### Scenario: GPU metrics sampling
- **WHEN** the metrics collector runs a sample cycle on an Apple Silicon Mac
- **THEN** it SHALL produce GPU utilization ratio, GPU frequency in MHz, and GPU power in Watts

### Requirement: Power metrics collection
The system SHALL collect power breakdown metrics including CPU, GPU, ANE (Neural Engine), DRAM, package total, and system total power in Watts.

> Reference: tech-spec/ioreport.md — Energy Model channels, nanojoule units, conversion formula power_W = nanojoules / (interval_ms * 1_000_000.0)

#### Scenario: Power breakdown
- **WHEN** the metrics collector reads power data
- **THEN** it SHALL report individual power values for CPU, GPU, ANE, DRAM, package (sum of SoC components), and system total, all in Watts

### Requirement: Temperature metrics collection
The system SHALL collect temperature readings for CPU (average across sensors) and GPU in degrees Celsius.

> Reference: tech-spec/smc.md — key naming (TC0P/TC0p for CPU die, TG0P/Tg0P for GPU), SMC read protocol

#### Scenario: Temperature with SMC available
- **WHEN** SMC temperature sensors are accessible
- **THEN** it SHALL report CPU average temperature and GPU temperature in Celsius

#### Scenario: Temperature with SMC unavailable
- **WHEN** SMC temperature sensors are not accessible
- **THEN** it SHALL attempt HID sensor fallback, and if that also fails, report the metric as unavailable rather than crashing

### Requirement: Memory metrics collection
The system SHALL collect RAM total, RAM used, swap total, and swap used in bytes.

> Reference: tech-spec/mach-host.md — vm_statistics64 struct (160 bytes with swapped_count), page size is 16KB on arm64

#### Scenario: Memory sampling
- **WHEN** the metrics collector runs a sample cycle
- **THEN** it SHALL report RAM total and used (active + inactive + wired + compressor pages) and swap total and used, all in bytes

### Requirement: Network metrics collection
The system SHALL collect per-network-interface byte counters and compute upload/download rates in bytes per second.

> Reference: tech-spec/network.md — if_data64 struct with u64 counters, getifaddrs AF_LINK entries

#### Scenario: Network rate computation
- **WHEN** two consecutive samples are collected at interval T
- **THEN** it SHALL compute per-interface rx_bytes_per_sec and tx_bytes_per_sec as (current - previous) / T

### Requirement: Disk I/O metrics collection
The system SHALL collect disk read and write rates in bytes per second.

> Reference: tech-spec/disk-iokit.md — IOBlockStorageDriver Statistics property for read/write byte counters

#### Scenario: Disk I/O sampling
- **WHEN** the metrics collector runs a sample cycle
- **THEN** it SHALL report disk read_bytes_per_sec and write_bytes_per_sec computed from counter deltas

### Requirement: Process list collection
The system SHALL collect a list of running processes with PID, name, CPU usage percentage, memory usage in bytes, and username.

> Reference: tech-spec/proc-taskinfo.md — proc_pidinfo for per-process task info, CPU% requires delta calculation

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

### Requirement: Process CPU% delta-based calculation [C1]
Process CPU usage percentage SHALL be computed using a time-delta calculation: delta of (pti_total_user + pti_total_system) over wall-clock delta, converted via mach_timebase_info. The system SHALL NOT use pti_numrunning as a CPU utilization metric.

> Reference: tech-spec/proc-taskinfo.md — "Computing CPU% Correctly" section; pti_numrunning is a thread-count snapshot, not utilization

#### Scenario: Steady-state process CPU%
- **WHEN** a process consumes 50% of one core steadily across two sample intervals
- **THEN** the reported CPU% SHALL be approximately 50.0 (within 5% tolerance), not a thread-count-based snapshot

#### Scenario: First sample for a new PID
- **WHEN** a process appears for the first time in the process list
- **THEN** the system SHALL report 0% CPU usage for that PID (no previous sample to delta against)

#### Scenario: Stale PID cleanup
- **WHEN** a tracked PID no longer appears in the current process list
- **THEN** the system SHALL remove its tracking state to prevent unbounded memory growth

### Requirement: No probe writes for disk I/O [C2]
Disk I/O collection SHALL NOT perform any write operations (probe writes, temp file writes, sync calls) to the filesystem. Counter deltas from IOBlockStorageDriver SHALL be used directly.

> Reference: tech-spec/disk-iokit.md — "Why Probe Writes Are Unnecessary" section; counters advance naturally from system I/O

#### Scenario: Disk I/O collection is read-only
- **WHEN** the disk I/O collector runs a sample cycle
- **THEN** it SHALL NOT create, write, or sync any files as part of measurement

### Requirement: Mach host port caching [H1]
The system SHALL call mach_host_self() once at startup and cache the returned port. All subsequent Mach host API calls (host_processor_info, host_statistics64) SHALL reuse the cached port.

> Reference: tech-spec/mach-host.md — "mach_host_self() Port Lifecycle" section; each call leaks one send right

#### Scenario: Port reuse across samples
- **WHEN** the collector completes 1000 sample cycles
- **THEN** mach_host_self() SHALL have been called exactly once, and the Mach port count SHALL not have grown

### Requirement: GPU frequency from IOReport state names [H2]
GPU frequency in MHz SHALL be derived from IOReport state names via IOReportStateGetNameForIndex, parsing the last 4 characters of the state name string (format "GPUPH_XXXX_YYYY" where YYYY is MHz). The system SHALL NOT use a hardcoded linear frequency model.

> Reference: tech-spec/ioreport.md — IOReportStateGetNameForIndex returns borrowed CFStringRef; state name format "GPUPH_XXXX_YYYY"

#### Scenario: Correct GPU frequency reporting
- **WHEN** the GPU is active in a P-state whose IOReport state name ends in "1398"
- **THEN** the reported GPU frequency SHALL be 1398 MHz, not a linear estimate

#### Scenario: Unparseable state name fallback
- **WHEN** a state name does not match the expected format
- **THEN** the system SHALL fall back to a linear estimate rather than crashing

### Requirement: Measured elapsed time for power calculation [H3]
Power duration for nanojoules-to-watts conversion SHALL use actual measured elapsed time (via Instant::now() or equivalent monotonic clock), not a hardcoded sleep duration value.

> Reference: tech-spec/ioreport.md — "Gotchas" item 1; OS scheduling jitter can cause actual sleep to be 110-150ms instead of 100ms

#### Scenario: Accurate power under scheduling jitter
- **WHEN** a requested 100ms sleep actually takes 130ms due to system load
- **THEN** the watts calculation SHALL use 130ms as the divisor, not 100ms

### Requirement: IOReport subscription reuse [H4]
IOReport subscriptions SHALL be created once and reused across sample cycles. The system SHALL NOT recreate subscriptions on every sample. Previous IOReport samples SHALL be stored in collector state for delta computation.

> Reference: tech-spec/ioreport.md — "Gotchas" item 2 (subscription reuse) and item 3 (thread safety: all calls on single thread or behind mutex)

#### Scenario: Subscription persistence across samples
- **WHEN** the collector completes 100 sample cycles
- **THEN** IOReportCreateSubscription SHALL have been called once (not 100 times)

#### Scenario: Delta from stored previous sample
- **WHEN** a new IOReport sample is taken
- **THEN** the delta SHALL be computed against the stored previous sample, and the new sample SHALL replace it in state

### Requirement: VmStatistics64 complete struct [M2]
The VmStatistics64 struct definition SHALL include the swapped_count field at offset 152, making the total struct size 160 bytes and HOST_VM_INFO64_COUNT = 40.

> Reference: tech-spec/mach-host.md — full vm_statistics64 struct layout, 160 bytes total

#### Scenario: Correct struct size
- **WHEN** the VmStatistics64 struct is defined
- **THEN** size_of::<VmStatistics64>() SHALL equal 160 bytes

### Requirement: Network interface type accuracy [M4]
Network interface type classification SHALL NOT label all en* interfaces as "wifi". The system SHALL use ifi_type or a more accurate heuristic to distinguish interface types.

> Reference: tech-spec/network.md — "Detecting WiFi vs Ethernet" section; en0 is wired Ethernet on Mac Mini/Studio/Pro

#### Scenario: Wired Ethernet on desktop Mac
- **WHEN** the system has en0 as a wired Ethernet adapter
- **THEN** the interface type label SHALL NOT be "wifi"

### Requirement: SMC connection caching [M5]
The SMC connection (io_connect_t) SHALL be opened once and reused across temperature samples. The connection SHALL be closed on shutdown (via Drop or equivalent cleanup).

> Reference: tech-spec/smc.md — "Gotchas" item 4; opening an SMC connection is expensive

#### Scenario: Connection reuse across samples
- **WHEN** the temperature collector completes 100 sample cycles
- **THEN** IOServiceOpen for AppleSMC SHALL have been called once (not 100 times)

### Requirement: Shared IOReport loading [M7]
IOReport framework loading (dlopen/dlsym) and helper functions (cfstring, cfstring_to_string) SHALL be shared between GPU and power collection modules. The system SHALL NOT duplicate the loading code.

> Reference: tech-spec/ioreport.md — single OnceLock pattern for shared function pointer struct

#### Scenario: Single dlopen call
- **WHEN** both GPU and power collectors initialize
- **THEN** dlopen for IOReport.framework SHALL be called once total, not once per module

### Requirement: GPU power wired from power collector [M8]
GPU power_w in GpuMetrics SHALL be populated from the power collector's gpu_w value. The system SHALL NOT hardcode GPU power as 0.0.

#### Scenario: GPU power display
- **WHEN** the power collector reports GPU power of 3.5W
- **THEN** GpuMetrics.power_w SHALL be 3.5, not 0.0

### Requirement: Memory page size validation [sysconf]
Memory collection SHALL validate the return value of sysconf(_SC_PAGESIZE) before using it in calculations. A return value of -1 SHALL be handled by falling back to vm_page_size or a safe default, not by casting -1 to u64.

> Reference: tech-spec/mach-host.md — sysconf returns -1 on error; casting to u64 produces 0xFFFFFFFFFFFFFFFF

#### Scenario: sysconf failure handling
- **WHEN** sysconf(_SC_PAGESIZE) returns -1
- **THEN** the system SHALL use vm_page_size or 16384 as a fallback, not produce corrupted memory values

## Breakdown: IOReport Collection Chain

### Subscription lifecycle
The IOReport subscription SHALL be created once during collector initialization and reused for all subsequent samples. The subscription SHALL be released on collector shutdown (Drop implementation). The system SHALL NOT recreate the subscription per sample cycle.

### Channel groups
- "Energy Model" channel group SHALL be used for power metrics (CPU, GPU, ANE, DRAM energy in nanojoules)
- "GPU Performance States" channel group SHALL be used for GPU utilization and frequency metrics
- Both channel groups MAY share a single dlopen handle and function pointer struct

### Delta computation
Two IOReport samples SHALL be taken with a measured time interval between them. Energy delta (nanojoules) SHALL be converted to power (watts) using the formula: `power_W = delta_nanojoules / (measured_interval_ms * 1_000_000.0)`. The measured interval SHALL use a monotonic clock (Instant::now()), not the requested sleep duration.

### Thread safety
All IOReport function calls SHALL execute on a single thread or be guarded by a mutex. IOReport functions are NOT thread-safe per framework behavior. If GPU and power collectors share a subscription, access SHALL be serialized.

### State names for GPU frequency
IOReportStateGetNameForIndex SHALL be called for each GPU P-state index to retrieve the state name. The last 4 characters of the name (format "GPUPH_XXXX_YYYY") SHALL be parsed as decimal MHz. Index 0 is always the idle state (0 MHz). The returned CFStringRef is borrowed (Get rule) and SHALL NOT be released by the caller.

> Reference: tech-spec/ioreport.md — full IOReport API lifecycle; tech-spec/corefoundation.md — Get rule for borrowed references

## Breakdown: SMC Sensor Reading

### Connection lifecycle
The SMC connection SHALL be opened once via IOServiceOpen("AppleSMC") during collector initialization. The io_connect_t handle SHALL be stored in collector state and reused for all temperature reads. IOServiceClose SHALL be called on shutdown (Drop implementation).

### Key encoding
SMC 4-character key names SHALL be encoded as big-endian u32 values. Example: "TC0P" encodes to 0x54_43_30_50 via u32::from_be_bytes().

### Read protocol
Each SMC key read follows a 3-step protocol:
1. SEARCH_KEY (selector 2, input8) — locate the key by encoded name
2. READ_KEYINFO (selector 9, input8) — retrieve type tag and data size
3. READ_KEY_STATUS (selector 5, input8) — read the actual value bytes

### Temperature keys
Primary temperature keys with fallback chain:
- CPU die: TC0P, TC0p, Tp01 (try in order, use first successful)
- GPU die: TG0P, Tg0P, Tg05 (try in order, use first successful)
- If all keys fail for a sensor, report the metric as unavailable (not 0.0)

### SmcKeyData struct
The SmcKeyData struct is 80 bytes total with a 32-byte result field. The struct layout SHALL match Apple's SMCParamStruct definition exactly for correct IOConnectCallStructMethod data exchange.

> Reference: tech-spec/smc.md — full SMC protocol, key encoding, struct layout

## Breakdown: Process CPU Delta Tracking

### State storage
The Sampler SHALL maintain a HashMap<pid_t, (u64, Instant)> mapping each tracked PID to its previous cumulative Mach time and wall-clock timestamp. This state SHALL persist across sample cycles.

### Mach timebase conversion
mach_timebase_info SHALL be called once at Sampler initialization to obtain the numer/denom ratio for converting Mach absolute time ticks to nanoseconds. On Apple Silicon, numer/denom = 1/1; on Intel, it varies. The formula is: `nanoseconds = mach_ticks * numer / denom`.

### CPU% formula
For each process with a previous sample:
```
delta_task_ticks = (current_total_user + current_total_system) - prev_total_mach_time
delta_task_ns = delta_task_ticks * timebase_numer / timebase_denom
delta_wall_ns = current_wall_instant.duration_since(prev_wall_instant).as_nanos()
cpu_percent = (delta_task_ns as f64 / delta_wall_ns as f64) * 100.0
```

### First sample behavior
When a PID appears for the first time (no entry in HashMap), the system SHALL store its current state and report 0% CPU usage. This is correct behavior — there is no delta to compute.

### Stale PID cleanup
After processing all current PIDs, the system SHALL remove HashMap entries for PIDs that are no longer in the current process list. This prevents unbounded memory growth from short-lived processes.

> Reference: tech-spec/proc-taskinfo.md — pti_total_user/pti_total_system fields, mach_timebase_info conversion
