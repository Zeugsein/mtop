use ratatui::style::Color;

use super::gradient::value_to_color;

/// 5×5 braille lookup table for multi-row graphs.
/// Index: [left_fill][right_fill] where fill = 0..4 (number of vertical dots set).
/// Matches btop's `braille_up` table exactly.
///
/// Braille dot layout per character (2 columns × 4 rows):
///   d1(0x01) d4(0x08)
///   d2(0x02) d5(0x10)
///   d3(0x04) d6(0x20)
///   d7(0x40) d8(0x80)
const BRAILLE_UP: [[char; 5]; 5] = [
    [' ', '⢀', '⢠', '⢰', '⢸'], // left=0
    ['⡀', '⣀', '⣠', '⣰', '⣸'], // left=1
    ['⡄', '⣄', '⣤', '⣴', '⣼'], // left=2
    ['⡆', '⣆', '⣦', '⣶', '⣾'], // left=3
    ['⡇', '⣇', '⣧', '⣷', '⣿'], // left=4
];

/// Render a multi-row braille graph from a time series.
///
/// Each braille character encodes two adjacent time samples (left column = even index,
/// right column = odd index). The graph fills `height` terminal rows with `height × 4`
/// vertical dot resolution.
///
/// Returns rows **bottom-to-top**: index 0 = bottom row, index height-1 = top row.
/// Each row is a Vec of `(char, Color)` pairs, `width` characters long.
///
/// `width` is the number of braille characters per row. Since each char holds 2 samples,
/// the function consumes up to `width * 2` values from the end of the input.
pub fn render_braille_graph(
    values: &[f64],
    max_value: f64,
    width: usize,
    height: usize,
) -> Vec<Vec<(char, Color)>> {
    if values.is_empty() || width == 0 || height == 0 {
        return vec![vec![]; height];
    }

    let safe_max = if max_value <= 0.0 { 1.0 } else { max_value };
    let total_dots = height * 4; // total vertical resolution

    // We need width*2 samples (2 per braille char). Take from the end.
    let needed = width * 2;
    let start = values.len().saturating_sub(needed);
    let visible = &values[start..];

    let mut rows: Vec<Vec<(char, Color)>> = Vec::with_capacity(height);

    for _ in 0..height {
        rows.push(Vec::with_capacity(width));
    }

    for col in 0..width {
        let idx_left = col * 2;
        let idx_right = col * 2 + 1;

        let v_left = if idx_left < visible.len() {
            visible[idx_left]
        } else {
            0.0
        };
        let v_right = if idx_right < visible.len() {
            visible[idx_right]
        } else {
            0.0
        };

        // Scale to total vertical dot range
        let scaled_left = ((v_left / safe_max).clamp(0.0, 1.0) * total_dots as f64).round() as usize;
        let scaled_right = ((v_right / safe_max).clamp(0.0, 1.0) * total_dots as f64).round() as usize;

        // Color based on the higher of the two values
        let max_val = v_left.max(v_right);
        let color = value_to_color((max_val / safe_max).clamp(0.0, 1.0));

        // Fill each row
        for (row_idx, row_vec) in rows.iter_mut().enumerate() {
            let row_base = row_idx * 4; // bottom dot position for this row

            let left_fill = if scaled_left > row_base {
                (scaled_left - row_base).min(4)
            } else {
                0
            };
            let right_fill = if scaled_right > row_base {
                (scaled_right - row_base).min(4)
            } else {
                0
            };

            let ch = BRAILLE_UP[left_fill][right_fill];
            row_vec.push((ch, color));
        }
    }

    rows
}

