// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
//! Constant folding (R4) and dead-code-after-return elimination (R5).
//!
//! R4: folds typeof, string concat, and bitwise ops on constant operands.
//! R5: removes statements after `return`/`throw` within the same block.

/// R4: Fold constant expressions in minified JavaScript.
///
/// Handles typeof, string concat, integer bitwise ops.
/// Loops until stable (max 3 passes).
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn fold_constants(source: &str) -> String {
    let mut result = source.to_string();
    for _ in 0..3 {
        let prev = result.clone();
        result = fold_typeof(&result);
        result = fold_string_concat(&result);
        result = fold_numeric_bitwise(&result);
        if result == prev {
            break;
        }
    }
    result
}

/// R5: Remove statements after `return`/`throw` within the same block.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn eliminate_dead_after_return(source: &str) -> String {
    let b = source.as_bytes();
    let len = b.len();
    let mut out = Vec::with_capacity(len);
    let mut i = 0;

    while i < len {
        // Skip string literals
        if matches!(b[i], b'"' | b'\'' | b'`') {
            i = push_string(b, i, &mut out);
            continue;
        }

        // Skip regex literals (prevents " inside /[..."]/g from corrupting state)
        if b[i] == b'/' && i + 1 < len && b[i + 1] != b'/' && b[i + 1] != b'*' {
            let prev = if out.is_empty() {
                0
            } else {
                *out.last().unwrap()
            };
            if is_regex_ctx(prev) {
                i = push_regex(b, i, &mut out);
                continue;
            }
        }

        let is_ret = i + 6 <= len
            && &b[i..i + 6] == b"return"
            && (i == 0 || (!is_id(b[i - 1]) && b[i - 1] != b'.'))
            && (i + 6 >= len || !is_id(b[i + 6]));
        let is_throw = !is_ret
            && i + 5 <= len
            && &b[i..i + 5] == b"throw"
            && (i == 0 || (!is_id(b[i - 1]) && b[i - 1] != b'.'))
            && (i + 5 >= len || !is_id(b[i + 5]));

        if is_ret || is_throw {
            // Only treat as unconditional if preceded by `{` or `;`
            // (braceless `if(cond)return;` has `)` before return — conditional, skip)
            let prev_nonws = {
                let mut p = i;
                while p > 0 && b[p - 1] == b' ' {
                    p -= 1;
                }
                if p > 0 {
                    b[p - 1]
                } else {
                    b'{'
                }
            };
            if !matches!(prev_nonws, b'{' | b';' | b':' | b'}') {
                out.push(b[i]);
                i += 1;
                continue;
            }

            let kw_len = if is_ret { 6 } else { 5 };
            // Find the `;` that ends this statement (tracking depth for nested braces/parens)
            let mut j = i + kw_len;
            let mut depth: i32 = 0;
            let mut semi = None;

            while j < len {
                match b[j] {
                    b'"' | b'\'' | b'`' => {
                        j = skip_string(b, j);
                        continue;
                    }
                    b'(' | b'[' | b'{' => depth += 1,
                    b')' | b']' => {
                        if depth > 0 {
                            depth -= 1;
                        }
                    }
                    b'}' => {
                        if depth == 0 {
                            break; // closing brace of block, no semicolon found
                        }
                        depth -= 1;
                    }
                    b';' if depth == 0 => {
                        semi = Some(j);
                        break;
                    }
                    _ => {}
                }
                j += 1;
            }

            if let Some(sp) = semi {
                // Push the return/throw statement including `;`
                out.extend_from_slice(&b[i..=sp]);
                // Skip dead code until `}` at depth 0, but stop at switch labels
                let mut k = sp + 1;
                let mut dd: i32 = 0;
                while k < len {
                    match b[k] {
                        b'"' | b'\'' | b'`' => {
                            k = skip_string(b, k);
                            continue;
                        }
                        b'{' => dd += 1,
                        b'}' if dd == 0 => break,
                        b'}' => dd -= 1,
                        _ => {
                            // Stop at `case` or `default` labels at depth 0
                            // (switch case labels are reachable via the switch)
                            if dd == 0 && (k == 0 || !is_id(b[k - 1])) {
                                if k + 5 <= len && &b[k..k + 4] == b"case" && !is_id(b[k + 4]) {
                                    break;
                                }
                                if k + 8 <= len && &b[k..k + 7] == b"default" && !is_id(b[k + 7]) {
                                    break;
                                }
                            }
                        }
                    }
                    k += 1;
                }
                i = k; // resume at `}` or switch label
                continue;
            }
        }

        out.push(b[i]);
        i += 1;
    }

    String::from_utf8(out).unwrap_or_else(|_| source.to_string())
}

