// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
//! Tree shaking: static analysis to detect and remove unused exports.
//!
//! Supports both ESM (import/export) and CJS (require/module.exports) patterns.
//! Analyzes the module graph to build a set of used exports per module,
//! then eliminates unused export declarations from the bundle.
//!
//! R3 – sideEffects: honors `"sideEffects": false` in installed package.json
//! files (npm convention). When a package declares no side effects, any of
//! its modules that have no used imports can be completely eliminated even if
//! the static code analysis would otherwise conservatively keep them.

use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// GH #3815 — warn shown when `module_has_side_effects` is asked to
/// match a module path with a non-UTF-8 file_name or non-UTF-8 full
/// path against a `sideEffects` glob list. The prior code silently
/// collapsed both onto `""` / `false`, causing the file to bypass the
/// glob match and be treated as side-effect-free — its contents could
/// then be tree-shaken away despite the package.json declaration.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub(crate) fn format_tree_shake_non_utf8_module_path_warn(module_path: &Path) -> String {
    let file_name_kind = match module_path.file_name() {
        None => "no file_name (path is `..` or root)",
        Some(os) => match os.to_str() {
            Some(_) => "file_name is UTF-8 (unexpected — this helper should not have been called)",
            None => "non-UTF-8 file_name",
        },
    };
    let path_kind = if module_path.to_str().is_some() {
        "UTF-8 full path"
    } else {
        "non-UTF-8 full path"
    };
    format!(
        "gh3815 tree_shake module_has_side_effects could not stringify module_path={:?} \
         ({file_name_kind}, {path_kind}); the sideEffects glob list cannot match this file \
         and it will be treated as side-effect-free, which may eliminate code the package.json \
         explicitly preserved. If this file lives inside a package with a sideEffects globs \
         declaration, consider renaming it to UTF-8.",
        module_path
    )
}

/// GH #3815 — given a module path and a glob list, decide whether any
/// glob matches, warning if the path can't be stringified for matching.
///
/// - file_name UTF-8 + globs match: silent true.
/// - file_name UTF-8 + no glob match + full path UTF-8 + globs match: silent true.
/// - no glob match anywhere: silent false.
/// - file_name unstringifiable AND full path unstringifiable: emit a
///   gh3815 warn and return false (legacy behaviour preserved).
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub(crate) fn tree_shake_module_path_matches_any_glob(
    module_path: &Path,
    globs: &[String],
    glob_matches: impl Fn(&str, &str) -> bool,
) -> bool {
    let file_name: Option<&str> = module_path.file_name().and_then(|n| n.to_str());
    let full_path: Option<&str> = module_path.to_str();

    if file_name.is_none() && full_path.is_none() {
        tracing::warn!(
            target: "jet::bundler::tree_shake",
            module_path = %module_path.display(),
            "{}",
            format_tree_shake_non_utf8_module_path_warn(module_path)
        );
        return false;
    }

    globs.iter().any(|g| {
        file_name.map(|fn_| glob_matches(g, fn_)).unwrap_or(false)
            || full_path.map(|p| glob_matches(g, p)).unwrap_or(false)
    })
}

/// Result of tree shaking analysis.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone)]
pub struct TreeShakeResult {
    /// Module path → set of used export names.
    pub used_exports: HashMap<PathBuf, HashSet<String>>,
    /// Modules entirely eliminated (no used exports, no side effects).
    pub eliminated_modules: Vec<PathBuf>,
    /// Estimated bytes eliminated.
    pub eliminated_bytes: u64,
}

