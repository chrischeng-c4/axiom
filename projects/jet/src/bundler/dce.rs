// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
//! Dead Code Elimination (DCE).
//!
//! Removes unreachable code branches after compile-time constant replacement.
//! For example, after `process.env.NODE_ENV` is replaced with `"production"`,
//! branches like `if ("production" !== "production") { ... }` become statically
//! evaluable and the dead branch can be removed.
//!
//! NOTE: All index variables in this module are *char indices* into a `Vec<char>`.
//! When slicing the original `&str` we must convert through `byte_offsets` to
//! avoid panics on multi-byte UTF-8 characters (e.g. `✓`, emoji).

/// Build a lookup table: byte_offsets[char_idx] = byte offset in `source`.
/// byte_offsets[chars.len()] = source.len() (one past the end).
fn build_byte_offsets(source: &str) -> Vec<usize> {
    let mut offsets: Vec<usize> = source.char_indices().map(|(i, _)| i).collect();
    offsets.push(source.len());
    offsets
}

/// Slice `source` using char indices, converting through byte offsets.
fn slice_source<'a>(source: &'a str, bo: &[usize], start: usize, end: usize) -> &'a str {
    &source[bo[start]..bo[end]]
}

/// Eliminate dead code from source after define replacement.
///
/// Handles:
/// - `if ("production" !== "production") { ... }` → removed
/// - `if ("production" === "production") { ... } else { ... }` → keeps if-body
/// - Ternary: `"production" !== "production" ? a : b` → `b`
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn eliminate_dead_code(source: &str) -> String {
    let mut result = source.to_string();

    // Iteratively apply DCE until no more changes (handles nested cases)
    loop {
        let prev = result.clone();
        result = eliminate_if_blocks(&result);
        result = eliminate_ternaries(&result);
        if result == prev {
            break;
        }
    }

    result
}

/// Evaluate a simple string comparison expression.
/// Returns Some(true/false) if statically evaluable, None otherwise.
fn eval_condition(cond: &str) -> Option<bool> {
    let cond = cond.trim();

    // "x" === "y" or "x" !== "y" or "x" == "y" or "x" != "y"
    for (op, invert) in &[("!==", true), ("===", false), ("!=", true), ("==", false)] {
        if let Some(pos) = cond.find(op) {
            let lhs = cond[..pos].trim();
            let rhs = cond[pos + op.len()..].trim();

            if let (Some(l), Some(r)) = (extract_string_literal(lhs), extract_string_literal(rhs)) {
                let equal = l == r;
                return Some(if *invert { !equal } else { equal });
            }

            // Handle boolean comparisons: false === false, true !== false, etc.
            if let (Some(l), Some(r)) = (parse_bool(lhs), parse_bool(rhs)) {
                let equal = l == r;
                return Some(if *invert { !equal } else { equal });
            }
        }
    }

    // Direct boolean: "false", "true"
    parse_bool(cond)
}

fn extract_string_literal(s: &str) -> Option<&str> {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        Some(&s[1..s.len() - 1])
    } else {
        None
    }
}

