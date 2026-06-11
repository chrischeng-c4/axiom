// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
//! Variable mangling: scope analysis + variable renaming.
//!
//! Conservative approach: only renames function-local variables and parameters.
//! Preserves globals, property names, and module-level declarations.

use std::collections::{HashMap, HashSet};

/// Mangle local variable names to short identifiers.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn mangle_variables(source: &str) -> String {
    mangle_variables_inner(source, false)
}

/// Mangle variables, optionally treating root scope as a function scope.
///
/// When `mangle_root_scope` is true, scope 0 declarations are mangled
/// just like any other function scope. This is correct when the source
/// is wrapped in an IIFE (e.g., scope-hoisted bundles).
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn mangle_variables_with_root(source: &str) -> String {
    mangle_variables_inner(source, true)
}

/// Apply explicit renames while respecting the lightweight scope model used by
/// the variable mangler.
///
/// `root_renames` only applies to bindings declared in the root scope and to
/// references that resolve to those bindings. `global_renames` applies only to
/// unresolved free identifiers. Property accesses and object-literal keys are
/// preserved.
pub(crate) fn apply_scoped_module_renames(
    source: &str,
    root_renames: &HashMap<String, String>,
    global_renames: &HashMap<String, String>,
) -> String {
    let tokens = tokenize(source);
    if tokens.is_empty() || (root_renames.is_empty() && global_renames.is_empty()) {
        return source.to_string();
    }

    let si = build_scopes(source, &tokens);
    let mut repls: Vec<(usize, usize, String)> = Vec::new();

    for (ti, tok) in tokens.iter().enumerate() {
        if tok.kind != TK::Ident || should_skip_identifier_rename(source, &tokens, ti) {
            continue;
        }

        let name = txt(source, tok);
        if let Some(bound_scope) = resolve_decl_scope(name, si.token_scope[ti], &si.scopes) {
            if bound_scope == 0 {
                if let Some(new_name) = root_renames.get(name) {
                    repls.push((
                        tok.start,
                        tok.end,
                        rename_replacement(source, &tokens, ti, name, new_name),
                    ));
                }
            }
        } else if let Some(new_name) = global_renames.get(name) {
            repls.push((
                tok.start,
                tok.end,
                rename_replacement(source, &tokens, ti, name, new_name),
            ));
        }
    }

    apply_replacements(source, repls)
}

fn mangle_variables_inner(source: &str, mangle_root: bool) -> String {
    let timing = std::env::var_os("JET_MANGLE_TIMING").is_some();
    let mut last = std::time::Instant::now();
    let mut lap = |stage: &str| {
        if timing {
            eprintln!("[mangle-timing] {stage}: {:?}", last.elapsed());
            last = std::time::Instant::now();
        }
    };
    let tokens = tokenize(source);
    if tokens.is_empty() {
        return source.to_string();
    }
    lap("tokenize");
    let si = build_scopes(source, &tokens);
    lap("build_scopes");
    let renames = compute_renames(source, &tokens, &si, mangle_root);
    lap("compute_renames");
    let mangled = apply_renames(source, &tokens, &si.token_scope, &si.scopes, &renames);
    lap("apply_renames");
    let mangled = repair_generated_module_slot_references(&mangled);
    lap("repair_slot_refs");
    let mangled = repair_generated_module_slot_local_decl_collisions(&mangled);
    lap("repair_slot_collisions");
    let mangled = repair_generated_require_helper_references(&mangled);
    let mangled = repair_retained_wrapper_cjs_param_references(&mangled);
    let mangled = repair_react_dom_event_helper_aliases(&mangled);
    let mangled = repair_react_symbol_constant_references(&mangled);
    let mangled = repair_react_dom_client_stale_references(&mangled);
    let mangled = repair_react_event_transition_alias_shadow(&mangled);
    lap("repairs_rest");
    let out = compress_generated_prefixed_names(&mangled);
    lap("prefixed_names");
    out
}

/// Compress leftover `_m<N>_name` identifiers to short names.
///
/// The flattener's `_m<N>_` prefix makes these names bundle-unique by
/// construction, so any occurrence is the same binding regardless of
/// scope. compute_renames only renames declarations its scope model
/// attributes to function scopes; block-scoped `function`/`const`/`let`
/// declarations inside the generated module blocks slipped through —
/// 1,927 long identifiers (~35KB) survived in the MUI corpus bundle.
/// Rename them all here with a single token-level pass.
fn compress_generated_prefixed_names(source: &str) -> String {
    let tokens = tokenize(source);
    let mut counts: HashMap<&str, usize> = HashMap::new();
    let mut used_names: HashSet<&str> = HashSet::new();
    for (ti, tok) in tokens.iter().enumerate() {
        if tok.kind != TK::Ident {
            continue;
        }
        let name = txt(source, tok);
        used_names.insert(name);
        if generated_prefixed_name_suffix(name).is_none() {
            continue;
        }
        if should_skip_identifier_rename(source, &tokens, ti) {
            continue;
        }
        *counts.entry(name).or_insert(0) += 1;
    }
    if counts.is_empty() {
        return source.to_string();
    }

    // Most-referenced names get the shortest aliases.
    let mut names: Vec<(&str, usize)> = counts.into_iter().collect();
    names.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    let mut renames: HashMap<&str, String> = HashMap::new();
    let mut assigned: HashSet<String> = HashSet::new();
    let mut counter = 0usize;
    for (name, _) in names {
        let short = loop {
            let cand = gen_name(counter);
            counter += 1;
            if !is_reserved(&cand) && !used_names.contains(cand.as_str()) && !assigned.contains(&cand)
            {
                break cand;
            }
        };
        if short.len() < name.len() {
            assigned.insert(short.clone());
            renames.insert(name, short);
        }
    }
    if renames.is_empty() {
        return source.to_string();
    }

    let mut repls: Vec<(usize, usize, String)> = Vec::new();
    for (ti, tok) in tokens.iter().enumerate() {
        if tok.kind != TK::Ident {
            continue;
        }
        let name = txt(source, tok);
        let Some(short) = renames.get(name) else {
            continue;
        };
        if should_skip_identifier_rename(source, &tokens, ti) {
            continue;
        }
        repls.push((
            tok.start,
            tok.end,
            rename_replacement(source, &tokens, ti, name, short),
        ));
    }
    let out = apply_replacements(source, repls);
    if crate::bundler::dce::js_parses_without_errors(&out) {
        out
    } else {
        source.to_string()
    }
}

/// `_m<digits>_<suffix>` — the flattener's bundle-unique namespace.
fn generated_prefixed_name_suffix(name: &str) -> Option<&str> {
    let rest = name.strip_prefix("_m")?;
    let digits_end = rest.find(|c: char| !c.is_ascii_digit())?;
    if digits_end == 0 {
        return None;
    }
    let after = &rest[digits_end..];
    after.strip_prefix('_').filter(|s| !s.is_empty())
}

fn repair_generated_module_slot_references(source: &str) -> String {
    let tokens = tokenize(source);
    if tokens.is_empty() {
        return source.to_string();
    }

    let slot_aliases = collect_generated_module_slot_aliases(source, &tokens);
    if slot_aliases.is_empty()
        || slot_aliases
            .iter()
            .all(|(idx, alias)| alias.as_str() == format!("_m{idx}"))
    {
        return source.to_string();
    }

    let si = build_scopes(source, &tokens);
    let mut repls: Vec<(usize, usize, String)> = Vec::new();
    for (ti, tok) in tokens.iter().enumerate() {
        if tok.kind != TK::Ident || should_skip_identifier_rename(source, &tokens, ti) {
            continue;
        }
        let name = txt(source, tok);
        let Some(idx) = generated_module_slot_index(name) else {
            continue;
        };
        let Some(new_name) = slot_aliases.get(&idx) else {
            continue;
        };
        if new_name == name {
            continue;
        }
        if resolve_decl_scope(name, si.token_scope[ti], &si.scopes).is_some() {
            continue;
        }
        repls.push((tok.start, tok.end, new_name.clone()));
    }

    if repls.is_empty() {
        source.to_string()
    } else {
        apply_replacements(source, repls)
    }
}

fn collect_generated_module_slot_aliases(source: &str, tokens: &[Tok]) -> HashMap<usize, String> {
    let mut aliases = HashMap::new();
    let mut i = 0usize;
    while i < tokens.len() {
        let Some((next, first_alias)) = match_generated_module_slot_decl(source, tokens, i) else {
            i += 1;
            continue;
        };

        aliases.insert(0, first_alias);
        i = next;
        while let Some((next, alias)) = match_generated_module_slot_decl(source, tokens, i) {
            let idx = aliases.len();
            aliases.insert(idx, alias);
            i = next;
        }
        break;
    }
    aliases
}

fn match_generated_module_slot_decl(
    source: &str,
    tokens: &[Tok],
    i: usize,
) -> Option<(usize, String)> {
    if i + 8 >= tokens.len() {
        return None;
    }
    if tokens[i].kind != TK::Ident || txt(source, &tokens[i]) != "var" {
        return None;
    }
    if tokens[i + 1].kind != TK::Ident {
        return None;
    }
    if tokens[i + 2].kind != TK::Punct || txt(source, &tokens[i + 2]) != "=" {
        return None;
    }
    if tokens[i + 3].kind != TK::Punct || txt(source, &tokens[i + 3]) != "{" {
        return None;
    }
    if tokens[i + 4].kind != TK::Ident || txt(source, &tokens[i + 4]) != "exports" {
        return None;
    }
    if tokens[i + 5].kind != TK::Punct || txt(source, &tokens[i + 5]) != ":" {
        return None;
    }
    if tokens[i + 6].kind != TK::Punct || txt(source, &tokens[i + 6]) != "{" {
        return None;
    }
    if tokens[i + 7].kind != TK::Punct || txt(source, &tokens[i + 7]) != "}" {
        return None;
    }
    if tokens[i + 8].kind != TK::Punct || txt(source, &tokens[i + 8]) != "}" {
        return None;
    }
    let mut next = i + 9;
    if next < tokens.len() && tokens[next].kind == TK::Punct && txt(source, &tokens[next]) == ";" {
        next += 1;
    }
    Some((next, txt(source, &tokens[i + 1]).to_string()))
}

fn generated_module_slot_index(name: &str) -> Option<usize> {
    let rest = name.strip_prefix("_m")?;
    if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    rest.parse().ok()
}

fn repair_generated_module_slot_local_decl_collisions(source: &str) -> String {
    let tokens = tokenize(source);
    if tokens.is_empty() {
        return source.to_string();
    }

    let mut slot_names = HashSet::new();
    let mut slot_decl_name_tokens: HashSet<usize> = HashSet::new();
    for i in 0..tokens.len() {
        if let Some((_, slot_name)) = match_generated_module_slot_decl(source, &tokens, i) {
            slot_names.insert(slot_name);
            // `var <name>={exports:{}}` — the name ident is token i+1.
            slot_decl_name_tokens.insert(i + 1);
        }
    }
    if slot_names.is_empty() {
        return source.to_string();
    }

    let mut used_names = HashSet::new();
    for tok in &tokens {
        if tok.kind == TK::Ident {
            used_names.insert(txt(source, tok).to_string());
        }
    }

    // Rename by binding, not by nearest block: `var` hoists, so a
    // colliding local declared inside a tiny `else{...}` is referenced
    // throughout the whole enclosing function. Renaming only the
    // declaration's block left the hoisted references resolving to the
    // top-level module wrapper (`TypeError: xa is not a function` on
    // react-dom event dispatch). Resolve every identifier occurrence
    // through the scope model and rename exactly the tokens bound to
    // the colliding declaration.
    let si = build_scopes(source, &tokens);
    let mut repaired: HashSet<(String, usize)> = HashSet::new();
    let mut repls = Vec::new();
    let mut i = 0usize;
    while i < tokens.len() {
        if tokens[i].kind != TK::Ident
            || !matches!(txt(source, &tokens[i]), "var" | "let" | "const")
        {
            i += 1;
            continue;
        }
        if match_generated_module_slot_decl(source, &tokens, i).is_some() {
            i += 1;
            continue;
        }

        let decl_tokens = declared_binding_tokens_after_keyword(source, &tokens, i + 1);
        for decl_ti in decl_tokens {
            let name = txt(source, &tokens[decl_ti]);
            if !slot_names.contains(name) {
                continue;
            }
            let Some(decl_scope) =
                resolve_decl_scope(name, si.token_scope[decl_ti], &si.scopes)
            else {
                continue;
            };
            // Top-level binding IS the wrapper slot itself; nothing to do.
            if decl_scope == 0 {
                continue;
            }
            if !repaired.insert((name.to_string(), decl_scope)) {
                continue;
            }
            let replacement = fresh_repair_identifier(name, &mut used_names);
            for ti in 0..tokens.len() {
                if tokens[ti].kind != TK::Ident || txt(source, &tokens[ti]) != name {
                    continue;
                }
                if should_skip_identifier_rename(source, &tokens, ti) {
                    continue;
                }
                // Wrapper-slot usages keep the slot name even when the
                // flattener merged the wrapper and the local into one
                // scope: the slot declaration itself and `<name>.exports`
                // accesses address the module wrapper, not the local.
                if slot_decl_name_tokens.contains(&ti)
                    || (matches_punct(source, &tokens, ti + 1, ".")
                        && matches_ident(source, &tokens, ti + 2, "exports"))
                {
                    continue;
                }
                if resolve_decl_scope(name, si.token_scope[ti], &si.scopes) == Some(decl_scope) {
                    repls.push((tokens[ti].start, tokens[ti].end, replacement.clone()));
                }
            }
        }

        i += 1;
    }

    if repls.is_empty() {
        source.to_string()
    } else {
        apply_replacements(source, repls)
    }
}

fn declared_binding_tokens_after_keyword(
    source: &str,
    tokens: &[Tok],
    start_ti: usize,
) -> Vec<usize> {
    let mut bindings = Vec::new();
    let mut ti = start_ti;
    let mut expect_decl_name = true;
    let mut paren_depth = 0i32;
    let mut brace_depth = 0i32;
    let mut bracket_depth = 0i32;

    while ti < tokens.len() {
        if tokens[ti].kind == TK::Ident && expect_decl_name {
            bindings.push(ti);
            expect_decl_name = false;
            ti += 1;
            continue;
        }

        if tokens[ti].kind == TK::Punct {
            match txt(source, &tokens[ti]) {
                "(" => paren_depth += 1,
                ")" => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                    }
                }
                "[" => bracket_depth += 1,
                "]" => {
                    if bracket_depth > 0 {
                        bracket_depth -= 1;
                    }
                }
                "{" => brace_depth += 1,
                "}" => {
                    if brace_depth > 0 {
                        brace_depth -= 1;
                    } else {
                        break;
                    }
                }
                "," if paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 => {
                    expect_decl_name = true;
                }
                ";" if paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 => break,
                _ => {}
            }
        }
        ti += 1;
    }

    bindings
}

fn fresh_repair_identifier(base_name: &str, used_names: &mut HashSet<String>) -> String {
    let mut idx = 0usize;
    loop {
        let candidate = if idx == 0 {
            format!("__jet_local_{base_name}")
        } else {
            format!("__jet_local_{base_name}_{idx}")
        };
        if used_names.insert(candidate.clone()) {
            return candidate;
        }
        idx += 1;
    }
}

fn repair_generated_require_helper_references(source: &str) -> String {
    let tokens = tokenize(source);
    if tokens.is_empty() {
        return source.to_string();
    }
    let Some(helper_alias) = find_generated_require_helper_alias(source, &tokens) else {
        return source.to_string();
    };
    if helper_alias == "_r" {
        return source.to_string();
    }

    let si = build_scopes(source, &tokens);
    let mut repls = Vec::new();
    for (ti, tok) in tokens.iter().enumerate() {
        if tok.kind != TK::Ident || txt(source, tok) != "_r" {
            continue;
        }
        if should_skip_identifier_rename(source, &tokens, ti) {
            continue;
        }
        if resolve_decl_scope("_r", si.token_scope[ti], &si.scopes).is_some() {
            continue;
        }
        repls.push((tok.start, tok.end, helper_alias.clone()));
    }

    if repls.is_empty() {
        source.to_string()
    } else {
        apply_replacements(source, repls)
    }
}

fn find_generated_require_helper_alias(source: &str, tokens: &[Tok]) -> Option<String> {
    for i in 0..tokens.len().saturating_sub(12) {
        if tokens[i].kind != TK::Ident || txt(source, &tokens[i]) != "function" {
            continue;
        }
        if tokens[i + 1].kind != TK::Ident {
            continue;
        }
        if tokens[i + 2].kind != TK::Punct || txt(source, &tokens[i + 2]) != "(" {
            continue;
        }
        let Some(body_open) = tokens[i + 3..]
            .iter()
            .position(|tok| tok.kind == TK::Punct && txt(source, tok) == "{")
            .map(|offset| i + 3 + offset)
        else {
            continue;
        };
        let body_end = matching_punct_token(source, tokens, body_open, "{", "}")
            .unwrap_or((body_open + 64).min(tokens.len().saturating_sub(1)));
        let scan_end = body_end.min(body_open + 64);
        let mut has_return = false;
        let mut has_exports = false;
        let mut has_ternary = false;
        for tok in &tokens[body_open..=scan_end] {
            let raw = txt(source, tok);
            has_return |= tok.kind == TK::Ident && raw == "return";
            has_exports |= tok.kind == TK::Ident && raw == "exports";
            has_ternary |= tok.kind == TK::Punct && raw == "?";
        }
        if has_return && has_exports && has_ternary {
            return Some(txt(source, &tokens[i + 1]).to_string());
        }
    }
    None
}

fn matching_punct_token(
    source: &str,
    tokens: &[Tok],
    open_ti: usize,
    open: &str,
    close: &str,
) -> Option<usize> {
    let mut depth = 0usize;
    for (ti, tok) in tokens.iter().enumerate().skip(open_ti) {
        if tok.kind != TK::Punct {
            continue;
        }
        let raw = txt(source, tok);
        if raw == open {
            depth += 1;
        } else if raw == close {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(ti);
            }
        }
    }
    None
}

fn repair_retained_wrapper_cjs_param_references(source: &str) -> String {
    let tokens = tokenize(source);
    if tokens.is_empty() {
        return source.to_string();
    }

    let si = build_scopes(source, &tokens);
    let mut repls = Vec::new();
    let mut i = 0usize;
    while i + 10 < tokens.len() {
        if tokens[i].kind != TK::Ident || txt(source, &tokens[i]) != "function" {
            i += 1;
            continue;
        }
        if i > 0 && tokens[i - 1].kind == TK::Punct && txt(source, &tokens[i - 1]) == "." {
            i += 1;
            continue;
        }
        let params_open = i + 1;
        if tokens[params_open].kind != TK::Punct || txt(source, &tokens[params_open]) != "(" {
            i += 1;
            continue;
        }
        let params_close = match matching_punct_token(source, &tokens, params_open, "(", ")") {
            Some(idx) => idx,
            None => {
                i += 1;
                continue;
            }
        };
        let param_idents: Vec<(usize, String)> = (params_open + 1..params_close)
            .filter(|&idx| tokens[idx].kind == TK::Ident)
            .map(|idx| (idx, txt(source, &tokens[idx]).to_string()))
            .collect();
        if param_idents.len() != 3 {
            i += 1;
            continue;
        }
        let body_open = params_close + 1;
        if body_open >= tokens.len()
            || tokens[body_open].kind != TK::Punct
            || txt(source, &tokens[body_open]) != "{"
        {
            i += 1;
            continue;
        }
        let Some(body_close) = matching_punct_token(source, &tokens, body_open, "{", "}") else {
            i += 1;
            continue;
        };
        if body_close + 1 >= tokens.len()
            || tokens[body_close + 1].kind != TK::Punct
            || txt(source, &tokens[body_close + 1]) != "("
        {
            i = body_close + 1;
            continue;
        }

        let aliases = [
            ("module", param_idents[0].1.as_str()),
            ("exports", param_idents[1].1.as_str()),
            ("require", param_idents[2].1.as_str()),
        ];
        for ti in body_open + 1..body_close {
            if tokens[ti].kind != TK::Ident || should_skip_identifier_rename(source, &tokens, ti) {
                continue;
            }
            let name = txt(source, &tokens[ti]);
            let Some((_, alias)) = aliases.iter().find(|(original, _)| *original == name) else {
                continue;
            };
            if *alias == name {
                continue;
            }
            if resolve_decl_scope(name, si.token_scope[ti], &si.scopes).is_some() {
                continue;
            }
            repls.push((tokens[ti].start, tokens[ti].end, (*alias).to_string()));
        }
        i = body_close + 1;
    }

    if repls.is_empty() {
        source.to_string()
    } else {
        apply_replacements(source, repls)
    }
}