// ---- helpers ----

fn is_id(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
}

/// Push a string literal into `out`, return index past closing quote.
fn push_string(b: &[u8], start: usize, out: &mut Vec<u8>) -> usize {
    let q = b[start];
    out.push(q);
    let mut i = start + 1;
    while i < b.len() {
        if b[i] == b'\\' {
            out.push(b[i]);
            i += 1;
            if i < b.len() {
                out.push(b[i]);
                i += 1;
            }
            continue;
        }
        out.push(b[i]);
        if b[i] == q {
            return i + 1;
        }
        i += 1;
    }
    i
}

/// Heuristic: `/` starts a regex after these chars (not after ident/num/`)`/`]`).
fn is_regex_ctx(prev: u8) -> bool {
    matches!(
        prev,
        b'=' | b'('
            | b','
            | b'['
            | b'!'
            | b'&'
            | b'|'
            | b'?'
            | b':'
            | b';'
            | b'{'
            | b'}'
            | 0
            | b'<'
            | b'>'
            | b'+'
            | b'-'
            | b'*'
            | b'%'
            | b'^'
            | b'~'
    )
}

/// Push a regex literal into `out`, return index past flags.
fn push_regex(b: &[u8], start: usize, out: &mut Vec<u8>) -> usize {
    out.push(b[start]); // opening /
    let mut i = start + 1;
    while i < b.len() && b[i] != b'/' {
        if b[i] == b'\\' {
            out.push(b[i]);
            i += 1;
            if i < b.len() {
                out.push(b[i]);
                i += 1;
            }
            continue;
        }
        if b[i] == b'[' {
            // Character class — skip until ]
            out.push(b[i]);
            i += 1;
            while i < b.len() && b[i] != b']' {
                if b[i] == b'\\' {
                    out.push(b[i]);
                    i += 1;
                    if i < b.len() {
                        out.push(b[i]);
                        i += 1;
                    }
                    continue;
                }
                out.push(b[i]);
                i += 1;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    if i < b.len() {
        out.push(b[i]); // closing /
        i += 1;
    }
    // Skip flags
    while i < b.len() && b[i].is_ascii_alphabetic() {
        out.push(b[i]);
        i += 1;
    }
    i
}

/// Skip a string literal, return index past closing quote.
fn skip_string(b: &[u8], start: usize) -> usize {
    let q = b[start];
    let mut i = start + 1;
    while i < b.len() {
        if b[i] == b'\\' {
            i += 2;
            continue;
        }
        if b[i] == q {
            return i + 1;
        }
        i += 1;
    }
    i
}

// ---- R4 sub-passes ----

/// typeof "x" → "string", typeof 42 → "number", etc.
fn fold_typeof(source: &str) -> String {
    let b = source.as_bytes();
    let len = b.len();
    let mut out = Vec::with_capacity(len);
    let mut i = 0;

    while i < len {
        if matches!(b[i], b'"' | b'\'' | b'`') {
            i = push_string(b, i, &mut out);
            continue;
        }

        if i + 6 <= len
            && &b[i..i + 6] == b"typeof"
            && (i == 0 || !is_id(b[i - 1]))
            && (i + 6 >= len || !is_id(b[i + 6]))
        {
            let mut j = i + 6;
            if j < len && b[j] == b' ' {
                j += 1;
            }

            // typeof "..." → "string"
            if j < len && matches!(b[j], b'"' | b'\'') {
                let end = skip_string(b, j);
                out.extend_from_slice(b"\"string\"");
                i = end;
                continue;
            }
            // typeof 123 → "number"
            if j < len && b[j].is_ascii_digit() {
                let mut k = j;
                while k < len && (b[k].is_ascii_digit() || b[k] == b'.') {
                    k += 1;
                }
                if k > j && (k >= len || !is_id(b[k])) {
                    out.extend_from_slice(b"\"number\"");
                    i = k;
                    continue;
                }
            }
            // typeof undefined → "undefined"
            if j + 9 <= len && &b[j..j + 9] == b"undefined" && (j + 9 >= len || !is_id(b[j + 9])) {
                out.extend_from_slice(b"\"undefined\"");
                i = j + 9;
                continue;
            }
            // typeof null → "object"
            if j + 4 <= len && &b[j..j + 4] == b"null" && (j + 4 >= len || !is_id(b[j + 4])) {
                out.extend_from_slice(b"\"object\"");
                i = j + 4;
                continue;
            }
        }

        out.push(b[i]);
        i += 1;
    }

    String::from_utf8(out).unwrap_or_else(|_| source.to_string())
}

/// "a"+"b" → "ab" (same quote char only).
fn fold_string_concat(source: &str) -> String {
    let b = source.as_bytes();
    let len = b.len();
    let mut out = Vec::with_capacity(len);
    let mut i = 0;

    while i < len {
        if matches!(b[i], b'"' | b'\'') {
            let q = b[i];
            let s1_content_start = i + 1;
            let s1_end = skip_string(b, i); // index past closing quote
            if s1_end <= i + 1 {
                out.push(b[i]);
                i += 1;
                continue;
            }
            let s1_close = s1_end - 1; // index of closing quote

            // Check for + followed by same-quote string
            if s1_end < len && b[s1_end] == b'+' && s1_end + 1 < len && b[s1_end + 1] == q {
                let s2_content_start = s1_end + 2;
                let s2_end = skip_string(b, s1_end + 1);
                if s2_end > s1_end + 2 {
                    let s2_close = s2_end - 1;
                    out.push(q);
                    out.extend_from_slice(&b[s1_content_start..s1_close]);
                    out.extend_from_slice(&b[s2_content_start..s2_close]);
                    out.push(q);
                    i = s2_end;
                    continue;
                }
            }

            // Not a concat, push original string
            i = push_string(b, i, &mut out);
            continue;
        }
        if b[i] == b'`' {
            i = push_string(b, i, &mut out);
            continue;
        }

        out.push(b[i]);
        i += 1;
    }

    String::from_utf8(out).unwrap_or_else(|_| source.to_string())
}

/// Fold integer bitwise ops: 8|4 → 12 (only |, &, ^ with safe preceding context).
fn fold_numeric_bitwise(source: &str) -> String {
    let b = source.as_bytes();
    let len = b.len();
    let mut out = Vec::with_capacity(len);
    let mut i = 0;

    while i < len {
        if matches!(b[i], b'"' | b'\'' | b'`') {
            i = push_string(b, i, &mut out);
            continue;
        }

        if b[i].is_ascii_digit() {
            // Only fold when preceded by a safe context (expression-start position)
            let safe = out.is_empty()
                || matches!(
                    *out.last().unwrap(),
                    b'|' | b'&'
                        | b'^'
                        | b'('
                        | b'['
                        | b'{'
                        | b','
                        | b';'
                        | b'='
                        | b':'
                        | b'?'
                        | b'!'
                );

            if safe {
                let n1_start = i;
                while i < len && b[i].is_ascii_digit() {
                    i += 1;
                }
                let n1_end = i;

                if i < len && matches!(b[i], b'|' | b'&' | b'^') {
                    let op = b[i];
                    let after_op = i + 1;

                    if after_op < len && b[after_op].is_ascii_digit() {
                        let n2_start = after_op;
                        let mut j = after_op;
                        while j < len && b[j].is_ascii_digit() {
                            j += 1;
                        }
                        // Ensure not followed by identifier char (e.g. 0x...)
                        if j >= len || !is_id(b[j]) {
                            let s1 = std::str::from_utf8(&b[n1_start..n1_end]).unwrap();
                            let s2 = std::str::from_utf8(&b[n2_start..j]).unwrap();
                            if let (Ok(a), Ok(bv)) = (s1.parse::<i64>(), s2.parse::<i64>()) {
                                let r = match op {
                                    b'|' => a | bv,
                                    b'&' => a & bv,
                                    b'^' => a ^ bv,
                                    _ => unreachable!(),
                                };
                                out.extend_from_slice(r.to_string().as_bytes());
                                i = j;
                                continue;
                            }
                        }
                    }
                }

                // Not foldable, push the digits
                out.extend_from_slice(&b[n1_start..i]);
                continue;
            }
        }

        out.push(b[i]);
        i += 1;
    }

    String::from_utf8(out).unwrap_or_else(|_| source.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- R4: typeof folding ----

    #[test]
    fn test_fold_typeof_string() {
        assert_eq!(fold_constants(r#"typeof "hello""#), r#""string""#);
    }

    #[test]
    fn test_fold_typeof_number() {
        assert_eq!(fold_constants("typeof 42"), "\"number\"");
    }

    #[test]
    fn test_fold_typeof_undefined() {
        assert_eq!(fold_constants("typeof undefined"), "\"undefined\"");
    }

    #[test]
    fn test_fold_typeof_null() {
        assert_eq!(fold_constants("typeof null"), "\"object\"");
    }

    #[test]
    fn test_fold_typeof_in_string_preserved() {
        assert_eq!(fold_constants(r#""typeof x""#), r#""typeof x""#);
    }

    #[test]
    fn test_fold_typeof_variable_unchanged() {
        assert_eq!(fold_constants("typeof x"), "typeof x");
    }

    // ---- R4: string concat ----

    #[test]
    fn test_fold_string_concat() {
        assert_eq!(fold_constants(r#""hello "+"world""#), r#""hello world""#);
    }

    #[test]
    fn test_fold_string_concat_chain() {
        assert_eq!(fold_constants(r#""a"+"b"+"c""#), r#""abc""#);
    }

    #[test]
    fn test_fold_string_concat_single_quotes() {
        assert_eq!(fold_constants("'a'+'b'"), "'ab'");
    }

    #[test]
    fn test_fold_no_mixed_quote_concat() {
        // Different quote types should not concat
        let input = r#""a"+'b'"#;
        assert_eq!(fold_constants(input), input);
    }

    // ---- R4: numeric bitwise ----

    #[test]
    fn test_fold_bitwise_or() {
        assert_eq!(fold_constants("(8|4)"), "(12)");
    }

    #[test]
    fn test_fold_bitwise_and() {
        assert_eq!(fold_constants("(15&6)"), "(6)");
    }

    #[test]
    fn test_fold_bitwise_xor() {
        assert_eq!(fold_constants("(5^3)"), "(6)");
    }

    #[test]
    fn test_fold_bitwise_unsafe_context_skipped() {
        // After `>`, folding would change semantics
        let input = "a>8|4";
        assert_eq!(fold_constants(input), input);
    }

    // ---- R5: dead after return ----

    #[test]
    fn test_dead_after_return() {
        let input = "function f(){return 1;var x=2;}";
        assert_eq!(
            eliminate_dead_after_return(input),
            "function f(){return 1;}"
        );
    }

    #[test]
    fn test_dead_after_throw() {
        let input = "function f(){throw new Error();cleanup();}";
        assert_eq!(
            eliminate_dead_after_return(input),
            "function f(){throw new Error();}"
        );
    }

    #[test]
    fn test_dead_in_nested_block() {
        let input = "function f(){if(x){return 1;dead();}live();}";
        assert_eq!(
            eliminate_dead_after_return(input),
            "function f(){if(x){return 1;}live();}"
        );
    }

    #[test]
    fn test_no_dead_code() {
        let input = "function f(){var x=1;return x;}";
        assert_eq!(eliminate_dead_after_return(input), input);
    }

    #[test]
    fn test_return_in_string_ignored() {
        let input = r#"var s="return 1;dead()";live();"#;
        assert_eq!(eliminate_dead_after_return(input), input);
    }

    #[test]
    fn test_void_return_dead_code() {
        let input = "function f(){return;dead();}";
        assert_eq!(eliminate_dead_after_return(input), "function f(){return;}");
    }

    #[test]
    fn test_nested_return_handled() {
        let input = "function f(){function g(){return 1;dead();}return g();}";
        assert_eq!(
            eliminate_dead_after_return(input),
            "function f(){function g(){return 1;}return g();}"
        );
    }

    #[test]
    fn test_return_object_literal() {
        let input = "function f(){return{a:1};dead();}";
        assert_eq!(
            eliminate_dead_after_return(input),
            "function f(){return{a:1};}"
        );
    }

    #[test]
    fn test_switch_case_preserved() {
        let input = "switch(a){case 0:return 1;case 1:return 2;}";
        assert_eq!(eliminate_dead_after_return(input), input);
    }

    #[test]
    fn test_switch_default_preserved() {
        let input = "switch(a){case 0:return 1;default:return 2;}";
        assert_eq!(eliminate_dead_after_return(input), input);
    }

    #[test]
    fn test_switch_dead_code_before_case() {
        let input = "switch(a){case 0:return 1;dead();case 1:return 2;}";
        assert_eq!(
            eliminate_dead_after_return(input),
            "switch(a){case 0:return 1;case 1:return 2;}"
        );
    }

    #[test]
    fn test_pipeline_sizes() {
        let bundle_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("examples/react-bench/dist/_debug_preminify.js");
        if !bundle_path.exists() {
            // Try the unminified dist-debug bundle
            let alt = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("examples/react-bench/dist-debug");
            if alt.exists() {
                if let Some(entry) = std::fs::read_dir(&alt).ok().and_then(|mut d| {
                    d.find(|e| {
                        e.as_ref().ok().map_or(false, |e| {
                            e.path().extension().map_or(false, |ext| ext == "js")
                                && !e.path().to_string_lossy().contains(".map")
                        })
                    })
                }) {
                    let path = entry.unwrap().path();
                    let source = std::fs::read_to_string(&path).unwrap();
                    run_pipeline_sizes(&source);
                    return;
                }
            }
            eprintln!("Skipping: no bundle found");
            return;
        }
        let source = std::fs::read_to_string(&bundle_path).unwrap();
        run_pipeline_sizes(&source);
    }

    fn run_pipeline_sizes(source: &str) {
        use crate::bundler::mangle::mangle_variables;
        use crate::bundler::minify::{minify_js, replace_bool_literals, DropStatement};

        let drops = &[DropStatement::Console, DropStatement::Debugger];
        eprintln!("  [0] raw:           {} bytes", source.len());

        let s1 = minify_js(source, drops);
        eprintln!("  [1] minify:        {} bytes", s1.len());

        let s2 = replace_bool_literals(&s1);
        eprintln!("  [2] +bool:         {} bytes", s2.len());

        let s3 = mangle_variables(&s2);
        eprintln!("  [3] +mangle:       {} bytes", s3.len());

        let s4 = fold_constants(&s3);
        eprintln!("  [4] +fold:         {} bytes", s4.len());

        let s5 = eliminate_dead_after_return(&s4);
        eprintln!(
            "  [5] +dead-return:  {} bytes (removed {} bytes)",
            s5.len(),
            s4.len() - s5.len()
        );
    }

    #[test]
    fn test_nested_switch_preserved() {
        let input = "switch(a){case 0:switch(b){case 0:return 1;case 1:return 2;}return 3;case 1:return 4;}";
        let output = eliminate_dead_after_return(input);
        assert!(
            output.contains("case 1:return 2;"),
            "inner case 1 should be preserved, got: {}",
            output
        );
        assert!(
            output.contains("case 1:return 4;"),
            "outer case 1 should be preserved, got: {}",
            output
        );
    }

    #[test]
    fn test_conditional_return_preserved() {
        // Braceless if-return: code after it is NOT dead
        let input = "function f(){if(!b)return;var c=b.x;return c;}";
        let output = eliminate_dead_after_return(input);
        assert!(
            output.contains("var c=b.x"),
            "code after conditional return should be preserved, got: {}",
            output
        );
    }

    #[test]
    fn test_else_return_preserved() {
        // else return: code after is live
        let input = "function f(){if(a)x();else return;y();}";
        let output = eliminate_dead_after_return(input);
        assert!(
            output.contains("y()"),
            "code after else return should be preserved, got: {}",
            output
        );
    }
}
// CODEGEN-END
