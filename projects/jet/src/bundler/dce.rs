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

use std::collections::HashSet;
use tree_sitter::{Node, Parser};

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

/// Syntax-aware static conditional elimination for production bundles.
///
/// This deliberately handles only conditionals whose condition is already a
/// literal boolean or string comparison after define replacement. It does not
/// try to prove general variable liveness, so it is safe to run on large third
/// party bundles where the older brace-scanning optimizer is too broad.
pub fn eliminate_static_conditionals_syntax(source: &str) -> String {
    let mut result = source.to_string();

    for _ in 0..8 {
        let next = eliminate_static_conditionals_syntax_once(&result);
        if next == result {
            break;
        }
        result = next;
    }

    result
}

/// Remove unused transformed import bindings only when every required module id
/// is known side-effect-free. This is intentionally narrower than general DCE:
/// it handles the production pattern left after libraries such as MUI erase
/// dev-only `propTypes` branches but keep an unused `var PropTypes = require(..)`.
pub fn eliminate_unused_side_effect_free_require_bindings(
    source: &str,
    side_effect_free_module_ids: &HashSet<usize>,
) -> String {
    if side_effect_free_module_ids.is_empty() {
        return source.to_string();
    }
    let Some(tree) = parse_js(source) else {
        return source.to_string();
    };
    let root = tree.root_node();
    if root.has_error() {
        return source.to_string();
    }

    let mut edits = Vec::new();
    collect_unused_require_binding_edits(
        source,
        root,
        root,
        side_effect_free_module_ids,
        &mut edits,
    );
    if edits.is_empty() {
        return source.to_string();
    }

    edits.sort_by_key(|edit| edit.start);
    let mut filtered: Vec<StaticEdit> = Vec::new();
    let mut last_end = 0usize;
    for edit in edits {
        if edit.start >= last_end {
            last_end = edit.end;
            filtered.push(edit);
        }
    }

    let mut out = source.to_string();
    for edit in filtered.into_iter().rev() {
        out.replace_range(edit.start..edit.end, "");
    }

    if parse_js(&out)
        .map(|tree| tree.root_node().has_error())
        .unwrap_or(true)
    {
        return source.to_string();
    }

    out
}

