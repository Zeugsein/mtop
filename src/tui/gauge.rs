use ratatui::style::Style;
use ratatui::text::Span;

use super::gradient::value_to_color;
use super::theme::Theme;

/// Gauge fill character — btop uses ■ (U+25A0 BLACK SQUARE).
const GAUGE_CHAR: &str = "■";

/// Render a horizontal gauge bar as a vector of styled spans.
///
/// The bar is `width` characters wide, filled proportionally to `value / max`.
/// Filled portion is gradient-colored (green→red); unfilled is muted.
/// An optional label (e.g. "12.4/16.0 GB") is appended after the bar.
pub fn render_gauge_bar<'a>(value: f64, max: f64, width: usize, label: &'a str, theme: &Theme) -> Vec<Span<'a>> {
    if width == 0 {
        return vec![];
    }

    let fraction = if max > 0.0 {
        (value / max).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let filled = (fraction * width as f64).round() as usize;
    let filled = filled.min(width);
    let empty = width - filled;

    let fill_color = value_to_color(fraction);
    let mut spans = Vec::with_capacity(3);

    if filled > 0 {
        spans.push(Span::styled(
            GAUGE_CHAR.repeat(filled),
            Style::default().fg(fill_color),
        ));
    }

    if empty > 0 {
        spans.push(Span::styled(
            GAUGE_CHAR.repeat(empty),
            Style::default().fg(theme.muted),
        ));
    }

    if !label.is_empty() {
        spans.push(Span::styled(
            format!(" {label}"),
            Style::default().fg(theme.fg),
        ));
    }

    spans
}

/// Render a compact gauge showing percentage and a short bar.
/// Format: "77% ■■■■■■■■░░" — useful for right-detail areas.
pub fn render_compact_gauge(fraction: f64, width: usize, theme: &Theme) -> Vec<Span<'static>> {
    let pct = (fraction * 100.0).round() as u32;
    let pct_str = format!("{pct:>3}% ");
    let bar_width = width.saturating_sub(pct_str.len());

    let filled = (fraction * bar_width as f64).round() as usize;
    let filled = filled.min(bar_width);
    let empty = bar_width - filled;

    let fill_color = value_to_color(fraction);
    let mut spans = Vec::with_capacity(3);

    spans.push(Span::styled(
        pct_str,
        Style::default().fg(fill_color),
    ));

    if filled > 0 {
        spans.push(Span::styled(
            GAUGE_CHAR.repeat(filled),
            Style::default().fg(fill_color),
        ));
    }

    if empty > 0 {
        spans.push(Span::styled(
            GAUGE_CHAR.repeat(empty),
            Style::default().fg(theme.muted),
        ));
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::theme::HORIZON;

    #[test]
    fn test_gauge_bar_zero_fill() {
        let spans = render_gauge_bar(0.0, 100.0, 10, "0/100", &HORIZON);
        assert!(!spans.is_empty());
        let content: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(content.contains('■'));
        assert!(content.contains("0/100"));
    }

    #[test]
    fn test_gauge_bar_half_fill() {
        let spans = render_gauge_bar(50.0, 100.0, 10, "50/100", &HORIZON);
        let content: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(content.contains('■'));
        assert!(content.contains("50/100"));
    }

    #[test]
    fn test_gauge_bar_full_fill() {
        let spans = render_gauge_bar(100.0, 100.0, 10, "100/100", &HORIZON);
        let content: String = spans.iter().map(|s| s.content.as_ref()).collect();
        // Should be all filled ■, no second span with muted color
        assert_eq!(spans.len(), 2); // filled + label, no empty
    }

    #[test]
    fn test_gauge_bar_zero_width() {
        let spans = render_gauge_bar(50.0, 100.0, 0, "test", &HORIZON);
        assert!(spans.is_empty());
    }

    #[test]
    fn test_gauge_bar_zero_max() {
        let spans = render_gauge_bar(50.0, 0.0, 10, "N/A", &HORIZON);
        let content: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(content.contains('■'));
    }

    #[test]
    fn test_compact_gauge() {
        let spans = render_compact_gauge(0.77, 15, &HORIZON);
        let content: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(content.contains("77%"));
        assert!(content.contains('■'));
    }

    #[test]
    fn test_gauge_uses_square_char() {
        let spans = render_gauge_bar(50.0, 100.0, 10, "", &HORIZON);
        let content: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(content.contains('■'));
        assert!(!content.contains('█'));
        assert!(!content.contains('░'));
    }
}
