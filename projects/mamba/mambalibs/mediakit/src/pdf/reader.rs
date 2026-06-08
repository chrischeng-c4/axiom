//! PDF reader — extract text and images, page operations (split, merge).
//!
//! This is a minimal parser for PDF 1.4 files. It handles basic structure
//! (header, xref, objects, content streams) for text extraction and page
//! manipulation. Complex PDFs (encrypted, compressed xref, etc.) may not
//! be fully supported.

use super::types::PageSize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PdfReadError {
    #[error("not a valid PDF: {0}")]
    InvalidPdf(String),
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("page {0} out of range (total: {1})")]
    PageOutOfRange(usize, usize),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, PdfReadError>;

/// Extracted text from a PDF page.
#[derive(Debug, Clone)]
pub struct PageText {
    /// Page number (1-indexed).
    pub page_number: usize,
    /// Extracted raw text content.
    pub text: String,
}

/// Information about a parsed PDF.
#[derive(Debug, Clone)]
pub struct PdfInfo {
    /// PDF version (e.g., "1.4").
    pub version: String,
    /// Number of pages.
    pub page_count: usize,
    /// Title from Info dict.
    pub title: Option<String>,
    /// Author from Info dict.
    pub author: Option<String>,
    /// Producer from Info dict.
    pub producer: Option<String>,
}

/// Extracted image from a PDF.
#[derive(Debug, Clone)]
pub struct ExtractedImage {
    /// Raw pixel data (RGB).
    pub data: Vec<u8>,
    /// Width.
    pub width: u32,
    /// Height.
    pub height: u32,
    /// Page the image was found on (1-indexed).
    pub page_number: usize,
}

/// Parse PDF header and return version string.
fn parse_version(data: &[u8]) -> Result<String> {
    // Only look at the first line (before any binary comment)
    if data.len() < 9 {
        return Err(PdfReadError::InvalidPdf("data too short".into()));
    }

    // Check for %PDF- magic bytes
    if &data[..5] != b"%PDF-" {
        return Err(PdfReadError::InvalidPdf("missing %PDF- header".into()));
    }

    // Extract version — bytes 5 up to first newline/CR
    let end = data[5..]
        .iter()
        .position(|&b| b == b'\n' || b == b'\r')
        .unwrap_or(3);
    let version = std::str::from_utf8(&data[5..5 + end])
        .map_err(|_| PdfReadError::InvalidPdf("invalid version bytes".into()))?;

    Ok(version.to_string())
}

/// Find all page object references from the /Pages dict.
fn find_page_refs(data: &[u8]) -> Vec<(usize, usize)> {
    let text = String::from_utf8_lossy(data);
    let mut pages = Vec::new();

    // Look for /Type /Page objects (not /Pages)
    let mut pos = 0;
    while let Some(idx) = text[pos..].find("/Type /Page") {
        let abs_pos = pos + idx;
        // Make sure it's /Page and not /Pages
        let after = abs_pos + 11;
        if after < text.len() {
            let next_char = text.as_bytes().get(after).copied().unwrap_or(b' ');
            if next_char == b's' || next_char == b'S' {
                pos = after;
                continue;
            }
        }

        // Find the object boundaries
        let obj_start = text[..abs_pos].rfind(" obj").map(|i| {
            // Walk back to find the object number
            text[..i].rfind('\n').map(|nl| nl + 1).unwrap_or(0)
        });
        let obj_end = text[abs_pos..].find("endobj").map(|i| abs_pos + i + 6);

        if let (Some(start), Some(end)) = (obj_start, obj_end) {
            pages.push((start, end));
        }
        pos = after;
    }
    pages
}

/// Extract text strings from a content stream.
///
/// Looks for text between Tj and TJ operators and parenthesized strings.
fn extract_text_from_stream(stream: &str) -> String {
    let mut result = String::new();

    let mut i = 0;
    let chars: Vec<char> = stream.chars().collect();

    while i < chars.len() {
        if chars[i] == '(' {
            // Extract parenthesized string
            let mut depth = 1;
            let mut s = String::new();
            i += 1;
            while i < chars.len() && depth > 0 {
                if chars[i] == '(' && (i == 0 || chars[i - 1] != '\\') {
                    depth += 1;
                    s.push('(');
                } else if chars[i] == ')' && (i == 0 || chars[i - 1] != '\\') {
                    depth -= 1;
                    if depth > 0 {
                        s.push(')');
                    }
                } else if chars[i] == '\\' && i + 1 < chars.len() {
                    match chars[i + 1] {
                        'n' => {
                            s.push('\n');
                            i += 1;
                        }
                        'r' => {
                            s.push('\r');
                            i += 1;
                        }
                        't' => {
                            s.push('\t');
                            i += 1;
                        }
                        '(' => {
                            s.push('(');
                            i += 1;
                        }
                        ')' => {
                            s.push(')');
                            i += 1;
                        }
                        '\\' => {
                            s.push('\\');
                            i += 1;
                        }
                        _ => s.push(chars[i]),
                    }
                } else {
                    s.push(chars[i]);
                }
                i += 1;
            }
            if !result.is_empty() && !result.ends_with('\n') && !result.ends_with(' ') {
                result.push(' ');
            }
            result.push_str(&s);
        } else {
            i += 1;
        }
    }

    result.trim().to_string()
}