/// Prune a retained module's lowered re-export glue down to the names the
/// tree-shake analysis proved used.
///
/// Barrel modules lower every `export { x } from "./x"` into an
/// unconditional `module.exports["x"] = require(id)[...];` statement (several per
/// line) and every `export * from "./y"` into a
/// `var __re = require(id); Object.keys(...)` copy loop. Those require calls
/// rescued every re-export target back into the bundle even when the
/// analysis had already proven the name unused — on MUI that re-imported
/// ~170KB of eliminated code. Dropping the assignment leaves the target to
/// the reachability walk: if nobody else requires it, it is eliminated
/// with the rest.
///
/// Statements are matched span-wise (NOT line-wise — the lowering emits
/// sibling assignments on one line) and only with safe value shapes
/// (`require(id)[...]`, `require(id)["default"] || require(id)`, or a bare identifier),
/// so arbitrary expressions are never deleted. The star-copy loop is kept
/// whenever any used name is not covered by an explicit assignment.
pub(crate) fn eliminate_unused_reexport_assignments(
    source: &str,
    used: &HashSet<String>,
    star_leaf_exports: Option<&dyn Fn(usize) -> Option<Vec<String>>>,
) -> String {
    // "*" marks whole-namespace consumption (import * as ns, namespace-style
    // CJS requires, dynamic import) — every export may be read at runtime,
    // so nothing is prunable.
    if used.contains("*") {
        return source.to_string();
    }
    use std::sync::OnceLock;
    static EXPLICIT: OnceLock<regex::Regex> = OnceLock::new();
    static STAR: OnceLock<regex::Regex> = OnceLock::new();
    let explicit = EXPLICIT.get_or_init(|| {
        regex::Regex::new(
            r#"module\.exports\["([A-Za-z0-9_$]+)"\]\s*=\s*(?:require\(\d+\)(?:\["[^"]+"\])?(?:\s*\|\|\s*require\(\d+\))?|[A-Za-z_$][A-Za-z0-9_$]*)\s*;\s?"#,
        )
        .unwrap()
    });
    let star = STAR.get_or_init(|| {
        regex::Regex::new(
            r#"var __re = require\((\d+)\); Object\.keys\(__re\)\.forEach\(function\(k\) \{ if \(k !== "default"\) module\.exports\[k\] = __re\[k\]; \}\);\s?"#,
        )
        .unwrap()
    });

    let explicit_names: HashSet<&str> = explicit
        .captures_iter(source)
        .map(|cap| cap.get(1).map(|m| m.as_str()).unwrap_or(""))
        .collect();

    // Names that must flow through the star copies: used on this barrel but
    // not provided by an explicit assignment.
    let mut star_needed_names: HashSet<&str> = used
        .iter()
        .map(|s| s.as_str())
        .filter(|name| *name != "default" && !explicit_names.contains(*name))
        .collect();

    // Edits: removals plus star materializations (replacement text).
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    for cap in explicit.captures_iter(source) {
        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        if !used.contains(name) {
            let whole = cap.get(0).unwrap();
            edits.push((whole.start(), whole.end(), String::new()));
        }
    }

    // Star loops: materialize the needed names as explicit assignments so
    // the dynamic Object.keys copy (which retains the WHOLE target module
    // graph) disappears. Without leaf-export knowledge the loop is kept
    // only when names still need it.
    let mut stars_resolvable = star_leaf_exports.is_some();
    if stars_resolvable {
        for cap in star.captures_iter(source) {
            let id_ok = cap
                .get(1)
                .and_then(|m| m.as_str().parse::<usize>().ok())
                .and_then(|id| star_leaf_exports.and_then(|f| f(id)))
                .is_some();
            if !id_ok {
                stars_resolvable = false;
                break;
            }
        }
    }
    if stars_resolvable {
        let lookup = star_leaf_exports.unwrap();
        for cap in star.captures_iter(source) {
            let whole = cap.get(0).unwrap();
            let id: usize = cap[1].parse().unwrap_or(usize::MAX);
            let leaf = lookup(id).unwrap_or_default();
            let claimed: Vec<&str> = leaf
                .iter()
                .map(|s| s.as_str())
                .filter(|n| star_needed_names.contains(*n))
                .collect();
            let mut replacement = String::new();
            for name in &claimed {
                replacement.push_str(&format!(
                    "module.exports[\"{name}\"] = require({id})[\"{name}\"]; "
                ));
                star_needed_names.remove(*name);
            }
            edits.push((whole.start(), whole.end(), replacement));
        }
    } else if star_needed_names.is_empty() {
        for m in star.find_iter(source) {
            edits.push((m.start(), m.end(), String::new()));
        }
    }

    if edits.is_empty() {
        return source.to_string();
    }
    edits.sort_by_key(|(start, _, _)| *start);

    let b = source.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut pos = 0usize;
    for (start, end, replacement) in edits {
        if start < pos {
            continue;
        }
        out.extend_from_slice(&b[pos..start]);
        out.extend_from_slice(replacement.as_bytes());
        pos = end;
    }
    out.extend_from_slice(&b[pos..]);
    String::from_utf8(out).unwrap_or_else(|_| source.to_string())
}

/// Remove transformed CJS re-export glue that points at modules already proven
/// unused by the source-level tree-shake pass.
pub fn eliminate_require_reexports_to_eliminated_modules(
    source: &str,
    eliminated_module_ids: &HashSet<usize>,
) -> String {
    if eliminated_module_ids.is_empty() {
        return source.to_string();
    }
    let Some(tree) = parse_js(source) else {
        return source.to_string();
    };
    let root = tree.root_node();
    if root.has_error() {
        return source.to_string();
    }

    let mut reexport_bindings = HashSet::new();
    collect_eliminated_reexport_bindings(
        source,
        root,
        eliminated_module_ids,
        &mut reexport_bindings,
    );

    let mut edits = Vec::new();
    collect_eliminated_require_reexport_edits(
        source,
        root,
        eliminated_module_ids,
        &reexport_bindings,
        &mut edits,
    );
    if edits.is_empty() {
        return source.to_string();
    }

    edits.sort_by_key(|edit| edit.start);
    let mut filtered: Vec<StaticEdit> = Vec::new();
    let mut last_end = 0usize;
    for edit in edits {
        if edit.start >= last_end {
            last_end = edit.end;
            filtered.push(edit);
        }
    }

    let mut out = source.to_string();
    for edit in filtered.into_iter().rev() {
        out.replace_range(edit.start..edit.end, "");
    }

    if parse_js(&out)
        .map(|tree| tree.root_node().has_error())
        .unwrap_or(true)
    {
        return source.to_string();
    }

    out
}

