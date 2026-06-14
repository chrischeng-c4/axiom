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
use std::collections::{HashMap, HashSet};

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

    // Phase 1: collect candidate constant bindings. Eligibility used to
    // rescan the whole bundle three times per candidate
    // (count_identifier_refs + has_mutating_identifier_ref +
    // template_literal_contains_identifier) — O(candidates x bundle size),
    // ~0.9s on the antd corpus bundle. One lexical sweep now gathers
    // counts, mutation flags, and template appearances for every
    // _m-prefixed identifier at once.
    let stats = collect_prefixed_ident_stats(code);
    let mut decl_spans: HashMap<&str, Vec<(usize, usize)>> = HashMap::new();
    let mut literals: HashMap<&str, &str> = HashMap::new();
    for cap in re.captures_iter(code) {
        let (Some(name), Some(lit), Some(whole)) = (cap.get(1), cap.get(2), cap.get(0)) else {
            continue;
        };
        decl_spans
            .entry(name.as_str())
            .or_default()
            .push((whole.start(), whole.end()));
        literals.insert(name.as_str(), lit.as_str());
    }

    let mut constants: HashMap<&str, &str> = HashMap::new();
    let mut removals: Vec<(usize, usize)> = Vec::new();
    for (name, spans) in &decl_spans {
        // Re-declared names are pathological; leave them alone.
        if spans.len() != 1 {
            continue;
        }
        let Some(stat) = stats.get(*name) else {
            continue;
        };
        // count >= 2 means: 1 for the declaration + at least 1 read.
        // Mutations and raw-template appearances disqualify inlining.
        if stat.count >= 2 && stat.mutations <= stat.decl_assignments && !stat.in_template {
            constants.insert(name, literals[name]);
            removals.push(spans[0]);
        }
    }

    if constants.is_empty() {
        return code.to_string();
    }
    removals.sort_unstable();

    // Phase 2: one forward rebuild — drop the constant declarations and
    // substitute every standalone reference with its literal.
    let b = code.as_bytes();
    let len = b.len();
    let mut out = Vec::with_capacity(len);
    let mut i = 0usize;
    let mut next_removal = 0usize;

    while i < len {
        if next_removal < removals.len() && removals[next_removal].0 == i {
            i = removals[next_removal].1;
            next_removal += 1;
            continue;
        }
        while next_removal < removals.len() && removals[next_removal].0 < i {
            next_removal += 1;
        }
        // Strings and templates are copied verbatim (template-involved
        // candidates were already disqualified above, matching the old
        // replace_identifier behavior of skipping backtick spans).
        if matches!(b[i], b'"' | b'\'') {
            let end = skip_quoted_literal(b, i).min(len);
            out.extend_from_slice(&b[i..end]);
            i = end;
            continue;
        }
        if b[i] == b'`' {
            let (next, _) = scan_template_literal_expr_ranges(b, i, |_, _| 0);
            let end = next.min(len);
            out.extend_from_slice(&b[i..end]);
            i = end;
            continue;
        }
        if b[i] == b'/' && i + 1 < len && (b[i + 1] == b'/' || b[i + 1] == b'*') {
            let start = i;
            if b[i + 1] == b'/' {
                while i < len && b[i] != b'\n' {
                    i += 1;
                }
            } else {
                i += 2;
                while i + 1 < len && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(len);
            }
            out.extend_from_slice(&b[start..i]);
            continue;
        }
        if is_id_cont_byte(b[i]) && !b[i].is_ascii_digit() {
            let ident_start = i;
            while i < len && is_id_cont_byte(b[i]) {
                i += 1;
            }
            let ident = &code[ident_start..i];
            let preceded_by_dot = {
                let mut p = ident_start;
                while p > 0 && matches!(b[p - 1], b' ' | b'\t') {
                    p -= 1;
                }
                p > 0 && b[p - 1] == b'.'
            };
            if !preceded_by_dot {
                if let Some(literal) = constants.get(ident) {
                    out.extend_from_slice(literal.as_bytes());
                    continue;
                }
            }
            out.extend_from_slice(ident.as_bytes());
            continue;
        }
        out.push(b[i]);
        i += 1;
    }

    String::from_utf8(out).unwrap_or_else(|_| code.to_string())
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
    let mut result = inline_direct_literal_export_reads(code);

    // Phase 1: Remove unused _m{i}e.NAME export assignments
    // Match pattern: _m{i}e.NAME = <expr>;
    let export_assign_re = Regex::new(r"(_m\d+e)\.([a-zA-Z_$][a-zA-Z0-9_$]*)\s*=[^=]").unwrap();

    let mut export_candidates: HashSet<(String, String)> = HashSet::new();

    for cap in export_assign_re.captures_iter(&result) {
        let export_obj = cap[1].to_string();
        let export_name = cap[2].to_string();
        export_candidates.insert((export_obj, export_name));
    }
    let direct_export_assignments = collect_direct_export_assignments(&result);
    for (id, export_name) in direct_export_assignments.keys() {
        export_candidates.insert((format!("_m{id}e"), export_name.clone()));
    }

    // Count all read references in one lexical sweep. The previous path
    // scanned the whole MUI bundle once per export candidate; even capped at
    // the first read, unused exports still paid O(exports x bundle) work.
    let used_exports = collect_used_export_refs(&result, &export_candidates);
    let bare_required_modules = collect_bare_require_module_ids(&result);
    let exports_to_remove: Vec<(String, String)> = export_candidates
        .into_iter()
        .filter(|candidate| !used_exports.contains(candidate))
        .collect();

    let direct_exports_to_remove: Vec<(usize, usize, String)> = direct_export_assignments
        .iter()
        .filter_map(|((id, export_name), assignment)| {
            let canonical = (format!("_m{id}e"), export_name.clone());
            (!used_exports.contains(&canonical) && !bare_required_modules.contains(id))
                .then(|| (assignment.span.0, assignment.span.1, String::new()))
        })
        .collect();

    if !direct_exports_to_remove.is_empty() {
        result = apply_static_replacements(&result, direct_exports_to_remove);
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
        Regex::new(r"\bvar\s+(_m\d+_[a-zA-Z_$][a-zA-Z0-9_$]*)\s*=\s*(?:_r|require)\s*\(").unwrap();
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

    // Phase 3: orphan-collect function declarations. Phase 1 removed the
    // `_mNe.name = _mN_name;` export assignments of unused exports, but the
    // backing `function _mN_name(...){...}` bodies stayed in the flattened
    // module block — colorManipulator ships every color helper when only
    // `alpha` is used. Dead helpers reference each other (emphasize ->
    // darken -> decomposeColor), so iterate to a fixpoint.
    for _ in 0..8 {
        let removed = remove_orphan_prefixed_functions(&result);
        match removed {
            Some(next) => result = next,
            None => break,
        }
    }

    result
}

#[derive(Debug, Clone)]
struct DirectExportAssignment {
    span: (usize, usize),
    expr: String,
}

fn inline_direct_literal_export_reads(code: &str) -> String {
    let assignments = collect_direct_export_assignments(code);
    if assignments.is_empty() {
        return code.to_string();
    }

    let literal_assignments: HashMap<(usize, String), String> = assignments
        .iter()
        .filter_map(|(key, assignment)| {
            let expr = assignment.expr.trim();
            is_inlineable_literal_export_expr(expr).then(|| (key.clone(), expr.to_string()))
        })
        .collect();
    if literal_assignments.is_empty() {
        return code.to_string();
    }

    let b = code.as_bytes();
    let len = b.len();
    let mut i = 0usize;
    let mut prev = b'(';
    let mut bare_requires: HashSet<usize> = HashSet::new();
    let mut reads: HashMap<(usize, String), Vec<(usize, usize)>> = HashMap::new();

    while i < len {
        if matches!(b[i], b'"' | b'\'') {
            i = skip_quoted_literal(b, i).min(len);
            prev = b'"';
            continue;
        }
        if b[i] == b'`' {
            let (next, _) = scan_template_literal_expr_ranges(b, i, |_, _| 0);
            i = next.min(len);
            prev = b'`';
            continue;
        }
        if b[i] == b'/'
            && i + 1 < len
            && !matches!(b[i + 1], b'/' | b'*')
            && regex_context_byte(prev)
        {
            i = skip_regex_literal(b, i).min(len);
            prev = b'/';
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
                i = (i + 2).min(len);
                continue;
            }
        }

        if is_require_call_ident_at(b, i) {
            if let Some((module_id, after_require)) = match_require_call_any(b, i) {
                let id = module_id.parse::<usize>().unwrap_or(usize::MAX);
                if let Some((prop, end_ref, is_assignment)) =
                    match_any_property_access_after_base(b, after_require, false)
                {
                    if !is_assignment && is_literal_inline_boundary(b, end_ref) {
                        let key = (id, prop);
                        if literal_assignments.contains_key(&key) {
                            reads.entry(key).or_default().push((i, end_ref));
                        }
                    }
                    i = end_ref;
                    prev = b'a';
                    continue;
                }
                bare_requires.insert(id);
                i = after_require;
                prev = b')';
                continue;
            }
        }

        if !matches!(b[i], b' ' | b'\t' | b'\r' | b'\n') {
            prev = b[i];
        }
        i += 1;
    }

    let mut replacements: Vec<(usize, usize, String)> = Vec::new();
    for (key, spans) in reads {
        if bare_requires.contains(&key.0) {
            continue;
        }
        let Some(expr) = literal_assignments.get(&key) else {
            continue;
        };
        for (start, end) in spans {
            replacements.push((start, end, expr.clone()));
        }
    }

    if replacements.is_empty() {
        return code.to_string();
    }
    apply_static_replacements(code, replacements)
}

fn collect_bare_require_module_ids(code: &str) -> HashSet<usize> {
    let object_keys_reexports = collect_object_keys_reexport_mappings(code);
    let wrapper_ids: HashSet<usize> = object_keys_reexports
        .iter()
        .map(|mapping| mapping.wrapper_id)
        .collect();
    let escaped_wrapper_ids = collect_module_namespace_escape_ids(code, &wrapper_ids);
    let mut reexport_counts: HashMap<usize, usize> = HashMap::new();
    for mapping in object_keys_reexports {
        if escaped_wrapper_ids.contains(&mapping.wrapper_id) {
            continue;
        }
        *reexport_counts.entry(mapping.source_id).or_default() += 1;
    }
    let b = code.as_bytes();
    let len = b.len();
    let mut i = 0usize;
    let mut prev = b'(';
    let mut bare_counts: HashMap<usize, usize> = HashMap::new();

    while i < len {
        if matches!(b[i], b'"' | b'\'') {
            i = skip_quoted_literal(b, i).min(len);
            prev = b'"';
            continue;
        }
        if b[i] == b'`' {
            let (next, _) = scan_template_literal_expr_ranges(b, i, |_, _| 0);
            i = next.min(len);
            prev = b'`';
            continue;
        }
        if b[i] == b'/'
            && i + 1 < len
            && !matches!(b[i + 1], b'/' | b'*')
            && regex_context_byte(prev)
        {
            i = skip_regex_literal(b, i).min(len);
            prev = b'/';
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
                i = (i + 2).min(len);
                continue;
            }
        }

        if is_require_call_ident_at(b, i) {
            if let Some((module_id, after_require)) = match_require_call_any(b, i) {
                if let Some((_, end_ref, _)) =
                    match_any_property_access_after_base(b, after_require, false)
                {
                    i = end_ref;
                    prev = b'a';
                    continue;
                }
                if let Ok(id) = module_id.parse::<usize>() {
                    *bare_counts.entry(id).or_default() += 1;
                }
                i = after_require;
                prev = b')';
                continue;
            }
        }

        if !matches!(b[i], b' ' | b'\t' | b'\r' | b'\n') {
            prev = b[i];
        }
        i += 1;
    }

    bare_counts
        .into_iter()
        .filter_map(|(id, count)| {
            (count > reexport_counts.get(&id).copied().unwrap_or(0)).then_some(id)
        })
        .collect()
}

#[derive(Debug, Clone)]
struct NamespaceAlias {
    module_id: usize,
    decl_range: (usize, usize),
    alias_range: (usize, usize),
}

fn collect_module_namespace_escape_ids(code: &str, module_ids: &HashSet<usize>) -> HashSet<usize> {
    if module_ids.is_empty() {
        return HashSet::new();
    }

    let aliases = namespace_aliases_for_modules(code, module_ids);
    let mut alias_by_name: HashMap<String, NamespaceAlias> = HashMap::new();
    let mut duplicate_aliases: HashSet<String> = HashSet::new();
    for alias in aliases {
        let name = alias_name_at(code, alias.alias_range).to_string();
        if alias_by_name.insert(name.clone(), alias).is_some() {
            duplicate_aliases.insert(name);
        }
    }

    let b = code.as_bytes();
    let len = b.len();
    let mut i = 0usize;
    let mut prev = b'(';
    let mut escaped = HashSet::new();

    while i < len {
        if matches!(b[i], b'"' | b'\'') {
            i = skip_quoted_literal(b, i).min(len);
            prev = b'"';
            continue;
        }
        if b[i] == b'`' {
            let (next, _) = scan_template_literal_expr_ranges(b, i, |_, _| 0);
            i = next.min(len);
            prev = b'`';
            continue;
        }
        if b[i] == b'/'
            && i + 1 < len
            && !matches!(b[i + 1], b'/' | b'*')
            && regex_context_byte(prev)
        {
            i = skip_regex_literal(b, i).min(len);
            prev = b'/';
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
                i = (i + 2).min(len);
                continue;
            }
        }

        if is_id_cont_byte(b[i]) && !b[i].is_ascii_digit() {
            let ident_start = i;
            while i < len && is_id_cont_byte(b[i]) {
                i += 1;
            }
            let ident = match std::str::from_utf8(&b[ident_start..i]) {
                Ok(ident) => ident,
                Err(_) => {
                    prev = b'a';
                    continue;
                }
            };

            if ident == "_r" || ident == "require" {
                if let Some((module_id, after_require)) = match_require_call_any(b, ident_start) {
                    if let Ok(id) = module_id.parse::<usize>() {
                        if module_ids.contains(&id) {
                            if is_namespace_alias_require_call(&alias_by_name, id, ident_start) {
                                i = after_require;
                                prev = b')';
                                continue;
                            }
                            if let Some((_, end_ref, _)) =
                                match_any_property_access_after_base(b, after_require, true)
                            {
                                i = end_ref;
                                prev = b'a';
                                continue;
                            }
                            escaped.insert(id);
                        }
                    }
                    i = after_require;
                    prev = b')';
                    continue;
                }
            } else if let Some(alias) = alias_by_name.get(ident) {
                if ident_start >= alias.alias_range.0 && ident_start < alias.alias_range.1 {
                    prev = b'a';
                    continue;
                }
                if duplicate_aliases.contains(ident) {
                    escaped.insert(alias.module_id);
                    prev = b'a';
                    continue;
                }
                if let Some((_, end_ref, _)) = match_any_property_access_after_base(b, i, true) {
                    i = end_ref;
                    prev = b'a';
                    continue;
                }
                if is_namespace_reflection_arg(b, ident_start, i) {
                    escaped.insert(alias.module_id);
                }
            }

            prev = b'a';
            continue;
        }

        if !matches!(b[i], b' ' | b'\t' | b'\r' | b'\n') {
            prev = b[i];
        }
        i += 1;
    }

    escaped
}

