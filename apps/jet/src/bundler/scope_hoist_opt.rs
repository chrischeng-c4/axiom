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
    eliminate_unused_exports_inner(code, None)
}

/// Like [`eliminate_unused_exports`], but preserves the public export object
/// of the production bundle entry. Its exports can be consumed by the host
/// after the bundle runs, so no in-bundle read is required to make them live.
pub fn eliminate_unused_exports_preserving_entry(code: &str, entry_module_id: usize) -> String {
    eliminate_unused_exports_inner(code, Some(entry_module_id))
}

fn eliminate_unused_exports_inner(code: &str, entry_module_id: Option<usize>) -> String {
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
        .filter(|candidate| {
            module_id_from_export_obj(&candidate.0).and_then(|id| id.parse::<usize>().ok())
                != entry_module_id
                && !used_exports.contains(candidate)
        })
        .collect();

    let direct_exports_to_remove: Vec<(usize, usize, String)> = direct_export_assignments
        .iter()
        .filter_map(|((id, export_name), assignment)| {
            let canonical = (format!("_m{id}e"), export_name.clone());
            (Some(*id) != entry_module_id
                && !used_exports.contains(&canonical)
                && !bare_required_modules.contains(id))
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
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
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
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
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
    if let Some(start) = code.find("var _mods=[") {
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
        return out;
    }

    // generate_entry_flat_region (issue #1993) emits the flat-region module
    // map as a sparse object literal instead (`var _mods={0:_m0,1:_m1};`),
    // since flat-region module ids are not contiguous from 0 the way
    // generate_flattened_bundle's dense array is. Same neutralization
    // intent as the array case above: blank the matching `id:_mN` entry's
    // value to `0` so `_r(id)` still safely falls through to
    // `__jet__.require(id)` instead of leaving a dangling reference to the
    // slot declaration remove_orphan_module_alias_and_slot just removed.
    let Some(start) = code.find("var _mods={") else {
        return code.to_string();
    };
    let body_start = start + "var _mods={".len();
    let Some(rel_end) = code[body_start..].find("};") else {
        return code.to_string();
    };
    let body_end = body_start + rel_end;
    let Some(id_str) = slot.strip_prefix("_m") else {
        return code.to_string();
    };
    let target_entry = format!("{id_str}:{slot}");
    let mut changed = false;
    let entries: Vec<String> = code[body_start..body_end]
        .split(',')
        .map(|entry| {
            if entry == target_entry {
                changed = true;
                format!("{id_str}:0")
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

/// Region-wide indices for deciding, per elision-touched module id, whether
/// its `_m{id}e` exports alias and/or `_m{id}` module-object slot have
/// become orphaned. Built ONCE per `elide_same_chunk_export_bindings` call
/// instead of `remove_orphan_module_alias_and_slot`'s old per-id sequential
/// region rescans (O(touched_modules x region) -> O(region), #2133). Only
/// used by `elide_same_chunk_export_bindings`; `lower_direct_export_reads`
/// still drives `remove_orphan_module_alias_and_slot` directly, unchanged.
struct OrphanCleanupIndex {
    /// Exact `_m<digits>` / `_m<digits>e` token text -> region-wide
    /// occurrence count (see `collect_module_slot_alias_ref_counts`).
    ident_counts: HashMap<String, usize>,
    /// Exact digit text captured from every unanchored, lexically-unaware
    /// `_r(<ws>digits<ws>)` occurrence in the region (see
    /// `collect_bare_r_call_id_texts`).
    bare_r_call_id_texts: HashSet<String>,
    /// `var _m<id>e=_m<id>.exports;` declaration span(s), by id.
    alias_decl_spans: HashMap<usize, Vec<(usize, usize)>>,
    /// `var _m<id>={exports:{}};` declaration span(s), by id.
    slot_decl_spans: HashMap<usize, Vec<(usize, usize)>>,
    /// `_mods` array/object-literal entry span(s), keyed by exact entry
    /// text (e.g. `_m13` for the dense-array form, `13:_m13` for the
    /// sparse entry-flatten object form).
    mods_entry_spans: HashMap<String, Vec<(usize, usize)>>,
    /// Whether the located `_mods` list is the dense-array form
    /// (`var _mods=[_m0,_m1,...]`) rather than the entry-flatten sparse
    /// object form (`var _mods={0:_m0,...}`, #1993) — decides both the
    /// target entry text and its zeroed replacement per id.
    mods_is_array_form: bool,
}

fn build_orphan_cleanup_index(code: &str) -> OrphanCleanupIndex {
    let (mods_entry_spans, mods_is_array_form) = match locate_mods_body(code) {
        Some((body_start, body_end, is_array)) => {
            (index_mods_entries(code, body_start, body_end), is_array)
        }
        None => (HashMap::new(), false),
    };
    OrphanCleanupIndex {
        ident_counts: collect_module_slot_alias_ref_counts(code),
        bare_r_call_id_texts: collect_bare_r_call_id_texts(code),
        alias_decl_spans: collect_alias_decl_spans(code),
        slot_decl_spans: collect_slot_decl_spans(code),
        mods_entry_spans,
        mods_is_array_form,
    }
}

/// Per-touched-module orphan cleanup decisions, computed purely from
/// `index` lookups (O(1) each) instead of `remove_orphan_module_alias_and_slot`'s
/// per-id region rescans. Mirrors that function's exact decision order and
/// edge-case asymmetries:
/// - alias-decl removal is gated on the decl span actually being found
///   (`.contains(&alias_decl)` in the original) AND its own ref count;
///   removing it drops the paired slot token's count by exactly one (the
///   alias decl's own `_m{id}.exports` reference), so that adjustment is
///   applied before the slot-count eligibility check below.
/// - slot-decl removal is *not* gated on a decl span being found (the
///   original calls `.replace(&slot_decl, "")` unconditionally once the
///   count check passes); `_mods` zeroing is attempted independently of
///   whether a slot-decl span was actually found, matching the original's
///   unconditional tail call into `replace_module_slot_entry_with_zero`.
fn collect_orphan_cleanup_replacements(
    index: &OrphanCleanupIndex,
    touched_modules: &HashSet<usize>,
) -> Vec<(usize, usize, String)> {
    let mut replacements: Vec<(usize, usize, String)> = Vec::new();

    for &id in touched_modules {
        let slot = format!("_m{id}");
        let alias = format!("_m{id}e");

        let mut alias_removed = false;
        if let Some(spans) = index.alias_decl_spans.get(&id) {
            let alias_count = index.ident_counts.get(&alias).copied().unwrap_or(0);
            if alias_count <= 1 {
                for &(start, end) in spans {
                    replacements.push((start, end, String::new()));
                }
                alias_removed = true;
            }
        }

        if index.bare_r_call_id_texts.contains(id.to_string().as_str()) {
            continue;
        }

        let slot_count = index.ident_counts.get(&slot).copied().unwrap_or(0);
        let effective_slot_count = slot_count.saturating_sub(if alias_removed { 1 } else { 0 });
        if effective_slot_count > 2 {
            continue;
        }

        if let Some(spans) = index.slot_decl_spans.get(&id) {
            for &(start, end) in spans {
                replacements.push((start, end, String::new()));
            }
        }

        let (target_entry, zeroed) = if index.mods_is_array_form {
            (slot.clone(), "0".to_string())
        } else {
            (format!("{id}:{slot}"), format!("{id}:0"))
        };
        if let Some(spans) = index.mods_entry_spans.get(&target_entry) {
            for &(start, end) in spans {
                replacements.push((start, end, zeroed.clone()));
            }
        }
    }

    replacements
}

/// Every id whose bare `_r(id)` require call-site appears anywhere in
/// `code`, keyed by the *exact digit text* captured (not the parsed
/// number) — matching `remove_orphan_module_alias_and_slot`'s original
/// per-id `Regex::new(&format!(r"_r\(\s*{}\s*\)", id))` check byte-for-byte
/// (a zero-padded id text like `007` would not satisfy an `id.to_string()`
/// lookup of `7`, exactly as the original per-id literal-digit pattern
/// would not match it either), but compiled and scanned ONCE for the whole
/// region instead of once per touched module id.
fn collect_bare_r_call_id_texts(code: &str) -> HashSet<String> {
    use std::sync::OnceLock;
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"_r\(\s*(\d+)\s*\)").unwrap());
    re.captures_iter(code)
        .map(|cap| cap[1].to_string())
        .collect()
}

/// `var _m<id>e=_m<id>.exports;` declaration span(s), by id. The regex
/// crate has no backreferences, so the "same id on both sides" requirement
/// implied by the original string-built `format!("var {alias}={slot}.exports;")`
/// is enforced with two capture groups plus an explicit equality check.
fn collect_alias_decl_spans(code: &str) -> HashMap<usize, Vec<(usize, usize)>> {
    use std::sync::OnceLock;
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"var _m(\d+)e=_m(\d+)\.exports;").unwrap());
    let mut spans: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();
    for caps in re.captures_iter(code) {
        if caps[1] != caps[2] {
            continue;
        }
        if let Ok(id) = caps[1].parse::<usize>() {
            let m = caps.get(0).unwrap();
            spans.entry(id).or_default().push((m.start(), m.end()));
        }
    }
    spans
}

/// `var _m<id>={exports:{}};` declaration span(s), by id.
fn collect_slot_decl_spans(code: &str) -> HashMap<usize, Vec<(usize, usize)>> {
    use std::sync::OnceLock;
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"var _m(\d+)=\{exports:\{\}\};").unwrap());
    let mut spans: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();
    for caps in re.captures_iter(code) {
        if let Ok(id) = caps[1].parse::<usize>() {
            let m = caps.get(0).unwrap();
            spans.entry(id).or_default().push((m.start(), m.end()));
        }
    }
    spans
}

/// Locates the flat-region `_mods` module-slot list body exactly once,
/// mirroring `replace_module_slot_entry_with_zero`'s per-call `.find()`
/// pair (dense array form from `generate_flattened_bundle`, checked first,
/// else the sparse object form from `generate_entry_flat_region`, #1993) —
/// but paid ONE time for the whole region instead of once per touched
/// module id. Returns `(body_start, body_end, is_array_form)`.
fn locate_mods_body(code: &str) -> Option<(usize, usize, bool)> {
    if let Some(start) = code.find("var _mods=[") {
        let body_start = start + "var _mods=[".len();
        let rel_end = code[body_start..].find("];")?;
        return Some((body_start, body_start + rel_end, true));
    }
    let start = code.find("var _mods={")?;
    let body_start = start + "var _mods={".len();
    let rel_end = code[body_start..].find("};")?;
    Some((body_start, body_start + rel_end, false))
}

/// Splits a `_mods` list body on literal `,` exactly like
/// `replace_module_slot_entry_with_zero`'s `str::split(',')` (deliberately
/// no whitespace trimming, matching its exact-text `entry == slot`
/// comparison), recording each entry's byte span keyed by its exact text
/// so later per-id lookups are O(1) instead of a fresh split+scan per id.
fn index_mods_entries(
    code: &str,
    body_start: usize,
    body_end: usize,
) -> HashMap<String, Vec<(usize, usize)>> {
    let mut spans: HashMap<String, Vec<(usize, usize)>> = HashMap::new();
    let b = code.as_bytes();
    let mut entry_start = body_start;
    let mut i = body_start;
    while i <= body_end {
        if i == body_end || b[i] == b',' {
            spans
                .entry(code[entry_start..i].to_string())
                .or_default()
                .push((entry_start, i));
            entry_start = i + 1;
        }
        i += 1;
    }
    spans
}

/// Region-wide occurrence counts of every `_m<digits>` (module-object
/// "slot") and `_m<digits>e` (exports-alias) identifier token, keyed by
/// exact token text. Replaces the per-candidate `count_identifier_refs`
/// sweep `remove_orphan_module_alias_and_slot` used to run once per touched
/// module id (#2133). Deliberately mirrors `count_identifier_refs_in_range`'s
/// exact word-boundary + not-preceded-by-`.` + string/template/comment-skip
/// semantics byte-for-byte (including its lack of regex-literal awareness,
/// unlike `collect_prefixed_ident_occurrences_in_range`) so every count this
/// produces is identical to what a direct `count_identifier_refs(code, name)`
/// call on the same `name` would have returned.
fn collect_module_slot_alias_ref_counts(code: &str) -> HashMap<String, usize> {
    let b = code.as_bytes();
    let len = b.len();
    let mut counts: HashMap<String, usize> = HashMap::new();
    collect_module_slot_alias_ref_counts_in_range(b, 0, len, &mut counts);
    counts
}

fn collect_module_slot_alias_ref_counts_in_range(
    b: &[u8],
    start: usize,
    end: usize,
    counts: &mut HashMap<String, usize>,
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
                collect_module_slot_alias_ref_counts_in_range(b, expr_start, expr_end, counts);
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

        if is_id_start_byte(b[i]) {
            let tok_start = i;
            i += 1;
            while i < len && is_id_cont_byte(b[i]) {
                i += 1;
            }
            let tok = &b[tok_start..i];
            if tok.len() > 2 && tok[0] == b'_' && tok[1] == b'm' && tok[2].is_ascii_digit() {
                let mut d = 2;
                while d < tok.len() && tok[d].is_ascii_digit() {
                    d += 1;
                }
                let is_slot = d == tok.len();
                let is_alias = !is_slot && d == tok.len() - 1 && tok[d] == b'e';
                if is_slot || is_alias {
                    let mut p = tok_start;
                    while p > 0 && matches!(b[p - 1], b' ' | b'\t') {
                        p -= 1;
                    }
                    if p == 0 || b[p - 1] != b'.' {
                        if let Ok(name) = std::str::from_utf8(tok) {
                            *counts.entry(name.to_string()).or_insert(0) += 1;
                        }
                    }
                }
            }
            continue;
        }

        i += 1;
    }
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

// ──────────────────────────────────────────────────────────────────────────
// R7: Same-chunk export-binding elision (#2128)
// ──────────────────────────────────────────────────────────────────────────

