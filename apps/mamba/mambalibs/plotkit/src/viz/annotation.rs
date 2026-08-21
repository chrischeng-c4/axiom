//! Annotations and labels for charts.
//!
//! Provides text annotations, arrows, reference lines, shapes,
//! and data labels that can be overlaid on any chart.

use super::style::Color;

/// An annotation to overlay on a chart.
#[derive(Debug, Clone)]
pub enum Annotation {
    /// Text label at a specific position.
    Text(TextAnnotation),
    /// Arrow from one point to another.
    Arrow(ArrowAnnotation),
    /// Horizontal or vertical reference line.
    ReferenceLine(ReferenceLineAnnotation),
    /// Rectangle shape overlay.
    Rect(RectAnnotation),
    /// Circle shape overlay.
    Circle(CircleAnnotation),
}

/// Text annotation at a data or pixel position.
#[derive(Debug, Clone)]
pub struct TextAnnotation {
    pub x: f64,
    pub y: f64,
    pub text: String,
    pub font_size: f64,
    pub color: Color,
    pub anchor: TextAnchor,
    /// If true, x/y are pixel coordinates; otherwise data coordinates.
    pub pixel_coords: bool,
    /// Optional rotation in degrees.
    pub rotation: Option<f64>,
}

impl TextAnnotation {
    /// Create a text annotation at data coordinates.
    pub fn new(x: f64, y: f64, text: &str) -> Self {
        Self {
            x,
            y,
            text: text.to_string(),
            font_size: 12.0,
            color: Color::new(51, 51, 51),
            anchor: TextAnchor::Start,
            pixel_coords: false,
            rotation: None,
        }
    }

    pub fn font_size(mut self, size: f64) -> Self {
        self.font_size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn anchor(mut self, anchor: TextAnchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn rotation(mut self, degrees: f64) -> Self {
        self.rotation = Some(degrees);
        self
    }

    pub fn pixel_coords(mut self) -> Self {
        self.pixel_coords = true;
        self
    }
}

/// Text anchor position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAnchor {
    Start,
    Middle,
    End,
}

impl TextAnchor {
    pub fn as_str(&self) -> &'static str {
        match self {
            TextAnchor::Start => "start",
            TextAnchor::Middle => "middle",
            TextAnchor::End => "end",
        }
    }
}

/// Arrow annotation from one point to another.
#[derive(Debug, Clone)]
pub struct ArrowAnnotation {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub color: Color,
    pub width: f64,
    pub head_size: f64,
    /// Optional text label near the arrow head.
    pub label: Option<String>,
}

impl ArrowAnnotation {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self {
            x1,
            y1,
            x2,
            y2,
            color: Color::new(51, 51, 51),
            width: 1.5,
            head_size: 8.0,
            label: None,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn width(mut self, w: f64) -> Self {
        self.width = w;
        self
    }

    pub fn label(mut self, text: &str) -> Self {
        self.label = Some(text.to_string());
        self
    }
}

/// Horizontal or vertical reference line.
#[derive(Debug, Clone)]
pub struct ReferenceLineAnnotation {
    pub direction: RefLineDirection,
    pub value: f64,
    pub color: Color,
    pub width: f64,
    pub dash: Option<String>,
    pub label: Option<String>,
}

/// Direction of a reference line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefLineDirection {
    Horizontal,
    Vertical,
}

impl ReferenceLineAnnotation {
    pub fn horizontal(y_value: f64) -> Self {
        Self {
            direction: RefLineDirection::Horizontal,
            value: y_value,
            color: Color::new(214, 39, 40),
            width: 1.5,
            dash: Some("5,3".to_string()),
            label: None,
        }
    }

    pub fn vertical(x_value: f64) -> Self {
        Self {
            direction: RefLineDirection::Vertical,
            value: x_value,
            color: Color::new(214, 39, 40),
            width: 1.5,
            dash: Some("5,3".to_string()),
            label: None,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn width(mut self, w: f64) -> Self {
        self.width = w;
        self
    }

    pub fn solid(mut self) -> Self {
        self.dash = None;
        self
    }

    pub fn label(mut self, text: &str) -> Self {
        self.label = Some(text.to_string());
        self
    }
}

/// Rectangle shape annotation.
#[derive(Debug, Clone)]
pub struct RectAnnotation {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub fill: Color,
    pub opacity: f64,
    pub stroke: Option<Color>,
}

impl RectAnnotation {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            fill: Color::new(200, 200, 200),
            opacity: 0.3,
            stroke: None,
        }
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill = color;
        self
    }

    pub fn opacity(mut self, o: f64) -> Self {
        self.opacity = o;
        self
    }

