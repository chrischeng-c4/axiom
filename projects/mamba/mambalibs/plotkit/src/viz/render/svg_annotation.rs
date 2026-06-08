//! SVG rendering of annotations.

use crate::viz::annotation::*;
use crate::viz::axis::TickInfo;
use crate::viz::style::ChartStyle;

use super::svg::{escape_xml, SvgRenderer};

impl SvgRenderer {
    /// Render all annotations onto the chart.
    ///
    /// `x_info` and `y_info` are needed to map data coordinates to pixels.
    pub(crate) fn render_annotations(
        &mut self,
        annotations: &[Annotation],
        x_info: Option<&TickInfo>,
        y_info: Option<&TickInfo>,
        style: &ChartStyle,
    ) {
        for ann in annotations {
            match ann {
                Annotation::Text(t) => self.render_text_annotation(t, x_info, y_info),
                Annotation::Arrow(a) => self.render_arrow_annotation(a, x_info, y_info, style),
                Annotation::ReferenceLine(r) => {
                    self.render_reference_line(r, x_info, y_info, style)
                }
                Annotation::Rect(r) => self.render_rect_annotation(r, x_info, y_info),
                Annotation::Circle(c) => self.render_circle_annotation(c, x_info, y_info),
            }
        }
    }

    fn resolve_x(&self, val: f64, pixel: bool, info: Option<&TickInfo>) -> f64 {
        if pixel {
            val
        } else if let Some(info) = info {
            self.map_x(val, info.min, info.max)
        } else {
            val
        }
    }

    fn resolve_y(&self, val: f64, pixel: bool, info: Option<&TickInfo>) -> f64 {
        if pixel {
            val
        } else if let Some(info) = info {
            self.map_y(val, info.min, info.max)
        } else {
            val
        }
    }

