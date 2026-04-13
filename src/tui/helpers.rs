//! Helper utility functions extracted from mod.rs (iteration 8).

use unicode_width::UnicodeWidthChar;

/// Truncate a string by display width (CJK-aware).
/// CJK characters count as 2 columns, ASCII as 1.
pub fn truncate_by_display_width(s: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    let mut width = 0;
    let mut result = String::new();
    for ch in s.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + ch_width > max_width {
            if width < max_width {
                result.push('\u{2026}'); // ellipsis
            }
            return result;
        }
        width += ch_width;
        result.push(ch);
    }
    result
}

/// Pad a string to a target display width with trailing spaces (CJK-aware).
pub fn pad_to_display_width(s: &str, target_width: usize) -> String {
    let current: usize = s.chars().map(|c| UnicodeWidthChar::width(c).unwrap_or(0)).sum();
    if current >= target_width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(target_width - current))
    }
}

pub fn format_bytes_rate_compact(b: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    if b >= GB {
        format!("{:.1}G/s", b / GB)
    } else if b >= MB {
        format!("{:.1}M/s", b / MB)
    } else if b >= KB {
        format!("{:.1}K/s", b / KB)
    } else {
        format!("{:.0}B/s", b)
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

pub fn format_bytes_compact(b: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    const TB: f64 = 1024.0 * 1024.0 * 1024.0 * 1024.0;
    if b >= TB {
        format!("{:.2} TB", b / TB)
    } else if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{:.0} B", b)
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
    const INFRA_PREFIXES: &[&str] = &["bridge", "awdl", "llw", "gif", "stf", "XHC", "ap", "utun", "ipsec"];
    INFRA_PREFIXES.iter().any(|prefix| name.starts_with(prefix))
}

pub fn format_baudrate(baudrate: u64) -> String {
    if baudrate >= 1_000_000_000 {
        let gbps = baudrate as f64 / 1_000_000_000.0;
        if (gbps - gbps.round()).abs() < 0.01 {
            format!("{} Gbps", gbps as u64)
        } else {
            format!("{:.1} Gbps", gbps)
        }
    } else if baudrate >= 1_000_000 {
        let mbps = baudrate as f64 / 1_000_000.0;
        if (mbps - mbps.round()).abs() < 0.01 {
            format!("{} Mbps", mbps as u64)
        } else {
            format!("{:.1} Mbps", mbps)
        }
    } else if baudrate > 0 {
        format!("{} Kbps", baudrate / 1_000)
    } else {
        "—".to_string()
    }
}

/// Color for temperature based on thresholds.
pub fn temp_color(temp_c: f32, warn_threshold: f32, crit_threshold: f32) -> ratatui::style::Color {
    if temp_c >= crit_threshold {
        ratatui::style::Color::Red
    } else if temp_c >= warn_threshold {
        ratatui::style::Color::Yellow
    } else {
        ratatui::style::Color::Green
    }
}

// Thermal thresholds (compile-time constants)
pub const CPU_TEMP_WARN: f32 = 80.0;
pub const CPU_TEMP_CRIT: f32 = 95.0;
pub const GPU_TEMP_WARN: f32 = 85.0;
pub const GPU_TEMP_CRIT: f32 = 100.0;

pub fn sort_indices(indices: &mut [usize], procs: &[crate::metrics::ProcessInfo], mode: crate::metrics::SortMode, max_cpu: f32, max_mem: u64, max_power: f32) {
    use crate::metrics::SortMode;
    use crate::platform::process::weighted_score;
    match mode {
        SortMode::WeightedScore => {
            indices.sort_by(|&a, &b| {
                let sa = weighted_score(&procs[a], max_cpu, max_mem, max_power);
                let sb = weighted_score(&procs[b], max_cpu, max_mem, max_power);
                sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        SortMode::Cpu => {
            indices.sort_by(|&a, &b| procs[b].cpu_pct.partial_cmp(&procs[a].cpu_pct).unwrap_or(std::cmp::Ordering::Equal));
        }
        SortMode::Memory => {
            indices.sort_by(|&a, &b| procs[b].mem_bytes.cmp(&procs[a].mem_bytes));
        }
        SortMode::Power => {
            indices.sort_by(|&a, &b| procs[b].power_w.partial_cmp(&procs[a].power_w).unwrap_or(std::cmp::Ordering::Equal));
        }
        SortMode::Pid => {
            indices.sort_by(|&a, &b| procs[a].pid.cmp(&procs[b].pid));
        }
        SortMode::Name => {
            indices.sort_by(|&a, &b| procs[a].name.to_lowercase().cmp(&procs[b].name.to_lowercase()));
        }
    }
}
