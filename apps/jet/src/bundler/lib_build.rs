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
//! @issue #784
//! @issue #795
//! @issue #797
//! @issue #798
//! @issue #936

use anyhow::{bail, Context, Result};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::types::{OutputFormat, SourceMapOption};
use crate::resolver::package::{external_package_names, library_entries, LibraryEntry};
use crate::resolver::{ModuleResolver, ResolveOptions};

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
    /// Emitted `.d.ts` declaration files. Single-file library builds record
    /// one per public entry (`.`, `./client`); preserve-modules builds record
    /// one per emitted source module. Empty when declaration emission is
    /// disabled.
    /// @issue #171
    /// @issue #798
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

    // @spec .aw/tech-design/projects/jet/config/jet-build-lib-lib-config-section-css-merge-raw-copy-referenced-i.md#logic
    entries.retain(|entry| !asset_export_entry(entry));

    if entries.is_empty() {
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

    for entry in &mut entries {
        if let Some(source) = source_entry_fallback(&options.project_root, entry) {
            entry.source = source;
        }
    }

    Ok(entries)
}

fn asset_export_entry(entry: &LibraryEntry) -> bool {
    let source = normalize_export_path(&entry.source);
    !source.as_os_str().is_empty() && !is_library_source_path(&source)
}

fn normalize_export_path(path: &str) -> PathBuf {
    let trimmed = path
        .trim_start_matches("./")
        .trim_matches('/')
        .replace('\\', "/");
    if trimmed.is_empty() || trimmed == "." {
        PathBuf::new()
    } else {
        PathBuf::from(trimmed)
    }
}

fn source_entry_fallback(root: &Path, entry: &LibraryEntry) -> Option<String> {
    let source = entry.source.trim_start_matches("./");
    let source_path = Path::new(source);
    let is_default_dist_output = source_path
        .components()
        .next()
        .and_then(|component| match component {
            std::path::Component::Normal(name) => name.to_str(),
            _ => None,
        })
        .map(|name| name == "dist")
        .unwrap_or(false);

    if !is_default_dist_output && resolve_entry_path(root, &entry.source).is_ok() {
        return None;
    }

    for stem in source_entry_fallback_stems(entry, source_path) {
        if let Some(candidate) = source_candidate(root, &stem) {
            return Some(candidate);
        }
    }
    None
}

fn source_entry_fallback_stems(entry: &LibraryEntry, source_path: &Path) -> Vec<PathBuf> {
    let mut stems = Vec::new();

    if let Ok(without_dist) = source_path.strip_prefix("dist") {
        let mut stem = without_dist.to_path_buf();
        stem.set_extension("");
        if !stem.as_os_str().is_empty() {
            stems.push(stem);
        }
    }

    if entry.subpath == "." {
        stems.push(PathBuf::from("index"));
    } else {
        let mut stem = PathBuf::from(entry.subpath.trim_start_matches("./"));
        stem.set_extension("");
        if !stem.as_os_str().is_empty() {
            stems.push(stem);
        }
    }

    stems
}

fn source_candidate(root: &Path, stem: &Path) -> Option<String> {
    for ext in ["tsx", "ts", "jsx", "js"] {
        let candidate = Path::new("src").join(stem).with_extension(ext);
        if root.join(&candidate).is_file() {
            return Some(path_to_slash(&candidate));
        }
    }
    for ext in ["tsx", "ts", "jsx", "js"] {
        let candidate = Path::new("src").join(stem).join(format!("index.{ext}"));
        if root.join(&candidate).is_file() {
            return Some(path_to_slash(&candidate));
        }
    }
    None
}

