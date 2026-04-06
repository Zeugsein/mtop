# metrics-collection Specification

## Purpose
Define requirements for collecting macOS system metrics (CPU, GPU, memory, disk, network, power, temperature, process) via native APIs without requiring elevated privileges.
## Requirements
### Requirement: CPU metrics collection
The system SHALL collect CPU metrics from macOS system APIs without requiring sudo privileges. Metrics SHALL include per-core utilization percentage, per-cluster (efficiency/performance) aggregate utilization and frequency in MHz, combined weighted CPU utilization, and CPU power draw in Watts. [T1-static]

> Reference: tech-spec/mach-host.md — host_processor_info format, E-core first ordering in per-CPU array

#### Scenario: CPU utilization sampling
- **WHEN** the metrics collector runs a sample cycle
- **THEN** it SHALL produce per-core utilization values (0.0–1.0) for all cores, cluster-level frequency (MHz) and usage ratio for both E and P clusters, and a combined CPU usage ratio weighted by core count

#### Scenario: CPU power measurement
- **WHEN** the metrics collector reads power data
- **THEN** it SHALL report CPU power consumption in Watts from the IOReport Energy Model

### Requirement: GPU metrics collection
The system SHALL collect GPU metrics including utilization ratio (0.0–1.0), frequency in MHz, and power draw in Watts. [T1-static]

> Reference: tech-spec/ioreport.md — IOReport GPU Performance States channel group, IOReportStateGetNameForIndex for frequency

#### Scenario: GPU metrics sampling
- **WHEN** the metrics collector runs a sample cycle on an Apple Silicon Mac
- **THEN** it SHALL produce GPU utilization ratio, GPU frequency in MHz, and GPU power in Watts

### Requirement: Power metrics collection
The system SHALL collect power breakdown metrics including CPU, GPU, ANE (Neural Engine), DRAM, package total, and system total power in Watts. [T1-static]

> Reference: tech-spec/ioreport.md — Energy Model channels, nanojoule units, conversion formula power_W = nanojoules / (interval_ms * 1_000_000.0)

#### Scenario: Power breakdown
- **WHEN** the metrics collector reads power data
- **THEN** it SHALL report individual power values for CPU, GPU, ANE, DRAM, package (sum of SoC components), and system total, all in Watts

### Requirement: Temperature metrics collection
The system SHALL collect temperature readings for CPU (average across sensors) and GPU in degrees Celsius. [T1-static]

> Reference: tech-spec/smc.md — key naming (TC0P/TC0p for CPU die, TG0P/Tg0P for GPU), SMC read protocol

#### Scenario: Temperature with SMC available
- **WHEN** SMC temperature sensors are accessible
- **THEN** it SHALL report CPU average temperature and GPU temperature in Celsius

#### Scenario: Temperature with SMC unavailable
- **WHEN** SMC temperature sensors are not accessible
- **THEN** it SHALL attempt HID sensor fallback, and if that also fails, report the metric as unavailable rather than crashing

### Requirement: Memory metrics collection
The system SHALL collect RAM total, RAM used, swap total, and swap used in bytes. [T1-static]

> Reference: tech-spec/mach-host.md — vm_statistics64 struct (160 bytes with swapped_count), page size is 16KB on arm64

#### Scenario: Memory sampling
- **WHEN** the metrics collector runs a sample cycle
- **THEN** it SHALL report RAM total and used (active + inactive + wired + compressor pages) and swap total and used, all in bytes

### Requirement: Network metrics collection
The system SHALL collect per-network-interface byte counters and compute upload/download rates in bytes per second. [T1-static]

> Reference: tech-spec/network.md — if_data64 struct with u64 counters, getifaddrs AF_LINK entries

#### Scenario: Network rate computation
- **WHEN** two consecutive samples are collected at interval T
- **THEN** it SHALL compute per-interface rx_bytes_per_sec and tx_bytes_per_sec as (current - previous) / T

### Requirement: Disk I/O metrics collection
The system SHALL collect disk read and write rates in bytes per second. [T1-static]

