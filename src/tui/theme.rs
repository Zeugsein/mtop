use ratatui::style::Color;
use std::sync::LazyLock;

/// A complete color theme for the mtop TUI.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Theme display name
    pub name: &'static str,
    /// Main background color
    pub bg: Color,
    /// Primary text color
    pub fg: Color,
    /// Accent color for borders, highlights
    pub accent: Color,
    /// Muted text (footer, secondary info)
    pub muted: Color,
    /// Border color for panel frames
    pub border: Color,
    /// Header bar background
    pub header_bg: Color,
    /// Header bar foreground
    pub header_fg: Color,

    // Per-dimension accent colors
    /// CPU-related elements
    pub cpu_accent: Color,
    /// GPU-related elements
    pub gpu_accent: Color,
    /// Memory-related elements
    pub mem_accent: Color,
    /// Network upload color
    pub net_upload: Color,
    /// Network download color
    pub net_download: Color,
    /// Power-related elements
    pub power_accent: Color,
    /// Process panel accent (btop proc_box)
    pub process_accent: Color,

    // Memory pressure indicator colors
    pub pressure_normal: Color,
    pub pressure_warn: Color,
    pub pressure_critical: Color,

    // Gradient stops (green → yellow → orange → red)
    pub gradient_green: Color,
    pub gradient_yellow: Color,
    pub gradient_orange: Color,
    pub gradient_red: Color,
}

/// Horizon theme — warm, dark background with vibrant accents.
/// Corrected palette: CPU=purple, Mem=green, Net=red (from btop Horizon).
pub const HORIZON: Theme = Theme {
    name: "horizon",
    bg: Color::Rgb(28, 30, 38),
    fg: Color::Rgb(205, 209, 219),
    accent: Color::Rgb(233, 175, 100),
    muted: Color::Rgb(107, 112, 127),
    border: Color::Rgb(60, 63, 75),
    header_bg: Color::Rgb(233, 175, 100),
    header_fg: Color::Rgb(28, 30, 38),

    cpu_accent: Color::Rgb(184, 119, 219), // purple (#B877DB, btop cpu_box)
    gpu_accent: Color::Rgb(0, 0, 0),       // derived at runtime
    mem_accent: Color::Rgb(39, 215, 150),  // teal (#27D796, btop mem_box)
    net_upload: Color::Rgb(38, 187, 194),  // cyan
    net_download: Color::Rgb(233, 86, 120), // red (#E95678, btop net_box)
    power_accent: Color::Rgb(0, 0, 0),     // derived at runtime
    process_accent: Color::Rgb(37, 178, 188), // cyan (#25B2BC, btop proc_box)
    pressure_normal: Color::Rgb(0, 200, 83),
    pressure_warn: Color::Rgb(255, 214, 0),
    pressure_critical: Color::Rgb(255, 61, 0),

    gradient_green: Color::Rgb(39, 215, 150),
    gradient_yellow: Color::Rgb(208, 198, 151),
    gradient_orange: Color::Rgb(243, 143, 134),
    gradient_red: Color::Rgb(233, 86, 120),
};

/// Dracula theme
pub const DRACULA: Theme = Theme {
    name: "dracula",
    bg: Color::Rgb(40, 42, 54),
    fg: Color::Rgb(248, 248, 242),
    accent: Color::Rgb(189, 147, 249),
    muted: Color::Rgb(98, 114, 164),
    border: Color::Rgb(68, 71, 90),
    header_bg: Color::Rgb(189, 147, 249),
    header_fg: Color::Rgb(40, 42, 54),
    cpu_accent: Color::Rgb(189, 147, 249), // purple (#BD93F9, btop cpu_box)
    gpu_accent: Color::Rgb(0, 0, 0),       // derived at runtime
    mem_accent: Color::Rgb(80, 250, 123),  // green (#50FA7B, btop mem_box)
    net_upload: Color::Rgb(80, 250, 123),  // green
    net_download: Color::Rgb(255, 85, 85), // red (#FF5555, btop net_box)
    power_accent: Color::Rgb(0, 0, 0),     // derived at runtime
    process_accent: Color::Rgb(139, 233, 253), // cyan (#8BE9FD, btop proc_box)
    pressure_normal: Color::Rgb(0, 200, 83),
    pressure_warn: Color::Rgb(255, 214, 0),
    pressure_critical: Color::Rgb(255, 61, 0),
    gradient_green: Color::Rgb(189, 147, 249),
    gradient_yellow: Color::Rgb(242, 126, 208),
    gradient_orange: Color::Rgb(255, 93, 186),
    gradient_red: Color::Rgb(255, 51, 168),
};

