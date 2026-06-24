// HANDWRITE-BEGIN gap="missing-generator:logic:3833b5e5" tracker="pending-tracker" reason="New library-build orchestrator implementing the contract flow: resolve entries + externals (dependencies + peerDependencies) from package.json, build/tree-shake per entry, emit ESM (bare `import` for externals) and optional CJS (`require()` for externals), write one output per (entry x format) under out_dir, return LibBuildResult."
//! Library-build orchestrator for `jet build --lib`.
//!
//! Unlike the app bundle path (`Bundler::bundle`), which inlines every
//! dependency and wraps the result in a runtime/IIFE scope, a library build
//! produces a *publishable* artifact: npm dependencies and `peerDependencies`
//! are kept as real top-level `import ... from "pkg"` (ESM) / `require("pkg")`
//! (CJS) statements, and internal relative modules are inlined.
//!
//! The flow per the contract:
//!   1. read `package.json`,
//!   2. resolve entries (`exports`, falling back to `module`/`main`) + the
//!      external package set (`dependencies` + `peerDependencies`),
//!   3. for each entry, inline internal relative modules while hoisting
//!      external imports verbatim,
//!   4. emit one file per (entry × format) under `out_dir`,
//!   5. return a [`LibBuildResult`].
//!
//! @issue #170

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::types::OutputFormat;
use crate::resolver::package::{external_package_names, library_entries};

/// Options driving a single library build.
#[derive(Debug, Clone)]
pub struct LibBuildOptions {
    /// Project root (directory containing `package.json`).
    pub project_root: PathBuf,
    /// Output directory (absolute, or resolved against `project_root`).
    pub out_dir: PathBuf,
    /// Output formats to emit. ESM is always supported; CJS is best-effort.
    pub formats: Vec<OutputFormat>,
    /// Export conditions used to pick entry sources from `exports`.
    pub conditions: Vec<String>,
    /// Extra package names to force-externalize beyond `package.json` deps.
    pub extra_externals: HashSet<String>,
    /// Preserve internal module structure instead of bundling each entry.
    /// Deferred — see [`build_library`].
    pub preserve_modules: bool,
    /// Emit a `<entry>.d.ts` type declaration file next to each entry's JS
    /// output (isolatedDeclarations-style). Defaults to `true` for library
    /// builds — see [`LibBuildOptions::default`]. When off, no `.d.ts` is
    /// written and [`EntryOutput::dts`] stays `None`.
    /// @issue #171
    pub declaration: bool,
}

/// Library builds default to emitting declarations on (`declaration: true`).
/// App-mode builds never go through this path.
/// @issue #171
impl Default for LibBuildOptions {
    fn default() -> Self {
        Self {
            project_root: PathBuf::new(),
            out_dir: PathBuf::from("dist"),
            formats: vec![OutputFormat::Esm],
            conditions: vec!["import".to_string(), "default".to_string()],
            extra_externals: HashSet::new(),
            preserve_modules: false,
            declaration: true,
        }
    }
}

/// One emitted output file.
#[derive(Debug, Clone)]
pub struct EntryOutput {
    /// Public export subpath the entry was published under (`.`, `./client`).
    pub subpath: String,
    /// Output format of this file.
    pub format: OutputFormat,
    /// Absolute path the file was written to.
    pub path: PathBuf,
    /// Emitted code (also written to `path`).
    pub code: String,
    /// Absolute path to the `<entry>.d.ts` emitted for this entry, when
    /// declaration emission is on. The same path is recorded once per format
    /// of an entry. `None` when declarations are disabled or emission failed.
    /// @issue #171
    pub dts: Option<PathBuf>,
}

/// Result of a library build: one [`EntryOutput`] per (entry × format).
#[derive(Debug, Clone)]
pub struct LibBuildResult {
    /// All emitted outputs.
    pub entries: Vec<EntryOutput>,
    /// Emitted `.d.ts` declaration files, keyed by the entry's public
    /// subpath (`.`, `./client`). One per library entry when `declaration`
    /// is on. Empty when declaration emission is disabled.
    /// @issue #171
    pub types: Vec<TypesOutput>,
}

/// A `.d.ts` type-declaration file emitted for one library entry.
/// @issue #171
#[derive(Debug, Clone)]
pub struct TypesOutput {
    /// Public export subpath the declarations belong to (`.`, `./client`).
    pub subpath: String,
    /// Absolute path the `.d.ts` was written to.
    pub path: PathBuf,
}

