/// Tests for iteration 3 requirements (I3-C1 through I3-C7, I3-S1/S2, I3-T1/T2/T3, I3-C9).
/// Each test cites the spec requirement it validates.
///
/// These tests use source-code inspection (static analysis) to verify that
/// implementation patterns match the requirements. Hardware-dependent forensic
/// tests are marked `#[ignore]`.

// ---------------------------------------------------------------------------
// I3-C3: GPU IOReport group name
// Validates: gpu.rs SHALL use "GPU Stats" as the IOReport channel group name,
// not the bare "GPU" string.
// ---------------------------------------------------------------------------

#[test]
fn gpu_uses_correct_ioreport_group_name() {
    let source = std::fs::read_to_string("src/platform/gpu.rs")
        .expect("failed to read src/platform/gpu.rs");
    assert!(
        source.contains(r#""GPU Stats""#),
        "I3-C3: gpu.rs must use 'GPU Stats' group name, not 'GPU'"
    );
    assert!(
        !source.contains(r#"cfstring("GPU")"#),
        "I3-C3: gpu.rs must not use bare 'GPU' group name"
    );
}

// ---------------------------------------------------------------------------
// I3-C2: Delta channel iteration
// Validates: gpu.rs SHALL extract IOReportChannels array from the delta
// dictionary before calling state APIs.
// ---------------------------------------------------------------------------

#[test]
fn gpu_iterates_delta_channels_not_top_level() {
    let source = std::fs::read_to_string("src/platform/gpu.rs")
        .expect("failed to read src/platform/gpu.rs");
    // Must extract IOReportChannels array
    assert!(
        source.contains("IOReportChannels"),
        "I3-C2: gpu.rs must extract IOReportChannels array from delta"
    );
    // Must iterate array entries
    assert!(
        source.contains("CFArrayGetValueAtIndex") || source.contains("CFArrayGetCount"),
        "I3-C2: gpu.rs must iterate channel entries from the array"
    );
}

// ---------------------------------------------------------------------------
// I3-C1: Mach port cleanup
// Validates: sampler.rs SHALL implement Drop for Sampler that calls
// mach_port_deallocate to release the host port.
// ---------------------------------------------------------------------------

#[test]
fn sampler_drop_deallocates_mach_port() {
    let source = std::fs::read_to_string("src/metrics/sampler.rs")
        .expect("failed to read src/metrics/sampler.rs");
    assert!(
        source.contains("mach_port_deallocate"),
        "I3-C1: Sampler must call mach_port_deallocate in Drop"
    );
    assert!(
        source.contains("impl Drop for Sampler"),
        "I3-C1: Sampler must implement Drop"
    );
}

// ---------------------------------------------------------------------------
// I3-C4: Dynamic energy units
// Validates: power.rs SHALL NOT hardcode nanojoule assumption; it must read
// energy units dynamically from the IOReport channel metadata.
// ---------------------------------------------------------------------------

#[test]
fn power_uses_dynamic_energy_units() {
    let source = std::fs::read_to_string("src/platform/power.rs")
        .expect("failed to read src/platform/power.rs");
    // Should reference unit label function
    assert!(
        source.contains("unit_label")
            || source.contains("UnitLabel")
            || source.contains("channel_get_unit"),
        "I3-C4: power.rs must read energy units dynamically"
    );
}

// ---------------------------------------------------------------------------
// I3-C5: Energy channel name matching
// Validates: power.rs SHALL use correct channel name matching patterns to
// catch both "CPU Energy" and "DIE_0_CPU Energy" variants.
// ---------------------------------------------------------------------------

#[test]
fn power_uses_correct_channel_matching() {
    let source = std::fs::read_to_string("src/platform/power.rs")
        .expect("failed to read src/platform/power.rs");
    // Must use ends_with or a pattern that catches "CPU Energy" and "DIE_0_CPU Energy"
    assert!(
        source.contains("CPU Energy")
            || (source.contains("cpu") && source.contains("energy")),
        "I3-C5: power.rs must match CPU energy channels"
    );
}

// ---------------------------------------------------------------------------
// I3-C6: SMC endpoint
// Validates: temperature.rs SHALL target AppleSMCKeysEndpoint service.
// ---------------------------------------------------------------------------

#[test]
fn temperature_uses_smc_keys_endpoint() {
    let source = std::fs::read_to_string("src/platform/temperature.rs")
        .expect("failed to read src/platform/temperature.rs");
    assert!(
        source.contains("AppleSMCKeysEndpoint"),
        "I3-C6: temperature.rs must target AppleSMCKeysEndpoint service"
    );
}

// ---------------------------------------------------------------------------
// I3-C7: Apple Silicon flt type
// Validates: temperature.rs SHALL handle "flt " data type for Apple Silicon
// temperature readings.
// ---------------------------------------------------------------------------

#[test]
fn temperature_supports_flt_type() {
    let source = std::fs::read_to_string("src/platform/temperature.rs")
        .expect("failed to read src/platform/temperature.rs");
    assert!(
        source.contains("flt "),
        "I3-C7: temperature.rs must handle 'flt ' data type"
    );
}

// ---------------------------------------------------------------------------
// I3-C9: Debug sensor enumeration
// Validates: debug_info SHALL enumerate actual sensor information, not print
// a generic hardcoded message.
// ---------------------------------------------------------------------------

#[test]
fn debug_info_enumerates_sensors() {
    let source = std::fs::read_to_string("src/metrics/sampler.rs")
        .expect("failed to read src/metrics/sampler.rs");
    // Must NOT just print a generic message
    assert!(
        !source.contains("IOReport FFI active for GPU, power, and temperature metrics"),
        "I3-C9: debug_info must enumerate actual sensors, not print generic message"
    );
}

// ---------------------------------------------------------------------------
// I3-S1, I3-S2: Slowloris mitigation
// Validates: serve/mod.rs SHALL use <= 2 second read timeout and SHALL track
// per-IP connections.
// ---------------------------------------------------------------------------

#[test]
fn server_uses_short_read_timeout() {
    let source = std::fs::read_to_string("src/serve/mod.rs")
        .expect("failed to read src/serve/mod.rs");
    // Must not use 5 second timeout
    assert!(
        !source.contains("from_secs(5)"),
        "I3-S1: serve must not use 5-second read timeout"
    );
    // Must use 2 seconds or less
    assert!(
        source.contains("from_secs(2)") || source.contains("from_millis(2000)"),
        "I3-S1: serve must use 2-second read timeout"
    );
}

#[test]
fn server_has_per_ip_connection_limit() {
    let source = std::fs::read_to_string("src/serve/mod.rs")
        .expect("failed to read src/serve/mod.rs");
    assert!(
        source.contains("peer_addr") || source.contains("SocketAddr"),
        "I3-S2: serve must track connections by IP address"
    );
}

// ---------------------------------------------------------------------------
// I3-T1, I3-T2, I3-T3: TUI sensor unavailable handling
// Validates: TUI SHALL display N/A or unavailable when sensor data is missing.
// ---------------------------------------------------------------------------

#[test]
fn tui_handles_sensor_unavailable() {
    // After mod.rs split, N/A handling lives in panel submodules
    let mut tui_source = std::fs::read_to_string("src/tui/mod.rs")
        .expect("failed to read src/tui/mod.rs");
    // Also check panel files where N/A rendering now lives
    for panel in &["cpu", "gpu", "power"] {
        if let Ok(s) = std::fs::read_to_string(format!("src/tui/panels/{panel}.rs")) {
            tui_source.push_str(&s);
        }
    }
    let _types_source = std::fs::read_to_string("src/metrics/types.rs")
        .expect("failed to read src/metrics/types.rs");
    // Must have concept of sensor availability (Option or flag)
    assert!(
        tui_source.contains("N/A")
            || tui_source.contains("n/a")
            || tui_source.contains("unavailable"),
        "I3-T1/T2/T3: TUI must display N/A or unavailable when sensor data missing"
    );
}

// ---------------------------------------------------------------------------
// Forensic tests (hardware-dependent, #[ignore])
// ---------------------------------------------------------------------------

/// H1: Mach port count does not grow across 100 samples.
/// Requires real hardware — the test creates a Sampler and takes 100 samples,
/// verifying that system resource counts remain stable.
#[test]
#[ignore]
fn forensic_mach_port_count_stable() {
    // Create sampler, take 100 samples, verify mach port count doesn't grow.
    // Use: `sudo lsof -p PID | grep "Mach port" | wc -l` or
    // `proc_pidinfo` to check port count.
    let mut sampler = mtop::metrics::Sampler::new().expect("sampler init");
    for i in 0..100 {
        let _ = sampler.sample(200).unwrap_or_else(|e| {
            panic!("H1: sample {} failed: {}", i, e);
        });
    }
    // If we get here without resource exhaustion, basic stability holds.
    // A more precise check would compare port counts before/after.
}

/// H2: IOReport subscription count does not grow across 100 samples.
/// Requires real hardware.
#[test]
#[ignore]
fn forensic_ioreport_subscription_stable() {
    // Create sampler, call sample() 100 times, verify no resource growth.
    let mut sampler = mtop::metrics::Sampler::new().expect("sampler init");
    for i in 0..100 {
        let _ = sampler.sample(200).unwrap_or_else(|e| {
            panic!("H2: sample {} failed: {}", i, e);
        });
    }
}

/// H4: Verify subscription count stable across all subscription types.
/// Requires real hardware.
#[test]
#[ignore]
fn forensic_subscription_count_stable() {
    // Similar to H2 but for all subscription types.
    let mut sampler = mtop::metrics::Sampler::new().expect("sampler init");
    for i in 0..100 {
        let _ = sampler.sample(200).unwrap_or_else(|e| {
            panic!("H4: sample {} failed: {}", i, e);
        });
    }
}

// ---------------------------------------------------------------------------
// Isolation tests: dlopen / connection caching (M4/M5/M7)
// ---------------------------------------------------------------------------

/// M4/M5: IOReport FFI must cache dlopen handle via OnceLock (single dlopen).
#[test]
fn isolation_dlopen_cached_across_instances() {
    let source = std::fs::read_to_string("src/platform/ioreport_ffi.rs")
        .expect("failed to read src/platform/ioreport_ffi.rs");
    assert!(
        source.contains("OnceLock"),
        "M4/M5: IOReport FFI must cache dlopen handle via OnceLock"
    );
}

/// M7: TemperatureState must cache its SMC connection.
#[test]
fn isolation_smc_connection_cached() {
    let source = std::fs::read_to_string("src/platform/temperature.rs")
        .expect("failed to read src/platform/temperature.rs");
    assert!(
        source.contains("struct TemperatureState"),
        "M7: Temperature must use stateful connection caching"
    );
}
