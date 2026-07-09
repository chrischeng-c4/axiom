// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
//! HTML minification: strip comments, collapse whitespace, simplify attributes.
//!
//! Custom implementation with no external dependencies. Preserves whitespace
//! inside `<pre>`, `<code>`, `<script>`, and `<style>` tags.

/// Minify HTML source code.
///
/// - Strips HTML comments (`<!-- ... -->`)
/// - Collapses whitespace between tags (preserves within `<pre>`, `<code>`,
///   `<script>`, `<style>`)
/// - Removes unnecessary quotes on attributes with simple values
/// - No new dependency -- custom implementation using string scanning
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn minify_html(source: &str) -> String {
    // Step 1: Strip HTML comments
    let no_comments = strip_html_comments(source);

    // Step 2: Collapse whitespace, preserving content in special tags
    collapse_html_whitespace(&no_comments)
}

/// Strip HTML comments (<!-- ... -->).
/// Preserves conditional comments (<!--[if ...]>) for IE compatibility.
fn strip_html_comments(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Check for comment start: <!--
        if i + 3 < len
            && bytes[i] == b'<'
            && bytes[i + 1] == b'!'
            && bytes[i + 2] == b'-'
            && bytes[i + 3] == b'-'
        {
            // Skip conditional comments (<!--[if)
            if i + 4 < len && bytes[i + 4] == b'[' {
                result.push_str("<!");
                i += 2;
                continue;
            }

            // Find comment end -->
            i += 4;
            while i + 2 < len {
                if bytes[i] == b'-' && bytes[i + 1] == b'-' && bytes[i + 2] == b'>' {
                    i += 3;
                    break;
                }
                i += 1;
            }
            // If we hit end of string without finding -->, skip remainder
            if i + 2 >= len && !(i < len && bytes.get(i) == Some(&b'-')) {
                break;
            }
            continue;
        }

        result.push(bytes[i] as char);
        i += 1;
    }

    result
}

/// Tags whose content whitespace must be preserved.
const PRESERVE_WS_TAGS: &[&str] = &["pre", "code", "script", "style", "textarea"];

/// Collapse whitespace in HTML while preserving content of special tags.
fn collapse_html_whitespace(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut preserve_depth: Vec<String> = Vec::new(); // stack of preserved tags

    while i < len {
        // Check for opening tag of preserved-whitespace elements
        if chars[i] == '<' && i + 1 < len && chars[i + 1].is_ascii_alphabetic() {
            let tag_start = i;
            let tag_name = extract_tag_name(&chars, i + 1);
            let lower_tag = tag_name.to_lowercase();

            if PRESERVE_WS_TAGS.contains(&lower_tag.as_str()) {
                preserve_depth.push(lower_tag.clone());
                // Copy everything until the matching closing tag
                // First, copy the opening tag
                while i < len && chars[i] != '>' {
                    result.push(chars[i]);
                    i += 1;
                }
                if i < len {
                    result.push(chars[i]); // push '>'
                    i += 1;
                }
                // Now copy content verbatim until closing tag
                let close_tag = format!("</{}", lower_tag);
                while i < len {
                    // Check for closing tag
                    let remaining: String = chars[i..].iter().take(close_tag.len() + 1).collect();
                    if remaining.to_lowercase().starts_with(&close_tag) {
                        preserve_depth.pop();
                        // Copy the closing tag
                        while i < len && chars[i] != '>' {
                            result.push(chars[i]);
                            i += 1;
                        }
                        if i < len {
                            result.push(chars[i]); // push '>'
                            i += 1;
                        }
                        break;
                    }
                    result.push(chars[i]);
                    i += 1;
                }
                continue;
            }

            // Not a preserved tag -- handle attribute quote removal
            let tag_content = collect_tag(&chars, tag_start);
            let minified_tag = minify_tag_attributes(&tag_content);
            result.push_str(&minified_tag);
            i = tag_start + tag_content.len();
            continue;
        }

        // Inside normal content: collapse whitespace between tags
        if !preserve_depth.is_empty() {
            result.push(chars[i]);
            i += 1;
            continue;
        }

        if chars[i].is_whitespace() {
            // Collapse consecutive whitespace to a single space
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }
            // Don't add space if we're right before or after a tag
            if !result.is_empty() && !result.ends_with('>') {
                if i < len && chars[i] != '<' {
                    result.push(' ');
                }
            }
            continue;
        }

        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Extract a tag name from position in char array.
fn extract_tag_name(chars: &[char], start: usize) -> String {
    let mut name = String::new();
    let mut i = start;
    while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '-') {
        name.push(chars[i]);
        i += 1;
    }
    name
}