/// Build a publishable library from `package.json`.
///
/// `preserve_modules` is accepted but not yet implemented — when set, the
/// build fails fast with a typed "unsupported yet" error rather than silently
/// bundling. See TODO(#170 follow-up) below.
pub fn build_library(options: LibBuildOptions) -> Result<LibBuildResult> {
    // TODO(#170 follow-up): preserve-modules emission (one output file per
    // source module) is not implemented. Fail loudly instead of silently
    // producing a single-file bundle that contradicts the requested mode.
    if options.preserve_modules {
        anyhow::bail!(
            "jet build --lib: preserve_modules is not supported yet (TODO #170 follow-up); \
             omit preserve_modules to produce a bundled library"
        );
    }

    let pkg_path = options.project_root.join("package.json");
    if !pkg_path.exists() {
        anyhow::bail!(
            "jet build --lib: no package.json found at {}",
            pkg_path.display()
        );
    }

    let conditions: Vec<&str> = options.conditions.iter().map(String::as_str).collect();
    let entries = library_entries(&pkg_path, &conditions)
        .with_context(|| format!("resolving library entries from {}", pkg_path.display()))?;

    // External set = dependencies + peerDependencies + caller-supplied extras.
    let mut externals = external_package_names(&pkg_path)
        .with_context(|| format!("collecting externals from {}", pkg_path.display()))?;
    externals.extend(options.extra_externals.iter().cloned());

    std::fs::create_dir_all(&options.out_dir)
        .with_context(|| format!("creating out_dir {}", options.out_dir.display()))?;

    let mut outputs = Vec::new();
    let mut types_outputs = Vec::new();

    for entry in &entries {
        let entry_path = resolve_entry_path(&options.project_root, &entry.source)
            .with_context(|| format!("resolving entry source {}", entry.source))?;

        // Inline internal relative modules; hoist external imports verbatim.
        let esm = bundle_library_entry(&entry_path, &externals)?;

        // Emit `<entry>.d.ts` once per entry (not per format) when declaration
        // emission is on. The isolatedDeclarations emitter reads the entry
        // source directly so type aliases / interfaces survive the JS inline.
        let dts_path = if options.declaration {
            let entry_source = std::fs::read_to_string(&entry_path)
                .with_context(|| format!("reading {} for .d.ts", entry_path.display()))?;
            let dts = super::dts::emit_declarations(&entry_source)
                .with_context(|| format!("emitting .d.ts for entry {}", entry.subpath))?;
            let dts_name = dts_file_name(&entry.subpath);
            let dts_out = options.out_dir.join(&dts_name);
            if let Some(parent) = dts_out.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("creating {}", parent.display()))?;
            }
            std::fs::write(&dts_out, &dts)
                .with_context(|| format!("writing {}", dts_out.display()))?;
            types_outputs.push(TypesOutput {
                subpath: entry.subpath.clone(),
                path: dts_out.clone(),
            });
            Some(dts_out)
        } else {
            None
        };

        for format in &options.formats {
            let code = match format {
                OutputFormat::Esm => esm.clone(),
                OutputFormat::Cjs => esm_to_cjs(&esm),
                OutputFormat::Iife => {
                    // TODO(#170 follow-up): IIFE library output. Libraries are
                    // ESM/CJS; an IIFE/global build is a separate concern.
                    anyhow::bail!(
                        "jet build --lib: IIFE output format is not supported yet \
                         (TODO #170 follow-up); use esm or cjs"
                    );
                }
            };

            let file_name = output_file_name(&entry.subpath, format);
            let out_path = options.out_dir.join(&file_name);
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("creating {}", parent.display()))?;
            }
            std::fs::write(&out_path, &code)
                .with_context(|| format!("writing {}", out_path.display()))?;

            outputs.push(EntryOutput {
                subpath: entry.subpath.clone(),
                format: format.clone(),
                path: out_path,
                code,
                dts: dts_path.clone(),
            });
        }
    }

    Ok(LibBuildResult {
        entries: outputs,
        types: types_outputs,
    })
}

/// Map a public export subpath to its `.d.ts` file name.
///
///   `.`        → `index.d.ts`
///   `./client` → `client.d.ts`
fn dts_file_name(subpath: &str) -> String {
    let stem = if subpath == "." {
        "index".to_string()
    } else {
        subpath
            .trim_start_matches("./")
            .trim_end_matches(".js")
            .trim_end_matches(".mjs")
            .trim_end_matches(".ts")
            .replace('/', "_")
    };
    format!("{stem}.d.ts")
}

