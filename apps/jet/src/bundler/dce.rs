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

use std::collections::{HashMap, HashSet};
use tree_sitter::{Node, Parser};

// WI #2126: reuses the whole-module eval/with/arguments[..] safety probe
// for `eliminate_dead_top_level_declarations`.
use super::scope_hoist;

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
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
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
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
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

    // Build a name -> occurrence-byte-ranges index with a single walk of the
    // tree, then answer every candidate binding's "referenced outside its
    // declaration span?" question from the index instead of re-walking the
    // whole module per binding. On large CJS barrels (e.g.
    // @mui/icons-material, ~10.6k single-declarator require bindings) the
    // old per-binding tree walk was O(bindings * AST nodes); this is
    // O(AST nodes + bindings).
    let mut identifier_index: HashMap<&str, Vec<(usize, usize)>> = HashMap::new();
    index_identifier_occurrences(source, root, &mut identifier_index);

    let mut edits = Vec::new();
    collect_unused_require_binding_edits(
        source,
        root,
        &identifier_index,
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

/// Index every `identifier` / `shorthand_property_identifier` occurrence in
/// the tree by name, recording each occurrence's byte range. Mirrors the
/// exact node-kind checks and skip-set of `identifier_has_reference_outside`
/// (does NOT skip `template_string` — substitutions like `${_Foo}` contain
/// real identifier references) so a single walk can answer every binding's
/// "referenced outside declaration span?" query afterward.
fn index_identifier_occurrences<'a>(
    source: &'a str,
    node: Node<'_>,
    index: &mut HashMap<&'a str, Vec<(usize, usize)>>,
) {
    if matches!(node.kind(), "identifier" | "shorthand_property_identifier") {
        let range = node.byte_range();
        index
            .entry(&source[range.clone()])
            .or_default()
            .push((range.start, range.end));
    }

    if matches!(
        node.kind(),
        "string" | "comment" | "regex" | "regex_pattern"
    ) {
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        index_identifier_occurrences(source, child, index);
    }
}

/// Answer "does any occurrence of `ident` fall outside `excluded`?" from the
/// index built by `index_identifier_occurrences`. Same rule as the retired
/// per-binding tree walk: the declaration's own name node is inside
/// `excluded` and thus never counts as an outside reference.
fn identifier_referenced_outside_index(
    index: &HashMap<&str, Vec<(usize, usize)>>,
    ident: &str,
    excluded: &std::ops::Range<usize>,
) -> bool {
    index.get(ident).is_some_and(|occurrences| {
        occurrences
            .iter()
            .any(|&(start, end)| start < excluded.start || end > excluded.end)
    })
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
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
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
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
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
    // (span, module id) per marker; `None` id = receiver we cannot
    // attribute to a flattened module slot (kept unconditionally).
    let mut markers: Vec<((usize, usize), Option<usize>)> = Vec::new();
    // Positions of `.__esModule` property READS (interop helper bodies).
    let mut reads: Vec<usize> = Vec::new();
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
            // Not a marker key. `<expr>.__esModule` is interop access:
            // a WRITE (`x.__esModule = true`) is just a marker in
            // expression form — kept, no demand; a READ creates demand
            // for the modules flowing into it. Anything else is an
            // unknown shape — keep every marker (old behavior).
            let preceded_by_dot = key_at > 0 && b[key_at - 1] == b'.';
            if !preceded_by_dot {
                return source.to_string();
            }
            let mut after = key_at + "__esModule".len();
            while after < b.len() && matches!(b[after], b' ' | b'\t') {
                after += 1;
            }
            let is_write = after < b.len()
                && b[after] == b'='
                && !(after + 1 < b.len() && b[after + 1] == b'=');
            if !is_write {
                reads.push(key_at);
            }
            continue;
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
        let Some(body_len) = es_module_marker_body_len(rest_trim) else {
            return source.to_string();
        };
        let stmt_end = key_end + consumed + body_len;

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
        let module_id = receiver_trim
            .strip_prefix("_m")
            .and_then(|r| r.strip_suffix(".exports"))
            .or_else(|| {
                receiver_trim
                    .strip_prefix("_m")
                    .and_then(|r| r.strip_suffix('e'))
            })
            .and_then(|digits| digits.parse::<usize>().ok())
            .or_else(|| {
                if matches!(receiver_trim, "module.exports" | "exports") {
                    module_id_for_position(source, prefix_at)
                } else {
                    None
                }
            });
        markers.push(((prefix_at, e), module_id));
    }

    if markers.is_empty() {
        return source.to_string();
    }

    // Demand analysis: a marker on `_mN.exports` is observable only when
    // module N's namespace flows into an `__esModule` read — directly
    // (`_r(N).__esModule`) or through an interop helper
    // (`helper(_r(N))` where helper's body reads the flag). Any read or
    // helper use we cannot attribute keeps every marker (old behavior).
    let mut demanded: HashSet<usize> = HashSet::new();
    let mut helper_names: HashSet<&str> = HashSet::new();
    for &read_at in &reads {
        // `.__esModule` — receiver text directly before the dot.
        let recv_end = read_at - 1;
        if let Some(id) = direct_marker_read_receiver(source, recv_end) {
            demanded.insert(id);
            continue;
        }
        match enclosing_function_name(source, b, read_at) {
            Some(name) => {
                helper_names.insert(name);
            }
            None => return source.to_string(),
        }
    }
    for name in helper_names {
        let mut helper_module_ids = HashSet::new();
        let mut at = 0usize;
        while let Some(rel) = source[at..].find(name) {
            let start = at + rel;
            at = start + name.len();
            // Token boundaries: skip substring hits inside longer names.
            let before_ok = start == 0 || !is_ident_byte(b[start - 1]);
            let after = start + name.len();
            let after_is_ident = after < b.len() && is_ident_byte(b[after]);
            if !before_ok || after_is_ident {
                continue;
            }
            // Definition site: `function NAME(`.
            if source[..start].trim_end().ends_with("function") {
                continue;
            }
            // Call site: NAME(<first-arg>) with a resolvable module arg.
            if after < b.len() && b[after] == b'(' {
                match first_call_arg_module_id(source, after) {
                    Some(id) => {
                        demanded.insert(id);
                        continue;
                    }
                    None => return source.to_string(),
                }
            }
            if let Some(id) = exported_helper_module_id(source, b, start, name) {
                helper_module_ids.insert(id);
                continue;
            }
            // Aliased / passed as a value — cannot trace, keep all.
            return source.to_string();
        }

        for id in helper_module_ids {
            let Some(ids) = exported_helper_demands(source, b, id) else {
                return source.to_string();
            };
            demanded.extend(ids);
        }
    }

    let removals: Vec<(usize, usize)> = markers
        .into_iter()
        .filter_map(|(span, module_id)| match module_id {
            // Reads exist but module N never flows into one → dead marker.
            Some(id) if !demanded.contains(&id) => Some(span),
            // Demanded, or unattributable receiver (`module.exports`):
            // observable, keep.
            Some(_) => None,
            None if reads.is_empty() => Some(span),
            None => None,
        })
        .collect();

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

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

/// `_r(N).__esModule` / `_mN.exports.__esModule` — resolve the read's
/// receiver to a flattened module id without helper-call tracing.
fn direct_marker_read_receiver(source: &str, recv_end: usize) -> Option<usize> {
    let before = &source[..recv_end];
    if let Some(open) = before.strip_suffix(')').and_then(|s| s.rfind("_r(")) {
        let digits = &before[open + 3..recv_end - 1];
        if !digits.is_empty() && digits.bytes().all(|d| d.is_ascii_digit()) {
            return digits.parse().ok();
        }
    }
    if let Some(rest) = before.strip_suffix(".exports") {
        let id_start = rest.rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '$'));
        let ident = match id_start {
            Some(i) => &rest[i + 1..],
            None => rest,
        };
        if let Some(digits) = ident.strip_prefix("_m") {
            if !digits.is_empty() && digits.bytes().all(|d| d.is_ascii_digit()) {
                return digits.parse().ok();
            }
        }
    }
    None
}

/// Walk backward from a read position to the function declaration that
/// encloses it, returning the function's name. Bails (None) on any
/// quote character — string contents would desynchronize the lexical
/// brace balance — and on anonymous enclosing functions.
fn enclosing_function_name<'a>(source: &'a str, b: &[u8], pos: usize) -> Option<&'a str> {
    let mut depth = 0i32;
    let mut i = pos;
    while i > 0 {
        i -= 1;
        match b[i] {
            b'"' | b'\'' | b'`' => return None,
            b'}' => depth += 1,
            b'{' => {
                if depth == 0 {
                    // Enclosing block. Function header? `function NAME(args) {`
                    if let Some(name) = function_header_name(source, b, i) {
                        return Some(name);
                    }
                    // Plain block (if/for body) — keep walking outward.
                } else {
                    depth -= 1;
                }
            }
            _ => {}
        }
    }
    None
}

/// If the `{` at `brace` closes a `function NAME(args)` header, return NAME.
fn function_header_name<'a>(source: &'a str, b: &[u8], brace: usize) -> Option<&'a str> {
    let mut i = brace;
    while i > 0 && matches!(b[i - 1], b' ' | b'\t' | b'\r' | b'\n') {
        i -= 1;
    }
    if i == 0 || b[i - 1] != b')' {
        return None;
    }
    let mut depth = 1i32;
    i -= 1;
    while i > 0 && depth > 0 {
        i -= 1;
        match b[i] {
            b')' => depth += 1,
            b'(' => depth -= 1,
            b'"' | b'\'' | b'`' => return None,
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    }
    let name_end = i;
    let mut name_start = name_end;
    while name_start > 0 && is_ident_byte(b[name_start - 1]) {
        name_start -= 1;
    }
    if name_start == name_end {
        return None;
    }
    if !source[..name_start].trim_end().ends_with("function") {
        return None;
    }
    Some(&source[name_start..name_end])
}

