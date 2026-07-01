// <HANDWRITE gap="missing-generator:logic:3833b5e5" tracker="standardize-gap-projects-jet-src-bundler-lib-build-rs" reason="New library-build orchestrator implementing the contract flow: resolve entries + externals (dependencies + peerDependencies) from package.json, build/tree-shake per entry, emit ESM (bare `import` for externals) and optional CJS (`require()` for externals), write one output per (entry x format) under out_dir, return LibBuildResult.">
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
//! @issue #722
//! @issue #757

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::types::{OutputFormat, SourceMapOption};
use crate::resolver::package::{external_package_names, library_entries, LibraryEntry};

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
    /// When set, one output file is emitted per source module (mirroring the
    /// source tree under `out_dir`); internal relative imports are rewritten
    /// to the emitted siblings and external imports stay as bare specifiers.
    /// Supports ESM and CJS output. IIFE remains single-file only.
    pub preserve_modules: bool,
    /// Emit a `<entry>.d.ts` type declaration file next to each entry's JS
    /// output (isolatedDeclarations-style). Defaults to `true` for library
    /// builds — see [`LibBuildOptions::default`]. When off, no `.d.ts` is
    /// written and [`EntryOutput::dts`] stays `None`.
    /// @issue #171
    pub declaration: bool,
    /// Global variable name an IIFE library output assigns its namespace to,
    /// e.g. `MyLib` → `var MyLib = (function () { ... })();`. Only consulted
    /// for [`OutputFormat::Iife`] outputs. When `None`, a global name is
    /// derived from the `package.json` `name` (see [`derive_global_name`]).
    pub library_global_name: Option<String>,
    /// Explicit source entry points (from `[lib].entry` of jet.toml), relative
    /// to `project_root`, e.g. `["src/index.ts"]`. When non-empty these are the
    /// SOURCE files to build; the first is published under `.`, the rest under
    /// `./<file-stem>`. When empty, entries are discovered from package.json
    /// `exports`/`module`/`main`, falling back to the conventional
    /// `src/index.{tsx,ts,jsx,js}` when those point at not-yet-built output
    /// (e.g. `./dist/index.js`). @issue #170
    pub entry: Vec<String>,

    /// CSS cascade-merge sources (from `[lib].css_merge` of jet.toml): an
    /// ordered list of `style.css` files relative to `project_root`, e.g.
    /// dependent packages' `dist/style.css`. After the normal lib emit, each
    /// file is read in this DECLARED order and concatenated (in order) into
    /// `out_dir/style.css` — declared order IS cascade order (first listed lands
    /// first, later rules can override). When empty, no merge runs and the build
    /// is byte-identical to today. Replaces the bespoke `mergeDepStyles` plugin.
    pub css_merge: Vec<String>,

    /// Raw-asset directory copies (from `[lib].raw_copy` of jet.toml): each
    /// directory tree is copied verbatim into `out_dir` (at the directive's
    /// `to`, defaulting to the same relative path as `from`), preserving
    /// subpaths so consumers can deep-import `@pkg/assets/icons/x.svg`. When
    /// empty, no copy runs. Replaces the bespoke `copyRawAssets` plugin.
    pub raw_copy: Vec<RawCopyDir>,

    /// Source map policy for emitted JS library outputs.
    pub sourcemap: SourceMapOption,
}

/// One raw-asset directory copy: the source dir (relative to `project_root`)
/// and an optional destination (relative to `out_dir`, default = `from`).
#[derive(Debug, Clone)]
pub struct RawCopyDir {
    /// Source directory relative to `project_root`.
    pub from: String,
    /// Destination relative to `out_dir`. `None` → same relative path as `from`.
    pub to: Option<String>,
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
            library_global_name: None,
            entry: Vec::new(),
            css_merge: Vec::new(),
            raw_copy: Vec::new(),
            sourcemap: SourceMapOption::None,
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

    /// Post-emit asset side-effects: the merged `out_dir/style.css` (when
    /// `css_merge` was configured) plus every file copied by `raw_copy`. Empty
    /// when neither was configured, so default builds carry no extra files.
    pub assets: Vec<AssetOutput>,
}

/// A post-emit asset written by the lib build's CSS cascade-merge or
/// raw-asset copy step.
#[derive(Debug, Clone)]
pub struct AssetOutput {
    /// Absolute path the asset was written to.
    pub path: PathBuf,
    /// How the asset was produced.
    pub kind: AssetKind,
}

/// Provenance of an [`AssetOutput`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetKind {
    /// The `out_dir/style.css` produced by concatenating `css_merge` sources.
    MergedCss,
    /// A file copied verbatim by a `raw_copy` directive.
    RawAsset,
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
/// Three emission shapes are supported:
///   * bundled single-file ESM/CJS (default),
///   * `preserve_modules` — one output file per source module mirroring the
///     source tree (ESM and CJS),
///   * [`OutputFormat::Iife`] — the bundled entry wrapped as a global-var IIFE.
/// Resolve the SOURCE entries to build. Explicit `[lib].entry`
/// (`options.entry`) wins. Otherwise entries are discovered from package.json
/// `exports`/`module`/`main` — but those usually point at BUILD OUTPUT
/// (e.g. `./dist/index.js`), so when the discovered sources don't exist on
/// disk we fall back to the conventional `src/index.{tsx,ts,jsx,js}`.
/// @issue #170
fn resolve_lib_entries(
    options: &LibBuildOptions,
    pkg_path: &Path,
    conditions: &[&str],
) -> Result<Vec<LibraryEntry>> {
    if !options.entry.is_empty() {
        return Ok(options
            .entry
            .iter()
            .enumerate()
            .map(|(i, src)| LibraryEntry {
                subpath: if i == 0 {
                    ".".to_string()
                } else {
                    let stem = Path::new(src)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("entry");
                    format!("./{stem}")
                },
                source: src.clone(),
            })
            .collect());
    }

    let mut entries = library_entries(pkg_path, conditions)
        .with_context(|| format!("resolving library entries from {}", pkg_path.display()))?;

    let any_missing = entries
        .iter()
        .any(|e| resolve_entry_path(&options.project_root, &e.source).is_err());
    if entries.is_empty() || any_missing {
        if let Some(conv) = [
            "src/index.tsx",
            "src/index.ts",
            "src/index.jsx",
            "src/index.js",
        ]
        .iter()
        .find(|p| options.project_root.join(p).exists())
        {
            entries = vec![LibraryEntry {
                subpath: ".".to_string(),
                source: (*conv).to_string(),
            }];
        }
    }
    Ok(entries)
}

