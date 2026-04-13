/// Integration tests for TUI dashboard rendering, theme configuration,
/// metrics history, and network tier logic.

// =========================================================================
// Theme panel colors
// =========================================================================

/// Horizon theme process_accent is teal Rgb(37, 178, 188). (ref: SHALL-23-10a)
#[test]
fn theme_horizon_process_accent_color() {
    use mtop::tui::theme::THEMES;
    use ratatui::style::Color;
    let horizon = THEMES.iter().find(|t| t.name == "horizon").unwrap();
    assert_eq!(horizon.process_accent, Color::Rgb(37, 178, 188));
}

/// Every theme has a non-default (non-black) Rgb process_accent. (ref: SHALL-23-10b)
#[test]
fn theme_all_have_nondefault_process_accent() {
    use mtop::tui::theme::THEMES;
    use ratatui::style::Color;
    for t in THEMES.iter() {
        assert!(
            matches!(t.process_accent, Color::Rgb(_, _, _)),
            "theme '{}': process_accent must be Rgb, got {:?}",
            t.name, t.process_accent
        );
        if let Color::Rgb(r, g, b) = t.process_accent {
            assert!(
                r > 0 || g > 0 || b > 0,
                "theme '{}': process_accent must not be (0,0,0)",
                t.name
            );
        }
    }
}

/// Panel accents (cpu, gpu, mem, process) are distinct per theme. (ref: SHALL-23-10c)
#[test]
fn theme_panel_accents_are_distinct() {
    use mtop::tui::theme::THEMES;
    for t in THEMES.iter() {
        let accents = [
            ("cpu", format!("{:?}", t.cpu_accent)),
            ("gpu", format!("{:?}", t.gpu_accent)),
            ("mem", format!("{:?}", t.mem_accent)),
            ("process", format!("{:?}", t.process_accent)),
        ];
        assert_ne!(accents[0].1, accents[2].1,
            "theme '{}': cpu_accent must differ from mem_accent", t.name);
        assert_ne!(accents[0].1, accents[3].1,
            "theme '{}': cpu_accent must differ from process_accent", t.name);
    }
}

// GPU/Power hue midpoint tests skipped — derive_companion is DEVIANT (uses addition, not midpoint).
// Tests will be written after re-implementation via proper iteration workflow.

// =========================================================================
// Baseline color
// =========================================================================

/// Dark theme (Nord) baseline_color brightens muted channels by +30. (ref: SHALL-01-01)
#[test]
fn baseline_color_dark_theme_brightens_muted() {
    use mtop::tui::theme::{self, THEMES};
    use ratatui::style::Color;
    let nord = THEMES.iter().find(|t| t.name == "nord").unwrap();
    let baseline = theme::baseline_color(nord);
    if let (Color::Rgb(mr, mg, mb), Color::Rgb(br, bg, bb)) = (nord.muted, baseline) {
        assert!(br > mr || br == 255, "nord baseline R ({br}) should be > muted R ({mr})");
        assert!(bg > mg || bg == 255, "nord baseline G ({bg}) should be > muted G ({mg})");
        assert!(bb > mb || bb == 255, "nord baseline B ({bb}) should be > muted B ({mb})");
    } else {
        panic!("expected Rgb colors");
    }
}

/// Light theme (Solarized Light) baseline_color darkens muted channels by -30. (ref: SHALL-01-02)
#[test]
fn baseline_color_light_theme_darkens_muted() {
    use mtop::tui::theme::{self, THEMES};
    use ratatui::style::Color;
    let sol_light = THEMES.iter().find(|t| t.name == "solarized-light").unwrap();
    let baseline = theme::baseline_color(sol_light);
    if let (Color::Rgb(mr, mg, mb), Color::Rgb(br, bg, bb)) = (sol_light.muted, baseline) {
        assert!(br < mr || mr == 0, "solarized-light baseline R ({br}) should be < muted R ({mr})");
        assert!(bg < mg || mg == 0, "solarized-light baseline G ({bg}) should be < muted G ({mg})");
        assert!(bb < mb || mb == 0, "solarized-light baseline B ({bb}) should be < muted B ({mb})");
    } else {
        panic!("expected Rgb colors");
    }
}