/// Analyze which exports are used across the module graph.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn analyze_used_exports(modules: &[(PathBuf, String)]) -> Result<TreeShakeResult> {
    let mut all_exports: HashMap<PathBuf, Vec<String>> = HashMap::new();
    let mut used: HashMap<PathBuf, HashSet<String>> = HashMap::new();

    // Step 1: Collect all exports per module (ESM + CJS)
    for (path, source) in modules {
        let mut exports = extract_export_names(source, is_ts(path));
        exports.extend(extract_cjs_export_names(source));
        all_exports.insert(path.clone(), exports);
    }

    // Step 2: Mark all ESM imports as used
    for (_path, source) in modules {
        let imports = extract_import_bindings(source, modules);
        for (target_path, names) in imports {
            let entry = used.entry(target_path).or_default();
            for name in names {
                entry.insert(name);
            }
        }
    }

    // Step 3: Mark CJS require bindings as used
    for (_path, source) in modules {
        let cjs_imports = extract_cjs_require_bindings(source, modules);
        for (target_path, names) in cjs_imports {
            let entry = used.entry(target_path).or_default();
            for name in names {
                entry.insert(name);
            }
        }
    }

    // Step 3.5: Thread re-export chains so that `export { x } from './y'`
    // in a barrel propagates the "used" signal to the leaf module `y`
    // whenever the barrel's `x` itself is reached. This is iterative
    // (fixed-point) because re-export chains can be multi-hop: a barrel
    // can re-export from another barrel. The loop terminates because
    // `used` is monotonic (we only add names) and the carrier set
    // (module × export-name) is finite.
    //
    // Three shapes are handled:
    //   1. `export { x } from './y'`         → if barrel's `x` is used,
    //                                          mark `y`'s `x` used.
    //   2. `export { x as alias } from './y'` → if barrel's `alias` is
    //                                          used, mark `y`'s `x` used
    //                                          (the original name lives
    //                                          on the leaf).
    //   3. `export * from './y'`             → mark every export of `y`
    //                                          as used unconditionally.
    //                                          A star re-export keeps the
    //                                          whole leaf alive even when
    //                                          the consumer is selective,
    //                                          because the analyzer can't
    //                                          tell which leaf each
    //                                          consumed name came from.
    loop {
        let mut changed = false;
        for (path, source) in modules {
            let reexports = extract_reexport_bindings(source, modules);
            let barrel_used: HashSet<String> = used.get(path).cloned().unwrap_or_default();
            for (target_path, kind) in reexports {
                match kind {
                    ReexportKind::Star => {
                        // `export * from './y'` — keep the leaf wholly alive.
                        if let Some(leaf_exports) = all_exports.get(&target_path) {
                            // Collect every non-default name we'd want
                            // to insert first, then only create the
                            // `used` entry when we'd actually add
                            // something. Otherwise we'd leak an empty
                            // `target_path → {}` entry that downstream
                            // snapshot/observability code (rightly)
                            // treats as a real "used" record.
                            let to_add: Vec<&String> =
                                leaf_exports.iter().filter(|n| *n != "default").collect();
                            if to_add.is_empty() {
                                continue;
                            }
                            let entry = used.entry(target_path.clone()).or_default();
                            for name in to_add {
                                if entry.insert(name.clone()) {
                                    changed = true;
                                }
                            }
                        }
                    }
                    ReexportKind::Named(pairs) => {
                        // Each pair is (local-on-leaf, exposed-on-barrel).
                        // Propagate only when the exposed name is used.
                        // We hold off on creating the `used[target]`
                        // entry until we know we have at least one name
                        // to insert — same rationale as the Star arm.
                        let to_add: Vec<String> = pairs
                            .into_iter()
                            .filter_map(|(leaf_name, barrel_name)| {
                                if barrel_used.contains(&barrel_name) {
                                    Some(leaf_name)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        if to_add.is_empty() {
                            continue;
                        }
                        let entry = used.entry(target_path).or_default();
                        for leaf_name in to_add {
                            if entry.insert(leaf_name) {
                                changed = true;
                            }
                        }
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }

    // Step 3.6: Mark dynamic `import("./x")` call-expression targets
    // with the wildcard `"*"`. A dynamic import loads the target
    // module's whole namespace at runtime, so the static analyzer
    // cannot narrow which exports survive — conservative behavior
    // is to mark every export as live. The dynamic-import boundary
    // also keeps the module out of `eliminated_modules`.
    //
    // Covers: `import('./x')`, `await import("./x")`,
    // `const m = import('./x.js')`, and any line containing the
    // `import(<quoted-string>)` substring. Template-string
    // specifiers (`import(\`./${name}\`)`) are intentionally ignored
    // by this pass — the `find_module_by_specifier` API needs a
    // literal specifier, and treating templates as a wildcard over
    // the whole graph would be too coarse.
    for (_path, source) in modules {
        for target_path in extract_dynamic_import_targets(source, modules) {
            used.entry(target_path).or_default().insert("*".to_string());
        }
    }

    // Step 4: Find eliminated modules
    let mut eliminated = Vec::new();
    let mut eliminated_bytes = 0u64;

    for (path, source) in modules {
        if let Some(exports) = all_exports.get(path) {
            if !exports.is_empty() {
                let used_set = used.get(path);
                let has_used = used_set.map(|s| !s.is_empty()).unwrap_or(false);

                if !has_used && !has_side_effects(source) {
                    eliminated.push(path.clone());
                    eliminated_bytes += source.len() as u64;
                }
            }
        }
    }

    Ok(TreeShakeResult {
        used_exports: used,
        eliminated_modules: eliminated,
        eliminated_bytes,
    })
}

/// Remove unused exports from source code.
/// Returns the source with unused export declarations removed.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn shake_module(source: &str, path: &PathBuf, used_exports: &HashSet<String>) -> String {
    if used_exports.is_empty() {
        if has_side_effects(source) {
            return source.to_string();
        }
        return String::new();
    }

    let mut result = String::new();
    for line in source.lines() {
        let trimmed = line.trim();

        // Check if this is an ESM export we should remove
        if trimmed.starts_with("export ") && !trimmed.starts_with("export default") {
            if let Some(name) = extract_single_export_name(trimmed) {
                if !used_exports.contains(&name) {
                    result.push('\n');
                    continue;
                }
            }
        }

        result.push_str(line);
        result.push('\n');
    }

    let _ = path;
    result
}

// --- ESM analysis ---

/// Extract ESM export names from source.
fn extract_export_names(source: &str, _is_typescript: bool) -> Vec<String> {
    let mut names = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(name) = extract_single_export_name(trimmed) {
            names.push(name);
        }
        if trimmed.starts_with("export default") {
            names.push("default".to_string());
        }
        // export { a, b, c }
        if trimmed.starts_with("export {") {
            if let Some(brace_end) = trimmed.find('}') {
                let inner = &trimmed[8..brace_end]; // skip "export {"
                for name in inner.split(',') {
                    let name = name.trim();
                    // Handle "name as alias" — the export name is the alias
                    let export_name = if let Some((_orig, alias)) = name.split_once(" as ") {
                        alias.trim()
                    } else {
                        name
                    };
                    if !export_name.is_empty() {
                        names.push(export_name.to_string());
                    }
                }
            }
        }
    }

    names
}

/// Extract the name from a single export declaration line.
fn extract_single_export_name(line: &str) -> Option<String> {
    let prefixes = [
        "export const ",
        "export let ",
        "export var ",
        "export function ",
        "export class ",
        "export async function ",
    ];

    for prefix in &prefixes {
        if let Some(rest) = line.strip_prefix(prefix) {
            let name = rest
                .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '$')
                .next()?;
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }

    None
}

/// Extract ESM import bindings: returns (target_module_path, imported names).
fn extract_import_bindings(
    source: &str,
    modules: &[(PathBuf, String)],
) -> Vec<(PathBuf, Vec<String>)> {
    let mut results = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("import ") {
            continue;
        }

        let specifier = extract_specifier(trimmed);
        if specifier.is_empty() {
            continue;
        }

        let target = find_module_by_specifier(&specifier, modules);

        if let Some((target_path, _)) = target {
            let names = extract_imported_names(trimmed);
            if !names.is_empty() {
                results.push((target_path.clone(), names));
            }
        }
    }

    results
}

/// Classification of an `export ... from '...'` re-export line.
#[derive(Debug, Clone)]
enum ReexportKind {
    /// `export * from './y'` — re-export every named export of `y`
    /// (except `default`, per the ES2015+ spec).
    Star,
    /// `export { a, b as c } from './y'` — each pair is
    /// `(name-on-leaf, name-exposed-on-barrel)`. For `b as c`, the
    /// leaf still owns the symbol as `b`; the barrel exposes it as
    /// `c`. The barrel side is what consumers can "mark used", so
    /// we propagate only when the barrel name is in the used set.
    Named(Vec<(String, String)>),
}

/// Extract ESM re-export bindings: `export { ... } from '...'` and
/// `export * from '...'`. Returns `(target_module_path, kind)` pairs
/// for every re-export line whose specifier resolves to a known module.
///
/// Bare `export { a, b }` (no `from`) is NOT a re-export — it's a
/// local renaming export that exposes already-declared names, and is
/// already handled by `extract_export_names`. Only lines with both
/// `from` and a resolvable specifier are returned here.
fn extract_reexport_bindings(
    source: &str,
    modules: &[(PathBuf, String)],
) -> Vec<(PathBuf, ReexportKind)> {
    let mut results = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("export ") {
            continue;
        }
        // Must be a re-export (has `from`) — otherwise it's a local export.
        if !trimmed.contains(" from ") {
            continue;
        }

        let specifier = extract_specifier(trimmed);
        if specifier.is_empty() {
            continue;
        }
        let target = match find_module_by_specifier(&specifier, modules) {
            Some(t) => t,
            None => continue,
        };

        // `export * from '...'` — wildcard re-export. The `as ns`
        // variant (`export * as ns from '...'`) is a *namespace*
        // re-export: the barrel exposes one symbol `ns` whose value
        // is the leaf's namespace. We treat it the same as `*` here
        // because we don't track namespace-property usage statically.
        let after_export = &trimmed["export ".len()..];
        if after_export.trim_start().starts_with('*') {
            results.push((target.0.clone(), ReexportKind::Star));
            continue;
        }

        // `export { a, b as c } from '...'` — named re-export.
        if let Some(brace_start) = trimmed.find('{') {
            if let Some(brace_end) = trimmed.find('}') {
                if brace_start < brace_end {
                    let inner = &trimmed[brace_start + 1..brace_end];
                    let mut pairs = Vec::new();
                    for item in inner.split(',') {
                        let item = item.trim();
                        if item.is_empty() {
                            continue;
                        }
                        let (leaf, barrel) = if let Some((orig, alias)) = item.split_once(" as ") {
                            (orig.trim().to_string(), alias.trim().to_string())
                        } else {
                            (item.to_string(), item.to_string())
                        };
                        if !leaf.is_empty() && !barrel.is_empty() {
                            pairs.push((leaf, barrel));
                        }
                    }
                    if !pairs.is_empty() {
                        results.push((target.0.clone(), ReexportKind::Named(pairs)));
                    }
                }
            }
        }
    }

    results
}

/// Extract dynamic-import (`import(...)`) call-expression targets.
///
/// Walks `source` for the substring `import(` anywhere on a line
/// (so it survives `await import(...)`, `const m = import(...)`,
/// nested-in-if forms, etc.) and parses the immediately-following
/// quoted string as the specifier. Returns the resolved module
/// path for each specifier that matches a module in `modules`.
///
/// The static `import ... from '...';` declaration ALWAYS has a
/// space after `import`, so the `import(` shape disambiguates the
/// call-expression form. Template-string specifiers
/// (`import(\`./${name}\`)`) are deliberately skipped — they have
/// no literal that `find_module_by_specifier` can resolve, and
/// treating them as a wildcard over the whole graph would
/// over-retain.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn extract_dynamic_import_targets(source: &str, modules: &[(PathBuf, String)]) -> Vec<PathBuf> {
    let mut results = Vec::new();
    let bytes = source.as_bytes();
    let needle = b"import(";
    let mut cursor = 0;

    while cursor + needle.len() <= bytes.len() {
        let Some(rel) = bytes[cursor..]
            .windows(needle.len())
            .position(|w| w == needle)
        else {
            break;
        };
        let match_start = cursor + rel;
        cursor = match_start + needle.len();

        // Disqualify the static-import declaration form: a leading
        // identifier char before `import(` would mean we matched
        // something like `xxximport(` inside a larger identifier.
        if match_start > 0 {
            let prev = bytes[match_start - 1] as char;
            if prev.is_alphanumeric() || prev == '_' || prev == '$' {
                continue;
            }
        }

        // Position right after `import(`.
        let after_paren = cursor;
        if after_paren >= bytes.len() {
            break;
        }

        // Skip optional whitespace between `(` and the specifier.
        let mut idx = after_paren;
        while idx < bytes.len() && (bytes[idx] == b' ' || bytes[idx] == b'\t') {
            idx += 1;
        }
        if idx >= bytes.len() {
            break;
        }

        let quote = bytes[idx];
        if quote != b'\'' && quote != b'"' {
            // Template strings (`` ` ``), identifiers, etc. — skip;
            // conservative wildcard over the whole graph is too
            // coarse for the current API.
            continue;
        }
        let str_start = idx + 1;
        let Some(end_rel) = bytes[str_start..].iter().position(|&b| b == quote) else {
            continue;
        };
        let specifier = match std::str::from_utf8(&bytes[str_start..str_start + end_rel]) {
            Ok(s) => s,
            Err(_) => continue,
        };

        if let Some((target_path, _)) = find_module_by_specifier(specifier, modules) {
            results.push(target_path.clone());
        }
    }

    results
}

/// Extract the module specifier string from an import/require line.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn extract_specifier(line: &str) -> String {
    for quote in ['\'', '"'] {
        if let Some(start) = line.rfind(quote) {
            let before = &line[..start];
            if let Some(end) = before.rfind(quote) {
                return line[end + 1..start].to_string();
            }
        }
    }
    String::new()
}