// <HANDWRITE gap="missing-generator:logic" tracker="pending-tracker" reason="logic section in scope_hoist_opt.rs is hand-written pending codegen support">
/// Counters for `elide_same_chunk_export_bindings`, surfaced via
/// `JET_BUNDLE_TIMING` as `export-elision: modules=N elided_keys=M kept=K
/// kept_registry=.. kept_cross_chunk=.. kept_namespace=..
/// kept_string_indexed=.. kept_barrel_glue=.. kept_other=..` (#2139: `kept`
/// used to be a single opaque aggregate, blocking any ranking of which
/// conservatism rung of the keep ladder costs the most bytes — see
/// [`ExportKeepReason`] for the exact code arm each `kept_*` field is
/// attributed from, including two rungs this pass cannot actually
/// distinguish today).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExportElisionStats {
    /// Distinct modules with at least one export key elided.
    pub modules: usize,
    /// Export keys whose `_m{id}.exports[...] = ...` assignment was
    /// dropped in favor of direct local-binding reads.
    pub elided_keys: usize,
    /// Export key assignments inspected but conservatively left alone.
    /// Equal to `kept_registry + kept_cross_chunk + kept_namespace +
    /// kept_string_indexed + kept_barrel_glue + kept_other` (checked by a
    /// `debug_assert_eq!` where this struct is finalized).
    pub kept: usize,
    /// Kept because a read of this key was observed through
    /// `require(id).key` rather than `_r(id).key` — registry-residue
    /// consumer code living in a different lexical scope than the flat
    /// region (see `ExportKeyUsage::require_side`). This text-only pass
    /// cannot see *why* the consumer used `require()` instead of `_r()`,
    /// so it also absorbs the issue's separately-named "cross-chunk
    /// reference" rung: a cross-chunk consumer reads an eligible producer
    /// through the same runtime registry accessor as a registry-resident
    /// one does, with no distinguishing signal available here — see
    /// `kept_cross_chunk`.
    pub kept_registry: usize,
    /// Always 0 today. A *cross-chunk-referenced* producer never reaches
    /// this pass as an `_m{id}.exports[...]` candidate at all — the
    /// entry-flatten partition keeps it CJS-wrapped, same as a
    /// registry-resident producer (see the "keep triggers satisfied by
    /// construction" note on `elide_same_chunk_export_bindings_unvalidated`).
    /// A cross-chunk *consumer* reading an otherwise-eligible producer's
    /// key is indistinguishable, from this pass, from a registry-resident
    /// consumer — both are a `require(id).key` read, counted under
    /// `kept_registry` instead. Kept as a named field (rather than
    /// dropped) so the counter schema matches the issue's five-rung keep
    /// ladder and this non-attribution is visible instead of silent.
    pub kept_cross_chunk: usize,
    /// Kept because the module has a bare/namespace/computed-key
    /// `_r(id)`/`require(id)` occurrence anywhere
    /// (`escaped_ids.contains(id)` — see
    /// `bare_require_module_ids_excluding_default_fallback`). Also
    /// absorbs the issue's separately-named "string-indexed access" rung:
    /// a computed subscript `_r(id)[expr]` fails
    /// `match_any_property_access_after_base`'s literal-shape match
    /// exactly like a bare namespace read (`_r(id)` with no property
    /// access at all) does, so both flag the *whole module* as escaped
    /// with no per-access-site distinction available — see
    /// `kept_string_indexed`.
    pub kept_namespace: usize,
    /// Always 0 today. A computed-key access `_r(id)[expr]` is the same
    /// code arm as `kept_namespace` (both fail the literal-property-shape
    /// match and flag the whole module as escaped), so this pass has no
    /// separate signal to attribute to "string-indexed" specifically.
    /// Kept as a named field for the same schema-parity reason as
    /// `kept_cross_chunk`.
    pub kept_string_indexed: usize,
    /// Kept because no `_r(id).key` / `_r(id)["key"]` / `require(id).key`
    /// read was ever recorded for this exact key
    /// (`usage.get(key) == None`). The dominant real-world source is
    /// barrel re-export glue's
    /// `Object.keys(alias).forEach(k => _m{id}.exports[k] = alias[k])`
    /// loop (see `collect_object_keys_reexport_mappings`): its forwarded
    /// keys use a *computed* `[k]` write, so `collect_direct_export_assignments`
    /// (literal keys only) never records that write, and a downstream
    /// consumer's read of the forwarded key is never attributed back to
    /// the original producer's own literal assignment either.
    pub kept_barrel_glue: usize,
    /// Every other keep reason: non-identifier RHS (`ComplexRhs`), a
    /// block-scoped producer binding (`BlockScopedBinding`), an observed
    /// write through the accessor (`WriteObserved`), or the
    /// structurally-unreachable defensive fallback (`NoLiveReadSpans`) —
    /// see [`ExportKeepReason`]. None of these map to one of the issue's
    /// five named keep-ladder rungs.
    pub kept_other: usize,
    /// #2161: export assignments whose non-identifier RHS was rewritten to
    /// a synthetic `var __jx_<m>_<key> = <RHS>;` binding ahead of the
    /// assignment, making the assignment itself identifier-RHS and
    /// therefore eligible for the same-chunk elision rungs above. Not part
    /// of the `kept` sum: a normalized key may still end up elided
    /// (counted in `elided_keys`) or kept for an unrelated reason (counted
    /// in one of the `kept_*` buckets above) — this field only reports how
    /// many keys were normalized on the way there, regardless of outcome.
    pub rhs_normalized: usize,
    /// #2161: candidate export assignments with a non-identifier RHS that
    /// [`is_pure_normalizable_export_rhs`] declined to normalize — the RHS
    /// shape falls outside the v1 purity ladder (a member chain, a call
    /// expression, or any other shape not provably side-effect-free to
    /// hoist). These are exactly the assignments that also surface under
    /// `ExportKeepReason::ComplexRhs` (`kept_other`) once elision itself
    /// runs, so this counter doesn't add a new dump tag of its own — see
    /// [`normalize_pure_export_rhs_unvalidated`].
    pub rhs_skipped_impure: usize,
}
// </HANDWRITE>

impl ExportElisionStats {
    /// Bump the counter bucket matching `reason`. See the matching
    /// `kept_*` field doc comments above (and [`ExportKeepReason`]'s own
    /// doc comment) for why `RegistryResidueRead`/`NamespaceEscape` each
    /// absorb one of the issue's five assumed rungs that turned out not
    /// to be independently observable at this pass.
    fn record_keep(&mut self, reason: ExportKeepReason) {
        match reason {
            ExportKeepReason::RegistryResidueRead => self.kept_registry += 1,
            ExportKeepReason::NamespaceEscape => self.kept_namespace += 1,
            ExportKeepReason::NoRecordedReads => self.kept_barrel_glue += 1,
            ExportKeepReason::ComplexRhs
            | ExportKeepReason::BlockScopedBinding
            | ExportKeepReason::WriteObserved
            | ExportKeepReason::NoLiveReadSpans => self.kept_other += 1,
        }
    }
}

/// Why one candidate `_m{id}.exports["key"] = local;` assignment was kept
/// (left on the indirection) instead of elided — tags the exact `continue`
/// arm in [`elide_same_chunk_export_bindings_unvalidated`]'s loop that
/// produced the decision (#2139).
///
/// Verified against the real code rather than assumed: the issue's
/// starting belief was five independent keep reasons (registry-resident
/// module, cross-chunk reference, namespace consumption, string-indexed
/// access, surviving barrel glue re-export). Two of those five turned out
/// to be structurally indistinguishable, at this pass, from two others —
/// see the `kept_cross_chunk` / `kept_string_indexed` field doc comments
/// on [`ExportElisionStats`] for exactly why — leaving the seven arms
/// below as what the code actually branches on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExportKeepReason {
    /// RHS isn't a bare identifier — `is_js_identifier` rejected it
    /// (function/object/conditional/chained expression).
    ComplexRhs,
    /// The module has a bare/namespace/computed-key escape anywhere
    /// (`escaped_ids.contains(id)`).
    NamespaceEscape,
    /// The producer's own local binding is `function`/`class`/`let`/`const`
    /// scoped to its own flattened-module block.
    BlockScopedBinding,
    /// No recorded read at all for this exact key
    /// (`usage.get(key) == None`).
    NoRecordedReads,
    /// At least one write (`_r(id).key = ...` / `require(id).key = ...`)
    /// was observed.
    WriteObserved,
    /// At least one `require(id).key` read was observed.
    RegistryResidueRead,
    /// Read spans recorded but empty, with no write and no registry-side
    /// read either — provably unreachable given
    /// `collect_export_key_usage`'s invariant that every map entry sets
    /// `written` or pushes a span before returning; kept only as an
    /// honest defensive bucket rather than folded silently into another
    /// reason.
    NoLiveReadSpans,
}

impl ExportKeepReason {
    /// Per-key `JET_ELISION_DEBUG` dump tag. Four of the seven arms fold
    /// into the coarse `"other"` counter bucket on `ExportElisionStats`
    /// but stay individually distinguishable in the dump.
    fn dump_tag(self) -> &'static str {
        match self {
            ExportKeepReason::RegistryResidueRead => "registry",
            ExportKeepReason::NamespaceEscape => "namespace",
            ExportKeepReason::NoRecordedReads => "barrel_glue",
            ExportKeepReason::ComplexRhs => "other:complex_rhs",
            ExportKeepReason::BlockScopedBinding => "other:block_scoped",
            ExportKeepReason::WriteObserved => "other:write_observed",
            ExportKeepReason::NoLiveReadSpans => "other:no_live_read_spans",
        }
    }
}

/// Approximate count of places `key`'s literal property-key text appears
/// in the scanned region: the assignment site itself, plus every recorded
/// read (`_r`/`require`), plus one more if any write was observed.
/// `ExportKeyUsage::written` is a bool, not a counter, so a key written
/// more than once still only adds 1 here — an approximation appropriate
/// for the `JET_ELISION_DEBUG` dump's "estimated bytes" purpose, not an
/// exact occurrence count.
fn export_key_occurrence_estimate(usage: Option<&ExportKeyUsage>) -> usize {
    1 + usage
        .map(|reads| reads.spans.len() + usize::from(reads.written))
        .unwrap_or(0)
}

/// Record one kept export-key decision: bump `stats`'s matching counter
/// bucket, and — only when `JET_ELISION_DEBUG` is set (`debug_enabled`) —
/// append a per-key row to `debug_rows` for the caller to dump.
fn mark_export_key_kept(
    stats: &mut ExportElisionStats,
    debug_rows: &mut Vec<String>,
    debug_enabled: bool,
    module_id: usize,
    key: &str,
    reason: ExportKeepReason,
    usage: Option<&ExportKeyUsage>,
) {
    stats.record_keep(reason);
    if debug_enabled {
        debug_rows.push(format!(
            "module={module_id} key={key} reason={} key_len={} occurrences={}",
            reason.dump_tag(),
            key.len(),
            export_key_occurrence_estimate(usage),
        ));
    }
}

/// Per-call counters for [`normalize_pure_export_rhs_unvalidated`] (#2161),
/// merged into the caller's `ExportElisionStats::rhs_normalized` /
/// `ExportElisionStats::rhs_skipped_impure` once the combined pipeline's
/// output is chosen (see `convert_and_elide_flat_region`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct RhsNormalizationStats {
    normalized: usize,
    skipped_impure: usize,
}

// <HANDWRITE gap="missing-generator:logic" tracker="pending-tracker" reason="logic section in scope_hoist_opt.rs is hand-written pending codegen support">
/// The v1 purity ladder for hoisting a non-identifier export RHS into a
/// synthetic `var` binding (#2161): arrow functions, function expressions,
/// and the same bare-literal shapes [`is_inlineable_literal_export_expr`]
/// already treats as side-effect-free elsewhere in this file. Member
/// chains and call expressions are deliberately excluded from v1 — a
/// member read can trigger a getter and a call can have arbitrary side
/// effects, so evaluating either any earlier than the original assignment
/// site (which is what hoisting into an unconditionally-evaluated `var`
/// initializer does) is not provably safe.
fn is_pure_normalizable_export_rhs(expr: &str) -> bool {
    is_inlineable_literal_export_expr(expr)
        || is_bare_function_expression(expr)
        || is_bare_arrow_function_expression(expr)
}
// </HANDWRITE>

// <HANDWRITE gap="missing-generator:logic" tracker="pending-tracker" reason="logic section in scope_hoist_opt.rs is hand-written pending codegen support">
/// `function` [name] `(` params `)` `{` body `}`, consuming `expr` in full.
/// `async`/generator forms fall outside the v1 ladder: an `async` prefix
/// means `expr` never starts with the literal text `"function"`, and a
/// generator's `*` (with or without a separating space before it) fails
/// either the post-`function` keyword-boundary check or the "next non-name
/// byte must be `(`" check below, so both are rejected without a dedicated
/// check for either.
fn is_bare_function_expression(expr: &str) -> bool {
    let b = expr.as_bytes();
    if !expr.starts_with("function") {
        return false;
    }
    match b.get("function".len()) {
        Some(c) if c.is_ascii_whitespace() || *c == b'(' => {}
        _ => return false,
    }
    let mut i = "function".len();
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    if i < b.len() && is_id_start_byte(b[i]) {
        i += 1;
        while i < b.len() && is_id_cont_byte(b[i]) {
            i += 1;
        }
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
    }
    if b.get(i) != Some(&b'(') {
        return false;
    }
    let Some(after_params) = skip_code_balanced(b, i, b'(', b')') else {
        return false;
    };
    let mut j = after_params;
    while j < b.len() && b[j].is_ascii_whitespace() {
        j += 1;
    }
    if b.get(j) != Some(&b'{') {
        return false;
    }
    skip_code_balanced(b, j, b'{', b'}') == Some(b.len())
}

