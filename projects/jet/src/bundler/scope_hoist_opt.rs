// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
//! Post-flattening optimizations for scope-hoisted bundles.
//!
//! R4: Cross-module constant inlining — propagate immutable bindings
//!     across module boundaries in the flattened scope.
//! R5: Unified cross-module DCE — eliminate unused exports and dead
//!     functions across the merged scope.
//! R6: sideEffects integration — use `sideEffects: false` from
//!     package.json to identify safe inlining candidates.

use regex::Regex;

use super::scope_hoist::is_id_cont_byte;
use super::CompiledModule;

// ──────────────────────────────────────────────────────────────────────────
// R4: Cross-module constant inlining
// ──────────────────────────────────────────────────────────────────────────

/// Inline cross-module constants in a flattened bundle.
///
/// After `generate_flattened_bundle` produces the merged output, scans for
/// `var _m{i}_NAME = <literal>;` patterns where the initializer is a string,
/// number, or boolean literal. Replaces all references to `_m{i}_NAME` with
/// the literal value. Removes the now-unused `var` declaration line.
///
/// Only applies to bindings that were originally `const` declarations, which
/// are identified by the `_m{i}_` prefix pattern (all flattened const bindings
/// pass through the prefix renaming in `inline_module_body_v2`).
///
/// Literals recognized:
/// - String: `"..."` or `'...'`
/// - Number: integer or float (optionally negative)
/// - Boolean: `true` or `false`
/// - `null`, `undefined`, `void 0`
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn inline_cross_module_constants(code: &str) -> String {
    // Match: var _m{i}_{name} = <literal>;
    // Captures: (1) full var name, (2) literal value
    let re = Regex::new(
        r#"var\s+(_m\d+_[a-zA-Z_$][a-zA-Z0-9_$]*)\s*=\s*("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?|true|false|null|undefined|void\s+0)\s*;"#,
    )
    .unwrap();

    // Phase 1: collect all constant bindings and their literal values
    let mut constants: Vec<(String, String)> = Vec::new();
    for cap in re.captures_iter(code) {
        let var_name = cap[1].to_string();
        let literal = cap[2].to_string();
        let Some(decl_match) = cap.get(0) else {
            continue;
        };

        // Only inline if the binding has more than 0 read references in the code
        // (otherwise it's dead and will be cleaned up by R5 DCE)
        // Count occurrences of the var name as a standalone identifier
        let count = count_identifier_refs(code, &var_name);
        // count >= 2 means: 1 for the declaration + at least 1 read reference
        if count >= 2
            && !has_mutating_identifier_ref(code, &var_name, decl_match.end())
            && !template_literal_contains_identifier(code, &var_name)
        {
            constants.push((var_name, literal));
        }
    }

    if constants.is_empty() {
        return code.to_string();
    }

    let mut result = code.to_string();

    // Phase 2: remove the var declarations FIRST (before replacing identifiers,
    // because `replace_identifier` would also replace the name in the LHS of the
    // declaration, making the removal pattern unmatchable).
    for (var_name, literal) in &constants {
        let decl_pattern = format!("var {}={};", var_name, literal);
        result = result.replace(&decl_pattern, "");
        // Also try with spaces around `=` and before `;`
        let decl_spaced = format!("var {} = {};", var_name, literal);
        result = result.replace(&decl_spaced, "");
    }

    // Phase 3: replace all remaining references with the literal value
    for (var_name, literal) in &constants {
        result = replace_identifier(&result, var_name, literal);
    }

    result
}

/// Count standalone identifier references (not preceded by `.` or part of a
/// longer identifier) in the given code.
fn count_identifier_refs(code: &str, ident: &str) -> usize {
    count_identifier_refs_in_range(code.as_bytes(), ident.as_bytes(), 0, code.len())
}

