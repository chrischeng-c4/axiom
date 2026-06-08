//! Color palettes for visualizations.

use super::style::Color;

/// A named color palette.
#[derive(Debug, Clone)]
pub struct Palette {
    pub name: String,
    pub colors: Vec<Color>,
}

impl Palette {
    /// Get color at index (wraps around).
    pub fn get(&self, idx: usize) -> Color {
        self.colors[idx % self.colors.len()]
    }

    /// Number of colors in the palette.
    pub fn len(&self) -> usize {
        self.colors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }
}

/// Default categorical palette (similar to matplotlib's tab10).
pub fn categorical() -> Palette {
    Palette {
        name: "categorical".into(),
        colors: vec![
            Color::new(31, 119, 180),   // blue
            Color::new(255, 127, 14),   // orange
            Color::new(44, 160, 44),    // green
            Color::new(214, 39, 40),    // red
            Color::new(148, 103, 189),  // purple
            Color::new(140, 86, 75),    // brown
            Color::new(227, 119, 194),  // pink
            Color::new(127, 127, 127),  // gray
            Color::new(188, 189, 34),   // olive
            Color::new(23, 190, 207),   // cyan
        ],
    }
}

/// Sequential blue palette (for heatmaps).
pub fn sequential_blue() -> Palette {
    Palette {
        name: "sequential_blue".into(),
        colors: vec![
            Color::new(247, 251, 255),
            Color::new(222, 235, 247),
            Color::new(198, 219, 239),
            Color::new(158, 202, 225),
            Color::new(107, 174, 214),
            Color::new(66, 146, 198),
            Color::new(33, 113, 181),
            Color::new(8, 81, 156),
            Color::new(8, 48, 107),
        ],
    }
}

/// Diverging red-blue palette.
pub fn diverging() -> Palette {
    Palette {
        name: "diverging".into(),
        colors: vec![
            Color::new(178, 24, 43),
            Color::new(214, 96, 77),
            Color::new(244, 165, 130),
            Color::new(253, 219, 199),
            Color::new(247, 247, 247),
            Color::new(209, 229, 240),
            Color::new(146, 197, 222),
            Color::new(67, 147, 195),
            Color::new(33, 102, 172),
        ],
    }
}

/// Interpolate a sequential palette to get a color for a normalized value [0,1].
pub fn interpolate_color(palette: &Palette, t: f64) -> Color {
    let t = t.clamp(0.0, 1.0);
    let n = palette.len();
    if n == 0 {
        return Color::new(0, 0, 0);
    }
    if n == 1 {
        return palette.colors[0];
    }

    let idx_f = t * (n - 1) as f64;
    let lo = idx_f.floor() as usize;
    let hi = (lo + 1).min(n - 1);
    let frac = idx_f - lo as f64;

    let c1 = &palette.colors[lo];
    let c2 = &palette.colors[hi];
    Color::new(
        (c1.r as f64 * (1.0 - frac) + c2.r as f64 * frac) as u8,
        (c1.g as f64 * (1.0 - frac) + c2.g as f64 * frac) as u8,
        (c1.b as f64 * (1.0 - frac) + c2.b as f64 * frac) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorical_palette() {
        let p = categorical();
        assert_eq!(p.len(), 10);
        assert_eq!(p.get(0), Color::new(31, 119, 180));
        assert_eq!(p.get(10), p.get(0)); // wraps
    }

    #[test]
    fn test_interpolate() {
        let p = sequential_blue();
        let c0 = interpolate_color(&p, 0.0);
        let c1 = interpolate_color(&p, 1.0);
        assert_eq!(c0, p.colors[0]);
        assert_eq!(c1, p.colors[p.len() - 1]);

        let mid = interpolate_color(&p, 0.5);
        assert!(mid.r < 200); // should be blueish
    }

    #[test]
    fn test_diverging() {
        let p = diverging();
        assert_eq!(p.len(), 9);
    }
}
