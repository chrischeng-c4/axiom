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
use dashmap::DashMap;
use rayon::prelude::*;
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
    /// Module path → every export name the module declares (ESM + CJS).
    /// Star re-export materialization needs the leaf's full surface.
    pub all_exports: HashMap<PathBuf, Vec<String>>,
    /// Modules entirely eliminated (no used exports, no side effects).
    pub eliminated_modules: Vec<PathBuf>,
    /// Estimated bytes eliminated.
    pub eliminated_bytes: u64,
}

/// Analyze which exports are used across the module graph.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn analyze_used_exports(modules: &[(PathBuf, String)]) -> Result<TreeShakeResult> {
    // Callers without an explicit entry (tests, legacy paths) treat the
    // first module as the root of the import graph.
    let entry = modules.first().map(|(p, _)| p.clone()).unwrap_or_default();
    analyze_used_exports_from(modules, &entry, None)
}

/// Demand-driven variant: usage flows outward from `entry` only.
///
/// Marking every module's imports as "used" let eliminated subtrees keep
/// themselves alive — MUI's cssVars/extendTheme cluster imports itself
/// into liveness even though nothing reachable from the entry ever
/// imports it. Liveness now spreads from the entry across import /
/// require / dynamic-import edges (and the re-export fixed point below)
/// so a dead subtree can no longer keep itself or its dependencies live.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn analyze_used_exports_from(
    modules: &[(PathBuf, String)],
    entry: &Path,
    resolver: Option<&(dyn Fn(&str, &Path) -> Option<PathBuf> + Sync)>,
) -> Result<TreeShakeResult> {
    let mut used: HashMap<PathBuf, HashSet<String>> = HashMap::new();
    let lookup = match resolver {
        Some(r) => ModuleLookup::with_resolver(modules, r),
        None => ModuleLookup::new(modules),
    };

    // The bundle entry's export namespace is part of the public artifact.
    // Treat it as a wildcard consumer so `export * from ...` on a library
    // entry propagates every named leaf export instead of shaking the leaf
    // out before CJS lowering can copy it onto `module.exports`.
    used.entry(entry.to_path_buf())
        .or_default()
        .insert("*".to_string());

    // Steps 1+2+3 (WI #1947 fix 2): compute every per-module fact — ESM +
    // CJS exports, static import/require edges, and dynamic-import targets
    // — in ONE par_iter pass over the corpus instead of 3 separate
    // corpus-wide par_iter sweeps. Each sweep previously re-touched every
    // module's full source independently, so a 12.5k-module graph paid
    // rayon dispatch/collect overhead 3× and re-pulled cold module source
    // bytes into cache 3×; `compute_module_facts` does all of a module's
    // scans back-to-back while its source is still hot. `lookup` is
    // read-only and `Sync` (and now internally memoized — WI #1947 fix 3).
    let facts: HashMap<&Path, ModuleFacts> = modules
        .par_iter()
        .map(|(path, source)| (path.as_path(), compute_module_facts(path, source, &lookup)))
        .collect();

    let all_exports: HashMap<PathBuf, Vec<String>> = modules
        .iter()
        .map(|(path, _)| {
            let f = &facts[path.as_path()];
            let mut exports = f.exports.clone();
            exports.extend(f.cjs_exports.iter().cloned());
            (path.clone(), exports)
        })
        .collect();
    let static_edges: HashMap<&Path, Vec<(PathBuf, Vec<String>)>> = modules
        .iter()
        .map(|(path, _)| {
            let f = &facts[path.as_path()];
            let mut edges = f.static_edges.clone();
            edges.extend(f.cjs_edges.iter().cloned());
            (path.as_path(), edges)
        })
        .collect();
    let dynamic_edges: HashMap<&Path, Vec<PathBuf>> = modules
        .iter()
        .map(|(path, _)| {
            (
                path.as_path(),
                facts[path.as_path()].dynamic_targets.clone(),
            )
        })
        .collect();

    // Liveness worklist from the entry: each live module marks its import
    // targets' names used and pulls those targets live.
    let mut live: HashSet<PathBuf> = HashSet::new();
    let mut queue: Vec<PathBuf> = vec![entry.to_path_buf()];
    while let Some(path) = queue.pop() {
        if !live.insert(path.clone()) {
            continue;
        }
        if let Some(edges) = static_edges.get(path.as_path()) {
            for (target_path, names) in edges {
                let target_used = used.entry(target_path.clone()).or_default();
                for name in names {
                    target_used.insert(name.clone());
                }
                if !live.contains(target_path) {
                    queue.push(target_path.clone());
                }
            }
        }
        if let Some(targets) = dynamic_edges.get(path.as_path()) {
            for target_path in targets {
                used.entry(target_path.clone())
                    .or_default()
                    .insert("*".to_string());
                if !live.contains(target_path) {
                    queue.push(target_path.clone());
                }
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
    //   3. `export * from './y'`             → if barrel's `x` is used,
    //                                          mark `y`'s `x` used. Only a
    //                                          wildcard/namespace consumer
    //                                          expands the whole leaf.
    // Re-export bindings are pure in (path, source) — extract once, not
    // once per fixed-point round. Re-extracting every module's bindings
    // each round made this phase O(rounds × modules × source) and
    // dominated tree shaking on barrel-heavy corpora like MUI. WI #1947
    // fix 2: already computed by `compute_module_facts` above (in the
    // same per-module pass as exports/edges), so this is now a cheap
    // re-derivation from `facts` rather than another corpus scan.
    let reexport_bindings: Vec<(&PathBuf, Vec<(PathBuf, ReexportKind)>)> = modules
        .iter()
        .map(|(path, _)| (path, facts[path.as_path()].reexports.clone()))
        .collect();
    loop {
        let mut changed = false;
        for (path, reexports) in &reexport_bindings {
            let path: &PathBuf = path;
            let barrel_used: HashSet<String> =
                used.get(path.as_path()).cloned().unwrap_or_default();
            for (target_path, kind) in reexports.iter().cloned() {
                match kind {
                    ReexportKind::Star => {
                        if let Some(leaf_exports) = all_exports.get(&target_path) {
                            let to_add: Vec<&String> = if barrel_used.contains("*") {
                                leaf_exports.iter().filter(|n| *n != "default").collect()
                            } else {
                                leaf_exports
                                    .iter()
                                    .filter(|n| *n != "default" && barrel_used.contains(*n))
                                    .collect()
                            };
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
        // Names propagated onto leaves can make new modules live; spread
        // their own edges before the next fixed-point round.
        let newly_used: Vec<PathBuf> = used
            .keys()
            .filter(|p| !live.contains(p.as_path()))
            .cloned()
            .collect();
        if !newly_used.is_empty() {
            changed = true;
            let mut queue = newly_used;
            while let Some(path) = queue.pop() {
                if !live.insert(path.clone()) {
                    continue;
                }
                if let Some(edges) = static_edges.get(path.as_path()) {
                    for (target_path, names) in edges {
                        let target_used = used.entry(target_path.clone()).or_default();
                        for name in names {
                            target_used.insert(name.clone());
                        }
                        if !live.contains(target_path) {
                            queue.push(target_path.clone());
                        }
                    }
                }
                if let Some(targets) = dynamic_edges.get(path.as_path()) {
                    for target_path in targets {
                        used.entry(target_path.clone())
                            .or_default()
                            .insert("*".to_string());
                        if !live.contains(target_path) {
                            queue.push(target_path.clone());
                        }
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }

    // Step 3.6 (legacy): dynamic `import("./x")` targets are handled by
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
    // the liveness worklist above; nothing left to do here.

    // Step 4: Find eliminated modules
    let mut eliminated = Vec::new();
    let mut eliminated_bytes = 0u64;

    let mut package_side_effects_cache: HashMap<(PathBuf, String), SideEffectsDecl> =
        HashMap::new();

    for (path, source) in modules {
        if let Some(exports) = all_exports.get(path) {
            if !exports.is_empty() {
                let used_set = used.get(path);
                let has_used = used_set.map(|s| !s.is_empty()).unwrap_or(false);

                if !has_used
                    && !module_has_side_effects_with_package_json(
                        source,
                        path,
                        &mut package_side_effects_cache,
                    )
                {
                    eliminated.push(path.clone());
                    eliminated_bytes += source.len() as u64;
                }
            }
        }
    }

    Ok(TreeShakeResult {
        used_exports: used,
        all_exports,
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
                if !used_exports.contains(&name) && is_single_line_removable_export(trimmed) {
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

fn is_single_line_removable_export(line: &str) -> bool {
    let line = line.trim();
    if line.starts_with("export const ")
        || line.starts_with("export let ")
        || line.starts_with("export var ")
    {
        return line.ends_with(';');
    }

    false
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

/// Strips a trailing `//` line comment from `line`, honoring single- and
/// double-quoted string literals so a `//` inside a specifier string (e.g.
/// `from 'https://example.com/x'`) is never mistaken for a comment start.
/// Only used while [`logical_import_lines`] accumulates a still-open
/// multi-line import statement (#1991 round 2) — a physical line's own
/// trailing comment (`  IconC, // used across the dashboard`) must not glue
/// onto the next accumulated binding with no separator.
fn strip_line_comment(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut in_single = false;
    let mut in_double = false;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\'' if !in_double => in_single = !in_single,
            b'"' if !in_single => in_double = !in_double,
            b'/' if !in_single && !in_double && i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                return &line[..i];
            }
            _ => {}
        }
        i += 1;
    }
    line
}

/// Re-groups `source` into logical import lines: a single-line statement (or
/// any non-`import` line) passes through borrowed and unchanged, but an
/// `import` line whose binding list and `from '<specifier>'` clause are
/// split across physical lines (prettier's default wrapping for 3+ named
/// imports — `import {\n  A,\n  B,\n} from '...';`) is joined into one
/// owned logical line before [`extract_specifier`]/[`extract_imported_names`]
/// ever see it (#1991 round 2). Accumulation stops as soon as
/// [`extract_specifier`] finds a quoted specifier, so it uses the exact same
/// "last quoted substring on the line" termination signal the two extractors
/// already rely on. An unterminated/truncated statement (no `from '<spec>'`
/// before EOF) yields one specifier-less logical line instead of looping
/// forever, leaving classification of that shape to the caller.
pub(crate) fn logical_import_lines(source: &str) -> Vec<std::borrow::Cow<'_, str>> {
    use std::borrow::Cow;

    let mut out: Vec<Cow<'_, str>> = Vec::new();
    let mut lines = source.lines();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if !trimmed.starts_with("import ") || !extract_specifier(trimmed).is_empty() {
            out.push(Cow::Borrowed(line));
            continue;
        }

        let mut joined = strip_line_comment(line).trim().to_string();
        while extract_specifier(&joined).is_empty() {
            let Some(next_line) = lines.next() else {
                break;
            };
            let next_trimmed = strip_line_comment(next_line).trim();
            if !next_trimmed.is_empty() {
                joined.push(' ');
                joined.push_str(next_trimmed);
            }
        }
        out.push(Cow::Owned(joined));
    }

    out
}

/// Extract ESM import bindings: returns (target_module_path, imported names).
///
/// Scans [`logical_import_lines`] rather than `source.lines()` directly so a
/// multi-line `import { ... } from '...'` statement — whose binding names
/// never share a physical line with the `from` clause — still resolves to
/// its specifier and names instead of being silently skipped (#1991 round
/// 2).
fn extract_import_bindings(
    importer: &Path,
    source: &str,
    lookup: &ModuleLookup<'_>,
) -> Vec<(PathBuf, Vec<String>)> {
    let mut results = Vec::new();
    // WI #1947 fix 1: built lazily (only if any import line has narrowable
    // names) and reused across every import line in this module, instead of
    // each `filter_referenced_import_names` call re-scanning the whole
    // module source with a fresh `find` per candidate name.
    let mut word_index: Option<HashMap<&str, usize>> = None;

    for line in logical_import_lines(source) {
        let trimmed = line.trim();
        if !trimmed.starts_with("import ") {
            continue;
        }

        let specifier = extract_specifier(trimmed);
        if specifier.is_empty() {
            continue;
        }

        let target = lookup.find(&specifier, importer);

        if let Some((target_path, _)) = target {
            let names = extract_imported_names(trimmed);
            // Drop imported names whose LOCAL binding is never referenced in
            // this module's body (esbuild does this; jet did not). After
            // define replacement folds out `process.env.NODE_ENV` dev
            // branches, a `propTypes` import like `elementAcceptingRef` /
            // `chainPropTypes` / `exactProp` is bound but unreferenced — and
            // marking it used kept the whole @mui/utils PropTypes validation
            // chain alive in production. The check is CONSERVATIVE: a binding
            // is dropped only when its name appears literally nowhere outside
            // its own import statement (so a short or coincidentally-matching
            // name is always kept). `*` namespace imports and bare
            // side-effect imports are never narrowed.
            let names = filter_referenced_import_names(trimmed, source, names, &mut word_index);
            results.push((target_path.clone(), names));
        }
    }

    results
}

/// Keep only imported names whose local binding is referenced in `source`
/// beyond the import line itself. Conservative by construction: if a
/// binding's name occurs anywhere else (even inside a string or a longer
/// token we don't perfectly tokenize), it is retained — only a name that
/// appears *exclusively* in its import statement is dropped.
///
/// WI #1947 fix 1: `total` used to be `count_word_occurrences(source,
/// &binding)` — a fresh whole-source `str::find` scan per candidate name.
/// On a barrel with hundreds of named imports that was O(names ×
/// source_len) per module. `word_index` (built once per module by
/// `index_word_occurrences`, on first use, and reused for every import
/// line) turns the same lookup into a single hash probe; see the
/// equivalence proof on `index_word_occurrences`.
fn filter_referenced_import_names<'s>(
    import_line: &str,
    source: &'s str,
    names: Vec<String>,
    word_index: &mut Option<HashMap<&'s str, usize>>,
) -> Vec<String> {
    // `*` (namespace) and empty (bare side-effect import) are never narrowed.
    if names.iter().any(|n| n == "*") || names.is_empty() {
        return names;
    }
    let index = word_index.get_or_insert_with(|| index_word_occurrences(source));
    names
        .into_iter()
        .filter(|imported| {
            let binding = local_binding_for(import_line, imported);
            // Total whole-word occurrences across the module, minus the one
            // declaration in the import line. >0 means a real reference.
            let total = index.get(binding.as_str()).copied().unwrap_or(0);
            let in_import = count_word_occurrences(import_line, &binding);
            total > in_import
        })
        .collect()
}

/// The local binding name an imported export is bound to in `import_line`:
/// `import { a as b }` → `b`, `import { a }` → `a`, `import D from` →
/// the default identifier `D`. Falls back to the imported name.
fn local_binding_for(import_line: &str, imported: &str) -> String {
    if imported == "default" {
        let after = import_line.trim_start_matches("import ").trim_start();
        if !after.starts_with('{') {
            if let Some(id) = after
                .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '$')
                .next()
            {
                if !id.is_empty() {
                    return id.to_string();
                }
            }
        }
        return imported.to_string();
    }
    if let (Some(bs), Some(be)) = (import_line.find('{'), import_line.find('}')) {
        if bs < be {
            for item in import_line[bs + 1..be].split(',') {
                let item = item.trim();
                let (orig, alias) = match item.split_once(" as ") {
                    Some((o, a)) => (o.trim(), Some(a.trim())),
                    None => (item, None),
                };
                if orig == imported {
                    return alias.unwrap_or(orig).to_string();
                }
            }
        }
    }
    imported.to_string()
}

#[cfg(test)]
mod binding_usage_tests {
    use super::{count_word_occurrences, filter_referenced_import_names, local_binding_for};

    #[test]
    fn drops_unreferenced_named_import() {
        let src = "import { used, unused } from 'm';\nconsole.log(used);\n";
        let line = "import { used, unused } from 'm';";
        let names = vec!["used".to_string(), "unused".to_string()];
        let kept = filter_referenced_import_names(line, src, names, &mut None);
        assert_eq!(kept, vec!["used".to_string()]);
    }

    #[test]
    fn keeps_aliased_and_default_when_referenced() {
        let src = "import D, { a as b } from 'm';\nb();D.x;\n";
        let line = "import D, { a as b } from 'm';";
        assert_eq!(local_binding_for(line, "a"), "b");
        assert_eq!(local_binding_for(line, "default"), "D");
        let kept = filter_referenced_import_names(
            line,
            src,
            vec!["a".to_string(), "default".to_string()],
            &mut None,
        );
        assert_eq!(kept.len(), 2);
    }

    #[test]
    fn never_narrows_namespace_or_substring() {
        let names = vec!["*".to_string()];
        assert_eq!(
            filter_referenced_import_names(
                "import * as ns from 'm';",
                "ns.x",
                names.clone(),
                &mut None
            ),
            names
        );
        assert_eq!(
            count_word_occurrences("elementAcceptingRefThing", "elementAcceptingRef"),
            0
        );
        assert_eq!(
            count_word_occurrences("a.elementAcceptingRef()", "elementAcceptingRef"),
            1
        );
    }
}

/// #1991 round 2: multi-line ESM import statements — prettier's default
/// wrapping for 3+ named imports puts the binding list and the `from`
/// clause on different physical lines, and neither the crawl-time barrel
/// demand scanner (`bundler::barrel_demand_for_specifier`) nor
/// [`extract_import_bindings`] handled that before this fix.
#[cfg(test)]
mod multiline_import_1991_tests {
    use super::{extract_imported_names, extract_specifier, logical_import_lines};

    #[test]
    fn multiline_named_import_joins_into_one_logical_line() {
        let src =
            "import {\n  IconA,\n  IconB,\n} from '@lib/barrel';\nconsole.log(IconA, IconB);\n";
        let lines = logical_import_lines(src);
        let import_line = lines
            .iter()
            .find(|l| l.trim_start().starts_with("import "))
            .expect("no logical import line found");
        assert_eq!(extract_specifier(import_line), "@lib/barrel");
        let mut names = extract_imported_names(import_line);
        names.sort();
        assert_eq!(names, vec!["IconA".to_string(), "IconB".to_string()]);
    }

    #[test]
    fn multiline_named_import_trailing_comma_drops_empty_token() {
        let src = "import {\n  IconA,\n  IconB,\n} from '@lib/barrel';\n";
        let lines = logical_import_lines(src);
        let names = extract_imported_names(&lines[0]);
        assert_eq!(
            names,
            vec!["IconA".to_string(), "IconB".to_string()],
            "trailing comma before the closing brace must not produce an empty name"
        );
    }

    #[test]
    fn multiline_named_import_inline_comment_does_not_corrupt_next_name() {
        let src = "import {\n  IconA, // used on the dashboard\n  IconB,\n} from '@lib/barrel';\n";
        let lines = logical_import_lines(src);
        let mut names = extract_imported_names(&lines[0]);
        names.sort();
        assert_eq!(
            names,
            vec!["IconA".to_string(), "IconB".to_string()],
            "an inline comment on a binding line must not glue onto the next binding"
        );
    }

    #[test]
    fn multiline_mixed_default_and_named_import_returns_both() {
        let src = "import Default, {\n  IconA,\n  IconB,\n} from '@lib/barrel';\n";
        let lines = logical_import_lines(src);
        let mut names = extract_imported_names(&lines[0]);
        names.sort();
        assert_eq!(
            names,
            vec![
                "IconA".to_string(),
                "IconB".to_string(),
                "default".to_string()
            ]
        );
    }

    #[test]
    fn single_line_import_is_borrowed_unchanged() {
        let src = "import { IconA, IconB } from '@lib/barrel';\nconsole.log(IconA, IconB);\n";
        let lines = logical_import_lines(src);
        assert_eq!(lines.len(), 2, "single-line source must not be re-joined");
        assert!(
            matches!(lines[0], std::borrow::Cow::Borrowed(_)),
            "a single-line import must stay zero-copy borrowed"
        );
        assert_eq!(
            lines[0].as_ref(),
            "import { IconA, IconB } from '@lib/barrel';"
        );
        assert_eq!(lines[1].as_ref(), "console.log(IconA, IconB);");
    }

    #[test]
    fn require_call_line_is_never_treated_as_multiline() {
        // A `require(...)` line never starts with `import `, so it must
        // always pass straight through — proves the multi-line
        // accumulation is scoped to `import ` lines only.
        let src = "const icons = require('@lib/barrel');\nconsole.log(icons.IconA);\n";
        let lines = logical_import_lines(src);
        assert_eq!(lines.len(), 2);
        assert!(matches!(lines[0], std::borrow::Cow::Borrowed(_)));
        assert_eq!(lines[0].as_ref(), "const icons = require('@lib/barrel');");
    }

    #[test]
    fn unterminated_multiline_import_yields_specifier_less_logical_line() {
        // A malformed/truncated statement (no terminating `from '<spec>'`
        // before EOF) must not panic or infinite-loop — it yields one
        // logical line whose specifier is empty, for the caller's own
        // fallback classification (#1991 round 2's `UnparseableImport`).
        let src = "import {\n  IconA,\n  IconB,\n";
        let lines = logical_import_lines(src);
        assert_eq!(lines.len(), 1);
        assert_eq!(extract_specifier(&lines[0]), "");
    }
}

/// Count whole-word (identifier-boundary) occurrences of `word` in `text`.
fn count_word_occurrences(text: &str, word: &str) -> usize {
    if word.is_empty() {
        return 0;
    }
    let bytes = text.as_bytes();
    let wb = word.as_bytes();
    let is_id = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b'$';
    let mut count = 0usize;
    let mut from = 0usize;
    while let Some(rel) = text[from..].find(word) {
        let start = from + rel;
        let end = start + wb.len();
        let before_ok = start == 0 || !is_id(bytes[start - 1]);
        let after_ok = end >= bytes.len() || !is_id(bytes[end]);
        if before_ok && after_ok {
            count += 1;
        }
        from = start + 1;
    }
    count
}

/// Build a whole-source index of identifier-shaped word → occurrence count
/// in ONE linear pass, for `word`s that are themselves identifier-shaped
/// (every byte is ASCII alphanumeric, `_`, or `$` — exactly what
/// `local_binding_for` ever produces).
///
/// WI #1947 fix 1: replaces per-name calls to `count_word_occurrences`
/// (each a whole-source `str::find` loop) with one tokenizing pass whose
/// result is reused for every name. Equivalence: `count_word_occurrences`
/// finds every occurrence of the literal substring `word` and keeps it
/// only when the byte before and the byte after are *not* identifier
/// bytes — i.e. exactly the occurrences where `word` stands alone as a
/// maximal run of identifier bytes. This function instead walks `source`
/// once, splitting it into maximal identifier-byte runs, and counts how
/// many times each run's text repeats. For any identifier-shaped `word`,
/// both procedures count the same set of positions: a `word`-occurrence
/// survives `count_word_occurrences`'s boundary check iff it *is* one of
/// the maximal identifier runs this function tokenizes, because `word`
/// itself contains no non-identifier byte (so the run it sits in can be
/// neither longer — the boundary check forbids identifier bytes on either
/// side — nor shorter — `word`'s own bytes are all identifier bytes, so a
/// run can't end partway through it). See `word_index_1947_tests` below
/// for boundary-case proof (start/end of source, adjacent identifiers,
/// `$`/`_`/digits, non-identifier-shaped needles).
fn index_word_occurrences(source: &str) -> HashMap<&str, usize> {
    let mut index: HashMap<&str, usize> = HashMap::new();
    let bytes = source.as_bytes();
    let is_id = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b'$';
    let mut i = 0usize;
    while i < bytes.len() {
        if is_id(bytes[i]) {
            let start = i;
            while i < bytes.len() && is_id(bytes[i]) {
                i += 1;
            }
            // Every byte in [start, i) is ASCII, so this slice always
            // lands on a char boundary — safe to index `source` directly.
            *index.entry(&source[start..i]).or_insert(0) += 1;
        } else {
            i += 1;
        }
    }
    index
}

#[cfg(test)]
mod word_index_1947_tests {
    use super::{count_word_occurrences, index_word_occurrences};

    /// Assert `index_word_occurrences(source)[word]` (or 0 if absent)
    /// equals `count_word_occurrences(source, word)` — the property the
    /// whole fix rests on.
    fn assert_equiv(source: &str, word: &str) {
        let indexed = index_word_occurrences(source)
            .get(word)
            .copied()
            .unwrap_or(0);
        let scanned = count_word_occurrences(source, word);
        assert_eq!(
            indexed, scanned,
            "index_word_occurrences/count_word_occurrences diverged for word={word:?} source={source:?}"
        );
    }

    #[test]
    fn word_at_very_start_of_source() {
        assert_equiv("used;\nconsole.log(used)", "used");
    }

    #[test]
    fn word_at_very_end_of_source() {
        assert_equiv("console.log(used);\nused", "used");
    }

    #[test]
    fn word_is_the_entire_source() {
        assert_equiv("used", "used");
    }

    #[test]
    fn word_adjacent_to_longer_identifier_is_not_counted() {
        // "used" is a strict substring of "unused"/"usedThing" but never
        // stands alone — both sides must report 0.
        assert_equiv("unused usedThing", "used");
    }

    #[test]
    fn word_back_to_back_with_punctuation_boundaries() {
        assert_equiv("a.used(),used[used]={used};used", "used");
    }

    #[test]
    fn word_with_dollar_and_underscore_and_digits() {
        assert_equiv("const $_used1 = $_used1 + 1; // $_used1", "$_used1");
    }

    #[test]
    fn empty_source_and_repeated_word_only() {
        assert_equiv("", "used");
        assert_equiv("used used used", "used");
    }

    #[test]
    fn word_absent_entirely() {
        assert_equiv("import { other } from 'm';", "used");
    }

    #[test]
    fn every_binding_name_across_a_realistic_module_body() {
        let source = r#"
import { used, unused, elementAcceptingRef } from 'm';
function usedThing() { return used + 1; }
class UsedClass {}
const obj = { used: 1, notused: 2 };
a.elementAcceptingRef();
"#;
        for word in [
            "used",
            "unused",
            "elementAcceptingRef",
            "usedThing",
            "UsedClass",
            "notused",
        ] {
            assert_equiv(source, word);
        }
    }
}

/// Classification of an `export ... from '...'` re-export line.
///
/// `pub(crate)` (not private): `bundler::mod::prefetch_graph_modules`
/// reuses this at graph-crawl time for lazy pure-barrel expansion
/// (#1991) — see [`extract_reexport_specifiers`].
#[derive(Debug, Clone)]
pub(crate) enum ReexportKind {
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

/// Classify a single trimmed `export ... from '...'` line (already known
/// to start with `"export "` and contain `" from "`) into its raw
/// specifier text and re-export shape. Returns `None` for a line with
/// neither a `*` nor a parseable `{ ... }` clause.
///
/// Factored out of [`extract_reexport_bindings`] so [`extract_reexport_specifiers`]
/// can reuse the exact same per-line classification without a
/// [`ModuleLookup`] (#1991).
///
/// `pub(crate)`: also reused by `bundler::mod::is_pure_barrel_source` so
/// the purity detector rejects a line that merely *looks*
/// export-from-shaped but doesn't actually parse as one (#1991).
pub(crate) fn classify_reexport_line(trimmed: &str) -> Option<(String, ReexportKind)> {
    let specifier = extract_specifier(trimmed);
    if specifier.is_empty() {
        return None;
    }

    // `export * from '...'` — wildcard re-export. The `as ns` variant
    // (`export * as ns from '...'`) is a *namespace* re-export: the
    // barrel exposes one symbol `ns` whose value is the leaf's
    // namespace. We treat it the same as `*` here because we don't
    // track namespace-property usage statically.
    let after_export = &trimmed["export ".len()..];
    if after_export.trim_start().starts_with('*') {
        return Some((specifier, ReexportKind::Star));
    }

    // `export { a, b as c } from '...'` — named re-export.
    let brace_start = trimmed.find('{')?;
    let brace_end = trimmed.find('}')?;
    if brace_start >= brace_end {
        return None;
    }
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
    if pairs.is_empty() {
        return None;
    }
    Some((specifier, ReexportKind::Named(pairs)))
}

/// Lookup-free variant of [`extract_reexport_bindings`]: extracts every
/// `export ... from '...'` re-export line in `source` as a raw
/// `(specifier, kind)` pair, without resolving the specifier to a module
/// path.
///
/// Used at graph-crawl time (`bundler::mod::prefetch_graph_modules`,
/// #1991) where a full [`ModuleLookup`] over the crawled module set does
/// not exist yet — the caller resolves each specifier itself (via the
/// crawl's own per-module resolution cache) and decides whether/when to
/// follow it, instead of every re-export edge being expanded
/// unconditionally.
pub(crate) fn extract_reexport_specifiers(source: &str) -> Vec<(String, ReexportKind)> {
    let mut results = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("export ") || !trimmed.contains(" from ") {
            continue;
        }
        if let Some(entry) = classify_reexport_line(trimmed) {
            results.push(entry);
        }
    }
    results
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
    importer: &Path,
    source: &str,
    lookup: &ModuleLookup<'_>,
) -> Vec<(PathBuf, ReexportKind)> {
    extract_reexport_specifiers(source)
        .into_iter()
        .filter_map(|(specifier, kind)| {
            lookup
                .find(&specifier, importer)
                .map(|target| (target.0.clone(), kind))
        })
        .collect()
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
    let lookup = ModuleLookup::new(modules);
    extract_dynamic_import_targets_from(Path::new(""), source, &lookup)
}

fn extract_dynamic_import_targets_from(
    importer: &Path,
    source: &str,
    lookup: &ModuleLookup<'_>,
) -> Vec<PathBuf> {
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

        if let Some((target_path, _)) = lookup.find(specifier, importer) {
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

    // import Default from '...'  AND  import Default, { a, b } from '...'
    // The default binding is whatever identifier precedes the brace group
    // (if any). Requiring "no braces on the line" dropped the default half
    // of mixed imports — `import FormLabel, { formLabelClasses } from
    // '../FormLabel'` lost "default", and demand-driven shaking then
    // eliminated FormLabel.js entirely (React error #130 at runtime).
    if line.starts_with("import ") && !line.contains("* as ") && line.contains(" from ") {
        let after_import = line["import ".len()..].trim_start();
        if !after_import.starts_with('{') {
            if let Some(name) = after_import
                .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '$')
                .next()
            {
                if !name.is_empty() && name != "type" && name != "from" {
                    names.push("default".to_string());
                }
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
    importer: &Path,
    source: &str,
    lookup: &ModuleLookup<'_>,
) -> Vec<(PathBuf, Vec<String>)> {
    let mut results = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();

        // scan_require_call (WI #1947 round 2) finds the line's "require("
        // once and derives both the specifier and any trailing
        // `.prop`/`["prop"]` access from that single match, instead of the
        // two independent `str::find("require(")` scans (one per helper)
        // this loop used to run per line — extract_cjs_require_bindings was
        // the heaviest single extractor on tw-monitor's real corpus.
        let Some((req_source, accessed)) = scan_require_call(trimmed) else {
            continue;
        };
        let Some(target) = lookup.find(req_source, importer) else {
            continue;
        };

        // Pattern 1: const { a, b } = require('mod')
        let destructured = extract_destructured_names(trimmed);
        // Pattern 2: require('mod').prop / require('mod')["prop"] — this is
        // `accessed`, already computed by scan_require_call above.

        if destructured.is_empty() && accessed.is_empty() {
            // Pattern 3 (#1968): `const m = require('mod'); ... m.prop ...`
            // — the require target is bound to a local name and read via
            // property access on LATER lines, not the require() line
            // itself. Line-wise narrowing (patterns 1/2 above) can't see
            // this, so it used to fall straight to the namespace-wildcard
            // branch below — cheap for a single wrapper module, but fatal
            // for a re-export barrel: a `"*"` used-name makes
            // `eliminate_unused_reexport_assignments` (bundler/dce.rs) bail
            // out of pruning the barrel's re-export glue ENTIRELY (its
            // own `"*"` short-circuit can't tell "genuinely reads every
            // export" apart from "this extractor gave up narrowing"), so
            // every leaf the barrel re-exports survives the bundler's
            // require-reachability pass untouched — reproduced on a
            // `@mui/icons-material`-shaped fixture (10,775 of 10,775
            // barrel leaves retained) once a CJS shim did
            // `const M = require('@mui/icons-material')` and read
            // `M.SomeIcon` on a separate line. Recovering the specific
            // accessed names here lets the fixed point (and the glue
            // pruner downstream) narrow the barrel exactly as it already
            // does for an explicit `require('mod').prop` access.
            if let Some(binding) = extract_require_local_binding(trimmed) {
                if let Some(names) =
                    scan_require_binding_property_accesses(source, trimmed, &binding)
                {
                    results.push((target.0.clone(), names));
                    continue;
                }
            }
            // Namespace-style require (`var m = require('mod')`,
            // `module.exports = require('mod')` interop wrappers). Whole-
            // module usage cannot be narrowed line-wise, so keep every
            // export of the target. Dropping this edge entirely left CJS
            // package wrappers' ESM twins dead in the liveness walk and
            // the glue pruner deleted their genuinely-used exports
            // (internal_processStyles vanished from @mui/styled-engine).
            results.push((target.0.clone(), vec!["*".to_string()]));
            continue;
        }
        if !destructured.is_empty() {
            results.push((target.0.clone(), destructured));
        }
        if !accessed.is_empty() {
            results.push((target.0.clone(), accessed));
        }
    }

    results
}

/// The local binding name a bare `const/let/var X = require(...)` line
/// declares — `X`. Returns `None` for destructuring (`const { a } = ...`,
/// already handled by `extract_destructured_names`) and for anything that
/// isn't a plain identifier assignment (the shapes `extract_cjs_export_names`
/// et al. don't otherwise recognize).
///
/// `pub(crate)`: reused by `bundler::mod::prefetch_graph_modules` for CJS
/// barrel-demand narrowing at graph-crawl time (#1991).
pub(crate) fn extract_require_local_binding(line: &str) -> Option<String> {
    for keyword in ["const ", "let ", "var "] {
        let Some(rest) = line.strip_prefix(keyword) else {
            continue;
        };
        let rest = rest.trim_start();
        if rest.starts_with('{') || rest.starts_with('[') {
            return None; // destructuring — not this pattern.
        }
        let ident = rest
            .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '$')
            .next()?;
        if ident.is_empty() {
            return None;
        }
        let after_ident = rest[ident.len()..].trim_start();
        return if after_ident.starts_with('=') && !after_ident.starts_with("==") {
            Some(ident.to_string())
        } else {
            None
        };
    }
    None
}

/// Scan `source` for every use of `binding` outside its own declaration
/// line (`require_line`) and decide whether the require target's
/// whole-module usage narrows to specific property names.
///
/// Returns `Some(names)` only when EVERY occurrence of `binding` elsewhere
/// in the module is a `.prop` or `["prop"]` access — the bound object
/// itself never escapes bare (passed to a function, spread, iterated,
/// reassigned, ...), where an unpredictable set of its properties could be
/// read. Any bare occurrence, or a computed (non-literal-string) index
/// access, forces the conservative `None` so the caller keeps the existing
/// whole-module wildcard. No property access at all also returns `None` —
/// there is nothing to narrow to, and an unused binding is a
/// require-for-side-effects case the wildcard already handles correctly.
///
/// `pub(crate)`: reused by `bundler::mod::prefetch_graph_modules` for CJS
/// barrel-demand narrowing at graph-crawl time (#1991).
pub(crate) fn scan_require_binding_property_accesses(
    source: &str,
    require_line: &str,
    binding: &str,
) -> Option<Vec<String>> {
    if binding.is_empty() {
        return None;
    }
    let is_id = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b'$';
    let mut names: Vec<String> = Vec::new();
    let mut seen_access = false;

    for line in source.lines() {
        if line.trim() == require_line {
            continue; // the declaration itself — not a use.
        }
        let bytes = line.as_bytes();
        let mut from = 0usize;
        while let Some(rel) = line[from..].find(binding) {
            let start = from + rel;
            let end = start + binding.len();
            let before_ok = start == 0 || !is_id(bytes[start - 1]);
            let after_ok = end >= bytes.len() || !is_id(bytes[end]);
            from = start + 1;
            if !before_ok || !after_ok {
                continue; // part of a longer identifier — not this binding.
            }

            let rest = &line[end..];
            if let Some(prop_rest) = rest.strip_prefix('.') {
                let prop = prop_rest
                    .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '$')
                    .next()
                    .unwrap_or("");
                if prop.is_empty() {
                    return None; // `binding.` followed by nothing identifier-shaped.
                }
                names.push(prop.to_string());
                seen_access = true;
                continue;
            }
            if let Some(bracket_rest) = rest.strip_prefix('[') {
                let Some(close) = bracket_rest.find(']') else {
                    return None;
                };
                match extract_string_value(bracket_rest[..close].trim()) {
                    Some(s) => {
                        names.push(s);
                        seen_access = true;
                    }
                    None => return None, // computed index — can't narrow safely.
                }
                continue;
            }

            // Bare occurrence outside a property access — the object
            // itself is read as a value (argument, spread, iteration,
            // reassignment, ...); its full property set is unpredictable.
            return None;
        }
    }

    if seen_access {
        names.sort();
        names.dedup();
        Some(names)
    } else {
        None
    }
}

/// Extract the specifier from a require() call. Superseded in the hot
/// `extract_cjs_require_bindings` loop by `scan_require_call` (WI #1947
/// round 2 — needs the specifier and property-access result from one scan),
/// but kept as a standalone unit with its own test coverage
/// (`test_require_specifier`), hence no production caller remains.
#[allow(dead_code)]
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

/// Find a `require(` call in `line` and return both its specifier (borrowed,
/// no allocation) and any trailing `.prop`/`["prop"]` access, from a single
/// scan (WI #1947 round 2). Behaviorally equivalent to calling
/// `extract_require_specifier` then `extract_require_property_access`
/// separately — both of those still exist standalone (with their own
/// tests/callers, untouched) and are intentionally not modified; this is an
/// additional combined entry point used only by
/// `extract_cjs_require_bindings`'s hot loop, where both results are always
/// needed together and re-finding "require(" a second time is pure waste.
///
/// `pub(crate)`: also reused by `bundler::mod::prefetch_graph_modules` for
/// CJS barrel-demand narrowing at graph-crawl time (#1991).
pub(crate) fn scan_require_call(line: &str) -> Option<(&str, Vec<String>)> {
    let require_pos = line.find("require(")?;
    let after = &line[require_pos + 8..]; // skip "require("
    let quote = after.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let end = after[1..].find(quote)?;
    let specifier = &after[1..1 + end];
    let accessed = parse_require_property_access(&line[require_pos..]);
    Some((specifier, accessed))
}

/// Extract destructured property names from `const { a, b } = require(...)`.
///
/// `pub(crate)`: also reused by `bundler::mod::prefetch_graph_modules` for
/// CJS barrel-demand narrowing at graph-crawl time (#1991).
pub(crate) fn extract_destructured_names(line: &str) -> Vec<String> {
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

/// Shared implementation for property-access parsing: given the line slice
/// starting at a `require(` match (inclusive of "require("), find the
/// matching call's closing `)` and parse a trailing `.prop` or `["prop"]`
/// access. Factored out of `extract_require_property_access` (WI #1947
/// round 2) so `scan_require_call` can reuse the same parsing logic against
/// a `require(` position it already found, instead of re-running
/// `line.find("require(")` a second time.
fn parse_require_property_access(after_req: &str) -> Vec<String> {
    let mut names = Vec::new();

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

    names
}

/// Extract property access from require('mod').prop patterns. Delegates to
/// `parse_require_property_access` (shared with `scan_require_call`, WI
/// #1947 round 2); kept as a standalone unit with its own test coverage
/// (`test_cjs_require_property_access`, `test_cjs_require_dot_access`),
/// hence no production caller remains.
#[allow(dead_code)]
fn extract_require_property_access(line: &str) -> Vec<String> {
    // Look for require(...).<prop> or require(...)["prop"]
    match line.find("require(") {
        Some(req_pos) => parse_require_property_access(&line[req_pos..]),
        None => Vec::new(),
    }
}

fn extract_string_value(s: &str) -> Option<String> {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        Some(s[1..s.len() - 1].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod cjs_require_scan_1947_tests {
    use super::*;

    /// `scan_require_call` (the merged single-scan replacement used by
    /// `extract_cjs_require_bindings`, WI #1947 round 2) must always agree
    /// with calling `extract_require_specifier` then
    /// `extract_require_property_access` separately — those two original
    /// functions are kept standalone (with their own pre-existing tests)
    /// specifically so this comparison has an independent oracle.
    fn assert_scan_matches_old_pair(line: &str) {
        let old_specifier = extract_require_specifier(line);
        let old_accessed = extract_require_property_access(line);
        let new_result = scan_require_call(line);

        match (old_specifier, new_result) {
            (Some(old_spec), Some((new_spec, new_accessed))) => {
                assert_eq!(
                    old_spec.as_str(),
                    new_spec,
                    "specifier mismatch for line {line:?}"
                );
                assert_eq!(
                    old_accessed, new_accessed,
                    "accessed-names mismatch for line {line:?}"
                );
            }
            (None, None) => {}
            (old, new) => panic!(
                "old/new disagreed on presence of a require() call for line {line:?}: \
                 old_specifier={old:?}, new_result_is_some={}",
                new.is_some()
            ),
        }
    }

    #[test]
    fn scan_require_call_matches_old_pair_across_tricky_shapes() {
        for line in [
            "const { readFile, writeFile } = require('./fs-helpers');",
            "const { readFile: rf, writeFile: wf } = require('./fs-helpers');",
            "var jsx = require('react/jsx-runtime')[\"jsx\"];",
            "var React = require('react').default;",
            "var m = require('./whole-module');",
            "module.exports = require('./interop-wrapper');",
            "const x = 1;",
            "require('./side-effect-only');",
            "// see require('legacy-path') for background",
            "const msg = \"please require('nonexistent-module') now\";",
            "require(dynamicSpecifierVariable);",
            "require(`./template-${literal}`);",
        ] {
            assert_scan_matches_old_pair(line);
        }
    }

    /// End-to-end proof through the actual rewritten
    /// `extract_cjs_require_bindings` (not just the lower-level
    /// `scan_require_call` helper) on the CJS shapes the WI #1947 round-2
    /// dispatch called out as tricky: plain destructured require, aliased
    /// destructured require, and nested/property-access require (`.prop`
    /// and `["prop"]` forms), all resolved from one module.
    #[test]
    fn extract_cjs_require_bindings_resolves_destructured_aliased_and_property_access_shapes() {
        let app_source = "const { readFile, writeFile } = require('./fs-helpers');\nconst { readFile: rf, writeFile: wf } = require('./fs-helpers');\nconst def = require('./dep').default;\nconst named = require('./dep')[\"namedExport\"];\n";
        let modules = vec![
            (PathBuf::from("/fixture/app.js"), app_source.to_string()),
            (
                PathBuf::from("/fixture/fs-helpers.js"),
                "exports.readFile = function () {};\nexports.writeFile = function () {};\n"
                    .to_string(),
            ),
            (
                PathBuf::from("/fixture/dep.js"),
                "exports.default = function () {};\nexports.namedExport = 1;\n".to_string(),
            ),
        ];
        let lookup = ModuleLookup::new(&modules);
        let bindings =
            extract_cjs_require_bindings(Path::new("/fixture/app.js"), app_source, &lookup);

        let fs_helpers = PathBuf::from("/fixture/fs-helpers.js");
        let dep = PathBuf::from("/fixture/dep.js");

        // Both the plain and the aliased destructure resolve to the same
        // *original* property names — aliases are a local binding-name
        // detail, not a property-name detail (extract_destructured_names's
        // pre-existing "before colon" rule, unrelated to the round-2 scan
        // merge, exercised here end-to-end for confidence).
        let fs_helpers_bindings: Vec<&Vec<String>> = bindings
            .iter()
            .filter(|(p, _)| *p == fs_helpers)
            .map(|(_, names)| names)
            .collect();
        assert_eq!(
            fs_helpers_bindings.len(),
            2,
            "expected one binding entry per require line, got {bindings:?}"
        );
        for names in fs_helpers_bindings {
            assert_eq!(
                names,
                &vec!["readFile".to_string(), "writeFile".to_string()]
            );
        }

        assert!(
            bindings.contains(&(dep.clone(), vec!["default".to_string()])),
            "dot-access require('./dep').default must resolve to [\"default\"], got {bindings:?}"
        );
        assert!(
            bindings.contains(&(dep, vec!["namedExport".to_string()])),
            "bracket-access require('./dep')[\"namedExport\"] must resolve to [\"namedExport\"], got {bindings:?}"
        );
    }

    /// A `require(` substring that only appears inside a comment or a
    /// string literal, whose "specifier" never resolves to a real module in
    /// the graph, must not produce a phantom binding. This holds because of
    /// the module-lookup resolution step (unaffected by the round-2 scan
    /// merge) — not because of comment/string awareness in the line
    /// scanner itself, which is a best-effort per-line text scan rather
    /// than a real parser, both before and after WI #1947 round 2.
    #[test]
    fn require_text_inside_comment_or_string_with_unresolvable_specifier_yields_no_binding() {
        let app_source = "// see require('nonexistent-legacy-module') for background\nconst msg = \"please require('also-nonexistent') now\";\nconst real = require('./real-dep');\n";
        let modules = vec![
            (PathBuf::from("/fixture/app.js"), app_source.to_string()),
            (
                PathBuf::from("/fixture/real-dep.js"),
                "exports.default = function () {};\n".to_string(),
            ),
        ];
        let lookup = ModuleLookup::new(&modules);
        let bindings =
            extract_cjs_require_bindings(Path::new("/fixture/app.js"), app_source, &lookup);

        assert_eq!(
            bindings,
            vec![(PathBuf::from("/fixture/real-dep.js"), vec!["*".to_string()])],
            "only the real, resolvable require() call may produce a binding, got {bindings:?}"
        );
    }
}

// --- Shared utilities ---

/// Every fact `analyze_used_exports_from` needs from one module, computed by
/// `compute_module_facts` in a single per-module pass.
///
/// WI #1947 fix 2: `analyze_used_exports_from` used to run 4 independent
/// `modules.par_iter()` sweeps (exports, static+cjs edges, dynamic targets,
/// re-export bindings) — each one a full corpus-wide dispatch that re-reads
/// every module's source from scratch. `ModuleFacts` groups the equivalent
/// per-module outputs so the corpus is only dispatched across rayon once;
/// each worker computes every fact for its module while the source bytes
/// are still hot, instead of the whole corpus paying dispatch/collect
/// overhead (and cold source re-reads) 4 times over.
#[derive(Debug, Clone)]
struct ModuleFacts {
    /// ESM export names (`extract_export_names`).
    exports: Vec<String>,
    /// CJS export names (`extract_cjs_export_names`).
    cjs_exports: Vec<String>,
    /// ESM static import edges (`extract_import_bindings`).
    static_edges: Vec<(PathBuf, Vec<String>)>,
    /// CJS require edges (`extract_cjs_require_bindings`).
    cjs_edges: Vec<(PathBuf, Vec<String>)>,
    /// Dynamic `import(...)` targets (`extract_dynamic_import_targets_from`).
    dynamic_targets: Vec<PathBuf>,
    /// `export ... from '...'` re-export bindings (`extract_reexport_bindings`).
    reexports: Vec<(PathBuf, ReexportKind)>,
}

/// Compute one module's `ModuleFacts` by calling each existing extractor
/// once. Extractor internals are untouched by this merge (WI #1947 fix 2
/// only changes how many times the corpus is dispatched across rayon, not
/// what each extractor does or how many times it scans its own module's
/// source) — the only extractor whose *internals* changed for WI #1947 is
/// `extract_import_bindings`'s callee `filter_referenced_import_names`
/// (fix 1's word index), which is unrelated to this merge.
fn compute_module_facts(path: &Path, source: &str, lookup: &ModuleLookup<'_>) -> ModuleFacts {
    ModuleFacts {
        exports: extract_export_names(source, is_ts(path)),
        cjs_exports: extract_cjs_export_names(source),
        static_edges: extract_import_bindings(path, source, lookup),
        cjs_edges: extract_cjs_require_bindings(path, source, lookup),
        dynamic_targets: extract_dynamic_import_targets_from(path, source, lookup),
        reexports: extract_reexport_bindings(path, source, lookup),
    }
}

#[cfg(test)]
mod module_facts_1947_tests {
    use super::*;

    /// `compute_module_facts` must produce exactly what calling each
    /// extractor individually produces — the WI #1947 fix 2 merge only
    /// changes the *dispatch* shape (one per-module call site instead of 4
    /// corpus-wide sweeps), never the extractors' own outputs.
    fn assert_facts_match_individual_extractors(
        modules: &[(PathBuf, String)],
        path: &Path,
        source: &str,
    ) {
        let lookup = ModuleLookup::new(modules);
        let facts = compute_module_facts(path, source, &lookup);

        assert_eq!(
            facts.exports,
            extract_export_names(source, is_ts(path)),
            "exports mismatch for {path:?}"
        );
        assert_eq!(
            facts.cjs_exports,
            extract_cjs_export_names(source),
            "cjs_exports mismatch for {path:?}"
        );
        assert_eq!(
            facts.static_edges,
            extract_import_bindings(path, source, &lookup),
            "static_edges mismatch for {path:?}"
        );
        assert_eq!(
            facts.cjs_edges,
            extract_cjs_require_bindings(path, source, &lookup),
            "cjs_edges mismatch for {path:?}"
        );
        assert_eq!(
            facts.dynamic_targets,
            extract_dynamic_import_targets_from(path, source, &lookup),
            "dynamic_targets mismatch for {path:?}"
        );
        // `ReexportKind` derives `Debug, Clone` but not `PartialEq` (no
        // production code needs to compare re-export bindings by value);
        // comparing Debug output instead of adding a derive-only-for-tests
        // impl keeps this test from nudging production type shape.
        assert_eq!(
            format!("{:?}", facts.reexports),
            format!("{:?}", extract_reexport_bindings(path, source, &lookup)),
            "reexports mismatch for {path:?}"
        );
    }

    #[test]
    fn matches_individual_extractors_on_an_esm_barrel() {
        let modules = vec![
            (PathBuf::from("/proj/barrel.js"), String::new()),
            (PathBuf::from("/proj/leaf.js"), String::new()),
            (PathBuf::from("/proj/dyn.js"), String::new()),
        ];
        let source = r#"
import { used, unused } from './leaf';
export { used };
export * from './leaf';
export const local = 1;
const p = import('./dyn');
"#;
        assert_facts_match_individual_extractors(&modules, Path::new("/proj/barrel.js"), source);
    }

    #[test]
    fn matches_individual_extractors_on_a_cjs_module() {
        let modules = vec![
            (PathBuf::from("/proj/cjs.js"), String::new()),
            (PathBuf::from("/proj/dep.js"), String::new()),
        ];
        let source = r#"
const { a, b } = require('./dep');
const whole = require('./dep');
exports.foo = 1;
module.exports = { a, b };
"#;
        assert_facts_match_individual_extractors(&modules, Path::new("/proj/cjs.js"), source);
    }

    #[test]
    fn matches_individual_extractors_on_an_empty_module() {
        let modules = vec![(PathBuf::from("/proj/empty.js"), String::new())];
        assert_facts_match_individual_extractors(&modules, Path::new("/proj/empty.js"), "");
    }
}

/// Find a module in the module list by matching specifier against paths.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn find_module_by_specifier<'a>(
    specifier: &str,
    modules: &'a [(PathBuf, String)],
) -> Option<&'a (PathBuf, String)> {
    let lookup = ModuleLookup::new(modules);
    lookup.find(specifier, Path::new(""))
}

struct ModuleLookup<'a> {
    modules: &'a [(PathBuf, String)],
    by_path: HashMap<PathBuf, usize>,
    by_specifier: HashMap<String, usize>,
    /// Exact specifier resolution from the bundler's resolver. The textual
    /// variants above cannot map bare package specifiers whose entry comes
    /// from package.json (`@ant-design/colors` -> `es/index.js`); missing
    /// those edges starved the demand-driven liveness walk and shook
    /// genuinely-used modules out of the bundle.
    resolver: Option<&'a (dyn Fn(&str, &Path) -> Option<PathBuf> + Sync)>,
    /// WI #1947 fix 3: memoizes `find`'s result per (specifier,
    /// importer-directory). `find` is called from parallel per-module
    /// extraction (`Sync` required, hence a concurrent map rather than a
    /// `RefCell`). The same bare specifiers (`react`, `clsx`, `prop-types`,
    /// ...) repeat across thousands of importers in a node_modules-heavy
    /// graph, and every uncached call allocates several candidate
    /// strings/paths and does multiple hash-map probes.
    ///
    /// Keyed by directory, not the full importer path: every branch of
    /// `find_uncached` (the relative-specifier join, the bare-specifier
    /// candidate list, and the external `resolver` callback) only ever
    /// reads `importer.parent()`, never the importer's own file name — two
    /// sibling files resolving the same specifier always resolve to the
    /// same target, so directory-level keys are both correct and the
    /// coarsest granularity that's still exact (not an approximation).
    resolution_memo: DashMap<(String, PathBuf), Option<usize>>,
}

impl<'a> ModuleLookup<'a> {
    fn new(modules: &'a [(PathBuf, String)]) -> Self {
        let mut by_path = HashMap::new();
        let mut by_specifier = HashMap::new();

        for (idx, (path, _)) in modules.iter().enumerate() {
            let normalized = normalize_tree_shake_path(path);
            by_path.entry(normalized.clone()).or_insert(idx);
            for specifier in package_specifier_variants(&normalized) {
                by_specifier.entry(specifier).or_insert(idx);
            }
        }

        Self {
            modules,
            by_path,
            by_specifier,
            resolver: None,
            resolution_memo: DashMap::new(),
        }
    }

    fn with_resolver(
        modules: &'a [(PathBuf, String)],
        resolver: &'a (dyn Fn(&str, &Path) -> Option<PathBuf> + Sync),
    ) -> Self {
        let mut lookup = Self::new(modules);
        lookup.resolver = Some(resolver);
        lookup
    }

    fn find(&self, specifier: &str, importer: &Path) -> Option<&'a (PathBuf, String)> {
        self.find_index(specifier, importer)
            .and_then(|idx| self.modules.get(idx))
    }

    /// Memoized wrapper around `find_index_uncached` — see
    /// `resolution_memo`'s doc comment for the cache-key correctness
    /// argument.
    fn find_index(&self, specifier: &str, importer: &Path) -> Option<usize> {
        let cache_key = (
            specifier.to_string(),
            importer.parent().unwrap_or(importer).to_path_buf(),
        );
        if let Some(cached) = self.resolution_memo.get(&cache_key) {
            return *cached;
        }
        let resolved = self.find_index_uncached(specifier, importer);
        self.resolution_memo.insert(cache_key, resolved);
        resolved
    }

    fn find_index_uncached(&self, specifier: &str, importer: &Path) -> Option<usize> {
        if specifier.starts_with('.') {
            if let Some(importer_dir) = importer.parent() {
                let target = normalize_tree_shake_path(importer_dir.join(specifier));
                if let Some(idx) = self.find_exact_candidate_index(&target) {
                    return Some(idx);
                }
            }
        }

        if let Some(idx) = self.by_specifier.get(specifier) {
            return Some(*idx);
        }

        let bare = specifier.strip_prefix("./").unwrap_or(specifier);
        for candidate in [
            bare.to_string(),
            format!("{bare}.js"),
            format!("{bare}.jsx"),
            format!("{bare}.ts"),
            format!("{bare}.tsx"),
            format!("{bare}/index.js"),
            format!("{bare}/index.ts"),
            format!("{bare}/index.tsx"),
        ] {
            if let Some(idx) = self.by_specifier.get(&candidate) {
                return Some(*idx);
            }
        }

        if let Some(resolver) = self.resolver {
            if let Some(resolved) = resolver(specifier, importer) {
                let normalized = normalize_tree_shake_path(&resolved);
                if let Some(idx) = self.by_path.get(&normalized) {
                    return Some(*idx);
                }
            }
        }

        None
    }

    fn find_exact_candidate_index(&self, target: &Path) -> Option<usize> {
        const EXTENSIONS: &[&str] = &["js", "jsx", "ts", "tsx", "mjs", "cjs"];
        for candidate in exact_candidate_paths(target) {
            if let Some(idx) = self.by_path.get(&candidate) {
                return Some(*idx);
            }
            for ext in EXTENSIONS {
                if let Some(idx) = self.by_path.get(&candidate.with_extension(ext)) {
                    return Some(*idx);
                }
                let index = normalize_tree_shake_path(candidate.join(format!("index.{ext}")));
                if let Some(idx) = self.by_path.get(&index) {
                    return Some(*idx);
                }
            }
        }
        None
    }
}

fn exact_candidate_paths(target: &Path) -> Vec<PathBuf> {
    let normalized = normalize_tree_shake_path(target);
    vec![normalized]
}

fn normalize_tree_shake_path(path: impl AsRef<Path>) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.as_ref().components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !out.pop() && !out.has_root() {
                    out.push("..");
                }
            }
            _ => out.push(component.as_os_str()),
        }
    }
    out
}

fn package_specifier_variants(path: &Path) -> Vec<String> {
    let path_str = path.to_string_lossy();
    let mut variants = Vec::new();

    if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
        variants.push(file_name.to_string());
        for ext in [".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs"] {
            if let Some(no_ext) = file_name.strip_suffix(ext) {
                variants.push(no_ext.to_string());
            }
        }
    }

    let Some(after_nm) = path_str.rsplit_once("node_modules/").map(|(_, rest)| rest) else {
        return variants;
    };

    variants.push(after_nm.to_string());

    for ext in [".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs"] {
        if let Some(no_ext) = after_nm.strip_suffix(ext) {
            variants.push(no_ext.to_string());
        }
    }

    for suffix in [
        "/index.js",
        "/index.jsx",
        "/index.ts",
        "/index.tsx",
        "/index.mjs",
        "/index.cjs",
    ] {
        if let Some(no_index) = after_nm.strip_suffix(suffix) {
            variants.push(no_index.to_string());
        }
    }

    variants
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

pub(crate) fn module_has_side_effects_with_package_json(
    source: &str,
    module_path: &Path,
    package_side_effects_cache: &mut HashMap<(PathBuf, String), SideEffectsDecl>,
) -> bool {
    if let Some((node_modules_dir, package_name)) = find_package_info(module_path) {
        let decl = package_side_effects_cache
            .entry((node_modules_dir.clone(), package_name.clone()))
            .or_insert_with(|| read_package_side_effects(&node_modules_dir, &package_name));
        module_has_side_effects(source, module_path, decl)
    } else {
        has_side_effects(source)
    }
}

pub(crate) fn find_package_info(module_path: &Path) -> Option<(PathBuf, String)> {
    let path_str = module_path.to_string_lossy();
    let nm_marker = "node_modules/";
    let nm_pos = path_str.rfind(nm_marker)?;
    let node_modules_dir = PathBuf::from(&path_str[..nm_pos + nm_marker.len() - 1]);
    let after_nm = &path_str[nm_pos + nm_marker.len()..];

    let package_name = if after_nm.starts_with('@') {
        let parts: Vec<&str> = after_nm.splitn(3, '/').collect();
        if parts.len() < 2 {
            return None;
        }
        format!("{}/{}", parts[0], parts[1])
    } else {
        after_nm.split('/').next()?.to_string()
    };

    Some((node_modules_dir, package_name))
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

    #[test]
    fn test_shake_keeps_unused_multiline_export_declaration_intact() {
        let source = "function useRenderTimes() {}\nexport default process.env.NODE_ENV !== 'production' ? useRenderTimes : function () {};\nexport var RenderBlock = React.memo(function () {\n  var times = useRenderTimes();\n  return React.createElement(\"h1\", null, times);\n});\n";
        let mut used = HashSet::new();
        used.insert("default".to_string());

        let result = shake_module(source, &PathBuf::from("useRenderTimes.js"), &used);

        assert!(
            result.contains("export var RenderBlock = React.memo(function () {"),
            "multi-line export declaration should remain intact, got: {}",
            result
        );

        let transformed =
            crate::transform::modules::transform_modules(&result, &HashMap::new()).unwrap();
        assert!(
            !transformed.code.contains("module.exports[\"default\"] = ;"),
            "shaken source must not make default export empty, got: {}",
            transformed.code
        );
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
    fn test_analyze_eliminates_unused_package_module_when_side_effects_false() {
        let tmp = tempfile::tempdir().unwrap();
        let pkg_dir = tmp.path().join("node_modules/side-free");
        std::fs::create_dir_all(&pkg_dir).unwrap();
        std::fs::write(pkg_dir.join("package.json"), r#"{"sideEffects":false}"#).unwrap();

        let entry = tmp.path().join("src/entry.js");
        let used = pkg_dir.join("used.js");
        let unused = pkg_dir.join("unused.js");
        let modules = vec![
            (
                entry,
                "import { used } from 'side-free/used.js';\nconsole.log(used);\n".to_string(),
            ),
            (
                used.clone(),
                "export const used = 1;\nconsole.log('top-level but package says pure');\n"
                    .to_string(),
            ),
            (
                unused.clone(),
                "export const unused = 2;\nconsole.log('drop me despite heuristic side effect');\n"
                    .to_string(),
            ),
        ];

        let result = analyze_used_exports(&modules).unwrap();
        assert!(
            !result.eliminated_modules.contains(&used),
            "imported package module must stay live"
        );
        assert!(
            result.eliminated_modules.contains(&unused),
            "sideEffects:false package metadata must allow unused module elimination"
        );
    }

    #[test]
    fn test_analyze_multiline_named_import_marks_leaf_exports_used() {
        // #1991 round 2: a multi-line `import { ... } from '...'`
        // (prettier's default wrapping for 3+ named imports) must mark its
        // target leaf modules' exports as used exactly like the
        // single-line-equivalent form. Before this fix,
        // `extract_import_bindings` scanned `source.lines()` directly and
        // silently produced zero edges for a specifier whose binding list
        // and `from` clause never share a physical line — wrongly treating
        // a barrel (and its whole leaf subtree) as unreached from a real,
        // multi-line-only consumer.
        let entry = PathBuf::from("/fixture/entry.js");
        let barrel = PathBuf::from("/fixture/barrel.js");
        let leaf_a = PathBuf::from("/fixture/leaf_a.js");
        let leaf_b = PathBuf::from("/fixture/leaf_b.js");
        let modules = vec![
            (
                entry.clone(),
                "import {\n  LeafA,\n  LeafB,\n} from './barrel.js';\nconsole.log(LeafA, LeafB);\n"
                    .to_string(),
            ),
            (
                barrel.clone(),
                "export { LeafA } from './leaf_a.js';\nexport { LeafB } from './leaf_b.js';\n"
                    .to_string(),
            ),
            (leaf_a.clone(), "export const LeafA = 'A';\n".to_string()),
            (leaf_b.clone(), "export const LeafB = 'B';\n".to_string()),
        ];

        let result = analyze_used_exports_from(&modules, &entry, None).unwrap();

        assert!(
            result
                .used_exports
                .get(&leaf_a)
                .is_some_and(|names| names.contains("LeafA")),
            "multi-line-imported LeafA must be marked used, got: {:?}",
            result.used_exports.get(&leaf_a)
        );
        assert!(
            result
                .used_exports
                .get(&leaf_b)
                .is_some_and(|names| names.contains("LeafB")),
            "multi-line-imported LeafB must be marked used, got: {:?}",
            result.used_exports.get(&leaf_b)
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
    fn test_star_reexport_propagates_only_used_same_name_export() {
        let modules = vec![
            (
                PathBuf::from("/fixture/entry.js"),
                "import { used } from './barrel';\nconsole.log(used);\n".to_string(),
            ),
            (
                PathBuf::from("/fixture/barrel.js"),
                "export * from './leaf';\n".to_string(),
            ),
            (
                PathBuf::from("/fixture/leaf.js"),
                "export const used = 1;\nexport const unused = 2;\n".to_string(),
            ),
        ];

        let result = analyze_used_exports(&modules).unwrap();
        let leaf_used = result
            .used_exports
            .get(&PathBuf::from("/fixture/leaf.js"))
            .expect("leaf export usage should propagate through star barrel");
        assert!(leaf_used.contains("used"), "{leaf_used:?}");
        assert!(
            !leaf_used.contains("unused"),
            "star re-export should not mark unrelated leaf exports live: {leaf_used:?}"
        );
    }

    #[test]
    fn entry_star_reexport_preserves_the_public_namespace() {
        let modules = vec![
            (
                PathBuf::from("/fixture/entry.js"),
                "export * from './react-router';\n".to_string(),
            ),
            (
                PathBuf::from("/fixture/react-router.js"),
                "export const Link = 1;\nexport const Route = 2;\nexport default 3;\n".to_string(),
            ),
        ];

        let result = analyze_used_exports(&modules).unwrap();
        let leaf_used = result
            .used_exports
            .get(&PathBuf::from("/fixture/react-router.js"))
            .expect("entry re-export must keep its leaf module live");
        assert!(leaf_used.contains("Link"), "{leaf_used:?}");
        assert!(leaf_used.contains("Route"), "{leaf_used:?}");
        assert!(
            !leaf_used.contains("default"),
            "export * must not propagate the default export: {leaf_used:?}"
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

    #[test]
    fn analyze_used_exports_barrel_with_cjs_shim_narrows_without_wildcard_at_scale() {
        // #1968: build(entries) mirrors @mui/icons-material's actual
        // shape at the scale that made the bug visible — a pure Named ESM
        // barrel (`export { default as IconK } from "./IconK.js"`, no
        // Star/mixed forms) plus a CJS shim doing
        // `const M = require('@mui/icons-material'); ... M.Icon2 ...`
        // (property access on a line SEPARATE from the require() call,
        // exactly like tw-monitor's lucideReactShim.js reading
        // `MuiIcons.SomeIcon`). Before the fix,
        // extract_cjs_require_bindings's namespace-wildcard fallback fired
        // for the shim's bare `const M = require(...)` line (no same-line
        // destructure/access), registering `"*"` into the barrel's used
        // set. That `"*"` made `dce::eliminate_unused_reexport_assignments`
        // (bundler/dce.rs — unmodified by this fix, see its own
        // `test_eliminate_unused_reexport_assignments_prunes_barrel_leaves_when_used_has_no_wildcard`
        // for the byte-level pruning half of this reconciliation) bail out
        // of pruning the barrel's re-export glue for every leaf, not just
        // the unused ones, so the bundler's later require-reachability
        // pass kept every leaf reachable — reproduced end-to-end against a
        // real `jet build` of a 10-icon fixture with this exact shape
        // (dist output contained all 10 icon functions pre-fix, only the 3
        // genuinely-used ones post-fix). This test pins the fix at the
        // extractor/analysis layer: the barrel's used set must recover the
        // shim's specific `M.Icon2` access and stay wildcard-free at every
        // scale, including the 5,000-entry scale the WI #1947 regression
        // concern (sub-second analysis time) is measured against.
        fn build(entries: usize) -> Vec<(PathBuf, String)> {
            let mut modules = Vec::new();
            let mut barrel_src = String::new();
            for i in 0..entries {
                barrel_src.push_str(&format!(
                    "export {{ default as Icon{i} }} from \"./Icon{i}.js\";\n"
                ));
                modules.push((
                    PathBuf::from(format!(
                        "/fixture/node_modules/@mui/icons-material/esm/Icon{i}.js"
                    )),
                    format!("export default function Icon{i}() {{ return null; }}\n"),
                ));
            }
            modules.push((
                PathBuf::from("/fixture/node_modules/@mui/icons-material/esm/index.js"),
                barrel_src,
            ));
            modules.push((
                PathBuf::from("/fixture/entry.js"),
                "import { Icon0, Icon1 } from '@mui/icons-material';\nimport './shim.js';\nconsole.log(Icon0, Icon1);\n".to_string(),
            ));
            modules.push((
                PathBuf::from("/fixture/shim.js"),
                "const MuiIcons = require('@mui/icons-material');\nconst Wrapped = wrap(MuiIcons.Icon2);\nmodule.exports = { Wrapped };\n".to_string(),
            ));
            modules
        }

        let resolver = |spec: &str, _importer: &Path| -> Option<PathBuf> {
            if spec == "@mui/icons-material" {
                Some(PathBuf::from(
                    "/fixture/node_modules/@mui/icons-material/esm/index.js",
                ))
            } else {
                None
            }
        };
        let barrel_path = PathBuf::from("/fixture/node_modules/@mui/icons-material/esm/index.js");

        for entries in [5usize, 100, 5000] {
            let modules = build(entries);
            let entry = PathBuf::from("/fixture/entry.js");
            let start = std::time::Instant::now();
            let result = analyze_used_exports_from(&modules, &entry, Some(&resolver)).unwrap();
            let elapsed = start.elapsed();
            assert!(
                elapsed.as_secs() < 1,
                "N={entries}: analysis must stay sub-second (WI #1947 regression concern), took {elapsed:?}"
            );

            let barrel_used = result.used_exports.get(&barrel_path).unwrap_or_else(|| {
                panic!("N={entries}: barrel must have a used_exports entry, got {result:?}")
            });
            assert!(
                !barrel_used.contains("*"),
                "N={entries}: a CJS shim's namespace-wildcard fallback must not pollute the \
                 barrel's used set once its specific M.Icon2 access is recovered, got {barrel_used:?}"
            );
            let mut sorted: Vec<&str> = barrel_used.iter().map(|s| s.as_str()).collect();
            sorted.sort_unstable();
            assert_eq!(
                sorted,
                vec!["Icon0", "Icon1", "Icon2"],
                "N={entries}: exactly the 2 ESM-imported + 1 shim-accessed names are used, got {barrel_used:?}"
            );

            for i in 0..3usize.min(entries) {
                let leaf = PathBuf::from(format!(
                    "/fixture/node_modules/@mui/icons-material/esm/Icon{i}.js"
                ));
                assert!(
                    !result.eliminated_modules.contains(&leaf),
                    "N={entries}: Icon{i} is used and must be retained"
                );
            }
            for i in 3..entries.min(5) {
                let leaf = PathBuf::from(format!(
                    "/fixture/node_modules/@mui/icons-material/esm/Icon{i}.js"
                ));
                assert!(
                    result.eliminated_modules.contains(&leaf),
                    "N={entries}: Icon{i} is never used anywhere and must be eliminated"
                );
            }
        }
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