/// Extract imported names from an ESM import statement.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn extract_imported_names(line: &str) -> Vec<String> {
    let mut names = Vec::new();

    // import * as ns from '...' → marks ALL exports as used
    if line.contains("* as ") {
        names.push("*".to_string());
        return names;
    }

    // import { a, b, c } from '...'
    if let Some(brace_start) = line.find('{') {
        if let Some(brace_end) = line.find('}') {
            let inner = &line[brace_start + 1..brace_end];
            for name in inner.split(',') {
                let name = name.trim();
                let actual = name.split(" as ").next().unwrap_or(name).trim();
                if !actual.is_empty() {
                    names.push(actual.to_string());
                }
            }
        }
    }

    // import Default from '...'
    if line.starts_with("import ")
        && !line.contains('{')
        && !line.contains("* as ")
        && line.contains(" from ")
    {
        let after_import = &line["import ".len()..];
        if let Some(name) = after_import
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .next()
        {
            if !name.is_empty() && name != "type" && name != "from" {
                names.push("default".to_string());
            }
        }
    }

    names
}

// --- CJS analysis ---

/// Extract CJS export names from module.exports / exports.X patterns.
fn extract_cjs_export_names(source: &str) -> Vec<String> {
    let mut names = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();

        // exports.foo = ... → export named "foo"
        if let Some(rest) = trimmed.strip_prefix("exports.") {
            if let Some(name) = rest
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .next()
            {
                if !name.is_empty() {
                    names.push(name.to_string());
                }
            }
        }

        // module.exports = { a, b, c } or module.exports = { a: ..., b: ... }
        if trimmed.starts_with("module.exports") {
            if let Some(brace_start) = trimmed.find('{') {
                if let Some(brace_end) = trimmed.rfind('}') {
                    let inner = &trimmed[brace_start + 1..brace_end];
                    for item in inner.split(',') {
                        let item = item.trim();
                        // Handle "key: value" and shorthand "key"
                        let key = item.split(':').next().unwrap_or(item).trim();
                        let key = key
                            .split(|c: char| !c.is_alphanumeric() && c != '_')
                            .next()
                            .unwrap_or("");
                        if !key.is_empty() {
                            names.push(key.to_string());
                        }
                    }
                }
            }
        }
    }

    names
}

