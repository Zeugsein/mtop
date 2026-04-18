/// Feature-organized tests: network
/// Covers: speed tiers, baudrate formatting, interface filtering, network state rates,
/// baseline floor calculations.

// ===========================================================================
// Speed tier from baudrate (iter6, iter19, iter23)
// ===========================================================================

#[test]
/// speed_tier_from_baudrate maps 1 Gbps to 125 MB/s ceiling
fn speed_tier_1gbps_maps_to_125_mb_per_sec() {
    use mtop::platform::network::speed_tier_from_baudrate;
    assert_eq!(speed_tier_from_baudrate(1_000_000_000), 125_000_000);
}

#[test]
/// speed_tier_from_baudrate maps 100 Mbps to 12.5 MB/s ceiling
fn speed_tier_100mbps_maps_to_12_5_mb_per_sec() {
    use mtop::platform::network::speed_tier_from_baudrate;
    assert_eq!(speed_tier_from_baudrate(100_000_000), 12_500_000);
}

#[test]
/// speed_tier_from_baudrate maps 10 Mbps to 1.25 MB/s ceiling
fn speed_tier_10mbps_maps_to_1_25_mb_per_sec() {
    use mtop::platform::network::speed_tier_from_baudrate;
    assert_eq!(speed_tier_from_baudrate(10_000_000), 1_250_000);
}

#[test]
/// speed_tier_from_baudrate falls back to 1.25 MB/s when baudrate is 0 (unknown link)
fn speed_tier_zero_baudrate_uses_fallback() {
    use mtop::platform::network::speed_tier_from_baudrate;
    assert_eq!(speed_tier_from_baudrate(0), 1_250_000);
}

#[test]
/// speed_tier_from_baudrate(0) should return the 10 Mbps fallback: 1_250_000.
fn network_speed_tier_zero() {
    use mtop::platform::network::speed_tier_from_baudrate;
    assert_eq!(
        speed_tier_from_baudrate(0),
        1_250_000,
        "baudrate 0 should map to 10 Mbps fallback (1_250_000 bytes/sec)"
    );
}

#[test]
/// speed_tier_from_baudrate(1_000_000_000) should return 125_000_000 (1 Gbps tier).
fn network_speed_tier_gigabit() {
    use mtop::platform::network::speed_tier_from_baudrate;
    assert_eq!(
        speed_tier_from_baudrate(1_000_000_000),
        125_000_000,
        "1 Gbps baudrate should map to 125_000_000 bytes/sec tier"
    );
}

// ===========================================================================
// Baudrate formatting (iter8, iter9)
// ===========================================================================

#[test]
fn format_baudrate_gbps() {
    let result = mtop::tui::helpers::format_baudrate(1_000_000_000);
    assert_eq!(result, "1 Gbps");
}

#[test]
fn format_baudrate_mbps() {
    let result = mtop::tui::helpers::format_baudrate(100_000_000);
    assert_eq!(result, "100 Mbps");
}

#[test]
fn format_baudrate_zero() {
    let result = mtop::tui::helpers::format_baudrate(0);
    assert_eq!(result, "—");
}

#[test]
fn format_baudrate_round_gbps() {
    let result = mtop::tui::helpers::format_baudrate(1_000_000_000);
    assert_eq!(result, "1 Gbps");
}

#[test]
fn format_baudrate_fractional_gbps() {
    let result = mtop::tui::helpers::format_baudrate(2_500_000_000);
    assert_eq!(result, "2.5 Gbps");
}

#[test]
fn format_baudrate_round_mbps() {
    let result = mtop::tui::helpers::format_baudrate(100_000_000);
    assert_eq!(result, "100 Mbps");
}

#[test]
fn format_baudrate_fractional_mbps() {
    let result = mtop::tui::helpers::format_baudrate(54_500_000);
    assert_eq!(result, "54.5 Mbps");
}

#[test]
fn format_baudrate_zero_returns_dash() {
    let result = mtop::tui::helpers::format_baudrate(0);
    assert_eq!(result, "—");
}

#[test]
fn format_baudrate_kbps() {
    let result = mtop::tui::helpers::format_baudrate(56_000);
    assert_eq!(result, "56 Kbps");
}

// ===========================================================================
// Interface filtering (iter8)
// ===========================================================================

#[test]
fn infra_filter_bridge() {
    assert!(mtop::tui::helpers::is_infrastructure_interface("bridge0"));
}

#[test]
fn infra_filter_awdl() {
    assert!(mtop::tui::helpers::is_infrastructure_interface("awdl0"));
}

#[test]
fn infra_filter_llw() {
    assert!(mtop::tui::helpers::is_infrastructure_interface("llw0"));
}

#[test]
fn infra_filter_gif() {
    assert!(mtop::tui::helpers::is_infrastructure_interface("gif0"));
}

#[test]
fn infra_filter_stf() {
    assert!(mtop::tui::helpers::is_infrastructure_interface("stf0"));
}