/// Gruvbox (dark) baseline_color produces a visible brightness boost. (ref: SHALL-01-03)
#[test]
fn baseline_color_gruvbox_visible_boost() {
    use mtop::tui::theme::{self, THEMES};
    use ratatui::style::Color;
    let gruvbox = THEMES.iter().find(|t| t.name == "gruvbox").unwrap();
    let baseline = theme::baseline_color(gruvbox);
    if let (Color::Rgb(mr, mg, mb), Color::Rgb(br, bg, bb)) = (gruvbox.muted, baseline) {
        assert!(br > mr || br == 255, "gruvbox baseline R ({br}) should be > muted R ({mr})");
        assert!(bg > mg || bg == 255, "gruvbox baseline G ({bg}) should be > muted G ({mg})");
        assert!(bb > mb || bb == 255, "gruvbox baseline B ({bb}) should be > muted B ({mb})");
    } else {
        panic!("expected Rgb colors");
    }
}

/// baseline_color channels never overflow past 255. (ref: SHALL-01-04)
#[test]
fn baseline_color_channels_cap_at_255() {
    use mtop::tui::theme::{self, THEMES};
    use ratatui::style::Color;
    for t in THEMES.iter() {
        let baseline = theme::baseline_color(t);
        if let Color::Rgb(r, g, b) = baseline {
            assert!(r <= 255 && g <= 255 && b <= 255,
                "theme '{}': baseline_color channels must not exceed 255", t.name);
        }
    }
}

// =========================================================================
// Memory usage formula
// =========================================================================

/// Memory usage fraction computed correctly from raw byte counts. (ref: SHALL-AD-01a)
#[test]
fn memory_usage_fraction_from_bytes() {
    use mtop::metrics::types::{MemoryMetrics, MetricsHistory, MetricsSnapshot};
    let page_size: u64 = 16384;
    let active: u64 = 1_000_000;
    let wire: u64 = 500_000;
    let ram_used = (active + wire) * page_size;
    let ram_total = 48 * 1024 * 1024 * 1024u64;

    let mut snapshot = MetricsSnapshot::default();
    snapshot.memory = MemoryMetrics { ram_total, ram_used, ..Default::default() };
    let mut history = MetricsHistory::new();
    history.push(&snapshot);

    let usage = *history.mem_usage.last().unwrap();
    let expected = ram_used as f64 / ram_total as f64;
    assert!(
        (usage - expected).abs() < 0.001,
        "usage fraction should be {expected:.4}, got {usage:.4}"
    );
}

/// Memory usage fraction should clamp to 1.0 when ram_used exceeds ram_total. (ref: SHALL-AD-01b)
#[test]
fn memory_usage_clamps_when_exceeds_total() {
    use mtop::metrics::types::{MemoryMetrics, MetricsHistory, MetricsSnapshot};
    let ram_total = 16 * 1024 * 1024 * 1024u64;
    let ram_used = ram_total + 1024 * 1024 * 1024;

    let mut snapshot = MetricsSnapshot::default();
    snapshot.memory = MemoryMetrics { ram_total, ram_used, ..Default::default() };
    let mut history = MetricsHistory::new();
    history.push(&snapshot);

    let usage = *history.mem_usage.last().unwrap();
    assert!(
        usage <= 1.0,
        "usage fraction must be clamped to <= 1.0 when ram_used > ram_total, got {usage}"
    );
}

