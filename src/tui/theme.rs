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

/// Get the default theme.
pub fn default_theme() -> &'static Theme {
    &HORIZON
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
    fn test_frame_chars_are_single_grapheme() {
        assert_eq!(frame_chars::TOP_LEFT.chars().count(), 1);
        assert_eq!(frame_chars::TOP_RIGHT.chars().count(), 1);
        assert_eq!(frame_chars::BOTTOM_LEFT.chars().count(), 1);
        assert_eq!(frame_chars::BOTTOM_RIGHT.chars().count(), 1);
    }
}
