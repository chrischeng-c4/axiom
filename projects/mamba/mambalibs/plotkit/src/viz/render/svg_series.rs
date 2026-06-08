//! Series rendering methods for SvgRenderer.

use crate::viz::axis::{format_tick, nice_ticks};
use crate::viz::style::{ChartStyle, Color};

use super::svg::{box_stats, viridis, SvgRenderer};

impl SvgRenderer {
    pub(crate) fn render_line(
        &mut self,
        x: &[f64],
        y: &[f64],
        color: &Color,
        width: f64,
        style: &ChartStyle,
    ) {
        if x.is_empty() || y.is_empty() {
            return;
        }
        let x_ticks = nice_ticks(
            x.iter().copied().fold(f64::INFINITY, f64::min),
            x.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            5,
        );
        let y_ticks = nice_ticks(
            y.iter().copied().fold(f64::INFINITY, f64::min),
            y.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            5,
        );

        self.draw_axes(&x_ticks.ticks, &y_ticks.ticks, &x_ticks, &y_ticks, style);

        let mut path = String::new();
        for (i, (&xi, &yi)) in x.iter().zip(y.iter()).enumerate() {
            let px = self.map_x(xi, x_ticks.min, x_ticks.max);
            let py = self.map_y(yi, y_ticks.min, y_ticks.max);
            if i == 0 {
                path.push_str(&format!("M{:.1},{:.1}", px, py));
            } else {
                path.push_str(&format!(" L{:.1},{:.1}", px, py));
            }
        }
        self.elements.push(format!(
            r#"<path d="{}" fill="none" stroke="{}" stroke-width="{}" />"#,
            path,
            color.to_hex(),
            width
        ));
    }

    pub(crate) fn render_bar(
        &mut self,
        labels: &[String],
        values: &[f64],
        color: &Color,
        opacity: f64,
        style: &ChartStyle,
    ) {
        if labels.is_empty() {
            return;
        }
        let y_ticks = nice_ticks(
            0.0,
            values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            5,
        );

        for &tick in &y_ticks.ticks {
            let py = self.map_y(tick, y_ticks.min, y_ticks.max);
            if style.show_grid {
                self.line(
                    self.margin.left, py,
                    self.margin.left + self.plot_width(), py,
                    &style.grid_color.to_hex(), 1.0,
                );
            }
            self.text(
                self.margin.left - 5.0, py + 4.0,
                &format_tick(tick),
                &style.text_color.to_hex(), style.font_size, "end",
            );
        }

        let bar_width = self.plot_width() / labels.len() as f64 * 0.7;
        let gap = self.plot_width() / labels.len() as f64 * 0.15;

        for (i, (label, &val)) in labels.iter().zip(values.iter()).enumerate() {
            let bx = self.margin.left + gap + i as f64 * (bar_width + 2.0 * gap);
            let bar_h = (val - y_ticks.min) / (y_ticks.max - y_ticks.min) * self.plot_height();
            let by = self.margin.top + self.plot_height() - bar_h;
            self.rect(bx, by, bar_width, bar_h, &color.to_hex(), opacity);

            self.text(
                bx + bar_width / 2.0,
                self.margin.top + self.plot_height() + 15.0,
                label,
                &style.text_color.to_hex(), style.font_size * 0.9, "middle",
            );
        }
    }

    pub(crate) fn render_scatter(
        &mut self,
        x: &[f64],
        y: &[f64],
        color: &Color,
        radius: f64,
        opacity: f64,
        style: &ChartStyle,
    ) {
        if x.is_empty() {
            return;
        }
        let x_ticks = nice_ticks(
            x.iter().copied().fold(f64::INFINITY, f64::min),
            x.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            5,
        );
        let y_ticks = nice_ticks(
            y.iter().copied().fold(f64::INFINITY, f64::min),
            y.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            5,
        );
        self.draw_axes(&x_ticks.ticks, &y_ticks.ticks, &x_ticks, &y_ticks, style);

        for (&xi, &yi) in x.iter().zip(y.iter()) {
            let px = self.map_x(xi, x_ticks.min, x_ticks.max);
            let py = self.map_y(yi, y_ticks.min, y_ticks.max);
            self.circle(px, py, radius, &color.to_hex(), opacity);
        }
    }

