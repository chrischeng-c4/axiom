// HANDWRITE-BEGIN gap="missing-generator:logic:8c2e58b8" tracker="standardize-gap-projects-jet-src-stories-build-rs" reason="Static exporter: build_stories_static(root, out_dir) -> discover StoryIndex, clean out_dir, write manager index.html (reuse manager::render_manager_html), and per story write preview/{id}.html (render_preview_html) + transform and emit each imported module to out_dir with relative URLs; copy referenced assets. Output is servable by any static host with no jet process."
//! Static export of the `jet stories` workbench (B4 / #190).
//!
//! [`build_stories_static`] turns a project's discovered stories into a static,
//! hostable site that needs **no jet process at serve time**: any plain file
//! server — or a `file://` open — can render it. The flow mirrors the merged TD
//! contract:
//!
//! 1. **discover** stories under `root` (reusing [`super::discover`]),
//! 2. **clean** `out_dir` so a rebuild is idempotent (no stale files survive),
//! 3. **render the manager** shell to `out_dir/index.html` via
//!    [`manager::render_manager_html_with_mode`] in [`UrlMode::Static`] so the
//!    iframe src + sidebar links are **relative** (`preview/{id}.html`),
//! 4. for **each story**: render `out_dir/preview/{id}.html`
//!    ([`manager::render_preview_html_with_mode`], static mode — no HMR client),
//!    then **transform + emit** the story module and every local relative module
//!    it transitively imports into `out_dir/modules/...js`, **rewriting** their
//!    import URLs to the relative emitted paths,
//! 5. **bare imports** (`import x from "clsx"`) that resolve to a real file in
//!    the project's `node_modules` are resolved via the shared [`super::deps`]
//!    helper, emitted under `out_dir/deps/<key>.js`, and the importer's
//!    specifier is rewritten to a **relative** URL into that `deps/` tree —
//!    recursively for the dep's own bare + relative imports. Bare specifiers
//!    that do NOT resolve on disk (e.g. `react` with no local install) stay
//!    as-authored and load from the esm.sh importmap baked into every preview.
//!
//! ## Layout (all relative, server-less)
//! ```text
//! out_dir/
//!   index.html                         # manager shell (UrlMode::Static)
//!   preview/<story_id>.html            # one isolated preview per story
//!   modules/<rel-path-with-.js>        # transformed JS for every project module
//!   deps/<node_modules-rel-with-.js>   # transformed JS for every resolved dep
//! ```
//! A preview at `preview/<id>.html` imports its module as
//! `../modules/<rel>.js`; inside a module, a relative import `./Button` is
//! rewritten to `./Button.js` (extension normalized to the emitted `.js`), and a
//! resolvable bare import `clsx` is rewritten to e.g.
//! `../../../deps/clsx/dist/clsx.js`, so the emitted tree is internally
//! consistent and resolves on any static host.
//!
//! ## Deferred (#197)
//! Advanced conditional-`exports` edge cases and CommonJS interop are out of
//! scope — see [`super::deps`]. A dep authored as browser ESM is the
//! expectation; bare specifiers with no local install ride the importmap.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::manager::{self, UrlMode};
use super::{discover, StoryEntry, StoryIndex};

/// Summary of a static stories build (returned for the CLI + tests to inspect).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildStaticResult {
    /// The directory everything was written under.
    pub out_dir: PathBuf,
    /// Number of stories that got a `preview/<id>.html`.
    pub story_count: usize,
    /// Relative paths (under `out_dir`) of every emitted file, sorted. Includes
    /// `index.html`, each `preview/<id>.html`, and each `modules/...js`.
    pub emitted: Vec<PathBuf>,
    /// Per-story / per-module problems that did not abort the build (an
    /// unreadable or unresolvable module, a transform error) — surfaced so a
    /// broken module is visible rather than silently missing.
    pub diagnostics: Vec<String>,
}

