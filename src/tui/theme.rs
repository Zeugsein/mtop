use ratatui::style::Color;

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

    // Gradient stops (green → yellow → orange → red)
    pub gradient_green: Color,
    pub gradient_yellow: Color,
    pub gradient_orange: Color,
    pub gradient_red: Color,
}

/// Horizon theme — warm, dark background with vibrant accents.
/// Inspired by the Horizon color scheme.
pub const HORIZON: Theme = Theme {
    name: "horizon",
    bg: Color::Rgb(28, 30, 38),
    fg: Color::Rgb(205, 209, 219),
    accent: Color::Rgb(233, 175, 100),
    muted: Color::Rgb(107, 112, 127),
    border: Color::Rgb(60, 63, 75),
    header_bg: Color::Rgb(233, 175, 100),
    header_fg: Color::Rgb(28, 30, 38),

    cpu_accent: Color::Rgb(38, 187, 194),    // teal
    gpu_accent: Color::Rgb(250, 200, 80),    // amber
    mem_accent: Color::Rgb(160, 120, 230),   // purple
    net_upload: Color::Rgb(38, 187, 194),    // cyan
    net_download: Color::Rgb(230, 100, 170), // magenta
    power_accent: Color::Rgb(233, 175, 100), // warm orange

    gradient_green: Color::Rgb(0, 200, 83),
    gradient_yellow: Color::Rgb(255, 214, 0),
    gradient_orange: Color::Rgb(255, 152, 0),
    gradient_red: Color::Rgb(255, 61, 0),
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
    cpu_accent: Color::Rgb(139, 233, 253),
    gpu_accent: Color::Rgb(241, 250, 140),
    mem_accent: Color::Rgb(189, 147, 249),
    net_upload: Color::Rgb(80, 250, 123),
    net_download: Color::Rgb(255, 121, 198),
    power_accent: Color::Rgb(255, 184, 108),
    gradient_green: Color::Rgb(80, 250, 123),
    gradient_yellow: Color::Rgb(241, 250, 140),
    gradient_orange: Color::Rgb(255, 184, 108),
    gradient_red: Color::Rgb(255, 85, 85),
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
    gradient_green: Color::Rgb(166, 227, 161),
    gradient_yellow: Color::Rgb(249, 226, 175),
    gradient_orange: Color::Rgb(250, 179, 135),
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
    gradient_green: Color::Rgb(163, 190, 140),
    gradient_yellow: Color::Rgb(235, 203, 139),
    gradient_orange: Color::Rgb(208, 135, 112),
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
    gradient_green: Color::Rgb(133, 153, 0),
    gradient_yellow: Color::Rgb(181, 137, 0),
    gradient_orange: Color::Rgb(203, 75, 22),
    gradient_red: Color::Rgb(220, 50, 47),
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
    gradient_green: Color::Rgb(133, 153, 0),
    gradient_yellow: Color::Rgb(181, 137, 0),
    gradient_orange: Color::Rgb(203, 75, 22),
    gradient_red: Color::Rgb(220, 50, 47),
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
    gradient_green: Color::Rgb(184, 187, 38),
    gradient_yellow: Color::Rgb(250, 189, 47),
    gradient_orange: Color::Rgb(254, 128, 25),
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
    gradient_green: Color::Rgb(158, 206, 106),
    gradient_yellow: Color::Rgb(224, 175, 104),
    gradient_orange: Color::Rgb(255, 158, 100),
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
    gradient_green: Color::Rgb(152, 195, 121),
    gradient_yellow: Color::Rgb(229, 192, 123),
    gradient_orange: Color::Rgb(209, 154, 102),
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
    gradient_green: Color::Rgb(166, 226, 46),
    gradient_yellow: Color::Rgb(230, 219, 116),
    gradient_orange: Color::Rgb(253, 151, 31),
    gradient_red: Color::Rgb(249, 38, 114),
};

/// All available themes, indexed for cycling.
pub const THEMES: &[&Theme] = &[
    &HORIZON,
    &DRACULA,
    &CATPPUCCIN,
    &NORD,
    &SOLARIZED_DARK,
    &SOLARIZED_LIGHT,
    &GRUVBOX,
    &TOKYO_NIGHT,
    &ONE_DARK,
    &MONOKAI,
];

/// Return the list of available theme names.
pub fn theme_names() -> Vec<&'static str> {
    THEMES.iter().map(|t| t.name).collect()
}

/// Look up a theme by name, falling back to the first theme (Horizon).
pub fn theme_by_name(name: &str) -> &'static Theme {
    THEMES
        .iter()
        .find(|t| t.name == name)
        .unwrap_or(&THEMES[0])
}

/// Get the default theme.
pub fn default_theme() -> &'static Theme {
    &HORIZON
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
        let t = &HORIZON;
        // Each dimension should have a distinct accent
        assert_ne!(format!("{:?}", t.cpu_accent), format!("{:?}", t.gpu_accent));
        assert_ne!(format!("{:?}", t.cpu_accent), format!("{:?}", t.mem_accent));
        assert_ne!(format!("{:?}", t.gpu_accent), format!("{:?}", t.mem_accent));
    }

    #[test]
    fn test_horizon_gradient_stops_defined() {
        let t = &HORIZON;
        // Gradient stops should all be RGB colors
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
    /// net_upload and net_download must use distinct accent colors so upload
    /// and download sparklines are visually distinguishable in the TUI.
    fn net_upload_and_download_have_distinct_colors() {
        let t = &HORIZON;
        assert_ne!(
            format!("{:?}", t.net_upload),
            format!("{:?}", t.net_download),
            "net_upload and net_download should be different colors"
        );
    }

    #[test]
    fn test_themes_array_has_at_least_10() {
        assert!(THEMES.len() >= 10, "expected >= 10 themes, got {}", THEMES.len());
    }

    #[test]
    fn test_all_themes_have_distinct_upload_download() {
        for theme in THEMES {
            assert_ne!(
                format!("{:?}", theme.net_upload),
                format!("{:?}", theme.net_download),
                "theme '{}' has identical upload/download colors", theme.name
            );
        }
    }

    #[test]
    fn test_all_themes_have_names() {
        for theme in THEMES {
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
        let dracula = THEMES.iter().find(|t| t.name == "dracula").expect("dracula theme missing");
        assert!(matches!(dracula.bg, Color::Rgb(40, 42, 54)));
    }

    #[test]
    fn test_nord_theme_bg() {
        let nord = THEMES.iter().find(|t| t.name == "nord").expect("nord theme missing");
        assert!(matches!(nord.bg, Color::Rgb(46, 52, 64)));
    }

    #[test]
    fn test_frame_chars_are_single_grapheme() {
        assert_eq!(frame_chars::TOP_LEFT.chars().count(), 1);
        assert_eq!(frame_chars::TOP_RIGHT.chars().count(), 1);
        assert_eq!(frame_chars::BOTTOM_LEFT.chars().count(), 1);
        assert_eq!(frame_chars::BOTTOM_RIGHT.chars().count(), 1);
    }
}
