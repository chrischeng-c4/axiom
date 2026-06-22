//! Legend rendering for SVG charts.
//!
//! Draws a legend box when multiple labeled series are present,
//! showing a colored rectangle + label text for each series.

use crate::viz::series_data::DataSeries;
use crate::viz::style::{ChartStyle, Color};

use super::svg::SvgRenderer;

/// Collected legend entry.
#[derive(Debug)]
struct LegendEntry {
    label: String,
    color: Color,
}

impl SvgRenderer {
    /// Render a legend if multiple labeled series are present.
    pub(crate) fn render_legend(
        &mut self,
        series: &[DataSeries],
        style: &ChartStyle,
        palette: &[Color],
    ) {
        let entries = collect_legend_entries(series, palette);
        if entries.len() < 2 {
            return; // no legend needed for 0-1 entries
        }

        let box_size = 12.0;
        let padding = 8.0;
        let line_height = style.font_size + 4.0;
        let text_offset = box_size + 6.0;

        // Estimate max label width (approximate: 7px per char)
        let max_label_width = entries
            .iter()
            .map(|e| e.label.len() as f64 * 7.0)
            .fold(0.0_f64, f64::max);

        let legend_w = text_offset + max_label_width + padding * 2.0;
        let legend_h = padding * 2.0 + entries.len() as f64 * line_height;

        // Position: top-right inside the plot area
        let lx = self.margin.left + self.plot_width() - legend_w - 10.0;
        let ly = self.margin.top + 10.0;

        // Background
        self.elements.push(format!(
            r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="{}" opacity="0.9" stroke="{}" stroke-width="0.5" rx="3" />"#,
            lx, ly, legend_w, legend_h,
            style.background.to_hex(),
            style.grid_color.to_hex()
        ));

        for (i, entry) in entries.iter().enumerate() {
            let ey = ly + padding + i as f64 * line_height;

            // Color box
            self.elements.push(format!(
                r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="{}" rx="2" />"#,
                lx + padding,
                ey,
                box_size,
                box_size,
                entry.color.to_hex()
            ));

            // Label text
            self.text(
                lx + padding + text_offset,
                ey + box_size - 2.0,
                &entry.label,
                &style.text_color.to_hex(),
                style.font_size * 0.9,
                "start",
            );
        }
    }
}