/// Catppuccin Mocha theme
pub const CATPPUCCIN: Theme = Theme {
    name: "catppuccin",
    bg: Color::Rgb(30, 30, 46),
    fg: Color::Rgb(205, 214, 244),
    accent: Color::Rgb(203, 166, 247),
    muted: Color::Rgb(108, 112, 134),
    border: Color::Rgb(69, 71, 90),
    header_bg: Color::Rgb(203, 166, 247),
    header_fg: Color::Rgb(30, 30, 46),
    cpu_accent: Color::Rgb(137, 220, 235),
    gpu_accent: Color::Rgb(249, 226, 175),
    mem_accent: Color::Rgb(203, 166, 247),
    net_upload: Color::Rgb(166, 227, 161),
    net_download: Color::Rgb(245, 194, 231),
    power_accent: Color::Rgb(250, 179, 135),
    process_accent: Color::Rgb(137, 180, 250), // blue (#89B4FA, catppuccin palette — matches btop convention)
    pressure_normal: Color::Rgb(0, 200, 83),
    pressure_warn: Color::Rgb(255, 214, 0),
    pressure_critical: Color::Rgb(255, 61, 0),
    gradient_green: Color::Rgb(166, 227, 161),
    gradient_yellow: Color::Rgb(232, 227, 172),
    gradient_orange: Color::Rgb(247, 186, 172),
    gradient_red: Color::Rgb(243, 139, 168),
};

/// Nord theme
pub const NORD: Theme = Theme {
    name: "nord",
    bg: Color::Rgb(46, 52, 64),
    fg: Color::Rgb(216, 222, 233),
    accent: Color::Rgb(136, 192, 208),
    muted: Color::Rgb(76, 86, 106),
    border: Color::Rgb(59, 66, 82),
    header_bg: Color::Rgb(136, 192, 208),
    header_fg: Color::Rgb(46, 52, 64),
    cpu_accent: Color::Rgb(136, 192, 208),
    gpu_accent: Color::Rgb(235, 203, 139),
    mem_accent: Color::Rgb(180, 142, 173),
    net_upload: Color::Rgb(163, 190, 140),
    net_download: Color::Rgb(191, 97, 106),
    power_accent: Color::Rgb(208, 135, 112),
    process_accent: Color::Rgb(143, 188, 187), // frost (#8FBCBB, nord palette)
    pressure_normal: Color::Rgb(0, 200, 83),
    pressure_warn: Color::Rgb(255, 214, 0),
    pressure_critical: Color::Rgb(255, 61, 0),
    gradient_green: Color::Rgb(163, 190, 140),
    gradient_yellow: Color::Rgb(221, 200, 139),
    gradient_orange: Color::Rgb(209, 155, 120),
    gradient_red: Color::Rgb(191, 97, 106),
};