/// Build a static, server-less export of the stories workbench under `out_dir`.
///
/// Cleans `out_dir` first (idempotent rebuilds), then writes the manager shell,
/// one isolated preview per discovered story, and the transformed JS for every
/// local module the previews import. All embedded URLs are relative so the
/// output is hostable by any static file server or `file://`.
pub fn build_stories_static(root: &Path, out_dir: &Path) -> Result<BuildStaticResult> {
    let mut index = discover(root);

    // When `out_dir` is nested under `root` (the common `root/dist-stories`
    // case), a previous build's emitted `*.stories.js` modules live inside it
    // and the discover walk re-picks them up as stories. Drop anything under
    // `out_dir` so a rebuild is stable and never compounds on its own output.
    if let Ok(out_abs) = out_dir.canonicalize().or_else(|_| Ok::<_, std::io::Error>(out_dir.to_path_buf())) {
        index
            .stories
            .retain(|s| !s.file.starts_with(&out_abs) && !s.file.starts_with(out_dir));
        index
            .metas
            .retain(|m| !m.file.starts_with(&out_abs) && !m.file.starts_with(out_dir));
    }

    clean_out_dir(out_dir)?;
    std::fs::create_dir_all(out_dir)
        .with_context(|| format!("creating out_dir {}", out_dir.display()))?;

    let mut result = BuildStaticResult {
        out_dir: out_dir.to_path_buf(),
        ..Default::default()
    };
    // Carry forward discovery diagnostics (parse errors etc.) so a broken story
    // file is visible in the build output, not silently dropped.
    result.diagnostics.extend(index.diagnostics.clone());

    // 1. Manager shell → index.html (relative links).
    let manager_html = manager_relative_html(&index);
    write_emitted(out_dir, Path::new("index.html"), &manager_html, &mut result)?;

    // 2. Per story: a relative preview + its transitively-imported modules.
    //    A module is emitted at most once across all stories (modules dedupe by
    //    their root-relative URL).
    let mut emitted_modules: BTreeSet<String> = BTreeSet::new();
    for story in &index.stories {
        let module_url = story_module_root_url(root, &story.file);
        let preview_html = preview_relative_html(story, &module_url);
        let preview_rel = PathBuf::from("preview").join(format!("{}.html", story.id));
        write_emitted(out_dir, &preview_rel, &preview_html, &mut result)?;
        result.story_count += 1;

        // Emit the story module + everything it transitively imports (local
        // relative modules only; bare specifiers stay for the importmap/browser).
        emit_module_graph(root, &module_url, out_dir, &mut emitted_modules, &mut result);
    }

    result.emitted.sort();
    result.emitted.dedup();
    Ok(result)
}

/// Render the manager shell HTML with **relative** URLs (B4), seeding the
/// initially-selected story's controls exactly like the dev manager does.
pub fn manager_relative_html(index: &StoryIndex) -> String {
    // B3 controls panel is dev-server state we cannot recompute without the
    // component-source resolution the server owns; the static manager renders
    // the panel placeholder (no live controls — the preview still honors the
    // story's authored args). Keeping it empty avoids shipping a non-functional
    // postMessage bridge to a frame that has no server behind it.
    manager::render_manager_html_with_mode(index, None, &[], UrlMode::Static)
}

/// Render one story's isolated preview HTML with a **relative** module URL (B4).
///
/// `module_root_url` is the story module's root-relative URL (`/src/x.tsx`); the
/// preview lives at `preview/<id>.html`, so it imports the emitted module as
/// `../modules/src/x.js`.
pub fn preview_relative_html(story: &StoryEntry, module_root_url: &str) -> String {
    let import_url = preview_module_import_url(module_root_url);
    manager::render_preview_html_with_mode(story, &import_url, UrlMode::Static)
}

/// The `../modules/...js` URL a preview document uses to import a module given
/// the module's root-relative URL (`/src/components/Button.stories.tsx`).
fn preview_module_import_url(module_root_url: &str) -> String {
    let rel = module_root_url.trim_start_matches('/');
    format!("../modules/{}", to_js_path(rel))
}

/// A unit of work in the transitive emit walk: a project module (under
/// `modules/`) or a resolved `node_modules` dependency (under `deps/`, #197).
#[derive(Clone)]
enum EmitItem {
    /// A project source module, identified by its root-relative URL
    /// (`/src/components/Button.tsx`). Emitted at `modules/<rel-with-.js>`.
    Module(String),
    /// A resolved dep, identified by its on-disk file under `node_modules`.
    /// Emitted at `deps/<node_modules-relative-with-.js>`.
    Dep(PathBuf),
}

impl EmitItem {
    /// The out_dir-relative POSIX path this item is emitted to (`.js`-normalized).
    /// This is the stable identity used for dedup and for computing the relative
    /// specifiers between any two emitted files.
    fn emitted_path(&self, root: &Path) -> String {
        match self {
            EmitItem::Module(url) => {
                format!("modules/{}", to_js_path(url.trim_start_matches('/')))
            }
            EmitItem::Dep(file) => {
                let _ = root;
                format!("deps/{}", to_js_path(&super::deps::dep_key(file)))
            }
        }
    }
}

