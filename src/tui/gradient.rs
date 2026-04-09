use ratatui::style::Color;

/// Gradient stops: green(0.0) -> yellow(0.4) -> orange(0.7) -> red(1.0)
const STOPS: &[(f64, u8, u8, u8)] = &[
    (0.0, 0, 200, 83),    // green
    (0.4, 255, 214, 0),   // yellow
    (0.7, 255, 152, 0),   // orange
    (1.0, 255, 61, 0),    // red
];

/// Map a normalized value (0.0-1.0) to an RGB gradient color.
pub fn value_to_color(normalized: f64) -> Color {
    let t = normalized.clamp(0.0, 1.0);

    // Find the two stops surrounding t
    let mut lo = 0;
    for (i, stop) in STOPS.iter().enumerate().skip(1) {
        if stop.0 >= t {
            lo = i - 1;
            break;
        }
    }
    let hi = (lo + 1).min(STOPS.len() - 1);

    let (t0, r0, g0, b0) = STOPS[lo];
    let (t1, r1, g1, b1) = STOPS[hi];

    let frac = if (t1 - t0).abs() < f64::EPSILON {
        0.0
    } else {
        (t - t0) / (t1 - t0)
    };

    let lerp = |a: u8, b: u8| -> u8 {
        (a as f64 + (b as f64 - a as f64) * frac).round() as u8
    };

    Color::Rgb(lerp(r0, r1), lerp(g0, g1), lerp(b0, b1))
}

/// Map a temperature in Celsius to a gradient color.
/// 30C -> green, 60C -> yellow, 80C -> orange, 95C+ -> red.
pub fn temp_to_color(celsius: f32) -> Color {
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
    value_to_color(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb(c: Color) -> (u8, u8, u8) {
        match c {
            Color::Rgb(r, g, b) => (r, g, b),
            _ => panic!("expected Rgb color"),
        }
    }

    #[test]
    fn test_zero_is_green() {
        let (r, g, b) = rgb(value_to_color(0.0));
        assert_eq!((r, g, b), (0, 200, 83));
    }

    #[test]
    fn test_one_is_red() {
        let (r, g, b) = rgb(value_to_color(1.0));
        assert_eq!((r, g, b), (255, 61, 0));
    }

    #[test]
    fn test_midpoint_is_yellow() {
        let (r, g, b) = rgb(value_to_color(0.4));
        assert_eq!((r, g, b), (255, 214, 0));
    }

    #[test]
    fn test_clamp_above_one() {
        assert_eq!(value_to_color(1.5), value_to_color(1.0));
    }

    #[test]
    fn test_clamp_below_zero() {
        assert_eq!(value_to_color(-0.5), value_to_color(0.0));
    }

    #[test]
    fn test_temp_cold() {
        let (r, g, b) = rgb(temp_to_color(30.0));
        assert_eq!((r, g, b), (0, 200, 83));
    }

    #[test]
    fn test_temp_hot() {
        let (r, g, b) = rgb(temp_to_color(95.0));
        assert_eq!((r, g, b), (255, 61, 0));
    }
}
