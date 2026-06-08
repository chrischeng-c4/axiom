//! Rendering methods for additional chart types: Polar, Donut, Surface3D.

use crate::viz::style::ChartStyle;

use super::svg::{viridis, SvgRenderer};

impl SvgRenderer {
    /// Render a polar/radar chart.
    pub(crate) fn render_polar(
        &mut self,
        axis_labels: &[String],
        datasets: &[(String, Vec<f64>)],
        style: &ChartStyle,
    ) {
        if axis_labels.is_empty() || datasets.is_empty() {
            return;
        }

        let n_axes = axis_labels.len();
        let cx = self.width / 2.0;
        let cy = self.height / 2.0;
        let radius = (self.plot_width().min(self.plot_height()) / 2.0) * 0.75;
        let palette = crate::viz::style::default_palette();

        // Find global max across all datasets for scaling
        let global_max = datasets
            .iter()
            .flat_map(|(_, vals)| vals.iter().copied())
            .fold(0.0_f64, f64::max)
            .max(1e-10);

        // Draw concentric grid circles (3 levels)
        let grid_levels = 3;
        for level in 1..=grid_levels {
            let r = radius * level as f64 / grid_levels as f64;
            self.elements.push(format!(
                r#"<circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="none" stroke="{}" stroke-width="0.5" opacity="0.5" />"#,
                cx, cy, r, style.grid_color.to_hex()
            ));
        }

        // Draw axis lines and labels
        let angle_step = 2.0 * std::f64::consts::PI / n_axes as f64;
        for (i, label) in axis_labels.iter().enumerate() {
            let angle = -std::f64::consts::FRAC_PI_2 + i as f64 * angle_step;
            let ex = cx + radius * angle.cos();
            let ey = cy + radius * angle.sin();

            // Axis line
            self.line(cx, cy, ex, ey, &style.grid_color.to_hex(), 0.5);

            // Label
            let lx = cx + (radius + 15.0) * angle.cos();
            let ly = cy + (radius + 15.0) * angle.sin();
            let anchor = if (angle.cos()).abs() < 0.1 {
                "middle"
            } else if angle.cos() > 0.0 {
                "start"
            } else {
                "end"
            };
            self.text(lx, ly + 4.0, label, &style.text_color.to_hex(), style.font_size * 0.85, anchor);
        }

        // Draw each dataset as a filled polygon
        for (di, (_, vals)) in datasets.iter().enumerate() {
            let color = &palette[di % palette.len()];
            let mut path = String::new();

            for (i, &val) in vals.iter().take(n_axes).enumerate() {
                let angle = -std::f64::consts::FRAC_PI_2 + i as f64 * angle_step;
                let r = radius * (val / global_max).clamp(0.0, 1.0);
                let px = cx + r * angle.cos();
                let py = cy + r * angle.sin();
                if i == 0 {
                    path.push_str(&format!("M{:.1},{:.1}", px, py));
                } else {
                    path.push_str(&format!(" L{:.1},{:.1}", px, py));
                }
            }
            path.push_str(" Z");

            // Filled polygon
            self.elements.push(format!(
                r#"<path d="{}" fill="{}" opacity="0.25" stroke="{}" stroke-width="2" />"#,
                path,
                color.to_hex(),
                color.to_hex()
            ));

            // Data points
            for (i, &val) in vals.iter().take(n_axes).enumerate() {
                let angle = -std::f64::consts::FRAC_PI_2 + i as f64 * angle_step;
                let r = radius * (val / global_max).clamp(0.0, 1.0);
                let px = cx + r * angle.cos();
                let py = cy + r * angle.sin();
                self.circle(px, py, 3.0, &color.to_hex(), 1.0);
            }
        }
    }

