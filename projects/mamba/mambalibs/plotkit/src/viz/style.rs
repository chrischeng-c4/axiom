//! Colors, styles, and theming for charts.

/// RGB color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn with_opacity(&self, opacity: f64) -> String {
        format!(
            "rgba({},{},{},{:.2})",
            self.r, self.g, self.b, opacity
        )
    }
}

// Default palette (colorblind-friendly)
pub const BLUE: Color = Color::new(31, 119, 180);
pub const ORANGE: Color = Color::new(255, 127, 14);
pub const GREEN: Color = Color::new(44, 160, 44);
pub const RED: Color = Color::new(214, 39, 40);
pub const PURPLE: Color = Color::new(148, 103, 189);
pub const BROWN: Color = Color::new(140, 86, 75);
pub const PINK: Color = Color::new(227, 119, 194);
pub const GRAY: Color = Color::new(127, 127, 127);

/// Default color palette.
pub fn default_palette() -> Vec<Color> {
    vec![BLUE, ORANGE, GREEN, RED, PURPLE, BROWN, PINK, GRAY]
}

/// Line style.
#[derive(Debug, Clone)]
pub struct LineStyle {
    pub color: Color,
    pub width: f64,
}

impl Default for LineStyle {
    fn default() -> Self {
        Self {
            color: BLUE,
            width: 2.0,
        }
    }
}

/// Bar style.
#[derive(Debug, Clone)]
pub struct BarStyle {
    pub color: Color,
    pub opacity: f64,
}

impl Default for BarStyle {
    fn default() -> Self {
        Self {
            color: BLUE,
            opacity: 0.8,
        }
    }
}

/// Point style for scatter plots.
#[derive(Debug, Clone)]
pub struct PointStyle {
    pub color: Color,
    pub radius: f64,
    pub opacity: f64,
}

impl Default for PointStyle {
    fn default() -> Self {
        Self {
            color: BLUE,
            radius: 4.0,
            opacity: 0.7,
        }
    }
}

/// Overall chart style.
#[derive(Debug, Clone)]
pub struct ChartStyle {
    pub background: Color,
    pub text_color: Color,
    pub grid_color: Color,
    pub font_size: f64,
    pub title_size: f64,
    pub show_grid: bool,
}

impl Default for ChartStyle {
    fn default() -> Self {
        Self {
            background: Color::new(255, 255, 255),
            text_color: Color::new(51, 51, 51),
            grid_color: Color::new(230, 230, 230),
            font_size: 12.0,
            title_size: 16.0,
            show_grid: true,
        }
    }
}