/// Map a public export subpath + format to an output file name.
///
///   `.`        + Esm → `index.js`     + Cjs → `index.cjs`
///   `./client` + Esm → `client.js`    + Cjs → `client.cjs`
fn output_file_name(subpath: &str, format: &OutputFormat) -> String {
    let stem = if subpath == "." {
        "index".to_string()
    } else {
        subpath
            .trim_start_matches("./")
            .trim_end_matches(".js")
            .trim_end_matches(".mjs")
            .trim_end_matches(".ts")
            .replace('/', "_")
    };
    let ext = match format {
        OutputFormat::Cjs => "cjs",
        _ => "js",
    };
    format!("{stem}.{ext}")
}

/// Resolve a `package.json`-relative entry source to an absolute file path.
///
/// Tries the literal path first, then common TS/JS extensions, then an
/// `index.*` directory entry — mirroring how published `exports` may point at
/// either built `.js` or source `.ts`.
fn resolve_entry_path(root: &Path, source: &str) -> Result<PathBuf> {
    let rel = source.trim_start_matches("./");
    let base = root.join(rel);

    if base.is_file() {
        return Ok(base);
    }

    let exts = ["ts", "tsx", "js", "jsx", "mjs", "cjs"];
    for ext in exts {
        let candidate = base.with_extension(ext);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    for ext in exts {
        let candidate = base.join(format!("index.{ext}"));
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    anyhow::bail!("entry source not found: {}", base.display())
}

/// Inline internal relative modules reachable from `entry`, hoisting every
/// external import (verbatim) to the top of the emitted ESM.
///
/// Returns ESM source: hoisted external imports first, then the inlined entry
/// body (with internal relative imports/re-exports spliced in).
fn bundle_library_entry(entry: &Path, externals: &HashSet<String>) -> Result<String> {
    let mut external_imports: Vec<String> = Vec::new();
    let mut seen_external: HashSet<String> = HashSet::new();
    let mut inlined_files: HashSet<PathBuf> = HashSet::new();

    let body = inline_module(
        entry,
        externals,
        &mut external_imports,
        &mut seen_external,
        &mut inlined_files,
    )?;

    let mut out = String::new();
    for stmt in &external_imports {
        out.push_str(stmt);
        if !stmt.ends_with('\n') {
            out.push('\n');
        }
    }
    if !external_imports.is_empty() {
        out.push('\n');
    }
    out.push_str(&body);
    Ok(out)
}

/// Recursively inline one module's body.
///
/// External imports are pushed (deduplicated by verbatim text) to
/// `external_imports`; internal relative imports/re-exports are replaced by
/// the inlined body of their target module. Every other statement is kept
/// verbatim.
fn inline_module(
    path: &Path,
    externals: &HashSet<String>,
    external_imports: &mut Vec<String>,
    seen_external: &mut HashSet<String>,
    inlined_files: &mut HashSet<PathBuf>,
) -> Result<String> {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if !inlined_files.insert(canonical.clone()) {
        // Already inlined (diamond / cycle) — emit nothing the second time.
        return Ok(String::new());
    }

    let source =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;

    let mut parser = tree_sitter::Parser::new();
    let ext = path.extension().and_then(|e| e.to_str());
    let is_ts = matches!(ext, Some("ts") | Some("tsx"));
    let language: tree_sitter::Language = if is_ts {
        tree_sitter_typescript::LANGUAGE_TSX.into()
    } else {
        tree_sitter_javascript::LANGUAGE.into()
    };
    parser
        .set_language(&language)
        .context("setting tree-sitter language")?;
    let tree = parser
        .parse(&source, None)
        .context("parsing module source")?;
    let root = tree.root_node();

    // Walk top-level statements in order, splicing internal modules inline.
    let mut out = String::new();
    let mut cursor = root.walk();
    let mut last_end = 0usize;

    for child in root.children(&mut cursor) {
        let kind = child.kind();
        if kind != "import_statement" && kind != "export_statement" {
            continue;
        }
        let Some(spec) = statement_specifier(&source, &child) else {
            continue;
        };

        let stmt_start = child.start_byte();
        let stmt_end = child.end_byte();
        // Emit any interstitial text (comments / other statements) verbatim.
        out.push_str(&source[last_end..stmt_start]);
        last_end = stmt_end;

        if is_external_specifier(&spec, externals) {
            // Keep the external import/re-export verbatim, hoisted.
            let stmt_text = source[stmt_start..stmt_end].to_string();
            if seen_external.insert(stmt_text.clone()) {
                external_imports.push(stmt_text);
            }
            // Re-exports (`export ... from "pkg"`) must stay where they are so
            // the binding is re-exported; a plain side-effect/default import is
            // fully satisfied by the hoisted statement above.
            if kind == "export_statement" {
                out.push_str(&source[stmt_start..stmt_end]);
            }
        } else {
            // Internal relative module — inline its body in place.
            if let Some(target) = resolve_relative(path, &spec) {
                let inlined = inline_module(
                    &target,
                    externals,
                    external_imports,
                    seen_external,
                    inlined_files,
                )?;
                out.push_str(&inlined);
            } else {
                // Unresolved relative import: keep verbatim rather than drop it.
                out.push_str(&source[stmt_start..stmt_end]);
            }
        }
    }

    // Trailing text after the last handled statement.
    out.push_str(&source[last_end..]);
    Ok(out)
}

/// Extract the string specifier of an `import`/`export ... from` statement,
/// or `None` when the statement has no source (e.g. `export const x = 1`).
fn statement_specifier(source: &str, node: &tree_sitter::Node) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "string" {
            let text = &source[child.byte_range()];
            return Some(strip_quotes(text));
        }
    }
    None
}