    /// Render a donut chart (pie with center hole).
    pub(crate) fn render_donut(
        &mut self,
        labels: &[String],
        values: &[f64],
        hole_ratio: f64,
        style: &ChartStyle,
    ) {
        if values.is_empty() {
            return;
        }
        let total: f64 = values.iter().sum();
        if total <= 0.0 {
            return;
        }

        let cx = self.width / 2.0;
        let cy = self.height / 2.0;
        let outer_r = (self.plot_width().min(self.plot_height()) / 2.0) * 0.8;
        let inner_r = outer_r * hole_ratio.clamp(0.0, 0.95);
        let palette = crate::viz::style::default_palette();

        let mut start_angle: f64 = -std::f64::consts::FRAC_PI_2;
        for (i, &val) in values.iter().enumerate() {
            let sweep = 2.0 * std::f64::consts::PI * val / total;
            let end_angle = start_angle + sweep;
            let large_arc = if sweep > std::f64::consts::PI { 1 } else { 0 };
            let color = &palette[i % palette.len()];

            // Outer arc start/end
            let ox1 = cx + outer_r * start_angle.cos();
            let oy1 = cy + outer_r * start_angle.sin();
            let ox2 = cx + outer_r * end_angle.cos();
            let oy2 = cy + outer_r * end_angle.sin();

            // Inner arc start/end (reversed)
            let ix1 = cx + inner_r * end_angle.cos();
            let iy1 = cy + inner_r * end_angle.sin();
            let ix2 = cx + inner_r * start_angle.cos();
            let iy2 = cy + inner_r * start_angle.sin();

            // Path: outer arc forward, line to inner, inner arc backward, close
            self.elements.push(format!(
                r##"<path d="M{:.1},{:.1} A{:.1},{:.1} 0 {} 1 {:.1},{:.1} L{:.1},{:.1} A{:.1},{:.1} 0 {} 0 {:.1},{:.1} Z" fill="{}" stroke="#ffffff" stroke-width="1" />"##,
                ox1, oy1, outer_r, outer_r, large_arc, ox2, oy2,
                ix1, iy1, inner_r, inner_r, large_arc, ix2, iy2,
                color.to_hex()
            ));

            // Label at midpoint of ring
            let mid = start_angle + sweep / 2.0;
            let label_r = (outer_r + inner_r) / 2.0;
            let lx = cx + label_r * mid.cos();
            let ly = cy + label_r * mid.sin();
            if i < labels.len() && sweep > 0.15 {
                // Only show label if slice is big enough
                self.text(
                    lx, ly + 4.0,
                    &labels[i], &style.text_color.to_hex(),
                    style.font_size * 0.8, "middle",
                );
            }

            start_angle = end_angle;
        }
    }