> Reference: tech-spec/disk-iokit.md — IOBlockStorageDriver Statistics property for read/write byte counters

#### Scenario: Disk I/O sampling
- **WHEN** the metrics collector runs a sample cycle
- **THEN** it SHALL report disk read_bytes_per_sec and write_bytes_per_sec computed from counter deltas

### Requirement: Process list collection
The system SHALL collect a list of running processes with PID, name, CPU usage percentage, memory usage in bytes, and username. [T1-static]

> Reference: tech-spec/proc-taskinfo.md — proc_pidinfo for per-process task info, CPU% requires delta calculation

#### Scenario: Process list sampling
- **WHEN** the metrics collector runs a sample cycle
- **THEN** it SHALL produce a list of processes sorted by CPU usage (descending) with PID, name, CPU %, memory bytes, and username

### Requirement: SoC identification
The system SHALL detect and report the Apple Silicon chip model, including core counts (efficiency, performance, GPU). [T1-static]

#### Scenario: Chip detection
- **WHEN** the application starts
- **THEN** it SHALL identify the chip name (e.g., "Apple M4 Pro"), efficiency core count, performance core count, GPU core count, and total memory

### Requirement: Configurable sampling interval
The system SHALL support a configurable metrics sampling interval with a default of 1000ms and a minimum of 100ms. [T1-static]

#### Scenario: Custom interval
- **WHEN** the user specifies --interval 500
- **THEN** the collector SHALL sample metrics every 500ms

#### Scenario: Minimum interval enforcement
- **WHEN** the user specifies an interval below 100ms
- **THEN** the system SHALL clamp the interval to 100ms

### Requirement: Graceful sensor degradation
The system SHALL continue operating when individual sensors or data sources are unavailable, reporting those metrics as absent rather than crashing. [T1-static]

#### Scenario: Unavailable sensor
- **WHEN** a temperature sensor fails to read
- **THEN** the system SHALL skip that sensor and continue collecting all other metrics without error

### Requirement: No sudo requirement
The system SHALL operate entirely with user-level privileges. No metric collection SHALL require root or sudo. [T1-static]

#### Scenario: Normal user execution
- **WHEN** a non-root user runs mtop
- **THEN** all metrics SHALL be collected successfully (assuming Apple Silicon hardware)

### Requirement: Process CPU% delta-based calculation [C1]
Process CPU usage percentage SHALL be computed using a time-delta calculation: delta of (pti_total_user + pti_total_system) over wall-clock delta, converted via mach_timebase_info. The system SHALL NOT use pti_numrunning as a CPU utilization metric. [T1-static]

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
Disk I/O collection SHALL NOT perform any write operations (probe writes, temp file writes, sync calls) to the filesystem. Counter deltas from IOBlockStorageDriver SHALL be used directly. [T1-static]

> Reference: tech-spec/disk-iokit.md — "Why Probe Writes Are Unnecessary" section; counters advance naturally from system I/O

#### Scenario: Disk I/O collection is read-only
- **WHEN** the disk I/O collector runs a sample cycle
- **THEN** it SHALL NOT create, write, or sync any files as part of measurement

### Requirement: Mach host port caching [H1]
The system SHALL call mach_host_self() once at startup and cache the returned port. All subsequent Mach host API calls (host_processor_info, host_statistics64) SHALL reuse the cached port. [T1-static]

> Reference: tech-spec/mach-host.md — "mach_host_self() Port Lifecycle" section; each call leaks one send right

#### Scenario: Port reuse across samples
- **WHEN** the collector completes 1000 sample cycles
- **THEN** mach_host_self() SHALL have been called exactly once, and the Mach port count SHALL not have grown

### Requirement: GPU frequency from IOReport state names [H2]
GPU frequency in MHz SHALL be derived from IOReport state names via IOReportStateGetNameForIndex, parsing the last 4 characters of the state name string (format "GPUPH_XXXX_YYYY" where YYYY is MHz). The system SHALL NOT use a hardcoded linear frequency model. [T1-static]