pub fn build_library(options: LibBuildOptions) -> Result<LibBuildResult> {
    let pkg_path = options.project_root.join("package.json");
    if !pkg_path.exists() {
        anyhow::bail!(
            "jet build --lib: no package.json found at {}",
            pkg_path.display()
        );
    }

    // Global name for IIFE output: explicit option wins, else derive from the
    // package name. Computed up front so it is available to the IIFE branch.
    let global_name = options
        .library_global_name
        .clone()
        .unwrap_or_else(|| derive_global_name(&read_package_name(&pkg_path)));

    let conditions: Vec<&str> = options.conditions.iter().map(String::as_str).collect();
    let entries = resolve_lib_entries(&options, &pkg_path, &conditions)?;

    // External set = dependencies + peerDependencies + caller-supplied extras.
    let mut externals = external_package_names(&pkg_path)
        .with_context(|| format!("collecting externals from {}", pkg_path.display()))?;
    externals.extend(options.extra_externals.iter().cloned());

    std::fs::create_dir_all(&options.out_dir)
        .with_context(|| format!("creating out_dir {}", options.out_dir.display()))?;

    // preserve_modules: emit one file per source module + an entry re-export,
    // mirroring the source tree under out_dir.
    if options.preserve_modules {
        return build_library_preserve_modules(&options, &entries, &externals);
    }

    let mut outputs = Vec::new();
    let mut types_outputs = Vec::new();

    for entry in &entries {
        let entry_path = resolve_entry_path(&options.project_root, &entry.source)
            .with_context(|| format!("resolving entry source {}", entry.source))?;
        ensure_library_source_path(&entry_path, &entry.source, "entry source")?;

        // Inline internal relative modules; hoist external imports verbatim.
        let esm = bundle_library_entry(&entry_path, &externals)?;

        // Emit `<entry>.d.ts` once per entry (not per format) when declaration
        // emission is on. Local barrel re-export targets also get sibling
        // declarations so preserved `export * from "./x"` statements do not
        // dangle in a published package.
        let dts_path = if options.declaration {
            let dts_out = emit_declaration_tree(&options, entry, &entry_path, &externals)
                .with_context(|| format!("emitting .d.ts for entry {}", entry.subpath))?;
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
                OutputFormat::Iife => wrap_iife(&esm, &entry_path, &global_name, &externals)?,
            };

            let file_name = output_file_name(&entry.subpath, format);
            let out_path = options.out_dir.join(&file_name);
            let code = apply_library_sourcemap(&options, &entry_path, &file_name, code)
                .with_context(|| format!("emitting source map for {}", out_path.display()))?;
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

    // Post-emit asset steps: CSS cascade-merge + raw-asset copy. No-ops (and
    // thus byte-identical to today) when neither is configured.
    let mut assets = run_post_emit_assets(&options)?;
    assets.extend(copy_wildcard_export_assets(
        &options,
        &pkg_path,
        &conditions,
    )?);

    Ok(LibBuildResult {
        entries: outputs,
        types: types_outputs,
        assets,
    })
}

/// Emit declarations for one public entry and every internal module reachable
/// through local `export ... from "./x"` barrel re-exports.
///
/// `LibBuildResult::types` still reports only public entry declarations. The
/// additional files are filesystem side effects needed by the preserved
/// re-export statements inside `index.d.ts`.
fn emit_declaration_tree(
    options: &LibBuildOptions,
    entry: &LibraryEntry,
    entry_path: &Path,
    externals: &HashSet<String>,
) -> Result<PathBuf> {
    let mut visited = HashSet::new();
    let mut modules = Vec::new();
    collect_reexport_declaration_modules(entry_path, externals, &mut visited, &mut modules)?;

    let source_root = common_source_root(&modules);
    let entry_canonical = entry_path
        .canonicalize()
        .unwrap_or_else(|_| entry_path.to_path_buf());
    let mut entry_dts = None;

    for module in modules {
        let source = std::fs::read_to_string(&module)
            .with_context(|| format!("reading {} for .d.ts", module.display()))?;
        let dts = super::dts::emit_declarations(&source)
            .with_context(|| format!("emitting .d.ts for {}", module.display()))?;
        let module_canonical = module.canonicalize().unwrap_or_else(|_| module.clone());
        let dts_out = if module_canonical == entry_canonical {
            options.out_dir.join(dts_file_name(&entry.subpath))
        } else {
            declaration_module_output_path(&options.out_dir, &source_root, &module)
        };
        if module_canonical == entry_canonical {
            entry_dts = Some(dts_out.clone());
        }
        if let Some(parent) = dts_out.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        std::fs::write(&dts_out, &dts).with_context(|| format!("writing {}", dts_out.display()))?;
    }

    entry_dts.ok_or_else(|| anyhow::anyhow!("entry declaration was not emitted"))
}

fn collect_reexport_declaration_modules(
    path: &Path,
    externals: &HashSet<String>,
    visited: &mut HashSet<PathBuf>,
    order: &mut Vec<PathBuf>,
) -> Result<()> {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if !visited.insert(canonical.clone()) {
        return Ok(());
    }
    order.push(canonical.clone());

    let source =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    for spec in reexport_specifiers(&source, path)? {
        if is_external_specifier(&spec, externals) {
            continue;
        }
        if let Some(target) = resolve_relative(path, &spec)? {
            collect_reexport_declaration_modules(&target, externals, visited, order)?;
        }
    }
    Ok(())
}

fn reexport_specifiers(source: &str, path: &Path) -> Result<Vec<String>> {
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
        .parse(source, None)
        .context("parsing module source")?;
    let root = tree.root_node();

    let mut specs = Vec::new();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() != "export_statement" {
            continue;
        }
        if let Some(spec) = statement_specifier(source, &child) {
            specs.push(spec);
        }
    }
    Ok(specs)
}

fn declaration_module_output_path(out_dir: &Path, source_root: &Path, module: &Path) -> PathBuf {
    let rel = module.strip_prefix(source_root).ok().and_then(|p| {
        if p.as_os_str().is_empty() {
            None
        } else {
            Some(p)
        }
    });
    match rel {
        Some(path) => out_dir.join(path).with_extension("d.ts"),
        None => out_dir
            .join(
                module
                    .file_name()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("module.ts")),
            )
            .with_extension("d.ts"),
    }
}

/// Run the post-emit asset side-effects for a library build:
///   1. CSS cascade-merge (`css_merge`) — concatenate the listed `style.css`
///      files, in DECLARED order, into `out_dir/style.css` (created or
///      extended); declared order is cascade order.
///   2. Raw-asset copy (`raw_copy`) — copy each `from` directory tree verbatim
///      into `out_dir` (at `to`, default = same relative path), preserving
///      subpaths.
///
/// Both are no-ops when their config is empty, so a default lib build emits no
/// extra files and is byte-identical to today. Missing sources are a clear
/// error (not a panic); empty/absent config is skipped silently.
fn run_post_emit_assets(options: &LibBuildOptions) -> Result<Vec<AssetOutput>> {
    let mut assets = Vec::new();
    if !options.css_merge.is_empty() {
        if let Some(asset) = merge_css(options)? {
            assets.push(asset);
        }
    }
    if !options.raw_copy.is_empty() {
        assets.extend(copy_raw_assets(options)?);
    }
    Ok(assets)
}

/// Concatenate every `css_merge` source into `out_dir/style.css`, in declared
/// (cascade) order. If `out_dir/style.css` already exists (e.g. emitted by an
/// earlier CSS pass), the merged dependent CSS is appended after it so the
/// meta-package's own rules keep their cascade position and the dependents'
/// declared order is preserved. Each source is separated by a newline so the
/// boundary between two files is never glued mid-rule.
fn merge_css(options: &LibBuildOptions) -> Result<Option<AssetOutput>> {
    let mut merged = String::new();

    // Preserve any style.css the normal emit already produced as the base of
    // the cascade, then append the declared dependents after it.
    let out_css = options.out_dir.join("style.css");
    if out_css.is_file() {
        let existing = std::fs::read_to_string(&out_css)
            .with_context(|| format!("reading existing {}", out_css.display()))?;
        merged.push_str(&existing);
        if !merged.is_empty() && !merged.ends_with('\n') {
            merged.push('\n');
        }
    }

    for rel in &options.css_merge {
        let src = options.project_root.join(rel);
        let css = std::fs::read_to_string(&src).with_context(|| {
            format!(
                "jet build --lib css_merge: reading CSS source {}",
                src.display()
            )
        })?;
        merged.push_str(&css);
        // Guard the boundary between concatenated files: a missing trailing
        // newline would otherwise glue the next file's first rule onto the
        // previous file's last one.
        if !css.ends_with('\n') {
            merged.push('\n');
        }
    }

    std::fs::create_dir_all(&options.out_dir)
        .with_context(|| format!("creating out_dir {}", options.out_dir.display()))?;
    std::fs::write(&out_css, &merged)
        .with_context(|| format!("writing merged {}", out_css.display()))?;

    Ok(Some(AssetOutput {
        path: out_css,
        kind: AssetKind::MergedCss,
    }))
}