/// Solarized Dark theme
pub const SOLARIZED_DARK: Theme = Theme {
    name: "solarized-dark",
    bg: Color::Rgb(0, 43, 54),
    fg: Color::Rgb(131, 148, 150),
    accent: Color::Rgb(38, 139, 210),
    muted: Color::Rgb(88, 110, 117),
    border: Color::Rgb(7, 54, 66),
    header_bg: Color::Rgb(38, 139, 210),
    header_fg: Color::Rgb(253, 246, 227),
    cpu_accent: Color::Rgb(42, 161, 152),
    gpu_accent: Color::Rgb(181, 137, 0),
    mem_accent: Color::Rgb(108, 113, 196),
    net_upload: Color::Rgb(133, 153, 0),
    net_download: Color::Rgb(211, 54, 130),
    power_accent: Color::Rgb(203, 75, 22),
    process_accent: Color::Rgb(38, 139, 210), // blue (#268BD2, solarized palette — distinct from cpu cyan)
    pressure_normal: Color::Rgb(0, 200, 83),
    pressure_warn: Color::Rgb(255, 214, 0),
    pressure_critical: Color::Rgb(255, 61, 0),
    gradient_green: Color::Rgb(173, 199, 0),
    gradient_yellow: Color::Rgb(206, 169, 0),
    gradient_orange: Color::Rgb(220, 130, 14),
    gradient_red: Color::Rgb(230, 83, 23),
};

/// Solarized Light theme
pub const SOLARIZED_LIGHT: Theme = Theme {
    name: "solarized-light",
    bg: Color::Rgb(253, 246, 227),
    fg: Color::Rgb(101, 123, 131),
    accent: Color::Rgb(38, 139, 210),
    muted: Color::Rgb(147, 161, 161),
    border: Color::Rgb(238, 232, 213),
    header_bg: Color::Rgb(38, 139, 210),
    header_fg: Color::Rgb(253, 246, 227),
    cpu_accent: Color::Rgb(42, 161, 152),
    gpu_accent: Color::Rgb(181, 137, 0),
    mem_accent: Color::Rgb(108, 113, 196),
    net_upload: Color::Rgb(133, 153, 0),
    net_download: Color::Rgb(211, 54, 130),
    power_accent: Color::Rgb(203, 75, 22),
    process_accent: Color::Rgb(38, 139, 210), // blue (#268BD2, solarized palette — distinct from cpu cyan)
    pressure_normal: Color::Rgb(0, 200, 83),
    pressure_warn: Color::Rgb(255, 214, 0),
    pressure_critical: Color::Rgb(255, 61, 0),
    gradient_green: Color::Rgb(173, 199, 0),
    gradient_yellow: Color::Rgb(206, 169, 0),
    gradient_orange: Color::Rgb(220, 130, 14),
    gradient_red: Color::Rgb(230, 83, 23),
};

/// Gruvbox Dark theme
pub const GRUVBOX: Theme = Theme {
    name: "gruvbox",
    bg: Color::Rgb(40, 40, 40),
    fg: Color::Rgb(235, 219, 178),
    accent: Color::Rgb(250, 189, 47),
    muted: Color::Rgb(146, 131, 116),
    border: Color::Rgb(80, 73, 69),
    header_bg: Color::Rgb(250, 189, 47),
    header_fg: Color::Rgb(40, 40, 40),
    cpu_accent: Color::Rgb(131, 165, 152),
    gpu_accent: Color::Rgb(250, 189, 47),
    mem_accent: Color::Rgb(211, 134, 155),
    net_upload: Color::Rgb(184, 187, 38),
    net_download: Color::Rgb(251, 73, 52),
    power_accent: Color::Rgb(254, 128, 25),
    process_accent: Color::Rgb(104, 157, 106), // aqua (#689D6A, gruvbox palette)
    pressure_normal: Color::Rgb(0, 200, 83),
    pressure_warn: Color::Rgb(255, 214, 0),
    pressure_critical: Color::Rgb(255, 61, 0),
    gradient_green: Color::Rgb(184, 187, 38),
    gradient_yellow: Color::Rgb(209, 161, 34),
    gradient_orange: Color::Rgb(237, 121, 45),
    gradient_red: Color::Rgb(251, 73, 52),
};