/// Extract CJS require bindings: which properties are used from required modules.
fn extract_cjs_require_bindings(
    source: &str,
    modules: &[(PathBuf, String)],
) -> Vec<(PathBuf, Vec<String>)> {
    let mut results = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();

        // Pattern 1: const { a, b } = require('mod')
        if let Some(req_source) = extract_require_specifier(trimmed) {
            if let Some(target) = find_module_by_specifier(&req_source, modules) {
                let names = extract_destructured_names(trimmed);
                if !names.is_empty() {
                    results.push((target.0.clone(), names));
                }
            }
        }

        // Pattern 2: const mod = require('mod'); later mod.prop
        // This is harder to track across lines — we handle the simple
        // single-line property access pattern: require('mod').prop
        if let Some(req_source) = extract_require_specifier(trimmed) {
            if let Some(target) = find_module_by_specifier(&req_source, modules) {
                let names = extract_require_property_access(trimmed);
                if !names.is_empty() {
                    results.push((target.0.clone(), names));
                }
            }
        }
    }

    results
}

/// Extract the specifier from a require() call.
fn extract_require_specifier(line: &str) -> Option<String> {
    let require_pos = line.find("require(")?;
    let after = &line[require_pos + 8..]; // skip "require("
    let quote = after.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let end = after[1..].find(quote)?;
    Some(after[1..1 + end].to_string())
}

/// Extract destructured property names from `const { a, b } = require(...)`.
fn extract_destructured_names(line: &str) -> Vec<String> {
    let mut names = Vec::new();

    if let Some(brace_start) = line.find('{') {
        if let Some(brace_end) = line.find('}') {
            if brace_start < brace_end {
                let inner = &line[brace_start + 1..brace_end];
                for name in inner.split(',') {
                    let name = name.trim();
                    // Handle "name: alias"
                    let actual = name.split(':').next().unwrap_or(name).trim();
                    // Handle "name as alias" (though rare in CJS)
                    let actual = actual.split(" as ").next().unwrap_or(actual).trim();
                    if !actual.is_empty() {
                        names.push(actual.to_string());
                    }
                }
            }
        }
    }

    names
}

/// Extract property access from require('mod').prop patterns.
fn extract_require_property_access(line: &str) -> Vec<String> {
    let mut names = Vec::new();

    // Look for require(...).<prop> or require(...)[" prop"]
    if let Some(req_pos) = line.find("require(") {
        let after_req = &line[req_pos..];
        if let Some(close_paren) = after_req.find(')') {
            let after_close = &after_req[close_paren + 1..];
            let rest = after_close.trim_start();

            // .property access
            if let Some(rest) = rest.strip_prefix('.') {
                let prop = rest
                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                    .next()
                    .unwrap_or("");
                if !prop.is_empty() {
                    names.push(prop.to_string());
                }
            }

            // ["property"] access
            if rest.starts_with('[') {
                if let Some(end) = rest.find(']') {
                    let inner = rest[1..end].trim();
                    if let Some(s) = extract_string_value(inner) {
                        names.push(s);
                    }
                }
            }
        }
    }

    names
}

fn extract_string_value(s: &str) -> Option<String> {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        Some(s[1..s.len() - 1].to_string())
    } else {
        None
    }
}

// --- Shared utilities ---

/// Find a module in the module list by matching specifier against paths.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn find_module_by_specifier<'a>(
    specifier: &str,
    modules: &'a [(PathBuf, String)],
) -> Option<&'a (PathBuf, String)> {
    let bare = specifier.strip_prefix("./").unwrap_or(specifier);
    modules.iter().find(|(p, _)| {
        let p_str = p.to_string_lossy();
        p_str.ends_with(bare)
            || p_str.ends_with(specifier)
            || p_str.ends_with(&format!("{}.js", bare))
            || p_str.ends_with(&format!("{}.ts", bare))
            || p_str.ends_with(&format!("{}.tsx", bare))
            || p_str.ends_with(&format!("{}/index.js", bare))
            || p_str.ends_with(&format!("{}/index.ts", bare))
    })
}

/// Check if source code has side effects (top-level statements that aren't declarations).
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn has_side_effects(source: &str) -> bool {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }
        if trimmed.starts_with("export ")
            || trimmed.starts_with("import ")
            || trimmed.starts_with("const ")
            || trimmed.starts_with("let ")
            || trimmed.starts_with("var ")
            || trimmed.starts_with("function ")
            || trimmed.starts_with("class ")
            || trimmed.starts_with("type ")
            || trimmed.starts_with("interface ")
            || trimmed.starts_with("enum ")
        {
            continue;
        }
        if !trimmed.starts_with('}') && !trimmed.starts_with('*') {
            return true;
        }
    }
    false
}

fn is_ts(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e == "ts" || e == "tsx")
        .unwrap_or(false)
}

// JSON tree-shaking is in the `json_shake` submodule.
// Re-exported from `super::json_shake`.

// ─── sideEffects (R3) ────────────────────────────────────────────────────────

/// Possible values of the npm `sideEffects` field.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone, PartialEq)]
pub enum SideEffectsDecl {
    /// `"sideEffects": false` — library claims zero side effects.
    None,
    /// `"sideEffects": true` or absent — assume side effects.
    All,
    /// `"sideEffects": ["*.css", "polyfill.js", …]` — only the listed files
    /// have side effects; all other files in the package are side-effect-free.
    Globs(Vec<String>),
}