// <HANDWRITE gap="missing-generator:logic" tracker="pending-tracker" reason="logic section in scope_hoist_opt.rs is hand-written pending codegen support">
/// `<params> => <body>`, consuming `expr` in full, where `<params>` is a
/// bare identifier or a parenthesized parameter list and `<body>` is a
/// block or a bare expression. `async` arrows are out of the v1 ladder
/// (#2161), mirroring the function-expression restriction above.
fn is_bare_arrow_function_expression(expr: &str) -> bool {
    if expr.starts_with("async")
        && matches!(expr.as_bytes().get(5), Some(c) if c.is_ascii_whitespace() || *c == b'(')
    {
        return false;
    }
    let b = expr.as_bytes();
    if b.is_empty() {
        return false;
    }
    let after_params = if b[0] == b'(' {
        let Some(end) = skip_code_balanced(b, 0, b'(', b')') else {
            return false;
        };
        end
    } else {
        let mut i = 0;
        while i < b.len() && is_id_cont_byte(b[i]) {
            i += 1;
        }
        if i == 0 || !is_js_identifier(&expr[..i]) {
            return false;
        }
        i
    };
    let mut i = after_params;
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    if !expr[i..].starts_with("=>") {
        return false;
    }
    i += 2;
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= b.len() {
        return false;
    }
    if b[i] == b'{' {
        skip_code_balanced(b, i, b'{', b'}') == Some(b.len())
    } else {
        // Expression body: the body's own *content* needs no further
        // purity check here (constructing the arrow function never
        // evaluates it), but a depth-0 comma would mean `expr` isn't a
        // single AssignmentExpression — seeing `<arrow>, <more>` here means
        // the *original* statement was a comma-operator sequence
        // expression (a legal RHS for a plain assignment statement), which
        // would silently become a second `var` declarator (a different
        // program, and likely a parse error) if hoisted verbatim into a
        // `var` initializer. Reject rather than risk it — the reparse
        // guard in `convert_and_elide_flat_region` would catch a mistake
        // here anyway, but this keeps the common case out of that fallback
        // path.
        !contains_top_level_comma(&expr[i..])
    }
}

// <HANDWRITE gap="missing-generator:logic" tracker="pending-tracker" reason="logic section in scope_hoist_opt.rs is hand-written pending codegen support">
/// Whether `s` contains a depth-0 (unparenthesized/unbracketed) comma.
/// Mirrors [`find_direct_export_assignment_semicolon`]'s depth-tracking
/// scan style; see [`is_bare_arrow_function_expression`]'s expression-body
/// case for why this matters.
fn contains_top_level_comma(s: &str) -> bool {
    let b = s.as_bytes();
    let mut i = 0;
    let (mut paren, mut bracket, mut brace) = (0usize, 0usize, 0usize);
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
            b',' if paren == 0 && bracket == 0 && brace == 0 => return true,
            _ => {}
        }
        i += 1;
    }
    false
}
// </HANDWRITE>

// <HANDWRITE gap="missing-generator:logic" tracker="pending-tracker" reason="logic section in scope_hoist_opt.rs is hand-written pending codegen support">
/// Rewrites `<exports_obj>.key = <RHS>;` to
/// `var __jx_<m>_<key> = <RHS>; <exports_obj>.key = __jx_<m>_<key>;` for
/// every flat-region export assignment whose RHS is a non-identifier but
/// provably pure shape (#2161, see [`is_pure_normalizable_export_rhs`]).
///
/// Purely a textual rewrite over the same
/// [`collect_direct_export_assignments`] shape
/// [`elide_same_chunk_export_bindings_unvalidated`] itself scans for: the
/// synthesized binding is `var`-hoisted (never block-scoped, so
/// `collect_block_scoped_declaration_names` — which only tracks
/// `function`/`class`/`let`/`const` — never flags it), which means every
/// one of elision's existing identifier-RHS rungs (block-scope, namespace,
/// registry, ...) applies to it exactly as it would to any other
/// identifier-RHS export assignment. This function does not duplicate any
/// of elision's keep/elide decision logic — it only creates the identifier
/// for that logic to run against. A normalized key that elision still
/// keeps (for an unrelated reason, e.g. a namespace escape) is expected:
/// it surfaces under the matching `kept_*` bucket exactly as it would have
/// pre-normalization, just via a one-hop-further indirection.
///
/// The always-dot-form `_m{module_id}.exports.{key}` used for the
/// rewritten assignment is safe regardless of whether the original source
/// used bracket or dot notation: `collect_direct_export_assignments`'s
/// regex restricts `key` to `[A-Za-z_$][A-Za-z0-9_$]*`, which is always a
/// syntactically valid property name in dot form too (property accessors
/// aren't subject to reserved-word restrictions).
fn normalize_pure_export_rhs_unvalidated(code: &str) -> (String, RhsNormalizationStats) {
    let assignments = collect_direct_export_assignments(code);
    let mut stats = RhsNormalizationStats::default();
    if assignments.is_empty() {
        return (code.to_string(), stats);
    }

    let mut replacements: Vec<(usize, usize, String)> = Vec::new();
    for ((module_id, key), assignment) in &assignments {
        if is_js_identifier(&assignment.expr) {
            continue; // already elision-eligible; nothing to normalize.
        }
        if !is_pure_normalizable_export_rhs(&assignment.expr) {
            stats.skipped_impure += 1;
            continue;
        }
        let synthetic = format!("__jx_{module_id}_{key}");
        let replacement = format!(
            "var {synthetic} = {}; _m{module_id}.exports.{key} = {synthetic};",
            assignment.expr,
        );
        replacements.push((assignment.span.0, assignment.span.1, replacement));
        stats.normalized += 1;
    }

    if replacements.is_empty() {
        return (code.to_string(), RhsNormalizationStats::default());
    }
    let rewritten = apply_static_replacements(code, replacements);
    if rewritten == code {
        // Mirrors `elide_same_chunk_export_bindings_unvalidated`'s own
        // bail-to-original-on-overlap-surprise handling: treat it
        // identically to "nothing to do" rather than reporting stats for
        // an effect that didn't actually land.
        return (code.to_string(), RhsNormalizationStats::default());
    }
    (rewritten, stats)
}
// </HANDWRITE>

/// Drop the exports-object property-key indirection for same-chunk,
/// statically-consumed named exports in the flattened region (#1993).
///
/// `_m{id}.exports["key"] = local;` survives flattening purely so
/// `_r(id).key` reads have a live object to read from. When every read of
/// `key` is a flat-region `_r(id).key` / `_r(id)["key"]` access — never a
/// bare/namespace read of the whole module, never a `require(id)` read
/// from registry-residue code (a *different* lexical scope that can't see
/// flat-region locals), never a write — the indirection is provably
/// redundant: the producer already has a plain local binding holding the
/// same value, so every consumer read can point straight at that binding:
///
/// ```js
/// // before
/// _m13.exports["getAlertUtilityClass"] = _m13_getAlertUtilityClass;
/// var _m4_getAlertUtilityClass = _r(13)["getAlertUtilityClass"];
/// // after
/// var _m4_getAlertUtilityClass = _m13_getAlertUtilityClass;
/// ```
///
/// That, in turn, lets the mangler compress the binding like any other
/// local — the property *key* on the exports object is what survives
/// minification unmangled today (`u.getAlertUtilityClass=e`-style output).
///
/// Conservative ladder (mirrors #1993's "any doubt → keep" discipline):
/// - Only `_m{id}.exports["key"] = IDENT;` / `_m{id}.exports.key = IDENT;`
///   assignments qualify (via `collect_direct_export_assignments`, already
///   statement-boundary-anchored with duplicate-assignment exclusion);
///   function/object/conditional/chained RHS forms are left alone.
/// - A module is ineligible entirely (every key kept) if any bare/
///   namespace/computed-key `_r(id)`/`require(id)` occurrence exists
///   anywhere, EXCEPT the fallback operand of the
///   `_r(id)["default"] || _r(id)` / `_r(id).default || _r(id)`
///   default-interop idiom — the dominant real-world shape for modules
///   with both a default and named export — which is masked out first so
///   it doesn't falsely flag the module as namespace-consumed (see
///   `bare_require_module_ids_excluding_default_fallback`).
/// - Per key: every recorded read must come from `_r(` (flat region); a
///   single `require(id).key` read or a single write (`_r(id).key = ...`)
///   keeps that key's indirection.
/// - `_m{id}.exports[...]`/`_r(id)` text only exists for modules the
///   entry-flatten partition already proved safe to flatten (registry-
///   resident and cross-chunk-referenced modules never emit this shape),
///   so those two "keep" triggers are satisfied by construction.
/// - When every key of a module is elided and nothing still references
///   `_m{id}e`/`_m{id}`, `remove_orphan_module_alias_and_slot` drops the
///   now-dead exports-object scaffolding too.
///
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
///
/// Core scan-and-rewrite logic for [`elide_same_chunk_export_bindings`],
/// shared with the combined [`convert_and_elide_flat_region`] pipeline
/// (#2133): builds and applies every replacement (primary elision +
/// orphan alias/slot cleanup) but performs no reparse-validation of its
/// own, so a caller that already knows it needs to validate a larger
/// combined region can defer that single region-wide parse instead of
/// paying one here too. [`elide_same_chunk_export_bindings`] wraps this
/// with its own validation (and its own "Reparse-guarded" contract) for
/// standalone callers.
fn elide_same_chunk_export_bindings_unvalidated(code: &str) -> (String, ExportElisionStats) {
    let assignments = collect_direct_export_assignments(code);
    if assignments.is_empty() {
        return (code.to_string(), ExportElisionStats::default());
    }

    let escaped_ids = bare_require_module_ids_excluding_default_fallback(code);
    let usage = collect_export_key_usage(code);
    let block_scoped_names = collect_block_scoped_declaration_names(code);

    let mut replacements: Vec<(usize, usize, String)> = Vec::new();
    let mut touched_modules: HashSet<usize> = HashSet::new();
    let mut elided_keys = 0usize;
    // #2139: per-reason keep attribution. Always accumulated into `stats`
    // (the loop already visits every assignment, so this costs nothing
    // extra); `debug_rows` is only populated when JET_ELISION_DEBUG is set.
    let mut stats = ExportElisionStats::default();
    let debug_dump_path = std::env::var_os("JET_ELISION_DEBUG");
    let debug_enabled = debug_dump_path.is_some();
    let mut debug_rows: Vec<String> = Vec::new();

    for (key, assignment) in &assignments {
        let (id, prop) = key;
        // Only "= identifier" RHS forms are eligible; the producer's own
        // local binding is reused verbatim as the consumer-facing name.
        // Function/object/conditional/chained RHS forms are left alone.
        if !is_js_identifier(&assignment.expr) {
            mark_export_key_kept(
                &mut stats,
                &mut debug_rows,
                debug_enabled,
                *id,
                prop,
                ExportKeepReason::ComplexRhs,
                usage.get(key),
            );
            continue;
        }
        if escaped_ids.contains(id) {
            mark_export_key_kept(
                &mut stats,
                &mut debug_rows,
                debug_enabled,
                *id,
                prop,
                ExportKeepReason::NamespaceEscape,
                usage.get(key),
            );
            continue;
        }
        // Each flattened module lives in its own `{ ... }` block (the
        // `// Module N: path` banner convention). Under ES-module
        // strict-mode semantics a `function`/`class`/`let`/`const`
        // binding is scoped to that block and invisible to sibling
        // modules, while `var` (and the exports-object property that
        // lives on it) is hoisted past block boundaries. Rewriting a
        // cross-module consumer straight to a block-scoped name would
        // trade a correct property read for a ReferenceError, so any
        // binding that isn't provably `var`-hoisted stays on the
        // indirection.
        if block_scoped_names.contains(&assignment.expr) {
            mark_export_key_kept(
                &mut stats,
                &mut debug_rows,
                debug_enabled,
                *id,
                prop,
                ExportKeepReason::BlockScopedBinding,
                usage.get(key),
            );
            continue;
        }
        let Some(reads) = usage.get(key) else {
            mark_export_key_kept(
                &mut stats,
                &mut debug_rows,
                debug_enabled,
                *id,
                prop,
                ExportKeepReason::NoRecordedReads,
                None,
            );
            continue;
        };
        // Decomposed from the original `reads.written || reads.require_side
        // || reads.spans.is_empty()` short-circuit OR — same order, same
        // first-true-wins result; #2139 needs each disjunct individually
        // attributed, which the combined boolean couldn't provide.
        if reads.written {
            mark_export_key_kept(
                &mut stats,
                &mut debug_rows,
                debug_enabled,
                *id,
                prop,
                ExportKeepReason::WriteObserved,
                Some(reads),
            );
            continue;
        }
        if reads.require_side {
            mark_export_key_kept(
                &mut stats,
                &mut debug_rows,
                debug_enabled,
                *id,
                prop,
                ExportKeepReason::RegistryResidueRead,
                Some(reads),
            );
            continue;
        }
        if reads.spans.is_empty() {
            mark_export_key_kept(
                &mut stats,
                &mut debug_rows,
                debug_enabled,
                *id,
                prop,
                ExportKeepReason::NoLiveReadSpans,
                Some(reads),
            );
            continue;
        }
        replacements.push((assignment.span.0, assignment.span.1, String::new()));
        for &(start, end) in &reads.spans {
            replacements.push((start, end, assignment.expr.clone()));
        }
        touched_modules.insert(*id);
        elided_keys += 1;
    }

    if replacements.is_empty() {
        return (code.to_string(), ExportElisionStats::default());
    }

    let mut rewritten = apply_static_replacements(code, replacements);
    if rewritten == code {
        // `apply_static_replacements` bails to the original text on any
        // span-overlap surprise; treat that identically to "nothing to do".
        return (code.to_string(), ExportElisionStats::default());
    }

    // Orphan alias/slot cleanup used to call `remove_orphan_module_alias_and_slot`
    // once per touched module, each call re-scanning the *entire* region from
    // scratch (word-boundary ref counts, a freshly compiled `_r(id)` regex, and
    // an `_mods` list re-parse). On the mui-visual-demo reference corpus that's
    // ~94 touched modules x a ~1.4MB region, which dominated build time
    // (O(touched_modules x region), #2133). Build the same lookups ONCE off
    // `rewritten` and turn every module's decision into O(1) index lookups.
    let orphan_index = build_orphan_cleanup_index(&rewritten);
    let orphan_replacements = collect_orphan_cleanup_replacements(&orphan_index, &touched_modules);
    if !orphan_replacements.is_empty() {
        rewritten = apply_static_replacements(&rewritten, orphan_replacements);
    }

    stats.modules = touched_modules.len();
    stats.elided_keys = elided_keys;
    stats.kept = assignments.len() - elided_keys;
    debug_assert_eq!(
        stats.kept,
        stats.kept_registry
            + stats.kept_cross_chunk
            + stats.kept_namespace
            + stats.kept_string_indexed
            + stats.kept_barrel_glue
            + stats.kept_other,
        "export-elision kept-reason counters must sum to kept (#2139): {stats:?}",
    );
    // JET_ELISION_DEBUG=<file> dumps one row per kept key. Mirrors the
    // JET_TREESHAKE_DEBUG convention (env-gated, best-effort, sorted,
    // overwritten each call) — a multi-chunk build where this pass runs
    // more than once retains only the last call's rows.
    if let Some(dump) = &debug_dump_path {
        debug_rows.sort();
        let _ = std::fs::write(dump, debug_rows.join("\n"));
    }
    (rewritten, stats)
}

