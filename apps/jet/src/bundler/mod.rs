// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
use anyhow::Result;
use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use crate::css::{CssPipeline, TailwindConfig};

pub mod css_bundle;
pub mod dce;
pub mod define;
pub mod dts;
pub mod fold;
pub mod graph;
pub mod html_minify;
pub mod imports;
pub mod json_shake;
pub mod lib_build;
pub mod mangle;
pub mod minify;
pub mod persistent_cache;
pub mod scope_hoist;
pub mod scope_hoist_opt;
pub mod sourcemap;
pub mod splitting;
pub mod tree_shake;
pub mod types;

pub use graph::{EdgeKind, ModuleGraph, ModuleNode};
pub use imports::{ImportDeclaration, ImportKind, ModuleImports};
pub use lib_build::{
    build_library, AssetKind, AssetOutput, EntryOutput, LibBuildOptions, LibBuildResult, RawCopyDir,
};
pub use splitting::SplitResult;
pub use types::{BundleOptions, BundleOutput, ChunkArtifact, ModuleId, PreloadHint};

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

fn normalize_bundler_path_lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() && !out.has_root() {
                    out.push("..");
                }
            }
            _ => out.push(component.as_os_str()),
        }
    }
    out
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

/// Generate the code-splitting runtime module system: `generate_runtime`'s
/// `define`/`require` (kept verbatim) plus `registerChunk`/`loadChunk`/
/// `dynamicImport` for lazy chunk loading.
///
/// Used only by `Bundler::generate_split_bundle` (the `--splitting` entry
/// chunk). Kept as a separate function — rather than extending
/// `generate_runtime` in place — so the non-splitting runtime-module-system
/// fallback (`generate_bundle_with_runtime`, used whenever a build has
/// cycles or dynamic imports regardless of `--splitting`) stays byte-for-byte
/// unchanged (AC2).
///
/// Issue #2123: `loadChunk`/`dynamicImport` must agree with
/// `cli.rs`'s manifest encoder on the manifest's shape, so the choice is
/// made once here, at build time, by reading the same `JET_VERBOSE_MANIFEST`
/// env var `cli.rs`'s call site reads for `build_chunk_manifest_js` vs
/// `build_chunk_manifest_compact_js` — two complete runtime string
/// variants (`_verbose`/`_compact` below), not one template with an
/// `if (manifest.n) ... else ...` runtime branch in the accessors
/// themselves. Build-time selection keeps both the common (compact) path
/// and the escape-hatch (verbose) path exactly as simple as the pre-#2123
/// single-shape runtime, and keeps the verbose path byte-for-byte
/// unchanged.
/// @issue #1930
/// @issue #2123
fn generate_split_runtime() -> String {
    if std::env::var_os("JET_VERBOSE_MANIFEST").is_some() {
        generate_split_runtime_verbose()
    } else {
        generate_split_runtime_compact()
    }
}

/// `generate_split_runtime`'s `JET_VERBOSE_MANIFEST=1` variant: `loadChunk`/
/// `dynamicImport` read the pre-#2123 verbose
/// `{ chunks: { name: { file, imports } }, moduleChunks: { id: name } }`
/// manifest shape `build_chunk_manifest_js` (cli.rs) emits for that same
/// escape hatch. Byte-for-byte the original (pre-#2123) `generate_split_runtime`
/// body — kept unchanged so the escape hatch is a faithful A/B baseline.
/// @issue #1930
/// @issue #2123
fn generate_split_runtime_verbose() -> String {
    r#"// Jet Module Runtime (code splitting)
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

  // Chunk registry, in-flight load dedup map, and the entry script's own
  // src — captured synchronously at IIFE-execution time, since
  // document.currentScript is only valid during the initial script's own
  // (top-level) execution — so async/shared chunk <script> tags resolve
  // against the right base path.
  var registeredChunks = {};
  var chunkPromises = {};
  var entryScriptSrc =
    (typeof document !== 'undefined' && document.currentScript)
      ? document.currentScript.src
      : '';

  // Register an async/shared chunk's modules. Called by the chunk file
  // itself once it loads, via a generated call into this function
  // (see Bundler::generate_split_bundle's chunk wrapper on the Rust side).
  function registerChunk(name, factory) {
    factory();
    registeredChunks[name] = true;
  }

  // Load an async/shared chunk by name, injecting a <script> tag. Dep-first:
  // loads the chunk's declared dependency chunks (chunkManifest imports)
  // before the chunk itself. Dedups concurrent loads of the same chunk.
  function loadChunk(name) {
    if (registeredChunks[name]) {
      return Promise.resolve();
    }
    if (typeof document === 'undefined') {
      return Promise.reject(new Error(
        '__jet__.loadChunk: no document available to load chunk "' + name + '"'
      ));
    }
    if (chunkPromises[name]) {
      return chunkPromises[name];
    }
    var manifest = (window.__jet__ && window.__jet__.chunkManifest) ||
      { chunks: {}, moduleChunks: {} };
    var info = manifest.chunks[name];
    if (!info) {
      return Promise.reject(new Error('__jet__.loadChunk: unknown chunk "' + name + '"'));
    }
    var promise = Promise.all((info.imports || []).map(loadChunk)).then(function() {
      if (registeredChunks[name]) {
        return;
      }
      return new Promise(function(resolve, reject) {
        var base = entryScriptSrc.substring(0, entryScriptSrc.lastIndexOf('/') + 1);
        var script = document.createElement('script');
        script.src = base + info.file;
        script.onload = function() {
          if (registeredChunks[name]) {
            resolve();
          } else {
            reject(new Error(
              '__jet__.loadChunk: chunk "' + name + '" loaded but did not register'
            ));
          }
        };
        script.onerror = function() {
          reject(new Error(
            '__jet__.loadChunk: failed to load chunk "' + name + '" (' + script.src + ')'
          ));
        };
        document.head.appendChild(script);
      });
    });
    chunkPromises[name] = promise;
    return promise;
  }

  // Dynamic import() lowering target: require the module directly if it is
  // already available (e.g. bundled into the entry chunk), otherwise load
  // its owning chunk first.
  function dynamicImport(id) {
    if (modules[id]) {
      return Promise.resolve().then(function() {
        return require(id);
      });
    }
    var manifest = (window.__jet__ && window.__jet__.chunkManifest) ||
      { chunks: {}, moduleChunks: {} };
    var chunkName = manifest.moduleChunks[id];
    if (!chunkName) {
      return Promise.reject(new Error('__jet__.dynamicImport: no chunk maps to module ' + id));
    }
    return loadChunk(chunkName).then(function() {
      return require(id);
    });
  }

  // Expose global runtime
  window.__jet__ = {
    define: define,
    require: require,
    modules: modules,
    cache: cache,
    registerChunk: registerChunk,
    loadChunk: loadChunk,
    dynamicImport: dynamicImport
  };
})();
"#
    .to_string()
}

/// `generate_split_runtime`'s default variant: `loadChunk`/`dynamicImport`
/// read the compact `{ n: [name, ...], h: [hash, ...],
/// i: [[importOrdinal, ...], ...], m: { moduleId: chunkOrdinal } }`
/// manifest shape `build_chunk_manifest_compact_js` (cli.rs) emits by
/// default (issue #2123). Chunk names live once, in `n`; every other field
/// refers back to a chunk by its index ("ordinal") into `n` instead of
/// repeating the name string, and the on-disk URL
/// (`"assets/" + n[k] + "." + h[k] + ".js"`, matching `build_hashed_chunk`'s
/// `assets/<name>.<hash>.js` naming) is reconstructed here rather than
/// stored. Otherwise identical structure/behavior to
/// `generate_split_runtime_verbose` above (same `define`/`require`/
/// `registerChunk`, same dep-first `Promise.all` chunk loading, same
/// `<script>`-tag injection + dedup-by-name).
/// @issue #2123
fn generate_split_runtime_compact() -> String {
    r#"// Jet Module Runtime (code splitting)
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

  // Chunk registry, in-flight load dedup map, and the entry script's own
  // src — captured synchronously at IIFE-execution time, since
  // document.currentScript is only valid during the initial script's own
  // (top-level) execution — so async/shared chunk <script> tags resolve
  // against the right base path.
  var registeredChunks = {};
  var chunkPromises = {};
  var entryScriptSrc =
    (typeof document !== 'undefined' && document.currentScript)
      ? document.currentScript.src
      : '';

  // Register an async/shared chunk's modules. Called by the chunk file
  // itself once it loads, via a generated call into this function
  // (see Bundler::generate_split_bundle's chunk wrapper on the Rust side).
  function registerChunk(name, factory) {
    factory();
    registeredChunks[name] = true;
  }

  // Load an async/shared chunk by name, injecting a <script> tag. Dep-first:
  // loads the chunk's declared dependency chunks (chunkManifest imports,
  // here an ordinal array into the compact manifest's `n`) before the
  // chunk itself. Dedups concurrent loads of the same chunk.
  function loadChunk(name) {
    if (registeredChunks[name]) {
      return Promise.resolve();
    }
    if (typeof document === 'undefined') {
      return Promise.reject(new Error(
        '__jet__.loadChunk: no document available to load chunk "' + name + '"'
      ));
    }
    if (chunkPromises[name]) {
      return chunkPromises[name];
    }
    var manifest = (window.__jet__ && window.__jet__.chunkManifest) ||
      { n: [], h: [], i: [], m: {} };
    var ordinal = manifest.n.indexOf(name);
    if (ordinal === -1) {
      return Promise.reject(new Error('__jet__.loadChunk: unknown chunk "' + name + '"'));
    }
    var importOrdinals = manifest.i[ordinal] || [];
    var promise = Promise.all(importOrdinals.map(function(idx) {
      return loadChunk(manifest.n[idx]);
    })).then(function() {
      if (registeredChunks[name]) {
        return;
      }
      return new Promise(function(resolve, reject) {
        var base = entryScriptSrc.substring(0, entryScriptSrc.lastIndexOf('/') + 1);
        var file = 'assets/' + manifest.n[ordinal] + '.' + manifest.h[ordinal] + '.js';
        var script = document.createElement('script');
        script.src = base + file;
        script.onload = function() {
          if (registeredChunks[name]) {
            resolve();
          } else {
            reject(new Error(
              '__jet__.loadChunk: chunk "' + name + '" loaded but did not register'
            ));
          }
        };
        script.onerror = function() {
          reject(new Error(
            '__jet__.loadChunk: failed to load chunk "' + name + '" (' + script.src + ')'
          ));
        };
        document.head.appendChild(script);
      });
    });
    chunkPromises[name] = promise;
    return promise;
  }

  // Dynamic import() lowering target: require the module directly if it is
  // already available (e.g. bundled into the entry chunk), otherwise load
  // its owning chunk first.
  function dynamicImport(id) {
    if (modules[id]) {
      return Promise.resolve().then(function() {
        return require(id);
      });
    }
    var manifest = (window.__jet__ && window.__jet__.chunkManifest) ||
      { n: [], h: [], i: [], m: {} };
    var ordinal = manifest.m[id];
    if (ordinal === undefined) {
      return Promise.reject(new Error('__jet__.dynamicImport: no chunk maps to module ' + id));
    }
    return loadChunk(manifest.n[ordinal]).then(function() {
      return require(id);
    });
  }

  // Expose global runtime
  window.__jet__ = {
    define: define,
    require: require,
    modules: modules,
    cache: cache,
    registerChunk: registerChunk,
    loadChunk: loadChunk,
    dynamicImport: dynamicImport
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
    inject_tags_into_head(html, &tags)
}

/// Generate `<link rel="preload" as="script">` tags from preload hints.
///
/// Code-split chunks load via classic `<script>` tag injection
/// (`generate_split_runtime`'s `loadChunk`, not an ES module `import`), so
/// their preload relation is `preload`/`as="script"` rather than
/// `modulepreload` (`generate_preload_tags` above stays unchanged and keeps
/// serving any ESM-import caller). Only static chunk dependencies are
/// included; async chunks are excluded since they load on demand.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
/// @issue #1931
pub fn generate_script_preload_tags(hints: &[PreloadHint]) -> String {
    let mut tags = String::new();
    for hint in hints {
        if hint.is_static {
            tags.push_str(&format!(
                "<link rel=\"preload\" as=\"script\" href=\"{}\">\n",
                hint.href
            ));
        }
    }
    tags
}

/// Inject classic-script preload hint tags into an HTML string's `<head>`
/// section. Same head-insertion/prepend-fallback behavior as
/// `inject_preload_hints`, using `generate_script_preload_tags` instead of
/// `generate_preload_tags`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
/// @issue #1931
pub fn inject_script_preload_hints(html: &str, hints: &[PreloadHint]) -> String {
    let tags = generate_script_preload_tags(hints);
    inject_tags_into_head(html, &tags)
}

/// Shared head-insertion helper for `inject_preload_hints` /
/// `inject_script_preload_hints`: insert `tags` right after `<head>`
/// (case-insensitive search), or prepend to `html` when no `<head>` is
/// found. No-op (returns `html` unchanged) when `tags` is empty.
/// @issue #1931
fn inject_tags_into_head(html: &str, tags: &str) -> String {
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
        result.push_str(tags);
        result.push_str(&html[insert_pos..]);
        result
    } else {
        format!("{}{}", tags, html)
    }
}

/// `bundler` is threaded through only so this can consult the per-build
/// `source_cache` (`Bundler::cached_source`) instead of re-reading every
/// module from disk a second time (#1999) — the crawl already read these
/// same bytes during `build_graph`'s prefetch.
fn collect_side_effect_free_module_indices(
    bundler: &Bundler,
    graph: &ModuleGraph,
    sorted_ids: &[ModuleId],
) -> HashSet<usize> {
    let mut package_side_effects_cache: HashMap<
        (PathBuf, String),
        crate::bundler::tree_shake::SideEffectsDecl,
    > = HashMap::new();

    sorted_ids
        .iter()
        .enumerate()
        .filter_map(|(idx, &id)| {
            let node = graph.get_node(id)?;
            if node.kind != graph::ModuleKind::Script {
                return None;
            }
            let source = bundler.cached_source(&node.path).ok()?;
            let has_side_effects =
                crate::bundler::tree_shake::module_has_side_effects_with_package_json(
                    &source,
                    &node.path,
                    &mut package_side_effects_cache,
                );
            (!has_side_effects).then_some(idx)
        })
        .collect()
}

/// A bare-specifier import that the resolver could not find on disk
/// and that the user did not explicitly mark as external.
///
/// @spec apps/jet/docs/build-fails-loudly-on-unresolved-bare-specifiers.md
/// @issue #1317
#[derive(Debug, Clone)]
struct UnresolvedDependency {
    specifier: String,
    importer: PathBuf,
    reason: String,
}

/// Core bundler that orchestrates the build process
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
/// Memoized result of `Bundler::prefetch_one_module` — everything the
/// serial `build_graph` walk needs for one module, with failures kept as
/// the strings the original warn branches print.
struct PrefetchedModule {
    source: std::result::Result<String, String>,
    imports: std::result::Result<imports::ModuleImports, String>,
    resolutions: HashMap<String, std::result::Result<PathBuf, String>>,
    /// Parsed tree-sitter tree, kept ONLY for plain-JS modules whose source the
    /// transform won't rewrite, so the module transform can reuse it instead of
    /// re-parsing. `None` for TS/TSX/JSX/CSS/etc. See `extract_imports_with_tree`.
    tree: Option<tree_sitter::Tree>,
    /// Which import-extraction path this module's `imports` came from
    /// (#1997): `Some(true)` = the string-scan fast path
    /// (`imports::extract_imports_fast`), `Some(false)` = the tree-sitter
    /// fallback, `None` = extraction was never attempted — not a
    /// `ModuleKind::Script`, the file read failed, or (#2140) a persistent
    /// import-scan cache hit supplied the already-narrowed result with no
    /// fresh scan needed. Read back in `build_graph` to print the
    /// `JET_BUNDLE_TIMING` `import-scan:` line; carries no other behavior.
    used_fast_import_scan: Option<bool>,
}

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
    /// @spec apps/jet/docs/build-fails-loudly-on-unresolved-bare-specifiers.md
    /// @issue #1317
    unresolved_deps: Mutex<Vec<UnresolvedDependency>>,
    /// Per-module tree-sitter trees parsed during `build_graph`, drained by
    /// `transform_modules` so plain-JS modules are parsed once, not twice.
    /// Keyed by the canonical module path. Empty for any module not safe to
    /// reuse (TS/TSX/JSX), or when the graph cache short-circuits a module.
    parsed_trees: Mutex<HashMap<PathBuf, tree_sitter::Tree>>,
    /// Final accumulated per-barrel demand from the lazy crawl
    /// (`prefetch_graph_modules_lazy`), restricted to confirmed pure
    /// barrels. `transform_modules` consults this to prune a barrel's own
    /// unrequested re-export lines from its source before codegen, so the
    /// bundle output matches what an eager crawl + tree-shake would have
    /// produced — tree-shake itself cannot make this call, since it never
    /// sees a graph edge for a leaf the crawl deliberately never fetched.
    /// Empty whenever `JET_EAGER_BARRELS` is set or no pure barrel was
    /// detected.
    /// @issue #1991
    barrel_demand: Mutex<HashMap<PathBuf, BarrelDemand>>,
    /// Non-textual `(importer, target)` edges `build_graph` fabricates from
    /// module *structure* rather than anything present in the importer's own
    /// source text — invisible to a purely textual pre-transform liveness
    /// scan. Reset at the start of every `build_graph` call, then consulted
    /// by `compute_transform_survivors` (via
    /// `tree_shake::analyze_used_exports_from_with_implicit_edges`) so the
    /// survivors-only transform filter never misclassifies a
    /// structurally-reachable module as dead.
    ///
    /// Inventory (WI #1995 round 4 — re-derive by grepping `build_graph` for
    /// `queue.push`/`graph.add_dependency` call sites whenever this list is
    /// suspected stale):
    /// - `react/jsx-runtime`: every `.tsx`/`.jsx` module gets an implicit
    ///   Import edge to `react/jsx-runtime` purely from file extension; the
    ///   textual JSX-runtime import is only synthesized later, during the
    ///   JSX transform pass itself, so it never appears in raw source.
    ///   That is the only implicit edge the crawl currently inserts.
    /// @issue #1995
    implicit_edges: Mutex<Vec<(PathBuf, PathBuf)>>,
    /// Canonicalized bundle entry path, captured by `build_graph` for
    /// `compute_transform_survivors` (which needs the exact entry path to
    /// seed `tree_shake::analyze_used_exports_from_with_implicit_edges`).
    /// `None` until the first `build_graph` call.
    /// @issue #1995
    entry_path: Mutex<Option<PathBuf>>,
    /// Single-pass analysis reuse (WI #1995 round 5): the full
    /// `TreeShakeResult` `compute_transform_survivors` computes over the
    /// whole crawled graph (raw source, define-folded, with implicit
    /// edges), cached here so `apply_tree_shaking` can reuse it for its own
    /// elimination stage instead of re-reading every module's source and
    /// re-running the same analysis a second time. `None` whenever the
    /// survivors filter didn't run this build (`JET_NO_SURVIVOR_FILTER=1`
    /// set, or the pre-pass bailed) — `apply_tree_shaking` falls back to its
    /// own recompute in that case, unchanged from round 4. Reset to `None`
    /// at the start of every `build_graph` call, same lifecycle as
    /// `implicit_edges`/`entry_path`.
    /// @issue #1995
    shake_analysis: Mutex<Option<tree_shake::TreeShakeResult>>,
    /// Per-build cache of every crawled module's raw source text, keyed by
    /// canonical module path. Populated during `build_graph`'s prefetch
    /// crawl (`prefetch_one_module`) and consulted by every downstream phase
    /// that would otherwise re-read the same file from disk
    /// (`compute_transform_survivors`'s liveness pre-pass,
    /// `collect_side_effect_free_module_indices`, `transform_modules`'s
    /// per-module closure, `apply_tree_shaking`'s cache-miss recompute path)
    /// instead of hitting the filesystem again — up to 4-5 redundant reads
    /// of the same bytes per build otherwise. `cached_source` is the shared
    /// read-through helper: a cache hit returns the shared `Arc<str>`; a
    /// miss falls back to `fs::read_to_string` and back-fills the cache so
    /// later readers in the same build still benefit.
    ///
    /// Reset at the start of every `build_graph` call, same lifecycle as
    /// `implicit_edges`/`entry_path`/`shake_analysis`, so a later rebuild
    /// reusing this `Bundler` (e.g. dev-server re-bundling after a file
    /// changes) never serves a stale snapshot.
    /// `JET_NO_SOURCE_CACHE=1` disables population/consultation entirely
    /// (every reader falls straight through to `fs::read_to_string`) for a
    /// byte-identity A/B diff.
    /// @issue #1999
    source_cache: Mutex<HashMap<PathBuf, Arc<str>>>,
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
    /// Nx/tsconfig path alias entries `(prefix, target)`, cloned from
    /// `options.resolve_options.alias` in `Bundler::new` before
    /// `resolve_options` moves into `ModuleResolver::new`. Threaded into the
    /// codegen-time `ModuleResolutionIndex` by `transform_modules` (WI
    /// #1305) so the same alias table `resolver/mod.rs::resolve_alias`
    /// already consults during graph-walk resolution also reaches the
    /// post-graph-walk resolver.
    alias_entries: Vec<(String, PathBuf)>,
    /// Explicit tsconfig `baseUrl`, retained for the codegen-time resolver so
    /// emitted requires use the same local-module mapping as graph discovery.
    base_url: Option<PathBuf>,
    /// When true, `generate_bundle` emits a multi-chunk `BundleOutput`
    /// (entry + async/shared `ChunkArtifact`s) instead of a single file, and
    /// dynamic `import()` lowers to `__jet__.dynamicImport(id)`. Default
    /// `false` keeps the existing single-file output unchanged.
    /// @issue #1930
    splitting: bool,
    /// `[build.manual_chunks]` routing table, cloned from
    /// `BundleOptions::manual_chunks`. Consulted only by
    /// `generate_split_bundle` (i.e. only when `splitting` is also `true`).
    /// @issue #1948
    manual_chunks: HashMap<String, Vec<String>>,
    /// Disk-backed sibling to `cache` (#2137): `transform_modules` consults
    /// this ONLY after `cache`'s existing in-memory `(path, mtime)` lookup
    /// misses, so dev-server/watch semantics on `cache` itself are
    /// completely unchanged. Disabled (`PersistentTransformCache::disabled`)
    /// unless `BundleOptions::cache_project_root` was set — every non-opted
    /// path (dev server, `--lib`, `--nx`) gets a cache whose `get`/`insert`
    /// are no-ops, so this field never needs an `Option<..>` at its call
    /// sites. See `persistent_cache`'s module doc comment for the full
    /// design.
    /// @issue #2137
    persistent_cache: persistent_cache::PersistentTransformCache,
    /// `persistent_cache`'s own `load()` stats, captured once in `new` so
    /// `bundle`'s `JET_BUNDLE_TIMING` line can report `loaded_in=`
    /// alongside `save`'s freshly-measured `saved_in=`.
    /// @issue #2137
    persistent_cache_load: persistent_cache::LoadStats,
    /// `persistent_cache::resolver_config_fingerprint` of this bundler's
    /// resolve options, captured once in `new` — like `alias_entries`/
    /// `base_url` above, computed from `options.resolve_options` before it
    /// moves into `ModuleResolver::new`. Part of every
    /// `persistent_cache::ResolutionKey` `resolve_dependency` builds, so a
    /// persisted node_modules resolution from a build with different
    /// aliases/baseUrl/conditions/externals can never be reused by this one.
    /// @issue #2141
    resolver_config_fingerprint: u64,
}

/// Compilation cache for incremental builds
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub struct CompilationCache {
    module_cache: DashMap<(PathBuf, u64), CompiledModule>,
}