/// Remove ESM marker definitions only when the final bundle never reads them.
///
/// The ESM-to-CJS transform marks every source module with
/// `Object.defineProperty(module.exports, "__esModule", { value: true })` for
/// Babel-style interop. Large ESM libraries can carry thousands of those
/// markers even when no helper reads `.__esModule`. This pass removes marker
/// statements only if deleting all candidate markers leaves no `__esModule`
/// literal anywhere else in the bundle.
pub fn eliminate_unread_es_module_markers(source: &str) -> String {
    if !source.contains("__esModule") {
        return source.to_string();
    }

    // The markers have a FIXED generated shape — transform/modules.rs emits
    // `Object.defineProperty(module.exports, "__esModule", { value: true });`
    // and the only later rewrite renames the receiver to `_mN`. Two full
    // tree-sitter parses of a multi-MB bundle (~0.6s on the antd corpus)
    // are unnecessary: match every `__esModule` occurrence lexically and
    // bail to the original source if ANY occurrence is not a removable
    // marker in statement position. That bail subsumes the old
    // `out.contains("__esModule")` revert (library code that genuinely
    // reads the flag keeps every marker), so the markers are only dropped
    // when nothing can observe them.
    const KEY_DQ: &str = "\"__esModule\"";
    const KEY_SQ: &str = "'__esModule'";
    const PREFIX: &str = "Object.defineProperty(";

    let b = source.as_bytes();
    let mut removals: Vec<(usize, usize)> = Vec::new();
    let mut search = 0usize;
    while let Some(rel) = source[search..].find("__esModule") {
        let key_at = search + rel;
        search = key_at + "__esModule".len();

        // The occurrence must be the quoted property key of the marker.
        let quoted_start = key_at.checked_sub(1);
        let is_quoted = quoted_start
            .map(|q| {
                (source[q..].starts_with(&KEY_DQ[..1]) && source[q..].starts_with(KEY_DQ))
                    || (source[q..].starts_with(&KEY_SQ[..1]) && source[q..].starts_with(KEY_SQ))
            })
            .unwrap_or(false);
        if !is_quoted {
            return source.to_string();
        }
        let key_start = key_at - 1;
        let key_end = key_at + "__esModule".len() + 1;

        // Backward: `Object.defineProperty(` + receiver (`module.exports`
        // or `_mN.exports` or a bare identifier) + `, `.
        let before = &source[..key_start];
        let Some(prefix_at) = before.rfind(PREFIX) else {
            return source.to_string();
        };
        let receiver = &source[prefix_at + PREFIX.len()..key_start];
        let receiver_trim = receiver.trim_end_matches(|c: char| c == ' ' || c == ',');
        let receiver_ok = !receiver_trim.is_empty()
            && receiver.trim_end().ends_with(',')
            && receiver_trim
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '$' || c == '.')
            && !receiver_trim.contains("..");
        if !receiver_ok {
            return source.to_string();
        }

        // Forward: `, { value: true });` (whitespace-flexible, `!0` accepted).
        let after = &source[key_end..];
        let after_trim = after.trim_start();
        let mut consumed = after.len() - after_trim.len();
        let Some(rest) = after_trim.strip_prefix(',') else {
            return source.to_string();
        };
        consumed += 1;
        let rest_trim = rest.trim_start();
        consumed += rest.len() - rest_trim.len();
        let body = ["{ value: true });", "{value:true});", "{value:!0});"]
            .iter()
            .find(|p| rest_trim.starts_with(**p));
        let Some(body) = body else {
            return source.to_string();
        };
        let stmt_end = key_end + consumed + body.len();

        // Statement position: the previous significant byte before the
        // prefix must open or end a statement.
        let mut p = prefix_at;
        while p > 0 && matches!(b[p - 1], b' ' | b'\t' | b'\r' | b'\n') {
            p -= 1;
        }
        if p > 0 && !matches!(b[p - 1], b'{' | b'}' | b';') {
            return source.to_string();
        }

        // Include trailing whitespace up to and including one newline.
        let mut e = stmt_end;
        while e < b.len() && matches!(b[e], b' ' | b'\t') {
            e += 1;
        }
        if e < b.len() && b[e] == b'\n' {
            e += 1;
        }
        removals.push((prefix_at, e));
    }

    if removals.is_empty() {
        return source.to_string();
    }

    let mut out = String::with_capacity(source.len());
    let mut pos = 0usize;
    for (start, end) in removals {
        if start < pos {
            return source.to_string();
        }
        out.push_str(&source[pos..start]);
        pos = end;
    }
    out.push_str(&source[pos..]);
    out
}