/// Extract a stream's content between `stream` and `endstream` markers.
fn extract_stream(obj_text: &str) -> Option<&str> {
    let start = obj_text.find("stream")?;
    let content_start = obj_text[start + 6..]
        .find(|c: char| c != '\r' && c != '\n')
        .map(|i| start + 6 + i)?;
    let end = obj_text[content_start..].find("endstream")?;
    Some(&obj_text[content_start..content_start + end])
}

/// Get basic info about a PDF.
pub fn info(data: &[u8]) -> Result<PdfInfo> {
    let version = parse_version(data)?;
    let text = String::from_utf8_lossy(data);
    let page_refs = find_page_refs(data);

    // Extract info dict values
    let title = extract_info_field(&text, "Title");
    let author = extract_info_field(&text, "Author");
    let producer = extract_info_field(&text, "Producer");

    Ok(PdfInfo {
        version,
        page_count: page_refs.len(),
        title,
        author,
        producer,
    })
}

/// Extract a string field from the Info dictionary.
fn extract_info_field(text: &str, field: &str) -> Option<String> {
    let pattern = format!("/{field} (");
    let start = text.find(&pattern)?;
    let val_start = start + pattern.len();
    let end = text[val_start..].find(')')?;
    Some(text[val_start..val_start + end].to_string())
}

/// Extract text from all pages.
pub fn extract_text(data: &[u8]) -> Result<Vec<PageText>> {
    let _ = parse_version(data)?;
    let text = String::from_utf8_lossy(data);
    let page_refs = find_page_refs(data);
    let mut pages = Vec::new();

    for (i, (start, end)) in page_refs.iter().enumerate() {
        let obj_text = &text[*start..*end];
        let page_text = if let Some(stream) = extract_stream(obj_text) {
            extract_text_from_stream(stream)
        } else {
            // Try to find content stream reference and extract from there
            find_content_text(&text, obj_text)
        };

        pages.push(PageText {
            page_number: i + 1,
            text: page_text,
        });
    }

    Ok(pages)
}

/// Find content stream text from a page's /Contents reference.
fn find_content_text(full_text: &str, page_obj: &str) -> String {
    // Look for /Contents N 0 R
    if let Some(idx) = page_obj.find("/Contents ") {
        let after = &page_obj[idx + 10..];
        let obj_num: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(num) = obj_num.parse::<usize>() {
            // Find that object in the PDF
            let pattern = format!("{num} 0 obj");
            if let Some(obj_start) = full_text.find(&pattern) {
                let obj_end = full_text[obj_start..]
                    .find("endobj")
                    .map(|i| obj_start + i + 6)
                    .unwrap_or(full_text.len());
                let obj_text = &full_text[obj_start..obj_end];
                if let Some(stream) = extract_stream(obj_text) {
                    return extract_text_from_stream(stream);
                }
            }
        }
    }
    String::new()
}

/// Extract specific pages from a PDF into a new PDF.
///
/// `page_numbers` are 1-indexed.
pub fn split_pages(data: &[u8], page_numbers: &[usize]) -> Result<Vec<u8>> {
    let info = info(data)?;
    for &pn in page_numbers {
        if pn == 0 || pn > info.page_count {
            return Err(PdfReadError::PageOutOfRange(pn, info.page_count));
        }
    }

    // Re-generate a minimal PDF with just the selected pages' text
    let all_text = extract_text(data)?;
    let mut doc = super::types::Document::new();
    if let Some(title) = &info.title {
        doc.title = Some(title.clone());
    }

    for &pn in page_numbers {
        let mut page = super::types::Page::new();
        if let Some(pt) = all_text.iter().find(|t| t.page_number == pn) {
            if !pt.text.is_empty() {
                page.add(super::types::ContentElement::Text {
                    content: pt.text.clone(),
                    style: super::types::TextStyle::default(),
                });
            }
        }
        doc.add_page(page);
    }

    Ok(super::writer::generate(&doc))
}