/// Compiled module with metadata
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
///
/// `Serialize`/`Deserialize` (#2137): every field here is already trivially
/// serializable, which is what makes this struct the exact payload the
/// persistent transform cache (`persistent_cache::PersistentTransformCache`)
/// round-trips through `node_modules/.jet/transform-cache.bin` — no
/// shadow-mirrored on-disk representation, this IS the cached value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledModule {
    pub id: usize,
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
        let splitting = options.splitting;
        let manual_chunks = options.manual_chunks.clone();
        // #2137 — computed (and, when enabled, loaded from disk) before
        // `options.transform_options` moves into `Transformer::new` below;
        // see `persistent_cache::config_fingerprint`'s doc comment for what
        // it covers.
        let config_fingerprint = persistent_cache::config_fingerprint(
            &defines,
            minify,
            splitting,
            &options.transform_options,
        );
        // #2141 — analysis section's own (narrower) fingerprint: only
        // `defines` affects `compute_raw_module_facts`'s output, so this is
        // deliberately not the same fingerprint as `config_fingerprint`
        // above (see `persistent_cache::analysis_fingerprint`'s doc comment).
        let analysis_fingerprint = persistent_cache::analysis_fingerprint(&defines);
        let (persistent_cache, persistent_cache_load) = match &options.cache_project_root {
            Some(root) => persistent_cache::PersistentTransformCache::load(
                root,
                config_fingerprint,
                analysis_fingerprint,
            ),
            None => (
                persistent_cache::PersistentTransformCache::disabled(),
                persistent_cache::LoadStats::default(),
            ),
        };
        let mut resolve_options = options.resolve_options;
        // WI #1305: retain a copy of the already-loaded alias table before
        // `resolve_options` moves into `ModuleResolver::new` below, so the
        // codegen-time resolver (`transform_modules`) can also consult it.
        let alias_entries = resolve_options.alias.clone();
        let base_url = resolve_options.base_url.clone();
        // Forward externalize_all_packages to the resolver
        if options.externalize_all_packages {
            resolve_options.externalize_all_packages = true;
        }
        // Forward explicit externals list
        for ext in &options.externals {
            resolve_options.externals.insert(ext.clone());
        }
        // #2141 — captured after the externalize_all_packages/externals
        // forwarding above so it reflects the exact `ResolveOptions` value
        // `ModuleResolver::new` below actually runs with; every
        // `resolve_dependency` node_modules-scoped cache key includes this,
        // so a persisted resolution from a build with different
        // aliases/baseUrl/conditions/externals can never be reused here.
        let resolver_config_fingerprint =
            persistent_cache::resolver_config_fingerprint(&resolve_options);
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
            alias_entries,
            base_url,
            unresolved_deps: Mutex::new(Vec::new()),
            parsed_trees: Mutex::new(HashMap::new()),
            barrel_demand: Mutex::new(HashMap::new()),
            implicit_edges: Mutex::new(Vec::new()),
            entry_path: Mutex::new(None),
            shake_analysis: Mutex::new(None),
            source_cache: Mutex::new(HashMap::new()),
            splitting,
            manual_chunks,
            persistent_cache,
            persistent_cache_load,
            resolver_config_fingerprint,
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

        // JET_BUNDLE_TIMING=1 prints per-phase wall-clock to stderr.
        let timing = std::env::var_os("JET_BUNDLE_TIMING").is_some();
        let mut last = std::time::Instant::now();
        let mut lap = |stage: &str| {
            if timing {
                eprintln!("[bundle-timing] {stage}: {:?}", last.elapsed());
                last = std::time::Instant::now();
            }
        };

        self.build_graph(&entry).await?;
        lap("build_graph");
        self.check_unresolved_deps()?;
        let (modules, has_cycle) = self.transform_modules().await?;
        lap("transform_modules");

        // Tree shaking: analyze used exports across the module graph, then
        // remove unused export declarations from each module.  Modules with
        // no used exports and no side effects are eliminated entirely.
        let modules = self.apply_tree_shaking(modules, &entry);
        lap("tree_shaking");

        let mut output = self.generate_bundle(modules, has_cycle)?;
        lap("generate_bundle");

        // Detect sibling CSS entry file and run it through the CSS pipeline.
        // Convention: if entry is `src/index.tsx`, look for `src/index.css`.
        if let Some(css_asset) = self.try_process_css_entry(&entry) {
            output.assets.push(css_asset);
        }

        // #2137 — persist the transform cache after a successful build. A
        // no-op (`SaveStats::default()`, no I/O) unless
        // `BundleOptions::cache_project_root` was set. Deliberately last:
        // a build that errors out earlier via `?` above never reaches this
        // line, so a partially-completed build never overwrites the store
        // with incomplete work.
        let save_stats = self.persistent_cache.save();
        if timing {
            eprintln!(
                "[bundle-timing] cache: hits={} misses={} loaded_in={:.2}ms saved_in={:.2}ms bytes={} r_hits={} r_misses={} a_hits={} a_misses={}",
                self.persistent_cache.hits(),
                self.persistent_cache.misses(),
                self.persistent_cache_load.duration.as_secs_f64() * 1000.0,
                save_stats.duration.as_secs_f64() * 1000.0,
                save_stats.bytes_written,
                // #2141 — resolution (node_modules-scoped) and analysis
                // (per-module liveness) section counters, alongside the
                // #2137 transform-section counters above.
                self.persistent_cache.resolution_hits(),
                self.persistent_cache.resolution_misses(),
                self.persistent_cache.analysis_hits(),
                self.persistent_cache.analysis_misses(),
            );
        }

        Ok(output)
    }

    /// #2143 — record this build's replay manifest, if replay is enabled
    /// and every consumed module's content could be tracked. `extra_inputs`/
    /// `extra_dirs` let `cli.rs` fold in inputs the `Bundler` itself never
    /// reads (`index.html`, `public/`, the project root's own listing) into
    /// the same manifest this collects from the module graph. A no-op
    /// (returns a zeroed [`persistent_cache::SaveStats`], writes nothing)
    /// when the persistent cache is disabled or [`Self::collect_replay_inputs`]
    /// declines — see that method's doc comment for why a decline
    /// deliberately does NOT clear whatever manifest is already on disk.
    #[allow(clippy::too_many_arguments)]
    pub fn record_replay_manifest(
        &self,
        replay_config_fingerprint: u64,
        extra_inputs: Vec<persistent_cache::ReplayInput>,
        extra_dirs: Vec<persistent_cache::ReplayDirFingerprint>,
        outputs: Vec<persistent_cache::ReplayOutput>,
        entry_rel_path: String,
        entry_size: u64,
    ) -> persistent_cache::SaveStats {
        if !self.persistent_cache.enabled() {
            return persistent_cache::SaveStats::default();
        }
        let Some((mut inputs, mut source_dirs)) = self.collect_replay_inputs() else {
            return persistent_cache::SaveStats::default();
        };
        inputs.extend(extra_inputs);
        source_dirs.extend(extra_dirs);
        let manifest = persistent_cache::ReplayManifest {
            replay_version: persistent_cache::REPLAY_VERSION,
            config_fingerprint: replay_config_fingerprint,
            inputs,
            source_dirs,
            outputs,
            entry_rel_path,
            entry_size,
        };
        self.persistent_cache.set_replay_manifest(manifest);
        self.persistent_cache.save()
    }

    /// #2143 — walk every module this build's graph discovered (not just
    /// tree-shake survivors: tracking a superset of what strictly matters is
    /// always safe, only ever occasionally wasteful, matching this whole
    /// section's "any doubt" philosophy) and directly re-read + re-hash each
    /// one's content, rather than reusing an already-computed hash from
    /// transform/import-scan time. Deliberately the simpler, uniform-
    /// coverage choice over threading collection through those perf-tuned
    /// hot paths: this only runs once, after a full build already did far
    /// more expensive work.
    ///
    /// Declines (`None`) — meaning `record_replay_manifest` records nothing,
    /// leaving any previous manifest on disk untouched rather than replacing
    /// it with a known-incomplete one — whenever any graph node is a `Json`
    /// or `Asset` module (neither of this pass's own file-content hash nor
    /// any other section here observes their consumed *values*, e.g. a JSON
    /// property or an asset's bytes reaching JS as an import, soundly), or
    /// whenever any consumed file's metadata/content cannot be read. Source
    /// directories under `node_modules` are excluded from the returned
    /// listing fingerprints — the resolution section (#2141) already
    /// guards node_modules package integrity independently.
    fn collect_replay_inputs(
        &self,
    ) -> Option<(
        Vec<persistent_cache::ReplayInput>,
        Vec<persistent_cache::ReplayDirFingerprint>,
    )> {
        let graph = self.graph.read();
        let mut inputs = Vec::new();
        let mut dirs: std::collections::BTreeSet<PathBuf> = std::collections::BTreeSet::new();
        for id in graph.all_node_ids() {
            let node = graph.get_node(id)?;
            if matches!(
                node.kind,
                graph::ModuleKind::Json | graph::ModuleKind::Asset
            ) {
                return None;
            }
            let meta = std::fs::metadata(&node.path).ok()?;
            let mtime_nanos = persistent_cache::mtime_nanos(&meta)?;
            let bytes = std::fs::read(&node.path).ok()?;
            let content_hash = persistent_cache::hash_bytes(&bytes);
            if let Some(parent) = node.path.parent() {
                if !parent.components().any(|c| c.as_os_str() == "node_modules") {
                    dirs.insert(parent.to_path_buf());
                }
            }
            inputs.push(persistent_cache::ReplayInput {
                path: node.path.clone(),
                content_hash,
                mtime_nanos,
                size: meta.len(),
            });
        }
        drop(graph);
        let mut source_dirs = Vec::new();
        for dir in dirs {
            let listing_hash = persistent_cache::single_dir_listing_hash(&dir)?;
            source_dirs.push(persistent_cache::ReplayDirFingerprint { dir, listing_hash });
        }
        Some((inputs, source_dirs))
    }

    /// Look for a CSS entry file alongside the JS entry and process it.
    ///
    /// Returns `None` if no CSS entry file is found, or if CSS processing fails
    /// (warnings are logged instead of propagating).
    fn try_process_css_entry(&self, js_entry: &PathBuf) -> Option<types::Asset> {
        let stem = js_entry.file_stem()?.to_string_lossy().into_owned();
        let dir = js_entry.parent()?;
        // Convention: sibling stylesheet entry named like the JS entry.
        // Prefer `.css`, then fall back to `.scss`/`.sass` so a Sass entry
        // (e.g. `src/index.scss` next to `src/index.tsx`) is compiled via
        // grass and run through the same CSS pipeline. The hashed output is
        // always a `.css` asset regardless of the source extension.
        let css_entry = [".css", ".scss", ".sass"]
            .iter()
            .map(|ext| dir.join(format!("{stem}{ext}")))
            .find(|p| p.exists())?;

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
    /// Wave-parallel discovery for [`Self::build_graph`]: walk the import
    /// graph breadth-first, and for every frontier run the pure per-module
    /// work (file read, tree-sitter import extraction, dependency
    /// resolution) across cores. Results are memoized by module path; the
    /// serial graph walk replays over them so its module-id assignment
    /// order is untouched. Resolution results are stored as
    /// `Result<PathBuf, String>` so the replay can preserve the original
    /// warn / external-module branches verbatim.
    ///
    /// Returns the memo alongside a *lazy barrel skip set* (#1991): the
    /// resolved path of every re-export leaf a pure barrel deliberately
    /// never fetched because no importer demanded it. [`Self::build_graph`]'s
    /// serial replay independently re-walks every module's own
    /// `static_imports`/`dynamic_imports` (so it can preserve its warn /
    /// external-module branches verbatim), and a barrel's own resolution
    /// memo always contains every leaf it re-exports regardless of demand
    /// — the skip set is what lets that replay tell "never fetched on
    /// purpose" apart from "not yet in the memo, read it fresh", so the
    /// leaves this function skips stay skipped instead of being
    /// synchronously re-read one at a time by the replay.
    ///
    /// Delegates to [`Self::prefetch_graph_modules_lazy`] by default; with
    /// `JET_EAGER_BARRELS` set, delegates instead to
    /// [`Self::prefetch_graph_modules_eager`], which reproduces the
    /// pre-#1991 crawl byte-for-byte (unconditional expansion of every
    /// resolved edge, empty skip set). The escape hatch exists for
    /// bisection and for the lazy-vs-eager output-identity test.
    fn prefetch_graph_modules(
        &self,
        entry_abs: &Path,
    ) -> (
        HashMap<PathBuf, PrefetchedModule>,
        HashSet<PathBuf>,
        HashMap<PathBuf, BarrelDemand>,
    ) {
        if std::env::var_os("JET_EAGER_BARRELS").is_some() {
            return (
                self.prefetch_graph_modules_eager(entry_abs),
                HashSet::new(),
                HashMap::new(),
            );
        }
        self.prefetch_graph_modules_lazy(entry_abs)
    }

    /// Pre-#1991 crawl: every specifier a module resolves is pushed onto
    /// the next wave's frontier unconditionally, including a pure
    /// re-export barrel's own leaves regardless of whether any importer
    /// actually requested them. Kept verbatim as the `JET_EAGER_BARRELS`
    /// escape hatch.
    fn prefetch_graph_modules_eager(&self, entry_abs: &Path) -> HashMap<PathBuf, PrefetchedModule> {
        use rayon::prelude::*;

        let mut prefetched: HashMap<PathBuf, PrefetchedModule> = HashMap::new();
        let mut frontier: Vec<PathBuf> = vec![entry_abs.to_path_buf()];

        while !frontier.is_empty() {
            let wave: Vec<(PathBuf, PrefetchedModule)> = frontier
                .par_iter()
                .map(|path| (path.clone(), self.prefetch_one_module(path)))
                .collect();

            let mut next: Vec<PathBuf> = Vec::new();
            for (path, module) in wave {
                for res in module.resolutions.values() {
                    if let Ok(target) = res {
                        if !prefetched.contains_key(target) {
                            next.push(target.clone());
                        }
                    }
                }
                prefetched.insert(path, module);
            }
            next.sort_unstable();
            next.dedup();
            next.retain(|p| !prefetched.contains_key(p));
            frontier = next;
        }

        prefetched
    }

    /// Lazy pure-barrel expansion (#1991): a *pure re-export barrel* — a
    /// module whose only content is `export { ... } from '...'` /
    /// `export * from '...'` lines plus comments/whitespace/`'use
    /// strict'` (see [`is_pure_barrel_source`]) — has its own outgoing
    /// edges pushed onto the crawl frontier only for the barrel-exposed
    /// names some already-discovered importer actually demands. A
    /// barrel's own file is always read (it is a real, direct
    /// dependency reached like any other module); only its *unrequested
    /// leaves* are skipped, so this returns a strict subset of what
    /// [`Self::prefetch_graph_modules_eager`] would prefetch, plus the skip
    /// set [`Self::build_graph`] needs to keep those leaves out of the
    /// final graph.
    ///
    /// Two-phase per wave, to make the "two importers of the same barrel
    /// land in the same wave" race safe: phase 1 (borrowing this wave's
    /// modules) records every resolved edge's demand into `barrel_demand`
    /// — skipping a wave module's own resolutions when that module is
    /// itself a confirmed pure barrel, since a barrel's re-export lines
    /// are not ordinary "this module imports names" edges — so a
    /// same-wave importer's request is never missed before any of this
    /// wave's own barrels are evaluated for expansion. Phase 2 (after the
    /// wave is absorbed into `prefetched`) walks every path touched this
    /// wave — newly-discovered targets and modules just absorbed — and,
    /// for each confirmed pure barrel among them, expands exactly the
    /// leaves its accumulated demand covers. `expanded_from` remembers
    /// which targets have already been queued from a given module so a
    /// later wave's incremental expansion (a fresh importer requesting
    /// more names from an already-partially-expanded barrel) only queues
    /// the newly-due leaves.
    ///
    /// Fallback to full expansion (matching [`Self::prefetch_graph_modules_eager`]
    /// for that one edge) whenever demand can't be proven narrow: a
    /// namespace import (`import * as ns`), a dynamic `import()`, a bare
    /// (non-property) CJS `require()` use, a star re-export target
    /// (leaf-side names unknown without reading the target — this covers
    /// both a star line inside a pure barrel and `export *` from a barrel
    /// into another module; #1991 does not implement the issue's optional
    /// single-hop pure-barrel-chain narrowing, see the module doc), a
    /// requested name with no matching entry in the barrel's own parsed
    /// re-export map (a safety net against this line-scanner
    /// under-parsing an unusual barrel shape), or a resolved specifier
    /// this scan can't account for in the source text at all (same
    /// safety net, for the importer side).
    fn prefetch_graph_modules_lazy(
        &self,
        entry_abs: &Path,
    ) -> (
        HashMap<PathBuf, PrefetchedModule>,
        HashSet<PathBuf>,
        HashMap<PathBuf, BarrelDemand>,
    ) {
        use rayon::prelude::*;

        let mut prefetched: HashMap<PathBuf, PrefetchedModule> = HashMap::new();
        let mut frontier: Vec<PathBuf> = vec![entry_abs.to_path_buf()];

        // Accumulated per-barrel demand (grows monotonically; `Full` once
        // anything can't be narrowed) and, per expanding module, which of
        // its targets have already been queued (so re-visiting a barrel
        // across waves only queues newly-due leaves).
        let mut barrel_demand: HashMap<PathBuf, BarrelDemand> = HashMap::new();
        let mut expanded_from: HashMap<PathBuf, HashSet<PathBuf>> = HashMap::new();
        let mut barrels_detected: HashSet<PathBuf> = HashSet::new();
        // Diagnostics only (#1991 round 2): the first escalation reason
        // recorded per barrel, for `JET_BUNDLE_TIMING`'s
        // `lazy-barrels escalated-full:` report. Never consulted by the
        // crawl's own fallback-to-full decisions.
        let mut escalation_reasons: HashMap<PathBuf, EscalationReason> = HashMap::new();

        while !frontier.is_empty() {
            let wave: Vec<(PathBuf, PrefetchedModule)> = frontier
                .par_iter()
                .map(|path| (path.clone(), self.prefetch_one_module(path)))
                .collect();

            // Phase 1: record every edge's demand before any of this
            // wave's own targets are evaluated for expansion.
            let mut touched: Vec<PathBuf> = wave.iter().map(|(path, _)| path.clone()).collect();
            for (_path, module) in &wave {
                let Ok(imports) = &module.imports else {
                    continue;
                };
                let Ok(src) = &module.source else {
                    continue;
                };
                if is_pure_barrel_source(src) {
                    // This wave module is itself a confirmed pure barrel:
                    // its resolutions are re-export edges, handled by
                    // phase 2's barrel branch below, not ordinary demand.
                    continue;
                }
                for (spec, res) in &module.resolutions {
                    let Ok(target) = res else { continue };
                    let is_dynamic = imports.dynamic_imports.iter().any(|d| d == spec);
                    let demand =
                        match barrel_demand_for_specifier_with_reason(src, spec, is_dynamic) {
                            Ok(names) => Some(names),
                            Err(reason) => {
                                escalation_reasons.entry(target.clone()).or_insert(reason);
                                None
                            }
                        };
                    merge_barrel_demand(&mut barrel_demand, target, demand);
                    touched.push(target.clone());
                }
            }

            for (path, module) in wave {
                prefetched.insert(path, module);
            }

            // Phase 2: expand every touched confirmed-pure barrel by its
            // accumulated demand; everything else behaves exactly like
            // the eager crawl.
            touched.sort_unstable();
            touched.dedup();

            let mut next: Vec<PathBuf> = Vec::new();
            for target in touched {
                let Some(module) = prefetched.get(&target) else {
                    next.push(target.clone());
                    continue;
                };
                let Ok(src) = &module.source else { continue };
                if !is_pure_barrel_source(src) {
                    // Not a pure barrel: every resolution this module
                    // makes is a real dependency, exactly like the eager
                    // crawl. `expanded_from` doubles as a general
                    // "already queued from here" memo so a repeat visit
                    // is a no-op.
                    let pushed = expanded_from.entry(target.clone()).or_default();
                    for res in module.resolutions.values() {
                        if let Ok(dep) = res {
                            if pushed.insert(dep.clone()) && !prefetched.contains_key(dep) {
                                next.push(dep.clone());
                            }
                        }
                    }
                    continue;
                }

                barrels_detected.insert(target.clone());
                let entries = tree_shake::extract_reexport_specifiers(src);
                let known_names: HashSet<&str> = entries
                    .iter()
                    .filter_map(|(_, kind)| match kind {
                        tree_shake::ReexportKind::Named(pairs) => {
                            Some(pairs.iter().map(|(_, barrel_name)| barrel_name.as_str()))
                        }
                        tree_shake::ReexportKind::Star => None,
                    })
                    .flatten()
                    .collect();
                let demand = barrel_demand
                    .get(&target)
                    .cloned()
                    .unwrap_or(BarrelDemand::Full);
                let effective_full = match &demand {
                    BarrelDemand::Full => true,
                    // A requested name with no home in the barrel's own
                    // parsed map is "any doubt → eager": either a
                    // genuinely broken import (harmless to over-expand)
                    // or this line-scanner missed an unusual shape.
                    BarrelDemand::Names(names) => {
                        let unresolvable = names.iter().any(|n| !known_names.contains(n.as_str()));
                        if unresolvable {
                            escalation_reasons
                                .entry(target.clone())
                                .or_insert(EscalationReason::UnresolvableName);
                        }
                        unresolvable
                    }
                };

                let pushed = expanded_from.entry(target.clone()).or_default();
                for (spec, kind) in &entries {
                    let Some(Ok(leaf_target)) = module.resolutions.get(spec) else {
                        continue;
                    };
                    let leaf_names: Option<Vec<String>> = match kind {
                        tree_shake::ReexportKind::Star => {
                            escalation_reasons
                                .entry(leaf_target.clone())
                                .or_insert(EscalationReason::ExportStarChain);
                            None // always full — see fn doc.
                        }
                        tree_shake::ReexportKind::Named(pairs) => {
                            if effective_full {
                                Some(pairs.iter().map(|(leaf, _)| leaf.clone()).collect())
                            } else if let BarrelDemand::Names(names) = &demand {
                                let matched: Vec<String> = pairs
                                    .iter()
                                    .filter(|(_, barrel_name)| names.contains(barrel_name))
                                    .map(|(leaf, _)| leaf.clone())
                                    .collect();
                                if matched.is_empty() {
                                    continue; // nothing demanded from this entry yet.
                                }
                                Some(matched)
                            } else {
                                continue; // unreachable: effective_full covers BarrelDemand::Full.
                            }
                        }
                    };
                    merge_barrel_demand(&mut barrel_demand, leaf_target, leaf_names);
                    if pushed.insert(leaf_target.clone()) && !prefetched.contains_key(leaf_target) {
                        next.push(leaf_target.clone());
                    }
                }
            }

            next.sort_unstable();
            next.dedup();
            next.retain(|p| !prefetched.contains_key(p));
            frontier = next;
        }

        // Final derivation pass: any re-export entry on a confirmed
        // barrel whose resolved target never made it into `prefetched`
        // was deliberately skipped — this is exactly what
        // `Self::build_graph`'s replay must exclude instead of
        // synchronously reading fresh.
        let mut skipped: HashSet<PathBuf> = HashSet::new();
        for barrel_path in &barrels_detected {
            let Some(module) = prefetched.get(barrel_path) else {
                continue;
            };
            let Ok(src) = &module.source else { continue };
            for (spec, kind) in tree_shake::extract_reexport_specifiers(src) {
                if matches!(kind, tree_shake::ReexportKind::Star) {
                    continue;
                }
                if let Some(Ok(leaf_target)) = module.resolutions.get(&spec) {
                    if !prefetched.contains_key(leaf_target) {
                        skipped.insert(leaf_target.clone());
                    }
                }
            }
        }

        if std::env::var_os("JET_BUNDLE_TIMING").is_some() {
            eprintln!(
                "[bundle-timing] lazy-barrels: {} barrel(s) detected, {} leaf(ves) skipped",
                barrels_detected.len(),
                skipped.len()
            );
            print_barrel_escalation_report(&barrels_detected, &escalation_reasons, &prefetched);
        }
        tracing::debug!(
            barrels_detected = barrels_detected.len(),
            leaves_skipped = skipped.len(),
            "lazy pure-barrel expansion (#1991)"
        );

        // Restrict to confirmed pure barrels: `barrel_demand` also carries
        // entries for ordinary (non-barrel) targets touched during Phase 1
        // demand recording, which `transform_modules` must never prune
        // against — only a module `is_pure_barrel_source` independently
        // reconfirms at transform time is eligible.
        let confirmed_demand: HashMap<PathBuf, BarrelDemand> = barrel_demand
            .into_iter()
            .filter(|(path, _)| barrels_detected.contains(path))
            .collect();

        (prefetched, skipped, confirmed_demand)
    }

    /// The pure per-module slice of `build_graph`'s loop body: read the
    /// source, extract imports when it is a script module, and resolve
    /// every specifier the replay will ask about (implicit jsx-runtime,
    /// static, dynamic) through the shared resolver.
    fn prefetch_one_module(&self, module_path: &Path) -> PrefetchedModule {
        let source = std::fs::read_to_string(module_path).map_err(|e| e.to_string());
        // #1999: seed the per-build source cache from the crawl's own read so
        // every downstream phase (`compute_transform_survivors`,
        // `collect_side_effect_free_module_indices`, `transform_modules`,
        // `apply_tree_shaking`'s recompute path) can reuse these bytes
        // instead of re-reading the file from disk. See `Bundler::
        // source_cache`'s doc comment for the reset/consult contract.
        if std::env::var_os("JET_NO_SOURCE_CACHE").is_none() {
            if let Ok(src) = &source {
                self.source_cache
                    .lock()
                    .insert(module_path.to_path_buf(), Arc::from(src.as_str()));
            }
        }
        let mut resolutions: HashMap<String, std::result::Result<PathBuf, String>> = HashMap::new();
        let mut imports: std::result::Result<imports::ModuleImports, String> =
            Err("not a script module".to_string());
        let mut tree: Option<tree_sitter::Tree> = None;
        let mut used_fast_import_scan: Option<bool> = None;

        if determine_module_kind(&module_path.to_path_buf()) == graph::ModuleKind::Script {
            if let Ok(src) = &source {
                let ext = module_path.extension().and_then(|e| e.to_str());
                let is_typescript = matches!(ext, Some("ts") | Some("tsx"));
                let is_jsx = matches!(ext, Some("tsx") | Some("jsx"));
                // Plain-JS source is not rewritten before the module transform,
                // so its JS-grammar parse can be reused there. Keep the tree only
                // for those modules; TS/TSX/JSX get re-parsed post-rewrite anyway.
                let reusable = matches!(ext, Some("js") | Some("cjs") | Some("mjs"));
                // #1997: try the cheap string-scan fast path first; a tree is
                // never produced there, so `tree` stays `None` for a fast-scanned
                // module and the transform stage re-parses it if needed --
                // exactly like it already does for every TS/TSX/JSX module today.
                // #2140: `scan_module_imports_cached` additionally consults the
                // persistent import-scan cache before running either scan (a
                // no-op when the cache is disabled, e.g. dev server/`--lib`).
                let (scanned_imports, scanned_tree, scanned_fast_flag) =
                    self.scan_module_imports_cached(src, module_path, is_typescript, reusable);
                imports = scanned_imports;
                tree = scanned_tree;
                used_fast_import_scan = scanned_fast_flag;
                if let Ok(module_imports) = &imports {
                    let mut specs: Vec<&str> = Vec::new();
                    if is_jsx {
                        specs.push("react/jsx-runtime");
                    }
                    specs.extend(
                        module_imports
                            .static_imports
                            .iter()
                            .map(|d| d.source.as_str()),
                    );
                    specs.extend(module_imports.dynamic_imports.iter().map(String::as_str));
                    for spec in specs {
                        if resolutions.contains_key(spec) {
                            continue;
                        }
                        let resolved = self
                            .resolve_dependency(&module_path.to_path_buf(), spec)
                            .map_err(|e| e.to_string());
                        resolutions.insert(spec.to_string(), resolved);
                    }
                }
            }
        }

        PrefetchedModule {
            source,
            imports,
            resolutions,
            tree,
            used_fast_import_scan,
        }
    }
}

/// Accumulated demand on a pure barrel's own re-exported names, across
/// however many importers/waves reference it (#1991). Monotonic: once
/// `Full`, stays `Full`; `Names` only grows via [`merge_barrel_demand`].
#[derive(Debug, Clone)]
enum BarrelDemand {
    Names(HashSet<String>),
    Full,
}

/// Merge a newly-discovered demand onto `target`'s accumulated entry in
/// `demand_map`, in place. `new_names = None` means the edge that produced
/// this demand could not be narrowed (one of #1991's fallback cases) and
/// escalates the whole barrel to [`BarrelDemand::Full`]; `Some(names)`
/// (possibly empty, e.g. a bare side-effect import) extends the
/// accumulated name set unless it is already `Full`.
fn merge_barrel_demand(
    demand_map: &mut HashMap<PathBuf, BarrelDemand>,
    target: &Path,
    new_names: Option<Vec<String>>,
) {
    let entry = demand_map
        .entry(target.to_path_buf())
        .or_insert_with(|| BarrelDemand::Names(HashSet::new()));
    if matches!(entry, BarrelDemand::Full) {
        return;
    }
    match new_names {
        None => *entry = BarrelDemand::Full,
        Some(names) => {
            if let BarrelDemand::Names(existing) = entry {
                existing.extend(names);
            }
        }
    }
}

/// Why a barrel's demand could not be narrowed and escalated to
/// [`BarrelDemand::Full`] (#1991 round 2 diagnostics — see
/// [`barrel_demand_for_specifier_with_reason`] and
/// [`print_barrel_escalation_report`]). Purely informational: the crawl's
/// correctness contract (demand uncertainty -> Full, never drop a used
/// leaf) does not depend on this classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EscalationReason {
    /// `import * as ns from '<barrel>'` — every leaf may be referenced via
    /// the namespace object, so nothing can be narrowed.
    NamespaceImport,
    /// `require('<barrel>')` bound to an identifier whose properties are
    /// never scanned back to a finite set (no destructure, no discoverable
    /// `.prop`/`['prop']` access anywhere in the module).
    BareCjsUse,
    /// `import('<barrel>')` — a dynamic import target is never narrowed.
    DynamicImport,
    /// `export ... from '<barrel>'` (star or named) re-export chains the
    /// barrel into another module instead of a leaf consumer using it
    /// directly; the chain is not followed for narrowing.
    ExportStarChain,
    /// A requested name has no home in the barrel's own parsed re-export
    /// map — either a genuinely broken import or a shape this line
    /// scanner missed; any doubt escalates to full.
    UnresolvableName,
    /// An `import` statement referencing the barrel could not be scanned
    /// to a specifier/name list at all (e.g. an unterminated multi-line
    /// statement with no `from '<spec>'` before EOF).
    UnparseableImport,
    /// The resolved specifier never matched any import/require line this
    /// scanner recognizes in the importer's source — the crawl still saw
    /// the edge (via AST-based resolution), but this text scan could not
    /// account for it.
    NoDemandRecorded,
}

impl EscalationReason {
    fn label(self) -> &'static str {
        match self {
            EscalationReason::NamespaceImport => "namespace-import",
            EscalationReason::BareCjsUse => "bare-cjs-use",
            EscalationReason::DynamicImport => "dynamic-import",
            EscalationReason::ExportStarChain => "export-star-chain",
            EscalationReason::UnresolvableName => "unresolvable-name",
            EscalationReason::UnparseableImport => "unparseable-import",
            EscalationReason::NoDemandRecorded => "no-demand-recorded",
        }
    }
}

/// Pure re-export barrel detector (#1991): true only when every statement
/// in `source` is a named/star re-export (`export { a, b } from '...'` /
/// `export * from '...'`), blank, a `//` or `/* ... */` comment, or a
/// `'use strict'` / `"use strict"` directive. Any other statement — a real
/// export declaration (`export function`, `export const`, `export
/// default`, ...), a plain `import`, or any executable code — makes the
/// module NOT pure, and its outgoing edges expand eagerly for that one
/// module (this function only decides purity; the crawl in
/// [`Bundler::prefetch_graph_modules_lazy`] does the actual gating).
///
/// The final `export_from_lines == extract_reexport_specifiers(..).len()`
/// check catches an export-from-shaped line that fails
/// `tree_shake::classify_reexport_line`'s stricter parse (e.g. an
/// unbalanced brace group) — if the cheap textual scan and the real
/// classifier disagree on how many re-export lines exist, this
/// conservatively reports NOT a pure barrel rather than trusting the
/// looser count.
fn is_pure_barrel_source(source: &str) -> bool {
    let mut in_block_comment = false;
    let mut export_from_lines = 0usize;

    for line in source.lines() {
        let trimmed = line.trim();

        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with("/*") {
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
            continue;
        }
        if trimmed == "'use strict';"
            || trimmed == "\"use strict\";"
            || trimmed == "'use strict'"
            || trimmed == "\"use strict\""
        {
            continue;
        }
        if trimmed.starts_with("export ") && trimmed.contains(" from ") {
            export_from_lines += 1;
            continue;
        }
        return false;
    }

    export_from_lines > 0
        && export_from_lines == tree_shake::extract_reexport_specifiers(source).len()
}

/// Prunes a confirmed pure barrel's own source, dropping named re-export
/// lines (`export { a, b as c } from '...'`) whose barrel-exposed names are
/// all absent from `demand` (#1991). Star re-export lines (`export * from
/// '...'`) are always kept — their leaf names are unknown without reading
/// the target, matching the "star always expands" fallback the crawl
/// itself already applies — as is any line the same-shaped scan in
/// [`is_pure_barrel_source`] wouldn't classify as a bare re-export (there
/// shouldn't be any in a confirmed-pure source, but leaving anything
/// unrecognized untouched is the conservative choice).
///
/// Only called with `BarrelDemand::Names(..)`; a `Full` demand means every
/// leaf was already crawled, so the barrel's original source is already
/// byte-for-byte what the eager crawl would transform and must not be
/// touched. Line-granularity only (a multi-name line survives whole if any
/// one of its names is demanded) — matches [`is_pure_barrel_source`]'s own
/// line-oriented scan, and real-world barrels (e.g. `@mui/icons-material`)
/// are one name per line.
fn prune_barrel_source_to_demand(source: &str, demand: &HashSet<String>) -> String {
    let mut out = String::with_capacity(source.len());
    let mut in_block_comment = false;

    for line in source.lines() {
        let trimmed = line.trim();

        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if trimmed.starts_with("/*") && !trimmed.contains("*/") {
            in_block_comment = true;
            out.push_str(line);
            out.push('\n');
            continue;
        }

        if let Some((_spec, kind)) = tree_shake::classify_reexport_line(trimmed) {
            match kind {
                tree_shake::ReexportKind::Star => {
                    out.push_str(line);
                    out.push('\n');
                }
                tree_shake::ReexportKind::Named(pairs) => {
                    if pairs
                        .iter()
                        .any(|(_, barrel_name)| demand.contains(barrel_name))
                    {
                        out.push_str(line);
                        out.push('\n');
                    }
                    // else: every name this line exposes is undemanded —
                    // drop it, matching what tree-shake would have pruned
                    // had the leaf been crawled and found unused.
                }
            }
            continue;
        }

        out.push_str(line);
        out.push('\n');
    }

    out
}

/// Demand a single resolved specifier's import site(s) in `source` place on
/// its target, for lazy pure-barrel expansion (#1991). Thin `Option`
/// wrapper over [`barrel_demand_for_specifier_with_reason`] for callers
/// that don't need the escalation reason; see that function for the full
/// contract. Production crawl code (`prefetch_graph_modules_lazy`) calls
/// `barrel_demand_for_specifier_with_reason` directly to record escalation
/// diagnostics (#1991 round 2) — this wrapper is kept `#[cfg(test)]`-only
/// so the round-1 test suite's existing `Option`-shaped call sites stay
/// green unmodified.
#[cfg(test)]
fn barrel_demand_for_specifier(source: &str, spec: &str, is_dynamic: bool) -> Option<Vec<String>> {
    barrel_demand_for_specifier_with_reason(source, spec, is_dynamic).ok()
}