> Reference: tech-spec/ioreport.md — IOReportStateGetNameForIndex returns borrowed CFStringRef; state name format "GPUPH_XXXX_YYYY"

#### Scenario: Correct GPU frequency reporting
- **WHEN** the GPU is active in a P-state whose IOReport state name ends in "1398"
- **THEN** the reported GPU frequency SHALL be 1398 MHz, not a linear estimate

#### Scenario: Unparseable state name fallback
- **WHEN** a state name does not match the expected format
- **THEN** the system SHALL fall back to a linear estimate rather than crashing

### Requirement: Measured elapsed time for power calculation [H3]
Power duration for nanojoules-to-watts conversion SHALL use actual measured elapsed time (via Instant::now() or equivalent monotonic clock), not a hardcoded sleep duration value. [T1-static]

> Reference: tech-spec/ioreport.md — "Gotchas" item 1; OS scheduling jitter can cause actual sleep to be 110-150ms instead of 100ms

#### Scenario: Accurate power under scheduling jitter
- **WHEN** a requested 100ms sleep actually takes 130ms due to system load
- **THEN** the watts calculation SHALL use 130ms as the divisor, not 100ms

### Requirement: IOReport subscription reuse [H4]
IOReport subscriptions SHALL be created once and reused across sample cycles. The system SHALL NOT recreate subscriptions on every sample. Previous IOReport samples SHALL be stored in collector state for delta computation. [T1-static]

> Reference: tech-spec/ioreport.md — "Gotchas" item 2 (subscription reuse) and item 3 (thread safety: all calls on single thread or behind mutex)

#### Scenario: Subscription persistence across samples
- **WHEN** the collector completes 100 sample cycles
- **THEN** IOReportCreateSubscription SHALL have been called once (not 100 times)

#### Scenario: Delta from stored previous sample
- **WHEN** a new IOReport sample is taken
- **THEN** the delta SHALL be computed against the stored previous sample, and the new sample SHALL replace it in state

### Requirement: VmStatistics64 complete struct [M2]
The VmStatistics64 struct definition SHALL include the swapped_count field at offset 152, making the total struct size 160 bytes and HOST_VM_INFO64_COUNT = 40. [T1-static]

> Reference: tech-spec/mach-host.md — full vm_statistics64 struct layout, 160 bytes total

#### Scenario: Correct struct size
- **WHEN** the VmStatistics64 struct is defined
- **THEN** size_of::<VmStatistics64>() SHALL equal 160 bytes

### Requirement: Network interface type accuracy [M4]
Network interface type classification SHALL NOT label all en* interfaces as "wifi". The system SHALL use ifi_type or a more accurate heuristic to distinguish interface types. [T1-static]

> Reference: tech-spec/network.md — "Detecting WiFi vs Ethernet" section; en0 is wired Ethernet on Mac Mini/Studio/Pro

#### Scenario: Wired Ethernet on desktop Mac
- **WHEN** the system has en0 as a wired Ethernet adapter
- **THEN** the interface type label SHALL NOT be "wifi"

### Requirement: SMC connection caching [M5]
The SMC connection (io_connect_t) SHALL be opened once and reused across temperature samples. The connection SHALL be closed on shutdown (via Drop or equivalent cleanup). [T1-static]

> Reference: tech-spec/smc.md — "Gotchas" item 4; opening an SMC connection is expensive

#### Scenario: Connection reuse across samples
- **WHEN** the temperature collector completes 100 sample cycles
- **THEN** IOServiceOpen for AppleSMC SHALL have been called once (not 100 times)

### Requirement: Shared IOReport loading [M7]
IOReport framework loading (dlopen/dlsym) and helper functions (cfstring, cfstring_to_string) SHALL be shared between GPU and power collection modules. The system SHALL NOT duplicate the loading code. [T1-static]

> Reference: tech-spec/ioreport.md — single OnceLock pattern for shared function pointer struct