/// For a call `NAME(<args>)` with the `(` at `open`, resolve the first
/// argument to a flattened module id (`_r(N)`, `_mN`, `_mN.exports`).
fn first_call_arg_module_id(source: &str, open: usize) -> Option<usize> {
    let b = source.as_bytes();
    let mut depth = 1i32;
    let mut i = open + 1;
    let arg_start = i;
    while i < b.len() {
        match b[i] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            b',' if depth == 1 => break,
            b'"' | b'\'' | b'`' => return None,
            _ => {}
        }
        i += 1;
    }
    let arg = source[arg_start..i].trim();
    if let Some(inner) = arg.strip_prefix("_r(").and_then(|a| a.strip_suffix(')')) {
        if !inner.is_empty() && inner.bytes().all(|d| d.is_ascii_digit()) {
            return inner.parse().ok();
        }
        return None;
    }
    let bare = arg.strip_suffix(".exports").unwrap_or(arg);
    if let Some(digits) = bare.strip_prefix("_m") {
        if !digits.is_empty() && digits.bytes().all(|d| d.is_ascii_digit()) {
            return digits.parse().ok();
        }
    }
    None
}

fn exported_helper_module_id(source: &str, b: &[u8], start: usize, name: &str) -> Option<usize> {
    let before = source[..start].trim_end();
    if !(before.ends_with("module.exports =")
        || before.ends_with("module.exports=")
        || before.ends_with(".exports =")
        || before.ends_with(".exports="))
    {
        return None;
    }

    let mut after = start + name.len();
    while after < b.len() && matches!(b[after], b' ' | b'\t') {
        after += 1;
    }
    if after < b.len() && !matches!(b[after], b',' | b';' | b'\n' | b'\r') {
        return None;
    }

    module_id_for_position(source, start)
}

fn module_id_for_position(source: &str, pos: usize) -> Option<usize> {
    let marker = source[..pos].rfind("// Module ")?;
    let start = marker + "// Module ".len();
    let mut end = start;
    let b = source.as_bytes();
    while end < pos && b[end].is_ascii_digit() {
        end += 1;
    }
    if start == end || b.get(end) != Some(&b':') {
        return None;
    }
    source[start..end].parse().ok()
}

fn exported_helper_demands(source: &str, b: &[u8], helper_id: usize) -> Option<HashSet<usize>> {
    let needle = format!("_r({helper_id})");
    let mut aliases: HashSet<String> = HashSet::new();
    let mut demanded = HashSet::new();
    let mut at = 0usize;

    while let Some(rel) = source[at..].find(&needle) {
        let start = at + rel;
        at = start + needle.len();
        if source[at..]
            .bytes()
            .next()
            .map(|byte| byte.is_ascii_digit())
            .unwrap_or(false)
        {
            continue;
        }
        if at < b.len() && b[at] == b'(' {
            demanded.insert(first_call_arg_module_id(source, at)?);
            continue;
        }
        if let Some(alias) = require_alias_name(source, b, start) {
            aliases.insert(alias);
            continue;
        }
        return None;
    }

    for alias in aliases {
        demanded.extend(alias_demands(source, b, &alias)?);
    }

    Some(demanded)
}

fn require_alias_name(source: &str, b: &[u8], require_start: usize) -> Option<String> {
    let stmt_start = source[..require_start]
        .rfind(|c| matches!(c, ';' | '{' | '}' | '\n'))
        .map(|idx| idx + 1)
        .unwrap_or(0);
    let prefix = source[stmt_start..require_start].trim();
    let rest = ["var ", "let ", "const "]
        .iter()
        .find_map(|keyword| prefix.strip_prefix(keyword))?;
    let (name, tail) = rest.split_once('=')?;
    if !tail.trim().is_empty() {
        return None;
    }
    let name = name.trim();
    if !is_ident(name.as_bytes()) {
        return None;
    }
    let mut after = require_start;
    while after < b.len() && b[after] != b';' && b[after] != b'\n' {
        after += 1;
    }
    Some(name.to_string())
}

fn alias_demands(source: &str, b: &[u8], alias: &str) -> Option<HashSet<usize>> {
    let mut demanded = HashSet::new();
    let mut at = 0usize;

    while let Some(rel) = source[at..].find(alias) {
        let start = at + rel;
        at = start + alias.len();
        let before_ok = start == 0 || !is_ident_byte(b[start - 1]);
        let after_is_ident = at < b.len() && is_ident_byte(b[at]);
        if !before_ok || after_is_ident {
            continue;
        }
        if is_alias_declaration_lhs(source, b, start, alias) {
            continue;
        }
        let mut call = at;
        while call < b.len() && matches!(b[call], b' ' | b'\t') {
            call += 1;
        }
        if call < b.len() && b[call] == b'(' {
            demanded.insert(first_call_arg_module_id(source, call)?);
            continue;
        }
        return None;
    }

    Some(demanded)
}

fn is_alias_declaration_lhs(source: &str, b: &[u8], start: usize, alias: &str) -> bool {
    let stmt_start = source[..start]
        .rfind(|c| matches!(c, ';' | '{' | '}' | '\n'))
        .map(|idx| idx + 1)
        .unwrap_or(0);
    let before = source[stmt_start..start].trim_end();
    if !matches!(before, "var" | "let" | "const") {
        return false;
    }
    let mut after = start + alias.len();
    while after < b.len() && matches!(b[after], b' ' | b'\t') {
        after += 1;
    }
    after < b.len() && b[after] == b'='
}

fn is_ident(bytes: &[u8]) -> bool {
    let Some((first, rest)) = bytes.split_first() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || *first == b'_' || *first == b'$') {
        return false;
    }
    rest.iter().all(|byte| is_ident_byte(*byte))
}

fn es_module_marker_body_len(source: &str) -> Option<usize> {
    let b = source.as_bytes();
    let mut i = 0usize;
    skip_ascii_ws(b, &mut i);
    if b.get(i) != Some(&b'{') {
        return None;
    }
    i += 1;
    skip_ascii_ws(b, &mut i);
    if !source[i..].starts_with("value") {
        return None;
    }
    i += "value".len();
    skip_ascii_ws(b, &mut i);
    if b.get(i) != Some(&b':') {
        return None;
    }
    i += 1;
    skip_ascii_ws(b, &mut i);
    if source[i..].starts_with("true") {
        i += "true".len();
    } else if source[i..].starts_with("!0") {
        i += "!0".len();
    } else {
        return None;
    }
    skip_ascii_ws(b, &mut i);
    if b.get(i) != Some(&b'}') {
        return None;
    }
    i += 1;
    skip_ascii_ws(b, &mut i);
    if !source[i..].starts_with(");") {
        return None;
    }
    Some(i + 2)
}