/// Demand a single resolved specifier's import site(s) in `source` place on
/// its target, for lazy pure-barrel expansion (#1991). `Err(reason)` means
/// the specifier could not be proven narrow — the caller must fall back to
/// full/eager expansion for that edge, matching the issue's fallback list:
/// namespace import (`import * as ns`), dynamic `import()` (handled by the
/// caller via `is_dynamic` before this scan runs), bare (non-property) CJS
/// `require()` use, or a specifier this text scan can't otherwise account
/// for at all (defensive — the AST-derived resolver found it, so an
/// unrecognized shape here means this line-scanner under-parsed it, not
/// that nothing imports it; see [`classify_unmatched_barrel_specifier`] for
/// that last case's `reason`). `reason` is diagnostics only (#1991 round 2
/// — see [`EscalationReason`]) and never changes the fallback contract.
///
/// Scans [`tree_shake::logical_import_lines`] rather than `source.lines()`
/// directly, so a multi-line `import { ... } from '...'` statement —
/// prettier's default wrapping for 3+ named imports, whose binding list and
/// `from` clause never share a physical line — narrows exactly like its
/// single-line-equivalent form instead of silently escalating the barrel to
/// full (#1991 round 2; matches the join/scan approach
/// [`tree_shake::extract_import_bindings`] now also uses, rather than a
/// second, divergent parser).
///
/// Reuses `tree_shake`'s existing line-oriented CJS analysis
/// (`scan_require_call`, `extract_destructured_names`,
/// `extract_require_local_binding`, `scan_require_binding_property_accesses`)
/// rather than `tree_shake::extract_cjs_require_bindings`, whose
/// resolve-via-lookup-first short-circuit is a deliberate hot-loop
/// optimization (#1947 round 2) that must stay lookup-driven; this
/// function instead runs lookup-free, before a `ModuleLookup` exists, and
/// is scoped to one already-resolved specifier.
fn barrel_demand_for_specifier_with_reason(
    source: &str,
    spec: &str,
    is_dynamic: bool,
) -> std::result::Result<Vec<String>, EscalationReason> {
    if is_dynamic {
        return Err(EscalationReason::DynamicImport);
    }

    let mut names: Vec<String> = Vec::new();
    let mut matched = false;

    for line in tree_shake::logical_import_lines(source) {
        let trimmed = line.trim();

        if trimmed.starts_with("import ") {
            if tree_shake::extract_specifier(trimmed) != spec {
                continue;
            }
            matched = true;
            let imported = tree_shake::extract_imported_names(trimmed);
            if imported.iter().any(|n| n == "*") {
                return Err(EscalationReason::NamespaceImport);
            }
            names.extend(imported); // may be empty (side-effect-only import).
            continue;
        }

        if let Some((req_source, accessed)) = tree_shake::scan_require_call(trimmed) {
            if req_source != spec {
                continue;
            }
            matched = true;
            let destructured = tree_shake::extract_destructured_names(trimmed);
            if destructured.is_empty() && accessed.is_empty() {
                if let Some(binding) = tree_shake::extract_require_local_binding(trimmed) {
                    if let Some(props) = tree_shake::scan_require_binding_property_accesses(
                        source, trimmed, &binding,
                    ) {
                        names.extend(props);
                        continue;
                    }
                }
                // Bare require use (no destructure, no property access
                // found anywhere in the module) — fall back to full.
                return Err(EscalationReason::BareCjsUse);
            }
            names.extend(destructured);
            names.extend(accessed);
        }
    }

    if matched {
        return Ok(names);
    }
    Err(classify_unmatched_barrel_specifier(source, spec))
}

/// Classifies why [`barrel_demand_for_specifier_with_reason`] found no
/// import/require line accounting for `spec` at all — diagnostics only
/// (#1991 round 2), consulted after the main scan already failed to match.
/// Distinguishes an unterminated multi-line `import` statement (no `from
/// '<spec>'` before EOF — [`EscalationReason::UnparseableImport`]) and an
/// `export ... from '<spec>'` re-export chain
/// ([`EscalationReason::ExportStarChain`], covering both star and named
/// shapes) from the default catch-all
/// ([`EscalationReason::NoDemandRecorded`]): the AST-derived resolver found
/// this edge, so a truly unrecognized shape here means this line scanner
/// under-parsed it, not that nothing imports it.
fn classify_unmatched_barrel_specifier(source: &str, spec: &str) -> EscalationReason {
    let mut saw_incomplete_import = false;
    for line in tree_shake::logical_import_lines(source) {
        let trimmed = line.trim();
        if trimmed.starts_with("import ") && tree_shake::extract_specifier(trimmed).is_empty() {
            saw_incomplete_import = true;
            continue;
        }
        if trimmed.starts_with("export ") && trimmed.contains(" from ") {
            if let Some((re_spec, _kind)) = tree_shake::classify_reexport_line(trimmed) {
                if re_spec == spec {
                    return EscalationReason::ExportStarChain;
                }
            }
        }
    }
    if saw_incomplete_import {
        EscalationReason::UnparseableImport
    } else {
        EscalationReason::NoDemandRecorded
    }
}

/// Top 5 Full-escalated barrels by leaf (re-export) count, largest first,
/// for [`print_barrel_escalation_report`] (#1991 round 2). A barrel with no
/// recorded [`EscalationReason`] narrowed cleanly and is excluded — the
/// smell this instrumentation exists to surface is
/// [`EscalationReason::NoDemandRecorded`] firing for a barrel with known
/// consumers.
fn top_barrel_escalations<'a>(
    barrels_detected: &'a HashSet<PathBuf>,
    escalation_reasons: &HashMap<PathBuf, EscalationReason>,
    prefetched: &HashMap<PathBuf, PrefetchedModule>,
) -> Vec<(&'a PathBuf, EscalationReason, usize)> {
    let mut escalated: Vec<(&PathBuf, EscalationReason, usize)> = barrels_detected
        .iter()
        .filter_map(|path| {
            let reason = *escalation_reasons.get(path)?;
            let leaf_count = prefetched
                .get(path)
                .and_then(|module| module.source.as_ref().ok())
                .map(|src| tree_shake::extract_reexport_specifiers(src).len())
                .unwrap_or(0);
            Some((path, reason, leaf_count))
        })
        .collect();
    escalated.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(b.0)));
    escalated.truncate(5);
    escalated
}

/// Prints the top 5 Full-escalated barrels (see [`top_barrel_escalations`])
/// as sibling `JET_BUNDLE_TIMING` lines to the existing `lazy-barrels:`
/// summary line (#1991 round 2). No-op (prints nothing) when no barrel
/// escalated to full.
fn print_barrel_escalation_report(
    barrels_detected: &HashSet<PathBuf>,
    escalation_reasons: &HashMap<PathBuf, EscalationReason>,
    prefetched: &HashMap<PathBuf, PrefetchedModule>,
) {
    for (path, reason, leaf_count) in
        top_barrel_escalations(barrels_detected, escalation_reasons, prefetched)
    {
        eprintln!(
            "[bundle-timing] lazy-barrels escalated-full: {} ({} leaf(ves)) reason={}",
            path.display(),
            leaf_count,
            reason.label()
        );
    }
}

impl Bundler {
    /// Remove TypeScript imports that the existing transform erases before
    /// runtime code generation. Graph construction happens first, so treating
    /// those declarations as dependencies would incorrectly resolve a
    /// declaration-only `.d.ts` file (or an unexported package type path) as a
    /// runtime module.
    ///
    /// Reusing the transform as the semantic source of truth covers both
    /// explicit `import type` syntax and ordinary PascalCase imports that are
    /// only referenced from type positions. If transformation or re-parsing
    /// fails, retain the original imports and let the normal diagnostic path
    /// report the problem rather than silently hiding a value dependency.
    fn runtime_static_imports(
        &self,
        source: &str,
        module_path: &Path,
        is_typescript: bool,
        mut module_imports: imports::ModuleImports,
    ) -> imports::ModuleImports {
        if !is_typescript || module_imports.static_imports.is_empty() {
            return module_imports;
        }

        let Ok(transformed) = self.transformer.transform_js(source, module_path) else {
            return module_imports;
        };
        // #1997: this narrowing step already pays for a full retransform of
        // the TS source, so trying the string-scan fast path on the
        // resulting (guaranteed-plain-JS) output first is a second, cheap
        // win independent of the primary crawl call site above.
        let runtime_imports = match imports::extract_imports_fast(&transformed.code) {
            Some(runtime_imports) => runtime_imports,
            None => {
                let Ok(runtime_imports) = imports::extract_imports(&transformed.code, false) else {
                    return module_imports;
                };
                runtime_imports
            }
        };
        let runtime_sources: std::collections::HashSet<String> = runtime_imports
            .static_imports
            .into_iter()
            .map(|import| import.source)
            .collect();

        module_imports
            .static_imports
            .retain(|import| runtime_sources.contains(&import.source));
        module_imports
    }

    /// #2140 — content-addressed import-scan cache consult, shared by
    /// `prefetch_one_module` and `build_graph`'s synchronous fallback (the
    /// two `extract_imports_fast`/fallback call sites). On a persistent-
    /// cache hit, returns the already-`runtime_static_imports`-narrowed
    /// `ModuleImports` with neither scan function invoked — mirrors
    /// `used_fast_import_scan: None`'s existing "extraction never
    /// attempted" meaning (now also true of a cache hit: there was nothing
    /// to attempt). On a miss, runs the exact same fast-path-first pipeline
    /// both call sites already ran before this WI, then inserts the
    /// narrowed result so a later build — or another module with
    /// byte-identical content — hits.
    ///
    /// Only pays for the content hash + cache lookup when the persistent
    /// cache is actually enabled (`jet build`'s one-shot CLI path);
    /// dev-server/`--lib`/`--nx` never set `cache_project_root`, so
    /// `enabled()` is `false` there and this degrades to exactly the
    /// pre-#2140 scan with no added cost.
    ///
    /// Returns `(imports, tree, used_fast_import_scan)`: `tree` is `Some`
    /// only for a fresh tree-sitter-fallback parse of a `reusable`
    /// (plain-JS) module — never populated on a cache hit, since no parse
    /// happened; the transform stage already re-parses on demand for every
    /// module that also lacks a tree today (every fast-scanned module), so
    /// this is a same-process perf side-channel, not a persisted "product".
    fn scan_module_imports_cached(
        &self,
        source: &str,
        module_path: &Path,
        is_typescript: bool,
        reusable: bool,
    ) -> (
        std::result::Result<imports::ModuleImports, String>,
        Option<tree_sitter::Tree>,
        Option<bool>,
    ) {
        let scan_key = self
            .persistent_cache
            .enabled()
            .then(|| persistent_cache::ImportScanKey {
                content_hash: persistent_cache::hash_str(source),
                is_typescript,
            });
        if let Some(key) = &scan_key {
            if let Some(cached) = self.persistent_cache.get_import_scan(key) {
                return (Ok(cached), None, None);
            }
        }

        let mut tree: Option<tree_sitter::Tree> = None;
        let mut used_fast_import_scan = Some(true);
        let raw = match imports::extract_imports_fast(source) {
            Some(module_imports) => module_imports,
            None => {
                used_fast_import_scan = Some(false);
                match imports::extract_imports_with_tree(source, is_typescript) {
                    Ok((module_imports, parsed)) => {
                        if reusable {
                            tree = Some(parsed);
                        }
                        module_imports
                    }
                    Err(e) => return (Err(e.to_string()), None, used_fast_import_scan),
                }
            }
        };
        let narrowed = self.runtime_static_imports(source, module_path, is_typescript, raw);
        if let Some(key) = scan_key {
            self.persistent_cache
                .insert_import_scan(key, narrowed.clone());
        }
        (Ok(narrowed), tree, used_fast_import_scan)
    }

    /// Read-through helper for `source_cache`: a cache hit (populated by the
    /// crawl's `prefetch_one_module`) returns the shared `Arc<str>` with no
    /// filesystem access; a miss falls back to `fs::read_to_string` and
    /// back-fills the cache so later readers in the same build also hit.
    /// `JET_NO_SOURCE_CACHE=1` bypasses the cache entirely (every call reads
    /// straight through to disk) for a byte-identity A/B diff against the
    /// pre-#1999 always-read-from-disk behavior.
    /// @issue #1999
    fn cached_source(&self, path: &Path) -> std::result::Result<Arc<str>, std::io::Error> {
        if std::env::var_os("JET_NO_SOURCE_CACHE").is_some() {
            return std::fs::read_to_string(path).map(|s| Arc::from(s.as_str()));
        }
        if let Some(cached) = self.source_cache.lock().get(path) {
            return Ok(cached.clone());
        }
        let src: Arc<str> = Arc::from(std::fs::read_to_string(path)?.as_str());
        self.source_cache
            .lock()
            .insert(path.to_path_buf(), src.clone());
        Ok(src)
    }

    async fn build_graph(&self, entry: &PathBuf) -> Result<()> {
        tracing::debug!("Building module graph from: {:?}", entry);

        let entry_abs = std::fs::canonicalize(entry)?;

        // WI #1995 round 4: reset the survivors-only-transform side channel
        // for this crawl. `entry_path` seeds
        // `compute_transform_survivors`' liveness walk with the same
        // canonicalized path this function seeds the graph with;
        // `implicit_edges` is repopulated below as the crawl fabricates
        // non-textual edges (currently just `react/jsx-runtime`). Round 5
        // adds `shake_analysis`: the cached `TreeShakeResult` from this
        // crawl's survivors pre-pass, reused by `apply_tree_shaking` so the
        // same analysis isn't recomputed a second time. It must be cleared
        // here too — stale results from a previous `build_graph` call must
        // never leak into this crawl's `apply_tree_shaking`.
        *self.entry_path.lock() = Some(entry_abs.clone());
        self.implicit_edges.lock().clear();
        *self.shake_analysis.lock() = None;
        // #1999: same reset rule — this build's crawl is about to repopulate
        // `source_cache` from scratch; a previous `build_graph` call's
        // snapshot must never leak into this one (dev-server re-bundle after
        // a file changes).
        self.source_cache.lock().clear();

        // Wave-parallel prefetch of the expensive pure work (file read,
        // tree-sitter import extraction, dependency resolution). The serial
        // walk below replays over these memos, so module-id assignment
        // order — and therefore bundle bytes — stay identical to the
        // sequential traversal while the dominant costs run across cores.
        // `barrel_skipped_leaves` (#1991) names every pure-barrel leaf the
        // prefetch deliberately never fetched because no importer demanded
        // it — the replay below must exclude those rather than resolve
        // them via a barrel's own (always-complete) resolution memo and
        // silently read them fresh one at a time.
        let (mut prefetched, barrel_skipped_leaves, barrel_demand) =
            self.prefetch_graph_modules(&entry_abs);
        // `transform_modules` prunes a confirmed barrel's own unrequested
        // re-export lines against this map before codegen (#1991) — see
        // `Bundler::barrel_demand`'s doc comment.
        *self.barrel_demand.lock() = barrel_demand;

        // #1997 — how many crawled modules the string-scan fast path
        // (`imports::extract_imports_fast`) covered vs. how many still
        // needed the tree-sitter fallback, as a JET_BUNDLE_TIMING sibling
        // line. Modules where extraction was never attempted (non-script
        // kinds, unreadable files) are excluded from both counts.
        if std::env::var_os("JET_BUNDLE_TIMING").is_some() {
            let fast = prefetched
                .values()
                .filter(|p| p.used_fast_import_scan == Some(true))
                .count();
            let fallback = prefetched
                .values()
                .filter(|p| p.used_fast_import_scan == Some(false))
                .count();
            // #2140: i_hits/i_misses count persistent import-scan cache
            // lookups across both `prefetch_one_module` and this
            // function's own synchronous fallback branch below.
            eprintln!(
                "[bundle-timing] import-scan: fast={fast} fallback={fallback} i_hits={} i_misses={}",
                self.persistent_cache.import_scan_hits(),
                self.persistent_cache.import_scan_misses(),
            );
        }

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

            // Move the reusable parse tree (plain-JS only) out of the prefetch
            // memo into the shared map so transform_modules can skip re-parsing.
            if let Some(t) = prefetched.get_mut(&module_path).and_then(|p| p.tree.take()) {
                self.parsed_trees.lock().insert(module_path.clone(), t);
            }

            let prefetch = prefetched.get(&module_path);
            if prefetch.is_none() && barrel_skipped_leaves.contains(&module_path) {
                // #1991: a pure-barrel leaf that no importer demanded —
                // the lazy crawl deliberately never read it. Exclude it
                // from the graph instead of falling back to the
                // synchronous read below, which would silently defeat
                // the optimization by re-fetching every skipped leaf one
                // at a time on this single-threaded replay.
                continue;
            }
            let source = match prefetch.map(|p| &p.source) {
                Some(Ok(s)) => s.clone(),
                Some(Err(e)) => {
                    tracing::warn!("Failed to read module {:?}: {}", module_path, e);
                    continue;
                }
                None => match std::fs::read_to_string(&module_path) {
                    Ok(s) => {
                        // #1999: this module missed the wave-parallel
                        // prefetch (e.g. discovered only via this serial
                        // replay); back-fill the cache so later phases don't
                        // also miss it.
                        if std::env::var_os("JET_NO_SOURCE_CACHE").is_none() {
                            self.source_cache
                                .lock()
                                .insert(module_path.clone(), Arc::from(s.as_str()));
                        }
                        s
                    }
                    Err(e) => {
                        tracing::warn!("Failed to read module {:?}: {}", module_path, e);
                        continue;
                    }
                },
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

                let module_imports = match prefetch.map(|p| &p.imports) {
                    Some(Ok(imports)) => imports.clone(),
                    Some(Err(e)) => {
                        tracing::warn!("Failed to extract imports from {:?}: {}", module_path, e);
                        continue;
                    }
                    // #1997: same fast-path-first preference as the parallel
                    // prefetch above, for this rare synchronous fallback (a
                    // module the wave crawl didn't already memoize). #2140:
                    // routed through the same cache-consulting helper as
                    // `prefetch_one_module`; this call site never reused a
                    // parse tree even before #2140 (`extract_imports`
                    // discards it), so `reusable=false`.
                    None => {
                        let (extracted, _tree, _used_fast) = self.scan_module_imports_cached(
                            &source,
                            &module_path,
                            is_typescript,
                            false,
                        );
                        match extracted {
                            Ok(imports) => imports,
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to extract imports from {:?}: {}",
                                    module_path,
                                    e
                                );
                                continue;
                            }
                        }
                    }
                };