    pub(crate) fn render_histogram(
        &mut self,
        data: &[f64],
        bins: usize,
        color: &Color,
        opacity: f64,
        style: &ChartStyle,
    ) {
        if data.is_empty() || bins == 0 {
            return;
        }

        let min_val = data.iter().copied().fold(f64::INFINITY, f64::min);
        let max_val = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let bin_width = (max_val - min_val) / bins as f64;

        if bin_width < 1e-15 {
            return;
        }

        let mut counts = vec![0usize; bins];
        for &val in data {
            let idx = ((val - min_val) / bin_width).floor() as usize;
            let idx = idx.min(bins - 1);
            counts[idx] += 1;
        }

        let max_count = *counts.iter().max().unwrap_or(&1);
        let y_ticks = nice_ticks(0.0, max_count as f64, 5);
        let x_ticks = nice_ticks(min_val, max_val, 5);

        self.draw_axes(&x_ticks.ticks, &y_ticks.ticks, &x_ticks, &y_ticks, style);

        let bar_w = self.plot_width() / bins as f64;
        for (i, &count) in counts.iter().enumerate() {
            let bx = self.margin.left + i as f64 * bar_w;
            let bar_h = count as f64 / (y_ticks.max - y_ticks.min) * self.plot_height();
            let by = self.margin.top + self.plot_height() - bar_h;
            self.rect(bx, by, bar_w - 1.0, bar_h, &color.to_hex(), opacity);
        }
    }

    pub(crate) fn render_box_plot(
        &mut self,
        groups: &[Vec<f64>],
        labels: &[String],
        color: &Color,
        style: &ChartStyle,
    ) {
        if groups.is_empty() {
            return;
        }

        let mut all_min = f64::INFINITY;
        let mut all_max = f64::NEG_INFINITY;
        let stats: Vec<_> = groups
            .iter()
            .map(|g| {
                let s = box_stats(g);
                all_min = all_min.min(s.whisker_low);
                all_max = all_max.max(s.whisker_high);
                s
            })
            .collect();

        let y_ticks = nice_ticks(all_min, all_max, 5);

        for &tick in &y_ticks.ticks {
            let py = self.map_y(tick, y_ticks.min, y_ticks.max);
            if style.show_grid {
                self.line(
                    self.margin.left, py,
                    self.margin.left + self.plot_width(), py,
                    &style.grid_color.to_hex(), 1.0,
                );
            }
            self.text(
                self.margin.left - 5.0, py + 4.0,
                &format_tick(tick), &style.text_color.to_hex(), style.font_size, "end",
            );
        }

        let group_w = self.plot_width() / groups.len() as f64;
        let box_w = group_w * 0.5;

        for (i, (s, label)) in stats.iter().zip(labels.iter()).enumerate() {
            let cx = self.margin.left + (i as f64 + 0.5) * group_w;
            let bx = cx - box_w / 2.0;

            let y_q1 = self.map_y(s.q1, y_ticks.min, y_ticks.max);
            let y_q3 = self.map_y(s.q3, y_ticks.min, y_ticks.max);
            let y_med = self.map_y(s.median, y_ticks.min, y_ticks.max);
            let y_wl = self.map_y(s.whisker_low, y_ticks.min, y_ticks.max);
            let y_wh = self.map_y(s.whisker_high, y_ticks.min, y_ticks.max);

            // Box fill
            self.rect(bx, y_q3, box_w, y_q1 - y_q3, &color.to_hex(), 0.3);
            // Box outline
            self.elements.push(format!(
                r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="none" stroke="{}" stroke-width="1.5" />"#,
                bx, y_q3, box_w, y_q1 - y_q3, color.to_hex()
            ));

            // Median line
            self.line(bx, y_med, bx + box_w, y_med, &color.to_hex(), 2.0);
            // Whiskers
            self.line(cx, y_q3, cx, y_wh, &color.to_hex(), 1.5);
            self.line(cx, y_q1, cx, y_wl, &color.to_hex(), 1.5);
            self.line(cx - box_w * 0.3, y_wh, cx + box_w * 0.3, y_wh, &color.to_hex(), 1.5);
            self.line(cx - box_w * 0.3, y_wl, cx + box_w * 0.3, y_wl, &color.to_hex(), 1.5);

            // Label
            self.text(
                cx, self.margin.top + self.plot_height() + 15.0,
                label, &style.text_color.to_hex(), style.font_size * 0.9, "middle",
            );
        }
    }