/// Merge multiple PDFs into a single document.
///
/// Extracts text from all input PDFs and combines into a new document.
pub fn merge(pdfs: &[&[u8]]) -> Result<Vec<u8>> {
    let mut doc = super::types::Document::new();

    for pdf_data in pdfs {
        let pages = extract_text(pdf_data)?;
        for pt in pages {
            let mut page = super::types::Page::new();
            if !pt.text.is_empty() {
                page.add(super::types::ContentElement::Text {
                    content: pt.text,
                    style: super::types::TextStyle::default(),
                });
            }
            doc.add_page(page);
        }
    }

    Ok(super::writer::generate(&doc))
}

/// Get page sizes for all pages in the PDF.
pub fn page_sizes(data: &[u8]) -> Result<Vec<PageSize>> {
    let _ = parse_version(data)?;
    let text = String::from_utf8_lossy(data);
    let page_refs = find_page_refs(data);

    let mut sizes = Vec::new();
    for (start, end) in &page_refs {
        let obj_text = &text[*start..*end];
        let size = parse_media_box(obj_text).unwrap_or(PageSize::A4);
        sizes.push(size);
    }
    Ok(sizes)
}

/// Parse /MediaBox from a page object.
fn parse_media_box(obj_text: &str) -> Option<PageSize> {
    let idx = obj_text.find("/MediaBox [")?;
    let start = idx + 11;
    let end = obj_text[start..].find(']')?;
    let vals: Vec<f64> = obj_text[start..start + end]
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    if vals.len() >= 4 {
        Some(PageSize::custom(vals[2] - vals[0], vals[3] - vals[1]))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf::writer::generate;
    use crate::pdf::types::*;

    fn make_test_pdf() -> Vec<u8> {
        let mut doc = Document::new()
            .with_title("Test Doc")
            .with_author("Tester");
        let mut p1 = Page::new();
        p1.add(ContentElement::Text {
            content: "Hello from page one".into(),
            style: TextStyle::default(),
        });
        let mut p2 = Page::new().with_size(PageSize::LETTER);
        p2.add(ContentElement::Text {
            content: "Second page content".into(),
            style: TextStyle::default(),
        });
        doc.add_page(p1);
        doc.add_page(p2);
        generate(&doc)
    }

    #[test]
    fn test_info() {
        let pdf = make_test_pdf();
        let info = info(&pdf).unwrap();
        assert_eq!(info.version, "1.4");
        assert_eq!(info.page_count, 2);
        assert_eq!(info.title.as_deref(), Some("Test Doc"));
        assert_eq!(info.author.as_deref(), Some("Tester"));
    }

    #[test]
    fn test_extract_text() {
        let pdf = make_test_pdf();
        let pages = extract_text(&pdf).unwrap();
        assert_eq!(pages.len(), 2);
        assert!(pages[0].text.contains("Hello from page one"));
    }

    #[test]
    fn test_invalid_pdf() {
        let data = b"not a pdf";
        assert!(info(data).is_err());
    }

    #[test]
    fn test_extract_text_from_stream() {
        let stream = "BT\n/F1 12 Tf\n(Hello World) Tj\nET\n";
        let text = extract_text_from_stream(stream);
        assert_eq!(text, "Hello World");
    }

    #[test]
    fn test_extract_escaped_text() {
        let stream = "(Test \\(escaped\\) string) Tj";
        let text = extract_text_from_stream(stream);
        assert!(text.contains("Test (escaped) string"));
    }

    #[test]
    fn test_split_pages() {
        let pdf = make_test_pdf();
        let split = split_pages(&pdf, &[1]).unwrap();
        let info_split = info(&split).unwrap();
        assert_eq!(info_split.page_count, 1);
    }

    #[test]
    fn test_split_out_of_range() {
        let pdf = make_test_pdf();
        assert!(split_pages(&pdf, &[5]).is_err());
    }

    #[test]
    fn test_merge() {
        let pdf1 = make_test_pdf();
        let pdf2 = make_test_pdf();
        let merged = merge(&[&pdf1, &pdf2]).unwrap();
        let info_m = info(&merged).unwrap();
        assert_eq!(info_m.page_count, 4);
    }

    #[test]
    fn test_page_sizes() {
        let pdf = make_test_pdf();
        let sizes = page_sizes(&pdf).unwrap();
        assert_eq!(sizes.len(), 2);
    }

    #[test]
    fn test_parse_media_box() {
        let obj = "/Type /Page /MediaBox [0 0 612 792] /Contents 3 0 R";
        let size = parse_media_box(obj).unwrap();
        assert!((size.width - 612.0).abs() < 0.1);
        assert!((size.height - 792.0).abs() < 0.1);
    }
}