fn skip_ascii_ws(bytes: &[u8], offset: &mut usize) {
    while *offset < bytes.len() && matches!(bytes[*offset], b' ' | b'\t' | b'\r' | b'\n') {
        *offset += 1;
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
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
    node: Node<'_>,
    identifier_index: &HashMap<&str, Vec<(usize, usize)>>,
    side_effect_free_module_ids: &HashSet<usize>,
    edits: &mut Vec<StaticEdit>,
) {
    match node.kind() {
        "variable_declaration" | "lexical_declaration" => {
            if let Some(edit) = unused_require_binding_edit(
                source,
                node,
                identifier_index,
                side_effect_free_module_ids,
            ) {
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
            child,
            identifier_index,
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
    declaration: Node<'_>,
    identifier_index: &HashMap<&str, Vec<(usize, usize)>>,
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

    if identifier_referenced_outside_index(identifier_index, ident, &declaration.byte_range()) {
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
            // GH #1930 — `--splitting` lowers `import(spec)` to
            // `__jet__.dynamicImport(id)` instead of `require(id)`. Without
            // this alias, the entry-reachability rescue pass below can't see
            // that a numeric id is still referenced, so async-chunk-only
            // modules get pruned as "unreachable" before code splitting ever
            // sees them (empty chunk bodies, empty `moduleChunks`).
            if function_text == "require"
                || function_text == "_r"
                || function_text == "__jet__.dynamicImport"
            {
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

/// #1995 — cheap textual pre-check gating
/// `eliminate_unused_side_effect_free_require_bindings`, which
/// tree-sitter-parses its input unconditionally even though every
/// candidate binding it can remove is a require-like call. Mirrors
/// [`collect_numeric_require_ids`]'s exact call-target set so it stays
/// sound if that matcher ever grows a new alias: `require(...)`, the
/// mangled `_r(...)` alias, and the splitting-lowered
/// `__jet__.dynamicImport(...)` (GH #1930 — `transform/modules.rs` lowers
/// a resolved dynamic `import()` directly into a module's own transform
/// output when `--splitting` is on, so this form is a real per-module,
/// pre-bundle occurrence and not only a post-mangle one).
pub(crate) fn could_contain_require_like_call(code: &str) -> bool {
    code.contains("require(") || code.contains("_r(") || code.contains("__jet__.dynamicImport(")
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

/// #1995 — cheap textual pre-check for whether `code` could contain a
/// statically foldable condition, gating the `fold_define_short_circuits`
/// + `eliminate_static_conditionals_syntax` pair (both tree-sitter-parse
/// their input unconditionally). Sound relative to every shape
/// [`eval_condition`] recognizes:
///   - literal string compares (`"a"==="b"`) — checking bare `==`/`!=`
///     covers all four operator spellings, since `===`/`!==` both contain
///     `==` as a substring and the remaining `!=` case is covered
///     directly.
///   - literal bool compares (`true===false`) — bare `true`/`false`
///     tokens.
///   - a bare `true`/`false`/`!0`/`!1` condition on its own (e.g.
///     hand-written `if (false) { … }`, or the minified form
///     `fold_define_short_circuits` itself produces — pre-minified vendor
///     sources can already contain `!0`/`!1` before defines ever touch
///     them).
/// Overapproximate on purpose: a false positive just runs the pass and
/// finds nothing to fold; a false negative would silently drop dead-code
/// elimination the pipeline performed unconditionally before this gate
/// existed, so every disjunct above is required for soundness, not just
/// the common define-substitution case.
///
/// One known, universal false-positive source is stripped before the scan:
/// `transform/modules.rs` prepends the fixed, non-conditional
/// `__esModule` CJS-interop marker (see [`ESMODULE_PROLOGUE`]) onto every
/// ESM-syntax module's transform output, and that marker's own `{ value:
/// true }` would otherwise trip the bare-`true` disjunct on essentially
/// every module in an ESM codebase, making the gate a near-permanent
/// no-op in practice (confirmed against the #1947-style synthetic
/// barrel-heavy timing fixture: 1006/1006 modules ran the pass before this
/// strip, 0/1006 after). The marker is a single statement — never a
/// shape `eval_condition` folds — so removing it before the probe cannot
/// hide a real foldable condition.
pub(crate) fn could_fold_static_conditional(code: &str) -> bool {
    let code = code.strip_prefix(ESMODULE_PROLOGUE).unwrap_or(code);
    code.contains("==")
        || code.contains("!=")
        || code.contains("!0")
        || code.contains("!1")
        || code.contains("true")
        || code.contains("false")
}

/// The fixed, verbatim `__esModule` CJS-interop marker
/// `transform/modules.rs`'s `transform_modules_with_dir_index_and_tree`
/// prepends onto every ESM-syntax module's transform output (never
/// data-dependent — always this exact statement when present). Shared
/// with [`could_fold_static_conditional`]'s prologue strip; mirrors the
/// same fixed-shape assumption `eliminate_unread_es_module_markers`
/// above already makes about this marker's generated text.
const ESMODULE_PROLOGUE: &str =
    "Object.defineProperty(module.exports, \"__esModule\", { value: true });\n";

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

// ---------------------------------------------------------------------
// WI #2126 — statement-level DCE for retained modules.
//
// `shake_module` (tree_shake.rs) prunes ESM `export { a, b }` clause
// members that are unused, but real vendor packages ship pre-built CJS
// (Babel output): every named export gets an *unconditional*
// `exports.NAME = value;` assignment regardless of downstream usage —
// `shake_module`'s ESM-syntax-only matching cannot see these at all, so
// the function/class/const bodies backing unused CJS exports (and any
// helper only those unused exports call) survive tree-shaking untouched.
//
// This pass closes that gap by parsing each retained module body's
// top-level statements and running a conservative mark-and-sweep over
// declaration-name references:
//   - seeds  = identifiers referenced by any "always-live" statement
//     (anything that is not a prunable-shaped declaration — including a
//     CJS export assignment whose target is in `used_exports`)
//   - reachability propagates transitively through live declarations'
//     own bodies (a live decl's callees become live; the reverse is not
//     true — a decl is not made live merely because it calls something
//     that happens to be live)
//   - anything never reached is spliced out
//
// Liveness lookups are a scope-blind, comment/string-INCLUSIVE raw-text
// word scan (`word_occurs_in_span`): over-approximating "is this name
// referenced anywhere in this span" is always safe here because the
// worst case is merely a missed opportunity to prune, never an incorrect
// deletion. This deliberately folds what the original spike described as
// two mechanisms (an AST-filtered identifier index, plus a separate
// string-literal-inclusive scan for the keep-decision) into one, since a
// raw-text scan already subsumes both and the module explicitly favors
// "pick the simple sound rule" over mirroring a two-mechanism design.
//
// Declarations are only ever considered prunable when they have one of a
// small set of side-effect-free shapes (see
// `is_side_effect_free_initializer_kind`); anything else — notably a
// `require(...)` call initializer, since dropping an unreferenced
// `require()` binding could silently skip a side-effecting module load —
// falls back to the always-live bucket. `is_module_flatten_safe` (shared
// with scope-hoisting) gates the whole module out of this pass on
// `eval(`/`with(`/`arguments[` — those constructs can reference any
// top-level binding by name at runtime in ways no static scan can see.

/// One prunable top-level declaration: a function/class declaration, or a
/// `var`/`let`/`const` statement whose declarator(s) all have a
/// side-effect-free shape (see [`is_side_effect_free_initializer_kind`]).
/// `names` holds every name bound by the statement — for a multi-declarator
/// `var a = 1, b = 2;` this is `["a", "b"]`, and the two are kept or pruned
/// atomically as one unit (conservative: any one live binding keeps the
/// whole statement, including its sibling declarators).
struct TopLevelDecl {
    names: Vec<String>,
    span: std::ops::Range<usize>,
}

/// A CJS `exports.NAME = ...;` / `module.exports.NAME = ...;` /
/// `exports["NAME"] = ...;` / `module.exports["NAME"] = ...;` assignment
/// statement whose liveness is decided purely by `used_exports` membership
/// (not by any reference scan) — this is the shape Babel emits
/// unconditionally for every named export.
struct ConditionalExportAssignment {
    name: String,
    span: std::ops::Range<usize>,
}

/// Result of [`eliminate_dead_top_level_declarations`].
pub(crate) struct StmtDceOutcome {
    pub(crate) code: String,
    pub(crate) pruned_decls: usize,
    pub(crate) pruned_bytes: usize,
    pub(crate) skipped_vendor: bool,
}

impl StmtDceOutcome {
    fn unchanged(source: &str) -> Self {
        StmtDceOutcome {
            code: source.to_string(),
            pruned_decls: 0,
            pruned_bytes: 0,
            skipped_vendor: false,
        }
    }

    fn skipped(source: &str) -> Self {
        StmtDceOutcome {
            code: source.to_string(),
            pruned_decls: 0,
            pruned_bytes: 0,
            skipped_vendor: true,
        }
    }
}

// A module at least this large AND this dense (see
// `STMT_DCE_VENDOR_AVG_LINE_LEN_THRESHOLD`) is heuristically treated as an
// already-minified, pre-shaken single-file vendor bundle: parsing +
// scanning it for statement-level DCE has a real cost and single-file
// minified vendor code is already about as small as it is going to get.
// Skipping only costs opportunity, never correctness, so the rule is
// deliberately cheap and self-contained (size + average-line-length only —
// no cross-module "is this the only retained module in its package"
// plumbing).
const STMT_DCE_VENDOR_SIZE_THRESHOLD: usize = 100_000;
const STMT_DCE_VENDOR_AVG_LINE_LEN_THRESHOLD: usize = 500;

fn looks_like_minified_vendor_bundle(source: &str) -> bool {
    if source.len() < STMT_DCE_VENDOR_SIZE_THRESHOLD {
        return false;
    }
    let newline_count = source.bytes().filter(|&b| b == b'\n').count();
    let avg_line_len = source.len() / newline_count.max(1);
    avg_line_len >= STMT_DCE_VENDOR_AVG_LINE_LEN_THRESHOLD
}

/// Declarator initializer node kinds that can never themselves be a
/// meaningful runtime side effect, so a `var`/`let`/`const` statement made
/// up only of declarators with these initializer kinds (or no initializer
/// at all) is eligible for pruning. Anything else — most importantly a
/// `call_expression` such as `require(...)` — keeps the whole statement in
/// the always-live bucket, because dropping an unreferenced binding could
/// silently skip a side-effecting call.
fn is_side_effect_free_initializer_kind(kind: &str) -> bool {
    matches!(
        kind,
        "arrow_function"
            | "function"
            | "function_expression"
            | "generator_function"
            | "class"
            | "number"
            | "string"
            | "true"
            | "false"
            | "null"
    )
}

/// Extracts the exported name from a CJS export-assignment statement's
/// source text, e.g. `exports.foo = foo;` -> `Some("foo")`,
/// `module.exports["bar"] = bar;` -> `Some("bar")`. Returns `None` both for
/// non-export-assignment statements and for chained assignments such as
/// `exports.a = exports.b = value;`, whose right-hand side is itself
/// another export target — `is_chained_export_assignment` rejects those so
/// the (rare) chained shape always falls back to the safe always-live
/// bucket rather than risking mis-attributing the statement to only its
/// first target name.
fn cjs_export_assignment_name(stmt_text: &str) -> Option<&str> {
    for prefix in ["module.exports.", "exports."] {
        let Some(rest) = stmt_text.strip_prefix(prefix) else {
            continue;
        };
        let Some(end) = rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '$'))
        else {
            continue;
        };
        if end == 0 {
            continue;
        }
        let name = &rest[..end];
        let Some(rhs) = rest[end..].trim_start().strip_prefix('=') else {
            continue;
        };
        if rhs.starts_with('=') {
            // `===`/`==` is a comparison, not an assignment.
            continue;
        }
        if is_chained_export_assignment(rhs.trim_start()) {
            return None;
        }
        return Some(name);
    }
    for prefix in ["module.exports[\"", "exports[\""] {
        let Some(rest) = stmt_text.strip_prefix(prefix) else {
            continue;
        };
        let Some(end) = rest.find('"') else {
            continue;
        };
        if end == 0 {
            continue;
        }
        let name = &rest[..end];
        let Some(after_bracket) = rest[end + 1..].trim_start().strip_prefix(']') else {
            continue;
        };
        let Some(rhs) = after_bracket.trim_start().strip_prefix('=') else {
            continue;
        };
        if rhs.starts_with('=') {
            continue;
        }
        if is_chained_export_assignment(rhs.trim_start()) {
            return None;
        }
        return Some(name);
    }
    None
}

fn is_chained_export_assignment(rhs: &str) -> bool {
    rhs.starts_with("exports.")
        || rhs.starts_with("exports[\"")
        || rhs.starts_with("module.exports.")
        || rhs.starts_with("module.exports[\"")
}

/// Classifies one top-level statement node into either a prunable
/// declaration (`decls`), a `used_exports`-gated CJS export assignment
/// (`conditional`), or an always-live statement (`root_spans`) — every
/// statement lands in exactly one bucket.
fn classify_top_level_statement(
    source: &str,
    node: Node<'_>,
    used_exports: &HashSet<String>,
    decls: &mut Vec<TopLevelDecl>,
    conditional: &mut Vec<ConditionalExportAssignment>,
    root_spans: &mut Vec<std::ops::Range<usize>>,
) {
    match node.kind() {
        "function_declaration" | "generator_function_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                if name_node.kind() == "identifier" {
                    decls.push(TopLevelDecl {
                        names: vec![source[name_node.byte_range()].to_string()],
                        span: node.byte_range(),
                    });
                    return;
                }
            }
        }
        "class_declaration" | "abstract_class_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                if name_node.kind() == "identifier" {
                    decls.push(TopLevelDecl {
                        names: vec![source[name_node.byte_range()].to_string()],
                        span: node.byte_range(),
                    });
                    return;
                }
            }
        }
        "variable_declaration" | "lexical_declaration" => {
            let mut cursor = node.walk();
            let declarators: Vec<Node<'_>> = node
                .named_children(&mut cursor)
                .filter(|c| c.kind() == "variable_declarator")
                .collect();
            if !declarators.is_empty() {
                let mut names = Vec::with_capacity(declarators.len());
                let mut all_safe = true;
                for declarator in &declarators {
                    let Some(name_node) = declarator.child_by_field_name("name") else {
                        all_safe = false;
                        break;
                    };
                    if name_node.kind() != "identifier" {
                        // Destructuring (`object_pattern`/`array_pattern`) —
                        // conservatively always-live rather than teaching
                        // this pass a second binding-name extractor.
                        all_safe = false;
                        break;
                    }
                    let value_ok = match declarator.child_by_field_name("value") {
                        None => true,
                        Some(v) => is_side_effect_free_initializer_kind(v.kind()),
                    };
                    if !value_ok {
                        all_safe = false;
                        break;
                    }
                    names.push(source[name_node.byte_range()].to_string());
                }
                if all_safe && !names.is_empty() {
                    decls.push(TopLevelDecl {
                        names,
                        span: node.byte_range(),
                    });
                    return;
                }
            }
        }
        "expression_statement" => {
            let text = source[node.byte_range()].trim();
            if let Some(name) = cjs_export_assignment_name(text) {
                if used_exports.contains(name) {
                    root_spans.push(node.byte_range());
                }
                conditional.push(ConditionalExportAssignment {
                    name: name.to_string(),
                    span: node.byte_range(),
                });
                return;
            }
        }
        _ => {}
    }
    root_spans.push(node.byte_range());
}