#[test]
fn infra_filter_xhc() {
    assert!(mtop::tui::helpers::is_infrastructure_interface("XHC20"));
}

#[test]
fn infra_filter_ap() {
    assert!(mtop::tui::helpers::is_infrastructure_interface("ap1"));
}

#[test]
fn infra_filter_utun() {
    assert!(mtop::tui::helpers::is_infrastructure_interface("utun3"));
}

#[test]
fn infra_filter_en0_is_not_infrastructure() {
    assert!(!mtop::tui::helpers::is_infrastructure_interface("en0"));
}

#[test]
fn infra_filter_en1_is_not_infrastructure() {
    assert!(!mtop::tui::helpers::is_infrastructure_interface("en1"));
}

#[test]
fn infra_filter_lo0_is_not_infrastructure() {
    assert!(!mtop::tui::helpers::is_infrastructure_interface("lo0"));
}

// ===========================================================================
// NetInterface struct fields (iter8)
// ===========================================================================

use mtop::metrics::NetInterface;

#[test]
fn net_interface_type_en() {
    let iface = NetInterface {
        name: "en0".into(),
        iface_type: "Ethernet/Wi-Fi".into(),
        ..Default::default()
    };
    assert_eq!(iface.iface_type, "Ethernet/Wi-Fi");
}

#[test]
fn net_interface_has_baudrate_and_packets() {
    let iface = NetInterface {
        name: "en0".into(),
        iface_type: "Ethernet/Wi-Fi".into(),
        rx_bytes_sec: 1000.0,
        tx_bytes_sec: 500.0,
        baudrate: 1_000_000_000,
        packets_in_sec: 100.0,
        packets_out_sec: 50.0,
        rx_bytes_total: 0,
        tx_bytes_total: 0,
    };
    assert_eq!(iface.baudrate, 1_000_000_000);
    assert_eq!(iface.packets_in_sec, 100.0);
    assert_eq!(iface.packets_out_sec, 50.0);
}

#[test]
fn net_interface_default_packets_zero() {
    let iface = NetInterface::default();
    assert_eq!(iface.packets_in_sec, 0.0);
    assert_eq!(iface.packets_out_sec, 0.0);
    assert_eq!(iface.baudrate, 0);
}

// ===========================================================================
// NetworkState collection rates (iter19)
// ===========================================================================

#[test]
fn network_rate_first_sample_zero() {
    use mtop::platform::network::NetworkState;
    let mut state = NetworkState::new();
    let metrics = state.collect();

    for iface in &metrics.interfaces {
        assert!(
            iface.rx_bytes_sec >= 0.0,
            "rx_bytes_sec should be >= 0.0 on first sample, got {} for {}",
            iface.rx_bytes_sec,
            iface.name
        );
        assert!(
            iface.tx_bytes_sec >= 0.0,
            "tx_bytes_sec should be >= 0.0 on first sample, got {} for {}",
            iface.tx_bytes_sec,
            iface.name
        );
    }
}

#[test]
fn network_rate_second_sample_nonnegative() {
    use mtop::platform::network::NetworkState;
    let mut state = NetworkState::new();
    let _ = state.collect();
    std::thread::sleep(std::time::Duration::from_millis(20));
    let metrics = state.collect();

    for iface in &metrics.interfaces {
        assert!(
            iface.rx_bytes_sec >= 0.0,
            "rx_bytes_sec must be >= 0 after second sample, got {} for {}",
            iface.rx_bytes_sec,
            iface.name
        );
        assert!(
            iface.tx_bytes_sec >= 0.0,
            "tx_bytes_sec must be >= 0 after second sample, got {} for {}",
            iface.tx_bytes_sec,
            iface.name
        );
    }
}

// ===========================================================================
// SHALL-23-08: Network border color derives from net_download (iter23)
// ===========================================================================

#[test]
fn shall_23_08_network_border_color_derives_from_net_download() {
    use mtop::tui::theme::{self, THEMES};

    for t in THEMES.iter() {
        let from_download = theme::dim_color(t.net_download, theme::adaptive_border_dim(t));
        let from_upload = theme::dim_color(t.net_upload, theme::adaptive_border_dim(t));
        assert_ne!(
            format!("{:?}", from_download),
            format!("{:?}", from_upload),
            "theme '{}': dim_color(net_download) must differ from dim_color(net_upload) to prove correct channel is used",
            t.name
        );
    }
}

#[test]
fn shall_23_08_dim_color_produces_darker_result_than_source() {
    use mtop::tui::theme::{self, THEMES};
    use ratatui::style::Color;

    let t = THEMES.iter().find(|t| t.name == "horizon").unwrap();
    let border = theme::dim_color(t.net_download, theme::adaptive_border_dim(t));

    if let (Color::Rgb(sr, sg, sb), Color::Rgb(br, bg, bb)) = (t.net_download, border) {
        assert!(
            br <= sr,
            "border R ({br}) should be <= source R ({sr}) after dimming"
        );
        assert!(
            bg <= sg,
            "border G ({bg}) should be <= source G ({sg}) after dimming"
        );
        assert!(
            bb <= sb,
            "border B ({bb}) should be <= source B ({sb}) after dimming"
        );
    } else {
        panic!("expected Rgb colors");
    }
}