/// Tokyo Night theme
pub const TOKYO_NIGHT: Theme = Theme {
    name: "tokyo-night",
    bg: Color::Rgb(26, 27, 38),
    fg: Color::Rgb(169, 177, 214),
    accent: Color::Rgb(122, 162, 247),
    muted: Color::Rgb(86, 95, 137),
    border: Color::Rgb(41, 46, 66),
    header_bg: Color::Rgb(122, 162, 247),
    header_fg: Color::Rgb(26, 27, 38),
    cpu_accent: Color::Rgb(125, 207, 255),
    gpu_accent: Color::Rgb(224, 175, 104),
    mem_accent: Color::Rgb(187, 154, 247),
    net_upload: Color::Rgb(158, 206, 106),
    net_download: Color::Rgb(247, 118, 142),
    power_accent: Color::Rgb(255, 158, 100),
    process_accent: Color::Rgb(115, 218, 202), // teal (#73DACA, tokyo-night palette)
    pressure_normal: Color::Rgb(0, 200, 83),
    pressure_warn: Color::Rgb(255, 214, 0),
    pressure_critical: Color::Rgb(255, 61, 0),
    gradient_green: Color::Rgb(158, 206, 106),
    gradient_yellow: Color::Rgb(211, 188, 105),
    gradient_orange: Color::Rgb(238, 150, 129),
    gradient_red: Color::Rgb(247, 118, 142),
};

/// One Dark theme
pub const ONE_DARK: Theme = Theme {
    name: "one-dark",
    bg: Color::Rgb(40, 44, 52),
    fg: Color::Rgb(171, 178, 191),
    accent: Color::Rgb(97, 175, 239),
    muted: Color::Rgb(92, 99, 112),
    border: Color::Rgb(62, 68, 81),
    header_bg: Color::Rgb(97, 175, 239),
    header_fg: Color::Rgb(40, 44, 52),
    cpu_accent: Color::Rgb(86, 182, 194),
    gpu_accent: Color::Rgb(229, 192, 123),
    mem_accent: Color::Rgb(198, 120, 221),
    net_upload: Color::Rgb(152, 195, 121),
    net_download: Color::Rgb(224, 108, 117),
    power_accent: Color::Rgb(209, 154, 102),
    process_accent: Color::Rgb(97, 175, 239), // blue (#61AFEF, onedark palette)
    pressure_normal: Color::Rgb(0, 200, 83),
    pressure_warn: Color::Rgb(255, 214, 0),
    pressure_critical: Color::Rgb(255, 61, 0),
    gradient_green: Color::Rgb(152, 195, 121),
    gradient_yellow: Color::Rgb(213, 193, 123),
    gradient_orange: Color::Rgb(227, 154, 119),
    gradient_red: Color::Rgb(224, 108, 117),
};

/// Monokai theme
pub const MONOKAI: Theme = Theme {
    name: "monokai",
    bg: Color::Rgb(39, 40, 34),
    fg: Color::Rgb(248, 248, 242),
    accent: Color::Rgb(166, 226, 46),
    muted: Color::Rgb(117, 113, 94),
    border: Color::Rgb(73, 72, 62),
    header_bg: Color::Rgb(166, 226, 46),
    header_fg: Color::Rgb(39, 40, 34),
    cpu_accent: Color::Rgb(102, 217, 239),
    gpu_accent: Color::Rgb(230, 219, 116),
    mem_accent: Color::Rgb(174, 129, 255),
    net_upload: Color::Rgb(166, 226, 46),
    net_download: Color::Rgb(249, 38, 114),
    power_accent: Color::Rgb(253, 151, 31),
    process_accent: Color::Rgb(174, 129, 255), // purple (#AE81FF, monokai palette — distinct from cpu cyan)
    pressure_normal: Color::Rgb(0, 200, 83),
    pressure_warn: Color::Rgb(255, 214, 0),
    pressure_critical: Color::Rgb(255, 61, 0),
    gradient_green: Color::Rgb(166, 226, 46),
    gradient_yellow: Color::Rgb(236, 181, 34),
    gradient_orange: Color::Rgb(251, 102, 75),
    gradient_red: Color::Rgb(249, 38, 114),
};