#### Scenario: Single dlopen call
- **WHEN** both GPU and power collectors initialize
- **THEN** dlopen for IOReport.framework SHALL be called once total, not once per module

### Requirement: GPU power wired from power collector [M8]
GPU power_w in GpuMetrics SHALL be populated from the power collector's gpu_w value. The system SHALL NOT hardcode GPU power as 0.0. [T1-static]

#### Scenario: GPU power display
- **WHEN** the power collector reports GPU power of 3.5W
- **THEN** GpuMetrics.power_w SHALL be 3.5, not 0.0

### Requirement: XswUsage struct field accuracy [M1]
The XswUsage struct field names SHALL match Apple's xsw_usage header definition. The final field SHALL be named xsu_pagesize (not _padding) to accurately reflect its meaning. [T1-static]

#### Scenario: Struct field naming
- **WHEN** the XswUsage struct is defined for swap usage collection
- **THEN** all field names SHALL match the Apple header: xsu_total, xsu_avail, xsu_used, xsu_encrypted, xsu_pagesize

### Requirement: Efficient metrics history buffer [M3]
MetricsHistory SHALL use an O(1) push/pop data structure (VecDeque or ring buffer) for storing historical data points. The system SHALL NOT use Vec::remove(0) which is O(n). [T1-static]

#### Scenario: History buffer performance
- **WHEN** the history buffer is at capacity (128 points) and a new sample arrives
- **THEN** the oldest sample SHALL be removed and the new sample added, both in O(1) time

### Requirement: Memory page size validation [sysconf]
Memory collection SHALL validate the return value of sysconf(_SC_PAGESIZE) before using it in calculations. A return value of -1 SHALL be handled by falling back to vm_page_size or a safe default, not by casting -1 to u64. [T1-static]

> Reference: tech-spec/mach-host.md — sysconf returns -1 on error; casting to u64 produces 0xFFFFFFFFFFFFFFFF

#### Scenario: sysconf failure handling
- **WHEN** sysconf(_SC_PAGESIZE) returns -1
- **THEN** the system SHALL use vm_page_size or 16384 as a fallback, not produce corrupted memory values

### Requirement: Mach port cleanup in Sampler::drop [I3-C1]
The system SHALL call `mach_port_deallocate(mach_task_self(), self.host_port)` on the cached host port in `Sampler::drop()`. [T1-static]

> Reference: challenge-iteration3.md — Challenge 2; mach_port_deallocate is the correct call for send rights from mach_host_self()

#### Scenario: Port deallocation on drop
- **WHEN** a Sampler instance is dropped
- **THEN** `mach_port_deallocate` SHALL be called on the cached host port

### Test Scenarios
- Verify by code inspection that `Sampler` implements `Drop` and the drop body calls `mach_port_deallocate(mach_task_self(), self.host_port)`
- Unit test: create a Sampler, drop it, verify no panic and that the FFI declaration for `mach_port_deallocate` exists

### Requirement: IOReport delta channel iteration [I3-C2]
The system SHALL extract the `IOReportChannels` array from the delta dictionary before calling state APIs (`IOReportStateGetCount`, `IOReportStateGetResidency`, `IOReportStateGetNameForIndex`, `IOReportSimpleGetIntegerValue`). These APIs SHALL be called on individual channel entries from the array, NOT on the top-level delta dictionary. [T1-static]

> Reference: tech-spec/ioreport.md — "Delta Channel Iteration Pattern" section; calling state APIs on top-level dict returns 0 or garbage

#### Scenario: GPU state iteration
- **WHEN** the GPU collector processes an IOReport delta
- **THEN** it SHALL call `CFDictionaryGetValue(delta, "IOReportChannels")` to get the array, iterate entries via `CFArrayGetValueAtIndex`, and call `IOReportStateGetCount` on each entry

#### Scenario: Power channel iteration
- **WHEN** the power collector processes an IOReport delta
- **THEN** it SHALL iterate channel entries from the `IOReportChannels` array and call `IOReportSimpleGetIntegerValue` on each entry

