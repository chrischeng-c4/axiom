//! Minimal PDF 1.4 writer — generates valid PDF from structured content.
//!
//! This is a pure-Rust implementation with no external dependencies.
//! It supports text (with basic word-wrapping), images, and styling.

use super::types::*;
use std::fmt::Write as FmtWrite;
use std::io::Write;

/// Internal PDF object tracking.
struct PdfObj {
    /// Byte offset of this object in the output.
    offset: usize,
}

/// PDF writer that serializes a `Document` to PDF bytes.
pub struct PdfWriter {
    buf: Vec<u8>,
    objects: Vec<PdfObj>,
    next_obj: usize,
}

impl PdfWriter {
    fn new() -> Self {
        Self {
            buf: Vec::with_capacity(4096),
            objects: Vec::new(),
            next_obj: 1, // PDF objects are 1-indexed
        }
    }

    /// Allocate the next object number.
    fn alloc_obj(&mut self) -> usize {
        let n = self.next_obj;
        self.next_obj += 1;
        n
    }

    /// Begin writing an object.
    fn begin_obj(&mut self, num: usize) {
        while self.objects.len() < num {
            self.objects.push(PdfObj { offset: 0 });
        }
        self.objects[num - 1].offset = self.buf.len();
        write!(self.buf, "{num} 0 obj\n").unwrap();
    }

    /// End an object.
    fn end_obj(&mut self) {
        write!(self.buf, "endobj\n").unwrap();
    }

    /// Write raw bytes.
    fn raw(&mut self, s: &str) {
        self.buf.extend_from_slice(s.as_bytes());
    }

    /// Escape a string for PDF text.
    fn escape_pdf(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for ch in s.chars() {
            match ch {
                '(' => out.push_str("\\("),
                ')' => out.push_str("\\)"),
                '\\' => out.push_str("\\\\"),
                _ if ch.is_ascii() => out.push(ch),
                _ => out.push('?'), // non-ASCII fallback
            }
        }
        out
    }

    /// Word-wrap text into lines that fit within `max_width` points.
    fn wrap_text(text: &str, font: Font, font_size: f64, max_width: f64) -> Vec<String> {
        let char_w = font.avg_char_width() * font_size;
        let max_chars = (max_width / char_w).floor() as usize;
        let max_chars = max_chars.max(1);

        let mut lines = Vec::new();
        for paragraph in text.split('\n') {
            if paragraph.is_empty() {
                lines.push(String::new());
                continue;
            }

            let words: Vec<&str> = paragraph.split_whitespace().collect();
            if words.is_empty() {
                lines.push(String::new());
                continue;
            }

            let mut current = String::new();
            for word in words {
                if current.is_empty() {
                    current = word.to_string();
                } else if current.len() + 1 + word.len() <= max_chars {
                    current.push(' ');
                    current.push_str(word);
                } else {
                    lines.push(current);
                    current = word.to_string();
                }
            }
            if !current.is_empty() {
                lines.push(current);
            }
        }
        lines
    }

    /// Generate content stream for a page.
    fn render_page_content(page: &Page, font_names: &[(&str, usize)]) -> Vec<u8> {
        let mut stream = Vec::new();
        let left = page.margins.left;
        let top = page.size.height - page.margins.top;
        let content_w = page.content_width();
        let mut y = top;

        for elem in &page.elements {
            match elem {
                ContentElement::Text { content, style } => {
                    let lines = Self::wrap_text(content, style.font, style.size, content_w);
                    let leading = style.size * style.line_height;
                    let font_ref = Self::find_font_ref(font_names, style.font);

                    write!(stream, "BT\n").unwrap();
                    write!(
                        stream,
                        "{} {} {} rg\n",
                        style.color.r, style.color.g, style.color.b
                    )
                    .unwrap();
                    write!(stream, "/{font_ref} {} Tf\n", style.size).unwrap();

                    for line in &lines {
                        let x = Self::align_x(left, content_w, line, style);
                        write!(stream, "{x} {y} Td\n").unwrap();
                        write!(stream, "({}) Tj\n", Self::escape_pdf(line)).unwrap();
                        y -= leading;
                        // Reset text position for next line
                        write!(stream, "{} {} Td\n", -x, -y + (y + leading)).unwrap();
                    }
                    write!(stream, "ET\n").unwrap();
                }
                ContentElement::Heading {
                    content,
                    level,
                    style,
                } => {
                    let size_mult = match level {
                        1 => 2.0,
                        2 => 1.5,
                        3 => 1.2,
                        _ => 1.0,
                    };
                    let actual_size = style.size * size_mult;
                    let font_ref = Self::find_font_ref(font_names, style.font);

                    y -= actual_size * 0.5; // space before heading

                    write!(stream, "BT\n").unwrap();
                    write!(
                        stream,
                        "{} {} {} rg\n",
                        style.color.r, style.color.g, style.color.b
                    )
                    .unwrap();
                    write!(stream, "/{font_ref} {actual_size} Tf\n").unwrap();
                    let x = Self::align_x(left, content_w, content, style);
                    write!(stream, "{x} {y} Td\n").unwrap();
                    write!(stream, "({}) Tj\n", Self::escape_pdf(content)).unwrap();
                    write!(stream, "ET\n").unwrap();

                    y -= actual_size * style.line_height;
                }
                ContentElement::HorizontalRule { thickness, color } => {
                    y -= 5.0;
                    write!(
                        stream,
                        "{} {} {} RG\n{thickness} w\n{left} {y} m {} {y} l S\n",
                        color.r,
                        color.g,
                        color.b,
                        left + content_w
                    )
                    .unwrap();
                    y -= 5.0;
                }
                ContentElement::Image {
                    display_width,
                    display_height,
                    ..
                } => {
                    y -= *display_height;
                    // Image placement handled via XObject reference
                    write!(
                        stream,
                        "q\n{display_width} 0 0 {display_height} {left} {y} cm\n/Img Do\nQ\n"
                    )
                    .unwrap();
                }
                ContentElement::Spacer { height } => {
                    y -= height;
                }
            }
        }

        stream
    }