pub fn js_parses_without_errors(source: &str) -> bool {
    parse_js(source)
        .map(|tree| !tree.root_node().has_error())
        .unwrap_or(false)
}

pub(crate) fn numeric_require_ids(source: &str) -> HashSet<usize> {
    let Some(tree) = parse_js(source) else {
        return HashSet::new();
    };
    let root = tree.root_node();
    if root.has_error() {
        return HashSet::new();
    }

    let mut ids = Vec::new();
    collect_numeric_require_ids(source, root, &mut ids);
    ids.into_iter().collect()
}

fn eliminate_static_conditionals_syntax_once(source: &str) -> String {
    let Some(tree) = parse_js(source) else {
        return source.to_string();
    };
    let root = tree.root_node();
    if root.has_error() {
        return source.to_string();
    }

    let mut edits = Vec::new();
    collect_static_condition_edits(source, root, &mut edits);
    if edits.is_empty() {
        return source.to_string();
    }

    edits.sort_by_key(|edit| edit.start);
    let mut filtered: Vec<StaticEdit> = Vec::new();
    let mut last_end = 0usize;
    for edit in edits {
        if edit.start >= last_end {
            last_end = edit.end;
            filtered.push(edit);
        }
    }

    let mut out = source.to_string();
    for edit in filtered.into_iter().rev() {
        out.replace_range(edit.start..edit.end, &edit.replacement);
    }

    if parse_js(&out)
        .map(|tree| tree.root_node().has_error())
        .unwrap_or(true)
    {
        return source.to_string();
    }

    out
}

fn collect_unused_require_binding_edits(
    source: &str,
    root: Node<'_>,
    node: Node<'_>,
    side_effect_free_module_ids: &HashSet<usize>,
    edits: &mut Vec<StaticEdit>,
) {
    match node.kind() {
        "variable_declaration" | "lexical_declaration" => {
            if let Some(edit) =
                unused_require_binding_edit(source, root, node, side_effect_free_module_ids)
            {
                edits.push(edit);
                return;
            }
        }
        "template_string" | "string" | "comment" | "regex" | "regex_pattern" => return,
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_unused_require_binding_edits(
            source,
            root,
            child,
            side_effect_free_module_ids,
            edits,
        );
    }
}