/// Emit the module at `module_url` and, transitively, every local relative
/// module **and** resolvable `node_modules` dep it imports — transforming each to
/// JS and rewriting its imports to the emitted siblings/deps. Best-effort per
/// item: a failure is recorded as a diagnostic and the rest of the graph emits.
fn emit_module_graph(
    root: &Path,
    module_url: &str,
    out_dir: &Path,
    emitted: &mut BTreeSet<String>,
    result: &mut BuildStaticResult,
) {
    let mut queue: Vec<EmitItem> = vec![EmitItem::Module(module_url.to_string())];
    while let Some(item) = queue.pop() {
        // Dedup by the out_dir-relative emitted path (so a dep imported from two
        // modules, or a module imported from two stories, emits once).
        let key = item.emitted_path(root);
        if !emitted.insert(key) {
            continue;
        }
        match emit_item(root, &item, out_dir) {
            Ok(emit) => {
                result.emitted.push(emit.rel_path);
                queue.extend(emit.imports);
            }
            Err(err) => {
                result.diagnostics.push(format!("module {err}"));
            }
        }
    }
}

/// What [`emit_item`] produced for one emitted file.
struct EmittedItem {
    /// Relative path under `out_dir` the JS was written to.
    rel_path: PathBuf,
    /// Further items to walk (resolved relative modules + resolved deps).
    imports: Vec<EmitItem>,
}

/// Transform one emit item to browser JS, rewrite its relative + resolvable bare
/// imports to the emitted siblings/deps, and write it under `out_dir`.
fn emit_item(root: &Path, item: &EmitItem, out_dir: &Path) -> Result<EmittedItem> {
    // The on-disk source file + the emitted out_dir-relative path of this item.
    let (source_file, self_emitted) = match item {
        EmitItem::Module(url) => {
            let rel = url.trim_start_matches('/');
            let file = resolve_url_to_file(root, rel)
                .with_context(|| format!("cannot resolve {url} under {}", root.display()))?;
            // Resolution may have added an extension / index file; recompute the
            // canonical emitted path from the file we actually read.
            let canonical = EmitItem::Module(file_to_root_url(root, &file));
            let emitted = canonical.emitted_path(root);
            (file, emitted)
        }
        EmitItem::Dep(file) => (file.clone(), item.emitted_path(root)),
    };

    let source = std::fs::read_to_string(&source_file)
        .with_context(|| format!("reading {}", source_file.display()))?;
    let code = transform_source(&source, &source_file)
        .with_context(|| format!("transforming {}", source_file.display()))?;

    let (rewritten, imports) = rewrite_imports(&code, &source_file, &self_emitted, root);

    let out_rel = PathBuf::from(&self_emitted);
    let out_path = out_dir.join(&out_rel);
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating dir {}", parent.display()))?;
    }
    std::fs::write(&out_path, rewritten)
        .with_context(|| format!("writing {}", out_path.display()))?;

    Ok(EmittedItem {
        rel_path: out_rel,
        imports,
    })
}

/// Rewrite the import specifiers in transformed JS so they resolve to the
/// emitted siblings (relative imports) and emitted deps (resolvable bare imports
/// under `node_modules`, #197), and collect each one's [`EmitItem`] for the walk.
///
/// `importer_file` is the importer's on-disk source file (the resolution base).
/// `importer_emitted` is its out_dir-relative emitted path (`modules/...js` or
/// `deps/...js`) — both surfaces live under `out_dir`, so the rewritten specifier
/// is a plain relative path between two emitted positions. A relative specifier
/// that does not resolve, and a bare specifier with no local install, are left
/// as-authored (the importmap covers the latter).
fn rewrite_imports(
    code: &str,
    importer_file: &Path,
    importer_emitted: &str,
    root: &Path,
) -> (String, Vec<EmitItem>) {
    let mut imports: Vec<EmitItem> = Vec::new();
    let mut rewrites: BTreeMap<String, String> = BTreeMap::new();

    for spec in super::deps::extract_all_import_specifiers(code) {
        let (item, target_emitted) = if spec.starts_with('.') {
            // Relative import → resolve against the importer's on-disk dir.
            let Some(target_file) = resolve_relative_file(importer_file, &spec) else {
                continue; // unresolvable — leave the original specifier in place
            };
            // A relative import inside a dep file resolves to another file in the
            // SAME node_modules package → keep it a dep; a relative import inside
            // a project module resolves to another project module.
            let item = if path_has_node_modules(&target_file) {
                EmitItem::Dep(target_file)
            } else {
                EmitItem::Module(file_to_root_url(root, &target_file))
            };
            let emitted = item.emitted_path(root);
            (item, emitted)
        } else {
            // Bare import → resolve against node_modules via the shared helper.
            let Some(dep_file) = super::deps::resolve_bare_specifier(root, importer_file, &spec)
            else {
                continue; // not installed locally — leave for the importmap
            };
            let item = EmitItem::Dep(dep_file);
            let emitted = item.emitted_path(root);
            (item, emitted)
        };

        let new_spec = relative_emitted_specifier(importer_emitted, &target_emitted);
        rewrites.insert(spec.clone(), new_spec);
        imports.push(item);
    }

    // Apply the rewrites textually. Only quoted forms are rewritten so we never
    // touch an identifier that merely shares the specifier's spelling.
    let mut out = code.to_string();
    for (old, new) in &rewrites {
        if old == new {
            continue;
        }
        out = out
            .replace(&format!("\"{old}\""), &format!("\"{new}\""))
            .replace(&format!("'{old}'"), &format!("'{new}'"));
    }
    (out, imports)
}