### Test Scenarios
- Code inspection: verify GPU and power parsers both extract the `IOReportChannels` array before calling state/value APIs
- Property test: mock a delta dictionary with known channel entries, verify state APIs are called on entries not the top-level dict

### Requirement: GPU channel group name [I3-C3]
The system SHALL use `"GPU Stats"` as the IOReport group name and `"GPU Performance States"` as the subgroup name when subscribing to GPU performance state channels. The system SHALL NOT use `"GPU"` as the group name. [T1-static]

> Reference: tech-spec/ioreport.md — Known Group and Sub-group Strings table; "GPU Stats" is the correct group

#### Scenario: GPU channel subscription
- **WHEN** the GPU collector creates its IOReport subscription
- **THEN** it SHALL pass `"GPU Stats"` as the group parameter to `IOReportCopyChannelsInGroup`

### Test Scenarios
- Code inspection: verify the string literal passed to `IOReportCopyChannelsInGroup` for GPU is `"GPU Stats"`, not `"GPU"`
- Unit test: if using a constant, verify the constant value is `"GPU Stats"`

### Requirement: Dynamic energy unit reading [I3-C4]
The system SHALL read energy units dynamically via `IOReportChannelGetUnitLabel` for each energy channel. The system SHALL NOT hardcode nanojoules as the unit for all channels. The unit label (e.g., `"nJ"`, `"uJ"`, `"mJ"`) SHALL determine the divisor used for joule conversion. [T1-static]

> Reference: tech-spec/ioreport.md — IOReportChannelGetUnitLabel section; energy units vary per channel

#### Scenario: Mixed energy units
- **WHEN** the power collector reads energy channels with different unit labels
- **THEN** each channel's energy value SHALL be converted using its own unit label: `"nJ"` divides by 1e9, `"uJ"` by 1e6, `"mJ"` by 1e3

#### Scenario: Unit label loading
- **WHEN** the IOReport FFI module initializes
- **THEN** `IOReportChannelGetUnitLabel` SHALL be loaded via dlsym alongside the other IOReport function pointers

### Test Scenarios
- Code inspection: verify `IOReportChannelGetUnitLabel` is declared in the FFI function pointer struct and loaded via dlsym
- Unit test: verify the conversion function handles `"nJ"`, `"uJ"`, and `"mJ"` labels with correct divisors
- Property test: given known energy values and unit labels, verify watts output matches expected values within tolerance

### Requirement: Energy channel name matching [I3-C5]
The system SHALL use the following channel name matching patterns for energy channels: CPU energy channels SHALL be matched by `name.ends_with("CPU Energy")`, GPU energy SHALL be matched by `name == "GPU Energy"` (exact match), ANE energy SHALL be matched by `name.starts_with("ANE")`. [T1-static]

> Reference: tech-spec/ioreport.md — Energy Channel Names table; patterns handle chip-generation variation (e.g., Ultra chips with "DIE_0_CPU Energy")

#### Scenario: CPU energy on Ultra chip
- **WHEN** the system has energy channels named `"DIE_0_CPU Energy"` and `"DIE_1_CPU Energy"`
- **THEN** both SHALL be matched as CPU energy channels via `ends_with("CPU Energy")`

#### Scenario: GPU energy exact match
- **WHEN** the system has an energy channel named `"GPU Energy"`
- **THEN** it SHALL be matched as the GPU energy channel

#### Scenario: ANE energy prefix match
- **WHEN** the system has an energy channel named `"ANE0 Energy"`
- **THEN** it SHALL be matched as an ANE energy channel via `starts_with("ANE")`

### Test Scenarios
- Unit test: verify matching function correctly classifies `"ECPU0 Energy"`, `"PCPU0 Energy"`, `"DIE_0_CPU Energy"`, `"GPU Energy"`, `"ANE0 Energy"`, `"DRAM0 Energy"` into the correct categories
- Unit test: verify `"GPU0 Energy"` is NOT matched by exact `"GPU Energy"` check (it should fall through to a more general matcher or be handled separately)

