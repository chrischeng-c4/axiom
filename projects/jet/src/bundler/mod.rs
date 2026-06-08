// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
use anyhow::Result;
use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::css::{CssPipeline, TailwindConfig};

pub mod css_bundle;
pub mod dce;
pub mod define;
pub mod fold;
pub mod graph;
pub mod html_minify;
pub mod imports;
pub mod json_shake;
pub mod mangle;
pub mod minify;
pub mod scope_hoist;
pub mod scope_hoist_opt;
pub mod sourcemap;
pub mod splitting;
pub mod tree_shake;
pub mod types;

pub use graph::{EdgeKind, ModuleGraph, ModuleNode};
pub use imports::{ImportDeclaration, ImportKind, ModuleImports};
pub use splitting::SplitResult;
pub use types::{BundleOptions, BundleOutput, ModuleId, PreloadHint};

/// Determine module kind from file extension
/// GH #3821 — fallback extension string used when a resolved-module
/// path has no extension at all (e.g., barrel module id with no
/// suffix). Kept as a named constant so call sites and tests pin the
/// same value.
pub(crate) const BUNDLER_EDGE_KIND_NO_EXTENSION_FALLBACK: &str = "";

/// GH #3821 — warn shown when the bundler module-graph edge-kind
/// classifier sees a resolved-module path with no `extension()`. The
/// prior code silently dropped to `""` and classified the dependency
/// as a plain `EdgeKind::Import`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub(crate) fn format_bundler_edge_kind_no_extension_warn(path: &std::path::Path) -> String {
    format!(
        "gh3821: jet bundler edge-kind classifier saw resolved-module path with no extension path={:?}; \
         falling back to empty extension — dependency will be classified as a plain JS Import edge. \
         If this module is a CSS/SCSS/SASS/LESS/WASM asset, the bundler will try to parse it as JavaScript \
         downstream and emit a confusing parse error.",
        path
    )
}

/// GH #3821 — warn shown when the bundler module-graph edge-kind
/// classifier sees a resolved-module path whose extension is non-UTF-8.
/// The prior code silently dropped to `""` and classified the
/// dependency as a plain `EdgeKind::Import`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub(crate) fn format_bundler_edge_kind_non_utf8_extension_warn(
    path: &std::path::Path,
    lossy: &str,
) -> String {
    format!(
        "gh3821: jet bundler edge-kind classifier saw resolved-module path with non-UTF-8 extension path={:?}; \
         lossy form is {:?}; routing through the lossy form so the classifier can attempt a match \
         instead of collapsing onto a plain JS Import edge",
        path, lossy
    )
}

/// GH #3821 — coerce a resolved-module path's extension into a string
/// for the bundler module-graph edge-kind classifier. Three-way branch:
/// - `Some(utf8)` → silent `Cow::Borrowed(utf8)`
/// - `Some(non-UTF-8)` → gh3821 warn + `Cow::Owned(lossy)`
/// - `None` → gh3821 warn + `Cow::Borrowed("")`
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub(crate) fn coerce_bundler_edge_kind_extension_or_warn(
    path: &std::path::Path,
) -> std::borrow::Cow<'_, str> {
    use std::borrow::Cow;
    match path.extension() {
        None => {
            tracing::warn!(
                target: "jet::bundler",
                path = %path.display(),
                "{}",
                format_bundler_edge_kind_no_extension_warn(path)
            );
            Cow::Borrowed(BUNDLER_EDGE_KIND_NO_EXTENSION_FALLBACK)
        }
        Some(os) => match os.to_str() {
            Some(s) => Cow::Borrowed(s),
            None => {
                let lossy = os.to_string_lossy().into_owned();
                tracing::warn!(
                    target: "jet::bundler",
                    path = %path.display(),
                    lossy = %lossy,
                    "{}",
                    format_bundler_edge_kind_non_utf8_extension_warn(path, &lossy)
                );
                Cow::Owned(lossy)
            }
        },
    }
}

fn determine_module_kind(path: &PathBuf) -> graph::ModuleKind {
    match path.extension().and_then(|e| e.to_str()) {
        Some("css") | Some("scss") | Some("sass") | Some("less") => graph::ModuleKind::Css,
        Some("json") => graph::ModuleKind::Json,
        Some("wasm") => graph::ModuleKind::Wasm,
        Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("svg") | Some("webp") => {
            graph::ModuleKind::Asset
        }
        Some("woff") | Some("woff2") | Some("ttf") | Some("eot") => graph::ModuleKind::Asset,
        _ => graph::ModuleKind::Script,
    }
}

/// Calculate simple hash of content
fn calculate_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Generate WASM glue code that fetches and instantiates a .wasm module
fn generate_wasm_glue(wasm_path: &str) -> String {
    format!(
        r#"// WASM module glue: {path}
var __wasm_module;
var __wasm_instance;

async function __wasm_init(input) {{
  var importObject = {{ env: {{}} }};
  if (typeof input === 'undefined') {{
    input = '{path}';
  }}
  if (typeof input === 'string') {{
    var response = await fetch(input);
    var bytes = await response.arrayBuffer();
    var result = await WebAssembly.instantiate(bytes, importObject);
    __wasm_module = result.module;
    __wasm_instance = result.instance;
  }} else {{
    var result = await WebAssembly.instantiate(input, importObject);
    __wasm_module = result.module;
    __wasm_instance = result.instance;
  }}
  return __wasm_instance.exports;
}}

module.exports = __wasm_init;
module.exports.default = __wasm_init;
"#,
        path = wasm_path
    )
}

/// Generate runtime module system code
fn generate_runtime() -> String {
    r#"// Jet Module Runtime
(function() {
  'use strict';

  var modules = {};
  var cache = {};

  // Module definition
  function define(id, factory) {
    modules[id] = factory;
  }

  // Module require
  function require(id) {
    // Return cached module if exists
    if (cache[id]) {
      return cache[id].exports;
    }

    // Create module object
    var module = cache[id] = {
      exports: {},
      id: id,
      loaded: false
    };

    // Execute module factory
    var factory = modules[id];
    if (!factory) {
      throw new Error('Module not found: ' + id);
    }

    factory.call(module.exports, require, module, module.exports);
    module.loaded = true;

    return module.exports;
  }

  // Expose global runtime
  window.__jet__ = {
    define: define,
    require: require,
    modules: modules,
    cache: cache
  };
})();
"#
    .to_string()
}