fn repair_react_symbol_constant_references(source: &str) -> String {
    let tokens = tokenize(source);
    if tokens.is_empty() {
        return source.to_string();
    }

    let mut repls: Vec<(usize, usize, String)> = Vec::new();
    for (ti, tok) in tokens.iter().enumerate() {
        if tok.kind != TK::Ident || should_skip_identifier_rename(source, &tokens, ti) {
            continue;
        }
        let name = txt(source, tok);
        let Some(symbol_name) = react_symbol_constant_name(name) else {
            continue;
        };
        let Some(alias) = nearest_symbol_for_alias_before(source, &tokens, ti, symbol_name) else {
            continue;
        };
        if alias != name {
            repls.push((tok.start, tok.end, alias));
        }
    }

    if repls.is_empty() {
        source.to_string()
    } else {
        apply_replacements(source, repls)
    }
}

fn react_symbol_constant_name(name: &str) -> Option<&'static str> {
    match name {
        "REACT_ELEMENT_TYPE" => Some("react.element"),
        "REACT_TRANSITIONAL_ELEMENT_TYPE" => Some("react.transitional.element"),
        "REACT_PORTAL_TYPE" => Some("react.portal"),
        "REACT_FRAGMENT_TYPE" => Some("react.fragment"),
        "REACT_STRICT_MODE_TYPE" => Some("react.strict_mode"),
        "REACT_PROFILER_TYPE" => Some("react.profiler"),
        "REACT_PROVIDER_TYPE" => Some("react.provider"),
        "REACT_CONSUMER_TYPE" => Some("react.consumer"),
        "REACT_CONTEXT_TYPE" => Some("react.context"),
        "REACT_FORWARD_REF_TYPE" => Some("react.forward_ref"),
        "REACT_SUSPENSE_TYPE" => Some("react.suspense"),
        "REACT_SUSPENSE_LIST_TYPE" => Some("react.suspense_list"),
        "REACT_MEMO_TYPE" => Some("react.memo"),
        "REACT_LAZY_TYPE" => Some("react.lazy"),
        "REACT_SCOPE_TYPE" => Some("react.scope"),
        "REACT_ACTIVITY_TYPE" => Some("react.activity"),
        "REACT_LEGACY_HIDDEN_TYPE" => Some("react.legacy_hidden"),
        "REACT_TRACING_MARKER_TYPE" => Some("react.tracing_marker"),
        "REACT_MEMO_CACHE_SENTINEL" => Some("react.memo_cache_sentinel"),
        "REACT_VIEW_TRANSITION_TYPE" => Some("react.view_transition"),
        _ => None,
    }
}

fn nearest_symbol_for_alias_before(
    source: &str,
    tokens: &[Tok],
    before_ti: usize,
    symbol_name: &str,
) -> Option<String> {
    if before_ti < 6 {
        return None;
    }

    let mut idx = before_ti - 6;
    loop {
        if tokens[idx].kind == TK::Ident
            && idx + 6 < before_ti
            && tokens[idx + 1].kind == TK::Punct
            && txt(source, &tokens[idx + 1]) == "="
            && tokens[idx + 2].kind == TK::Ident
            && txt(source, &tokens[idx + 2]) == "Symbol"
            && tokens[idx + 3].kind == TK::Punct
            && txt(source, &tokens[idx + 3]) == "."
            && tokens[idx + 4].kind == TK::Ident
            && txt(source, &tokens[idx + 4]) == "for"
            && tokens[idx + 5].kind == TK::Punct
            && txt(source, &tokens[idx + 5]) == "("
            && tokens[idx + 6].kind == TK::Str
            && string_token_value(source, &tokens[idx + 6]).as_deref() == Some(symbol_name)
        {
            return Some(txt(source, &tokens[idx]).to_string());
        }
        if idx == 0 {
            break;
        }
        idx -= 1;
    }

    None
}

fn string_token_value(source: &str, tok: &Tok) -> Option<String> {
    let raw = txt(source, tok);
    let mut chars = raw.chars();
    let quote = chars.next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    if !raw.ends_with(quote) || raw.len() < 2 {
        return None;
    }
    Some(raw[1..raw.len() - 1].to_string())
}

#[derive(Default)]
struct ReactDomClientAliases {
    error_formatter: Option<String>,
    object_assign: Option<String>,
    react_shared_internals: Option<String>,
    react_module: Option<String>,
    exports_param: Option<String>,
}

fn repair_react_dom_client_stale_references(source: &str) -> String {
    let tokens = tokenize(source);
    if tokens.is_empty() {
        return source.to_string();
    }

    let mut repls: Vec<(usize, usize, String)> = Vec::new();
    for (ti, tok) in tokens.iter().enumerate() {
        if tok.kind != TK::Ident || should_skip_identifier_rename(source, &tokens, ti) {
            continue;
        }
        let name = txt(source, tok);
        let replacement = match name {
            "formatProdErrorMessage" => {
                react_dom_client_aliases_before(source, &tokens, ti).error_formatter
            }
            "assign" => react_dom_client_aliases_before(source, &tokens, ti).object_assign,
            "ReactSharedInternals" => {
                react_dom_client_aliases_before(source, &tokens, ti).react_shared_internals
            }
            "React" => react_dom_client_aliases_before(source, &tokens, ti).react_module,
            "exports" => react_dom_client_aliases_before(source, &tokens, ti).exports_param,
            _ => None,
        };
        if let Some(replacement) = replacement {
            if replacement != name {
                repls.push((tok.start, tok.end, replacement));
            }
        }
    }

    repls.extend(repair_react_update_properties_fallback_refs(
        source, &tokens,
    ));

    if repls.is_empty() {
        source.to_string()
    } else {
        apply_replacements(source, repls)
    }
}

fn react_dom_client_aliases_before(
    source: &str,
    tokens: &[Tok],
    before_ti: usize,
) -> ReactDomClientAliases {
    let (react_shared_internals, react_module) =
        nearest_react_shared_internals_aliases_before(source, tokens, before_ti)
            .unwrap_or((None, None));
    ReactDomClientAliases {
        error_formatter: nearest_react_error_formatter_alias_before(source, tokens, before_ti),
        object_assign: nearest_object_assign_alias_before(source, tokens, before_ti),
        react_shared_internals,
        react_module,
        exports_param: nearest_commonjs_exports_param_before(source, tokens, before_ti),
    }
}

fn nearest_react_error_formatter_alias_before(
    source: &str,
    tokens: &[Tok],
    before_ti: usize,
) -> Option<String> {
    let mut idx = before_ti;
    while idx > 0 {
        idx -= 1;
        if tokens[idx].kind != TK::Ident || txt(source, &tokens[idx]) != "function" {
            continue;
        }
        let Some(name_ti) = function_name_token_after(source, tokens, idx) else {
            continue;
        };
        let search_end = (idx + 80).min(before_ti);
        if tokens[idx..search_end].iter().any(|tok| {
            tok.kind == TK::Str
                && string_token_value(source, tok).as_deref() == Some("https://react.dev/errors/")
        }) {
            return Some(txt(source, &tokens[name_ti]).to_string());
        }
    }
    None
}

fn function_name_token_after(source: &str, tokens: &[Tok], function_ti: usize) -> Option<usize> {
    if function_ti + 1 >= tokens.len() {
        return None;
    }
    if tokens[function_ti + 1].kind == TK::Ident
        && function_ti + 2 < tokens.len()
        && tokens[function_ti + 2].kind == TK::Punct
        && txt(source, &tokens[function_ti + 2]) == "("
    {
        return Some(function_ti + 1);
    }
    None
}

fn nearest_object_assign_alias_before(
    source: &str,
    tokens: &[Tok],
    before_ti: usize,
) -> Option<String> {
    let mut idx = before_ti.saturating_sub(5);
    loop {
        if tokens[idx].kind == TK::Ident
            && idx + 4 < before_ti
            && tokens[idx + 1].kind == TK::Punct
            && txt(source, &tokens[idx + 1]) == "="
            && tokens[idx + 2].kind == TK::Ident
            && txt(source, &tokens[idx + 2]) == "Object"
            && tokens[idx + 3].kind == TK::Punct
            && txt(source, &tokens[idx + 3]) == "."
            && tokens[idx + 4].kind == TK::Ident
            && txt(source, &tokens[idx + 4]) == "assign"
        {
            return Some(txt(source, &tokens[idx]).to_string());
        }
        if idx == 0 {
            break;
        }
        idx -= 1;
    }
    None
}

fn nearest_react_shared_internals_aliases_before(
    source: &str,
    tokens: &[Tok],
    before_ti: usize,
) -> Option<(Option<String>, Option<String>)> {
    let mut idx = before_ti.saturating_sub(5);
    loop {
        if tokens[idx].kind == TK::Ident
            && idx + 4 < before_ti
            && tokens[idx + 1].kind == TK::Punct
            && txt(source, &tokens[idx + 1]) == "="
            && tokens[idx + 2].kind == TK::Ident
            && tokens[idx + 3].kind == TK::Punct
            && txt(source, &tokens[idx + 3]) == "."
            && tokens[idx + 4].kind == TK::Ident
            && txt(source, &tokens[idx + 4])
                == "__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE"
        {
            return Some((
                Some(txt(source, &tokens[idx]).to_string()),
                Some(txt(source, &tokens[idx + 2]).to_string()),
            ));
        }
        if idx == 0 {
            break;
        }
        idx -= 1;
    }
    None
}

fn nearest_commonjs_exports_param_before(
    source: &str,
    tokens: &[Tok],
    before_ti: usize,
) -> Option<String> {
    let mut idx = before_ti;
    while idx > 0 {
        idx -= 1;
        if tokens[idx].kind != TK::Ident || txt(source, &tokens[idx]) != "function" {
            continue;
        }
        let Some(open_ti) = function_param_open_after(source, tokens, idx) else {
            continue;
        };
        let params = simple_function_param_tokens(source, tokens, open_ti);
        if params.len() >= 3 && txt(source, &tokens[params[2]]) == "require" {
            return Some(txt(source, &tokens[params[1]]).to_string());
        }
    }
    None
}

fn function_param_open_after(source: &str, tokens: &[Tok], function_ti: usize) -> Option<usize> {
    if function_ti + 1 >= tokens.len() {
        return None;
    }
    if tokens[function_ti + 1].kind == TK::Punct && txt(source, &tokens[function_ti + 1]) == "(" {
        return Some(function_ti + 1);
    }
    if function_ti + 2 < tokens.len()
        && tokens[function_ti + 1].kind == TK::Ident
        && tokens[function_ti + 2].kind == TK::Punct
        && txt(source, &tokens[function_ti + 2]) == "("
    {
        return Some(function_ti + 2);
    }
    None
}

fn simple_function_param_tokens(source: &str, tokens: &[Tok], open_ti: usize) -> Vec<usize> {
    let mut params = Vec::new();
    let mut depth = 0usize;
    let mut ti = open_ti + 1;
    while ti < tokens.len() {
        if tokens[ti].kind == TK::Punct {
            match txt(source, &tokens[ti]) {
                "(" | "{" | "[" => depth += 1,
                ")" if depth == 0 => break,
                ")" | "}" | "]" => depth = depth.saturating_sub(1),
                _ => {}
            }
        } else if depth == 0 && tokens[ti].kind == TK::Ident {
            params.push(ti);
        }
        ti += 1;
    }
    params
}

fn repair_react_event_transition_alias_shadow(source: &str) -> String {
    let tokens = tokenize(source);
    if tokens.is_empty() {
        return source.to_string();
    }

    let mut repls: Vec<(usize, usize, String)> = Vec::new();
    for ti in 0..tokens.len().saturating_sub(15) {
        if !matches_ident(source, &tokens, ti, "var")
            || tokens[ti + 3].kind != TK::Ident
            || !matches_punct(source, &tokens, ti + 4, ".")
            || !matches_ident(source, &tokens, ti + 5, "T")
            || !matches_punct(source, &tokens, ti + 6, ";")
        {
            continue;
        }
        let outer_alias = txt(source, &tokens[ti + 3]);
        if tokens[ti + 7].kind != TK::Ident
            || txt(source, &tokens[ti + 7]) != outer_alias
            || !matches_punct(source, &tokens, ti + 8, ".")
            || !matches_ident(source, &tokens, ti + 9, "T")
            || !matches_punct(source, &tokens, ti + 10, "=")
            || !matches_ident(source, &tokens, ti + 11, "null")
            || !matches_punct(source, &tokens, ti + 12, ";")
            || !matches_ident(source, &tokens, ti + 13, "var")
            || tokens[ti + 14].kind != TK::Ident
            || txt(source, &tokens[ti + 14]) != outer_alias
        {
            continue;
        }

        let end_ti = next_function_token_after(source, &tokens, ti).unwrap_or(tokens.len());
        let replacement = next_available_alias_in_token_range(source, &tokens, ti, end_ti);
        for idx in (ti + 14)..end_ti {
            let tok = &tokens[idx];
            if tok.kind != TK::Ident || txt(source, tok) != outer_alias {
                continue;
            }
            if idx > 0 && matches_punct(source, &tokens, idx - 1, ".") {
                continue;
            }
            if idx + 1 < tokens.len() && matches_punct(source, &tokens, idx + 1, ".") {
                continue;
            }
            repls.push((tok.start, tok.end, replacement.clone()));
        }
    }

    if repls.is_empty() {
        source.to_string()
    } else {
        apply_replacements(source, repls)
    }
}

fn next_available_alias_in_token_range(
    source: &str,
    tokens: &[Tok],
    start_ti: usize,
    end_ti: usize,
) -> String {
    let mut used = HashSet::new();
    for tok in tokens.iter().take(end_ti).skip(start_ti) {
        if tok.kind == TK::Ident {
            used.insert(txt(source, tok).to_string());
        }
    }
    NameGen::new(&used).next_name()
}

fn repair_react_update_properties_fallback_refs(
    source: &str,
    tokens: &[Tok],
) -> Vec<(usize, usize, String)> {
    let mut repls = Vec::new();
    for ti in 0..tokens.len().saturating_sub(5) {
        if !matches_ident(source, tokens, ti, "for")
            || !matches_punct(source, tokens, ti + 1, "(")
            || !matches_ident(source, tokens, ti + 2, "var")
            || tokens[ti + 3].kind != TK::Ident
            || !matches_ident(source, tokens, ti + 4, "in")
            || tokens[ti + 5].kind != TK::Ident
        {
            continue;
        }
        let stale_last_props_ident = txt(source, &tokens[ti + 5]).to_string();

        let Some(params) = nearest_four_param_function_before(source, tokens, ti) else {
            continue;
        };
        let dom_alias = txt(source, &tokens[params[0]]).to_string();
        let tag_alias = txt(source, &tokens[params[1]]).to_string();
        let last_props_alias = txt(source, &tokens[params[2]]).to_string();
        let next_props_alias = txt(source, &tokens[params[3]]).to_string();
        let Some(prop_alias) =
            nearest_index_value_alias_before(source, tokens, ti, &last_props_alias)
        else {
            continue;
        };
        let end_ti = next_function_token_after(source, tokens, ti).unwrap_or(tokens.len());
        let scan_end_ti = (ti + 96).min(end_ti);
        if !tokens
            .iter()
            .enumerate()
            .take(scan_end_ti)
            .skip(ti)
            .any(|(idx, tok)| {
                tok.kind == TK::Ident
                    && !should_skip_identifier_rename(source, tokens, idx)
                    && matches!(txt(source, tok), "nextProps" | "domElement" | "propKey")
            })
        {
            continue;
        }

        for (idx, tok) in tokens.iter().enumerate().take(end_ti).skip(ti) {
            if tok.kind != TK::Ident || should_skip_identifier_rename(source, tokens, idx) {
                continue;
            }
            let replacement = match txt(source, tok) {
                name if name == stale_last_props_ident.as_str() => Some(last_props_alias.as_str()),
                "nextProps" => Some(next_props_alias.as_str()),
                "domElement" => Some(dom_alias.as_str()),
                "tag" => Some(tag_alias.as_str()),
                "propKey" => Some(prop_alias.as_str()),
                _ => None,
            };
            if let Some(replacement) = replacement {
                repls.push((tok.start, tok.end, replacement.to_string()));
            }
        }
    }
    repls
}

fn matches_ident(source: &str, tokens: &[Tok], ti: usize, expected: &str) -> bool {
    ti < tokens.len() && tokens[ti].kind == TK::Ident && txt(source, &tokens[ti]) == expected
}

fn matches_punct(source: &str, tokens: &[Tok], ti: usize, expected: &str) -> bool {
    ti < tokens.len() && tokens[ti].kind == TK::Punct && txt(source, &tokens[ti]) == expected
}

fn nearest_four_param_function_before(
    source: &str,
    tokens: &[Tok],
    before_ti: usize,
) -> Option<[usize; 4]> {
    let mut idx = before_ti;
    while idx > 0 {
        idx -= 1;
        if tokens[idx].kind != TK::Ident || txt(source, &tokens[idx]) != "function" {
            continue;
        }
        let Some(open_ti) = function_param_open_after(source, tokens, idx) else {
            continue;
        };
        let params = simple_function_param_tokens(source, tokens, open_ti);
        if params.len() == 4 {
            return Some([params[0], params[1], params[2], params[3]]);
        }
    }
    None
}

fn nearest_index_value_alias_before(
    source: &str,
    tokens: &[Tok],
    before_ti: usize,
    object_alias: &str,
) -> Option<String> {
    let mut idx = before_ti.saturating_sub(5);
    loop {
        if tokens[idx].kind == TK::Ident
            && idx + 4 < before_ti
            && tokens[idx + 1].kind == TK::Punct
            && txt(source, &tokens[idx + 1]) == "="
            && tokens[idx + 2].kind == TK::Ident
            && txt(source, &tokens[idx + 2]) == object_alias
            && tokens[idx + 3].kind == TK::Punct
            && txt(source, &tokens[idx + 3]) == "["
        {
            return Some(txt(source, &tokens[idx]).to_string());
        }
        if idx == 0 {
            break;
        }
        idx -= 1;
    }
    None
}

fn next_function_token_after(source: &str, tokens: &[Tok], start_ti: usize) -> Option<usize> {
    for (ti, tok) in tokens.iter().enumerate().skip(start_ti + 1) {
        if tok.kind == TK::Ident && txt(source, tok) == "function" {
            return Some(ti);
        }
    }
    None
}

// === Tokenizer ===

#[derive(Debug, Clone, Copy, PartialEq)]
enum TK {
    Ident,
    Str,
    Num,
    Regex,
    Punct,
}

#[derive(Debug, Clone)]
struct Tok {
    kind: TK,
    start: usize,
    end: usize,
}

fn txt<'a>(s: &'a str, t: &Tok) -> &'a str {
    &s[t.start..t.end]
}