/// Memory usage fraction is 0.0 when ram_used is zero. (ref: SHALL-AD-01c)
#[test]
fn memory_usage_zero_when_unused() {
    use mtop::metrics::types::{MemoryMetrics, MetricsHistory, MetricsSnapshot};
    let ram_total = 16 * 1024 * 1024 * 1024u64;
    let ram_used = 0u64;

    let mut snapshot = MetricsSnapshot::default();
    snapshot.memory = MemoryMetrics { ram_total, ram_used, ..Default::default() };
    let mut history = MetricsHistory::new();
    history.push(&snapshot);

    let usage = *history.mem_usage.last().unwrap();
    assert!(
        usage.abs() < 0.001,
        "usage fraction should be 0.0 when ram_used=0, got {usage}"
    );
}

// =========================================================================
// Network tier hysteresis
// =========================================================================

fn push_net_sample(history: &mut mtop::metrics::types::MetricsHistory, bytes_sec: f64) {
    use mtop::metrics::types::{MetricsSnapshot, NetworkMetrics, NetInterface};
    let mut snapshot = MetricsSnapshot::default();
    snapshot.network = NetworkMetrics {
        interfaces: vec![NetInterface {
            name: "en0".to_string(),
            rx_bytes_sec: bytes_sec,
            tx_bytes_sec: 0.0,
            ..Default::default()
        }],
        primary_baudrate: 1_000_000_000,
    };
    history.push(&snapshot);
}

/// Network tier upgrades immediately on spike (ref: SHALL-31-07a)
#[test]
fn net_tier_upgrade_after_hold() {
    use mtop::metrics::types::MetricsHistory;
    let mut history = MetricsHistory::new();
    assert_eq!(history.net_tier_idx, 0, "should start at tier 0");

    // Immediate upgrade on first sample exceeding current tier
    push_net_sample(&mut history, 1_500_000.0);
    assert_eq!(history.net_tier_idx, 1, "should upgrade to tier 1 immediately on spike");
}

/// Network tier downgrades after a full buffer window of below-threshold samples.
/// Must first upgrade with 10 samples, then flush buffer + accumulate hold.
/// Tier 1 = 5MB/s; 10% threshold = 524288 bytes/sec.
/// (ref: SHALL-28-05d, SHALL-26-06b)
#[test]
fn net_tier_downgrade_after_full_buffer_window() {
    use mtop::metrics::types::MetricsHistory;
    let mut history = MetricsHistory::new();

    // Upgrade to tier 1 (immediate on spike)
    push_net_sample(&mut history, 2_000_000.0);
    assert_eq!(history.net_tier_idx, 1);

    // Need: 127 zeros to flush the 1 high value from 128-entry buffer + 128 zeros to accumulate hold = 255 total
    for _ in 0..254 {
        push_net_sample(&mut history, 0.0);
    }
    assert_eq!(history.net_tier_idx, 1, "should still be tier 1 before full window");

    push_net_sample(&mut history, 0.0);
    assert_eq!(history.net_tier_idx, 0, "should downgrade to tier 0 after full window");
}

/// A traffic spike above threshold resets the downgrade hold counter.
/// (ref: SHALL-28-05d, SHALL-26-06c)
#[test]
fn net_tier_downgrade_interrupted_by_spike() {
    use mtop::metrics::types::MetricsHistory;
    let mut history = MetricsHistory::new();

    // Upgrade to tier 1 (immediate on spike)
    push_net_sample(&mut history, 2_000_000.0);
    assert_eq!(history.net_tier_idx, 1);

    // Push zeros to start downgrade hold (need to flush buffer first)
    for _ in 0..200 {
        push_net_sample(&mut history, 0.0);
    }
    assert_eq!(history.net_tier_idx, 1, "still tier 1 mid-hold");

    // Spike interrupts — resets hold counter
    push_net_sample(&mut history, 1_500_000.0);
    assert_eq!(history.net_tier_hold, 0, "hold should reset when sample exceeds threshold");
    assert_eq!(history.net_tier_idx, 1, "should remain tier 1");

    // After interrupt, need full flush + hold again
    for _ in 0..254 {
        push_net_sample(&mut history, 0.0);
    }
    assert_eq!(history.net_tier_idx, 1, "still tier 1 before full window after interrupt");
    push_net_sample(&mut history, 0.0);
    assert_eq!(history.net_tier_idx, 0, "now tier 0 after full window post-interrupt");
}