fn namespace_aliases_for_modules(code: &str, module_ids: &HashSet<usize>) -> Vec<NamespaceAlias> {
    let re = Regex::new(
        r"(?:^|[;{}\n])\s*(?:var|let|const)\s+([a-zA-Z_$][a-zA-Z0-9_$]*)\s*=\s*(?:_r|require)\s*\(\s*(\d+)\s*\)\s*;",
    )
    .unwrap();
    re.captures_iter(code)
        .filter_map(|cap| {
            let alias = cap.get(1)?;
            let module_id = cap.get(2)?.as_str().parse::<usize>().ok()?;
            module_ids.contains(&module_id).then(|| NamespaceAlias {
                module_id,
                decl_range: (cap.get(0).unwrap().start(), cap.get(0).unwrap().end()),
                alias_range: (alias.start(), alias.end()),
            })
        })
        .collect()
}

fn alias_name_at(code: &str, range: (usize, usize)) -> &str {
    code.get(range.0..range.1).unwrap_or("")
}

fn is_namespace_alias_require_call(
    aliases: &HashMap<String, NamespaceAlias>,
    module_id: usize,
    require_start: usize,
) -> bool {
    aliases.values().any(|alias| {
        alias.module_id == module_id
            && require_start >= alias.decl_range.0
            && require_start < alias.decl_range.1
    })
}

fn is_namespace_reflection_arg(b: &[u8], start: usize, end: usize) -> bool {
    let before = skip_ascii_ws_back(b, start);
    if before == 0 {
        return false;
    }
    if b[before - 1] == b'.' && before >= 3 && &b[before - 3..before] == b"..." {
        return true;
    }
    if b[before - 1] != b'(' {
        return false;
    }

    let Some((method, method_start)) = previous_identifier(b, before - 1) else {
        return false;
    };
    let Some((object, _)) = previous_identifier_before_dot(b, method_start) else {
        return false;
    };
    let reflected = matches!(
        (object, method),
        ("Object", "keys" | "values" | "entries") | ("Reflect", "ownKeys")
    );
    if !reflected {
        return false;
    }

    let after = skip_ascii_ws(b, end);
    after >= b.len() || matches!(b[after], b')' | b',')
}

fn skip_ascii_ws_back(b: &[u8], mut i: usize) -> usize {
    while i > 0 && b[i - 1].is_ascii_whitespace() {
        i -= 1;
    }
    i
}

fn previous_identifier<'a>(b: &'a [u8], before: usize) -> Option<(&'a str, usize)> {
    let end = skip_ascii_ws_back(b, before);
    let mut start = end;
    while start > 0 && is_id_cont_byte(b[start - 1]) {
        start -= 1;
    }
    if start == end || b[start].is_ascii_digit() {
        return None;
    }
    Some((std::str::from_utf8(&b[start..end]).ok()?, start))
}

fn previous_identifier_before_dot<'a>(b: &'a [u8], before: usize) -> Option<(&'a str, usize)> {
    let dot = skip_ascii_ws_back(b, before);
    if dot == 0 || b[dot - 1] != b'.' {
        return None;
    }
    previous_identifier(b, dot - 1)
}

fn is_inlineable_literal_export_expr(expr: &str) -> bool {
    let expr = expr.trim();
    if expr.is_empty() || expr.starts_with('-') {
        return false;
    }
    if matches!(expr, "true" | "false" | "null" | "undefined" | "void 0") {
        return true;
    }
    let b = expr.as_bytes();
    if matches!(b.first(), Some(b'"' | b'\'')) {
        return skip_quoted_literal(b, 0) == b.len();
    }
    b[0].is_ascii_digit()
        && b.iter()
            .all(|c| c.is_ascii_digit() || matches!(c, b'.' | b'e' | b'E' | b'+' | b'-'))
}

fn is_literal_inline_boundary(b: &[u8], mut i: usize) -> bool {
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    !matches!(b.get(i), Some(b'.' | b'[' | b'?' | b'`'))
}

/// Lower direct `require(id).export` reads to local bindings.
///
/// Scope-hoisted production bundles still carry CommonJS-shaped export glue
/// for modules whose exports are only read as direct properties:
///
/// ```js
/// _m1.exports["default"] = makeButton();
/// var Button = _r(1)["default"];
/// ```
///
/// When a module has no bare namespace require (`_r(1)`) and every observed
/// require read is a concrete property, this rewrites the export assignment to
/// a local binding and points reads at that binding:
///
/// ```js
/// var _m1_export_default = makeButton();
/// var Button = _m1_export_default;
/// ```
///
/// The follow-up mangle pass then compresses the generated local names, and
/// orphan slot/alias declarations can be dropped.
pub fn lower_direct_export_reads(code: &str) -> String {
    let mut result = code.to_string();
    for _ in 0..8 {
        let next = lower_direct_export_reads_once(&result);
        if next == result {
            break;
        }
        result = next;
    }
    result
}

fn lower_direct_export_reads_once(code: &str) -> String {
    let assignments = collect_direct_export_assignments(code);
    if assignments.is_empty() {
        return code.to_string();
    }

    let require_re = Regex::new(
        r#"_r\(\s*(\d+)\s*\)(?:\[\s*"([A-Za-z_$][A-Za-z0-9_$]*)"\s*\]|\.([A-Za-z_$][A-Za-z0-9_$]*))?"#,
    )
    .unwrap();
    let mut bare_requires: HashSet<usize> = HashSet::new();
    let mut reads_by_module: HashMap<usize, HashMap<String, Vec<(usize, usize)>>> = HashMap::new();

    for cap in require_re.captures_iter(code) {
        let Some(id) = cap.get(1).and_then(|m| m.as_str().parse::<usize>().ok()) else {
            continue;
        };
        let prop = cap
            .get(2)
            .or_else(|| cap.get(3))
            .map(|m| m.as_str().to_string());
        let Some(prop) = prop else {
            bare_requires.insert(id);
            continue;
        };
        let Some(whole) = cap.get(0) else {
            continue;
        };
        reads_by_module
            .entry(id)
            .or_default()
            .entry(prop)
            .or_default()
            .push((whole.start(), whole.end()));
    }

    if reads_by_module.is_empty() {
        return code.to_string();
    }

    let mut bare_replacements: Vec<(usize, usize, String)> = Vec::new();
    let mut candidate_modules: HashMap<usize, HashMap<String, Vec<(usize, usize)>>> =
        HashMap::new();
    let mut planned_read_spans: Vec<(usize, usize)> = Vec::new();

    for (id, prop_reads) in reads_by_module {
        if bare_requires.contains(&id) {
            for (prop, read_spans) in prop_reads {
                let Some(assign) = assignments.get(&(id, prop.clone())) else {
                    continue;
                };
                if !is_direct_export_alias_expr(&assign.expr)
                    || is_module_local_export_alias(&assign.expr, id)
                {
                    continue;
                }
                for (start, end) in read_spans {
                    bare_replacements.push((start, end, assign.expr.clone()));
                    planned_read_spans.push((start, end));
                }
            }
            continue;
        }
        if prop_reads
            .keys()
            .any(|prop| !assignments.contains_key(&(id, prop.clone())))
        {
            continue;
        }
        for read_spans in prop_reads.values() {
            planned_read_spans.extend(read_spans.iter().copied());
        }
        candidate_modules.insert(id, prop_reads);
    }

    let mut replacements = bare_replacements;
    let mut lowered_modules: HashSet<usize> = HashSet::new();

    for (id, prop_reads) in candidate_modules {
        let assignment_spans: Vec<(usize, usize)> = prop_reads
            .keys()
            .filter_map(|prop| {
                assignments
                    .get(&(id, prop.clone()))
                    .map(|assignment| assignment.span)
            })
            .collect();
        if planned_read_spans.iter().any(|span| {
            assignment_spans
                .iter()
                .any(|assignment_span| span_within(*span, *assignment_span))
        }) {
            continue;
        }

        lowered_modules.insert(id);
        for (prop, read_spans) in prop_reads {
            let Some(assign) = assignments.get(&(id, prop.clone())) else {
                continue;
            };
            let local = format!("_m{id}_export_{}", sanitize_export_local_suffix(&prop));
            for (start, end) in read_spans {
                replacements.push((start, end, local.clone()));
            }
            replacements.push((
                assign.span.0,
                assign.span.1,
                format!("var {local}={};", assign.expr),
            ));
        }
    }

    if replacements.is_empty() {
        return code.to_string();
    }

    let mut out = apply_static_replacements(code, replacements);
    for id in lowered_modules {
        out = remove_orphan_module_alias_and_slot(&out, id);
    }
    out
}

fn span_within(inner: (usize, usize), outer: (usize, usize)) -> bool {
    inner.0 >= outer.0 && inner.1 <= outer.1
}

/// Shrink generated CommonJS glue before final AST minification.
///
/// Scope-hoisted output starts every module slot as `{exports:{}}`, even when
/// the slot is only ever used as an export-object container:
///
/// ```js
/// var _m1={exports:{}};
/// _m1.exports.default=value;
/// var x=_r(1).default||_r(1);
/// ```
///
/// For those simple slots, the slot itself can be the export object:
///
/// ```js
/// var _m1={};
/// _m1.default=value;
/// var x=_r(1,1);
/// ```
///
/// Modules that are passed as a retained CommonJS `module` object, reassign
/// `module.exports`, or export a property literally named `exports` are left
/// untouched. The caller keeps the existing parse guard around this pass.
pub fn optimize_generated_module_glue(code: &str) -> String {
    if !code.contains("var _mods=[") || !code.contains("function _r") {
        return code.to_string();
    }

    let slots = collect_generated_module_slots(code);
    if slots.is_empty() {
        return code.to_string();
    }
    let Some(mods_array_range) = generated_mods_array_range(code) else {
        return code.to_string();
    };
    let simple_slots = collect_simple_export_container_slots(code, &slots, mods_array_range);

    let optimized = rewrite_generated_module_glue(code, &simple_slots);
    let optimized = prune_unrequired_generated_mods_slots(&optimized);
    if optimized == code {
        return code.to_string();
    }

    let helper_from = "function _r(id){var m=_mods[id];return m?m.exports:{}}";
    let helper_to = r#"function _r(id,d){var m=_mods[id];if(!m)return{};m="exports"in m?m.exports:m;return d?m.default||m:m}"#;
    optimized.replacen(helper_from, helper_to, 1)
}

fn collect_generated_module_slots(code: &str) -> HashSet<String> {
    let re = Regex::new(r"\bvar (_m\d+)=\{exports:\{\}\}").unwrap();
    re.captures_iter(code)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

fn generated_mods_array_range(code: &str) -> Option<(usize, usize)> {
    let start = code.find("var _mods=[")?;
    let rest = &code[start..];
    let end_rel = rest.find("];function _r")?;
    Some((start, start + end_rel + 1))
}

fn prune_unrequired_generated_mods_slots(code: &str) -> String {
    let Some(mods_array_range) = generated_mods_array_range(code) else {
        return code.to_string();
    };
    let Some(required_ids) = collect_static_generated_require_ids(code) else {
        return code.to_string();
    };

    let open = code[mods_array_range.0..mods_array_range.1]
        .find('[')
        .map(|offset| mods_array_range.0 + offset);
    let Some(open) = open else {
        return code.to_string();
    };
    let close = mods_array_range.1.saturating_sub(1);
    if open >= close || close > code.len() {
        return code.to_string();
    }

    let entries_src = &code[open + 1..close];
    let entries: Vec<&str> = entries_src.split(',').collect();
    if entries.is_empty() {
        return code.to_string();
    }

    let mut changed = false;
    let mut rewritten_entries = String::with_capacity(entries_src.len());
    for (id, entry) in entries.iter().enumerate() {
        if id > 0 {
            rewritten_entries.push(',');
        }
        let trimmed = entry.trim();
        if required_ids.contains(&id) || !is_generated_module_slot_name(trimmed) {
            rewritten_entries.push_str(entry);
        } else {
            changed = true;
        }
    }

    if !changed {
        return code.to_string();
    }

    let mut out = String::with_capacity(code.len());
    out.push_str(&code[..open + 1]);
    out.push_str(&rewritten_entries);
    out.push_str(&code[close..]);
    out
}

fn is_generated_module_slot_name(value: &str) -> bool {
    let Some(digits) = value.strip_prefix("_m") else {
        return false;
    };
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}

fn collect_static_generated_require_ids(code: &str) -> Option<HashSet<usize>> {
    let mut ids = HashSet::new();
    collect_static_generated_require_ids_in_range(code.as_bytes(), 0, code.len(), &mut ids)
        .then_some(ids)
}

fn collect_static_generated_require_ids_in_range(
    b: &[u8],
    start: usize,
    end: usize,
    ids: &mut HashSet<usize>,
) -> bool {
    let len = end.min(b.len());
    let mut i = start.min(len);
    let mut prev = b'(';

    while i < len {
        if matches!(b[i], b'"' | b'\'') {
            i = skip_quoted_literal(b, i).min(len);
            prev = b'"';
            continue;
        }
        if b[i] == b'`' {
            let mut ok = true;
            let (next, _) = scan_template_literal_expr_ranges(b, i, |expr_start, expr_end| {
                if !collect_static_generated_require_ids_in_range(b, expr_start, expr_end, ids) {
                    ok = false;
                }
                0
            });
            if !ok {
                return false;
            }
            i = next.min(len);
            prev = b'`';
            continue;
        }
        if b[i] == b'/'
            && i + 1 < len
            && !matches!(b[i + 1], b'/' | b'*')
            && regex_context_byte(prev)
        {
            i = skip_regex_literal(b, i).min(len);
            prev = b'/';
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
                i = (i + 2).min(len);
                continue;
            }
        }

        if is_id_start_byte(b[i]) {
            let ident_start = i;
            i += 1;
            while i < len && is_id_cont_byte(b[i]) {
                i += 1;
            }
            let ident = &b[ident_start..i];
            if (ident == b"_r" || ident == b"require")
                && !is_property_access_at(b, ident_start)
                && !is_function_declaration_name_at(b, ident_start)
            {
                let call_start = skip_ascii_ws(b, i);
                if call_start < len && b[call_start] == b'(' {
                    let arg_start = skip_ascii_ws(b, call_start + 1);
                    let mut arg_end = arg_start;
                    while arg_end < len && b[arg_end].is_ascii_digit() {
                        arg_end += 1;
                    }
                    if arg_end == arg_start {
                        return false;
                    }
                    let after_arg = skip_ascii_ws(b, arg_end);
                    if after_arg >= len || !matches!(b[after_arg], b')' | b',') {
                        return false;
                    }
                    let Ok(id_text) = std::str::from_utf8(&b[arg_start..arg_end]) else {
                        return false;
                    };
                    let Ok(id) = id_text.parse::<usize>() else {
                        return false;
                    };
                    ids.insert(id);
                }
            }
            prev = b'a';
            continue;
        }

        if !b[i].is_ascii_whitespace() {
            prev = b[i];
        }
        i += 1;
    }

    true
}

