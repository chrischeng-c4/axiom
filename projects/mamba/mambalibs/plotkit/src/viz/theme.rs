//! Chart theme system with preset themes.
//!
//! Provides named themes (Light, Dark, Minimal, Publication) that configure
//! background, text, grid, axis colors, font settings, and color palettes.

use super::style::{ChartStyle, Color};

/// Named theme presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeName {
    Light,
    Dark,
    Minimal,
    Publication,
}

/// A complete visual theme for charts.
#[derive(Debug, Clone)]
pub struct Theme {
    pub background: String,
    pub text_color: String,
    pub grid_color: String,
    pub font_family: String,
    pub font_size: f64,
    pub axis_color: String,
    pub colors: Vec<String>,
}

impl Theme {
    /// Create a theme from a preset name.
    pub fn from_name(name: ThemeName) -> Self {
        match name {
            ThemeName::Light => Self::light(),
            ThemeName::Dark => Self::dark(),
            ThemeName::Minimal => Self::minimal(),
            ThemeName::Publication => Self::publication(),
        }
    }

    fn light() -> Self {
        Self {
            background: "#ffffff".into(),
            text_color: "#333333".into(),
            grid_color: "#e6e6e6".into(),
            font_family: "sans-serif".into(),
            font_size: 12.0,
            axis_color: "#333333".into(),
            colors: vec![
                "#1f77b4".into(), "#ff7f0e".into(), "#2ca02c".into(),
                "#d62728".into(), "#9467bd".into(), "#8c564b".into(),
                "#e377c2".into(), "#7f7f7f".into(), "#bcbd22".into(),
                "#17becf".into(),
            ],
        }
    }

    fn dark() -> Self {
        Self {
            background: "#1e1e2e".into(),
            text_color: "#cdd6f4".into(),
            grid_color: "#45475a".into(),
            font_family: "sans-serif".into(),
            font_size: 12.0,
            axis_color: "#a6adc8".into(),
            colors: vec![
                "#89b4fa".into(), "#fab387".into(), "#a6e3a1".into(),
                "#f38ba8".into(), "#cba6f7".into(), "#f5c2e7".into(),
                "#94e2d5".into(), "#f9e2af".into(), "#74c7ec".into(),
                "#b4befe".into(),
            ],
        }
    }

    fn minimal() -> Self {
        Self {
            background: "#ffffff".into(),
            text_color: "#555555".into(),
            grid_color: "#f0f0f0".into(),
            font_family: "Helvetica, Arial, sans-serif".into(),
            font_size: 11.0,
            axis_color: "#999999".into(),
            colors: vec![
                "#333333".into(), "#666666".into(), "#999999".into(),
                "#bbbbbb".into(), "#1f77b4".into(), "#ff7f0e".into(),
            ],
        }
    }

    fn publication() -> Self {
        Self {
            background: "#ffffff".into(),
            text_color: "#000000".into(),
            grid_color: "#d9d9d9".into(),
            font_family: "Times New Roman, serif".into(),
            font_size: 14.0,
            axis_color: "#000000".into(),
            colors: vec![
                "#000000".into(), "#e41a1c".into(), "#377eb8".into(),
                "#4daf4a".into(), "#984ea3".into(), "#ff7f00".into(),
                "#a65628".into(), "#f781bf".into(),
            ],
        }
    }

    /// Convert this theme into a `ChartStyle`.
    pub fn to_chart_style(&self) -> ChartStyle {
        ChartStyle {
            background: parse_hex_color(&self.background),
            text_color: parse_hex_color(&self.text_color),
            grid_color: parse_hex_color(&self.grid_color),
            font_size: self.font_size,
            title_size: self.font_size + 4.0,
            show_grid: true,
        }
    }

    /// Get the palette colors parsed as `Color` values.
    pub fn palette_colors(&self) -> Vec<Color> {
        self.colors.iter().map(|c| parse_hex_color(c)).collect()
    }
}

/// Parse a hex color string (e.g., "#1f77b4") into a `Color`.
fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() < 6 {
        return Color::new(0, 0, 0);
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Color::new(r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_light() {
        let theme = Theme::from_name(ThemeName::Light);
        assert_eq!(theme.background, "#ffffff");
        assert!(!theme.colors.is_empty());
    }

    #[test]
    fn test_theme_dark() {
        let theme = Theme::from_name(ThemeName::Dark);
        assert_eq!(theme.background, "#1e1e2e");
        assert_eq!(theme.text_color, "#cdd6f4");
    }

    #[test]
    fn test_theme_minimal() {
        let theme = Theme::from_name(ThemeName::Minimal);
        assert!(theme.font_family.contains("Helvetica"));
        assert_eq!(theme.font_size, 11.0);
    }

    #[test]
    fn test_theme_publication() {
        let theme = Theme::from_name(ThemeName::Publication);
        assert!(theme.font_family.contains("Times"));
        assert_eq!(theme.font_size, 14.0);
    }

    #[test]
    fn test_to_chart_style() {
        let theme = Theme::from_name(ThemeName::Dark);
        let style = theme.to_chart_style();
        assert_eq!(style.background, Color::new(0x1e, 0x1e, 0x2e));
        assert_eq!(style.font_size, 12.0);
        assert_eq!(style.title_size, 16.0);
        assert!(style.show_grid);
    }

    #[test]
    fn test_palette_colors() {
        let theme = Theme::from_name(ThemeName::Light);
        let colors = theme.palette_colors();
        assert_eq!(colors.len(), 10);
        assert_eq!(colors[0], Color::new(31, 119, 180)); // #1f77b4
    }

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("#ff0000"), Color::new(255, 0, 0));
        assert_eq!(parse_hex_color("#00ff00"), Color::new(0, 255, 0));
        assert_eq!(parse_hex_color("#0000ff"), Color::new(0, 0, 255));
        assert_eq!(parse_hex_color("1f77b4"), Color::new(31, 119, 180));
    }

    #[test]
    fn test_parse_hex_color_invalid() {
        assert_eq!(parse_hex_color("#xyz"), Color::new(0, 0, 0));
        assert_eq!(parse_hex_color(""), Color::new(0, 0, 0));
    }

    #[test]
    fn test_all_themes_have_colors() {
        for name in [ThemeName::Light, ThemeName::Dark, ThemeName::Minimal, ThemeName::Publication] {
            let theme = Theme::from_name(name);
            assert!(theme.colors.len() >= 6, "Theme {:?} should have at least 6 colors", name);
        }
    }
}
