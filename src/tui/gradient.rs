use ratatui::style::Color;

use super::theme::Theme;

/// Map a normalized value (0.0-1.0) to an RGB gradient color using theme stops.
/// Stops: green(0.0) -> yellow(0.4) -> orange(0.7) -> red(1.0).
pub fn value_to_color(normalized: f64, theme: &Theme) -> Color {
    let t = normalized.clamp(0.0, 1.0);

    let stops: [(f64, Color); 4] = [
        (0.0, theme.gradient_green),
        (0.4, theme.gradient_yellow),
        (0.7, theme.gradient_orange),
        (1.0, theme.gradient_red),
    ];

    // Find the two stops surrounding t
    let mut lo = 0;
    for (i, stop) in stops.iter().enumerate().skip(1) {
        if stop.0 >= t {
            lo = i - 1;
            break;
        }
    }
    let hi = (lo + 1).min(stops.len() - 1);

    let (t0, c0) = stops[lo];
    let (t1, c1) = stops[hi];

    let (r0, g0, b0) = match c0 {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (0, 0, 0),
    };
    let (r1, g1, b1) = match c1 {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (0, 0, 0),
    };

    let frac = if (t1 - t0).abs() < f64::EPSILON {
        0.0
    } else {
        (t - t0) / (t1 - t0)
    };

    let lerp = |a: u8, b: u8| -> u8 { (a as f64 + (b as f64 - a as f64) * frac).round() as u8 };

    Color::Rgb(lerp(r0, r1), lerp(g0, g1), lerp(b0, b1))
}

/// Map a temperature in Celsius to a gradient color using theme stops.
/// 30C -> green, 60C -> yellow, 80C -> orange, 95C+ -> red.
pub fn temp_to_color(celsius: f32, theme: &Theme) -> Color {
    let normalized = if celsius <= 30.0 {
        0.0
    } else if celsius >= 95.0 {
        1.0
    } else {
        // Piecewise linear mapping matching the gradient stops:
        // 30C -> 0.0, 60C -> 0.4, 80C -> 0.7, 95C -> 1.0
        if celsius <= 60.0 {
            (celsius - 30.0) as f64 / 30.0 * 0.4
        } else if celsius <= 80.0 {
            0.4 + (celsius - 60.0) as f64 / 20.0 * 0.3
        } else {
            0.7 + (celsius - 80.0) as f64 / 15.0 * 0.3
        }
    };
    value_to_color(normalized, theme)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::theme::{HORIZON, THEMES};

    fn rgb(c: Color) -> (u8, u8, u8) {
        match c {
            Color::Rgb(r, g, b) => (r, g, b),
            _ => panic!("expected Rgb color"),
        }
    }

    #[test]
    fn test_zero_is_gradient_start() {
        let theme = &THEMES[0]; // Horizon
        let (r, g, b) = rgb(value_to_color(0.0, theme));
        assert_eq!((r, g, b), (39, 215, 150));
    }

    #[test]
    fn test_one_is_gradient_end() {
        let theme = &THEMES[0];
        let (r, g, b) = rgb(value_to_color(1.0, theme));
        assert_eq!((r, g, b), (233, 86, 120));
    }

    #[test]
    fn test_midpoint_is_gradient_yellow() {
        let theme = &THEMES[0];
        let (r, g, b) = rgb(value_to_color(0.4, theme));
        assert_eq!((r, g, b), (208, 198, 151));
    }

    #[test]
    fn test_clamp_above_one() {
        let theme = &THEMES[0];
        assert_eq!(value_to_color(1.5, theme), value_to_color(1.0, theme));
    }

    #[test]
    fn test_clamp_below_zero() {
        let theme = &THEMES[0];
        assert_eq!(value_to_color(-0.5, theme), value_to_color(0.0, theme));
    }

    #[test]
    fn test_temp_cold() {
        let theme = &THEMES[0];
        let (r, g, b) = rgb(temp_to_color(30.0, theme));
        assert_eq!((r, g, b), (39, 215, 150));
    }

    #[test]
    fn test_temp_hot() {
        let theme = &THEMES[0];
        let (r, g, b) = rgb(temp_to_color(95.0, theme));
        assert_eq!((r, g, b), (233, 86, 120));
    }

    #[test]
    fn test_temp_60c_matches_yellow() {
        let theme = THEMES.iter().find(|t| t.name == "gruvbox").unwrap();
        assert_eq!(temp_to_color(60.0, theme), value_to_color(0.4, theme));
    }

    #[test]
    fn test_temp_80c_matches_orange() {
        let theme = THEMES.iter().find(|t| t.name == "gruvbox").unwrap();
        assert_eq!(temp_to_color(80.0, theme), value_to_color(0.7, theme));
    }

    #[test]
    fn test_themes_have_distinct_gradient_start() {
        // Solarized Dark and Light intentionally share gradient values (same btop source).
        // Verify at least 9 distinct gradient_start colors across 10 themes.
        let mut starts = std::collections::HashSet::new();
        for theme in THEMES.iter() {
            let c = value_to_color(0.0, theme);
            starts.insert(format!("{:?}", c));
        }
        assert!(
            starts.len() >= 9,
            "expected >= 9 distinct gradient starts, got {}",
            starts.len()
        );
    }

    #[test]
    fn test_interpolation_midpoint() {
        // 0.25 is between green(0.0) and yellow(0.4) for Nord
        let nord = THEMES.iter().find(|t| t.name == "nord").unwrap();
        let (r, g, b) = rgb(value_to_color(0.25, nord));
        // Should be between (163,190,140) and (221,200,139)
        assert!(r > 163 && r < 221, "r={r} not between 163 and 221");
        assert!(g > 190 && g <= 200, "g={g} not between 190 and 200");
    }
}