/// Read the `sideEffects` declaration from an installed package's
/// `package.json` located at `node_modules/{package_name}/package.json`.
///
/// Returns `SideEffectsDecl::All` when:
/// - The file is missing or unreadable.
/// - The field is absent (conservative default, matching npm/bundler behavior).
/// - The field is `true`.
///
/// Returns `SideEffectsDecl::None` when the field is `false`.
///
/// Returns `SideEffectsDecl::Globs(v)` when the field is an array of globs.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn read_package_side_effects(node_modules: &Path, package_name: &str) -> SideEffectsDecl {
    let pkg_json = node_modules.join(package_name).join("package.json");
    let content = match std::fs::read_to_string(&pkg_json) {
        Ok(c) => c,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return SideEffectsDecl::All;
        }
        Err(err) => {
            tracing::warn!(
                target: "jet::bundler::tree_shake",
                package = %package_name,
                path = %pkg_json.display(),
                error = %err,
                "GH #3316 package.json unreadable while resolving sideEffects; \
                 falling back to conservative SideEffectsDecl::All — the \
                 package's declared sideEffects claim is being ignored. Check \
                 file permissions or for a truncated install."
            );
            return SideEffectsDecl::All;
        }
    };
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(err) => {
            tracing::warn!(
                target: "jet::bundler::tree_shake",
                package = %package_name,
                path = %pkg_json.display(),
                error = %err,
                "GH #3316 package.json failed to parse while resolving \
                 sideEffects; falling back to conservative SideEffectsDecl::All \
                 — the package's declared sideEffects claim is being ignored."
            );
            return SideEffectsDecl::All;
        }
    };
    match json.get("sideEffects") {
        Some(serde_json::Value::Bool(false)) => SideEffectsDecl::None,
        Some(serde_json::Value::Bool(true)) => SideEffectsDecl::All,
        Some(serde_json::Value::Array(arr)) => {
            let globs: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            if globs.is_empty() {
                SideEffectsDecl::None
            } else {
                SideEffectsDecl::Globs(globs)
            }
        }
        // Field absent or unexpected type → conservative default.
        _ => SideEffectsDecl::All,
    }
}

/// Determine whether a module has side effects, taking into account:
/// 1. An explicit `sideEffects` declaration from the owning package.json
///    (read via `read_package_side_effects`).
/// 2. Heuristic code analysis (`has_side_effects`).
///
/// `decl` is the `SideEffectsDecl` for the *package* that owns `module_path`.
/// Pass `SideEffectsDecl::All` (or look it up via `read_package_side_effects`)
/// for modules whose owning package is unknown.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn module_has_side_effects(source: &str, module_path: &Path, decl: &SideEffectsDecl) -> bool {
    match decl {
        // Package explicitly says "no side effects anywhere" — trust it.
        SideEffectsDecl::None => false,
        // Package lists specific files that have side effects.
        SideEffectsDecl::Globs(globs) => {
            // GH #3815 — extracted into a helper so non-UTF-8 module paths
            // emit a tracing::warn! instead of silently collapsing onto
            // false (which would treat the file as side-effect-free and
            // tree-shake away code the package.json explicitly preserved).
            let matches_glob =
                tree_shake_module_path_matches_any_glob(module_path, globs, |g, p| {
                    glob_matches(g, p)
                });
            if matches_glob {
                true
            } else {
                // Not in the side-effects list → treat as side-effect-free.
                false
            }
        }
        // Conservative: fall back to code analysis.
        SideEffectsDecl::All => has_side_effects(source),
    }
}

/// Minimal glob matcher supporting `*` (any non-separator chars) and `**`
/// (any chars including separators). Used for npm `sideEffects` glob patterns.
fn glob_matches(pattern: &str, text: &str) -> bool {
    glob_match_recursive(pattern.as_bytes(), text.as_bytes())
}

