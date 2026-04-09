use ratatui::style::Color;

use super::gradient::value_to_color;

/// Braille dot positions for the left column (single-column mode).
/// Each braille char uses dots 1,2,3,7 (left column, top to bottom).
/// Dot numbering: 1=top-left(0x01), 2=mid-left(0x02), 3=bottom-left(0x04), 7=bottom-bottom-left(0x40)
const LEFT_DOTS: [u32; 4] = [
    0x40, // dot 7 (bottom row)
    0x04, // dot 3
    0x02, // dot 2
    0x01, // dot 1 (top row)
];

const BRAILLE_BASE: u32 = 0x2800;

/// Render a braille sparkline from a slice of values.
///
/// Each value maps to one braille character using the left-column dots (4 vertical levels).
/// Returns `(braille_char, color)` pairs, truncated or padded to `width`.
pub fn render_braille_sparkline(values: &[f64], max_value: f64, width: usize) -> Vec<(char, Color)> {
    if values.is_empty() || width == 0 {
        return Vec::new();
    }

    let safe_max = if max_value <= 0.0 { 1.0 } else { max_value };

    // Take the last `width` values (most recent), or all if fewer
    let start = values.len().saturating_sub(width);
    let visible = &values[start..];

    let mut result = Vec::with_capacity(width);

    for &v in visible {
        let normalized = (v / safe_max).clamp(0.0, 1.0);
        // Map to 0-4 filled dots
        let filled = (normalized * 4.0).round() as usize;

        let mut code = 0u32;
        for &dot in &LEFT_DOTS[..filled.min(4)] {
            code |= dot;
        }

        let ch = char::from_u32(BRAILLE_BASE + code).unwrap_or('\u{2800}');
        let color = value_to_color(normalized);
        result.push((ch, color));
    }

    // Pad with empty braille if fewer values than width
    while result.len() < width {
        result.push(('\u{2800}', value_to_color(0.0)));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_buffer() {
        let result = render_braille_sparkline(&[], 100.0, 10);
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_value() {
        let result = render_braille_sparkline(&[50.0], 100.0, 5);
        assert_eq!(result.len(), 5);
        // First char is padded empty, last is the value (or first is value depending on padding)
        // Actually: start = 0, visible = [50.0], then padded to 5
        assert_ne!(result[0].0, '\u{2800}'); // the value itself
    }

    #[test]
    fn test_full_scale() {
        let result = render_braille_sparkline(&[100.0], 100.0, 1);
        assert_eq!(result.len(), 1);
        // All 4 dots filled: 0x40 | 0x04 | 0x02 | 0x01 = 0x47
        let expected = char::from_u32(0x2800 + 0x47).unwrap();
        assert_eq!(result[0].0, expected);
    }

    #[test]
    fn test_zero_value() {
        let result = render_braille_sparkline(&[0.0], 100.0, 1);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, '\u{2800}');
    }

    #[test]
    fn test_correct_width() {
        let values: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let result = render_braille_sparkline(&values, 20.0, 10);
        assert_eq!(result.len(), 10);
    }
}
