// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
//! Tree-sitter based minifier.
//!
//! Phase 1: whitespace removal, comment stripping, console.log/debugger drop.
//! No identifier mangling in this phase.

/// Statements to drop during minification.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone)]
pub enum DropStatement {
    Console,
    Debugger,
}

/// Minify JavaScript source code.
///
/// - Removes comments (// and /* */)
/// - Collapses whitespace (multiple spaces/newlines → single space)
/// - Drops specified statements (console.log, debugger)
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn minify_js(source: &str, drops: &[DropStatement]) -> String {
    let mut result = String::with_capacity(source.len());

    // Pass 1: strip comments and drop statements
    let stripped = strip_comments(source);
    let dropped = drop_statements(&stripped, drops);

    // Pass 2: collapse whitespace
    let mut prev_char = '\0';
    let mut prev_non_ws = '\0'; // last non-whitespace char (for regex detection)
    let mut in_string = false;
    let mut string_char = '\0';
    let mut in_regex = false;

    let dropped_chars: Vec<char> = dropped.chars().collect();
    let dropped_len = dropped_chars.len();
    let mut idx = 0;

    while idx < dropped_len {
        let ch = dropped_chars[idx];

        // Track string literals
        if !in_string && !in_regex && (ch == '"' || ch == '\'' || ch == '`') {
            in_string = true;
            string_char = ch;
            result.push(ch);
            prev_char = ch;
            prev_non_ws = ch;
            idx += 1;
            continue;
        }
        if in_string {
            result.push(ch);
            if ch == string_char && !is_escaped(&dropped_chars, idx) {
                in_string = false;
            }
            prev_char = ch;
            if !ch.is_whitespace() {
                prev_non_ws = ch;
            }
            idx += 1;
            continue;
        }

        // Track regex literals: /pattern/flags
        // Use prev_non_ws to decide — space before / doesn't mean it's regex
        if !in_regex && ch == '/' && is_regex_start(prev_non_ws) {
            // Check it's not a comment
            if idx + 1 < dropped_len
                && (dropped_chars[idx + 1] == '/' || dropped_chars[idx + 1] == '*')
            {
                // It's a comment, not regex — shouldn't happen after strip_comments
                result.push(ch);
                prev_char = ch;
                prev_non_ws = ch;
                idx += 1;
                continue;
            }
            in_regex = true;
            result.push(ch);
            prev_char = ch;
            prev_non_ws = ch;
            idx += 1;
            continue;
        }
        if in_regex {
            result.push(ch);
            if ch == '/' && !is_escaped(&dropped_chars, idx) {
                in_regex = false;
            }
            prev_char = ch;
            if !ch.is_whitespace() {
                prev_non_ws = ch;
            }
            idx += 1;
            continue;
        }

        // Collapse whitespace outside strings and regexes. Newlines can be
        // semantic in semicolon-free JavaScript, so preserve ASI boundaries by
        // materializing a semicolon before removing the line break.
        if ch.is_whitespace() {
            let whitespace_start = idx;
            let mut has_newline = false;
            while idx < dropped_len && dropped_chars[idx].is_whitespace() {
                if dropped_chars[idx] == '\n' || dropped_chars[idx] == '\r' {
                    has_newline = true;
                }
                idx += 1;
            }

            let next_non_ws = dropped_chars.get(idx).copied();
            if has_newline
                && should_insert_asi_semicolon(prev_non_ws, next_non_ws, &dropped_chars, idx)
            {
                if !matches!(prev_char, ';' | '{' | '(' | '[' | ',' | ':' | '\0') {
                    if prev_char == ' ' {
                        result.pop();
                    }
                    result.push(';');
                    prev_char = ';';
                    prev_non_ws = ';';
                }
                continue;
            }

            if !prev_char.is_whitespace() && prev_char != '\0' {
                // Keep one space if needed for syntax
                if needs_space_after(prev_char) {
                    result.push(' ');
                    prev_char = ' ';
                }
            }
            if idx == whitespace_start {
                idx += 1;
            }
            continue;
        }

        // Remove space before certain chars
        if is_no_space_before(ch) && prev_char == ' ' {
            result.pop(); // remove trailing space
        }

        result.push(ch);
        prev_char = ch;
        prev_non_ws = ch;
        idx += 1;
    }

    result.trim().to_string()
}