fn strip_quotes(s: &str) -> String {
    s.trim()
        .trim_start_matches(['"', '\'', '`'])
        .trim_end_matches(['"', '\'', '`'])
        .to_string()
}

/// A specifier is external when it is bare (not `.`/`/`-relative) and either
/// listed in `externals` or otherwise not a local file reference.
fn is_external_specifier(spec: &str, externals: &HashSet<String>) -> bool {
    if spec.starts_with('.') || spec.starts_with('/') {
        return false;
    }
    if externals.contains(spec) {
        return true;
    }
    // Sub-path imports (`pkg/sub`) inherit their package's externality.
    if let Some(root) = spec.split('/').next() {
        if externals.contains(root) {
            return true;
        }
    }
    // Any remaining bare specifier is treated as an external package: a
    // library build must never inline node_modules code.
    true
}

/// Resolve a relative specifier against the importing file.
fn resolve_relative(from: &Path, spec: &str) -> Option<PathBuf> {
    let base = from.parent()?.join(spec.trim_start_matches("./"));
    if base.is_file() {
        return Some(base);
    }
    let exts = ["ts", "tsx", "js", "jsx", "mjs", "cjs"];
    for ext in exts {
        let candidate = base.with_extension(ext);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    for ext in exts {
        let candidate = base.join(format!("index.{ext}"));
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Best-effort ESM → CJS rewrite for library output.
///
/// Handles the import/export shapes a typical published entry uses:
///   * `import X from "pkg"`            → `const X = require("pkg")`
///   * `import { a, b } from "pkg"`     → `const { a, b } = require("pkg")`
///   * `import * as X from "pkg"`       → `const X = require("pkg")`
///   * `import "pkg"`                   → `require("pkg")`
///   * `export const|let|var|function|class …` → `<decl>; exports.<name> = …`
///   * `export default <expr>`         → `module.exports = <expr>`
///   * `export { a, b }`               → `exports.a = a; exports.b = b`
///
/// TODO(#170 follow-up): `export … from "pkg"` re-export forms and renamed
/// `export { a as b }` aliases fall through unchanged — they are rare in
/// library entry points and a full CST rewrite is deferred.
fn esm_to_cjs(esm: &str) -> String {
    let mut out = String::new();
    for line in esm.lines() {
        let trimmed = line.trim();
        if let Some(rewritten) = rewrite_cjs_line(trimmed) {
            out.push_str(&rewritten);
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

fn rewrite_cjs_line(line: &str) -> Option<String> {
    // import * as X from "pkg";
    if let Some(rest) = line.strip_prefix("import * as ") {
        let (name, spec) = split_import_from(rest)?;
        return Some(format!("const {name} = require(\"{spec}\");"));
    }
    // import { a, b } from "pkg";
    if let Some(rest) = line.strip_prefix("import {") {
        let (names, spec) = rest.split_once('}')?;
        let spec = import_spec(spec)?;
        return Some(format!("const {{{names}}} = require(\"{spec}\");"));
    }
    // import "pkg"; (side-effect) or import Default from "pkg";
    if let Some(rest) = line.strip_prefix("import ") {
        if rest.starts_with('"') || rest.starts_with('\'') {
            let spec = strip_quotes(rest.trim_end_matches(';'));
            return Some(format!("require(\"{spec}\");"));
        }
        let (name, spec) = split_import_from(rest)?;
        return Some(format!("const {name} = require(\"{spec}\");"));
    }
    // export default <expr>;
    if let Some(rest) = line.strip_prefix("export default ") {
        return Some(format!("module.exports = {}", rest));
    }
    // export const|let|var NAME = ...
    for kw in ["const", "let", "var"] {
        if let Some(rest) = line.strip_prefix(&format!("export {kw} ")) {
            let name = rest.split(['=', ' ', ':']).next()?.trim();
            return Some(format!("{kw} {rest}\nexports.{name} = {name};"));
        }
    }
    // export function NAME / export class NAME
    for kw in ["function", "class"] {
        if let Some(rest) = line.strip_prefix(&format!("export {kw} ")) {
            let name = rest.split(['(', ' ', '{', '<']).next().unwrap_or("").trim();
            if !name.is_empty() {
                return Some(format!("{kw} {rest}\nexports.{name} = {name};"));
            }
        }
    }
    // export { a, b };
    if let Some(rest) = line.strip_prefix("export {") {
        let names = rest.split('}').next()?;
        let mut buf = String::new();
        for raw in names.split(',') {
            let name = raw.trim();
            if name.is_empty() {
                continue;
            }
            // Skip `a as b` aliases for the deferred follow-up.
            if name.contains(" as ") {
                continue;
            }
            buf.push_str(&format!("exports.{name} = {name};\n"));
        }
        if !buf.is_empty() {
            return Some(buf.trim_end().to_string());
        }
    }
    None
}

/// Helper: `Name from "pkg";` → `(Name, pkg)`.
fn split_import_from(rest: &str) -> Option<(String, String)> {
    let (name, spec) = rest.split_once(" from ")?;
    let spec = import_spec(spec)?;
    Some((name.trim().to_string(), spec))
}

/// Helper: extract a quoted specifier from the tail of an import, e.g.
/// ` from "pkg";` or `"pkg";`.
fn import_spec(tail: &str) -> Option<String> {
    let tail = tail.trim().trim_start_matches("from").trim();
    let spec = strip_quotes(tail.trim_end_matches(';').trim());
    if spec.is_empty() {
        None
    } else {
        Some(spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_file_name_maps_subpath_and_format() {
        assert_eq!(output_file_name(".", &OutputFormat::Esm), "index.js");
        assert_eq!(output_file_name(".", &OutputFormat::Cjs), "index.cjs");
        assert_eq!(output_file_name("./client", &OutputFormat::Esm), "client.js");
        assert_eq!(output_file_name("./client", &OutputFormat::Cjs), "client.cjs");
    }

    #[test]
    fn dts_file_name_maps_subpath() {
        assert_eq!(dts_file_name("."), "index.d.ts");
        assert_eq!(dts_file_name("./client"), "client.d.ts");
        assert_eq!(dts_file_name("./sub/mod"), "sub_mod.d.ts");
    }

    #[test]
    fn lib_build_options_default_enables_declarations() {
        assert!(LibBuildOptions::default().declaration);
    }

    #[test]
    fn external_specifier_classification() {
        let mut ext = HashSet::new();
        ext.insert("react".to_string());
        assert!(is_external_specifier("react", &ext));
        assert!(is_external_specifier("react/jsx-runtime", &ext));
        assert!(is_external_specifier("lodash", &ext)); // bare → external
        assert!(!is_external_specifier("./util", &ext));
        assert!(!is_external_specifier("../util", &ext));
        assert!(!is_external_specifier("/abs", &ext));
    }

    #[test]
    fn cjs_rewrite_named_import() {
        let out = esm_to_cjs("import { useState } from \"react\";\n");
        assert!(
            out.contains("const { useState } = require(\"react\")"),
            "{out}"
        );
    }

    #[test]
    fn cjs_rewrite_default_export() {
        let out = esm_to_cjs("export default foo;\n");
        assert!(out.contains("module.exports = foo;"), "{out}");
    }
}
// HANDWRITE-END