/// Copy each `raw_copy` directory tree verbatim into `out_dir`, preserving
/// subpaths. A directive's `to` (default = `from`) is the destination relative
/// to `out_dir`. Files are copied byte-for-byte; intermediate directories are
/// created as needed. A missing source directory is a clear error.
fn copy_raw_assets(options: &LibBuildOptions) -> Result<Vec<AssetOutput>> {
    let mut copied = Vec::new();

    for dir in &options.raw_copy {
        let src_root = options.project_root.join(&dir.from);
        if !src_root.is_dir() {
            anyhow::bail!(
                "jet build --lib raw_copy: source directory not found: {}",
                src_root.display()
            );
        }
        // Destination root under out_dir: explicit `to`, else mirror `from`.
        let dest_rel = dir.to.clone().unwrap_or_else(|| dir.from.clone());
        let dest_root = options.out_dir.join(&dest_rel);

        for entry in walkdir::WalkDir::new(&src_root).follow_links(false) {
            let entry = entry.with_context(|| {
                format!("jet build --lib raw_copy: walking {}", src_root.display())
            })?;
            if !entry.file_type().is_file() {
                continue;
            }
            let rel = entry.path().strip_prefix(&src_root).with_context(|| {
                format!(
                    "computing relative path of {} under {}",
                    entry.path().display(),
                    src_root.display()
                )
            })?;
            let dest = dest_root.join(rel);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("creating {}", parent.display()))?;
            }
            std::fs::copy(entry.path(), &dest).with_context(|| {
                format!("copying {} → {}", entry.path().display(), dest.display())
            })?;
            copied.push(AssetOutput {
                path: dest,
                kind: AssetKind::RawAsset,
            });
        }
    }

    Ok(copied)
}

/// Copy non-code files that are exposed through package.json wildcard export
/// patterns such as `"./icons/*": "./dist/icons/*"`.
///
/// Library entry discovery intentionally skips wildcard exports because code
/// wildcard entries need a real graph expansion step. Raw assets are simpler:
/// when the public wildcard prefix has a matching `src/<prefix>` directory and
/// the target points under `out_dir`, copy every non-JS/TS file verbatim.
fn copy_wildcard_export_assets(
    options: &LibBuildOptions,
    pkg_path: &Path,
    conditions: &[&str],
) -> Result<Vec<AssetOutput>> {
    let package_json = std::fs::read_to_string(pkg_path)
        .with_context(|| format!("reading {}", pkg_path.display()))?;
    let package: serde_json::Value = serde_json::from_str(&package_json)
        .with_context(|| format!("parsing {}", pkg_path.display()))?;
    let Some(exports) = package.get("exports").and_then(|v| v.as_object()) else {
        return Ok(Vec::new());
    };

    let mut copied = Vec::new();
    for (public_pattern, value) in exports {
        if !public_pattern.contains('*') {
            continue;
        }
        let Some(target_pattern) = wildcard_export_target(value, conditions) else {
            continue;
        };
        let Some((source_dir, dest_dir)) =
            wildcard_asset_dirs(options, public_pattern, &target_pattern)?
        else {
            continue;
        };
        if !source_dir.is_dir() {
            continue;
        }

        for entry in walkdir::WalkDir::new(&source_dir).follow_links(false) {
            let entry = entry.with_context(|| {
                format!(
                    "jet build --lib wildcard export: walking {}",
                    source_dir.display()
                )
            })?;
            if !entry.file_type().is_file() || is_library_source_path(entry.path()) {
                continue;
            }
            let rel = entry.path().strip_prefix(&source_dir).with_context(|| {
                format!(
                    "computing relative path of {} under {}",
                    entry.path().display(),
                    source_dir.display()
                )
            })?;
            let dest = dest_dir.join(rel);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("creating {}", parent.display()))?;
            }
            std::fs::copy(entry.path(), &dest).with_context(|| {
                format!("copying {} → {}", entry.path().display(), dest.display())
            })?;
            copied.push(AssetOutput {
                path: dest,
                kind: AssetKind::RawAsset,
            });
        }
    }

    Ok(copied)
}

fn wildcard_export_target(value: &serde_json::Value, conditions: &[&str]) -> Option<String> {
    match value {
        serde_json::Value::String(path) => Some(path.clone()),
        serde_json::Value::Object(map) => {
            for (key, nested) in map {
                if conditions.contains(&key.as_str()) {
                    if let Some(path) = wildcard_export_target(nested, conditions) {
                        return Some(path);
                    }
                }
            }
            map.get("default")
                .and_then(|nested| wildcard_export_target(nested, conditions))
        }
        _ => None,
    }
}

fn wildcard_asset_dirs(
    options: &LibBuildOptions,
    public_pattern: &str,
    target_pattern: &str,
) -> Result<Option<(PathBuf, PathBuf)>> {
    let Some(public_prefix) = public_pattern.split('*').next() else {
        return Ok(None);
    };
    let Some(target_prefix) = target_pattern.split('*').next() else {
        return Ok(None);
    };
    let public_prefix = public_prefix.trim_start_matches("./").trim_matches('/');
    if public_prefix.is_empty() {
        return Ok(None);
    }

    let out_dir_name = options
        .out_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("dist");
    let target_prefix = target_prefix.trim_start_matches("./");
    let Some(dest_rel) = target_prefix.strip_prefix(&format!("{out_dir_name}/")) else {
        return Ok(None);
    };

    let source_dir = options.project_root.join("src").join(public_prefix);
    let dest_dir = options.out_dir.join(dest_rel.trim_matches('/'));
    Ok(Some((source_dir, dest_dir)))
}

fn apply_library_sourcemap(
    options: &LibBuildOptions,
    entry_path: &Path,
    file_name: &str,
    code: String,
) -> Result<String> {
    match options.sourcemap {
        SourceMapOption::None => Ok(code),
        SourceMapOption::External | SourceMapOption::Hidden | SourceMapOption::Inline => {
            let source = std::fs::read_to_string(entry_path)
                .with_context(|| format!("reading source map input {}", entry_path.display()))?;
            let source_name = entry_path
                .strip_prefix(&options.project_root)
                .unwrap_or(entry_path)
                .to_string_lossy()
                .replace('\\', "/");
            let map =
                super::sourcemap::generate_source_map(file_name, &[(source_name, source)], &code);
            match options.sourcemap {
                SourceMapOption::External => {
                    let map_filename = format!("{file_name}.map");
                    super::sourcemap::write_external_map(
                        &options.out_dir,
                        &map_filename,
                        &map.json,
                    )
                    .with_context(|| {
                        format!(
                            "writing source map {}",
                            options.out_dir.join(&map_filename).display()
                        )
                    })?;
                    Ok(super::sourcemap::append_source_map_url(
                        &code,
                        &map_filename,
                    ))
                }
                SourceMapOption::Hidden => {
                    let map_filename = format!("{file_name}.map");
                    super::sourcemap::write_external_map(
                        &options.out_dir,
                        &map_filename,
                        &map.json,
                    )
                    .with_context(|| {
                        format!(
                            "writing source map {}",
                            options.out_dir.join(&map_filename).display()
                        )
                    })?;
                    Ok(code)
                }
                SourceMapOption::Inline => {
                    Ok(super::sourcemap::inline_source_map(&code, &map.json))
                }
                SourceMapOption::None => unreachable!(),
            }
        }
    }
}

fn transpile_library_esm(source: &str) -> Result<String> {
    let options = crate::transform::TransformOptions {
        jsx_pragma: None,
        jsx_fragment: None,
        jsx_automatic: true,
        ts_target: crate::transform::TypeScriptTarget::ES2020,
        source_maps: false,
        minify: false,
        dev_mode: false,
    };
    crate::transform::transform_tsx::transform_tsx(source, &options).map(|result| result.code)
}

fn ensure_library_source_path(path: &Path, spec: &str, role: &str) -> Result<()> {
    if is_library_source_path(path) {
        return Ok(());
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("<none>");
    anyhow::bail!(
        "jet build --lib: unsupported local {role} extension '.{ext}' for {spec} at {}; \
         library mode only inlines JS/TS source modules. Configure css_merge/raw_copy \
         or add a loader before importing this asset.",
        path.display()
    )
}

fn is_library_source_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs")
    )
}