/// Reparse-guarded: on any parse failure after rewriting, the original
/// code is returned unchanged with zeroed stats. See
/// [`elide_same_chunk_export_bindings_unvalidated`] for the scan/rewrite
/// algorithm this wraps.
pub fn elide_same_chunk_export_bindings(code: &str) -> (String, ExportElisionStats) {
    let (rewritten, stats) = elide_same_chunk_export_bindings_unvalidated(code);
    if rewritten == code || !super::dce::js_parses_without_errors(&rewritten) {
        return (code.to_string(), ExportElisionStats::default());
    }
    (rewritten, stats)
}

/// Names declared via `function`, `class`, `let`, or `const` anywhere in
/// `code`. Each flattened module lives in its own `{ ... }` block; a
/// binding declared with one of these keywords is scoped to that block
/// under ES-module strict-mode semantics and must not be treated as safe
/// for a cross-module direct reference (see `elide_same_chunk_export_bindings`).
/// Deliberately over-inclusive (e.g. it also picks up `extends`/generic
/// clause tokens near a `class` declaration) — extra names only cost a
/// missed optimization, never an incorrect rewrite.
fn collect_block_scoped_declaration_names(code: &str) -> HashSet<String> {
    let b = code.as_bytes();
    let len = b.len();
    let mut i = 0usize;
    let mut prev = b'\n';
    let mut names: HashSet<String> = HashSet::new();

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

        if !is_id_cont_byte(prev) {
            let matched_kw_len = ["function", "class", "let", "const"]
                .iter()
                .find(|kw| b[i..].starts_with(kw.as_bytes()))
                .map(|kw| kw.len());
            if let Some(kw_len) = matched_kw_len {
                let after_kw = i + kw_len;
                let boundary_ok = after_kw >= len || !is_id_cont_byte(b[after_kw]);
                if boundary_ok {
                    i = collect_declarator_names_from(b, after_kw, len, &mut names).max(after_kw);
                    prev = b';';
                    continue;
                }
            }
        }

        if !matches!(b[i], b' ' | b'\t' | b'\r' | b'\n') {
            prev = b[i];
        }
        i += 1;
    }

    names
}

/// From just after a `function`/`class`/`let`/`const` keyword, collects
/// every declarator identifier into `names` — including each name in a
/// `let a = 1, b = 2;`-style multi-declarator list — and returns the byte
/// offset the outer scan should resume from.
fn collect_declarator_names_from(
    b: &[u8],
    start: usize,
    len: usize,
    names: &mut HashSet<String>,
) -> usize {
    let mut j = start;
    loop {
        while j < len && matches!(b[j], b' ' | b'\t' | b'\r' | b'\n') {
            j += 1;
        }
        if j < len && b[j] == b'*' {
            // generator `function* name(...)`
            j += 1;
            continue;
        }
        if j >= len || !is_id_start_byte(b[j]) {
            return j;
        }
        let name_start = j;
        while j < len && is_id_cont_byte(b[j]) {
            j += 1;
        }
        if let Ok(name) = std::str::from_utf8(&b[name_start..j]) {
            names.insert(name.to_string());
        }
        while j < len && matches!(b[j], b' ' | b'\t' | b'\r' | b'\n') {
            j += 1;
        }
        if j >= len {
            return j;
        }
        match b[j] {
            // Function param list / class or function body / class
            // `extends` clause / TS type annotation: no further
            // declarator names follow at the top level.
            b'(' | b'{' | b':' | b'<' => return j,
            b';' => return j + 1,
            b',' => {
                j += 1;
            }
            b'=' => {
                j = skip_declarator_initializer(b, j + 1, len);
                if j < len && b[j] == b',' {
                    j += 1;
                } else {
                    return j;
                }
            }
            _ => return j,
        }
    }
}

/// Best-effort skip from just after a declarator's `=` to the next
/// top-level (depth-0) `,` or `;`, tracking `()`/`[]`/`{}` nesting and
/// skipping strings/templates/comments/regex literals. Only used to find
/// subsequent names in a `let`/`const` multi-declarator list; imprecision
/// here only changes which names get conservatively treated as
/// block-scoped, never whether an unsafe rewrite happens.
fn skip_declarator_initializer(b: &[u8], start: usize, len: usize) -> usize {
    let mut i = start;
    let mut depth: i32 = 0;
    let mut prev = b'=';
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
        match b[i] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                if depth == 0 {
                    return i;
                }
                depth -= 1;
            }
            b',' | b';' if depth == 0 => return i,
            _ => {}
        }
        if !matches!(b[i], b' ' | b'\t' | b'\r' | b'\n') {
            prev = b[i];
        }
        i += 1;
    }
    i
}

/// Per-`(module_id, key)` usage summary produced by
/// `collect_export_key_usage`.
#[derive(Debug, Default)]
struct ExportKeyUsage {
    /// Byte spans of `_r(id).key` / `_r(id)["key"]` reads — safe to
    /// rewrite to the producer's local binding.
    spans: Vec<(usize, usize)>,
    /// At least one read went through `require(id).key`: registry-residue
    /// code in a different lexical scope than the flat IIFE, which can't
    /// see the producer's local binding. Forces "keep".
    require_side: bool,
    /// At least one occurrence was a write (`_r(id).key = ...` /
    /// `require(id).key = ...`) rather than a read. Forces "keep" — a
    /// consumer mutating another module's exports object is visible to
    /// every other reader of that object; a local `var` write would not
    /// be.
    written: bool,
}

/// Scan for `_r(id).key` / `_r(id)["key"]` / `require(id).key` /
/// `require(id)["key"]` occurrences and group them by `(id, key)`.
///
/// Deliberately does not trace `var alias = _r(id); alias.key` aliasing:
/// jet's own ESM→CJS transform never emits that shape for the dominant
/// named-import case (`NamedImports` compiles directly to
/// `var LOCAL = requireTarget["imported"];`), so skipping it costs no
/// real-world elisions while keeping this scan a single linear pass.
fn collect_export_key_usage(code: &str) -> HashMap<(usize, String), ExportKeyUsage> {
    let b = code.as_bytes();
    let len = b.len();
    let mut i = 0usize;
    let mut prev = b'(';
    let mut usage: HashMap<(usize, String), ExportKeyUsage> = HashMap::new();

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
                        if let Some((prop, end_ref, is_assignment)) =
                            match_any_property_access_after_base(b, after_require, true)
                        {
                            let entry = usage.entry((id, prop)).or_default();
                            if is_assignment {
                                entry.written = true;
                            } else {
                                entry.spans.push((ident_start, end_ref));
                                if ident == "require" {
                                    entry.require_side = true;
                                }
                            }
                            i = end_ref;
                            prev = b'a';
                            continue;
                        }
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

    usage
}

/// `collect_bare_require_module_ids`, but first masks out the fallback
/// operand of every `_r(id)["default"] || _r(id)` / `_r(id).default ||
/// _r(id)` default-interop idiom so it isn't misread as a namespace/bare
/// escape of `id`. Real-world mixed default+named imports
/// (`var x = _r(id)["default"] || _r(id); var y = _r(id)["namedKey"];`)
/// are the dominant shape blocking this pass without the exemption.
fn bare_require_module_ids_excluding_default_fallback(code: &str) -> HashSet<usize> {
    let spans = collect_default_fallback_spans(code);
    if spans.is_empty() {
        return collect_bare_require_module_ids(code);
    }
    let mut masked = code.as_bytes().to_vec();
    for (start, end) in spans {
        // Every masked byte was already confirmed ASCII (part of a matched
        // `_r(`/`require(` call), so overwriting in place with an ASCII
        // digit keeps the buffer valid UTF-8 and byte-length-identical,
        // and a run of `0`s can never re-parse as a require call.
        for byte in &mut masked[start..end] {
            *byte = b'0';
        }
    }
    let masked_code = String::from_utf8(masked).unwrap_or_else(|_| code.to_string());
    collect_bare_require_module_ids(&masked_code)
}

