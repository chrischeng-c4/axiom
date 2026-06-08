// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub use super::graph::ModuleId;

/// Bundle configuration options (dev server / basic build)
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone)]
pub struct BundleOptions {
    /// Entry point
    pub entry: PathBuf,

    /// Output directory
    pub output_dir: PathBuf,

    /// Enable source maps
    pub source_maps: bool,

    /// Enable minification
    pub minify: bool,

    /// Module resolution options
    pub resolve_options: crate::resolver::ResolveOptions,

    /// Transform options
    pub transform_options: crate::transform::TransformOptions,

    /// Asset processing options
    pub asset_options: crate::asset::AssetOptions,

    /// Packages to mark as external (not bundled)
    pub externals: HashSet<String>,

    /// When true, treat all bare package specifiers as external.
    /// Used for lib builds where node_modules deps should not be bundled.
    pub externalize_all_packages: bool,

    /// Compile-time define map applied after code transformation.
    ///
    /// Map from expression string to replacement value, e.g.
    /// `"import.meta.env.MODE"` → `"\"development\""`.
    ///
    /// Use `crate::bundler::define::build_import_meta_env_defines` to build
    /// this map from scanned `.env` files.
    pub defines: HashMap<String, String>,

    /// Emit imported CSS as linked assets instead of runtime style injection.
    ///
    /// Production DOM builds set this so CSS is not duplicated in both the JS
    /// bundle and a sidecar stylesheet.
    pub css_bundle: bool,
}

/// Production build configuration.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Entry point(s).
    pub entry: Vec<PathBuf>,
    /// Output directory.
    pub out_dir: PathBuf,
    /// Enable minification.
    pub minify: bool,
    /// Source map mode.
    pub sourcemap: SourceMapOption,
    /// Compile-time constant replacement.
    pub define: HashMap<String, String>,
    /// Packages to exclude from bundle.
    pub external: HashSet<String>,
    /// Enable code splitting.
    pub splitting: bool,
    /// Output format.
    pub format: OutputFormat,
    /// Enable CSS bundling.
    pub css_bundle: bool,
    /// Statements to drop (console, debugger).
    pub drop: Vec<super::minify::DropStatement>,
    /// Manual chunk definitions: chunk name → glob patterns.
    ///
    /// Modules matching any glob pattern are routed to the named chunk
    /// during code splitting. Manual chunks take priority over automatic
    /// shared chunk extraction.
    ///
    /// Example: `{ "vendor": ["node_modules/react/**", "node_modules/react-dom/**"] }`
    pub manual_chunks: HashMap<String, Vec<String>>,
}

/// Preload hint metadata for code-split chunks.
///
/// Used to generate `<link rel="modulepreload">` tags in HTML output.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone)]
pub struct PreloadHint {
    /// Chunk filename (e.g. "vendor.abc123.js").
    pub href: String,
    /// Whether this is a static dependency (true) or dynamic import (false).
    /// Only static dependencies should be preloaded.
    pub is_static: bool,
}

/// Source map output option.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceMapOption {
    None,
    External,
    Inline,
    Hidden,
}

/// Output module format.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputFormat {
    Esm,
    Cjs,
    Iife,
}

/// Result of a production build.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone)]
pub struct BuildResult {
    /// Output chunks.
    pub chunks: Vec<OutputChunk>,
    /// Build duration in milliseconds.
    pub duration_ms: u64,
    /// Build warnings.
    pub warnings: Vec<String>,
}

/// A single output chunk.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone)]
pub struct OutputChunk {
    /// Output file name (e.g. "main.abc123.js").
    pub name: String,
    /// Chunk type.
    pub chunk_type: OutputChunkType,
    /// Size in bytes.
    pub size: usize,
    /// Source modules included in this chunk.
    pub modules: Vec<String>,
    /// Other chunks this chunk imports.
    pub imports: Vec<String>,
}

/// Output chunk type.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputChunkType {
    Entry,
    Chunk,
    Asset,
}

/// Bundle output
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone)]
pub struct BundleOutput {
    /// Bundled JavaScript code
    pub code: String,

    /// Source map (if enabled)
    pub source_map: Option<String>,

    /// Generated assets
    pub assets: Vec<Asset>,
}

/// Asset output
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone)]
pub struct Asset {
    /// Asset file name
    pub filename: String,

    /// Asset content
    pub content: Vec<u8>,

    /// Asset type
    pub asset_type: AssetType,
}

/// Type of asset
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    Css,
    Image,
    Font,
    Other,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
impl Default for BundleOptions {
    fn default() -> Self {
        Self {
            entry: PathBuf::from("src/index.js"),
            output_dir: PathBuf::from("dist"),
            source_maps: true,
            minify: false,
            resolve_options: Default::default(),
            transform_options: Default::default(),
            asset_options: Default::default(),
            externals: HashSet::new(),
            externalize_all_packages: false,
            defines: HashMap::new(),
            css_bundle: false,
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            entry: vec![PathBuf::from("src/index.ts")],
            out_dir: PathBuf::from("dist"),
            minify: true,
            sourcemap: SourceMapOption::External,
            define: super::define::production_defines(),
            external: HashSet::new(),
            splitting: false,
            format: OutputFormat::Esm,
            css_bundle: true,
            drop: Vec::new(),
            manual_chunks: HashMap::new(),
        }
    }
}
// CODEGEN-END