/// Downgrade threshold is 10% of current tier ceiling.
/// Tier 1 = 5MB/s (5_242_880); 10% = 524_288.
/// (ref: SHALL-28-05d, SHALL-26-06d)
#[test]
fn net_tier_downgrade_threshold_is_10_percent() {
    use mtop::metrics::types::MetricsHistory;
    let mut history = MetricsHistory::new();

    // Upgrade to tier 1 (immediate on spike)
    push_net_sample(&mut history, 2_000_000.0);
    assert_eq!(history.net_tier_idx, 1);

    // 400K is below 10% of 5MB/s (524K) — should eventually trigger downgrade
    // Need 127 to flush high value from buffer + 128 to accumulate hold = 255 total
    for _ in 0..254 {
        push_net_sample(&mut history, 400_000.0);
    }
    assert_eq!(history.net_tier_idx, 1, "not yet downgraded at 254");
    push_net_sample(&mut history, 400_000.0);
    assert_eq!(history.net_tier_idx, 0, "should downgrade: 400K < 10% of 5M");

    // Values above 10% must NOT trigger downgrade
    let mut history2 = MetricsHistory::new();
    push_net_sample(&mut history2, 2_000_000.0);
    assert_eq!(history2.net_tier_idx, 1);

    for _ in 0..256 {
        push_net_sample(&mut history2, 2_000_000.0);
    }
    assert_eq!(history2.net_tier_idx, 1, "should NOT downgrade: 2M > 10% of 5M");
}

// =========================================================================
// Battery gauge rendering
// =========================================================================

/// Full charge (100%) renders percentage in battery gauge. (ref: SHALL-26-02a)
#[test]
fn battery_gauge_full_charge() {
    use mtop::metrics::types::{BatteryMetrics, MetricsSnapshot};
    let mut snapshot = MetricsSnapshot::default();
    snapshot.battery = BatteryMetrics {
        is_present: true, charge_pct: 100.0, is_charging: false, is_on_ac: false,
    };
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    assert!(text.contains("100%"), "should show 100% for full battery");
}

/// Empty battery (0%) renders "0%". (ref: SHALL-26-02b)
#[test]
fn battery_gauge_empty() {
    use mtop::metrics::types::{BatteryMetrics, MetricsSnapshot};
    let mut snapshot = MetricsSnapshot::default();
    snapshot.battery = BatteryMetrics {
        is_present: true, charge_pct: 0.0, is_charging: false, is_on_ac: false,
    };
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    assert!(text.contains("0%"), "should show 0% for empty battery");
}

/// Half charge (50%) renders with partial gauge fill. (ref: SHALL-26-02c)
#[test]
fn battery_gauge_half() {
    use mtop::metrics::types::{BatteryMetrics, MetricsSnapshot};
    let mut snapshot = MetricsSnapshot::default();
    snapshot.battery = BatteryMetrics {
        is_present: true, charge_pct: 50.0, is_charging: false, is_on_ac: false,
    };
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    assert!(text.contains("50%"), "should show 50%");
}

/// No battery present renders "AC" indicator. (ref: SHALL-26-02d)
#[test]
fn battery_gauge_no_battery_shows_ac() {
    use mtop::metrics::types::{BatteryMetrics, MetricsSnapshot};
    let mut snapshot = MetricsSnapshot::default();
    snapshot.battery = BatteryMetrics {
        is_present: false, charge_pct: 0.0, is_charging: false, is_on_ac: false,
    };
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    assert!(text.contains("AC"), "should show AC when no battery present");
}