/// Read the `name` field from a `package.json`, falling back to `"lib"` when
/// it is missing or the file cannot be parsed. Used to derive an IIFE global
/// name when the caller did not supply one.
fn read_package_name(pkg_path: &Path) -> String {
    crate::resolver::package::read_package_json(pkg_path)
        .ok()
        .and_then(|p| p.name)
        .unwrap_or_else(|| "lib".to_string())
}

/// Derive a JS-identifier global name from a package name.
///
///   `my-lib`            → `myLib`
///   `@scope/widget-kit` → `widgetKit`  (scope dropped)
///   `123abc`            → `_123abc`     (leading digit guarded)
///
/// The result is always a valid identifier: scope (`@scope/`) is dropped, the
/// remaining segments are camel-cased on `-`/`.`/`/` boundaries, any other
/// non-identifier byte becomes `_`, and a leading digit is prefixed with `_`.
pub(crate) fn derive_global_name(pkg_name: &str) -> String {
    // Drop an npm scope: `@scope/name` → `name`.
    let base = pkg_name.rsplit('/').next().unwrap_or(pkg_name);

    let mut out = String::new();
    let mut upper_next = false;
    for ch in base.chars() {
        if ch == '-' || ch == '.' || ch == '/' || ch == ' ' || ch == '@' {
            // Word boundary: camel-case the next kept char.
            upper_next = !out.is_empty();
            continue;
        }
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' {
            if upper_next {
                out.extend(ch.to_uppercase());
            } else {
                out.push(ch);
            }
        } else {
            out.push('_');
        }
        upper_next = false;
    }

    if out.is_empty() {
        return "lib".to_string();
    }
    // A JS identifier must not start with a digit.
    if out
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        out.insert(0, '_');
    }
    out
}

/// Wrap a bundled ESM entry as a global-var IIFE.
///
/// The bundled `esm` body already has every external import hoisted to the top
/// as `import ... from "pkg"` statements (see [`bundle_library_entry`]). For an
/// IIFE we cannot keep `import`s — the script must run as a classic global —
/// so each hoisted external import is rewritten to read from a browser global
/// (`window`/`globalThis`). The mapping is the conventional one: the package's
/// global is `globalThis[<derive_global_name(pkg)>]`, e.g. `react` → the
/// `React` global, `react-dom` → `ReactDom`.
///
/// The remaining body is `export`-stripped (named exports are collected onto a
/// returned namespace object; `export default` becomes the namespace itself),
/// and the whole thing is assigned to `var <global_name> = (function () { … })();`.
///
/// TODO(#170 follow-up): the global-name mapping for externals is a simple
/// derive-from-specifier heuristic. A configurable `globals` map (à la Rollup
/// `output.globals`) and UMD wrapping are deferred — anything beyond the
/// convention above (renamed default imports, `import * as`, re-export forms)
/// is best-effort.
///
/// `entry_path` is read directly to determine which symbols are *public*
/// (the entry's own `export`s) — distinct from the inlined internal modules
/// whose `export` keywords are stripped so they stay private to the IIFE.
fn wrap_iife(
    esm: &str,
    entry_path: &Path,
    global_name: &str,
    externals: &HashSet<String>,
) -> Result<String> {
    // Public surface = the entry module's own exports. Internal modules are
    // inlined into the body but their exports do not become public.
    let entry_source = std::fs::read_to_string(entry_path)
        .with_context(|| format!("reading entry {} for IIFE exports", entry_path.display()))?;
    let public = collect_entry_exports(&entry_source);

    let mut prelude = String::new();
    let mut body = String::new();

    for line in esm.lines() {
        let trimmed = line.trim();

        // Rewrite a hoisted external import into a `const … = globalThis.X` read.
        if trimmed.starts_with("import ") {
            if let Some(rewritten) = rewrite_iife_import(trimmed, externals) {
                prelude.push_str(&rewritten);
                prelude.push('\n');
                continue;
            }
            // Non-external / unrecognised import: drop it (an IIFE has no module
            // system to satisfy a bare import); keep going.
            continue;
        }

        // `export default <expr>;` → keep the value as a bare statement; the
        // default expression is also returned as the namespace below.
        if let Some(rest) = trimmed.strip_prefix("export default ") {
            // Emitted inline (rare for non-entry); the entry's default is
            // captured via `public.default_expr` and returned.
            let _ = rest;
            continue;
        }

        // `export { a, b };` → drop the statement (names handled via `public`).
        if trimmed.starts_with("export {") {
            continue;
        }

        // `export const|let|var|function|class NAME …` → strip the `export `
        // keyword (entry + inlined internals alike) so nothing leaks to the
        // module scope; the public ones are re-exposed via the namespace.
        if let Some(rest) = trimmed.strip_prefix("export ") {
            let indent_len = line.len() - line.trim_start().len();
            body.push_str(&line[..indent_len]);
            body.push_str(rest);
            body.push('\n');
            continue;
        }

        body.push_str(line);
        body.push('\n');
    }

    // Build the returned namespace.
    let mut out = String::new();
    out.push_str(&format!("var {global_name} = (function () {{\n"));
    if !prelude.is_empty() {
        out.push_str(&prelude);
    }
    out.push_str(&body);
    if !body.ends_with('\n') {
        out.push('\n');
    }
    if let Some(expr) = public.default_expr {
        // A default export defines the module value directly.
        out.push_str(&format!("return {expr};\n"));
    } else {
        out.push_str("return {\n");
        for name in &public.names {
            out.push_str(&format!("  {name}: {name},\n"));
        }
        out.push_str("};\n");
    }
    out.push_str("})();\n");
    Ok(out)
}

/// The public export surface of a single module.
struct EntryExports {
    /// Named exports (from `export const/function/class/{…}`).
    names: Vec<String>,
    /// `export default <expr>` target, when present. Takes precedence over
    /// `names` for the IIFE return value.
    default_expr: Option<String>,
}