                // For TSX/JSX files with automatic runtime, add react/jsx-runtime as implicit dependency
                let is_jsx = module_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e == "tsx" || e == "jsx")
                    .unwrap_or(false);

                let resolve_cached = |spec: &str| -> std::result::Result<PathBuf, String> {
                    if let Some(res) = prefetch.and_then(|p| p.resolutions.get(spec)) {
                        return res.clone();
                    }
                    self.resolve_dependency(&module_path, spec)
                        .map_err(|e| e.to_string())
                };

                if is_jsx {
                    match resolve_cached("react/jsx-runtime") {
                        Ok(resolved_path) => {
                            // WI #1995 round 4: this edge is fabricated from
                            // file extension alone, not from anything in
                            // `module_path`'s own source text — record it
                            // in the side channel so a raw-source liveness
                            // pre-pass can still see it.
                            self.implicit_edges
                                .lock()
                                .push((module_path.clone(), resolved_path.clone()));
                            queue.push((
                                resolved_path,
                                Some(module_id),
                                Some(graph::EdgeKind::Import),
                            ));
                        }
                        Err(err_msg) => {
                            if !err_msg.contains("External module") {
                                tracing::warn!(
                                    "Failed to resolve 'react/jsx-runtime': {}",
                                    err_msg
                                );
                                self.record_unresolved("react/jsx-runtime", &module_path, &err_msg);
                            }
                        }
                    }
                }

                for import_decl in &module_imports.static_imports {
                    match resolve_cached(&import_decl.source) {
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
                        Err(err_msg) => {
                            if !err_msg.contains("External module") {
                                tracing::warn!(
                                    "Failed to resolve '{}' from {:?}: {}",
                                    import_decl.source,
                                    module_path,
                                    err_msg
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
                    match resolve_cached(dynamic_import) {
                        Ok(resolved_path) => {
                            queue.push((
                                resolved_path,
                                Some(module_id),
                                Some(graph::EdgeKind::DynamicImport),
                            ));
                        }
                        Err(err_msg) => {
                            if !err_msg.contains("External module") {
                                tracing::warn!(
                                    "Failed to resolve '{}' from {:?}: {}",
                                    dynamic_import,
                                    module_path,
                                    err_msg
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
    /// @spec apps/jet/docs/build-fails-loudly-on-unresolved-bare-specifiers.md
    /// @issue #1317
    fn check_unresolved_deps(&self) -> Result<()> {
        let deps = std::mem::take(&mut *self.unresolved_deps.lock());
        if deps.is_empty() {
            return Ok(());
        }
        Err(format_unresolved_error(&deps))
    }

    /// #2141 — node_modules-scoped resolution cache. Eligibility is
    /// deliberately narrow and two-sided:
    ///
    /// 1. `specifier` must be a bare package specifier (`ModuleResolver::
    ///    is_bare_package_specifier` — the same boundary its own in-memory
    ///    `resolution_cache` already uses): a relative/alias specifier's
    ///    target depends on `from`'s *exact* directory, not just its
    ///    enclosing package scope, so keying by package scope alone would
    ///    silently conflate two different sibling directories' same-named
    ///    relative imports.
    /// 2. Both `from` (the importer) and the resolved target must live
    ///    under node_modules (`persistent_cache::node_modules_scope_
    ///    realpath`, checked on both ends). App-source resolutions are
    ///    never cached here — a relative/bare import reached from
    ///    application source is exposed to file-appearance and probe-order
    ///    hazards across a live dev/watch session (a file created after
    ///    this build started, a directory-listing order difference) that a
    ///    once-installed node_modules package layout is not exposed to
    ///    once a package is on disk and fixed for the run. The second half
    ///    of this check (the *target* side) additionally excludes
    ///    workspace-linked / `file:`-protocol packages that resolve out to
    ///    app-source despite being reached via a bare specifier.
    fn resolve_dependency(&self, from: &PathBuf, specifier: &str) -> Result<PathBuf> {
        let cache_key = if self.resolver.is_bare_package_specifier(specifier) {
            persistent_cache::node_modules_scope_realpath(from).map(|scope_realpath| {
                persistent_cache::ResolutionKey {
                    scope_realpath,
                    specifier: specifier.to_string(),
                    resolver_config_fingerprint: self.resolver_config_fingerprint,
                }
            })
        } else {
            None
        };

        if let Some(key) = &cache_key {
            if let Some(cached) = self.persistent_cache.get_resolution(key) {
                return Ok(cached);
            }
        }

        // `resolve_with_probe` (uncached, package.json-probe-capturing) only
        // when this resolution might be worth persisting; every other
        // specifier keeps using the hot in-memory-memoized `resolve` path
        // completely unchanged.
        let (resolved, probe) = match &cache_key {
            Some(_) => {
                let (result, probe) = self.resolver.resolve_with_probe(specifier, from);
                (result?, probe)
            }
            None => (self.resolver.resolve(specifier, from)?, Vec::new()),
        };

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

        let final_path = normalize_bundler_path_lexical(&abs);

        if let Some(key) = cache_key {
            if persistent_cache::node_modules_scope_realpath(&final_path).is_some() {
                let mut seen = std::collections::HashSet::new();
                let guard: Vec<(PathBuf, Option<u64>)> = probe
                    .into_iter()
                    .filter(|p| seen.insert(p.clone()))
                    .map(|p| {
                        let hash = std::fs::read(&p)
                            .ok()
                            .map(|bytes| persistent_cache::hash_bytes(&bytes));
                        (p, hash)
                    })
                    .collect();
                self.persistent_cache.insert_resolution(
                    key,
                    persistent_cache::ResolutionValue {
                        resolved_path: final_path.clone(),
                        guard,
                    },
                );
            }
        }

        Ok(final_path)
    }

    /// Pre-transform liveness pre-pass for the survivors-only transform
    /// filter (WI #1995 round 4). Runs
    /// `tree_shake::analyze_used_exports_from_with_implicit_edges` over
    /// every crawled module's raw, untransformed source — define-folded
    /// exactly like `apply_tree_shaking`'s own `module_pairs` construction
    /// (see the matching comment there: without this, a
    /// `process.env.NODE_ENV !== 'production'` dev-only branch keeps its
    /// requires looking used forever, which is where most of this filter's
    /// skip opportunity actually lives) — unioned with `self.implicit_edges`
    /// (non-textual edges `build_graph` fabricated from module structure —
    /// see that field's doc comment for the inventory). Returns the
    /// resulting live-path set.
    ///
    /// This define-folded-raw-source analysis is a deliberately
    /// conservative over-approximation of what `apply_tree_shaking`'s
    /// later, post-*transform* second-layer (numeric-require-id) DFS will
    /// find reachable: the transform step (JSX/TS lowering, Gate 1/Gate 2
    /// DCE) can only ever remove or relabel syntax the crawl already
    /// walked, never invent a require the crawl never discovered, so
    /// anything dead here is provably dead there too — "transform more,
    /// never fewer". `transform_modules` uses the returned set to skip the
    /// expensive transform entirely for modules outside it;
    /// `apply_tree_shaking` itself is untouched and remains the sole
    /// authority for what actually ships in the bundle.
    ///
    /// Returns `None` (meaning: transform everything, filter disabled) when
    /// `JET_NO_SURVIVOR_FILTER=1` is set — WI #1995 round 6 flips this
    /// filter to **default-on**: round 5's single-pass analysis reuse
    /// (`shake_analysis`) fixed round 4's double-analysis regression
    /// (double-reading and double-analyzing the whole corpus), and the
    /// real-corpus verdict (tw-monitor, dispatcher-measured, release build)
    /// is filter-on beats filter-off warm (2.26s vs 2.64s). Round 5's
    /// opt-in shape (`JET_SURVIVOR_FILTER=1`) is retired — the filter now
    /// simply runs unless explicitly disabled.
    /// `JET_NO_SURVIVOR_FILTER=1` is also the A/B knob for diffing filtered
    /// vs. unfiltered bundle output byte-for-byte (see the escape-hatch
    /// byte-identity tests).
    /// Also returns `None` when `build_graph` has not yet populated
    /// `self.entry_path`, or when any crawled module's source cannot be
    /// re-read. A partial liveness graph cannot be trusted to skip
    /// anything, so any of those bails to the fully-conservative "transform
    /// everything" behavior rather than risk a false-dead verdict.
    /// @issue #1995
    fn compute_transform_survivors(
        &self,
        graph: &ModuleGraph,
        sorted_ids: &[ModuleId],
    ) -> Option<HashSet<PathBuf>> {
        if std::env::var_os("JET_NO_SURVIVOR_FILTER").is_some() {
            return None;
        }
        let entry = self.entry_path.lock().clone()?;

        use rayon::prelude::*;

        // Round 5 (#1995): was a sequential `for &id in sorted_ids { ... }`
        // loop — its dominant cost (file read + defines-fold) is the same
        // shape every other pure-prefetch pass in this file already runs
        // through `par_iter` (see `prefetch_graph_modules_eager`,
        // `transform_modules`'s own per-module transform loop below).
        // `module_pairs`' order does not affect
        // `analyze_used_exports_from_with_implicit_edges`'s result — every
        // per-module fact it computes is immediately re-keyed into a
        // `HashMap<PathBuf, _>` inside that function, and this method's own
        // return value is itself an unordered `HashSet<PathBuf>` — so
        // collecting out of input order is safe. `collect::<Option<Vec<_>>>()`
        // preserves the original "any one module's node lookup or source
        // read failing aborts the whole pre-pass" semantics (short-circuits
        // to `None`, matching the sequential loop's `?` behavior), just
        // without the sequential loop's early-exit-on-first-failure — an
        // acceptable difference on this already-rare error path.
        let module_pairs: Vec<(PathBuf, String)> = sorted_ids
            .par_iter()
            .map(|&id| {
                let node = graph.get_node(id)?;
                // #1999: consult the per-build source cache the crawl
                // already populated instead of re-reading this module's
                // bytes from disk a second time.
                let source = self.cached_source(&node.path).ok()?.to_string();
                // Mirrors `apply_tree_shaking`'s `module_pairs` construction
                // exactly (same condition, same three calls) so this pre-pass
                // predicts that function's first-layer liveness rather than a
                // strictly-more-conservative (and far less useful) unfolded
                // scan.
                let source = if self.defines.is_empty() {
                    source
                } else {
                    let replaced = define::replace_defines(&source, &self.defines);
                    if replaced == source {
                        source
                    } else {
                        let folded = fold::fold_define_short_circuits(&replaced);
                        dce::eliminate_static_conditionals_syntax(&folded)
                    }
                };
                Some((node.path.clone(), source))
            })
            .collect::<Option<Vec<_>>>()?;

        let implicit_edges = self.implicit_edges.lock().clone();
        // #2141 — per-module raw-facts cache: `analyze_used_exports_from_
        // with_raw_facts_provider`'s `raw_facts_provider` hook lets this
        // pre-pass reuse a previous build's `compute_raw_module_facts`
        // output for any module whose source content (and `defines`
        // fingerprint) is unchanged, instead of re-running the full
        // export/import/reexport extraction over its raw text every time.
        // Keyed the same way the #2140 import-scan section is
        // (`content_hash` + `is_typescript`) since `compute_raw_module_
        // facts` is a pure function of exactly those two inputs — never of
        // this build's graph shape or module discovery order — and gated
        // at load time by `persistent_cache::analysis_fingerprint(&self.
        // defines)` (see `PersistentTransformCache::load`'s doc comment),
        // so a stale `defines` config can never surface here as a false
        // hit. `resolve_module_facts` (inside `analyze_used_exports_from_
        // with_raw_facts_provider`) turns a cached or freshly-computed
        // `RawModuleFacts` into the exact same `ModuleFacts` either way, so
        // a hit is indistinguishable downstream — including in
        // `shake_analysis`'s reuse by `apply_tree_shaking` below.
        let raw_facts_provider = |path: &Path, source: &str| -> tree_shake::RawModuleFacts {
            let is_typescript = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e == "ts" || e == "tsx")
                .unwrap_or(false);
            let key = persistent_cache::AnalysisKey {
                content_hash: persistent_cache::hash_str(source),
                is_typescript,
            };
            if let Some(cached) = self.persistent_cache.get_analysis(&key) {
                return cached;
            }
            let raw = tree_shake::compute_raw_module_facts(path, source);
            self.persistent_cache.insert_analysis(key, raw.clone());
            raw
        };
        let resolve_specifier = |spec: &str, importer: &Path| -> Option<PathBuf> {
            self.resolver
                .resolve(spec, importer)
                .ok()
                .filter(|r| !r.is_external)
                .map(|r| r.path)
        };
        match tree_shake::analyze_used_exports_from_with_raw_facts_provider(
            &module_pairs,
            &entry,
            Some(&resolve_specifier),
            &implicit_edges,
            Some(&raw_facts_provider),
        ) {
            Ok(result) => {
                let survivors: HashSet<PathBuf> = result.used_exports.keys().cloned().collect();
                // Round 5 (#1995): cache the full analysis so
                // `apply_tree_shaking` can reuse it for its own elimination
                // stage instead of re-reading every module's source and
                // re-running this same analysis a second time — see
                // `shake_analysis`'s field doc for the reuse contract.
                *self.shake_analysis.lock() = Some(result);
                Some(survivors)
            }
            Err(e) => {
                tracing::warn!(
                    "survivors-only transform pre-pass failed, transforming everything: {}",
                    e
                );
                None
            }
        }
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
        let resolution_index =
            crate::transform::modules::ModuleResolutionIndex::from_module_map_and_aliases_and_base_url(
                &module_map,
                &self.alias_entries,
                self.base_url.clone(),
            )
            .with_splitting(self.splitting);
        let side_effect_free_module_ids =
            collect_side_effect_free_module_indices(self, &graph, &sorted_ids);

        // WI #1995 round 4 — survivors-only transform: a raw-source
        // liveness pre-pass (`compute_transform_survivors`) predicts which
        // modules `apply_tree_shaking` will keep, so this pass can skip
        // the (dominant-cost) transform entirely for the rest.
        // `apply_tree_shaking` itself is untouched and remains the sole
        // authority for what ships — this is purely a work-skip, not an
        // elimination decision. `None` means the filter is disabled for
        // this build (see that method's doc comment for every bail
        // condition); every module is transformed exactly as before this
        // WI.
        //
        // Round 5 (#1995): this call used to be folded into the outer
        // `transform_modules` `JET_BUNDLE_TIMING` lap with no way to tell
        // how much of that lap was the pre-pass itself vs. the per-module
        // transform loop below — measured here as its own `analysis` lap
        // so the two are visible separately. 0 (or near-0) whenever the
        // filter is off (`compute_transform_survivors` bails before doing
        // any work — see its doc comment).
        let analysis_start = std::time::Instant::now();
        let survivors = self.compute_transform_survivors(&graph, &sorted_ids);
        if std::env::var_os("JET_BUNDLE_TIMING").is_some() {
            eprintln!("[bundle-timing] analysis: {:?}", analysis_start.elapsed());
        }
        let transformed_modules = std::sync::atomic::AtomicUsize::new(0);
        let skipped_modules = std::sync::atomic::AtomicUsize::new(0);
        // #1999 (beat-vite round 7) — per-module transform sub-lap
        // breakdown: `transform_sub_laps` covers `Transformer`'s own
        // two-step pipeline (ts_strip/jsx dispatch + modules_rewrite, see
        // `TransformStepTimings`'s doc comment); `define_dce_tail_ns`
        // covers this closure's own bundler-side defines+fold+DCE tail
        // below. Printed together as a `JET_BUNDLE_TIMING` sibling line.
        let transform_sub_laps = crate::transform::TransformStepTimings::default();
        let define_dce_tail_ns = std::sync::atomic::AtomicU64::new(0);
        // #2135 (beat-vite round 8) — `define_dce_tail_ns` was the largest
        // single transform-sub-lap (1.45s CPU summed across par_iter workers
        // on the reference corpus) with no way to tell which of its steps
        // actually burns that time. These five split it into: the
        // `replace_defines` string-replacement pass itself (`dce_defines_ns`),
        // each pass-gate predicate's own cost (`could_fold_static_conditional`
        // / `could_contain_require_like_call` — `dce_fold_gate_ns` /
        // `dce_require_gate_ns`), and each pass's actual run when its gate
        // lets it through (`dce_fold_run_ns` / `dce_require_run_ns`). Their
        // sum plus the empty-defines fast path accounts for
        // `define_dce_tail_ns` in full — see the `define-dce-sub-laps` print
        // below.
        let dce_defines_ns = std::sync::atomic::AtomicU64::new(0);
        let dce_fold_gate_ns = std::sync::atomic::AtomicU64::new(0);
        let dce_fold_run_ns = std::sync::atomic::AtomicU64::new(0);
        let dce_require_gate_ns = std::sync::atomic::AtomicU64::new(0);
        let dce_require_run_ns = std::sync::atomic::AtomicU64::new(0);
        // #2138 (beat-vite round 10) — `dce_require_run_ns` above still
        // times every require-binding-DCE run, but as of this WI that run
        // no longer always pays for its own analysis parse: when
        // fold+syntax-DCE ran on this module too (`dce_fold_run_ns`'s
        // branch), its validated tree is carried forward instead of being
        // discarded, so `eliminate_unused_side_effect_free_require_bindings`
        // can reuse it directly. `dce_require_tree_reused` counts how many
        // of `require_dce_ran`'s runs took that reused-tree path (vs.
        // parsing `after_dce` fresh, exactly as before this WI, when fold
        // did not run) — the permanent, cheap parse-count proof requested
        // by #2138, printed as a `define-dce-sub-laps` field.
        let dce_require_tree_reused = std::sync::atomic::AtomicUsize::new(0);

        // #2140 (beat-vite round 9) — persistent-cache pure-hit-path
        // attribution: how much of each module's pre-transform work below
        // goes to reading its source, hashing that content, assembling the
        // rest of the persistent-cache key (the barrel-demand snapshot plus
        // `dependency_fingerprint`/`barrel_fingerprint`), the store's own
        // lookup, cloning the matched `CompiledModule` out on a hit, vs.
        // everything else per module (the GH #3136 `fs::metadata`/mtime
        // read, the always-miss-in-a-one-shot-build in-memory `self.cache`
        // probe, `cached.id` assignment). Every module runs source-read
        // through store-lookup regardless of hit or miss — only `clone` and
        // the hit-only slice of `other` are exclusive to a hit — so on a
        // run with any misses, `other` also carries those misses' own small
        // per-module overhead here, never their (unrelated, much larger)
        // real transform cost, which begins after this block. Summed across
        // every worker thread in the `par_iter` below, like the sub-laps
        // above, so the total can exceed this phase's own wall-clock
        // `transform_modules` lap; on a fully warm rebuild (`cache: ...
        // misses=0`) every module takes the hit branch, so these six sum to
        // the whole pure-hit-path cost the WI is about. Printed as a
        // `persistent-cache-hit-laps` `JET_BUNDLE_TIMING` sibling line
        // regardless of whether the persistent cache is enabled (all
        // near-zero when it's off, since `get_with_laps` returns a fast
        // miss immediately).
        let hit_source_read_ns = std::sync::atomic::AtomicU64::new(0);
        let hit_content_hash_ns = std::sync::atomic::AtomicU64::new(0);
        let hit_key_assembly_ns = std::sync::atomic::AtomicU64::new(0);
        let hit_store_lookup_ns = std::sync::atomic::AtomicU64::new(0);
        let hit_clone_ns = std::sync::atomic::AtomicU64::new(0);
        let hit_other_ns = std::sync::atomic::AtomicU64::new(0);

        use rayon::prelude::*;
        // #2140 — the hit-path source read below keeps the cached `Arc<str>`
        // as-is (no `.to_string()` copy) and only allocates an owned
        // `String` in the barrel-prune branch that actually needs one; see
        // that match for the `Cow::Borrowed`/`Cow::Owned` split.
        use std::borrow::Cow;

        // #1995 — per-module pass gating. `fold_define_short_circuits` +
        // `eliminate_static_conditionals_syntax` + `eliminate_unused_side_
        // effect_free_require_bindings` each unconditionally tree-sitter-parse
        // their input even on modules with nothing for them to do; skip a
        // pass only when a cheap textual probe proves it has no candidates
        // (see `dce::could_fold_static_conditional` /
        // `dce::could_contain_require_like_call` for the soundness argument).
        // JET_NO_PASS_GATES=1 forces every gate open, reproducing pre-#1995
        // behavior exactly — the A/B knob for diffing gated vs ungated
        // bundle output byte-for-byte.
        let force_run_all_gates = std::env::var_os("JET_NO_PASS_GATES").is_some();
        let fold_dce_candidates = std::sync::atomic::AtomicUsize::new(0);
        let fold_dce_ran = std::sync::atomic::AtomicUsize::new(0);
        let require_dce_candidates = std::sync::atomic::AtomicUsize::new(0);
        let require_dce_ran = std::sync::atomic::AtomicUsize::new(0);

        let modules: Vec<CompiledModule> = sorted_ids
            .par_iter()
            .enumerate()
            .filter_map(|(module_id, &id)| {
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

                // WI #1995 round 4 — survivors-only transform. Skip the
                // expensive transform entirely for a Script module the
                // pre-pass proved unreachable from the entry: placed after
                // the GH #3136 metadata/mtime reads above (so IO problems
                // on a module still surface once it IS selected for
                // transform) but before the cache lookup below, so a
                // skipped module is simply never looked up — it can
                // neither poison nor be poisoned by a cache entry. CSS and
                // Wasm modules are never gated: their own transform is
                // already cheap (empty code / glue generation), not the
                // cost this filter exists to avoid.
                if node.kind == graph::ModuleKind::Script {
                    if let Some(live) = &survivors {
                        if !live.contains(&node.path) {
                            skipped_modules.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            return None;
                        }
                    }
                }
                transformed_modules.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                let other_start = std::time::Instant::now();
                let in_memory_hit = self.cache.get(&node.path, mtime);
                hit_other_ns.fetch_add(
                    other_start.elapsed().as_nanos() as u64,
                    std::sync::atomic::Ordering::Relaxed,
                );
                if let Some(mut cached) = in_memory_hit {
                    cached.id = module_id;
                    tracing::debug!("Using cached module: {:?}", node.path);
                    return Some(Ok(cached));
                }

                // #1999: consult the per-build source cache instead of
                // re-reading this module's bytes from disk a third time
                // (already read once by the crawl, once by the survivors
                // pre-pass).
                // #2140 — keep the cache's `Arc<str>` handle as-is (a clone
                // is just a refcount bump) instead of eagerly `.to_string()`-
                // copying every module's full source on every hit; the one
                // consumer that needs an owned `String` (the barrel-prune
                // branch below) allocates its own via `Cow::Owned`, and
                // every other module now reads through with zero byte copy.
                let read_start = std::time::Instant::now();
                let source: Arc<str> = match self.cached_source(&node.path) {
                    Ok(s) => s,
                    Err(e) => {
                        return Some(Err(anyhow::anyhow!(
                            "bundler: cannot read module {:?}: {e} (GH #3136)",
                            node.path
                        )));
                    }
                };
                hit_source_read_ns.fetch_add(
                    read_start.elapsed().as_nanos() as u64,
                    std::sync::atomic::Ordering::Relaxed,
                );

                // #2137 — persistent transform cache lookup, reachable only
                // once the in-memory `self.cache` check above has already
                // missed: unconditionally true in a one-shot `jet build`
                // process (its `DashMap` always starts empty), and only on a
                // process's first touch of `node.path` in a long-lived
                // dev-server run. `barrel_demand_snapshot` is taken once
                // here and reused below by the barrel-prune match, so this
                // replaces what used to be a second separate lock+lookup on
                // the same map. `self.persistent_cache` is a no-op
                // `get`/`insert` unless `BundleOptions::cache_project_root`
                // was set, so every opted-out path (dev server, `--lib`,
                // `--nx`) pays only one `bool` check here. See
                // `persistent_cache`'s module doc comment for what each
                // `EntryKey` field guards against.
                let barrel_snapshot_start = std::time::Instant::now();
                let barrel_demand_snapshot = self.barrel_demand.lock().get(&node.path).cloned();
                hit_key_assembly_ns.fetch_add(
                    barrel_snapshot_start.elapsed().as_nanos() as u64,
                    std::sync::atomic::Ordering::Relaxed,
                );
                let content_hash_start = std::time::Instant::now();
                let content_hash = persistent_cache::hash_str(&source);
                hit_content_hash_ns.fetch_add(
                    content_hash_start.elapsed().as_nanos() as u64,
                    std::sync::atomic::Ordering::Relaxed,
                );
                let key_assembly_start = std::time::Instant::now();
                let persistent_key = persistent_cache::EntryKey {
                    content_hash,
                    own_id: module_id,
                    dep_fingerprint: persistent_cache::dependency_fingerprint(
                        &graph,
                        id,
                        &module_map,
                    ),
                    barrel_fingerprint: persistent_cache::barrel_fingerprint(
                        barrel_demand_snapshot.as_ref(),
                    ),
                };
                hit_key_assembly_ns.fetch_add(
                    key_assembly_start.elapsed().as_nanos() as u64,
                    std::sync::atomic::Ordering::Relaxed,
                );
                let persistent_hit = self.persistent_cache.get_with_laps(
                    &node.path,
                    &persistent_key,
                    &hit_store_lookup_ns,
                    &hit_clone_ns,
                );
                if let Some(mut cached) = persistent_hit {
                    // #2140 — no in-memory `self.cache` backfill here (this
                    // used to `self.cache.insert(node.path.clone(), mtime,
                    // cached.clone())`, paying a path clone + module clone +
                    // DashMap insert on every persistent-cache hit). A
                    // persistent-cache hit is only reachable when
                    // `cache_project_root` is set, which only `jet build`'s
                    // one-shot CLI path does (`cli.rs`'s `cache_project_root:
                    // if cache_enabled { Some(root_dir.clone()) } else {
                    // None }` — dev server / `--lib` / `--nx` never set it).
                    // `jet build --watch` is a no-op warning today (GH
                    // #3708), not a real loop, so `transform_modules` never
                    // runs a second time in the same process to consult
                    // this backfill — it was pure write-only cost on every
                    // hit.
                    cached.id = module_id;
                    tracing::debug!("Using persistent-cached module: {:?}", node.path);
                    return Some(Ok(cached));
                }

                // #1991: a confirmed pure barrel's own body still lists
                // every leaf it re-exports, including names the lazy
                // crawl never fetched (they have no graph edge for
                // tree-shake to prune against). Strip those lines here,
                // before codegen sees them, so output matches an eager
                // crawl + tree-shake byte-for-byte. Discard any
                // tree-sitter parse cached for this path during the crawl
                // (built against the original, un-pruned source): its
                // node byte-offsets no longer match the shortened text,
                // so it must not be reused below — a cache miss here
                // falls back to a fresh parse, the same as any TS/TSX/JSX
                // module already takes.
                let source: Cow<'_, str> = match &barrel_demand_snapshot {
                    Some(BarrelDemand::Names(names)) => {
                        let pruned = prune_barrel_source_to_demand(&source, names);
                        self.parsed_trees.lock().remove(&node.path);
                        Cow::Owned(pruned)
                    }
                    _ => Cow::Borrowed(&*source),
                };

                let result = match node.kind {
                    graph::ModuleKind::Script => {
                        // Reuse the tree-sitter parse from graph construction
                        // (plain-JS modules only) so this module is parsed once.
                        let reuse_tree = self.parsed_trees.lock().remove(&node.path);
                        self.transformer
                            .transform_js_with_context_resolution_tree_and_timings(
                                &source,
                                &node.path,
                                &module_map,
                                Some(&resolution_index),
                                reuse_tree,
                                Some(&transform_sub_laps),
                            )
                    }
                    graph::ModuleKind::Css => Ok(crate::transform::TransformResult {
                        code: String::new(),
                        source_map: None,
                    }),
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
                        // When defines are present, also run syntax-aware DCE to eliminate dead
                        // branches created by the replacements (e.g. `if ("production" !==
                        // "production")`) without corrupting third-party nested if/else shapes.
                        //
                        // #1999 (beat-vite round 7): `dce_tail_start` times this whole
                        // block (both the empty-defines fast path and the real
                        // fold+DCE tail) into `define_dce_tail_ns`.
                        let dce_tail_start = std::time::Instant::now();
                        let final_code = if self.defines.is_empty() {
                            transform_result.code.clone()
                        } else {
                            // #2135 — sub-lap (a): the replace_defines pass
                            // itself, plus the `defines_changed` compare its
                            // output feeds Gate 1 below (one extra string
                            // compare that cannot exist without this step,
                            // so it is timed together with its producer).
                            let defines_start = std::time::Instant::now();
                            let after_define =
                                define::replace_defines(&transform_result.code, &self.defines);
                            let defines_changed = after_define != transform_result.code;
                            dce_defines_ns.fetch_add(
                                defines_start.elapsed().as_nanos() as u64,
                                std::sync::atomic::Ordering::Relaxed,
                            );
                            // Fold define-produced literal comparisons and
                            // their short-circuit consumers before the
                            // syntax DCE: `"production"!=="production"&&x`
                            // never folded (the condition pass needs the
                            // whole condition to be one literal compare),
                            // keeping multi-KB dev branches and their
                            // dev-only imports alive through tree shaking.
                            //
                            // #1995 — Gate 1: skip fold+syntax-DCE unless
                            // either (a) `replace_defines` actually changed
                            // this module's text (the only way a define
                            // token turns into a foldable literal), or (b)
                            // the cheap `could_fold_static_conditional`
                            // probe proves the post-define text already
                            // contains a shape `eval_condition` can fold
                            // independent of defines (see dce.rs for the
                            // soundness argument). False positives just run
                            // the pass and find nothing; JET_NO_PASS_GATES=1
                            // forces it open unconditionally.
                            fold_dce_candidates.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            // #2135 — sub-lap (b): only the probe call
                            // itself is timed. `fold_probe_hit` is written so
                            // `could_fold_static_conditional` is invoked
                            // under exactly the same condition as the
                            // original `force_run_all_gates || defines_changed
                            // || could_fold_static_conditional(..)` chain
                            // (both already-known bools first, probe only
                            // when both are false) — same short-circuit,
                            // same result, now with the probe's own cost
                            // attributed separately from the bools.
                            let fold_gate_start = std::time::Instant::now();
                            let fold_probe_hit = !(force_run_all_gates || defines_changed)
                                && dce::could_fold_static_conditional(&after_define);
                            dce_fold_gate_ns.fetch_add(
                                fold_gate_start.elapsed().as_nanos() as u64,
                                std::sync::atomic::Ordering::Relaxed,
                            );
                            // #2138 — `fold_tree` carries the tree
                            // `eliminate_static_conditionals_syntax_with_tree`
                            // ends its round-trip loop with (only ever set
                            // when fold actually ran) forward to the
                            // require-binding-DCE gate below, so that pass
                            // can reuse it instead of re-parsing `after_dce`
                            // from scratch when it happens to be the exact
                            // same text fold just finished validating. Stays
                            // `None` on the gate-skipped path below (nothing
                            // to fuse when fold never ran).
                            let mut fold_tree: Option<tree_sitter::Tree> = None;
                            let after_dce = if force_run_all_gates
                                || defines_changed
                                || fold_probe_hit
                            {
                                fold_dce_ran.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                // #2135 — sub-lap (c): the actual fold +
                                // syntax-DCE run.
                                let fold_run_start = std::time::Instant::now();
                                let after_fold = fold::fold_define_short_circuits(&after_define);
                                let (folded, tree) =
                                    dce::eliminate_static_conditionals_syntax_with_tree(
                                        &after_fold,
                                    );
                                fold_tree = tree;
                                dce_fold_run_ns.fetch_add(
                                    fold_run_start.elapsed().as_nanos() as u64,
                                    std::sync::atomic::Ordering::Relaxed,
                                );
                                folded
                            } else {
                                after_define
                            };

                            // #1995 — Gate 2: every candidate binding
                            // `eliminate_unused_side_effect_free_require_bindings`
                            // can remove is a require-like call
                            // (`require(...)` / `_r(...)` /
                            // `__jet__.dynamicImport(...)` — see
                            // `dce::could_contain_require_like_call`); skip
                            // the tree-sitter parse when none of those three
                            // textual forms are present.
                            require_dce_candidates
                                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            // #2135 — sub-lap (d): mirrors sub-lap (b) —
                            // only the probe call is timed, invoked under
                            // the same `!force_run_all_gates && ..`
                            // short-circuit the original `force_run_all_gates
                            // || could_contain_require_like_call(..)` chain
                            // already implied.
                            let require_gate_start = std::time::Instant::now();
                            let require_probe_hit = !force_run_all_gates
                                && dce::could_contain_require_like_call(&after_dce);
                            dce_require_gate_ns.fetch_add(
                                require_gate_start.elapsed().as_nanos() as u64,
                                std::sync::atomic::Ordering::Relaxed,
                            );
                            if force_run_all_gates || require_probe_hit {
                                require_dce_ran.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                // #2138 — count this run's reused-tree path
                                // before consuming `fold_tree` below, purely
                                // for the sub-lap proof; does not change
                                // which branch `_with_tree` itself takes.
                                if fold_tree.is_some() {
                                    dce_require_tree_reused
                                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                }
                                // #2135 — sub-lap (e): the actual
                                // require-binding-DCE run. #2138: reuses
                                // `fold_tree` (when fold ran on this module)
                                // instead of this pass's own from-scratch
                                // analysis parse of `after_dce` — see
                                // `eliminate_unused_side_effect_free_require_bindings_with_tree`'s
                                // doc comment for why this is behavior-
                                // identical to the un-fused call.
                                let require_run_start = std::time::Instant::now();
                                let result =
                                    dce::eliminate_unused_side_effect_free_require_bindings_with_tree(
                                        &after_dce,
                                        &side_effect_free_module_ids,
                                        fold_tree.as_ref(),
                                    );
                                dce_require_run_ns.fetch_add(
                                    require_run_start.elapsed().as_nanos() as u64,
                                    std::sync::atomic::Ordering::Relaxed,
                                );
                                result
                            } else {
                                after_dce
                            }
                        };
                        define_dce_tail_ns.fetch_add(
                            dce_tail_start.elapsed().as_nanos() as u64,
                            std::sync::atomic::Ordering::Relaxed,
                        );

                        let compiled = CompiledModule {
                            id: module_id,
                            path: node.path.clone(),
                            code: final_code.clone(),
                            source_map: transform_result.source_map.clone(),
                            dependencies: Vec::new(),
                            hash: calculate_hash(&final_code),
                        };

                        self.cache
                            .insert(node.path.clone(), mtime, compiled.clone());
                        // #2137 — mirror the insert into the persistent
                        // layer; a no-op unless the cache is enabled.
                        self.persistent_cache.insert(
                            node.path.clone(),
                            persistent_key,
                            compiled.clone(),
                        );

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

        // #1995 — pass-gate counters, printed as JET_BUNDLE_TIMING sibling
        // lines: how many defines-bearing modules actually ran the
        // fold+syntax-DCE pass vs. the require-binding-elimination pass,
        // out of how many were candidates (had `self.defines` non-empty).
        if std::env::var_os("JET_BUNDLE_TIMING").is_some() {
            use std::sync::atomic::Ordering;
            let fdc = fold_dce_candidates.load(Ordering::Relaxed);
            let fdr = fold_dce_ran.load(Ordering::Relaxed);
            let rdc = require_dce_candidates.load(Ordering::Relaxed);
            let rdr = require_dce_ran.load(Ordering::Relaxed);
            eprintln!(
                "[bundle-timing] pass-gates: fold+syntax-dce ran={fdr}/{fdc} (skipped {}), \
                 require-binding-dce ran={rdr}/{rdc} (skipped {})",
                fdc.saturating_sub(fdr),
                rdc.saturating_sub(rdr),
            );

            // #1995 round 4 — survivors-only transform counters, as a
            // JET_BUNDLE_TIMING sibling line to `pass-gates` above.
            // `implicit-edges` is `self.implicit_edges`'s length at the end
            // of this build's crawl (currently: one entry per `.tsx`/`.jsx`
            // module that resolved `react/jsx-runtime`) — 0 whenever the
            // filter never ran (round 6: `JET_NO_SURVIVOR_FILTER=1` set —
            // the filter is default-on — or no `.tsx`/`.jsx` modules in
            // this build).
            eprintln!(
                "[bundle-timing] survivor-filter: transformed={} skipped={} implicit-edges={}",
                transformed_modules.load(Ordering::Relaxed),
                skipped_modules.load(Ordering::Relaxed),
                self.implicit_edges.lock().len(),
            );

            // #1999 (beat-vite round 7) — transform_modules internal
            // sub-lap breakdown: how much of the per-module loop above went
            // to TS-strip vs. JSX lowering (`.tsx` is single-pass and
            // attributes wholly to `jsx` — see `TransformStepTimings`'s doc
            // comment) vs. the ES6/CJS module rewrite vs. this closure's
            // own bundler-side defines+fold+DCE tail. These are summed
            // across every worker thread in the `par_iter` above, so their
            // total can exceed (and is not directly comparable to) this
            // phase's own wall-clock `transform_modules` lap.
            eprintln!(
                "[bundle-timing] transform-sub-laps: ts_strip={:?} jsx={:?} modules_rewrite={:?} define_dce_tail={:?}",
                std::time::Duration::from_nanos(
                    transform_sub_laps.ts_strip_ns.load(Ordering::Relaxed)
                ),
                std::time::Duration::from_nanos(transform_sub_laps.jsx_ns.load(Ordering::Relaxed)),
                std::time::Duration::from_nanos(
                    transform_sub_laps.modules_rewrite_ns.load(Ordering::Relaxed)
                ),
                std::time::Duration::from_nanos(define_dce_tail_ns.load(Ordering::Relaxed)),
            );

            // #2135 (beat-vite round 8) — `define_dce_tail`'s own internal
            // breakdown, as a `transform-sub-laps` sibling line: how much of
            // that tail went to the `replace_defines` pass, each pass-gate
            // predicate's own cost, and each pass's actual run when its gate
            // let it through. Same cross-worker-summed caveat as the line
            // above. Kept in the final commit (not scratch instrumentation)
            // — one more eprintln under the same flag, cheap and
            // future-proof for the next define_dce_tail investigation.
            //
            // #2138 (beat-vite round 10) — `require_parse_reused` is the
            // permanent parse-count proof this WI adds: of `require_run`'s
            // `require_dce_ran` executions (see the `pass-gates` line
            // above), how many reused fold's already-validated tree instead
            // of paying for their own from-scratch `parse_js` analysis call.
            eprintln!(
                "[bundle-timing] define-dce-sub-laps: defines={:?} fold_gate={:?} fold_run={:?} require_gate={:?} require_run={:?} require_parse_reused={}/{}",
                std::time::Duration::from_nanos(dce_defines_ns.load(Ordering::Relaxed)),
                std::time::Duration::from_nanos(dce_fold_gate_ns.load(Ordering::Relaxed)),
                std::time::Duration::from_nanos(dce_fold_run_ns.load(Ordering::Relaxed)),
                std::time::Duration::from_nanos(dce_require_gate_ns.load(Ordering::Relaxed)),
                std::time::Duration::from_nanos(dce_require_run_ns.load(Ordering::Relaxed)),
                dce_require_tree_reused.load(Ordering::Relaxed),
                require_dce_ran.load(Ordering::Relaxed),
            );

            // #2140 (beat-vite round 9) — persistent-cache pure-hit-path
            // attribution, as a `transform-sub-laps` sibling line: see the
            // `hit_*_ns` declaration comment above for what each bucket
            // covers and the cross-worker-summed caveat.
            eprintln!(
                "[bundle-timing] persistent-cache-hit-laps: source_read={:?} content_hash={:?} key_assembly={:?} store_lookup={:?} clone={:?} other={:?}",
                std::time::Duration::from_nanos(hit_source_read_ns.load(Ordering::Relaxed)),
                std::time::Duration::from_nanos(hit_content_hash_ns.load(Ordering::Relaxed)),
                std::time::Duration::from_nanos(hit_key_assembly_ns.load(Ordering::Relaxed)),
                std::time::Duration::from_nanos(hit_store_lookup_ns.load(Ordering::Relaxed)),
                std::time::Duration::from_nanos(hit_clone_ns.load(Ordering::Relaxed)),
                std::time::Duration::from_nanos(hit_other_ns.load(Ordering::Relaxed)),
            );
        }

        Ok((modules, has_cycle))
    }

    /// Run tree-shaking analysis across all modules and remove unused exports.
    ///
    /// This is Phase 3 of the bundler pipeline (after transform + define + DCE,
    /// before generate_bundle).  Modules whose exports are entirely unused and
    /// have no side effects are eliminated.
    fn apply_tree_shaking(
        &self,
        modules: Vec<CompiledModule>,
        entry: &Path,
    ) -> Vec<CompiledModule> {
        // JET_NO_TREESHAKE=1 bypasses shaking entirely — the A/B knob for
        // bisecting runtime breakage to this phase (pair with
        // JET_TREESHAKE_DEBUG=<file> for per-module used-export dumps).
        if std::env::var_os("JET_NO_TREESHAKE").is_some() {
            tracing::warn!("JET_NO_TREESHAKE set: skipping tree shaking");
            return modules;
        }

        // WI #1995 round 5 — single-pass analysis reuse. When the survivors
        // filter ran this build, `compute_transform_survivors` already
        // computed the exact liveness/used-exports analysis this function
        // used to unconditionally redundantly recompute below — both read
        // raw, pre-transform source from disk with the same defines-folding
        // (see the recompute branch's `module_pairs` construction, mirrored
        // exactly), so the two calls only ever differed in (b) which
        // unreachable modules also happened to be in the input list — and
        // (b) is provably inert: `compute_transform_survivors`' survivor
        // set is a conservative over-approximation ("transform more, never
        // fewer" — see that method's doc comment), so any module it
        // excludes is unreachable even under its own more-generous
        // (implicit-edges-inclusive) walk, hence unreachable under this
        // function's own walk too — it can never have contributed a
        // `used_exports` entry to any module this function's own recompute
        // would have found reachable. Reusing the cached analysis is
        // therefore safe: same meaning, computed once instead of twice.
        // `None` (filter didn't run this build, or its pre-pass bailed)
        // falls through to the recompute below. `JET_DOUBLE_SHAKE_ANALYSIS=1`
        // forces the recompute even when a cached analysis is available,
        // for benchmark comparison.
        //
        // The recompute branch below now threads `self.implicit_edges`
        // through too (previously it always called the implicit-edges-blind
        // `analyze_used_exports_from` wrapper). That wasn't just a caching
        // gap: it was a latent correctness bug, independent of the
        // survivors filter, that this round's mixed-size fixture surfaced
        // empirically (Mandate 3's "+10KB" audit) — the automatic JSX
        // runtime import (`jsx`/`jsxs` from `react/jsx-runtime`) is
        // synthetic, never present in any `.tsx`/`.jsx` file's raw source
        // text (only the *transform* writes it), so a raw-source scan
        // without implicit edges can never discover it. `Fragment` often
        // survived anyway (explicit `import { Fragment }` in real code is
        // common), but `jsx`/`jsxs` did not: `shake_module` pruned their
        // `export const` lines out of `react/jsx-runtime`'s own shimmed
        // body, which is silently harmless only for entirely-dead call
        // sites (a bare property read like `$(id).jsx` evaluates to
        // `undefined` and is discarded) — a genuinely live JSX call site
        // would throw (`... .jsx is not a function`) at runtime. This
        // recompute path is the pre-round-4 baseline behavior (it's what
        // every build ran before the survivors filter existed, and it's
        // still what any filter-off build runs today), so the bug was not
        // introduced by the filter — the filter's implicit-edges-aware
        // analysis happened to mask it whenever it ran. `self.implicit_edges`
        // is populated unconditionally during `build_graph`'s crawl (see
        // its push site's own comment), independent of whether the
        // survivors filter is enabled, so passing it here costs nothing
        // extra and closes the gap for the filter-off path too.
        let cached_analysis = if std::env::var_os("JET_DOUBLE_SHAKE_ANALYSIS").is_none() {
            self.shake_analysis.lock().clone()
        } else {
            None
        };
        let analysis = if let Some(analysis) = cached_analysis {
            analysis
        } else {
            let module_pairs: Vec<(PathBuf, String)> = modules
                .iter()
                .map(|m| {
                    // #1999: consult the per-build source cache instead of
                    // re-reading this module's bytes from disk again.
                    let source = self
                        .cached_source(&m.path)
                        .map(|s| s.to_string())
                        .unwrap_or_else(|_| m.code.clone());
                    // Analyze post-define sources: without this, the dead
                    // `process.env.NODE_ENV !== 'production'` branch in packages
                    // like prop-types still marks its dev-only requires
                    // (factoryWithTypeCheckers, react-is, object-assign) as used
                    // even though the transformed code only requires the
                    // production shim.
                    let source = if self.defines.is_empty() {
                        source
                    } else {
                        let replaced = define::replace_defines(&source, &self.defines);
                        if replaced == source {
                            source
                        } else {
                            let folded = fold::fold_define_short_circuits(&replaced);
                            dce::eliminate_static_conditionals_syntax(&folded)
                        }
                    };
                    (m.path.clone(), source)
                })
                .collect();

            let resolve_specifier = |spec: &str, importer: &Path| -> Option<PathBuf> {
                self.resolver
                    .resolve(spec, importer)
                    .ok()
                    .filter(|r| !r.is_external)
                    .map(|r| r.path)
            };
            let implicit_edges = self.implicit_edges.lock().clone();
            match tree_shake::analyze_used_exports_from_with_implicit_edges(
                &module_pairs,
                entry,
                Some(&resolve_specifier),
                &implicit_edges,
            ) {
                Ok(a) => a,
                Err(e) => {
                    tracing::warn!("Tree shake analysis failed, skipping: {}", e);
                    return modules;
                }
            }
        };
        // JET_TREESHAKE_DEBUG=<file> dumps per-module used-export sets.
        if let Some(dump) = std::env::var_os("JET_TREESHAKE_DEBUG") {
            let mut lines: Vec<String> = analysis
                .used_exports
                .iter()
                .map(|(p, names)| {
                    let mut sorted: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
                    sorted.sort_unstable();
                    format!("{} => [{}]", p.display(), sorted.join(","))
                })
                .collect();
            lines.sort();
            let _ = std::fs::write(dump, lines.join("\n"));
        }

        if !analysis.eliminated_modules.is_empty() {
            tracing::info!(
                "Tree shaking: eliminating {} modules (~{} bytes)",
                analysis.eliminated_modules.len(),
                analysis.eliminated_bytes
            );
        }

        let eliminated_paths: HashSet<PathBuf> =
            analysis.eliminated_modules.iter().cloned().collect();

        // Prune retained modules' lowered re-export glue down to the names
        // the analysis proved used BEFORE computing require edges. The old
        // rescue criterion ("any retained module's code contains _r(id)")
        // resurrected every barrel re-export target — unconditional barrel
        // glue re-imported ~170KB of eliminated MUI code.
        let id_to_path: HashMap<usize, &PathBuf> =
            modules.iter().map(|m| (m.id, &m.path)).collect();
        let star_leaf_exports = |id: usize| -> Option<Vec<String>> {
            id_to_path
                .get(&id)
                .and_then(|path| analysis.all_exports.get(*path))
                .cloned()
        };
        let mut pruned_codes: HashMap<usize, String> = modules
            .iter()
            .filter(|m| !eliminated_paths.contains(&m.path))
            .map(|m| {
                let code = match analysis.used_exports.get(&m.path) {
                    Some(used) if !used.is_empty() => dce::eliminate_unused_reexport_assignments(
                        &m.code,
                        used,
                        Some(&star_leaf_exports),
                    ),
                    _ => m.code.clone(),
                };
                (m.id, code)
            })
            .collect();

        // Reachability from the entry over the pruned require edges decides
        // what stays. Eliminated modules reached directly (a retained module
        // genuinely requires one) are rescued and traversed via their
        // original code, preserving the old rescue semantics for real
        // dependencies.
        let entry_id = modules
            .iter()
            .find(|m| m.path == entry)
            .map(|m| m.id)
            .or_else(|| modules.iter().map(|m| m.id).min());
        // Captured alongside `reachable` for the elimination-walk skip-filter
        // below (WI #1947 round 2): every id this DFS visits already has its
        // full outgoing numeric-require-id set computed right here to decide
        // where to walk next, so recording it costs nothing beyond the
        // `HashMap` insert — no extra parsing or scanning versus before.
        let mut module_require_ids: HashMap<usize, HashSet<usize>> = HashMap::new();
        let reachable: HashSet<usize> = if let Some(entry_id) = entry_id {
            let mut reachable = HashSet::new();
            let mut stack = vec![entry_id];
            while let Some(id) = stack.pop() {
                if !reachable.insert(id) {
                    continue;
                }
                let requires = match pruned_codes.get(&id) {
                    Some(code) => dce::numeric_require_ids(code),
                    None => modules
                        .iter()
                        .find(|m| m.id == id)
                        .map(|m| dce::numeric_require_ids(&m.code))
                        .unwrap_or_default(),
                };
                stack.extend(requires.iter().copied());
                module_require_ids.insert(id, requires);
            }
            reachable
        } else {
            modules.iter().map(|m| m.id).collect()
        };

        let eliminated_module_ids: HashSet<usize> = modules
            .iter()
            .filter(|m| !reachable.contains(&m.id))
            .map(|m| m.id)
            .collect();
        if !eliminated_module_ids.is_empty() {
            tracing::info!(
                "Tree shaking: {} of {} modules unreachable after re-export pruning",
                eliminated_module_ids.len(),
                modules.len()
            );
        }

        // WI #2126 — statement-level DCE for retained modules. Runs
        // adjacent to (immediately after) `shake_module`'s ESM export-line
        // pruning below, on the same already-shaken body: `shake_module`
        // narrows a barrel's `export { a, b }` clause, and this pass then
        // walks what's left looking for CJS `exports.NAME = ...;`
        // assignments and top-level function/class/var declarations that
        // `used_exports`-reachability proves dead — the gap real
        // pre-built-CJS vendor packages leave open, since Babel emits an
        // unconditional `exports.NAME = value;` for every named export
        // regardless of downstream usage (see dce.rs's
        // `eliminate_dead_top_level_declarations` doc comment for the full
        // mark-and-sweep algorithm). Deliberately only wired into the
        // `used.is_empty()` == false branch below (where `shake_module`
        // itself already runs): an empty `used` set there means "no
        // specific per-export usage signal," not "nothing is used," and
        // `shake_module` already treats that case as unsafe to touch for
        // the same reason. `JET_NO_STMT_DCE=1` is the A/B bypass knob,
        // mirroring `JET_NO_TREESHAKE` above; `JET_BUNDLE_TIMING` (already
        // established convention) gets one extra lap line with per-build
        // counters.
        //
        // WI #2134 — gated off entirely on the entry-flatten path.
        // #2126's own close-out measured only 184 net bytes on the mui
        // corpus's default (minified, splitting-on) build, because
        // `scope_hoist`'s R5 (`eliminate_unused_exports_preserving_entry`)
        // and the entry-flatten partition's own downstream pipeline
        // already remove the same dead declarations later — this pass
        // pays ~200ms of sequential tree-sitter parsing per build for
        // that near-zero marginal yield whenever the build is headed
        // there. `self.splitting` is the exact already-resolved boolean
        // that later decides whether `generate_bundle` even attempts
        // `generate_split_bundle`/entry-flatten (see that call site, and
        // `build_splitting_enabled` in cli.rs, which is what produced
        // this value in the first place) — reused here as-is instead of
        // re-derived, so this gate can never drift from the real
        // decision. Non-flatten builds (`--no-splitting`; library builds
        // never reach this method at all — `library` mode runs through
        // `lib_build::build_library` instead of `Bundler::bundle`) are
        // unaffected: the pass still runs exactly as before.
        // `JET_FORCE_STMT_DCE=1` forces the pass back on even on the
        // flatten path (A/B measurement + escape hatch); `JET_NO_STMT_DCE=1`
        // still wins over it when both are set, mirroring the "off flag
        // always wins" precedence convention used elsewhere (e.g.
        // `build_minify_enabled_from_matches` in cli.rs).
        struct StmtDceCounters {
            modules: std::cell::Cell<usize>,
            pruned_decls: std::cell::Cell<usize>,
            pruned_bytes: std::cell::Cell<usize>,
            skipped_vendor: std::cell::Cell<usize>,
        }
        let stmt_dce_no = std::env::var_os("JET_NO_STMT_DCE").is_some();
        let stmt_dce_force = std::env::var_os("JET_FORCE_STMT_DCE").is_some();
        let entry_flatten_path = self.splitting;
        let stmt_dce_skipped_flatten = entry_flatten_path && !stmt_dce_no && !stmt_dce_force;
        let stmt_dce_enabled = if stmt_dce_no {
            false
        } else if stmt_dce_force {
            true
        } else {
            !entry_flatten_path
        };
        let stmt_dce_counters = StmtDceCounters {
            modules: std::cell::Cell::new(0),
            pruned_decls: std::cell::Cell::new(0),
            pruned_bytes: std::cell::Cell::new(0),
            skipped_vendor: std::cell::Cell::new(0),
        };
        let apply_stmt_dce = |code: String, used: &HashSet<String>| -> String {
            if !stmt_dce_enabled {
                return code;
            }
            stmt_dce_counters
                .modules
                .set(stmt_dce_counters.modules.get() + 1);
            let outcome = dce::eliminate_dead_top_level_declarations(&code, used);
            if outcome.skipped_vendor {
                stmt_dce_counters
                    .skipped_vendor
                    .set(stmt_dce_counters.skipped_vendor.get() + 1);
                return code;
            }
            stmt_dce_counters
                .pruned_decls
                .set(stmt_dce_counters.pruned_decls.get() + outcome.pruned_decls);
            stmt_dce_counters
                .pruned_bytes
                .set(stmt_dce_counters.pruned_bytes.get() + outcome.pruned_bytes);
            outcome.code
        };

        let result: Vec<CompiledModule> = modules
            .into_iter()
            .filter(|m| reachable.contains(&m.id))
            .map(|m| {
                let code_base = pruned_codes.remove(&m.id).unwrap_or_else(|| m.code.clone());
                let used = analysis
                    .used_exports
                    .get(&m.path)
                    .cloned()
                    .unwrap_or_default();
                // WI #1947 round 2: `eliminate_require_reexports_to_eliminated_modules`
                // tree-sitter-parses this module and walks the whole AST twice
                // purely to find require()/re-export edges into
                // `eliminated_module_ids` — work that's wasted whenever this
                // module can't reach any eliminated id at all. `module_require_ids`
                // (collected by the reachability DFS above) already carries the
                // exact numeric ids reachable from `code_base`'s code string for
                // the `used.is_empty()` case below; for the `shake_module` case,
                // it's still a safe superset, because `shake_module` only ever
                // blanks whole ESM-export lines to `\n` — it can never introduce
                // a require() call that wasn't already textually present in
                // `code_base`, so `shaken`'s reachable ids are always a subset of
                // `code_base`'s. Either way, a disjoint `module_require_ids` entry
                // proves the walk below would be a no-op, so it's skipped
                // outright. A missing map entry (every id reaching this closure
                // was visited by the DFS above, so this should not happen) or a
                // non-disjoint set both fall back to the real call — see this
                // file's `test_apply_tree_shaking_retained_module_with_no_eliminated_references_is_byte_untouched`
                // / `..._still_eliminates_unreachable_module_alongside_skip_filter`
                // and dce.rs's `test_numeric_require_ids_disjoint_from_eliminated_set_predicts_noop`
                // for the pruning correctness this must never regress.
                let skip_elimination_walk = module_require_ids
                    .get(&m.id)
                    .is_some_and(|ids| ids.is_disjoint(&eliminated_module_ids));
                if used.is_empty() {
                    let code = if skip_elimination_walk {
                        code_base
                    } else {
                        dce::eliminate_require_reexports_to_eliminated_modules(
                            &code_base,
                            &eliminated_module_ids,
                        )
                    };
                    return CompiledModule { code, ..m };
                }
                let shaken = tree_shake::shake_module(&code_base, &m.path, &used);
                let shaken = if skip_elimination_walk {
                    shaken
                } else {
                    dce::eliminate_require_reexports_to_eliminated_modules(
                        &shaken,
                        &eliminated_module_ids,
                    )
                };
                let shaken = apply_stmt_dce(shaken, &used);
                CompiledModule { code: shaken, ..m }
            })
            .collect();
        if std::env::var_os("JET_BUNDLE_TIMING").is_some() {
            eprintln!(
                "[bundle-timing] stmt-dce: modules={} pruned_decls={} bytes={} skipped_vendor={} skipped_flatten={}",
                stmt_dce_counters.modules.get(),
                stmt_dce_counters.pruned_decls.get(),
                stmt_dce_counters.pruned_bytes.get(),
                stmt_dce_counters.skipped_vendor.get(),
                stmt_dce_skipped_flatten as u8
            );
        }
        result
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
                chunks: Vec::new(),
                preload_hints: Vec::new(),
            });
        }

        // Code splitting (`--splitting`): produce a multi-chunk output
        // instead of the single-file formats below. Wins over
        // has_cycle/scope-hoist selection since a split build always needs
        // the runtime module registry (dynamic imports are chunk
        // boundaries). Returns early so the single-file paths below stay
        // byte-for-byte unchanged when splitting is off.
        //
        // `generate_split_bundle` returns `None` when the graph has no
        // dynamic-import boundaries to split on — the emergent fallback
        // that makes default-on splitting (WI #1932) safe: callers no
        // longer need to pre-scan for `import()` before setting
        // `self.splitting`, since a graph with nothing to split falls
        // straight through to the exact single-file paths below, so output
        // stays byte-for-byte identical to `--no-splitting`.
        // @issue #1930
        // @issue #1932
        if self.splitting {
            if let Some((entry_code, chunks, preload_hints)) =
                self.generate_split_bundle(&modules)?
            {
                return Ok(BundleOutput {
                    code: entry_code,
                    source_map: None,
                    assets: Vec::new(),
                    chunks,
                    preload_hints,
                });
            }
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
            if self.minify {
                tracing::debug!(
                    "Using Phase 2 true module flattening \
                     (minify=true; unsafe modules keep wrapper boundaries)"
                );
                let timing = std::env::var_os("JET_BUNDLE_TIMING").is_some();
                let mut last = std::time::Instant::now();
                let mut lap = |stage: &str| {
                    if timing {
                        eprintln!("[bundle-timing]   generate/{stage}: {:?}", last.elapsed());
                        last = std::time::Instant::now();
                    }
                };
                let raw = scope_hoist::generate_flattened_bundle(&modules);
                lap("flatten");
                // R4: Cross-module constant inlining → R5: DCE
                let after_r4 = scope_hoist::inline_cross_module_constants(&raw);
                lap("r4_inline_constants");
                let after_r5 = scope_hoist::eliminate_unused_exports_preserving_entry(&after_r4, 0);
                lap("r5_unused_exports");
                let after_markers = dce::eliminate_unread_es_module_markers(&after_r5);
                lap("es_module_markers");
                let after_reexport_wrappers =
                    scope_hoist_opt::collapse_pure_reexport_wrappers(&after_markers);
                lap("reexport_wrappers");
                let after_interop =
                    scope_hoist_opt::hoist_default_interop_thunks(&after_reexport_wrappers);
                lap("interop_thunks");
                // Flat-region function-declaration → var-hoisted conversion
                // (#2132) followed by same-chunk export-binding elision
                // (#2128): conversion unblocks elision for function-declared
                // exports, which are block-scoped (and therefore ineligible
                // for elision) until rewritten to a var-hoisted anonymous
                // function expression. Both default-on; JET_NO_FN_DECL_CONVERSION=1
                // / JET_NO_EXPORT_ELISION=1 are independent testing escape
                // hatches. When both passes are enabled (the common case),
                // `convert_and_elide_flat_region` runs them as one pipeline
                // sharing a single region-wide reparse-validation instead of
                // one per pass — each pass's own from-scratch reparse of the
                // ~1.4MB flat region was the dominant per-pass cost (#2133).
                let no_fn_decl_conv = std::env::var_os("JET_NO_FN_DECL_CONVERSION").is_some();
                let no_export_elision = std::env::var_os("JET_NO_EXPORT_ELISION").is_some();
                let out = if !no_fn_decl_conv && !no_export_elision {
                    let (out, conv_stats, elision_stats) =
                        scope_hoist_opt::convert_and_elide_flat_region(&after_interop);
                    if timing {
                        eprintln!(
                            "[bundle-timing]   generate/fn-decl-conversion: converted={} skipped_order={} skipped_shape={}",
                            conv_stats.converted, conv_stats.skipped_order, conv_stats.skipped_shape
                        );
                    }
                    lap("fn_decl_conversion");
                    if timing {
                        eprintln!(
                            "[bundle-timing]   generate/export-elision: modules={} elided_keys={} kept={} kept_registry={} kept_cross_chunk={} kept_namespace={} kept_string_indexed={} kept_barrel_glue={} kept_other={} rhs_normalized={} rhs_skipped_impure={}",
                            elision_stats.modules,
                            elision_stats.elided_keys,
                            elision_stats.kept,
                            elision_stats.kept_registry,
                            elision_stats.kept_cross_chunk,
                            elision_stats.kept_namespace,
                            elision_stats.kept_string_indexed,
                            elision_stats.kept_barrel_glue,
                            elision_stats.kept_other,
                            elision_stats.rhs_normalized,
                            elision_stats.rhs_skipped_impure,
                        );
                    }
                    lap("export_elision");
                    out
                } else {
                    let after_fn_decl_conv = if no_fn_decl_conv {
                        after_interop
                    } else {
                        let (converted, stats) =
                            scope_hoist_opt::convert_flat_region_function_declarations_to_var(
                                &after_interop,
                            );
                        if timing {
                            eprintln!(
                                "[bundle-timing]   generate/fn-decl-conversion: converted={} skipped_order={} skipped_shape={}",
                                stats.converted, stats.skipped_order, stats.skipped_shape
                            );
                        }
                        converted
                    };
                    lap("fn_decl_conversion");
                    let out = if no_export_elision {
                        after_fn_decl_conv
                    } else {
                        let (elided, stats) =
                            scope_hoist_opt::elide_same_chunk_export_bindings(&after_fn_decl_conv);
                        if timing {
                            eprintln!(
                                "[bundle-timing]   generate/export-elision: modules={} elided_keys={} kept={} kept_registry={} kept_cross_chunk={} kept_namespace={} kept_string_indexed={} kept_barrel_glue={} kept_other={}",
                                stats.modules,
                                stats.elided_keys,
                                stats.kept,
                                stats.kept_registry,
                                stats.kept_cross_chunk,
                                stats.kept_namespace,
                                stats.kept_string_indexed,
                                stats.kept_barrel_glue,
                                stats.kept_other,
                            );
                        }
                        elided
                    };
                    lap("export_elision");
                    out
                };
                out
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
            chunks: Vec::new(),
            preload_hints: Vec::new(),
        })
    }

    /// Multi-chunk bundle generation for `--splitting`.
    ///
    /// Partitions `modules` into an entry chunk plus async/shared chunks via
    /// `splitting::split_chunks_with_config`, then wraps every non-entry
    /// chunk's module defines in `__jet__.registerChunk(name, function(){...})`.
    /// Modules within a chunk are emitted in compiled-module `id` order
    /// (chunk partitioning is set-based internally, so this keeps output
    /// deterministic across runs).
    ///
    /// The returned entry code is intentionally the raw
    /// `generate_runtime() + defines + require(entry_id)` shape — it does
    /// NOT yet contain `__jet__.chunkManifest`. The manifest needs every
    /// chunk's final content-hashed filename, which isn't known until the
    /// caller (`cli.rs`'s build handler) has minified and hashed each
    /// `ChunkArtifact`; that caller injects the manifest object literal into
    /// this entry code before running it through the same minify tail.
    ///
    /// Also returns the entry chunk's static preload hints
    /// (`splitting::generate_preload_hints`'s result, unmodified). `href`
    /// values are pre-hash chunk names (e.g. `"assets/shared.js"`); the
    /// caller must remap them to the final content-hashed filename before
    /// emitting HTML.
    ///
    /// Returns `Ok(None)` when the partitioned graph has no dynamic-import
    /// boundaries — `split_chunks_with_config` always emits exactly one
    /// Entry chunk, and with no split points that Entry chunk is the only
    /// chunk (every module inlined, same as the single-file path). Signals
    /// the caller to fall back to the pre-existing single-file assembly
    /// instead of building a redundant one-chunk split bundle whose runtime
    /// (`generate_split_runtime`) differs from the legacy
    /// `generate_runtime`, which would otherwise change output for apps
    /// with no dynamic imports once splitting defaults on for web builds.
    /// @issue #1930
    /// @issue #1931
    /// @issue #1932
    fn generate_split_bundle(
        &self,
        modules: &[CompiledModule],
    ) -> Result<Option<(String, Vec<ChunkArtifact>, Vec<PreloadHint>)>> {
        let entry = modules
            .iter()
            .find(|m| m.id == 0)
            .ok_or_else(|| anyhow::anyhow!("code splitting: no entry module (id 0) in bundle"))?;

        let id_to_path: HashMap<usize, PathBuf> =
            modules.iter().map(|m| (m.id, m.path.clone())).collect();
        let all_module_ids: Vec<usize> = modules.iter().map(|m| m.id).collect();

        let edges = {
            let graph = self.graph.read();
            split_edges_from_graph(&graph, modules)
        };

        let manual_chunk_config = splitting::ManualChunkConfig {
            entries: self.manual_chunks.clone(),
        };
        let split_result = splitting::split_chunks_with_config(
            entry.id,
            &edges,
            &all_module_ids,
            &manual_chunk_config,
            &id_to_path,
        );

        // Nothing to split (no dynamic-import boundaries in this graph):
        // fall through to the single-file path in `generate_bundle`. See
        // this function's doc comment for why this must be checked before
        // any entry/chunk assembly work below.
        // @issue #1932
        if split_result
            .chunks
            .iter()
            .all(|c| c.chunk_type == splitting::ChunkType::Entry)
        {
            return Ok(None);
        }

        let by_id: HashMap<usize, &CompiledModule> = modules.iter().map(|m| (m.id, m)).collect();
        let modules_for = |ids: &[usize]| -> Vec<&CompiledModule> {
            let mut found: Vec<&CompiledModule> =
                ids.iter().filter_map(|id| by_id.get(id).copied()).collect();
            found.sort_by_key(|m| m.id);
            found
        };

        let entry_chunk = split_result
            .chunks
            .iter()
            .find(|c| c.chunk_type == splitting::ChunkType::Entry)
            .ok_or_else(|| anyhow::anyhow!("code splitting: no entry chunk produced"))?;

        // `entry_chunk.imports` mixes two different load semantics: async
        // chunk names (dynamic `import()` targets — loaded on demand,
        // already correctly deferred by `dynamicImport`) and static
        // shared/manual chunk names (the entry itself statically depends on
        // them; see `splitting::split_chunks`'s `entry_imports` and
        // `split_chunks_with_config`'s manual-chunk append). Only the latter
        // must be loaded before `require(entry.id)` below — mirrors
        // `splitting::generate_preload_hints`'s own async-exclusion filter,
        // which already proves this is the correct way to separate the two.
        // @issue #1963
        let async_chunk_names: HashSet<&str> = split_result
            .chunks
            .iter()
            .filter(|c| c.chunk_type == splitting::ChunkType::Async)
            .map(|c| c.name.as_str())
            .collect();
        let entry_static_imports: Vec<&str> = entry_chunk
            .imports
            .iter()
            .map(String::as_str)
            .filter(|name| !async_chunk_names.contains(name))
            .collect();

        // Entry flat region (issue #1993): flatten the SAFE SUBSET of the
        // entry chunk into one flat scope with unified mangling, keeping
        // the `__jet__` registry for the residue the fallback ladder in
        // `scope_hoist::partition_entry_for_flatten` can't prove safe.
        // `JET_NO_ENTRY_FLATTEN=1` is a testing escape hatch that forces
        // every entry module to the registry, reproducing the pre-#1993
        // output byte-for-byte (see the `flat_ids.is_empty()` branch below,
        // which is the exact same loop as before this change).
        let entry_modules_sorted = modules_for(&entry_chunk.modules);
        let no_entry_flatten = std::env::var_os("JET_NO_ENTRY_FLATTEN").is_some();
        // Mirrors the single-file Phase 1/Phase 2 gate above: flattening's
        // `_m{n}_`-prefixed identifiers only pay off once `mangle`/the oxc
        // minifier compresses them. Under `--no-minify` (dev/debug builds)
        // flattening would only cost readability with no offsetting size
        // win, so keep the registry-per-module shape there too.
        let partition = if no_entry_flatten || !self.minify {
            scope_hoist::EntryFlattenPartition {
                flat_ids: Vec::new(),
                registry_ids: entry_modules_sorted.iter().map(|m| m.id).collect(),
            }
        } else {
            scope_hoist::partition_entry_for_flatten(&entry_modules_sorted, &edges)
        };

        let timing = std::env::var_os("JET_BUNDLE_TIMING").is_some();
        if timing {
            eprintln!(
                "[bundle-timing] entry-flatten partition: flattened={} registry={}",
                partition.flat_ids.len(),
                partition.registry_ids.len()
            );
        }

        let mut entry_code = String::new();
        entry_code.push_str(&generate_split_runtime());
        entry_code.push_str("\n\n");

        if partition.flat_ids.is_empty() {
            // No-flatten fallback (escape hatch, or the fallback ladder
            // excluded every entry module): byte-identical to the
            // pre-#1993 registry-only path.
            for module in &entry_modules_sorted {
                entry_code.push_str(&format_module_define(module));
            }
        } else {
            let flat_modules = modules_for(&partition.flat_ids);
            let registry_modules = modules_for(&partition.registry_ids);

            let mut last = std::time::Instant::now();
            let mut lap = |stage: &str| {
                if timing {
                    eprintln!(
                        "[bundle-timing]   entry-flatten/{stage}: {:?}",
                        last.elapsed()
                    );
                    last = std::time::Instant::now();
                }
            };

            let raw_flat = scope_hoist::generate_entry_flat_region(
                &flat_modules,
                &edges,
                entry.id,
                !partition.registry_ids.is_empty(),
            );
            lap("flatten");
            // R4 only: cross-module constant inlining is safe on flat-only
            // text (its `_m{i}_NAME` pattern can only exist there). R5
            // (cross-module unused-export DCE) is skipped: its
            // `require_aliases_for_modules` helper only recognizes the
            // `_r(id)` call form, not the registry residue's
            // `var dep = require(id); dep.prop` alias pattern, so running
            // it over combined flat+registry text risks stripping an
            // export the registry side still reads. Documented limitation,
            // not required by #1993's scope.
            let flat_region = scope_hoist::inline_cross_module_constants(&raw_flat);
            lap("r4_inline_constants");

            // Registry `__jet__.define(...)` calls come FIRST, ahead of the
            // flat region's IIFE: `define` only registers a factory (no
            // invocation), so this ordering costs nothing, but it is load
            // -bearing for interop — see `generate_entry_flat_region`'s doc
            // comment. A flat module's synchronous top-level call into a
            // registry-residue module needs `__jet__.modules[id]` already
            // populated at that point, which only holds if every registry
            // define has already run before the flat IIFE starts.
            let mut body = String::with_capacity(flat_region.len() + registry_modules.len() * 256);
            for module in &registry_modules {
                body.push_str(&format_module_define(module));
            }
            body.push_str(&flat_region);

            // Confirmed safe on combined flat+registry text: each either
            // targets a flat-only-producible pattern, or explicitly
            // recognizes both `_r(id)`/`require(id)` call forms and
            // conservatively bails out per-occurrence when it can't
            // attribute one.
            let after_markers = dce::eliminate_unread_es_module_markers(&body);
            lap("es_module_markers");
            let after_reexport_wrappers =
                scope_hoist_opt::collapse_pure_reexport_wrappers(&after_markers);
            lap("reexport_wrappers");
            let after_interop =
                scope_hoist_opt::hoist_default_interop_thunks(&after_reexport_wrappers);
            lap("interop_thunks");
            // Flat-region function-declaration → var-hoisted conversion
            // (#2132) followed by same-chunk export-binding elision
            // (#2128): both safe on combined flat+registry text too —
            // registry residue never uses the flattener's `_m<n>_` prefix
            // (conversion's regex can only match inside the flat region),
            // and registry-residue reads use the literal `require(id)`
            // token (a distinct lexical scope from the flat IIFE's `_r`),
            // which elision always treats as a force-keep signal, never a
            // rewrite target. Both default-on; JET_NO_FN_DECL_CONVERSION=1
            // / JET_NO_EXPORT_ELISION=1 are independent testing escape
            // hatches. When both passes are enabled (the common case),
            // `convert_and_elide_flat_region` runs them as one pipeline
            // sharing a single region-wide reparse-validation instead of
            // one per pass — each pass's own from-scratch reparse of the
            // ~1.4MB flat region was the dominant per-pass cost (#2133).
            let no_fn_decl_conv = std::env::var_os("JET_NO_FN_DECL_CONVERSION").is_some();
            let no_export_elision = std::env::var_os("JET_NO_EXPORT_ELISION").is_some();
            let processed_body = if !no_fn_decl_conv && !no_export_elision {
                let (out, conv_stats, elision_stats) =
                    scope_hoist_opt::convert_and_elide_flat_region(&after_interop);
                if timing {
                    eprintln!(
                        "[bundle-timing]   entry-flatten/fn-decl-conversion: converted={} skipped_order={} skipped_shape={}",
                        conv_stats.converted, conv_stats.skipped_order, conv_stats.skipped_shape
                    );
                }
                lap("fn_decl_conversion");
                if timing {
                    eprintln!(
                        "[bundle-timing]   entry-flatten/export-elision: modules={} elided_keys={} kept={} kept_registry={} kept_cross_chunk={} kept_namespace={} kept_string_indexed={} kept_barrel_glue={} kept_other={} rhs_normalized={} rhs_skipped_impure={}",
                        elision_stats.modules,
                        elision_stats.elided_keys,
                        elision_stats.kept,
                        elision_stats.kept_registry,
                        elision_stats.kept_cross_chunk,
                        elision_stats.kept_namespace,
                        elision_stats.kept_string_indexed,
                        elision_stats.kept_barrel_glue,
                        elision_stats.kept_other,
                        elision_stats.rhs_normalized,
                        elision_stats.rhs_skipped_impure,
                    );
                }
                lap("export_elision");
                out
            } else {
                let after_fn_decl_conv = if no_fn_decl_conv {
                    after_interop
                } else {
                    let (converted, stats) =
                        scope_hoist_opt::convert_flat_region_function_declarations_to_var(
                            &after_interop,
                        );
                    if timing {
                        eprintln!(
                            "[bundle-timing]   entry-flatten/fn-decl-conversion: converted={} skipped_order={} skipped_shape={}",
                            stats.converted, stats.skipped_order, stats.skipped_shape
                        );
                    }
                    converted
                };
                lap("fn_decl_conversion");
                let processed_body = if no_export_elision {
                    after_fn_decl_conv
                } else {
                    let (elided, stats) =
                        scope_hoist_opt::elide_same_chunk_export_bindings(&after_fn_decl_conv);
                    if timing {
                        eprintln!(
                            "[bundle-timing]   entry-flatten/export-elision: modules={} elided_keys={} kept={} kept_registry={} kept_cross_chunk={} kept_namespace={} kept_string_indexed={} kept_barrel_glue={} kept_other={}",
                            stats.modules,
                            stats.elided_keys,
                            stats.kept,
                            stats.kept_registry,
                            stats.kept_cross_chunk,
                            stats.kept_namespace,
                            stats.kept_string_indexed,
                            stats.kept_barrel_glue,
                            stats.kept_other,
                        );
                    }
                    elided
                };
                lap("export_elision");
                processed_body
            };

            entry_code.push_str(&processed_body);
        }

        entry_code.push_str("// Execute entry point\n");
        entry_code.push_str(&entry_bootstrap_js(entry.id, &entry_static_imports));

        let chunks: Vec<ChunkArtifact> = split_result
            .chunks
            .iter()
            .filter(|c| c.chunk_type != splitting::ChunkType::Entry)
            .map(|chunk| {
                let chunk_modules = modules_for(&chunk.modules);
                let module_ids: Vec<usize> = chunk_modules.iter().map(|m| m.id).collect();
                let mut body = String::new();
                for module in &chunk_modules {
                    body.push_str(&format_module_define(module));
                }
                let code = format!(
                    "__jet__.registerChunk({:?}, function() {{\n{}}});\n",
                    chunk.name, body
                );
                ChunkArtifact {
                    name: chunk.name.clone(),
                    code,
                    imports: chunk.imports.clone(),
                    module_ids,
                }
            })
            .collect();

        let preload_hints = split_result.preload_hints;

        Ok(Some((entry_code, chunks, preload_hints)))
    }
}

/// Build code-splitting edges from the module graph: one `SplitEdgeId` per
/// graph dependency edge, `is_dynamic` set from `EdgeKind::DynamicImport`.
/// Free function (not a `Bundler` method) so it's unit-testable directly
/// against a hand-built `ModuleGraph`.
///
/// Edges are translated from the graph's own path-keyed nodes into
/// `CompiledModule::id` — the bundler's stable numeric identity — via
/// `modules`. An edge endpoint with no corresponding `CompiledModule` (e.g.
/// a graph node for a filtered-out asset/JSON module that never made it
/// into the compiled set) is skipped rather than guessed at: it cannot
/// affect chunk membership either way, and skipping it keeps the
/// translation exact instead of falling back to path-string matching —
/// which is the entire bug this keys away from. See `splitting.rs`'s module
/// doc comment for why chunk partitioning must not be keyed on `PathBuf`.
/// @issue #1930
/// @issue #1941
fn split_edges_from_graph(
    graph: &ModuleGraph,
    modules: &[CompiledModule],
) -> Vec<splitting::SplitEdgeId> {
    let path_to_id: HashMap<&PathBuf, usize> = modules.iter().map(|m| (&m.path, m.id)).collect();
    let mut edges = Vec::new();
    for id in graph.all_node_ids() {
        let Some(node) = graph.get_node(id) else {
            continue;
        };
        let Some(&from) = path_to_id.get(&node.path) else {
            continue;
        };
        for (dep_id, kind) in graph.dependencies(id) {
            let Some(dep_node) = graph.get_node(dep_id) else {
                continue;
            };
            let Some(&to) = path_to_id.get(&dep_node.path) else {
                continue;
            };
            edges.push(splitting::SplitEdgeId {
                from,
                to,
                is_dynamic: kind == graph::EdgeKind::DynamicImport,
            });
        }
    }
    edges
}

/// Format one compiled module as a `__jet__.define(id, function(...){...})`
/// block. Shared by the single-file runtime bundle
/// (`generate_bundle_with_runtime`) and per-chunk code-splitting output
/// (`Bundler::generate_split_bundle`) so both emit byte-identical module
/// blocks.
fn format_module_define(module: &CompiledModule) -> String {
    let module_path = module.path.to_string_lossy();
    format!(
        "// Module {}: {}\n__jet__.define({}, function(require, module, exports) {{\n{}\n}});\n\n",
        module.id, module_path, module.id, module.code
    )
}

/// Build the entry chunk's final statement: run `entry_id` immediately once
/// its static shared/manual chunk dependencies (if any) have actually
/// loaded.
///
/// `static_imports` must already exclude async (dynamic-`import()`) chunk
/// names — see the call site in `Bundler::generate_split_bundle` — so this
/// never eagerly fetches a chunk that is supposed to load on demand.
///
/// - Empty (the common case: no shared/manual chunk is a static entry
///   dependency): emits the pre-#1963 bare `__jet__.require(<id>);\n` byte
///   for byte, so graphs with no such chunk keep byte-identical output.
/// - Non-empty: a `ChunkType::Shared` chunk (auto-detected `shared` OR a
///   `[build.manual_chunks]` chunk) the entry statically depends on must
///   finish loading — and self-registering via `__jet__.registerChunk` —
///   before `require` can resolve it, so the entry is only required inside
///   the `Promise.all(...).then(...)` success callback. Pre-#1963 this was a
///   bare synchronous `require(entry.id)`, which threw
///   `Uncaught Error: Module not found: <id>` in a real page because nothing
///   had ever called `__jet__.loadChunk` for the dependency first (#1948's
///   STOP clause / #1930-#1931's carried-over gap).
/// @issue #1963
fn entry_bootstrap_js(entry_id: usize, static_imports: &[&str]) -> String {
    if static_imports.is_empty() {
        return format!("__jet__.require({entry_id});\n");
    }
    let chunk_list = static_imports
        .iter()
        .map(|name| format!("{name:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "Promise.all([{chunk_list}].map(__jet__.loadChunk)).then(function() {{\n  __jet__.require({entry_id});\n}}, function(err) {{\n  console.error('jet: failed to load startup chunk', err);\n  throw err;\n}});\n"
    )
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

    for module in modules {
        bundle.push_str(&format_module_define(module));
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
/// @spec apps/jet/docs/build-fails-loudly-on-unresolved-bare-specifiers.md
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
            id: test_module_id(path),
            path: std::path::PathBuf::from(path),
            code: code.to_string(),
            source_map: None,
            dependencies: Vec::new(),
            hash: String::new(),
        }
    }

    fn make_compiled_with_id(id: usize, path: &str, code: &str) -> CompiledModule {
        CompiledModule {
            id,
            path: std::path::PathBuf::from(path),
            code: code.to_string(),
            source_map: None,
            dependencies: Vec::new(),
            hash: String::new(),
        }
    }

    fn test_module_id(path: &str) -> usize {
        match path {
            "dep.js" | "config.js" | "lib.js" => 1,
            "debug.js" => 2,
            _ => 0,
        }
    }

    fn js_parses_without_errors(source: &str) -> bool {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .unwrap();
        parser
            .parse(source, None)
            .map(|tree| !tree.root_node().has_error())
            .unwrap_or(false)
    }

    fn first_js_parse_error(source: &str) -> Option<String> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(source, None)?;
        let node = first_error_node(tree.root_node())?;
        let start = node.start_byte().saturating_sub(160);
        let end = (node.end_byte() + 160).min(source.len());
        let pos = node.start_position();
        Some(format!(
            "row={} column={} byte={} kind={} snippet={}",
            pos.row,
            pos.column,
            node.start_byte(),
            node.kind(),
            source[start..end].replace('\n', "\\n")
        ))
    }

    fn first_error_node(node: tree_sitter::Node<'_>) -> Option<tree_sitter::Node<'_>> {
        if node.is_error() || node.is_missing() {
            return Some(node);
        }
        if !node.has_error() {
            return None;
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = first_error_node(child) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn test_tree_shaking_rescues_modules_required_by_retained_transformed_code() {
        let bundler = Bundler::new(BundleOptions::default()).unwrap();
        let modules = vec![
            make_compiled_with_id(
                0,
                "entry.js",
                r#"var live = require(2)["default"] || require(2); live();"#,
            ),
            make_compiled_with_id(1, "unused.js", r#"exports.default = function unused() {};"#),
            make_compiled_with_id(
                2,
                "live-index.js",
                r#"module.exports["default"] = require(3)["default"]; var __re = require(3);"#,
            ),
            make_compiled_with_id(3, "live.js", r#"exports.default = function live() {};"#),
        ];

        let shaken = bundler.apply_tree_shaking(modules, Path::new("entry.js"));
        let ids: HashSet<usize> = shaken.iter().map(|module| module.id).collect();

        assert!(ids.contains(&0), "{ids:?}");
        assert!(
            ids.contains(&2),
            "retained transformed require(2) must rescue module 2: {ids:?}"
        );
        assert!(
            ids.contains(&3),
            "rescued module 2's transformed require(3) must also rescue module 3: {ids:?}"
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // WI #1947 round 2 — dce elimination-walk skip-filter (apply_tree_shaking)
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_apply_tree_shaking_retained_module_with_no_eliminated_references_is_byte_untouched() {
        // dce::eliminate_require_reexports_to_eliminated_modules is skipped
        // outright when a retained module's outgoing numeric require ids
        // (captured by the reachability DFS) are disjoint from
        // eliminated_module_ids. retained.js only ever requires live-dep.js
        // (id 3) — never dead.js (id 1, genuinely unreachable) — so its code
        // must come out of apply_tree_shaking byte-for-byte, not merely
        // "equivalent": a filter that engaged when it shouldn't could still
        // coincidentally produce correct-looking output under a looser check.
        let bundler = Bundler::new(BundleOptions::default()).unwrap();
        let retained_code = r#"var __re = require(3); Object.keys(__re).forEach(function (k) { module.exports[k] = __re[k]; }); exports.live = function live() {};"#;
        let modules = vec![
            make_compiled_with_id(0, "entry.js", r#"require(2); require(3);"#),
            make_compiled_with_id(1, "dead.js", r#"exports.default = function dead() {};"#),
            make_compiled_with_id(2, "retained.js", retained_code),
            make_compiled_with_id(3, "live-dep.js", r#"exports.a = 1;"#),
        ];

        let shaken = bundler.apply_tree_shaking(modules, Path::new("entry.js"));
        let retained = shaken
            .iter()
            .find(|m| m.id == 2)
            .expect("retained.js is required by entry.js and must survive");
        assert_eq!(
            retained.code, retained_code,
            "a retained module referencing no eliminated id must pass through the elimination stage byte-for-byte, got: {}",
            retained.code
        );
    }

    #[test]
    fn test_apply_tree_shaking_still_eliminates_unreachable_module_alongside_skip_filter() {
        // The skip-filter only gates the per-retained-module require-reexport
        // GLUE cleanup walk — it must never affect which modules the outer
        // reachability DFS decides to keep. dead.js is never required by
        // anything reachable and must still be eliminated.
        let bundler = Bundler::new(BundleOptions::default()).unwrap();
        let modules = vec![
            make_compiled_with_id(0, "entry.js", r#"require(2); require(3);"#),
            make_compiled_with_id(1, "dead.js", r#"exports.default = function dead() {};"#),
            make_compiled_with_id(
                2,
                "retained.js",
                r#"var __re = require(3); Object.keys(__re).forEach(function (k) { module.exports[k] = __re[k]; }); exports.live = function live() {};"#,
            ),
            make_compiled_with_id(3, "live-dep.js", r#"exports.a = 1;"#),
        ];

        let shaken = bundler.apply_tree_shaking(modules, Path::new("entry.js"));
        let ids: HashSet<usize> = shaken.iter().map(|m| m.id).collect();
        assert!(
            !ids.contains(&1),
            "dead.js is never required by anything reachable and must still be eliminated: {ids:?}"
        );
        assert!(
            ids.contains(&2) && ids.contains(&3),
            "retained.js/live-dep.js must survive: {ids:?}"
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // WI #2134 — stmt-DCE gated off on the entry-flatten path
    // ──────────────────────────────────────────────────────────────────

    /// Two-module fixture whose only interesting export is `dead`:
    /// reachable (a numeric `require(1)` keeps vendor.js in
    /// `apply_tree_shaking`'s outer DFS, which only understands numeric
    /// require ids), and narrowed by the tree-shake analysis to exactly
    /// `used_exports = {"used"}` via the *string-literal* destructure
    /// `const { used } = require("./vendor.js")` — `extract_cjs_require_bindings`
    /// only narrows string-literal specifiers, never numeric ones. Both
    /// forms are needed on entry.js: the numeric require keeps vendor.js in
    /// `reachable`, and the string-literal require narrows `used_exports`
    /// past the `used.is_empty()` early return that would otherwise skip
    /// `shake_module`/`apply_stmt_dce` entirely (as every other synthetic
    /// fixture in this file, which only uses numeric requires, already
    /// does). `deadFn` has no side effects and is never referenced from
    /// anything live, so it is exactly the shape
    /// `eliminate_dead_top_level_declarations` prunes when stmt-dce runs;
    /// `helper` is referenced from the live `used` export so it stays live
    /// either way and isn't part of either assertion.
    fn stmt_dce_gate_fixture_modules() -> Vec<CompiledModule> {
        vec![
            make_compiled_with_id(
                0,
                "entry.js",
                "require(1);\nconst { used } = require(\"./vendor.js\");\nused();\n",
            ),
            make_compiled_with_id(
                1,
                "vendor.js",
                "function helper() { return 'HELPER_MARKER'; }\nexports.used = function used() { return helper(); };\nexports.dead = function deadFn() { return 'DEAD_MARKER_TEXT'; };\n",
            ),
        ]
    }

    /// Single test, deliberately not split further: `JET_NO_STMT_DCE` /
    /// `JET_FORCE_STMT_DCE` are process-global env vars, and `cargo test`
    /// runs `#[test]` fns concurrently by default, so two separate test
    /// functions each mutating these same two vars around their own
    /// `apply_tree_shaking` call would race each other (observed directly:
    /// splitting this into two tests made the first one flake under the
    /// default multi-threaded runner, because the second test's
    /// `JET_FORCE_STMT_DCE=1` window could be observed mid-flight by the
    /// first). Keeping every case sequential in one test body is the fix,
    /// not a serial-test framework this codebase doesn't otherwise use for
    /// its other env-hatch tests (`JET_NO_SURVIVOR_FILTER`/
    /// `JET_NO_PASS_GATES`), which get away with it only because they don't
    /// currently have a same-file sibling test touching the same var.
    #[test]
    fn test_apply_tree_shaking_gates_stmt_dce_off_on_entry_flatten_path() {
        fn vendor_code(bundler: &Bundler) -> String {
            let shaken =
                bundler.apply_tree_shaking(stmt_dce_gate_fixture_modules(), Path::new("entry.js"));
            shaken
                .into_iter()
                .find(|m| m.id == 1)
                .expect("vendor.js is required by entry.js and must survive")
                .code
        }

        std::env::remove_var("JET_NO_STMT_DCE");
        std::env::remove_var("JET_FORCE_STMT_DCE");

        let non_flatten = Bundler::new(BundleOptions {
            splitting: false,
            ..Default::default()
        })
        .unwrap();
        let flatten = Bundler::new(BundleOptions {
            splitting: true,
            ..Default::default()
        })
        .unwrap();

        // No env hatch involved — the gate `apply_tree_shaking` derives
        // from `self.splitting` alone, both ways.
        let code = vendor_code(&non_flatten);
        assert!(
            !code.contains("DEAD_MARKER_TEXT"),
            "non-flatten build (splitting=false) must run stmt-dce and prune \
             the unused `dead` export, got: {code}"
        );
        let code = vendor_code(&flatten);
        assert!(
            code.contains("DEAD_MARKER_TEXT"),
            "entry-flatten build (splitting=true) must gate stmt-dce off and \
             leave the unused `dead` export in place, got: {code}"
        );

        // JET_FORCE_STMT_DCE=1 forces the pass back on even on the flatten
        // path (A/B + escape hatch).
        std::env::set_var("JET_FORCE_STMT_DCE", "1");
        let code = vendor_code(&flatten);
        assert!(
            !code.contains("DEAD_MARKER_TEXT"),
            "JET_FORCE_STMT_DCE=1 must force stmt-dce back on even on the \
             flatten path, got: {code}"
        );

        // JET_NO_STMT_DCE=1 still wins when both are set (mirrors the "off
        // flag always wins" precedence convention used elsewhere, e.g.
        // `build_minify_enabled_from_matches` in cli.rs).
        std::env::set_var("JET_NO_STMT_DCE", "1");
        let code = vendor_code(&flatten);
        assert!(
            code.contains("DEAD_MARKER_TEXT"),
            "JET_NO_STMT_DCE=1 must still win over JET_FORCE_STMT_DCE=1 when \
             both are set, got: {code}"
        );

        std::env::remove_var("JET_NO_STMT_DCE");
        std::env::remove_var("JET_FORCE_STMT_DCE");
    }

    #[tokio::test]
    async fn test_per_module_defines_use_syntax_safe_dce() {
        let tmp = tempfile::TempDir::new().unwrap();
        let entry = tmp.path().join("entry.js");
        std::fs::write(
            &entry,
            r#"
if (process.env.NODE_ENV !== "production") {
  if (window.__JET_DEV_FLAG__) {
    console.log("dev branch");
  } else {
    console.log("inner dev else");
  }
} else {
  console.log("prod branch");
}
export const value = 1;
"#,
        )
        .unwrap();

        let bundler = Bundler::new(BundleOptions {
            entry: entry.clone(),
            output_dir: tmp.path().join("dist"),
            minify: true,
            source_maps: false,
            externalize_all_packages: false,
            transform_options: crate::transform::TransformOptions {
                dev_mode: false,
                ..Default::default()
            },
            defines: crate::bundler::define::production_defines(),
            ..Default::default()
        })
        .unwrap();

        bundler.build_graph(&entry).await.unwrap();
        let (modules, _has_cycle) = bundler.transform_modules().await.unwrap();
        let canonical_entry = std::fs::canonicalize(&entry).unwrap();
        let compiled = modules
            .iter()
            .find(|module| {
                module.path == entry
                    || module.path == canonical_entry
                    || module.path.ends_with(std::path::Path::new("entry.js"))
            })
            .expect("entry module should be transformed");

        assert!(
            js_parses_without_errors(&compiled.code),
            "per-module define+DCE output must remain valid JS:\n{}",
            compiled.code
        );
        assert!(compiled.code.contains("prod branch"), "{}", compiled.code);
        assert!(
            !compiled.code.contains("dev branch"),
            "production define+DCE should remove dev-only branch:\n{}",
            compiled.code
        );
    }

    // #1995 — per-module pass gating equivalence tests.

    #[tokio::test]
    async fn test_gate1_probe_folds_literal_compare_with_no_define_change() {
        let tmp = tempfile::TempDir::new().unwrap();
        let entry = tmp.path().join("entry.js");
        // No `process.env.NODE_ENV` / `process.env` / `__DEV__` token
        // anywhere in this module's source, so `replace_defines` is a
        // no-op for it (Gate 1 condition (a) is false). The literal
        // string compare below is unrelated to any configured define;
        // only the `could_fold_static_conditional` probe (condition (b))
        // can prove this module still needs the fold+syntax-DCE pass.
        std::fs::write(
            &entry,
            r#"
if ("a" === "a") {
  keep();
} else {
  drop();
}
export const value = 1;
"#,
        )
        .unwrap();

        let bundler = Bundler::new(BundleOptions {
            entry: entry.clone(),
            output_dir: tmp.path().join("dist"),
            minify: false,
            source_maps: false,
            externalize_all_packages: false,
            transform_options: crate::transform::TransformOptions {
                dev_mode: false,
                ..Default::default()
            },
            // Non-empty so the gated `else` branch runs at all; this
            // specific module never references any of these tokens, so
            // `replace_defines` leaves it byte-identical.
            defines: crate::bundler::define::production_defines(),
            ..Default::default()
        })
        .unwrap();

        bundler.build_graph(&entry).await.unwrap();
        let (modules, _has_cycle) = bundler.transform_modules().await.unwrap();
        let compiled = modules
            .iter()
            .find(|m| m.path.ends_with(std::path::Path::new("entry.js")))
            .expect("entry module should be transformed");

        assert!(
            compiled.code.contains("keep()"),
            "true branch of a literal string compare must survive:\n{}",
            compiled.code
        );
        assert!(
            !compiled.code.contains("drop()"),
            "dead branch of a literal string compare unrelated to any \
             define must still be eliminated even though replace_defines \
             made no change to this module:\n{}",
            compiled.code
        );
    }

    #[tokio::test]
    async fn test_gate_skip_produces_byte_identical_output_to_ungated() {
        let tmp = tempfile::TempDir::new().unwrap();
        let entry = tmp.path().join("entry.js");
        // No comparison operators, no boolean literals, no require-like
        // calls anywhere: both Gate 1 and Gate 2 should skip their passes
        // entirely for this module.
        std::fs::write(
            &entry,
            r#"
export function add(a, b) {
  return a + b;
}
export const value = add(1, 2);
"#,
        )
        .unwrap();

        async fn build(entry: &std::path::PathBuf, out_dir: std::path::PathBuf) -> String {
            let bundler = Bundler::new(BundleOptions {
                entry: entry.to_path_buf(),
                output_dir: out_dir,
                minify: false,
                source_maps: false,
                externalize_all_packages: false,
                transform_options: crate::transform::TransformOptions {
                    dev_mode: false,
                    ..Default::default()
                },
                defines: crate::bundler::define::production_defines(),
                ..Default::default()
            })
            .unwrap();
            bundler.build_graph(entry).await.unwrap();
            let (modules, _has_cycle) = bundler.transform_modules().await.unwrap();
            modules
                .into_iter()
                .find(|m| m.path.ends_with(std::path::Path::new("entry.js")))
                .expect("entry module should be transformed")
                .code
        }

        // JET_NO_PASS_GATES=1 forces both gates open, reproducing
        // pre-#1995 (always-run) behavior exactly. Set/unset around each
        // build so the two calls in this test never observe each other's
        // state; only ever forces gates *open* for any other test that
        // might transiently observe it mid-flight, which cannot change
        // those tests' correctness assertions (running more passes than
        // strictly necessary is always output-safe).
        std::env::remove_var("JET_NO_PASS_GATES");
        let gated = build(&entry, tmp.path().join("dist-gated")).await;

        std::env::set_var("JET_NO_PASS_GATES", "1");
        let ungated = build(&entry, tmp.path().join("dist-ungated")).await;
        std::env::remove_var("JET_NO_PASS_GATES");

        assert_eq!(
            gated, ungated,
            "gate skip must be byte-identical to the ungated \
             (JET_NO_PASS_GATES=1) pipeline:\ngated:\n{gated}\nungated:\n{ungated}"
        );
    }

    // WI #1995 round 4 — survivors-only transform tests.

    /// The round-3 blocker case (see `Bundler::implicit_edges`'s doc
    /// comment): a `.tsx` entry with zero textual `react`/`react/jsx-
    /// runtime` reference anywhere in its own source must still record
    /// (and keep live) the implicit `react/jsx-runtime` edge `build_graph`
    /// fabricates from the file extension alone. Regression proof for the
    /// implicit-edge side channel `compute_transform_survivors` unions
    /// into its raw-source liveness pre-pass.
    #[tokio::test]
    async fn test_survivor_filter_keeps_implicit_jsx_runtime_edge_live() {
        let tmp = tempfile::TempDir::new().unwrap();
        let entry = tmp.path().join("entry.tsx");
        std::fs::write(
            &entry,
            r#"
export function App() {
  return <div>Hello</div>;
}
"#,
        )
        .unwrap();

        // Minimal resolvable `react/jsx-runtime` package, same shape as
        // `tests/test-runner/test_runner_smoke.rs`'s fixture.
        let node_modules = tmp.path().join("node_modules");
        let react = node_modules.join("react");
        std::fs::create_dir_all(&react).unwrap();
        std::fs::write(
            react.join("package.json"),
            r#"{"name":"react","type":"module","exports":{"./jsx-runtime":"./jsx-runtime.js"}}"#,
        )
        .unwrap();
        std::fs::write(
            react.join("jsx-runtime.js"),
            r#"export const Fragment = Symbol.for("fragment");
export const jsx = (tag, props) => ({ tag, props });
export const jsxs = jsx;
"#,
        )
        .unwrap();

        let bundler = Bundler::new(BundleOptions {
            entry: entry.clone(),
            output_dir: tmp.path().join("dist"),
            minify: false,
            source_maps: false,
            externalize_all_packages: false,
            transform_options: crate::transform::TransformOptions {
                dev_mode: false,
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap();

        // WI #1995 round 6: the survivors-only filter is default-on — this
        // test exists specifically to exercise it, so just defensively
        // ensure the opt-out escape hatch (`JET_NO_SURVIVOR_FILTER=1`)
        // isn't leaked from a prior test (env vars are process-global
        // across parallel test threads; every test that sets it cleans up,
        // but this guards the default path explicitly rather than
        // assuming that).
        std::env::remove_var("JET_NO_SURVIVOR_FILTER");

        bundler.build_graph(&entry).await.unwrap();
        assert_eq!(
            bundler.implicit_edges.lock().len(),
            1,
            "build_graph must record exactly one implicit edge (entry.tsx -> \
             react/jsx-runtime) for this fixture"
        );

        let (modules, _has_cycle) = bundler.transform_modules().await.unwrap();
        assert!(
            modules
                .iter()
                .any(|m| m.path.ends_with(std::path::Path::new("jsx-runtime.js"))),
            "react/jsx-runtime must still be transformed even though nothing \
             in entry.tsx's own source textually imports it: {:?}",
            modules.iter().map(|m| &m.path).collect::<Vec<_>>()
        );
    }

    /// A subtree (`dead_root.js` -> `dead_leaf.js`) reachable only through a
    /// `process.env.NODE_ENV`-gated dev-only branch must be skipped by the
    /// pre-transform survivors-only filter (default-on since WI #1995
    /// round 6; opted OUT of via `JET_NO_SURVIVOR_FILTER=1`), and the
    /// filtered build must stay byte-identical (after the real elimination
    /// authority, `apply_tree_shaking`, runs on both) to the escape-hatch
    /// (filter off) build.
    #[tokio::test]
    async fn test_survivor_filter_skips_dead_only_subtree_and_stays_byte_identical() {
        async fn build(
            fixture_dir: &std::path::Path,
            no_filter: bool,
        ) -> (Vec<CompiledModule>, Vec<CompiledModule>) {
            let entry = fixture_dir.join("entry.js");
            std::fs::write(
                &entry,
                r#"
import { live } from './live.js';
if (process.env.NODE_ENV !== "production") {
  const deadRoot = require('./dead_root.js');
  console.log('dev branch', deadRoot);
} else {
  console.log('prod branch');
}
export const value = live();
"#,
            )
            .unwrap();
            std::fs::write(
                fixture_dir.join("live.js"),
                "export function live() { return 'LIVE_MARKER'; }\n",
            )
            .unwrap();
            std::fs::write(
                fixture_dir.join("dead_root.js"),
                "const deadLeaf = require('./dead_leaf.js');\n\
                 module.exports = { deadLeaf, marker: 'DEAD_ROOT_MARKER' };\n",
            )
            .unwrap();
            std::fs::write(
                fixture_dir.join("dead_leaf.js"),
                "module.exports = { marker: 'DEAD_LEAF_MARKER' };\n",
            )
            .unwrap();

            // WI #1995 round 6: the survivors-only filter is default-on —
            // `no_filter=true` here means "exercise the opt-out escape
            // hatch" (`JET_NO_SURVIVOR_FILTER=1`), `no_filter=false` means
            // "exercise the (now default) filtered path", so defensively
            // ensure the escape hatch isn't set for that branch.
            if no_filter {
                std::env::set_var("JET_NO_SURVIVOR_FILTER", "1");
            } else {
                std::env::remove_var("JET_NO_SURVIVOR_FILTER");
            }

            let bundler = Bundler::new(BundleOptions {
                entry: entry.clone(),
                output_dir: fixture_dir.join("dist"),
                minify: false,
                source_maps: false,
                externalize_all_packages: false,
                transform_options: crate::transform::TransformOptions {
                    dev_mode: false,
                    ..Default::default()
                },
                defines: crate::bundler::define::production_defines(),
                ..Default::default()
            })
            .unwrap();

            bundler.build_graph(&entry).await.unwrap();
            let (transformed, _has_cycle) = bundler.transform_modules().await.unwrap();
            std::env::remove_var("JET_NO_SURVIVOR_FILTER");
            let shaken = bundler.apply_tree_shaking(transformed.clone(), &entry);
            (transformed, shaken)
        }

        let tmp_filtered = tempfile::TempDir::new().unwrap();
        let (transformed_filtered, shaken_filtered) = build(tmp_filtered.path(), false).await;

        let tmp_unfiltered = tempfile::TempDir::new().unwrap();
        let (transformed_unfiltered, shaken_unfiltered) = build(tmp_unfiltered.path(), true).await;

        let has_module = |modules: &[CompiledModule], name: &str| {
            modules
                .iter()
                .any(|m| m.path.ends_with(std::path::Path::new(name)))
        };
        assert!(
            has_module(&transformed_unfiltered, "dead_root.js")
                && has_module(&transformed_unfiltered, "dead_leaf.js"),
            "default (survivor filter off) build must still transform the \
             dead-only subtree: {:?}",
            transformed_unfiltered
                .iter()
                .map(|m| &m.path)
                .collect::<Vec<_>>()
        );
        assert!(
            !has_module(&transformed_filtered, "dead_root.js")
                && !has_module(&transformed_filtered, "dead_leaf.js"),
            "survivor filter must skip the dead-only subtree (reachable only \
             through a process.env.NODE_ENV dev-only branch): {:?}",
            transformed_filtered
                .iter()
                .map(|m| &m.path)
                .collect::<Vec<_>>()
        );

        // Byte-identical FINAL output: after both paths run through the
        // real elimination authority (`apply_tree_shaking`), the surviving
        // (filename, code) sets must match exactly.
        let final_set = |modules: &[CompiledModule]| -> std::collections::BTreeMap<String, String> {
            modules
                .iter()
                .map(|m| {
                    let name = m.path.file_name().unwrap().to_string_lossy().into_owned();
                    (name, m.code.clone())
                })
                .collect()
        };
        assert_eq!(
            final_set(&shaken_filtered),
            final_set(&shaken_unfiltered),
            "survivor filter must be byte-identical to the default \
             (survivor filter off) build after apply_tree_shaking"
        );
        assert!(
            !final_set(&shaken_filtered).contains_key("dead_root.js"),
            "apply_tree_shaking itself must also have eliminated the dead \
             subtree (both paths converge on the same final answer)"
        );
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
    fn test_phase2_dce_keeps_styled_components_entry_import_bindings() {
        let source = r##"import React, { useState } from "react";
import { createRoot } from "react-dom/client";
import styled, { createGlobalStyle, css } from "styled-components";

const GlobalStyle = createGlobalStyle`
  body { margin: 0; }
`;
const Matrix = styled.main`
  min-height: 100vh;
`;
const Button = styled.button`
  ${(props) => css`
    background: ${props.$accent || "#2563eb"};
  `}
`;

function App() {
  const [active] = useState(0);
  return <Matrix><GlobalStyle /><Button $accent="#2563eb">{active}</Button></Matrix>;
}

createRoot(document.getElementById("root")!).render(<App />);"##;
        let transformer =
            crate::transform::Transformer::new(crate::transform::TransformOptions::default());
        let entry = transformer
            .transform_js_with_context(source, std::path::Path::new("entry.tsx"), &HashMap::new())
            .unwrap();
        let modules = vec![make_compiled("entry.tsx", &entry.code)];
        let raw = scope_hoist::generate_flattened_bundle(&modules);
        let after_r4 = scope_hoist::inline_cross_module_constants(&raw);
        let after_r5 = scope_hoist::eliminate_unused_exports(&after_r4);

        for name in [
            "_m0_jsx",
            "_m0_jsxs",
            "_m0_useState",
            "_m0_createRoot",
            "_m0_styled",
            "_m0_createGlobalStyle",
            "_m0_css",
        ] {
            assert!(
                after_r5.contains(name),
                "R5 must keep live styled-components entry binding {name}: {after_r5}"
            );
        }
    }

    #[tokio::test]
    async fn test_phase2_real_styled_components_fixture_keeps_entry_import_bindings() {
        let fixture_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/dom-production-build/styled-components-visual");
        let entry = fixture_root.join("src/main.tsx");
        assert!(
            entry.exists(),
            "styled-components visual fixture entry must exist at {}",
            entry.display()
        );

        let mut resolve_options = crate::resolver::ResolveOptions::for_browser_production();
        resolve_options.base_dirs = vec![fixture_root.clone()];

        let bundler = Bundler::new(BundleOptions {
            entry: entry.clone(),
            output_dir: fixture_root.join("dist-test"),
            minify: true,
            source_maps: false,
            resolve_options,
            externalize_all_packages: false,
            transform_options: crate::transform::TransformOptions {
                dev_mode: false,
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap();

        bundler.build_graph(&entry).await.unwrap();
        bundler.check_unresolved_deps().unwrap();
        let (modules, has_cycle) = bundler.transform_modules().await.unwrap();
        assert!(!has_cycle, "fixture should stay on Phase 2 flattening path");
        let modules = bundler.apply_tree_shaking(modules, &entry);
        let raw = scope_hoist::generate_flattened_bundle(&modules);
        let after_r4 = scope_hoist::inline_cross_module_constants(&raw);
        let after_r5 = scope_hoist::eliminate_unused_exports(&after_r4);
        let defines = crate::bundler::define::production_defines();
        let dump = |stage: &str, code: &str| {
            if let Ok(dir) = std::env::var("JET_TEST_DUMP_STAGES") {
                let _ = std::fs::create_dir_all(&dir);
                let _ = std::fs::write(format!("{dir}/{stage}.js"), code);
            }
        };
        let mut post_processed = crate::bundler::define::replace_defines(&after_r5, &defines);
        dump("a-defines", &post_processed);
        post_processed = crate::bundler::minify::minify_js(&post_processed, &[]);
        dump("b-minify", &post_processed);
        post_processed = crate::bundler::minify::replace_bool_literals(&post_processed);
        dump("c-bool", &post_processed);
        post_processed = crate::bundler::mangle::mangle_variables(&post_processed);
        dump("d-mangle", &post_processed);
        post_processed = crate::bundler::fold::fold_constants(&post_processed);
        dump("e-fold", &post_processed);

        // Pre-mangle stages must keep the entry bindings under their
        // generated names (R4/R5 must not eliminate live bindings).
        for (stage, code) in [
            ("raw", &raw),
            ("after_r4", &after_r4),
            ("after_r5", &after_r5),
        ] {
            for name in [
                "_m0_jsx",
                "_m0_jsxs",
                "_m0_useState",
                "_m0_createRoot",
                "_m0_styled",
                "_m0_createGlobalStyle",
                "_m0_css",
            ] {
                let declaration = format!("var {name}");
                assert!(
                    code.contains(&declaration),
                    "{stage} must keep live styled-components fixture entry binding declaration {declaration}"
                );
            }
        }
        // After mangling the bindings may be legitimately renamed (the
        // scope model attributes IIFE-body declarations correctly now);
        // what must hold is that the optimized bundle still parses.
        if let Ok(dump) = std::env::var("JET_TEST_DUMP_POST") {
            let _ = std::fs::write(&dump, &post_processed);
        }
        assert!(
            crate::bundler::dce::js_parses_without_errors(&post_processed),
            "post_processed bundle must parse"
        );
    }

    #[tokio::test]
    async fn test_phase2_real_mui_fixture_mangle_candidate_parses_and_compresses() {
        let fixture_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/dom-production-build/mui-visual");
        let entry = fixture_root.join("src/main.tsx");
        assert!(
            entry.exists(),
            "MUI visual fixture entry must exist at {}",
            entry.display()
        );

        let mut resolve_options = crate::resolver::ResolveOptions::for_browser_production();
        resolve_options.base_dirs = vec![fixture_root.clone()];

        let bundler = Bundler::new(BundleOptions {
            entry: entry.clone(),
            output_dir: fixture_root.join("dist-test"),
            minify: true,
            source_maps: false,
            resolve_options,
            externalize_all_packages: false,
            transform_options: crate::transform::TransformOptions {
                dev_mode: false,
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap();

        bundler.build_graph(&entry).await.unwrap();
        bundler.check_unresolved_deps().unwrap();
        let (modules, has_cycle) = bundler.transform_modules().await.unwrap();
        assert!(!has_cycle, "fixture should stay on Phase 2 flattening path");
        let modules = bundler.apply_tree_shaking(modules, &entry);
        let raw = scope_hoist::generate_flattened_bundle(&modules);
        let after_r4 = scope_hoist::inline_cross_module_constants(&raw);
        let after_r5 = scope_hoist::eliminate_unused_exports(&after_r4);
        let defines = crate::bundler::define::production_defines();
        let mut post_processed = crate::bundler::define::replace_defines(&after_r5, &defines);
        post_processed = crate::bundler::dce::eliminate_static_conditionals_syntax(&post_processed);
        post_processed = crate::bundler::minify::minify_js(&post_processed, &[]);
        post_processed = crate::bundler::minify::replace_bool_literals(&post_processed);
        let mangled = crate::bundler::mangle::mangle_variables(&post_processed);

        assert!(
            js_parses_without_errors(&mangled),
            "MUI fixture mangle candidate must parse so CLI does not fall back to unmangled output: {}",
            first_js_parse_error(&mangled).unwrap_or_else(|| "unknown parse error".to_string())
        );
        assert!(
            !mangled.contains("var _m0={exports"),
            "MUI fixture module slot names must be compressed, not emitted unmangled"
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
    fn test_phase2_pipeline_keeps_mui_css_vars_provider_import_bindings() {
        let tmp = tempfile::TempDir::new().unwrap();
        let package_dir = tmp.path().join("node_modules/@mui/material");
        std::fs::create_dir_all(package_dir.join("styles")).unwrap();
        std::fs::write(package_dir.join("package.json"), r#"{"sideEffects":false}"#).unwrap();
        let css_vars_provider = package_dir.join("styles/CssVarsProvider.js");
        let styles_index = package_dir.join("styles/index.js");
        let modules = vec![
            CompiledModule {
                id: 0,
                path: styles_index,
                code: r#"
Object.defineProperty(module.exports, "__esModule", { value: true });
var _CssVarsProvider = require(1);
Object.keys(_CssVarsProvider).forEach(function (key) {
  if (key !== "default") module.exports[key] = _CssVarsProvider[key];
});
"#
                .to_string(),
                source_map: None,
                dependencies: Vec::new(),
                hash: String::new(),
            },
            CompiledModule {
                id: 1,
                path: css_vars_provider,
                code: r#"
Object.defineProperty(module.exports, "__esModule", { value: true });
'use client';
// do not remove the following import
/* eslint-disable @typescript-eslint/no-unused-vars */
// @ts-ignore
var _extends = require(699)["default"] || require(699);
var createCssVarsProvider = require(321)["unstable_createCssVarsProvider"];
var styleFunctionSx = require(643)["default"] || require(643);
var experimental_extendTheme = require(90)["default"] || require(90);
var createTypography = require(686)["default"] || require(686);
var excludeVariablesFromRoot = require(92)["default"] || require(92);
var THEME_ID = require(694)["default"] || require(694);
var defaultConfig = require(88)["defaultConfig"];
const defaultTheme = experimental_extendTheme();
const {
  CssVarsProvider,
  useColorScheme,
  getInitColorSchemeScript: getInitColorSchemeScriptSystem
} = createCssVarsProvider({
  themeId: THEME_ID,
  theme: defaultTheme,
  attribute: defaultConfig.attribute,
  resolveTheme: theme => {
    const newTheme = _extends({}, theme, {
      typography: createTypography(theme.palette, theme.typography)
    });
    newTheme.unstable_sx = function sx(props) {
      return styleFunctionSx({ sx: props, theme: this });
    };
    return newTheme;
  },
  excludeVariablesFromRoot
});
const getInitColorSchemeScript = getInitColorSchemeScriptSystem;
module.exports["getInitColorSchemeScript"] = getInitColorSchemeScript;
module.exports["useColorScheme"] = useColorScheme;
module.exports["Experimental_CssVarsProvider"] = CssVarsProvider;
"#
                .to_string(),
                source_map: None,
                dependencies: Vec::new(),
                hash: String::new(),
            },
            make_compiled("dep.js", "exports.default = function dep() { return {}; };"),
        ];

        let raw = scope_hoist::generate_flattened_bundle(&modules);
        let after_r4 = scope_hoist::inline_cross_module_constants(&raw);
        let after_r5 = scope_hoist::eliminate_unused_exports(&after_r4);
        let minified = crate::bundler::minify::minify_js(&after_r5, &[]);

        assert!(
            minified.contains("var _m1_experimental_extendTheme"),
            "live MUI default import declaration must survive Phase2 pipeline, got: {}",
            minified
        );
        assert!(
            minified.contains("_m1_experimental_extendTheme()"),
            "live MUI default import read must stay tied to declaration, got: {}",
            minified
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

#[cfg(test)]
mod resolved_path_tests {
    use super::*;
    use crate::bundler::types::BundleOptions;

    #[test]
    fn normalize_bundler_path_lexical_collapses_parent_components() {
        assert_eq!(
            normalize_bundler_path_lexical(Path::new("/app/node_modules/pkg/../dep/index.js")),
            PathBuf::from("/app/node_modules/dep/index.js")
        );
    }

    #[test]
    fn resolve_dependency_returns_lexically_normalized_path() {
        let tmp = tempfile::TempDir::new().unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir_all(src.join("nested")).unwrap();
        let importer = src.join("importer.js");
        let target = src.join("target.js");
        std::fs::write(&importer, "import './nested/../target';").unwrap();
        std::fs::write(&target, "export {};").unwrap();

        let bundler = Bundler::new(BundleOptions::default()).unwrap();
        let resolved = bundler
            .resolve_dependency(&importer, "./nested/../target")
            .unwrap();

        assert_eq!(resolved, normalize_bundler_path_lexical(&target));
        assert!(!resolved.to_string_lossy().contains("/../"));
    }
}

/// Pins the #1317 behaviour: `jet build` must fail loudly when a bare
/// specifier can neither be resolved on disk nor was opted into being
/// external, and must continue to silently skip when the user did opt in.
///
/// @spec apps/jet/docs/build-fails-loudly-on-unresolved-bare-specifiers.md
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

    #[tokio::test]
    async fn type_only_declaration_import_is_elided_before_graph_resolution() {
        let tmp = tempfile::tempdir().unwrap();
        let entry = write_fixture(
            tmp.path(),
            &[
                (
                    "entry.ts",
                    "import { CreateWorkspaceValues } from './type';\nexport const marker = (value: CreateWorkspaceValues) => value;\n",
                ),
                (
                    "type.d.ts",
                    "export interface CreateWorkspaceValues { name: string; }\n",
                ),
            ],
        );

        let opts = BundleOptions {
            entry: entry.clone(),
            output_dir: tmp.path().join("dist"),
            ..Default::default()
        };
        let bundler = Bundler::new(opts).unwrap();
        let output = bundler
            .bundle(entry)
            .await
            .expect("type-only declaration import must not become a runtime dependency");

        assert!(
            output.code.contains("marker"),
            "bundle lost value code: {}",
            output.code
        );
        assert!(
            !output.code.contains("CreateWorkspaceValues") && !output.code.contains("./type"),
            "type-only declaration must not survive as a runtime import: {}",
            output.code
        );
    }

    #[tokio::test]
    async fn type_only_unexported_package_subpath_is_elided_before_graph_resolution() {
        let tmp = tempfile::tempdir().unwrap();
        let entry = write_fixture(
            tmp.path(),
            &[
                (
                    "entry.ts",
                    "import type { ClosestEdge } from '@scope/pkg/dist/types/closest-edge';\nexport const marker = 'TYPE_PATH_ELIDED';\n",
                ),
                (
                    "node_modules/@scope/pkg/package.json",
                    "{\"exports\":{\".\":\"./index.js\"}}",
                ),
                (
                    "node_modules/@scope/pkg/index.js",
                    "export const value = 'runtime';\n",
                ),
                (
                    "node_modules/@scope/pkg/dist/types/closest-edge.d.ts",
                    "export interface ClosestEdge { edge: string; }\n",
                ),
            ],
        );

        let opts = BundleOptions {
            entry: entry.clone(),
            output_dir: tmp.path().join("dist"),
            ..Default::default()
        };
        let bundler = Bundler::new(opts).unwrap();
        let output = bundler
            .bundle(entry)
            .await
            .expect("unexported package type path must not become a runtime dependency");

        assert!(output.code.contains("TYPE_PATH_ELIDED"));
        assert!(
            !output.code.contains("@scope/pkg/dist/types/closest-edge"),
            "type-only package subpath must not survive in runtime code: {}",
            output.code
        );
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

/// WI #1930 — `jet build --splitting` chunk codegen core + runtime loader.
///
/// Covers `SplitEdgeId` extraction from a hand-built `ModuleGraph` and
/// structural assertions on `Bundler::generate_split_bundle`'s entry/chunk
/// output (the `dynamicImport`/`chunkManifest`/`registerChunk` runtime
/// wiring). Transform-level dynamic-import lowering on/off is covered in
/// `transform::modules`'s test module; the full multi-file on-disk build
/// (including the OFF-path single-file byte-stability proxy) is covered by
/// the `tests/build/code_splitting.rs` integration test. WI #1932's
/// emergent no-dynamic-import fallback (`generate_split_bundle` returning
/// `None`) is also covered here.
#[cfg(test)]
mod code_splitting_tests {
    use super::*;

    fn compiled(id: usize, path: &str, code: &str) -> CompiledModule {
        CompiledModule {
            id,
            path: PathBuf::from(path),
            code: code.to_string(),
            source_map: None,
            dependencies: Vec::new(),
            hash: String::new(),
        }
    }

    // ── SplitEdge extraction (split_edges_from_graph) ───────────────────

    #[test]
    fn split_edges_from_graph_marks_dynamic_vs_static() {
        let mut graph = ModuleGraph::new();
        let entry = graph.add_module(PathBuf::from("entry.js"), graph::ModuleKind::Script, 0);
        let shared = graph.add_module(PathBuf::from("shared.js"), graph::ModuleKind::Script, 0);
        let lazy = graph.add_module(PathBuf::from("lazy.js"), graph::ModuleKind::Script, 0);
        graph.add_dependency(entry, shared, EdgeKind::Import);
        graph.add_dependency(entry, lazy, EdgeKind::DynamicImport);

        // Ids are assigned by this `modules` list, not by the graph's own
        // `NodeIndex` — the translation in `split_edges_from_graph` goes
        // through path lookup, so these ids need not (and in general will
        // not) match petgraph's internal node indices.
        let modules = vec![
            compiled(0, "entry.js", ""),
            compiled(1, "shared.js", ""),
            compiled(2, "lazy.js", ""),
        ];

        let mut edges = split_edges_from_graph(&graph, &modules);
        edges.sort_by_key(|e| (e.from, e.to));

        assert_eq!(
            edges.len(),
            2,
            "expected exactly the 2 registered edges: {edges:?}"
        );
        let static_edge = edges
            .iter()
            .find(|e| e.to == 1)
            .expect("static edge to shared.js (id 1) missing");
        assert_eq!(static_edge.from, 0);
        assert!(
            !static_edge.is_dynamic,
            "Import edge must not be marked dynamic"
        );

        let dynamic_edge = edges
            .iter()
            .find(|e| e.to == 2)
            .expect("dynamic edge to lazy.js (id 2) missing");
        assert_eq!(dynamic_edge.from, 0);
        assert!(
            dynamic_edge.is_dynamic,
            "DynamicImport edge must be marked dynamic"
        );
    }

    #[test]
    fn split_edges_from_graph_empty_graph_yields_no_edges() {
        let graph = ModuleGraph::new();
        assert!(split_edges_from_graph(&graph, &[]).is_empty());
    }

    // ── generate_split_bundle structural assertions ─────────────────────

    /// entry.js statically imports shared.js and dynamically imports
    /// lazy1.js. Mirrors the shape of the integration fixture in
    /// `tests/build/code_splitting.rs` at unit-test scale.
    fn split_bundle_fixture() -> (String, Vec<ChunkArtifact>, Vec<PreloadHint>) {
        let bundler = Bundler::new(BundleOptions {
            splitting: true,
            ..Default::default()
        })
        .expect("bundler new");

        {
            let mut g = bundler.graph.write();
            let entry = g.add_module(
                PathBuf::from("/fixture/entry.js"),
                graph::ModuleKind::Script,
                0,
            );
            let shared = g.add_module(
                PathBuf::from("/fixture/shared.js"),
                graph::ModuleKind::Script,
                0,
            );
            let lazy = g.add_module(
                PathBuf::from("/fixture/lazy1.js"),
                graph::ModuleKind::Script,
                0,
            );
            g.add_dependency(entry, shared, EdgeKind::Import);
            g.add_dependency(entry, lazy, EdgeKind::DynamicImport);
        }

        let modules = vec![
            compiled(0, "/fixture/entry.js", "__jet__.dynamicImport(2);"),
            compiled(1, "/fixture/shared.js", "exports.shared = 1;"),
            compiled(2, "/fixture/lazy1.js", "exports.lazy = 1;"),
        ];

        bundler
            .generate_split_bundle(&modules)
            .expect("generate_split_bundle should succeed")
            .expect("fixture has a dynamic import boundary; must not fall back to None")
    }

    #[test]
    fn entry_code_contains_dynamic_import_and_chunk_manifest_reference() {
        let (entry_code, _chunks, _preload_hints) = split_bundle_fixture();
        assert!(
            entry_code.contains("dynamicImport"),
            "entry runtime must expose dynamicImport: {entry_code}"
        );
        assert!(
            entry_code.contains("chunkManifest"),
            "entry runtime must reference chunkManifest: {entry_code}"
        );
    }

    #[test]
    fn entry_code_does_not_self_register_as_a_chunk() {
        // Async/shared chunk FILES are wrapped in a `__jet__.registerChunk(
        // "<name>", function(){...})` CALL; the entry chunk is executed
        // directly (`__jet__.require(<entry id>)`) and must never be
        // wrapped in that call — even though its own runtime DEFINES the
        // registerChunk capability (a bare `registerChunk: registerChunk`
        // property) for other chunks to call into once loaded.
        let (entry_code, _chunks, _preload_hints) = split_bundle_fixture();
        assert!(
            !entry_code.contains("__jet__.registerChunk("),
            "entry code must not call __jet__.registerChunk(...): {entry_code}"
        );
        assert!(
            entry_code.contains("__jet__.require("),
            "entry code must directly require the entry module: {entry_code}"
        );
    }

    #[test]
    fn async_chunk_files_call_register_chunk_with_their_own_name() {
        let (_entry_code, chunks, _preload_hints) = split_bundle_fixture();
        let lazy_chunk = chunks
            .iter()
            .find(|c| c.name == "chunk-lazy1")
            .unwrap_or_else(|| panic!("expected a chunk-lazy1 chunk, got: {chunks:#?}"));
        assert!(
            lazy_chunk
                .code
                .contains("__jet__.registerChunk(\"chunk-lazy1\""),
            "chunk file must call registerChunk with its own name: {}",
            lazy_chunk.code
        );
        assert!(
            lazy_chunk.code.contains("exports.lazy = 1;"),
            "chunk file must contain its module body: {}",
            lazy_chunk.code
        );
        assert_eq!(lazy_chunk.module_ids, vec![2]);
    }

    #[test]
    fn single_reference_module_stays_inlined_in_entry_not_split_into_shared() {
        // shared.js is a static dependency of the entry only (not also
        // reachable from the lazy split point in this fixture), so
        // `split_chunks_with_config`'s shared-module detection (referenced
        // by 2+ chunks) must NOT carve it into a separate "shared" chunk.
        let (entry_code, chunks, _preload_hints) = split_bundle_fixture();
        assert!(
            !chunks.iter().any(|c| c.name == "shared"),
            "single-reference module must not produce a shared chunk: {chunks:#?}"
        );
        assert!(
            entry_code.contains("exports.shared = 1;"),
            "entry must inline the only-statically-referenced module: {entry_code}"
        );
    }

    // ── WI #1932: emergent no-dynamic-import fallback ────────────────────

    #[test]
    fn generate_split_bundle_returns_none_when_graph_has_no_dynamic_imports() {
        // Same shape as `split_bundle_fixture()` but entry only statically
        // imports shared.js — no dynamic import boundary anywhere in the
        // graph. `self.splitting = true` (the caller doesn't pre-scan for
        // import() before enabling splitting under default-on), but there
        // is nothing to split, so this must return `None` rather than a
        // one-chunk split bundle.
        let bundler = Bundler::new(BundleOptions {
            splitting: true,
            ..Default::default()
        })
        .expect("bundler new");

        {
            let mut g = bundler.graph.write();
            let entry = g.add_module(
                PathBuf::from("/fixture/entry.js"),
                graph::ModuleKind::Script,
                0,
            );
            let shared = g.add_module(
                PathBuf::from("/fixture/shared.js"),
                graph::ModuleKind::Script,
                0,
            );
            g.add_dependency(entry, shared, EdgeKind::Import);
        }

        let modules = vec![
            compiled(0, "/fixture/entry.js", "__jet__.require(1);"),
            compiled(1, "/fixture/shared.js", "exports.shared = 1;"),
        ];

        let result = bundler
            .generate_split_bundle(&modules)
            .expect("generate_split_bundle should succeed");
        assert!(
            result.is_none(),
            "graph with no dynamic imports must fall back to None, got: {result:#?}"
        );
    }

    // ── #1963: entry bootstrap loads static shared/manual chunks first ──
    //
    // #1948's STOP clause: a `ChunkType::Shared` chunk (auto-detected
    // `shared` OR a `[build.manual_chunks]` chunk) the ENTRY statically
    // depends on used to never load — the bootstrap was a bare synchronous
    // `__jet__.require(entry.id)`, so a real page threw `Uncaught Error:
    // Module not found: <id>` before anything ever called
    // `__jet__.loadChunk` for it. `entry_bootstrap_js` is the fix's unit;
    // `shared_chunk_that_is_a_static_entry_dependency_is_loaded_before_require_end_to_end`
    // below proves `generate_split_bundle` actually threads the filtered
    // static-import list through end to end.

    #[test]
    fn entry_bootstrap_js_no_static_imports_is_byte_identical_bare_require() {
        // The common case (no shared/manual chunk is a static entry
        // dependency — e.g. `split_bundle_fixture()` above, where
        // shared.js is single-reference and the only name in
        // `entry_chunk.imports` is the async "chunk-lazy1") must keep the
        // pre-#1963 bare-require byte shape exactly, so
        // `cli.rs::inject_chunk_manifest`'s anchor rework stays a
        // byte-identical no-op for this path.
        assert_eq!(entry_bootstrap_js(0, &[]), "__jet__.require(0);\n");
        assert_eq!(entry_bootstrap_js(42, &[]), "__jet__.require(42);\n");
    }

    #[test]
    fn entry_bootstrap_js_with_static_imports_loads_chunks_before_require_with_error_path() {
        let js = entry_bootstrap_js(0, &["shared"]);
        // Ordering, not just presence: the load must be scheduled BEFORE
        // the require, matching `cli.rs::inject_chunk_manifest`'s
        // requirement that the manifest (and therefore every loadChunk
        // call depending on it) lands ahead of require.
        let load_pos = js
            .find("Promise.all([\"shared\"].map(__jet__.loadChunk))")
            .unwrap_or_else(|| {
                panic!("must call loadChunk via Promise.all before requiring the entry: {js}")
            });
        let require_pos = js
            .find("__jet__.require(0);")
            .unwrap_or_else(|| panic!("must still require the entry once loaded: {js}"));
        assert!(
            load_pos < require_pos,
            "loadChunk must be scheduled before require(entry): {js}"
        );
        assert!(
            js.contains("}, function(err) {"),
            "must wire an error callback for a rejected chunk load: {js}"
        );
        assert!(
            js.contains("console.error('jet: failed to load startup chunk', err);")
                && js.contains("throw err;"),
            "load failure must be logged and rethrown, not silently swallowed: {js}"
        );
    }

    #[test]
    fn entry_bootstrap_js_multiple_static_imports_preserves_order() {
        let js = entry_bootstrap_js(7, &["shared", "vendor"]);
        assert!(
            js.contains("Promise.all([\"shared\", \"vendor\"].map(__jet__.loadChunk))"),
            "must load every static dependency, in the given order, before requiring: {js}"
        );
    }

    #[test]
    fn shared_chunk_that_is_a_static_entry_dependency_is_loaded_before_require_end_to_end() {
        // entry.js statically imports common.js, and lazy1.js (an async
        // split point) ALSO statically imports common.js, so
        // `splitting::split_chunks`'s 2+-reachability rule promotes
        // common.js into its own "shared" chunk that the ENTRY statically
        // depends on — mirrors `tests/build/code_splitting.rs`'s
        // `write_shared_promotion_fixture` at unit-test scale.
        let bundler = Bundler::new(BundleOptions {
            splitting: true,
            ..Default::default()
        })
        .expect("bundler new");

        {
            let mut g = bundler.graph.write();
            let entry = g.add_module(
                PathBuf::from("/fixture/entry.js"),
                graph::ModuleKind::Script,
                0,
            );
            let common = g.add_module(
                PathBuf::from("/fixture/common.js"),
                graph::ModuleKind::Script,
                0,
            );
            let lazy1 = g.add_module(
                PathBuf::from("/fixture/lazy1.js"),
                graph::ModuleKind::Script,
                0,
            );
            g.add_dependency(entry, common, EdgeKind::Import);
            g.add_dependency(entry, lazy1, EdgeKind::DynamicImport);
            g.add_dependency(lazy1, common, EdgeKind::Import);
        }

        let modules = vec![
            compiled(0, "/fixture/entry.js", "__jet__.dynamicImport(2);"),
            compiled(1, "/fixture/common.js", "exports.common = 1;"),
            compiled(2, "/fixture/lazy1.js", "exports.lazy = 1;"),
        ];

        let (entry_code, chunks, _preload_hints) = bundler
            .generate_split_bundle(&modules)
            .expect("generate_split_bundle should succeed")
            .expect("fixture has a dynamic import boundary; must not fall back to None");

        assert!(
            chunks.iter().any(|c| c.name == "shared"),
            "common.js is reachable from 2 chunks (entry + lazy1); must be promoted to a \
             shared chunk: {chunks:#?}"
        );
        assert!(
            !entry_code.contains("exports.common = 1;"),
            "the promoted shared module must be excluded from the entry chunk: {entry_code}"
        );

        // The actual #1963 assertion: the entry bootstrap must load
        // "shared" via loadChunk before requiring the entry, and must NOT
        // eagerly load "chunk-lazy1" (still an on-demand async chunk).
        let load_pos = entry_code
            .find("Promise.all([\"shared\"].map(__jet__.loadChunk))")
            .unwrap_or_else(|| {
                panic!(
                    "entry bootstrap must loadChunk the static shared dependency before \
                     require(entry): {entry_code}"
                )
            });
        let require_pos = entry_code
            .rfind("__jet__.require(0);")
            .expect("entry must still require itself once loaded");
        assert!(
            load_pos < require_pos,
            "shared chunk must load before require(entry): {entry_code}"
        );
        assert!(
            !entry_code.contains("chunk-lazy1"),
            "async chunk name must not be eagerly loaded in the entry bootstrap: {entry_code}"
        );
    }
}

/// Unit coverage for #1991 (lazy pure-barrel expansion): the pure-barrel
/// detector, per-specifier demand narrowing, demand accumulation, and the
/// wave-parallel crawl's actual skip/expand behavior via
/// `Bundler::build_graph`. The 1,000-leaf/3-used AC1 fixture and the
/// byte-identical lazy-vs-eager output comparison live in
/// `tests/build/lazy_barrel_expansion.rs` (integration-level; they need a
/// real `jet build` subprocess to compare final bundle output).
#[cfg(test)]
mod lazy_barrel_expansion_tests {
    use super::*;

    // ── is_pure_barrel_source: positive cases ───────────────────────────

    #[test]
    fn is_pure_barrel_source_all_named_reexports_is_pure() {
        let src = "export { IconA } from './IconA.js';\nexport { IconB } from './IconB.js';\n";
        assert!(is_pure_barrel_source(src));
    }

    #[test]
    fn is_pure_barrel_source_all_star_reexports_is_pure() {
        let src = "export * from './a.js';\nexport * from './b.js';\n";
        assert!(is_pure_barrel_source(src));
    }

    #[test]
    fn is_pure_barrel_source_mixed_named_and_star_with_trivia_is_pure() {
        let src = "'use strict';\n// a leading comment\n/* a block comment\nspanning lines */\nexport { IconA } from './IconA.js';\n\nexport * from './extra.js';\n";
        assert!(is_pure_barrel_source(src));
    }

    // ── is_pure_barrel_source: negative cases ───────────────────────────

    #[test]
    fn is_pure_barrel_source_one_real_statement_is_not_pure() {
        // The exact "one real statement" negative the issue calls out: an
        // otherwise-barrel-shaped file with a single non-reexport line mixed
        // in must NOT be treated as pure.
        let src = "export { IconA } from './IconA.js';\nconst x = 1;\nexport { IconB } from './IconB.js';\n";
        assert!(!is_pure_barrel_source(src));
    }

    #[test]
    fn is_pure_barrel_source_export_default_is_not_pure() {
        let src = "export { IconA } from './IconA.js';\nexport default IconA;\n";
        assert!(!is_pure_barrel_source(src));
    }

    #[test]
    fn is_pure_barrel_source_plain_import_is_not_pure() {
        let src = "import './side-effect.js';\nexport { IconA } from './IconA.js';\n";
        assert!(!is_pure_barrel_source(src));
    }

    #[test]
    fn is_pure_barrel_source_empty_or_all_trivia_file_is_not_pure() {
        let src = "// nothing here\n'use strict';\n\n";
        assert!(
            !is_pure_barrel_source(src),
            "zero re-export lines must not count as a pure barrel"
        );
    }

    // ── barrel_demand_for_specifier: ESM shapes ─────────────────────────

    #[test]
    fn barrel_demand_for_specifier_esm_named_import_returns_requested_names() {
        let src = "import { IconA, IconB } from '@lib/barrel';\nconsole.log(IconA, IconB);\n";
        let mut names = barrel_demand_for_specifier(src, "@lib/barrel", false)
            .expect("named import must narrow, not fall back to full");
        names.sort();
        assert_eq!(names, vec!["IconA".to_string(), "IconB".to_string()]);
    }

    #[test]
    fn barrel_demand_for_specifier_esm_namespace_import_falls_back_to_full() {
        let src = "import * as icons from '@lib/barrel';\nconsole.log(icons.IconA);\n";
        assert_eq!(
            barrel_demand_for_specifier(src, "@lib/barrel", false),
            None,
            "namespace import must escalate to full expansion"
        );
    }

    #[test]
    fn barrel_demand_for_specifier_esm_default_import_returns_default() {
        let src = "import Default from '@lib/barrel';\nconsole.log(Default);\n";
        assert_eq!(
            barrel_demand_for_specifier(src, "@lib/barrel", false),
            Some(vec!["default".to_string()])
        );
    }

    #[test]
    fn barrel_demand_for_specifier_esm_side_effect_import_returns_empty_names() {
        // `import '@lib/barrel';` binds no name at all — unlike the CJS bare
        // `require('pkg');` fallback (whose return value could be used in an
        // unpredictable way not visible to line scanning), ESM side-effect
        // imports are syntactically guaranteed to read nothing, so narrowing
        // to zero demanded names (not falling back to full) is correct.
        let src = "import '@lib/barrel';\n";
        assert_eq!(
            barrel_demand_for_specifier(src, "@lib/barrel", false),
            Some(vec![])
        );
    }

    #[test]
    fn barrel_demand_for_specifier_dynamic_import_falls_back_to_full() {
        // A source that would otherwise narrow cleanly still escalates once
        // the caller marks the edge dynamic.
        let src = "import { IconA } from '@lib/barrel';\n";
        assert_eq!(
            barrel_demand_for_specifier(src, "@lib/barrel", true),
            None,
            "dynamic import() of a barrel must always fall back to full"
        );
    }

    // ── barrel_demand_for_specifier: multi-line ESM shapes (#1991 round 2) ──

    #[test]
    fn barrel_demand_for_specifier_esm_multiline_named_import_returns_requested_names() {
        // The real-corpus shape the round-2 evidence comment calls out:
        // prettier's default wrapping for 3+ named imports puts the
        // binding list and `from` clause on different physical lines.
        let src =
            "import {\n  IconA,\n  IconB,\n} from '@lib/barrel';\nconsole.log(IconA, IconB);\n";
        let mut names = barrel_demand_for_specifier(src, "@lib/barrel", false)
            .expect("multi-line named import must narrow, not fall back to full");
        names.sort();
        assert_eq!(names, vec!["IconA".to_string(), "IconB".to_string()]);
    }

    #[test]
    fn barrel_demand_for_specifier_esm_multiline_trailing_comma_returns_requested_names() {
        let src = "import {\n  IconA,\n  IconB,\n} from '@lib/barrel';\n";
        let mut names = barrel_demand_for_specifier(src, "@lib/barrel", false)
            .expect("trailing comma before the closing brace must still narrow");
        names.sort();
        assert_eq!(names, vec!["IconA".to_string(), "IconB".to_string()]);
    }

    #[test]
    fn barrel_demand_for_specifier_esm_multiline_inline_comment_does_not_corrupt_names() {
        let src = "import {\n  IconA, // used on the dashboard\n  IconB,\n} from '@lib/barrel';\n";
        let mut names = barrel_demand_for_specifier(src, "@lib/barrel", false)
            .expect("an inline comment inside the brace list must still narrow");
        names.sort();
        assert_eq!(names, vec!["IconA".to_string(), "IconB".to_string()]);
    }

    #[test]
    fn barrel_demand_for_specifier_esm_multiline_mixed_default_and_named_returns_both() {
        let src = "import Default, {\n  IconA,\n  IconB,\n} from '@lib/barrel';\n";
        let mut names = barrel_demand_for_specifier(src, "@lib/barrel", false)
            .expect("mixed default+named across lines must still narrow");
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
    fn barrel_demand_for_specifier_esm_single_line_regression_unchanged() {
        // #1991 round 2 regression guard: a single-line import — the common
        // case, and every pre-round-2 test's shape — must keep resolving
        // through the exact same path.
        let src = "import { IconA, IconB } from '@lib/barrel';\nconsole.log(IconA, IconB);\n";
        let mut names = barrel_demand_for_specifier(src, "@lib/barrel", false)
            .expect("single-line named import must narrow");
        names.sort();
        assert_eq!(names, vec!["IconA".to_string(), "IconB".to_string()]);
    }

    // ── barrel_demand_for_specifier: CJS shapes ─────────────────────────

    #[test]
    fn barrel_demand_for_specifier_cjs_destructured_returns_names() {
        let src = "const { a, b } = require('@lib/barrel');\n";
        let mut names = barrel_demand_for_specifier(src, "@lib/barrel", false)
            .expect("destructured require must narrow");
        names.sort();
        assert_eq!(names, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn barrel_demand_for_specifier_cjs_direct_property_access_returns_names() {
        let src = "const a = require('@lib/barrel').IconA;\n";
        assert_eq!(
            barrel_demand_for_specifier(src, "@lib/barrel", false),
            Some(vec!["IconA".to_string()])
        );
    }

    #[test]
    fn barrel_demand_for_specifier_cjs_bound_then_later_property_access_returns_names() {
        let src = "const m = require('@lib/barrel');\nconsole.log(m.IconB);\n";
        assert_eq!(
            barrel_demand_for_specifier(src, "@lib/barrel", false),
            Some(vec!["IconB".to_string()])
        );
    }

    #[test]
    fn barrel_demand_for_specifier_cjs_bare_require_falls_back_to_full() {
        let src = "require('@lib/barrel');\n";
        assert_eq!(
            barrel_demand_for_specifier(src, "@lib/barrel", false),
            None,
            "a require() return value that is never bound or accessed must fall back to full"
        );
    }

    #[test]
    fn barrel_demand_for_specifier_cjs_bound_but_never_used_falls_back_to_full() {
        let src = "const m = require('@lib/barrel');\n";
        assert_eq!(
            barrel_demand_for_specifier(src, "@lib/barrel", false),
            None,
            "a bound-but-unread require() target must fall back to full, not zero names"
        );
    }

    // ── barrel_demand_for_specifier: defensive fallback ──────────────────

    #[test]
    fn barrel_demand_for_specifier_unmatched_specifier_returns_none() {
        let src = "import { Other } from '@lib/other';\n";
        assert_eq!(
            barrel_demand_for_specifier(src, "@lib/barrel", false),
            None,
            "a specifier this text scan never actually saw referenced must not claim zero demand"
        );
    }

    // ── barrel_demand_for_specifier_with_reason: EscalationReason (#1991 round 2) ──

    #[test]
    fn barrel_demand_for_specifier_with_reason_namespace_import_is_namespace_import() {
        let src = "import * as icons from '@lib/barrel';\nconsole.log(icons.IconA);\n";
        assert_eq!(
            barrel_demand_for_specifier_with_reason(src, "@lib/barrel", false),
            Err(EscalationReason::NamespaceImport)
        );
    }

    #[test]
    fn barrel_demand_for_specifier_with_reason_dynamic_import_is_dynamic_import() {
        let src = "import { IconA } from '@lib/barrel';\n";
        assert_eq!(
            barrel_demand_for_specifier_with_reason(src, "@lib/barrel", true),
            Err(EscalationReason::DynamicImport)
        );
    }

    #[test]
    fn barrel_demand_for_specifier_with_reason_bare_cjs_use_is_bare_cjs_use() {
        let src = "require('@lib/barrel');\n";
        assert_eq!(
            barrel_demand_for_specifier_with_reason(src, "@lib/barrel", false),
            Err(EscalationReason::BareCjsUse)
        );
    }

    #[test]
    fn barrel_demand_for_specifier_with_reason_export_star_chain_for_star_reexport_line() {
        let src = "export * from '@lib/barrel';\n";
        assert_eq!(
            barrel_demand_for_specifier_with_reason(src, "@lib/barrel", false),
            Err(EscalationReason::ExportStarChain)
        );
    }

    #[test]
    fn barrel_demand_for_specifier_with_reason_export_star_chain_for_named_reexport_line() {
        // A named re-export-from line is bucketed under the same
        // export-star-chain reason as a star re-export — both chain the
        // barrel into another module rather than a leaf consumer using it
        // directly, and #1991's enum vocabulary has one bucket for both.
        let src = "export { IconA } from '@lib/barrel';\n";
        assert_eq!(
            barrel_demand_for_specifier_with_reason(src, "@lib/barrel", false),
            Err(EscalationReason::ExportStarChain)
        );
    }

    #[test]
    fn barrel_demand_for_specifier_with_reason_unmatched_specifier_is_no_demand_recorded() {
        let src = "import { Other } from '@lib/other';\n";
        assert_eq!(
            barrel_demand_for_specifier_with_reason(src, "@lib/barrel", false),
            Err(EscalationReason::NoDemandRecorded)
        );
    }

    #[test]
    fn barrel_demand_for_specifier_with_reason_truncated_multiline_import_is_unparseable_import() {
        let src = "import {\n  IconA,\n  IconB,\n";
        assert_eq!(
            barrel_demand_for_specifier_with_reason(src, "@lib/barrel", false),
            Err(EscalationReason::UnparseableImport)
        );
    }

    // ── merge_barrel_demand / BarrelDemand ───────────────────────────────

    #[test]
    fn merge_barrel_demand_extends_names_across_calls() {
        let mut map: HashMap<PathBuf, BarrelDemand> = HashMap::new();
        let target = PathBuf::from("/fixture/icons/index.js");
        merge_barrel_demand(&mut map, &target, Some(vec!["IconA".to_string()]));
        merge_barrel_demand(&mut map, &target, Some(vec!["IconB".to_string()]));
        match map.get(&target) {
            Some(BarrelDemand::Names(names)) => {
                assert_eq!(names.len(), 2);
                assert!(names.contains("IconA") && names.contains("IconB"));
            }
            other => panic!("expected accumulated Names, got {other:?}"),
        }
    }

    #[test]
    fn merge_barrel_demand_none_escalates_to_full() {
        let mut map: HashMap<PathBuf, BarrelDemand> = HashMap::new();
        let target = PathBuf::from("/fixture/icons/index.js");
        merge_barrel_demand(&mut map, &target, Some(vec!["IconA".to_string()]));
        merge_barrel_demand(&mut map, &target, None);
        assert!(
            matches!(map.get(&target), Some(BarrelDemand::Full)),
            "None must escalate an existing Names entry to Full"
        );
    }

    #[test]
    fn merge_barrel_demand_full_is_sticky_against_further_names() {
        let mut map: HashMap<PathBuf, BarrelDemand> = HashMap::new();
        let target = PathBuf::from("/fixture/icons/index.js");
        merge_barrel_demand(&mut map, &target, None);
        merge_barrel_demand(&mut map, &target, Some(vec!["IconA".to_string()]));
        assert!(
            matches!(map.get(&target), Some(BarrelDemand::Full)),
            "Full must stay Full even after a later Names merge"
        );
    }

    // ── top_barrel_escalations (#1991 round 2) ───────────────────────────

    fn fixture_barrel_module(leaf_count: usize) -> PrefetchedModule {
        let src: String = (0..leaf_count)
            .map(|n| format!("export {{ N{n} }} from './n{n}.js';\n"))
            .collect();
        PrefetchedModule {
            source: Ok(src),
            imports: Err("not a script module".to_string()),
            resolutions: HashMap::new(),
            tree: None,
            used_fast_import_scan: None,
        }
    }

    #[test]
    fn top_barrel_escalations_sorts_by_leaf_count_desc_and_truncates_to_five() {
        let mut barrels_detected: HashSet<PathBuf> = HashSet::new();
        let mut escalation_reasons: HashMap<PathBuf, EscalationReason> = HashMap::new();
        let mut prefetched: HashMap<PathBuf, PrefetchedModule> = HashMap::new();

        let reasons = [
            EscalationReason::NamespaceImport,
            EscalationReason::BareCjsUse,
            EscalationReason::DynamicImport,
            EscalationReason::ExportStarChain,
            EscalationReason::UnresolvableName,
            EscalationReason::UnparseableImport,
            EscalationReason::NoDemandRecorded,
        ];
        for (i, reason) in reasons.iter().enumerate() {
            let path = PathBuf::from(format!("/fixture/barrel{i}.js"));
            // barrel0 has 1 leaf, barrel6 has 7 leaves — descending sort
            // must put the biggest barrel first.
            let leaf_count = i + 1;
            barrels_detected.insert(path.clone());
            escalation_reasons.insert(path.clone(), *reason);
            prefetched.insert(path, fixture_barrel_module(leaf_count));
        }

        let top = top_barrel_escalations(&barrels_detected, &escalation_reasons, &prefetched);

        assert_eq!(top.len(), 5, "must truncate to the top 5");
        let leaf_counts: Vec<usize> = top.iter().map(|(_, _, count)| *count).collect();
        assert_eq!(
            leaf_counts,
            vec![7, 6, 5, 4, 3],
            "must sort by leaf count descending, largest barrel first"
        );
        assert_eq!(top[0].1, EscalationReason::NoDemandRecorded);
        assert_eq!(top[4].1, EscalationReason::DynamicImport);
    }

    #[test]
    fn top_barrel_escalations_excludes_non_escalated_barrels() {
        let mut barrels_detected: HashSet<PathBuf> = HashSet::new();
        let escalation_reasons: HashMap<PathBuf, EscalationReason> = HashMap::new();
        let mut prefetched: HashMap<PathBuf, PrefetchedModule> = HashMap::new();

        let path = PathBuf::from("/fixture/clean_barrel.js");
        barrels_detected.insert(path.clone());
        prefetched.insert(path, fixture_barrel_module(1));

        let top = top_barrel_escalations(&barrels_detected, &escalation_reasons, &prefetched);
        assert!(
            top.is_empty(),
            "a barrel with no recorded escalation reason must not appear in the report"
        );
    }

    // ── crawl-level: Bundler::build_graph over real fixtures ─────────────

    fn write_fixture(tmp: &std::path::Path, rel: &str, contents: &str) {
        let path = tmp.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create fixture dir");
        }
        std::fs::write(&path, contents).expect("write fixture file");
    }

    async fn crawl(tmp: &std::path::Path, entry_rel: &str) -> Bundler {
        let entry = tmp.join(entry_rel);
        let bundler = Bundler::new(BundleOptions {
            entry: entry.clone(),
            output_dir: tmp.join("dist"),
            ..Default::default()
        })
        .expect("Bundler::new");
        bundler
            .build_graph(&entry)
            .await
            .expect("build_graph must succeed");
        bundler
    }

    fn crawled(bundler: &Bundler, tmp: &std::path::Path, rel: &str) -> bool {
        let path = std::fs::canonicalize(tmp.join(rel)).expect("fixture file must exist on disk");
        bundler.graph.read().get_module(&path).is_some()
    }

    #[tokio::test]
    async fn build_graph_lazy_barrel_skips_unrequested_leaves() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        write_fixture(
            dir,
            "entry.js",
            "import { IconB } from './icons/index.js';\nconsole.log(IconB);\n",
        );
        write_fixture(
            dir,
            "icons/index.js",
            "export { IconA } from './IconA.js';\nexport { IconB } from './IconB.js';\nexport { IconC } from './IconC.js';\n",
        );
        write_fixture(dir, "icons/IconA.js", "export const IconA = 'A';\n");
        write_fixture(dir, "icons/IconB.js", "export const IconB = 'B';\n");
        write_fixture(dir, "icons/IconC.js", "export const IconC = 'C';\n");

        let bundler = crawl(dir, "entry.js").await;

        assert!(
            crawled(&bundler, dir, "icons/index.js"),
            "the barrel itself must always be crawled"
        );
        assert!(
            crawled(&bundler, dir, "icons/IconB.js"),
            "IconB is demanded by name and must be crawled"
        );
        assert!(
            !crawled(&bundler, dir, "icons/IconA.js"),
            "IconA is never demanded and must be skipped"
        );
        assert!(
            !crawled(&bundler, dir, "icons/IconC.js"),
            "IconC is never demanded and must be skipped"
        );
    }

    #[tokio::test]
    async fn build_graph_lazy_barrel_same_wave_multi_importer_union_demand() {
        // entry.js imports the barrel directly AND imports mid1.js in the
        // same statement list, so the barrel and mid1.js land in the SAME
        // wave-parallel BFS wave together — mid1.js's demand must still be
        // recorded onto the barrel before that wave's expansion runs.
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        write_fixture(
            dir,
            "entry.js",
            "import { IconDirect } from './icons/index.js';\nimport { A } from './mid1.js';\nconsole.log(IconDirect, A());\n",
        );
        write_fixture(
            dir,
            "mid1.js",
            "import { IconA } from './icons/index.js';\nexport function A() { return IconA; }\n",
        );
        write_fixture(
            dir,
            "icons/index.js",
            "export { IconDirect } from './IconDirect.js';\nexport { IconA } from './IconA.js';\nexport { IconUnused } from './IconUnused.js';\n",
        );
        write_fixture(
            dir,
            "icons/IconDirect.js",
            "export const IconDirect = 'D';\n",
        );
        write_fixture(dir, "icons/IconA.js", "export const IconA = 'A';\n");
        write_fixture(
            dir,
            "icons/IconUnused.js",
            "export const IconUnused = 'U';\n",
        );

        let bundler = crawl(dir, "entry.js").await;

        assert!(crawled(&bundler, dir, "icons/IconDirect.js"));
        assert!(crawled(&bundler, dir, "icons/IconA.js"));
        assert!(
            !crawled(&bundler, dir, "icons/IconUnused.js"),
            "same-wave multi-importer union must not over-include an undemanded leaf"
        );
    }

    #[tokio::test]
    async fn build_graph_lazy_barrel_incremental_expansion_across_waves() {
        // mid1.js demands IconA at wave depth 2 (the barrel's first
        // expansion); a second, much deeper importer chain
        // (mid2 -> mid3 -> mid4) demands IconB only after the barrel has
        // already been partially expanded once. The later demand must still
        // incrementally expand IconB without re-pushing IconA and without
        // pulling in IconUnused.
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        write_fixture(
            dir,
            "entry.js",
            "import { A } from './mid1.js';\nimport { B } from './deep/d2/mid2.js';\nconsole.log(A(), B());\n",
        );
        write_fixture(
            dir,
            "mid1.js",
            "import { IconA } from './icons/index.js';\nexport function A() { return IconA; }\n",
        );
        write_fixture(
            dir,
            "deep/d2/mid2.js",
            "import { fromMid3 } from '../d3/mid3.js';\nexport function B() { return fromMid3(); }\n",
        );
        write_fixture(
            dir,
            "deep/d3/mid3.js",
            "import { fromMid4 } from '../d4/mid4.js';\nexport function fromMid3() { return fromMid4(); }\n",
        );
        write_fixture(
            dir,
            "deep/d4/mid4.js",
            "import { IconB } from '../../icons/index.js';\nexport function fromMid4() { return IconB; }\n",
        );
        write_fixture(
            dir,
            "icons/index.js",
            "export { IconA } from './IconA.js';\nexport { IconB } from './IconB.js';\nexport { IconUnused } from './IconUnused.js';\n",
        );
        write_fixture(dir, "icons/IconA.js", "export const IconA = 'A';\n");
        write_fixture(dir, "icons/IconB.js", "export const IconB = 'B';\n");
        write_fixture(
            dir,
            "icons/IconUnused.js",
            "export const IconUnused = 'U';\n",
        );

        let bundler = crawl(dir, "entry.js").await;

        assert!(
            crawled(&bundler, dir, "icons/IconA.js"),
            "first-wave demand must expand"
        );
        assert!(
            crawled(&bundler, dir, "icons/IconB.js"),
            "later-wave demand must incrementally expand the already-visited barrel"
        );
        assert!(!crawled(&bundler, dir, "icons/IconUnused.js"));
    }

    #[tokio::test]
    async fn build_graph_lazy_barrel_namespace_import_fallback_expands_full() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        write_fixture(
            dir,
            "entry.js",
            "import * as icons from './icons/index.js';\nconsole.log(icons.IconA, icons.IconB, icons.IconC);\n",
        );
        write_fixture(
            dir,
            "icons/index.js",
            "export { IconA } from './IconA.js';\nexport { IconB } from './IconB.js';\nexport { IconC } from './IconC.js';\n",
        );
        write_fixture(dir, "icons/IconA.js", "export const IconA = 'A';\n");
        write_fixture(dir, "icons/IconB.js", "export const IconB = 'B';\n");
        write_fixture(dir, "icons/IconC.js", "export const IconC = 'C';\n");

        let bundler = crawl(dir, "entry.js").await;

        assert!(crawled(&bundler, dir, "icons/IconA.js"));
        assert!(crawled(&bundler, dir, "icons/IconB.js"));
        assert!(
            crawled(&bundler, dir, "icons/IconC.js"),
            "namespace import must force full expansion of every leaf"
        );
    }

    #[tokio::test]
    async fn build_graph_lazy_barrel_star_reexport_inside_barrel_always_expands() {
        // The barrel's star entry (-> extra.js) must always expand, even
        // though nothing ever demands a name through it; its sibling named
        // entry (IconUnused) must stay excluded since nothing demands it.
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        write_fixture(
            dir,
            "entry.js",
            "import { IconA } from './icons/index.js';\nconsole.log(IconA);\n",
        );
        write_fixture(
            dir,
            "icons/index.js",
            "export { IconA } from './IconA.js';\nexport { IconUnused } from './IconUnused.js';\nexport * from './extra.js';\n",
        );
        write_fixture(dir, "icons/IconA.js", "export const IconA = 'A';\n");
        write_fixture(
            dir,
            "icons/IconUnused.js",
            "export const IconUnused = 'U';\n",
        );
        write_fixture(
            dir,
            "icons/extra.js",
            "export const EXTRA_MARKER = 'EXTRA';\n",
        );

        let bundler = crawl(dir, "entry.js").await;

        assert!(
            crawled(&bundler, dir, "icons/IconA.js"),
            "demanded named entry"
        );
        assert!(
            crawled(&bundler, dir, "icons/extra.js"),
            "a star re-export inside a pure barrel must always expand (v1 unconditional rule)"
        );
        assert!(
            !crawled(&bundler, dir, "icons/IconUnused.js"),
            "an undemanded named entry must stay excluded even though a sibling star entry expanded"
        );
    }

    #[tokio::test]
    async fn build_graph_lazy_barrel_export_star_from_another_module_expands_full() {
        // other.js is itself a (single-entry, star) pure barrel over the
        // original icons barrel — `export * from` a barrel into another
        // module must fully expand the target regardless of which specific
        // names other.js's own importers request.
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        write_fixture(
            dir,
            "entry.js",
            "import { IconA, IconB } from './other.js';\nconsole.log(IconA, IconB);\n",
        );
        write_fixture(dir, "other.js", "export * from './icons/index.js';\n");
        write_fixture(
            dir,
            "icons/index.js",
            "export { IconA } from './IconA.js';\nexport { IconB } from './IconB.js';\nexport { IconC } from './IconC.js';\n",
        );
        write_fixture(dir, "icons/IconA.js", "export const IconA = 'A';\n");
        write_fixture(dir, "icons/IconB.js", "export const IconB = 'B';\n");
        write_fixture(dir, "icons/IconC.js", "export const IconC = 'C';\n");

        let bundler = crawl(dir, "entry.js").await;

        assert!(crawled(&bundler, dir, "icons/IconA.js"));
        assert!(crawled(&bundler, dir, "icons/IconB.js"));
        assert!(
            crawled(&bundler, dir, "icons/IconC.js"),
            "export * from the barrel into another module must expand it fully, including a leaf (IconC) nobody ever names"
        );
    }

    #[tokio::test]
    async fn build_graph_lazy_barrel_unresolvable_name_escalates_to_full() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        write_fixture(
            dir,
            "entry.js",
            "import { IconA, ThisNameDoesNotExist } from './icons/index.js';\nconsole.log(IconA, ThisNameDoesNotExist);\n",
        );
        write_fixture(
            dir,
            "icons/index.js",
            "export { IconA } from './IconA.js';\nexport { IconB } from './IconB.js';\n",
        );
        write_fixture(dir, "icons/IconA.js", "export const IconA = 'A';\n");
        write_fixture(dir, "icons/IconB.js", "export const IconB = 'B';\n");

        // Must not error — a requested name that doesn't match any of the
        // barrel's own known export names is a resolver-failure-shaped
        // fallback, not a crawl failure.
        let bundler = crawl(dir, "entry.js").await;

        assert!(crawled(&bundler, dir, "icons/IconA.js"));
        assert!(
            crawled(&bundler, dir, "icons/IconB.js"),
            "an unresolvable requested name must escalate the whole barrel to full, not silently drop the other leaves"
        );
    }

    #[tokio::test]
    async fn build_graph_lazy_barrel_cjs_property_accesses_spanning_many_later_lines() {
        // #1991 round 2: the CJS narrowing path
        // (`scan_require_binding_property_accesses`) already scans the
        // WHOLE module source for later-line property accesses regardless
        // of how far they sit from the `require()` call — round 1 only
        // unit-covered the underlying tree_shake helpers, so this proves
        // it end to end at the graph level, matching the round-2 evidence
        // comment's CJS shim shape: bound once, then accessed many lines
        // later.
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        write_fixture(
            dir,
            "entry.js",
            "const icons = require('./icons/index.js');\n\
             function noop1() {}\nfunction noop2() {}\nfunction noop3() {}\n\
             function noop4() {}\nfunction noop5() {}\nfunction noop6() {}\n\
             function noop7() {}\nfunction noop8() {}\nfunction noop9() {}\n\
             console.log(icons.IconA);\nconsole.log(icons.IconB);\n",
        );
        write_fixture(
            dir,
            "icons/index.js",
            "export { IconA } from './IconA.js';\nexport { IconB } from './IconB.js';\nexport { IconC } from './IconC.js';\n",
        );
        write_fixture(dir, "icons/IconA.js", "export const IconA = 'A';\n");
        write_fixture(dir, "icons/IconB.js", "export const IconB = 'B';\n");
        write_fixture(dir, "icons/IconC.js", "export const IconC = 'C';\n");

        let bundler = crawl(dir, "entry.js").await;

        assert!(
            crawled(&bundler, dir, "icons/IconA.js"),
            "property access many lines after the require() binding must still narrow demand"
        );
        assert!(
            crawled(&bundler, dir, "icons/IconB.js"),
            "property access many lines after the require() binding must still narrow demand"
        );
        assert!(
            !crawled(&bundler, dir, "icons/IconC.js"),
            "an undemanded leaf must stay excluded even with many intervening lines"
        );
    }
}
// CODEGEN-END