/// Collect an entire tag (from '<' to '>') from the char array.
fn collect_tag(chars: &[char], start: usize) -> String {
    let mut tag = String::new();
    let mut i = start;
    while i < chars.len() {
        tag.push(chars[i]);
        if chars[i] == '>' {
            break;
        }
        i += 1;
    }
    tag
}

/// Remove unnecessary quotes from tag attributes with simple values.
///
/// A "simple value" is one containing only [a-zA-Z0-9_-], which does
/// not need quoting per the HTML spec.
fn minify_tag_attributes(tag: &str) -> String {
    // Quick check: if no '=' found, nothing to simplify
    if !tag.contains('=') {
        return tag.to_string();
    }

    let mut result = String::with_capacity(tag.len());
    let chars: Vec<char> = tag.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '=' && i + 1 < len {
            result.push('=');
            i += 1;
            // Check for quoted attribute value
            if chars[i] == '"' || chars[i] == '\'' {
                let quote = chars[i];
                let val_start = i + 1;
                let mut val_end = val_start;
                while val_end < len && chars[val_end] != quote {
                    val_end += 1;
                }
                let value: String = chars[val_start..val_end].iter().collect();
                if is_simple_attr_value(&value) {
                    result.push_str(&value);
                } else {
                    result.push(quote);
                    result.push_str(&value);
                    if val_end < len {
                        result.push(quote);
                    }
                }
                i = if val_end < len { val_end + 1 } else { val_end };
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Check if an attribute value is "simple" (only contains safe characters).
fn is_simple_attr_value(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_comment_removal() {
        // T14: Strip HTML comments
        let input = "<!-- todo -->  <div>hello</div>";
        let result = minify_html(input);
        assert!(
            !result.contains("<!-- todo -->"),
            "HTML comments should be removed, got: {}",
            result
        );
        assert!(
            result.contains("<div>hello</div>"),
            "Content should be preserved, got: {}",
            result
        );
    }

    #[test]
    fn test_html_whitespace_collapse() {
        // S11: Whitespace between tags should be collapsed
        let input = "<!-- comment -->  <div>  <p>text</p>  </div>";
        let result = minify_html(input);
        assert!(
            !result.contains("<!-- comment -->"),
            "Comments should be removed, got: {}",
            result
        );
        assert!(
            result.contains("<div>"),
            "Tags should be preserved, got: {}",
            result
        );
        assert!(
            result.contains("<p>text</p>"),
            "Inner content should be preserved, got: {}",
            result
        );
    }

    #[test]
    fn test_html_preserves_pre_content() {
        // T15: Whitespace inside <pre> should be preserved
        let input = "<pre>  spaces  matter  </pre>";
        let result = minify_html(input);
        assert!(
            result.contains("  spaces  matter  "),
            "Whitespace inside <pre> must be preserved, got: {}",
            result
        );
    }

    #[test]
    fn test_html_preserves_code_content() {
        let input = "<code>  let x = 1;  </code>";
        let result = minify_html(input);
        assert!(
            result.contains("  let x = 1;  "),
            "Whitespace inside <code> must be preserved, got: {}",
            result
        );
    }

    #[test]
    fn test_html_preserves_script_content() {
        let input = "<script>  var x = 1;  </script>";
        let result = minify_html(input);
        assert!(
            result.contains("  var x = 1;  "),
            "Whitespace inside <script> must be preserved, got: {}",
            result
        );
    }

    #[test]
    fn test_html_preserves_style_content() {
        let input = "<style>  body { color: red; }  </style>";
        let result = minify_html(input);
        assert!(
            result.contains("  body { color: red; }  "),
            "Whitespace inside <style> must be preserved, got: {}",
            result
        );
    }

    #[test]
    fn test_html_attribute_quote_removal() {
        let input = r#"<div class="main" id="app">"#;
        let result = minify_html(input);
        assert!(
            result.contains("class=main"),
            "Simple attribute quotes should be removed, got: {}",
            result
        );
    }

    #[test]
    fn test_html_complex_attribute_keeps_quotes() {
        let input = r#"<a href="https://example.com">"#;
        let result = minify_html(input);
        assert!(
            result.contains("\"https://example.com\"") || result.contains("'https://example.com'"),
            "Complex attribute values should keep quotes, got: {}",
            result
        );
    }

    #[test]
    fn test_html_multiple_comments() {
        let input = "<!-- first -->text<!-- second -->more";
        let result = minify_html(input);
        assert!(!result.contains("first"), "First comment removed");
        assert!(!result.contains("second"), "Second comment removed");
        assert!(result.contains("text"), "Text preserved");
        assert!(result.contains("more"), "More text preserved");
    }
}
// CODEGEN-END