/// All available themes, indexed for cycling.
/// GPU and Power accents are derived from CPU and MEM accents via derive_companion.
pub static THEMES: LazyLock<Vec<Theme>> = LazyLock::new(|| {
    let bases = [
        HORIZON,
        DRACULA,
        CATPPUCCIN,
        NORD,
        SOLARIZED_DARK,
        SOLARIZED_LIGHT,
        GRUVBOX,
        TOKYO_NIGHT,
        ONE_DARK,
        MONOKAI,
    ];
    bases
        .into_iter()
        .map(|mut t| {
            t.gpu_accent = hue_midpoint(t.cpu_accent, t.mem_accent);
            t.power_accent = hue_midpoint(t.net_download, t.process_accent);
            t
        })
        .collect()
});

/// Return the list of available theme names.
pub fn theme_names() -> Vec<&'static str> {
    THEMES.iter().map(|t| t.name).collect()
}

/// Look up a theme by name, falling back to the first theme (Horizon).
pub fn theme_by_name(name: &str) -> &'static Theme {
    THEMES.iter().find(|t| t.name == name).unwrap_or(&THEMES[0])
}

/// Get the default theme.
pub fn default_theme() -> &'static Theme {
    &THEMES[0]
}

/// Dim an RGB color by a factor (0.0 = black, 1.0 = unchanged).
/// Used to create subtle tinted border colors from panel accent colors.
pub fn dim_color(color: Color, factor: f64) -> Color {
    match color {
        Color::Rgb(r, g, b) => {
            let f = factor.clamp(0.0, 1.0);
            Color::Rgb(
                (r as f64 * f).round() as u8,
                (g as f64 * f).round() as u8,
                (b as f64 * f).round() as u8,
            )
        }
        other => other,
    }
}

/// Compute the HSL hue midpoint between two colors (shorter arc on the color wheel).
/// Used to derive GPU accent (midpoint of CPU and MEM) and Power accent (midpoint of NET and Process).
pub fn hue_midpoint(a: Color, b: Color) -> Color {
    let (ar, ag, ab) = match a {
        Color::Rgb(r, g, b) => (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0),
        _ => return a,
    };
    let (br, bg, bb) = match b {
        Color::Rgb(r, g, b) => (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0),
        _ => return a,
    };

    let to_hsl = |r: f32, g: f32, b: f32| -> (f32, f32, f32) {
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;
        let delta = max - min;
        if delta < 1e-6 {
            return (0.0, 0.0, l);
        }
        let s = if l <= 0.5 {
            delta / (max + min)
        } else {
            delta / (2.0 - max - min)
        };
        let h = if (max - r).abs() < 1e-6 {
            let mut h = (g - b) / delta;
            if h < 0.0 {
                h += 6.0;
            }
            h * 60.0
        } else if (max - g).abs() < 1e-6 {
            ((b - r) / delta + 2.0) * 60.0
        } else {
            ((r - g) / delta + 4.0) * 60.0
        };
        (h, s, l)
    };

    let (h1, s1, l1) = to_hsl(ar, ag, ab);
    let (h2, s2, l2) = to_hsl(br, bg, bb);

    // Average hue via shorter arc
    let mut diff = h2 - h1;
    if diff > 180.0 {
        diff -= 360.0;
    }
    if diff < -180.0 {
        diff += 360.0;
    }
    let mid_h = (h1 + diff / 2.0).rem_euclid(360.0);
    let mid_s = (s1 + s2) / 2.0;
    let mid_l = (l1 + l2) / 2.0;

    hsl_to_rgb(mid_h, mid_s, mid_l)
}