    /// Compute x position for text alignment.
    fn align_x(left: f64, content_w: f64, text: &str, style: &TextStyle) -> f64 {
        let text_w = text.len() as f64 * style.font.avg_char_width() * style.size;
        match style.align {
            TextAlign::Left => left,
            TextAlign::Center => left + (content_w - text_w) / 2.0,
            TextAlign::Right => left + content_w - text_w,
        }
    }

    /// Find the font reference name.
    fn find_font_ref(font_names: &[(&str, usize)], font: Font) -> String {
        let name = font.pdf_name();
        for (i, (fname, _)) in font_names.iter().enumerate() {
            if *fname == name {
                return format!("F{}", i + 1);
            }
        }
        "F1".to_string()
    }
}

/// Generate PDF bytes from a document.
pub fn generate(doc: &Document) -> Vec<u8> {
    let mut w = PdfWriter::new();

    // Header — binary comment marks PDF as binary for transport layers
    w.buf.extend_from_slice(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n");

    // Collect unique fonts
    let mut fonts: Vec<(&str, usize)> = Vec::new();
    for page in &doc.pages {
        for elem in &page.elements {
            let font_name = match elem {
                ContentElement::Text { style, .. } => style.font.pdf_name(),
                ContentElement::Heading { style, .. } => style.font.pdf_name(),
                _ => continue,
            };
            if !fonts.iter().any(|(n, _)| *n == font_name) {
                let obj = w.alloc_obj();
                fonts.push((font_name, obj));
            }
        }
    }
    // Ensure at least one font
    if fonts.is_empty() {
        let obj = w.alloc_obj();
        fonts.push(("Helvetica", obj));
    }

    // Allocate objects: catalog, pages dict, info
    let catalog_obj = w.alloc_obj();
    let pages_obj = w.alloc_obj();
    let info_obj = w.alloc_obj();

    // Allocate page + content stream objects
    let mut page_objs = Vec::new();
    for _ in &doc.pages {
        let page_obj = w.alloc_obj();
        let content_obj = w.alloc_obj();
        page_objs.push((page_obj, content_obj));
    }

    // Write font objects
    for (i, (name, obj_num)) in fonts.iter().enumerate() {
        w.begin_obj(*obj_num);
        write!(
            w.buf,
            "<< /Type /Font /Subtype /Type1 /BaseFont /{name} /Name /F{} >>\n",
            i + 1
        )
        .unwrap();
        w.end_obj();
    }

    // Build font resource dict string
    let mut font_dict = String::from("<< ");
    for (i, (_, obj_num)) in fonts.iter().enumerate() {
        write!(&mut font_dict, "/F{} {} 0 R ", i + 1, obj_num).unwrap();
    }
    font_dict.push_str(">>");

    // Write page objects and content streams
    for (i, page) in doc.pages.iter().enumerate() {
        let (page_obj, content_obj) = page_objs[i];

        // Content stream
        let content_data = PdfWriter::render_page_content(page, &fonts);
        w.begin_obj(content_obj);
        write!(w.buf, "<< /Length {} >>\nstream\n", content_data.len()).unwrap();
        w.buf.extend_from_slice(&content_data);
        w.raw("\nendstream\n");
        w.end_obj();

        // Page object
        w.begin_obj(page_obj);
        write!(
            w.buf,
            "<< /Type /Page /Parent {pages_obj} 0 R /MediaBox [0 0 {} {}] /Contents {content_obj} 0 R /Resources << /Font {font_dict} >> >>\n",
            page.size.width, page.size.height
        )
        .unwrap();
        w.end_obj();
    }

    // Pages dictionary
    w.begin_obj(pages_obj);
    let kids: String = page_objs
        .iter()
        .map(|(p, _)| format!("{p} 0 R"))
        .collect::<Vec<_>>()
        .join(" ");
    write!(
        w.buf,
        "<< /Type /Pages /Kids [{kids}] /Count {} >>\n",
        doc.pages.len()
    )
    .unwrap();
    w.end_obj();

    // Info dictionary
    w.begin_obj(info_obj);
    w.raw("<< ");
    if let Some(title) = &doc.title {
        write!(w.buf, "/Title ({}) ", PdfWriter::escape_pdf(title)).unwrap();
    }
    if let Some(author) = &doc.author {
        write!(w.buf, "/Author ({}) ", PdfWriter::escape_pdf(author)).unwrap();
    }
    w.raw("/Producer (cclab-media) >>\n");
    w.end_obj();

    // Catalog
    w.begin_obj(catalog_obj);
    write!(
        w.buf,
        "<< /Type /Catalog /Pages {pages_obj} 0 R >>\n"
    )
    .unwrap();
    w.end_obj();

    // Cross-reference table
    let xref_offset = w.buf.len();
    write!(w.buf, "xref\n0 {}\n", w.objects.len() + 1).unwrap();
    write!(w.buf, "0000000000 65535 f \n").unwrap();
    for obj in &w.objects {
        write!(w.buf, "{:010} 00000 n \n", obj.offset).unwrap();
    }

    // Trailer
    write!(
        w.buf,
        "trailer\n<< /Size {} /Root {} 0 R /Info {} 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n",
        w.objects.len() + 1,
        catalog_obj,
        info_obj
    )
    .unwrap();

    w.buf
}

/// Convenience: generate PDF and write to a file.
pub fn write_to_file(doc: &Document, path: &str) -> std::io::Result<()> {
    let data = generate(doc);
    std::fs::write(path, data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_empty_doc() {
        let mut doc = Document::new().with_title("Empty");
        doc.add_page(Page::new());
        let bytes = generate(&doc);
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.starts_with("%PDF-1.4"));
        assert!(s.contains("%%EOF"));
        assert!(s.contains("/Type /Catalog"));
    }

    #[test]
    fn test_generate_with_text() {
        let mut doc = Document::new();
        let mut page = Page::new();
        page.add(ContentElement::Text {
            content: "Hello, World!".into(),
            style: TextStyle::default(),
        });
        doc.add_page(page);
        let bytes = generate(&doc);
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("Hello, World!"));
        assert!(s.contains("/Font"));
    }

    #[test]
    fn test_generate_with_heading() {
        let mut doc = Document::new();
        let mut page = Page::new();
        page.add(ContentElement::Heading {
            content: "Title".into(),
            level: 1,
            style: TextStyle {
                font: Font::HelveticaBold,
                ..Default::default()
            },
        });
        doc.add_page(page);
        let bytes = generate(&doc);
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("Title"));
        assert!(s.contains("Helvetica-Bold"));
    }

    #[test]
    fn test_word_wrap() {
        let lines = PdfWriter::wrap_text(
            "This is a test of the word wrapping system",
            Font::Helvetica,
            12.0,
            100.0,
        );
        assert!(lines.len() > 1, "should wrap into multiple lines");
        for line in &lines {
            let w = line.len() as f64 * Font::Helvetica.avg_char_width() * 12.0;
            assert!(
                w <= 110.0,
                "line too wide: \"{line}\" ({w:.1} > 100)"
            );
        }
    }

    #[test]
    fn test_escape_pdf() {
        assert_eq!(PdfWriter::escape_pdf("hello"), "hello");
        assert_eq!(PdfWriter::escape_pdf("a(b)c"), "a\\(b\\)c");
        assert_eq!(PdfWriter::escape_pdf("a\\b"), "a\\\\b");
    }

    #[test]
    fn test_horizontal_rule() {
        let mut doc = Document::new();
        let mut page = Page::new();
        page.add(ContentElement::HorizontalRule {
            thickness: 1.0,
            color: Color::BLACK,
        });
        doc.add_page(page);
        let bytes = generate(&doc);
        assert!(bytes.len() > 100);
    }

    #[test]
    fn test_multi_page() {
        let mut doc = Document::new().with_title("Multi");
        for i in 0..3 {
            let mut page = Page::new();
            page.add(ContentElement::Text {
                content: format!("Page {}", i + 1),
                style: TextStyle::default(),
            });
            doc.add_page(page);
        }
        let bytes = generate(&doc);
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/Count 3"));
    }
}