### Requirement: SMC endpoint targeting [I3-C6]
The system SHALL target the `AppleSMCKeysEndpoint` service (not just `AppleSMC`) for SMC connection. The system SHALL use `IOServiceGetMatchingServices` (plural) to iterate all matches and find the entry named `"AppleSMCKeysEndpoint"`. [T1-static]

> Reference: tech-spec/smc.md — "Opening an SMC Connection" section; AppleSMCKeysEndpoint is required for reliable key access on all Mac models

#### Scenario: SMC service discovery
- **WHEN** the temperature collector opens an SMC connection
- **THEN** it SHALL iterate IOService matches for `"AppleSMC"` and select the entry whose registry name is `"AppleSMCKeysEndpoint"`

#### Scenario: Endpoint not found
- **WHEN** no `AppleSMCKeysEndpoint` service exists in the IORegistry
- **THEN** the system SHALL report temperature as unavailable rather than crashing

### Test Scenarios
- Code inspection: verify `IOServiceGetMatchingServices` (plural, with iterator) is used instead of `IOServiceGetMatchingService` (singular)
- Code inspection: verify the iterator loop checks `IORegistryEntryGetName` for `"AppleSMCKeysEndpoint"`

### Requirement: Apple Silicon flt temperature type [I3-C7]
The system SHALL support the `flt ` (IEEE 754 f32, 4 bytes) data type when reading SMC temperature keys on Apple Silicon. The system SHALL NOT assume all temperature keys use `sp78` format. [T1-static]

> Reference: tech-spec/smc.md — Data Type Decoding table; Apple Silicon uses `flt ` not `sp78` for temperature keys

#### Scenario: Apple Silicon temperature decoding
- **WHEN** an SMC temperature key has `dataType == "flt "` and `dataSize == 4`
- **THEN** the value SHALL be decoded as `f32::from_be_bytes(bytes[0..4])`

#### Scenario: Intel temperature decoding
- **WHEN** an SMC temperature key has `dataType == "sp78"` and `dataSize == 2`
- **THEN** the value SHALL be decoded as `(bytes[0] << 8 | bytes[1]) as i16 as f32 / 256.0`

### Test Scenarios
- Unit test: verify `flt ` decoding of known byte patterns produces correct temperature values (e.g., 42.5 C)
- Unit test: verify `sp78` decoding still works for Intel compatibility
- Unit test: verify the decoder dispatches on `dataType` field, not hardcoded format

### Requirement: Dynamic SMC key enumeration [I3-C8]
The system SHALL enumerate SMC keys dynamically when possible, using `SMC_CMD_READ_INDEX` to discover available temperature keys. When dynamic enumeration is not available or fails, the system SHALL fall back to a static key list that includes Apple Silicon keys (Tp01-Tp09, Te01-Te04, Tg05, Tg0f, Tg0j). [T2-soak]

> Reference: tech-spec/smc.md — "Dynamic Key Enumeration" section; key names vary by chip generation

#### Scenario: Dynamic enumeration success
- **WHEN** the `#KEY` count read succeeds
- **THEN** the system SHALL enumerate all keys via `SMC_CMD_READ_INDEX`, filter by temperature-relevant prefixes (`Tp`, `Te`, `Ts`, `Tg`, `TC`, `TG`) and `flt ` or `sp78` data type

#### Scenario: Dynamic enumeration fallback
- **WHEN** the `#KEY` count read fails
- **THEN** the system SHALL try the static key list: TC0P, TC0p, Tp01-Tp09, Te01-Te04, TG0P, Tg0P, Tg05, Tg0f, Tg0j

### Test Scenarios
- Integration test (real hardware): run dynamic enumeration and verify at least one temperature key is discovered
- Unit test: verify static fallback list contains both Intel (TC0P, TG0P) and Apple Silicon (Tp01, Te01, Tg05) keys