    fn render_text_annotation(
        &mut self,
        t: &TextAnnotation,
        x_info: Option<&TickInfo>,
        y_info: Option<&TickInfo>,
    ) {
        let px = self.resolve_x(t.x, t.pixel_coords, x_info);
        let py = self.resolve_y(t.y, t.pixel_coords, y_info);

        let rotation_attr = match t.rotation {
            Some(deg) => format!(r#" transform="rotate({:.1},{:.1},{:.1})""#, deg, px, py),
            None => String::new(),
        };

        self.elements.push(format!(
            r#"<text x="{:.1}" y="{:.1}" fill="{}" font-size="{:.1}" text-anchor="{}" font-family="sans-serif"{}>{}</text>"#,
            px, py,
            t.color.to_hex(),
            t.font_size,
            t.anchor.as_str(),
            rotation_attr,
            escape_xml(&t.text)
        ));
    }

    fn render_arrow_annotation(
        &mut self,
        a: &ArrowAnnotation,
        x_info: Option<&TickInfo>,
        y_info: Option<&TickInfo>,
        style: &ChartStyle,
    ) {
        let px1 = self.resolve_x(a.x1, false, x_info);
        let py1 = self.resolve_y(a.y1, false, y_info);
        let px2 = self.resolve_x(a.x2, false, x_info);
        let py2 = self.resolve_y(a.y2, false, y_info);

        // Shaft line
        self.elements.push(format!(
            r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{}" stroke-width="{}" />"#,
            px1, py1, px2, py2, a.color.to_hex(), a.width
        ));

        // Arrowhead (triangle)
        let dx = px2 - px1;
        let dy = py2 - py1;
        let len = (dx * dx + dy * dy).sqrt().max(1e-10);
        let ux = dx / len;
        let uy = dy / len;
        let hs = a.head_size;

        let tip_x = px2;
        let tip_y = py2;
        let left_x = px2 - ux * hs + uy * hs * 0.4;
        let left_y = py2 - uy * hs - ux * hs * 0.4;
        let right_x = px2 - ux * hs - uy * hs * 0.4;
        let right_y = py2 - uy * hs + ux * hs * 0.4;

        self.elements.push(format!(
            r#"<polygon points="{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}" fill="{}" />"#,
            tip_x, tip_y, left_x, left_y, right_x, right_y, a.color.to_hex()
        ));

        // Label near arrowhead
        if let Some(label) = &a.label {
            self.text(
                px2 + 5.0,
                py2 - 5.0,
                label,
                &style.text_color.to_hex(),
                style.font_size * 0.85,
                "start",
            );
        }
    }

    fn render_reference_line(
        &mut self,
        r: &ReferenceLineAnnotation,
        x_info: Option<&TickInfo>,
        y_info: Option<&TickInfo>,
        style: &ChartStyle,
    ) {
        let dash_attr = match &r.dash {
            Some(d) => format!(r#" stroke-dasharray="{}""#, d),
            None => String::new(),
        };

        match r.direction {
            RefLineDirection::Horizontal => {
                let py = if let Some(info) = y_info {
                    self.map_y(r.value, info.min, info.max)
                } else {
                    r.value
                };
                self.elements.push(format!(
                    r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{}" stroke-width="{}"{} />"#,
                    self.margin.left, py,
                    self.margin.left + self.plot_width(), py,
                    r.color.to_hex(), r.width, dash_attr
                ));
                if let Some(label) = &r.label {
                    self.text(
                        self.margin.left + self.plot_width() + 5.0,
                        py + 4.0,
                        label,
                        &style.text_color.to_hex(),
                        style.font_size * 0.85,
                        "start",
                    );
                }
            }
            RefLineDirection::Vertical => {
                let px = if let Some(info) = x_info {
                    self.map_x(r.value, info.min, info.max)
                } else {
                    r.value
                };
                self.elements.push(format!(
                    r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{}" stroke-width="{}"{} />"#,
                    px, self.margin.top,
                    px, self.margin.top + self.plot_height(),
                    r.color.to_hex(), r.width, dash_attr
                ));
                if let Some(label) = &r.label {
                    self.text(
                        px,
                        self.margin.top - 5.0,
                        label,
                        &style.text_color.to_hex(),
                        style.font_size * 0.85,
                        "middle",
                    );
                }
            }
        }
    }

    fn render_rect_annotation(
        &mut self,
        r: &RectAnnotation,
        x_info: Option<&TickInfo>,
        y_info: Option<&TickInfo>,
    ) {
        let px = self.resolve_x(r.x, false, x_info);
        let py = self.resolve_y(r.y, false, y_info);
        let px2 = self.resolve_x(r.x + r.width, false, x_info);
        let py2 = self.resolve_y(r.y - r.height, false, y_info);
        let w = (px2 - px).abs();
        let h = (py2 - py).abs();
        let sx = px.min(px2);
        let sy = py.min(py2);

        let stroke_attr = match &r.stroke {
            Some(c) => format!(r#" stroke="{}" stroke-width="1""#, c.to_hex()),
            None => String::new(),
        };

        self.elements.push(format!(
            r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="{}" opacity="{:.2}"{}  />"#,
            sx, sy, w, h, r.fill.to_hex(), r.opacity, stroke_attr
        ));
    }

    fn render_circle_annotation(
        &mut self,
        c: &CircleAnnotation,
        x_info: Option<&TickInfo>,
        y_info: Option<&TickInfo>,
    ) {
        let px = self.resolve_x(c.cx, false, x_info);
        let py = self.resolve_y(c.cy, false, y_info);

        let stroke_attr = match &c.stroke {
            Some(col) => format!(r#" stroke="{}" stroke-width="1""#, col.to_hex()),
            None => String::new(),
        };

        self.elements.push(format!(
            r#"<circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="{}" opacity="{:.2}"{} />"#,
            px, py, c.radius, c.fill.to_hex(), c.opacity, stroke_attr
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::viz::axis::nice_ticks;
    use crate::viz::style::{ChartStyle, Color};

    #[test]
    fn test_render_text_annotation() {
        let style = ChartStyle::default();
        let mut r = SvgRenderer::new(400.0, 300.0);
        let x_info = nice_ticks(0.0, 10.0, 5);
        let y_info = nice_ticks(0.0, 100.0, 5);
        let ann = vec![Annotation::Text(
            TextAnnotation::new(5.0, 50.0, "Hello!")
                .font_size(14.0)
                .anchor(TextAnchor::Middle),
        )];
        r.render_annotations(&ann, Some(&x_info), Some(&y_info), &style);
        let svg = r.build_svg();
        assert!(svg.contains("Hello!"));
    }

    #[test]
    fn test_render_text_annotation_rotated() {
        let style = ChartStyle::default();
        let mut r = SvgRenderer::new(400.0, 300.0);
        let ann = vec![Annotation::Text(
            TextAnnotation::new(100.0, 200.0, "Rotated")
                .pixel_coords()
                .rotation(45.0),
        )];
        r.render_annotations(&ann, None, None, &style);
        let svg = r.build_svg();
        assert!(svg.contains("rotate("));
        assert!(svg.contains("Rotated"));
    }

    #[test]
    fn test_render_arrow_annotation() {
        let style = ChartStyle::default();
        let mut r = SvgRenderer::new(400.0, 300.0);
        let x_info = nice_ticks(0.0, 10.0, 5);
        let y_info = nice_ticks(0.0, 100.0, 5);
        let ann = vec![Annotation::Arrow(
            ArrowAnnotation::new(2.0, 30.0, 8.0, 70.0).label("peak"),
        )];
        r.render_annotations(&ann, Some(&x_info), Some(&y_info), &style);
        let svg = r.build_svg();
        assert!(svg.contains("<line"));
        assert!(svg.contains("<polygon"));
        assert!(svg.contains("peak"));
    }

    #[test]
    fn test_render_reference_line_horizontal() {
        let style = ChartStyle::default();
        let mut r = SvgRenderer::new(400.0, 300.0);
        let y_info = nice_ticks(0.0, 100.0, 5);
        let ann = vec![Annotation::ReferenceLine(
            ReferenceLineAnnotation::horizontal(50.0).label("avg"),
        )];
        r.render_annotations(&ann, None, Some(&y_info), &style);
        let svg = r.build_svg();
        assert!(svg.contains("stroke-dasharray"));
        assert!(svg.contains("avg"));
    }

    #[test]
    fn test_render_reference_line_vertical() {
        let style = ChartStyle::default();
        let mut r = SvgRenderer::new(400.0, 300.0);
        let x_info = nice_ticks(0.0, 10.0, 5);
        let ann = vec![Annotation::ReferenceLine(
            ReferenceLineAnnotation::vertical(5.0).solid(),
        )];
        r.render_annotations(&ann, Some(&x_info), None, &style);
        let svg = r.build_svg();
        assert!(svg.contains("<line"));
        // Solid line should NOT have dasharray
        assert!(!svg.contains("stroke-dasharray"));
    }

    #[test]
    fn test_render_rect_annotation() {
        let style = ChartStyle::default();
        let mut r = SvgRenderer::new(400.0, 300.0);
        let x_info = nice_ticks(0.0, 10.0, 5);
        let y_info = nice_ticks(0.0, 100.0, 5);
        let ann = vec![Annotation::Rect(
            RectAnnotation::new(2.0, 80.0, 4.0, 40.0)
                .fill(Color::new(255, 255, 0))
                .opacity(0.2),
        )];
        r.render_annotations(&ann, Some(&x_info), Some(&y_info), &style);
        let svg = r.build_svg();
        assert!(svg.contains("<rect"));
        assert!(svg.contains("opacity=\"0.20\""));
    }

    #[test]
    fn test_render_circle_annotation() {
        let style = ChartStyle::default();
        let mut r = SvgRenderer::new(400.0, 300.0);
        let x_info = nice_ticks(0.0, 10.0, 5);
        let y_info = nice_ticks(0.0, 100.0, 5);
        let ann = vec![Annotation::Circle(
            CircleAnnotation::new(5.0, 50.0, 20.0)
                .fill(Color::new(0, 0, 255))
                .opacity(0.3)
                .stroke(Color::new(0, 0, 0)),
        )];
        r.render_annotations(&ann, Some(&x_info), Some(&y_info), &style);
        let svg = r.build_svg();
        assert!(svg.contains("<circle"));
        assert!(svg.contains("stroke="));
    }

    #[test]
    fn test_render_empty_annotations() {
        let style = ChartStyle::default();
        let mut r = SvgRenderer::new(400.0, 300.0);
        r.render_annotations(&[], None, None, &style);
        assert!(r.elements.is_empty());
    }
}