    pub(crate) fn render_heatmap(
        &mut self,
        data: &[Vec<f64>],
        x_labels: &Option<Vec<String>>,
        y_labels: &Option<Vec<String>>,
        style: &ChartStyle,
    ) {
        if data.is_empty() {
            return;
        }
        let n_rows = data.len();
        let n_cols = data[0].len();

        let mut vmin = f64::INFINITY;
        let mut vmax = f64::NEG_INFINITY;
        for row in data {
            for &v in row {
                vmin = vmin.min(v);
                vmax = vmax.max(v);
            }
        }

        let cell_w = self.plot_width() / n_cols as f64;
        let cell_h = self.plot_height() / n_rows as f64;

        for (i, row) in data.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                let t = if (vmax - vmin).abs() < 1e-15 {
                    0.5
                } else {
                    (val - vmin) / (vmax - vmin)
                };
                let color = viridis(t);
                let cx = self.margin.left + j as f64 * cell_w;
                let cy = self.margin.top + i as f64 * cell_h;
                self.rect(cx, cy, cell_w, cell_h, &color.to_hex(), 1.0);
            }
        }

        if let Some(labels) = x_labels {
            for (j, label) in labels.iter().enumerate() {
                self.text(
                    self.margin.left + (j as f64 + 0.5) * cell_w,
                    self.margin.top + self.plot_height() + 15.0,
                    label, &style.text_color.to_hex(), style.font_size * 0.8, "middle",
                );
            }
        }

        if let Some(labels) = y_labels {
            for (i, label) in labels.iter().enumerate() {
                self.text(
                    self.margin.left - 5.0,
                    self.margin.top + (i as f64 + 0.5) * cell_h + 4.0,
                    label, &style.text_color.to_hex(), style.font_size * 0.8, "end",
                );
            }
        }
    }

    pub(crate) fn render_area(
        &mut self,
        x: &[f64],
        y: &[f64],
        color: &Color,
        width: f64,
        style: &ChartStyle,
    ) {
        if x.is_empty() || y.is_empty() {
            return;
        }
        let x_ticks = nice_ticks(
            x.iter().copied().fold(f64::INFINITY, f64::min),
            x.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            5,
        );
        let y_min = y.iter().copied().fold(f64::INFINITY, f64::min).min(0.0);
        let y_ticks = nice_ticks(
            y_min,
            y.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            5,
        );

        self.draw_axes(&x_ticks.ticks, &y_ticks.ticks, &x_ticks, &y_ticks, style);

        let mut path = String::new();
        let baseline_y = self.map_y(y_ticks.min.max(0.0), y_ticks.min, y_ticks.max);
        let first_px = self.map_x(x[0], x_ticks.min, x_ticks.max);
        path.push_str(&format!("M{:.1},{:.1}", first_px, baseline_y));
        for (&xi, &yi) in x.iter().zip(y.iter()) {
            let px = self.map_x(xi, x_ticks.min, x_ticks.max);
            let py = self.map_y(yi, y_ticks.min, y_ticks.max);
            path.push_str(&format!(" L{:.1},{:.1}", px, py));
        }
        let last_px = self.map_x(x[x.len() - 1], x_ticks.min, x_ticks.max);
        path.push_str(&format!(" L{:.1},{:.1} Z", last_px, baseline_y));

        self.elements.push(format!(
            r#"<path d="{}" fill="{}" opacity="0.3" stroke="{}" stroke-width="{}" />"#,
            path,
            color.to_hex(),
            color.to_hex(),
            width
        ));
    }

    pub(crate) fn render_pie(
        &mut self,
        labels: &[String],
        values: &[f64],
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
        let r = (self.plot_width().min(self.plot_height()) / 2.0) * 0.8;
        let palette = crate::viz::style::default_palette();

        let mut start_angle: f64 = -std::f64::consts::FRAC_PI_2;
        for (i, &val) in values.iter().enumerate() {
            let sweep = 2.0 * std::f64::consts::PI * val / total;
            let end_angle = start_angle + sweep;

            let x1 = cx + r * start_angle.cos();
            let y1 = cy + r * start_angle.sin();
            let x2 = cx + r * end_angle.cos();
            let y2 = cy + r * end_angle.sin();
            let large_arc = if sweep > std::f64::consts::PI { 1 } else { 0 };
            let color = &palette[i % palette.len()];

            self.elements.push(format!(
                r##"<path d="M{:.1},{:.1} L{:.1},{:.1} A{:.1},{:.1} 0 {} 1 {:.1},{:.1} Z" fill="{}" stroke="#ffffff" stroke-width="1" />"##,
                cx, cy, x1, y1, r, r, large_arc, x2, y2, color.to_hex()
            ));

            let mid = start_angle + sweep / 2.0;
            let lx = cx + r * 0.65 * mid.cos();
            let ly = cy + r * 0.65 * mid.sin();
            if i < labels.len() {
                self.text(
                    lx, ly + 4.0,
                    &labels[i], &style.text_color.to_hex(),
                    style.font_size * 0.85, "middle",
                );
            }

            start_angle = end_angle;
        }
    }

    pub(crate) fn render_stacked_bar(
        &mut self,
        labels: &[String],
        datasets: &[(String, Vec<f64>)],
        style: &ChartStyle,
    ) {
        if labels.is_empty() || datasets.is_empty() {
            return;
        }

        // Compute max stack height
        let n = labels.len();
        let mut max_total = 0.0_f64;
        for i in 0..n {
            let total: f64 = datasets.iter().map(|(_, vals)| vals.get(i).copied().unwrap_or(0.0)).sum();
            max_total = max_total.max(total);
        }

        let y_ticks = nice_ticks(0.0, max_total, 5);

        // Draw y-axis ticks
        for &tick in &y_ticks.ticks {
            let py = self.map_y(tick, y_ticks.min, y_ticks.max);
            if style.show_grid {
                self.line(
                    self.margin.left, py,
                    self.margin.left + self.plot_width(), py,
                    &style.grid_color.to_hex(), 1.0,
                );
            }
            self.text(
                self.margin.left - 5.0, py + 4.0,
                &format_tick(tick),
                &style.text_color.to_hex(), style.font_size, "end",
            );
        }

        let palette = crate::viz::style::default_palette();
        let bar_width = self.plot_width() / n as f64 * 0.7;
        let gap = self.plot_width() / n as f64 * 0.15;

        for i in 0..n {
            let bx = self.margin.left + gap + i as f64 * (bar_width + 2.0 * gap);
            let mut cumulative = 0.0_f64;

            for (di, (_, vals)) in datasets.iter().enumerate() {
                let val = vals.get(i).copied().unwrap_or(0.0);
                if val <= 0.0 {
                    continue;
                }
                let y_bottom = cumulative;
                cumulative += val;
                let y_top = cumulative;

                let py_bottom = self.map_y(y_bottom, y_ticks.min, y_ticks.max);
                let py_top = self.map_y(y_top, y_ticks.min, y_ticks.max);
                let bar_h = py_bottom - py_top;

                let color = &palette[di % palette.len()];
                self.rect(bx, py_top, bar_width, bar_h, &color.to_hex(), 0.85);
            }

            // Label
            self.text(
                bx + bar_width / 2.0,
                self.margin.top + self.plot_height() + 15.0,
                &labels[i],
                &style.text_color.to_hex(), style.font_size * 0.9, "middle",
            );
        }
    }

    pub(crate) fn render_violin(
        &mut self,
        groups: &[Vec<f64>],
        labels: &[String],
        style: &ChartStyle,
    ) {
        if groups.is_empty() {
            return;
        }

        // Global y range
        let mut all_min = f64::INFINITY;
        let mut all_max = f64::NEG_INFINITY;
        for g in groups {
            for &v in g {
                all_min = all_min.min(v);
                all_max = all_max.max(v);
            }
        }
        if (all_max - all_min).abs() < 1e-15 {
            all_min -= 1.0;
            all_max += 1.0;
        }

        let y_ticks = nice_ticks(all_min, all_max, 5);

        // Draw y-axis ticks
        for &tick in &y_ticks.ticks {
            let py = self.map_y(tick, y_ticks.min, y_ticks.max);
            if style.show_grid {
                self.line(
                    self.margin.left, py,
                    self.margin.left + self.plot_width(), py,
                    &style.grid_color.to_hex(), 1.0,
                );
            }
            self.text(
                self.margin.left - 5.0, py + 4.0,
                &format_tick(tick), &style.text_color.to_hex(), style.font_size, "end",
            );
        }

        let palette = crate::viz::style::default_palette();
        let group_w = self.plot_width() / groups.len() as f64;
        let half_w = group_w * 0.4; // max half-width for the violin shape

        for (gi, group) in groups.iter().enumerate() {
            if group.is_empty() {
                continue;
            }
            let cx = self.margin.left + (gi as f64 + 0.5) * group_w;
            let color = &palette[gi % palette.len()];

            // Compute kernel density estimate at N evenly spaced points
            let density = kernel_density(group, y_ticks.min, y_ticks.max, 50);
            let max_density = density.iter().copied().fold(0.0_f64, f64::max);
            if max_density < 1e-15 {
                continue;
            }

            // Build mirrored path
            let n_pts = density.len();
            let mut right_points = Vec::with_capacity(n_pts);
            let mut left_points = Vec::with_capacity(n_pts);

            for (i, &d) in density.iter().enumerate() {
                let t = i as f64 / (n_pts - 1) as f64;
                let val = y_ticks.min + t * (y_ticks.max - y_ticks.min);
                let py = self.map_y(val, y_ticks.min, y_ticks.max);
                let dx = d / max_density * half_w;
                right_points.push((cx + dx, py));
                left_points.push((cx - dx, py));
            }

            // Build SVG path: right side top-to-bottom, then left side bottom-to-top
            let mut path = String::new();
            for (i, &(px, py)) in right_points.iter().enumerate() {
                if i == 0 {
                    path.push_str(&format!("M{:.1},{:.1}", px, py));
                } else {
                    path.push_str(&format!(" L{:.1},{:.1}", px, py));
                }
            }
            for &(px, py) in left_points.iter().rev() {
                path.push_str(&format!(" L{:.1},{:.1}", px, py));
            }
            path.push_str(" Z");

            self.elements.push(format!(
                r#"<path d="{}" fill="{}" opacity="0.6" stroke="{}" stroke-width="1" />"#,
                path,
                color.to_hex(),
                color.to_hex()
            ));

            // Label
            if gi < labels.len() {
                self.text(
                    cx, self.margin.top + self.plot_height() + 15.0,
                    &labels[gi], &style.text_color.to_hex(),
                    style.font_size * 0.9, "middle",
                );
            }
        }
    }
}