/// Scope-blind, comment/string-inclusive whole-word occurrence check: is
/// `name` present anywhere in `source[span]`, bounded by non-identifier
/// bytes (or the span edges) on both sides? Deliberately does not skip
/// `string`/`comment` node contents — a name mentioned only inside a
/// string (e.g. `window["riskyImpl"]`) or a comment must still count as a
/// live reference, because over-keeping is safe and under-keeping is not.
fn word_occurs_in_span(source: &str, name: &str, span: &std::ops::Range<usize>) -> bool {
    if name.is_empty() || span.start >= span.end || span.end > source.len() {
        return false;
    }
    let slice = &source[span.start..span.end];
    let bytes = slice.as_bytes();
    for (start, _) in slice.match_indices(name) {
        let end = start + name.len();
        let before_ok = start == 0 || !is_ident_byte(bytes[start - 1]);
        let after_ok = end >= bytes.len() || !is_ident_byte(bytes[end]);
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

/// Statement-level dead-code elimination for one already-transformed,
/// CJS-lowered module body: drops top-level function/class/var/let/const
/// declarations (and CJS export assignments) that are unreachable from
/// `used_exports` per the conservative mark-and-sweep documented above the
/// struct definitions in this section. Returns the original source
/// unchanged (`StmtDceOutcome::unchanged`) whenever the module contains a
/// parse error, `eval`/`with`/`arguments[`, or a `used_exports` `"*"`
/// (whole-namespace) marker, and `StmtDceOutcome::skipped` when the module
/// looks like an already-minified vendor bundle
/// (`looks_like_minified_vendor_bundle`). The re-parse-guard-after-splice
/// pattern mirrors `eliminate_unused_side_effect_free_require_bindings`
/// above: if the post-splice text fails to re-parse cleanly, the original
/// source is returned untouched rather than risking corrupted output.
pub(crate) fn eliminate_dead_top_level_declarations(
    source: &str,
    used_exports: &HashSet<String>,
) -> StmtDceOutcome {
    if used_exports.contains("*") {
        return StmtDceOutcome::unchanged(source);
    }
    if looks_like_minified_vendor_bundle(source) {
        return StmtDceOutcome::skipped(source);
    }
    let Some(tree) = parse_js(source) else {
        return StmtDceOutcome::unchanged(source);
    };
    let root = tree.root_node();
    if root.has_error() {
        return StmtDceOutcome::unchanged(source);
    }
    if !scope_hoist::is_module_flatten_safe(source) {
        return StmtDceOutcome::unchanged(source);
    }

    let mut decls: Vec<TopLevelDecl> = Vec::new();
    let mut conditional: Vec<ConditionalExportAssignment> = Vec::new();
    let mut root_spans: Vec<std::ops::Range<usize>> = Vec::new();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        classify_top_level_statement(
            source,
            child,
            used_exports,
            &mut decls,
            &mut conditional,
            &mut root_spans,
        );
    }
    if decls.is_empty() {
        return StmtDceOutcome::unchanged(source);
    }

    // Seed: decls referenced by any always-live statement.
    let mut live = vec![false; decls.len()];
    for (i, decl) in decls.iter().enumerate() {
        if decl.names.iter().any(|name| {
            root_spans
                .iter()
                .any(|span| word_occurs_in_span(source, name, span))
        }) {
            live[i] = true;
        }
    }
    // Fixed-point propagation: a live decl's own body can reference other
    // decls, which become live in turn (direction matters — a decl is
    // never marked live merely because it calls something live).
    loop {
        let mut changed = false;
        for i in 0..decls.len() {
            if live[i] {
                continue;
            }
            let reached = (0..decls.len()).any(|j| {
                j != i
                    && live[j]
                    && decls[i]
                        .names
                        .iter()
                        .any(|name| word_occurs_in_span(source, name, &decls[j].span))
            });
            if reached {
                live[i] = true;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let mut edits: Vec<StaticEdit> = Vec::new();
    let mut pruned_decls = 0usize;
    for (i, decl) in decls.iter().enumerate() {
        if !live[i] {
            edits.push(StaticEdit {
                start: decl.span.start,
                end: decl.span.end,
                replacement: String::new(),
            });
            pruned_decls += 1;
        }
    }
    for cond in &conditional {
        if !used_exports.contains(cond.name.as_str()) {
            edits.push(StaticEdit {
                start: cond.span.start,
                end: cond.span.end,
                replacement: String::new(),
            });
        }
    }
    if edits.is_empty() {
        return StmtDceOutcome::unchanged(source);
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
    let pruned_bytes: usize = filtered.iter().map(|edit| edit.end - edit.start).sum();
    let mut out = source.to_string();
    for edit in filtered.into_iter().rev() {
        out.replace_range(edit.start..edit.end, "");
    }
    if parse_js(&out)
        .map(|tree| tree.root_node().has_error())
        .unwrap_or(true)
    {
        return StmtDceOutcome::unchanged(source);
    }
    StmtDceOutcome {
        code: out,
        pruned_decls,
        pruned_bytes,
        skipped_vendor: false,
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

    // #1995 — gate-probe soundness: every shape `eval_condition` can fold
    // must trip `could_fold_static_conditional`, or the transform_modules
    // pass gate would silently skip live dead-code elimination.

    #[test]
    fn test_could_fold_static_conditional_catches_string_compare() {
        assert!(could_fold_static_conditional(
            r#"if ("production" !== "production") { dead(); }"#
        ));
        assert!(could_fold_static_conditional(
            r#"if ("production" === "production") { live(); }"#
        ));
    }

    #[test]
    fn test_could_fold_static_conditional_catches_bool_compare() {
        assert!(could_fold_static_conditional(
            "if (false === false) { live(); }"
        ));
        assert!(could_fold_static_conditional(
            "if (true !== false) { live(); }"
        ));
    }

    #[test]
    fn test_could_fold_static_conditional_catches_bare_literal_and_minified_forms() {
        // Hand-written / vendor dead-branch guards with no comparison
        // operator and no relation to any configured define.
        assert!(could_fold_static_conditional("if (false) { dead(); }"));
        assert!(could_fold_static_conditional("if (true) { live(); }"));
        assert!(could_fold_static_conditional("cond ? true : false"));
        // Pre-minified vendor input that already uses the `!0`/`!1` forms
        // `fold_define_short_circuits` itself would otherwise produce.
        assert!(could_fold_static_conditional("if (!0) { live(); }"));
        assert!(could_fold_static_conditional("if (!1) { dead(); }"));
    }

    #[test]
    fn test_could_fold_static_conditional_skips_plain_code() {
        // No comparison operator, no boolean literal anywhere: neither
        // `fold_define_short_circuits` nor `eliminate_static_conditionals_syntax`
        // could act on this module, so the gate should be able to skip it.
        assert!(!could_fold_static_conditional(
            "function add(a, b) { return a + b; } export default add;"
        ));
    }

    #[test]
    fn test_could_fold_static_conditional_strips_esmodule_prologue_false_positive() {
        // Every ESM-syntax module's transform output is prefixed with the
        // fixed `__esModule` CJS-interop marker (transform/modules.rs),
        // whose own `{ value: true }` would otherwise trip the bare-`true`
        // disjunct on essentially every module in an ESM codebase — the
        // synthetic #1947-style timing fixture measured 1006/1006 modules
        // running the pass before this strip existed. A plain module body
        // with nothing else foldable must be skippable once the known
        // prologue is discounted.
        assert!(!could_fold_static_conditional(
            "Object.defineProperty(module.exports, \"__esModule\", { value: true });\n\
             function add(a, b) { return a + b; }\n\
             module.exports.add = add;"
        ));
        // A genuine foldable condition anywhere after the prologue must
        // still trip the probe — stripping the marker must not hide real
        // analysis surface.
        assert!(could_fold_static_conditional(
            "Object.defineProperty(module.exports, \"__esModule\", { value: true });\n\
             if (\"production\" === \"production\") { live(); }"
        ));
        // A module that merely CONTAINS `__esModule` handling without the
        // exact fixed prologue shape (e.g. code that reads the flag, not
        // jet's own generated marker) is not stripped, so its own literal
        // `true` still trips the probe — the strip only ever removes the
        // one exact, known, non-conditional statement.
        assert!(could_fold_static_conditional(
            "if (mod.__esModule === true) { live(); }"
        ));
    }

    // #1995 — gate-probe soundness for `eliminate_unused_side_effect_free_require_bindings`:
    // every call-target text `collect_numeric_require_ids` recognizes must
    // trip `could_contain_require_like_call`.

    #[test]
    fn test_could_contain_require_like_call_catches_all_aliases() {
        assert!(could_contain_require_like_call(
            "var PropTypes = require(7);"
        ));
        assert!(could_contain_require_like_call("var PropTypes = _r(7);"));
        assert!(could_contain_require_like_call(
            "__jet__.dynamicImport(7).then(function (m) { return m; });"
        ));
    }

    #[test]
    fn test_could_contain_require_like_call_skips_plain_code() {
        assert!(!could_contain_require_like_call(
            "function add(a, b) { return a + b; } export default add;"
        ));
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

    // #1995 — gate/pass agreement: whenever `could_contain_require_like_call`
    // says "run the pass", `eliminate_unused_side_effect_free_require_bindings`
    // must still prune every alias form `collect_numeric_require_ids`
    // recognizes, not just the plain `require(...)` spelling.

    #[test]
    fn test_gate2_probe_and_pass_agree_on_mangled_alias_require_binding() {
        let input = r#"var PropTypes = _r(7)["default"] || _r(7);
const value = 1;
module.exports["value"] = value;"#;
        assert!(
            could_contain_require_like_call(input),
            "probe must recognize the `_r(` alias form: {input}"
        );
        let output =
            eliminate_unused_side_effect_free_require_bindings(input, &HashSet::from([7usize]));
        assert!(!output.contains("PropTypes"), "{}", output);
        assert!(!output.contains("_r(7)"), "{}", output);
        assert!(output.contains("module.exports"), "{}", output);
    }

    #[test]
    fn test_gate2_probe_and_pass_agree_on_dynamic_import_lowering_alias() {
        let input = r#"var Chunk = __jet__.dynamicImport(7)["default"] || __jet__.dynamicImport(7);
const value = 1;
module.exports["value"] = value;"#;
        assert!(
            could_contain_require_like_call(input),
            "probe must recognize the `__jet__.dynamicImport(` alias form: {input}"
        );
        let output =
            eliminate_unused_side_effect_free_require_bindings(input, &HashSet::from([7usize]));
        assert!(!output.contains("Chunk"), "{}", output);
        assert!(output.contains("module.exports"), "{}", output);
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
    fn test_unused_require_binding_referenced_only_in_string_is_removed() {
        // The binding name appears verbatim inside a string literal, which is
        // not a real reference: `string` nodes are excluded from the
        // identifier occurrence index, same as the retired per-binding walk.
        let input = r#"var _Foo = require(7)["default"] || require(7);
const label = "_Foo";
module.exports["label"] = label;"#;
        let output =
            eliminate_unused_side_effect_free_require_bindings(input, &HashSet::from([7usize]));
        assert!(!output.contains("require(7)"), "{}", output);
        assert!(!output.contains("var _Foo"), "{}", output);
        assert!(output.contains("\"_Foo\""), "{}", output);
    }

    #[test]
    fn test_require_binding_referenced_in_template_substitution_is_kept_by_index() {
        let input = r#"var _Foo = require(7)["default"] || require(7);
const label = `${_Foo}`;
module.exports["label"] = label;"#;
        let output =
            eliminate_unused_side_effect_free_require_bindings(input, &HashSet::from([7usize]));
        assert!(output.contains("_Foo"), "{}", output);
        assert!(output.contains("require(7)"), "{}", output);
    }

    #[test]
    fn test_require_binding_referenced_via_shorthand_property_is_kept_by_index() {
        let input = r#"var _Foo = require(7)["default"] || require(7);
const bag = { _Foo };
module.exports["bag"] = bag;"#;
        let output =
            eliminate_unused_side_effect_free_require_bindings(input, &HashSet::from([7usize]));
        assert!(output.contains("_Foo"), "{}", output);
        assert!(output.contains("require(7)"), "{}", output);
    }

    #[test]
    fn test_mixed_module_keeps_referenced_binding_removes_unreferenced_binding() {
        let input = r#"var _Used = require(7)["default"] || require(7);
var _Unused = require(8)["default"] || require(8);
const value = _Used.thing;
module.exports["value"] = value;"#;
        let output = eliminate_unused_side_effect_free_require_bindings(
            input,
            &HashSet::from([7usize, 8usize]),
        );
        assert!(output.contains("_Used"), "{}", output);
        assert!(output.contains("require(7)"), "{}", output);
        assert!(!output.contains("_Unused"), "{}", output);
        assert!(!output.contains("require(8)"), "{}", output);
    }

    #[test]
    fn test_mui_icons_material_shaped_barrel_keeps_every_referenced_binding() {
        // Mirrors @mui/icons-material's real barrel shape (issue #1894): every
        // `_IconN` require binding is read back inside its own lazy-getter
        // `Object.defineProperty` re-export, so the algorithm must keep all
        // of them even though each is a single-declarator side-effect-free
        // require binding — the exact shape that made the retired
        // per-binding O(bindings * AST nodes) walk quadratic.
        const COUNT: usize = 200;
        let mut input = String::new();
        let mut side_effect_free_ids: HashSet<usize> = HashSet::with_capacity(COUNT);
        for i in 0..COUNT {
            side_effect_free_ids.insert(i);
            input.push_str(&format!(
                "var _Icon{i} = _interopRequireDefault(require({i}));\n"
            ));
            input.push_str(&format!(
                "Object.defineProperty(exports, \"Icon{i}\", {{ get: function () {{ return _Icon{i}.default; }} }});\n"
            ));
        }

        let output =
            eliminate_unused_side_effect_free_require_bindings(&input, &side_effect_free_ids);

        for i in 0..COUNT {
            assert!(
                output.contains(&format!("_Icon{i} = _interopRequireDefault(require({i}))")),
                "binding _Icon{i} must be kept (referenced in its getter)"
            );
        }
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

    /// Demand-driven path: a named helper reading `__esModule` with one
    /// traceable call keeps only the demanded module's marker.
    #[test]
    fn test_markers_kept_only_for_modules_flowing_into_interop_reads() {
        let input = r#"function _m9__interopRequireWildcard(e, r) { if (!r && e && e.__esModule) return e; }
var ns = _m9__interopRequireWildcard(_r(364));
Object.defineProperty(_m1.exports, "__esModule", { value: true });
Object.defineProperty(_m364.exports, "__esModule", { value: true });
"#;
        let output = eliminate_unread_es_module_markers(input);
        assert!(
            !output.contains("_m1.exports, \"__esModule\""),
            "undemanded marker must drop: {output}"
        );
        assert!(
            output.contains("_m364.exports, \"__esModule\""),
            "demanded marker must stay: {output}"
        );
    }

    #[test]
    fn test_multiline_exports_alias_es_module_marker_is_removed() {
        let input = r#"// Module 7: node_modules/pkg/index.js
{
var _m7e=_m7.exports;
Object.defineProperty(_m7e, "__esModule", {
  value: true
});
_m7e.value = 1;
}"#;
        let output = eliminate_unread_es_module_markers(input);
        assert!(!output.contains("__esModule"), "{output}");
        assert!(output.contains("_m7e.value = 1"), "{output}");
    }

    #[test]
    fn test_wrapper_module_exports_marker_uses_module_banner_for_demand() {
        let input = r#"function helper(e) { return e && e.__esModule ? e : { "default": e }; }
var wrapped = helper(_r(2));
// Module 1: node_modules/pkg/a.js
!function(module,exports,require){Object.defineProperty(module.exports, "__esModule", { value: true });
module.exports.value = 1;
}(_m1,_m1.exports,_r);
// Module 2: node_modules/pkg/b.js
!function(module,exports,require){Object.defineProperty(module.exports, "__esModule", { value: true });
module.exports.value = 2;
}(_m2,_m2.exports,_r);
"#;
        let output = eliminate_unread_es_module_markers(input);
        assert!(
            !output.contains("Module 1: node_modules/pkg/a.js\n!function(module,exports,require){Object.defineProperty"),
            "undemanded wrapper marker must drop: {output}"
        );
        assert!(
            output.contains("Module 2: node_modules/pkg/b.js\n!function(module,exports,require){Object.defineProperty"),
            "demanded wrapper marker must stay: {output}"
        );
    }

    /// Aliased helper (`module.exports = helper`) escapes lexical
    /// tracing — every marker must survive.
    #[test]
    fn test_aliased_interop_helper_keeps_all_markers() {
        let input = r#"function _interopRequireDefault(e) { return e && e.__esModule ? e : { "default": e }; }
module.exports = _interopRequireDefault;
Object.defineProperty(_m1.exports, "__esModule", { value: true });
"#;
        let output = eliminate_unread_es_module_markers(input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_exported_interop_helper_keeps_only_called_module_markers() {
        let input = r#"// Module 9: node_modules/@babel/runtime/helpers/interopRequireDefault.js
!function(module,exports,require){function _interopRequireDefault(e) {
  return e && e.__esModule ? e : { "default": e };
}
module.exports = _interopRequireDefault, module.exports.__esModule = true;
}(_m9,_m9.exports,_r);
var helper = _r(9);
var wrapped = helper(_r(1));
Object.defineProperty(_m1.exports, "__esModule", { value: true });
Object.defineProperty(_m2.exports, "__esModule", { value: true });
"#;
        let output = eliminate_unread_es_module_markers(input);
        assert!(
            output.contains("_m1.exports, \"__esModule\""),
            "demanded marker must stay: {output}"
        );
        assert!(
            !output.contains("_m2.exports, \"__esModule\""),
            "undemanded marker must drop: {output}"
        );
    }

    #[test]
    fn test_js_parses_without_errors_reports_syntax_errors() {
        assert!(js_parses_without_errors("const value = `${name}`;"));
        assert!(!js_parses_without_errors("const value = ;"));
    }

    #[test]
    fn test_numeric_require_ids_recognizes_dynamic_import_lowering() {
        // GH #1930 — `--splitting` lowers `import(spec)` to
        // `__jet__.dynamicImport(id)` instead of `require(id)`. The entry-
        // reachability rescue pass (apply_tree_shaking) walks this id set to
        // decide which compiled modules survive into the final bundle; if a
        // member-expression call like `__jet__.dynamicImport(2)` isn't
        // recognized here, async-chunk-only modules get pruned as
        // "unreachable" before code splitting ever sees them, producing
        // empty chunk bodies and an empty moduleChunks manifest.
        let source = "__jet__.dynamicImport(2).then(function (mod) { return mod.default(); });";
        let ids = numeric_require_ids(source);
        assert!(
            ids.contains(&2),
            "must recognize __jet__.dynamicImport(id) as a reachability edge, got {ids:?}"
        );
    }

    #[test]
    fn test_numeric_require_ids_still_recognizes_require_and_mangled_alias() {
        let source = "var a = require(1); var b = _r(2);";
        let ids = numeric_require_ids(source);
        assert!(
            ids.contains(&1) && ids.contains(&2),
            "require(id) and _r(id) must both stay recognized, got {ids:?}"
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // #1968 — eliminate_unused_reexport_assignments pruning proof. This
    // function had zero direct test coverage before #1968: the actual bug
    // lived in tree_shake.rs's extract_cjs_require_bindings, which fed this
    // (unmodified) function a `used` set polluted with "*" whenever a CJS
    // shim did `const M = require('barrel'); ... M.someName ...` (property
    // access on a line separate from the require() call) — see
    // tree_shake.rs's
    // `analyze_used_exports_barrel_with_cjs_shim_narrows_without_wildcard_at_scale`
    // for the extractor-side half of this fix. These two tests pin this
    // function's own pre-existing, correct behavior at both ends of that
    // input: it prunes cleanly once `used` has no wildcard, and it still
    // conservatively no-ops when `used` genuinely does.
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_eliminate_unused_reexport_assignments_prunes_barrel_leaves_when_used_has_no_wildcard() {
        let source = concat!(
            "module.exports[\"Icon0\"] = require(2)[\"default\"];\n",
            "module.exports[\"Icon1\"] = require(3)[\"default\"];\n",
            "module.exports[\"Icon2\"] = require(4)[\"default\"];\n",
            "module.exports[\"Icon3\"] = require(5)[\"default\"];\n",
            "module.exports[\"Icon4\"] = require(6)[\"default\"];\n",
        );
        let used: HashSet<String> = ["Icon0", "Icon1", "Icon2"]
            .into_iter()
            .map(String::from)
            .collect();

        let pruned = eliminate_unused_reexport_assignments(source, &used, None);

        assert!(
            pruned.contains("Icon0") && pruned.contains("Icon1") && pruned.contains("Icon2"),
            "used names' re-export assignments must survive: {pruned}"
        );
        assert!(
            !pruned.contains("Icon3") && !pruned.contains("Icon4"),
            "unused barrel leaves must be pruned once the used set has no wildcard, got: {pruned}"
        );
    }

    #[test]
    fn test_eliminate_unused_reexport_assignments_is_noop_when_used_contains_wildcard() {
        // Documents the existing, intentional conservative behavior the
        // #1968 fix works AROUND (by keeping "*" out of the used set for a
        // narrowable CJS shim access) rather than changing: once "*" is
        // genuinely in `used` (a real `import * as ns` or a CJS require
        // this module's own analysis truly could not narrow), this
        // function must still not prune anything, since any export could
        // be read at runtime.
        let source = "module.exports[\"Icon0\"] = require(2)[\"default\"];\n";
        let used: HashSet<String> = ["*"].into_iter().map(String::from).collect();

        let pruned = eliminate_unused_reexport_assignments(source, &used, None);

        assert_eq!(
            pruned, source,
            "a wildcard used set must remain a strict no-op"
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // WI #1947 round 2 — eliminate_require_reexports_to_eliminated_modules
    // no-op/pruning proof cases. bundler::apply_tree_shaking now skips this
    // walk outright for a retained module whenever `numeric_require_ids`
    // (the same primitive used below) proves its outgoing ids are disjoint
    // from `eliminated_module_ids`; these tests establish, shape by shape,
    // that the walk really does nothing when that's true (justifying the
    // skip) and still prunes correctly when it's false (the skip must never
    // fire on real work).
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_eliminate_require_reexports_removes_bare_require_referencing_eliminated_id() {
        let source = "require(5);\nconsole.log('kept');\n";
        let mut eliminated = HashSet::new();
        eliminated.insert(5);
        let out = eliminate_require_reexports_to_eliminated_modules(source, &eliminated);
        assert!(
            !out.contains("require(5)"),
            "a bare require() of an eliminated id must be pruned: {out}"
        );
        assert!(
            out.contains("console.log('kept')"),
            "unrelated statements must survive: {out}"
        );
    }

    #[test]
    fn test_eliminate_require_reexports_removes_module_exports_reassignment_to_eliminated_id() {
        let source = "module.exports = require(7);\nvar kept = 1;\n";
        let mut eliminated = HashSet::new();
        eliminated.insert(7);
        let out = eliminate_require_reexports_to_eliminated_modules(source, &eliminated);
        assert!(
            !out.contains("require(7)"),
            "a module.exports reassignment sourced from an eliminated id must be pruned: {out}"
        );
        assert!(
            out.contains("var kept = 1;"),
            "unrelated statements must survive: {out}"
        );
    }

    #[test]
    fn test_eliminate_require_reexports_removes_re_prefixed_declarator_and_its_object_keys_reexport_loop(
    ) {
        let source = "var __re = require(9);\nObject.keys(__re).forEach(function (k) { module.exports[k] = __re[k]; });\nvar kept = 2;\n";
        let mut eliminated = HashSet::new();
        eliminated.insert(9);
        let out = eliminate_require_reexports_to_eliminated_modules(source, &eliminated);
        assert!(
            !out.contains("require(9)"),
            "the __re declarator sourcing an eliminated id must be pruned: {out}"
        );
        assert!(
            !out.contains("Object.keys(__re)"),
            "its Object.keys reexport loop must be pruned too: {out}"
        );
        assert!(
            out.contains("var kept = 2;"),
            "unrelated statements must survive: {out}"
        );
    }

    #[test]
    fn test_eliminate_require_reexports_is_noop_when_no_referenced_id_is_eliminated() {
        // Mirrors the invariant bundler::apply_tree_shaking's round-2
        // skip-filter relies on: every numeric id this source can reach
        // (bare require, module.exports reassignment, __re-prefixed
        // reexport declarator + its Object.keys reexport loop) is disjoint
        // from eliminated_module_ids, so none of the edit-collecting
        // branches in collect_eliminated_require_reexport_edits /
        // collect_eliminated_reexport_bindings can fire — zero edits, and
        // the source comes back byte-for-byte unchanged.
        let source = "require(1);\nvar __re = require(2);\nObject.keys(__re).forEach(function (k) { module.exports[k] = __re[k]; });\nmodule.exports = require(3);\nvar kept = 3;\n";
        let mut eliminated = HashSet::new();
        eliminated.insert(99);
        let out = eliminate_require_reexports_to_eliminated_modules(source, &eliminated);
        assert_eq!(
            out, source,
            "a source referencing no eliminated id must be byte-identical"
        );
    }

    #[test]
    fn test_numeric_require_ids_disjoint_from_eliminated_set_predicts_noop() {
        // The exact property bundler::apply_tree_shaking's round-2
        // skip-filter depends on: numeric_require_ids(source) disjoint from
        // eliminated_module_ids implies eliminate_require_reexports_to_eliminated_modules
        // is a no-op, across the require/reexport shapes it recognizes.
        let source = "require(1);\nvar __re = require(2);\nObject.keys(__re).forEach(function (k) { module.exports[k] = __re[k]; });\n";
        let ids = numeric_require_ids(source);
        let mut eliminated = HashSet::new();
        eliminated.insert(42);
        assert!(
            ids.is_disjoint(&eliminated),
            "test setup: ids and eliminated must be disjoint, got ids={ids:?}"
        );
        let out = eliminate_require_reexports_to_eliminated_modules(source, &eliminated);
        assert_eq!(
            out, source,
            "a numeric_require_ids-disjoint eliminated set must be a guaranteed no-op"
        );
    }

    // WI #2126 — statement-level DCE for retained modules. The three
    // "topology" fixtures below are minimal synthetic module bodies with
    // the same reference shape as spike #2121's hand-verified real-world
    // examples (function names kept for traceability; bodies are
    // invented/trivial).

    #[test]
    fn test_stmt_dce_generate_utility_class_topology_one_dead_three_live() {
        let source = r#""use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isGlobalState = isGlobalState;
exports.generateUtilityClass = generateUtilityClass;
exports.isSlotComponent = isSlotComponent;
exports.globalStateClassesMapping = void 0;
function isGlobalState(name) {
  return name === "disabled";
}
function generateUtilityClass(componentName, slot) {
  return componentName + "-" + slot;
}
function isSlotComponent(slot) {
  return slot.indexOf("Root") !== -1;
}
const globalStateClassesMapping = {
  active: "Mui-active"
};
exports.globalStateClassesMapping = globalStateClassesMapping;
"#;
        let mut used = HashSet::new();
        used.insert("generateUtilityClass".to_string());
        used.insert("isSlotComponent".to_string());
        used.insert("globalStateClassesMapping".to_string());
        let outcome = eliminate_dead_top_level_declarations(source, &used);
        assert!(!outcome.skipped_vendor);
        assert_eq!(
            outcome.pruned_decls, 1,
            "only isGlobalState should be pruned: {}",
            outcome.code
        );
        assert!(
            !outcome.code.contains("function isGlobalState"),
            "isGlobalState must be pruned, got: {}",
            outcome.code
        );
        assert!(outcome.code.contains("function generateUtilityClass"));
        assert!(outcome.code.contains("function isSlotComponent"));
        assert!(outcome.code.contains("globalStateClassesMapping"));
        assert!(js_parses_without_errors(&outcome.code));
    }

    #[test]
    fn test_stmt_dce_css_utils_topology_five_dead_two_live() {
        let source = r#""use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getUnit = getUnit;
exports.toUnitless = toUnitless;
exports.getSpacingVar = getSpacingVar;
exports.getStyleValue = getStyleValue;
exports.padding = padding;
exports.margin = margin;
exports.convertLength = convertLength;
function getUnit(input) {
  return String(input).replace(/[\d.\-+]/g, "");
}
function toUnitless(length) {
  return parseFloat(length);
}
function getSpacingVar(unit) {
  return toUnitless(unit) + "px";
}
function getStyleValue(value) {
  return getUnit(value);
}
function padding(value) {
  return getStyleValue(value) + " " + getUnit(value);
}
function margin(value) {
  return padding(value);
}
function convertLength(base) {
  return function (length) {
    return toUnitless(length) / toUnitless(base) * 100 + "%";
  };
}
"#;
        let mut used = HashSet::new();
        used.insert("getUnit".to_string());
        used.insert("toUnitless".to_string());
        let outcome = eliminate_dead_top_level_declarations(source, &used);
        assert!(!outcome.skipped_vendor);
        assert_eq!(
            outcome.pruned_decls, 5,
            "getSpacingVar/getStyleValue/padding/margin/convertLength should be pruned (red-herring internal calls to live functions do not save them): {}",
            outcome.code
        );
        assert!(outcome.code.contains("function getUnit"));
        assert!(outcome.code.contains("function toUnitless"));
        for dead in [
            "getSpacingVar",
            "getStyleValue",
            "padding",
            "margin",
            "convertLength",
        ] {
            assert!(
                !outcome.code.contains(&format!("function {dead}")),
                "{dead} must be pruned, got: {}",
                outcome.code
            );
        }
        assert!(js_parses_without_errors(&outcome.code));
    }

    #[test]
    fn test_stmt_dce_color_manipulator_topology_nine_dead_alpha_chain_live() {
        let source = r##""use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.alpha = alpha;
exports.darken = darken;
exports.lighten = lighten;
exports.blend = blend;
exports.emphasize = emphasize;
exports.getContrastRatio = getContrastRatio;
exports.hslToRgb = hslToRgb;
exports.rgbToHex = rgbToHex;
exports.intToHex = intToHex;
exports.colorChannel = void 0;
exports.decomposeColor = decomposeColor;
exports.hexToRgb = hexToRgb;
exports.recomposeColor = recomposeColor;
function clampWrapper(value, min = 0, max = 1) {
  return Math.min(Math.max(min, value), max);
}
function hexToRgb(color) {
  return color;
}
function decomposeColor(color) {
  if (color.charAt(0) === "#") {
    return decomposeColor(hexToRgb(color));
  }
  return { type: "rgb", values: [0, 0, 0] };
}
function recomposeColor(color) {
  return "rgb(" + color.values.join(",") + ")";
}
function alpha(color, value) {
  const parsed = decomposeColor(color);
  parsed.values[3] = clampWrapper(value);
  return recomposeColor(parsed);
}
function darken(color, coefficient) {
  return recomposeColor(decomposeColor(color));
}
function lighten(color, coefficient) {
  return recomposeColor(decomposeColor(color));
}
function blend(background, overlay, opacity) {
  return recomposeColor(decomposeColor(background));
}
function emphasize(color, coefficient) {
  return darken(color, coefficient);
}
function getContrastRatio(foreground, background) {
  return hslToRgb(foreground);
}
function hslToRgb(color) {
  return recomposeColor(decomposeColor(color));
}
function rgbToHex(color) {
  return intToHex(0);
}
function intToHex(int) {
  return int.toString(16);
}
const colorChannel = color => {
  return decomposeColor(color).values.join(" ");
};
exports.colorChannel = colorChannel;
"##;
        let mut used = HashSet::new();
        used.insert("alpha".to_string());
        let outcome = eliminate_dead_top_level_declarations(source, &used);
        assert!(!outcome.skipped_vendor);
        assert_eq!(
            outcome.pruned_decls, 9,
            "darken/lighten/blend/emphasize/getContrastRatio/hslToRgb/rgbToHex/intToHex/colorChannel should be pruned: {}",
            outcome.code
        );
        for live in [
            "alpha",
            "decomposeColor",
            "hexToRgb",
            "recomposeColor",
            "clampWrapper",
        ] {
            assert!(
                outcome.code.contains(&format!("function {live}")),
                "{live} must survive via the alpha->decomposeColor->{{hexToRgb,recomposeColor,clampWrapper}} chain, got: {}",
                outcome.code
            );
        }
        for dead in [
            "darken",
            "lighten",
            "blend",
            "emphasize",
            "getContrastRatio",
            "hslToRgb",
            "rgbToHex",
            "intToHex",
            "colorChannel",
        ] {
            assert!(
                !outcome.code.contains(&format!("function {dead}"))
                    && !outcome.code.contains(&format!("const {dead}")),
                "{dead} must be pruned, got: {}",
                outcome.code
            );
        }
        assert!(js_parses_without_errors(&outcome.code));
    }

    #[test]
    fn test_stmt_dce_multi_declarator_statement_is_atomic() {
        let source = r#"function useIt() {
  return b;
}
exports.useIt = useIt;
var a = 1, b = 2;
var c = 3, d = 4;
"#;
        let mut used = HashSet::new();
        used.insert("useIt".to_string());
        let outcome = eliminate_dead_top_level_declarations(source, &used);
        assert!(
            outcome.code.contains("var a = 1, b = 2;"),
            "atomic multi-declarator: b is referenced so a must survive alongside it, got: {}",
            outcome.code
        );
        assert!(
            !outcome.code.contains("var c = 3, d = 4;"),
            "wholly-unreferenced multi-declarator statement must be pruned atomically, got: {}",
            outcome.code
        );
        assert!(js_parses_without_errors(&outcome.code));
    }

    #[test]
    fn test_stmt_dce_shadowed_generic_name_conservative_keep() {
        // Scope-blind word-scan: the nested `helper` inside `useIt` is a
        // DIFFERENT binding from the top-level `helper`, but this pass
        // cannot tell the two apart — it must conservatively keep the
        // top-level one rather than risk deleting a live declaration.
        let source = r#"function helper() {
  return 1;
}
function useIt() {
  function helper() {
    return 2;
  }
  return helper();
}
exports.useIt = useIt;
"#;
        let mut used = HashSet::new();
        used.insert("useIt".to_string());
        let outcome = eliminate_dead_top_level_declarations(source, &used);
        assert!(
            outcome.code.contains("function helper() {\n  return 1;\n}"),
            "shadowed top-level helper must be conservatively kept, got: {}",
            outcome.code
        );
        assert_eq!(outcome.pruned_decls, 0);
        assert!(js_parses_without_errors(&outcome.code));
    }

    #[test]
    fn test_stmt_dce_string_literal_access_keep() {
        let source = r#"function riskyImpl() {
  return 42;
}
function lookup() {
  return window["riskyImpl"];
}
exports.lookup = lookup;
"#;
        let mut used = HashSet::new();
        used.insert("lookup".to_string());
        let outcome = eliminate_dead_top_level_declarations(source, &used);
        assert!(
            outcome.code.contains("function riskyImpl"),
            "a name mentioned only inside a string literal must conservatively keep the declaration, got: {}",
            outcome.code
        );
        assert_eq!(outcome.pruned_decls, 0);
        assert!(js_parses_without_errors(&outcome.code));
    }

    #[test]
    fn test_stmt_dce_side_effect_iife_kept_and_does_not_block_pruning() {
        let source = "(function () {\n  console.log(\"boot\");\n})();\nfunction unused() {\n  return 1;\n}\n";
        let outcome = eliminate_dead_top_level_declarations(source, &HashSet::new());
        assert!(
            outcome.code.contains("console.log(\"boot\")"),
            "the side-effecting IIFE must never be touched, got: {}",
            outcome.code
        );
        assert!(
            !outcome.code.contains("function unused"),
            "an actually-unreferenced declaration elsewhere must still be pruned, got: {}",
            outcome.code
        );
        assert_eq!(outcome.pruned_decls, 1);
        assert!(js_parses_without_errors(&outcome.code));
    }

    #[test]
    fn test_stmt_dce_eval_module_skips_whole_module_unchanged() {
        let source = "function unused() {\n  return 1;\n}\neval(\"var x = 1;\");\n";
        let outcome = eliminate_dead_top_level_declarations(source, &HashSet::new());
        assert_eq!(
            outcome.code, source,
            "eval(..) anywhere must bail out untouched"
        );
        assert_eq!(outcome.pruned_decls, 0);
        assert!(!outcome.skipped_vendor);
    }

    #[test]
    fn test_stmt_dce_with_statement_skips_whole_module_unchanged() {
        // `is_module_flatten_safe` (reused from scope_hoist.rs) matches the
        // exact substring `"with("` with no space — this fixture matches
        // that shape deliberately (as transformed/minified code typically
        // does) rather than exercising a `with (obj)` spacing variant that
        // probe does not catch; see this test module's other coverage for
        // the substring-matching characteristic this implies.
        let source = "function unused() {\n  return 1;\n}\nwith(obj) {\n  x = 1;\n}\n";
        let outcome = eliminate_dead_top_level_declarations(source, &HashSet::new());
        assert_eq!(
            outcome.code, source,
            "with(..) anywhere must bail out untouched"
        );
    }

    #[test]
    fn test_stmt_dce_dynamic_arguments_index_skips_whole_module_unchanged() {
        let source =
            "function unused() {\n  return 1;\n}\nfunction reader() {\n  return arguments[0];\n}\nexports.reader = reader;\n";
        let mut used = HashSet::new();
        used.insert("reader".to_string());
        let outcome = eliminate_dead_top_level_declarations(source, &used);
        assert_eq!(
            outcome.code, source,
            "dynamic arguments[..] indexing anywhere must bail out untouched"
        );
    }

    #[test]
    fn test_stmt_dce_star_used_export_skips_whole_module_unchanged() {
        let source = "function unused() {\n  return 1;\n}\n";
        let mut used = HashSet::new();
        used.insert("*".to_string());
        let outcome = eliminate_dead_top_level_declarations(source, &used);
        assert_eq!(
            outcome.code, source,
            "used_exports containing \"*\" means whole-namespace consumption -- nothing is safe to prune"
        );
        assert_eq!(outcome.pruned_decls, 0);
    }

    #[test]
    fn test_stmt_dce_require_bound_binding_never_pruned() {
        let source =
            "var sideEffecting = require(\"./polyfill\");\nfunction unused() {\n  return 1;\n}\n";
        let outcome = eliminate_dead_top_level_declarations(source, &HashSet::new());
        assert!(
            outcome.code.contains("require(\"./polyfill\")"),
            "a require()-initialized binding must never be a pruning candidate, got: {}",
            outcome.code
        );
        assert!(!outcome.code.contains("function unused"));
        assert_eq!(outcome.pruned_decls, 1);
    }

    #[test]
    fn test_stmt_dce_destructuring_declaration_conservative_keep() {
        let source =
            "const { a, b } = require(\"./thing\");\nfunction unused() {\n  return 1;\n}\n";
        let outcome = eliminate_dead_top_level_declarations(source, &HashSet::new());
        assert!(
            outcome
                .code
                .contains("const { a, b } = require(\"./thing\");"),
            "a destructuring declaration must never be a pruning candidate, got: {}",
            outcome.code
        );
        assert!(!outcome.code.contains("function unused"));
    }

    #[test]
    fn test_stmt_dce_minified_vendor_bundle_is_skipped() {
        let padding = "x".repeat(150_000);
        let source = format!("var pad = \"{padding}\";function unused(){{return 1;}}");
        let outcome = eliminate_dead_top_level_declarations(&source, &HashSet::new());
        assert!(
            outcome.skipped_vendor,
            "a large, dense single-line module must be skipped as vendor"
        );
        assert_eq!(outcome.code, source);
        assert_eq!(outcome.pruned_decls, 0);
    }

    #[test]
    fn test_stmt_dce_large_but_normally_formatted_module_is_not_skipped() {
        // Same order of magnitude in size as the minified-skip test above,
        // but formatted with real newlines (low average line length) —
        // must NOT trip the vendor-density heuristic, and must still
        // prune the ~999 genuinely-unreferenced helpers.
        let mut source = String::new();
        for i in 0..1000 {
            source.push_str(&format!(
                "function helper{i}() {{\n  // this comment pads the body length for the size threshold test\n  return {i};\n}}\n"
            ));
        }
        source.push_str("function used() {\n  return helper0();\n}\nexports.used = used;\n");
        assert!(
            source.len() >= STMT_DCE_VENDOR_SIZE_THRESHOLD,
            "fixture must clear the size threshold: {}",
            source.len()
        );
        let mut used = HashSet::new();
        used.insert("used".to_string());
        let outcome = eliminate_dead_top_level_declarations(&source, &used);
        assert!(
            !outcome.skipped_vendor,
            "normally-formatted large source must not be treated as a minified vendor bundle"
        );
        assert!(
            outcome.pruned_decls > 0,
            "helper1..helper999 should be pruned"
        );
        assert!(!outcome.code.contains("function helper999"));
        assert!(outcome.code.contains("function helper0"));
        assert!(js_parses_without_errors(&outcome.code));
    }
}
// CODEGEN-END

// <HANDWRITE gap="standardize:projects-jet-src-bundler-dce-rs-empty-statement-cleanup" tracker="standardize-gap-projects-jet-src-bundler-dce-rs" reason="Existing hand-written empty statement cleanup lives outside generated block; generator primitive does not yet cover tree-sitter empty-statement pruning.">
/// Remove redundant empty statements (`;` directly under a program or
/// statement block — the transform emits `function f(){...};` with a
/// trailing semicolon per declaration, ~1KB of `};` on the react-bench
/// bundle). Empty statements that are a branch body (`if(x);`) are
/// children of the `if_statement`, not block-level siblings, and are
/// never touched.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn remove_redundant_empty_statements(source: &str) -> String {
    let Some(tree) = parse_js(source) else {
        return source.to_string();
    };
    let root = tree.root_node();
    if root.has_error() {
        return source.to_string();
    }
    let mut spans: Vec<(usize, usize)> = Vec::new();
    collect_block_level_empty_statements(root, &mut spans);
    if spans.is_empty() {
        return source.to_string();
    }
    spans.sort_unstable();
    let mut out = String::with_capacity(source.len());
    let mut pos = 0usize;
    for (start, end) in spans {
        if start < pos {
            continue;
        }
        out.push_str(&source[pos..start]);
        pos = end;
    }
    out.push_str(&source[pos..]);
    // Callers run a combined parse guard over the whole polish pipeline.
    out
}

fn collect_block_level_empty_statements(node: Node<'_>, spans: &mut Vec<(usize, usize)>) {
    let container = matches!(node.kind(), "program" | "statement_block");
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if container && child.kind() == "empty_statement" {
            spans.push((child.start_byte(), child.end_byte()));
            continue;
        }
        if matches!(
            child.kind(),
            "string" | "template_string" | "comment" | "regex"
        ) {
            continue;
        }
        collect_block_level_empty_statements(child, spans);
    }
}
// </HANDWRITE>