fn parse_bool(s: &str) -> Option<bool> {
    match s.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// Find matching closing brace, handling nested braces.
/// All positions are char indices.
fn find_matching_brace(chars: &[char], open_pos: usize) -> Option<usize> {
    let mut depth = 0;
    let mut in_string = false;
    let mut string_char = '"';
    let mut i = open_pos;

    while i < chars.len() {
        let ch = chars[i];

        if in_string {
            if ch == '\\' {
                i += 1; // skip escaped char
            } else if ch == string_char {
                in_string = false;
            }
            i += 1;
            continue;
        }

        match ch {
            '"' | '\'' | '`' => {
                in_string = true;
                string_char = ch;
            }
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Eliminate if-blocks with statically evaluable conditions.
fn eliminate_if_blocks(source: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = source.chars().collect();
    let bo = build_byte_offsets(source);
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Look for "if" keyword
        if i + 2 < len
            && chars[i] == 'i'
            && chars[i + 1] == 'f'
            && (i == 0 || !chars[i - 1].is_alphanumeric())
        {
            let after_if = i + 2;
            // Skip whitespace
            let mut j = after_if;
            while j < len && chars[j].is_whitespace() {
                j += 1;
            }

            if j < len && chars[j] == '(' {
                // Find matching closing paren
                if let Some(close_paren) = find_matching_paren(&chars, j) {
                    let cond = slice_source(source, &bo, j + 1, close_paren);

                    if let Some(val) = eval_condition(cond) {
                        // Skip whitespace after condition
                        let mut k = close_paren + 1;
                        while k < len && chars[k].is_whitespace() {
                            k += 1;
                        }

                        if k < len && chars[k] == '{' {
                            if let Some(close_brace) = find_matching_brace(&chars, k) {
                                let if_body = slice_source(source, &bo, k + 1, close_brace);

                                // Check for else
                                let mut m = close_brace + 1;
                                while m < len && chars[m].is_whitespace() {
                                    m += 1;
                                }

                                let has_else = m + 4 <= len
                                    && slice_source(source, &bo, m, m + 4) == "else"
                                    && (m + 4 >= len || !chars[m + 4].is_alphanumeric());

                                if has_else {
                                    let mut n = m + 4;
                                    while n < len && chars[n].is_whitespace() {
                                        n += 1;
                                    }

                                    if n < len && chars[n] == '{' {
                                        if let Some(else_close) = find_matching_brace(&chars, n) {
                                            let else_body =
                                                slice_source(source, &bo, n + 1, else_close);

                                            if val {
                                                result.push_str(if_body);
                                            } else {
                                                result.push_str(else_body);
                                            }
                                            i = else_close + 1;
                                            continue;
                                        }
                                    }
                                    // else if (...) — don't handle, fall through
                                } else {
                                    // No else clause
                                    if val {
                                        result.push_str(if_body);
                                    }
                                    // else: dead block, just skip it
                                    i = close_brace + 1;
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
        }

        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Find matching closing paren, handling nested parens and strings.
/// All positions are char indices.
fn find_matching_paren(chars: &[char], open_pos: usize) -> Option<usize> {
    let mut depth = 0;
    let mut in_string = false;
    let mut string_char = '"';
    let mut i = open_pos;

    while i < chars.len() {
        let ch = chars[i];

        if in_string {
            if ch == '\\' {
                i += 1;
            } else if ch == string_char {
                in_string = false;
            }
            i += 1;
            continue;
        }

        match ch {
            '"' | '\'' | '`' => {
                in_string = true;
                string_char = ch;
            }
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Eliminate ternary expressions with statically evaluable conditions.
/// `"production" !== "production" ? devExpr : prodExpr` → `prodExpr`
fn eliminate_ternaries(source: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = source.chars().collect();
    let bo = build_byte_offsets(source);
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Look for string literal comparison patterns before ?
        if chars[i] == '"' || chars[i] == '\'' {
            let quote = chars[i];
            // Find end of string
            let mut j = i + 1;
            while j < len && chars[j] != quote {
                if chars[j] == '\\' {
                    j += 1;
                }
                j += 1;
            }
            if j >= len {
                result.push(chars[i]);
                i += 1;
                continue;
            }
            let str_end = j + 1; // past closing quote

            // Check for comparison operator after string
            let mut k = str_end;
            while k < len && chars[k] == ' ' {
                k += 1;
            }

            let op_start = k;
            let ops = ["!==", "===", "!=", "=="];
            let mut found_op = None;
            for op in &ops {
                if k + op.len() <= len && slice_source(source, &bo, k, k + op.len()) == *op {
                    found_op = Some(*op);
                    break;
                }
            }

            if let Some(op) = found_op {
                let after_op = op_start + op.len();
                let mut m = after_op;
                while m < len && chars[m] == ' ' {
                    m += 1;
                }

                // Second string literal
                if m < len && (chars[m] == '"' || chars[m] == '\'') {
                    let q2 = chars[m];
                    let mut n = m + 1;
                    while n < len && chars[n] != q2 {
                        if chars[n] == '\\' {
                            n += 1;
                        }
                        n += 1;
                    }
                    if n < len {
                        let cond_end = n + 1;
                        let cond_str = slice_source(source, &bo, i, cond_end);

                        if let Some(val) = eval_condition(cond_str) {
                            // Look for ? after condition
                            let mut p = cond_end;
                            while p < len && chars[p] == ' ' {
                                p += 1;
                            }

                            if p < len && chars[p] == '?' {
                                // Find the : that separates true/false branches
                                if let Some((colon_pos, q_end)) =
                                    find_ternary_colon(&chars, &bo, source, p + 1)
                                {
                                    let true_expr =
                                        slice_source(source, &bo, p + 1, colon_pos).trim();
                                    let false_expr =
                                        slice_source(source, &bo, colon_pos + 1, q_end).trim();

                                    if val {
                                        result.push_str(true_expr);
                                    } else {
                                        result.push_str(false_expr);
                                    }
                                    i = q_end;
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
        }

        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Find the colon separator and end of a ternary expression.
/// Returns (colon_pos, end_pos) as char indices.
fn find_ternary_colon(
    chars: &[char],
    _bo: &[usize],
    _source: &str,
    start: usize,
) -> Option<(usize, usize)> {
    let len = chars.len();
    let mut depth = 0; // track nested ternaries
    let mut paren_depth = 0;
    let mut in_string = false;
    let mut string_char = '"';
    let mut i = start;
    let mut colon_pos = None;

    while i < len {
        let ch = chars[i];

        if in_string {
            if ch == '\\' {
                i += 2;
                continue;
            }
            if ch == string_char {
                in_string = false;
            }
            i += 1;
            continue;
        }

        match ch {
            '"' | '\'' | '`' => {
                in_string = true;
                string_char = ch;
            }
            '(' => paren_depth += 1,
            ')' => {
                if paren_depth > 0 {
                    paren_depth -= 1;
                } else if colon_pos.is_some() {
                    // End of ternary inside parens
                    return Some((colon_pos.unwrap(), i));
                }
            }
            '?' if paren_depth == 0 => depth += 1,
            ':' if paren_depth == 0 => {
                if depth > 0 {
                    depth -= 1;
                } else if colon_pos.is_none() {
                    colon_pos = Some(i);
                }
            }
            // Ternary ends at statement boundary
            ';' | ',' | '\n' if colon_pos.is_some() && paren_depth == 0 && depth == 0 => {
                return Some((colon_pos.unwrap(), i));
            }
            _ => {}
        }
        i += 1;
    }

    // End of source
    if let Some(cp) = colon_pos {
        Some((cp, len))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_condition_string_equal() {
        assert_eq!(
            eval_condition(r#""production" === "production""#),
            Some(true)
        );
        assert_eq!(
            eval_condition(r#""production" !== "production""#),
            Some(false)
        );
        assert_eq!(
            eval_condition(r#""production" === "development""#),
            Some(false)
        );
        assert_eq!(
            eval_condition(r#""production" !== "development""#),
            Some(true)
        );
    }

    #[test]
    fn test_eval_condition_bool() {
        assert_eq!(eval_condition("true"), Some(true));
        assert_eq!(eval_condition("false"), Some(false));
        assert_eq!(eval_condition("false === false"), Some(true));
    }

    #[test]
    fn test_dce_if_false_removed() {
        let input = r#"before(); if ("production" !== "production") { dead(); } after();"#;
        let output = eliminate_dead_code(input);
        assert!(!output.contains("dead()"));
        assert!(output.contains("before()"));
        assert!(output.contains("after()"));
    }

    #[test]
    fn test_dce_if_true_kept() {
        let input = r#"if ("production" === "production") { live(); }"#;
        let output = eliminate_dead_code(input);
        assert!(output.contains("live()"));
        assert!(!output.contains("if"));
    }

    #[test]
    fn test_dce_if_else_keeps_true_branch() {
        let input = r#"if ("production" === "production") { live(); } else { dead(); }"#;
        let output = eliminate_dead_code(input);
        assert!(output.contains("live()"));
        assert!(!output.contains("dead()"));
    }

    #[test]
    fn test_dce_if_else_keeps_false_branch() {
        let input = r#"if ("production" !== "production") { dead(); } else { live(); }"#;
        let output = eliminate_dead_code(input);
        assert!(output.contains("live()"));
        assert!(!output.contains("dead()"));
    }

    #[test]
    fn test_dce_ternary_false() {
        let input = r#"var x = "production" !== "production" ? devFn() : prodFn();"#;
        let output = eliminate_dead_code(input);
        assert!(output.contains("prodFn()"));
        assert!(!output.contains("devFn()"));
    }

    #[test]
    fn test_dce_ternary_true() {
        let input = r#"var x = "production" === "production" ? prodFn() : devFn();"#;
        let output = eliminate_dead_code(input);
        assert!(output.contains("prodFn()"));
        assert!(!output.contains("devFn()"));
    }

    #[test]
    fn test_dce_no_change_for_dynamic() {
        let input = r#"if (someVar === "production") { code(); }"#;
        let output = eliminate_dead_code(input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_dce_preserves_normal_code() {
        let input = "var x = 1;\nfunction foo() { return x + 1; }\n";
        let output = eliminate_dead_code(input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_dce_react_pattern() {
        // React's index.js pattern after define replacement
        let input = r#"if ("production" === "production") {
  module.exports = require('./cjs/react.production.min.js');
} else {
  module.exports = require('./cjs/react.development.js');
}"#;
        let output = eliminate_dead_code(input);
        assert!(output.contains("react.production.min.js"));
        assert!(!output.contains("react.development.js"));
    }

    #[test]
    fn test_dce_with_multibyte_utf8() {
        // Test that DCE handles multi-byte UTF-8 characters (✓ is 3 bytes)
        let input =
            r#"var x = "✓ done"; if ("production" !== "production") { dead(); } var y = "✓ ok";"#;
        let output = eliminate_dead_code(input);
        assert!(!output.contains("dead()"));
        assert!(output.contains("✓ done"));
        assert!(output.contains("✓ ok"));
    }
}
// CODEGEN-END