/// Gaussian kernel density estimation at evenly spaced points.
fn kernel_density(data: &[f64], min_val: f64, max_val: f64, n_points: usize) -> Vec<f64> {
    let n = data.len() as f64;
    if n < 1.0 {
        return vec![0.0; n_points];
    }

    // Silverman's rule of thumb for bandwidth
    let mean = data.iter().sum::<f64>() / n;
    let variance = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
    let std_dev = variance.sqrt().max(1e-10);
    let bandwidth = 1.06 * std_dev * n.powf(-0.2);

    let mut density = Vec::with_capacity(n_points);
    for i in 0..n_points {
        let t = i as f64 / (n_points - 1) as f64;
        let x = min_val + t * (max_val - min_val);

        let sum: f64 = data.iter().map(|&xi| {
            let u = (x - xi) / bandwidth;
            (-0.5 * u * u).exp() / (2.0 * std::f64::consts::PI).sqrt()
        }).sum();

        density.push(sum / (n * bandwidth));
    }
    density
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_density_basic() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let density = kernel_density(&data, 0.0, 6.0, 20);
        assert_eq!(density.len(), 20);
        // Density should be positive in range
        assert!(density.iter().any(|&d| d > 0.0));
    }

    #[test]
    fn test_kernel_density_empty() {
        let density = kernel_density(&[], 0.0, 10.0, 10);
        assert!(density.iter().all(|&d| d == 0.0));
    }

    #[test]
    fn test_kernel_density_peak() {
        let data = vec![5.0, 5.0, 5.0, 5.0, 5.0];
        let density = kernel_density(&data, 0.0, 10.0, 21);
        // Peak should be near the middle (at x=5)
        let mid_idx = 10;
        let peak_idx = density
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap()
            .0;
        assert!((peak_idx as i32 - mid_idx as i32).unsigned_abs() <= 1);
    }
}