#[test]
fn shall_23_08_net_download_is_not_net_upload_for_all_themes() {
    use mtop::tui::theme::THEMES;

    for t in THEMES.iter() {
        assert_ne!(
            format!("{:?}", t.net_download),
            format!("{:?}", t.net_upload),
            "theme '{}': net_download must differ from net_upload (border uses net_download)",
            t.name
        );
    }
}

// ===========================================================================
// SHALL-23-10: Network baseline floor = scale * 0.005 (iter23)
// ===========================================================================

#[test]
fn shall_23_10_baseline_floor_for_1gbps_link() {
    use mtop::platform::network::speed_tier_from_baudrate;

    let baudrate = 1_000_000_000u64;
    let scale = speed_tier_from_baudrate(baudrate) as f64;
    let baseline_floor = scale * 0.005;

    assert_eq!(scale, 125_000_000.0, "1 Gbps should map to 125 MB/s scale");
    assert!(
        (baseline_floor - 625_000.0).abs() < 1.0,
        "baseline_floor for 1 Gbps should be ~625000 bytes/sec, got {baseline_floor}"
    );
}

#[test]
fn shall_23_10_baseline_floor_for_100mbps_link() {
    use mtop::platform::network::speed_tier_from_baudrate;

    let baudrate = 100_000_000u64;
    let scale = speed_tier_from_baudrate(baudrate) as f64;
    let baseline_floor = scale * 0.005;

    assert_eq!(
        scale, 12_500_000.0,
        "100 Mbps should map to 12.5 MB/s scale"
    );
    assert!(
        (baseline_floor - 62_500.0).abs() < 1.0,
        "baseline_floor for 100 Mbps should be ~62500 bytes/sec, got {baseline_floor}"
    );
}

#[test]
fn shall_23_10_baseline_floor_for_10mbps_fallback() {
    use mtop::platform::network::speed_tier_from_baudrate;

    let baudrate = 10_000_000u64;
    let scale = speed_tier_from_baudrate(baudrate) as f64;
    let baseline_floor = scale * 0.005;

    assert_eq!(
        scale, 1_250_000.0,
        "10 Mbps should use the 1.25 MB/s fallback scale"
    );
    assert!(
        (baseline_floor - 6_250.0).abs() < 1.0,
        "baseline_floor for 10 Mbps fallback should be ~6250 bytes/sec, got {baseline_floor}"
    );
}

#[test]
fn shall_23_10_baseline_floor_is_positive_for_all_tiers() {
    use mtop::platform::network::speed_tier_from_baudrate;

    let test_baudrates = [
        0u64,
        1_000_000,
        10_000_000,
        100_000_000,
        1_000_000_000,
        10_000_000_000,
    ];
    for baud in test_baudrates {
        let scale = speed_tier_from_baudrate(baud) as f64;
        let baseline_floor = scale * 0.005;
        assert!(
            baseline_floor > 0.0,
            "baseline_floor must be positive for baudrate {baud}, got {baseline_floor}"
        );
    }
}

#[test]
fn shall_23_10_baseline_floor_is_small_fraction_of_scale() {
    use mtop::platform::network::speed_tier_from_baudrate;

    let test_baudrates = [0u64, 10_000_000, 100_000_000, 1_000_000_000];
    for baud in test_baudrates {
        let scale = speed_tier_from_baudrate(baud) as f64;
        let baseline_floor = scale * 0.005;
        assert!(
            baseline_floor < scale,
            "baseline_floor ({baseline_floor}) must be less than full scale ({scale})"
        );
        assert!(
            baseline_floor <= scale * 0.01,
            "baseline_floor should be <= 1% of scale; floor={baseline_floor} scale={scale}"
        );
    }
}

// ===========================================================================
// Network labels (iter17)
// ===========================================================================

#[test]
fn network_label_no_cur_prefix() {
    let net_src = include_str!("../src/tui/panels/network.rs");
    assert!(
        !net_src.contains("cur:"),
        "network.rs should not contain old `cur:` label prefix"
    );
}

#[test]
fn network_label_uses_total() {
    let net_src = include_str!("../src/tui/panels/network.rs");
    assert!(
        net_src.contains("total"),
        "network.rs should use `total` label (not `tot:`)"
    );
    assert!(
        !net_src.contains("tot:"),
        "network.rs should not contain old `tot:` label"
    );
}

#[test]
fn idle_threshold_network() {
    let net_src = include_str!("../src/tui/panels/network.rs");
    assert!(
        net_src.contains("< 1024.0"),
        "expected network idle threshold `< 1024.0` in network.rs"
    );
}