fn path_to_slash(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
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
            ensure_library_output_parses(&code, &file_name)?;
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

    let source_style_asset = emit_library_style_imports(&options, &entries, &externals)?;

    // Post-emit asset steps: CSS cascade-merge + raw-asset copy. No-ops (and
    // thus byte-identical to today) when neither is configured.
    let mut assets = run_post_emit_assets(&options)?;
    if assets.is_empty() {
        if let Some(asset) = source_style_asset {
            assets.push(asset);
        }
    }
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

/// A library build must not claim success for an artifact Node cannot parse.
/// This is a final safety net for lowering and scope-isolation regressions.
fn ensure_library_output_parses(code: &str, output_name: &str) -> Result<()> {
    if crate::bundler::dce::js_parses_without_errors(code) {
        Ok(())
    } else {
        anyhow::bail!(
            "jet build --lib generated invalid JavaScript for `{output_name}`; refusing to write a successful artifact"
        )
    }
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
    let mut pending_outputs = Vec::new();
    let mut diagnostics = Vec::new();

    for module in modules {
        let source = std::fs::read_to_string(&module)
            .with_context(|| format!("reading {} for .d.ts", module.display()))?;
        let emit = super::dts::emit_declarations_with_diagnostics(&source)
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
        for diagnostic in emit.diagnostics {
            diagnostics.push(format!(
                "{}:{}:{}: {}",
                module.display(),
                diagnostic.line,
                diagnostic.column,
                diagnostic.message
            ));
        }
        pending_outputs.push((dts_out, emit.text));
    }

    if !diagnostics.is_empty() {
        anyhow::bail!(
            "dts: isolatedDeclarations found {} error(s):\n  - {}",
            diagnostics.len(),
            diagnostics.join("\n  - ")
        );
    }

    for (dts_out, dts) in pending_outputs {
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
        // @spec .aw/tech-design/projects/jet/logic/jet-build-lib-dts-svgr-style-asset-re-exports-build-correctly-bu.md#logic
        if !should_chase_declaration_reexport_target(&spec) {
            continue;
        }
        if let Some(target) = resolve_relative(path, &spec)? {
            collect_reexport_declaration_modules(&target, externals, visited, order)?;
        }
    }
    Ok(())
}

fn should_chase_declaration_reexport_target(spec: &str) -> bool {
    let path = Path::new(specifier_path_part(spec));
    match path.extension().and_then(|ext| ext.to_str()) {
        None => true,
        Some("ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs") => true,
        Some(_) => false,
    }
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
    if out_css.is_file() && !css_merge_includes_output(options, &out_css) {
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

fn css_merge_includes_output(options: &LibBuildOptions, out_css: &Path) -> bool {
    options.css_merge.iter().any(|rel| {
        let src = options.project_root.join(rel);
        same_existing_path(&src, out_css)
    })
}

fn same_existing_path(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
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

fn is_library_asset_path(path: &Path) -> bool {
    path.is_file() && !is_library_source_path(path)
}

fn is_library_style_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some(ext)
            if ext.eq_ignore_ascii_case("css")
                || ext.eq_ignore_ascii_case("scss")
                || ext.eq_ignore_ascii_case("sass")
    )
}

fn specifier_path_part(spec: &str) -> &str {
    spec.split(['?', '#']).next().unwrap_or(spec)
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

    // TypeScript lowering may erase an unused external import before this IIFE
    // pass sees it. Preserve the entry's external-global contract anyway: an
    // IIFE library declares every entry peer/dependency global it authored,
    // whether or not the current optimizer retained a local reference.
    for line in entry_source.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("import ") {
            continue;
        }
        if let Some(rewritten) = rewrite_iife_import(trimmed, externals) {
            if !prelude.contains(&rewritten) {
                prelude.push_str(&rewritten);
                prelude.push('\n');
            }
        }
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
    let rest = line.trim_start().strip_prefix("import ")?.trim_start();
    // import "pkg"; (side-effect) → nothing to bind under an IIFE.
    if rest.starts_with('"') || rest.starts_with('\'') {
        return Some(String::new());
    }

    let (clause, spec) = parse_import_from_clause(rest)?;
    if !is_external_specifier(&spec, externals) {
        return None;
    }
    let g = external_global_path(&spec);
    let clause = clause.trim();

    // `import Default, { named } from "pkg"` needs two declarations in an
    // IIFE. Keeping it explicit also means named-import coalescing never turns
    // a previously-loadable IIFE into invalid `const Default, { named } = …`.
    if let Some((default_binding, names)) = split_named_import_clause(clause) {
        let mut declarations = Vec::new();
        if let Some(default_binding) = default_binding {
            declarations.push(format!("const {default_binding} = {g};"));
        }
        if !names.trim().is_empty() {
            let names = cjs_object_destructure_bindings(names);
            declarations.push(format!("const {{ {names} }} = {g};"));
        }
        return (!declarations.is_empty()).then(|| declarations.join("\n"));
    }

    // `import Default, * as Namespace from "pkg"` likewise needs two
    // declarations because the ESM mixed form has no direct destructuring
    // equivalent.
    if let Some((default_binding, namespace)) = clause.split_once(", * as ") {
        let default_binding = default_binding.trim();
        let namespace = namespace.trim();
        if is_js_identifier(default_binding) && is_js_identifier(namespace) {
            return Some(format!(
                "const {default_binding} = {g};\nconst {namespace} = {g};"
            ));
        }
    }

    if let Some(namespace) = clause.strip_prefix("* as ").map(str::trim) {
        return is_js_identifier(namespace).then(|| format!("const {namespace} = {g};"));
    }
    is_js_identifier(clause).then(|| format!("const {clause} = {g};"))
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
    let (dts_by_module, types_outputs) = if options.declaration {
        emit_preserve_module_declarations(options, &module_paths, &src_root)?
    } else {
        (HashMap::new(), Vec::new())
    };

    let mut outputs = Vec::new();

    for module in &module_paths {
        let module_key = module.canonicalize().unwrap_or_else(|_| module.clone());
        let dts_path = dts_by_module.get(&module_key).cloned();
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
            ensure_library_output_parses(&code, &out_rel.display().to_string())?;
            std::fs::write(&out_path, &code)
                .with_context(|| format!("writing {}", out_path.display()))?;

            outputs.push(EntryOutput {
                subpath: format!("./{}", out_rel.to_string_lossy().replace('\\', "/")),
                format: format.clone(),
                path: out_path,
                code,
                dts: dts_path.clone(),
            });
        }
    }

    let source_style_asset = emit_library_style_imports(options, entries, externals)?;

    // Post-emit asset steps run for preserve_modules builds too.
    let mut assets = run_post_emit_assets(options)?;
    if assets.is_empty() {
        if let Some(asset) = source_style_asset {
            assets.push(asset);
        }
    }

    Ok(LibBuildResult {
        entries: outputs,
        types: types_outputs,
        assets,
    })
}

// @spec .aw/tech-design/projects/jet/logic/jet-build-lib-dts-preserve-modules-dts-silently-emits-no-d-ts-fi.md#logic
fn emit_preserve_module_declarations(
    options: &LibBuildOptions,
    modules: &[PathBuf],
    source_root: &Path,
) -> Result<(HashMap<PathBuf, PathBuf>, Vec<TypesOutput>)> {
    let mut by_module = HashMap::new();
    let mut types_outputs = Vec::new();
    let mut pending_outputs = Vec::new();
    let mut diagnostics = Vec::new();

    for module in modules {
        let source = std::fs::read_to_string(module)
            .with_context(|| format!("reading {} for .d.ts", module.display()))?;
        let emit = super::dts::emit_declarations_with_diagnostics(&source)
            .with_context(|| format!("emitting .d.ts for {}", module.display()))?;
        let dts_out = declaration_module_output_path(&options.out_dir, source_root, module);
        let module_key = module.canonicalize().unwrap_or_else(|_| module.clone());

        by_module.insert(module_key, dts_out.clone());
        types_outputs.push(TypesOutput {
            subpath: preserve_type_subpath(&options.out_dir, &dts_out),
            path: dts_out.clone(),
        });
        for diagnostic in emit.diagnostics {
            diagnostics.push(format!(
                "{}:{}:{}: {}",
                module.display(),
                diagnostic.line,
                diagnostic.column,
                diagnostic.message
            ));
        }
        pending_outputs.push((dts_out, emit.text));
    }

    if !diagnostics.is_empty() {
        anyhow::bail!(
            "dts: isolatedDeclarations found {} error(s):\n  - {}",
            diagnostics.len(),
            diagnostics.join("\n  - ")
        );
    }

    for (dts_out, dts) in pending_outputs {
        if let Some(parent) = dts_out.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        std::fs::write(&dts_out, &dts).with_context(|| format!("writing {}", dts_out.display()))?;
    }

    Ok((by_module, types_outputs))
}

fn preserve_type_subpath(out_dir: &Path, dts_out: &Path) -> String {
    let rel = dts_out.strip_prefix(out_dir).unwrap_or(dts_out);
    format!("./{}", rel.to_string_lossy().replace('\\', "/"))
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

fn emit_library_style_imports(
    options: &LibBuildOptions,
    entries: &[LibraryEntry],
    externals: &HashSet<String>,
) -> Result<Option<AssetOutput>> {
    let mut visited_modules = HashSet::new();
    let mut seen_styles = HashSet::new();
    let mut styles = Vec::new();
    for entry in entries {
        let entry_path = resolve_entry_path(&options.project_root, &entry.source)
            .with_context(|| format!("resolving entry source {}", entry.source))?;
        collect_style_imports(
            &entry_path,
            externals,
            &mut visited_modules,
            &mut seen_styles,
            &mut styles,
        )?;
    }
    if styles.is_empty() {
        return Ok(None);
    }

    let config = crate::css::TailwindConfig::load(&options.project_root).unwrap_or_default();
    let pipeline = crate::css::CssPipeline::new(options.project_root.clone(), config, false);
    let mut bundle = String::new();
    for style in styles {
        let output = pipeline
            .process(&style)
            .with_context(|| format!("processing library style import {}", style.display()))?;
        bundle.push_str(&output.css);
        if !bundle.ends_with('\n') {
            bundle.push('\n');
        }
    }

    std::fs::create_dir_all(&options.out_dir)
        .with_context(|| format!("creating out_dir {}", options.out_dir.display()))?;
    let out_css = options.out_dir.join("style.css");
    std::fs::write(&out_css, bundle)
        .with_context(|| format!("writing library style bundle {}", out_css.display()))?;
    Ok(Some(AssetOutput {
        path: out_css,
        kind: AssetKind::MergedCss,
    }))
}

fn collect_style_imports(
    path: &Path,
    externals: &HashSet<String>,
    visited_modules: &mut HashSet<PathBuf>,
    seen_styles: &mut HashSet<PathBuf>,
    styles: &mut Vec<PathBuf>,
) -> Result<()> {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if !visited_modules.insert(canonical) {
        return Ok(());
    }

    let source =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    for spec in module_specifiers(&source, path)? {
        if is_external_specifier(&spec, externals) {
            continue;
        }
        if let Some(asset_path) = resolve_relative_asset(path, &spec) {
            if is_library_style_path(&asset_path) {
                let key = asset_path
                    .canonicalize()
                    .unwrap_or_else(|_| asset_path.clone());
                if seen_styles.insert(key) {
                    styles.push(asset_path);
                }
            }
            continue;
        }
        if let Some(target) = resolve_relative(path, &spec)? {
            collect_style_imports(&target, externals, visited_modules, seen_styles, styles)?;
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
            let Some((str_start, str_end)) = first_string_range(&child) else {
                continue;
            };
            let rewritten = rewrite_external_library_specifier_for_node(path, &spec);
            if rewritten != spec {
                out.push_str(&source[last_end..str_start]);
                out.push_str(&format!("\"{rewritten}\""));
                last_end = str_end;
            }
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
    rewrite_statement_specifier(stmt, spec, &normalised)
}

/// Replace one statement's module specifier, preserving all surrounding import
/// / export syntax. The output uses a double-quoted string just as the library
/// relative-specifier rewrite does.
fn rewrite_statement_specifier(stmt: &str, spec: &str, replacement: &str) -> String {
    // Replace the quoted specifier in place, preserving the original quote
    // style. The specifier always appears verbatim (sans quotes) in `stmt`.
    for quote in ['"', '\'', '`'] {
        let needle = format!("{quote}{spec}{quote}");
        if let Some(idx) = stmt.find(&needle) {
            let mut out = String::with_capacity(stmt.len());
            out.push_str(&stmt[..idx]);
            out.push('"');
            out.push_str(replacement);
            out.push('"');
            out.push_str(&stmt[idx + needle.len()..]);
            return out;
        }
    }
    // Specifier not found verbatim (unexpected): return the statement unchanged.
    stmt.to_string()
}

/// Rewrite a legacy bare package subpath to the file Node ESM can actually
/// load. Node deliberately does not probe `.js` for `pkg/subpath` imports, so
/// a library bundle that leaves an extensionless legacy deep import behind is
/// syntactically valid but unloadable at runtime.
///
/// This is intentionally conservative: packages with an `exports` map retain
/// their authored specifier because that map owns their public subpaths. We
/// only rewrite an extensionless subpath when the in-repo resolver proves an
/// `exports`-less package file exists below the same package root.
fn rewrite_external_library_specifier_for_node(from: &Path, spec: &str) -> String {
    let Some((package_name, subpath)) = bare_package_name_and_subpath(spec) else {
        return spec.to_string();
    };
    if is_explicit_node_loadable_extension(subpath)
        || crate::resolver::is_node_builtin_specifier(spec)
    {
        return spec.to_string();
    }

    let Ok(resolver) = ModuleResolver::new(ResolveOptions::for_node_test()) else {
        return spec.to_string();
    };
    let Ok(resolved) = resolver.resolve(spec, from) else {
        return spec.to_string();
    };
    let Some(package_dir) = package_dir_for_resolved_subpath(&resolved.path, package_name) else {
        return spec.to_string();
    };
    if package_declares_exports(&package_dir) {
        return spec.to_string();
    }
    let Ok(relative) = resolved.path.strip_prefix(&package_dir) else {
        return spec.to_string();
    };
    if !is_node_loadable_library_module(relative) {
        return spec.to_string();
    }
    let relative = relative.to_string_lossy().replace('\\', "/");
    if relative.is_empty() {
        return spec.to_string();
    }
    let canonical = format!("{package_name}/{relative}");
    if canonical == spec {
        return spec.to_string();
    }

    canonical
}

/// A dot in a package subpath is not necessarily a Node-loadable extension:
/// legacy packages often ship `chunk.min.js` behind an authored `chunk.min`
/// specifier. Only concrete JavaScript module suffixes are already safe for
/// Node ESM to load without extension probing.
fn is_explicit_node_loadable_extension(subpath: &str) -> bool {
    matches!(
        Path::new(subpath)
            .extension()
            .and_then(|extension| extension.to_str()),
        Some("js" | "mjs" | "cjs")
    )
}

fn bare_package_name_and_subpath(spec: &str) -> Option<(&str, &str)> {
    if spec.starts_with('@') {
        let mut parts = spec.splitn(3, '/');
        let scope = parts.next()?;
        let package = parts.next()?;
        let subpath = parts.next()?;
        if subpath.is_empty() {
            return None;
        }
        let package_len = scope.len() + 1 + package.len();
        Some((&spec[..package_len], subpath))
    } else {
        let (package, subpath) = spec.split_once('/')?;
        (!package.is_empty() && !subpath.is_empty()).then_some((package, subpath))
    }
}

fn package_dir_for_resolved_subpath(resolved: &Path, package_name: &str) -> Option<PathBuf> {
    let package_parts = package_name.split('/').collect::<Vec<_>>();
    let package_leaf = *package_parts.last()?;
    for ancestor in resolved.ancestors().skip(1) {
        if ancestor.file_name().and_then(|name| name.to_str()) != Some(package_leaf) {
            continue;
        }
        let parent = ancestor.parent()?;
        let is_package_root = if package_parts.len() == 1 {
            parent.file_name().and_then(|name| name.to_str()) == Some("node_modules")
        } else {
            parent.file_name().and_then(|name| name.to_str()) == package_parts.first().copied()
                && parent
                    .parent()
                    .and_then(Path::file_name)
                    .and_then(|name| name.to_str())
                    == Some("node_modules")
        };
        if is_package_root {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

fn package_declares_exports(package_dir: &Path) -> bool {
    std::fs::read_to_string(package_dir.join("package.json"))
        .ok()
        .and_then(|source| serde_json::from_str::<serde_json::Value>(&source).ok())
        .is_some_and(|package| package.get("exports").is_some())
}

fn is_node_loadable_library_module(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("js" | "mjs" | "cjs")
    )
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
    let mut used_top_level_bindings: HashSet<String> = HashSet::new();
    let mut external_binding_owners: HashMap<String, (String, String)> = HashMap::new();

    let body = inline_module(
        entry,
        externals,
        &mut external_imports,
        &mut seen_external,
        &mut inlined_files,
        false,
        &mut used_top_level_bindings,
        &mut external_binding_owners,
    )?;

    // Inlined modules can each import the same external package.  Deduplicating
    // by the complete statement is insufficient: two otherwise-valid imports
    // such as `{ forwardRef, useMemo }` and `{ forwardRef, useRef }` would
    // leave two declarations of the `forwardRef` binding in the emitted ESM.
    // Coalesce compatible named-import clauses by package and local binding
    // before handing the source to the TypeScript transform / CJS lowering.
    let external_imports = coalesce_external_named_imports(&external_imports)?;

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

/// A named external import that may be safely coalesced with another named
/// import from the same package. Namespace and side-effect imports intentionally
/// stay outside this representation: their syntax cannot be combined with a
/// named clause without changing bindings or evaluation semantics.
#[derive(Debug)]
struct NamedExternalImport {
    specifier: String,
    default_binding: Option<String>,
    named_bindings: Vec<String>,
}

/// One emitted named-import clause for an external package.
#[derive(Debug)]
struct CoalescedNamedExternalImport {
    output_index: usize,
    default_binding: Option<String>,
    named_bindings: Vec<String>,
}

/// The original exported name behind one local binding. Separate source
/// modules may legitimately reuse one local alias for *different* exports;
/// a flat bundle cannot preserve that meaning without renaming every use, so
/// that case must be reported rather than silently selecting the first import.
#[derive(Debug)]
struct NamedExternalBindingOwner {
    entry_index: usize,
    imported_name: String,
}

/// All compatible named imports for one external package. `binding_owners`
/// gives each local binding one canonical emitted clause, preventing duplicate
/// ESM declarations even when the imports came from separate inlined modules.
#[derive(Debug, Default)]
struct NamedExternalImportGroup {
    entries: Vec<CoalescedNamedExternalImport>,
    binding_owners: HashMap<String, NamedExternalBindingOwner>,
}

/// Semantically coalesce compatible external named imports.
///
/// A library bundle hoists imports from every inlined module. The source
/// modules can legally each import a shared name, but their concatenation
/// cannot declare that local binding twice. This keeps one binding per package
/// and local name while preserving default-import aliases in separate clauses
/// when they cannot share one ESM import declaration. Namespace, side-effect,
/// and re-export statements are deliberately preserved verbatim.
fn coalesce_external_named_imports(imports: &[String]) -> Result<Vec<String>> {
    let mut output: Vec<Option<String>> = Vec::new();
    let mut groups: HashMap<String, NamedExternalImportGroup> = HashMap::new();

    for statement in imports {
        let Some(import) = parse_named_external_import(statement) else {
            output.push(Some(statement.clone()));
            continue;
        };

        let group = groups.entry(import.specifier.clone()).or_default();
        let target = if let Some(default_binding) = import.default_binding.as_deref() {
            if let Some(owner) = group.binding_owners.get(default_binding) {
                if owner.imported_name != "default" {
                    bail!(
                        "jet build --lib cannot safely hoist external import from `{}`: local binding `{}` refers to both `{}` and `default` in separate modules; rename one binding before bundling",
                        import.specifier,
                        default_binding,
                        owner.imported_name,
                    );
                }
                owner.entry_index
            } else if let Some(index) = group
                .entries
                .iter()
                .position(|entry| entry.default_binding.is_none())
            {
                group.entries[index].default_binding = Some(default_binding.to_string());
                group.binding_owners.insert(
                    default_binding.to_string(),
                    NamedExternalBindingOwner {
                        entry_index: index,
                        imported_name: "default".to_string(),
                    },
                );
                index
            } else {
                let index = group.entries.len();
                group.entries.push(CoalescedNamedExternalImport {
                    output_index: output.len(),
                    default_binding: Some(default_binding.to_string()),
                    named_bindings: Vec::new(),
                });
                group.binding_owners.insert(
                    default_binding.to_string(),
                    NamedExternalBindingOwner {
                        entry_index: index,
                        imported_name: "default".to_string(),
                    },
                );
                output.push(None);
                index
            }
        } else if group.entries.is_empty() {
            group.entries.push(CoalescedNamedExternalImport {
                output_index: output.len(),
                default_binding: None,
                named_bindings: Vec::new(),
            });
            output.push(None);
            0
        } else {
            0
        };

        for binding in import.named_bindings {
            let binding_key = named_import_binding_local_name(&binding)
                .unwrap_or_else(|| binding.trim().to_string());
            let imported_name =
                named_import_binding_imported_name(&binding).unwrap_or_else(|| binding_key.clone());
            if let Some(owner) = group.binding_owners.get(&binding_key) {
                if owner.imported_name != imported_name {
                    bail!(
                        "jet build --lib cannot safely hoist external import from `{}`: local binding `{}` refers to both `{}` and `{}` in separate modules; rename one binding before bundling",
                        import.specifier,
                        binding_key,
                        owner.imported_name,
                        imported_name,
                    );
                }
                continue;
            }
            group.entries[target].named_bindings.push(binding);
            group.binding_owners.insert(
                binding_key,
                NamedExternalBindingOwner {
                    entry_index: target,
                    imported_name,
                },
            );
        }
    }

    for (specifier, group) in groups {
        for entry in group.entries {
            if entry.default_binding.is_some() || !entry.named_bindings.is_empty() {
                output[entry.output_index] = Some(render_named_external_import(
                    &specifier,
                    entry.default_binding.as_deref(),
                    &entry.named_bindings,
                ));
            }
        }
    }

    let output = output.into_iter().flatten().collect::<Vec<_>>();
    validate_external_import_binding_collisions(&output)?;
    Ok(output)
}

#[derive(Debug)]
struct ExternalImportBinding {
    specifier: String,
    imported_name: String,
    local_name: String,
}

/// Library modules are flattened into one ESM scope. Reject duplicate local
/// bindings across every surviving import form (named, default, and namespace)
/// instead of producing a syntactically-invalid artifact or silently changing
/// a source module's imported value. Compatible named imports are coalesced
/// above; remaining collisions need a future alpha-renaming pass.
fn validate_external_import_binding_collisions(imports: &[String]) -> Result<()> {
    let mut owners: HashMap<String, ExternalImportBinding> = HashMap::new();
    for statement in imports {
        for binding in external_import_bindings(statement) {
            if let Some(previous) = owners.get(&binding.local_name) {
                bail!(
                    "jet build --lib cannot safely hoist external imports: local binding `{}` is declared by `{}` from `{}` and `{}` from `{}` in separate modules; rename one binding before bundling",
                    binding.local_name,
                    previous.imported_name,
                    previous.specifier,
                    binding.imported_name,
                    binding.specifier,
                );
            }
            owners.insert(binding.local_name.clone(), binding);
        }
    }
    Ok(())
}

fn external_import_bindings(statement: &str) -> Vec<ExternalImportBinding> {
    let Some(rest) = statement.trim().strip_prefix("import").map(str::trim_start) else {
        return Vec::new();
    };
    if rest.starts_with('\"') || rest.starts_with('\'') {
        return Vec::new();
    }
    let Some((clause, specifier)) = parse_import_from_clause(rest) else {
        return Vec::new();
    };
    let clause = clause.trim();
    if clause == "type" || clause.starts_with("type ") || clause.starts_with("type\t") {
        return Vec::new();
    }

    let mut bindings = Vec::new();
    let mut push = |imported_name: &str, local_name: &str| {
        if is_js_identifier(local_name) {
            bindings.push(ExternalImportBinding {
                specifier: specifier.clone(),
                imported_name: imported_name.to_string(),
                local_name: local_name.to_string(),
            });
        }
    };

    if let Some((default_binding, names)) = split_named_import_clause(clause) {
        if let Some(default_binding) = default_binding {
            push("default", default_binding);
        }
        for name in names.split(',').map(str::trim) {
            if name.is_empty() || is_inline_type_import_binding(name) {
                continue;
            }
            let Some(local_name) = named_import_binding_local_name(name) else {
                continue;
            };
            let Some(imported_name) = named_import_binding_imported_name(name) else {
                continue;
            };
            push(&imported_name, &local_name);
        }
        return bindings;
    }

    if let Some((default_binding, namespace)) = clause.split_once(", * as ") {
        let default_binding = default_binding.trim();
        let namespace = namespace.trim();
        push("default", default_binding);
        push("*", namespace);
        return bindings;
    }
    if let Some(namespace) = clause.strip_prefix("* as ").map(str::trim) {
        push("*", namespace);
        return bindings;
    }
    push("default", clause);
    bindings
}

fn parse_named_external_import(statement: &str) -> Option<NamedExternalImport> {
    let rest = statement.trim().strip_prefix("import")?.trim_start();
    let (clause, specifier) = parse_import_from_clause(rest)?;
    let clause = clause.trim();
    // `import type` must stay in the source until the TypeScript transform
    // erases it. Treating `type` as a default binding would turn a type-only
    // import into an invalid runtime default import during rendering.
    if clause == "type" || clause.starts_with("type ") || clause.starts_with("type\t") {
        return None;
    }
    let open = clause.find('{')?;
    let close = clause.rfind('}')?;
    if close <= open || !clause[close + 1..].trim().is_empty() {
        return None;
    }

    let default_prefix = clause[..open].trim().trim_end_matches(',').trim();
    let default_binding = if default_prefix.is_empty() {
        None
    } else if is_js_identifier(default_prefix) {
        Some(default_prefix.to_string())
    } else {
        return None;
    };

    let named_bindings = clause[open + 1..close]
        .split(',')
        .map(str::trim)
        .filter(|binding| !binding.is_empty() && !is_inline_type_import_binding(binding))
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if named_bindings.is_empty() {
        return None;
    }

    Some(NamedExternalImport {
        specifier,
        default_binding,
        named_bindings,
    })
}

fn is_inline_type_import_binding(binding: &str) -> bool {
    let binding = binding.trim();
    binding.starts_with("type ") || binding.starts_with("type\t")
}

fn named_import_binding_local_name(binding: &str) -> Option<String> {
    let binding = binding.trim();
    let local = binding
        .rsplit_once(" as ")
        .map(|(_, local)| local.trim())
        .unwrap_or_else(|| {
            binding
                .strip_prefix("type ")
                .or_else(|| binding.strip_prefix("type\t"))
                .unwrap_or(binding)
                .trim()
        });
    is_js_identifier(local).then(|| local.to_string())
}

fn named_import_binding_imported_name(binding: &str) -> Option<String> {
    let binding = binding
        .trim()
        .strip_prefix("type ")
        .or_else(|| binding.trim().strip_prefix("type\t"))
        .unwrap_or_else(|| binding.trim());
    let imported = binding
        .rsplit_once(" as ")
        .map(|(imported, _)| imported.trim())
        .unwrap_or(binding);
    is_js_identifier(imported).then(|| imported.to_string())
}

fn is_js_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first == '$' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}

fn render_named_external_import(
    specifier: &str,
    default_binding: Option<&str>,
    named_bindings: &[String],
) -> String {
    let mut out = String::from("import ");
    if let Some(default_binding) = default_binding {
        out.push_str(default_binding);
        if !named_bindings.is_empty() {
            out.push_str(", ");
        }
    }
    if !named_bindings.is_empty() {
        out.push_str("{ ");
        out.push_str(&named_bindings.join(", "));
        out.push_str(" }");
    }
    out.push_str(" from \"");
    out.push_str(specifier);
    out.push_str("\";");
    out
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
    used_top_level_bindings: &mut HashSet<String>,
    external_binding_owners: &mut HashMap<String, (String, String)>,
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
    // This library path flattens source modules directly instead of routing
    // through the app bundler's CJS scope-hoister. Give every module its own
    // binding namespace before splicing it into the shared ESM body: imports
    // such as `Form` from two packages and private locals such as `Panel`
    // otherwise become duplicate root declarations in the emitted file.
    let module_index = inlined_files.len() - 1;
    let (module_source, external_import_renames) = isolate_library_module_scope(
        &source,
        root,
        externals,
        module_index,
        used_top_level_bindings,
        external_binding_owners,
    );

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
        let rewritten_start = module_source.output_offset_for_input_offset(last_end);
        let rewritten_end = module_source.output_offset_for_input_offset(stmt_start);
        out.push_str(&module_source.source()[rewritten_start..rewritten_end]);
        last_end = stmt_end;

        let stmt_text = &source[stmt_start..stmt_end];

        if is_external_specifier(&spec, externals) {
            // External `export ... from "pkg"` re-exports stay as their own
            // statement so the binding is re-exported from the package; the CJS
            // pass rewrites them to `exports.x = require("pkg").x`. Hoisting one
            // copy (deduplicated) is enough — do not also splice it into the
            // body, or the re-export would be emitted twice.
            let rewritten_specifier = rewrite_external_library_specifier_for_node(path, &spec);
            let renamed_statement =
                rename_external_import_local_bindings(stmt_text, &external_import_renames);
            let external_statement = if rewritten_specifier == spec {
                renamed_statement
            } else {
                rewrite_statement_specifier(&renamed_statement, &spec, &rewritten_specifier)
            };
            if seen_external.insert(external_statement.clone()) {
                external_imports.push(external_statement);
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
            if let Some(svg_reexport) =
                inline_svg_named_reexport(path, &spec, stmt_text, external_imports, seen_external)?
            {
                out.push_str(&svg_reexport);
            } else if resolve_relative_asset(path, &spec)
                .filter(|p| is_library_asset_path(p))
                .is_some()
            {
                out.push_str(stmt_text);
            } else if let Some(target) = resolve_relative(path, &spec)? {
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
                        used_top_level_bindings,
                        external_binding_owners,
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
                        used_top_level_bindings,
                        external_binding_owners,
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
            if let Some(asset_path) =
                resolve_relative_asset(path, &spec).filter(|p| is_library_asset_path(p))
            {
                if is_library_style_path(&asset_path) {
                    // Style side-effect imports are not JS modules. Library
                    // asset emission is handled by `[lib].css_merge` or the
                    // package's own published CSS exports, so do not inline or
                    // preserve a browser-unresolvable SCSS import here.
                } else {
                    let rewritten_start = module_source.output_offset_for_input_offset(stmt_start);
                    let rewritten_end = module_source.output_offset_for_input_offset(stmt_end);
                    out.push_str(&module_source.source()[rewritten_start..rewritten_end]);
                }
            } else if let Some(target) = resolve_relative(path, &spec)? {
                let inlined = inline_module(
                    &target,
                    externals,
                    external_imports,
                    seen_external,
                    inlined_files,
                    false,
                    used_top_level_bindings,
                    external_binding_owners,
                )?;
                out.push_str(&inlined);
            } else {
                // Unresolved relative import: keep verbatim rather than drop it.
                let rewritten_start = module_source.output_offset_for_input_offset(stmt_start);
                let rewritten_end = module_source.output_offset_for_input_offset(stmt_end);
                out.push_str(&module_source.source()[rewritten_start..rewritten_end]);
            }
        }
    }

    // Trailing text after the last handled statement.
    let rewritten_start = module_source.output_offset_for_input_offset(last_end);
    out.push_str(&module_source.source()[rewritten_start..]);

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

/// Isolate bindings private to one source module before library-mode inlining
/// flattens all modules into one ESM scope.
///
/// Runtime external imports are blanked while the scoped rename runs, then
/// emitted separately with local aliases rewritten. This preserves the
/// imported export name (`Form`) while rewriting this module's uses to its
/// unique local binding (`__jet_m1_Form`). JSX component aliases instead keep
/// an uppercase first character so the later JSX transform continues to emit
/// an identifier reference rather than an intrinsic tag string.
fn isolate_library_module_scope(
    source: &str,
    root: tree_sitter::Node<'_>,
    externals: &HashSet<String>,
    module_index: usize,
    used_top_level_bindings: &mut HashSet<String>,
    external_binding_owners: &mut HashMap<String, (String, String)>,
) -> (super::mangle::ScopedModuleRename, HashMap<String, String>) {
    let jsx_component_bindings = collect_library_jsx_component_bindings(source, root);
    let mut external_import_renames = HashMap::new();
    let mut external_ranges = Vec::new();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() != "import_statement" {
            continue;
        }
        let Some(specifier) = statement_specifier(source, &child) else {
            continue;
        };
        if !is_external_specifier(&specifier, externals) {
            continue;
        }
        let statement = &source[child.start_byte()..child.end_byte()];
        for binding in external_import_bindings(statement) {
            let local_name = binding.local_name;
            let owner = (binding.specifier, binding.imported_name);
            let same_external_owner = external_binding_owners
                .get(&local_name)
                .is_some_and(|previous| previous == &owner);
            if !same_external_owner
                && (external_binding_owners.contains_key(&local_name)
                    || used_top_level_bindings.contains(&local_name))
            {
                let alias = library_module_binding_alias(
                    module_index,
                    &local_name,
                    &jsx_component_bindings,
                );
                external_import_renames.entry(local_name).or_insert(alias);
            } else {
                external_binding_owners
                    .entry(local_name.clone())
                    .or_insert(owner);
                used_top_level_bindings.insert(local_name);
            }
        }
        external_ranges.push((child.start_byte(), child.end_byte()));
    }

    let mut private_bindings = collect_library_private_top_level_bindings(source, root);
    let exported_bindings = collect_library_exported_top_level_bindings(source, root);
    private_bindings.retain(|name| !exported_bindings.contains(name));
    let mut root_renames = HashMap::new();
    for name in private_bindings {
        used_top_level_bindings.insert(name.clone());
        root_renames.insert(
            name.clone(),
            library_module_binding_alias(module_index, &name, &jsx_component_bindings),
        );
    }
    for name in exported_bindings {
        if !used_top_level_bindings.insert(name.clone()) {
            root_renames.entry(name.clone()).or_insert_with(|| {
                library_module_binding_alias(module_index, &name, &jsx_component_bindings)
            });
        }
    }

    // Keep byte offsets stable while `inline_module` walks the tree parsed from
    // the original source to splice relative modules.
    let mut without_external_imports = source.as_bytes().to_vec();
    for (start, end) in external_ranges {
        for byte in &mut without_external_imports[start..end] {
            *byte = b' ';
        }
    }
    let without_external_imports = String::from_utf8(without_external_imports)
        .expect("replacing UTF-8 source bytes with ASCII spaces stays valid UTF-8");

    (
        super::mangle::apply_scoped_module_renames_with_offsets(
            &without_external_imports,
            &root_renames,
            &external_import_renames,
        ),
        external_import_renames,
    )
}

fn library_module_binding_alias(
    module_index: usize,
    name: &str,
    jsx_component_bindings: &HashSet<String>,
) -> String {
    if jsx_component_bindings.contains(name) {
        format!("JetM{module_index}_{name}")
    } else {
        format!("__jet_m{module_index}_{name}")
    }
}

fn collect_library_jsx_component_bindings(
    source: &str,
    root: tree_sitter::Node<'_>,
) -> HashSet<String> {
    fn visit(node: tree_sitter::Node<'_>, source: &str, names: &mut HashSet<String>) {
        if matches!(
            node.kind(),
            "jsx_opening_element" | "jsx_self_closing_element"
        ) {
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                if child.kind() != "identifier" {
                    continue;
                }
                let name = &source[child.start_byte()..child.end_byte()];
                if name.as_bytes().first().is_some_and(u8::is_ascii_uppercase)
                    && is_js_identifier(name)
                {
                    names.insert(name.to_string());
                }
                break;
            }
        }

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            visit(child, source, names);
        }
    }

    let mut names = HashSet::new();
    visit(root, source, &mut names);
    names
}

/// Public declarations retain their authored name so `export const A` still
/// exports `A`. Other top-level bindings are module-local and can be safely
/// namespaced before concatenation.
fn collect_library_private_top_level_bindings(
    source: &str,
    root: tree_sitter::Node<'_>,
) -> HashSet<String> {
    let mut names = HashSet::new();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if matches!(child.kind(), "import_statement" | "export_statement") {
            continue;
        }
        collect_library_declaration_bindings(source, child, &mut names);
    }
    names
}

/// Export declarations normally retain their authored public name. When two
/// inlined modules declare the same exported root binding, the later module
/// still needs a private alpha-name so concatenation cannot emit duplicate ESM
/// declarations. Bare `export { local as public }` clauses introduce no new
/// declaration, but their local side still marks an existing binding public so
/// it must not be isolated as a private module-local name.
fn collect_library_exported_top_level_bindings(
    source: &str,
    root: tree_sitter::Node<'_>,
) -> HashSet<String> {
    let mut names = HashSet::new();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() != "export_statement" {
            continue;
        }
        let mut export_cursor = child.walk();
        for exported in child.named_children(&mut export_cursor) {
            collect_library_declaration_bindings(source, exported, &mut names);
        }
        let statement = &source[child.start_byte()..child.end_byte()];
        if !statement.contains(" from ") {
            if let Some(clause) = export_named_clause(statement) {
                for binding in clause.split(',').map(str::trim) {
                    let local = binding
                        .strip_prefix("type ")
                        .unwrap_or(binding)
                        .split_once(" as ")
                        .map(|(local, _)| local.trim())
                        .unwrap_or(binding);
                    if is_js_identifier(local) {
                        names.insert(local.to_string());
                    }
                }
            }
        }
    }
    names
}

fn collect_library_declaration_bindings(
    source: &str,
    declaration: tree_sitter::Node<'_>,
    names: &mut HashSet<String>,
) {
    match declaration.kind() {
        "function_declaration" | "class_declaration" => {
            if let Some(name) = declaration.child_by_field_name("name") {
                collect_library_binding_pattern(source, name, names);
            }
        }
        "lexical_declaration" | "variable_declaration" => {
            let mut cursor = declaration.walk();
            for declarator in declaration.named_children(&mut cursor) {
                if declarator.kind() != "variable_declarator" {
                    continue;
                }
                if let Some(name) = declarator.child_by_field_name("name") {
                    collect_library_binding_pattern(source, name, names);
                }
            }
        }
        _ => {}
    }
}

fn collect_library_binding_pattern(
    source: &str,
    pattern: tree_sitter::Node<'_>,
    names: &mut HashSet<String>,
) {
    match pattern.kind() {
        "identifier" | "shorthand_property_identifier_pattern" => {
            let name = &source[pattern.start_byte()..pattern.end_byte()];
            if is_js_identifier(name) {
                names.insert(name.to_string());
            }
        }
        "pair_pattern" => {
            if let Some(value) = pattern.child_by_field_name("value") {
                collect_library_binding_pattern(source, value, names);
            }
        }
        "rest_pattern" => {
            if let Some(argument) = pattern.child_by_field_name("argument") {
                collect_library_binding_pattern(source, argument, names);
            }
        }
        "assignment_pattern" => {
            if let Some(left) = pattern.child_by_field_name("left") {
                collect_library_binding_pattern(source, left, names);
            }
        }
        "object_pattern" | "array_pattern" => {
            let mut cursor = pattern.walk();
            for child in pattern.named_children(&mut cursor) {
                collect_library_binding_pattern(source, child, names);
            }
        }
        _ => {}
    }
}

/// Rewrite only the local side of an external import. `{ Form }` becomes
/// `{ Form as __jet_m1_Form }`; the package's exported symbol remains `Form`.
fn rename_external_import_local_bindings(
    statement: &str,
    renames: &HashMap<String, String>,
) -> String {
    if renames.is_empty() {
        return statement.to_string();
    }
    let Some(rest) = statement.trim().strip_prefix("import").map(str::trim_start) else {
        return statement.to_string();
    };
    let Some((clause, specifier)) = parse_import_from_clause(rest) else {
        return statement.to_string();
    };
    let clause = clause.trim();
    if clause.starts_with("type ") || clause == "type" {
        return statement.to_string();
    }

    let renamed_clause =
        if let Some((default_binding, named_bindings)) = split_named_import_clause(clause) {
            let default_binding = default_binding.map(|binding| {
                renames
                    .get(binding)
                    .cloned()
                    .unwrap_or_else(|| binding.to_string())
            });
            let named_bindings = named_bindings
                .split(',')
                .map(str::trim)
                .filter(|binding| !binding.is_empty())
                .map(|binding| rename_named_import_binding(binding, renames))
                .collect::<Vec<_>>();
            let mut rendered = default_binding.unwrap_or_default();
            if !rendered.is_empty() && !named_bindings.is_empty() {
                rendered.push_str(", ");
            }
            if !named_bindings.is_empty() {
                rendered.push_str("{ ");
                rendered.push_str(&named_bindings.join(", "));
                rendered.push_str(" }");
            }
            rendered
        } else if let Some(namespace) = clause.strip_prefix("* as ").map(str::trim) {
            let local = renames
                .get(namespace)
                .cloned()
                .unwrap_or_else(|| namespace.to_string());
            format!("* as {local}")
        } else {
            renames
                .get(clause)
                .cloned()
                .unwrap_or_else(|| clause.to_string())
        };

    format!("import {renamed_clause} from \"{specifier}\";")
}

fn rename_named_import_binding(binding: &str, renames: &HashMap<String, String>) -> String {
    if binding.starts_with("type ") {
        return binding.to_string();
    }
    let (imported, local) = binding
        .split_once(" as ")
        .map(|(imported, local)| (imported.trim(), local.trim()))
        .unwrap_or((binding, binding));
    let local = renames
        .get(local)
        .cloned()
        .unwrap_or_else(|| local.to_string());
    if imported == local {
        imported.to_string()
    } else {
        format!("{imported} as {local}")
    }
}

fn inline_svg_named_reexport(
    from: &Path,
    spec: &str,
    stmt_text: &str,
    external_imports: &mut Vec<String>,
    seen_external: &mut HashSet<String>,
) -> Result<Option<String>> {
    if !crate::bundler::imports::is_svg_specifier(spec) {
        return Ok(None);
    }
    let Some(asset_path) = resolve_relative_asset(from, spec) else {
        return Ok(None);
    };
    let aliases = svgr_reexport_public_aliases(stmt_text);
    if aliases.is_empty() {
        return Ok(None);
    }

    let react_import = "import * as React from \"react\";".to_string();
    if seen_external.insert(react_import.clone()) {
        external_imports.push(react_import);
    }

    let svg_src = std::fs::read_to_string(&asset_path)
        .with_context(|| format!("reading SVG re-export {}", asset_path.display()))?;
    let mut module =
        crate::asset::transform_svg_to_component(&svg_src, crate::asset::SvgrExportType::Named)
            .with_context(|| format!("transforming SVG re-export {}", asset_path.display()))?;

    let local_name = svg_component_local_name(&aliases[0], &asset_path);
    module = module.replace("import * as React from \"react\";\n\n", "");
    module = module.replace("const ReactComponent =", &format!("const {local_name} ="));
    module = module.replace("export { ReactComponent };\n", "");

    let reexports = aliases
        .iter()
        .map(|alias| format!("{local_name} as {alias}"))
        .collect::<Vec<_>>()
        .join(", ");
    Ok(Some(format!("{module}export {{ {reexports} }};\n")))
}

fn resolve_relative_asset(from: &Path, spec: &str) -> Option<PathBuf> {
    let parent = from.parent()?;
    let base = parent.join(specifier_path_part(spec).trim_start_matches("./"));
    base.is_file().then_some(base)
}

fn svgr_reexport_public_aliases(stmt_text: &str) -> Vec<String> {
    let Some(clause) = export_named_clause(stmt_text) else {
        return Vec::new();
    };
    clause
        .split(',')
        .filter_map(|binding| svgr_reexport_public_alias(binding.trim()))
        .collect()
}

fn svgr_reexport_public_alias(binding: &str) -> Option<String> {
    let mut parts = binding.split_whitespace();
    let first = parts.next()?;
    if first != "ReactComponent" {
        return None;
    }
    match (parts.next(), parts.next(), parts.next()) {
        (None, None, None) => Some(first.to_string()),
        (Some("as"), Some(alias), None) => Some(alias.to_string()),
        _ => None,
    }
}

fn svg_component_local_name(public_alias: &str, asset_path: &Path) -> String {
    let seed = if public_alias == "ReactComponent" {
        asset_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("Icon")
    } else {
        public_alias
    };
    format!("Svg{}", sanitize_identifier_part(seed))
}

fn sanitize_identifier_part(seed: &str) -> String {
    let mut out = String::new();
    let mut uppercase_next = true;
    for ch in seed.chars() {
        if ch.is_ascii_alphanumeric() {
            if uppercase_next {
                out.push(ch.to_ascii_uppercase());
                uppercase_next = false;
            } else {
                out.push(ch);
            }
        } else {
            uppercase_next = true;
        }
    }
    if out.is_empty() {
        "Icon".to_string()
    } else {
        out
    }
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
    let mut parser = tree_sitter::Parser::new();
    let language: tree_sitter::Language = tree_sitter_javascript::LANGUAGE.into();
    if parser.set_language(&language).is_err() {
        return esm_to_cjs_linewise(esm);
    }
    let Some(tree) = parser.parse(esm, None) else {
        return esm_to_cjs_linewise(esm);
    };

    let root = tree.root_node();
    let mut out = String::with_capacity(esm.len());
    let mut export_assignments = Vec::new();
    let mut cursor = root.walk();
    let mut last_end = 0usize;

    // Work statement-by-statement rather than line-by-line. The transform may
    // legitimately place `import …;import …;` on one line; treating that as a
    // single line used to make the first import consume the second one's
    // specifier and produce unloadable CJS.
    for child in root.children(&mut cursor) {
        let kind = child.kind();
        if kind != "import_statement" && kind != "export_statement" {
            continue;
        }

        let start = child.start_byte();
        let end = child.end_byte();
        out.push_str(&esm[last_end..start]);
        let original = &esm[start..end];
        let trimmed = original.trim();

        if let Some((rewritten, assignment)) = rewrite_cjs_export_declaration(trimmed, original) {
            out.push_str(&rewritten);
            export_assignments.push(assignment);
        } else if let Some(rewritten) = rewrite_cjs_line(trimmed) {
            out.push_str(&rewritten);
        } else {
            out.push_str(original);
        }
        last_end = end;
    }
    out.push_str(&esm[last_end..]);

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

/// Conservative fallback for an unexpected parser setup failure. Normal
/// library output uses the AST-aware path above; this keeps the legacy
/// best-effort behavior available rather than returning the original ESM.
fn esm_to_cjs_linewise(esm: &str) -> String {
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
    if let Some(rewritten) = rewrite_cjs_import(line) {
        return Some(rewritten);
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

fn rewrite_cjs_import(line: &str) -> Option<String> {
    let rest = line.trim_start().strip_prefix("import ")?.trim_start();
    // `import "pkg";`
    if rest.starts_with('"') || rest.starts_with('\'') {
        let spec = import_spec(rest)?;
        return Some(format!("require(\"{spec}\");"));
    }

    let (clause, spec) = parse_import_from_clause(rest)?;
    let clause = clause.trim();

    // `import Default, { named } from "pkg"` lowers to two valid CJS
    // declarations. The old generic default branch emitted
    // `const Default, { named } = require(...)`, which is invalid JavaScript.
    if let Some((default_binding, names)) = split_named_import_clause(clause) {
        let mut declarations = Vec::new();
        if let Some(default_binding) = default_binding {
            declarations.push(format!("const {default_binding} = require(\"{spec}\");"));
        }
        if !names.trim().is_empty() {
            let names = cjs_object_destructure_bindings(names);
            declarations.push(format!("const {{ {names} }} = require(\"{spec}\");"));
        }
        return (!declarations.is_empty()).then(|| declarations.join("\n"));
    }

    // `import Default, * as Namespace from "pkg"` is another mixed form
    // that cannot be represented by one `const` declaration.
    if let Some((default_binding, namespace)) = clause.split_once(", * as ") {
        let default_binding = default_binding.trim();
        let namespace = namespace.trim();
        if is_js_identifier(default_binding) && is_js_identifier(namespace) {
            return Some(format!(
                "const {default_binding} = require(\"{spec}\");\nconst {namespace} = require(\"{spec}\");"
            ));
        }
    }

    if let Some(namespace) = clause.strip_prefix("* as ").map(str::trim) {
        return is_js_identifier(namespace)
            .then(|| format!("const {namespace} = require(\"{spec}\");"));
    }
    is_js_identifier(clause).then(|| format!("const {clause} = require(\"{spec}\");"))
}

/// ESM uses `imported as local` inside named import clauses, while JavaScript
/// object destructuring needs `imported: local`. Keep unaliased names intact
/// so the CJS/IIFE lowerings accept the same merged external import clauses as
/// the ESM artifact.
fn cjs_object_destructure_bindings(names: &str) -> String {
    names
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(|name| {
            name.rsplit_once(" as ")
                .map(|(imported, local)| format!("{}: {}", imported.trim(), local.trim()))
                .unwrap_or_else(|| name.to_string())
        })
        .collect::<Vec<_>>()
        .join(", ")
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

/// Split an import clause from its static module specifier. This accepts
/// multiline whitespace and deliberately ignores `from` identifiers inside a
/// named-import list, unlike a simple `split_once(" from ")`.
fn parse_import_from_clause(rest: &str) -> Option<(&str, String)> {
    let from_index = find_import_from_keyword(rest)?;
    let clause = rest[..from_index].trim_end();
    let spec = import_spec(&rest[from_index + "from".len()..])?;
    Some((clause, spec))
}

fn find_import_from_keyword(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut index = 0usize;
    let mut brace_depth = 0usize;
    let mut quote = None;

    while index < bytes.len() {
        if let Some(quote_char) = quote {
            if bytes[index] == b'\\' {
                index += 2;
                continue;
            }
            if bytes[index] == quote_char {
                quote = None;
            }
            index += 1;
            continue;
        }

        match bytes[index] {
            b'\'' | b'\"' | b'`' => quote = Some(bytes[index]),
            b'{' => brace_depth += 1,
            b'}' => brace_depth = brace_depth.saturating_sub(1),
            b'f' if brace_depth == 0
                && bytes[index..].starts_with(b"from")
                && index > 0
                && bytes[index - 1].is_ascii_whitespace()
                && bytes
                    .get(index + "from".len())
                    .is_some_and(u8::is_ascii_whitespace) =>
            {
                return Some(index);
            }
            _ => {}
        }
        index += 1;
    }
    None
}

/// Split `Default, { named }` or `{ named }` into its optional default binding
/// and braced named clause. Namespace imports intentionally return `None`.
fn split_named_import_clause(clause: &str) -> Option<(Option<&str>, &str)> {
    let open = clause.find('{')?;
    let close = clause.rfind('}')?;
    if close <= open || !clause[close + 1..].trim().is_empty() {
        return None;
    }
    let default_prefix = clause[..open].trim().trim_end_matches(',').trim();
    let default_binding = if default_prefix.is_empty() {
        None
    } else if is_js_identifier(default_prefix) {
        Some(default_prefix)
    } else {
        return None;
    };
    Some((default_binding, clause[open + 1..close].trim()))
}

/// Helper: extract a quoted specifier from the tail of an import, e.g.
/// ` from "pkg";` or `"pkg";`.
fn import_spec(tail: &str) -> Option<String> {
    let tail = tail.trim();
    let tail = tail.strip_prefix("from").unwrap_or(tail).trim();
    let quote = tail.chars().next()?;
    if quote != '\'' && quote != '\"' && quote != '`' {
        return None;
    }
    let after_open = &tail[quote.len_utf8()..];
    let end = after_open.find(quote)?;
    if !after_open[end + quote.len_utf8()..]
        .trim()
        .trim_end_matches(';')
        .trim()
        .is_empty()
    {
        return None;
    }
    let spec = &after_open[..end];
    (!spec.is_empty()).then(|| spec.to_string())
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
    fn library_output_parse_gate_rejects_invalid_javascript() {
        let err = ensure_library_output_parses("const incomplete = ;", "index.js")
            .expect_err("invalid JavaScript must not be emitted as a successful library artifact");
        assert!(
            err.to_string().contains("refusing to write"),
            "parse gate error must explain the refused output: {err}"
        );
    }

    #[test]
    fn library_export_binding_collector_finds_exported_functions() {
        let source = "export function getSelectedNode() { return 1; }\nexport const value = 2;\n";
        let mut parser = tree_sitter::Parser::new();
        let language: tree_sitter::Language = tree_sitter_typescript::LANGUAGE_TSX.into();
        parser.set_language(&language).unwrap();
        let tree = parser.parse(source, None).unwrap();
        let names = collect_library_exported_top_level_bindings(source, tree.root_node());
        assert!(names.contains("getSelectedNode"), "{names:?}");
        assert!(names.contains("value"), "{names:?}");
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
    fn iife_import_lowering_converts_esm_aliases_to_object_patterns() {
        let externals = HashSet::from(["react".to_string()]);
        assert_eq!(
            rewrite_iife_import(
                "import { forwardRef as render, useMemo } from \"react\";",
                &externals,
            ),
            Some("const { forwardRef: render, useMemo } = globalThis.react;".to_string())
        );
    }

    #[test]
    fn legacy_node_subpath_rewrite_keeps_export_mapped_packages_authored() {
        let tmp = tempfile::tempdir().unwrap();
        let package = tmp.path().join("node_modules/export-mapped-package");
        let importer = tmp.path().join("src/index.js");
        std::fs::create_dir_all(&package).unwrap();
        std::fs::create_dir_all(importer.parent().unwrap()).unwrap();
        std::fs::write(
            package.join("package.json"),
            r#"{"name":"export-mapped-package","exports":{".":"./index.js"}}"#,
        )
        .unwrap();
        std::fs::write(package.join("index.js"), "export const root = true;\n").unwrap();
        std::fs::write(package.join("chunk.js"), "export const chunk = true;\n").unwrap();
        std::fs::write(&importer, "export {};\n").unwrap();

        assert_eq!(
            rewrite_external_library_specifier_for_node(&importer, "export-mapped-package/chunk"),
            "export-mapped-package/chunk",
            "an exports map owns package subpaths even when a physical fallback exists"
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
    fn library_bundle_isolates_external_and_private_module_bindings() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        let entry = src.join("index.ts");
        std::fs::write(&entry, "export * from \"./a\";\nexport * from \"./b\";\n").unwrap();
        std::fs::write(
            src.join("a.ts"),
            "import { Form, Collapse } from \"antd\";\n\
             const { Panel } = Collapse;\n\
             export function getSelectedNode() { return Panel; }\n\
             export const A = () => [Form, Panel];\n",
        )
        .unwrap();
        std::fs::write(
            src.join("b.ts"),
            "import { Form } from \"formik\";\n\
             import { Collapse } from \"antd\";\n\
             const { Panel } = Collapse;\n\
             export function getSelectedNode() { return Panel; }\n\
             export const B = () => [Form, Panel];\n",
        )
        .unwrap();

        let externals = HashSet::from(["antd".to_string(), "formik".to_string()]);
        let esm = bundle_library_entry(&entry, &externals).unwrap();
        assert!(
            crate::bundler::dce::js_parses_without_errors(&esm),
            "ESM output must remain syntactically valid:\n{esm}"
        );
        assert!(esm.contains("Form"), "{esm}");
        assert!(esm.contains("__jet_m2_Form"), "{esm}");
        assert!(esm.contains("__jet_m1_Panel"), "{esm}");
        assert!(esm.contains("__jet_m2_Panel"), "{esm}");
        assert!(esm.contains("__jet_m2_getSelectedNode"), "{esm}");

        let cjs = esm_to_cjs(&esm);
        assert!(
            crate::bundler::dce::js_parses_without_errors(&cjs),
            "CJS output must remain syntactically valid:\n{cjs}"
        );
    }

    #[test]
    fn library_bundle_preserves_jsx_component_and_attribute_bindings_during_isolation() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        let entry = src.join("index.ts");
        std::fs::write(
            &entry,
            "export * from \"./first\";\nexport * from \"./second\";\n",
        )
        .unwrap();
        std::fs::write(
            src.join("first.tsx"),
            "import { AutoLinkPlugin } from \"first-plugin\";\n\
             const MATCHERS = [\"first\"];\n\
             export function FirstView() { return <AutoLinkPlugin matchers={MATCHERS} />; }\n",
        )
        .unwrap();
        std::fs::write(
            src.join("second.tsx"),
            "import { AutoLinkPlugin } from \"second-plugin\";\n\
             const MATCHERS = [\"second\"];\n\
             export function SecondView() { return <AutoLinkPlugin matchers={MATCHERS} />; }\n",
        )
        .unwrap();

        let esm = bundle_library_entry(&entry, &HashSet::new()).unwrap();
        assert!(
            crate::bundler::dce::js_parses_without_errors(&esm),
            "JSX library output must remain syntactically valid:\n{esm}"
        );
        assert!(
            esm.contains("jsx(AutoLinkPlugin"),
            "an isolated JSX component must remain an identifier reference: {esm}"
        );
        assert!(
            esm.contains("jsx(JetM2_AutoLinkPlugin"),
            "each isolated JSX component must retain its uppercase alias: {esm}"
        );
        assert!(
            !esm.contains("matchers:__jet_m1_MATCHERS:")
                && !esm.contains("matchers:__jet_m2_MATCHERS:"),
            "JSX attribute expressions must not become object shorthand: {esm}"
        );
    }

    #[test]
    fn library_bundle_keeps_statement_boundaries_after_scoped_renames() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        let entry = src.join("index.tsx");
        std::fs::write(
            &entry,
            "import { Widget } from \"widget-lib\";\n\
             const sharedProps = { label: \"ready\" };\n\
             export function RenderWidget() { return <Widget props={sharedProps} />; }\n\
             export * from \"./types\";\n",
        )
        .unwrap();
        std::fs::write(
            src.join("types.ts"),
            "export const typeMarker = \"type\";\n",
        )
        .unwrap();

        let esm = bundle_library_entry(&entry, &HashSet::new()).unwrap();
        assert!(
            crate::bundler::dce::js_parses_without_errors(&esm),
            "library output must remain valid after a renamed JSX module is followed by a re-export:\n{esm}"
        );
        assert!(
            esm.contains("jsx(Widget"),
            "the JSX component must remain an identifier reference: {esm}"
        );
        assert!(
            esm.contains("typeMarker"),
            "the following re-export must be inlined without truncation: {esm}"
        );
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

    #[test]
    fn external_named_imports_coalesce_by_package_and_local_binding() {
        let imports = vec![
            "import { forwardRef, useMemo } from \"react\";".to_string(),
            "import { forwardRef, useRef } from \"react\";".to_string(),
            "import * as React from \"react\";".to_string(),
        ];

        let out = coalesce_external_named_imports(&imports).unwrap();
        assert_eq!(
            out,
            vec![
                "import { forwardRef, useMemo, useRef } from \"react\";",
                "import * as React from \"react\";",
            ],
            "only named clauses may merge; namespace bindings stay separate"
        );
    }

    #[test]
    fn external_named_imports_preserve_default_binding_while_merging_names() {
        let imports = vec![
            "import React, { forwardRef } from \"react\";".to_string(),
            "import { forwardRef, useMemo } from \"react\";".to_string(),
        ];

        let out = coalesce_external_named_imports(&imports).unwrap();
        assert_eq!(
            out,
            vec!["import React, { forwardRef, useMemo } from \"react\";"],
            "a compatible default clause remains the canonical declaration"
        );
    }

    #[test]
    fn external_named_imports_reject_conflicting_local_aliases() {
        let imports = vec![
            "import { foo as shared } from \"pkg\";".to_string(),
            "import { bar as shared } from \"pkg\";".to_string(),
        ];

        let err = coalesce_external_named_imports(&imports)
            .expect_err("a flat bundle cannot silently retarget a conflicting local alias");
        let message = err.to_string();
        assert!(
            message.contains("cannot safely hoist external import"),
            "{message}"
        );
        assert!(
            message.contains("foo") && message.contains("bar"),
            "{message}"
        );
        assert!(message.contains("shared"), "{message}");
    }

    #[test]
    fn external_imports_reject_conflicting_aliases_from_different_packages() {
        let imports = vec![
            "import { foo as shared } from \"first-pkg\";".to_string(),
            "import { bar as shared } from \"second-pkg\";".to_string(),
        ];

        let err = coalesce_external_named_imports(&imports)
            .expect_err("cross-package aliases would otherwise make invalid ESM");
        let message = err.to_string();
        assert!(
            message.contains("first-pkg") && message.contains("second-pkg"),
            "{message}"
        );
        assert!(message.contains("shared"), "{message}");
    }

    #[test]
    fn external_imports_reject_duplicate_default_bindings_left_unmerged() {
        let imports = vec![
            "import React from \"react\";".to_string(),
            "import React, { useMemo } from \"react\";".to_string(),
        ];

        let err = coalesce_external_named_imports(&imports)
            .expect_err("duplicate default declarations would make invalid ESM");
        assert!(err.to_string().contains("local binding `React`"), "{err}");
    }

    #[test]
    fn external_named_imports_leave_type_only_statements_for_ts_stripping() {
        let imports = vec![
            "import type { Shape } from \"pkg\";".to_string(),
            "import { Shape } from \"pkg\";".to_string(),
        ];
        let hoisted = coalesce_external_named_imports(&imports).unwrap();
        let transpiled =
            transpile_library_esm(&format!("{}\nconsole.log(Shape);\n", hoisted.join("\n")))
                .unwrap();

        assert!(
            !transpiled.contains("import type"),
            "type-only imports must be erased instead of becoming default imports: {transpiled}"
        );
        assert!(
            transpiled.contains("import { Shape } from \"pkg\";"),
            "the runtime value import must survive type stripping: {transpiled}"
        );
    }

    #[test]
    fn cjs_rewrite_handles_semicolon_adjacent_import_statements() {
        let out = esm_to_cjs(
            "import { forwardRef as render, useMemo } from \"react\";import { useRef } from \"react\";\n\
             export const value = render(useMemo(useRef(1)));\n",
        );
        assert!(
            out.contains("const { forwardRef: render, useMemo } = require(\"react\");"),
            "aliased first same-line import must lower to valid object destructuring, got:\n{out}"
        );
        assert!(
            out.contains("const { useRef } = require(\"react\");"),
            "second same-line import must lower independently, got:\n{out}"
        );
        assert!(
            !out.contains("import {"),
            "no ESM import may leak into CJS, got:\n{out}"
        );
        assert!(out.contains("exports.value = value;"), "{out}");
    }
}
// </HANDWRITE>