/// Generate `<link rel="modulepreload">` tags from preload hints.
///
/// Returns HTML tags suitable for injection into `<head>`. Only static
/// dependencies are included; dynamic imports are excluded since they
/// load on demand.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn generate_preload_tags(hints: &[PreloadHint]) -> String {
    let mut tags = String::new();
    for hint in hints {
        if hint.is_static {
            tags.push_str(&format!(
                "<link rel=\"modulepreload\" href=\"{}\">\n",
                hint.href
            ));
        }
    }
    tags
}

/// Inject preload hint tags into an HTML string's `<head>` section.
///
/// If `<head>` is found, the tags are inserted right after it.
/// Otherwise the tags are prepended to the HTML.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn inject_preload_hints(html: &str, hints: &[PreloadHint]) -> String {
    let tags = generate_preload_tags(hints);
    if tags.is_empty() {
        return html.to_string();
    }

    // Try to insert after <head> (case-insensitive search)
    let lower = html.to_lowercase();
    if let Some(pos) = lower.find("<head>") {
        let insert_pos = pos + "<head>".len();
        let mut result = String::with_capacity(html.len() + tags.len() + 1);
        result.push_str(&html[..insert_pos]);
        result.push('\n');
        result.push_str(&tags);
        result.push_str(&html[insert_pos..]);
        result
    } else {
        format!("{}{}", tags, html)
    }
}

/// A bare-specifier import that the resolver could not find on disk
/// and that the user did not explicitly mark as external.
///
/// @spec projects/jet/docs/build-fails-loudly-on-unresolved-bare-specifiers.md
/// @issue #1317
#[derive(Debug, Clone)]
struct UnresolvedDependency {
    specifier: String,
    importer: PathBuf,
    reason: String,
}

/// Core bundler that orchestrates the build process
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub struct Bundler {
    resolver: Arc<crate::resolver::ModuleResolver>,
    transformer: Arc<crate::transform::Transformer>,
    #[allow(dead_code)]
    asset_processor: Arc<crate::asset::AssetProcessor>,
    graph: Arc<RwLock<ModuleGraph>>,
    cache: Arc<CompilationCache>,
    /// Collected during `build_graph`; drained into a typed error from
    /// `bundle()` if non-empty so the build exits non-zero instead of
    /// silently shipping invalid JS.
    ///
    /// @spec projects/jet/docs/build-fails-loudly-on-unresolved-bare-specifiers.md
    /// @issue #1317
    unresolved_deps: Mutex<Vec<UnresolvedDependency>>,
    /// When true, use Phase 2 flat bundle in `generate_bundle`.
    ///
    /// Phase 2 (`generate_flattened_bundle`) merges all module bodies into a
    /// single flat scope with collision-avoiding `_m{n}_` prefixes.  The
    /// post-processing `mangle_variables_with_root` pass then compresses all
    /// prefixed names to 1-2 byte identifiers, yielding Webpack-level bundle
    /// size (≤ 196 KB for react-bench vs 215 KB with Phase 1 IIFE wrappers).
    minify: bool,
    /// Compile-time define map applied to every transformed module.
    ///
    /// Entries map expression strings to their replacement values, e.g.
    /// `import.meta.env.MODE → "\"production\""`.  Applied via
    /// `define::replace_defines` after the transform step so that the bundler
    /// can eliminate dead code branches at build time.
    defines: HashMap<String, String>,
}

/// Compilation cache for incremental builds
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub struct CompilationCache {
    module_cache: DashMap<(PathBuf, u64), CompiledModule>,
}

/// Compiled module with metadata
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone)]
pub struct CompiledModule {
    pub path: PathBuf,
    pub code: String,
    pub source_map: Option<String>,
    pub dependencies: Vec<String>,
    pub hash: String,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
impl Bundler {
    /// Create a new bundler instance
    pub fn new(options: BundleOptions) -> Result<Self> {
        let minify = options.minify;
        let defines = options.defines.clone();
        let mut resolve_options = options.resolve_options;
        // Forward externalize_all_packages to the resolver
        if options.externalize_all_packages {
            resolve_options.externalize_all_packages = true;
        }
        // Forward explicit externals list
        for ext in &options.externals {
            resolve_options.externals.insert(ext.clone());
        }
        Ok(Self {
            resolver: Arc::new(crate::resolver::ModuleResolver::new(resolve_options)?),
            transformer: Arc::new(crate::transform::Transformer::new(
                options.transform_options,
            )),
            asset_processor: Arc::new(crate::asset::AssetProcessor::new(options.asset_options)),
            graph: Arc::new(RwLock::new(ModuleGraph::new())),
            cache: Arc::new(CompilationCache::new()),
            minify,
            defines,
            unresolved_deps: Mutex::new(Vec::new()),
        })
    }

    /// Create a bundler with explicit compile-time defines.
    ///
    /// Convenience constructor for callers that load `import.meta.env` defines
    /// from `.env` files before constructing the bundler.
    pub fn with_defines(options: BundleOptions, defines: HashMap<String, String>) -> Result<Self> {
        let mut bundler = Self::new(options)?;
        bundler.defines.extend(defines);
        Ok(bundler)
    }

    /// Bundle the application starting from entry point
    pub async fn bundle(&self, entry: PathBuf) -> Result<BundleOutput> {
        tracing::info!("Starting bundle from entry: {:?}", entry);

        self.build_graph(&entry).await?;
        self.check_unresolved_deps()?;
        let (modules, has_cycle) = self.transform_modules().await?;

        // Tree shaking: analyze used exports across the module graph, then
        // remove unused export declarations from each module.  Modules with
        // no used exports and no side effects are eliminated entirely.
        let modules = self.apply_tree_shaking(modules);

        let mut output = self.generate_bundle(modules, has_cycle)?;

        // Detect sibling CSS entry file and run it through the CSS pipeline.
        // Convention: if entry is `src/index.tsx`, look for `src/index.css`.
        if let Some(css_asset) = self.try_process_css_entry(&entry) {
            output.assets.push(css_asset);
        }

        Ok(output)
    }

    /// Look for a CSS entry file alongside the JS entry and process it.
    ///
    /// Returns `None` if no CSS entry file is found, or if CSS processing fails
    /// (warnings are logged instead of propagating).
    fn try_process_css_entry(&self, js_entry: &PathBuf) -> Option<types::Asset> {
        let stem = js_entry.file_stem()?.to_string_lossy().into_owned();
        let dir = js_entry.parent()?;
        let css_entry = dir.join(format!("{}.css", stem));

        if !css_entry.exists() {
            return None;
        }

        tracing::info!("CSS entry detected: {:?}", css_entry);

        let root = dir.to_path_buf();
        // GH #3086 — surface tailwind.config.js / [css.tailwind] parse errors
        // instead of silently falling back to defaults during production builds.
        let config = match TailwindConfig::load(&root) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("[jet build] Failed to parse Tailwind config: {e:#}");
                eprintln!("[jet build] Continuing with built-in Tailwind defaults; your tailwind.config.js / [css.tailwind] settings will NOT take effect until the parse error is fixed.");
                TailwindConfig::default()
            }
        };
        let pipeline = CssPipeline::new(root, config, self.minify);

