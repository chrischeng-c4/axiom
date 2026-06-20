//! PDF types — document structure, pages, content elements.

/// Page size in points (1 point = 1/72 inch).
#[derive(Debug, Clone, Copy)]
pub struct PageSize {
    pub width: f64,
    pub height: f64,
}

impl PageSize {
    /// US Letter (8.5 x 11 inches).
    pub const LETTER: Self = Self {
        width: 612.0,
        height: 792.0,
    };
    /// A4 (210 x 297 mm).
    pub const A4: Self = Self {
        width: 595.28,
        height: 841.89,
    };
    /// A3 (297 x 420 mm).
    pub const A3: Self = Self {
        width: 841.89,
        height: 1190.55,
    };
    /// Custom size.
    pub fn custom(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}

/// Page margins in points.
#[derive(Debug, Clone, Copy)]
pub struct Margins {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Margins {
    /// Uniform margins.
    pub fn uniform(m: f64) -> Self {
        Self {
            top: m,
            right: m,
            bottom: m,
            left: m,
        }
    }

    /// Default 1-inch margins.
    pub fn default_margins() -> Self {
        Self::uniform(72.0)
    }
}

impl Default for Margins {
    fn default() -> Self {
        Self::default_margins()
    }
}

/// Text alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// Font selection (PDF built-in base 14 fonts).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Font {
    Helvetica,
    HelveticaBold,
    HelveticaItalic,
    TimesRoman,
    TimesBold,
    TimesItalic,
    Courier,
    CourierBold,
}

impl Font {
    /// PDF font name string.
    pub fn pdf_name(&self) -> &'static str {
        match self {
            Font::Helvetica => "Helvetica",
            Font::HelveticaBold => "Helvetica-Bold",
            Font::HelveticaItalic => "Helvetica-Oblique",
            Font::TimesRoman => "Times-Roman",
            Font::TimesBold => "Times-Bold",
            Font::TimesItalic => "Times-Italic",
            Font::Courier => "Courier",
            Font::CourierBold => "Courier-Bold",
        }
    }

    /// Approximate character width at size 1 (for layout).
    pub fn avg_char_width(&self) -> f64 {
        match self {
            Font::Courier | Font::CourierBold => 0.6,
            _ => 0.52,
        }
    }
}

/// RGB color (0.0 - 1.0).
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
    };
    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
    };
    pub const GRAY: Self = Self {
        r: 0.5,
        g: 0.5,
        b: 0.5,
    };

    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }
}

/// Text styling.
#[derive(Debug, Clone)]
pub struct TextStyle {
    pub font: Font,
    pub size: f64,
    pub color: Color,
    pub align: TextAlign,
    pub line_height: f64,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font: Font::Helvetica,
            size: 12.0,
            color: Color::BLACK,
            align: TextAlign::Left,
            line_height: 1.4,
        }
    }
}

/// A content element that can be placed on a page.
#[derive(Debug, Clone)]
pub enum ContentElement {
    /// A paragraph of text.
    Text { content: String, style: TextStyle },
    /// A heading (bold, larger).
    Heading {
        content: String,
        level: u8,
        style: TextStyle,
    },
    /// A horizontal rule / separator line.
    HorizontalRule { thickness: f64, color: Color },
    /// Raw RGB image data.
    Image {
        data: Vec<u8>,
        width: u32,
        height: u32,
        display_width: f64,
        display_height: f64,
    },
    /// Vertical spacing.
    Spacer { height: f64 },
}

/// A PDF page with its content elements.
#[derive(Debug, Clone)]
pub struct Page {
    pub size: PageSize,
    pub margins: Margins,
    pub elements: Vec<ContentElement>,
}

impl Page {
    /// Create a new page with defaults.
    pub fn new() -> Self {
        Self {
            size: PageSize::A4,
            margins: Margins::default(),
            elements: Vec::new(),
        }
    }

    /// Set page size.
    pub fn with_size(mut self, size: PageSize) -> Self {
        self.size = size;
        self
    }

    /// Set margins.
    pub fn with_margins(mut self, margins: Margins) -> Self {
        self.margins = margins;
        self
    }

    /// Add a content element.
    pub fn add(&mut self, elem: ContentElement) {
        self.elements.push(elem);
    }

    /// Usable content width.
    pub fn content_width(&self) -> f64 {
        self.size.width - self.margins.left - self.margins.right
    }

    /// Usable content height.
    pub fn content_height(&self) -> f64 {
        self.size.height - self.margins.top - self.margins.bottom
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::new()
    }
}

/// A complete PDF document.
#[derive(Debug, Clone)]
pub struct Document {
    pub pages: Vec<Page>,
    pub title: Option<String>,
    pub author: Option<String>,
}

impl Document {
    /// Create an empty document.
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            title: None,
            author: None,
        }
    }

    /// Add a page.
    pub fn add_page(&mut self, page: Page) {
        self.pages.push(page);
    }

    /// Set document title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set document author.
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_content_area() {
        let page = Page::new().with_size(PageSize::LETTER);
        let cw = page.content_width();
        let ch = page.content_height();
        assert!(cw > 0.0 && cw < 612.0);
        assert!(ch > 0.0 && ch < 792.0);
    }

    #[test]
    fn test_document_builder() {
        let mut doc = Document::new()
            .with_title("Test")
            .with_author("Test Author");
        let mut page = Page::new();
        page.add(ContentElement::Text {
            content: "Hello".into(),
            style: TextStyle::default(),
        });
        doc.add_page(page);
        assert_eq!(doc.pages.len(), 1);
        assert_eq!(doc.title.as_deref(), Some("Test"));
    }
}