/// Derive a companion color via HSL hue rotation and saturation adjustment.
/// Used to compute GPU accent from CPU accent, and Power accent from MEM accent.
/// `hue_shift_deg`: degrees to rotate hue (positive = clockwise on color wheel)
/// `sat_factor`: multiply saturation by this (0.9 = slightly desaturated)
pub fn derive_companion(base: Color, hue_shift_deg: f32, sat_factor: f32) -> Color {
    let (r, g, b) = match base {
        Color::Rgb(r, g, b) => (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0),
        _ => return base,
    };

    // RGB to HSL
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let delta = max - min;

    if delta < 1e-6 {
        // Achromatic — hue rotation has no effect, only sat_factor dims
        let dimmed = (l * sat_factor).clamp(0.0, 1.0);
        let v = (dimmed * 255.0).round() as u8;
        return Color::Rgb(v, v, v);
    }

    let s = if l <= 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    };

    let h = if (max - r).abs() < 1e-6 {
        let mut h = (g - b) / delta;
        if h < 0.0 {
            h += 6.0;
        }
        h * 60.0
    } else if (max - g).abs() < 1e-6 {
        ((b - r) / delta + 2.0) * 60.0
    } else {
        ((r - g) / delta + 4.0) * 60.0
    };

    // Apply shift and saturation
    let new_h = (h + hue_shift_deg).rem_euclid(360.0);
    let new_s = (s * sat_factor).clamp(0.0, 1.0);

    // HSL to RGB
    hsl_to_rgb(new_h, new_s, l)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color {
    if s < 1e-6 {
        let v = (l * 255.0).round() as u8;
        return Color::Rgb(v, v, v);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let h_norm = h / 360.0;

    let hue_to_rgb = |t: f32| -> f32 {
        let t = t.rem_euclid(1.0);
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };

    let r = (hue_to_rgb(h_norm + 1.0 / 3.0) * 255.0).round() as u8;
    let g = (hue_to_rgb(h_norm) * 255.0).round() as u8;
    let b = (hue_to_rgb(h_norm - 1.0 / 3.0) * 255.0).round() as u8;
    Color::Rgb(r, g, b)
}

/// Compute relative luminance of a theme's background (WCAG formula).
/// Returns 0.0 for pure black, 1.0 for pure white.
pub fn bg_luminance(theme: &Theme) -> f64 {
    match theme.bg {
        Color::Rgb(r, g, b) => {
            let to_linear = |c: u8| -> f64 {
                let s = c as f64 / 255.0;
                if s <= 0.03928 {
                    s / 12.92
                } else {
                    ((s + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * to_linear(r) + 0.7152 * to_linear(g) + 0.0722 * to_linear(b)
        }
        _ => 0.0,
    }
}

/// Adaptive border dim factor based on theme brightness.
/// Dark themes (luminance < 0.5) get 0.55 (brighter borders).
/// Light themes (luminance >= 0.5) get 0.35 (darker borders for contrast).
pub fn adaptive_border_dim(theme: &Theme) -> f64 {
    if bg_luminance(theme) >= 0.5 {
        0.35
    } else {
        0.55
    }
}

/// Baseline color for sparse braille dots (network, GPU, power at near-zero).
/// Guarantees visible separation from theme.bg by applying a relative brightness
/// boost (+30 on dark themes, -30 on light themes) to theme.muted.
const BASELINE_DELTA: u8 = 30;

pub fn baseline_color(theme: &Theme) -> Color {
    match theme.muted {
        Color::Rgb(r, g, b) => {
            if bg_luminance(theme) >= 0.5 {
                // Light theme: darken muted
                Color::Rgb(
                    r.saturating_sub(BASELINE_DELTA),
                    g.saturating_sub(BASELINE_DELTA),
                    b.saturating_sub(BASELINE_DELTA),
                )
            } else {
                // Dark theme: brighten muted
                Color::Rgb(
                    r.saturating_add(BASELINE_DELTA),
                    g.saturating_add(BASELINE_DELTA),
                    b.saturating_add(BASELINE_DELTA),
                )
            }
        }
        other => other,
    }
}

/// Panel superscript number characters, indexed 1-6.
pub const PANEL_SUPERSCRIPTS: [char; 6] = [
    '\u{00B9}', '\u{00B2}', '\u{00B3}', '\u{2074}', '\u{2075}', '\u{2076}',
];

/// Rounded corner box-drawing characters for panel frames.
pub mod frame_chars {
    pub const TOP_LEFT: &str = "╭";
    pub const TOP_RIGHT: &str = "╮";
    pub const BOTTOM_LEFT: &str = "╰";
    pub const BOTTOM_RIGHT: &str = "╯";
    pub const HORIZONTAL: &str = "─";
    pub const VERTICAL: &str = "│";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_horizon_theme_has_distinct_dimension_colors() {
        let t = &THEMES[0];
        assert_ne!(format!("{:?}", t.cpu_accent), format!("{:?}", t.gpu_accent));
        assert_ne!(format!("{:?}", t.cpu_accent), format!("{:?}", t.mem_accent));
        assert_ne!(format!("{:?}", t.gpu_accent), format!("{:?}", t.mem_accent));
    }

    #[test]
    fn test_horizon_gradient_stops_defined() {
        let t = &THEMES[0];
        assert!(matches!(t.gradient_green, Color::Rgb(_, _, _)));
        assert!(matches!(t.gradient_yellow, Color::Rgb(_, _, _)));
        assert!(matches!(t.gradient_orange, Color::Rgb(_, _, _)));
        assert!(matches!(t.gradient_red, Color::Rgb(_, _, _)));
    }

    #[test]
    fn test_default_theme_is_horizon() {
        assert_eq!(default_theme().name, "horizon");
    }

    #[test]
    fn net_upload_and_download_have_distinct_colors() {
        let t = &THEMES[0];
        assert_ne!(
            format!("{:?}", t.net_upload),
            format!("{:?}", t.net_download),
            "net_upload and net_download should be different colors"
        );
    }

    #[test]
    fn test_themes_array_has_at_least_10() {
        assert!(
            THEMES.len() >= 10,
            "expected >= 10 themes, got {}",
            THEMES.len()
        );
    }

    #[test]
    fn test_all_themes_have_distinct_upload_download() {
        for theme in THEMES.iter() {
            assert_ne!(
                format!("{:?}", theme.net_upload),
                format!("{:?}", theme.net_download),
                "theme '{}' has identical upload/download colors",
                theme.name
            );
        }
    }

    #[test]
    fn test_all_themes_have_names() {
        for theme in THEMES.iter() {
            assert!(!theme.name.is_empty(), "theme has empty name");
        }
    }

    #[test]
    fn test_theme_names_returns_all() {
        let names = theme_names();
        assert_eq!(names.len(), THEMES.len());
        assert_eq!(names[0], "horizon");
    }

    #[test]
    fn test_dracula_theme_bg() {
        let dracula = THEMES
            .iter()
            .find(|t| t.name == "dracula")
            .expect("dracula theme missing");
        assert!(matches!(dracula.bg, Color::Rgb(40, 42, 54)));
    }

    #[test]
    fn test_nord_theme_bg() {
        let nord = THEMES
            .iter()
            .find(|t| t.name == "nord")
            .expect("nord theme missing");
        assert!(matches!(nord.bg, Color::Rgb(46, 52, 64)));
    }

    #[test]
    fn test_frame_chars_are_single_grapheme() {
        assert_eq!(frame_chars::TOP_LEFT.chars().count(), 1);
        assert_eq!(frame_chars::TOP_RIGHT.chars().count(), 1);
        assert_eq!(frame_chars::BOTTOM_LEFT.chars().count(), 1);
        assert_eq!(frame_chars::BOTTOM_RIGHT.chars().count(), 1);
    }

    // --- Iteration 16 tests ---

    #[test]
    fn test_derive_companion_basic() {
        // Purple CPU → shifted companion should be different
        let base = Color::Rgb(184, 119, 219);
        let derived = derive_companion(base, 30.0, 0.9);
        assert_ne!(format!("{:?}", base), format!("{:?}", derived));
        assert!(matches!(derived, Color::Rgb(_, _, _)));
    }

    #[test]
    fn test_derive_companion_achromatic() {
        // Gray input: hue rotation should have no perceptual effect
        let gray = Color::Rgb(128, 128, 128);
        let derived = derive_companion(gray, 30.0, 0.9);
        // Should be dimmed gray (sat_factor applies to already-zero saturation → dimming only)
        match derived {
            Color::Rgb(r, g, b) => {
                assert_eq!(r, g);
                assert_eq!(g, b);
            }
            _ => panic!("expected Rgb"),
        }
    }

    #[test]
    fn test_gpu_accent_differs_from_cpu() {
        for theme in THEMES.iter() {
            assert_ne!(
                format!("{:?}", theme.gpu_accent),
                format!("{:?}", theme.cpu_accent),
                "theme '{}': gpu_accent should differ from cpu_accent",
                theme.name
            );
        }
    }

    #[test]
    fn test_power_accent_differs_from_mem() {
        for theme in THEMES.iter() {
            assert_ne!(
                format!("{:?}", theme.power_accent),
                format!("{:?}", theme.mem_accent),
                "theme '{}': power_accent should differ from mem_accent",
                theme.name
            );
        }
    }

    #[test]
    fn test_gpu_power_derived_from_hue_midpoint() {
        for theme in THEMES.iter() {
            let expected_gpu = hue_midpoint(theme.cpu_accent, theme.mem_accent);
            let expected_power = hue_midpoint(theme.net_download, theme.process_accent);
            assert_eq!(
                format!("{:?}", theme.gpu_accent),
                format!("{:?}", expected_gpu),
                "theme '{}': gpu_accent should match hue_midpoint(cpu_accent, mem_accent)",
                theme.name
            );
            assert_eq!(
                format!("{:?}", theme.power_accent),
                format!("{:?}", expected_power),
                "theme '{}': power_accent should match hue_midpoint(net_download, process_accent)",
                theme.name
            );
        }
    }

    #[test]
    fn test_bg_luminance_dark_vs_light() {
        let horizon = &THEMES[0];
        let sol_light = THEMES.iter().find(|t| t.name == "solarized-light").unwrap();
        assert!(bg_luminance(horizon) < 0.2, "Horizon should be dark");
        assert!(
            bg_luminance(sol_light) > 0.7,
            "Solarized Light should be light"
        );
    }

    #[test]
    fn test_adaptive_border_dim_dark_vs_light() {
        let horizon = &THEMES[0];
        let sol_light = THEMES.iter().find(|t| t.name == "solarized-light").unwrap();
        assert_eq!(adaptive_border_dim(horizon), 0.55);
        assert_eq!(adaptive_border_dim(sol_light), 0.35);
    }

    #[test]
    fn test_horizon_corrected_palette() {
        let h = &THEMES[0];
        // CPU should be purple (#B877DB)
        assert!(matches!(h.cpu_accent, Color::Rgb(184, 119, 219)));
        // MEM should be teal (#27D796, btop mem_box)
        assert!(matches!(h.mem_accent, Color::Rgb(39, 215, 150)));
        // Net download should be red (#E95678, btop net_box)
        assert!(matches!(h.net_download, Color::Rgb(233, 86, 120)));
    }

    #[test]
    fn test_panel_superscripts() {
        assert_eq!(PANEL_SUPERSCRIPTS.len(), 6);
        assert_eq!(PANEL_SUPERSCRIPTS[0], '¹');
        assert_eq!(PANEL_SUPERSCRIPTS[5], '⁶');
    }
}