fn count_identifier_refs_in_range(b: &[u8], ident_bytes: &[u8], start: usize, end: usize) -> usize {
    let ident_len = ident_bytes.len();
    let len = end.min(b.len());
    let mut count = 0;
    let mut i = start.min(len);

    while i < len {
        // Skip plain string literals.
        if matches!(b[i], b'"' | b'\'') {
            i = skip_quoted_literal(b, i).min(len);
            continue;
        }

        // Template raw text is inert, but `${...}` expressions contain real
        // JS references. Scanning raw text as code lets quotes in CSS/theme
        // snippets corrupt the lexical state for the rest of the bundle.
        if b[i] == b'`' {
            let (next, refs) = scan_template_literal_expr_ranges(b, i, |expr_start, expr_end| {
                count_identifier_refs_in_range(b, ident_bytes, expr_start, expr_end)
            });
            count += refs;
            i = next.min(len);
            continue;
        }

        // Skip comments
        if b[i] == b'/' && i + 1 < len {
            if b[i + 1] == b'/' {
                while i < len && b[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if b[i + 1] == b'*' {
                i += 2;
                while i + 1 < len && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                if i + 1 < len {
                    i += 2;
                }
                continue;
            }
        }

        // Try to match identifier
        if i + ident_len <= len && &b[i..i + ident_len] == ident_bytes {
            // Check word boundaries
            let prev_ok = i == 0 || !is_id_cont_byte(b[i - 1]);
            let next_ok = i + ident_len >= len || !is_id_cont_byte(b[i + ident_len]);
            // Not preceded by '.'
            let not_prop = {
                let mut p = i;
                while p > 0 && matches!(b[p - 1], b' ' | b'\t') {
                    p -= 1;
                }
                p == 0 || b[p - 1] != b'.'
            };
            if prev_ok && next_ok && not_prop {
                count += 1;
                i += ident_len;
                continue;
            }
        }

        i += 1;
    }

    count
}

fn template_literal_contains_identifier(code: &str, ident: &str) -> bool {
    let b = code.as_bytes();
    let ident_bytes = ident.as_bytes();
    let ident_len = ident_bytes.len();
    let mut i = 0;

    while i < b.len() {
        if b[i] != b'`' {
            i += 1;
            continue;
        }

        i += 1;
        while i < b.len() {
            if b[i] == b'\\' {
                i += 2;
                continue;
            }
            if b[i] == b'`' {
                i += 1;
                break;
            }
            if i + ident_len <= b.len() && &b[i..i + ident_len] == ident_bytes {
                let prev_ok = i == 0 || !is_id_cont_byte(b[i - 1]);
                let next_ok = i + ident_len >= b.len() || !is_id_cont_byte(b[i + ident_len]);
                if prev_ok && next_ok {
                    return true;
                }
            }
            i += 1;
        }
    }

    false
}

fn has_mutating_identifier_ref(code: &str, ident: &str, scan_start: usize) -> bool {
    let b = code.as_bytes();
    let ident_bytes = ident.as_bytes();
    has_mutating_identifier_ref_in_range(b, ident_bytes, scan_start, b.len())
}

fn has_mutating_identifier_ref_in_range(
    b: &[u8],
    ident_bytes: &[u8],
    start: usize,
    end: usize,
) -> bool {
    let ident_len = ident_bytes.len();
    let len = end.min(b.len());
    let mut i = start.min(len);

    while i < len {
        // Skip plain string literals.
        if matches!(b[i], b'"' | b'\'') {
            i = skip_quoted_literal(b, i).min(len);
            continue;
        }

        if b[i] == b'`' {
            let (next, found) = scan_template_literal_expr_ranges(b, i, |expr_start, expr_end| {
                usize::from(has_mutating_identifier_ref_in_range(
                    b,
                    ident_bytes,
                    expr_start,
                    expr_end,
                ))
            });
            if found > 0 {
                return true;
            }
            i = next.min(len);
            continue;
        }

        // Skip comments
        if b[i] == b'/' && i + 1 < len {
            if b[i + 1] == b'/' {
                while i < len && b[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if b[i + 1] == b'*' {
                i += 2;
                while i + 1 < len && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                if i + 1 < len {
                    i += 2;
                }
                continue;
            }
        }

        if i + ident_len <= len && &b[i..i + ident_len] == ident_bytes {
            let prev_ok = i == 0 || !is_id_cont_byte(b[i - 1]);
            let next_ok = i + ident_len >= len || !is_id_cont_byte(b[i + ident_len]);
            let not_prop = {
                let mut p = i;
                while p > 0 && matches!(b[p - 1], b' ' | b'\t') {
                    p -= 1;
                }
                p == 0 || b[p - 1] != b'.'
            };

            if prev_ok && next_ok && not_prop {
                let mut next = i + ident_len;
                while next < len && matches!(b[next], b' ' | b'\t' | b'\r' | b'\n') {
                    next += 1;
                }
                if next < len {
                    if b[next] == b'=' && (next + 1 >= len || !matches!(b[next + 1], b'=' | b'>')) {
                        return true;
                    }
                    if next + 1 < len && matches!(b[next], b'+' | b'-') && b[next + 1] == b[next] {
                        return true;
                    }
                    if next + 1 < len
                        && matches!(
                            b[next],
                            b'+' | b'-'
                                | b'*'
                                | b'/'
                                | b'%'
                                | b'&'
                                | b'|'
                                | b'^'
                                | b'?'
                                | b'<'
                                | b'>'
                        )
                        && b[next + 1] == b'='
                    {
                        return true;
                    }
                }

                let mut prev = i;
                while prev > 0 && matches!(b[prev - 1], b' ' | b'\t' | b'\r' | b'\n') {
                    prev -= 1;
                }
                if prev >= 2 && matches!(b[prev - 1], b'+' | b'-') && b[prev - 2] == b[prev - 1] {
                    return true;
                }

                i += ident_len;
                continue;
            }
        }

        i += 1;
    }

    false
}

/// Replace all standalone identifier references (not inside strings, comments,
/// or property accesses) with the given replacement string.
fn replace_identifier(code: &str, ident: &str, replacement: &str) -> String {
    let b = code.as_bytes();
    let ident_bytes = ident.as_bytes();
    let ident_len = ident_bytes.len();
    let len = b.len();
    let mut out = Vec::with_capacity(len + 64);
    let mut i = 0;

    while i < len {
        // Skip string literals
        if matches!(b[i], b'"' | b'\'' | b'`') {
            let q = b[i];
            out.push(b[i]);
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

        // Skip comments
        if b[i] == b'/' && i + 1 < len {
            if b[i + 1] == b'/' {
                while i < len && b[i] != b'\n' {
                    out.push(b[i]);
                    i += 1;
                }
                continue;
            }
            if b[i + 1] == b'*' {
                out.push(b[i]);
                i += 1;
                out.push(b[i]);
                i += 1;
                while i + 1 < len && !(b[i] == b'*' && b[i + 1] == b'/') {
                    out.push(b[i]);
                    i += 1;
                }
                if i + 1 < len {
                    out.push(b[i]);
                    i += 1;
                    out.push(b[i]);
                    i += 1;
                }
                continue;
            }
        }

        // Try to match identifier at word boundary
        if i + ident_len <= len && &b[i..i + ident_len] == ident_bytes {
            let prev_ok = i == 0 || !is_id_cont_byte(b[i - 1]);
            let next_ok = i + ident_len >= len || !is_id_cont_byte(b[i + ident_len]);
            let not_prop = {
                let mut p = out.len();
                while p > 0 && matches!(out[p - 1], b' ' | b'\t') {
                    p -= 1;
                }
                p == 0 || out[p - 1] != b'.'
            };
            if prev_ok && next_ok && not_prop {
                out.extend_from_slice(replacement.as_bytes());
                i += ident_len;
                continue;
            }
        }

        out.push(b[i]);
        i += 1;
    }

    String::from_utf8(out).unwrap_or_else(|_| code.to_string())
}

// ──────────────────────────────────────────────────────────────────────────
// R5: Unified cross-module DCE
// ──────────────────────────────────────────────────────────────────────────

/// Eliminate unused exports and dead variables in a flattened bundle.
///
/// After constant inlining (R4), scans the flattened bundle for:
/// 1. `_m{i}e.NAME` assignment sites. If `_m{i}e.NAME` has zero read
///    references elsewhere in the bundle, remove the assignment statement.
/// 2. Prefixed variable declarations (`var _m{i}_NAME`) with zero remaining
///    references — remove the entire declaration.
///
/// Must compose with existing per-module `dce.rs` pass (which runs before
/// scope hoisting).
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn eliminate_unused_exports(code: &str) -> String {
    let mut result = code.to_string();

    // Phase 1: Remove unused _m{i}e.NAME export assignments
    // Match pattern: _m{i}e.NAME = <expr>;
    let export_assign_re = Regex::new(r"(_m\d+e)\.([a-zA-Z_$][a-zA-Z0-9_$]*)\s*=[^=]").unwrap();

    let mut exports_to_remove: Vec<(String, String)> = Vec::new();

    for cap in export_assign_re.captures_iter(&result) {
        let export_obj = cap[1].to_string();
        let export_name = cap[2].to_string();
        let full_ref = format!("{}.{}", export_obj, export_name);

        // Count read references (not the assignment site itself)
        // A "read" is `_m{i}e.NAME` not followed by `=` (or `==`/`===`)
        let read_count = count_export_reads(&result, &full_ref);

        if read_count == 0 {
            exports_to_remove.push((export_obj, export_name));
        }
    }

    // Remove unused export assignment statements
    for (export_obj, export_name) in &exports_to_remove {
        result = remove_export_assignment(&result, export_obj, export_name);
    }

    // Phase 2: Remove unused prefixed variable declarations
    // Match: var _m{i}_NAME = ...;  or  var _m{i}_NAME;
    let prefixed_var_re = Regex::new(r"var\s+(_m\d+_[a-zA-Z_$][a-zA-Z0-9_$]*)").unwrap();

    let mut vars_to_remove: Vec<String> = Vec::new();

    // Single-pass precomputation. Per-candidate full-bundle scans (a fresh
    // Regex::new in is_prefixed_require_binding plus a count_identifier_refs
    // sweep, per candidate) made this O(candidates x bundle size) — ~3s of
    // pure scanning on the antd corpus bundle with 2,226 candidates.
    let require_binding_re =
        Regex::new(r"\bvar\s+(_m\d+_[a-zA-Z_$][a-zA-Z0-9_$]*)\s*=\s*(?:_r|require)\s*\(")
            .unwrap();
    let require_bindings: std::collections::HashSet<String> = require_binding_re
        .captures_iter(&result)
        .map(|cap| cap[1].to_string())
        .collect();
    let prefixed_ref_counts = count_all_prefixed_identifier_refs(&result);

    // Collect candidates first from the current state
    for cap in prefixed_var_re.captures_iter(&result) {
        let var_name = cap[1].to_string();
        if require_bindings.contains(&var_name) {
            continue;
        }
        // Total references include the declaration itself; a count of 1
        // means the var is unused.
        if prefixed_ref_counts.get(&var_name).copied().unwrap_or(0) <= 1 {
            vars_to_remove.push(var_name);
        }
    }

    // Remove unused variable declarations
    for var_name in &vars_to_remove {
        result = remove_var_declaration(&result, var_name);
    }

    result
}

/// Count every `_m`-prefixed identifier occurrence in one lexical sweep
/// (same string/template/comment skipping as count_identifier_refs_in_range,
/// generalized to collect all candidates instead of matching one name).
fn count_all_prefixed_identifier_refs(code: &str) -> std::collections::HashMap<String, usize> {
    let mut counts = std::collections::HashMap::new();
    collect_prefixed_refs_in_range(code.as_bytes(), 0, code.len(), &mut counts);
    counts
}

fn collect_prefixed_refs_in_range(
    b: &[u8],
    start: usize,
    end: usize,
    counts: &mut std::collections::HashMap<String, usize>,
) {
    let len = end.min(b.len());
    let mut i = start.min(len);

    while i < len {
        if matches!(b[i], b'"' | b'\'') {
            i = skip_quoted_literal(b, i).min(len);
            continue;
        }
        if b[i] == b'`' {
            let (next, _) = scan_template_literal_expr_ranges(b, i, |expr_start, expr_end| {
                collect_prefixed_refs_in_range(b, expr_start, expr_end, counts);
                0
            });
            i = next.min(len);
            continue;
        }
        if b[i] == b'/' && i + 1 < len {
            if b[i + 1] == b'/' {
                while i < len && b[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if b[i + 1] == b'*' {
                i += 2;
                while i + 1 < len && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                if i + 1 < len {
                    i += 2;
                }
                continue;
            }
        }
        if is_id_cont_byte(b[i]) && !b[i].is_ascii_digit() {
            let ident_start = i;
            while i < len && is_id_cont_byte(b[i]) {
                i += 1;
            }
            let ident = &b[ident_start..i];
            if ident.len() > 2 && ident.starts_with(b"_m") {
                if let Ok(name) = std::str::from_utf8(ident) {
                    *counts.entry(name.to_string()).or_insert(0) += 1;
                }
            }
            continue;
        }
        i += 1;
    }
}

/// Count read references to an export property like `_m0e.foo`, excluding
/// assignment sites (where it's followed by `=` but not `==`).
fn count_export_reads(code: &str, full_ref: &str) -> usize {
    let b = code.as_bytes();
    let ref_bytes = full_ref.as_bytes();
    let (export_obj, export_name) = full_ref.rsplit_once('.').unwrap_or((full_ref, ""));
    let obj_bytes = export_obj.as_bytes();
    let module_id = module_id_from_export_obj(export_obj);
    let require_aliases = module_id
        .map(|id| require_aliases_for_module(code, id))
        .unwrap_or_default();
    count_export_reads_in_range(
        b,
        0,
        b.len(),
        ref_bytes,
        export_name,
        obj_bytes,
        module_id,
        &require_aliases,
    )
}

fn count_export_reads_in_range(
    b: &[u8],
    start: usize,
    end: usize,
    ref_bytes: &[u8],
    export_name: &str,
    obj_bytes: &[u8],
    module_id: Option<&str>,
    require_aliases: &[String],
) -> usize {
    let ref_len = ref_bytes.len();
    let len = end.min(b.len());
    let mut count = 0;
    let mut i = start.min(len);

    while i < len {
        // Skip plain string literals.
        if matches!(b[i], b'"' | b'\'') {
            i = skip_quoted_literal(b, i).min(len);
            continue;
        }

        if b[i] == b'`' {
            let (next, refs) = scan_template_literal_expr_ranges(b, i, |expr_start, expr_end| {
                count_export_reads_in_range(
                    b,
                    expr_start,
                    expr_end,
                    ref_bytes,
                    export_name,
                    obj_bytes,
                    module_id,
                    require_aliases,
                )
            });
            count += refs;
            i = next.min(len);
            continue;
        }

        // Skip comments
        if b[i] == b'/' && i + 1 < len {
            if b[i + 1] == b'/' {
                while i < len && b[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if b[i + 1] == b'*' {
                i += 2;
                while i + 1 < len && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                if i + 1 < len {
                    i += 2;
                }
                continue;
            }
        }

        // Try to match dot export reads: `_m0e.foo`.
        if i + ref_len <= len && &b[i..i + ref_len] == ref_bytes {
            // Check it's not part of a longer identifier
            let next_ok = i + ref_len >= len || !is_id_cont_byte(b[i + ref_len]);
            if next_ok {
                // Check if this is an assignment (followed by `=` but not `==`)
                let mut j = i + ref_len;
                while j < len && matches!(b[j], b' ' | b'\t') {
                    j += 1;
                }
                let is_assignment = j < len && b[j] == b'=' && (j + 1 >= len || b[j + 1] != b'=');

                if !is_assignment {
                    count += 1;
                }
                i += ref_len;
                continue;
            }
        }

        // Try to match bracket export reads: `_m0e["foo"]` / `_m0e['foo']`.
        if !export_name.is_empty()
            && i + obj_bytes.len() <= len
            && &b[i..i + obj_bytes.len()] == obj_bytes
        {
            let prev_ok = i == 0 || !is_id_cont_byte(b[i - 1]);
            let next = i + obj_bytes.len();
            let next_ok = next >= len || !is_id_cont_byte(b[next]);
            if prev_ok && next_ok {
                if let Some((end_ref, is_assignment)) =
                    match_bracket_property_access(b, next, export_name)
                {
                    if !is_assignment {
                        count += 1;
                    }
                    i = end_ref;
                    continue;
                }
            }
        }

        // Try to match require export reads: `_r(672)["getContrastRatio"]`,
        // `_r(672).getContrastRatio`, or aliases such as
        // `var color = _r(672); color.getContrastRatio`.
        if let Some(id) = module_id {
            if let Some((end_ref, is_assignment)) =
                match_require_export_access(b, i, id, export_name)
            {
                if !is_assignment {
                    count += 1;
                }
                i = end_ref;
                continue;
            }
            if let Some((end_ref, is_assignment)) =
                match_require_alias_export_access(b, i, &require_aliases, export_name)
            {
                if !is_assignment {
                    count += 1;
                }
                i = end_ref;
                continue;
            }
        }

        i += 1;
    }

    count
}

fn module_id_from_export_obj(export_obj: &str) -> Option<&str> {
    let id = export_obj.strip_prefix("_m")?.strip_suffix('e')?;
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(id)
}

fn require_aliases_for_module(code: &str, module_id: &str) -> Vec<String> {
    let pattern = format!(
        r"(?:^|[;{{}}\n])\s*(?:var|let|const)\s+([a-zA-Z_$][a-zA-Z0-9_$]*)\s*=\s*_r\s*\(\s*{}\s*\)\s*;",
        regex::escape(module_id)
    );
    let re = Regex::new(&pattern).unwrap();
    re.captures_iter(code)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

fn match_bracket_property_access(
    b: &[u8],
    after_obj: usize,
    export_name: &str,
) -> Option<(usize, bool)> {
    let mut i = after_obj;
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= b.len() || b[i] != b'[' {
        return None;
    }
    i += 1;
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= b.len() || !matches!(b[i], b'"' | b'\'') {
        return None;
    }
    let quote = b[i];
    i += 1;
    let name_start = i;
    while i < b.len() {
        if b[i] == b'\\' {
            return None;
        }
        if b[i] == quote {
            break;
        }
        i += 1;
    }
    if i >= b.len() || &b[name_start..i] != export_name.as_bytes() {
        return None;
    }
    i += 1;
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= b.len() || b[i] != b']' {
        return None;
    }
    i += 1;

    let mut j = i;
    while j < b.len() && b[j].is_ascii_whitespace() {
        j += 1;
    }
    let is_assignment = j < b.len() && b[j] == b'=' && (j + 1 >= b.len() || b[j + 1] != b'=');
    Some((i, is_assignment))
}

fn match_require_export_access(
    b: &[u8],
    start: usize,
    module_id: &str,
    export_name: &str,
) -> Option<(usize, bool)> {
    let after_require = match_require_call_for_module(b, start, module_id)?;
    match_property_access_after_base(b, after_require, export_name)
}

fn match_require_alias_export_access(
    b: &[u8],
    start: usize,
    aliases: &[String],
    export_name: &str,
) -> Option<(usize, bool)> {
    for alias in aliases {
        let alias_bytes = alias.as_bytes();
        if start + alias_bytes.len() > b.len()
            || &b[start..start + alias_bytes.len()] != alias_bytes
        {
            continue;
        }
        let prev_ok = start == 0 || !is_id_cont_byte(b[start - 1]);
        let next = start + alias_bytes.len();
        let next_ok = next >= b.len() || !is_id_cont_byte(b[next]);
        if prev_ok && next_ok {
            if let Some(access) = match_property_access_after_base(b, next, export_name) {
                return Some(access);
            }
        }
    }
    None
}

fn match_require_call_for_module(b: &[u8], start: usize, module_id: &str) -> Option<usize> {
    let req = b"_r";
    if start + req.len() > b.len() || &b[start..start + req.len()] != req {
        return None;
    }
    if start > 0 && is_id_cont_byte(b[start - 1]) {
        return None;
    }
    let mut i = start + req.len();
    if i < b.len() && is_id_cont_byte(b[i]) {
        return None;
    }
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= b.len() || b[i] != b'(' {
        return None;
    }
    i += 1;
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    let id_bytes = module_id.as_bytes();
    if i + id_bytes.len() > b.len() || &b[i..i + id_bytes.len()] != id_bytes {
        return None;
    }
    i += id_bytes.len();
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= b.len() || b[i] != b')' {
        return None;
    }
    Some(i + 1)
}

fn match_property_access_after_base(
    b: &[u8],
    after_base: usize,
    export_name: &str,
) -> Option<(usize, bool)> {
    let mut i = after_base;
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }

    if let Some(end_exports) = match_dot_property(b, i, "exports") {
        i = end_exports;
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
    }

    if let Some(end_ref) = match_dot_property(b, i, export_name) {
        return Some((end_ref, is_assignment_after_ref(b, end_ref)));
    }

    match_bracket_property_access(b, i, export_name)
}

fn match_dot_property(b: &[u8], start: usize, property: &str) -> Option<usize> {
    if start >= b.len() || b[start] != b'.' {
        return None;
    }
    let mut i = start + 1;
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    let prop_bytes = property.as_bytes();
    if i + prop_bytes.len() > b.len() || &b[i..i + prop_bytes.len()] != prop_bytes {
        return None;
    }
    let end = i + prop_bytes.len();
    if end < b.len() && is_id_cont_byte(b[end]) {
        return None;
    }
    Some(end)
}

fn is_assignment_after_ref(b: &[u8], end_ref: usize) -> bool {
    let mut j = end_ref;
    while j < b.len() && b[j].is_ascii_whitespace() {
        j += 1;
    }
    j < b.len() && b[j] == b'=' && (j + 1 >= b.len() || b[j + 1] != b'=')
}

/// Remove an export assignment statement like `_m0e.foo = <expr>;` from code.
fn remove_export_assignment(code: &str, export_obj: &str, export_name: &str) -> String {
    let full_ref = format!("{}.{}", export_obj, export_name);
    let full_ref_bytes = full_ref.as_bytes();
    let b = code.as_bytes();
    let mut result = String::with_capacity(code.len());
    let mut cursor = 0;
    let mut i = 0;

    while let Some(relative_start) = code[i..].find(&full_ref) {
        let start = i + relative_start;
        let end_ref = start + full_ref_bytes.len();

        if !is_export_assignment_match(b, start, end_ref) {
            i = end_ref;
            continue;
        }

        let Some(statement_end) = find_assignment_statement_end(b, end_ref) else {
            i = end_ref;
            continue;
        };

        result.push_str(&code[cursor..start]);
        cursor = statement_end;
        i = statement_end;
    }

    result.push_str(&code[cursor..]);
    result
}

fn is_export_assignment_match(b: &[u8], start: usize, end_ref: usize) -> bool {
    if start > 0 && is_id_cont_byte(b[start - 1]) {
        return false;
    }
    let previous = previous_non_ws_byte(b, start);
    if !matches!(previous, None | Some(b';') | Some(b'{') | Some(b'}')) {
        return false;
    }

    let mut i = end_ref;
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }

    i < b.len() && b[i] == b'=' && (i + 1 >= b.len() || b[i + 1] != b'=')
}

fn previous_non_ws_byte(b: &[u8], before: usize) -> Option<u8> {
    let mut i = before;
    while i > 0 {
        i -= 1;
        if !b[i].is_ascii_whitespace() {
            return Some(b[i]);
        }
    }
    None
}

fn find_assignment_statement_end(b: &[u8], after_ref: usize) -> Option<usize> {
    let mut i = after_ref;
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= b.len() || b[i] != b'=' || (i + 1 < b.len() && b[i + 1] == b'=') {
        return None;
    }
    i += 1;

    let mut paren_depth = 0i32;
    let mut bracket_depth = 0i32;
    let mut brace_depth = 0i32;
    while i < b.len() {
        match b[i] {
            b'"' | b'\'' | b'`' => {
                i = skip_quoted_literal(b, i);
                continue;
            }
            b'/' if i + 1 < b.len() && b[i + 1] == b'/' => {
                i += 2;
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            b'/' if i + 1 < b.len() && b[i + 1] == b'*' => {
                i += 2;
                while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(b.len());
                continue;
            }
            b'(' => paren_depth += 1,
            b')' if paren_depth > 0 => paren_depth -= 1,
            b'[' => bracket_depth += 1,
            b']' if bracket_depth > 0 => bracket_depth -= 1,
            b'{' => brace_depth += 1,
            b'}' if brace_depth > 0 => brace_depth -= 1,
            b';' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                return Some(i + 1);
            }
            _ => {}
        }
        i += 1;
    }

    Some(b.len())
}

fn skip_quoted_literal(b: &[u8], start: usize) -> usize {
    let quote = b[start];
    let mut i = start + 1;
    while i < b.len() {
        if b[i] == b'\\' {
            i += 2;
            continue;
        }
        if b[i] == quote {
            return i + 1;
        }
        i += 1;
    }
    i
}

fn scan_template_literal_expr_ranges<F>(b: &[u8], start: usize, mut on_expr: F) -> (usize, usize)
where
    F: FnMut(usize, usize) -> usize,
{
    debug_assert!(start < b.len() && b[start] == b'`');
    let mut count = 0;
    let mut i = start + 1;

    while i < b.len() {
        match b[i] {
            b'\\' => {
                i = (i + 2).min(b.len());
            }
            b'`' => {
                return (i + 1, count);
            }
            b'$' if i + 1 < b.len() && b[i + 1] == b'{' => {
                let expr_start = i + 2;
                let expr_end = find_template_expression_end(b, expr_start);
                count += on_expr(expr_start, expr_end);
                i = (expr_end + 1).min(b.len());
            }
            _ => {
                i += 1;
            }
        }
    }

    (b.len(), count)
}

fn find_template_expression_end(b: &[u8], start: usize) -> usize {
    let mut i = start;
    let mut brace_depth = 0usize;

    while i < b.len() {
        match b[i] {
            b'"' | b'\'' => {
                i = skip_quoted_literal(b, i);
                continue;
            }
            b'`' => {
                i = scan_template_literal_expr_ranges(b, i, |_, _| 0).0;
                continue;
            }
            b'/' if i + 1 < b.len() && b[i + 1] == b'/' => {
                i += 2;
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            b'/' if i + 1 < b.len() && b[i + 1] == b'*' => {
                i += 2;
                while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                if i + 1 < b.len() {
                    i += 2;
                }
                continue;
            }
            b'{' => {
                brace_depth += 1;
            }
            b'}' => {
                if brace_depth == 0 {
                    return i;
                }
                brace_depth -= 1;
            }
            _ => {}
        }
        i += 1;
    }

    b.len()
}

/// Remove a `var _m{i}_NAME = <expr>;` or `var _m{i}_NAME;` declaration
/// from code.
fn remove_var_declaration(code: &str, var_name: &str) -> String {
    // Match: var NAME = <anything up to ;>;  or  var NAME;
    let pattern = format!(r"var\s+{}\s*(?:=[^;]*)?;", regex::escape(var_name));
    let re = Regex::new(&pattern).unwrap();
    re.replace_all(code, "").to_string()
}

// ──────────────────────────────────────────────────────────────────────────
// R6: sideEffects integration
// ──────────────────────────────────────────────────────────────────────────

/// Check if a compiled module is side-effect-free based on its source path.
///
/// Uses the `sideEffects` field from the owning package's `package.json`:
/// - `sideEffects: false` → module is side-effect-free (safe to inline)
/// - `sideEffects: true` or absent → check code heuristically
/// - `sideEffects: ["*.css", ...]` → side-effect-free unless path matches a glob
///
/// Modules with side effects must NOT be inlined during scope hoisting —
/// they retain their wrapper boundary to preserve execution order.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn is_side_effect_free(module: &CompiledModule) -> bool {
    use super::tree_shake::{has_side_effects, module_has_side_effects, read_package_side_effects};

    // Try to find the owning package's node_modules directory.
    // Walk up from the module path to find `node_modules/{package}/package.json`.
    let module_path = &module.path;
    if let Some(nm_and_pkg) = find_package_info(module_path) {
        let (node_modules_dir, package_name) = nm_and_pkg;
        let decl = read_package_side_effects(&node_modules_dir, &package_name);
        !module_has_side_effects(&module.code, module_path, &decl)
    } else {
        // Not inside node_modules — use heuristic code analysis.
        // Project source files are conservatively assumed to have side effects
        // unless analysis says otherwise.
        !has_side_effects(&module.code)
    }
}

/// Extract the `node_modules` directory path and the package name from a
/// module's absolute path.
///
/// For example:
///   `/project/node_modules/react/cjs/react.production.min.js`
///   → `("/project/node_modules", "react")`
///
///   `/project/node_modules/@scope/pkg/index.js`
///   → `("/project/node_modules", "@scope/pkg")`
fn find_package_info(module_path: &std::path::Path) -> Option<(std::path::PathBuf, String)> {
    let path_str = module_path.to_string_lossy();

    // Find the last `node_modules/` in the path
    let nm_marker = "node_modules/";
    let nm_pos = path_str.rfind(nm_marker)?;

    let node_modules_dir = std::path::PathBuf::from(&path_str[..nm_pos + nm_marker.len() - 1]);
    let after_nm = &path_str[nm_pos + nm_marker.len()..];

    // Extract package name: either `@scope/name` or `name`
    let package_name = if after_nm.starts_with('@') {
        // Scoped package: @scope/name
        let parts: Vec<&str> = after_nm.splitn(3, '/').collect();
        if parts.len() >= 2 {
            format!("{}/{}", parts[0], parts[1])
        } else {
            return None;
        }
    } else {
        // Regular package: name
        after_nm.split('/').next()?.to_string()
    };

    Some((node_modules_dir, package_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_module(path: &str, code: &str) -> CompiledModule {
        CompiledModule {
            id: 0,
            path: PathBuf::from(path),
            code: code.to_string(),
            source_map: None,
            dependencies: Vec::new(),
            hash: String::new(),
        }
    }

    // ──────────────────────────────────────────────────────────────────
    // R4: inline_cross_module_constants
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_inline_cross_module_constants_string() {
        // A const string binding `_m1_MODE = "production"` is referenced
        // in a conditional — after inlining, the reference is replaced
        // with the literal value.
        let code = r#"var _m1_MODE = "production";
if (_m1_MODE !== "production") { debugSetup(); }
console.log(_m1_MODE);"#;

        let result = inline_cross_module_constants(code);

        // The literal "production" should replace all references
        assert!(
            !result.contains("_m1_MODE"),
            "all references to _m1_MODE should be inlined, got: {}",
            result
        );
        // The inlined literal should appear in the conditional
        assert!(
            result.contains(r#""production" !== "production""#),
            "conditional should have inlined literal, got: {}",
            result
        );
        // The var declaration line should be removed
        assert!(
            !result.contains("var "),
            "var declaration should be removed, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_cross_module_constants_number() {
        // A const number binding is propagated to all usage sites.
        let code = "var _m0_MAX_SIZE = 1024;\nvar _m0_arr = new Array(_m0_MAX_SIZE);\nconsole.log(_m0_MAX_SIZE);";

        let result = inline_cross_module_constants(code);

        assert!(
            !result.contains("_m0_MAX_SIZE"),
            "all references to _m0_MAX_SIZE should be inlined, got: {}",
            result
        );
        assert!(
            result.contains("new Array(1024)"),
            "number literal should be propagated, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_cross_module_constants_skips_reassigned_binding() {
        let code = "var _m0_position = 0;\nfunction next(){ _m0_position = _m0_position + 1; return _m0_position; }";

        let result = inline_cross_module_constants(code);

        assert!(
            result.contains("_m0_position = _m0_position + 1"),
            "mutable binding must not be inlined into assignment targets, got: {}",
            result
        );
        assert!(
            result.contains("var _m0_position = 0"),
            "mutable binding declaration must remain, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_cross_module_constants_skips_incremented_binding() {
        let code = "var _m0_line = 1;\nfunction prev(){ _m0_line--; --_m0_line; return _m0_line; }";

        let result = inline_cross_module_constants(code);

        assert!(
            result.contains("_m0_line--") && result.contains("--_m0_line"),
            "increment/decrement targets must stay identifiers, got: {}",
            result
        );
        assert!(
            result.contains("var _m0_line = 1"),
            "incremented binding declaration must remain, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_cross_module_constants_boolean() {
        let code = "var _m2_DEV = false;\nif (_m2_DEV) { enableDevTools(); }\nvar _m2_x = _m2_DEV;";

        let result = inline_cross_module_constants(code);

        assert!(
            !result.contains("_m2_DEV"),
            "_m2_DEV should be inlined, got: {}",
            result
        );
        assert!(
            result.contains("if (false)"),
            "boolean literal should be propagated, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_cross_module_constants_no_inline_non_literal() {
        // Non-literal initializers (function calls, object expressions) must
        // NOT be inlined.
        let code = "var _m0_config = getConfig();\nconsole.log(_m0_config);";

        let result = inline_cross_module_constants(code);

        // Should remain unchanged — getConfig() is not a literal
        assert!(
            result.contains("_m0_config"),
            "non-literal binding should not be inlined, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_cross_module_constants_single_ref_not_inlined() {
        // A binding with only the declaration (no read references) should
        // NOT be inlined (it's dead code, handled by R5 DCE instead).
        let code = "var _m0_UNUSED = 42;\nconsole.log('hello');";

        let result = inline_cross_module_constants(code);

        // Only 1 reference (the declaration itself) — should not be inlined
        assert!(
            result.contains("_m0_UNUSED"),
            "unused binding should not be inlined by R4, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_cross_module_constants_preserves_strings() {
        // References inside string literals should NOT be replaced
        let code = r#"var _m0_NAME = "foo";
var _m0_msg = "the value of _m0_NAME is " + _m0_NAME;"#;

        let result = inline_cross_module_constants(code);

        // The string content should be preserved
        assert!(
            result.contains("\"the value of _m0_NAME is \""),
            "string content should not be modified, got: {}",
            result
        );
        // The identifier reference outside the string should be replaced
        assert!(
            result.contains("+ \"foo\""),
            "identifier reference should be replaced with literal, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_cross_module_constants_skips_template_literal_refs() {
        let code = r#"var _m0_prefix = "Mui";
const _m0_className = `${_m0_prefix}-Button`;"#;

        let result = inline_cross_module_constants(code);

        assert!(
            result.contains("var _m0_prefix = \"Mui\";")
                || result.contains("var _m0_prefix=\"Mui\";"),
            "template literal ref should keep the declaration, got: {}",
            result
        );
        assert!(
            result.contains("${_m0_prefix}-Button"),
            "template literal expression should stay intact, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_cross_module_constants_null_undefined() {
        let code = "var _m0_val = null;\nif (_m0_val) { doSomething(); }\nvar _m0_x = _m0_val;";

        let result = inline_cross_module_constants(code);

        assert!(
            !result.contains("_m0_val"),
            "null literal should be inlined, got: {}",
            result
        );
        assert!(
            result.contains("if (null)"),
            "null should replace the reference, got: {}",
            result
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // R5: eliminate_unused_exports
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_eliminate_unused_exports() {
        // An export assignment `_m0e.unusedFn = ...` with zero read references
        // in the bundle should be removed entirely.
        let code = r#"var _m0e = _m0.exports;
_m0e.usedFn = function() { return 42; };
_m0e.unusedFn = function() { return 99; };
var _m1_result = _m0e.usedFn();"#;

        let result = eliminate_unused_exports(code);

        // usedFn is referenced → must survive
        assert!(
            result.contains("_m0e.usedFn"),
            "used export should survive, got: {}",
            result
        );
        // unusedFn has no read reference → should be removed
        assert!(
            !result.contains("unusedFn"),
            "unused export should be removed, got: {}",
            result
        );
    }

    #[test]
    fn test_eliminate_unused_exports_removes_whole_function_assignment() {
        // Function-valued exports contain semicolons in their body. The export
        // remover must delete the whole assignment, not only up to the first
        // `return ...;` inside the function.
        let code = r#"var _m0e = _m0.exports;
_m0e.usedFn = function() { return 42; };
_m0e.unusedFn = function(value) {
  if (value) {
    return "yes";
  }
  return "no";
};
var _m1_result = _m0e.usedFn();"#;

        let result = eliminate_unused_exports(code);

        assert!(
            result.contains("_m0e.usedFn"),
            "used export should survive, got: {}",
            result
        );
        assert!(
            !result.contains("unusedFn"),
            "unused export name should be removed, got: {}",
            result
        );
        assert!(
            !result.contains("return \"no\";\n};"),
            "unused function tail must not be left behind, got: {}",
            result
        );
    }

    #[test]
    fn test_eliminate_unused_exports_preserves_chained_assignment_initializer() {
        // MUI emits `const local = exports.name = createTheme();`. The export
        // write can be unused, but the initializer is still the local binding's
        // value and must not be removed as a standalone export assignment.
        let code = r#"const _m0_systemDefaultTheme = _m0e.systemDefaultTheme = _m0_createTheme();
function _m0_readTheme() { return _m0_systemDefaultTheme; }
_m0_readTheme();"#;

        let result = eliminate_unused_exports(code);

        assert!(
            result.contains(
                "const _m0_systemDefaultTheme = _m0e.systemDefaultTheme = _m0_createTheme();"
            ),
            "chained assignment initializer must survive, got: {}",
            result
        );
        assert!(
            !result.contains("const _m0_systemDefaultTheme =\n"),
            "initializer must not be blanked, got: {}",
            result
        );
    }

    #[test]
    fn test_eliminate_unused_exports_keeps_read_refs() {
        // Both exports are read — neither should be removed.
        let code = r#"_m0e.foo = 1;
_m0e.bar = 2;
var _m1_x = _m0e.foo + _m0e.bar;"#;

        let result = eliminate_unused_exports(code);

        assert!(
            result.contains("_m0e.foo = 1"),
            "foo export should survive (has read ref), got: {}",
            result
        );
        assert!(
            result.contains("_m0e.bar = 2"),
            "bar export should survive (has read ref), got: {}",
            result
        );
    }

    #[test]
    fn test_eliminate_unused_exports_keeps_bracket_read_refs() {
        // MUI CJS consumers read named exports through bracket property access
        // after require lowering: `require(672)["getContrastRatio"]`. That is
        // a live export read and must keep the assignment.
        let code = r##"_m0e.getContrastRatio = function getContrastRatio(a, b) { return 7; };
_m0e.darken = function darken(color) { return color; };
var _m1_getContrastRatio = _m0e["getContrastRatio"];
console.log(_m1_getContrastRatio("#000", "#fff"));"##;

        let result = eliminate_unused_exports(code);

        assert!(
            result.contains("_m0e.getContrastRatio"),
            "bracket-read export should survive, got: {}",
            result
        );
        assert!(
            !result.contains("_m0e.darken"),
            "unread sibling export should still be removed, got: {}",
            result
        );
    }

    #[test]
    fn test_eliminate_unused_exports_keeps_direct_require_bracket_refs() {
        let code = r##"_m672e.getContrastRatio = function getContrastRatio(a, b) { return 7; };
_m672e.darken = function darken(color) { return color; };
var _m0_getContrastRatio = _r(672)["getContrastRatio"];
console.log(_m0_getContrastRatio("#000", "#fff"));"##;

        let result = eliminate_unused_exports(code);

        assert!(
            result.contains("_m672e.getContrastRatio"),
            "direct _r(id)[name] read should keep export, got: {}",
            result
        );
        assert!(
            !result.contains("_m672e.darken"),
            "unread sibling export should still be removed, got: {}",
            result
        );
    }

    #[test]
    fn test_eliminate_unused_exports_keeps_require_alias_refs() {
        let code = r##"_m1e.used = function used() { return 1; };
_m1e.alsoUsed = function alsoUsed() { return 2; };
_m1e.unused = function unused() { return 3; };
var _m0_lib = _r(1);
console.log(_m0_lib.used(), _m0_lib["alsoUsed"]());"##;

        let result = eliminate_unused_exports(code);

        assert!(
            result.contains("_m1e.used"),
            "alias dot read should keep export, got: {}",
            result
        );
        assert!(
            result.contains("_m1e.alsoUsed"),
            "alias bracket read should keep export, got: {}",
            result
        );
        assert!(
            !result.contains("_m1e.unused"),
            "unread sibling export should still be removed, got: {}",
            result
        );
    }

    #[test]
    fn test_eliminate_unused_prefixed_vars() {
        // A prefixed var `_m0_helper` with no remaining references after DCE
        // should be removed.
        let code = r#"var _m0_used = 42;
var _m0_helper = function() { return 99; };
console.log(_m0_used);"#;

        let result = eliminate_unused_exports(code);

        // _m0_used has a reference → survive
        assert!(
            result.contains("_m0_used"),
            "used var should survive, got: {}",
            result
        );
        // _m0_helper has only the declaration → removed
        assert!(
            !result.contains("_m0_helper"),
            "unused prefixed var should be removed, got: {}",
            result
        );
    }

    #[test]
    fn test_eliminate_unused_prefixed_vars_counts_template_literal_refs() {
        // MUI generateUtilityClass reads ClassNameGenerator inside a template
        // literal expression. That is a live JS reference, not inert string
        // content, so DCE must keep the local require binding.
        let code = r#"var _m736_ClassNameGenerator = _r(737)["default"] || _r(737);
var _m736_globalStateClasses = { active: "active" };
_m736e.default = function(componentName, slot) {
  return _m736_globalStateClasses[slot]
    ? `Mui-${slot}`
    : `${_m736_ClassNameGenerator.generate(componentName)}-${slot}`;
};
var _m1_read = _m736e.default;"#;

        let result = eliminate_unused_exports(code);

        assert!(
            result.contains("_m736_ClassNameGenerator"),
            "template literal ref should keep ClassNameGenerator binding, got: {}",
            result
        );
    }

    #[test]
    fn test_eliminate_unused_prefixed_vars_keeps_tagged_template_bindings() {
        let code = r##"var _m0_createGlobalStyle = _r(8)["createGlobalStyle"];
var _m0_styled = _r(8)["default"] || _r(8);
var _m0_css = _r(8)["css"];
const _m0_GlobalStyle = _m0_createGlobalStyle`
  body { margin: 0; }
`;
const _m0_Matrix = _m0_styled.main`
  min-height: 100vh;
`;
const _m0_Button = _m0_styled.button`
  ${(props) => _m0_css`
    background: ${props.$accent || "#2563eb"};
  `}
`;
console.log(_m0_GlobalStyle, _m0_Matrix, _m0_Button);"##;

        let result = eliminate_unused_exports(code);

        assert!(
            result.contains("var _m0_createGlobalStyle"),
            "tagged template function binding must survive, got: {}",
            result
        );
        assert!(
            result.contains("var _m0_styled"),
            "tagged template member base binding must survive, got: {}",
            result
        );
        assert!(
            result.contains("var _m0_css"),
            "nested tagged template binding must survive, got: {}",
            result
        );
    }

    #[test]
    fn test_eliminate_unused_prefixed_vars_keeps_require_import_bindings() {
        let code = r##"var _m0_jsx = _r(1)["jsx"];
var _m0_createGlobalStyle = _r(8)["createGlobalStyle"];
var _m0_styled = _r(8)["default"] || _r(8);
const _m0_GlobalStyle = _m0_createGlobalStyle`
  body { margin: 0; }
`;
const _m0_Button = _m0_styled.button`
  color: red;
`;
function _m0_App() {
  return _m0_jsx(_m0_Button, { children: "ok" });
}"##;

        let result = eliminate_unused_exports(code);

        assert!(
            result.contains("var _m0_jsx"),
            "require import binding used after templates must survive, got: {}",
            result
        );
        assert!(
            result.contains("var _m0_createGlobalStyle"),
            "tagged require import binding must survive, got: {}",
            result
        );
        assert!(
            result.contains("var _m0_styled"),
            "member tagged require import binding must survive, got: {}",
            result
        );
    }

    #[test]
    fn test_eliminate_unused_prefixed_vars_ignores_template_raw_quotes() {
        // Theme/CSS template raw text can contain quotes. Those quotes must
        // not corrupt the scanner and hide later live references.
        let code = r#"const _m1_css = `modeStorageKey: 'mui-mode';
color: "${not_a_reference}";
`;
var _m87_experimental_extendTheme = _r(90)["default"] || _r(90);
const _m87_defaultTheme = _m87_experimental_extendTheme();
console.log(_m87_defaultTheme);"#;

        let result = eliminate_unused_exports(code);

        assert!(
            result.contains("var _m87_experimental_extendTheme"),
            "template raw quotes must not hide live binding refs, got: {}",
            result
        );
    }

    #[test]
    fn test_eliminate_unused_prefixed_vars_with_refs() {
        // A prefixed var that IS referenced should NOT be removed.
        let code = r#"var _m0_count = 0;
_m0_count++;
console.log(_m0_count);"#;

        let result = eliminate_unused_exports(code);

        assert!(
            result.contains("_m0_count"),
            "referenced prefixed var should survive, got: {}",
            result
        );
    }

    #[test]
    fn test_eliminate_unused_prefixed_vars_keeps_mui_default_import_bindings() {
        // MUI CssVarsProvider lowers ESM imports into prefixed require
        // bindings. They are not exports, but they are live module-local reads.
        let code = r#"var _m87_experimental_extendTheme = _r(90)["default"] || _r(90);
var _m87_createCssVarsProvider = _r(321)["unstable_createCssVarsProvider"];
var _m87_defaultConfig = _r(88)["defaultConfig"];
const _m87_defaultTheme = _m87_experimental_extendTheme();
const { CssVarsProvider } = _m87_createCssVarsProvider({
  theme: _m87_defaultTheme,
  attribute: _m87_defaultConfig.attribute
});
_m87e.Experimental_CssVarsProvider = CssVarsProvider;"#;

        let result = eliminate_unused_exports(code);

        assert!(
            result.contains("var _m87_experimental_extendTheme"),
            "live default import binding must survive, got: {}",
            result
        );
        assert!(
            result.contains("var _m87_createCssVarsProvider"),
            "live named import binding must survive, got: {}",
            result
        );
        assert!(
            result.contains("var _m87_defaultConfig"),
            "live config import binding must survive property reads, got: {}",
            result
        );
    }

    #[test]
    fn test_eliminate_unused_prefixed_vars_keeps_mui_after_r4_module_shape() {
        let code = r#"'use client';

// do not remove the following import (https://github.com/microsoft/TypeScript/issues/29808#issuecomment-1320713018)
/* eslint-disable @typescript-eslint/no-unused-vars */
// @ts-ignore
var _m87__extends = _r(699)["default"] || _r(699);
var _m87_createCssVarsProvider = _r(321)["unstable_createCssVarsProvider"];
var _m87_styleFunctionSx = _r(643)["default"] || _r(643);
var _m87_experimental_extendTheme = _r(90)["default"] || _r(90);
var _m87_createTypography = _r(686)["default"] || _r(686);
var _m87_excludeVariablesFromRoot = _r(92)["default"] || _r(92);
var _m87_THEME_ID = _r(694)["default"] || _r(694);
var _m87_defaultConfig = _r(88)["defaultConfig"];
const _m87_defaultTheme = _m87_experimental_extendTheme();
const {
  CssVarsProvider,
  useColorScheme,
  getInitColorSchemeScript: getInitColorSchemeScriptSystem
} = _m87_createCssVarsProvider({
  themeId: _m87_THEME_ID,
  theme: _m87_defaultTheme,
  attribute: _m87_defaultConfig.attribute,
  colorSchemeStorageKey: _m87_defaultConfig.colorSchemeStorageKey,
  modeStorageKey: _m87_defaultConfig.modeStorageKey,
  defaultColorScheme: {
    light: _m87_defaultConfig.defaultLightColorScheme,
    dark: _m87_defaultConfig.defaultDarkColorScheme
  },
  resolveTheme: theme => {
    const newTheme = _m87__extends({}, theme, {
      typography: _m87_createTypography(theme.palette, theme.typography)
    });
    newTheme.unstable_sx = function sx(props) {
      return _m87_styleFunctionSx({
        sx: props,
        theme: this
      });
    };
    return newTheme;
  },
  _m87_excludeVariablesFromRoot
});

/**
 * @deprecated Use `InitColorSchemeScript` instead
 * ```diff
 * - import { getInitColorSchemeScript } from '@mui/material/styles';
 * + import InitColorSchemeScript from '@mui/material/InitColorSchemeScript';
 *
 * - getInitColorSchemeScript();
 * + <InitColorSchemeScript />;
 * ```
 */
const _m87_getInitColorSchemeScript = getInitColorSchemeScriptSystem;; _m87.exports["getInitColorSchemeScript"] = _m87_getInitColorSchemeScript;
_m87.exports["useColorScheme"] = useColorScheme; _m87.exports["Experimental_CssVarsProvider"] = CssVarsProvider;"#;

        let result = eliminate_unused_exports(code);

        for binding in [
            "_m87__extends",
            "_m87_createCssVarsProvider",
            "_m87_styleFunctionSx",
            "_m87_experimental_extendTheme",
            "_m87_createTypography",
            "_m87_excludeVariablesFromRoot",
            "_m87_THEME_ID",
            "_m87_defaultConfig",
        ] {
            assert!(
                result.contains(&format!("var {binding}")),
                "live MUI binding {binding} must survive R5, got: {}",
                result
            );
        }
    }

    #[test]
    fn test_eliminate_unused_exports_comparison_not_counted_as_assignment() {
        // `_m0e.foo === "bar"` is a read (comparison), not an assignment.
        // The export should survive if it has comparisons as reads.
        let code = r#"_m0e.foo = "bar";
if (_m0e.foo === "bar") { doSomething(); }"#;

        let result = eliminate_unused_exports(code);

        assert!(
            result.contains("_m0e.foo"),
            "export with comparison reads should survive, got: {}",
            result
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // R6: is_side_effect_free
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_side_effect_free_pure_module() {
        // A module with only declarations (var/function/class) is
        // considered side-effect-free by the heuristic code analysis.
        // Note: `exports.X = ...` lines are conservatively treated as
        // side-effectful by `has_side_effects` since they don't start
        // with a recognized declaration keyword.
        let module = make_module(
            "/project/src/utils.js",
            "function add(a, b) { return a + b; }\nvar PI = 3.14;",
        );
        assert!(
            is_side_effect_free(&module),
            "pure declaration-only module should be side-effect-free"
        );
    }

    #[test]
    fn test_side_effect_cjs_exports_considered_effectful() {
        // CJS `exports.xxx = ...` is treated as a side effect by the
        // heuristic since it doesn't start with a declaration keyword.
        // This is conservative but correct for non-node_modules code.
        let module = make_module(
            "/project/src/lib.js",
            "exports.add = function(a, b) { return a + b; };",
        );
        assert!(
            !is_side_effect_free(&module),
            "CJS exports assignment should be conservatively treated as side-effectful"
        );
    }

    #[test]
    fn test_side_effect_module_not_flattened() {
        // A module with top-level side effects (DOM manipulation, global writes)
        // should NOT be considered side-effect-free.
        let module = make_module(
            "/project/src/init.js",
            "document.title = 'Hello';\nexports.ready = true;",
        );
        assert!(
            !is_side_effect_free(&module),
            "module with DOM side effects should NOT be side-effect-free"
        );
    }

    #[test]
    fn test_side_effect_module_global_assignment() {
        // Global variable assignment is a side effect.
        let module = make_module(
            "/project/src/polyfill.js",
            "window.Promise = require('./promise');\nexports.done = true;",
        );
        assert!(
            !is_side_effect_free(&module),
            "module with global assignment should NOT be side-effect-free"
        );
    }

    #[test]
    fn test_side_effect_free_const_only() {
        // A module with only var/const declarations is side-effect-free.
        // Note: `exports.MODE = MODE` would be treated as a side effect
        // by the heuristic, so we use pure declarations only.
        let module = make_module(
            "/project/src/constants.js",
            "var MODE = 'production';\nconst VERSION = '1.0';",
        );
        assert!(
            is_side_effect_free(&module),
            "const+var declaration module should be side-effect-free"
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // Helper: find_package_info
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_find_package_info_regular() {
        let path = PathBuf::from("/project/node_modules/react/cjs/react.production.min.js");
        let result = find_package_info(&path);
        assert!(result.is_some());
        let (nm_dir, pkg_name) = result.unwrap();
        assert_eq!(nm_dir, PathBuf::from("/project/node_modules"));
        assert_eq!(pkg_name, "react");
    }

    #[test]
    fn test_find_package_info_scoped() {
        let path = PathBuf::from("/project/node_modules/@babel/core/lib/index.js");
        let result = find_package_info(&path);
        assert!(result.is_some());
        let (nm_dir, pkg_name) = result.unwrap();
        assert_eq!(nm_dir, PathBuf::from("/project/node_modules"));
        assert_eq!(pkg_name, "@babel/core");
    }

    #[test]
    fn test_find_package_info_not_in_node_modules() {
        let path = PathBuf::from("/project/src/utils.js");
        let result = find_package_info(&path);
        assert!(result.is_none());
    }

    // ──────────────────────────────────────────────────────────────────
    // Helper: count_identifier_refs
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_count_identifier_refs_basic() {
        let code = "var _m0_x = 1; console.log(_m0_x); return _m0_x;";
        assert_eq!(count_identifier_refs(code, "_m0_x"), 3);
    }

    #[test]
    fn test_count_identifier_refs_skips_strings() {
        let code = r#"var _m0_x = 1; var s = "_m0_x"; return _m0_x;"#;
        // 2 real refs (declaration + return), 1 inside string (skipped)
        assert_eq!(count_identifier_refs(code, "_m0_x"), 2);
    }

    #[test]
    fn test_count_identifier_refs_skips_property_access() {
        let code = "var _m0_x = 1; obj._m0_x = 2; return _m0_x;";
        // obj._m0_x is preceded by `.` — should be skipped
        // Only declaration + return = 2
        assert_eq!(count_identifier_refs(code, "_m0_x"), 2);
    }

    #[test]
    fn test_count_identifier_refs_skips_comments() {
        let code = "var _m0_x = 1; // _m0_x is defined here\nreturn _m0_x;";
        // Comment reference is skipped
        assert_eq!(count_identifier_refs(code, "_m0_x"), 2);
    }

    #[test]
    fn test_count_identifier_refs_ignores_template_raw_quotes() {
        let code = r#"const css = `modeStorageKey: 'mui-mode';
content: "_m0_x";
`;
var _m0_x = 1;
return _m0_x;"#;
        assert_eq!(count_identifier_refs(code, "_m0_x"), 2);
    }

    #[test]
    fn test_count_identifier_refs_counts_template_expression_refs() {
        let code = r#"var _m0_x = 1;
const label = `${_m0_x}`;
const raw = `_m0_x`;"#;
        assert_eq!(count_identifier_refs(code, "_m0_x"), 2);
    }
}
// CODEGEN-END