fn is_id_start_byte(byte: u8) -> bool {
    byte == b'_' || byte == b'$' || byte.is_ascii_alphabetic()
}

fn is_property_access_at(b: &[u8], start: usize) -> bool {
    let mut p = start;
    while p > 0 && b[p - 1].is_ascii_whitespace() {
        p -= 1;
    }
    p > 0 && b[p - 1] == b'.'
}

fn is_function_declaration_name_at(b: &[u8], start: usize) -> bool {
    let mut p = start;
    while p > 0 && b[p - 1].is_ascii_whitespace() {
        p -= 1;
    }
    const FUNCTION: &[u8] = b"function";
    if p < FUNCTION.len() || &b[p - FUNCTION.len()..p] != FUNCTION {
        return false;
    }
    let before = p - FUNCTION.len();
    before == 0 || !is_id_cont_byte(b[before - 1])
}

fn collect_simple_export_container_slots(
    code: &str,
    slots: &HashSet<String>,
    mods_array_range: (usize, usize),
) -> HashSet<String> {
    let b = code.as_bytes();
    let mut bad: HashSet<String> = HashSet::new();
    let mut i = 0usize;
    let mut prev = b'(';

    while i < b.len() {
        if matches!(b[i], b'"' | b'\'') {
            i = skip_quoted_literal(b, i);
            prev = b'"';
            continue;
        }
        if b[i] == b'`' {
            let (next, _) = scan_template_literal_expr_ranges(b, i, |_, _| 0);
            i = next;
            prev = b'`';
            continue;
        }
        if b[i] == b'/'
            && i + 1 < b.len()
            && !matches!(b[i + 1], b'/' | b'*')
            && regex_context_byte(prev)
        {
            i = skip_regex_literal(b, i);
            prev = b'/';
            continue;
        }
        if b[i] == b'/' && i + 1 < b.len() {
            if b[i + 1] == b'/' {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if b[i + 1] == b'*' {
                i += 2;
                while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(b.len());
                continue;
            }
        }

        if let Some((slot, end)) = parse_module_slot_ident(b, i) {
            if slots.contains(slot) && !is_generated_slot_allowed_use(b, i, end, mods_array_range) {
                bad.insert(slot.to_string());
            }
            i = end;
            prev = b'a';
            continue;
        }

        if !b[i].is_ascii_whitespace() {
            prev = b[i];
        }
        i += 1;
    }

    slots
        .iter()
        .filter(|slot| !bad.contains(*slot))
        .cloned()
        .collect()
}

fn is_generated_slot_allowed_use(
    b: &[u8],
    start: usize,
    end: usize,
    mods_array_range: (usize, usize),
) -> bool {
    if start >= mods_array_range.0 && end <= mods_array_range.1 {
        return true;
    }
    if is_generated_slot_decl_at(b, start, end) {
        return true;
    }
    if !b[end..].starts_with(b".exports") {
        return false;
    }

    let after_exports = end + ".exports".len();
    let after = skip_ascii_ws(b, after_exports);
    if after < b.len() && b[after] == b'=' && !matches!(b.get(after + 1), Some(b'=')) {
        return false;
    }
    if b[after_exports..].starts_with(b".exports")
        || b[after_exports..].starts_with(br#"["exports"]"#)
        || b[after_exports..].starts_with(br#"['exports']"#)
        || b[after_exports..].starts_with(b"[`exports`]")
    {
        return false;
    }
    true
}

fn is_generated_slot_decl_at(b: &[u8], start: usize, end: usize) -> bool {
    start >= "var ".len()
        && &b[start - "var ".len()..start] == b"var "
        && b[end..].starts_with(b"={exports:{}}")
}

fn rewrite_generated_module_glue(code: &str, simple_slots: &HashSet<String>) -> String {
    let b = code.as_bytes();
    let mut out = Vec::with_capacity(code.len());
    let mut i = 0usize;
    let mut prev = b'(';

    while i < b.len() {
        if matches!(b[i], b'"' | b'\'') {
            let next = skip_quoted_literal(b, i);
            out.extend_from_slice(&b[i..next]);
            i = next;
            prev = b'"';
            continue;
        }
        if b[i] == b'`' {
            let (next, _) = scan_template_literal_expr_ranges(b, i, |_, _| 0);
            out.extend_from_slice(&b[i..next]);
            i = next;
            prev = b'`';
            continue;
        }
        if b[i] == b'/'
            && i + 1 < b.len()
            && !matches!(b[i + 1], b'/' | b'*')
            && regex_context_byte(prev)
        {
            let next = skip_regex_literal(b, i);
            out.extend_from_slice(&b[i..next]);
            i = next;
            prev = b'/';
            continue;
        }
        if b[i] == b'/' && i + 1 < b.len() {
            if b[i + 1] == b'/' {
                let start = i;
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
                out.extend_from_slice(&b[start..i]);
                continue;
            }
            if b[i + 1] == b'*' {
                let start = i;
                i += 2;
                while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(b.len());
                out.extend_from_slice(&b[start..i]);
                continue;
            }
        }

        if let Some((end, id)) = match_default_fallback_require(b, i) {
            out.extend_from_slice(format!("_r({id},1)").as_bytes());
            i = end;
            prev = b')';
            continue;
        }

        if let Some((slot, end)) = parse_module_slot_ident(b, i) {
            if simple_slots.contains(slot) {
                if is_generated_slot_decl_at(b, i, end) {
                    out.extend_from_slice(slot.as_bytes());
                    out.extend_from_slice(b"={}");
                    i = end + "={exports:{}}".len();
                    prev = b'}';
                    continue;
                }
                if b[end..].starts_with(b".exports") {
                    let after_exports = end + ".exports".len();
                    let after = skip_ascii_ws(b, after_exports);
                    let prev_sig = out
                        .iter()
                        .rev()
                        .find(|c| !c.is_ascii_whitespace())
                        .copied()
                        .unwrap_or(b';');
                    if after < b.len() && b[after] == b';' && matches!(prev_sig, b'{' | b';') {
                        i = after + 1;
                        continue;
                    }
                    out.extend_from_slice(slot.as_bytes());
                    i = after_exports;
                    prev = b'a';
                    continue;
                }
            }
        }

        out.push(b[i]);
        if !b[i].is_ascii_whitespace() {
            prev = b[i];
        }
        i += 1;
    }

    String::from_utf8(out).unwrap_or_else(|_| code.to_string())
}

fn parse_module_slot_ident(b: &[u8], i: usize) -> Option<(&str, usize)> {
    if i + 3 > b.len() || !b[i..].starts_with(b"_m") || !b[i + 2].is_ascii_digit() {
        return None;
    }
    if i > 0 && is_id_cont_byte(b[i - 1]) {
        return None;
    }
    let mut end = i + 3;
    while end < b.len() && b[end].is_ascii_digit() {
        end += 1;
    }
    if end < b.len() && is_id_cont_byte(b[end]) {
        return None;
    }
    Some((std::str::from_utf8(&b[i..end]).ok()?, end))
}

fn match_default_fallback_require(b: &[u8], i: usize) -> Option<(usize, String)> {
    let (id, mut j) = match_require_id_at(b, i)?;
    if b[j..].starts_with(b".default") {
        j += ".default".len();
    } else if b[j..].starts_with(br#"["default"]"#)
        || b[j..].starts_with(br#"['default']"#)
        || b[j..].starts_with(b"[`default`]")
    {
        j += br#"["default"]"#.len();
    } else {
        return None;
    }
    j = skip_ascii_ws(b, j);
    if !b[j..].starts_with(b"||") {
        return None;
    }
    j = skip_ascii_ws(b, j + 2);
    let (rhs_id, end) = match_require_id_at(b, j)?;
    (id == rhs_id).then_some((end, id))
}

fn match_require_id_at(b: &[u8], i: usize) -> Option<(String, usize)> {
    if !b[i..].starts_with(b"_r(") {
        return None;
    }
    let mut j = skip_ascii_ws(b, i + 3);
    let start = j;
    while j < b.len() && b[j].is_ascii_digit() {
        j += 1;
    }
    if j == start {
        return None;
    }
    let id = std::str::from_utf8(&b[start..j]).ok()?.to_string();
    j = skip_ascii_ws(b, j);
    if j >= b.len() || b[j] != b')' {
        return None;
    }
    Some((id, j + 1))
}

fn skip_ascii_ws(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

fn is_direct_export_alias_expr(expr: &str) -> bool {
    let mut parts = expr.split('.');
    let Some(first) = parts.next() else {
        return false;
    };
    if !is_js_identifier(first) {
        return false;
    }
    parts.all(is_js_identifier)
}

fn is_module_local_export_alias(expr: &str, module_id: usize) -> bool {
    expr.split('.')
        .next()
        .is_some_and(|ident| ident.starts_with(&format!("_m{module_id}_")))
}

fn is_js_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first == '$' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}

fn collect_direct_export_assignments(
    code: &str,
) -> HashMap<(usize, String), DirectExportAssignment> {
    let export_re = Regex::new(
        r#"_m(\d+)\.exports(?:\[\s*"([A-Za-z_$][A-Za-z0-9_$]*)"\s*\]|\.([A-Za-z_$][A-Za-z0-9_$]*))\s*="#,
    )
    .unwrap();
    let mut assignments = HashMap::new();
    let mut duplicates = HashSet::new();
    let b = code.as_bytes();

    for cap in export_re.captures_iter(code) {
        let Some(whole) = cap.get(0) else {
            continue;
        };
        if !is_statement_boundary_before(b, whole.start()) {
            continue;
        }
        let Some(id) = cap.get(1).and_then(|m| m.as_str().parse::<usize>().ok()) else {
            continue;
        };
        let Some(prop) = cap
            .get(2)
            .or_else(|| cap.get(3))
            .map(|m| m.as_str().to_string())
        else {
            continue;
        };
        let Some(expr_end) = find_direct_export_assignment_semicolon(b, whole.end()) else {
            continue;
        };
        let key = (id, prop);
        if assignments.contains_key(&key) {
            duplicates.insert(key);
            continue;
        }
        assignments.insert(
            key,
            DirectExportAssignment {
                span: (whole.start(), expr_end + 1),
                expr: code[whole.end()..expr_end].trim().to_string(),
            },
        );
    }

    for key in duplicates {
        assignments.remove(&key);
    }
    assignments
}

fn is_statement_boundary_before(b: &[u8], start: usize) -> bool {
    let mut p = start;
    while p > 0 && b[p - 1].is_ascii_whitespace() {
        p -= 1;
    }
    p == 0 || matches!(b[p - 1], b';' | b'{' | b'}')
}

fn find_direct_export_assignment_semicolon(b: &[u8], mut i: usize) -> Option<usize> {
    let mut paren = 0usize;
    let mut bracket = 0usize;
    let mut brace = 0usize;
    while i < b.len() {
        match b[i] {
            b'"' | b'\'' => {
                i = skip_quoted_literal(b, i);
                continue;
            }
            b'`' => {
                let (next, _) = scan_template_literal_expr_ranges(b, i, |_, _| 0);
                i = next;
                continue;
            }
            b'/' if i + 1 < b.len() && b[i + 1] == b'/' => {
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
            b'(' => paren += 1,
            b')' => paren = paren.saturating_sub(1),
            b'[' => bracket += 1,
            b']' => bracket = bracket.saturating_sub(1),
            b'{' => brace += 1,
            b'}' => brace = brace.saturating_sub(1),
            b';' if paren == 0 && bracket == 0 && brace == 0 => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

fn sanitize_export_local_suffix(prop: &str) -> String {
    let mut out = String::with_capacity(prop.len());
    for ch in prop.chars() {
        if ch == '_' || ch == '$' || ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "value".to_string()
    } else {
        out
    }
}

fn apply_static_replacements(code: &str, mut replacements: Vec<(usize, usize, String)>) -> String {
    replacements.sort_by_key(|(start, end, _)| (*start, *end));
    let mut out = String::with_capacity(code.len());
    let mut pos = 0usize;
    for (start, end, replacement) in replacements {
        if start < pos || end > code.len() || start > end {
            return code.to_string();
        }
        out.push_str(&code[pos..start]);
        out.push_str(&replacement);
        pos = end;
    }
    out.push_str(&code[pos..]);
    out
}

fn remove_orphan_module_alias_and_slot(code: &str, id: usize) -> String {
    let mut out = code.to_string();
    let slot = format!("_m{id}");
    let alias = format!("_m{id}e");
    let alias_decl = format!("var {alias}={slot}.exports;");
    if out.contains(&alias_decl) && count_identifier_refs(&out, &alias) <= 1 {
        out = out.replace(&alias_decl, "");
    }

    if Regex::new(&format!(r#"_r\(\s*{}\s*\)"#, id))
        .unwrap()
        .is_match(&out)
    {
        return out;
    }
    if count_identifier_refs(&out, &slot) > 2 {
        return out;
    }

    let slot_decl = format!("var {slot}={{exports:{{}}}};");
    out = out.replace(&slot_decl, "");
    replace_module_slot_entry_with_zero(&out, &slot)
}

fn replace_module_slot_entry_with_zero(code: &str, slot: &str) -> String {
    let Some(start) = code.find("var _mods=[") else {
        return code.to_string();
    };
    let body_start = start + "var _mods=[".len();
    let Some(rel_end) = code[body_start..].find("];") else {
        return code.to_string();
    };
    let body_end = body_start + rel_end;
    let mut changed = false;
    let entries: Vec<String> = code[body_start..body_end]
        .split(',')
        .map(|entry| {
            if entry == slot {
                changed = true;
                "0".to_string()
            } else {
                entry.to_string()
            }
        })
        .collect();
    if !changed {
        return code.to_string();
    }
    let mut out = String::with_capacity(code.len());
    out.push_str(&code[..body_start]);
    out.push_str(&entries.join(","));
    out.push_str(&code[body_end..]);
    out
}

/// Remove `function`/`class` declarations and `var`/`const`/`let`
/// declarations with side-effect-free initializers whose `_mN_`-prefixed
/// name has no remaining references beyond the declaration itself.
/// Returns None when nothing was removable.
fn remove_orphan_prefixed_functions(code: &str) -> Option<String> {
    use std::sync::OnceLock;
    static FUNC: OnceLock<Regex> = OnceLock::new();
    static VAR: OnceLock<Regex> = OnceLock::new();
    let func =
        FUNC.get_or_init(|| Regex::new(r"(function|class)\s+(_m\d+_[a-zA-Z0-9_$]+)\b").unwrap());
    let var_decl = VAR
        .get_or_init(|| Regex::new(r"(?:var|const|let)\s+(_m\d+_[a-zA-Z0-9_$]+)\s*=\s*").unwrap());

    let b = code.as_bytes();
    // Pass 1: collect candidate declarations (name, full span). Liveness is
    // decided by reachability from references OUTSIDE any candidate span —
    // reference counting kept mutually-referencing dead-code islands alive
    // (a dead ServerStyleSheet class calling dead helpers that call back).
    let mut candidates: Vec<(String, usize, usize)> = Vec::new();

    let statement_position = |start: usize| -> bool {
        let mut p = start;
        while p > 0 && matches!(b[p - 1], b' ' | b'\t' | b'\r' | b'\n') {
            p -= 1;
        }
        p == 0 || matches!(b[p - 1], b';' | b'{' | b'}')
    };
    let consume_tail = |mut end: usize| -> usize {
        if end < b.len() && b[end] == b';' {
            end += 1;
        }
        while end < b.len() && matches!(b[end], b' ' | b'\t') {
            end += 1;
        }
        if end < b.len() && b[end] == b'\n' {
            end += 1;
        }
        end
    };

    for cap in func.captures_iter(code) {
        let name = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let whole = cap.get(0)?;
        let start = whole.start();
        // Declaration position only: `= function`, `(class`, `return
        // function` etc. are expressions and stay.
        if !statement_position(start) {
            continue;
        }
        let kind = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let mut q = whole.end();
        if kind == "function" {
            // params (...) then body {...}
            while q < b.len() && b[q] != b'(' {
                if !matches!(b[q], b' ' | b'\t' | b'\r' | b'\n') {
                    break;
                }
                q += 1;
            }
            if q >= b.len() || b[q] != b'(' {
                continue;
            }
            let Some(params_close) = skip_code_balanced(b, q, b'(', b')') else {
                continue;
            };
            q = params_close;
        } else {
            // class: optional `extends <expr>` before the body brace.
            while q < b.len() && b[q] != b'{' {
                match b[q] {
                    b'(' => {
                        let Some(next) = skip_code_balanced(b, q, b'(', b')') else {
                            break;
                        };
                        q = next;
                    }
                    b';' | b'}' => break,
                    _ => q += 1,
                }
            }
        }
        while q < b.len() && matches!(b[q], b' ' | b'\t' | b'\r' | b'\n') {
            q += 1;
        }
        if q >= b.len() || b[q] != b'{' {
            continue;
        }
        let Some(body_close) = skip_code_balanced(b, q, b'{', b'}') else {
            continue;
        };
        candidates.push((name.to_string(), start, consume_tail(body_close)));
    }

    // var/const/let with a provably side-effect-free initializer: function
    // expressions, arrow functions, and object/array/string/number
    // literals. Anything else (calls, member chains with potential
    // getters) is left alone.
    for cap in var_decl.captures_iter(code) {
        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let whole = cap.get(0)?;
        let start = whole.start();
        if !statement_position(start) {
            continue;
        }
        let init_start = whole.end();
        let Some(init_end) = side_effect_free_initializer_end(b, init_start) else {
            continue;
        };
        // The initializer must terminate the statement (`;`) — multi-
        // declarator statements (`var a = ..., b = ...`) are skipped.
        if init_end >= b.len() || b[init_end] != b';' {
            continue;
        }
        candidates.push((name.to_string(), start, consume_tail(init_end)));
    }

    if candidates.is_empty() {
        return None;
    }
    candidates.sort_by_key(|(_, start, _)| *start);
    // Nested candidates (a function inside a class body) confuse span
    // attribution; keep outermost spans only.
    let mut outer: Vec<(String, usize, usize)> = Vec::new();
    for cand in candidates {
        if outer.last().map(|(_, _, e)| cand.1 >= *e).unwrap_or(true) {
            outer.push(cand);
        }
    }
    let candidates = outer;

    // Pass 2: reference graph. Every `_mN_*` occurrence either falls inside
    // a candidate span (edge: that candidate -> referenced name) or outside
    // (root: the name is live).
    let mut occurrences: Vec<(usize, String)> = Vec::new();
    collect_prefixed_ident_occurrences(b, &mut occurrences);
    let name_to_idx: HashMap<&str, usize> = candidates
        .iter()
        .enumerate()
        .map(|(i, (n, _, _))| (n.as_str(), i))
        .collect();
    let spans: Vec<(usize, usize)> = candidates.iter().map(|(_, s, e)| (*s, *e)).collect();
    let owner_of = |pos: usize| -> Option<usize> {
        let idx = spans.partition_point(|(s, _)| *s <= pos);
        if idx == 0 {
            return None;
        }
        let (s, e) = spans[idx - 1];
        (pos >= s && pos < e).then_some(idx - 1)
    };

    let mut edges: Vec<Vec<usize>> = vec![Vec::new(); candidates.len()];
    let mut live: Vec<bool> = vec![false; candidates.len()];
    let mut queue: Vec<usize> = Vec::new();
    for (pos, name) in &occurrences {
        let Some(&target) = name_to_idx.get(name.as_str()) else {
            continue;
        };
        match owner_of(*pos) {
            Some(owner) if owner == target => {} // self-reference
            Some(owner) => edges[owner].push(target),
            None => {
                if !live[target] {
                    live[target] = true;
                    queue.push(target);
                }
            }
        }
    }
    while let Some(i) = queue.pop() {
        for &t in &edges[i] {
            if !live[t] {
                live[t] = true;
                queue.push(t);
            }
        }
    }

    let mut removals: Vec<(usize, usize)> = candidates
        .iter()
        .enumerate()
        .filter(|(i, _)| !live[*i])
        .map(|(_, (_, s, e))| (*s, *e))
        .collect();

    if removals.is_empty() {
        return None;
    }
    removals.sort_unstable();

    let mut out = Vec::with_capacity(b.len());
    let mut pos = 0usize;
    for (start, end) in removals {
        if start < pos {
            continue;
        }
        out.extend_from_slice(&b[pos..start]);
        pos = end;
    }
    out.extend_from_slice(&b[pos..]);
    String::from_utf8(out).ok()
}

/// End offset of a side-effect-free initializer expression, or None.
/// Accepted shapes: `function [name](params){body}`, `(params) => body`,
/// `ident => body`, object/array literals, string/template/number/bool
/// literals, and `class [name] {...}` expressions.
fn side_effect_free_initializer_end(b: &[u8], mut i: usize) -> Option<usize> {
    let len = b.len();
    while i < len && matches!(b[i], b' ' | b'\t') {
        i += 1;
    }
    if i >= len {
        return None;
    }
    // function expression
    if b[i..].starts_with(b"function") {
        let mut q = i + 8;
        while q < len && b[q] != b'(' {
            if !matches!(b[q], b' ' | b'\t') && !is_id_cont_byte(b[q]) {
                return None;
            }
            q += 1;
        }
        let params_close = skip_code_balanced(b, q, b'(', b')')?;
        let mut r = params_close;
        while r < len && matches!(b[r], b' ' | b'\t' | b'\r' | b'\n') {
            r += 1;
        }
        if r >= len || b[r] != b'{' {
            return None;
        }
        return skip_code_balanced(b, r, b'{', b'}');
    }
    // arrow with parenthesized params
    if b[i] == b'(' {
        let params_close = skip_code_balanced(b, i, b'(', b')')?;
        let mut r = params_close;
        while r < len && matches!(b[r], b' ' | b'\t') {
            r += 1;
        }
        if !b[r..].starts_with(b"=>") {
            return None;
        }
        r += 2;
        while r < len && matches!(b[r], b' ' | b'\t') {
            r += 1;
        }
        if r < len && b[r] == b'{' {
            return skip_code_balanced(b, r, b'{', b'}');
        }
        return None; // expression-bodied arrows: end detection ambiguous
    }
    // class expression
    if b[i..].starts_with(b"class") {
        let mut q = i + 5;
        while q < len && b[q] != b'{' {
            match b[q] {
                b'(' => {
                    let Some(next) = skip_code_balanced(b, q, b'(', b')') else {
                        return None;
                    };
                    q = next;
                }
                b';' | b'}' => return None,
                _ => q += 1,
            }
        }
        if q >= len {
            return None;
        }
        return skip_code_balanced(b, q, b'{', b'}');
    }
    // object / array literal
    if b[i] == b'{' {
        return skip_code_balanced(b, i, b'{', b'}');
    }
    if b[i] == b'[' {
        return skip_code_balanced(b, i, b'[', b']');
    }
    // string / template literal
    if matches!(b[i], b'"' | b'\'') {
        return Some(skip_quoted_literal(b, i));
    }
    if b[i] == b'`' {
        let (next, _) = scan_template_literal_expr_ranges(b, i, |_, _| 0);
        return Some(next);
    }
    // number / boolean / null / undefined
    if b[i].is_ascii_digit() {
        let mut q = i;
        while q < len && (b[q].is_ascii_alphanumeric() || matches!(b[q], b'.' | b'x' | b'e')) {
            q += 1;
        }
        return Some(q);
    }
    for kw in [&b"true"[..], &b"false"[..], &b"null"[..], &b"undefined"[..]] {
        if b[i..].starts_with(kw) {
            let q = i + kw.len();
            if q >= len || !is_id_cont_byte(b[q]) {
                return Some(q);
            }
        }
    }
    None
}

/// Balanced-bracket skip that honors strings, templates, comments, and
/// regex literals inside the span.
fn skip_code_balanced(b: &[u8], start: usize, open: u8, close: u8) -> Option<usize> {
    debug_assert_eq!(b[start], open);
    let mut depth = 0usize;
    let mut i = start;
    let mut prev = b'(';
    while i < b.len() {
        match b[i] {
            b'"' | b'\'' => {
                i = skip_quoted_literal(b, i);
                prev = b'"';
                continue;
            }
            b'`' => {
                let (next, _) = scan_template_literal_expr_ranges(b, i, |_, _| 0);
                i = next;
                prev = b'`';
                continue;
            }
            b'/' if i + 1 < b.len() && b[i + 1] == b'/' => {
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
            b'/' if regex_context_byte(prev) => {
                i = skip_regex_literal(b, i);
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

/// `/` starts a regex (not division) after these context bytes.
fn regex_context_byte(prev: u8) -> bool {
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

/// Skip a regex literal including character classes and flags.
fn skip_regex_literal(b: &[u8], start: usize) -> usize {
    let mut i = start + 1;
    let mut in_class = false;
    while i < b.len() {
        match b[i] {
            b'\\' => i += 1,
            b'[' => in_class = true,
            b']' => in_class = false,
            b'/' if !in_class => {
                i += 1;
                while i < b.len() && b[i].is_ascii_alphabetic() {
                    i += 1;
                }
                return i;
            }
            _ => {}
        }
        i += 1;
    }
    i
}

/// Count every `_m`-prefixed identifier occurrence in one lexical sweep
/// (same string/template/comment skipping as count_identifier_refs_in_range,
/// generalized to collect all candidates instead of matching one name).
fn count_all_prefixed_identifier_refs(code: &str) -> std::collections::HashMap<String, usize> {
    let mut counts = std::collections::HashMap::new();
    collect_prefixed_refs_in_range(code.as_bytes(), 0, code.len(), &mut counts);
    counts
}

/// Per-identifier facts gathered in one sweep for R4 constant inlining.
#[derive(Default)]
struct PrefixedIdentStats {
    /// Standalone occurrences (template `${...}` expressions included,
    /// raw template text excluded) — count_identifier_refs semantics.
    count: usize,
    /// Occurrences followed by an assignment/update operator
    /// (has_mutating_identifier_ref semantics, property reads excluded).
    mutations: usize,
    /// Of those, ones immediately preceded by `var` — the declaration's
    /// own initializer, which is not a disqualifying mutation.
    decl_assignments: usize,
    /// Appears anywhere between backticks (raw or `${}`),
    /// template_literal_contains_identifier semantics.
    in_template: bool,
}

fn collect_prefixed_ident_stats(
    code: &str,
) -> std::collections::HashMap<String, PrefixedIdentStats> {
    let mut stats = std::collections::HashMap::new();
    collect_prefixed_ident_stats_in_range(code.as_bytes(), 0, code.len(), false, &mut stats);
    stats
}

fn collect_prefixed_ident_stats_in_range(
    b: &[u8],
    start: usize,
    end: usize,
    in_template: bool,
    stats: &mut std::collections::HashMap<String, PrefixedIdentStats>,
) {
    let len = end.min(b.len());
    let mut i = start.min(len);
    // Previous significant ident was `var` (decl-initializer detection).
    let mut prev_was_var = false;
    // Last significant byte for regex-vs-division disambiguation.
    let mut stats_prev = b'(';

    while i < len {
        match b[i] {
            b'"' | b'\'' => {
                i = skip_quoted_literal(b, i).min(len);
                prev_was_var = false;
                stats_prev = b'"';
                continue;
            }
            b'`' => {
                // Raw template text only marks `in_template`; the `${...}`
                // expressions are real code and recurse with the flag set.
                let tpl_start = i;
                let (next, _) = scan_template_literal_expr_ranges(b, i, |expr_start, expr_end| {
                    collect_prefixed_ident_stats_in_range(b, expr_start, expr_end, true, stats);
                    0
                });
                mark_template_raw_idents(b, tpl_start, next.min(len), stats);
                i = next.min(len);
                prev_was_var = false;
                stats_prev = b'`';
                continue;
            }
            b'/' if i + 1 < len && (b[i + 1] == b'/' || b[i + 1] == b'*') => {
                if b[i + 1] == b'/' {
                    while i < len && b[i] != b'\n' {
                        i += 1;
                    }
                } else {
                    i += 2;
                    while i + 1 < len && !(b[i] == b'*' && b[i + 1] == b'/') {
                        i += 1;
                    }
                    i = (i + 2).min(len);
                }
                continue;
            }
            b'/' if regex_context_byte(stats_prev) => {
                i = skip_regex_literal(b, i).min(len);
                stats_prev = b'/';
                continue;
            }
            _ => {}
        }
        if is_id_cont_byte(b[i]) && !b[i].is_ascii_digit() {
            let ident_start = i;
            while i < len && is_id_cont_byte(b[i]) {
                i += 1;
            }
            let ident = &b[ident_start..i];
            let was_var_kw = ident == b"var";
            if ident.len() > 2 && ident.starts_with(b"_m") {
                if let Ok(name) = std::str::from_utf8(ident) {
                    let entry = stats.entry(name.to_string()).or_default();
                    entry.count += 1;
                    if in_template {
                        entry.in_template = true;
                    }
                    let preceded_by_dot = {
                        let mut p = ident_start;
                        while p > 0 && matches!(b[p - 1], b' ' | b'\t') {
                            p -= 1;
                        }
                        p > 0 && b[p - 1] == b'.'
                    };
                    if !preceded_by_dot {
                        let mut next = i;
                        while next < len && matches!(b[next], b' ' | b'\t' | b'\r' | b'\n') {
                            next += 1;
                        }
                        let mutated = if next < len {
                            (b[next] == b'='
                                && (next + 1 >= len || !matches!(b[next + 1], b'=' | b'>')))
                                || (next + 1 < len
                                    && matches!(b[next], b'+' | b'-')
                                    && b[next + 1] == b[next])
                                || (next + 1 < len
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
                                    && b[next + 1] == b'=')
                        } else {
                            false
                        };
                        if mutated {
                            entry.mutations += 1;
                            if prev_was_var {
                                entry.decl_assignments += 1;
                            }
                        }
                    }
                }
            }
            prev_was_var = was_var_kw;
            stats_prev = b'a';
            continue;
        }
        if !matches!(b[i], b' ' | b'\t' | b'\r' | b'\n') {
            prev_was_var = false;
            stats_prev = b[i];
        }
        i += 1;
    }
}

/// Mark identifiers that appear anywhere inside a template literal span
/// (raw text included) — disqualifies them from textual inlining.
fn mark_template_raw_idents(
    b: &[u8],
    start: usize,
    end: usize,
    stats: &mut std::collections::HashMap<String, PrefixedIdentStats>,
) {
    let mut i = start;
    while i < end {
        if is_id_cont_byte(b[i]) && !b[i].is_ascii_digit() {
            let ident_start = i;
            while i < end && is_id_cont_byte(b[i]) {
                i += 1;
            }
            let ident = &b[ident_start..i];
            if ident.len() > 2 && ident.starts_with(b"_m") {
                if let Ok(name) = std::str::from_utf8(ident) {
                    stats.entry(name.to_string()).or_default().in_template = true;
                }
            }
            continue;
        }
        if b[i] == b'\\' {
            i += 2;
            continue;
        }
        i += 1;
    }
}

fn collect_prefixed_refs_in_range(
    b: &[u8],
    start: usize,
    end: usize,
    counts: &mut std::collections::HashMap<String, usize>,
) {
    let len = end.min(b.len());
    let mut i = start.min(len);
    // Last significant byte, for regex-vs-division disambiguation. Regex
    // literals may contain quotes inside character classes; scanning them
    // as strings desynchronized quote pairing and undercounted every
    // reference after the regex (orphan collection then deleted live
    // helpers in styled-components).
    let mut prev = b'(';

    while i < len {
        if matches!(b[i], b'"' | b'\'') {
            i = skip_quoted_literal(b, i).min(len);
            prev = b'"';
            continue;
        }
        if b[i] == b'/'
            && i + 1 < len
            && !matches!(b[i + 1], b'/' | b'*')
            && regex_context_byte(prev)
        {
            i = skip_regex_literal(b, i).min(len);
            prev = b'/';
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
            prev = b'a';
            continue;
        }
        if !matches!(b[i], b' ' | b'\t' | b'\r' | b'\n') {
            prev = b[i];
        }
        i += 1;
    }
}

fn collect_used_export_refs(
    code: &str,
    candidates: &HashSet<(String, String)>,
) -> HashSet<(String, String)> {
    if candidates.is_empty() {
        return HashSet::new();
    }

    let mut original_names_by_module_id: HashMap<String, HashSet<String>> = HashMap::new();
    for (obj, name) in candidates {
        if let Some(id) = module_id_from_export_obj(obj) {
            original_names_by_module_id
                .entry(id.to_string())
                .or_default()
                .insert(name.clone());
        }
    }

    let object_keys_reexports = collect_object_keys_reexport_mappings(code);
    let mut virtual_candidates: HashSet<(String, String)> = HashSet::new();
    for mapping in &object_keys_reexports {
        if let Some(names) = original_names_by_module_id.get(&mapping.source_id.to_string()) {
            for name in names {
                virtual_candidates.insert((format!("_m{}e", mapping.wrapper_id), name.clone()));
            }
        }
    }

    let mut names_by_obj: HashMap<&str, HashSet<&str>> = HashMap::new();
    let mut names_by_module_id: HashMap<&str, HashSet<&str>> = HashMap::new();
    for (obj, name) in candidates.iter().chain(virtual_candidates.iter()) {
        names_by_obj
            .entry(obj.as_str())
            .or_default()
            .insert(name.as_str());
        if let Some(id) = module_id_from_export_obj(obj) {
            names_by_module_id
                .entry(id)
                .or_default()
                .insert(name.as_str());
        }
    }

    let alias_to_module_id = require_aliases_for_modules(code, &names_by_module_id);
    let mut used = HashSet::new();
    collect_used_export_refs_in_range(
        code.as_bytes(),
        0,
        code.len(),
        &names_by_obj,
        &names_by_module_id,
        &alias_to_module_id,
        &mut used,
    );
    for mapping in &object_keys_reexports {
        let Some(names) = original_names_by_module_id.get(&mapping.source_id.to_string()) else {
            continue;
        };
        for name in names {
            if used.contains(&(format!("_m{}e", mapping.wrapper_id), name.clone())) {
                used.insert((format!("_m{}e", mapping.source_id), name.clone()));
            }
        }
    }
    used
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ObjectKeysReexportMapping {
    source_id: usize,
    wrapper_id: usize,
}

fn collect_object_keys_reexport_mappings(code: &str) -> Vec<ObjectKeysReexportMapping> {
    let re = Regex::new(
        r#"var\s+([A-Za-z_$][A-Za-z0-9_$]*)\s*=\s*(?:_r|require)\s*\(\s*(\d+)\s*\)\s*;\s*Object\.keys\s*\(\s*([A-Za-z_$][A-Za-z0-9_$]*)\s*\)\s*\.forEach\s*\(\s*function\s*\(\s*([A-Za-z_$][A-Za-z0-9_$]*)\s*\)\s*\{\s*if\s*\(\s*([A-Za-z_$][A-Za-z0-9_$]*)\s*!==\s*["']default["']\s*\)\s*_m(\d+)\.exports\s*\[\s*([A-Za-z_$][A-Za-z0-9_$]*)\s*\]\s*=\s*([A-Za-z_$][A-Za-z0-9_$]*)\s*\[\s*([A-Za-z_$][A-Za-z0-9_$]*)\s*\]\s*;\s*\}\s*\)"#,
    )
    .unwrap();
    re.captures_iter(code)
        .filter_map(|cap| {
            let alias = cap.get(1)?.as_str();
            let source_id = cap.get(2)?.as_str().parse::<usize>().ok()?;
            let object_keys_alias = cap.get(3)?.as_str();
            let key_param = cap.get(4)?.as_str();
            let if_key = cap.get(5)?.as_str();
            let wrapper_id = cap.get(6)?.as_str().parse::<usize>().ok()?;
            let output_key = cap.get(7)?.as_str();
            let rhs_alias = cap.get(8)?.as_str();
            let rhs_key = cap.get(9)?.as_str();
            (alias == object_keys_alias
                && alias == rhs_alias
                && key_param == if_key
                && key_param == output_key
                && key_param == rhs_key)
                .then_some(ObjectKeysReexportMapping {
                    source_id,
                    wrapper_id,
                })
        })
        .collect()
}

fn collect_used_export_refs_in_range(
    b: &[u8],
    start: usize,
    end: usize,
    names_by_obj: &HashMap<&str, HashSet<&str>>,
    names_by_module_id: &HashMap<&str, HashSet<&str>>,
    alias_to_module_id: &HashMap<String, String>,
    used: &mut HashSet<(String, String)>,
) {
    let len = end.min(b.len());
    let mut i = start.min(len);
    let mut prev = b'(';

    while i < len {
        if matches!(b[i], b'"' | b'\'') {
            i = skip_quoted_literal(b, i).min(len);
            prev = b'"';
            continue;
        }
        if b[i] == b'/'
            && i + 1 < len
            && !matches!(b[i + 1], b'/' | b'*')
            && regex_context_byte(prev)
        {
            i = skip_regex_literal(b, i).min(len);
            prev = b'/';
            continue;
        }
        if b[i] == b'`' {
            let (next, _) = scan_template_literal_expr_ranges(b, i, |expr_start, expr_end| {
                collect_used_export_refs_in_range(
                    b,
                    expr_start,
                    expr_end,
                    names_by_obj,
                    names_by_module_id,
                    alias_to_module_id,
                    used,
                );
                0
            });
            i = next.min(len);
            prev = b'`';
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
                i = (i + 2).min(len);
                continue;
            }
        }

        if is_id_cont_byte(b[i]) && !b[i].is_ascii_digit() {
            let ident_start = i;
            while i < len && is_id_cont_byte(b[i]) {
                i += 1;
            }
            let ident = match std::str::from_utf8(&b[ident_start..i]) {
                Ok(ident) => ident,
                Err(_) => {
                    prev = b'a';
                    continue;
                }
            };

            if let Some(names) = names_by_obj.get(ident) {
                if let Some((prop, end_ref, is_assignment)) =
                    match_any_property_access_after_base(b, i, false)
                {
                    if !is_assignment && names.contains(prop.as_str()) {
                        used.insert((ident.to_string(), prop));
                    }
                    i = end_ref;
                    prev = b'a';
                    continue;
                }
            }

            if ident == "_r" || ident == "require" {
                if let Some((module_id, after_require)) = match_require_call_any(b, ident_start) {
                    if let Some(names) = names_by_module_id.get(module_id.as_str()) {
                        if let Some((prop, end_ref, is_assignment)) =
                            match_any_property_access_after_base(b, after_require, true)
                        {
                            if !is_assignment && names.contains(prop.as_str()) {
                                used.insert((format!("_m{}e", module_id), prop));
                            }
                            i = end_ref;
                            prev = b'a';
                            continue;
                        }
                    }
                }
            } else if let Some(module_id) = alias_to_module_id.get(ident) {
                if let Some(names) = names_by_module_id.get(module_id.as_str()) {
                    if let Some((prop, end_ref, is_assignment)) =
                        match_any_property_access_after_base(b, i, true)
                    {
                        if !is_assignment && names.contains(prop.as_str()) {
                            used.insert((format!("_m{}e", module_id), prop));
                        }
                        i = end_ref;
                        prev = b'a';
                        continue;
                    }
                }
            }

            prev = b'a';
            continue;
        }

        if !matches!(b[i], b' ' | b'\t' | b'\r' | b'\n') {
            prev = b[i];
        }
        i += 1;
    }
}

fn require_aliases_for_modules(
    code: &str,
    names_by_module_id: &HashMap<&str, HashSet<&str>>,
) -> HashMap<String, String> {
    let re = Regex::new(
        r"(?:^|[;{}\n])\s*(?:var|let|const)\s+([a-zA-Z_$][a-zA-Z0-9_$]*)\s*=\s*_r\s*\(\s*(\d+)\s*\)\s*;",
    )
    .unwrap();
    re.captures_iter(code)
        .filter_map(|cap| {
            let alias = cap.get(1)?.as_str();
            let id = cap.get(2)?.as_str();
            names_by_module_id
                .contains_key(id)
                .then(|| (alias.to_string(), id.to_string()))
        })
        .collect()
}

fn match_any_property_access_after_base(
    b: &[u8],
    after_base: usize,
    allow_exports_prefix: bool,
) -> Option<(String, usize, bool)> {
    let mut i = after_base;
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }

    if allow_exports_prefix {
        if let Some(end_exports) = match_dot_property(b, i, "exports") {
            i = end_exports;
            while i < b.len() && b[i].is_ascii_whitespace() {
                i += 1;
            }
        }
    }

    if i < b.len() && b[i] == b'.' {
        let mut prop_start = i + 1;
        while prop_start < b.len() && b[prop_start].is_ascii_whitespace() {
            prop_start += 1;
        }
        if prop_start >= b.len()
            || !is_id_cont_byte(b[prop_start])
            || b[prop_start].is_ascii_digit()
        {
            return None;
        }
        let mut end_ref = prop_start + 1;
        while end_ref < b.len() && is_id_cont_byte(b[end_ref]) {
            end_ref += 1;
        }
        let prop = std::str::from_utf8(&b[prop_start..end_ref])
            .ok()?
            .to_string();
        return Some((prop, end_ref, is_assignment_after_ref(b, end_ref)));
    }

    match_string_property_access_after_base(b, i)
}

fn match_string_property_access_after_base(
    b: &[u8],
    after_base: usize,
) -> Option<(String, usize, bool)> {
    let mut i = after_base;
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
    if i >= b.len() {
        return None;
    }
    let prop = std::str::from_utf8(&b[name_start..i]).ok()?.to_string();
    i += 1;
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= b.len() || b[i] != b']' {
        return None;
    }
    let end_ref = i + 1;
    Some((prop, end_ref, is_assignment_after_ref(b, end_ref)))
}

fn match_require_call_any(b: &[u8], start: usize) -> Option<(String, usize)> {
    let req_len = if b[start..].starts_with(b"_r") {
        2
    } else if b[start..].starts_with(b"require") {
        "require".len()
    } else {
        return None;
    };
    if start > 0 && is_id_cont_byte(b[start - 1]) {
        return None;
    }
    if start + req_len < b.len() && is_id_cont_byte(b[start + req_len]) {
        return None;
    }
    let mut i = start + req_len;
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
    let id_start = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i == id_start {
        return None;
    }
    let module_id = std::str::from_utf8(&b[id_start..i]).ok()?.to_string();
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= b.len() || b[i] != b')' {
        return None;
    }
    Some((module_id, i + 1))
}

fn is_require_call_ident_at(b: &[u8], start: usize) -> bool {
    match_require_call_any(b, start).is_some()
}

fn module_id_from_export_obj(export_obj: &str) -> Option<&str> {
    let id = export_obj.strip_prefix("_m")?.strip_suffix('e')?;
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(id)
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
    let b = code.as_bytes();
    let name = var_name.as_bytes();
    let mut result = String::with_capacity(code.len());
    let mut cursor = 0usize;
    let mut i = 0usize;

    while let Some(relative_start) = code[i..].find("var") {
        let start = i + relative_start;
        let after_var = start + 3;

        if (start > 0 && is_id_cont_byte(b[start - 1]))
            || after_var >= b.len()
            || !b[after_var].is_ascii_whitespace()
        {
            i = after_var;
            continue;
        }

        let mut after_ws = after_var;
        while after_ws < b.len() && b[after_ws].is_ascii_whitespace() {
            after_ws += 1;
        }

        let after_name = after_ws + name.len();
        if after_name > b.len()
            || &b[after_ws..after_name] != name
            || (after_name < b.len() && is_id_cont_byte(b[after_name]))
        {
            i = after_var;
            continue;
        }

        let mut after_binding = after_name;
        while after_binding < b.len() && b[after_binding].is_ascii_whitespace() {
            after_binding += 1;
        }

        let statement_end = if after_binding < b.len() && b[after_binding] == b';' {
            Some(after_binding + 1)
        } else if after_binding < b.len() && b[after_binding] == b'=' {
            find_assignment_statement_end(b, after_name)
        } else {
            None
        };

        let Some(statement_end) = statement_end else {
            i = after_var;
            continue;
        };

        result.push_str(&code[cursor..start]);
        cursor = statement_end;
        i = statement_end;
    }

    result.push_str(&code[cursor..]);
    result
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
    if is_esm_module_path(module_path) {
        return true;
    }
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

fn is_esm_module_path(module_path: &std::path::Path) -> bool {
    let path = module_path.to_string_lossy().replace('\\', "/");
    path.ends_with(".mjs")
        || path.contains("/esm/")
        || path.contains(".esm.")
        || path.contains("/es/")
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
    fn test_orphan_function_collection_iterates_to_fixpoint() {
        // emphasize -> darken -> decompose: once the export assignment of
        // `emphasize` is pruned, the whole helper chain must fall out;
        // `alpha` stays because its export survives.
        let code = "var _m0e = _m0.exports;\n\
            function _m0_decompose(c){ return c; };\n\
            function _m0_darken(c){ return _m0_decompose(c); };\n\
            function _m0_emphasize(c){ return _m0_darken(c); };\n\
            function _m0_alpha(c){ return _m0_decompose(c); };\n\
            _m0e.alpha = _m0_alpha;\n\
            var _m1_x = _m0e.alpha(1);";
        let result = eliminate_unused_exports(code);
        assert!(result.contains("_m0_alpha"), "{result}");
        assert!(
            result.contains("_m0_decompose"),
            "shared dep of live alpha must stay: {result}"
        );
        assert!(!result.contains("_m0_emphasize"), "{result}");
        assert!(!result.contains("_m0_darken"), "{result}");
        // Function expressions are never collected (only declarations).
        let expr = "var _m0e=_m0.exports;_m0e.k = function _m0_keep(){}; var _m1_y=_m0e.k();";
        assert!(eliminate_unused_exports(expr).contains("_m0_keep"));
    }

    #[test]
    fn test_lower_direct_export_reads_uses_local_binding_and_drops_slot() {
        let code = r#"(function(){
var _m0={exports:{}};
var _m1={exports:{}};
var _mods=[_m0,_m1];
function _r(id){var m=_mods[id];return m?m.exports:{}}
_m1.exports["default"]=makeButton();
var Button=_r(1)["default"];
Button();
})();"#;

        let result = lower_direct_export_reads(code);

        assert!(
            result.contains("var _m1_export_default=makeButton();"),
            "export assignment should become a local binding, got: {result}"
        );
        assert!(
            result.contains("var Button=_m1_export_default;"),
            "direct require read should use local binding, got: {result}"
        );
        assert!(
            !result.contains("_r(1)[\"default\"]")
                && !result.contains("_m1.exports[\"default\"]")
                && !result.contains("var _m1={exports:{}};")
                && result.contains("var _mods=[_m0,0];"),
            "lowered module slot/export glue should be removed, got: {result}"
        );
    }

    #[test]
    fn test_lower_direct_export_reads_keeps_bare_namespace_require() {
        let code = r#"(function(){
var _m0={exports:{}};
var _m1={exports:{}};
var _mods=[_m0,_m1];
function _r(id){var m=_mods[id];return m?m.exports:{}}
_m1.exports["default"]=makeButton();
var ns=_r(1);
ns["default"]();
})();"#;

        let result = lower_direct_export_reads(code);

        assert_eq!(
            result, code,
            "bare namespace require must keep CommonJS export object semantics"
        );
    }

    #[test]
    fn test_lower_direct_export_reads_rewrites_alias_when_bare_fallback_remains() {
        let code = r#"(function(){
var _m0={exports:{}};
var _m1={exports:{}};
var _mods=[_m0,_m1];
function _r(id){var m=_mods[id];return m?m.exports:{}}
var Local=makeButton();
_m1.exports["default"]=Local;
var Button=_r(1)["default"]||_r(1);
Button();
})();"#;

        let result = lower_direct_export_reads(code);

        assert!(
            result.contains("var Button=Local||_r(1);"),
            "direct property read should lower while fallback remains, got: {result}"
        );
        assert!(
            result.contains("_m1.exports[\"default\"]=Local;")
                && result.contains("var _m1={exports:{}};"),
            "namespace fallback still needs export object glue, got: {result}"
        );
    }

    #[test]
    fn test_lower_direct_export_reads_keeps_block_local_alias_fallback() {
        let code = r#"(function(){
var _m0={exports:{}};
var _m1={exports:{}};
var _mods=[_m0,_m1];
function _r(id){var m=_mods[id];return m?m.exports:{}}
{
const _m1_Local=makeButton();
_m1.exports["default"]=_m1_Local;
}
var Button=_r(1)["default"]||_r(1);
Button();
})();"#;

        let result = lower_direct_export_reads(code);

        assert_eq!(
            result, code,
            "block-local module bindings are not visible at fallback read sites"
        );
    }

    #[test]
    fn test_lower_direct_export_reads_iterates_nested_export_deps() {
        let code = r#"(function(){
var _m0={exports:{}};
var _m1={exports:{}};
var _m2={exports:{}};
var _mods=[_m0,_m1,_m2];
function _r(id){var m=_mods[id];return m?m.exports:{}}
_m1.exports["foo"]=makeFoo();
_m2.exports["bar"]=_r(1)["foo"];
var Bar=_r(2)["bar"];
Bar();
})();"#;

        let result = lower_direct_export_reads(code);

        assert!(
            result.contains("var _m1_export_foo=makeFoo();")
                && result.contains("var _m2_export_bar=_m1_export_foo;")
                && result.contains("var Bar=_m2_export_bar;"),
            "nested direct export reads should lower over multiple passes, got: {result}"
        );
        assert!(
            !result.contains("_r(1)[\"foo\"]")
                && !result.contains("_r(2)[\"bar\"]")
                && !result.contains("_m1.exports[\"foo\"]")
                && !result.contains("_m2.exports[\"bar\"]"),
            "lowered output should not keep direct require/export glue, got: {result}"
        );
    }

    #[test]
    fn test_optimize_generated_module_glue_collapses_simple_export_slot() {
        let code = r#"(function(){var _m0={exports:{}};var _m1={exports:{}};var _mods=[_m0,_m1];function _r(id){var m=_mods[id];return m?m.exports:{}}{_m1.exports;_m1.exports.default=makeButton();var Button=_r(1).default||_r(1);Button();}})();"#;

        let result = optimize_generated_module_glue(code);

        assert!(result.contains("var _m1={}"), "{result}");
        assert!(result.contains("_m1.default=makeButton()"), "{result}");
        assert!(result.contains("var Button=_r(1,1);"), "{result}");
        assert!(!result.contains("_m1.exports;"), "{result}");
        assert!(
            result.contains(r#""exports"in m?m.exports:m"#),
            "mixed direct-export/module-object helper must be installed: {result}"
        );
    }

    #[test]
    fn test_optimize_generated_module_glue_keeps_retained_module_objects() {
        let code = r#"(function(){var _m0={exports:{}};var _m1={exports:{}};var _mods=[_m0,_m1];function _r(id){var m=_mods[id];return m?m.exports:{}}!function(module,exports){module.exports=makeButton();}(_m1,_m1.exports);var Button=_r(1).default||_r(1);Button();})();"#;

        let result = optimize_generated_module_glue(code);

        assert!(result.contains("var _m1={exports:{}}"), "{result}");
        assert!(result.contains("}(_m1,_m1.exports)"), "{result}");
        assert!(
            result.contains("var Button=_r(1,1);"),
            "default fallback can still shrink independently: {result}"
        );
    }

    #[test]
    fn test_optimize_generated_module_glue_keeps_module_exports_reassignment() {
        let code = r#"(function(){var _m0={exports:{}};var _m1={exports:{}};var _mods=[_m0,_m1];function _r(id){var m=_mods[id];return m?m.exports:{}}{_m1.exports=makeButton();var Button=_r(1).default||_r(1);Button();}})();"#;

        let result = optimize_generated_module_glue(code);

        assert!(result.contains("var _m1={exports:{}}"), "{result}");
        assert!(result.contains("_m1.exports=makeButton()"), "{result}");
    }

    #[test]
    fn test_optimize_generated_module_glue_keeps_literal_exports_export() {
        let code = r#"(function(){var _m0={exports:{}};var _m1={exports:{}};var _mods=[_m0,_m1];function _r(id){var m=_mods[id];return m?m.exports:{}}{_m1.exports.exports=makeButton();var Button=_r(1).exports;Button();}})();"#;

        let result = optimize_generated_module_glue(code);

        assert!(result.contains("var _m1={exports:{}}"), "{result}");
        assert!(
            result.contains("_m1.exports.exports=makeButton()"),
            "{result}"
        );
    }

    #[test]
    fn test_optimize_generated_module_glue_prunes_unrequired_mods_slots() {
        let code = r#"(function(){var _m0={exports:{}};var _m1={exports:{}};var _m2={exports:{}};var _mods=[_m0,_m1,_m2];function _r(id){var m=_mods[id];return m?m.exports:{}}{_m0.exports.side=side();_m1.exports.default=makeButton();}var Button=_r(1).default||_r(1);!function(module,exports,require){exports.value=require(2).default;}(_m2,_m2.exports,_r);Button();})();"#;

        let result = optimize_generated_module_glue(code);

        assert!(
            result.contains("var _mods=[,_m1,_m2]"),
            "unrequired slot must not stay pinned in the runtime table: {result}"
        );
        assert!(result.contains("_m0.side=side()"), "{result}");
        assert!(result.contains("var Button=_r(1,1);"), "{result}");
        assert!(result.contains("require(2).default"), "{result}");
    }

    #[test]
    fn test_optimize_generated_module_glue_keeps_mods_array_for_dynamic_require() {
        let code = r#"(function(){var _m0={exports:{}};var _m1={exports:{}};var _mods=[_m0,_m1];function _r(id){var m=_mods[id];return m?m.exports:{}}{_m1.exports.default=makeButton();}var id=1;var Button=_r(id).default||_r(id);Button();})();"#;

        let result = optimize_generated_module_glue(code);

        assert!(
            result.contains("var _mods=[_m0,_m1]"),
            "dynamic require keeps every slot addressable: {result}"
        );
    }

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
    fn test_eliminate_unused_exports_removes_unread_direct_exports() {
        let code = r#"(function(){var _m19={exports:{}};var _mods=[_m19];function _r(id){var m=_mods[id];return m?m.exports:{}}
function makePrefix(){return "-ms-";}
{_m19.exports["MS"]=makePrefix();_m19.exports["PAGE"]="@page";_m19.exports["SCOPE"]="@scope";}
var prefix=_r(19).MS;console.log(prefix);})();"#;

        let result = eliminate_unused_exports(code);

        assert!(
            result.contains("_m19.exports[\"MS\"]"),
            "read direct export should survive, got: {result}"
        );
        assert!(
            !result.contains("_m19.exports[\"PAGE\"]"),
            "unread direct export should be removed, got: {result}"
        );
        assert!(
            !result.contains("_m19.exports[\"SCOPE\"]"),
            "unread direct export should be removed, got: {result}"
        );
        assert!(
            crate::bundler::dce::js_parses_without_errors(&result),
            "result should remain valid JS, got: {result}"
        );
    }

    #[test]
    fn test_eliminate_unused_exports_inlines_direct_literal_export_reads() {
        let code = r#"(function(){var _m19={exports:{}};var _mods=[_m19];function _r(id){var m=_mods[id];return m?m.exports:{}}
{_m19.exports["MS"]="-ms-";_m19.exports["RULESET"]="rule";}
var prefix=_r(19).MS;var kind=_r(19).RULESET;console.log(prefix,kind);})();"#;

        let result = eliminate_unused_exports(code);

        assert!(
            result.contains(r#"var prefix="-ms-";"#),
            "direct literal export read should be inlined, got: {result}"
        );
        assert!(
            result.contains(r#"var kind="rule";"#),
            "direct literal export read should be inlined, got: {result}"
        );
        assert!(
            !result.contains("_m19.exports[\"MS\"]"),
            "inlined direct literal export assignment should be removed, got: {result}"
        );
        assert!(
            crate::bundler::dce::js_parses_without_errors(&result),
            "result should remain valid JS, got: {result}"
        );
    }

    #[test]
    fn test_eliminate_unused_exports_keeps_alias_read_direct_exports() {
        let code = r#"(function(){var _m19={exports:{}};var _mods=[_m19];function _r(id){var m=_mods[id];return m?m.exports:{}}
{_m19.exports["MS"]="-ms-";_m19.exports["PAGE"]="@page";}
var enum_ns=_r(19);console.log(enum_ns["MS"]);})();"#;

        let result = eliminate_unused_exports(code);

        assert!(
            result.contains("_m19.exports[\"MS\"]"),
            "alias-read direct export should survive, got: {result}"
        );
        assert!(
            result.contains("_m19.exports[\"PAGE\"]"),
            "bare namespace require should keep sibling direct exports, got: {result}"
        );
    }

    #[test]
    fn test_eliminate_unused_exports_tracks_generated_object_keys_reexports() {
        let code = r#"(function(){var _m13={exports:{}};var _m19={exports:{}};var _mods=[_m13,_m19];function _r(id){var m=_mods[id];return m?m.exports:{}}
{_m19.exports["MS"]="-ms-";_m19.exports["PAGE"]="@page";_m19.exports["SCOPE"]="@scope";}
var _m13___re=_r(19);Object.keys(_m13___re).forEach(function(k){if(k!=="default")_m13.exports[k]=_m13___re[k];});
var stylis=_r(13);console.log(stylis.MS);})();"#;

        let result = eliminate_unused_exports(code);

        assert!(
            result.contains("_m19.exports[\"MS\"]"),
            "statically-read re-export source export should survive, got: {result}"
        );
        assert!(
            !result.contains("_m19.exports[\"PAGE\"]"),
            "unread re-export source export should be removed, got: {result}"
        );
        assert!(
            !result.contains("_m19.exports[\"SCOPE\"]"),
            "unread re-export source export should be removed, got: {result}"
        );
        assert!(
            crate::bundler::dce::js_parses_without_errors(&result),
            "result should remain valid JS, got: {result}"
        );
    }

    #[test]
    fn test_eliminate_unused_exports_keeps_object_keys_reexports_when_wrapper_escapes() {
        let code = r#"(function(){var _m13={exports:{}};var _m19={exports:{}};var _mods=[_m13,_m19];function _r(id){var m=_mods[id];return m?m.exports:{}}
{_m19.exports["MS"]="-ms-";_m19.exports["PAGE"]="@page";}
var _m13___re=_r(19);Object.keys(_m13___re).forEach(function(k){if(k!=="default")_m13.exports[k]=_m13___re[k];});
var stylis=_r(13);console.log(Object.keys(stylis));})();"#;

        let result = eliminate_unused_exports(code);

        assert!(
            result.contains("_m19.exports[\"MS\"]"),
            "escaped wrapper namespace should keep source exports, got: {result}"
        );
        assert!(
            result.contains("_m19.exports[\"PAGE\"]"),
            "escaped wrapper namespace should keep sibling source exports, got: {result}"
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
    fn test_eliminate_unused_exports_removes_whole_object_literal_var_initializer() {
        // Ant Design cssinjs exposes an unused `_experimental` object whose
        // initializer contains a nested function and semicolons. Removing only
        // up to the first semicolon leaves an invalid object tail in the
        // flattened bundle.
        let code = r#"var _m800e = _m800.exports;
var _m800__experimental = {
  supportModernCSS: function supportModernCSS() {
    return supportWhere() && supportLogicProps();
  }
};
_m800e._experimental = _m800__experimental;
var _m801_used = 1;
console.log(_m801_used);"#;

        let result = eliminate_unused_exports(code);

        assert!(
            !result.contains("_m800__experimental"),
            "unused object literal export binding should be removed, got: {}",
            result
        );
        assert!(
            !result.contains("supportModernCSS"),
            "object literal initializer tail must not be left behind, got: {}",
            result
        );
        assert!(
            crate::bundler::dce::js_parses_without_errors(&result),
            "result should remain valid JS, got: {}",
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
    fn test_eliminate_unused_exports_keeps_retained_cjs_require_refs() {
        let code = r##"_m376e.h = function hasOwn() { return true; };
_m376e.w = function withEmotionCache() { return function Wrapped() {}; };
_m376e.unused = function unused() { return 3; };
!function(module, exports, require) {
  var hasOwn = require(376)["h"];
  var withEmotionCache = require(376).w;
  exports.Component = withEmotionCache(function() { return hasOwn(); });
}(_m375, _m375.exports, _r);"##;

        let result = eliminate_unused_exports(code);

        assert!(
            result.contains("_m376e.h"),
            "retained CJS require(id)[name] read should keep export, got: {}",
            result
        );
        assert!(
            result.contains("_m376e.w"),
            "retained CJS require(id).name read should keep export, got: {}",
            result
        );
        assert!(
            !result.contains("_m376e.unused"),
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
    fn test_esm_distribution_module_is_flatten_eligible() {
        let module = make_module(
            "/project/node_modules/@emotion/cache/dist/emotion-cache.browser.esm.js",
            "var cache = createCache();\nexports[\"default\"] = cache;",
        );
        assert!(
            is_side_effect_free(&module),
            "resolved ESM distribution modules should be eligible for flattening"
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

#[derive(Debug, Clone)]
struct ReexportWrapper {
    id: usize,
    start: usize,
    end: usize,
    exports: HashMap<String, ReexportTarget>,
}

#[derive(Debug, Clone)]
struct ReexportTarget {
    module_id: usize,
    export_name: String,
    default_interop: bool,
}

impl ReexportTarget {
    fn expr(&self) -> String {
        if self.default_interop {
            self.default_thunk_expr()
        } else {
            format!("_r({})[\"{}\"]", self.module_id, self.export_name)
        }
    }

    fn default_thunk_expr(&self) -> String {
        format!(
            "_r({})[\"{}\"] || _r({})",
            self.module_id, self.export_name, self.module_id
        )
    }
}

/// Collapse pure re-export wrapper modules after flattening.
///
/// MUI-style subpath entries often compile to a module whose only remaining
/// work is `exports["default"] = _r(leaf)["default"]`.  Keeping the wrapper
/// forces downstream code to read through `_r(wrapper)` and keeps an otherwise
/// empty module section in the bundle.  This pass redirects property reads to
/// the leaf module and removes the wrapper section, but only when every
/// `_r(wrapper)` use is a property read or the default-import thunk shape that
/// Jet itself emits.
pub fn collapse_pure_reexport_wrappers(code: &str) -> String {
    let mut current = code.to_string();
    for _ in 0..4 {
        let next = collapse_pure_reexport_wrappers_once(&current);
        if next == current {
            return current;
        }
        current = next;
    }
    current
}

fn collapse_pure_reexport_wrappers_once(code: &str) -> String {
    let mut wrappers = collect_pure_reexport_wrappers(code);
    if wrappers.is_empty() {
        return code.to_string();
    }
    resolve_reexport_wrapper_chains(&mut wrappers);

    let section_ranges: HashMap<usize, (usize, usize)> = wrappers
        .iter()
        .map(|wrapper| (wrapper.id, (wrapper.start, wrapper.end)))
        .collect();
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    let mut removed = HashSet::new();

    for wrapper in wrappers {
        let Some(mut wrapper_edits) = reexport_wrapper_read_edits(code, &wrapper) else {
            continue;
        };
        wrapper_edits.retain(|(start, _, _)| {
            !section_ranges
                .values()
                .any(|(section_start, section_end)| start >= section_start && start < section_end)
        });
        if wrapper_edits.is_empty() {
            continue;
        }
        removed.insert(wrapper.id);
        edits.append(&mut wrapper_edits);
        edits.push((wrapper.start, wrapper.end, String::new()));
    }

    if removed.is_empty() {
        return code.to_string();
    }

    apply_ordered_edits(code, edits)
}

fn resolve_reexport_wrapper_chains(wrappers: &mut [ReexportWrapper]) {
    let exports_by_id: HashMap<usize, HashMap<String, ReexportTarget>> = wrappers
        .iter()
        .map(|wrapper| (wrapper.id, wrapper.exports.clone()))
        .collect();
    for wrapper in wrappers {
        for target in wrapper.exports.values_mut() {
            *target = resolve_reexport_target(target.clone(), &exports_by_id);
        }
    }
}

fn resolve_reexport_target(
    mut target: ReexportTarget,
    exports_by_id: &HashMap<usize, HashMap<String, ReexportTarget>>,
) -> ReexportTarget {
    let mut seen = HashSet::new();
    while seen.insert(target.module_id) {
        let Some(exports) = exports_by_id.get(&target.module_id) else {
            break;
        };
        let Some(next) = exports.get(&target.export_name) else {
            break;
        };
        let default_interop = target.default_interop || next.default_interop;
        target = ReexportTarget {
            default_interop,
            ..next.clone()
        };
    }
    target
}

fn collect_pure_reexport_wrappers(code: &str) -> Vec<ReexportWrapper> {
    let mut wrappers = Vec::new();
    let mut search = 0usize;
    while let Some(rel) = code[search..].find("// Module ") {
        let start = search + rel;
        let Some(line_end_rel) = code[start..].find('\n') else {
            break;
        };
        let line_end = start + line_end_rel;
        let Some(id) = parse_module_id(&code[start..line_end]) else {
            search = line_end + 1;
            continue;
        };
        let body_start = line_end + 1;
        let end = code[body_start..]
            .find("\n// Module ")
            .map(|next| body_start + next + 1)
            .unwrap_or_else(|| {
                code[body_start..]
                    .find("\n})();")
                    .map_or(code.len(), |r| body_start + r)
            });
        if id != 0 {
            if let Some(exports) = pure_reexport_exports(id, &code[body_start..end]) {
                if !exports.is_empty() {
                    wrappers.push(ReexportWrapper {
                        id,
                        start,
                        end,
                        exports,
                    });
                }
            }
        }
        search = end;
    }
    wrappers
}

fn parse_module_id(line: &str) -> Option<usize> {
    let rest = line.strip_prefix("// Module ")?;
    let id = rest.split(':').next()?;
    id.parse().ok()
}

fn pure_reexport_exports(id: usize, section_body: &str) -> Option<HashMap<String, ReexportTarget>> {
    let mut aliases: HashMap<String, ReexportTarget> = HashMap::new();
    let mut exports = HashMap::new();
    for raw_line in section_body.lines() {
        let line = raw_line.trim();
        if line.is_empty()
            || line == "{"
            || line == "}"
            || line == "'use client';"
            || line == "\"use client\";"
            || line == "'use strict';"
            || line == "\"use strict\";"
            || line == format!("var _m{id}e=_m{id}.exports;")
            || line == format!("var _m{id}e = _m{id}.exports;")
        {
            continue;
        }
        if is_es_module_marker_line(id, line) {
            continue;
        }
        if let Some((alias, target)) = parse_reexport_alias(line) {
            if target.module_id == id {
                return None;
            }
            aliases.insert(alias, target);
            continue;
        }
        if let Some((export_name, target)) = parse_reexport_assignment(id, line, &aliases) {
            if target.module_id == id {
                return None;
            }
            exports.insert(export_name, target);
            continue;
        }
        return None;
    }
    Some(exports)
}

fn is_es_module_marker_line(id: usize, line: &str) -> bool {
    let receiver_matches = line.starts_with(&format!("Object.defineProperty(_m{id}.exports"))
        || line.starts_with(&format!("Object.defineProperty(_m{id}e"))
        || line.starts_with("Object.defineProperty(module.exports")
        || line.starts_with("Object.defineProperty(exports");
    receiver_matches && line.contains("\"__esModule\"") && line.contains("value: true")
}

fn parse_reexport_alias(line: &str) -> Option<(String, ReexportTarget)> {
    let rest = line.strip_prefix("var ")?;
    let (alias, rest) = rest.split_once('=')?;
    let alias = alias.trim();
    if alias.is_empty()
        || !alias
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'$')
    {
        return None;
    }
    let (target, rest) = parse_reexport_target_expr(rest.trim_start())?;
    if rest.trim() == ";" {
        return Some((alias.to_string(), target));
    }
    None
}

fn parse_reexport_assignment(
    id: usize,
    line: &str,
    aliases: &HashMap<String, ReexportTarget>,
) -> Option<(String, ReexportTarget)> {
    let lhs_prefixes = [format!("_m{id}.exports[\""), format!("_m{id}e[\"")];
    for prefix in lhs_prefixes {
        if let Some(rest) = line.strip_prefix(&prefix) {
            let (export_name, rest) = rest.split_once("\"]")?;
            let rest = rest.trim_start();
            let rest = rest.strip_prefix('=')?.trim_start();
            let (target, rest) = parse_assignment_target(rest, aliases)?;
            if rest.trim() == ";" {
                return Some((export_name.to_string(), target));
            }
        }
    }

    let dotted_prefixes = [format!("_m{id}.exports."), format!("_m{id}e.")];
    for prefix in dotted_prefixes {
        if let Some(rest) = line.strip_prefix(&prefix) {
            let name_end =
                rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '$'))?;
            let export_name = &rest[..name_end];
            let rest = rest[name_end..].trim_start();
            let rest = rest.strip_prefix('=')?.trim_start();
            let (target, rest) = parse_assignment_target(rest, aliases)?;
            if rest.trim() == ";" {
                return Some((export_name.to_string(), target));
            }
        }
    }
    None
}

fn parse_assignment_target<'a>(
    input: &'a str,
    aliases: &HashMap<String, ReexportTarget>,
) -> Option<(ReexportTarget, &'a str)> {
    if let Some((target, rest)) = parse_reexport_target_expr(input) {
        return Some((target, rest));
    }
    let ident_len = input
        .bytes()
        .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_' || *b == b'$')
        .count();
    if ident_len == 0 {
        return None;
    }
    let ident = &input[..ident_len];
    aliases
        .get(ident)
        .cloned()
        .map(|target| (target, &input[ident_len..]))
}

fn parse_reexport_target_expr(input: &str) -> Option<(ReexportTarget, &str)> {
    let (module_id, export_name, mut rest) = parse_require_property(input)?;
    let mut default_interop = false;
    let trimmed = rest.trim_start();
    if export_name == "default" {
        if let Some(after_or) = trimmed.strip_prefix("||") {
            let after_or = after_or.trim_start();
            if let Some(after_require) = after_or.strip_prefix(&format!("_r({module_id})")) {
                default_interop = true;
                rest = after_require;
            }
        }
    }
    Some((
        ReexportTarget {
            module_id,
            export_name,
            default_interop,
        },
        rest,
    ))
}

fn parse_require_property(input: &str) -> Option<(usize, String, &str)> {
    let rest = input.strip_prefix("_r(")?;
    let (id_raw, rest) = rest.split_once(')')?;
    let target_id = id_raw.parse().ok()?;
    let rest = rest.strip_prefix("[\"")?;
    let (name, rest) = rest.split_once("\"]")?;
    Some((target_id, name.to_string(), rest))
}

fn reexport_wrapper_read_edits(
    code: &str,
    wrapper: &ReexportWrapper,
) -> Option<Vec<(usize, usize, String)>> {
    let mut edits = Vec::new();
    collect_reexport_wrapper_read_edits_for_needle(
        code,
        wrapper,
        &format!("_r({})", wrapper.id),
        &mut edits,
    )?;
    collect_reexport_wrapper_read_edits_for_needle(
        code,
        wrapper,
        &format!("require({})", wrapper.id),
        &mut edits,
    )?;
    if edits.is_empty() {
        return None;
    }
    Some(edits)
}

fn collect_reexport_wrapper_read_edits_for_needle(
    code: &str,
    wrapper: &ReexportWrapper,
    needle: &str,
    edits: &mut Vec<(usize, usize, String)>,
) -> Option<()> {
    let mut pos = 0usize;
    while let Some(rel) = code[pos..].find(&needle) {
        let start = pos + rel;
        let after = start + needle.len();
        if start >= wrapper.start && start < wrapper.end {
            pos = after;
            continue;
        }
        let suffix = &code[after..];
        if let Some((export_name, prop_end_rel)) = parse_property_suffix(suffix) {
            let prop_end = after + prop_end_rel;
            let target = wrapper.exports.get(export_name)?;
            let (replacement, end) = if export_name == "default" {
                if let Some(thunk_end) = default_thunk_end(code, prop_end, &needle) {
                    (target.default_thunk_expr(), thunk_end)
                } else {
                    (target.expr(), prop_end)
                }
            } else {
                (target.expr(), prop_end)
            };
            edits.push((start, end, replacement));
            pos = end;
            continue;
        }
        return None;
    }
    Some(())
}

fn parse_property_suffix(suffix: &str) -> Option<(&str, usize)> {
    if let Some(rest) = suffix.strip_prefix("[\"") {
        let (name, rest) = rest.split_once("\"]")?;
        return Some((name, suffix.len() - rest.len()));
    }
    if let Some(rest) = suffix.strip_prefix('.') {
        let name_len = rest
            .bytes()
            .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_' || *b == b'$')
            .count();
        if name_len > 0 {
            return Some((&rest[..name_len], 1 + name_len));
        }
    }
    None
}

fn default_thunk_end(code: &str, prop_end: usize, needle: &str) -> Option<usize> {
    let b = code.as_bytes();
    let mut i = skip_ws_bytes(b, prop_end);
    if !code[i..].starts_with("||") {
        return None;
    }
    i = skip_ws_bytes(b, i + 2);
    if code[i..].starts_with(needle) {
        return Some(i + needle.len());
    }
    None
}

fn skip_ws_bytes(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && matches!(b[i], b' ' | b'\n' | b'\r' | b'\t') {
        i += 1;
    }
    i
}

fn apply_ordered_edits(code: &str, mut edits: Vec<(usize, usize, String)>) -> String {
    if edits.is_empty() {
        return code.to_string();
    }
    edits.sort_by_key(|(start, end, _)| (*start, *end));
    let mut out = String::with_capacity(code.len());
    let mut pos = 0usize;
    for (start, end, replacement) in edits {
        if start < pos {
            continue;
        }
        out.push_str(&code[pos..start]);
        out.push_str(&replacement);
        pos = end;
    }
    out.push_str(&code[pos..]);
    out
}

#[cfg(test)]
mod reexport_wrapper_collapse_tests {
    use super::*;

    #[test]
    fn collapses_default_reexport_wrapper_and_redirects_default_thunk() {
        let code = r#"(function(){
var _m0={exports:{}};
var _m1={exports:{}};
var _m2={exports:{}};
var _mods=[_m0,_m1,_m2];
function _r(id){var m=_mods[id];return m?m.exports:{}}

// Module 2: leaf.js
{
var _m2e=_m2.exports;
const Leaf = function Leaf(){};
_m2.exports["default"] = Leaf;
}

// Module 1: wrapper.js
{
var _m1e=_m1.exports;
'use client';
_m1.exports["default"] = _r(2)["default"];
}

// Module 0: entry.js
{
var _m0e=_m0.exports;
var Button = _r(1)["default"] || _r(1);
Button();
}
})();
"#;

        let out = collapse_pure_reexport_wrappers(code);

        assert!(!out.contains("// Module 1: wrapper.js"), "{out}");
        assert!(
            out.contains(r#"var Button = _r(2)["default"] || _r(2);"#),
            "{out}"
        );
    }

    #[test]
    fn collapses_named_reexport_property_reads() {
        let code = r#"(function(){
var _m0={exports:{}};
var _m1={exports:{}};
var _m2={exports:{}};
var _mods=[_m0,_m1,_m2];
function _r(id){var m=_mods[id];return m?m.exports:{}}

// Module 2: leaf.js
{
var _m2e=_m2.exports;
_m2.exports["bar"] = 1;
}

// Module 1: wrapper.js
{
var _m1e=_m1.exports;
_m1.exports["foo"] = _r(2)["bar"];
}

// Module 0: entry.js
{
var _m0e=_m0.exports;
var value = _r(1)["foo"];
}
})();
"#;

        let out = collapse_pure_reexport_wrappers(code);

        assert!(!out.contains("// Module 1: wrapper.js"), "{out}");
        assert!(out.contains(r#"var value = _r(2)["bar"];"#), "{out}");
    }

    #[test]
    fn collapses_named_reexport_property_reads_inside_runtime_wrappers() {
        let code = r#"(function(){
var _m0={exports:{}};
var _m1={exports:{}};
var _m2={exports:{}};
var _mods=[_m0,_m1,_m2];
function _r(id){var m=_mods[id];return m?m.exports:{}}

// Module 2: FastColor.js
{
var _m2e=_m2.exports;
class FastColor {}
_m2.exports["FastColor"] = FastColor;
}

// Module 1: index.js
{
var _m1e=_m1.exports;
_m1.exports["FastColor"] = _r(2)["FastColor"];
}

// Module 0: generate.js
!function(module,exports,require){
var FastColor = require(1)["FastColor"];
exports.default = function generate(color) { return new FastColor(color); };
}(_m0,_m0.exports,_r);
})();
"#;

        let out = collapse_pure_reexport_wrappers(code);

        assert!(!out.contains("// Module 1: index.js"), "{out}");
        assert!(
            out.contains(r#"var FastColor = _r(2)["FastColor"];"#),
            "{out}"
        );
        assert!(!out.contains(r#"require(1)["FastColor"]"#), "{out}");
    }

    #[test]
    fn collapses_chained_reexport_wrappers_without_dangling_intermediate_reads() {
        let code = r#"(function(){
var _m0={exports:{}};
var _m1={exports:{}};
var _m2={exports:{}};
var _m3={exports:{}};
var _mods=[_m0,_m1,_m2,_m3];
function _r(id){var m=_mods[id];return m?m.exports:{}}

// Module 3: leaf.js
{
var _m3e=_m3.exports;
_m3.exports["default"] = function Leaf(){};
}

// Module 2: default-wrapper.js
{
var _m2e=_m2.exports;
_m2.exports["default"] = _r(3)["default"];
}

// Module 1: named-wrapper.js
{
var _m1e=_m1.exports;
_m1.exports["usePreviousProps"] = _r(2)["default"];
}

// Module 0: entry.js
{
var _m0e=_m0.exports;
var usePreviousProps = _r(1)["usePreviousProps"];
usePreviousProps();
}
})();
"#;

        let out = collapse_pure_reexport_wrappers(code);

        assert!(!out.contains("// Module 1: named-wrapper.js"), "{out}");
        assert!(
            out.contains(r#"var usePreviousProps = _r(3)["default"];"#),
            "{out}"
        );
        assert!(
            !out.contains(r#"var usePreviousProps = _r(2)["default"];"#),
            "{out}"
        );
    }

    #[test]
    fn resolves_chained_reexports_when_intermediate_wrapper_also_has_direct_reads() {
        let code = r#"(function(){
var _m0={exports:{}};
var _m1={exports:{}};
var _m2={exports:{}};
var _m3={exports:{}};
var _mods=[_m0,_m1,_m2,_m3];
function _r(id){var m=_mods[id];return m?m.exports:{}}

// Module 3: leaf.js
{
var _m3e=_m3.exports;
_m3.exports["default"] = function Leaf(){};
}

// Module 2: default-wrapper.js
{
var _m2e=_m2.exports;
_m2.exports["default"] = _r(3)["default"];
}

// Module 1: named-wrapper.js
{
var _m1e=_m1.exports;
_m1.exports["usePreviousProps"] = _r(2)["default"];
}

// Module 0: entry.js
{
var _m0e=_m0.exports;
var viaNamed = _r(1)["usePreviousProps"];
var viaIntermediate = _r(2)["default"] || _r(2);
viaNamed();
viaIntermediate();
}
})();
"#;

        let out = collapse_pure_reexport_wrappers(code);

        assert!(!out.contains("// Module 1: named-wrapper.js"), "{out}");
        assert!(!out.contains("// Module 2: default-wrapper.js"), "{out}");
        assert!(out.contains(r#"var viaNamed = _r(3)["default"];"#), "{out}");
        assert!(
            out.contains(r#"var viaIntermediate = _r(3)["default"] || _r(3);"#),
            "{out}"
        );
        assert!(!out.contains("_r(2)"), "{out}");
    }

    #[test]
    fn collapses_default_interop_alias_wrapper() {
        let code = r#"(function(){
var _m0={exports:{}};
var _m1={exports:{}};
var _m2={exports:{}};
var _mods=[_m0,_m1,_m2];
function _r(id){var m=_mods[id];return m?m.exports:{}}

// Module 2: leaf.js
{
var _m2e=_m2.exports;
_m2.exports["default"] = function Leaf(){};
}

// Module 1: alias-wrapper.js
{
var _m1e=_m1.exports;
var _m1_leaf = _r(2)["default"] || _r(2);
_m1.exports["default"] = _m1_leaf;
}

// Module 0: entry.js
{
var _m0e=_m0.exports;
var viaDefault = _r(1)["default"];
var viaThunk = _r(1)["default"] || _r(1);
viaDefault();
viaThunk();
}
})();
"#;

        let out = collapse_pure_reexport_wrappers(code);

        assert!(!out.contains("// Module 1: alias-wrapper.js"), "{out}");
        assert!(
            out.contains(r#"var viaDefault = _r(2)["default"] || _r(2);"#),
            "{out}"
        );
        assert!(
            out.contains(r#"var viaThunk = _r(2)["default"] || _r(2);"#),
            "{out}"
        );
        assert!(!out.contains("_r(1)"), "{out}");
    }

    #[test]
    fn keeps_wrapper_when_namespace_object_is_read() {
        let code = r#"(function(){
var _m0={exports:{}};
var _m1={exports:{}};
var _m2={exports:{}};
var _mods=[_m0,_m1,_m2];
function _r(id){var m=_mods[id];return m?m.exports:{}}

// Module 2: leaf.js
{
var _m2e=_m2.exports;
_m2.exports["default"] = function Leaf(){};
}

// Module 1: wrapper.js
{
var _m1e=_m1.exports;
_m1.exports["default"] = _r(2)["default"];
}

// Module 0: entry.js
{
var _m0e=_m0.exports;
var namespace = _r(1);
}
})();
"#;

        let out = collapse_pure_reexport_wrappers(code);

        assert!(out.contains("// Module 1: wrapper.js"), "{out}");
        assert!(out.contains("var namespace = _r(1);"), "{out}");
    }
}

/// Hoist repeated default-interop thunks into one cached var per module.
///
/// Every default import of module N lowers to `_r(N)["default"] || _r(N)`
/// inline — 668 copies on the MUI corpus bundle (~16KB). Modules execute
/// in dependency order in the flat bundle, so a single
/// `var _di<N> = _r(N)["default"] || _r(N);` placed right after module
/// N's block is initialized before any consumer runs. Runs pre-minify
/// (module banners must still be present to locate the blocks).
pub fn hoist_default_interop_thunks(code: &str) -> String {
    use std::sync::OnceLock;
    static THUNK: OnceLock<Regex> = OnceLock::new();
    let thunk = THUNK
        .get_or_init(|| Regex::new(r#"_r\((\d+)\)\["default"\]\s*\|\|\s*_r\((\d+)\)"#).unwrap());

    let mut counts: HashMap<usize, usize> = HashMap::new();
    for cap in thunk.captures_iter(code) {
        let (Ok(a), Ok(b)) = (cap[1].parse::<usize>(), cap[2].parse::<usize>()) else {
            continue;
        };
        if a == b {
            *counts.entry(a).or_insert(0) += 1;
        }
    }
    let hoistable: std::collections::HashSet<usize> = counts
        .iter()
        .filter(|(_, n)| **n >= 2)
        .map(|(id, _)| *id)
        .collect();
    if hoistable.is_empty() {
        return code.to_string();
    }

    // Insert after the module's whole section (just before the next
    // banner / at EOF). Brace-matching the first block was wrong for
    // retained-wrapper modules — `!function(...){body}(args);` got its
    // call split off the function expression and the module never
    // initialized (React booted with undefined internals).
    let mut insert_at: HashMap<usize, usize> = HashMap::new();
    for id in &hoistable {
        let banner = format!("// Module {id}: ");
        let Some(pos) = code.find(&banner) else {
            continue;
        };
        let section_end = code[pos..]
            .find("\n// Module ")
            .map(|rel| pos + rel)
            .unwrap_or(code.len());
        insert_at.insert(*id, section_end);
    }

    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    for cap in thunk.captures_iter(code) {
        let (Ok(a), Ok(bb)) = (cap[1].parse::<usize>(), cap[2].parse::<usize>()) else {
            continue;
        };
        if a != bb || !insert_at.contains_key(&a) {
            continue;
        }
        let whole = cap.get(0).unwrap();
        // Occurrences before the cached var's init point keep the inline
        // thunk (shouldn't happen in dependency order, but stay safe).
        if whole.start() <= insert_at[&a] {
            continue;
        }
        edits.push((whole.start(), whole.end(), format!("_di{a}")));
    }
    for (id, pos) in &insert_at {
        edits.push((
            *pos,
            *pos,
            format!("\nvar _di{id} = _r({id})[\"default\"] || _r({id});"),
        ));
    }
    if edits.is_empty() {
        return code.to_string();
    }
    edits.sort_by_key(|(start, end, _)| (*start, *end));

    let mut out = String::with_capacity(code.len());
    let mut posn = 0usize;
    for (start, end, replacement) in edits {
        if start < posn {
            continue;
        }
        out.push_str(&code[posn..start]);
        out.push_str(&replacement);
        posn = end;
    }
    out.push_str(&code[posn..]);
    out
}

/// Collect every standalone `_m<N>_*` identifier occurrence (position,
/// name), with the same string/template/comment/regex skipping as the
/// reference counters.
fn collect_prefixed_ident_occurrences(b: &[u8], out: &mut Vec<(usize, String)>) {
    collect_prefixed_ident_occurrences_in_range(b, 0, b.len(), out);
}

fn collect_prefixed_ident_occurrences_in_range(
    b: &[u8],
    start: usize,
    end: usize,
    out: &mut Vec<(usize, String)>,
) {
    let len = end.min(b.len());
    let mut i = start.min(len);
    let mut prev = b'(';
    while i < len {
        match b[i] {
            b'"' | b'\'' => {
                i = skip_quoted_literal(b, i).min(len);
                prev = b'"';
                continue;
            }
            b'`' => {
                let (next, _) = scan_template_literal_expr_ranges(b, i, |es, ee| {
                    collect_prefixed_ident_occurrences_in_range(b, es, ee, out);
                    0
                });
                i = next.min(len);
                prev = b'`';
                continue;
            }
            b'/' if i + 1 < len && b[i + 1] == b'/' => {
                while i < len && b[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            b'/' if i + 1 < len && b[i + 1] == b'*' => {
                i += 2;
                while i + 1 < len && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(len);
                continue;
            }
            b'/' if regex_context_byte(prev) => {
                i = skip_regex_literal(b, i).min(len);
                prev = b'/';
                continue;
            }
            _ => {}
        }
        if is_id_cont_byte(b[i]) && !b[i].is_ascii_digit() {
            let start = i;
            while i < len && is_id_cont_byte(b[i]) {
                i += 1;
            }
            let ident = &b[start..i];
            if ident.len() > 3 && ident.starts_with(b"_m") {
                if let Ok(name) = std::str::from_utf8(ident) {
                    if name[2..].chars().next().map(|c| c.is_ascii_digit()) == Some(true)
                        && name.contains('_')
                        && name[2..].contains('_')
                    {
                        out.push((start, name.to_string()));
                    }
                }
            }
            prev = b'a';
            continue;
        }
        if !matches!(b[i], b' ' | b'\t' | b'\r' | b'\n') {
            prev = b[i];
        }
        i += 1;
    }
}