/// Find every default-interop-fallback idiom in `code` and return the
/// byte span of just the trailing fallback operand (the second
/// `_r(id)`/`require(id)`), for
/// `bare_require_module_ids_excluding_default_fallback` to mask.
fn collect_default_fallback_spans(code: &str) -> Vec<(usize, usize)> {
    let b = code.as_bytes();
    let len = b.len();
    let mut i = 0usize;
    let mut prev = b'(';
    let mut spans = Vec::new();

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
            if let Some((fallback_span, end)) = match_default_fallback_operand_span(b, i) {
                spans.push(fallback_span);
                i = end;
                prev = b')';
                continue;
            }
            if let Some((_, after_require)) = match_require_call_any(b, i) {
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

    spans
}

/// Like `match_default_fallback_require`, but recognizes both `_r(id)`
/// and `require(id)` call forms (via `match_require_call_any`) and
/// returns the fallback operand's own span rather than just the end of
/// the whole expression.
fn match_default_fallback_operand_span(b: &[u8], i: usize) -> Option<((usize, usize), usize)> {
    let (id, mut j) = match_require_call_any(b, i)?;
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
    let fallback_start = j;
    let (rhs_id, end) = match_require_call_any(b, fallback_start)?;
    (id == rhs_id).then_some(((fallback_start, end), end))
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
    fn test_eliminate_unused_exports_keeps_entry_exports() {
        let code = r#"var _m0={exports:{}};
var _m0_TrendingUp = wrapIcon(_m1_MuiTrendingUp);
_m0.exports["TrendingUp"] = _m0_TrendingUp;"#;

        let result = eliminate_unused_exports_preserving_entry(code, 0);

        assert!(
            result.contains("_m0.exports[\"TrendingUp\"] = _m0_TrendingUp"),
            "the public entry export must survive without an internal read, got: {result}"
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

    // ──────────────────────────────────────────────────────────────────
    // R7: Same-chunk export-binding elision (#2128)
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_elide_same_chunk_export_bindings_eligible_simple_case() {
        let code = concat!(
            "var _m1={exports:{}};",
            "var _m1_x=1;",
            "_m1.exports[\"x\"]=_m1_x;",
            "var _m2_y=_r(1)[\"x\"];",
        );
        let (out, stats) = elide_same_chunk_export_bindings(code);
        assert_eq!(stats.modules, 1);
        assert_eq!(stats.elided_keys, 1);
        assert_eq!(stats.kept, 0);
        assert!(!out.contains("_m1.exports"));
        assert!(out.contains("_m2_y=_m1_x;"));
    }

    #[test]
    fn test_elide_same_chunk_export_bindings_namespace_consumer_keeps_all_keys() {
        let code = concat!(
            "var _m1={exports:{}};",
            "var _m1_x=1;",
            "var _m1_y=2;",
            "_m1.exports[\"x\"]=_m1_x;",
            "_m1.exports[\"y\"]=_m1_y;",
            "var _m2_a=_r(1)[\"x\"];",
            "var _m2_ns=_r(1);",
        );
        let (out, stats) = elide_same_chunk_export_bindings(code);
        assert_eq!(stats.elided_keys, 0);
        assert_eq!(out, code);
    }

    #[test]
    fn test_elide_same_chunk_export_bindings_string_index_keeps() {
        let code = concat!(
            "var _m1={exports:{}};",
            "var _m1_x=1;",
            "_m1.exports[\"x\"]=_m1_x;",
            "var key='x';",
            "var _m2_y=_r(1)[key];",
        );
        let (out, stats) = elide_same_chunk_export_bindings(code);
        assert_eq!(stats.elided_keys, 0);
        assert_eq!(out, code);
    }

    #[test]
    fn test_elide_same_chunk_export_bindings_registry_residue_shape_has_no_candidates() {
        // Registry-residue (cross-chunk / cyclic / eval-unsafe) modules
        // never emit the `_m{id}.exports[...]` shape this pass keys off —
        // they keep the generic CJS `module`/`exports` factory parameter
        // names instead, by construction of the entry-flatten partition
        // (#1993). This documents "cross-chunk keeps" as structurally
        // guaranteed rather than independently re-checked by this pass.
        let code = concat!(
            "__jet__.define(1, function(require, module, exports) {",
            "var x = 1; module.exports[\"x\"] = x;",
            "});",
            "var _m2_y = require(1)[\"x\"];",
        );
        let (out, stats) = elide_same_chunk_export_bindings(code);
        assert_eq!(stats.elided_keys, 0);
        assert_eq!(out, code);
    }

    #[test]
    fn test_elide_same_chunk_export_bindings_chained_conditional_assignment_keeps() {
        let code = concat!(
            "var _m1={exports:{}};",
            "_m1.exports[\"x\"]=condition?a:b;",
            "var _m2_y=_r(1)[\"x\"];",
        );
        let (out, stats) = elide_same_chunk_export_bindings(code);
        assert_eq!(stats.elided_keys, 0);
        assert_eq!(out, code);
    }

    #[test]
    fn test_elide_same_chunk_export_bindings_require_side_read_keeps() {
        let code = concat!(
            "var _m1={exports:{}};",
            "var _m1_x=1;",
            "_m1.exports[\"x\"]=_m1_x;",
            "var _m2_y=require(1)[\"x\"];",
        );
        let (out, stats) = elide_same_chunk_export_bindings(code);
        assert_eq!(stats.elided_keys, 0);
        assert_eq!(out, code);
    }

    #[test]
    fn test_elide_same_chunk_export_bindings_write_through_require_keeps() {
        let code = concat!(
            "var _m1={exports:{}};",
            "var _m1_x=1;",
            "_m1.exports[\"x\"]=_m1_x;",
            "var _m2_y=_r(1)[\"x\"];",
            "_r(1)[\"x\"]=2;",
        );
        let (out, stats) = elide_same_chunk_export_bindings(code);
        assert_eq!(stats.elided_keys, 0);
        assert_eq!(out, code);
    }

    #[test]
    fn test_elide_same_chunk_export_bindings_drops_scaffolding_when_all_keys_elided_and_unreferenced(
    ) {
        let code = concat!(
            "var _m1={exports:{}};",
            "var _m1_x=1;",
            "_m1.exports[\"x\"]=_m1_x;",
            "var _m2_y=_r(1)[\"x\"];",
        );
        let (out, stats) = elide_same_chunk_export_bindings(code);
        assert_eq!(stats.modules, 1);
        assert_eq!(stats.elided_keys, 1);
        assert!(!out.contains("_m1.exports"));
        assert!(!out.contains("_m1={exports:{}}"));
        assert!(out.contains("_m2_y=_m1_x;"));
    }

    #[test]
    fn test_elide_same_chunk_export_bindings_cleans_up_object_shaped_mods_map() {
        // Regression for a genuine `ReferenceError: _m1 is not defined` this
        // pass caused on the entry-flatten path (generate_entry_flat_region,
        // #1993): that path's preamble uses a sparse OBJECT literal
        // (`var _mods={0:_m0,1:_m1};`), not generate_flattened_bundle's
        // dense ARRAY literal (`var _mods=[_m0,_m1];`).
        // remove_orphan_module_alias_and_slot's reference-count guard
        // treats the _mods entry as one of the (at most 2) legitimate
        // remaining references to the slot before removing
        // `var _m1={exports:{}};` — so the _mods entry must be neutralized
        // in lockstep, or slot removal orphans a dangling `_m1` read inside
        // the still-present object literal (found via #2132's var-hoisted
        // function-declaration conversion, which is the first pass able to
        // make a function-declared export like this one elision-eligible).
        let code = concat!(
            "var _m0={exports:{}};",
            "var _m1={exports:{}};",
            "var _mods={0:_m0,1:_m1};",
            "function _r(id){var m=_mods[id];return m?m.exports:__jet__.require(id)}",
            "var _m1_x=1;",
            "_m1.exports[\"x\"]=_m1_x;",
            "var _m0_y=_r(1)[\"x\"];",
        );
        let (out, stats) = elide_same_chunk_export_bindings(code);
        assert_eq!(stats.elided_keys, 1);
        assert!(!out.contains("_m1.exports"));
        assert!(
            !out.contains("_m1={exports:{}}"),
            "slot decl should be removed, got: {out}"
        );
        assert!(
            !out.contains(":_m1"),
            "the _mods object entry for module 1 must be neutralized \
             (zeroed), not left dangling, got: {out}"
        );
        assert!(
            out.contains("1:0"),
            "the neutralized _mods entry should read 1:0, got: {out}"
        );
        assert!(out.contains("_m0_y=_m1_x;"));
    }

    #[test]
    fn test_elide_same_chunk_export_bindings_cleans_up_array_shaped_mods_map() {
        // Same regression coverage as the object-shape test above, but for
        // generate_flattened_bundle's dense ARRAY literal shape
        // (`var _mods=[_m0,_m1];`) — locks in the pre-existing, correct
        // behavior alongside the object-shape fix so a future change can't
        // quietly regress one shape while fixing the other.
        let code = concat!(
            "var _m0={exports:{}};",
            "var _m1={exports:{}};",
            "var _mods=[_m0,_m1];",
            "function _r(id){var m=_mods[id];return m?m.exports:__jet__.require(id)}",
            "var _m1_x=1;",
            "_m1.exports[\"x\"]=_m1_x;",
            "var _m0_y=_r(1)[\"x\"];",
        );
        let (out, stats) = elide_same_chunk_export_bindings(code);
        assert_eq!(stats.elided_keys, 1);
        assert!(!out.contains("_m1.exports"));
        assert!(
            !out.contains("_m1={exports:{}}"),
            "slot decl should be removed, got: {out}"
        );
        assert!(
            out.contains("_mods=[_m0,0]"),
            "the _mods array entry for module 1 must be neutralized (zeroed), got: {out}"
        );
        assert!(out.contains("_m0_y=_m1_x;"));
    }

    #[test]
    fn test_elide_same_chunk_export_bindings_default_interop_mixed_import() {
        // Ground-truth shape from MUI's Alert.js -> alertClasses.js
        // (module 13): a mixed default+named import. The bare fallback
        // operand of the `_r(13)["default"] || _r(13)` idiom must not
        // block elision of the *named* key `default`; `getAlertUtilityClass`
        // is a `function` declaration (block-scoped — see
        // `test_elide_same_chunk_export_bindings_cross_block_function_declaration_keeps`)
        // so it correctly stays on the indirection even though it has a
        // clean dedicated read.
        let code = concat!(
            "var _m13={exports:{}};",
            "var _m13_alertClasses=makeClasses();",
            "function _m13_getAlertUtilityClass(slot){return slot;}",
            "_m13.exports[\"getAlertUtilityClass\"]=_m13_getAlertUtilityClass;",
            "_m13.exports[\"default\"]=_m13_alertClasses;",
            "var _m4_alertClasses=_r(13)[\"default\"]||_r(13);",
            "var _m4_getAlertUtilityClass=_r(13)[\"getAlertUtilityClass\"];",
        );
        let (out, stats) = elide_same_chunk_export_bindings(code);
        assert_eq!(stats.modules, 1);
        assert_eq!(stats.elided_keys, 1);
        assert!(out.contains("_m13.exports[\"getAlertUtilityClass\"]=_m13_getAlertUtilityClass;"));
        assert!(!out.contains("_m13.exports[\"default\"]"));
        assert!(out.contains("_m4_getAlertUtilityClass=_r(13)[\"getAlertUtilityClass\"];"));
        assert!(out.contains("_m4_alertClasses=_m13_alertClasses||_r(13);"));
        // Scaffolding kept: `_m13.exports["getAlertUtilityClass"]` and the
        // fallback operand `_r(13)` are both still live.
        assert!(out.contains("_m13={exports:{}}"));
    }

    #[test]
    fn test_elide_same_chunk_export_bindings_cross_block_function_declaration_keeps() {
        // Regression for the real production_build_regression break this
        // pass caused against the mui-visual corpus: `@mui/utils`'s
        // deepmerge.js (module 245, `export default function deepmerge`)
        // is re-exported unchanged by its barrel index.js (module 244,
        // `export { default } from './deepmerge'`). Each flattened module
        // lives in its own `{ ... }` block; `function _m245_deepmerge`
        // is block-scoped there under ES-module strict-mode semantics,
        // so a same-name reference from module 244's block — even one
        // reached only indirectly, by rewriting a *usage* of module 245's
        // "default" key that happens to sit inside module 244's own
        // export-assignment RHS — is a ReferenceError at runtime, not a
        // syntax error, so only a declaration-kind check (not
        // `js_parses_without_errors`) can catch it. Confirmed live via
        // JET_MINIFY_STAGE_DUMP: pre-fix, jet emitted
        // `N.default=_m245_deepmerge` with no declaration for
        // `_m245_deepmerge` left anywhere in the bundle.
        let code = concat!(
            "var _m245={exports:{}};",
            "function _m245_deepmerge(a,b){return _m245_deepmerge(a,b);}",
            "_m245.exports[\"default\"]=_m245_deepmerge;",
            "var _m244={exports:{}};",
            "_m244.exports[\"default\"]=_r(245)[\"default\"];",
            "var _m9_x=_r(244)[\"default\"];",
        );
        let (out, stats) = elide_same_chunk_export_bindings(code);
        assert_eq!(stats.elided_keys, 0);
        assert_eq!(out, code);
    }

    #[test]
    fn test_elide_same_chunk_export_bindings_hatch_disabled_matches_input() {
        // JET_NO_EXPORT_ELISION is wired at the `mod.rs` call sites, not
        // inside this pure function — this test just pins that calling
        // the function directly on already-eligible input always
        // elides, so the hatch's effect (skipping the call entirely) is
        // observable as a real byte-diff in the A/B build smoke test.
        let code = concat!(
            "var _m1={exports:{}};",
            "var _m1_x=1;",
            "_m1.exports[\"x\"]=_m1_x;",
            "var _m2_y=_r(1)[\"x\"];",
        );
        let (out, stats) = elide_same_chunk_export_bindings(code);
        assert_ne!(out, code);
        assert_eq!(stats.elided_keys, 1);
    }

    #[test]
    fn test_elide_same_chunk_export_bindings_no_assignments_returns_unchanged() {
        let code = "var _m1={exports:{}};var _m1_x=1;console.log(_m1_x);";
        let (out, stats) = elide_same_chunk_export_bindings(code);
        assert_eq!(out, code);
        assert_eq!(stats, ExportElisionStats::default());
    }

    #[test]
    fn test_elide_same_chunk_export_bindings_stats_attribute_kept_reasons() {
        // #2139: one assignment per populated keep-reason bucket, plus one
        // elided key, verifying each `continue` arm lands in its matching
        // `kept_*` counter (not just the pre-existing aggregate `kept`).
        let code = concat!(
            // Module 1: "x" elides normally (unchanged baseline signal);
            // "y" has a non-identifier (ternary) RHS -> kept_other.
            "var _m1={exports:{}};",
            "var _m1_x=1;",
            "_m1.exports[\"x\"]=_m1_x;",
            "_m1.exports[\"y\"]=condition?a:b;",
            "var _m9_x=_r(1)[\"x\"];",
            // Module 2: a bare `_r(2)` namespace read escapes the whole
            // module -> kept_namespace.
            "var _m2={exports:{}};",
            "var _m2_a=1;",
            "_m2.exports[\"a\"]=_m2_a;",
            "var _m9_ns=_r(2);",
            // Module 3: no `_r(3)`/`require(3)` occurrence anywhere, so
            // this key has zero recorded reads -> kept_barrel_glue (the
            // same signature as barrel re-export glue's computed-key
            // forward, see `ExportElisionStats::kept_barrel_glue`'s doc).
            "var _m3={exports:{}};",
            "var _m3_b=1;",
            "_m3.exports[\"b\"]=_m3_b;",
            // Module 4: read via `require(4)["c"]` (registry-residue
            // consumer) -> kept_registry.
            "var _m4={exports:{}};",
            "var _m4_c=1;",
            "_m4.exports[\"c\"]=_m4_c;",
            "var _m9_c=require(4)[\"c\"];",
        );
        let (_out, stats) = elide_same_chunk_export_bindings(code);
        assert_eq!(
            stats.modules, 1,
            "only module 1 has an elided key: {stats:?}"
        );
        assert_eq!(stats.elided_keys, 1, "only module 1's x elides: {stats:?}");
        assert_eq!(
            stats.kept_other, 1,
            "module 1's y is a non-identifier RHS: {stats:?}"
        );
        assert_eq!(
            stats.kept_namespace, 1,
            "module 2 is namespace-escaped: {stats:?}"
        );
        assert_eq!(
            stats.kept_barrel_glue, 1,
            "module 3's key has no recorded read: {stats:?}"
        );
        assert_eq!(
            stats.kept_registry, 1,
            "module 4's key is require()-read: {stats:?}"
        );
        assert_eq!(
            stats.kept_cross_chunk, 0,
            "never populated by this pass: {stats:?}"
        );
        assert_eq!(
            stats.kept_string_indexed, 0,
            "never populated by this pass: {stats:?}"
        );
        assert_eq!(
            stats.kept, 4,
            "4 assignments kept across the 4 buckets above: {stats:?}"
        );
        assert_eq!(
            stats.kept,
            stats.kept_registry
                + stats.kept_cross_chunk
                + stats.kept_namespace
                + stats.kept_string_indexed
                + stats.kept_barrel_glue
                + stats.kept_other,
            "kept-reason counters must sum to kept: {stats:?}"
        );
    }
}
// </HANDWRITE>
// CODEGEN-END

// <HANDWRITE gap="standardize:projects-jet-src-bundler-scope-hoist-opt-rs-reexport-wrapper-collapse" tracker="standardize-gap-projects-jet-src-bundler-scope-hoist-opt-rs" reason="Existing hand-written re-export wrapper collapse lives outside generated block; generator primitive does not yet cover post-flattening wrapper redirection.">
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

/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
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
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
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
// </HANDWRITE>

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

/// Counters for `convert_flat_region_function_declarations_to_var`,
/// surfaced via `JET_BUNDLE_TIMING` as
/// `fn-decl-conversion: converted=N skipped_order=M skipped_shape=K`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FnDeclConversionStats {
    /// Top-level `function` declarations rewritten to `var`-hoisted
    /// anonymous function expressions.
    pub converted: usize,
    /// Candidates skipped because some occurrence of the prefixed name
    /// appears earlier in the flat region's top-to-bottom execution order
    /// than the declaration itself (the conservative textual-precede
    /// scan — see the function doc comment).
    pub skipped_order: usize,
    /// Candidates skipped for shape reasons: the prefixed name is
    /// declared more than once as a `function`, or is also declared via
    /// `var`/`let`/`const` elsewhere in the region (would shadow instead
    /// of hoist). Async/generator/expression-position declarations are
    /// excluded before candidacy and are not counted here.
    pub skipped_shape: usize,
}

/// Convert eligible flat-region top-level `function` declarations into
/// `var`-hoisted anonymous function expressions (#2132).
///
/// Each flattened module lives in its own `{ ... }` block (see
/// `generate_flattened_bundle` / `generate_entry_flat_region`). Under ES
/// strict-mode semantics a `function` declared inside that block is
/// block-scoped — invisible outside the block — while `var` hoists past
/// the block to the enclosing flat-region function scope. That is exactly
/// the split `elide_same_chunk_export_bindings` (#2128) already draws via
/// `collect_block_scoped_declaration_names`: only `var`-hoisted bindings
/// are eligible for direct cross-module reference, so function-*declared*
/// exports — the dominant shape for helper-style export families — never
/// qualified no matter how safe the actual data flow was. This pass runs
/// first so elision's existing eligibility check sees a plain `var`.
///
/// ```js
/// // before (block-scoped; invisible outside this module's block)
/// function _m3_getButtonUtilityClass(slot) { return 'Button-' + slot; }
/// // after (var-hoisted; now the shape elision already recognizes)
/// var _m3_getButtonUtilityClass=function(slot) { return 'Button-' + slot; };
/// ```
///
/// The replacement function expression is deliberately left **anonymous**.
/// Giving it back the original short name would leak the full source
/// identifier through `Function.prototype.name` — but today's baseline
/// (block-scoped `function NAME(){}`) already has its OWN declared name
/// compressed away by the mangler's final, scope-blind
/// `compress_generated_prefixed_names` pass (confirmed empirically: a
/// `function _m1_getButtonUtilityClass(){}` export compiles today to
/// `function e(){}`, i.e. `.name` is already an opaque short alias, never
/// the original readable identifier). Reusing the exact same `_m<n>_name`
/// token as the anonymous expression's `var` target keeps that same
/// blanket rename in sole control of the visible `.name` post-mangle
/// (ES2015 `NamedEvaluation` infers `.name` from the `var` target only
/// when the expression itself is nameless), matching today's observable
/// "opaque compressed name" behavior instead of regressing it to the full
/// readable name. A *named* function expression here would additionally
/// be misclassified as block-scoped by `collect_block_scoped_declaration_names`
/// (it matches on any identifier immediately following `function`,
/// declaration or expression alike), silently defeating the elision
/// unlock this pass exists for.
///
/// Four safety conditions gate a rewrite (conservative on any doubt, per
/// the file's existing #1993/#2128 discipline):
/// - **Top-level only**: only names carrying the flattener's `_m<n>_`
///   prefix qualify (inner/nested declarations never receive that
///   prefix), and the declaration must sit in statement position
///   (immediately after `;`, `{`, `}`, or start-of-text past only
///   whitespace) — which also naturally excludes `async function` (the
///   `async` keyword breaks the immediately-preceding-boundary check) and
///   expression-position uses (`= function NAME`, `return function NAME`).
/// - **No earlier-in-execution-order reference**: the flat region executes
///   top-to-bottom at load, so a `var`'s *assignment* — unlike a function
///   declaration's *hoist* — only takes effect once execution reaches it.
///   Conservatively scan the whole region for the earliest textual
///   occurrence of the prefixed name; if it precedes the declaration's own
///   start (whether that earlier text is eagerly executed or merely
///   defined inside another not-yet-called function — the scan can't tell
///   the difference, so it doesn't try), skip. Cross-module references can
///   never trigger this (an import edge already orders producer before
///   consumer in the acyclic flatten), so only intra-region hazards like a
///   same-module forward reference or module-internal mutual recursion can
///   trip it; the rule is applied uniformly and is sound for both — of a
///   mutually-recursive pair, the textually-first declaration converts and
///   the second (referenced by the first, before its own declaration)
///   stays a hoisted function declaration.
/// - **Plain function only**: generators and getters/setters/method
///   shorthand never match the `function\s+NAME\s*\(` shape in the first
///   place (no "function" keyword token at all, or no whitespace between
///   "function" and "*").
/// - **No top-level shadowing**: a prefixed name declared more than once
///   as a `function`, or also declared via `var`/`let`/`const` elsewhere
///   in the region, is left alone — converting one of several bindings
///   sharing a name risks changing which declaration wins.
///
/// `JET_NO_FN_DECL_CONVERSION=1` is a testing escape hatch, checked by
/// callers in `mod.rs` — this function itself always converts when called.
///
/// Core scan-and-rewrite logic for
/// [`convert_flat_region_function_declarations_to_var`], shared with the
/// combined [`convert_and_elide_flat_region`] pipeline (#2133): builds and
/// applies every replacement but performs no reparse-validation of its
/// own, so a caller that already knows it needs to validate a larger
/// combined region can defer that single region-wide parse instead of
/// paying one here too. [`convert_flat_region_function_declarations_to_var`]
/// wraps this with its own validation (and its own "Reparse-guarded"
/// contract) for standalone callers.
fn convert_flat_region_function_declarations_to_var_unvalidated(
    code: &str,
) -> (String, FnDeclConversionStats) {
    use std::sync::OnceLock;
    static FUNC: OnceLock<Regex> = OnceLock::new();
    static VAR_DECL: OnceLock<Regex> = OnceLock::new();
    let func = FUNC.get_or_init(|| Regex::new(r"function\s+(_m\d+_[a-zA-Z0-9_$]+)\s*\(").unwrap());
    let var_decl = VAR_DECL
        .get_or_init(|| Regex::new(r"(?:var|let|const)\s+(_m\d+_[a-zA-Z0-9_$]+)\b").unwrap());

    let b = code.as_bytes();

    let statement_position = |start: usize| -> bool {
        let mut p = start;
        while p > 0 && matches!(b[p - 1], b' ' | b'\t' | b'\r' | b'\n') {
            p -= 1;
        }
        p == 0 || matches!(b[p - 1], b';' | b'{' | b'}')
    };

    struct Candidate {
        name: String,
        start: usize,
        params_open: usize,
        body_close: usize,
    }

    let mut candidates: Vec<Candidate> = Vec::new();
    let mut name_decl_counts: HashMap<String, usize> = HashMap::new();

    for cap in func.captures_iter(code) {
        let whole = cap.get(0).unwrap();
        let name = cap.get(1).unwrap().as_str().to_string();
        let start = whole.start();
        if !statement_position(start) {
            continue;
        }
        // The regex consumes through the opening "(" of the param list.
        let params_open = whole.end() - 1;
        if params_open >= b.len() || b[params_open] != b'(' {
            continue;
        }
        let Some(params_close) = skip_code_balanced(b, params_open, b'(', b')') else {
            continue;
        };
        let mut q = params_close;
        while q < b.len() && matches!(b[q], b' ' | b'\t' | b'\r' | b'\n') {
            q += 1;
        }
        if q >= b.len() || b[q] != b'{' {
            continue;
        }
        let Some(body_close) = skip_code_balanced(b, q, b'{', b'}') else {
            continue;
        };
        *name_decl_counts.entry(name.clone()).or_insert(0) += 1;
        candidates.push(Candidate {
            name,
            start,
            params_open,
            body_close,
        });
    }

    let mut stats = FnDeclConversionStats::default();
    if candidates.is_empty() {
        return (code.to_string(), stats);
    }

    // Condition 4 (no top-level shadowing), var/let/const half: collected
    // globally, position-independent — a later `var` of the same name is
    // just as much a shadow risk as an earlier one.
    let mut var_declared: HashSet<String> = HashSet::new();
    for cap in var_decl.captures_iter(code) {
        var_declared.insert(cap[1].to_string());
    }

    // Condition 2 (no earlier-in-execution-order reference): earliest
    // textual occurrence per prefixed name, region-wide.
    let mut occurrences: Vec<(usize, String)> = Vec::new();
    collect_prefixed_ident_occurrences(b, &mut occurrences);
    let mut first_occurrence: HashMap<String, usize> = HashMap::new();
    for (pos, name) in occurrences {
        first_occurrence
            .entry(name)
            .and_modify(|p| *p = (*p).min(pos))
            .or_insert(pos);
    }

    let mut replacements: Vec<(usize, usize, String)> = Vec::new();
    for cand in &candidates {
        if *name_decl_counts.get(&cand.name).unwrap_or(&0) > 1 || var_declared.contains(&cand.name)
        {
            stats.skipped_shape += 1;
            continue;
        }
        // The declaration's own name token always sits at/after `start`,
        // so this can only match a genuinely earlier occurrence.
        if let Some(&first) = first_occurrence.get(&cand.name) {
            if first < cand.start {
                stats.skipped_order += 1;
                continue;
            }
        }
        let rest = &code[cand.params_open..cand.body_close];
        let replacement = format!("var {}=function{};", cand.name, rest);
        replacements.push((cand.start, cand.body_close, replacement));
        stats.converted += 1;
    }

    if replacements.is_empty() {
        // Every candidate was skipped; the skip counts are still
        // meaningful diagnostic output, unlike the true-bail paths below.
        return (code.to_string(), stats);
    }

    (apply_static_replacements(code, replacements), stats)
}

/// Reparse-guarded: on any parse failure after rewriting, the original
/// code is returned unchanged with zeroed stats. See
/// [`convert_flat_region_function_declarations_to_var_unvalidated`] for the
/// scan/rewrite algorithm this wraps.
pub fn convert_flat_region_function_declarations_to_var(
    code: &str,
) -> (String, FnDeclConversionStats) {
    let (rewritten, stats) = convert_flat_region_function_declarations_to_var_unvalidated(code);
    if stats.converted == 0 {
        // No replacement was ever built (no candidates, or every candidate
        // was skipped) — `rewritten` is `code` unchanged; `apply_static_replacements`
        // was never called, so unlike the failure path below, the skip
        // counters are real diagnostic output, not a validation reset.
        return (rewritten, stats);
    }
    if rewritten == code || !super::dce::js_parses_without_errors(&rewritten) {
        return (code.to_string(), FnDeclConversionStats::default());
    }
    (rewritten, stats)
}

// <HANDWRITE gap="missing-generator:logic" tracker="pending-tracker" reason="logic section in scope_hoist_opt.rs is hand-written pending codegen support">
// <HANDWRITE gap="missing-generator:logic" tracker="pending-tracker" reason="logic section in scope_hoist_opt.rs is hand-written pending codegen support">
/// Runs flat-region function-declaration→var-hoisting conversion (#2132)
/// immediately followed by same-chunk export-binding elision (#2128) as one
/// pipeline, sharing a single region-wide reparse-validation pass across
/// both instead of one per pass (#2133): on the mui-visual-demo reference
/// corpus, each pass's own from-scratch `js_parses_without_errors` reparse
/// of the ~1.4MB flat region was the single largest cost within that pass
/// (measured ~75% of `fn_decl_conversion` and ~41% of `export_elision` on a
/// reference-shaped synthetic region), yet the second reparse is redundant
/// work: elision's input is exactly conversion's already-validated output,
/// so the combined output only needs to be proven parseable once.
///
/// On the (unobserved-on-real-corpora) rare path where the combined result
/// fails to parse, a second reparse of just the post-conversion text
/// disambiguates which pass to keep, preserving each pass's existing
/// independent fall-back-to-original guarantee exactly.
///
/// Also runs pure-export-RHS normalization (#2161,
/// [`normalize_pure_export_rhs_unvalidated`]) between the two: a purely
/// textual pre-pass over conversion's output, so it costs nothing beyond
/// its own (cheap, regex-based) `collect_direct_export_assignments` scan —
/// no extra region-wide `js_parses_without_errors` reparse on the common
/// path, same "one shared reparse" contract as the conversion+elision pair
/// above.
pub fn convert_and_elide_flat_region(
    code: &str,
) -> (String, FnDeclConversionStats, ExportElisionStats) {
    let (after_conv, conv_stats) =
        convert_flat_region_function_declarations_to_var_unvalidated(code);
    // Mirrors `convert_flat_region_function_declarations_to_var`'s own
    // no-replacement tier: when conversion never built a replacement,
    // `after_conv` is `code` unchanged (always valid, no reparse needed
    // for it), and `conv_stats`'s skip counters (if any) are real
    // diagnostics to report as-is, not a validation reset.
    let conv_is_noop = conv_stats.converted == 0;

    // #2161: normalize pure non-identifier export RHS shapes (arrow
    // functions, function expressions, bare literals) into a synthetic
    // `var __jx_<m>_<key>` binding ahead of the export assignment, so the
    // identifier-only rungs in `elide_same_chunk_export_bindings_unvalidated`
    // can fire on them too. JET_NO_RHS_NORMALIZE=1 is a dedicated escape
    // hatch, independent of JET_NO_EXPORT_ELISION (which disables elision
    // itself, not just this feeding step) — lets an A/B comparison isolate
    // this specific rewrite from the rest of the (already-shipped)
    // conv+elision pipeline.
    let no_rhs_normalize = std::env::var_os("JET_NO_RHS_NORMALIZE").is_some();
    let (after_normalize, normalize_stats) = if no_rhs_normalize {
        (after_conv.clone(), RhsNormalizationStats::default())
    } else {
        normalize_pure_export_rhs_unvalidated(&after_conv)
    };

    let (after_elision, mut elision_stats) =
        elide_same_chunk_export_bindings_unvalidated(&after_normalize);

    if after_elision == code {
        return (code.to_string(), conv_stats, ExportElisionStats::default());
    }
    if super::dce::js_parses_without_errors(&after_elision) {
        elision_stats.rhs_normalized = normalize_stats.normalized;
        elision_stats.rhs_skipped_impure = normalize_stats.skipped_impure;
        return (after_elision, conv_stats, elision_stats);
    }

    // The combined result doesn't parse. Degrade one stage at a time: if
    // normalization changed anything, first retry with it dropped —
    // elision running directly on `after_conv` is exactly the pre-#2161
    // pipeline, so a normalization-induced parse failure never regresses
    // below that already-established-safe baseline.
    if after_normalize != after_conv {
        let (after_elision_no_norm, elision_stats_no_norm) =
            elide_same_chunk_export_bindings_unvalidated(&after_conv);
        if after_elision_no_norm != code
            && super::dce::js_parses_without_errors(&after_elision_no_norm)
        {
            return (after_elision_no_norm, conv_stats, elision_stats_no_norm);
        }
    }

    // If conversion made no edit, the fault is entirely elision's-and/or-
    // normalization's (both fed by `after_conv` == `code`, always valid);
    // report conversion's real stats and reset the rest — exactly what
    // calling the standalone, self-validating wrappers in sequence would
    // produce. Otherwise disambiguate with a second reparse of
    // conversion's output alone — a cost only ever paid on this path,
    // never observed on real corpora.
    if conv_is_noop {
        return (code.to_string(), conv_stats, ExportElisionStats::default());
    }
    if after_conv != code && super::dce::js_parses_without_errors(&after_conv) {
        return (after_conv, conv_stats, ExportElisionStats::default());
    }
    (
        code.to_string(),
        FnDeclConversionStats::default(),
        ExportElisionStats::default(),
    )
}
// </HANDWRITE>
// </HANDWRITE>

#[cfg(test)]
mod fn_decl_conversion_tests {
    use super::*;

    #[test]
    fn converts_simple_top_level_function_declaration() {
        let code = "function _m3_getButtonUtilityClass(slot) { return 'Button-' + slot; }\n_m3.exports[\"getButtonUtilityClass\"] = _m3_getButtonUtilityClass;\n";
        let (out, stats) = convert_flat_region_function_declarations_to_var(code);
        assert_eq!(stats.converted, 1, "{out}");
        assert_eq!(stats.skipped_order, 0, "{out}");
        assert_eq!(stats.skipped_shape, 0, "{out}");
        assert!(
            out.contains(
                "var _m3_getButtonUtilityClass=function(slot) { return 'Button-' + slot; };"
            ),
            "{out}"
        );
        assert!(
            !out.contains("function _m3_getButtonUtilityClass("),
            "{out}"
        );
    }

    #[test]
    fn anonymous_function_expression_does_not_leak_original_name() {
        let code = "function _m5_computeAlertUtilityClassLongName() { return 1; }";
        let (out, stats) = convert_flat_region_function_declarations_to_var(code);
        assert_eq!(stats.converted, 1, "{out}");
        // No inner name: the mangler's blanket `_m<n>_name` compression
        // pass stays in sole control of the visible `.name`, matching
        // today's observable "opaque compressed name" behavior instead of
        // leaking the original readable identifier.
        assert!(out.contains("=function()"), "{out}");
        assert!(!out.contains("computeAlertUtilityClassLongName()"), "{out}");
    }

    #[test]
    fn no_candidates_is_a_no_op() {
        let code = "var _m1e=_m1.exports;\n_m1e.value=1;\n";
        let (out, stats) = convert_flat_region_function_declarations_to_var(code);
        assert_eq!(out, code);
        assert_eq!(stats, FnDeclConversionStats::default());
    }

    #[test]
    fn skips_when_a_reference_precedes_the_declaration() {
        // A local re-export alias reads the name before its own
        // declaration is reached in top-to-bottom execution order.
        let code = "var _m1_alias = _m1_bar;\nfunction _m1_bar() { return 1; }\n";
        let (out, stats) = convert_flat_region_function_declarations_to_var(code);
        assert_eq!(stats.converted, 0, "{out}");
        assert_eq!(stats.skipped_order, 1, "{out}");
        assert_eq!(out, code);
    }

    #[test]
    fn module_internal_mutual_recursion_converts_the_first_and_skips_the_second() {
        // A (declared first) calls B; B (declared second) calls A. B's
        // reference to A is textually after A (safe); A's reference to B
        // is textually before B (unsafe) -- the textual-precede rule
        // applies uniformly and correctly splits the pair.
        let code = "function _m2_isEven(n) { return n === 0 ? true : _m2_isOdd(n - 1); }\nfunction _m2_isOdd(n) { return n === 0 ? false : _m2_isEven(n - 1); }\n";
        let (out, stats) = convert_flat_region_function_declarations_to_var(code);
        assert_eq!(stats.converted, 1, "{out}");
        assert_eq!(stats.skipped_order, 1, "{out}");
        assert!(out.contains("var _m2_isEven=function(n)"), "{out}");
        assert!(out.contains("function _m2_isOdd(n)"), "{out}");
    }

    #[test]
    fn skips_async_and_generator_declarations() {
        let code = "async function _m4_fetchThing() { return 1; }\nfunction* _m4_genThing() { return 1; }\n";
        let (out, stats) = convert_flat_region_function_declarations_to_var(code);
        assert_eq!(stats.converted, 0, "{out}");
        assert_eq!(out, code);
    }

    #[test]
    fn skips_names_also_declared_via_var_elsewhere() {
        let code =
            "function _m6_helper() { return 1; }\nvar _m6_helper2 = _m6_helper;\nvar _m6_helper;\n";
        let (out, stats) = convert_flat_region_function_declarations_to_var(code);
        assert_eq!(stats.converted, 0, "{out}");
        assert_eq!(stats.skipped_shape, 1, "{out}");
        assert_eq!(out, code);
    }

    #[test]
    fn skips_duplicate_function_declarations_of_the_same_name() {
        let code = "function _m7_dup() { return 1; }\nfunction _m7_dup() { return 2; }\n";
        let (out, stats) = convert_flat_region_function_declarations_to_var(code);
        assert_eq!(stats.converted, 0, "{out}");
        assert_eq!(stats.skipped_shape, 2, "{out}");
        assert_eq!(out, code);
    }

    #[test]
    fn converts_multiple_independent_declarations_in_one_pass() {
        let code = "function _m8_a() { return 1; }\nfunction _m9_b() { return 2; }\n_m8.exports[\"a\"]=_m8_a;\n_m9.exports[\"b\"]=_m9_b;\n";
        let (out, stats) = convert_flat_region_function_declarations_to_var(code);
        assert_eq!(stats.converted, 2, "{out}");
        assert!(out.contains("var _m8_a=function()"), "{out}");
        assert!(out.contains("var _m9_b=function()"), "{out}");
    }
}

#[cfg(test)]
mod convert_and_elide_flat_region_tests {
    use super::*;

    /// Builds a reference-shaped synthetic flat region: `module_count`
    /// modules, each with one function-declared export, one plain-var
    /// export, and (past the first few ids) direct `_r(id).key` reads of
    /// two earlier modules' exports — plus a safe (no quotes/backslashes)
    /// ~1.3KB filler literal per module so the region's total size lands
    /// near the ~1.4MB mui-visual-demo reference corpus (#2133) instead of
    /// a tiny-helper toy size.
    fn build_reference_shaped_flat_region(module_count: usize) -> String {
        let filler = "M0,0 L4,8 L12,3 Z ".repeat(70);

        let mut out = String::with_capacity(module_count * 2200);
        out.push_str("(function(){\n'use strict';\n\n");
        for id in 0..module_count {
            out.push_str(&format!("var _m{id}={{exports:{{}}}};\n"));
        }
        out.push('\n');
        out.push_str("var _mods={");
        for id in 0..module_count {
            if id > 0 {
                out.push(',');
            }
            out.push_str(&format!("{id}:_m{id}"));
        }
        out.push_str("};\n");
        out.push_str("function _r(id){var m=_mods[id];return m?m.exports:__jet__.require(id)}\n\n");

        for id in 0..module_count {
            out.push_str(&format!("// Module {id}: src/mod{id}.js\n"));
            out.push_str("{\n");
            out.push_str(&format!("var _m{id}e=_m{id}.exports;\n"));
            out.push_str(&format!(
                "function _m{id}_helper(x) {{\n  var total = x;\n  for (var i = 0; i < 12; i++) {{\n    total = total + i * 2 - 1;\n  }}\n  var label = \"computed-\" + total + \"-suffix\";\n  return total + label.length;\n}}\n"
            ));
            out.push_str(&format!(
                "var _m{id}_config = {{ id: {id}, label: \"module-{id}\", enabled: true, path: \"{filler}\" }};\n"
            ));
            out.push_str(&format!("_m{id}.exports[\"helper\"] = _m{id}_helper;\n"));
            out.push_str(&format!("var _m{id}_value = {id} * 3 + 7;\n"));
            out.push_str(&format!("_m{id}.exports[\"value\"] = _m{id}_value;\n"));
            if id > 2 {
                let a = id - 1;
                let b = id / 2;
                out.push_str(&format!("var _m{id}_a = _r({a}).helper(_r({a}).value);\n"));
                out.push_str(&format!("var _m{id}_b = _r({b}).value + _m{id}_a;\n"));
            }
            out.push_str("}\n");
            out.push_str(&format!(
                "__jet__.cache[{id}]={{exports:_m{id}.exports,id:{id},loaded:true}};\n\n"
            ));
        }
        out.push_str("})();\n");
        out
    }

    #[test]
    fn combined_pipeline_matches_running_both_passes_sequentially_on_a_large_region() {
        let code = build_reference_shaped_flat_region(700);
        assert!(
            super::super::dce::js_parses_without_errors(&code),
            "synthetic fixture must itself be valid JS"
        );

        let (sequential_conv, sequential_conv_stats) =
            convert_flat_region_function_declarations_to_var(&code);
        let (sequential_out, sequential_elision_stats) =
            elide_same_chunk_export_bindings(&sequential_conv);

        let (combined_out, combined_conv_stats, combined_elision_stats) =
            convert_and_elide_flat_region(&code);

        assert_eq!(
            combined_out, sequential_out,
            "sharing one reparse across both passes must be byte-identical to running \
             convert_flat_region_function_declarations_to_var then \
             elide_same_chunk_export_bindings sequentially"
        );
        assert_eq!(combined_conv_stats, sequential_conv_stats);
        assert_eq!(combined_elision_stats, sequential_elision_stats);
        assert!(
            combined_conv_stats.converted > 0,
            "fixture should exercise the conversion pass"
        );
        assert!(
            combined_elision_stats.elided_keys > 0,
            "fixture should exercise the elision pass"
        );
    }

    #[test]
    fn combined_pipeline_preserves_conversion_skip_counters_when_both_passes_are_otherwise_no_ops()
    {
        // Regression guard for the #2133 refactor: when both passes are
        // no-ops overall (nothing rewritten), conversion's own
        // skipped_order/skipped_shape diagnostics must still surface
        // as-is rather than collapsing to a validation-failure reset —
        // exactly as the standalone `convert_flat_region_function_declarations_to_var`
        // wrapper already preserves them (`js_parses_without_errors` is
        // never even called in this tier, on either the sequential or the
        // combined path).
        let code = "var _m1_alias = _m1_bar;\nfunction _m1_bar() { return 1; }\n";
        let (out, conv_stats, elision_stats) = convert_and_elide_flat_region(code);
        assert_eq!(out, code);
        assert_eq!(conv_stats.converted, 0);
        assert_eq!(conv_stats.skipped_order, 1);
        assert_eq!(elision_stats, ExportElisionStats::default());
    }

    #[test]
    fn combined_pipeline_stays_well_under_a_generous_time_budget_on_a_large_region() {
        // Not a tight perf gate (CI hardware varies) — a coarse regression
        // guard against reintroducing an O(passes) full-region reparse or
        // similar quadratic-in-region-size cost on the flat-region passes
        // (#2133).
        let code = build_reference_shaped_flat_region(700);
        let start = std::time::Instant::now();
        let (_out, conv_stats, elision_stats) = convert_and_elide_flat_region(&code);
        let elapsed = start.elapsed();
        assert!(conv_stats.converted > 0);
        assert!(elision_stats.elided_keys > 0);
        assert!(
            elapsed.as_millis() < 2000,
            "combined convert+elide pass took {elapsed:?} on a {}-byte reference-shaped region; \
             expected comfortably under 2s even on slow/loaded CI hardware",
            code.len()
        );
    }
}

#[cfg(test)]
mod rhs_normalization_tests {
    use super::*;

    // ── is_pure_normalizable_export_rhs: the v1 purity ladder ──────────

    #[test]
    fn purity_ladder_accepts_block_bodied_arrow_function() {
        assert!(is_pure_normalizable_export_rhs("() => { return 1; }"));
    }

    #[test]
    fn purity_ladder_accepts_expression_bodied_arrow_function_with_paren_params() {
        assert!(is_pure_normalizable_export_rhs("(a, b) => a + b"));
    }

    #[test]
    fn purity_ladder_accepts_single_identifier_param_arrow_function() {
        assert!(is_pure_normalizable_export_rhs("x => x * 2"));
    }

    #[test]
    fn purity_ladder_accepts_arrow_function_with_default_parameter_containing_a_nested_arrow() {
        // The nested arrow lives entirely inside the (balanced) parameter
        // list; constructing the outer arrow is side-effect-free regardless
        // of what a parameter default's own expression looks like.
        assert!(is_pure_normalizable_export_rhs("(fn = () => {}) => fn()"));
    }

    #[test]
    fn purity_ladder_accepts_anonymous_function_expression() {
        assert!(is_pure_normalizable_export_rhs("function () { return 1; }"));
    }

    #[test]
    fn purity_ladder_accepts_named_function_expression() {
        assert!(is_pure_normalizable_export_rhs(
            "function Foo() { return 1; }"
        ));
    }

    #[test]
    fn purity_ladder_accepts_bare_literals() {
        assert!(is_pure_normalizable_export_rhs("\"hello\""));
        assert!(is_pure_normalizable_export_rhs("42"));
        assert!(is_pure_normalizable_export_rhs("true"));
        assert!(is_pure_normalizable_export_rhs("null"));
    }

// <HANDWRITE gap="missing-generator:unit-test" tracker="pending-tracker" reason="unit-test section in scope_hoist_opt.rs is hand-written pending codegen support">
    #[test]
    fn purity_ladder_rejects_member_chains() {
        // v1 ladder explicitly excludes member reads: a getter could fire.
        assert!(!is_pure_normalizable_export_rhs("a.b.c"));
        assert!(!is_pure_normalizable_export_rhs("_r(3).css"));
    }
// </HANDWRITE>

    #[test]
    fn purity_ladder_rejects_call_expressions() {
        // v1 ladder explicitly excludes calls: arbitrary side effects.
        assert!(!is_pure_normalizable_export_rhs("createSvgIcon(x, y)"));
        assert!(!is_pure_normalizable_export_rhs("(() => {})()"));
    }

// <HANDWRITE gap="missing-generator:unit-test" tracker="pending-tracker" reason="unit-test section in scope_hoist_opt.rs is hand-written pending codegen support">
    #[test]
    fn purity_ladder_rejects_async_and_generator_functions() {
        // Out of the v1 ladder -- see is_bare_function_expression /
        // is_bare_arrow_function_expression's doc comments for exactly
        // which check rejects each shape.
        assert!(!is_pure_normalizable_export_rhs("async () => {}"));
        assert!(!is_pure_normalizable_export_rhs("async x => x"));
        assert!(!is_pure_normalizable_export_rhs("async function () {}"));
        assert!(!is_pure_normalizable_export_rhs("function* () {}"));
        assert!(!is_pure_normalizable_export_rhs("function *named() {}"));
    }
// </HANDWRITE>

    #[test]
    fn purity_ladder_rejects_sequence_expression_disguised_as_an_arrow_body() {
        // `_m5.exports.f = (a) => a, sideEffect();` is a legal *assignment
        // statement* RHS today (comma operator: assign the arrow, then
        // separately evaluate `sideEffect()`). Hoisting the text verbatim
        // into `var __jx_5_f = (a) => a, sideEffect();` would silently
        // become a second (syntactically invalid) `var` declarator instead
        // -- `var` initializers parse as `AssignmentExpression`, not
        // `Expression`, so a top-level comma there means something
        // different. Must be rejected outright, not left to the reparse
        // guard to catch.
        assert!(!is_pure_normalizable_export_rhs("(a) => a, sideEffect()"));
        assert!(!is_pure_normalizable_export_rhs("x => x, sideEffect()"));
    }

    // ── normalize_pure_export_rhs_unvalidated: the textual rewrite ─────

// <HANDWRITE gap="missing-generator:unit-test" tracker="pending-tracker" reason="unit-test section in scope_hoist_opt.rs is hand-written pending codegen support">
    #[test]
    fn normalize_rewrites_arrow_function_export_to_synthetic_var() {
        let code = concat!("var _m1={exports:{}};", "_m1.exports[\"f\"]=()=>{};",);
        let (out, stats) = normalize_pure_export_rhs_unvalidated(code);
        assert_eq!(stats.normalized, 1);
        assert_eq!(stats.skipped_impure, 0);
        assert!(
            out.contains("var __jx_1_f = ()=>{};"),
            "synthetic declarator missing: {out}"
        );
        assert!(
            out.contains("_m1.exports.f = __jx_1_f;"),
            "export assignment should now be identifier-RHS: {out}"
        );
        assert!(super::super::dce::js_parses_without_errors(&out));
    }
// </HANDWRITE>

// <HANDWRITE gap="missing-generator:unit-test" tracker="pending-tracker" reason="unit-test section in scope_hoist_opt.rs is hand-written pending codegen support">
    #[test]
    fn normalize_counts_skipped_impure_candidates() {
        let code = concat!(
            "var _m1={exports:{}};",
            "_m1.exports[\"f\"]=()=>{};",
            "_m1.exports[\"g\"]=createSvgIcon(x,y);",
            "_m1.exports[\"h\"]=_r(2).css;",
        );
        let (_out, stats) = normalize_pure_export_rhs_unvalidated(code);
        assert_eq!(stats.normalized, 1, "only f is a pure shape: {stats:?}");
        assert_eq!(
            stats.skipped_impure, 2,
            "g (call) and h (member chain) are impure: {stats:?}"
        );
    }
// </HANDWRITE>

    #[test]
    fn normalize_leaves_identifier_rhs_untouched() {
        let code = concat!(
            "var _m1={exports:{}};",
            "var _m1_x=1;",
            "_m1.exports[\"x\"]=_m1_x;",
        );
        let (out, stats) = normalize_pure_export_rhs_unvalidated(code);
        assert_eq!(out, code);
        assert_eq!(stats, RhsNormalizationStats::default());
    }

    #[test]
    fn normalize_is_a_no_op_with_no_export_assignments() {
        let code = "var x = 1; function f() { return x; }";
        let (out, stats) = normalize_pure_export_rhs_unvalidated(code);
        assert_eq!(out, code);
        assert_eq!(stats, RhsNormalizationStats::default());
    }

    // ── convert_and_elide_flat_region: full-pipeline integration ───────

// <HANDWRITE gap="missing-generator:unit-test" tracker="pending-tracker" reason="unit-test section in scope_hoist_opt.rs is hand-written pending codegen support">
    #[test]
    fn combined_pipeline_normalizes_then_elides_an_arrow_function_export() {
        let code = concat!(
            "var _m1={exports:{}};",
            "_m1.exports[\"f\"]=()=>{};",
            "var _m2_y=_r(1)[\"f\"];",
        );
        let (out, _conv_stats, elision_stats) = convert_and_elide_flat_region(code);
        assert_eq!(elision_stats.rhs_normalized, 1, "{elision_stats:?}");
        assert_eq!(elision_stats.rhs_skipped_impure, 0, "{elision_stats:?}");
        assert_eq!(elision_stats.elided_keys, 1, "{elision_stats:?}");
        assert!(
            !out.contains("_m1.exports"),
            "property-key indirection should be gone: {out}"
        );
        assert!(
            out.contains("__jx_1_f"),
            "synthetic binding should carry the value through: {out}"
        );
        assert!(super::super::dce::js_parses_without_errors(&out));
    }
// </HANDWRITE>

// <HANDWRITE gap="missing-generator:unit-test" tracker="pending-tracker" reason="unit-test section in scope_hoist_opt.rs is hand-written pending codegen support">
    #[test]
    fn combined_pipeline_normalized_then_still_kept_key_is_fine() {
        // A normalizable RHS on a namespace-escaped module still gets
        // normalized (rhs_normalized counts it) but is correctly kept (not
        // elided) for the pre-existing namespace-escape reason -- exactly
        // the "normalized-then-still-kept key is fine" case named in
        // #2161. Module 3's "g" elides normally alongside it so
        // `replacements` is non-empty overall: mirrors the *pre-existing*
        // `test_elide_same_chunk_export_bindings_stats_attribute_kept_reasons`
        // fixture shape, because `elide_same_chunk_export_bindings_unvalidated`
        // resets *all* stats (including already-attributed keep reasons) to
        // `ExportElisionStats::default()` whenever nothing elides in a call
        // -- a pre-existing characteristic this pass doesn't touch, worked
        // around here exactly like that existing test already does.
        let code = concat!(
            "var _m1={exports:{}};",
            "_m1.exports[\"f\"]=()=>{};",
            "var _m2_ns=_r(1);",
            "var _m3={exports:{}};",
            "_m3.exports[\"g\"]=()=>{};",
            "var _m9_g=_r(3)[\"g\"];",
        );
        let (_out, _conv_stats, elision_stats) = convert_and_elide_flat_region(code);
        assert_eq!(elision_stats.rhs_normalized, 2, "{elision_stats:?}");
        assert_eq!(
            elision_stats.elided_keys, 1,
            "only module 3's g elides: {elision_stats:?}"
        );
        assert_eq!(
            elision_stats.kept_namespace, 1,
            "module 1's f is namespace-escaped: {elision_stats:?}"
        );
    }
// </HANDWRITE>

    #[test]
    fn jet_no_rhs_normalize_hatch_disables_normalization_only() {
        let code = concat!(
            "var _m1={exports:{}};",
            "_m1.exports[\"f\"]=()=>{};",
            "var _m2_y=_r(1)[\"f\"];",
        );

        // SAFETY: test-only env mutation, set immediately before use and
        // removed immediately after (mirrors the established
        // JET_NO_PASS_GATES/JET_NO_STMT_DCE pattern elsewhere in this
        // crate, e.g. `bundler::mod` tests) -- no other test reads or
        // writes JET_NO_RHS_NORMALIZE, so there's no cross-test
        // interference despite the default multi-threaded test runner.
        std::env::set_var("JET_NO_RHS_NORMALIZE", "1");
        let (out_disabled, _conv_stats, elision_stats_disabled) =
            convert_and_elide_flat_region(code);
        std::env::remove_var("JET_NO_RHS_NORMALIZE");

        let (out_enabled, _conv_stats2, elision_stats_enabled) =
            convert_and_elide_flat_region(code);

        // Hatch on: normalization never runs, so "f"'s arrow-function RHS
        // stays a non-identifier -- elision's pre-#2161 keep-on-ComplexRhs
        // behavior applies unchanged, and with nothing else to do the
        // whole pipeline is a byte-identical no-op.
        assert_eq!(out_disabled, code, "hatch must fully disable normalization");
        assert_eq!(
            elision_stats_disabled,
            ExportElisionStats::default(),
            "{elision_stats_disabled:?}"
        );

        // Hatch off (default): the same fixture normalizes and then elides.
        assert_eq!(
            elision_stats_enabled.rhs_normalized, 1,
            "{elision_stats_enabled:?}"
        );
        assert_eq!(
            elision_stats_enabled.elided_keys, 1,
            "{elision_stats_enabled:?}"
        );
        assert_ne!(
            out_enabled, code,
            "without the hatch, the fixture should change"
        );
    }

    // ── #2161 Step 0: the `.name` / NamedEvaluation decision ───────────

    /// Documents the decided `.name` behavior via the shape of the rewrite
    /// itself, rather than re-deriving ECMAScript NamedEvaluation semantics
    /// in a Rust assertion (this crate has no embedded JS engine to
    /// actually execute the output and read `.name` back). Verified
    /// empirically with Node ahead of writing this pass (ECMA-262 13.15.2,
    /// NamedEvaluation): a MemberExpression-LHS assignment
    /// (`<exports_obj>.key = () => {}`) NEVER triggers NamedEvaluation --
    /// `IsIdentifierRef(LeftHandSideExpression)` is false for a member
    /// expression, so `.name` is `""` there today, for every ComplexRhs
    /// anonymous-function/arrow-function export RHS, both before and after
    /// this change. This pass instead binds the value to a `var`
    /// *declarator* (`var __jx_<m>_<key> = <RHS>`), which DOES trigger
    /// NamedEvaluation (`VariableDeclarator` is one of the spec's named
    /// forms) -- taking `.name` from `""` to the (pre-mangle) synthetic
    /// name, and after mangling to whatever short name the mangler assigns
    /// it. Decision: proceed with plain hoisting: `""` -> non-empty is a
    /// neutral-to-positive change (never a regression), so no
    /// NamedEvaluation-preserving mitigation is needed.
    #[test]
    fn normalized_binding_moves_the_function_from_a_member_expression_assignment_to_a_var_declarator_gaining_a_name(
    ) {
        let code = concat!("var _m1={exports:{}};", "_m1.exports[\"f\"]=()=>{};",);
        let (out, stats) = normalize_pure_export_rhs_unvalidated(code);
        assert_eq!(stats.normalized, 1);
        // Before: `_m1.exports["f"]=()=>{}` -- a MemberExpression-LHS
        // assignment, never a NamedEvaluation site, so `.name === ""`.
        assert!(!code.contains("var __jx_1_f"));
        // After: `var __jx_1_f = ()=>{}` -- a VariableDeclarator, the
        // NamedEvaluation-eligible shape that gives the function a real
        // (pre-mangle) `.name === "__jx_1_f"`.
        assert!(
            out.contains("var __jx_1_f = ()=>{};"),
            "expected a VariableDeclarator NamedEvaluation site: {out}"
        );
    }

    // ── Mangler interaction ─────────────────────────────────────────────

    #[test]
    fn synthetic_binding_survives_and_gets_renamed_by_the_scope_based_mangler() {
        // Explicit requirement from #2161: the synthetic `var __jx_<m>_<key>`
        // must not be special-cased away from mangling -- it's just an
        // ordinary `var` declared in the flattened root IIFE scope, so it
        // goes through the same `compute_renames`/`apply_renames` path as
        // every other flattened-module local (see mangle.rs; unlike the
        // `_m<digits>_<suffix>` catch-all `compress_generated_prefixed_names`
        // exists for, which only matters for *block-scoped* leftovers that
        // slip past the primary scope-based pass -- #2132's `_mN_f` vars
        // are the precedent this mirrors).
        let code = concat!(
            "(function(){\n",
            "var _m1={exports:{}};",
            "_m1.exports[\"f\"]=()=>{};",
            "var _m2_y=_r(1)[\"f\"];",
            "console.log(_m2_y);",
            "})();\n",
        );
        let (normalized, stats) = normalize_pure_export_rhs_unvalidated(code);
        assert_eq!(stats.normalized, 1);
        assert!(normalized.contains("__jx_1_f"));

        let mangled = super::super::mangle::mangle_variables_with_root(&normalized);
        assert!(
            !mangled.contains("__jx_1_f"),
            "mangler should have renamed the synthetic binding to a short name: {mangled}"
        );
    }

    // ── Counters: ExportElisionStats::rhs_normalized / rhs_skipped_impure ──

    #[test]
    fn export_elision_stats_rhs_counters_default_to_zero() {
        let stats = ExportElisionStats::default();
        assert_eq!(stats.rhs_normalized, 0);
        assert_eq!(stats.rhs_skipped_impure, 0);
    }

    #[test]
    fn export_elision_stats_rhs_counters_are_independent_of_kept_and_elided_totals() {
        // rhs_normalized/rhs_skipped_impure are not summands of `kept` or
        // `elided_keys` -- they report how many candidates were fed into
        // normalization, regardless of the outcome elision assigns them
        // afterward. This mixed fixture (one normalized+elided, one
        // normalized+kept, one impure+kept) exercises all three states.
        let code = concat!(
            "var _m1={exports:{}};",
            "_m1.exports[\"f\"]=()=>{};",
            "var _m2_ns=_r(1);",
            "var _m3={exports:{}};",
            "_m3.exports[\"g\"]=()=>{};",
            "var _m9_g=_r(3)[\"g\"];",
            "var _m4={exports:{}};",
            "_m4.exports[\"h\"]=createSvgIcon(x,y);",
            "var _m9_h=_r(4)[\"h\"];",
        );
        let (_out, _conv_stats, elision_stats) = convert_and_elide_flat_region(code);
        assert_eq!(
            elision_stats.rhs_normalized, 2,
            "f and g are pure shapes: {elision_stats:?}"
        );
        assert_eq!(
            elision_stats.rhs_skipped_impure, 1,
            "h is a call expression: {elision_stats:?}"
        );
        assert_eq!(elision_stats.elided_keys, 1, "only g: {elision_stats:?}");
    }
}
// </HANDWRITE>