        match pipeline.process(&css_entry) {
            Ok(output) => {
                let filename = format!("{}.{}.css", stem, output.hash);
                tracing::info!(
                    "CSS pipeline produced: {} ({} bytes)",
                    filename,
                    output.css.len()
                );
                Some(types::Asset {
                    filename,
                    content: output.css.into_bytes(),
                    asset_type: types::AssetType::Css,
                })
            }
            Err(e) => {
                tracing::warn!("CSS pipeline failed for {:?}: {}", css_entry, e);
                None
            }
        }
    }

    /// Build the module dependency graph using iterative approach
    async fn build_graph(&self, entry: &PathBuf) -> Result<()> {
        tracing::debug!("Building module graph from: {:?}", entry);

        let entry_abs = std::fs::canonicalize(entry)?;

        let mut queue: Vec<(PathBuf, Option<ModuleId>, Option<graph::EdgeKind>)> =
            vec![(entry_abs, None, None)];
        let mut visited = std::collections::HashSet::new();

        while let Some((module_path, parent_id, edge_kind)) = queue.pop() {
            if visited.contains(&module_path) {
                if let (Some(parent), Some(kind)) = (parent_id, edge_kind) {
                    let graph = self.graph.read();
                    if let Some(module_id) = graph.get_module(&module_path) {
                        drop(graph);
                        let mut graph = self.graph.write();
                        graph.add_dependency(parent, module_id, kind);
                    }
                }
                continue;
            }

            visited.insert(module_path.clone());

            tracing::debug!("Processing module: {:?}", module_path);

            let source = match std::fs::read_to_string(&module_path) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!("Failed to read module {:?}: {}", module_path, e);
                    continue;
                }
            };

            let file_size = source.len() as u64;
            let module_kind = determine_module_kind(&module_path);

            let module_id = {
                let mut graph = self.graph.write();
                graph.add_module(module_path.clone(), module_kind, file_size)
            };

            if let (Some(parent), Some(kind)) = (parent_id, edge_kind) {
                let mut graph = self.graph.write();
                graph.add_dependency(parent, module_id, kind);
            }

            if module_kind == graph::ModuleKind::Script {
                let is_typescript = module_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e == "ts" || e == "tsx")
                    .unwrap_or(false);

                let module_imports = match imports::extract_imports(&source, is_typescript) {
                    Ok(imports) => imports,
                    Err(e) => {
                        tracing::warn!("Failed to extract imports from {:?}: {}", module_path, e);
                        continue;
                    }
                };

                // For TSX/JSX files with automatic runtime, add react/jsx-runtime as implicit dependency
                let is_jsx = module_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e == "tsx" || e == "jsx")
                    .unwrap_or(false);

                if is_jsx {
                    match self.resolve_dependency(&module_path, "react/jsx-runtime") {
                        Ok(resolved_path) => {
                            queue.push((
                                resolved_path,
                                Some(module_id),
                                Some(graph::EdgeKind::Import),
                            ));
                        }
                        Err(e) => {
                            let err_msg = e.to_string();
                            if !err_msg.contains("External module") {
                                tracing::warn!("Failed to resolve 'react/jsx-runtime': {}", e);
                                self.record_unresolved("react/jsx-runtime", &module_path, &err_msg);
                            }
                        }
                    }
                }

                for import_decl in &module_imports.static_imports {
                    match self.resolve_dependency(&module_path, &import_decl.source) {
                        Ok(resolved_path) => {
                            let ext_cow =
                                coerce_bundler_edge_kind_extension_or_warn(&resolved_path);
                            let ext = ext_cow.as_ref();
                            let edge_kind = match ext {
                                "css" | "scss" | "sass" | "less" => graph::EdgeKind::CssImport,
                                "wasm" => graph::EdgeKind::WasmImport,
                                _ => graph::EdgeKind::Import,
                            };

                            queue.push((resolved_path, Some(module_id), Some(edge_kind)));
                        }
                        Err(e) => {
                            let err_msg = e.to_string();
                            if !err_msg.contains("External module") {
                                tracing::warn!(
                                    "Failed to resolve '{}' from {:?}: {}",
                                    import_decl.source,
                                    module_path,
                                    e
                                );
                                self.record_unresolved(&import_decl.source, &module_path, &err_msg);
                            } else {
                                tracing::debug!(
                                    "External module '{}' (not bundled)",
                                    import_decl.source
                                );
                            }
                        }
                    }
                }

                for dynamic_import in &module_imports.dynamic_imports {
                    match self.resolve_dependency(&module_path, dynamic_import) {
                        Ok(resolved_path) => {
                            queue.push((
                                resolved_path,
                                Some(module_id),
                                Some(graph::EdgeKind::DynamicImport),
                            ));
                        }
                        Err(e) => {
                            let err_msg = e.to_string();
                            if !err_msg.contains("External module") {
                                tracing::warn!(
                                    "Failed to resolve '{}' from {:?}: {}",
                                    dynamic_import,
                                    module_path,
                                    e
                                );
                                self.record_unresolved(dynamic_import, &module_path, &err_msg);
                            } else {
                                tracing::debug!(
                                    "External module '{}' (not bundled)",
                                    dynamic_import
                                );
                            }
                        }
                    }
                }
            }
        }

        let graph = self.graph.read();
        let module_count = graph.module_count();

        if graph.has_cycle() {
            tracing::warn!(
                "Circular dependencies detected in module graph — \
                 will use runtime module system (generate_bundle_with_runtime)"
            );
        }

        tracing::info!("Module graph built: {} modules", module_count);

        Ok(())
    }

    fn record_unresolved(&self, specifier: &str, importer: &PathBuf, reason: &str) {
        self.unresolved_deps.lock().push(UnresolvedDependency {
            specifier: specifier.to_string(),
            importer: importer.clone(),
            reason: reason.to_string(),
        });
    }

    /// Fail the build if `build_graph` collected any non-external unresolved
    /// bare-specifier imports. The diagnostic enumerates each missing
    /// specifier with its importer (deduplicated by specifier, stable
    /// lexical order) so CI can act on it.
    ///
    /// @spec projects/jet/docs/build-fails-loudly-on-unresolved-bare-specifiers.md
    /// @issue #1317
    fn check_unresolved_deps(&self) -> Result<()> {
        let deps = std::mem::take(&mut *self.unresolved_deps.lock());
        if deps.is_empty() {
            return Ok(());
        }
        Err(format_unresolved_error(&deps))
    }

    fn resolve_dependency(&self, from: &PathBuf, specifier: &str) -> Result<PathBuf> {
        let resolved = self.resolver.resolve(specifier, from)?;

        if resolved.is_external {
            tracing::debug!("Skipping external module: {}", specifier);
            return Err(anyhow::anyhow!("External module: {}", specifier));
        }

        // Use the resolved path directly instead of canonicalize().
        // canonicalize() follows hardlinks to ~/.jet-store/ which breaks
        // node_modules walk-up resolution for transitive dependencies.
        let abs = if resolved.path.is_absolute() {
            resolved.path
        } else {
            std::env::current_dir()?.join(&resolved.path)
        };

        Ok(abs)
    }

    async fn transform_modules(&self) -> Result<(Vec<CompiledModule>, bool)> {
        tracing::debug!("Transforming modules");

        let graph = self.graph.read();

        let (sorted_ids, has_cycle) = match graph.topological_sort() {
            Ok(ids) => (ids, false),
            Err(cycle_paths) => {
                tracing::warn!(
                    "Circular dependency cycle detected ({} modules): {:?}",
                    cycle_paths.len(),
                    cycle_paths
                );
                tracing::warn!(
                    "Using graph insertion order as module ID assignment; \
                     bundle will use runtime module system"
                );
                (graph.all_node_ids(), true)
            }
        };

        let module_map: std::collections::HashMap<PathBuf, usize> = sorted_ids
            .iter()
            .enumerate()
            .filter_map(|(idx, &id)| {
                let node = graph.get_node(id)?;
                Some((node.path.clone(), idx))
            })
            .collect();

        tracing::debug!("Built module map with {} entries", module_map.len());

        use rayon::prelude::*;

        let modules: Vec<CompiledModule> = sorted_ids
            .par_iter()
            .filter_map(|&id| {
                let node = graph.get_node(id)?;

                // GH #3136 — IO failures must surface, not get silently
                // dropped via `.ok()?`. A dropped module here produces a
                // bundle with dangling module-id references and a runtime
                // "module N is not defined" with zero diagnostic.
                let metadata = match std::fs::metadata(&node.path) {
                    Ok(m) => m,
                    Err(e) => {
                        return Some(Err(anyhow::anyhow!(
                            "bundler: cannot stat module {:?}: {e} (GH #3136)",
                            node.path
                        )));
                    }
                };
                let modified = match metadata.modified() {
                    Ok(t) => t,
                    Err(e) => {
                        return Some(Err(anyhow::anyhow!(
                            "bundler: cannot read mtime for {:?}: {e} (GH #3136)",
                            node.path
                        )));
                    }
                };
                let mtime = match modified.duration_since(std::time::UNIX_EPOCH) {
                    Ok(d) => d.as_secs(),
                    Err(e) => {
                        return Some(Err(anyhow::anyhow!(
                            "bundler: mtime for {:?} predates UNIX epoch: {e} (GH #3136)",
                            node.path
                        )));
                    }
                };

                if let Some(cached) = self.cache.get(&node.path, mtime) {
                    tracing::debug!("Using cached module: {:?}", node.path);
                    return Some(Ok(cached));
                }

                let source = match std::fs::read_to_string(&node.path) {
                    Ok(s) => s,
                    Err(e) => {
                        return Some(Err(anyhow::anyhow!(
                            "bundler: cannot read module {:?}: {e} (GH #3136)",
                            node.path
                        )));
                    }
                };

                let result = match node.kind {
                    graph::ModuleKind::Script => {
                        self.transformer
                            .transform_js_with_context(&source, &node.path, &module_map)
                    }
                    graph::ModuleKind::Css => self.transformer.transform_css(&source),
                    graph::ModuleKind::Wasm => {
                        let wasm_path = node.path.to_string_lossy();
                        let glue = generate_wasm_glue(&wasm_path);
                        Ok(crate::transform::TransformResult {
                            code: glue,
                            source_map: None,
                        })
                    }
                    _ => {
                        tracing::debug!("Skipping unsupported module kind: {:?}", node.path);
                        return None;
                    }
                };

                match result {
                    Ok(transform_result) => {
                        // Apply compile-time defines (import.meta.env.*, process.env.*, etc.)
                        // after transformation so the define replacements are applied to the
                        // already-transpiled output.  This is a no-op when `self.defines` is empty.
                        //
                        // When defines are present, also run DCE to eliminate dead branches
                        // created by the replacements (e.g. `if ("production" !== "production")`).
                        let final_code = if self.defines.is_empty() {
                            transform_result.code.clone()
                        } else {
                            let after_define =
                                define::replace_defines(&transform_result.code, &self.defines);
                            dce::eliminate_dead_code(&after_define)
                        };

                        let compiled = CompiledModule {
                            path: node.path.clone(),
                            code: final_code.clone(),
                            source_map: transform_result.source_map.clone(),
                            dependencies: Vec::new(),
                            hash: calculate_hash(&final_code),
                        };

                        self.cache
                            .insert(node.path.clone(), mtime, compiled.clone());

                        tracing::debug!("Transformed module: {:?}", node.path);
                        Some(Ok(compiled))
                    }
                    Err(e) => {
                        tracing::error!("Failed to transform {:?}: {}", node.path, e);
                        Some(Err(e))
                    }
                }
            })
            .collect::<Result<Vec<_>>>()?;

        tracing::info!("Transformed {} modules", modules.len());

        Ok((modules, has_cycle))
    }

    /// Run tree-shaking analysis across all modules and remove unused exports.
    ///
    /// This is Phase 3 of the bundler pipeline (after transform + define + DCE,
    /// before generate_bundle).  Modules whose exports are entirely unused and
    /// have no side effects are eliminated.
    fn apply_tree_shaking(&self, modules: Vec<CompiledModule>) -> Vec<CompiledModule> {
        let module_pairs: Vec<(PathBuf, String)> = modules
            .iter()
            .map(|m| (m.path.clone(), m.code.clone()))
            .collect();

        let analysis = match tree_shake::analyze_used_exports(&module_pairs) {
            Ok(a) => a,
            Err(e) => {
                tracing::warn!("Tree shake analysis failed, skipping: {}", e);
                return modules;
            }
        };

        if !analysis.eliminated_modules.is_empty() {
            tracing::info!(
                "Tree shaking: eliminating {} modules (~{} bytes)",
                analysis.eliminated_modules.len(),
                analysis.eliminated_bytes
            );
        }

        modules
            .into_iter()
            .filter(|m| !analysis.eliminated_modules.contains(&m.path))
            .map(|m| {
                let used = analysis
                    .used_exports
                    .get(&m.path)
                    .cloned()
                    .unwrap_or_default();
                if used.is_empty() {
                    return m;
                }
                let shaken = tree_shake::shake_module(&m.code, &m.path, &used);
                CompiledModule { code: shaken, ..m }
            })
            .collect()
    }

    fn generate_bundle(
        &self,
        modules: Vec<CompiledModule>,
        has_cycle: bool,
    ) -> Result<BundleOutput> {
        tracing::debug!("Generating bundle from {} modules", modules.len());

        if modules.is_empty() {
            return Ok(BundleOutput {
                code: String::new(),
                source_map: None,
                assets: Vec::new(),
            });
        }

        // Bundle format selection:
        //
        //   Runtime (`generate_bundle_with_runtime`) — used when:
        //     • circular dependencies are present (cycles prevent topo-sort;
        //       the `__jet__.require` runtime handles circular refs natively
        //       via the pre-seeded `cache[id] = { exports: {} }` pattern)
        //     • dynamic import() calls are present (async chunks need the
        //       module registry at runtime)
        //
        //   Phase 2 (true flattening) — used when `minify=true` and safe:
        //     `generate_flattened_bundle` merges all module bodies into a
        //     single flat IIFE scope.  Each module's top-level variables are
        //     renamed with collision-avoiding `_m{n}_` prefixes and CJS
        //     globals are substituted (`exports` → `_m{n}e`, `module` →
        //     `_m{n}`, `require` → `_r`).  The post-processing
        //     `mangle_variables_with_root` pass then compresses every
        //     prefixed name to a 1-2 byte identifier in a single unified
        //     scope — matching Webpack/Terser bundle size (≤ 196 KB for
        //     react-bench vs 215 KB with Phase 1).
        //
        //   Phase 1 (per-module IIFE wrappers) — used when:
        //     • minify=false (dev builds; prefixed names would enlarge output)
        //     • any module uses eval/with/arguments[ (unsafe to merge scopes)
        let bundle = if has_cycle {
            tracing::debug!("Using runtime module system (circular dependencies present)");
            generate_bundle_with_runtime(&modules)
        } else if scope_hoist::is_scope_hoist_safe(&modules) {
            if self.minify && scope_hoist::is_flatten_safe(&modules) {
                tracing::debug!(
                    "Using Phase 2 true module flattening \
                     (minify=true, no eval/with/arguments[)"
                );
                let raw = scope_hoist::generate_flattened_bundle(&modules);
                // R4: Cross-module constant inlining → R5: DCE
                let after_r4 = scope_hoist::inline_cross_module_constants(&raw);
                scope_hoist::eliminate_unused_exports(&after_r4)
            } else {
                tracing::debug!("Using Phase 1 scope hoisting (no dynamic imports)");
                scope_hoist::generate_scope_hoisted_bundle(&modules)
            }
        } else {
            tracing::debug!("Falling back to runtime module system (dynamic imports present)");
            generate_bundle_with_runtime(&modules)
        };

        Ok(BundleOutput {
            code: bundle,
            source_map: None,
            assets: Vec::new(),
        })
    }
}