/// Parse the *entry module's own* top-level exports (named + default).
///
/// Used to decide the IIFE's public namespace without confusing it with the
/// exports of inlined internal modules. `export … from "pkg"` re-export forms
/// are best-effort: the bare names in `export { a, b }` are collected; renamed
/// (`a as b`) and `* from` forms are deferred (TODO #170 follow-up).
fn collect_entry_exports(source: &str) -> EntryExports {
    let mut names: Vec<String> = Vec::new();
    let mut default_expr: Option<String> = None;

    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("export default ") {
            default_expr = Some(rest.trim_end_matches(';').trim().to_string());
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("export {") {
            if let Some(group) = rest.split('}').next() {
                for raw in group.split(',') {
                    let name = raw.trim();
                    if name.is_empty() || name.contains(" as ") {
                        continue;
                    }
                    names.push(name.to_string());
                }
            }
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("export ") {
            if let Some(name) = declared_name(rest) {
                names.push(name);
            }
        }
    }

    EntryExports {
        names,
        default_expr,
    }
}

/// Rewrite one hoisted external `import` line into a global-read `const`.
///
///   `import React from "react";`            → `const React = globalThis.React;`
///   `import { useState } from "react";`      → `const { useState } = globalThis.React;`
///   `import * as React from "react";`        → `const React = globalThis.React;`
///   `import "side-effect";`                  → ``  (dropped)
///
/// Returns `None` when the specifier is not external (should not happen for a
/// bundled library entry, whose only surviving imports are external).
fn rewrite_iife_import(line: &str, externals: &HashSet<String>) -> Option<String> {
    // import * as X from "pkg";
    if let Some(rest) = line.strip_prefix("import * as ") {
        let (name, spec) = split_import_from(rest)?;
        if !is_external_specifier(&spec, externals) {
            return None;
        }
        let g = external_global_path(&spec);
        return Some(format!("const {name} = {g};"));
    }
    // import { a, b } from "pkg";
    if let Some(rest) = line.strip_prefix("import {") {
        let (names, spec) = rest.split_once('}')?;
        let spec = import_spec(spec)?;
        if !is_external_specifier(&spec, externals) {
            return None;
        }
        let g = external_global_path(&spec);
        return Some(format!("const {{{names}}} = {g};"));
    }
    // import "pkg"; (side-effect) → nothing to bind under an IIFE.
    if let Some(rest) = line.strip_prefix("import ") {
        if rest.starts_with('"') || rest.starts_with('\'') {
            return Some(String::new());
        }
        // import Default from "pkg";
        let (name, spec) = split_import_from(rest)?;
        if !is_external_specifier(&spec, externals) {
            return None;
        }
        let g = external_global_path(&spec);
        return Some(format!("const {name} = {g};"));
    }
    None
}

/// Map an external specifier to the `globalThis.<Name>` expression an IIFE
/// reads it from. Sub-path specifiers (`react/jsx-runtime`) resolve to their
/// root package's global.
fn external_global_path(spec: &str) -> String {
    let root = spec.split('/').next().unwrap_or(spec);
    format!("globalThis.{}", derive_global_name(root))
}

/// Extract the binding name declared by an `export`-stripped declaration head,
/// i.e. the `NAME` in `const NAME =`, `function NAME(`, `class NAME {`.
fn declared_name(decl: &str) -> Option<String> {
    let decl = decl.trim();
    for kw in ["const", "let", "var", "function", "class", "async function"] {
        if let Some(rest) = decl.strip_prefix(&format!("{kw} ")) {
            let name = rest
                .split(['=', ' ', '(', '{', ':', '<', ';'])
                .find(|s| !s.is_empty())?
                .trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// `preserve_modules` emission: one output file per source module reachable
/// from the entries, mirroring the source tree under `out_dir`.
///
/// Internal relative imports are rewritten to point at the emitted siblings
/// (`./relative.js` or `./relative.cjs`); external imports stay bare. The
/// entry file keeps its original `export … from "./x"` / re-export structure
/// so a consumer can `import` the entry *or* deep-import any emitted module.
fn build_library_preserve_modules(
    options: &LibBuildOptions,
    entries: &[crate::resolver::package::LibraryEntry],
    externals: &HashSet<String>,
) -> Result<LibBuildResult> {
    for format in &options.formats {
        if matches!(format, OutputFormat::Iife) {
            anyhow::bail!(
                "jet build --lib --preserve-modules: iife output is not supported; \
                 use esm or cjs preserve-modules output, or drop --preserve-modules \
                 for single-file iife output"
            );
        }
    }

    // Collect every module reachable from all entries (BFS over relative
    // imports). The map key is the canonical absolute path; the value is the
    // path relative to the common source root, used to mirror the tree.
    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut module_paths: Vec<PathBuf> = Vec::new();

    let mut entry_abs: Vec<(String, PathBuf)> = Vec::new();
    for entry in entries {
        let entry_path = resolve_entry_path(&options.project_root, &entry.source)
            .with_context(|| format!("resolving entry source {}", entry.source))?;
        entry_abs.push((entry.subpath.clone(), entry_path.clone()));
        collect_modules(&entry_path, externals, &mut visited, &mut module_paths)?;
    }

    // Source root = the project's `src` dir if every module lives under it,
    // else the deepest common ancestor of all modules. The emitted tree
    // mirrors each module's path relative to this root.
    let src_root = common_source_root(&module_paths);

    let mut outputs = Vec::new();

    for module in &module_paths {
        let rel = module
            .strip_prefix(&src_root)
            .unwrap_or(module)
            .to_path_buf();
        for format in &options.formats {
            let out_rel = preserve_module_output_rel(&rel, format)?;
            let out_path = options.out_dir.join(&out_rel);
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("creating {}", parent.display()))?;
            }

            let esm = rewrite_module_for_preserve(module, externals, format)?;
            let code = transpile_library_esm(&esm)
                .with_context(|| format!("transpiling {}", module.display()))?;
            let code = match format {
                OutputFormat::Esm => code,
                OutputFormat::Cjs => esm_to_cjs(&code),
                OutputFormat::Iife => unreachable!("validated above"),
            };
            std::fs::write(&out_path, &code)
                .with_context(|| format!("writing {}", out_path.display()))?;

            outputs.push(EntryOutput {
                subpath: format!("./{}", out_rel.to_string_lossy().replace('\\', "/")),
                format: format.clone(),
                path: out_path,
                code,
                dts: None,
            });
        }
    }

    // Post-emit asset steps run for preserve_modules builds too.
    let assets = run_post_emit_assets(options)?;

    Ok(LibBuildResult {
        entries: outputs,
        types: Vec::new(),
        assets,
    })
}

/// Recursively collect all internal relative modules reachable from `path`.
fn collect_modules(
    path: &Path,
    externals: &HashSet<String>,
    visited: &mut HashSet<PathBuf>,
    order: &mut Vec<PathBuf>,
) -> Result<()> {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if !visited.insert(canonical.clone()) {
        return Ok(());
    }
    order.push(canonical.clone());

    let source =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    for spec in module_specifiers(&source, path)? {
        if is_external_specifier(&spec, externals) {
            continue;
        }
        if let Some(target) = resolve_relative(path, &spec)? {
            collect_modules(&target, externals, visited, order)?;
        }
    }
    Ok(())
}

/// Parse the import/export-from specifiers of a module's top-level statements.
fn module_specifiers(source: &str, path: &Path) -> Result<Vec<String>> {
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
        .parse(source, None)
        .context("parsing module source")?;
    let root = tree.root_node();

    let mut specs = Vec::new();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        let kind = child.kind();
        if kind != "import_statement" && kind != "export_statement" {
            continue;
        }
        if let Some(spec) = statement_specifier(source, &child) {
            specs.push(spec);
        }
    }
    Ok(specs)
}

/// Determine the source root the emitted tree mirrors. Prefers the deepest
/// common ancestor of all modules so the relative layout under `out_dir`
/// matches the source layout (without leaking the absolute prefix).
fn common_source_root(modules: &[PathBuf]) -> PathBuf {
    let mut iter = modules.iter();
    let Some(first) = iter.next() else {
        return PathBuf::new();
    };
    let mut prefix: Vec<&std::ffi::OsStr> = first
        .parent()
        .map(|p| p.iter().collect())
        .unwrap_or_default();
    for m in iter {
        let comps: Vec<&std::ffi::OsStr> =
            m.parent().map(|p| p.iter().collect()).unwrap_or_default();
        let common = prefix
            .iter()
            .zip(comps.iter())
            .take_while(|(a, b)| a == b)
            .count();
        prefix.truncate(common);
    }
    prefix.iter().collect()
}

/// Rewrite a relative path's extension to `.js` for the emitted sibling.
fn with_js_extension(rel: &Path) -> PathBuf {
    rel.with_extension("js")
}

/// Rewrite a relative path's extension for preserve-modules output.
fn preserve_module_output_rel(rel: &Path, format: &OutputFormat) -> Result<PathBuf> {
    match format {
        OutputFormat::Esm => Ok(with_js_extension(rel)),
        OutputFormat::Cjs => Ok(rel.with_extension("cjs")),
        OutputFormat::Iife => {
            anyhow::bail!("jet build --lib --preserve-modules: iife output is not supported")
        }
    }
}

fn preserve_module_specifier_extension(format: &OutputFormat) -> &'static str {
    match format {
        OutputFormat::Cjs => "cjs",
        OutputFormat::Esm | OutputFormat::Iife => "js",
    }
}