### Requirement: Debug sensor enumeration [I3-C9]
The `debug_info()` output SHALL list all discovered SMC key names and all discovered IOReport channel names. [T1-static]

> Reference: challenge-iteration3.md — "Debug" section; aids hardware-specific diagnosis

#### Scenario: Debug output with sensors available
- **WHEN** `debug_info()` is called on a system with SMC and IOReport access
- **THEN** the output SHALL include a section listing SMC key names (e.g., "Tp01", "Te01", "Tg05") and a section listing IOReport channel names (e.g., "GPUPH", "ECPU0 Energy")

#### Scenario: Debug output with sensors unavailable
- **WHEN** SMC or IOReport is not available
- **THEN** the output SHALL indicate which subsystem is unavailable, not omit the section silently

### Test Scenarios
- Code inspection: verify `debug_info()` calls SMC key enumeration and IOReport channel listing
- Integration test (real hardware): run `mtop debug` and verify output contains at least one SMC key and one IOReport channel

### Requirement: Mach port count stability [I3-C10]
Mach port count SHALL NOT grow across 100 consecutive samples. The port count after 100 samples SHALL equal the port count after the initial warmup sample (tolerance: 0). [T2-soak]

> Reference: challenge-iteration3.md — Challenge 4; forensic test for Mach port leak regression

#### Scenario: Port count stability
- **WHEN** the sampler completes 100 consecutive sample cycles after an initial warmup
- **THEN** the Mach port count delta SHALL be 0

### Test Scenarios
- Soak test (real hardware, 60s timeout): create Sampler, run 1 warmup sample, record port count (via /dev/fd count or mach_port_space_info), run 100 samples, record port count again, assert delta == 0

### Requirement: IOReport subscription count stability [I3-C11]
IOReport subscription count SHALL NOT grow across 100 consecutive samples. `IOReportCreateSubscription` SHALL be called at most once per channel group per Sampler lifetime. [T2-soak]

> Reference: challenge-iteration3.md — Challenge 4; forensic test for subscription leak regression

#### Scenario: Subscription count stability
- **WHEN** the sampler completes 100 consecutive sample cycles
- **THEN** `IOReportCreateSubscription` SHALL have been called exactly once per channel group (once for GPU, once for power)

### Test Scenarios
- Soak test (real hardware, 60s timeout): wrap `IOReportCreateSubscription` in a counted wrapper (cfg(test)), run 100 samples, assert call count == 1 per channel group
- Alternative: FD-count stability test — record FD count before and after 100 samples, assert delta <= 0

## Non-Functional Requirements

### Resource Safety
The metrics collector SHALL NOT leak Mach ports, file descriptors, IOKit connections, or IOReport subscriptions across sample cycles. [T2-soak]

All owned CoreFoundation objects (CFDictionaryRef, CFStringRef from Copy/Create functions) SHALL be released via `CFRelease` when no longer needed. Borrowed references (from Get-rule functions like `IOReportChannelGetChannelName`, `IOReportStateGetNameForIndex`) SHALL NOT be released by the caller. [T1-static]

### Performance
A single sample cycle (all metrics) SHALL complete in under 200ms on Apple Silicon hardware at default configuration. [T2-soak]

The `dlopen` handle for IOReport.framework SHALL be loaded once (via `OnceLock` or equivalent) and SHALL NOT be closed during process lifetime to avoid use-after-free on active function pointers. [T1-static]

### Correctness
All IOReport API calls SHALL follow the documented calling conventions: correct parameter types, correct ownership semantics (Create/Copy rule vs Get rule), and correct iteration patterns (channel entries, not top-level dictionaries). [T1-static]

SMC data decoding SHALL dispatch on the `dataType` field returned by `READ_KEYINFO`, supporting at minimum `sp78` (signed 7.8 fixed-point) and `flt ` (IEEE 754 f32). [T1-static]

### Resilience
The system SHALL continue collecting all other metrics when any single sensor subsystem fails (IOReport unavailable, SMC connection refused, individual key read failure). Each failure SHALL be logged at debug level, not silently swallowed. [T1-static]

