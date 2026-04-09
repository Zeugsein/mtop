//! Helper utility functions extracted from mod.rs (iteration 8).

pub fn format_bytes_rate_compact(b: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    if b >= GB {
        format!("{:.1}G", b / GB)
    } else if b >= MB {
        format!("{:.1}M", b / MB)
    } else if b >= KB {
        format!("{:.1}K", b / KB)
    } else {
        format!("{:.0}B", b)
    }
}

pub fn format_bytes_rate(b: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    if b >= GB {
        format!("{:.1} GB/s", b / GB)
    } else if b >= MB {
        format!("{:.1} MB/s", b / MB)
    } else if b >= KB {
        format!("{:.1} KB/s", b / KB)
    } else {
        format!("{:.0} B/s", b)
    }
}

pub fn truncate_with_ellipsis(name: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    let chars: Vec<char> = name.chars().collect();
    if chars.len() <= max_width {
        name.to_string()
    } else if max_width == 1 {
        "\u{2026}".to_string()
    } else {
        let truncated: String = chars[..max_width - 1].iter().collect();
        format!("{}\u{2026}", truncated)
    }
}

pub fn is_infrastructure_interface(name: &str) -> bool {
    const INFRA_PREFIXES: &[&str] = &["bridge", "awdl", "llw", "gif", "stf", "XHC", "ap", "utun"];
    INFRA_PREFIXES.iter().any(|prefix| name.starts_with(prefix))
}

pub fn format_baudrate(baudrate: u64) -> String {
    if baudrate >= 1_000_000_000 {
        format!("{} Gbps", baudrate / 1_000_000_000)
    } else if baudrate >= 1_000_000 {
        format!("{} Mbps", baudrate / 1_000_000)
    } else if baudrate > 0 {
        format!("{} Kbps", baudrate / 1_000)
    } else {
        "—".to_string()
    }
}