/// Rewrite one module's source for `preserve_modules` emission:
///   * internal relative imports point at the emitted same-format sibling,
///   * external imports are kept bare,
///   * everything else is verbatim.
fn rewrite_module_for_preserve(
    path: &Path,
    externals: &HashSet<String>,
    format: &OutputFormat,
) -> Result<String> {
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
    let tree = parser.parse(&source, None).context("parsing module")?;
    let root = tree.root_node();

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
        if is_external_specifier(&spec, externals) {
            continue;
        }

        // Internal relative specifier — rewrite its extension to `.js` so it
        // points at the emitted sibling.
        let Some((str_start, str_end)) = first_string_range(&child) else {
            continue;
        };

        // Emit text up to the string literal, then the rewritten specifier.
        out.push_str(&source[last_end..str_start]);
        let rewritten = rewrite_relative_specifier_with_extension(
            &spec,
            preserve_module_specifier_extension(format),
        );
        out.push_str(&format!("\"{rewritten}\""));
        last_end = str_end;
    }
    out.push_str(&source[last_end..]);
    Ok(out)
}

/// Find the byte range of the first `string` child of an import/export
/// statement (the module specifier literal).
fn first_string_range(node: &tree_sitter::Node) -> Option<(usize, usize)> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "string" {
            return Some((child.start_byte(), child.end_byte()));
        }
    }
    None
}

/// Rewrite a relative specifier to its emitted `.js` sibling, keeping the
/// `./` / `../` prefix. `./util.ts` → `./util.js`, `./util` → `./util.js`,
/// `./sub/mod` → `./sub/mod.js`.
fn rewrite_relative_specifier(spec: &str) -> String {
    rewrite_relative_specifier_with_extension(spec, "js")
}

fn rewrite_relative_specifier_with_extension(spec: &str, extension: &str) -> String {
    // Strip a known source extension, then append the requested emitted
    // extension.
    let stripped = spec
        .strip_suffix(".ts")
        .or_else(|| spec.strip_suffix(".tsx"))
        .or_else(|| spec.strip_suffix(".jsx"))
        .or_else(|| spec.strip_suffix(".mjs"))
        .or_else(|| spec.strip_suffix(".cjs"))
        .or_else(|| spec.strip_suffix(".js"))
        .unwrap_or(spec);
    format!("{stripped}.{extension}")
}

/// Rewrite the relative specifier inside an `export … from "./m"` re-export
/// statement to its emitted `.js` sibling, leaving the export clause untouched.
///
///   `export { a as b } from "./m"`  → `export { a as b } from "./m.js"`
///   `export * from "../util.ts"`     → `export * from "../util.js"`
///
/// Only the first string literal (the module specifier) is replaced. `spec` is
/// the already-unquoted specifier extracted from the statement.
fn rewrite_export_from_specifier(stmt: &str, spec: &str) -> String {
    let normalised = rewrite_relative_specifier(spec);
    // Replace the quoted specifier in place, preserving the original quote
    // style. The specifier always appears verbatim (sans quotes) in `stmt`.
    for quote in ['"', '\'', '`'] {
        let needle = format!("{quote}{spec}{quote}");
        if let Some(idx) = stmt.find(&needle) {
            let mut out = String::with_capacity(stmt.len());
            out.push_str(&stmt[..idx]);
            out.push('"');
            out.push_str(&normalised);
            out.push('"');
            out.push_str(&stmt[idx + needle.len()..]);
            return out;
        }
    }
    // Specifier not found verbatim (unexpected): return the statement unchanged.
    stmt.to_string()
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
///   `.`        + Esm → `index.js`     + Cjs → `index.cjs`  + Iife → `index.iife.js`
///   `./client` + Esm → `client.js`    + Cjs → `client.cjs` + Iife → `client.iife.js`
///
/// IIFE gets its own `.iife.js` suffix so an `[esm, iife]` build does not
/// overwrite the ESM output with the global-script flavour.
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
        OutputFormat::Iife => "iife.js",
        OutputFormat::Esm => "js",
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
        false,
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
    transpile_library_esm(&out)
}

/// Recursively inline one module's body.
///
/// External imports are pushed (deduplicated by verbatim text) to
/// `external_imports`; internal relative imports/re-exports are replaced by
/// the inlined body of their target module. Every other statement is kept
/// verbatim.
///
/// `make_private` strips this module's (and every module it transitively
/// inlines) top-level `export `/`export default ` keywords so its bindings
/// stay private to the bundle. It is set when a parent inlines the module to
/// satisfy a *named* re-export (`export { a } from "./m"`): only the named
/// bindings should become public, so the target's own `export` keywords are
/// dropped and the parent re-exports the chosen names explicitly. A `export *
/// from "./m"` inlines with `make_private = false` so every export survives.
#[allow(clippy::too_many_arguments)]
fn inline_module(
    path: &Path,
    externals: &HashSet<String>,
    external_imports: &mut Vec<String>,
    seen_external: &mut HashSet<String>,
    inlined_files: &mut HashSet<PathBuf>,
    make_private: bool,
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

        let stmt_text = &source[stmt_start..stmt_end];

        if is_external_specifier(&spec, externals) {
            // External `export ... from "pkg"` re-exports stay as their own
            // statement so the binding is re-exported from the package; the CJS
            // pass rewrites them to `exports.x = require("pkg").x`. Hoisting one
            // copy (deduplicated) is enough — do not also splice it into the
            // body, or the re-export would be emitted twice.
            if seen_external.insert(stmt_text.to_string()) {
                external_imports.push(stmt_text.to_string());
            }
            // A plain side-effect / default / named *import* is fully satisfied
            // by the hoisted statement above; an export re-export is also
            // satisfied by the hoisted copy, so nothing is spliced into `out`.
        } else if kind == "export_statement" {
            // Internal relative *re-export* (`export … from "./m"`): in
            // single-file bundle mode we FOLLOW and INLINE the target module
            // so the emitted entry is self-contained — there is no emitted
            // `./m.js` sibling to reference (preserve_modules mode handles the
            // per-file case separately and is not routed through here).
            //
            //   `export * from "./m"`        → inline `./m` keeping its own
            //       `export` keywords; every named export of `./m` is hoisted
            //       and so re-exported from the bundle, matching `export *`.
            //   `export { a, b as c } from "./m"` → inline `./m` with its top-
            //       level `export` keywords stripped (its bindings become
            //       private to the bundle), then emit a local `export { a, b as
            //       c };` referencing the now-inlined bindings.
            //
            // Recursion + the shared `inlined_files` visited-set make this
            // transitive (a re-export of a re-export is followed) and cycle-
            // safe (a module is inlined at most once).
            if let Some(target) = resolve_relative(path, &spec)? {
                if is_star_reexport(stmt_text) {
                    // `export * from "./m"` — inline keeping export keywords so
                    // the target's exports become the bundle's exports.
                    let inlined = inline_module(
                        &target,
                        externals,
                        external_imports,
                        seen_external,
                        inlined_files,
                        false,
                    )?;
                    out.push_str(&inlined);
                } else {
                    // `export { … } from "./m"` — inline the target privately
                    // (export keywords stripped) then re-export the named
                    // bindings under their public names.
                    let inlined = inline_module(
                        &target,
                        externals,
                        external_imports,
                        seen_external,
                        inlined_files,
                        true,
                    )?;
                    out.push_str(&inlined);
                    if let Some(clause) = export_named_clause(stmt_text) {
                        out.push_str(&format!("export {{{clause}}};\n"));
                    }
                }
            } else {
                // Unresolved relative re-export: keep verbatim (with the `.js`
                // sibling extension stamped on) rather than drop it.
                let rewritten = rewrite_export_from_specifier(stmt_text, &spec);
                out.push_str(&rewritten);
            }
        } else {
            // Internal relative *import* — inline the target module body in
            // place so the bundled entry stays self-contained. The target's
            // own `export` keywords are kept (verbatim inline), matching the
            // pre-existing single-file behaviour.
            if let Some(target) = resolve_relative(path, &spec)? {
                let inlined = inline_module(
                    &target,
                    externals,
                    external_imports,
                    seen_external,
                    inlined_files,
                    false,
                )?;
                out.push_str(&inlined);
            } else {
                // Unresolved relative import: keep verbatim rather than drop it.
                out.push_str(stmt_text);
            }
        }
    }

    // Trailing text after the last handled statement.
    out.push_str(&source[last_end..]);

    // When this module was inlined to satisfy a *named* re-export, strip its
    // (and every module it transitively inlined — all concatenated at this
    // same top level) `export ` keywords so the bindings stay private; the
    // parent re-exports the chosen names explicitly. Done once on the fully
    // assembled body so nested inlines are covered in a single pass.
    if make_private {
        out = strip_top_level_exports(&out);
    }
    Ok(out)
}