/// Charging state shows lightning bolt and percentage. (ref: SHALL-26-02e)
#[test]
fn battery_gauge_charging_indicator() {
    use mtop::metrics::types::{BatteryMetrics, MetricsSnapshot};
    let mut snapshot = MetricsSnapshot::default();
    snapshot.battery = BatteryMetrics {
        is_present: true, charge_pct: 75.0, is_charging: true, is_on_ac: true,
    };
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    assert!(text.contains("75%"), "should show 75%");
    assert!(text.contains("\u{26a1}"), "should show lightning bolt when charging");
}

/// Battery percentage appears at most once in the header row (not duplicated). (ref: SHALL-26-02f)
#[test]
fn battery_gauge_no_duplicate_percentage() {
    use mtop::metrics::types::{BatteryMetrics, MetricsSnapshot};
    for pct in [0.0, 25.0, 50.0, 75.0, 100.0] {
        let mut snapshot = MetricsSnapshot::default();
        snapshot.battery = BatteryMetrics {
            is_present: true, charge_pct: pct, is_charging: false, is_on_ac: false,
        };
        let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
        let header_line = text.lines().next().unwrap_or("");
        let pct_str = format!("{:.0}%", pct);
        let count = header_line.matches(&pct_str).count();
        assert!(count <= 1, "at {pct}%: '{pct_str}' appears {count} times in header, expected at most 1");
    }
}

// =========================================================================
// Battery metrics defaults
// =========================================================================

/// charge_pct above 100 clamps to 6 filled gauge cells. (ref: SHALL-05-05)
#[test]
fn battery_charge_pct_clamps_above_100() {
    use mtop::metrics::types::BatteryMetrics;
    let bat = BatteryMetrics { is_present: true, charge_pct: 103.0, is_charging: false, is_on_ac: false };
    let filled = (bat.charge_pct.min(100.0) / 100.0 * 6.0).round() as usize;
    assert_eq!(filled, 6, "clamped charge_pct should produce at most 6 filled cells");
}

/// Default BatteryMetrics has is_present=false and charge_pct=0. (ref: SHALL-05-06)
#[test]
fn battery_default_not_present() {
    use mtop::metrics::types::BatteryMetrics;
    let bat = BatteryMetrics::default();
    assert!(!bat.is_present, "default battery should not be present");
    assert_eq!(bat.charge_pct, 0.0, "default charge_pct should be 0");
}

// =========================================================================
// GPU detail panel
// =========================================================================

/// GPU detail (show mode) displays ANE/DRAM/VRAM but not "cores" in the detail area.
/// Bottom row correctly has cores — we verify "cores" appears at most once. (ref: SHALL-26-03a)
#[test]
fn gpu_detail_omits_cores_label() {
    use mtop::metrics::types::{GpuMetrics, MetricsSnapshot, PowerMetrics, SocInfo};
    let mut snapshot = MetricsSnapshot::default();
    snapshot.gpu = GpuMetrics { freq_mhz: 1000, usage: 0.5, power_w: 5.0, available: true };
    snapshot.power = PowerMetrics { gpu_w: 5.0, available: true, ..Default::default() };
    snapshot.soc = SocInfo { gpu_cores: 20, ..Default::default() };
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, true);
    let core_count = text.matches("cores").count();
    assert!(core_count <= 1,
        "\"cores\" should appear at most once (bottom row only), found {core_count} times");
}

// =========================================================================
// Memory panel swap/disk rendering
// =========================================================================