/// Extract legend entries from series, using the series label and color.
fn collect_legend_entries(series: &[DataSeries], palette: &[Color]) -> Vec<LegendEntry> {
    let mut entries = Vec::new();
    for (i, s) in series.iter().enumerate() {
        let fallback_color = if palette.is_empty() {
            Color::new(0, 0, 0)
        } else {
            palette[i % palette.len()]
        };

        match s {
            DataSeries::Line {
                label: Some(l),
                style: ls,
                ..
            } => {
                entries.push(LegendEntry {
                    label: l.clone(),
                    color: ls.color,
                });
            }
            DataSeries::Bar {
                label: Some(l),
                style: bs,
                ..
            } => {
                entries.push(LegendEntry {
                    label: l.clone(),
                    color: bs.color,
                });
            }
            DataSeries::Scatter {
                label: Some(l),
                style: ps,
                ..
            } => {
                entries.push(LegendEntry {
                    label: l.clone(),
                    color: ps.color,
                });
            }
            DataSeries::Histogram {
                label: Some(l),
                style: bs,
                ..
            } => {
                entries.push(LegendEntry {
                    label: l.clone(),
                    color: bs.color,
                });
            }
            DataSeries::Area {
                label: Some(l),
                style: ls,
                ..
            } => {
                entries.push(LegendEntry {
                    label: l.clone(),
                    color: ls.color,
                });
            }
            DataSeries::StackedBar { datasets, .. } => {
                for (di, (name, _)) in datasets.iter().enumerate() {
                    let color = if palette.is_empty() {
                        Color::new(0, 0, 0)
                    } else {
                        palette[di % palette.len()]
                    };
                    entries.push(LegendEntry {
                        label: name.clone(),
                        color,
                    });
                }
            }
            DataSeries::Violin { labels, .. } => {
                for (vi, lbl) in labels.iter().enumerate() {
                    let color = if palette.is_empty() {
                        Color::new(0, 0, 0)
                    } else {
                        palette[vi % palette.len()]
                    };
                    entries.push(LegendEntry {
                        label: lbl.clone(),
                        color,
                    });
                }
            }
            DataSeries::Polar { datasets, .. } => {
                for (di, (name, _)) in datasets.iter().enumerate() {
                    let color = if palette.is_empty() {
                        Color::new(0, 0, 0)
                    } else {
                        palette[di % palette.len()]
                    };
                    entries.push(LegendEntry {
                        label: name.clone(),
                        color,
                    });
                }
            }
            DataSeries::Donut { labels: dl, .. } => {
                for (di, lbl) in dl.iter().enumerate() {
                    let color = if palette.is_empty() {
                        Color::new(0, 0, 0)
                    } else {
                        palette[di % palette.len()]
                    };
                    entries.push(LegendEntry {
                        label: lbl.clone(),
                        color,
                    });
                }
            }
            _ => {
                // No label or unsupported type -- use fallback if there's a label
                let lbl = match s {
                    DataSeries::Line { label, .. } => label.clone(),
                    DataSeries::Bar { label, .. } => label.clone(),
                    DataSeries::Scatter { label, .. } => label.clone(),
                    DataSeries::Histogram { label, .. } => label.clone(),
                    DataSeries::Area { label, .. } => label.clone(),
                    _ => None,
                };
                if let Some(l) = lbl {
                    entries.push(LegendEntry {
                        label: l,
                        color: fallback_color,
                    });
                }
            }
        }
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::viz::style::{BLUE, GREEN, ORANGE};

    #[test]
    fn test_collect_no_labels() {
        let series = vec![
            DataSeries::line(vec![1.0], vec![2.0]),
            DataSeries::scatter(vec![3.0], vec![4.0]),
        ];
        let entries = collect_legend_entries(&series, &[BLUE, ORANGE]);
        assert!(entries.is_empty());
    }

    #[test]
    fn test_collect_with_labels() {
        let series = vec![
            DataSeries::line(vec![1.0], vec![2.0]).with_label("Line A"),
            DataSeries::scatter(vec![3.0], vec![4.0]).with_label("Scatter B"),
        ];
        let entries = collect_legend_entries(&series, &[BLUE, ORANGE]);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].label, "Line A");
        assert_eq!(entries[1].label, "Scatter B");
    }

    #[test]
    fn test_collect_stacked_bar_entries() {
        let series = vec![DataSeries::stacked_bar(
            vec!["Q1".into(), "Q2".into()],
            vec![
                ("Product A".into(), vec![10.0, 20.0]),
                ("Product B".into(), vec![15.0, 25.0]),
            ],
        )];
        let entries = collect_legend_entries(&series, &[BLUE, ORANGE, GREEN]);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].label, "Product A");
        assert_eq!(entries[1].label, "Product B");
    }

    #[test]
    fn test_collect_violin_entries() {
        let series = vec![DataSeries::violin(
            vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            vec!["Group A".into(), "Group B".into()],
        )];
        let entries = collect_legend_entries(&series, &[BLUE, ORANGE]);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].label, "Group A");
    }

    #[test]
    fn test_render_legend_skips_single() {
        let style = ChartStyle::default();
        let mut r = SvgRenderer::new(400.0, 300.0);
        let series = vec![DataSeries::line(vec![1.0], vec![2.0]).with_label("Only one")];
        let before = r.elements.len();
        r.render_legend(&series, &style, &[BLUE]);
        // Should not add legend elements for a single entry
        assert_eq!(r.elements.len(), before);
    }

    #[test]
    fn test_render_legend_draws_for_multiple() {
        let style = ChartStyle::default();
        let mut r = SvgRenderer::new(400.0, 300.0);
        let series = vec![
            DataSeries::line(vec![1.0], vec![2.0]).with_label("Series A"),
            DataSeries::line(vec![1.0], vec![3.0]).with_label("Series B"),
        ];
        r.render_legend(&series, &style, &[BLUE, ORANGE]);
        // Should have background rect + 2 color boxes + 2 text labels = 5
        assert!(r.elements.len() >= 5);
        let svg = r.build_svg();
        assert!(svg.contains("Series A"));
        assert!(svg.contains("Series B"));
    }
}