    pub fn stroke(mut self, color: Color) -> Self {
        self.stroke = Some(color);
        self
    }
}

/// Circle shape annotation.
#[derive(Debug, Clone)]
pub struct CircleAnnotation {
    pub cx: f64,
    pub cy: f64,
    pub radius: f64,
    pub fill: Color,
    pub opacity: f64,
    pub stroke: Option<Color>,
}

impl CircleAnnotation {
    pub fn new(cx: f64, cy: f64, radius: f64) -> Self {
        Self {
            cx,
            cy,
            radius,
            fill: Color::new(200, 200, 200),
            opacity: 0.3,
            stroke: None,
        }
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill = color;
        self
    }

    pub fn opacity(mut self, o: f64) -> Self {
        self.opacity = o;
        self
    }

    pub fn stroke(mut self, color: Color) -> Self {
        self.stroke = Some(color);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_annotation_builder() {
        let ann = TextAnnotation::new(1.0, 2.0, "hello")
            .font_size(14.0)
            .color(Color::new(255, 0, 0))
            .anchor(TextAnchor::Middle)
            .rotation(45.0);
        assert_eq!(ann.text, "hello");
        assert_eq!(ann.font_size, 14.0);
        assert_eq!(ann.rotation, Some(45.0));
        assert_eq!(ann.anchor, TextAnchor::Middle);
    }

    #[test]
    fn test_arrow_annotation_builder() {
        let arr = ArrowAnnotation::new(0.0, 0.0, 10.0, 10.0)
            .color(Color::new(0, 0, 255))
            .width(2.0)
            .label("important");
        assert_eq!(arr.label, Some("important".into()));
        assert_eq!(arr.width, 2.0);
    }

    #[test]
    fn test_reference_line_horizontal() {
        let line = ReferenceLineAnnotation::horizontal(50.0)
            .color(Color::new(0, 128, 0))
            .solid()
            .label("threshold");
        assert_eq!(line.direction, RefLineDirection::Horizontal);
        assert_eq!(line.value, 50.0);
        assert!(line.dash.is_none());
        assert_eq!(line.label, Some("threshold".into()));
    }

    #[test]
    fn test_reference_line_vertical() {
        let line = ReferenceLineAnnotation::vertical(3.0).width(2.0);
        assert_eq!(line.direction, RefLineDirection::Vertical);
        assert_eq!(line.value, 3.0);
        assert!(line.dash.is_some());
    }

    #[test]
    fn test_rect_annotation_builder() {
        let r = RectAnnotation::new(1.0, 2.0, 10.0, 20.0)
            .fill(Color::new(255, 0, 0))
            .opacity(0.5)
            .stroke(Color::new(0, 0, 0));
        assert_eq!(r.opacity, 0.5);
        assert!(r.stroke.is_some());
    }

    #[test]
    fn test_circle_annotation_builder() {
        let c = CircleAnnotation::new(5.0, 5.0, 10.0)
            .fill(Color::new(0, 255, 0))
            .opacity(0.4)
            .stroke(Color::new(0, 0, 0));
        assert_eq!(c.radius, 10.0);
        assert_eq!(c.opacity, 0.4);
    }

    #[test]
    fn test_annotation_enum_variants() {
        let text = Annotation::Text(TextAnnotation::new(0.0, 0.0, "test"));
        let arrow = Annotation::Arrow(ArrowAnnotation::new(0.0, 0.0, 1.0, 1.0));
        let hline = Annotation::ReferenceLine(ReferenceLineAnnotation::horizontal(5.0));
        let rect = Annotation::Rect(RectAnnotation::new(0.0, 0.0, 10.0, 10.0));
        let circ = Annotation::Circle(CircleAnnotation::new(5.0, 5.0, 3.0));
        // Just ensure they can be created and matched
        assert!(matches!(text, Annotation::Text(_)));
        assert!(matches!(arrow, Annotation::Arrow(_)));
        assert!(matches!(hline, Annotation::ReferenceLine(_)));
        assert!(matches!(rect, Annotation::Rect(_)));
        assert!(matches!(circ, Annotation::Circle(_)));
    }

    #[test]
    fn test_text_anchor_as_str() {
        assert_eq!(TextAnchor::Start.as_str(), "start");
        assert_eq!(TextAnchor::Middle.as_str(), "middle");
        assert_eq!(TextAnchor::End.as_str(), "end");
    }

    #[test]
    fn test_text_pixel_coords() {
        let ann = TextAnnotation::new(100.0, 200.0, "px").pixel_coords();
        assert!(ann.pixel_coords);
    }
}