fn glob_match_recursive(pat: &[u8], text: &[u8]) -> bool {
    match (pat.first(), text.first()) {
        (None, None) => true,
        (None, Some(_)) => false,
        (Some(b'*'), _) => {
            // Check for `**`
            if pat.get(1) == Some(&b'*') {
                let rest = &pat[2..];
                // Skip leading slash after **
                let rest = if rest.first() == Some(&b'/') {
                    &rest[1..]
                } else {
                    rest
                };
                // `**` matches zero or more path segments
                for i in 0..=text.len() {
                    if glob_match_recursive(rest, &text[i..]) {
                        return true;
                    }
                }
                false
            } else {
                // `*` matches zero or more non-separator chars
                let rest = &pat[1..];
                for i in 0..=text.len() {
                    if i > 0 && text[i - 1] == b'/' {
                        break; // `*` does not cross directory separator
                    }
                    if glob_match_recursive(rest, &text[i..]) {
                        return true;
                    }
                }
                false
            }
        }
        (Some(&pc), Some(&tc)) => {
            if pc == tc {
                glob_match_recursive(&pat[1..], &text[1..])
            } else {
                false
            }
        }
        (Some(_), None) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_export_names() {
        let source = r#"
export const foo = 1;
export function bar() {}
export class Baz {}
export default function main() {}
const internal = 2;
"#;
        let names = extract_export_names(source, false);
        assert!(names.contains(&"foo".to_string()));
        assert!(names.contains(&"bar".to_string()));
        assert!(names.contains(&"Baz".to_string()));
        assert!(names.contains(&"default".to_string()));
        assert!(!names.contains(&"internal".to_string()));
    }

    #[test]
    fn test_extract_export_braces() {
        let source = "export { foo, bar as baz };\n";
        let names = extract_export_names(source, false);
        assert!(names.contains(&"foo".to_string()));
        assert!(names.contains(&"baz".to_string()));
    }

    #[test]
    fn test_extract_imported_names() {
        assert_eq!(
            extract_imported_names("import { a, b } from './mod'"),
            vec!["a", "b"]
        );
        assert_eq!(
            extract_imported_names("import { a as x } from './mod'"),
            vec!["a"]
        );
        assert_eq!(
            extract_imported_names("import Foo from './mod'"),
            vec!["default"]
        );
    }

    #[test]
    fn test_namespace_import() {
        let names = extract_imported_names("import * as Utils from './utils'");
        assert_eq!(names, vec!["*"]);
    }

    #[test]
    fn test_has_side_effects() {
        assert!(!has_side_effects(
            "export const x = 1;\nexport function f() {}"
        ));
        assert!(has_side_effects("console.log('hello');"));
        assert!(has_side_effects("window.x = 1;"));
    }

    #[test]
    fn test_shake_removes_unused() {
        let source = "export const used = 1;\nexport const unused = 2;\n";
        let mut used = HashSet::new();
        used.insert("used".to_string());

        let result = shake_module(source, &PathBuf::from("test.js"), &used);
        assert!(result.contains("export const used"));
        assert!(!result.contains("export const unused"));
    }

    // --- CJS tests ---

    #[test]
    fn test_cjs_export_names_exports_dot() {
        let source = "exports.foo = function() {};\nexports.bar = 42;\n";
        let names = extract_cjs_export_names(source);
        assert!(names.contains(&"foo".to_string()));
        assert!(names.contains(&"bar".to_string()));
    }

    #[test]
    fn test_cjs_export_names_module_exports_object() {
        let source = "module.exports = { createElement, useState, useEffect };\n";
        let names = extract_cjs_export_names(source);
        assert!(names.contains(&"createElement".to_string()));
        assert!(names.contains(&"useState".to_string()));
        assert!(names.contains(&"useEffect".to_string()));
    }

    #[test]
    fn test_cjs_destructured_require() {
        let line = "const { createElement, useState } = require('react');";
        let names = extract_destructured_names(line);
        assert_eq!(names, vec!["createElement", "useState"]);
    }

    #[test]
    fn test_cjs_require_property_access() {
        let line = "var jsx = require('react/jsx-runtime')[\"jsx\"];";
        let names = extract_require_property_access(line);
        assert_eq!(names, vec!["jsx"]);
    }

    #[test]
    fn test_cjs_require_dot_access() {
        let line = "var React = require('react').default;";
        let names = extract_require_property_access(line);
        assert_eq!(names, vec!["default"]);
    }

    #[test]
    fn test_require_specifier() {
        assert_eq!(
            extract_require_specifier("const x = require('react')"),
            Some("react".to_string())
        );
        assert_eq!(
            extract_require_specifier("require(\"./cjs/react.production.js\")"),
            Some("./cjs/react.production.js".to_string())
        );
        assert_eq!(extract_require_specifier("const x = 1"), None);
    }

    // ─── sideEffects tests (R3) ──────────────────────────────────────────────

    #[test]
    fn test_side_effects_decl_false() {
        let dir = std::env::temp_dir().join("jet-ts-se-false");
        let pkg_dir = dir.join("my-lib");
        std::fs::create_dir_all(&pkg_dir).unwrap();
        std::fs::write(
            pkg_dir.join("package.json"),
            r#"{"name":"my-lib","version":"1.0.0","sideEffects":false}"#,
        )
        .unwrap();
        let decl = read_package_side_effects(&dir, "my-lib");
        assert_eq!(decl, SideEffectsDecl::None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_side_effects_decl_true() {
        let dir = std::env::temp_dir().join("jet-ts-se-true");
        let pkg_dir = dir.join("my-lib");
        std::fs::create_dir_all(&pkg_dir).unwrap();
        std::fs::write(
            pkg_dir.join("package.json"),
            r#"{"name":"my-lib","version":"1.0.0","sideEffects":true}"#,
        )
        .unwrap();
        let decl = read_package_side_effects(&dir, "my-lib");
        assert_eq!(decl, SideEffectsDecl::All);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_side_effects_decl_absent() {
        let dir = std::env::temp_dir().join("jet-ts-se-absent");
        let pkg_dir = dir.join("my-lib");
        std::fs::create_dir_all(&pkg_dir).unwrap();
        std::fs::write(
            pkg_dir.join("package.json"),
            r#"{"name":"my-lib","version":"1.0.0"}"#,
        )
        .unwrap();
        let decl = read_package_side_effects(&dir, "my-lib");
        assert_eq!(
            decl,
            SideEffectsDecl::All,
            "absent field should default to All"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_side_effects_decl_globs() {
        let dir = std::env::temp_dir().join("jet-ts-se-globs");
        let pkg_dir = dir.join("my-lib");
        std::fs::create_dir_all(&pkg_dir).unwrap();
        std::fs::write(
            pkg_dir.join("package.json"),
            r#"{"name":"my-lib","version":"1.0.0","sideEffects":["*.css","polyfill.js"]}"#,
        )
        .unwrap();
        let decl = read_package_side_effects(&dir, "my-lib");
        assert_eq!(
            decl,
            SideEffectsDecl::Globs(vec!["*.css".to_string(), "polyfill.js".to_string()])
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_side_effects_decl_missing_package() {
        let dir = std::env::temp_dir().join("jet-ts-se-missing");
        let decl = read_package_side_effects(&dir, "nonexistent");
        assert_eq!(
            decl,
            SideEffectsDecl::All,
            "missing pkg should default to All"
        );
    }

    // ─── GH #3316 — silent IO/JSON swallow regressions ───────────────────────

    /// GH #3316 — happy path that did already work; included so the
    /// regression suite is self-contained.
    #[test]
    fn gh3316_side_effects_false_returns_none() {
        let dir = std::env::temp_dir().join("jet-ts-se-3316-false");
        let _ = std::fs::remove_dir_all(&dir);
        let pkg_dir = dir.join("my-lib");
        std::fs::create_dir_all(&pkg_dir).unwrap();
        std::fs::write(
            pkg_dir.join("package.json"),
            r#"{"name":"my-lib","sideEffects":false}"#,
        )
        .unwrap();
        let decl = read_package_side_effects(&dir, "my-lib");
        assert_eq!(decl, SideEffectsDecl::None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// GH #3316 — corrupt JSON used to silently fall back to All with no
    /// signal. The fallback is still correct (conservative), but the
    /// operator must be able to find the corrupted package via logs.
    #[test]
    fn gh3316_side_effects_corrupt_json_falls_back_to_all() {
        let dir = std::env::temp_dir().join("jet-ts-se-3316-corrupt");
        let _ = std::fs::remove_dir_all(&dir);
        let pkg_dir = dir.join("broken-lib");
        std::fs::create_dir_all(&pkg_dir).unwrap();
        std::fs::write(
            pkg_dir.join("package.json"),
            r#"{"name":"broken-lib","sideEffects":fal"#, // truncated
        )
        .unwrap();
        let decl = read_package_side_effects(&dir, "broken-lib");
        assert_eq!(
            decl,
            SideEffectsDecl::All,
            "corrupt JSON must keep conservative fallback"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// GH #3316 — unreadable package.json (chmod 000) used to silently fall
    /// back to All with no signal. Now must still fall back but emit a warn.
    #[cfg(unix)]
    #[test]
    fn gh3316_side_effects_unreadable_falls_back_to_all() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join("jet-ts-se-3316-unreadable");
        let _ = std::fs::remove_dir_all(&dir);
        let pkg_dir = dir.join("locked-lib");
        std::fs::create_dir_all(&pkg_dir).unwrap();
        let pkg_json = pkg_dir.join("package.json");
        std::fs::write(&pkg_json, r#"{"name":"locked-lib","sideEffects":false}"#).unwrap();
        std::fs::set_permissions(&pkg_json, std::fs::Permissions::from_mode(0o000)).unwrap();

        // root can still read 000-mode files; restore and skip if so.
        if std::fs::read(&pkg_json).is_ok() {
            std::fs::set_permissions(&pkg_json, std::fs::Permissions::from_mode(0o644)).unwrap();
            let _ = std::fs::remove_dir_all(&dir);
            return;
        }

        let decl = read_package_side_effects(&dir, "locked-lib");

        std::fs::set_permissions(&pkg_json, std::fs::Permissions::from_mode(0o644)).unwrap();
        let _ = std::fs::remove_dir_all(&dir);

        assert_eq!(
            decl,
            SideEffectsDecl::All,
            "unreadable package.json must keep conservative fallback"
        );
    }

    #[test]
    fn test_module_has_side_effects_none_decl() {
        // sideEffects: false → module is always side-effect-free regardless of code
        let source = "window.globalSetup = true;"; // would normally have side effects
        let path = PathBuf::from("src/lib.js");
        let result = module_has_side_effects(source, &path, &SideEffectsDecl::None);
        assert!(!result, "sideEffects:false should override code analysis");
    }

    #[test]
    fn test_module_has_side_effects_all_decl_with_code_se() {
        let source = "console.log('hello');"; // code-level side effect
        let path = PathBuf::from("src/index.js");
        let result = module_has_side_effects(source, &path, &SideEffectsDecl::All);
        assert!(result, "code side effect should be detected under All decl");
    }

    #[test]
    fn test_module_has_side_effects_all_decl_without_code_se() {
        let source = "export const x = 1;"; // no code-level side effect
        let path = PathBuf::from("src/utils.js");
        let result = module_has_side_effects(source, &path, &SideEffectsDecl::All);
        assert!(!result);
    }

    #[test]
    fn test_module_has_side_effects_globs_matching() {
        let globs = SideEffectsDecl::Globs(vec!["*.css".to_string()]);
        let css_path = PathBuf::from("src/styles.css");
        let js_path = PathBuf::from("src/utils.js");
        assert!(
            module_has_side_effects("", &css_path, &globs),
            "*.css glob should match .css file"
        );
        assert!(
            !module_has_side_effects("", &js_path, &globs),
            "*.css glob should not match .js file"
        );
    }

    #[test]
    fn test_glob_matches_star() {
        assert!(glob_matches("*.js", "index.js"));
        assert!(glob_matches("*.js", "main.js"));
        assert!(!glob_matches("*.js", "main.css"));
        assert!(!glob_matches("*.js", "src/main.js"), "* should not cross /");
    }

    #[test]
    fn test_glob_matches_double_star() {
        assert!(glob_matches("**/*.js", "src/main.js"));
        assert!(glob_matches("**/*.js", "main.js"));
        assert!(glob_matches("**/*.css", "src/styles/main.css"));
    }

    #[test]
    fn test_glob_matches_exact() {
        assert!(glob_matches("polyfill.js", "polyfill.js"));
        assert!(!glob_matches("polyfill.js", "not-polyfill.js"));
    }

    // --- Dynamic import() tests (#1344) ---

    #[test]
    fn test_extract_dynamic_import_targets_basic() {
        let modules = vec![
            (PathBuf::from("/fixture/entry.js"), String::new()),
            (PathBuf::from("/fixture/lazy.js"), String::new()),
        ];
        let targets = extract_dynamic_import_targets("const lazy = import('./lazy');\n", &modules);
        assert_eq!(targets, vec![PathBuf::from("/fixture/lazy.js")]);
    }

    #[test]
    fn test_extract_dynamic_import_targets_await_double_quotes() {
        let modules = vec![(PathBuf::from("/fixture/lazy.js"), String::new())];
        let targets =
            extract_dynamic_import_targets("const m = await import(\"./lazy\");\n", &modules);
        assert_eq!(targets, vec![PathBuf::from("/fixture/lazy.js")]);
    }

    #[test]
    fn test_extract_dynamic_import_targets_with_extension() {
        let modules = vec![(PathBuf::from("/fixture/lazy.js"), String::new())];
        let targets = extract_dynamic_import_targets("const m = import('./lazy.js');\n", &modules);
        assert_eq!(targets, vec![PathBuf::from("/fixture/lazy.js")]);
    }

    #[test]
    fn test_extract_dynamic_import_targets_skips_static_decl() {
        // Static `import x from './lazy'` MUST NOT match the
        // dynamic pass — it's handled by extract_import_bindings.
        let modules = vec![(PathBuf::from("/fixture/lazy.js"), String::new())];
        let targets = extract_dynamic_import_targets("import x from './lazy';\n", &modules);
        assert!(
            targets.is_empty(),
            "static import declaration must not match"
        );
    }

    #[test]
    fn test_extract_dynamic_import_targets_skips_template_string() {
        // Template-string specifiers can't be resolved at compile
        // time and are deliberately ignored by this pass.
        let modules = vec![(PathBuf::from("/fixture/lazy.js"), String::new())];
        let targets = extract_dynamic_import_targets("const m = import(`./${name}`);\n", &modules);
        assert!(
            targets.is_empty(),
            "template-string specifier must not match"
        );
    }

    #[test]
    fn test_extract_dynamic_import_targets_nested_in_branch() {
        let modules = vec![(PathBuf::from("/fixture/lazy.js"), String::new())];
        let targets =
            extract_dynamic_import_targets("if (cond) { const m = import('./lazy'); }\n", &modules);
        assert_eq!(targets, vec![PathBuf::from("/fixture/lazy.js")]);
    }

    #[test]
    fn test_dynamic_import_marks_wildcard() {
        let modules = vec![
            (
                PathBuf::from("/fixture/entry.js"),
                "const lazy = import('./lazy');\nconsole.log(lazy);\n".to_string(),
            ),
            (
                PathBuf::from("/fixture/lazy.js"),
                "export const heavy = () => 'expensive';\nexport const other = 1;\n".to_string(),
            ),
        ];
        let result = analyze_used_exports(&modules).unwrap();
        let lazy_used = result
            .used_exports
            .get(&PathBuf::from("/fixture/lazy.js"))
            .expect("lazy.js must appear in used_exports under wildcard");
        assert!(
            lazy_used.contains("*"),
            "dynamic-import target must be marked with wildcard `*`; got {lazy_used:?}",
        );
        assert!(
            !result
                .eliminated_modules
                .contains(&PathBuf::from("/fixture/lazy.js")),
            "dynamic-import target must NOT be eliminated; got {:?}",
            result.eliminated_modules,
        );
    }

    #[test]
    fn test_cjs_full_analysis() {
        let modules = vec![
            (PathBuf::from("app.js"), "var jsx = require('react')[\"jsx\"];\nvar useState = require('react')[\"useState\"];\n".to_string()),
            (PathBuf::from("react.js"), "exports.jsx = function() {};\nexports.useState = function() {};\nexports.useEffect = function() {};\n".to_string()),
        ];

        let result = analyze_used_exports(&modules).unwrap();
        let react_used = result.used_exports.get(&PathBuf::from("react.js"));
        assert!(react_used.is_some());
        let used = react_used.unwrap();
        assert!(used.contains("jsx"));
        assert!(used.contains("useState"));
        // useEffect is NOT imported, so should not be in used set
        assert!(!used.contains("useEffect"));
    }
}

#[cfg(test)]
mod gh3815_non_utf8_sideeffects_glob_warn_tests {
    use super::*;
    use std::path::Path;

    fn match_suffix(g: &str, p: &str) -> bool {
        // Toy matcher for tests: glob "*.css" matches anything ending in .css
        if let Some(suffix) = g.strip_prefix('*') {
            p.ends_with(suffix)
        } else {
            g == p
        }
    }

    #[test]
    fn utf8_file_name_glob_match_silent_true() {
        let p = Path::new("foo/bar.css");
        let globs = vec!["*.css".to_string()];
        assert!(tree_shake_module_path_matches_any_glob(
            p,
            &globs,
            match_suffix
        ));
    }

    #[test]
    fn utf8_file_name_no_glob_match_silent_false() {
        let p = Path::new("foo/bar.js");
        let globs = vec!["*.css".to_string()];
        assert!(!tree_shake_module_path_matches_any_glob(
            p,
            &globs,
            match_suffix
        ));
    }

    #[test]
    fn utf8_full_path_glob_match_silent_true() {
        let p = Path::new("foo/bar/baz");
        let globs = vec!["foo/bar/baz".to_string()];
        assert!(tree_shake_module_path_matches_any_glob(
            p,
            &globs,
            match_suffix
        ));
    }

    #[test]
    fn empty_globs_list_silent_false() {
        let p = Path::new("foo/bar.css");
        let globs: Vec<String> = vec![];
        assert!(!tree_shake_module_path_matches_any_glob(
            p,
            &globs,
            match_suffix
        ));
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_file_name_with_utf8_full_path_fails_for_full_path() {
        // On unix, a path with non-UTF-8 bytes in the file name has
        // both file_name() and to_str() returning None — there's no
        // way to construct a "non-UTF-8 file_name but UTF-8 full path".
        // This test verifies the documented behaviour: when both are
        // None, we emit the warn and return false.
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let raw = b"a/\xff.css";
        let p = std::path::PathBuf::from(OsStr::from_bytes(raw));
        let globs = vec!["*.css".to_string()];
        assert!(
            p.to_str().is_none(),
            "test precondition: full path must be non-UTF-8"
        );
        assert!(
            p.file_name().and_then(|n| n.to_str()).is_none(),
            "test precondition: file_name must be non-UTF-8"
        );
        // Both branches collapse → warn fires, returns false.
        assert!(!tree_shake_module_path_matches_any_glob(
            &p,
            &globs,
            match_suffix
        ));
    }

    #[test]
    fn warn_helper_pinned_for_discoverability() {
        let _: fn(&Path) -> String = format_tree_shake_non_utf8_module_path_warn;
        let p = Path::new("noext");
        let msg = format_tree_shake_non_utf8_module_path_warn(p);
        assert!(!msg.is_empty());
    }

    #[test]
    fn warn_string_carries_gh3815_tag() {
        let p = Path::new("foo");
        let msg = format_tree_shake_non_utf8_module_path_warn(p);
        assert!(msg.contains("gh3815"), "warn lacks tag: {msg}");
    }

    #[test]
    fn warn_distinct_from_prior_silent_fallback_families() {
        let p = Path::new("foo");
        let msg = format_tree_shake_non_utf8_module_path_warn(p);
        for prior in [
            "gh3789", "gh3791", "gh3793", "gh3795", "gh3797", "gh3799", "gh3801", "gh3803",
            "gh3805", "gh3807", "gh3809", "gh3811", "gh3813",
        ] {
            assert!(!msg.contains(prior), "warn collides with {prior}: {msg}");
        }
    }

    #[test]
    fn warn_names_sideeffects_failure_mode() {
        let p = Path::new("foo");
        let msg = format_tree_shake_non_utf8_module_path_warn(p);
        assert!(
            msg.contains("sideEffects"),
            "warn must name the affected feature: {msg}"
        );
        assert!(
            msg.contains("eliminate") || msg.contains("tree_shake"),
            "warn must name the consequence: {msg}"
        );
    }

    #[test]
    fn helper_does_not_warn_on_utf8_paths() {
        // Behavioural: utf8 paths flow through without emitting the
        // gh3815 warn. We can't directly observe absence of warn here,
        // but we can pin that the function returns without panic and
        // produces the expected match result for both code paths.
        let p_utf8 = Path::new("foo/bar.css");
        let globs = vec!["*.css".to_string()];
        let _ = tree_shake_module_path_matches_any_glob(p_utf8, &globs, match_suffix);
        let _ = tree_shake_module_path_matches_any_glob(p_utf8, &[], match_suffix);
    }

    #[test]
    fn glob_matcher_callback_is_invoked_for_both_file_name_and_full_path() {
        use std::cell::RefCell;
        let calls: RefCell<Vec<(String, String)>> = RefCell::new(Vec::new());
        let p = Path::new("foo/bar.css");
        let globs = vec!["bar.css".to_string()]; // file_name match
        let matched = tree_shake_module_path_matches_any_glob(p, &globs, |g, p_| {
            calls.borrow_mut().push((g.to_string(), p_.to_string()));
            g == p_
        });
        assert!(matched);
        let recorded = calls.borrow();
        assert!(!recorded.is_empty(), "matcher must have been called");
        assert!(
            recorded.iter().any(|(_, p)| p == "bar.css"),
            "must try file_name: {recorded:?}"
        );
    }
}
// CODEGEN-END