fn collect_eliminated_reexport_bindings(
    source: &str,
    node: Node<'_>,
    eliminated_module_ids: &HashSet<usize>,
    bindings: &mut HashSet<String>,
) {
    if matches!(
        node.kind(),
        "template_string" | "string" | "comment" | "regex" | "regex_pattern"
    ) {
        return;
    }

    if matches!(node.kind(), "variable_declaration" | "lexical_declaration") {
        if let Some((ident, ids)) = single_require_declarator(source, node) {
            if ident.starts_with("__re")
                && !ids.is_empty()
                && ids.iter().all(|id| eliminated_module_ids.contains(id))
            {
                bindings.insert(ident.to_string());
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_eliminated_reexport_bindings(source, child, eliminated_module_ids, bindings);
    }
}

fn collect_eliminated_require_reexport_edits(
    source: &str,
    node: Node<'_>,
    eliminated_module_ids: &HashSet<usize>,
    reexport_bindings: &HashSet<String>,
    edits: &mut Vec<StaticEdit>,
) {
    if matches!(
        node.kind(),
        "template_string" | "string" | "comment" | "regex" | "regex_pattern"
    ) {
        return;
    }

    match node.kind() {
        "variable_declaration" | "lexical_declaration" => {
            if let Some((ident, ids)) = single_require_declarator(source, node) {
                if reexport_bindings.contains(ident)
                    && !ids.is_empty()
                    && ids.iter().all(|id| eliminated_module_ids.contains(id))
                {
                    edits.push(StaticEdit {
                        start: node.start_byte(),
                        end: node.end_byte(),
                        replacement: String::new(),
                    });
                    return;
                }
            }
        }
        "expression_statement" => {
            let text = source[node.byte_range()].trim();
            let mut ids = Vec::new();
            collect_numeric_require_ids(source, node, &mut ids);
            if !ids.is_empty() && ids.iter().all(|id| eliminated_module_ids.contains(id)) {
                if is_module_exports_require_assignment(text)
                    || is_bare_require_statement(text)
                    || reexport_bindings
                        .iter()
                        .any(|ident| is_object_keys_reexport_statement(text, ident))
                {
                    edits.push(StaticEdit {
                        start: node.start_byte(),
                        end: node.end_byte(),
                        replacement: String::new(),
                    });
                    return;
                }
            }
            if reexport_bindings
                .iter()
                .any(|ident| is_object_keys_reexport_statement(text, ident))
            {
                edits.push(StaticEdit {
                    start: node.start_byte(),
                    end: node.end_byte(),
                    replacement: String::new(),
                });
                return;
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_eliminated_require_reexport_edits(
            source,
            child,
            eliminated_module_ids,
            reexport_bindings,
            edits,
        );
    }
}

fn collect_es_module_marker_edits(source: &str, node: Node<'_>, edits: &mut Vec<StaticEdit>) {
    if matches!(
        node.kind(),
        "template_string" | "string" | "comment" | "regex" | "regex_pattern"
    ) {
        return;
    }

    if node.kind() == "expression_statement" {
        let text = source[node.byte_range()].trim();
        if text.starts_with("Object.defineProperty(")
            && (text.contains("\"__esModule\"") || text.contains("'__esModule'"))
        {
            edits.push(StaticEdit {
                start: node.start_byte(),
                end: node.end_byte(),
                replacement: String::new(),
            });
            return;
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_es_module_marker_edits(source, child, edits);
    }
}

fn single_require_declarator<'a>(
    source: &'a str,
    declaration: Node<'_>,
) -> Option<(&'a str, Vec<usize>)> {
    let mut cursor = declaration.walk();
    let declarators: Vec<Node<'_>> = declaration
        .named_children(&mut cursor)
        .filter(|child| child.kind() == "variable_declarator")
        .collect();
    if declarators.len() != 1 {
        return None;
    }
    let declarator = declarators[0];
    let name = declarator.child_by_field_name("name")?;
    if name.kind() != "identifier" {
        return None;
    }
    let value = declarator.child_by_field_name("value")?;
    let mut ids = Vec::new();
    collect_numeric_require_ids(source, value, &mut ids);
    Some((&source[name.byte_range()], ids))
}

fn is_module_exports_require_assignment(text: &str) -> bool {
    text.starts_with("module.exports") && (text.contains("require(") || text.contains("_r("))
}

fn is_bare_require_statement(text: &str) -> bool {
    let trimmed = text.trim_end_matches(';').trim();
    (trimmed.starts_with("require(") || trimmed.starts_with("_r("))
        && (trimmed.ends_with(')') || trimmed.contains(")"))
}

fn is_object_keys_reexport_statement(text: &str, ident: &str) -> bool {
    text.starts_with(&format!("Object.keys({ident})"))
        && text.contains("forEach")
        && text.contains("module.exports")
}

fn unused_require_binding_edit(
    source: &str,
    root: Node<'_>,
    declaration: Node<'_>,
    side_effect_free_module_ids: &HashSet<usize>,
) -> Option<StaticEdit> {
    let mut cursor = declaration.walk();
    let declarators: Vec<Node<'_>> = declaration
        .named_children(&mut cursor)
        .filter(|child| child.kind() == "variable_declarator")
        .collect();
    if declarators.len() != 1 {
        return None;
    }

    let declarator = declarators[0];
    let name = declarator.child_by_field_name("name")?;
    if name.kind() != "identifier" {
        return None;
    }
    let ident = &source[name.byte_range()];
    let value = declarator.child_by_field_name("value")?;
    let mut require_ids = Vec::new();
    collect_numeric_require_ids(source, value, &mut require_ids);
    if require_ids.is_empty()
        || !require_ids
            .iter()
            .all(|id| side_effect_free_module_ids.contains(id))
    {
        return None;
    }

    if identifier_has_reference_outside(source, root, ident, declaration.byte_range()) {
        return None;
    }

    Some(StaticEdit {
        start: declaration.start_byte(),
        end: declaration.end_byte(),
        replacement: String::new(),
    })
}

fn collect_numeric_require_ids(source: &str, node: Node<'_>, ids: &mut Vec<usize>) {
    if node.kind() == "call_expression" {
        if let Some(function) = node.child_by_field_name("function") {
            let function_text = &source[function.byte_range()];
            if function_text == "require" || function_text == "_r" {
                if let Some(arguments) = node.child_by_field_name("arguments") {
                    if let Some(first) = arguments.named_child(0) {
                        if first.kind() == "number" {
                            if let Ok(id) = source[first.byte_range()].parse::<usize>() {
                                ids.push(id);
                            }
                        }
                    }
                }
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_numeric_require_ids(source, child, ids);
    }
}

fn identifier_has_reference_outside(
    source: &str,
    node: Node<'_>,
    ident: &str,
    excluded: std::ops::Range<usize>,
) -> bool {
    if matches!(node.kind(), "identifier" | "shorthand_property_identifier")
        && &source[node.byte_range()] == ident
        && (node.start_byte() < excluded.start || node.end_byte() > excluded.end)
    {
        return true;
    }

    if matches!(
        node.kind(),
        "string" | "comment" | "regex" | "regex_pattern"
    ) {
        return false;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if identifier_has_reference_outside(source, child, ident, excluded.clone()) {
            return true;
        }
    }
    false
}

fn parse_js(source: &str) -> Option<tree_sitter::Tree> {
    let mut parser = Parser::new();
    if parser
        .set_language(&tree_sitter_javascript::LANGUAGE.into())
        .is_err()
    {
        return None;
    }
    parser.parse(source, None)
}

#[derive(Debug)]
struct StaticEdit {
    start: usize,
    end: usize,
    replacement: String,
}

fn collect_static_condition_edits(source: &str, node: Node<'_>, edits: &mut Vec<StaticEdit>) {
    if matches!(
        node.kind(),
        "template_string" | "string" | "comment" | "regex" | "regex_pattern"
    ) {
        return;
    }

    match node.kind() {
        "if_statement" => {
            if let Some(edit) = static_if_edit(source, node) {
                edits.push(edit);
                return;
            }
        }
        "ternary_expression" => {
            if let Some(edit) = static_ternary_edit(source, node) {
                edits.push(edit);
                return;
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_static_condition_edits(source, child, edits);
    }
}

fn static_if_edit(source: &str, node: Node<'_>) -> Option<StaticEdit> {
    if node.parent().map(|parent| parent.kind()) == Some("if_statement") {
        return None;
    }

    let condition = node.child_by_field_name("condition")?;
    let condition = normalize_condition_text(&source[condition.byte_range()]);
    let value = eval_condition(condition)?;

    let replacement = if value {
        branch_replacement(source, node.child_by_field_name("consequence")?)
    } else if let Some(alternative) = node.child_by_field_name("alternative") {
        branch_replacement(source, alternative)
    } else {
        "{}".to_string()
    };

    Some(StaticEdit {
        start: node.start_byte(),
        end: node.end_byte(),
        replacement,
    })
}

fn static_ternary_edit(source: &str, node: Node<'_>) -> Option<StaticEdit> {
    let condition = node.child_by_field_name("condition")?;
    let condition = normalize_condition_text(&source[condition.byte_range()]);
    let value = eval_condition(condition)?;
    let selected = if value {
        node.child_by_field_name("consequence")?
    } else {
        node.child_by_field_name("alternative")?
    };

    Some(StaticEdit {
        start: node.start_byte(),
        end: node.end_byte(),
        replacement: source[selected.byte_range()].to_string(),
    })
}

fn branch_replacement(source: &str, node: Node<'_>) -> String {
    let branch = if node.kind() == "else_clause" {
        node.named_child(0).unwrap_or(node)
    } else {
        node
    };
    source[branch.byte_range()].to_string()
}

fn normalize_condition_text(raw: &str) -> &str {
    let mut s = raw.trim();
    loop {
        let stripped = strip_outer_parens(s);
        if stripped == s {
            return s;
        }
        s = stripped.trim();
    }
}

fn strip_outer_parens(s: &str) -> &str {
    let s = s.trim();
    if !(s.starts_with('(') && s.ends_with(')')) {
        return s;
    }

    let chars: Vec<char> = s.chars().collect();
    let Some(close) = find_matching_paren(&chars, 0) else {
        return s;
    };
    if close + 1 != chars.len() {
        return s;
    }

    let bo = build_byte_offsets(s);
    slice_source(s, &bo, 1, close)
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

    // Direct boolean: "false", "true", and the minified `!0` / `!1`
    // forms produced by fold_define_short_circuits.
    match cond {
        "!0" => return Some(true),
        "!1" => return Some(false),
        _ => {}
    }
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
    fn test_dce_ternary_false_branch_function_expression() {
        let input =
            r#"module.exports["default"]="production"!=='production'?useRenderTimes:function(){};"#;
        let output = eliminate_dead_code(input);
        assert!(
            output.contains(r#"module.exports["default"]=function(){}"#),
            "false branch function expression should be preserved, got: {}",
            output
        );
        assert!(
            !output.contains(r#"module.exports["default"]=;"#),
            "ternary DCE must not empty the default export expression: {}",
            output
        );
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

    #[test]
    fn test_syntax_static_if_false_removes_template_branch() {
        let input =
            r#"before();if("production"!=="production"){console.error(`dead ${value}`);}after();"#;
        let output = eliminate_static_conditionals_syntax(input);
        assert!(output.contains("before()"));
        assert!(output.contains("after()"));
        assert!(!output.contains("console.error"));
        assert!(!output.contains("dead ${value}"));
    }

    #[test]
    fn test_syntax_static_if_else_keeps_production_branch() {
        let input = r#"if("production"==="production"){module.exports=require("./prod.js");}else{module.exports=require("./dev.js");}"#;
        let output = eliminate_static_conditionals_syntax(input);
        assert!(output.contains("./prod.js"));
        assert!(!output.contains("./dev.js"));
    }

    #[test]
    fn test_syntax_static_ternary_keeps_selected_branch() {
        let input = r#"var mode="production"!=="production"?devMode():prodMode();"#;
        let output = eliminate_static_conditionals_syntax(input);
        assert!(output.contains("prodMode()"));
        assert!(!output.contains("devMode()"));
    }

    #[test]
    fn test_syntax_static_if_false_preserves_dangling_else_shape() {
        let input = r#"if (outer) if ("production" !== "production") { dead(); } else { inner(); } else { outerElse(); }"#;
        let output = eliminate_static_conditionals_syntax(input);
        assert_eq!(output, input, "nested if/else ambiguity must be skipped");

        let safe = r#"if (outer) { if ("production" !== "production") { dead(); } } else { outerElse(); }"#;
        let safe_output = eliminate_static_conditionals_syntax(safe);
        assert!(safe_output.contains("if (outer) { {} }"));
        assert!(safe_output.contains("else { outerElse(); }"));
        assert!(!safe_output.contains("dead()"));
    }

    #[test]
    fn test_syntax_static_if_else_handles_transformed_module_prefix() {
        let input = r#"Object.defineProperty(module.exports, "__esModule", { value: true });
if ("production" !== "production") {
  if (window.__JET_DEV_FLAG__) {
    console.log("dev branch");
  } else {
    console.log("inner dev else");
  }
} else {
  console.log("prod branch");
}
const value = 1;; module.exports["value"] = value;"#;
        let output = eliminate_static_conditionals_syntax(input);
        assert!(output.contains("prod branch"), "{}", output);
        assert!(!output.contains("dev branch"), "{}", output);
    }

    #[test]
    fn test_unused_side_effect_free_require_binding_is_removed() {
        let input = r#"var PropTypes = require(7)["default"] || require(7);
const value = 1;
module.exports["value"] = value;"#;
        let output =
            eliminate_unused_side_effect_free_require_bindings(input, &HashSet::from([7usize]));
        assert!(!output.contains("PropTypes"), "{}", output);
        assert!(!output.contains("require(7)"), "{}", output);
        assert!(output.contains("module.exports"), "{}", output);
    }

    #[test]
    fn test_used_require_binding_is_kept() {
        let input = r#"var PropTypes = require(7)["default"] || require(7);
const value = PropTypes.string;
module.exports["value"] = value;"#;
        let output =
            eliminate_unused_side_effect_free_require_bindings(input, &HashSet::from([7usize]));
        assert!(output.contains("PropTypes"), "{}", output);
        assert!(output.contains("require(7)"), "{}", output);
    }

    #[test]
    fn test_require_binding_used_inside_template_expression_is_kept() {
        let input = r#"var ClassNameGenerator = require(7)["default"] || require(7);
function className(componentName, slot) {
  return `${ClassNameGenerator.generate(componentName)}-${slot}`;
}
module.exports["default"] = className;"#;
        let output =
            eliminate_unused_side_effect_free_require_bindings(input, &HashSet::from([7usize]));
        assert!(output.contains("ClassNameGenerator"), "{}", output);
        assert!(output.contains("require(7)"), "{}", output);
    }

    #[test]
    fn test_require_binding_used_as_object_shorthand_is_kept() {
        let input = r#"var grey = require(7)["default"] || require(7);
const palette = {
  common: {},
  grey,
  contrastThreshold: 3,
};
module.exports["default"] = palette;"#;
        let output =
            eliminate_unused_side_effect_free_require_bindings(input, &HashSet::from([7usize]));
        assert!(output.contains("grey = require(7)"), "{}", output);
        assert!(output.contains("grey,"), "{}", output);
    }

    #[test]
    fn test_unused_require_binding_for_unknown_side_effect_target_is_kept() {
        let input = r#"var init = require(7);
const value = 1;"#;
        let output =
            eliminate_unused_side_effect_free_require_bindings(input, &HashSet::from([8usize]));
        assert!(output.contains("init"), "{}", output);
        assert!(output.contains("require(7)"), "{}", output);
    }

    #[test]
    fn test_unread_es_module_markers_are_removed() {
        let input = r#"Object.defineProperty(module.exports, "__esModule", { value: true });
const value = 1;
module.exports["value"] = value;
Object.defineProperty(_m1.exports, "__esModule", { value: true });
_m1.exports["other"] = 2;"#;
        let output = eliminate_unread_es_module_markers(input);
        assert!(!output.contains("__esModule"), "{}", output);
        assert!(output.contains("module.exports"), "{}", output);
        assert!(output.contains("_m1.exports"), "{}", output);
    }

    #[test]
    fn test_es_module_markers_are_kept_when_interop_reads_marker() {
        let input = r#"Object.defineProperty(module.exports, "__esModule", { value: true });
function _interopRequireDefault(obj) {
  return obj && obj.__esModule ? obj : { default: obj };
}
module.exports["value"] = 1;"#;
        let output = eliminate_unread_es_module_markers(input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_js_parses_without_errors_reports_syntax_errors() {
        assert!(js_parses_without_errors("const value = `${name}`;"));
        assert!(!js_parses_without_errors("const value = ;"));
    }
}
// CODEGEN-END