/// Render a single-row braille sparkline (backward compatibility wrapper).
///
/// Each value maps to one braille character using the left-column dots (4 vertical levels).
/// Returns `(braille_char, color)` pairs, truncated or padded to `width`.
pub fn render_braille_sparkline(values: &[f64], max_value: f64, width: usize) -> Vec<(char, Color)> {
    if values.is_empty() || width == 0 {
        return Vec::new();
    }

    let safe_max = if max_value <= 0.0 { 1.0 } else { max_value };

    let start = values.len().saturating_sub(width);
    let visible = &values[start..];

    let mut result = Vec::with_capacity(width);

    for &v in visible {
        let normalized = (v / safe_max).clamp(0.0, 1.0);
        let filled = (normalized * 4.0).round() as usize;

        // Use left-column only from BRAILLE_UP table (right_fill = 0)
        let ch = BRAILLE_UP[filled.min(4)][0];
        let color = value_to_color(normalized);
        result.push((ch, color));
    }

    // Pad with empty braille if fewer values than width
    while result.len() < width {
        result.push((' ', value_to_color(0.0)));
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
        assert_ne!(result[0].0, ' ');
    }

    #[test]
    fn test_full_scale() {
        let result = render_braille_sparkline(&[100.0], 100.0, 1);
        assert_eq!(result.len(), 1);
        // All 4 left dots filled = BRAILLE_UP[4][0] = '⡇'
        assert_eq!(result[0].0, '⡇');
    }

    #[test]
    fn test_zero_value() {
        let result = render_braille_sparkline(&[0.0], 100.0, 1);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, ' ');
    }

    #[test]
    fn test_correct_width() {
        let values: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let result = render_braille_sparkline(&values, 20.0, 10);
        assert_eq!(result.len(), 10);
    }

    // Multi-row graph tests

    #[test]
    fn test_graph_empty() {
        let result = render_braille_graph(&[], 100.0, 10, 5);
        assert_eq!(result.len(), 5);
        for row in &result {
            assert!(row.is_empty());
        }
    }

    #[test]
    fn test_graph_full_scale() {
        // 100% value with height=2 (8 dots total) should fill all rows with ⣿
        let values = vec![100.0, 100.0];
        let result = render_braille_graph(&values, 100.0, 1, 2);
        assert_eq!(result.len(), 2);
        // Both rows should be fully filled (left=4, right=4 = ⣿)
        assert_eq!(result[0][0].0, '⣿'); // bottom row
        assert_eq!(result[1][0].0, '⣿'); // top row
    }

    #[test]
    fn test_graph_zero_scale() {
        let values = vec![0.0, 0.0];
        let result = render_braille_graph(&values, 100.0, 1, 2);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0].0, ' '); // bottom row empty
        assert_eq!(result[1][0].0, ' '); // top row empty
    }

    #[test]
    fn test_graph_half_height() {
        // 50% with height=2 (8 dots). 50% of 8 = 4 dots from bottom.
        // Row 0 (bottom): base=0, fill=4 for both → ⣿
        // Row 1 (top): base=4, fill=0 for both → ' '
        let values = vec![50.0, 50.0];
        let result = render_braille_graph(&values, 100.0, 1, 2);
        assert_eq!(result[0][0].0, '⣿'); // bottom row fully filled
        assert_eq!(result[1][0].0, ' ');  // top row empty
    }

    #[test]
    fn test_graph_29_percent() {
        // 29% with height=10 (40 dots). 29% of 40 = 11.6 → rounds to 12 dots.
        // Row 0 (base=0): fill=4 → ⣿
        // Row 1 (base=4): fill=4 → ⣿
        // Row 2 (base=8): fill=4 → ⣿
        // Row 3 (base=12): fill=0 → ' '
        let values = vec![29.0, 29.0];
        let result = render_braille_graph(&values, 100.0, 1, 10);
        assert_eq!(result.len(), 10);
        assert_eq!(result[0][0].0, '⣿'); // row 0
        assert_eq!(result[1][0].0, '⣿'); // row 1
        assert_eq!(result[2][0].0, '⣿'); // row 2
        assert_eq!(result[3][0].0, ' ');  // row 3 empty
        assert_eq!(result[9][0].0, ' ');  // top row empty
    }

    #[test]
    fn test_graph_width_and_samples() {
        // width=3 needs 6 samples, we provide 4 → last col gets 0,0
        let values = vec![25.0, 50.0, 75.0, 100.0];
        let result = render_braille_graph(&values, 100.0, 3, 1);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 3);
        // Col 0: values 25.0, 50.0 → left=1, right=2 → ⣠
        assert_eq!(result[0][0].0, '⣠');
        // Col 1: values 75.0, 100.0 → left=3, right=4 → ⣾
        assert_eq!(result[0][1].0, '⣾');
        // Col 2: no data (out of bounds) → space
        assert_eq!(result[0][2].0, ' ');
    }

    #[test]
    fn test_graph_asymmetric_values() {
        // Left value high, right value low within same braille char
        let values = vec![100.0, 0.0];
        let result = render_braille_graph(&values, 100.0, 1, 1);
        // left=4, right=0 → ⡇
        assert_eq!(result[0][0].0, '⡇');
    }

    #[test]
    fn test_braille_up_table_corners() {
        assert_eq!(BRAILLE_UP[0][0], ' ');   // nothing
        assert_eq!(BRAILLE_UP[4][4], '⣿');   // everything
        assert_eq!(BRAILLE_UP[4][0], '⡇');   // left full, right empty
        assert_eq!(BRAILLE_UP[0][4], '⢸');   // left empty, right full
    }
}