fn tokenize(source: &str) -> Vec<Tok> {
    let b = source.as_bytes();
    let len = b.len();
    let mut toks = Vec::new();
    let mut i = 0;
    let mut prev = TK::Punct;

    while i < len {
        if b[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if is_id_start(b[i]) {
            let s = i;
            while i < len && is_id_cont(b[i]) {
                i += 1;
            }
            toks.push(Tok {
                kind: TK::Ident,
                start: s,
                end: i,
            });
            prev = TK::Ident;
            continue;
        }
        if b[i].is_ascii_digit() || (b[i] == b'.' && i + 1 < len && b[i + 1].is_ascii_digit()) {
            let s = i;
            if b[i] == b'0'
                && i + 1 < len
                && matches!(b[i + 1], b'x' | b'X' | b'b' | b'B' | b'o' | b'O')
            {
                i += 2;
            } else {
                i += 1;
            }
            while i < len && (b[i].is_ascii_alphanumeric() || b[i] == b'.' || b[i] == b'_') {
                i += 1;
            }
            if i < len && matches!(b[i], b'e' | b'E') {
                i += 1;
                if i < len && matches!(b[i], b'+' | b'-') {
                    i += 1;
                }
                while i < len && b[i].is_ascii_digit() {
                    i += 1;
                }
            }
            toks.push(Tok {
                kind: TK::Num,
                start: s,
                end: i,
            });
            prev = TK::Num;
            continue;
        }
        if matches!(b[i], b'"' | b'\'') {
            let s = i;
            let q = b[i];
            i += 1;
            while i < len {
                if b[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if b[i] == q {
                    i += 1;
                    break;
                }
                i += 1;
            }
            toks.push(Tok {
                kind: TK::Str,
                start: s,
                end: i,
            });
            prev = TK::Str;
            continue;
        }
        if b[i] == b'`' {
            let (end, mut inner_tokens) = tokenize_template_expressions(source, i + 1);
            toks.append(&mut inner_tokens);
            i = end;
            prev = TK::Str;
            continue;
        }
        if b[i] == b'/' {
            if i + 1 < len && b[i + 1] == b'/' {
                while i < len && b[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if i + 1 < len && b[i + 1] == b'*' {
                i += 2;
                while i + 1 < len && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                i += 2;
                continue;
            }
            let is_rx = match prev {
                TK::Ident | TK::Num | TK::Str => false,
                _ => {
                    if let Some(last) = toks.last() {
                        let lt = txt(source, last);
                        lt != ")" && lt != "]"
                    } else {
                        true
                    }
                }
            };
            if is_rx && i + 1 < len && b[i + 1] != b'/' && b[i + 1] != b'*' {
                let s = i;
                i += 1;
                while i < len && b[i] != b'/' {
                    if b[i] == b'\\' {
                        i += 1;
                    }
                    if i < len && b[i] == b'[' {
                        i += 1;
                        while i < len && b[i] != b']' {
                            if b[i] == b'\\' {
                                i += 1;
                            }
                            i += 1;
                        }
                    }
                    i += 1;
                }
                if i < len {
                    i += 1;
                }
                while i < len && b[i].is_ascii_alphabetic() {
                    i += 1;
                }
                toks.push(Tok {
                    kind: TK::Regex,
                    start: s,
                    end: i,
                });
                prev = TK::Regex;
                continue;
            }
        }
        if b[i] == b'=' && i + 1 < len && b[i + 1] == b'>' {
            let s = i;
            i += 2;
            toks.push(Tok {
                kind: TK::Punct,
                start: s,
                end: i,
            });
            prev = TK::Punct;
            continue;
        }
        let s = i;
        i += 1;
        toks.push(Tok {
            kind: TK::Punct,
            start: s,
            end: i,
        });
        prev = TK::Punct;
    }
    toks
}

fn tokenize_template_expressions(source: &str, mut i: usize) -> (usize, Vec<Tok>) {
    let b = source.as_bytes();
    let len = b.len();
    let mut toks = Vec::new();

    while i < len {
        match b[i] {
            b'\\' => {
                i += 2;
            }
            b'`' => {
                return (i + 1, toks);
            }
            b'$' if i + 1 < len && b[i + 1] == b'{' => {
                let expr_start = i + 2;
                let expr_end = find_template_expr_end(source, expr_start);
                let mut expr_tokens = tokenize(&source[expr_start..expr_end]);
                for tok in &mut expr_tokens {
                    tok.start += expr_start;
                    tok.end += expr_start;
                }
                toks.append(&mut expr_tokens);
                i = (expr_end + 1).min(len);
            }
            _ => {
                i += 1;
            }
        }
    }

    (len, toks)
}

fn find_template_expr_end(source: &str, mut i: usize) -> usize {
    let b = source.as_bytes();
    let len = b.len();
    let mut depth = 1i32;

    while i < len {
        match b[i] {
            b'"' | b'\'' => {
                i = skip_quoted_literal(b, i);
            }
            b'`' => {
                let (end, _) = tokenize_template_expressions(source, i + 1);
                i = end;
            }
            b'/' if i + 1 < len && b[i + 1] == b'/' => {
                i += 2;
                while i < len && b[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < len && b[i + 1] == b'*' => {
                i += 2;
                while i + 1 < len && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(len);
            }
            b'{' => {
                depth += 1;
                i += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    len
}

fn skip_quoted_literal(b: &[u8], mut i: usize) -> usize {
    let len = b.len();
    let q = b[i];
    i += 1;
    while i < len {
        if b[i] == b'\\' {
            i += 2;
            continue;
        }
        if b[i] == q {
            return i + 1;
        }
        i += 1;
    }
    len
}

fn is_id_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_' || c == b'$'
}
fn is_id_cont(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
}

// === Scope Analysis ===

struct Scope {
    parent: Option<usize>,
    is_function: bool,
    decls: HashSet<String>,
}

struct ScopeInfo {
    scopes: Vec<Scope>,
    token_scope: Vec<usize>,
}

fn is_param_binding_identifier(source: &str, tokens: &[Tok], ti: usize) -> bool {
    if tokens[ti].kind != TK::Ident {
        return false;
    }
    if ti > 0 && tokens[ti - 1].kind == TK::Punct && txt(source, &tokens[ti - 1]) == "." {
        return false;
    }
    if ti + 1 < tokens.len()
        && tokens[ti + 1].kind == TK::Punct
        && txt(source, &tokens[ti + 1]) == ":"
    {
        return false;
    }
    if is_after_param_default_operator(source, tokens, ti) {
        return false;
    }
    if ti > 0 && tokens[ti - 1].kind == TK::Punct {
        let prev = txt(source, &tokens[ti - 1]);
        return matches!(prev, "(" | "{" | "[" | "," | ":");
    }
    false
}

fn is_object_pattern_binding_identifier(source: &str, tokens: &[Tok], ti: usize) -> bool {
    if tokens[ti].kind != TK::Ident {
        return false;
    }
    if ti > 0 && tokens[ti - 1].kind == TK::Punct && txt(source, &tokens[ti - 1]) == "." {
        return false;
    }
    if ti + 1 < tokens.len()
        && tokens[ti + 1].kind == TK::Punct
        && txt(source, &tokens[ti + 1]) == ":"
    {
        return false;
    }
    if ti > 0 && tokens[ti - 1].kind == TK::Punct && txt(source, &tokens[ti - 1]) == ":" {
        return true;
    }
    if ti > 0 && tokens[ti - 1].kind == TK::Punct {
        let prev = txt(source, &tokens[ti - 1]);
        if matches!(prev, "{" | ",") {
            return ti + 1 >= tokens.len()
                || tokens[ti + 1].kind != TK::Punct
                || matches!(txt(source, &tokens[ti + 1]), "," | "}" | "=");
        }
    }
    false
}

fn is_function_declaration_context(source: &str, tokens: &[Tok], ti: usize) -> bool {
    if ti == 0 {
        return true;
    }

    let prev = &tokens[ti - 1];
    if prev.kind == TK::Punct {
        return matches!(txt(source, prev), "{" | "}" | ";");
    }

    false
}

fn is_after_param_default_operator(source: &str, tokens: &[Tok], ti: usize) -> bool {
    let mut idx = ti;
    while idx > 0 {
        idx -= 1;
        if tokens[idx].kind != TK::Punct {
            continue;
        }
        match txt(source, &tokens[idx]) {
            "=" => return true,
            "," | "(" | ":" => return false,
            _ => {}
        }
    }
    false
}

fn arrow_param_binding_tokens_before(source: &str, tokens: &[Tok], arrow_ti: usize) -> Vec<usize> {
    let mut binding_tokens = Vec::new();
    if arrow_ti == 0 {
        return binding_tokens;
    }
    let prev_ti = arrow_ti - 1;
    if tokens[prev_ti].kind == TK::Ident {
        let name = txt(source, &tokens[prev_ti]);
        if !is_reserved(name) {
            binding_tokens.push(prev_ti);
        }
        return binding_tokens;
    }

    if tokens[prev_ti].kind != TK::Punct || txt(source, &tokens[prev_ti]) != ")" {
        return binding_tokens;
    }

    let mut depth = 0usize;
    let mut open_ti = None;
    for ti in (0..=prev_ti).rev() {
        if tokens[ti].kind != TK::Punct {
            continue;
        }
        match txt(source, &tokens[ti]) {
            ")" => depth += 1,
            "(" => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    open_ti = Some(ti);
                    break;
                }
            }
            _ => {}
        }
    }

    let Some(open_ti) = open_ti else {
        return binding_tokens;
    };

    for ti in (open_ti + 1)..prev_ti {
        if tokens[ti].kind != TK::Ident {
            continue;
        }
        let name = txt(source, &tokens[ti]);
        if is_reserved(name) {
            continue;
        }
        if ti > open_ti + 1
            && tokens[ti - 1].kind == TK::Punct
            && txt(source, &tokens[ti - 1]) == "."
        {
            continue;
        }
        if ti + 1 < prev_ti
            && tokens[ti + 1].kind == TK::Punct
            && txt(source, &tokens[ti + 1]) == ":"
        {
            continue;
        }
        if ti > open_ti + 1
            && tokens[ti - 1].kind == TK::Punct
            && txt(source, &tokens[ti - 1]) == "="
        {
            continue;
        }
        binding_tokens.push(ti);
    }

    binding_tokens
}

fn arrow_expression_body_end(source: &str, tokens: &[Tok], body_start: usize) -> Option<usize> {
    if body_start >= tokens.len() {
        return None;
    }

    let mut depth = 0usize;
    for ti in body_start..tokens.len() {
        if tokens[ti].kind != TK::Punct {
            continue;
        }
        match txt(source, &tokens[ti]) {
            "(" | "{" | "[" => depth += 1,
            ")" | "}" | "]" => {
                if depth == 0 {
                    return (ti > body_start).then_some(ti - 1);
                }
                depth = depth.saturating_sub(1);
            }
            ";" | "," if depth == 0 => {
                return (ti > body_start).then_some(ti - 1);
            }
            _ => {}
        }
    }

    Some(tokens.len() - 1)
}

fn build_scopes(source: &str, tokens: &[Tok]) -> ScopeInfo {
    let n = tokens.len();
    let mut scopes = vec![Scope {
        parent: None,
        is_function: true,
        decls: HashSet::new(),
    }];
    let mut stack: Vec<usize> = vec![0];
    let mut ts = vec![0usize; n];
    let mut pending_fn: Option<usize> = None;
    let mut in_decl = false; // between var/let/const and ;
    let mut expect_decl_name = false; // expecting the next ident to be a decl name
    let mut decl_is_var = false; // true if current decl is `var` (function-scoped)
    let mut decl_paren_depth: i32 = 0; // track parens inside declarations
    let mut decl_brace_depth: i32 = 0; // track braces inside declarations (object literals)
    let mut decl_bracket_depth: i32 = 0; // track brackets inside declarations (arrays/computed keys)
    let mut decl_object_pattern_depth: i32 = 0; // track object destructuring binding patterns
                                                // Save/restore decl state when entering/leaving function scopes.
                                                // Without this, `var o = {f: function() { var inner = 1; }}, next = 0;`
                                                // would lose the outer `in_decl` when the inner `var` resets it.
    let mut decl_state_stack: Vec<(bool, bool, bool, i32, i32, i32, i32)> = Vec::new();
    let mut expect_fn_params = false;
    let mut pending_class = false; // between `class` and its body `{`
    let mut expect_class_name = false; // expecting the class name ident
    let mut pending_fn_name: Option<String> = None;
    let mut pending_fn_name_token: Option<usize> = None;
    let mut pending_fn_is_declaration = false;
    let mut in_params = false;
    let mut param_depth = 0;
    let mut param_scope: Option<usize> = None;
    let mut expr_arrow_scope_stack: Vec<(usize, usize)> = Vec::new();

    for i in 0..n {
        while let Some(&(scope_id, end_ti)) = expr_arrow_scope_stack.last() {
            if i <= end_ti {
                break;
            }
            expr_arrow_scope_stack.pop();
            if stack.last().copied() == Some(scope_id) {
                stack.pop();
            }
        }

        let cur = *stack.last().unwrap();
        ts[i] = cur;
        let tok = &tokens[i];

        if tok.kind == TK::Ident {
            let name = txt(source, tok);
            // Skip ident after dot (property access)
            if i > 0 && tokens[i - 1].kind == TK::Punct && txt(source, &tokens[i - 1]) == "." {
                expect_fn_params = false;
                continue;
            }
            match name {
                "function" => {
                    expect_fn_params = true;
                    pending_fn_name = None;
                    pending_fn_name_token = None;
                    pending_fn_is_declaration = is_function_declaration_context(source, tokens, i);
                    continue;
                }
                "var" | "let" | "const" => {
                    in_decl = true;
                    expect_decl_name = true;
                    decl_is_var = name == "var";
                    decl_paren_depth = 0;
                    decl_brace_depth = 0;
                    decl_bracket_depth = 0;
                    decl_object_pattern_depth = 0;
                    continue;
                }
                "class" => {
                    // Not the keyword when used as an object-literal key
                    // (`{class: ...}`); a `:` follows in that case.
                    if !matches_punct(source, tokens, i + 1, ":") {
                        // The class body `{` must open a real scope. Without
                        // this, a class expression inside a declaration
                        // initializer was absorbed into decl_brace_depth, a
                        // `let`/`var` inside a method body reset that counter,
                        // and the class's closing braces popped real scopes —
                        // collapsing the model to the program root and letting
                        // NameGen mint names that collided with later `class`
                        // declarations (duplicate-declaration SyntaxError in
                        // styled-components bundles).
                        pending_class = true;
                        expect_class_name = true;
                        continue;
                    }
                }
                "extends" if pending_class => {
                    // Anonymous `class extends Base {` — no name to record.
                    expect_class_name = false;
                    continue;
                }
                _ => {}
            }
            if expect_class_name {
                // Class names are block-scoped declarations like `let`.
                scopes[cur].decls.insert(name.to_string());
                expect_class_name = false;
                continue;
            }
            if in_decl && decl_object_pattern_depth > 0 {
                if is_object_pattern_binding_identifier(source, tokens, i) {
                    if decl_is_var {
                        let target = stack
                            .iter()
                            .rev()
                            .find(|&&s| scopes[s].is_function)
                            .copied()
                            .unwrap_or(cur);
                        scopes[target].decls.insert(name.to_string());
                    } else {
                        scopes[cur].decls.insert(name.to_string());
                    }
                }
                continue;
            }
            if expect_fn_params {
                pending_fn_name = Some(name.to_string());
                pending_fn_name_token = Some(i);
                if pending_fn_is_declaration {
                    scopes[cur].decls.insert(name.to_string());
                }
                continue;
            }
            if in_params {
                if let Some(ps) = param_scope {
                    ts[i] = ps;
                    if is_param_binding_identifier(source, tokens, i) {
                        scopes[ps].decls.insert(name.to_string());
                    }
                }
                continue;
            }
            if expect_decl_name {
                if decl_is_var {
                    // `var` is function-scoped: hoist to nearest function scope
                    let target = stack
                        .iter()
                        .rev()
                        .find(|&&s| scopes[s].is_function)
                        .copied()
                        .unwrap_or(cur);
                    scopes[target].decls.insert(name.to_string());
                } else {
                    // `let`/`const` are block-scoped
                    scopes[cur].decls.insert(name.to_string());
                }
                expect_decl_name = false;
                continue;
            }
        }

        if tok.kind == TK::Punct {
            let p = txt(source, tok);
            match p {
                "(" => {
                    if expect_fn_params {
                        let new_id = scopes.len();
                        scopes.push(Scope {
                            parent: Some(cur),
                            is_function: true,
                            decls: if pending_fn_is_declaration {
                                HashSet::new()
                            } else {
                                pending_fn_name
                                    .take()
                                    .map(|name| HashSet::from([name]))
                                    .unwrap_or_default()
                            },
                        });
                        if !pending_fn_is_declaration {
                            if let Some(name_ti) = pending_fn_name_token.take() {
                                ts[name_ti] = new_id;
                            }
                        }
                        in_params = true;
                        param_depth = 1;
                        param_scope = Some(new_id);
                        pending_fn = Some(new_id);
                        expect_fn_params = false;
                        pending_fn_name = None;
                        pending_fn_name_token = None;
                        pending_fn_is_declaration = false;
                        continue;
                    }
                    if in_params {
                        param_depth += 1;
                        continue;
                    }
                    if in_decl {
                        decl_paren_depth += 1;
                    }
                }
                ")" => {
                    if in_params {
                        param_depth -= 1;
                        if param_depth == 0 {
                            in_params = false;
                            param_scope = None;
                        }
                        continue;
                    }
                    if in_decl && decl_paren_depth > 0 {
                        decl_paren_depth -= 1;
                    } else if in_decl
                        && decl_paren_depth == 0
                        && decl_brace_depth == 0
                        && decl_bracket_depth == 0
                    {
                        // A `)` with no open decl-parens closes an enclosing
                        // construct — a `for (var x in y)` / `for (var x of y)`
                        // head. The declaration ends here; without this, the
                        // loop body `{` was swallowed into decl_brace_depth,
                        // an inner `var` then reset that counter, and the
                        // orphaned `}` popped a real scope. Every binding
                        // after that point resolved against the wrong scope
                        // chain (react-dom's updateProperties broke at
                        // runtime with stale single-letter references).
                        in_decl = false;
                        expect_decl_name = false;
                    }
                }
                "=>" => {
                    if pending_fn.is_none() {
                        let has_block_body = i + 1 < n
                            && tokens[i + 1].kind == TK::Punct
                            && txt(source, &tokens[i + 1]) == "{";
                        let param_tokens = arrow_param_binding_tokens_before(source, tokens, i);
                        let decls: HashSet<String> = param_tokens
                            .iter()
                            .map(|&ti| txt(source, &tokens[ti]).to_string())
                            .collect();
                        if has_block_body {
                            if i > 0 && tokens[i - 1].kind == TK::Ident {
                                scopes[cur].decls.remove(txt(source, &tokens[i - 1]));
                            }
                            let new_id = scopes.len();
                            scopes.push(Scope {
                                parent: Some(cur),
                                is_function: true,
                                decls,
                            });
                            for param_ti in param_tokens {
                                ts[param_ti] = new_id;
                            }
                            pending_fn = Some(new_id);
                        } else if let Some(end_ti) =
                            arrow_expression_body_end(source, tokens, i + 1)
                        {
                            if i > 0 && tokens[i - 1].kind == TK::Ident {
                                scopes[cur].decls.remove(txt(source, &tokens[i - 1]));
                            }
                            let new_id = scopes.len();
                            scopes.push(Scope {
                                parent: Some(cur),
                                is_function: true,
                                decls,
                            });
                            for param_ti in param_tokens {
                                ts[param_ti] = new_id;
                            }
                            stack.push(new_id);
                            expr_arrow_scope_stack.push((new_id, end_ti));
                        }
                    }
                }
                "{" => {
                    if in_params {
                        param_depth += 1;
                        continue;
                    }
                    if pending_class {
                        // Class body: open a real scope and save the decl
                        // state exactly like a function body, so methods'
                        // `var`/`let` declarations cannot corrupt an
                        // enclosing declaration's brace bookkeeping.
                        pending_class = false;
                        expect_class_name = false; // anonymous `class {`
                        decl_state_stack.push((
                            in_decl,
                            expect_decl_name,
                            decl_is_var,
                            decl_paren_depth,
                            decl_brace_depth,
                            decl_bracket_depth,
                            decl_object_pattern_depth,
                        ));
                        in_decl = false;
                        expect_decl_name = false;
                        decl_paren_depth = 0;
                        decl_brace_depth = 0;
                        decl_bracket_depth = 0;
                        decl_object_pattern_depth = 0;
                        let new_id = scopes.len();
                        scopes.push(Scope {
                            parent: Some(cur),
                            is_function: true,
                            decls: HashSet::new(),
                        });
                        stack.push(new_id);
                        ts[i] = new_id;
                        continue;
                    }
                    if let Some(fn_id) = pending_fn.take() {
                        // Entering a function scope: save current decl state.
                        // Inner `var` declarations will reset in_decl, and we
                        // need to restore it when leaving this function scope.
                        decl_state_stack.push((
                            in_decl,
                            expect_decl_name,
                            decl_is_var,
                            decl_paren_depth,
                            decl_brace_depth,
                            decl_bracket_depth,
                            decl_object_pattern_depth,
                        ));
                        // Reset decl state for the new function scope
                        in_decl = false;
                        expect_decl_name = false;
                        decl_paren_depth = 0;
                        decl_brace_depth = 0;
                        decl_bracket_depth = 0;
                        decl_object_pattern_depth = 0;
                        stack.push(fn_id);
                        ts[i] = fn_id;
                    } else {
                        if in_decl {
                            if expect_decl_name {
                                decl_object_pattern_depth += 1;
                            }
                            decl_brace_depth += 1;
                            continue;
                        }
                        let new_id = scopes.len();
                        scopes.push(Scope {
                            parent: Some(cur),
                            is_function: false,
                            decls: HashSet::new(),
                        });
                        stack.push(new_id);
                        ts[i] = new_id;
                    }
                }
                "}" => {
                    if in_params {
                        param_depth -= 1;
                        if param_depth == 0 {
                            in_params = false;
                            param_scope = None;
                        }
                        continue;
                    }
                    if in_decl && decl_brace_depth > 0 {
                        if decl_object_pattern_depth > 0 {
                            decl_object_pattern_depth -= 1;
                            if decl_object_pattern_depth == 0 {
                                expect_decl_name = false;
                            }
                        }
                        decl_brace_depth -= 1;
                        continue;
                    }
                    if in_decl {
                        // A block-closing `}` with no open decl braces ends
                        // the declaration implicitly (`{var na=1}` — ASI at
                        // block close). Leaving in_decl set made the next
                        // block's `{` get swallowed into decl_brace_depth,
                        // and its `}` then popped a real scope.
                        in_decl = false;
                        expect_decl_name = false;
                    }
                    if stack.len() > 1 {
                        let leaving_scope = stack.pop().unwrap();
                        // If leaving a function scope, restore saved decl state
                        if scopes[leaving_scope].is_function && !decl_state_stack.is_empty() {
                            let (
                                saved_in_decl,
                                saved_expect,
                                saved_is_var,
                                saved_paren,
                                saved_brace,
                                saved_bracket,
                                saved_object_pattern,
                            ) = decl_state_stack.pop().unwrap();
                            in_decl = saved_in_decl;
                            expect_decl_name = saved_expect;
                            decl_is_var = saved_is_var;
                            decl_paren_depth = saved_paren;
                            decl_brace_depth = saved_brace;
                            decl_bracket_depth = saved_bracket;
                            decl_object_pattern_depth = saved_object_pattern;
                        }
                    }
                    ts[i] = *stack.last().unwrap();
                }
                "[" => {
                    if in_params {
                        param_depth += 1;
                        continue;
                    }
                    if in_decl {
                        decl_bracket_depth += 1;
                    }
                }
                "]" => {
                    if in_params {
                        param_depth -= 1;
                        if param_depth == 0 {
                            in_params = false;
                            param_scope = None;
                        }
                        continue;
                    }
                    if in_decl && decl_bracket_depth > 0 {
                        decl_bracket_depth -= 1;
                    }
                }
                ";" => {
                    // Only end `in_decl` at the top level of the declaration.
                    // Semicolons inside object literals with function bodies
                    // (e.g., `var o = {f: function() { return 1; }}, x = 0;`)
                    // must NOT reset in_decl.
                    if decl_brace_depth == 0 && decl_paren_depth == 0 && decl_bracket_depth == 0 {
                        in_decl = false;
                        expect_decl_name = false;
                        decl_paren_depth = 0;
                        decl_bracket_depth = 0;
                        decl_object_pattern_depth = 0;
                    }
                }
                "," => {
                    // Re-enable decl name for: var a = 1, b = 2
                    // Only at the top level of the declaration (not inside
                    // parens, braces, object literals, or arrays).
                    if in_decl
                        && decl_paren_depth == 0
                        && decl_brace_depth == 0
                        && decl_bracket_depth == 0
                    {
                        expect_decl_name = true;
                    }
                }
                _ => {}
            }
        }
    }

    ScopeInfo {
        scopes,
        token_scope: ts,
    }
}

// === Rename Computation ===

fn compute_renames(
    source: &str,
    tokens: &[Tok],
    si: &ScopeInfo,
    mangle_root: bool,
) -> Vec<HashMap<String, String>> {
    let scope_count = si.scopes.len();
    let mut renames: Vec<HashMap<String, String>> = vec![HashMap::new(); scope_count];
    // Short names actually recorded per scope, kept parallel to `renames`
    // so descendant probes don't rebuild value sets.
    let mut recorded_shorts: Vec<HashSet<String>> = vec![HashSet::new(); scope_count];
    let physical_body_decls = collect_physical_function_body_decl_names(source, tokens, si);
    let physical_scopes =
        physical_function_scope_context(source, tokens, &si.token_scope, scope_count);

    // Pre-compute: for each scope, all ident names referenced in it and its
    // descendants. Borrowed &str throughout — materializing owned per-scope
    // skip sets was O(scopes x total_decls) string clones (68M on the antd
    // bundle, ~7s of a 16s build).
    let mut scope_refs: Vec<HashSet<&str>> = vec![HashSet::new(); scope_count];
    for (ti, tok) in tokens.iter().enumerate() {
        if tok.kind == TK::Ident {
            scope_refs[si.token_scope[ti]].insert(txt(source, tok));
        }
    }
    // Propagate child refs up to parent scopes (parents have smaller ids).
    for sid in (1..scope_count).rev() {
        if let Some(pid) = si.scopes[sid].parent {
            let child = std::mem::take(&mut scope_refs[sid]);
            for &name in &child {
                scope_refs[pid].insert(name);
            }
            scope_refs[sid] = child;
        }
    }
    let mut scope_decl_ref_counts: Vec<HashMap<&str, usize>> =
        vec![HashMap::new(); scope_count];
    for (ti, tok) in tokens.iter().enumerate() {
        if tok.kind != TK::Ident || should_skip_identifier_rename(source, tokens, ti) {
            continue;
        }
        let name = txt(source, tok);
        if let Some(scope_id) = resolve_decl_scope(name, si.token_scope[ti], &si.scopes) {
            *scope_decl_ref_counts[scope_id].entry(name).or_insert(0) += 1;
        }
    }

    // Process scopes in order (parents before children)
    for sid in 0..scope_count {
        let scope = &si.scopes[sid];
        if !scope.is_function {
            continue;
        }
        // Skip root scope unless explicitly asked to mangle it
        // (e.g., when source is a scope-hoisted IIFE bundle).
        if sid == 0 && !mangle_root {
            continue;
        }
        // Child scopes avoid every alias already assigned by ancestors. That is
        // slightly more conservative than JavaScript strictly requires, but it
        // keeps output correct when this lightweight parser underestimates a
        // function body's end inside complex declaration initializers.
        //
        // Instead of materializing the union as an owned set per scope, probe
        // candidates against the constituent sets directly: subtree refs
        // (covers free vars and referenced own decls), own decls + physical
        // body decls, and each ancestor's decls/physical decls/recorded
        // short names. Candidates are generated monotonically, so the total
        // probe count per scope is O(decls + rejected candidates).
        let ancestors =
            scope_and_physical_ancestor_ids(sid, &si.scopes, &physical_scopes.parent_by_scope);
        let candidate_taken = |cand: &str| -> bool {
            scope_refs[sid].contains(cand)
                || si.scopes[sid].decls.contains(cand)
                || physical_body_decls[sid].contains(cand)
                || ancestors.iter().any(|&aid| {
                    si.scopes[aid].decls.contains(cand)
                        || physical_body_decls[aid].contains(cand)
                        || recorded_shorts[aid].contains(cand)
                })
        };

        let mut decls: Vec<&String> = si.scopes[sid].decls.iter().collect();
        decls.sort_by(|a, b| {
            let a_count = scope_decl_ref_counts[sid]
                .get(a.as_str())
                .copied()
                .unwrap_or(0);
            let b_count = scope_decl_ref_counts[sid]
                .get(b.as_str())
                .copied()
                .unwrap_or(0);
            b_count.cmp(&a_count).then_with(|| a.cmp(b))
        });
        let mut counter = 0usize;
        let mut scope_renames = HashMap::new();
        let mut scope_shorts = HashSet::new();
        for name in decls {
            if is_reserved(name) {
                continue;
            }
            let short = loop {
                let cand = gen_name(counter);
                counter += 1;
                if !is_reserved(&cand) && !candidate_taken(&cand) {
                    break cand;
                }
            };
            if short.len() < name.len() {
                scope_shorts.insert(short.clone());
                scope_renames.insert(name.clone(), short);
            }
        }
        renames[sid] = scope_renames;
        recorded_shorts[sid] = scope_shorts;
    }

    renames
}

fn scope_and_physical_ancestor_ids(
    sid: usize,
    scopes: &[Scope],
    physical_parent_by_scope: &[Option<usize>],
) -> Vec<usize> {
    let mut ancestors = Vec::new();
    let mut seen = HashSet::new();
    let mut stack = Vec::new();
    stack.push(scopes[sid].parent);
    stack.push(physical_parent_by_scope.get(sid).copied().flatten());

    // `while let Some(Some(aid))` would stop on the FIRST `None` popped
    // (e.g. a scope with no physical parent), silently dropping the real
    // parent still on the stack. Scopes then missed their ancestors'
    // assigned short names and the generator reused them — two bindings
    // in one live chain both became `g` (`var g=g(10)["jsx"]`), breaking
    // every flattened React bundle at boot.
    while let Some(entry) = stack.pop() {
        let Some(aid) = entry else { continue };
        if aid >= scopes.len() || !seen.insert(aid) {
            continue;
        }
        ancestors.push(aid);
        stack.push(scopes[aid].parent);
        stack.push(physical_parent_by_scope.get(aid).copied().flatten());
    }

    ancestors
}

fn collect_physical_function_body_decl_names(
    source: &str,
    tokens: &[Tok],
    si: &ScopeInfo,
) -> Vec<HashSet<String>> {
    let mut decls: Vec<HashSet<String>> = vec![HashSet::new(); si.scopes.len()];

    for i in 0..tokens.len() {
        if !matches_ident(source, tokens, i, "function") {
            continue;
        }
        let Some(body_open) = function_body_open_token(source, tokens, i) else {
            continue;
        };
        let Some(body_close) = matching_punct_token(source, tokens, body_open, "{", "}") else {
            continue;
        };
        let scope_id = si.token_scope[body_open];
        if scope_id >= decls.len() {
            continue;
        }
        decls[scope_id].extend(collect_declared_names_in_physical_range(
            source,
            tokens,
            body_open + 1,
            body_close,
        ));
    }

    decls
}

fn function_body_open_token(source: &str, tokens: &[Tok], function_ti: usize) -> Option<usize> {
    if !matches_ident(source, tokens, function_ti, "function") {
        return None;
    }
    let mut ti = function_ti + 1;
    if ti < tokens.len() && tokens[ti].kind == TK::Ident {
        ti += 1;
    }
    if ti >= tokens.len() || !matches_punct(source, tokens, ti, "(") {
        return None;
    }
    let params_close = matching_punct_token(source, tokens, ti, "(", ")")?;
    let body_open = params_close + 1;
    if body_open < tokens.len() && matches_punct(source, tokens, body_open, "{") {
        Some(body_open)
    } else {
        None
    }
}

fn collect_declared_names_in_physical_range(
    source: &str,
    tokens: &[Tok],
    start_ti: usize,
    end_ti: usize,
) -> HashSet<String> {
    let mut names = HashSet::new();
    let mut ti = start_ti;

    while ti < end_ti {
        if matches_ident(source, tokens, ti, "function") {
            if ti + 1 < end_ti
                && tokens[ti + 1].kind == TK::Ident
                && is_function_declaration_context(source, tokens, ti)
            {
                names.insert(txt(source, &tokens[ti + 1]).to_string());
            }
            if let Some(body_open) = function_body_open_token(source, tokens, ti) {
                if let Some(body_close) = matching_punct_token(source, tokens, body_open, "{", "}")
                {
                    ti = body_close + 1;
                    continue;
                }
            }
        }

        if tokens[ti].kind == TK::Ident {
            let name = txt(source, &tokens[ti]);
            if matches!(name, "var" | "let" | "const") {
                let (next_ti, decl_names) =
                    collect_declared_names_after_keyword(source, tokens, ti + 1, end_ti);
                names.extend(decl_names);
                ti = next_ti;
                continue;
            }
        }

        ti += 1;
    }

    names
}

fn collect_declared_names_after_keyword(
    source: &str,
    tokens: &[Tok],
    start_ti: usize,
    end_ti: usize,
) -> (usize, HashSet<String>) {
    let mut names = HashSet::new();
    let mut ti = start_ti;
    let mut expect_decl_name = true;
    let mut paren_depth = 0i32;
    let mut brace_depth = 0i32;
    let mut bracket_depth = 0i32;
    let mut object_pattern_depth = 0i32;

    while ti < end_ti {
        if tokens[ti].kind == TK::Ident && expect_decl_name {
            let name = txt(source, &tokens[ti]);
            if !is_reserved(name) {
                names.insert(name.to_string());
            }
            expect_decl_name = false;
            ti += 1;
            continue;
        }

        if tokens[ti].kind == TK::Punct {
            match txt(source, &tokens[ti]) {
                "(" => paren_depth += 1,
                ")" => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                    }
                }
                "[" => bracket_depth += 1,
                "]" => {
                    if bracket_depth > 0 {
                        bracket_depth -= 1;
                    }
                }
                "{" => {
                    if expect_decl_name {
                        object_pattern_depth += 1;
                    }
                    brace_depth += 1;
                }
                "}" => {
                    if brace_depth > 0 {
                        if object_pattern_depth > 0 {
                            object_pattern_depth -= 1;
                            if object_pattern_depth == 0 {
                                expect_decl_name = false;
                            }
                        }
                        brace_depth -= 1;
                    } else {
                        return (ti, names);
                    }
                }
                "," if paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 => {
                    expect_decl_name = true;
                }
                ";" if paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 => {
                    return (ti + 1, names);
                }
                _ => {}
            }
        }

        ti += 1;
    }

    (ti, names)
}

fn apply_renames(
    source: &str,
    tokens: &[Tok],
    token_scope: &[usize],
    scopes: &[Scope],
    renames: &[HashMap<String, String>],
) -> String {
    let timing = std::env::var_os("JET_MANGLE_TIMING").is_some();
    let mut last = std::time::Instant::now();
    let mut lap = |stage: &str| {
        if timing {
            eprintln!("[mangle-timing]   apply_renames/{stage}: {:?}", last.elapsed());
            last = std::time::Instant::now();
        }
    };
    let mut repls: Vec<(usize, usize, String)> = Vec::new();
    let physical_scopes =
        physical_function_scope_context(source, tokens, token_scope, renames.len());
    lap("physical_ctx");

    // (start scope, name) -> declaring scope. Resolution walks the scope
    // chain per identifier token; bundles have hundreds of thousands of
    // tokens but few distinct (scope, name) pairs, so memoize the walk
    // (~1.3s -> negligible on the antd corpus bundle).
    let mut decl_scope_memo: HashMap<(usize, &str), Option<usize>> = HashMap::new();
    // The physical-scope fallback walk below is also (start scope, name) ->
    // resolved rename, and was the remaining unmemoized O(depth)-per-token cost
    // — deeply-nested react-dom drove apply_renames to ~97ms. Memoize it too.
    let mut physical_memo: HashMap<(usize, &str), Option<&str>> = HashMap::new();

    for (ti, tok) in tokens.iter().enumerate() {
        if tok.kind != TK::Ident {
            continue;
        }
        let name = txt(source, tok);
        if should_skip_identifier_rename(source, tokens, ti) {
            continue;
        }
        let mut replaced = false;
        let resolved_decl = *decl_scope_memo
            .entry((token_scope[ti], name))
            .or_insert_with(|| resolve_decl_scope(name, token_scope[ti], scopes));
        // Resolve through scope chain
        if let Some(mut sid) = resolved_decl {
            loop {
                if let Some(new_name) = renames[sid].get(name) {
                    repls.push((
                        tok.start,
                        tok.end,
                        rename_replacement(source, tokens, ti, name, new_name),
                    ));
                    replaced = true;
                    break;
                }
                if scopes[sid].decls.contains(name) {
                    break; // Declared here but not renamed
                }
                match scopes[sid].parent {
                    Some(p) => sid = p,
                    None => break, // Global
                }
            }
        }

        if replaced {
            continue;
        }

        let Some(physical_sid) = physical_scopes.scope_by_token[ti] else {
            continue;
        };
        if resolved_decl
            .map(|decl_sid| is_scope_descendant_or_same(decl_sid, physical_sid, scopes))
            .unwrap_or(false)
        {
            continue;
        }
        let resolved_physical = *physical_memo
            .entry((physical_sid, name))
            .or_insert_with(|| {
                let mut sid = Some(physical_sid);
                while let Some(candidate_sid) = sid {
                    if let Some(new_name) = renames[candidate_sid].get(name) {
                        return Some(new_name.as_str());
                    }
                    if scopes[candidate_sid].decls.contains(name) {
                        return None;
                    }
                    sid = physical_scopes.parent_by_scope[candidate_sid]
                        .or(scopes[candidate_sid].parent);
                }
                None
            });
        if let Some(new_name) = resolved_physical {
            repls.push((
                tok.start,
                tok.end,
                rename_replacement(source, tokens, ti, name, new_name),
            ));
        }
    }
    lap("token_loop");

    let out = apply_replacements(source, repls);
    lap("apply_replacements");
    out
}

struct PhysicalScopeContext {
    scope_by_token: Vec<Option<usize>>,
    parent_by_scope: Vec<Option<usize>>,
}

fn physical_function_scope_context(
    source: &str,
    tokens: &[Tok],
    token_scope: &[usize],
    scope_count: usize,
) -> PhysicalScopeContext {
    let mut intervals: Vec<(usize, usize, usize)> = Vec::new();

    for i in 0..tokens.len() {
        if !matches_ident(source, tokens, i, "function") {
            if matches_punct(source, tokens, i, "=>") {
                let Some((body_start, body_end)) = arrow_body_token_range(source, tokens, i) else {
                    continue;
                };
                let Some(&scope_id) = token_scope.get(body_start) else {
                    continue;
                };
                intervals.push((body_start, body_end, scope_id));
            }
            continue;
        }
        let Some(body_open) = function_body_open_token(source, tokens, i) else {
            continue;
        };
        let Some(body_close) = matching_punct_token(source, tokens, body_open, "{", "}") else {
            continue;
        };
        if body_open + 1 >= body_close {
            continue;
        }
        let Some(&scope_id) = token_scope.get(body_open) else {
            continue;
        };
        intervals.push((body_open + 1, body_close - 1, scope_id));
    }

    intervals.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)));
    let mut by_token = vec![None; tokens.len()];
    let mut parent_by_scope = vec![None; scope_count];
    let mut active: Vec<(usize, usize)> = Vec::new();
    let mut next_interval = 0usize;

    for (ti, slot) in by_token.iter_mut().enumerate() {
        while active
            .last()
            .map(|(end_ti, _)| *end_ti < ti)
            .unwrap_or(false)
        {
            active.pop();
        }
        while next_interval < intervals.len() && intervals[next_interval].0 <= ti {
            let scope_id = intervals[next_interval].2;
            if scope_id < parent_by_scope.len() && parent_by_scope[scope_id].is_none() {
                parent_by_scope[scope_id] = active.last().map(|(_, parent_scope)| *parent_scope);
            }
            active.push((intervals[next_interval].1, scope_id));
            next_interval += 1;
        }
        while active
            .last()
            .map(|(end_ti, _)| *end_ti < ti)
            .unwrap_or(false)
        {
            active.pop();
        }
        if let Some((_, scope_id)) = active.last() {
            *slot = Some(*scope_id);
        }
    }

    PhysicalScopeContext {
        scope_by_token: by_token,
        parent_by_scope,
    }
}

fn arrow_body_token_range(source: &str, tokens: &[Tok], arrow_ti: usize) -> Option<(usize, usize)> {
    let body_start = arrow_ti + 1;
    if body_start >= tokens.len() {
        return None;
    }
    if matches_punct(source, tokens, body_start, "{") {
        let body_end = matching_punct_token(source, tokens, body_start, "{", "}")?;
        return Some((body_start, body_end));
    }
    let body_end = arrow_expression_body_end(source, tokens, body_start)?;
    Some((body_start, body_end))
}

fn is_scope_descendant_or_same(scope_id: usize, maybe_ancestor: usize, scopes: &[Scope]) -> bool {
    let mut current = Some(scope_id);
    while let Some(sid) = current {
        if sid == maybe_ancestor {
            return true;
        }
        current = scopes[sid].parent;
    }
    false
}

fn repair_react_dom_event_helper_aliases(source: &str) -> String {
    let tokens = tokenize(source);
    if tokens.is_empty() {
        return source.to_string();
    }

    let mut repls = Vec::new();
    let mut i = 0usize;
    while i + 8 < tokens.len() {
        if !matches_ident(source, &tokens, i, "function") || tokens[i + 1].kind != TK::Ident {
            i += 1;
            continue;
        }
        let two_phase_alias = txt(source, &tokens[i + 1]).to_string();
        if !matches_punct(source, &tokens, i + 2, "(") {
            i += 1;
            continue;
        }
        let Some(params_close) = matching_punct_token(source, &tokens, i + 2, "(", ")") else {
            i += 1;
            continue;
        };
        let body_open = params_close + 1;
        if !matches_punct(source, &tokens, body_open, "{") {
            i += 1;
            continue;
        }
        let Some(body_close) = matching_punct_token(source, &tokens, body_open, "{", "}") else {
            i += 1;
            continue;
        };
        let body = &source[tokens[body_open].end..tokens[body_close].start];
        if !body.contains("\"Capture\"") && !body.contains("'Capture'") {
            i = body_close + 1;
            continue;
        }
        let Some(direct_alias) =
            first_call_ident_in_range(source, &tokens, body_open + 1, body_close)
        else {
            i = body_close + 1;
            continue;
        };
        if direct_alias == two_phase_alias {
            i = body_close + 1;
            continue;
        }
        if !nearby_function_decl_after(source, &tokens, body_close + 1, &direct_alias) {
            i = body_close + 1;
            continue;
        }
        let Some(wrapper_open) = nearest_unclosed_opening_punct_index(source, &tokens, i) else {
            i = body_close + 1;
            continue;
        };
        if txt(source, &tokens[wrapper_open]) != "{" {
            i = body_close + 1;
            continue;
        }
        let Some(wrapper_close) = matching_punct_token(source, &tokens, wrapper_open, "{", "}")
        else {
            i = body_close + 1;
            continue;
        };

        for ti in body_close + 1..wrapper_close {
            if tokens[ti].kind != TK::Ident {
                continue;
            }
            let name = txt(source, &tokens[ti]);
            let replacement = match name {
                "fa" => &two_phase_alias,
                "ha" => &direct_alias,
                _ => continue,
            };
            if replacement == name {
                continue;
            }
            if should_skip_identifier_rename(source, &tokens, ti) {
                continue;
            }
            if ti > 0 && matches_ident(source, &tokens, ti - 1, "function") {
                continue;
            }
            if ti + 1 >= tokens.len() || !matches_punct(source, &tokens, ti + 1, "(") {
                continue;
            }
            repls.push((tokens[ti].start, tokens[ti].end, replacement.clone()));
        }

        i = wrapper_close + 1;
    }

    if repls.is_empty() {
        source.to_string()
    } else {
        apply_replacements(source, repls)
    }
}

fn first_call_ident_in_range(
    source: &str,
    tokens: &[Tok],
    start_ti: usize,
    end_ti: usize,
) -> Option<String> {
    for ti in start_ti..end_ti {
        if tokens[ti].kind == TK::Ident
            && ti + 1 < end_ti
            && matches_punct(source, tokens, ti + 1, "(")
            && (ti == 0 || !matches_ident(source, tokens, ti - 1, "function"))
            && !should_skip_identifier_rename(source, tokens, ti)
        {
            return Some(txt(source, &tokens[ti]).to_string());
        }
    }
    None
}

fn nearby_function_decl_after(source: &str, tokens: &[Tok], start_ti: usize, name: &str) -> bool {
    let end_ti = (start_ti + 12).min(tokens.len());
    for ti in start_ti..end_ti.saturating_sub(2) {
        if matches_ident(source, tokens, ti, "function")
            && tokens[ti + 1].kind == TK::Ident
            && txt(source, &tokens[ti + 1]) == name
            && matches_punct(source, tokens, ti + 2, "(")
        {
            return true;
        }
    }
    false
}

fn should_skip_identifier_rename(source: &str, tokens: &[Tok], ti: usize) -> bool {
    // Skip property access (after .)
    if ti > 0 && tokens[ti - 1].kind == TK::Punct && txt(source, &tokens[ti - 1]) == "." {
        let after_spread = ti >= 3
            && tokens[ti - 2].kind == TK::Punct
            && tokens[ti - 3].kind == TK::Punct
            && txt(source, &tokens[ti - 2]) == "."
            && txt(source, &tokens[ti - 3]) == ".";
        if !after_spread {
            return true;
        }
    }
    // Skip object literal keys: { key: value }
    if ti + 1 < tokens.len()
        && tokens[ti + 1].kind == TK::Punct
        && txt(source, &tokens[ti + 1]) == ":"
        && ti > 0
        && tokens[ti - 1].kind == TK::Punct
    {
        let prev = txt(source, &tokens[ti - 1]);
        if prev == "{" || prev == "," {
            return true;
        }
    }
    // Skip object literal method keys: { generate(value) { ... } }.
    // Renaming these changes the public property name and breaks later
    // property reads such as `ClassNameGenerator.generate(...)`.
    if ti + 1 < tokens.len()
        && tokens[ti + 1].kind == TK::Punct
        && txt(source, &tokens[ti + 1]) == "("
        && ti > 0
        && tokens[ti - 1].kind == TK::Punct
        && matches!(txt(source, &tokens[ti - 1]), "{" | ",")
    {
        let has_method_body = matching_punct_token(source, tokens, ti + 1, "(", ")")
            .and_then(|params_close| tokens.get(params_close + 1))
            .map(|tok| tok.kind == TK::Punct && txt(source, tok) == "{")
            .unwrap_or(false);
        if has_method_body {
            if let Some(open_idx) = nearest_unclosed_opening_punct_index(source, tokens, ti) {
                if txt(source, &tokens[open_idx]) == "{"
                    && is_object_literal_opening(source, tokens, open_idx)
                {
                    return true;
                }
            }
        }
    }
    false
}

fn rename_replacement(
    source: &str,
    tokens: &[Tok],
    ti: usize,
    old_name: &str,
    new_name: &str,
) -> String {
    if is_shorthand_property_position(source, tokens, ti) {
        // Expand shorthand `{x}` -> `{x:y}` when the binding is renamed.
        // No space after the colon: minify_js already stripped object-key
        // colon-spaces, and re-introducing one here (1459 on the mui
        // bundle) is pure dead weight terser/esbuild never emit.
        format!("{}:{}", old_name, new_name)
    } else {
        new_name.to_string()
    }
}

fn is_shorthand_property_position(source: &str, tokens: &[Tok], ti: usize) -> bool {
    if ti == 0 || ti + 1 >= tokens.len() {
        return false;
    }
    if tokens[ti - 1].kind != TK::Punct || tokens[ti + 1].kind != TK::Punct {
        return false;
    }

    let prev = txt(source, &tokens[ti - 1]);
    let next = txt(source, &tokens[ti + 1]);
    if !matches!(prev, "{" | ",") || !matches!(next, "," | "}" | "=") {
        return false;
    }

    let Some(open_idx) = nearest_unclosed_opening_punct_index(source, tokens, ti) else {
        return false;
    };
    if txt(source, &tokens[open_idx]) != "{" {
        return false;
    }

    if next == "=" {
        return is_object_binding_pattern_opening(source, tokens, open_idx);
    }

    is_object_literal_opening(source, tokens, open_idx)
        || is_object_binding_pattern_opening(source, tokens, open_idx)
}

fn nearest_unclosed_opening_punct_index(source: &str, tokens: &[Tok], ti: usize) -> Option<usize> {
    let mut brace_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;

    for idx in (0..ti).rev() {
        let tok = &tokens[idx];
        if tok.kind != TK::Punct {
            continue;
        }
        match txt(source, tok) {
            "}" => brace_depth += 1,
            ")" => paren_depth += 1,
            "]" => bracket_depth += 1,
            "{" => {
                if brace_depth == 0 && paren_depth == 0 && bracket_depth == 0 {
                    return Some(idx);
                }
                brace_depth = brace_depth.saturating_sub(1);
            }
            "(" => {
                if brace_depth == 0 && paren_depth == 0 && bracket_depth == 0 {
                    return Some(idx);
                }
                paren_depth = paren_depth.saturating_sub(1);
            }
            "[" => {
                if brace_depth == 0 && paren_depth == 0 && bracket_depth == 0 {
                    return Some(idx);
                }
                bracket_depth = bracket_depth.saturating_sub(1);
            }
            _ => {}
        }
    }

    None
}

fn is_object_literal_opening(source: &str, tokens: &[Tok], open_idx: usize) -> bool {
    let Some(prev_idx) = tokens[..open_idx]
        .iter()
        .rposition(|tok| tok.kind != TK::Str || !txt(source, tok).is_empty())
    else {
        return true;
    };
    let prev = txt(source, &tokens[prev_idx]);
    match prev {
        "(" | "[" | "," | ":" | "=" | "?" | "!" => true,
        "return" | "yield" | "case" => true,
        _ => false,
    }
}

fn is_object_binding_pattern_opening(source: &str, tokens: &[Tok], open_idx: usize) -> bool {
    let Some(prev_idx) = tokens[..open_idx]
        .iter()
        .rposition(|tok| tok.kind != TK::Str || !txt(source, tok).is_empty())
    else {
        return false;
    };
    if matches!(txt(source, &tokens[prev_idx]), "var" | "let" | "const") {
        return true;
    }

    let Some(close_idx) = matching_punct_token(source, tokens, open_idx, "{", "}") else {
        return false;
    };
    let Some(next_idx) = tokens[close_idx + 1..]
        .iter()
        .position(|tok| tok.kind != TK::Str || !txt(source, tok).is_empty())
        .map(|offset| close_idx + 1 + offset)
    else {
        return false;
    };
    if tokens[next_idx].kind != TK::Punct || txt(source, &tokens[next_idx]) != "=" {
        return false;
    }

    let mut brace_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    for idx in (0..open_idx).rev() {
        let tok = &tokens[idx];
        if tok.kind == TK::Punct {
            match txt(source, tok) {
                "}" => brace_depth += 1,
                "{" => {
                    if brace_depth == 0 && paren_depth == 0 && bracket_depth == 0 {
                        break;
                    }
                    brace_depth = brace_depth.saturating_sub(1);
                }
                ")" => paren_depth += 1,
                "(" => {
                    if brace_depth == 0 && paren_depth == 0 && bracket_depth == 0 {
                        break;
                    }
                    paren_depth = paren_depth.saturating_sub(1);
                }
                "]" => bracket_depth += 1,
                "[" => {
                    if brace_depth == 0 && paren_depth == 0 && bracket_depth == 0 {
                        break;
                    }
                    bracket_depth = bracket_depth.saturating_sub(1);
                }
                ";" if brace_depth == 0 && paren_depth == 0 && bracket_depth == 0 => break,
                _ => {}
            }
            continue;
        }

        if brace_depth == 0 && paren_depth == 0 && bracket_depth == 0 {
            let raw = txt(source, tok);
            if matches!(raw, "var" | "let" | "const") {
                return true;
            }
        }
    }

    false
}

fn resolve_decl_scope(name: &str, start_scope: usize, scopes: &[Scope]) -> Option<usize> {
    let mut sid = start_scope;
    loop {
        if scopes[sid].decls.contains(name) {
            return Some(sid);
        }
        match scopes[sid].parent {
            Some(parent) => sid = parent,
            None => return None,
        }
    }
}

fn apply_replacements(source: &str, mut repls: Vec<(usize, usize, String)>) -> String {
    // Single forward pass. Reverse-order Vec::splice shifted the whole
    // tail per replacement — O(replacements x bundle size), ~1.6s on the
    // antd corpus bundle with hundreds of thousands of renames.
    repls.sort_by(|a, b| a.0.cmp(&b.0));
    let src = source.as_bytes();
    let mut out = Vec::with_capacity(src.len());
    let mut pos = 0usize;
    for (start, end, new_name) in &repls {
        if *start < pos {
            // Overlapping/duplicate replacement — first one wins.
            continue;
        }
        out.extend_from_slice(&src[pos..*start]);
        out.extend_from_slice(new_name.as_bytes());
        pos = *end;
    }
    out.extend_from_slice(&src[pos..]);
    String::from_utf8(out).unwrap_or_else(|_| source.to_string())
}

// === Name Generator ===

struct NameGen {
    counter: usize,
    skip: HashSet<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
impl NameGen {
    fn new(skip: &HashSet<String>) -> Self {
        let mut all_skip = skip.clone();
        for kw in RESERVED {
            all_skip.insert(kw.to_string());
        }
        Self {
            counter: 0,
            skip: all_skip,
        }
    }

    fn next_name(&mut self) -> String {
        loop {
            let name = gen_name(self.counter);
            self.counter += 1;
            if !self.skip.contains(&name) {
                self.skip.insert(name.clone());
                return name;
            }
        }
    }
}

fn gen_name(mut n: usize) -> String {
    const FIRST: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_$";
    const REST: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$";
    let mut name = vec![FIRST[n % FIRST.len()]];
    n /= FIRST.len();
    while n > 0 {
        n -= 1;
        name.push(REST[n % REST.len()]);
        n /= REST.len();
    }
    String::from_utf8(name).unwrap()
}

fn is_reserved(name: &str) -> bool {
    RESERVED.contains(&name)
}

const RESERVED: &[&str] = &[
    // Keywords
    "break",
    "case",
    "catch",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "finally",
    "for",
    "function",
    "if",
    "in",
    "instanceof",
    "new",
    "return",
    "switch",
    "this",
    "throw",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "class",
    "const",
    "enum",
    "export",
    "extends",
    "import",
    "super",
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
    "async",
    "await",
    "of",
    // Literals
    "null",
    "undefined",
    "true",
    "false",
    "NaN",
    "Infinity",
    // Module system — `require` kept reserved as it appears as a global in
    // many environments; `module` and `exports` are intentionally NOT
    // reserved so the mangler can rename them when they appear as function
    // parameters inside per-module IIFE wrappers (e.g., the scope-hoisted
    // `!function(module,exports,require){...}()` format), turning the
    // 7-byte `exports` parameter into a 1-byte short name.
    "require",
    "arguments",
    // Common globals (safety net)
    "window",
    "document",
    "console",
    "navigator",
    "location",
    "history",
    "setTimeout",
    "setInterval",
    "clearTimeout",
    "clearInterval",
    "requestAnimationFrame",
    "cancelAnimationFrame",
    "Promise",
    "Symbol",
    "Map",
    "Set",
    "WeakMap",
    "WeakSet",
    "Proxy",
    "Reflect",
    "Object",
    "Array",
    "String",
    "Number",
    "Boolean",
    "Function",
    "RegExp",
    "Date",
    "Error",
    "TypeError",
    "RangeError",
    "SyntaxError",
    "ReferenceError",
    "Math",
    "JSON",
    "parseInt",
    "parseFloat",
    "isNaN",
    "isFinite",
    "encodeURIComponent",
    "decodeURIComponent",
    "encodeURI",
    "decodeURI",
    "eval",
    "globalThis",
    "self",
    "queueMicrotask",
    "structuredClone",
    "fetch",
    "AbortController",
    "URL",
    "URLSearchParams",
    "TextEncoder",
    "TextDecoder",
    "Blob",
    "File",
    "FormData",
    "Headers",
    "Request",
    "Response",
    "ReadableStream",
    "WritableStream",
    "TransformStream",
    "Event",
    "EventTarget",
    "CustomEvent",
    "MutationObserver",
    "IntersectionObserver",
    "ResizeObserver",
    "performance",
    "crypto",
    "atob",
    "btoa",
    "Node",
    "Element",
    "HTMLElement",
    "SVGElement",
    "DocumentFragment",
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression: the require helper (`function _r`) and a flattened
    /// module local (`var _m0_jsx`) live in the same root scope; mangling
    /// must never give them the same short name. The broken output was
    /// `var g=g(10)["jsx"]` — a self-shadowing redeclaration that parses
    /// but calls jsx() as require at runtime (react-bench boot failure).
    #[test]
    fn test_require_helper_and_module_local_never_collide() {
        let src = r#"(function(){var _m0={exports:{}};var _m1={exports:{}};var _mods=[_m0,_m1];function _r(id){var m=_mods[id];return m?m.exports:{}}!function(module,exports,require){exports.jsx=function(t,c){return {t:t,c:c};};}(_m1,_m1.exports,_r);{var _m0e=_m0.exports;var _m0_jsx=_r(1)["jsx"];var _m0_jsxs=_r(1)["jsxs"];_m0_jsx("div",{children:_m0_jsxs("span",{})});}})();"#;
        let out = mangle_variables_with_root(src);
        // A `var X = X(...)` declaration is the collision signature: the
        // initializer must never call the binding it declares.
        for cap in out.split(';') {
            let cap = cap.trim();
            if let Some(rest) = cap.strip_prefix("var ") {
                if let Some((name, init)) = rest.split_once('=') {
                    let (name, init) = (name.trim(), init.trim());
                    assert!(
                        !init.starts_with(&format!("{name}(")),
                        "mangled output redeclares `{name}` from its own initializer \
                         (require-helper collision): {cap}\nfull: {out}"
                    );
                }
            }
        }
    }

    /// Regression: a colliding `var` declared inside a nested block hoists
    /// to the whole function — the repair must rename the references that
    /// live OUTSIDE that block too, or they resolve to the module wrapper
    /// at runtime (`TypeError: xa is not a function` on react-dom event
    /// dispatch in the tailwind corpus case).
    #[test]
    fn test_slot_collision_repair_covers_hoisted_refs_outside_block() {
        let src = r#"(function(){var xa={exports:{}};!function(module,exports){function dispatch(d){if(d){var na=1}else{na=2;var xa=getTargetInst(d)}xa&&xa(d);xa=d?win(d):window;return na}module.exports=dispatch}(xa,xa.exports);})()"#;
        let out = repair_generated_module_slot_local_decl_collisions(src);
        assert!(
            out.contains("var __jet_local_xa=getTargetInst"),
            "colliding var decl must be renamed: {out}"
        );
        assert!(
            out.contains("__jet_local_xa&&__jet_local_xa(d)"),
            "hoisted references outside the else-block must be renamed: {out}"
        );
        assert!(
            out.contains("}(xa,xa.exports)"),
            "wrapper call args must keep the slot name: {out}"
        );
        assert!(
            out.contains("var xa={exports:{}}"),
            "the slot declaration must keep its name: {out}"
        );
    }

    /// Regression: a class expression in a declaration initializer must not
    /// corrupt scope bookkeeping. Previously the class body was absorbed
    /// into decl_brace_depth, a `let` in a method reset that counter, the
    /// closing braces popped real scopes, and NameGen later minted a name
    /// colliding with a sibling `class` declaration (duplicate-declaration
    /// SyntaxError in styled-components bundles).
    #[test]
    fn test_class_expression_in_decl_keeps_scope_stack_intact() {
        let src = "(function(){var hoisted=1;const a=class{m(){let x=1;return x}},b=2;\
                   class _e{static go(){return 1}}var _m8_renameme=3;use(a,b,_e,_m8_renameme,hoisted);})()";
        let tokens = tokenize(src);
        let si = build_scopes(src, &tokens);
        // `b`, `_e`, and the trailing var must all live in the IIFE scope
        // (non-root); scope collapse put them at root.
        for name in ["b", "_e", "_m8_renameme"] {
            let scope = si
                .scopes
                .iter()
                .position(|s| s.decls.contains(name))
                .unwrap_or_else(|| panic!("{name} not declared anywhere"));
            assert_ne!(scope, 0, "{name} collapsed to the root scope");
        }
        // And a full mangle round-trip must not create duplicate
        // declarations with existing class names.
        let out = mangle_variables_with_root(src);
        let class_pos = out.find("class ").expect("class survives");
        let class_name: String = out[class_pos + 6..]
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
            .collect();
        let var_dup = format!("var {class_name},");
        assert!(
            !out.contains(&var_dup),
            "mangle minted a var colliding with class {class_name}: {out}"
        );
    }

    /// Regression: a `for (var k in obj) { ... var inner = ...; ... }` body
    /// must stay a real scope. The for-head `)` previously left `in_decl`
    /// set, the body `{` was swallowed into decl_brace_depth, the inner
    /// `var` reset that counter, and the orphaned `}` popped the enclosing
    /// scope — bindings after the loop then resolved against the wrong
    /// chain (react-dom updateProperties broke at runtime).
    #[test]
    fn test_for_in_var_body_does_not_pop_enclosing_scope() {
        let src = "function outer(alpha,beta){switch(alpha){case 1:\
                   for(var key in beta){var inner=beta[key];use(inner);}\
                   ;break;}for(var tail in alpha)use(alpha[tail],beta);\
                   return beta;}function other(gamma,beta){return gamma+beta;}";
        let tokens = tokenize(src);
        let si = build_scopes(src, &tokens);
        // `beta` after the for-in loops must still resolve to outer's param
        // scope (the same scope that declares `alpha`).
        let alpha_scope = si
            .scopes
            .iter()
            .position(|s| s.decls.contains("alpha"))
            .expect("outer param scope");
        let last_beta_ti = (0..tokens.len())
            .rev()
            .find(|&i| {
                tokens[i].kind == TK::Ident
                    && txt(src, &tokens[i]) == "beta"
                    && resolve_decl_scope("beta", si.token_scope[i], &si.scopes)
                        != Some(alpha_scope)
                    // ignore `other`'s own beta param/uses
                    && tokens[i].start < src.find("function other").unwrap()
            })
            .is_none();
        assert!(
            last_beta_ti,
            "a `beta` reference inside `outer` no longer resolves to outer's \
             param scope — for-in body popped an enclosing scope"
        );
    }

    /// Env-gated scope-debug probe for real bundles:
    /// JET_MANGLE_DEBUG_INPUT=<file> cargo test debug_real_bundle_mangle -- --nocapture
    /// Optional: JET_MANGLE_DEBUG_OFFSETS=123,456 prints the scope chain
    /// of the token covering each byte offset.
    #[test]
    fn debug_real_bundle_mangle() {
        let Ok(p) = std::env::var("JET_MANGLE_DEBUG_INPUT") else {
            return;
        };
        let src = std::fs::read_to_string(p).unwrap();
        let tokens = tokenize(&src);
        let si = build_scopes(&src, &tokens);
        if let Ok(offsets) = std::env::var("JET_MANGLE_DEBUG_OFFSETS") {
            for off in offsets.split(',').filter_map(|o| o.trim().parse::<usize>().ok()) {
                let Some(ti) = tokens.iter().position(|t| t.start <= off && off < t.end + 12)
                else {
                    continue;
                };
                // walk forward to the nearest ident token
                let ti = (ti..tokens.len().min(ti + 8))
                    .find(|&i| tokens[i].kind == TK::Ident)
                    .unwrap_or(ti);
                let tok = &tokens[ti];
                let mut chain = vec![si.token_scope[ti]];
                while let Some(p) = si.scopes[*chain.last().unwrap()].parent {
                    chain.push(p);
                }
                println!(
                    "offset {off}: token `{}` scope chain {:?}, decl_scope={:?}",
                    txt(&src, tok),
                    chain,
                    resolve_decl_scope(txt(&src, tok), si.token_scope[ti], &si.scopes)
                );
            }
        }
        if let Ok(sid_str) = std::env::var("JET_MANGLE_DEBUG_SCOPE_LAST_TOKEN") {
            if let Ok(target) = sid_str.parse::<usize>() {
                let last = (0..tokens.len()).rev().find(|&i| si.token_scope[i] == target);
                if let Some(ti) = last {
                    let tok = &tokens[ti];
                    let lo = tok.start.saturating_sub(200);
                    let hi = (tok.end + 120).min(src.len());
                    println!(
                        "scope {target} last token at byte {}..{} `{}`\ncontext: {}",
                        tok.start,
                        tok.end,
                        txt(&src, tok),
                        &src[lo..hi]
                    );
                }
            }
        }
        if let Ok(out_path) = std::env::var("JET_MANGLE_DEBUG_OUT") {
            let mangled = if std::env::var("JET_MANGLE_DEBUG_MODE").as_deref() == Ok("noroot") {
                mangle_variables(&src)
            } else {
                mangle_variables_with_root(&src)
            };
            std::fs::write(&out_path, &mangled).unwrap();
            println!(
                "wrote mangled output to {out_path}; tree_sitter_parses={}",
                crate::bundler::dce::js_parses_without_errors(&mangled)
            );
        }
        let probes = ["_r", "_m0_jsx", "_m0_jsxs", "_m0_React", "_m0e"];
        for (sid, scope) in si.scopes.iter().enumerate() {
            for n in probes {
                if scope.decls.contains(n) {
                    println!(
                        "decl {n} in scope {sid} is_function={} parent={:?}",
                        scope.is_function, scope.parent
                    );
                }
            }
        }
        let renames = compute_renames(&src, &tokens, &si, true);
        for (sid, m) in renames.iter().enumerate() {
            for n in probes {
                if let Some(s) = m.get(n) {
                    println!("rename scope={sid} {n} -> {s}");
                }
            }
        }
    }

    #[test]
    fn test_simple_var_mangling() {
        let src = "function f() { var longName = 1; return longName + 2; }";
        let out = mangle_variables(src);
        assert!(
            !out.contains("longName"),
            "should mangle longName, got: {}",
            out
        );
        assert!(
            out.contains("return a + 2") || out.contains("return a+2"),
            "should use short name, got: {}",
            out
        );
    }

    #[test]
    fn test_param_mangling() {
        let src = "function add(first, second) { return first + second; }";
        let out = mangle_variables(src);
        assert!(!out.contains("first"), "should mangle first, got: {}", out);
        assert!(
            !out.contains("second"),
            "should mangle second, got: {}",
            out
        );
    }

    #[test]
    fn test_mangling_does_not_collide_with_existing_short_param() {
        let src = "\"use strict\";function batchedUpdates(callback,a){return callback(a);}";
        let out = mangle_variables(src);
        assert!(
            !out.contains("function batchedUpdates(a,a)"),
            "mangler must not create duplicate parameter names, got: {}",
            out
        );
    }

    #[test]
    fn test_mangling_does_not_reuse_param_short_name_for_const() {
        let src = r#"function asyncMap(objArr, option, func, callback, source) {
            const firstFields = option.firstFields === true ? Object.keys(objArr) : option.firstFields || [];
            const keys = Object.keys(objArr);
            return firstFields.indexOf(keys[0]) + source;
        }"#;
        let out = mangle_variables(src);
        assert!(
            !out.contains("const e=") && !out.contains("const e ="),
            "const binding must not reuse the fifth parameter short name, got: {}",
            out
        );
        assert!(
            crate::bundler::dce::js_parses_without_errors(&out),
            "mangled output must remain valid JS, got: {}",
            out
        );
    }

    #[test]
    fn test_globals_preserved() {
        let src = "function f() { console.log(document.title); }";
        let out = mangle_variables(src);
        assert!(
            out.contains("console"),
            "console should be preserved, got: {}",
            out
        );
        assert!(
            out.contains("document"),
            "document should be preserved, got: {}",
            out
        );
    }

    #[test]
    fn test_property_access_preserved() {
        let src = "function f() { var obj = {}; obj.longProp = 1; return obj.longProp; }";
        let out = mangle_variables(src);
        assert!(
            out.contains("longProp"),
            "property should be preserved, got: {}",
            out
        );
        assert!(
            !out.contains("var obj"),
            "local var should be mangled, got: {}",
            out
        );
    }

    #[test]
    fn test_object_shorthand_key_preserved_when_value_is_mangled() {
        let src = "function f() { var longName = 1; return {longName}; }";
        let out = mangle_variables(src);
        assert!(
            out.contains("{longName:"),
            "object shorthand key must be preserved when value is renamed, got: {}",
            out
        );
        assert!(
            !out.contains("{longName: "),
            "no dead space after the expanded shorthand colon, got: {}",
            out
        );
        assert!(
            !out.contains("{longName}"),
            "object shorthand must be expanded during rename, got: {}",
            out
        );
    }

    #[test]
    fn test_multi_var_declarations_are_not_object_shorthand() {
        let src = r#"
function f(assign, createSyntheticEvent, event) {
  var EventInterface = {eventPhase: 0},
    SyntheticEvent = createSyntheticEvent(EventInterface),
    UIEventInterface = assign({}, EventInterface, {view: 0, detail: 0}),
    SyntheticUIEvent = createSyntheticEvent(UIEventInterface),
    lastMovementX,
    lastMovementY,
    lastMouseEvent,
    MouseEventInterface = assign({}, UIEventInterface, {
      movementX: function (event) {
        if ("movementX" in event) return event.movementX;
        event !== lastMouseEvent && (lastMovementX = event.screenX - lastMouseEvent.screenX);
        return lastMovementX;
      },
      movementY: function () {
        return lastMovementY;
      }
    });
  return MouseEventInterface;
}
"#;
        let out = mangle_variables(src);
        assert!(
            !out.contains("lastMovementX:"),
            "var declarator must not be expanded as an object shorthand key, got: {}",
            out
        );
        assert!(
            !out.contains("lastMovementY:"),
            "var declarator must not be expanded as an object shorthand key, got: {}",
            out
        );
        assert!(
            !out.contains("lastMouseEvent:"),
            "var declarator must not be expanded as an object shorthand key, got: {}",
            out
        );
        assert!(
            out.contains("movementX:"),
            "object literal property keys should still be preserved, got: {}",
            out
        );
    }

    #[test]
    fn test_module_level_not_mangled() {
        let src = "var topLevel = 1; function f() { var local = topLevel; }";
        let out = mangle_variables(src);
        assert!(
            out.contains("topLevel"),
            "module-level should not be mangled, got: {}",
            out
        );
    }

    #[test]
    fn test_iife_vars_mangled_with_root() {
        // Scope-hoisted bundle: outer IIFE wraps everything.
        // !function(module,exports,require){var workInProgress = null;...}(...)
        // workInProgress is declared inside a function scope and should be mangled.
        let src = "!function(module,exports,require){var workInProgress=null;var executionContext=0;workInProgress=executionContext}(_m0,_m0.exports,_r)";
        let out = mangle_variables(src);
        // These should already be mangled since !function creates a function scope
        assert!(
            !out.contains("workInProgress"),
            "workInProgress should be mangled in IIFE, got: {}",
            out
        );
    }

    #[test]
    fn test_outer_iife_vars_mangled_with_root() {
        // The outer IIFE of a scope-hoisted bundle
        let src = "(function(){var _m0={exports:{}};var longVarName=1;longVarName=2})()";
        let out = mangle_variables_with_root(src);
        assert!(
            !out.contains("longVarName"),
            "IIFE var should be mangled with root, got: {}",
            out
        );
    }

    #[test]
    fn test_iife_closure_vars_mangled() {
        // Real pattern: var declared in !function wrapper, referenced in nested functions.
        // The mangler should rename these since they're function-local (closure vars are ok to mangle
        // as long as ALL references are renamed consistently).
        let src = r#"!function(module,exports,require){var workInProgress=null;var executionContext=0;function performWork(){if(workInProgress!==null){executionContext=1}}function commitRoot(){workInProgress=null;executionContext=0}exports.performWork=performWork;exports.commitRoot=commitRoot}(_m0,_m0.exports,_r)"#;
        let out = mangle_variables(src);
        assert!(
            !out.contains("workInProgress"),
            "workInProgress should be mangled in !function IIFE, got: {}",
            out
        );
        assert!(
            !out.contains("executionContext"),
            "executionContext should be mangled, got: {}",
            out
        );
    }

    #[test]
    fn test_multi_var_with_object_literal() {
        // Bug: comma inside object literal was incorrectly re-triggering
        // expect_decl_name, causing subsequent var names to be missed.
        let src =
            r#"!function(){var config={x:1,y:2},longVarName=3;function f(){return longVarName}}()"#;
        let out = mangle_variables(src);
        assert!(
            !out.contains("longVarName"),
            "longVarName after object literal should be mangled, got: {}",
            out
        );
    }

    #[test]
    fn test_multi_var_with_object_containing_functions() {
        let src = r#"!function(){var dispatcher={getCacheForType:function(a){return a},cacheSignal:function(){return 1}},executionContext=0,workInProgressRoot=null;function f(){workInProgressRoot=1;executionContext=2}}()"#;
        let out = mangle_variables(src);
        assert!(
            !out.contains("executionContext"),
            "executionContext after object-with-functions should be mangled, got: {}",
            out
        );
        assert!(
            !out.contains("workInProgressRoot"),
            "workInProgressRoot should be mangled, got: {}",
            out
        );
    }

    #[test]
    fn test_multi_var_after_minify() {
        // Simulate the real pipeline: minify first, then mangle.
        // The minifier may alter token boundaries that confuse scope analysis.
        let raw = r#"!function(){
  var dispatcher = {
    getCacheForType: function (a) { return a; },
    cacheSignal: function () { return 1; }
  },
  executionContext = 0,
  workInProgressRoot = null;
  function f() { workInProgressRoot = 1; executionContext = 2; }
}()"#;
        let minified = crate::bundler::minify::minify_js(raw, &[]);
        let out = mangle_variables(&minified);
        assert!(
            !out.contains("executionContext"),
            "executionContext should be mangled after minify+mangle, got: {}",
            out
        );
        assert!(
            !out.contains("workInProgressRoot"),
            "workInProgressRoot should be mangled after minify+mangle, got: {}",
            out
        );
    }

    #[test]
    fn test_multi_var_with_ternary() {
        // React pattern: var PossiblyWeakMap = "function" === typeof WeakMap ? WeakMap : Map,
        //                    executionContext = 0, workInProgressRoot = null;
        let src = r#"!function(){var PossiblyWeakMap="function"===typeof WeakMap?WeakMap:Map,executionContext=0,workInProgressRoot=null;function f(){workInProgressRoot=1;executionContext=2}}()"#;
        let out = mangle_variables(src);
        assert!(
            !out.contains("executionContext"),
            "executionContext should be mangled, got: {}",
            out
        );
        assert!(
            !out.contains("workInProgressRoot"),
            "workInProgressRoot should be mangled, got: {}",
            out
        );
    }

    #[test]
    fn test_many_module_scope_vars_mangled() {
        // React DOM pattern: many long var names at !function top level,
        // all referenced in nested functions. All should be mangled.
        let src = r#"!function(a,b,c){var workInProgressRoot=null;var workInProgressRootRenderLanes=0;var executionContext=0;var workInProgressSuspendedReason=0;function performWork(){workInProgressRoot=1;workInProgressRootRenderLanes=2;executionContext=3}function check(){return workInProgressSuspendedReason}performWork();check()}(_m0,_m0.exports,_r)"#;
        let out = mangle_variables(src);
        assert!(
            !out.contains("workInProgressRoot"),
            "workInProgressRoot should be mangled, got: {}",
            out
        );
        assert!(
            !out.contains("workInProgressRootRenderLanes"),
            "workInProgressRootRenderLanes should be mangled, got: {}",
            out
        );
        assert!(
            !out.contains("executionContext"),
            "executionContext should be mangled, got: {}",
            out
        );
        assert!(
            !out.contains("workInProgressSuspendedReason"),
            "workInProgressSuspendedReason should be mangled, got: {}",
            out
        );
    }

    #[test]
    fn test_string_content_preserved() {
        let src = r#"function f() { var x = "longVariableName"; return x; }"#;
        let out = mangle_variables(src);
        assert!(
            out.contains("\"longVariableName\""),
            "string should be preserved, got: {}",
            out
        );
    }

    #[test]
    fn test_name_generator() {
        let skip = HashSet::new();
        let mut gen = NameGen::new(&skip);
        assert_eq!(gen.next_name(), "a");
        assert_eq!(gen.next_name(), "b");
    }

    #[test]
    fn test_gen_name_sequence() {
        assert_eq!(gen_name(0), "a");
        assert_eq!(gen_name(25), "z");
        assert_eq!(gen_name(26), "A");
        assert_eq!(gen_name(51), "Z");
        assert_eq!(gen_name(52), "_");
        assert_eq!(gen_name(53), "$");
    }

    #[test]
    fn test_reserved_skipped() {
        let mut skip = HashSet::new();
        skip.insert("a".to_string());
        let mut gen = NameGen::new(&skip);
        let first = gen.next_name();
        assert_ne!(first, "a", "should skip 'a'");
        assert_eq!(first, "b");
    }

    #[test]
    fn test_wrapper_function_mangling() {
        let src = "__jet__.define(2,function(require,module,exports){var longVarName=0;function foo(bar){var innerLong=bar;return innerLong+longVarName;}})";
        let out = mangle_variables(src);
        eprintln!("wrapper output: {}", out);
        assert!(
            !out.contains("longVarName"),
            "wrapper var should be mangled, got: {}",
            out
        );
        assert!(
            !out.contains("innerLong"),
            "inner var should be mangled, got: {}",
            out
        );
        assert!(out.contains("require"), "require preserved, got: {}", out);
        // `module` and `exports` are no longer reserved — they should be mangled
        // to short names when used as function parameters.
        // The function signature must not contain the original long names as params.
        assert!(
            !out.contains(",module,"),
            "module param should be mangled, got: {}",
            out
        );
        // `exports` as a param appears as `,exports)` in the original; it should be renamed.
        assert!(
            !out.contains(",exports)"),
            "exports param should be mangled, got: {}",
            out
        );
        assert!(
            !out.contains("module:") && !out.contains("exports:"),
            "function params must not be treated as object shorthand, got: {}",
            out
        );
    }

    #[test]
    fn test_scope_hoisted_module_mangling() {
        // Simulate a scope-hoisted Phase-1 module IIFE: exports/module are params
        // and should be mangled to short names.
        let src = "!function(module,exports,require){var workInProgress=null;exports.render=workInProgress;}(_m0,_m0.exports,_r)";
        let out = mangle_variables(src);
        eprintln!("scope-hoisted output: {}", out);
        // workInProgress is a local var — must be mangled
        assert!(
            !out.contains("workInProgress"),
            "workInProgress should be mangled, got: {}",
            out
        );
        // `exports` as a function parameter must be renamed — the signature
        // `(module,exports,require)` must be gone. Note that `_m0.exports`
        // (property access) is NOT renamed, so we check the param list, not
        // the whole output.
        assert!(
            !out.contains(",exports,") && !out.contains("(exports,") && !out.contains(",exports)"),
            "exports param should be mangled (only .exports property access remains), got: {}",
            out
        );
        // module is a parameter — must be mangled (no longer appears as standalone `module,`)
        assert!(
            !out.contains("(module,") && !out.contains(",module,") && !out.contains(",module)"),
            "module param should be mangled, got: {}",
            out
        );
        assert!(
            !out.contains("module:") && !out.contains("exports:"),
            "function params must not be treated as object shorthand, got: {}",
            out
        );
        // require is still reserved — must NOT be mangled
        assert!(
            out.contains("require"),
            "require should be preserved, got: {}",
            out
        );
        // The property access `.exports` (in the argument `_m0.exports`) must be preserved
        assert!(
            out.contains(".exports"),
            ".exports property access preserved, got: {}",
            out
        );
    }

    #[test]
    fn test_root_scope_mangle_order_is_deterministic() {
        let src = r#"(function(){var zebra={exports:{}},alpha={exports:{}},middle={exports:{}};function read(which){switch(which){case 0:return zebra.exports;case 1:return alpha.exports;default:return middle.exports}}return read(0)})()"#;
        let first = mangle_variables_with_root(src);
        for _ in 0..32 {
            assert_eq!(mangle_variables_with_root(src), first);
        }
    }

    #[test]
    fn test_module_slot_refs_inside_flattened_blocks_are_mangled() {
        let src = r#"(function(){var _m0={exports:{}},_m1={exports:{}},_m2={exports:{}};var _mods=[_m0,_m1,_m2];function _r(id){var m=_mods[id];return m?m.exports:{}}{var _m1e=_m1.exports;Object.defineProperty(_m1.exports,"__esModule",{value:true});_m1.exports["default"]=_r(2)["default"];}{var _m2e=_m2.exports;_m2.exports["default"]=function run(){return "ok";};}})()"#;
        for out in [mangle_variables(src), mangle_variables_with_root(src)] {
            assert!(
                !out.contains("_m1.exports") && !out.contains("_m2.exports"),
                "module slot references inside flattened blocks must be renamed with declarations, got: {}",
                out
            );
        }
    }

    #[test]
    fn test_large_module_slot_refs_inside_flattened_blocks_are_mangled() {
        let mut src = String::from("(function(){");
        for idx in 0..740 {
            src.push_str(&format!("var _m{idx}={{exports:{{}}}};"));
        }
        src.push_str("var _mods=[");
        for idx in 0..740 {
            if idx > 0 {
                src.push(',');
            }
            src.push_str(&format!("_m{idx}"));
        }
        src.push_str("];function _r(id){var m=_mods[id];return m?m.exports:{}}");
        src.push_str(r#"{var _m737e=_m737.exports;Object.defineProperty(_m737.exports,"__esModule",{value:true});_m737.exports["default"]=_r(738)["default"];}"#);
        src.push_str(
            r#"{var _m738e=_m738.exports;_m738.exports["default"]=function run(){return "ok";};}"#,
        );
        src.push_str("})()");

        for out in [mangle_variables(&src), mangle_variables_with_root(&src)] {
            assert!(
                !out.contains("_m737.exports") && !out.contains("_m738.exports"),
                "large module slot references must be renamed with declarations, got stale module slot in: {}",
                out
            );
        }
    }

    #[test]
    fn test_module_slot_refs_after_arrow_returning_object_methods_are_mangled() {
        let src = r#"(function(){var _m736={exports:{}},_m737={exports:{}},_m738={exports:{}};var _mods=[_m736,_m737,_m738];function _r(id){var m=_mods[id];return m?m.exports:{}}{var _m738e=_m738.exports;Object.defineProperty(_m738.exports,"__esModule",{value:true});const _m738_defaultGenerator=componentName=>componentName;const _m738_createClassNameGenerator=()=>{let generate=_m738_defaultGenerator;return{configure(generator){generate=generator;},generate(componentName){return generate(componentName);},reset(){generate=_m738_defaultGenerator;}};};const _m738_ClassNameGenerator=_m738_createClassNameGenerator();_m738.exports["default"]=_m738_ClassNameGenerator;};{var _m737e=_m737.exports;Object.defineProperty(_m737.exports,"__esModule",{value:true});_m737.exports["default"]=_r(738)["default"];};{var _m736e=_m736.exports;Object.defineProperty(_m736.exports,"__esModule",{value:true});var _m736_ClassNameGenerator=_r(737)["default"]||_r(737);_m736.exports["default"]=_m736_ClassNameGenerator;}})()"#;
        let out = mangle_variables(src);

        for stale in [
            "_m737.exports",
            "_m737e",
            "_m736.exports",
            "_m736e",
            "_m736_ClassNameGenerator",
        ] {
            assert!(
                !out.contains(stale),
                "identifier `{stale}` must be renamed after previous arrow/object-method module block, got: {}",
                out
            );
        }
    }

    #[test]
    fn test_arrow_block_multi_var_declarations_are_not_object_shorthand() {
        let src = r#"(({theme,ownerState})=>{var _theme$transitions$create=theme.transitions.create,_theme$transitions2=theme.transitions,_theme$typography=theme.typography;return _theme$transitions$create(_theme$transitions2.duration,_theme$typography.fontSize);})({theme,ownerState});"#;
        let out = mangle_variables(src);

        assert!(
            !out.contains("_theme$transitions$create:"),
            "var declaration inside arrow block must not be rewritten as object shorthand, got: {}",
            out
        );
    }

    #[test]
    fn test_arrow_parameter_tokens_are_renamed_with_body_references() {
        let src = r#"(function(){const list=[{id:1}];const toggle=(id)=>{return list.map((item)=>item.id===id?{...item,done:!item.done}:item);};return toggle(1);})()"#;
        let out = mangle_variables_with_root(src);

        assert!(
            !out.contains("(id)=>"),
            "arrow parameter token must be renamed with its scope, got: {}",
            out
        );
        assert!(
            !out.contains("===id"),
            "arrow body reference must be renamed with the parameter, got: {}",
            out
        );
        assert!(
            !out.contains("item"),
            "nested expression-body arrow parameter should be scoped and renamed, got: {}",
            out
        );
        assert!(
            crate::bundler::dce::js_parses_without_errors(&out),
            "mangled output must remain valid JS: {}",
            out
        );
    }

    #[test]
    fn test_large_module_slot_refs_inside_retained_wrapper_calls_are_mangled() {
        let mut src = String::from("(function(){");
        for idx in 0..740 {
            src.push_str(&format!("var _m{idx}={{exports:{{}}}};"));
        }
        src.push_str("var _mods=[");
        for idx in 0..740 {
            if idx > 0 {
                src.push(',');
            }
            src.push_str(&format!("_m{idx}"));
        }
        src.push_str("];function _r(id){var m=_mods[id];return m?m.exports:{}}");
        src.push_str(r#"!function(module,exports,require){exports["default"]=require(738)["default"];}(_m737,_m737.exports,_r);"#);
        src.push_str(r#"!function(module,exports,require){exports["default"]=function run(){return "ok";};}(_m738,_m738.exports,_r);"#);
        src.push_str("})()");

        for out in [mangle_variables(&src), mangle_variables_with_root(&src)] {
            assert!(
                !out.contains("_m737") && !out.contains("_m738"),
                "retained wrapper call module slots must be renamed with declarations, got stale module slot in: {}",
                out
            );
        }
    }

    #[test]
    fn test_low_module_slot_refs_inside_retained_wrapper_calls_are_mangled() {
        let src = r#"(function(){var _m0={exports:{}},_m1={exports:{}},_m2={exports:{}};var _mods=[_m0,_m1,_m2];function _r(id){var m=_mods[id];return m?m.exports:{}}!function(module,exports,require){for(var key in exports){if(key){exports[key]=exports[key];}}exports.createRoot=function(){};}(_m2,_m2.exports,_r);!function(module,exports,require){module.exports=require(2);}(_m1,_m1.exports,_r);{var _m0e=_m0.exports;var ReactDOM=_r(1)["default"]||_r(1);ReactDOM.createRoot();}})()"#;

        for out in [mangle_variables(src), mangle_variables_with_root(src)] {
            assert!(
                !out.contains("_m2") && !out.contains("_m1") && !out.contains("_m0"),
                "low retained wrapper call module slots must be renamed with declarations, got stale module slot in: {}",
                out
            );
        }
    }

    #[test]
    fn test_generated_module_slot_repair_updates_stale_retained_wrapper_call_args() {
        let src = r#"(function(){'use strict';var o={exports:{}};var q={exports:{}};var z={exports:{}};var fa=[o,q,z];function _r(id){var m=fa[id];return m?m.exports:{}}!function(module,exports,require){exports.createRoot=function(){};}(_m2,_m2.exports,_r);!function(module,exports,require){var _m2={exports:{local:true}};module.exports=_m2.exports;}(_m1,_m1.exports,_r);})()"#;
        let out = repair_generated_module_slot_references(src);

        assert!(
            out.contains("}(z,z.exports,_r);"),
            "stale generated module slot call args should be repaired, got: {}",
            out
        );
        assert!(
            out.contains("var _m2={exports:{local:true}}"),
            "local declarations that happen to use the generated slot shape must be preserved, got: {}",
            out
        );
        assert!(
            out.contains("module.exports=_m2.exports"),
            "local references must resolve to their local declaration, got: {}",
            out
        );
    }

    #[test]
    fn test_generated_module_slot_local_decl_collision_repair_preserves_slot() {
        let src = r#"(function(){var Vl={exports:{}};{var local=1;var Vl=getMargin();var cfg={m:{style:Vl}};}Object.defineProperty(Vl.exports,"__esModule",{value:true});})()"#;
        let out = repair_generated_module_slot_local_decl_collisions(src);

        assert!(
            out.contains("var __jet_local_Vl=getMargin()"),
            "local collision should be renamed, got: {}",
            out
        );
        assert!(
            out.contains("style:__jet_local_Vl"),
            "local references in the block should be renamed, got: {}",
            out
        );
        assert!(
            out.contains("Object.defineProperty(Vl.exports"),
            "module slot reference should stay intact, got: {}",
            out
        );
    }

    #[test]
    fn test_generated_module_slot_local_decl_collision_repair_updates_hoisted_var_refs() {
        let src = r#"(function(){var _b={exports:{}};!function(m,require){function push(anim){_b={name:anim.name,next:_b};return _b.name};var _b;function serialize(){_b=undefined;push({name:"x"});return{name:"x",next:_b}};m.exports.serializeStyles=serialize}(kl,j);Object.defineProperty(_b.exports,"__esModule",{value:true});})()"#;
        let out = repair_generated_module_slot_local_decl_collisions(src);

        assert!(
            out.contains("var __jet_local__b"),
            "hoisted local collision should be renamed, got: {}",
            out
        );
        assert!(
            out.contains("__jet_local__b={name:anim.name,next:__jet_local__b}"),
            "references before the var declaration should be renamed, got: {}",
            out
        );
        assert!(
            out.contains("__jet_local__b=undefined"),
            "references after the var declaration should be renamed, got: {}",
            out
        );
        assert!(
            out.contains("Object.defineProperty(_b.exports"),
            "module slot reference should stay intact, got: {}",
            out
        );
    }

    #[test]
    fn test_labeled_block_function_decl_refs_are_mangled() {
        let src = r#"(function(){function advanceTimers(now){return now};function flushWork(){a:{b:{advanceTimers(1);break a}}}})()"#;
        let out = mangle_variables_with_root(src);

        assert!(
            !out.contains("advanceTimers(1)"),
            "function declaration references inside labels must be renamed, got: {}",
            out
        );
    }

    #[test]
    fn test_react_dom_shared_internals_multi_var_refs_are_mangled() {
        let src = r#"!function(React,ReactDOM,exports){var isArrayImpl=Array.isArray,ReactSharedInternals=React.__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE,ReactDOMSharedInternals=ReactDOM.__DOM_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE,sharedNotPendingObject={pending:!1,data:null,method:null,action:null},valueStack=[],index=-1;var previousDispatcher=ReactDOMSharedInternals.d;ReactDOMSharedInternals.d={f:function(){return previousDispatcher.f();}};var context={_currentValue:sharedNotPendingObject,_currentValue2:sharedNotPendingObject};exports.context=context;}(React,ReactDOM,exports)"#;
        let out = mangle_variables(src);

        for stale in [
            "ReactSharedInternals",
            "ReactDOMSharedInternals",
            "sharedNotPendingObject",
            "previousDispatcher",
        ] {
            assert!(
                !out.contains(stale),
                "multi-var binding `{stale}` must be renamed with all references, got: {}",
                out
            );
        }
        assert!(
            crate::bundler::dce::js_parses_without_errors(&out),
            "mangled output must remain valid JS: {}",
            out
        );
    }

    #[test]
    fn test_react_symbol_repair_uses_nearest_matching_symbol_alias() {
        let src = r#"(function(){var j=Symbol.for("react.context");function first(){return j;}!function(){var Z=Symbol.for("react.context");var h={$$typeof:REACT_CONTEXT_TYPE};return h;}();})()"#;
        let out = repair_react_symbol_constant_references(src);

        assert!(
            out.contains("$$typeof:Z"),
            "stale React symbol constant should use nearest matching alias, got: {}",
            out
        );
        assert!(
            !out.contains("REACT_CONTEXT_TYPE"),
            "stale React symbol name should be repaired, got: {}",
            out
        );
        assert!(
            out.contains("return j"),
            "earlier module alias must not be rewritten, got: {}",
            out
        );
    }

    #[test]
    fn test_react_dom_client_stale_alias_repair() {
        let src = r#"!function(vh,Ce,require){"use strict";var xa=require(3),la=require(12),ma=require(5);function Ve(a){var c="https://react.dev/errors/"+a;return c;};var Lb=Object.assign,oa=la.__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE,Uk={pending:!1};function fm(d,o,g,i){for(var a in g)(k=g[a]),(j=g[a]);for(var Oi in Vg)(k=Vg[Oi]),Vg.hasOwnProperty(Oi)&&null!=k&&!nextProps.hasOwnProperty(Oi)&&Lk(domElement,tag,Oi,null,nextProps,k);for(f in nextProps)(k=nextProps[f]),(propKey=Vg[f]),!nextProps.hasOwnProperty(f)||k===propKey||(null==k&&null==propKey)||Lk(domElement,tag,f,k,nextProps,propKey);};var lb=React.version;if("19.2.7"!==lb) throw Error(formatProdErrorMessage(527,lb,"19.2.7"));var fb={currentDispatcherRef:ReactSharedInternals};function ac(a){return assign({},a);};exports.createRoot=function(a){if(!a) throw Error(formatProdErrorMessage(299));return ac({});};}(z,z.exports,_r)"#;
        let out = repair_react_dom_client_stale_references(src);

        for stale in [
            "formatProdErrorMessage",
            "ReactSharedInternals",
            "React.version",
            "exports.createRoot",
            "assign(",
            "Vg",
            "nextProps",
            "domElement",
            "tag,",
            "propKey",
        ] {
            assert!(
                !out.contains(stale),
                "stale React DOM client identifier `{stale}` must be repaired, got: {}",
                out
            );
        }
        for expected in [
            "Error(Ve(527,lb,\"19.2.7\"))",
            "currentDispatcherRef:oa",
            "Ce.createRoot",
            "return Lb({},a)",
            "for(var Oi in g)",
            "!i.hasOwnProperty(Oi)",
            "Lk(d,o,Oi,null,i,k)",
            "(j=g[f])",
        ] {
            assert!(
                out.contains(expected),
                "expected repaired fragment `{expected}`, got: {}",
                out
            );
        }
    }

    #[test]
    fn test_react_dom_update_properties_stale_last_props_alias_repair() {
        let src = r#"function go(i,fn,y,H){for(var wi in y)(T=y[wi]),y.hasOwnProperty(wi)&&void 0!==T&&!H.hasOwnProperty(wi)&&Km(i,fn,wi,void 0,H,T);for(b in H)(T=H[b]),(S=y[b]),!H.hasOwnProperty(b)||T===S||(void 0===T&&void 0===S)||Km(i,fn,b,T,H,S);for(var Mk in Ui)(T=Ui[Mk]),Ui.hasOwnProperty(Mk)&&null!=T&&!nextProps.hasOwnProperty(Mk)&&Jm(domElement,tag,Mk,null,nextProps,T);for(x in nextProps)(T=nextProps[x]),(propKey=Ui[x]),!nextProps.hasOwnProperty(x)||T===propKey||(null==T&&null==propKey)||Jm(domElement,tag,x,T,nextProps,propKey);};"#;
        let out = repair_react_dom_client_stale_references(src);

        for stale in ["Ui", "nextProps", "domElement", "tag,", "propKey"] {
            assert!(
                !out.contains(stale),
                "stale React DOM updateProperties identifier `{stale}` must be repaired, got: {}",
                out
            );
        }
        for expected in [
            "for(var Mk in y)",
            "!H.hasOwnProperty(Mk)",
            "Jm(i,fn,Mk,null,H,T)",
            "(S=y[x])",
            "Jm(i,fn,x,T,H,S)",
        ] {
            assert!(
                out.contains(expected),
                "expected repaired fragment `{expected}`, got: {}",
                out
            );
        }
    }

    #[test]
    fn test_react_dom_root_constructor_alias_does_not_collide_with_var() {
        let src = r#"
function clientModule(assign, createSyntheticEvent, EventInterface) {
  "use strict";
  var lastMouseEvent,
    lastMovementX = 0,
    lastMovementY = 0,
    MouseEventInterface = assign({}, EventInterface, {
      screenX: 0,
      movementX: function (event) {
        if ("movementX" in event) return event.movementX;
        event !== lastMouseEvent &&
          (lastMovementX = event.screenX - lastMouseEvent.screenX);
        return lastMovementX;
      },
      movementY: function (event) {
        return "movementY" in event ? event.movementY : lastMovementY;
      }
    }),
    SyntheticMouseEvent = createSyntheticEvent(MouseEventInterface),
    AnimationEventInterface = assign({}, EventInterface, {
      animationName: 0,
      elapsedTime: 0,
      pseudoElement: 0
    }),
    SyntheticAnimationEvent = createSyntheticEvent(AnimationEventInterface),
    ClipboardEventInterface = assign({}, EventInterface, {clipboardData: 0});
  function ReactDOMRoot(internalRoot) {
    this._internalRoot = internalRoot;
  }
  ReactDOMHydrationRoot.prototype.render = ReactDOMRoot.prototype.render =
    function (children) {
      return this._internalRoot.current + children;
    };
  ReactDOMHydrationRoot.prototype.unmount = ReactDOMRoot.prototype.unmount =
    function () {
      this._internalRoot = null;
    };
  function ReactDOMHydrationRoot(internalRoot) {
    this._internalRoot = internalRoot;
  }
  ReactDOMHydrationRoot.prototype.unstable_scheduleHydration = function (target) {
    return target;
  };
  return [
    SyntheticAnimationEvent,
    ClipboardEventInterface,
    new ReactDOMRoot({current: 1}),
    new ReactDOMHydrationRoot({current: 2})
  ];
}
"#;
        let out = mangle_variables(src);
        let hyd_alias = alias_before_prototype_access(&out, ".prototype.render")
            .expect("hydration root alias should be present");
        assert!(
            !has_var_declarator_named(&out, hyd_alias),
            "constructor alias `{hyd_alias}` must not collide with a var declarator, got: {out}"
        );
    }

    #[test]
    fn test_child_scope_aliases_do_not_reuse_ancestor_aliases() {
        let src = r#"
function outer(parentLongName) {
  function inner(childLongName) {
    return childLongName;
  }
  return parentLongName + inner(1);
}
"#;
        let tokens = tokenize(src);
        let si = build_scopes(src, &tokens);
        let renames = compute_renames(src, &tokens, &si, false);
        let child_scope = si
            .scopes
            .iter()
            .position(|scope| scope.decls.contains("childLongName"))
            .expect("child scope should declare childLongName");
        let child_alias = renames[child_scope]
            .get("childLongName")
            .expect("childLongName should be shortened");

        let mut ancestor_aliases = HashSet::new();
        let mut ancestor = si.scopes[child_scope].parent;
        while let Some(scope_id) = ancestor {
            ancestor_aliases.extend(renames[scope_id].values().cloned());
            ancestor = si.scopes[scope_id].parent;
        }

        assert!(
            !ancestor_aliases.contains(child_alias),
            "child alias `{child_alias}` must avoid ancestor aliases {ancestor_aliases:?}"
        );
    }

    #[test]
    fn test_physical_function_body_decl_names_are_reserved_when_scope_model_misses_them() {
        let src = r#"
function wrapper(scheduler) {
  var a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,A,B;
  var eventDispatchMapLongName = {};
  return eventDispatchMapLongName;
}
"#;
        let tokens = tokenize(src);
        let mut si = build_scopes(src, &tokens);
        let wrapper_scope = si
            .scopes
            .iter()
            .position(|scope| scope.decls.contains("eventDispatchMapLongName"))
            .expect("wrapper scope should declare eventDispatchMapLongName");
        si.scopes[wrapper_scope].decls.remove("B");

        let renames = compute_renames(src, &tokens, &si, false);
        assert_ne!(
            renames[wrapper_scope].get("eventDispatchMapLongName"),
            Some(&"B".to_string()),
            "physical function body decl `B` must stay reserved even if the lightweight scope model misses it"
        );
    }

    #[test]
    fn test_physical_function_body_fallback_repairs_stale_reference_scope() {
        let src = r#"
function wrapper() {
  var eventDispatchMapLongName = {};
  return eventDispatchMapLongName;
}
"#;
        let tokens = tokenize(src);
        let si = build_scopes(src, &tokens);
        let mut token_scope = si.token_scope.clone();
        let wrapper_scope = si
            .scopes
            .iter()
            .position(|scope| scope.decls.contains("eventDispatchMapLongName"))
            .expect("wrapper scope should declare eventDispatchMapLongName");
        let return_ref_ti = tokens
            .iter()
            .enumerate()
            .filter(|(_, tok)| tok.kind == TK::Ident && txt(src, tok) == "eventDispatchMapLongName")
            .nth(1)
            .map(|(ti, _)| ti)
            .expect("return reference should be present");
        token_scope[return_ref_ti] = 0;

        let renames = compute_renames(src, &tokens, &si, false);
        let alias = renames[wrapper_scope]
            .get("eventDispatchMapLongName")
            .expect("long wrapper name should be renamed")
            .clone();
        let out = apply_renames(src, &tokens, &token_scope, &si.scopes, &renames);

        assert!(
            out.contains(&format!("return {alias}")),
            "stale reference should be repaired to wrapper alias `{alias}`, got: {out}"
        );
        assert!(
            !out.contains("return eventDispatchMapLongName"),
            "stale original return reference should not survive, got: {out}"
        );
    }

    #[test]
    fn test_physical_function_body_fallback_repairs_outer_free_reference_scope() {
        let src = r#"
function wrapper() {
  var outerHashFunctionLongName = function(value) { return value; };
  function serializeStylesLongName(value) {
    return outerHashFunctionLongName(value);
  }
  return serializeStylesLongName("x");
}
"#;
        let tokens = tokenize(src);
        let si = build_scopes(src, &tokens);
        let mut token_scope = si.token_scope.clone();
        let wrapper_scope = si
            .scopes
            .iter()
            .position(|scope| scope.decls.contains("outerHashFunctionLongName"))
            .expect("wrapper scope should declare outerHashFunctionLongName");
        let inner_ref_ti = tokens
            .iter()
            .enumerate()
            .filter(|(_, tok)| {
                tok.kind == TK::Ident && txt(src, tok) == "outerHashFunctionLongName"
            })
            .nth(1)
            .map(|(ti, _)| ti)
            .expect("inner free reference should be present");
        token_scope[inner_ref_ti] = 0;

        let renames = compute_renames(src, &tokens, &si, false);
        let alias = renames[wrapper_scope]
            .get("outerHashFunctionLongName")
            .expect("outer helper should be renamed")
            .clone();
        let out = apply_renames(src, &tokens, &token_scope, &si.scopes, &renames);

        assert!(
            out.contains(&format!("return {alias}(")),
            "inner stale free reference should be repaired to outer alias `{alias}`, got: {out}"
        );
        assert!(
            !out.contains("return outerHashFunctionLongName(value)"),
            "stale outer helper reference should not survive, got: {out}"
        );
    }

    #[test]
    fn test_child_scope_aliases_avoid_physical_parent_aliases_when_scope_parent_is_wrong() {
        let src = r#"
function wrapper() {
  var outerHashFunctionLongName = function(value) { return value; };
  function serializeStylesLongName(childValueLongName) {
    return outerHashFunctionLongName(childValueLongName);
  }
  return serializeStylesLongName("x");
}
"#;
        let tokens = tokenize(src);
        let mut si = build_scopes(src, &tokens);
        let wrapper_scope = si
            .scopes
            .iter()
            .position(|scope| scope.decls.contains("outerHashFunctionLongName"))
            .expect("wrapper scope should declare outerHashFunctionLongName");
        let child_scope = si
            .scopes
            .iter()
            .position(|scope| scope.decls.contains("childValueLongName"))
            .expect("child scope should declare childValueLongName");
        si.scopes[child_scope].parent = Some(0);

        let renames = compute_renames(src, &tokens, &si, false);
        let outer_alias = renames[wrapper_scope]
            .get("outerHashFunctionLongName")
            .expect("outer helper should be renamed");
        let child_alias = renames[child_scope]
            .get("childValueLongName")
            .expect("child parameter should be renamed");

        assert_ne!(
            child_alias, outer_alias,
            "child alias `{child_alias}` must avoid physical parent alias `{outer_alias}`"
        );
    }

    #[test]
    fn test_arrow_scope_aliases_avoid_physical_parent_aliases_when_scope_parent_is_wrong() {
        let src = r#"
function generateUtilityClassesLongName(slotsLongName) {
  const resultClassesLongName = {};
  slotsLongName.forEach(slotNameLongName => {
    resultClassesLongName[slotNameLongName] = slotNameLongName;
  });
  return resultClassesLongName;
}
"#;
        let tokens = tokenize(src);
        let mut si = build_scopes(src, &tokens);
        let outer_scope = si
            .scopes
            .iter()
            .position(|scope| scope.decls.contains("resultClassesLongName"))
            .expect("outer scope should declare resultClassesLongName");
        let arrow_scope = si
            .scopes
            .iter()
            .position(|scope| scope.decls.contains("slotNameLongName"))
            .expect("arrow scope should declare slotNameLongName");
        si.scopes[arrow_scope].parent = Some(0);

        let renames = compute_renames(src, &tokens, &si, false);
        let outer_alias = renames[outer_scope]
            .get("resultClassesLongName")
            .expect("outer accumulator should be renamed");
        let arrow_alias = renames[arrow_scope]
            .get("slotNameLongName")
            .expect("arrow parameter should be renamed");

        assert_ne!(
            arrow_alias, outer_alias,
            "arrow alias `{arrow_alias}` must avoid physical parent alias `{outer_alias}`"
        );
    }

    #[test]
    fn test_free_outer_alias_is_not_reused_for_local_var() {
        let src = r#"
function module(ReactSharedInternals, ReactDOMSharedInternals) {
  function dispatchDiscreteEvent(domEventName, eventSystemFlags, container, nativeEvent) {
    var previousTransition = ReactSharedInternals.T;
    ReactSharedInternals.T = null;
    var previousUpdatePriority = ReactDOMSharedInternals.p;
    try {
      handle(domEventName, eventSystemFlags, container, nativeEvent);
    } finally {
      ReactDOMSharedInternals.p = previousUpdatePriority;
      ReactSharedInternals.T = previousTransition;
    }
  }
  return dispatchDiscreteEvent;
}
"#;
        let out = mangle_variables_with_root(src);
        let tokens = tokenize(&out);
        let mut checked = false;

        for ti in 0..tokens.len().saturating_sub(15) {
            if !matches_ident(&out, &tokens, ti, "var")
                || tokens[ti + 3].kind != TK::Ident
                || !matches_punct(&out, &tokens, ti + 4, ".")
                || !matches_ident(&out, &tokens, ti + 5, "T")
                || !matches_punct(&out, &tokens, ti + 6, ";")
            {
                continue;
            }
            let outer_alias = txt(&out, &tokens[ti + 3]);
            if txt(&out, &tokens[ti + 7]) != outer_alias
                || !matches_punct(&out, &tokens, ti + 8, ".")
                || !matches_ident(&out, &tokens, ti + 9, "T")
                || !matches_punct(&out, &tokens, ti + 10, "=")
                || !matches_ident(&out, &tokens, ti + 11, "null")
                || !matches_punct(&out, &tokens, ti + 12, ";")
                || !matches_ident(&out, &tokens, ti + 13, "var")
                || tokens[ti + 14].kind != TK::Ident
            {
                continue;
            }
            checked = true;
            let local_alias = txt(&out, &tokens[ti + 14]);
            assert_ne!(
                local_alias, outer_alias,
                "local var alias must not shadow free outer alias `{outer_alias}`, got: {out}"
            );
        }

        assert!(
            checked,
            "test fixture should expose the React event wrapper alias pattern, got: {out}"
        );
    }

    fn alias_before_prototype_access<'a>(source: &'a str, access: &str) -> Option<&'a str> {
        let access_start = source.find(access)?;
        let before = &source[..access_start];
        let start = before
            .rfind(|ch: char| !(ch == '_' || ch == '$' || ch.is_ascii_alphanumeric()))
            .map(|idx| idx + 1)
            .unwrap_or(0);
        Some(&before[start..])
    }

    fn has_var_declarator_named(source: &str, name: &str) -> bool {
        let needle_first = format!("var {name}=");
        let needle_next = format!(",{name}=");
        source.contains(&needle_first) || source.contains(&needle_next)
    }

    #[test]
    fn test_root_scope_template_expression_refs_are_mangled() {
        let src = r#"var styledComponentId="sc-a";var selector=`style[${styledComponentId}]`;function read(){return selector;}read();"#;
        let out = mangle_variables_with_root(src);
        assert!(
            !out.contains("styledComponentId"),
            "template expression ref must be renamed with its declaration, got: {}",
            out
        );
        assert!(
            !out.contains("${styledComponentId}"),
            "template expression must not keep the stale root name, got: {}",
            out
        );
        assert!(
            out.contains("`style[${"),
            "template literal shell should be preserved, got: {}",
            out
        );
    }

    #[test]
    fn test_root_scope_spread_expression_refs_are_mangled() {
        let src = r#"const SPACINGS=[0,1,2];const classes=[...SPACINGS.map((value)=>`spacing-${value}`)];console.log(classes);"#;
        let out = mangle_variables_with_root(src);
        assert!(
            !out.contains("SPACINGS"),
            "spread expression ref must be renamed with its declaration, got: {}",
            out
        );
        assert!(
            out.contains("..."),
            "spread operator must be preserved, got: {}",
            out
        );
    }

    #[test]
    fn test_var_array_initializer_comma_does_not_create_fake_declaration() {
        let src = r#"function wrapper(require){var combineImport=require(1)["combine"];var compatPlugin=function compatPlugin(element){return element.type;};var removeLabelPlugin=function removeLabelPlugin(element){return element.value;};var createCache=function createCache(options){var omnipresentPlugins=[compatPlugin,removeLabelPlugin];return combineImport(omnipresentPlugins,function(plugin){return plugin.name;});};return createCache;}"#;
        let out = mangle_variables(src);
        let combine_pos = out
            .find("=require(1)[\"combine\"]")
            .expect("combine import should remain");
        let combine_decl_prefix = &out[..combine_pos];
        let combine_name_start = combine_decl_prefix
            .rfind("var ")
            .expect("combine import should be a var")
            + 4;
        let combine_name = combine_decl_prefix[combine_name_start..].trim();
        let array_start = out
            .find("=[")
            .expect("plugin array should remain after mangling")
            + 2;
        let array_end = out[array_start..]
            .find(']')
            .map(|offset| array_start + offset)
            .expect("plugin array should close");
        let plugins = &out[array_start..array_end];

        assert!(
            !plugins.split(',').any(|name| name.trim() == combine_name),
            "plugin array must not be rewritten to the combine import `{}`: {}",
            combine_name,
            out
        );
    }

    #[test]
    fn test_var_function_expression_reassignment_renames_lhs() {
        let src = r#"!function(module,exports,require){var raf=function raf(callback){return +setTimeout(callback,16);};var caf=function caf(num){return clearTimeout(num);};if(typeof window!=="undefined"&&"requestAnimationFrame" in window){raf=function raf(callback){return window.requestAnimationFrame(callback);};caf=function caf(handle){return window.cancelAnimationFrame(handle);};};var rafIds=new Map();function call(){var id=raf(function(){return caf(1);});rafIds.set(1,id);}call();exports.default=raf;}(_m0,_m0.exports,_r)"#;
        let out = mangle_variables(src);

        assert!(
            !out.contains("raf=function"),
            "reassignment lhs must be renamed with the var binding, got: {}",
            out
        );
        assert!(
            !out.contains("caf=function"),
            "reassignment lhs must be renamed with the var binding, got: {}",
            out
        );
    }

    #[test]
    fn test_named_function_expression_self_reference_uses_own_rename() {
        let src = r#"!function(module,exports,require){var parseStyle=function parseStyle(value){if(value&&value.next){return parseStyle(value.next);}return ["ok",{}];};exports.parseStyle=parseStyle;var extract=function extract(cache){return cache?extract(null):null;};exports.extract=extract;}(_m0,_m0.exports,_r)"#;
        let out = mangle_variables(src);
        let assign_pos = out
            .find(".parseStyle")
            .expect("parseStyle export should remain");
        let parse_style_module = &out[..assign_pos];

        assert!(
            !parse_style_module.contains("extract"),
            "parseStyle self reference must not be rewritten to the later extract binding, got: {}",
            out
        );
        assert!(
            !parse_style_module.contains("return parseStyle("),
            "parseStyle self reference should be consistently renamed, got: {}",
            out
        );
    }

    #[test]
    fn test_object_destructuring_shorthand_binding_renames_template_refs() {
        let src = r#"!function(module,exports,require){function genLoopGridColumnsStyle(token,sizeCls){const{prefixCls,componentCls,gridColumns}=token;const gridColumnsStyle={};gridColumnsStyle[`${componentCls}${sizeCls}-flex`]={flex:`var(--${prefixCls}${sizeCls}-flex)`};return gridColumnsStyle;}exports.default=genLoopGridColumnsStyle;}(_m0,_m0.exports,_r)"#;
        let out = mangle_variables(src);

        assert!(
            !out.contains("${prefixCls}"),
            "template reference should be renamed with destructured binding, got: {}",
            out
        );
        assert!(
            out.contains("prefixCls:"),
            "destructured shorthand must keep property key when binding is renamed, got: {}",
            out
        );
        assert!(
            !out.contains("${lnb}"),
            "template reference must not become an unresolved stale alias, got: {}",
            out
        );
    }

    #[test]
    fn test_object_destructuring_shorthand_default_keeps_property_key_and_default_ref() {
        let src = r#"function createTypography(options){const defaultFontFamily="Roboto";const{fontFamily=defaultFontFamily,fontSize=14}=options;return fontFamily===defaultFontFamily?fontSize:0}"#;
        let out = mangle_variables_with_root(src);

        assert!(
            out.contains("fontFamily:"),
            "shorthand default binding must keep the source property key, got: {}",
            out
        );
        assert!(
            !out.contains("{a=") && !out.contains("{b=") && !out.contains("{c="),
            "shorthand default binding must not become a renamed property/default pair, got: {}",
            out
        );
    }

    #[test]
    fn test_object_destructuring_shorthand_default_after_prior_declarator() {
        let src = r#"{const _m686_defaultFontFamily="Roboto";function _m686_createTypography(p,f){const g=typeof f==="function"?f(p):f,{fontFamily=_m686_defaultFontFamily,fontSize=14}=g,o=omit(g);return fontFamily===_m686_defaultFontFamily?fontSize:o}}"#;
        let out = mangle_variables_with_root(src);

        assert!(
            out.contains("fontFamily:"),
            "multi-declarator object binding must keep the source property key, got: {}",
            out
        );
        assert!(
            !out.contains("{d=m") && !out.contains("{c=b") && !out.contains("{b=c"),
            "multi-declarator object binding must not produce an unresolved renamed default pair, got: {}",
            out
        );
        if out.contains("const _m686_defaultFontFamily") {
            assert!(
                out.contains("=_m686_defaultFontFamily")
                    && out.contains("===_m686_defaultFontFamily"),
                "unrenamed outer default binding must keep matching references, got: {}",
                out
            );
        }
    }

    #[test]
    fn test_retained_wrapper_cjs_param_repair_updates_stale_free_refs_only() {
        let src = r#"!function(a,b,c){function local(module){return module.exports};module.exports["serializeStyles"]=Xo;exports.named=require(0)}(_m1,_m1.exports,_r)"#;
        let out = repair_retained_wrapper_cjs_param_references(src);

        assert!(
            out.contains(r#"a.exports["serializeStyles"]=Xo"#),
            "free module reference should use retained wrapper param, got: {}",
            out
        );
        assert!(
            out.contains("b.named=c(0)"),
            "free exports/require references should use retained wrapper params, got: {}",
            out
        );
        assert!(
            out.contains("function local(module){return module.exports}"),
            "local module binding should not be rewritten, got: {}",
            out
        );
    }

    #[test]
    fn test_react_dom_event_helper_alias_repair_updates_stale_calls() {
        let src = r#"!function(X,U,require){var y=new Set,B={};function V(a,b){s(a,b);s(a+"Capture",b)}function s(a,b){B[a]=b;for(a=0;a<b.length;a++)y.add(b[a])}var df=new Map;function ff(a,b){df.set(a,b);fa(b,[a])}ha("onMouseEnter",["mouseout","mouseover"]);fa("onChange",["change"])}(_m2,_m2.exports,_r)"#;
        let out = repair_react_dom_event_helper_aliases(src);

        assert!(
            out.contains("function ff(a,b){df.set(a,b);V(b,[a])}"),
            "stale two-phase helper call must use renamed alias, got: {}",
            out
        );
        assert!(
            out.contains(r#"s("onMouseEnter",["mouseout","mouseover"])"#),
            "stale direct helper call must use renamed alias, got: {}",
            out
        );
        assert!(
            !out.contains("fa(b,[a])") && !out.contains(r#"ha("onMouseEnter""#),
            "stale helper names must be removed, got: {}",
            out
        );
    }

    #[test]
    fn test_nested_function_mangling() {
        let src = "function outer(){var longOuter=1;function inner(){var longInner=2;return longInner;}return longOuter;}";
        let out = mangle_variables(src);
        eprintln!("nested output: {}", out);
        assert!(
            !out.contains("longOuter"),
            "should mangle longOuter, got: {}",
            out
        );
        assert!(
            !out.contains("longInner"),
            "should mangle longInner, got: {}",
            out
        );
    }

    #[test]
    fn test_keywords_not_generated() {
        // "in" is a keyword — name generator should skip it
        let skip = HashSet::new();
        let mut gen = NameGen::new(&skip);
        let mut names = Vec::new();
        for _ in 0..100 {
            names.push(gen.next_name());
        }
        assert!(
            !names.contains(&"in".to_string()),
            "should not generate keyword 'in'"
        );
        assert!(
            !names.contains(&"do".to_string()),
            "should not generate keyword 'do'"
        );
        assert!(
            !names.contains(&"if".to_string()),
            "should not generate keyword 'if'"
        );
    }

    #[test]
    fn test_object_pattern_alias_does_not_collide_with_later_lexical_decl() {
        let src = r#"function createTheme(options={},...args){const{breakpoints:breakpointsInput={},palette:paletteInput={},spacing:spacingInput,shape:shapeInput={}}=options,other=omit(options,_excluded);const breakpoints=createBreakpoints(breakpointsInput);const spacing=createSpacing(spacingInput);let muiTheme=deepmerge({breakpoints,direction:'ltr',components:{},palette:extend({mode:'light'},paletteInput),spacing,shape:extend({},shape,shapeInput)},other);muiTheme.applyStyles=applyStyles;muiTheme=args.reduce((acc,argument)=>deepmerge(acc,argument),muiTheme);return muiTheme;}"#;
        let out = mangle_variables_with_root(src);
        assert!(
            !(out.contains("spacing:a") && out.contains("let a=")),
            "object-pattern alias must not collide with a later lexical decl, got: {}",
            out
        );
    }

    #[test]
    fn test_object_literal_method_keys_are_preserved() {
        let src = r#"const defaultGenerator=componentName=>componentName;const createClassNameGenerator=()=>{let generate=defaultGenerator;return{configure(generator){generate=generator},generate(componentName){return generate(componentName)},reset(){generate=defaultGenerator}}};const ClassNameGenerator=createClassNameGenerator();ClassNameGenerator.generate("MuiButton");"#;
        let out = mangle_variables_with_root(src);
        assert!(out.contains("configure("), "got: {}", out);
        assert!(out.contains("generate("), "got: {}", out);
        assert!(out.contains("reset("), "got: {}", out);
        assert!(out.contains(".generate("), "got: {}", out);
    }

    #[test]
    fn test_generated_require_helper_repair_updates_stale_free_refs_only() {
        let src = r#"(function(){var a=[{exports:{}}];function b(id){var m=a[id];return m?m.exports:{}};function local(_r){return _r(0)};{var value=_r(0);}})()"#;
        let out = repair_generated_require_helper_references(src);
        assert!(
            out.contains("var value=b(0)"),
            "stale generated _r ref should be repaired, got: {}",
            out
        );
        assert!(
            out.contains("function local(_r){return _r(0)}"),
            "local _r binding must be preserved, got: {}",
            out
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // UTF-8 multi-byte safety tests (issue #904)
    //
    // The tokenizer operates on raw bytes (`source.as_bytes()`) and only
    // matches ASCII identifier characters, so multi-byte UTF-8 sequences
    // pass through as opaque Punct tokens.  `apply_renames` reconstructs
    // the output via `result.splice(byte_start..byte_end, ...)` where
    // offsets come from the byte-level scan — no char-index-as-byte-offset
    // bug is possible.  These tests verify that.
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_utf8_string_content_unchanged() {
        // ✓ is 3-byte UTF-8; the string literal must survive intact.
        // Use a multi-char name so the mangler actually renames it.
        let src = r#"function f() { var checkResult = "✓ ok"; return checkResult; }"#;
        let out = mangle_variables(src);
        assert!(
            out.contains("\"✓ ok\""),
            "UTF-8 string must be unchanged, got: {}",
            out
        );
        // Local `checkResult` should be mangled to a shorter name
        assert!(
            !out.contains("checkResult"),
            "local checkResult should be mangled, got: {}",
            out
        );
    }

    #[test]
    fn test_utf8_emoji_in_string_preserved() {
        // 🎉 is 4-byte UTF-8
        let src = r#"function f() { var msg = "Hello 🎉"; return msg; }"#;
        let out = mangle_variables(src);
        assert!(
            out.contains("\"Hello 🎉\""),
            "emoji string must be preserved, got: {}",
            out
        );
    }

    #[test]
    fn test_utf8_cjk_in_string_preserved() {
        // Use a multi-char name so the mangler actually renames it.
        let src = "function f() { var strVal = '日本語テスト'; return strVal; }";
        let out = mangle_variables(src);
        assert!(
            out.contains("'日本語テスト'"),
            "CJK string must be preserved, got: {}",
            out
        );
        // `strVal` should be mangled to a shorter name
        assert!(
            !out.contains("strVal"),
            "local strVal should be mangled, got: {}",
            out
        );
    }

    #[test]
    fn test_utf8_mixed_code_and_strings() {
        // Multi-byte chars before and after identifiers that get renamed
        let src = "function f() { var longName = '✓'; var other = longName + '日'; return other; }";
        let out = mangle_variables(src);
        // Strings preserved
        assert!(out.contains("'✓'"), "✓ string preserved, got: {}", out);
        assert!(out.contains("'日'"), "日 string preserved, got: {}", out);
        // Identifiers mangled
        assert!(
            !out.contains("longName"),
            "longName should be mangled, got: {}",
            out
        );
        assert!(
            !out.contains("other"),
            "other should be mangled, got: {}",
            out
        );
    }
}
// CODEGEN-END