    /// Render a 3D surface as projected wireframe with color-coded cells.
    pub(crate) fn render_surface_3d(
        &mut self,
        x: &[f64],
        y: &[f64],
        z: &[Vec<f64>],
        style: &ChartStyle,
    ) {
        if x.is_empty() || y.is_empty() || z.is_empty() {
            return;
        }

        let n_rows = y.len().min(z.len());
        let n_cols = x.len();

        // Find z range
        let mut z_min = f64::INFINITY;
        let mut z_max = f64::NEG_INFINITY;
        for row in z.iter().take(n_rows) {
            for &v in row.iter().take(n_cols) {
                z_min = z_min.min(v);
                z_max = z_max.max(v);
            }
        }
        let z_range = (z_max - z_min).max(1e-15);

        // Simple isometric projection parameters
        let cx = self.width / 2.0;
        let cy = self.height * 0.55;
        let scale_x = self.plot_width() * 0.4 / n_cols.max(1) as f64;
        let scale_y = self.plot_height() * 0.4 / n_rows.max(1) as f64;
        let z_scale = self.plot_height() * 0.3;

        let project = |col: usize, row: usize, zv: f64| -> (f64, f64) {
            let iso_x = (col as f64 - n_cols as f64 / 2.0) * scale_x
                - (row as f64 - n_rows as f64 / 2.0) * scale_y * 0.5;
            let iso_y = (col as f64 - n_cols as f64 / 2.0) * scale_x * 0.3
                + (row as f64 - n_rows as f64 / 2.0) * scale_y * 0.6
                - (zv - z_min) / z_range * z_scale;
            (cx + iso_x, cy + iso_y)
        };

        // Draw filled quads from back to front
        for row in 0..n_rows.saturating_sub(1) {
            for col in 0..n_cols.saturating_sub(1) {
                let z00 = z.get(row).and_then(|r| r.get(col)).copied().unwrap_or(0.0);
                let z10 = z.get(row + 1).and_then(|r| r.get(col)).copied().unwrap_or(0.0);
                let z01 = z.get(row).and_then(|r| r.get(col + 1)).copied().unwrap_or(0.0);
                let z11 = z.get(row + 1).and_then(|r| r.get(col + 1)).copied().unwrap_or(0.0);

                let (x0, y0) = project(col, row, z00);
                let (x1, y1) = project(col + 1, row, z01);
                let (x2, y2) = project(col + 1, row + 1, z11);
                let (x3, y3) = project(col, row + 1, z10);

                let avg_z = (z00 + z01 + z10 + z11) / 4.0;
                let t = (avg_z - z_min) / z_range;
                let color = viridis(t);

                self.elements.push(format!(
                    r#"<path d="M{:.1},{:.1} L{:.1},{:.1} L{:.1},{:.1} L{:.1},{:.1} Z" fill="{}" stroke="{}" stroke-width="0.5" opacity="0.85" />"#,
                    x0, y0, x1, y1, x2, y2, x3, y3,
                    color.to_hex(),
                    style.grid_color.to_hex()
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::viz::style::ChartStyle;

    #[test]
    fn test_render_polar_happy() {
        let style = ChartStyle::default();
        let mut r = SvgRenderer::new(400.0, 300.0);
        r.render_polar(
            &["Speed".into(), "Power".into(), "Range".into()],
            &[("Car A".into(), vec![80.0, 60.0, 70.0])],
            &style,
        );
        let svg = r.build_svg();
        assert!(svg.contains("<path"));
        assert!(svg.contains("<circle"));
        assert!(svg.contains("Speed"));
    }

    #[test]
    fn test_render_polar_empty() {
        let style = ChartStyle::default();
        let mut r = SvgRenderer::new(400.0, 300.0);
        r.render_polar(&[], &[], &style);
        assert!(r.elements.is_empty());
    }

    #[test]
    fn test_render_polar_multi_dataset() {
        let style = ChartStyle::default();
        let mut r = SvgRenderer::new(400.0, 300.0);
        r.render_polar(
            &["A".into(), "B".into(), "C".into(), "D".into()],
            &[
                ("Set 1".into(), vec![90.0, 70.0, 80.0, 60.0]),
                ("Set 2".into(), vec![60.0, 80.0, 70.0, 90.0]),
            ],
            &style,
        );
        let svg = r.build_svg();
        // Should have two polygon paths
        assert!(svg.matches("opacity=\"0.25\"").count() >= 2);
    }

    #[test]
    fn test_render_donut_happy() {
        let style = ChartStyle::default();
        let mut r = SvgRenderer::new(400.0, 300.0);
        r.render_donut(
            &["A".into(), "B".into(), "C".into()],
            &[40.0, 35.0, 25.0],
            0.5,
            &style,
        );
        let svg = r.build_svg();
        assert!(svg.contains("<path"));
    }

    #[test]
    fn test_render_donut_empty() {
        let style = ChartStyle::default();
        let mut r = SvgRenderer::new(400.0, 300.0);
        r.render_donut(&[], &[], 0.5, &style);
        assert!(r.elements.is_empty());
    }

    #[test]
    fn test_render_donut_zero_total() {
        let style = ChartStyle::default();
        let mut r = SvgRenderer::new(400.0, 300.0);
        r.render_donut(&["A".into()], &[0.0], 0.5, &style);
        assert!(r.elements.is_empty());
    }

    #[test]
    fn test_render_surface_3d_happy() {
        let style = ChartStyle::default();
        let mut r = SvgRenderer::new(400.0, 300.0);
        r.render_surface_3d(
            &[0.0, 1.0, 2.0],
            &[0.0, 1.0, 2.0],
            &[
                vec![1.0, 2.0, 3.0],
                vec![4.0, 5.0, 6.0],
                vec![7.0, 8.0, 9.0],
            ],
            &style,
        );
        let svg = r.build_svg();
        assert!(svg.contains("<path"));
    }

    #[test]
    fn test_render_surface_3d_empty() {
        let style = ChartStyle::default();
        let mut r = SvgRenderer::new(400.0, 300.0);
        r.render_surface_3d(&[], &[], &[], &style);
        assert!(r.elements.is_empty());
    }
}