/// Minify CSS source code.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn minify_css(source: &str) -> String {
    let stripped = strip_css_comments(source);
    let css_chars: Vec<char> = stripped.chars().collect();
    let mut result = String::with_capacity(stripped.len());

    let mut prev_char = '\0';
    let mut in_string = false;
    let mut string_char = '\0';

    for (css_idx, &ch) in css_chars.iter().enumerate() {
        if !in_string && (ch == '"' || ch == '\'') {
            in_string = true;
            string_char = ch;
            result.push(ch);
            prev_char = ch;
            continue;
        }
        if in_string {
            result.push(ch);
            if ch == string_char && !is_escaped(&css_chars, css_idx) {
                in_string = false;
            }
            prev_char = ch;
            continue;
        }

        if ch.is_whitespace() {
            if !prev_char.is_whitespace()
                && prev_char != '\0'
                && prev_char != '{'
                && prev_char != ';'
                && prev_char != ':'
                && prev_char != ','
            {
                result.push(' ');
                prev_char = ' ';
            }
            continue;
        }

        if (ch == '{' || ch == ':' || ch == ';') && prev_char == ' ' {
            result.pop();
        }

        result.push(ch);
        prev_char = ch;
    }

    result.trim().to_string()
}

/// Strip JavaScript comments (single-line and multi-line).
/// Tracks string AND regex literals to avoid stripping content inside them.
fn strip_comments(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_string = false;
    let mut string_char = '\0';
    let mut in_regex = false;
    let mut prev_non_ws = '\0';

    while i < len {
        // Track strings
        if !in_string && !in_regex && (chars[i] == '"' || chars[i] == '\'' || chars[i] == '`') {
            in_string = true;
            string_char = chars[i];
            result.push(chars[i]);
            prev_non_ws = chars[i];
            i += 1;
            continue;
        }
        if in_string {
            result.push(chars[i]);
            if chars[i] == string_char && !is_escaped(&chars, i) {
                in_string = false;
            }
            if !chars[i].is_whitespace() {
                prev_non_ws = chars[i];
            }
            i += 1;
            continue;
        }

        // Track regex literals
        if !in_regex
            && chars[i] == '/'
            && i + 1 < len
            && chars[i + 1] != '/'
            && chars[i + 1] != '*'
            && is_regex_start(prev_non_ws)
        {
            in_regex = true;
            result.push(chars[i]);
            prev_non_ws = chars[i];
            i += 1;
            continue;
        }
        if in_regex {
            result.push(chars[i]);
            // Handle character class [...] — / inside [] doesn't close regex
            if chars[i] == '[' {
                i += 1;
                while i < len && chars[i] != ']' {
                    result.push(chars[i]);
                    if chars[i] == '\\' && i + 1 < len {
                        i += 1;
                        result.push(chars[i]);
                    }
                    i += 1;
                }
                if i < len {
                    result.push(chars[i]); // push ']'
                }
                i += 1;
                continue;
            }
            if chars[i] == '/' && !is_escaped(&chars, i) {
                in_regex = false;
            }
            if !chars[i].is_whitespace() {
                prev_non_ws = chars[i];
            }
            i += 1;
            continue;
        }

        // Single-line comment
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '/' {
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // Multi-line comment
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i += 2; // skip */
            continue;
        }

        if !chars[i].is_whitespace() {
            prev_non_ws = chars[i];
        }
        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Strip CSS comments (/* ... */).
fn strip_css_comments(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i += 2;
            continue;
        }
        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Drop specified statements from source.
fn drop_statements(source: &str, drops: &[DropStatement]) -> String {
    let mut result = String::new();
    for line in source.lines() {
        let trimmed = line.trim();
        let should_drop = drops.iter().any(|drop| match drop {
            DropStatement::Console => trimmed.starts_with("console.") && trimmed.contains('('),
            DropStatement::Debugger => trimmed == "debugger;" || trimmed == "debugger",
        });

        if !should_drop {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

/// Check if the character at position `pos` is escaped by counting preceding backslashes.
/// An even number of backslashes means NOT escaped, odd means escaped.
fn is_escaped(chars: &[char], pos: usize) -> bool {
    let mut backslash_count = 0;
    let mut j = pos;
    while j > 0 {
        j -= 1;
        if chars[j] == '\\' {
            backslash_count += 1;
        } else {
            break;
        }
    }
    backslash_count % 2 != 0
}

/// Heuristic: `/` starts a regex if preceded by these chars (or start of input).
/// After identifiers, numbers, `)`, `]` → `/` is division.
fn is_regex_start(prev_non_ws: char) -> bool {
    matches!(
        prev_non_ws,
        '=' | '('
            | ','
            | '['
            | '!'
            | '&'
            | '|'
            | '?'
            | ':'
            | ';'
            | '{'
            | '}'
            | '\0'
            | '<'
            | '>'
            | '+'
            | '-'
            | '*'
            | '%'
            | '^'
            | '~'
    )
}

fn needs_space_after(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '$' || ch == ')' || ch == ']' || ch == '}'
}

fn should_insert_asi_semicolon(
    prev_non_ws: char,
    next_non_ws: Option<char>,
    chars: &[char],
    next_idx: usize,
) -> bool {
    let Some(next) = next_non_ws else {
        return false;
    };

    if !can_end_statement(prev_non_ws) || !can_start_statement(next) {
        return false;
    }

    if prev_non_ws == '}' && starts_with_keyword(chars, next_idx, "else") {
        return false;
    }
    if prev_non_ws == '}' && starts_with_keyword(chars, next_idx, "catch") {
        return false;
    }
    if prev_non_ws == '}' && starts_with_keyword(chars, next_idx, "finally") {
        return false;
    }
    if prev_non_ws == '}' && starts_with_keyword(chars, next_idx, "while") {
        return false;
    }

    true
}

fn can_end_statement(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '_' | '$' | ')' | ']' | '}' | '"' | '\'' | '`')
}

fn can_start_statement(ch: char) -> bool {
    ch.is_alphabetic() || matches!(ch, '_' | '$' | '(' | '[' | '{' | '"' | '\'' | '`')
}

fn starts_with_keyword(chars: &[char], start: usize, keyword: &str) -> bool {
    for (offset, expected) in keyword.chars().enumerate() {
        if chars.get(start + offset).copied() != Some(expected) {
            return false;
        }
    }

    let before_ok = start == 0 || !is_identifier_char(chars[start - 1]);
    let after_idx = start + keyword.chars().count();
    let after_ok = chars
        .get(after_idx)
        .copied()
        .is_none_or(|ch| !is_identifier_char(ch));

    before_ok && after_ok
}

fn is_identifier_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '$'
}

fn is_no_space_before(ch: char) -> bool {
    matches!(
        ch,
        '{' | '}' | '(' | ')' | '[' | ']' | ';' | ',' | '.' | ':' | '=' | '?' | '<' | '>'
    )
}

/// Replace `true` with `!0` and `false` with `!1` (saves bytes).
/// Only replaces standalone keyword occurrences outside strings.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn replace_bool_literals(source: &str) -> String {
    let b = source.as_bytes();
    let len = b.len();
    let mut out = Vec::with_capacity(len);
    let mut i = 0;

    while i < len {
        // Skip string literals
        if matches!(b[i], b'"' | b'\'' | b'`') {
            let q = b[i];
            out.push(q);
            i += 1;
            while i < len {
                if b[i] == b'\\' {
                    out.push(b[i]);
                    i += 1;
                    if i < len {
                        out.push(b[i]);
                        i += 1;
                    }
                    continue;
                }
                out.push(b[i]);
                if b[i] == q {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // true → !0
        if i + 4 <= len
            && &b[i..i + 4] == b"true"
            && (i == 0 || !is_id_char(b[i - 1]))
            && (i + 4 >= len || !is_id_char(b[i + 4]))
        {
            out.extend_from_slice(b"!0");
            i += 4;
            continue;
        }

        // false → !1
        if i + 5 <= len
            && &b[i..i + 5] == b"false"
            && (i == 0 || !is_id_char(b[i - 1]))
            && (i + 5 >= len || !is_id_char(b[i + 5]))
        {
            out.extend_from_slice(b"!1");
            i += 5;
            continue;
        }

        out.push(b[i]);
        i += 1;
    }

    String::from_utf8(out).unwrap_or_else(|_| source.to_string())
}

fn is_id_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
}

// HTML minification is in the `html_minify` submodule.
// Re-exported from `super::html_minify::minify_html`.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minify_whitespace() {
        let source = "const  x  =  1 ;\nconst  y  =  2 ;";
        let result = minify_js(source, &[]);
        assert!(!result.contains("  ")); // no double spaces
        assert!(result.contains("const x"));
    }

    #[test]
    fn test_minify_inserts_semicolon_between_asi_statements() {
        let source = "const x = 1\nconst y = 2";
        let result = minify_js(source, &[]);
        assert_eq!(result, "const x=1;const y=2");
    }

    #[test]
    fn test_minify_preserves_return_asi_before_if() {
        let source = "function f(value) {\n  if (value) return 'yes'\n  if (!value) return 'no'\n  return 'unknown'\n}";
        let result = minify_js(source, &[]);
        assert!(result.contains("return 'yes';if"), "got: {}", result);
        assert!(result.contains("return 'no';return"), "got: {}", result);
    }

    #[test]
    fn test_minify_does_not_insert_semicolon_before_else() {
        let source = "if (ok) {\n  run()\n} else {\n  stop()\n}";
        let result = minify_js(source, &[]);
        assert!(result.contains("} else{"), "got: {}", result);
        assert!(!result.contains("};else"), "got: {}", result);
        assert!(!result.contains("}; else"), "got: {}", result);
    }

    #[test]
    fn test_strip_comments() {
        let source = "const x = 1; // comment\nconst y = /* block */ 2;";
        let result = strip_comments(source);
        assert!(!result.contains("comment"));
        assert!(!result.contains("block"));
        assert!(result.contains("const x = 1;"));
    }

    #[test]
    fn test_drop_console() {
        let source = "console.log('test');\nconst x = 1;";
        let result = drop_statements(source, &[DropStatement::Console]);
        assert!(!result.contains("console.log"));
        assert!(result.contains("const x = 1;"));
    }

    #[test]
    fn test_drop_debugger() {
        let source = "debugger;\nconst x = 1;";
        let result = drop_statements(source, &[DropStatement::Debugger]);
        assert!(!result.contains("debugger"));
        assert!(result.contains("const x = 1;"));
    }

    #[test]
    fn test_preserve_strings() {
        let source = r#"const x = "hello  world";"#;
        let result = minify_js(source, &[]);
        assert!(result.contains("\"hello  world\""));
    }

    #[test]
    fn test_preserve_regex_spaces() {
        let source = r#"var match = x.stack.trim().match(/\n( *(at )?)/)"#;
        let result = minify_js(source, &[]);
        assert!(
            result.contains(r"/\n( *(at )?)/"),
            "regex spaces should be preserved, got: {}",
            result
        );
    }

    #[test]
    fn test_regex_with_quotes_no_corrupt_strings() {
        // Regex /[\n"\\]/g contains a quote — must not confuse string tracking
        let source = r#"var re = /[\n"\\]/g;
var url = "http://example.com";
console.log(url);"#;
        let result = minify_js(source, &[]);
        assert!(
            result.contains("http://example.com"),
            "URL inside string should be preserved, got: {}",
            result
        );
    }

    #[test]
    fn test_strip_comments_regex_with_quote() {
        // Regex /[\n"\\]/g contains a literal " — must not confuse string tracking
        let source = r#"var re = /[\n"\\]/g;
nextResource = ownerDocument.createElementNS(
    "http://www.w3.org/2000/svg",
    type
);"#;
        let result = strip_comments(source);
        assert!(
            result.contains("http://www.w3.org/2000/svg"),
            "URL in string must survive strip_comments, got: {}",
            result
        );
    }

    #[test]
    fn test_division_not_regex() {
        let source = "return (31 - ((log(x) / LN2) | 0)) | 0;";
        let result = minify_js(source, &[]);
        // / should be treated as division, not regex — full expression preserved
        assert!(
            result.contains("/LN2)"),
            "division should be preserved, got: {}",
            result
        );
        assert!(
            result.contains("|0))"),
            "outer parens should be preserved, got: {}",
            result
        );
    }

    #[test]
    fn test_escaped_backslash_in_string() {
        // "\\" is a string containing one backslash — the closing " must not be treated as escaped
        let source = r#"return "\\" + ch; // comment
var x = 1;"#;
        let result = strip_comments(source);
        assert!(
            result.contains("var x = 1"),
            "code after string must survive, got: {}",
            result
        );
        assert!(
            !result.contains("comment"),
            "comment should be stripped, got: {}",
            result
        );

        // Also test minify_js
        let result2 = minify_js(source, &[]);
        assert!(
            result2.contains("var x"),
            "code after string must survive minify, got: {}",
            result2
        );
    }

    #[test]
    fn test_strip_comments_real_bundle() {
        // Read the actual pre-minified bundle to reproduce the SVG URL corruption
        let bundle_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("examples/react-bench/dist/_debug_preminify.js");
        if !bundle_path.exists() {
            eprintln!("Skipping: bundle not found at {:?}", bundle_path);
            return;
        }
        let source = std::fs::read_to_string(&bundle_path).unwrap();
        let result = strip_comments(&source);

        // Check all SVG URL occurrences survive
        let svg_count_before = source.matches("http://www.w3.org/2000/svg").count();
        let svg_count_after = result.matches("http://www.w3.org/2000/svg").count();
        assert_eq!(
            svg_count_before, svg_count_after,
            "SVG URLs lost: {} before, {} after strip_comments",
            svg_count_before, svg_count_after
        );

        // Also check Math/MathML URL
        let math_before = source.matches("http://www.w3.org/1998/Math/MathML").count();
        let math_after = result.matches("http://www.w3.org/1998/Math/MathML").count();
        assert_eq!(
            math_before, math_after,
            "MathML URLs lost: {} before, {} after strip_comments",
            math_before, math_after
        );
    }

    #[test]
    fn test_minify_js_real_bundle() {
        let bundle_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("examples/react-bench/dist/_debug_preminify.js");
        if !bundle_path.exists() {
            eprintln!("Skipping: bundle not found at {:?}", bundle_path);
            return;
        }
        let source = std::fs::read_to_string(&bundle_path).unwrap();
        let result = minify_js(&source, &[DropStatement::Console, DropStatement::Debugger]);

        // All SVG URLs must survive full minification
        let svg_count = source.matches("http://www.w3.org/2000/svg").count();
        let svg_after = result.matches("http://www.w3.org/2000/svg").count();
        assert_eq!(
            svg_count, svg_after,
            "SVG URLs lost in minify_js: {} before, {} after",
            svg_count, svg_after
        );
    }

    #[test]
    fn test_no_space_before_equals() {
        let result = minify_js("var a = 1;", &[]);
        assert_eq!(result, "var a=1;");
    }

    #[test]
    fn test_no_space_before_ternary() {
        let result = minify_js("x ? a : b", &[]);
        assert_eq!(result, "x?a:b");
    }

    #[test]
    fn test_no_space_before_comparison() {
        let result = minify_js("a < b", &[]);
        assert_eq!(result, "a<b");
        let result2 = minify_js("a > b", &[]);
        assert_eq!(result2, "a>b");
    }

    #[test]
    fn test_compound_ops_preserved() {
        // <= and >= should still work
        let result = minify_js("a <= b", &[]);
        assert_eq!(result, "a<=b");
        let result2 = minify_js("a >= b", &[]);
        assert_eq!(result2, "a>=b");
        let result3 = minify_js("a === b", &[]);
        assert_eq!(result3, "a===b");
    }

    #[test]
    fn test_replace_bool_true() {
        assert_eq!(replace_bool_literals("return true;"), "return !0;");
    }

    #[test]
    fn test_replace_bool_false() {
        assert_eq!(replace_bool_literals("x=false"), "x=!1");
    }

    #[test]
    fn test_replace_bool_in_string_preserved() {
        assert_eq!(replace_bool_literals(r#""true""#), r#""true""#);
        assert_eq!(replace_bool_literals(r#"'false'"#), r#"'false'"#);
    }

    #[test]
    fn test_replace_bool_in_identifier_preserved() {
        assert_eq!(replace_bool_literals("trueValue"), "trueValue");
        assert_eq!(replace_bool_literals("isFalse"), "isFalse");
    }

    #[test]
    fn test_minify_css() {
        let source = "body {\n  color: red;\n  margin: 0;\n}\n";
        let result = minify_css(source);
        assert!(!result.contains('\n'));
        assert!(result.contains("color:red"));
    }

    // ──────────────────────────────────────────────────────────────────
    // UTF-8 multi-byte safety tests (issue #904)
    //
    // The minifier iterates with `chars().collect::<Vec<char>>()` and
    // pushes chars directly to the result — it never slices `source` by
    // char index.  These tests confirm that multi-byte characters are
    // handled correctly end-to-end.
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_minify_utf8_multibyte_in_string() {
        // ✓ is 3 bytes (E2 9C 93); should survive unchanged inside string
        let source = r#"var x = "✓ passed";"#;
        let result = minify_js(source, &[]);
        assert!(
            result.contains("\"✓ passed\""),
            "UTF-8 string preserved, got: {}",
            result
        );
    }

    #[test]
    fn test_minify_utf8_emoji_in_string() {
        // 🎉 is 4 bytes (F0 9F 8E 89)
        let source = r#"console.log("Hello 🎉");  const x = 1;"#;
        let result = minify_js(source, &[]);
        assert!(
            result.contains("\"Hello 🎉\""),
            "emoji in string preserved, got: {}",
            result
        );
        assert!(
            result.contains("const x"),
            "code after emoji string intact, got: {}",
            result
        );
    }

    #[test]
    fn test_minify_utf8_cjk_in_string() {
        // CJK characters: 日本語 (3 bytes each)
        let source = "var label = '日本語テスト';  var x = 1;";
        let result = minify_js(source, &[]);
        assert!(
            result.contains("'日本語テスト'"),
            "CJK string preserved, got: {}",
            result
        );
    }

    #[test]
    fn test_strip_comments_utf8_preserved() {
        let source = "// comment\nvar x = '日本語'; /* block */ var y = 1;";
        let result = strip_comments(source);
        assert!(!result.contains("comment"), "comment stripped");
        assert!(!result.contains("block"), "block comment stripped");
        assert!(
            result.contains("'日本語'"),
            "CJK string preserved, got: {}",
            result
        );
    }

    #[test]
    fn test_minify_utf8_outside_string() {
        // UTF-8 identifier characters (JS allows them, though uncommon in practice).
        // The minifier should pass them through without panic.
        let source = "var café = 1; /* strip */ var x = café;";
        let result = minify_js(source, &[]);
        // Should not panic, and the identifier should survive
        assert!(
            result.contains("café"),
            "UTF-8 identifier preserved, got: {}",
            result
        );
    }
}
// CODEGEN-END