/// Fallback bundle generator using the full `__jet__` runtime.
///
/// Used when `is_scope_hoist_safe` returns `false` (dynamic imports
/// present). Preserves the module registry so that async chunks can
/// be loaded and registered at runtime.
fn generate_bundle_with_runtime(modules: &[CompiledModule]) -> String {
    let mut bundle = String::new();
    bundle.push_str(&generate_runtime());
    bundle.push_str("\n\n");

    for (idx, module) in modules.iter().enumerate() {
        let module_path = module.path.to_string_lossy();
        bundle.push_str(&format!("// Module {}: {}\n", idx, module_path));
        bundle.push_str(&format!(
            "__jet__.define({}, function(require, module, exports) {{\n",
            idx
        ));
        bundle.push_str(&module.code);
        bundle.push_str("\n});\n\n");
    }

    bundle.push_str("// Execute entry point\n");
    bundle.push_str("__jet__.require(0);\n");
    bundle
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
impl CompilationCache {
    pub fn new() -> Self {
        Self {
            module_cache: DashMap::new(),
        }
    }

    pub fn get(&self, path: &PathBuf, mtime: u64) -> Option<CompiledModule> {
        self.module_cache
            .get(&(path.clone(), mtime))
            .map(|entry| entry.clone())
    }

    pub fn insert(&self, path: PathBuf, mtime: u64, module: CompiledModule) {
        self.module_cache.insert((path, mtime), module);
    }

    pub fn clear(&self) {
        self.module_cache.clear();
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
impl Default for CompilationCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a list of collected `UnresolvedDependency` rows into the
/// `anyhow::Error` returned by `Bundler::check_unresolved_deps`.
///
/// Output is deterministic: deduplicated by specifier, sorted lexically.
/// Extracted as a free function so it can be unit-tested without spinning
/// up a full `Bundler`.
///
/// @spec projects/jet/docs/build-fails-loudly-on-unresolved-bare-specifiers.md
/// @issue #1317
fn format_unresolved_error(deps: &[UnresolvedDependency]) -> anyhow::Error {
    use std::collections::BTreeMap;

    // First sighting wins per specifier; BTreeMap gives lexical order.
    let mut by_specifier: BTreeMap<&str, &UnresolvedDependency> = BTreeMap::new();
    for d in deps {
        by_specifier.entry(d.specifier.as_str()).or_insert(d);
    }

    let mut msg = String::from(
        "Unresolved imports — `jet build` cannot continue. Resolve these \
         specifiers (install the missing package, fix the import path, or \
         mark the specifier as external) and re-run:\n",
    );
    for (_, d) in &by_specifier {
        msg.push_str(&format!(
            "  • `{}` imported from {} — {}\n",
            d.specifier,
            d.importer.display(),
            d.reason,
        ));
    }
    msg.push_str("See: https://github.com/anthropics/cclab/issues/1317");
    anyhow::anyhow!(msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = CompilationCache::new();
        assert_eq!(cache.module_cache.len(), 0);
    }

    // ──────────────────────────────────────────────────────────────────
    // Preload hints tests (R8 / T12)
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_generate_preload_tags() {
        let hints = vec![
            PreloadHint {
                href: "assets/vendor.abc123.js".to_string(),
                is_static: true,
            },
            PreloadHint {
                href: "assets/chunk-lazy.def456.js".to_string(),
                is_static: false, // dynamic, should be excluded
            },
        ];
        let tags = generate_preload_tags(&hints);
        assert!(
            tags.contains(r#"<link rel="modulepreload" href="assets/vendor.abc123.js">"#),
            "Static preload hint should generate a modulepreload tag"
        );
        assert!(
            !tags.contains("chunk-lazy"),
            "Dynamic imports should not be preloaded"
        );
    }

    #[test]
    fn test_inject_preload_hints_into_head() {
        let html = "<html><head><title>App</title></head><body></body></html>";
        let hints = vec![PreloadHint {
            href: "assets/vendor.abc123.js".to_string(),
            is_static: true,
        }];
        let result = inject_preload_hints(html, &hints);
        assert!(
            result.contains(r#"<link rel="modulepreload" href="assets/vendor.abc123.js">"#),
            "Preload tag should be injected"
        );
        // Should appear after <head>
        let head_pos = result.find("<head>").unwrap();
        let link_pos = result.find("modulepreload").unwrap();
        assert!(link_pos > head_pos, "Preload tag should be after <head>");
    }

    #[test]
    fn test_inject_preload_hints_no_head() {
        let html = "<div>Content</div>";
        let hints = vec![PreloadHint {
            href: "assets/shared.js".to_string(),
            is_static: true,
        }];
        let result = inject_preload_hints(html, &hints);
        assert!(
            result.contains("modulepreload"),
            "Preload tag should be prepended when no <head>"
        );
    }

    #[test]
    fn test_inject_preload_hints_empty() {
        let html = "<html><head></head></html>";
        let hints: Vec<PreloadHint> = Vec::new();
        let result = inject_preload_hints(html, &hints);
        assert_eq!(result, html, "Empty hints should not modify HTML");
    }

    // ──────────────────────────────────────────────────────────────────
    // Phase 2 flattening + mangling pipeline tests (#882, #903)
    // ──────────────────────────────────────────────────────────────────

    fn make_compiled(path: &str, code: &str) -> CompiledModule {
        CompiledModule {
            path: std::path::PathBuf::from(path),
            code: code.to_string(),
            source_map: None,
            dependencies: Vec::new(),
            hash: String::new(),
        }
    }

    /// Simulate the full production pipeline:
    ///   Phase 2 flatten → R4 constant inlining → R5 DCE →
    ///   minify → mangle_with_root → bool literals → fold
    fn simulate_prod_pipeline(modules: &[CompiledModule]) -> String {
        let raw = scope_hoist::generate_flattened_bundle(modules);
        // R4: Cross-module constant inlining
        let after_r4 = scope_hoist::inline_cross_module_constants(&raw);
        // R5: Unified cross-module DCE
        let after_r5 = scope_hoist::eliminate_unused_exports(&after_r4);
        let minified = crate::bundler::minify::minify_js(
            &after_r5,
            &[crate::bundler::minify::DropStatement::Console],
        );
        let mangled = crate::bundler::mangle::mangle_variables_with_root(&minified);
        let with_bools = crate::bundler::minify::replace_bool_literals(&mangled);
        crate::bundler::fold::fold_constants(&with_bools)
    }

    #[test]
    fn test_phase2_bundle_uses_flat_format_when_minify() {
        // Phase 2 output must NOT contain per-module !function wrappers.
        let modules = vec![make_compiled(
            "entry.js",
            "exports.main = function() { return 42; };",
        )];
        let bundle = scope_hoist::generate_flattened_bundle(&modules);
        assert!(
            !bundle.contains("!function(module,exports,require)"),
            "Phase 2 must not contain per-module IIFE wrappers, got: {}",
            bundle
        );
        assert!(
            bundle.contains("(function()"),
            "Phase 2 must have outer IIFE, got: {}",
            bundle
        );
    }

    #[test]
    fn test_phase2_pipeline_compresses_prefixed_names() {
        // After the full pipeline (flatten → minify → mangle_with_root),
        // module-prefixed names like _m0_workInProgress must be compressed.
        let modules = vec![make_compiled(
            "entry.js",
            "var workInProgress = null; exports.render = function() { return workInProgress; };",
        )];
        let result = simulate_prod_pipeline(&modules);
        assert!(
            !result.contains("workInProgress"),
            "workInProgress must be compressed after full pipeline, got: {}",
            result
        );
        assert!(
            !result.contains("_m0_workInProgress"),
            "prefixed name must be compressed, got: {}",
            result
        );
    }

    #[test]
    fn test_phase2_pipeline_two_modules_no_collision() {
        // Two modules both declare `var count`. After Phase 2 + mangling,
        // the names must be distinct short identifiers with no raw collision.
        let modules = vec![
            make_compiled("entry.js", "var dep = require(1); dep.exports.inc();"),
            make_compiled(
                "dep.js",
                "var count = 0; exports.inc = function() { count++; };",
            ),
        ];
        let bundle = scope_hoist::generate_flattened_bundle(&modules);
        // Prefixed in Phase 2 — no raw `count` at module boundary
        assert!(
            !bundle.contains("var count"),
            "raw 'count' must be prefixed in Phase 2 output, got: {}",
            bundle
        );
        assert!(
            bundle.contains("_m1_count"),
            "count prefixed to _m1_count in Phase 2, got: {}",
            bundle
        );

        let result = simulate_prod_pipeline(&modules);
        // After mangling, no long name survives
        assert!(
            !result.contains("_m1_count"),
            "prefixed _m1_count must be mangled away, got: {}",
            result
        );
    }

    #[test]
    fn test_phase2_pipeline_with_cross_module_dce() {
        // End-to-end: Module 0 (entry) requires Module 1 (config) and Module 2 (lib).
        // config exports a const string; lib exports used+unused functions.
        // After R4 (constant inlining) + R5 (DCE), the unused function and
        // the const declaration should be eliminated, reducing bundle size.
        let modules = vec![
            make_compiled(
                "entry.js",
                "var cfg = require(1); var lib = require(2); lib.exports.render(cfg.exports.MODE);",
            ),
            make_compiled(
                "config.js",
                "var MODE = 'production'; exports.MODE = MODE;",
            ),
            make_compiled(
                "lib.js",
                "exports.render = function(mode) { return mode; };\nexports.debug = function() { console.log('debug'); };",
            ),
        ];

        // Pipeline without R4/R5 (raw flatten only)
        let raw = scope_hoist::generate_flattened_bundle(&modules);

        // Pipeline with R4/R5
        let optimized = simulate_prod_pipeline(&modules);

        // The optimized output should be smaller (R4 inlines MODE, R5 removes debug)
        assert!(
            optimized.len() < raw.len(),
            "R4+R5 should reduce bundle size: {} < {} (raw)",
            optimized.len(),
            raw.len()
        );

        // The unused 'debug' export should NOT appear in optimized output
        assert!(
            !optimized.contains("debug"),
            "unused 'debug' export should be eliminated, got: {}",
            optimized
        );
    }

    #[test]
    fn test_phase2_pipeline_size_smaller_than_phase1() {
        // For a bundle with many long variable names, Phase 2 + mangling
        // should produce a strictly smaller output than Phase 1 + mangling.
        let long_code = r#"
var workInProgressRoot = null;
var workInProgressRootRenderLanes = 0;
var executionContext = 0;
var workInProgressSuspendedReason = 0;
exports.render = function() {
    workInProgressRoot = 1;
    workInProgressRootRenderLanes = 2;
    executionContext = 3;
    return workInProgressSuspendedReason;
};
"#;
        let modules = vec![make_compiled("react-dom.js", long_code)];

        // Phase 1 pipeline
        let phase1_raw = scope_hoist::generate_scope_hoisted_bundle(&modules);
        let phase1_min = crate::bundler::minify::minify_js(&phase1_raw, &[]);
        let phase1_out = crate::bundler::mangle::mangle_variables_with_root(&phase1_min);

        // Phase 2 pipeline
        let phase2_out = simulate_prod_pipeline(&modules);

        assert!(
            phase2_out.len() <= phase1_out.len(),
            "Phase 2 output ({} bytes) should be ≤ Phase 1 ({} bytes)",
            phase2_out.len(),
            phase1_out.len()
        );
    }
}

/// Pins the #1317 behaviour: `jet build` must fail loudly when a bare
/// specifier can neither be resolved on disk nor was opted into being
/// external, and must continue to silently skip when the user did opt in.
///
/// @spec projects/jet/docs/build-fails-loudly-on-unresolved-bare-specifiers.md
/// @issue #1317
#[cfg(test)]
mod unresolved_deps_tests {
    use super::*;
    use crate::bundler::types::BundleOptions;
    use std::collections::HashSet;
    use std::io::Write;

    fn write_fixture(dir: &std::path::Path, files: &[(&str, &str)]) -> PathBuf {
        for (name, contents) in files {
            let path = dir.join(name);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(contents.as_bytes()).unwrap();
        }
        dir.join(files[0].0)
    }

    #[tokio::test]
    async fn unresolved_bare_specifier_fails_build() {
        let tmp = tempfile::tempdir().unwrap();
        let entry = write_fixture(
            tmp.path(),
            &[(
                "entry.tsx",
                // Imports a package that does not exist on disk and that
                // the user did NOT mark external.
                "import { useState } from 'react';\nexport const X = useState;\n",
            )],
        );

        let opts = BundleOptions {
            entry: entry.clone(),
            output_dir: tmp.path().to_path_buf(),
            ..Default::default()
        };
        let bundler = Bundler::new(opts).unwrap();
        let err = bundler.bundle(entry).await.unwrap_err();
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("Unresolved imports"),
            "expected unresolved-imports diagnostic, got: {msg}"
        );
        assert!(
            msg.contains("`react`"),
            "diagnostic should name the unresolved specifier, got: {msg}"
        );
    }

    #[tokio::test]
    async fn externalize_all_packages_suppresses_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let entry = write_fixture(
            tmp.path(),
            &[(
                "entry.tsx",
                "import { useState } from 'react';\nexport const X = useState;\n",
            )],
        );

        let opts = BundleOptions {
            entry: entry.clone(),
            output_dir: tmp.path().to_path_buf(),
            externalize_all_packages: true,
            ..Default::default()
        };
        let bundler = Bundler::new(opts).unwrap();
        // Lib mode opts into externalizing bare specifiers — the new error
        // path must not trigger here.
        let _ = bundler
            .bundle(entry)
            .await
            .expect("lib mode must continue to skip external bare specifiers");
    }

    #[tokio::test]
    async fn explicit_externals_set_suppresses_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let entry = write_fixture(
            tmp.path(),
            &[(
                "entry.tsx",
                "import { useState } from 'react';\nexport const X = useState;\n",
            )],
        );

        let mut externals = HashSet::new();
        externals.insert("react".to_string());
        let opts = BundleOptions {
            entry: entry.clone(),
            output_dir: tmp.path().to_path_buf(),
            externals,
            ..Default::default()
        };
        let bundler = Bundler::new(opts).unwrap();
        // User explicitly marked `react` external — must not error.
        let _ = bundler
            .bundle(entry)
            .await
            .expect("explicit externals must continue to skip the specifier");
    }

    #[test]
    fn format_unresolved_error_dedups_and_sorts() {
        let deps = vec![
            UnresolvedDependency {
                specifier: "react-dom".into(),
                importer: PathBuf::from("/p/src/main.tsx"),
                reason: "Cannot resolve package: react-dom".into(),
            },
            UnresolvedDependency {
                specifier: "react".into(),
                importer: PathBuf::from("/p/src/App.tsx"),
                reason: "Cannot resolve package: react".into(),
            },
            // Duplicate of `react` from a different importer — first sighting wins.
            UnresolvedDependency {
                specifier: "react".into(),
                importer: PathBuf::from("/p/src/Other.tsx"),
                reason: "Cannot resolve package: react".into(),
            },
        ];

        let msg = format!("{:#}", format_unresolved_error(&deps));
        // Lexical order: react then react-dom.
        let react_pos = msg.find("`react`").expect("must mention react");
        let react_dom_pos = msg.find("`react-dom`").expect("must mention react-dom");
        assert!(
            react_pos < react_dom_pos,
            "specifiers must appear in lexical order, got:\n{msg}"
        );
        // First-sighting importer wins (App.tsx, not Other.tsx).
        assert!(
            msg.contains("App.tsx"),
            "diagnostic should keep first-sighting importer, got:\n{msg}"
        );
        assert!(
            !msg.contains("Other.tsx"),
            "diagnostic must dedup by specifier, got:\n{msg}"
        );
    }
}

/// GH #3136 — `transform_modules` must surface IO errors instead of
/// silently dropping the affected module via `.ok()?`. A dropped module
/// produces a bundle with dangling module-id references whose only
/// runtime symptom is `"module N is not defined"` with no path/file
/// breadcrumb.
#[cfg(test)]
mod transform_modules_silent_drop_tests {
    use super::*;
    use crate::bundler::graph::ModuleKind;
    use crate::bundler::types::BundleOptions;

    #[tokio::test]
    async fn transform_modules_surfaces_io_error_for_missing_file() {
        let bundler = Bundler::new(BundleOptions::default()).expect("bundler new");

        // Register a graph node whose path does not exist on disk.
        // `std::fs::metadata` will return NotFound; the pre-fix code
        // turned that into `None` via `.ok()?`, silently producing an
        // empty `modules` vec.
        let missing = std::path::PathBuf::from("/this/path/does/not/exist/jet_3136.js");
        {
            let mut g = bundler.graph.write();
            g.add_module(missing.clone(), ModuleKind::Script, 0);
        }

        let result = bundler.transform_modules().await;

        let err = result.expect_err(
            "transform_modules must propagate IO error rather than silently \
             drop the module (GH #3136)",
        );
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("GH #3136"),
            "error must include the searchable issue tag, got: {msg}"
        );
        assert!(
            msg.contains("jet_3136.js"),
            "error must name the failing module path, got: {msg}"
        );
    }
}

#[cfg(test)]
mod gh3821_bundler_edge_kind_extension_warn_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn utf8_extension_borrows_silently_for_css() {
        let cow = coerce_bundler_edge_kind_extension_or_warn(Path::new("a.css"));
        assert_eq!(cow.as_ref(), "css");
        assert!(matches!(cow, std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn utf8_extension_borrows_silently_for_all_classified_kinds() {
        for (path, expected) in [
            ("a.css", "css"),
            ("a.scss", "scss"),
            ("a.sass", "sass"),
            ("a.less", "less"),
            ("a.wasm", "wasm"),
            ("a.js", "js"),
        ] {
            let cow = coerce_bundler_edge_kind_extension_or_warn(Path::new(path));
            assert_eq!(cow.as_ref(), expected, "path {path}");
        }
    }

    #[test]
    fn unrecognised_utf8_extension_borrows_silently() {
        let cow = coerce_bundler_edge_kind_extension_or_warn(Path::new("weird.toml"));
        assert_eq!(cow.as_ref(), "toml");
        assert!(matches!(cow, std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn no_extension_falls_back_to_named_constant() {
        let cow = coerce_bundler_edge_kind_extension_or_warn(Path::new("noext"));
        assert_eq!(cow.as_ref(), BUNDLER_EDGE_KIND_NO_EXTENSION_FALLBACK);
        assert_eq!(cow.as_ref(), "");
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_extension_produces_lossy_form_not_empty() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let raw = b"a.\xffweird";
        let path = std::path::PathBuf::from(OsStr::from_bytes(raw));
        let cow = coerce_bundler_edge_kind_extension_or_warn(&path);
        assert!(!cow.as_ref().is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn two_distinct_non_utf8_extensions_do_not_collide_onto_empty() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let a = std::path::PathBuf::from(OsStr::from_bytes(b"a.\xffone"));
        let b = std::path::PathBuf::from(OsStr::from_bytes(b"a.\xfetwo"));
        let ca = coerce_bundler_edge_kind_extension_or_warn(&a).into_owned();
        let cb = coerce_bundler_edge_kind_extension_or_warn(&b).into_owned();
        assert_ne!(ca, cb);
    }

    #[test]
    fn warn_helpers_pinned_for_discoverability() {
        let _: fn(&Path) -> String = format_bundler_edge_kind_no_extension_warn;
        let _: fn(&Path, &str) -> String = format_bundler_edge_kind_non_utf8_extension_warn;
        let _: fn(&Path) -> std::borrow::Cow<'_, str> = coerce_bundler_edge_kind_extension_or_warn;
        assert_eq!(BUNDLER_EDGE_KIND_NO_EXTENSION_FALLBACK, "");
    }

    #[test]
    fn each_warn_string_carries_gh3821_tag() {
        let no_ext = format_bundler_edge_kind_no_extension_warn(Path::new("noext"));
        let non_utf8 =
            format_bundler_edge_kind_non_utf8_extension_warn(Path::new("a.bad"), "\u{FFFD}");
        assert!(no_ext.contains("gh3821"), "no-ext warn lacks tag: {no_ext}");
        assert!(
            non_utf8.contains("gh3821"),
            "non-utf8 warn lacks tag: {non_utf8}"
        );
    }

    #[test]
    fn warn_distinct_from_prior_silent_fallback_families() {
        let no_ext = format_bundler_edge_kind_no_extension_warn(Path::new("noext"));
        let non_utf8 =
            format_bundler_edge_kind_non_utf8_extension_warn(Path::new("a.bad"), "\u{FFFD}");
        for prior in [
            "gh3789", "gh3791", "gh3793", "gh3795", "gh3797", "gh3799", "gh3801", "gh3803",
            "gh3805", "gh3807", "gh3809", "gh3811", "gh3813", "gh3815", "gh3817", "gh3819",
        ] {
            assert!(
                !no_ext.contains(prior),
                "no-ext warn collides with {prior}: {no_ext}"
            );
            assert!(
                !non_utf8.contains(prior),
                "non-utf8 warn collides with {prior}: {non_utf8}"
            );
        }
    }

    #[test]
    fn two_sibling_warns_are_mutually_distinct() {
        let no_ext = format_bundler_edge_kind_no_extension_warn(Path::new("noext"));
        let non_utf8 =
            format_bundler_edge_kind_non_utf8_extension_warn(Path::new("a.bad"), "\u{FFFD}");
        assert_ne!(no_ext, non_utf8);
        assert!(no_ext.contains("no extension"));
        assert!(non_utf8.contains("non-UTF-8"));
    }

    #[test]
    fn warn_names_edge_kind_classification_consequence() {
        let no_ext = format_bundler_edge_kind_no_extension_warn(Path::new("noext"));
        let non_utf8 =
            format_bundler_edge_kind_non_utf8_extension_warn(Path::new("a.bad"), "\u{FFFD}");
        assert!(
            no_ext.contains("Import") || no_ext.contains("classif"),
            "no-ext warn must name the consequence: {no_ext}"
        );
        assert!(
            non_utf8.contains("Import") || non_utf8.contains("classif"),
            "non-utf8 warn must name the consequence: {non_utf8}"
        );
    }
}
// CODEGEN-END