/// Strip top-level `export ` / `export default ` keywords from a concatenated
/// module body, leaving the underlying declaration in place but private.
///
///   `export function f() {}`     → `function f() {}`
///   `export const X = 1;`        → `const X = 1;`
///   `export default foo;`        → `foo;`
///   `export { a, b as c };`      → ``            (a bare named re-export of
///                                                 already-inlined bindings is
///                                                 dropped wholesale)
///
/// Operates per physical line on the top-level (un-indented) statements an
/// inlined library module produces. Indented lines (function bodies etc.) are
/// left untouched, so a nested `return export` substring is never mangled.
fn strip_top_level_exports(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    for line in body.lines() {
        // Only top-level (column-0) `export` statements form the module's
        // public surface; indented `export`-looking text is inside a block.
        let is_top_level = !line.starts_with(char::is_whitespace);
        if is_top_level {
            let trimmed = line.trim_start();
            if trimmed.starts_with("export {") {
                // Bare `export { … };` (no `from`) of now-inlined bindings:
                // drop the whole statement — the binding itself was already
                // emitted by the declaration line.
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("export default ") {
                out.push_str(rest);
                out.push('\n');
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("export ") {
                out.push_str(rest);
                out.push('\n');
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// `true` when an `export … from "…"` statement is the `export * from "…"`
/// (or `export * as ns from "…"`) star form, as opposed to a named
/// `export { … } from "…"` clause.
fn is_star_reexport(stmt: &str) -> bool {
    let after = stmt.trim_start().trim_start_matches("export").trim_start();
    after.starts_with('*')
}

/// Extract the `{ … }` clause body of an `export { a, b as c } from "…"`
/// statement (without the surrounding braces), to be re-emitted as a local
/// `export { … };` over the now-inlined bindings. Returns `None` when no
/// braced clause is present.
fn export_named_clause(stmt: &str) -> Option<String> {
    let open = stmt.find('{')?;
    let close = stmt[open..].find('}')? + open;
    Some(stmt[open + 1..close].trim().to_string())
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
fn resolve_relative(from: &Path, spec: &str) -> Result<Option<PathBuf>> {
    let Some(parent) = from.parent() else {
        return Ok(None);
    };
    let base = parent.join(spec.trim_start_matches("./"));
    if base.is_file() {
        ensure_library_source_path(&base, spec, "import")?;
        return Ok(Some(base));
    }
    let exts = ["ts", "tsx", "js", "jsx", "mjs", "cjs"];
    for ext in exts {
        let candidate = base.with_extension(ext);
        if candidate.is_file() {
            ensure_library_source_path(&candidate, spec, "import")?;
            return Ok(Some(candidate));
        }
    }
    for ext in exts {
        let candidate = base.join(format!("index.{ext}"));
        if candidate.is_file() {
            ensure_library_source_path(&candidate, spec, "import")?;
            return Ok(Some(candidate));
        }
    }
    Ok(None)
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
///   * `export { a as b }`             → `exports.b = a`
///   * `export { a as b } from "m"`    → `exports.b = require("m").a`
///   * `export * from "m"`             → re-export every named key of `require("m")`
///
/// External (`pkg`) specifiers stay bare (`require("pkg")`); relative
/// specifiers carry the emitted `.js` extension stamped on upstream by
/// [`rewrite_export_from_specifier`], so the CJS pass uses them verbatim.
///
/// TODO(#170 follow-up): `export { default as X } from "m"` interop nuances
/// (CJS `__esModule` default unwrapping) and live-binding getters (vs the
/// value-copy `exports.x = …` emitted here) are deferred — the value-copy form
/// is correct for the eagerly-evaluated modules a published library entry uses.
fn esm_to_cjs(esm: &str) -> String {
    let mut out = String::new();
    let mut export_assignments = Vec::new();
    for line in esm.lines() {
        let trimmed = line.trim();
        if let Some((rewritten, assignment)) = rewrite_cjs_export_declaration(trimmed, line) {
            out.push_str(&rewritten);
            export_assignments.push(assignment);
        } else if let Some(rewritten) = rewrite_cjs_line(trimmed) {
            out.push_str(&rewritten);
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    if !export_assignments.is_empty() {
        if !out.ends_with('\n') {
            out.push('\n');
        }
        for assignment in export_assignments {
            out.push_str(&assignment);
            out.push('\n');
        }
    }
    out
}

fn rewrite_cjs_export_declaration(trimmed: &str, original: &str) -> Option<(String, String)> {
    for kw in ["const", "let", "var"] {
        if let Some(rest) = trimmed.strip_prefix(&format!("export {kw} ")) {
            let name = rest.split(['=', ' ', ':']).next()?.trim();
            if name.is_empty() || name.starts_with('{') || name.starts_with('[') {
                return None;
            }
            return Some((
                strip_export_keyword_preserving_indent(original),
                format!("exports.{name} = {name};"),
            ));
        }
    }
    for kw in ["function", "class"] {
        if let Some(rest) = trimmed.strip_prefix(&format!("export {kw} ")) {
            let name = rest.split(['(', ' ', '{', '<']).next().unwrap_or("").trim();
            if name.is_empty() {
                return None;
            }
            return Some((
                strip_export_keyword_preserving_indent(original),
                format!("exports.{name} = {name};"),
            ));
        }
    }
    None
}

fn strip_export_keyword_preserving_indent(line: &str) -> String {
    if let Some(idx) = line.find("export ") {
        let mut out = String::with_capacity(line.len().saturating_sub("export ".len()));
        out.push_str(&line[..idx]);
        out.push_str(&line[idx + "export ".len()..]);
        out
    } else {
        line.to_string()
    }
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
    // export * from "spec";  (re-export every named binding of `spec`)
    //   → re-export all keys except `default` onto `exports`.
    // Works for both external (`pkg`) and relative (`./m.js`) specifiers; the
    // specifier is used verbatim, so a relative one already carries the `.js`
    // extension stamped on by `rewrite_export_from_specifier`.
    if let Some(rest) = line.strip_prefix("export * from ") {
        let spec = import_spec(rest)?;
        return Some(format!(
            "Object.keys(require(\"{spec}\")).forEach(function (k) {{ \
             if (k !== \"default\") exports[k] = require(\"{spec}\")[k]; }});"
        ));
    }
    // export { a, b as c } from "spec";  (named re-export from another module)
    //   → exports.a = require("spec").a; exports.c = require("spec").b;
    // The specifier is used verbatim (external `pkg` stays bare; a relative one
    // already carries `.js`). `a as b` maps local `a` to exported name `b`.
    if let Some(rest) = line.strip_prefix("export {") {
        if let Some((clause, tail)) = rest.split_once('}') {
            // Only the `... } from "spec"` shape is a re-export; a bare
            // `export { ... };` (no `from`) is handled by the local branch
            // further down.
            if tail.trim_start().starts_with("from ") {
                let spec = import_spec(tail.trim_start().trim_start_matches("from"))?;
                let mut buf = String::new();
                for raw in clause.split(',') {
                    let entry = raw.trim();
                    if entry.is_empty() {
                        continue;
                    }
                    let (local, exported) = split_export_alias(entry);
                    buf.push_str(&format!(
                        "exports.{exported} = require(\"{spec}\").{local};\n"
                    ));
                }
                if !buf.is_empty() {
                    return Some(buf.trim_end().to_string());
                }
                return Some(String::new());
            }
        }
    }
    // export { a, b as c };  (local re-export, no `from` — handled above)
    //   → exports.a = a; exports.c = b;
    // A renamed alias (`b as c`) binds the exported name `c` to the local `b`.
    if let Some(rest) = line.strip_prefix("export {") {
        let names = rest.split('}').next()?;
        let mut buf = String::new();
        for raw in names.split(',') {
            let entry = raw.trim();
            if entry.is_empty() {
                continue;
            }
            let (local, exported) = split_export_alias(entry);
            buf.push_str(&format!("exports.{exported} = {local};\n"));
        }
        if !buf.is_empty() {
            return Some(buf.trim_end().to_string());
        }
    }
    None
}

/// Split one entry of an `export { … }` clause into `(local, exported)`.
///
///   `a`        → (`a`, `a`)
///   `a as b`   → (`a`, `b`)   (local `a` re-exported under the name `b`)
fn split_export_alias(entry: &str) -> (String, String) {
    if let Some((local, exported)) = entry.split_once(" as ") {
        (local.trim().to_string(), exported.trim().to_string())
    } else {
        let name = entry.trim().to_string();
        (name.clone(), name)
    }
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
        assert_eq!(output_file_name(".", &OutputFormat::Iife), "index.iife.js");
        assert_eq!(
            output_file_name("./client", &OutputFormat::Esm),
            "client.js"
        );
        assert_eq!(
            output_file_name("./client", &OutputFormat::Cjs),
            "client.cjs"
        );
        assert_eq!(
            output_file_name("./client", &OutputFormat::Iife),
            "client.iife.js"
        );
    }

    #[test]
    fn derive_global_name_camel_cases_and_drops_scope() {
        assert_eq!(derive_global_name("my-lib"), "myLib");
        assert_eq!(derive_global_name("@scope/widget-kit"), "widgetKit");
        assert_eq!(derive_global_name("react"), "react");
        assert_eq!(derive_global_name("react-dom"), "reactDom");
        assert_eq!(derive_global_name("lodash.merge"), "lodashMerge");
        // Leading digit guarded into a valid identifier.
        assert_eq!(derive_global_name("123abc"), "_123abc");
        // Empty / pathological names fall back to `lib`.
        assert_eq!(derive_global_name(""), "lib");
        assert_eq!(derive_global_name("@scope/"), "lib");
    }

    #[test]
    fn rewrite_relative_specifier_targets_emitted_js_sibling() {
        assert_eq!(rewrite_relative_specifier("./util"), "./util.js");
        assert_eq!(rewrite_relative_specifier("./util.ts"), "./util.js");
        assert_eq!(rewrite_relative_specifier("./util.js"), "./util.js");
        assert_eq!(
            rewrite_relative_specifier("../sub/mod.tsx"),
            "../sub/mod.js"
        );
    }

    #[test]
    fn external_global_path_uses_root_package_global() {
        assert_eq!(external_global_path("react"), "globalThis.react");
        assert_eq!(external_global_path("react-dom"), "globalThis.reactDom");
        // Sub-path inherits its package's global.
        assert_eq!(
            external_global_path("react/jsx-runtime"),
            "globalThis.react"
        );
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

    #[test]
    fn split_export_alias_handles_plain_and_renamed() {
        assert_eq!(split_export_alias("a"), ("a".to_string(), "a".to_string()));
        assert_eq!(
            split_export_alias("a as b"),
            ("a".to_string(), "b".to_string())
        );
        assert_eq!(
            split_export_alias("  Foo as Bar  "),
            ("Foo".to_string(), "Bar".to_string())
        );
    }

    #[test]
    fn cjs_rewrite_named_reexport_from_external() {
        // `export { x } from "pkg"` keeps the external `require("pkg")`.
        let out = esm_to_cjs("export { useState } from \"react\";\n");
        assert!(
            out.contains("exports.useState = require(\"react\").useState;"),
            "{out}"
        );
    }

    #[test]
    fn cjs_rewrite_renamed_reexport_from_relative() {
        // `export { a as b } from "./m.js"` → exports.b = require("./m.js").a.
        let out = esm_to_cjs("export { Foo as Bar } from \"./foo.js\";\n");
        assert!(
            out.contains("exports.Bar = require(\"./foo.js\").Foo;"),
            "{out}"
        );
    }

    #[test]
    fn cjs_rewrite_star_reexport() {
        // `export * from "m"` → re-export every key except `default`.
        let out = esm_to_cjs("export * from \"./util.js\";\n");
        assert!(out.contains("Object.keys(require(\"./util.js\"))"), "{out}");
        assert!(out.contains("if (k !== \"default\")"), "{out}");
        assert!(
            out.contains("exports[k] = require(\"./util.js\")[k]"),
            "{out}"
        );
    }

    #[test]
    fn cjs_rewrite_local_renamed_export() {
        // `export { a as b };` (no `from`, `a` local) → exports.b = a.
        let out = esm_to_cjs("export { localA as renamedA };\n");
        assert!(out.contains("exports.renamedA = localA;"), "{out}");
        // Plain local export keeps the same name on both sides.
        let plain = esm_to_cjs("export { thing };\n");
        assert!(plain.contains("exports.thing = thing;"), "{plain}");
    }

    #[test]
    fn cjs_rewrite_multi_binding_reexport_from_relative() {
        // Mixed plain + renamed bindings in one `export { … } from` clause.
        let out = esm_to_cjs("export { a, b as c } from \"./m.js\";\n");
        assert!(out.contains("exports.a = require(\"./m.js\").a;"), "{out}");
        assert!(out.contains("exports.c = require(\"./m.js\").b;"), "{out}");
    }

    #[test]
    fn is_star_reexport_distinguishes_star_from_named() {
        assert!(is_star_reexport("export * from \"./m\";"));
        assert!(is_star_reexport("export * as ns from \"./m\";"));
        assert!(!is_star_reexport("export { a, b } from \"./m\";"));
        assert!(!is_star_reexport("export { Foo as Bar } from './m';"));
    }

    #[test]
    fn export_named_clause_extracts_braced_clause() {
        assert_eq!(
            export_named_clause("export { a, b as c } from \"./m\";").as_deref(),
            Some("a, b as c")
        );
        assert_eq!(
            export_named_clause("export { Foo } from './m';").as_deref(),
            Some("Foo")
        );
        // No braced clause (star form) → None.
        assert_eq!(export_named_clause("export * from \"./m\";"), None);
    }

    #[test]
    fn strip_top_level_exports_privatises_declarations() {
        let body = "export function f() { return 1; }\n\
                    export const X = 2;\n\
                    export default foo;\n\
                    export { a, b as c };\n\
                    function inner() {\n  export;\n}\n";
        let out = strip_top_level_exports(body);
        assert!(out.contains("function f() { return 1; }"), "{out}");
        assert!(!out.contains("export function f"), "{out}");
        assert!(out.contains("const X = 2;"), "{out}");
        assert!(out.contains("foo;"), "{out}");
        // The bare named-export clause is dropped wholesale.
        assert!(!out.contains("export {"), "{out}");
        // Indented `export`-looking text inside a block is left untouched.
        assert!(
            out.contains("  export;"),
            "indented export preserved: {out}"
        );
    }

    #[test]
    fn rewrite_export_from_specifier_stamps_js_extension() {
        assert_eq!(
            rewrite_export_from_specifier("export { Foo as Bar } from \"./foo\";", "./foo"),
            "export { Foo as Bar } from \"./foo.js\";"
        );
        assert_eq!(
            rewrite_export_from_specifier("export * from \"../util.ts\";", "../util.ts"),
            "export * from \"../util.js\";"
        );
        // Single-quoted specifier is normalised to a double-quoted `.js` one.
        assert_eq!(
            rewrite_export_from_specifier("export { x } from './m';", "./m"),
            "export { x } from \"./m.js\";"
        );
    }
}
// </HANDWRITE>