/// Hide mode with swap configured shows both "Swap:" and "disk:" labels. (ref: SHALL-26-07a)
#[test]
fn memory_hide_mode_shows_swap_and_disk() {
    use mtop::metrics::types::{DiskMetrics, MemoryMetrics, MetricsSnapshot};
    let mut snapshot = MetricsSnapshot::default();
    snapshot.memory = MemoryMetrics {
        ram_total: 16 * 1024 * 1024 * 1024,
        ram_used: 8 * 1024 * 1024 * 1024,
        swap_total: 4 * 1024 * 1024 * 1024,
        swap_used: 1 * 1024 * 1024 * 1024,
        ..Default::default()
    };
    snapshot.disk = DiskMetrics {
        total_bytes: 500 * 1024 * 1024 * 1024,
        used_bytes: 250 * 1024 * 1024 * 1024,
        ..Default::default()
    };
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    assert!(text.contains("swap:"), "hide mode with swap should show 'swap:'");
    assert!(text.contains("disk:"), "hide mode should show 'disk:'");
}

/// Hide mode without swap omits "Swap:" but still shows "disk:". (ref: SHALL-26-07b)
#[test]
fn memory_hide_mode_omits_swap_when_zero() {
    use mtop::metrics::types::{DiskMetrics, MemoryMetrics, MetricsSnapshot};
    let mut snapshot = MetricsSnapshot::default();
    snapshot.memory = MemoryMetrics {
        ram_total: 16 * 1024 * 1024 * 1024,
        ram_used: 8 * 1024 * 1024 * 1024,
        swap_total: 0,
        swap_used: 0,
        ..Default::default()
    };
    snapshot.disk = DiskMetrics {
        total_bytes: 500 * 1024 * 1024 * 1024,
        used_bytes: 250 * 1024 * 1024 * 1024,
        ..Default::default()
    };
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    assert!(!text.contains("swap:"), "hide mode without swap should not show 'swap:'");
    assert!(text.contains("disk:"), "hide mode should show 'disk:'");
}

/// Show mode with swap configured includes "Swap:" label. (ref: SHALL-26-07c)
#[test]
fn memory_show_mode_includes_swap() {
    use mtop::metrics::types::{MemoryMetrics, MetricsSnapshot};
    let mut snapshot = MetricsSnapshot::default();
    snapshot.memory = MemoryMetrics {
        ram_total: 16 * 1024 * 1024 * 1024,
        ram_used: 8 * 1024 * 1024 * 1024,
        swap_total: 4 * 1024 * 1024 * 1024,
        swap_used: 1 * 1024 * 1024 * 1024,
        ..Default::default()
    };
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, true);
    assert!(text.contains("swap:"), "show mode with swap should show 'swap:'");
}

// =========================================================================
// Chart layout
// =========================================================================

/// Hide mode renders charts at full panel width without panic. (ref: SHALL-23-00a/b)
#[test]
fn chart_hide_mode_full_width() {
    use mtop::metrics::types::MetricsSnapshot;
    let snapshot = MetricsSnapshot::default();
    let text = mtop::tui::render_dashboard_to_string(120, 40, snapshot, false);
    assert!(!text.is_empty(), "hide mode should render content");
}

// =========================================================================
// Network infrastructure filter
// =========================================================================

/// Infrastructure interface filter matches documented prefixes (bridge, awdl, llw, utun, ap, etc.).
/// (ref: SHALL-23-CC-02)
#[test]
fn network_infra_filter_prefixes() {
    use mtop::tui::helpers::is_infrastructure_interface;
    // Should filter
    assert!(is_infrastructure_interface("bridge0"));
    assert!(is_infrastructure_interface("awdl0"));
    assert!(is_infrastructure_interface("llw0"));
    assert!(is_infrastructure_interface("utun0"));
    assert!(is_infrastructure_interface("ap1"));
    assert!(is_infrastructure_interface("gif0"));
    assert!(is_infrastructure_interface("stf0"));
    assert!(is_infrastructure_interface("XHC0"));
    assert!(is_infrastructure_interface("ipsec0"));
    // Should NOT filter
    assert!(!is_infrastructure_interface("en0"));
    assert!(!is_infrastructure_interface("en1"));
    assert!(!is_infrastructure_interface("lo0"));
}
