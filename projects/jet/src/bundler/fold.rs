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

/// Fold define-produced literal string comparisons and the short-circuit
/// expressions they feed, per module, before tree shaking.
///
/// After define replacement, library dev guards look like
/// `"production" !== "production" && warnOnce(...)` — expression-level
/// shapes the whole-condition DCE pass cannot touch. Fold the comparison
/// to `!0`/`!1`, collapse `!1 && <chain>` / `!0 || <chain>` (short-circuit:
/// the right side never evaluates), drop `!0 &&` / `!1 ||` wrappers, and
/// sweep bare `!0;`/`!1;` statements. Anything that fails the conservative
/// boundary checks is left alone, and the whole result is parse-guarded.
pub fn fold_define_short_circuits(source: &str) -> String {
    if !source.contains("===") && !source.contains("!==") {
        return source.to_string();
    }
    let mut result = fold_literal_string_compares(source);
    for _ in 0..8 {
        let prev = result.clone();
        result = collapse_known_short_circuits(&result);
        if result == prev {
            break;
        }
    }
    result = sweep_bare_bool_statements(&result);
    if result != source && !crate::bundler::dce::js_parses_without_errors(&result) {
        return source.to_string();
    }
    result
}

/// `"<lit>" ===|!==|==|!= "<lit>"` → `!0` / `!1`, with conservative
/// expression-boundary checks on both sides.
fn fold_literal_string_compares(source: &str) -> String {
    let b = source.as_bytes();
    let len = b.len();
    let mut out = Vec::with_capacity(len);
    let mut i = 0usize;

    while i < len {
        if b[i] == b'`' {
            i = push_string(b, i, &mut out);
            continue;
        }
        if b[i] == b'/' && is_regex_ctx(out.last().copied().unwrap_or(0)) {
            i = push_regex(b, i, &mut out);
            continue;
        }
        if matches!(b[i], b'"' | b'\'') {
            // Left boundary: previous significant byte must start an
            // expression, never continue one.
            let prev = out
                .iter()
                .rev()
                .find(|c| !matches!(**c, b' ' | b'\t' | b'\r' | b'\n'))
                .copied()
                .unwrap_or(b'(');
            let left_ok = matches!(
                prev,
                b'(' | b',' | b';' | b'{' | b'}' | b'!' | b'&' | b'|' | b'?' | b':' | b'=' | b'['
            ) || prev_is_expression_keyword(&out);
            let lit1_end = skip_string(b, i);
            if !left_ok || lit1_end <= i + 1 {
                i = push_string(b, i, &mut out);
                continue;
            }
            // Operator
            let mut j = lit1_end;
            while j < len && matches!(b[j], b' ' | b'\t') {
                j += 1;
            }
            let (op_len, negated) = if b[j..].starts_with(b"===") {
                (3, false)
            } else if b[j..].starts_with(b"!==") {
                (3, true)
            } else if b[j..].starts_with(b"==") {
                (2, false)
            } else if b[j..].starts_with(b"!=") {
                (2, true)
            } else {
                i = push_string(b, i, &mut out);
                continue;
            };
            let mut k = j + op_len;
            while k < len && matches!(b[k], b' ' | b'\t') {
                k += 1;
            }
            if k >= len || !matches!(b[k], b'"' | b'\'') {
                i = push_string(b, i, &mut out);
                continue;
            }
            let lit2_end = skip_string(b, k);
            // Right boundary: the comparison must end the expression atom.
            let mut m = lit2_end;
            while m < len && matches!(b[m], b' ' | b'\t') {
                m += 1;
            }
            let right_ok = m >= len
                || matches!(
                    b[m],
                    b'&' | b'|' | b')' | b';' | b',' | b'?' | b':' | b'}' | b']'
                )
                || b[m..].starts_with(b"==")
                || b[m..].starts_with(b"!=");
            if !right_ok {
                i = push_string(b, i, &mut out);
                continue;
            }
            let lhs = &source[i + 1..lit1_end - 1];
            let rhs = &source[k + 1..lit2_end - 1];
            let truth = (lhs == rhs) != negated;
            out.extend_from_slice(if truth { b"!0" } else { b"!1" });
            i = lit2_end;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| source.to_string())
}

/// Whether the bytes already emitted end in a keyword that introduces an
/// expression — so a following string literal (`return"x"`, `typeof"x"`,
/// `case"x"`) is the left operand of a comparison, a valid fold position the
/// punctuation-only boundary check misses. In valid JS an identifier is never
/// directly followed by a string literal except after such a keyword, so this
/// only ever accepts genuine expression starts.
fn prev_is_expression_keyword(out: &[u8]) -> bool {
    let mut end = out.len();
    while end > 0 && matches!(out[end - 1], b' ' | b'\t' | b'\r' | b'\n') {
        end -= 1;
    }
    let mut start = end;
    while start > 0 {
        let c = out[start - 1];
        if c.is_ascii_alphanumeric() || c == b'_' || c == b'$' {
            start -= 1;
        } else {
            break;
        }
    }
    matches!(
        &out[start..end],
        b"return"
            | b"typeof"
            | b"case"
            | b"void"
            | b"delete"
            | b"throw"
            | b"in"
            | b"of"
            | b"do"
            | b"else"
            | b"yield"
            | b"await"
            | b"new"
    )
}

/// One round of short-circuit collapsing over known boolean atoms.
fn collapse_known_short_circuits(source: &str) -> String {
    let b = source.as_bytes();
    let len = b.len();
    let mut out = Vec::with_capacity(len);
    let mut i = 0usize;

    while i < len {
        match b[i] {
            b'"' | b'\'' | b'`' => {
                i = push_string(b, i, &mut out);
                continue;
            }
            b'/' if is_regex_ctx(out.last().copied().unwrap_or(0)) => {
                i = push_regex(b, i, &mut out);
                continue;
            }
            b'!' if i + 1 < len && matches!(b[i + 1], b'0' | b'1') => {
                // Boundary: `x!0` can't occur; still require the previous
                // significant byte to not be an identifier char.
                let prev = out
                    .iter()
                    .rev()
                    .find(|c| !matches!(**c, b' ' | b'\t' | b'\r' | b'\n'))
                    .copied()
                    .unwrap_or(b'(');
                // An identifier char before `!0`/`!1` means it continues an
                // expression (`x!0` can't occur) — UNLESS it ends an
                // expression-keyword like `return!1` / `case!0`, which the fold
                // produces from `return"a"!=="b"`. Those are genuine boolean
                // atoms and must still collapse.
                if is_id(prev) && !prev_is_expression_keyword(&out) {
                    out.push(b[i]);
                    i += 1;
                    continue;
                }
                // JS minified booleans: `!0` is true, `!1` is false.
                let truthy = b[i + 1] == b'0';
                let mut j = i + 2;
                while j < len && matches!(b[j], b' ' | b'\t') {
                    j += 1;
                }
                let falsy = !truthy;
                if falsy && b[j..].starts_with(b"&&") {
                    // `!1 && <chain>` → `!1` (RHS never evaluates).
                    if let Some(end) = scan_short_circuit_chain(b, j + 2, b"&&") {
                        out.extend_from_slice(b"!1");
                        i = end;
                        continue;
                    }
                } else if truthy && b[j..].starts_with(b"||") {
                    // `!0 || <chain>` → `!0`.
                    if let Some(end) = scan_short_circuit_chain(b, j + 2, b"||") {
                        out.extend_from_slice(b"!0");
                        i = end;
                        continue;
                    }
                } else if truthy && b[j..].starts_with(b"&&") {
                    // `!0 && X` → `X`.
                    i = j + 2;
                    continue;
                } else if falsy && b[j..].starts_with(b"||") {
                    // `!1 || X` → `X`.
                    i = j + 2;
                    continue;
                } else if j < len && b[j] == b'?' && !matches!(b.get(j + 1), Some(b'.')) {
                    // Constant ternary `!0 ? A : B` → A, `!1 ? A : B` → B.
                    // Conservatively require BOTH arms to be a single balanced
                    // operand (primary + postfixes) so the kept/dropped spans
                    // are unambiguous — this is what carries dead branches like
                    // styled-components' `!1?{<3.7KB error dictionary>}:{}`,
                    // whose unique English text gzips poorly. `?.` optional
                    // chaining is excluded above.
                    if let Some(conseq_end) = scan_operand(b, j + 1) {
                        let mut c = conseq_end;
                        while c < len && matches!(b[c], b' ' | b'\t') {
                            c += 1;
                        }
                        if c < len && b[c] == b':' {
                            if let Some(alt_end) = scan_operand(b, c + 1) {
                                let cs = j + 1;
                                if truthy {
                                    out.extend_from_slice(&b[cs..conseq_end]);
                                } else {
                                    out.extend_from_slice(&b[c + 1..alt_end]);
                                }
                                i = alt_end;
                                continue;
                            }
                        }
                    }
                    out.push(b[i]);
                    out.push(b[i + 1]);
                    i += 2;
                    continue;
                }
                out.push(b[i]);
                out.push(b[i + 1]);
                i += 2;
                continue;
            }
            _ => {}
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| source.to_string())
}

/// Scan one operand chain after `!1&&` / `!0||`: a unary-prefixed primary
/// with member/call/index/template postfixes, repeated across the SAME
/// operator (short-circuit keeps eating). Returns the end offset, or None
/// when the shape is unsupported (caller leaves the source untouched).
fn scan_short_circuit_chain(b: &[u8], mut i: usize, op: &[u8]) -> Option<usize> {
    let len = b.len();
    loop {
        i = scan_short_circuit_atom(b, i)?;

        // Comparisons bind tighter than `&&`/`||`, so they are part of the
        // RHS skipped by `!1 && rhs` / `!0 || rhs`. Without this,
        // `"production"!=="production"&&"undefined"!=typeof navigator&&...`
        // collapsed to the invalid remnant `!1!=typeof navigator&&...`.
        loop {
            let j = skip_hspace(b, i);
            if let Some(op_len) = comparison_operator_len(b, j) {
                i = scan_short_circuit_atom(b, j + op_len)?;
            } else {
                break;
            }
        }

        let j = skip_hspace(b, i);
        if j + 2 <= len && &b[j..j + 2] == op {
            i = j + 2;
            continue;
        }
        if op == b"||" && j + 2 <= len && &b[j..j + 2] == b"&&" {
            i = j + 2;
            continue;
        }
        if is_short_circuit_boundary(b, j) {
            return Some(i);
        }
        return None;
    }
}

fn scan_short_circuit_atom(b: &[u8], mut i: usize) -> Option<usize> {
    let len = b.len();
    // unary prefixes
    loop {
        i = skip_hspace(b, i);
        if i < len && matches!(b[i], b'!' | b'+' | b'-' | b'~') {
            i += 1;
        } else if i + 6 <= len && &b[i..i + 6] == b"typeof" {
            i += 6;
        } else if i + 4 <= len && &b[i..i + 4] == b"void" {
            i += 4;
        } else {
            break;
        }
    }
    if i >= len {
        return None;
    }
    // primary
    match b[i] {
        b'(' => i = skip_balanced(b, i, b'(', b')')?,
        b'[' => i = skip_balanced(b, i, b'[', b']')?,
        b'"' | b'\'' | b'`' => i = skip_string(b, i),
        c if is_id(c) || c.is_ascii_digit() => {
            while i < len && is_id(b[i]) {
                i += 1;
            }
        }
        _ => return None,
    }
    // postfixes
    loop {
        let save = i;
        i = skip_hspace(b, i);
        if i < len && b[i] == b'.' && i + 1 < len && is_id(b[i + 1]) {
            i += 1;
            while i < len && is_id(b[i]) {
                i += 1;
            }
            continue;
        }
        if i < len && b[i] == b'(' {
            i = skip_balanced(b, i, b'(', b')')?;
            continue;
        }
        if i < len && b[i] == b'[' {
            i = skip_balanced(b, i, b'[', b']')?;
            continue;
        }
        if i < len && b[i] == b'`' {
            i = skip_string(b, i);
            continue;
        }
        i = save;
        break;
    }
    Some(i)
}

fn skip_hspace(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && matches!(b[i], b' ' | b'\t') {
        i += 1;
    }
    i
}

fn comparison_operator_len(b: &[u8], i: usize) -> Option<usize> {
    if i + 3 <= b.len() && matches!(&b[i..i + 3], b"===" | b"!==") {
        return Some(3);
    }
    if i + 2 <= b.len() && matches!(&b[i..i + 2], b"==" | b"!=" | b"<=" | b">=") {
        return Some(2);
    }
    if i < b.len()
        && (b[i] == b'<' || b[i] == b'>')
        && !matches!(b.get(i + 1), Some(b'<' | b'>' | b'='))
    {
        return Some(1);
    }
    None
}

fn is_short_circuit_boundary(b: &[u8], i: usize) -> bool {
    i >= b.len()
        || matches!(b[i], b';' | b',' | b')' | b']' | b'}' | b':' | b'?')
        || (i + 2 <= b.len() && &b[i..i + 2] == b"||")
}

/// Scan exactly ONE operand: optional unary prefixes, a balanced primary
/// (`(...)`, `[...]`, `{...}`, string/template, or identifier/number), then
/// member/call/index/template postfixes. Returns the end offset, or None on
/// an unsupported shape. Unlike [`scan_short_circuit_chain`], it does NOT
/// continue across binary operators — the caller uses it to delimit the
/// arms of a constant ternary unambiguously.
fn scan_operand(b: &[u8], mut i: usize) -> Option<usize> {
    let len = b.len();
    // unary prefixes
    loop {
        while i < len && matches!(b[i], b' ' | b'\t') {
            i += 1;
        }
        if i < len && matches!(b[i], b'!' | b'+' | b'-' | b'~') {
            i += 1;
        } else if i + 6 <= len && &b[i..i + 6] == b"typeof" {
            i += 6;
        } else if i + 4 <= len && &b[i..i + 4] == b"void" {
            i += 4;
        } else {
            break;
        }
    }
    if i >= len {
        return None;
    }
    // primary
    match b[i] {
        b'(' => i = skip_balanced(b, i, b'(', b')')?,
        b'[' => i = skip_balanced(b, i, b'[', b']')?,
        b'{' => i = skip_balanced(b, i, b'{', b'}')?,
        b'"' | b'\'' | b'`' => i = skip_string(b, i),
        c if is_id(c) || c.is_ascii_digit() => {
            while i < len && is_id(b[i]) {
                i += 1;
            }
        }
        _ => return None,
    }
    // postfixes
    loop {
        let save = i;
        while i < len && matches!(b[i], b' ' | b'\t') {
            i += 1;
        }
        if i < len && b[i] == b'.' && i + 1 < len && is_id(b[i + 1]) {
            i += 1;
            while i < len && is_id(b[i]) {
                i += 1;
            }
            continue;
        }
        if i < len && b[i] == b'(' {
            i = skip_balanced(b, i, b'(', b')')?;
            continue;
        }
        if i < len && b[i] == b'[' {
            i = skip_balanced(b, i, b'[', b']')?;
            continue;
        }
        if i < len && b[i] == b'`' {
            i = skip_string(b, i);
            continue;
        }
        i = save;
        break;
    }
    Some(i)
}

/// Skip a balanced bracket pair, honoring strings/templates/regex inside.
fn skip_balanced(b: &[u8], start: usize, open: u8, close: u8) -> Option<usize> {
    debug_assert_eq!(b[start], open);
    let mut depth = 0usize;
    let mut i = start;
    let mut prev = b'(';
    while i < b.len() {
        match b[i] {
            b'"' | b'\'' | b'`' => {
                i = skip_string(b, i);
                prev = b'"';
                continue;
            }
            b'/' if is_regex_ctx(prev) => {
                let mut sink = Vec::new();
                i = push_regex(b, i, &mut sink);
                prev = b'/';
                continue;
            }
            c if c == open => depth += 1,
            c if c == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
        if !matches!(b[i], b' ' | b'\t' | b'\r' | b'\n') {
            prev = b[i];
        }
        i += 1;
    }
    None
}

/// Remove no-op `!0;` / `!1;` expression statements left by collapsing.
fn sweep_bare_bool_statements(source: &str) -> String {
    let b = source.as_bytes();
    let len = b.len();
    let mut out = Vec::with_capacity(len);
    let mut i = 0usize;
    while i < len {
        if matches!(b[i], b'"' | b'\'' | b'`') {
            i = push_string(b, i, &mut out);
            continue;
        }
        if b[i] == b'!' && i + 2 < len && matches!(b[i + 1], b'0' | b'1') && b[i + 2] == b';' {
            let prev = out
                .iter()
                .rev()
                .find(|c| !matches!(**c, b' ' | b'\t' | b'\r' | b'\n'))
                .copied()
                .unwrap_or(b';');
            if matches!(prev, b';' | b'{' | b'}') {
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| source.to_string())
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

pub(crate) fn is_id(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
}

/// Push a string literal into `out`, return index past closing quote.
/// Template literals are copied verbatim with full `${...}` awareness.
pub(crate) fn push_string(b: &[u8], start: usize, out: &mut Vec<u8>) -> usize {
    let q = b[start];
    if q == b'`' {
        let end = skip_template(b, start);
        out.extend_from_slice(&b[start..end]);
        return end;
    }
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
pub(crate) fn is_regex_ctx(prev: u8) -> bool {
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
pub(crate) fn push_regex(b: &[u8], start: usize, out: &mut Vec<u8>) -> usize {
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
pub(crate) fn skip_string(b: &[u8], start: usize) -> usize {
    let q = b[start];
    if q == b'`' {
        return skip_template(b, start);
    }
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

/// Skip a template literal, honoring `${...}` interpolations whose
/// expressions may contain quotes and NESTED template literals
/// (styled-components: `` `<style ${le([t && `nonce="${t}"`])}>` ``).
/// Stopping at the first inner backtick desynchronized the scanner and
/// later fold passes rewrote real string content as code.
pub(crate) fn skip_template(b: &[u8], start: usize) -> usize {
    debug_assert_eq!(b[start], b'`');
    let mut i = start + 1;
    while i < b.len() {
        match b[i] {
            b'\\' => i += 2,
            b'`' => return i + 1,
            b'$' if i + 1 < b.len() && b[i + 1] == b'{' => {
                i += 2;
                let mut depth = 1usize;
                while i < b.len() && depth > 0 {
                    match b[i] {
                        b'{' => {
                            depth += 1;
                            i += 1;
                        }
                        b'}' => {
                            depth -= 1;
                            i += 1;
                        }
                        b'"' | b'\'' | b'`' => i = skip_string(b, i),
                        b'\\' => i += 2,
                        _ => i += 1,
                    }
                }
            }
            _ => i += 1,
        }
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
        // Regex literals may contain quotes and backticks inside character
        // classes (styled-components: /[!"#$%&'()*+,./:;<=>?@[\\\]^`{|}~-]+/g).
        // Without this skip, the scanner started a fake string at the
        // in-class quote, every later quote paired off-by-one, and the
        // merge branch deleted real string content (`"+"` vanished from
        // e.indexOf("+") and concat chains).
        if b[i] == b'/' && is_regex_ctx(out.last().copied().unwrap_or(0)) {
            i = push_regex(b, i, &mut out);
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
        // Same regex-literal hazard as fold_string_concat: character
        // classes may contain quotes/backticks/digits.
        if b[i] == b'/' && is_regex_ctx(out.last().copied().unwrap_or(0)) {
            i = push_regex(b, i, &mut out);
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

    /// Env-gated probe: JET_FOLD_DEBUG_INPUT=<file> [JET_FOLD_DEBUG_OUT=<file>]
    /// runs fold_constants over a real bundle for corruption bisection.
    #[test]
    fn debug_fold_input() {
        let Ok(p) = std::env::var("JET_FOLD_DEBUG_INPUT") else {
            return;
        };
        let src = std::fs::read_to_string(p).unwrap();
        let out = fold_constants(&src);
        if let Ok(out_path) = std::env::var("JET_FOLD_DEBUG_OUT") {
            std::fs::write(&out_path, &out).unwrap();
            println!("wrote fold output to {out_path}");
        }
    }

    // ---- R4: string concat ----

    #[test]
    fn test_fold_string_concat() {
        assert_eq!(fold_constants(r#""hello "+"world""#), r#""hello world""#);
    }

    /// Short-circuit folding: define-produced dev guards collapse, code
    /// with real conditions survives untouched.
    #[test]
    fn test_fold_define_short_circuits() {
        // `"production" !== "production" && warn(...)` dies entirely.
        let src =
            r#"function f(){"production"!=="production"&&console.warn("dev only");return 1;}"#;
        let out = fold_define_short_circuits(src);
        assert!(!out.contains("console.warn"), "dev guard must fold: {out}");
        assert!(out.contains("return 1"), "live code stays: {out}");

        // `cmp ===` truthy keeps the RHS.
        let src2 = r#"var x="production"==="production"&&compute();"#;
        let out2 = fold_define_short_circuits(src2);
        assert!(out2.contains("var x=compute()"), "{out2}");

        // Chains collapse across the same operator.
        let src3 = r#"if(o){"production"!=="production"&&a(b,c).d&&e[f];}g();"#;
        let out3 = fold_define_short_circuits(src3);
        assert!(!out3.contains("a(b,c)"), "{out3}");
        assert!(out3.contains("g()"), "{out3}");

        // Non-literal comparisons survive.
        let src4 = r#"if(mode!=="production"&&warn()){x();}"#;
        assert_eq!(fold_define_short_circuits(src4), src4);

        // Mixed-operator chains fold to their short-circuit value:
        // ("a"==="b" && p) || q  ≡  !1 || q  ≡  q.
        let src5 = r#"var ok="a"==="b"&&p||q;"#;
        let out5 = fold_define_short_circuits(src5);
        assert_eq!(out5, r#"var ok=q;"#, "{out5}");

        // styled-components browser production guards chain a false literal
        // compare into a `typeof` comparison. The whole dead guard must fold;
        // leaving `!1!=typeof navigator` both keeps dev warnings and changes
        // runtime semantics.
        let src6 = r#"function f(){"production"!=="production"&&"undefined"!=typeof navigator&&"ReactNative"===navigator.product&&console.warn("dev");return 1;}"#;
        let out6 = fold_define_short_circuits(src6);
        assert!(!out6.contains("console.warn"), "{out6}");
        assert!(!out6.contains("!1!="), "{out6}");
        assert!(out6.contains("return 1"), "{out6}");

        // `||` has lower precedence than `&&`: `!0 || a && b` is wholly true.
        // The skipped RHS must include the `&&` chain, not leave `!0&&b`.
        let src7 = r#"var ok="production"==="production"||probe()&&side();"#;
        let out7 = fold_define_short_circuits(src7);
        assert_eq!(out7, r#"var ok=!0;"#, "{out7}");
    }

    #[test]
    fn test_constant_ternary_collapse() {
        // `"production"!=="production"` folds to `!1`, then the ternary
        // collapses to the false arm, dropping the dead dictionary —
        // the styled-components `ap=!1?{...}:{}` shape.
        let src = r#"var ap="production"!=="production"?{1:"err one",2:"err two"}:{};"#;
        let out = fold_define_short_circuits(src);
        assert_eq!(out, r#"var ap={};"#, "{out}");
        assert!(!out.contains("err one"), "dead consequent dropped: {out}");

        // Truthy keeps the consequent.
        let src2 = r#"var x="a"==="a"?keep(y):drop(z);"#;
        let out2 = fold_define_short_circuits(src2);
        assert_eq!(out2, r#"var x=keep(y);"#, "{out2}");

        // Optional chaining `?.` must NOT be treated as a ternary.
        let src3 = r#"var n="a"!=="a"||obj?.prop;"#;
        let out3 = fold_define_short_circuits(src3);
        assert!(
            out3.contains("obj?.prop"),
            "optional chain preserved: {out3}"
        );
    }

    /// Regression: regex literals whose character classes contain quotes
    /// and backticks must be skipped, not scanned as strings. The styled-
    /// components escape regex desynchronized quote pairing for the rest
    /// of the file and the concat-merge branch then deleted real string
    /// content (`"+"` vanished from `e.indexOf("+")`).
    #[test]
    fn test_fold_skips_regex_with_quotes_in_character_class() {
        let src = r##"const j=/[!"#$%&'()*+,./:;<=>?@[\\\]^`{|}~-]+/g,x=/(^-|-$)/g;function T(e){return e.replace(j,"-").replace(x,"")}if(-1===e.indexOf("+"))return;t.push(n+"+"+jt+"+"+o);"##;
        let out = fold_constants(src);
        assert!(
            out.contains(r##"/[!"#$%&'()*+,./:;<=>?@[\\\]^`{|}~-]+/g"##),
            "regex literal must survive verbatim: {out}"
        );
        assert!(
            out.contains(r#"e.indexOf("+")"#),
            "string args after the regex must stay intact: {out}"
        );
        assert!(
            out.contains(r#"n+"+"+jt+"+"+o"#),
            "concat chains with plus-sign strings must stay intact: {out}"
        );
    }

    /// Regression: template literals with nested templates inside `${...}`
    /// must be copied verbatim. The scanner used to stop at the first inner
    /// backtick, desynchronizing string state for the rest of the file, and
    /// later fold passes rewrote real string content as code
    /// (styled-components ServerStyleSheet/stylisPluginRSC corruption).
    #[test]
    fn test_fold_preserves_nested_template_literals() {
        let src = r#"const css=()=>`<style ${le([t&&`nonce="${t}"`,`${c}="!0"`].filter(Boolean)," ")}>${e}</style>`;t.push(n+`${jt}+`+o);t.push("a"+"b");"#;
        let out = fold_constants(src);
        assert!(
            out.contains(r#"`<style ${le([t&&`nonce="${t}"`,`${c}="!0"`].filter(Boolean)," ")}>${e}</style>`"#),
            "nested templates must survive verbatim: {out}"
        );
        assert!(
            out.contains(r#"t.push(n+`${jt}+`+o)"#),
            "template content after a nested template must stay intact: {out}"
        );
        assert!(out.contains(r#""ab""#), "plain concat still folds: {out}");
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
            eprintln!("Optional bundle fixture absent: no bundle found");
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