The `cfstring_to_string` helper SHALL guard against `CFStringGetMaximumSizeForEncoding` returning `kCFNotFound` (-1) by returning an empty string instead of allocating a zero-length buffer. [T1-static]

### Security
The metrics collector SHALL NOT require or attempt to acquire elevated privileges. All APIs used SHALL be accessible to unprivileged user processes. [T1-static]

### Longevity
Channel name matching patterns SHALL use flexible matching (ends_with, starts_with, exact match) rather than hardcoded full channel names, to accommodate new chip generations that may introduce new channel name prefixes (e.g., Ultra chips with `"DIE_0_"` prefix). [T1-static]

SMC key discovery SHALL prefer dynamic enumeration over hardcoded key lists, falling back to static lists only when enumeration fails, to support future chip generations with new key names. [T2-soak]

## Breakdown: IOReport Collection Chain

### Subscription lifecycle
The IOReport subscription SHALL be created once during collector initialization and reused for all subsequent samples. The subscription SHALL be released on collector shutdown (Drop implementation). The system SHALL NOT recreate the subscription per sample cycle. [T1-static]

### Channel groups
- `"Energy Model"` channel group SHALL be used for power metrics (CPU, GPU, ANE, DRAM energy)
- `"GPU Stats"` / `"GPU Performance States"` channel group SHALL be used for GPU utilization and frequency metrics
- Both channel groups MAY share a single dlopen handle and function pointer struct

### Delta computation
Two IOReport samples SHALL be taken with a measured time interval between them. Energy delta SHALL be converted to power (watts) using dynamically-read unit labels per channel. The measured interval SHALL use a monotonic clock (Instant::now()), not the requested sleep duration.

### Thread safety
All IOReport function calls SHALL execute on a single thread or be guarded by a mutex. IOReport functions are NOT thread-safe per framework behavior. If GPU and power collectors share a subscription, access SHALL be serialized.

### State names for GPU frequency
IOReportStateGetNameForIndex SHALL be called for each GPU P-state index to retrieve the state name. The last 4 characters of the name (format "GPUPH_XXXX_YYYY") SHALL be parsed as decimal MHz. Index 0 is always the idle state (0 MHz). The returned CFStringRef is borrowed (Get rule) and SHALL NOT be released by the caller.

> Reference: tech-spec/ioreport.md — full IOReport API lifecycle; tech-spec/corefoundation.md — Get rule for borrowed references

## Breakdown: SMC Sensor Reading

### Connection lifecycle
The SMC connection SHALL be opened once via IOServiceOpen targeting `AppleSMCKeysEndpoint` during collector initialization. The io_connect_t handle SHALL be stored in collector state and reused for all temperature reads. IOServiceClose SHALL be called on shutdown (Drop implementation).

### Key encoding
SMC 4-character key names SHALL be encoded as big-endian u32 values. Example: "TC0P" encodes to 0x54_43_30_50 via u32::from_be_bytes().

### Read protocol
Each SMC key read follows a 2-step protocol:
1. READ_KEYINFO (selector 9, data8) — retrieve type tag and data size
2. READ_BYTES (selector 5, data8) — read the actual value bytes

### Temperature keys
Primary temperature keys with fallback chain:
- CPU die: TC0P, TC0p, Tp01 (try in order, use first successful)
- GPU die: TG0P, Tg0P, Tg05 (try in order, use first successful)
- If all keys fail for a sensor, report the metric as unavailable (not 0.0)
- When dynamic enumeration is available, prefer discovered keys over the static fallback chain

### Data type decoding
The SMC reader SHALL support at minimum:
- `sp78`: signed 7.8 fixed-point (2 bytes, big-endian) — `(bytes[0] << 8 | bytes[1]) as i16 as f32 / 256.0`
- `flt `: IEEE 754 f32 (4 bytes, big-endian) — `f32::from_be_bytes(bytes[0..4])`

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