/// Resolve a relative specifier (`./Button`, `../lib/x.tsx`) against the
/// importer's on-disk file, probing the same extensions + `index.*` the dev
/// server does. Returns the resolved on-disk file (which may be a project file
/// or a sibling inside the same `node_modules` package), lexically normalized so
/// no `.`/`..` segments leak into the emitted path / dedup key.
fn resolve_relative_file(importer_file: &Path, spec: &str) -> Option<PathBuf> {
    let base_dir = importer_file.parent().unwrap_or(Path::new("."));
    let joined = lexically_normalize(&base_dir.join(spec));
    if joined.is_file() {
        return Some(joined);
    }
    const EXTS: &[&str] = &["tsx", "ts", "jsx", "js", "mjs", "cjs", "json"];
    for ext in EXTS {
        let candidate = joined.with_extension(ext);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    for ext in EXTS {
        let candidate = joined.join(format!("index.{ext}"));
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Lexically collapse `.` and `..` segments in `path` (no filesystem access), so
/// a path like `/proj/src/components/./Button` becomes
/// `/proj/src/components/Button`. A leading `..` that would pop the root is kept
/// (it never happens for a path joined from an absolute importer dir).
fn lexically_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        use std::path::Component;
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// True when `path` contains a `node_modules` path segment.
fn path_has_node_modules(path: &Path) -> bool {
    path.iter().any(|c| c == "node_modules")
}

/// Express `target_emitted` (out_dir-relative, `.js`-normalized) as an import
/// specifier relative to `importer_emitted` (same form). Both live under
/// `out_dir`, so this is a plain relative path between two emitted positions —
/// it crosses the `modules/` ↔ `deps/` boundary as needed.
fn relative_emitted_specifier(importer_emitted: &str, target_emitted: &str) -> String {
    let importer_segs: Vec<&str> = importer_emitted.split('/').filter(|s| !s.is_empty()).collect();
    let target_segs: Vec<&str> = target_emitted.split('/').filter(|s| !s.is_empty()).collect();

    // Drop the importer's own filename — relativity is from its directory.
    let importer_dir = &importer_segs[..importer_segs.len().saturating_sub(1)];

    // Longest common directory prefix.
    let mut common = 0;
    let max = importer_dir.len().min(target_segs.len().saturating_sub(1));
    while common < max && importer_dir[common] == target_segs[common] {
        common += 1;
    }

    let ups = importer_dir.len() - common;
    let mut parts: Vec<String> = Vec::new();
    for _ in 0..ups {
        parts.push("..".to_string());
    }
    for seg in &target_segs[common..] {
        parts.push((*seg).to_string());
    }
    let mut spec = parts.join("/");
    // A sibling (or descendant) import must keep a leading `./` so it stays a
    // relative specifier rather than becoming a bare one.
    if !spec.starts_with('.') {
        spec = format!("./{spec}");
    }
    spec
}

/// Transform a source file to browser JS using the same per-extension
/// entrypoints the dev server's module route uses.
fn transform_source(source: &str, file: &Path) -> Result<String> {
    let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
    let options = crate::transform::TransformOptions::default();
    let result = match ext {
        "tsx" => crate::transform::transform_tsx::transform_tsx(source, &options),
        "ts" => crate::transform::typescript::transform_typescript(source, &options),
        "jsx" => crate::transform::jsx::transform_jsx(source, &options),
        _ => Ok(crate::transform::TransformResult {
            code: source.to_string(),
            source_map: None,
        }),
    };
    result.map(|r| r.code).map_err(|e| anyhow::anyhow!("{e}"))
}

/// Resolve a root-relative path string to an existing file under `root`, probing
/// the common TS/JS extensions and an `index.*` barrel (mirrors the dev server's
/// `resolve_module_file`).
fn resolve_url_to_file(root: &Path, rel: &str) -> Option<PathBuf> {
    let joined = root.join(rel);
    if joined.is_file() {
        return Some(joined);
    }
    const EXTS: &[&str] = &["tsx", "ts", "jsx", "js"];
    for ext in EXTS {
        let candidate = joined.with_extension(ext);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    for ext in EXTS {
        let candidate = joined.join(format!("index.{ext}"));
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// The root-relative URL of a file on disk (`/src/components/Button.tsx`).
fn file_to_root_url(root: &Path, file: &Path) -> String {
    let rel = file.strip_prefix(root).unwrap_or(file);
    let mut url = String::from("/");
    url.push_str(rel.to_string_lossy().replace('\\', "/").trim_start_matches('/'));
    url
}

/// The story module's root-relative URL (same form the dev server serves).
fn story_module_root_url(root: &Path, file: &Path) -> String {
    file_to_root_url(root, file)
}

/// Normalize a path's file extension to `.js` (the emitted module extension).
/// `src/Button.tsx` → `src/Button.js`; `src/util.js` stays `src/util.js`; a
/// dep's `dist/clsx.mjs` → `dist/clsx.js`; an extensionless path gets a `.js`
/// appended.
fn to_js_path(path: &str) -> String {
    const SRC_EXTS: &[&str] = &[".tsx", ".ts", ".jsx", ".js", ".mjs", ".cjs"];
    for ext in SRC_EXTS {
        if let Some(stem) = path.strip_suffix(ext) {
            return format!("{stem}.js");
        }
    }
    format!("{path}.js")
}

/// Write `contents` to `out_dir/rel`, creating parents, and record the relative
/// path in `result.emitted`.
fn write_emitted(
    out_dir: &Path,
    rel: &Path,
    contents: &str,
    result: &mut BuildStaticResult,
) -> Result<()> {
    let path = out_dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    std::fs::write(&path, contents).with_context(|| format!("writing {}", path.display()))?;
    result.emitted.push(rel.to_path_buf());
    Ok(())
}

/// Remove `out_dir` so a rebuild starts from a clean slate (idempotency). A
/// missing directory is fine. Refuses obviously dangerous targets (filesystem
/// root, empty path) so a misconfigured `--out-dir` can't wipe the world.
fn clean_out_dir(out_dir: &Path) -> Result<()> {
    if out_dir.as_os_str().is_empty() {
        anyhow::bail!("refusing to clean an empty out_dir path");
    }
    if out_dir.parent().is_none() {
        anyhow::bail!("refusing to clean filesystem root {}", out_dir.display());
    }
    match std::fs::remove_dir_all(out_dir) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => {
            Err(err).with_context(|| format!("cleaning out_dir {}", out_dir.display()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn js_path_normalizes_extension() {
        assert_eq!(to_js_path("src/Button.tsx"), "src/Button.js");
        assert_eq!(to_js_path("src/util.ts"), "src/util.js");
        assert_eq!(to_js_path("src/util.js"), "src/util.js");
        assert_eq!(to_js_path("src/Button"), "src/Button.js");
    }

    #[test]
    fn preview_imports_module_relative_to_preview_dir() {
        // A preview lives at preview/<id>.html, so it reaches up into modules/.
        assert_eq!(
            preview_module_import_url("/src/components/Button.stories.tsx"),
            "../modules/src/components/Button.stories.js"
        );
    }

    #[test]
    fn relative_specifier_is_sibling_with_js_ext() {
        // Story imports a sibling component — emitted side by side under modules/.
        let spec = relative_emitted_specifier(
            "modules/src/components/Button.stories.js",
            "modules/src/components/Button.js",
        );
        assert_eq!(spec, "./Button.js");
    }

    #[test]
    fn relative_specifier_walks_up_directories() {
        let spec = relative_emitted_specifier(
            "modules/src/components/Button.stories.js",
            "modules/src/lib/util.js",
        );
        assert_eq!(spec, "../lib/util.js");
    }

    #[test]
    fn bare_dep_specifier_crosses_into_deps_tree() {
        // A project module importing a resolved dep rewrites across the
        // modules/ ↔ deps/ boundary (#197).
        let spec = relative_emitted_specifier(
            "modules/src/components/Button.stories.js",
            "deps/clsx/dist/clsx.js",
        );
        assert_eq!(spec, "../../../deps/clsx/dist/clsx.js");
    }

    #[test]
    fn clean_refuses_dangerous_targets() {
        assert!(clean_out_dir(Path::new("")).is_err());
        assert!(clean_out_dir(Path::new("/")).is_err());
    }
}
// HANDWRITE-END
