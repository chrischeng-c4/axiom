// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
// CODEGEN-BEGIN
//! `[wasm]` section of `jet.toml`.
//!
//! @spec `.aw/tech-design/projects/jet/config/jet-config-validation.md`
//! @issue #1233 — Slice 2 (typed ConfigError + deny_unknown_fields +
//!     did-you-mean Levenshtein). Slice 3 adds the `DEPRECATED_KEYS`
//!     allowlist + `tracing::warn!` migration hints anchored on
//!     `dev-port` → `dev.port`.

use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WasmConfig {
    /// Path to the TSX entry file, relative to the project root.
    pub entry: String,
    /// Root component name (PascalCase), must match an exported
    /// function component in `entry`.
    pub root_component: String,
    /// Renderer backend used by the generated browser entrypoint.
    #[serde(default)]
    pub renderer: WasmRenderer,
    /// Props passed to the root component factory. Order matches the
    /// TS interface field order.
    #[serde(default)]
    pub root_props: Vec<RootPropValue>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum WasmRenderer {
    Dom,
    WebGpu,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
impl Default for WasmRenderer {
    fn default() -> Self {
        Self::WebGpu
    }
}

/// The set of keys `[wasm]` accepts. Source of truth for the
/// did-you-mean suggestion when a typo lands. Keep in lockstep with
/// the struct fields above. (Slice 5 will derive this from the
/// `schemars::JsonSchema` instead of hand-listing it.)
const WASM_SECTION_KEYS: &[&str] = &["entry", "root_component", "renderer", "root_props"];

/// Top-level keys we accept in `jet.toml`. The WASM loader
/// only *consumes* `[wasm]`, but it also tolerates every section the
/// non-WASM `JetConfig` (`task_runner::config`) recognises so that a
/// single `jet.toml` can drive both the WASM and the non-WASM
/// pipelines (issue #1403 — Cue dogfood). Sections other than `wasm`
/// are stored as raw `toml::Value` on `ConfigFile`; the WASM build
/// neither re-validates them nor depends on the `JetConfig` schema.
///
/// Keep in lockstep with `task_runner::config::JET_TOP_LEVEL_KEYS`.
const TOP_LEVEL_KEYS: &[&str] = &[
    "wasm", "pipeline", "dev", "alias", "build", "resolve", "test",
];

/// Shared config sections accepted by the WASM loader as raw TOML so
/// one `jet.toml` can serve both build modes. A legacy
/// `renderer` knob inside these sections is different: it is stale
/// renderer selection state and must fail loudly instead of being
/// accepted-and-discarded.
const LEGACY_RENDERER_SHARED_SECTIONS: &[&str] = &["build", "dev", "resolve", "test"];

/// One entry in the deprecated-key allowlist. `deprecated` is the
/// source-of-truth path the user wrote (single-segment for now);
/// `replacement` is the dotted path the loader rewrites it into;
/// `removal_version` is the semver the rename is scheduled to be
/// hard-removed in. The migration warning includes all three.
struct DeprecatedKeyEntry {
    deprecated: &'static str,
    replacement: &'static str,
    removal_version: &'static str,
}

/// Static allowlist of deprecated keys we silently rewrite into the
/// current schema. The loader emits a `tracing::warn!` per hit; the
/// future `jet config lint` subcommand promotes these to errors when
/// invoked with `--strict-warn`.
///
/// **Anchor.** `dev-port` → `dev.port` is the closed-bug precedent
/// (`bug-jet-dev-ignores-jet-config-yaml-dev-port-and-dev-p`) that
/// motivated the validation spec. The current schema has no `[dev]`
/// section yet, so a config that hits this entry sees BOTH the
/// migration warning AND a downstream `UnknownKey` error on `dev` —
/// that's the spec-compliant decision-tree behavior (see
/// `jet-config-validation.md` §"Decision tree").
const DEPRECATED_KEYS: &[DeprecatedKeyEntry] = &[DeprecatedKeyEntry {
    deprecated: "dev-port",
    replacement: "dev.port",
    removal_version: "0.4.0",
}];

/// Migration hint emitted when the loader rewrites a deprecated key
/// into its current-schema replacement. Surfaced via `tracing::warn!`
/// from `parse_str` and returned structurally from
/// `parse_str_with_warnings`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
#[derive(Debug, Clone)]
pub struct DeprecatedKeyWarning {
    pub path: PathBuf,
    pub key: String,
    pub replacement: String,
    pub removal_version: String,
    pub span: ConfigSpan,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
impl std::fmt::Display for DeprecatedKeyWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let where_ = if self.span.line > 0 {
            format!(" at line {}", self.span.line)
        } else {
            String::new()
        };
        write!(
            f,
            "deprecated key {:?} in {}{} — replaced by {:?} (will be removed in {})",
            self.key,
            self.path.display(),
            where_,
            self.replacement,
            self.removal_version,
        )
    }
}

/// Prop value — numbers, booleans, strings, or homogeneous arrays.
/// The generated factory takes `i64` / `bool` / `String` / `Vec<T>`,
/// matching the prop interface's Rust lowering. Display impl emits
/// valid Rust source that can be interpolated directly into the
/// WASM-entry factory call.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum RootPropValue {
    Int(i64),
    Bool(bool),
    Str(String),
    /// Homogeneous array — serializes as `vec![elem, ...]`. Inner
    /// elements reuse the `RootPropValue::Display` contract, so
    /// nested arrays also work (though the transpiler only lowers
    /// one level today).
    Array(Vec<RootPropValue>),
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
impl std::fmt::Display for RootPropValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RootPropValue::Int(n) => write!(f, "{n}"),
            RootPropValue::Bool(b) => write!(f, "{b}"),
            RootPropValue::Str(s) => write!(f, "{:?}.to_string()", s),
            RootPropValue::Array(items) => {
                write!(f, "vec![")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
        }
    }
}

/// In-memory shape of `jet.toml` as seen by the WASM loader.
/// `wasm` is the only field consumed downstream; the remaining
/// sections mirror the non-WASM `JetConfig` schema (`pipeline`,
/// `dev`, `alias`, `build`, `resolve`, `test`) and are accepted as
/// raw `toml::Value` so projects can keep a single config file for
/// both pipelines.
///
/// @spec projects/jet/docs/wasm-config-accept-shared-jet-sections.md#interface
/// @issue #1403
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    wasm: Option<WasmConfig>,
    // The remaining fields are accept-and-discard — they exist
    // purely so `deny_unknown_fields` doesn't reject configs that
    // also drive the non-WASM `JetConfig` pipeline. The WASM build
    // never reads them; the non-WASM loader owns their schemas.
    #[allow(dead_code)]
    #[serde(default)]
    pipeline: Option<toml::Value>,
    #[allow(dead_code)]
    #[serde(default)]
    dev: Option<toml::Value>,
    #[allow(dead_code)]
    #[serde(default)]
    alias: Option<toml::Value>,
    #[allow(dead_code)]
    #[serde(default)]
    build: Option<toml::Value>,
    #[allow(dead_code)]
    #[serde(default)]
    resolve: Option<toml::Value>,
    #[allow(dead_code)]
    #[serde(default)]
    test: Option<toml::Value>,
}

/// Span where an offending key appeared in the source file. `(0, 0)`
/// is reserved for synthesized errors that have no source location
/// (e.g. a missing required section). Slice 2 fills `(0, 0)` for
/// every span; Slice 2b will lift the byte-offset from
/// `toml::de::Error::span()` and convert it to 1-based line/column.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConfigSpan {
    pub line: usize,
    pub column: usize,
}

/// Typed errors for `jet.toml` validation. Caller wraps into
/// `anyhow::Error` for the legacy `WasmConfig::load` path; the
/// upcoming `jet config lint` subcommand consumes the typed variants
/// directly to format structured diagnostics.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("no jet.toml at {0}")]
    NotFound(PathBuf),

    #[error("reading {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("missing [wasm] section in {0}")]
    MissingWasmSection(PathBuf),

    #[error("unknown key {key:?} in {path}{}{}",
        if span.line > 0 { format!(" at line {}", span.line) } else { String::new() },
        suggestion.as_ref().map(|s| format!(" — did you mean {s:?}?")).unwrap_or_default()
    )]
    UnknownKey {
        path: PathBuf,
        key: String,
        suggestion: Option<String>,
        span: ConfigSpan,
    },

    #[error("invalid value in {path}: {message}")]
    InvalidValue {
        path: PathBuf,
        message: String,
        span: ConfigSpan,
    },
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
impl WasmConfig {
    /// Legacy entry point — wraps the typed error path into anyhow
    /// for callers that haven't migrated yet (the existing
    /// `wasm_build::build_with_profile` path).
    pub fn load(project_root: &Path) -> Result<Self> {
        Self::load_typed(project_root).map_err(anyhow::Error::from)
    }

    /// Typed entry point — `Result<WasmConfig, ConfigError>` so the
    /// `jet config lint` subcommand can pattern-match on variants
    /// and emit structured diagnostics.
    pub fn load_typed(project_root: &Path) -> Result<Self, ConfigError> {
        let path = project_root.join("jet.toml");
        if !path.exists() {
            return Err(ConfigError::NotFound(path));
        }
        let body = fs::read_to_string(&path).map_err(|source| ConfigError::Io {
            path: path.clone(),
            source,
        })?;
        Self::parse_str(&body, &path)
    }

    /// Parse from an in-memory string (test entry + the future
    /// `jet config lint --stdin` path). Caller supplies `path` for
    /// error context. Deprecated-key warnings are emitted via
    /// `tracing::warn!` and otherwise discarded; use
    /// `parse_str_with_warnings` if you need to inspect them
    /// (the `jet config lint` subcommand will).
    pub fn parse_str(body: &str, path: &Path) -> Result<Self, ConfigError> {
        let (cfg, warnings) = Self::parse_str_with_warnings(body, path)?;
        for w in &warnings {
            tracing::warn!("{w}");
        }
        Ok(cfg)
    }

    /// Same as `parse_str` but returns the deprecated-key warnings
    /// alongside the parsed config so callers (the future `jet
    /// config lint` subcommand, structured-diagnostics consumers)
    /// can format them their own way.
    pub fn parse_str_with_warnings(
        body: &str,
        path: &Path,
    ) -> Result<(Self, Vec<DeprecatedKeyWarning>), ConfigError> {
        let (effective, warnings) = apply_deprecated_remap(body, path, DEPRECATED_KEYS)?;
        reject_legacy_renderer_keys_in_shared_sections(&effective, path)?;
        let cfg: ConfigFile =
            toml::from_str(&effective).map_err(|e| classify_toml_error(e, path, &effective))?;
        let wasm = cfg
            .wasm
            .ok_or_else(|| ConfigError::MissingWasmSection(path.to_path_buf()))?;
        Ok((wasm, warnings))
    }
}

fn reject_legacy_renderer_keys_in_shared_sections(
    body: &str,
    path: &Path,
) -> Result<(), ConfigError> {
    let value: toml::Value =
        toml::from_str(body).map_err(|e| classify_toml_error(e, path, body))?;
    let Some(root) = value.as_table() else {
        return Ok(());
    };
    for section in LEGACY_RENDERER_SHARED_SECTIONS {
        let Some(table) = root.get(*section).and_then(toml::Value::as_table) else {
            continue;
        };
        if table.contains_key("renderer") {
            return Err(ConfigError::UnknownKey {
                path: path.to_path_buf(),
                key: format!("{section}.renderer"),
                suggestion: None,
                span: find_section_key_span(body, section, "renderer"),
            });
        }
    }
    Ok(())
}

/// Convert a `toml::de::Error` into a `ConfigError`. When the message
/// contains the well-known `unknown field` prefix that `serde` emits
/// under `deny_unknown_fields`, lift the offending key out and run
/// did-you-mean against the candidate set for the section it appeared
/// in.
fn classify_toml_error(err: toml::de::Error, path: &Path, body: &str) -> ConfigError {
    let msg = err.message().to_string();
    let span = err
        .span()
        .map(|range| span_for_byte_range(body, range.start))
        .unwrap_or_default();

    if let Some(key) = extract_unknown_field(&msg) {
        let candidates = candidate_keys_for(&msg);
        let suggestion = nearest_candidate(&key, candidates);
        return ConfigError::UnknownKey {
            path: path.to_path_buf(),
            key,
            suggestion,
            span,
        };
    }
    ConfigError::InvalidValue {
        path: path.to_path_buf(),
        message: msg,
        span,
    }
}

/// Pull the offending key out of serde's `unknown field \`XYZ\`,
/// expected one of \`a\`, \`b\`` style message.
fn extract_unknown_field(msg: &str) -> Option<String> {
    let prefix = "unknown field `";
    let start = msg.find(prefix)? + prefix.len();
    let rest = &msg[start..];
    let end = rest.find('`')?;
    Some(rest[..end].to_string())
}

/// Best-effort: when serde says `expected one of \`a\`, \`b\``, lift
/// those names to use as the candidate set. If we can't find that
/// list, fall back to the union of known top-level + wasm keys —
/// the suggestion will still usually be the right level because
/// our schema is currently shallow.
fn candidate_keys_for(msg: &str) -> Vec<&'static str> {
    let listed: Vec<&'static str> = parse_expected_list(msg)
        .into_iter()
        .filter_map(|c| {
            // Map back to the static slices so we return `&'static str`.
            WASM_SECTION_KEYS
                .iter()
                .chain(TOP_LEVEL_KEYS.iter())
                .find(|k| **k == c)
                .copied()
        })
        .collect();
    if !listed.is_empty() {
        return listed;
    }
    let mut all: Vec<&'static str> = TOP_LEVEL_KEYS.to_vec();
    all.extend_from_slice(WASM_SECTION_KEYS);
    all
}

fn parse_expected_list(msg: &str) -> Vec<String> {
    // Matches the "expected one of `a`, `b`, `c`" tail.
    let Some(start) = msg.find("expected one of") else {
        return Vec::new();
    };
    let tail = &msg[start..];
    tail.split('`')
        .skip(1)
        .step_by(2)
        .map(|s| s.to_string())
        .collect()
}

/// Convert a byte offset into a 1-based (line, column) span. Returns
/// `(0, 0)` if the offset is past the end of the source. UTF-8 safe
/// — we count chars within each line, not bytes.
fn span_for_byte_range(body: &str, byte_offset: usize) -> ConfigSpan {
    if byte_offset > body.len() {
        return ConfigSpan::default();
    }
    let prefix = &body[..byte_offset];
    let line = prefix.bytes().filter(|b| *b == b'\n').count() + 1;
    let last_nl = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let col = body[last_nl..byte_offset].chars().count() + 1;
    ConfigSpan { line, column: col }
}

/// Suggest the closest candidate within a tolerant Levenshtein
/// threshold. Threshold: `max(2, key.len() / 3)`. On ties at the
/// minimum distance, return `None` (don't guess wrong).
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
pub fn nearest_candidate(key: &str, candidates: Vec<&'static str>) -> Option<String> {
    let threshold = std::cmp::max(2, key.len() / 3);
    let mut best_dist = usize::MAX;
    let mut best: Option<&'static str> = None;
    let mut tied = false;
    for c in &candidates {
        let d = levenshtein(key, c);
        if d < best_dist {
            best_dist = d;
            best = Some(*c);
            tied = false;
        } else if d == best_dist && best != Some(*c) {
            tied = true;
        }
    }
    if best_dist <= threshold && !tied {
        best.map(String::from)
    } else {
        None
    }
}

/// Vendored Levenshtein — 30-line iterative DP, no external dep.
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0usize; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr[j + 1] =
                std::cmp::min(std::cmp::min(curr[j] + 1, prev[j + 1] + 1), prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

/// Walk the deprecated-key allowlist; for each entry whose
/// `deprecated` path appears in `body` (single-segment / top-level
/// only for now), remove the key and re-insert at the dotted
/// `replacement` path. Returns the rewritten TOML body and the list
/// of warnings to surface. If the allowlist is empty (or has no
/// hits), returns the original body unchanged so we don't pay a
/// re-serialization round-trip on the happy path.
fn apply_deprecated_remap(
    body: &str,
    path: &Path,
    table: &[DeprecatedKeyEntry],
) -> Result<(String, Vec<DeprecatedKeyWarning>), ConfigError> {
    if table.is_empty() {
        return Ok((body.to_string(), Vec::new()));
    }
    // Cheap fast-path: if no entry's deprecated string appears in the
    // body at all, skip the parse + reserialize round-trip.
    if table.iter().all(|e| !body.contains(e.deprecated)) {
        return Ok((body.to_string(), Vec::new()));
    }

    let mut value: toml::Value =
        toml::from_str(body).map_err(|e| classify_toml_error(e, path, body))?;
    let mut warnings = Vec::new();

    let toml::Value::Table(ref mut root) = value else {
        // Top-level wasn't a table — nothing to remap; let strict
        // parse handle the structural complaint.
        return Ok((body.to_string(), Vec::new()));
    };

    for entry in table {
        // Single-segment deprecated paths only for the anchor.
        // Multi-segment paths land when a real rename needs them.
        if entry.deprecated.contains('.') {
            continue;
        }
        let Some(taken) = root.remove(entry.deprecated) else {
            continue;
        };
        let span = find_key_span(body, entry.deprecated);
        let parts: Vec<&str> = entry.replacement.split('.').collect();
        let leaf = parts[parts.len() - 1];
        let mut cursor: &mut toml::map::Map<String, toml::Value> = root;
        for seg in &parts[..parts.len() - 1] {
            let slot = cursor
                .entry((*seg).to_string())
                .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
            cursor = match slot.as_table_mut() {
                Some(t) => t,
                None => {
                    return Err(ConfigError::InvalidValue {
                        path: path.to_path_buf(),
                        message: format!(
                            "cannot remap deprecated `{}` to `{}`: parent `{}` exists with non-table value",
                            entry.deprecated, entry.replacement, seg,
                        ),
                        span: ConfigSpan::default(),
                    });
                }
            };
        }
        cursor.insert(leaf.to_string(), taken);
        warnings.push(DeprecatedKeyWarning {
            path: path.to_path_buf(),
            key: entry.deprecated.to_string(),
            replacement: entry.replacement.to_string(),
            removal_version: entry.removal_version.to_string(),
            span,
        });
    }

    if warnings.is_empty() {
        return Ok((body.to_string(), warnings));
    }

    let rewritten = toml::to_string(&value).map_err(|e| ConfigError::InvalidValue {
        path: path.to_path_buf(),
        message: format!("failed to re-serialize remapped config: {e}"),
        span: ConfigSpan::default(),
    })?;
    Ok((rewritten, warnings))
}

/// Best-effort: find the line/column of a top-level key assignment
/// in the original source. Matches `^\s*<key>\s*=` style. Returns
/// `(0, 0)` when no match — callers downgrade to the synthesized
/// span sentinel.
fn find_key_span(body: &str, key: &str) -> ConfigSpan {
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim_start();
        if let Some(after) = trimmed.strip_prefix(key) {
            let next = after.trim_start();
            if next.starts_with('=') {
                let leading = line.len() - trimmed.len();
                return ConfigSpan {
                    line: i + 1,
                    column: leading + 1,
                };
            }
        }
    }
    ConfigSpan::default()
}

fn find_section_key_span(body: &str, section: &str, key: &str) -> ConfigSpan {
    let mut current_section = "";
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim_start();
        if let Some(header) = trimmed.strip_prefix('[').and_then(|s| s.split(']').next()) {
            current_section = header.trim();
            continue;
        }
        if current_section != section {
            continue;
        }
        if let Some(after) = trimmed.strip_prefix(key) {
            let next = after.trim_start();
            if next.starts_with('=') {
                let leading = line.len() - trimmed.len();
                return ConfigSpan {
                    line: i + 1,
                    column: leading + 1,
                };
            }
        }
    }
    ConfigSpan::default()
}

// Re-export for tests + integration callers (the CLI lint subcommand
// in Slice 4 will format these structured errors).
pub use ConfigError as JetConfigError;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn p() -> PathBuf {
        PathBuf::from("/tmp/jet.toml")
    }

    #[test]
    fn parses_minimal_valid_config() {
        let src = r#"
            [wasm]
            entry = "src/Counter.tsx"
            root_component = "Counter"
        "#;
        let cfg = WasmConfig::parse_str(src, &p()).unwrap();
        assert_eq!(cfg.entry, "src/Counter.tsx");
        assert_eq!(cfg.root_component, "Counter");
        assert_eq!(cfg.renderer, WasmRenderer::WebGpu);
        assert!(cfg.root_props.is_empty());
    }

    #[test]
    fn parses_dom_renderer() {
        let src = r#"
            [wasm]
            entry = "src/Counter.tsx"
            root_component = "Counter"
            renderer = "dom"
        "#;
        let cfg = WasmConfig::parse_str(src, &p()).unwrap();
        assert_eq!(cfg.renderer, WasmRenderer::Dom);
    }

    #[test]
    fn parses_webgpu_renderer() {
        let src = r#"
            [wasm]
            entry = "src/Counter.tsx"
            root_component = "Counter"
            renderer = "web-gpu"
        "#;
        let cfg = WasmConfig::parse_str(src, &p()).unwrap();
        assert_eq!(cfg.renderer, WasmRenderer::WebGpu);
    }

    #[test]
    fn legacy_build_renderer_key_is_rejected() {
        let src = r#"
            [build]
            renderer = "dom"
            out_dir = "dist"

            [wasm]
            entry = "src/Counter.tsx"
            root_component = "Counter"
        "#;
        let err = WasmConfig::parse_str(src, &p()).unwrap_err();
        match err {
            ConfigError::UnknownKey { key, span, .. } => {
                assert_eq!(key, "build.renderer");
                assert!(span.line >= 3, "expected renderer span, got {span:?}");
            }
            other => panic!("expected UnknownKey, got {other:?}"),
        }
    }

    #[test]
    fn shared_build_out_dir_without_renderer_is_still_accepted() {
        let src = r#"
            [build]
            out_dir = "dist"

            [wasm]
            entry = "src/Counter.tsx"
            root_component = "Counter"
        "#;
        let cfg = WasmConfig::parse_str(src, &p()).unwrap();
        assert_eq!(cfg.entry, "src/Counter.tsx");
    }

    #[test]
    fn missing_wasm_section_typed_error() {
        let src = "";
        let err = WasmConfig::parse_str(src, &p()).unwrap_err();
        assert!(matches!(err, ConfigError::MissingWasmSection(_)));
    }

    #[test]
    fn unknown_top_level_key_rejected() {
        let src = r#"
            [wasm]
            entry = "x"
            root_component = "X"

            [bogus]
            key = 1
        "#;
        let err = WasmConfig::parse_str(src, &p()).unwrap_err();
        match err {
            ConfigError::UnknownKey { key, .. } => assert_eq!(key, "bogus"),
            other => panic!("expected UnknownKey, got {other:?}"),
        }
    }

    #[test]
    fn unknown_wasm_section_key_rejected_with_suggestion() {
        let src = r#"
            [wasm]
            entry = "x"
            root_component = "X"
            root_propz = []
        "#;
        let err = WasmConfig::parse_str(src, &p()).unwrap_err();
        match err {
            ConfigError::UnknownKey {
                key, suggestion, ..
            } => {
                assert_eq!(key, "root_propz");
                assert_eq!(suggestion.as_deref(), Some("root_props"));
            }
            other => panic!("expected UnknownKey, got {other:?}"),
        }
    }

    #[test]
    fn unknown_key_far_from_any_candidate_yields_no_suggestion() {
        let src = r#"
            [wasm]
            entry = "x"
            root_component = "X"
            xylophone = []
        "#;
        let err = WasmConfig::parse_str(src, &p()).unwrap_err();
        match err {
            ConfigError::UnknownKey { suggestion, .. } => assert!(suggestion.is_none()),
            other => panic!("expected UnknownKey, got {other:?}"),
        }
    }

    #[test]
    fn invalid_value_kind_classified_as_invalid_value() {
        let src = r#"
            [wasm]
            entry = 12345
            root_component = "X"
        "#;
        let err = WasmConfig::parse_str(src, &p()).unwrap_err();
        assert!(matches!(err, ConfigError::InvalidValue { .. }));
    }

    #[test]
    fn unknown_key_message_renders_did_you_mean_hint() {
        let src = r#"
            [wasm]
            entry = "x"
            root_component = "X"
            root_propz = []
        "#;
        let err = WasmConfig::parse_str(src, &p()).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("root_propz"));
        assert!(msg.contains("root_props"));
    }

    #[test]
    fn unknown_key_carries_line_span() {
        // The unknown-key span should point at the offending line, not
        // (0, 0). Exact column varies by toml parser version, but line
        // is robustly the line the typo is on.
        let src = "[wasm]\nentry = \"x\"\nroot_component = \"X\"\nbroken_key = 1\n";
        let err = WasmConfig::parse_str(src, &p()).unwrap_err();
        match err {
            ConfigError::UnknownKey { span, .. } => {
                assert!(span.line >= 4, "expected span on line 4+, got {span:?}");
            }
            other => panic!("expected UnknownKey, got {other:?}"),
        }
    }

    #[test]
    fn levenshtein_basic_distances() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("same", "same"), 0);
    }

    #[test]
    fn nearest_candidate_picks_unique_close_match() {
        let cands = vec!["entry", "root_component", "root_props"];
        assert_eq!(
            nearest_candidate("root_propz", cands).as_deref(),
            Some("root_props")
        );
    }

    #[test]
    fn nearest_candidate_returns_none_for_far_input() {
        let cands = vec!["entry", "root_component", "root_props"];
        assert!(nearest_candidate("xylophone", cands).is_none());
    }

    #[test]
    fn nearest_candidate_returns_none_on_distance_tie() {
        // "abc" is distance 1 from both "abcx" and "xabc"; we'd
        // rather suggest nothing than guess.
        let cands = vec!["abcx", "xabc"];
        assert!(nearest_candidate("abc", cands).is_none());
    }

    #[test]
    fn extract_unknown_field_lifts_key_from_serde_message() {
        let msg = "unknown field `bogus`, expected one of `wasm`";
        assert_eq!(extract_unknown_field(msg).as_deref(), Some("bogus"));
    }

    #[test]
    fn parse_expected_list_pulls_candidate_keys() {
        let msg = "unknown field `bogus`, expected one of `entry`, `root_component`, `root_props`";
        let got = parse_expected_list(msg);
        // First "expected one of" tail tick gives back `entry`,
        // `root_component`, `root_props` — three picks.
        assert_eq!(got, vec!["entry", "root_component", "root_props"]);
    }

    #[test]
    fn span_for_byte_range_handles_first_line() {
        let body = "abc\ndef\n";
        let s = span_for_byte_range(body, 0);
        assert_eq!(s, ConfigSpan { line: 1, column: 1 });
    }

    #[test]
    fn span_for_byte_range_counts_lines_and_columns() {
        let body = "abc\ndef\nghi";
        // offset 5 is the 'e' on line 2 — column 2.
        let s = span_for_byte_range(body, 5);
        assert_eq!(s, ConfigSpan { line: 2, column: 2 });
    }

    #[test]
    fn legacy_load_path_still_returns_anyhow_for_unknown_key() {
        // Smoke check: callers staying on `WasmConfig::load` see the
        // typed error round-tripped into anyhow with the same hint.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("jet.toml"),
            r#"
[wasm]
entry = "x"
root_component = "X"
root_propz = []
"#,
        )
        .unwrap();
        let err = WasmConfig::load(dir.path()).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("root_propz"), "got: {msg}");
    }

    // ---- Slice 3: deprecated-key allowlist + warnings ----

    const TEST_DEPRECATED: &[DeprecatedKeyEntry] = &[
        DeprecatedKeyEntry {
            deprecated: "old-entry",
            replacement: "wasm.entry",
            removal_version: "0.5.0",
        },
        DeprecatedKeyEntry {
            deprecated: "old-component",
            replacement: "wasm.root_component",
            removal_version: "0.5.0",
        },
    ];

    #[test]
    fn apply_deprecated_remap_passes_through_when_no_hit() {
        let body = "[wasm]\nentry = \"src/index.tsx\"\nroot_component = \"App\"\n";
        let (out, warnings) = apply_deprecated_remap(body, &p(), TEST_DEPRECATED).unwrap();
        assert_eq!(out, body);
        assert!(warnings.is_empty());
    }

    #[test]
    fn apply_deprecated_remap_rewrites_into_dotted_replacement() {
        // `old-entry` → `wasm.entry` lifts the value into the [wasm]
        // section so the strict pass succeeds end-to-end.
        let body = r#"
old-entry = "src/main.tsx"
old-component = "Root"
"#;
        let (out, warnings) = apply_deprecated_remap(body, &p(), TEST_DEPRECATED).unwrap();
        assert_eq!(warnings.len(), 2);
        // Re-parse and confirm the rewritten body validates.
        let cfg: ConfigFile = toml::from_str(&out).unwrap();
        let wasm = cfg.wasm.unwrap();
        assert_eq!(wasm.entry, "src/main.tsx");
        assert_eq!(wasm.root_component, "Root");
    }

    #[test]
    fn deprecated_remap_attaches_line_span_to_warning() {
        let body = "\nold-entry = \"x\"\n";
        let (_out, warnings) = apply_deprecated_remap(body, &p(), TEST_DEPRECATED).unwrap();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].span.line, 2);
        assert_eq!(warnings[0].span.column, 1);
    }

    #[test]
    fn deprecated_key_warning_display_includes_replacement_and_version() {
        let w = DeprecatedKeyWarning {
            path: PathBuf::from("/x/jet.toml"),
            key: "dev-port".into(),
            replacement: "dev.port".into(),
            removal_version: "0.4.0".into(),
            span: ConfigSpan { line: 7, column: 1 },
        };
        let msg = format!("{w}");
        assert!(msg.contains("\"dev-port\""), "got: {msg}");
        assert!(msg.contains("\"dev.port\""), "got: {msg}");
        assert!(msg.contains("0.4.0"), "got: {msg}");
        assert!(msg.contains("line 7"), "got: {msg}");
    }

    #[test]
    fn parse_str_with_warnings_surfaces_dev_port_anchor() {
        // The production `DEPRECATED_KEYS` table maps `dev-port` →
        // `dev.port`. Since #1403 the WASM loader accepts `[dev]` as
        // a raw section (shared with the non-WASM JetConfig schema),
        // so the deprecation now lands end-to-end: warning emitted,
        // parse succeeds, no `UnknownKey` on `dev`.
        let body = r#"
dev-port = 5173
[wasm]
entry = "src/index.tsx"
root_component = "App"
"#;
        let (cfg, warnings) = WasmConfig::parse_str_with_warnings(body, &p()).unwrap();
        assert_eq!(cfg.entry, "src/index.tsx");
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].key, "dev-port");
        assert_eq!(warnings[0].replacement, "dev.port");
    }

    #[test]
    fn parse_str_with_warnings_returns_warning_when_replacement_section_exists() {
        // Build a config where the replacement parent already exists:
        // `[wasm]` + a deprecated `old-entry` that lifts into
        // `wasm.entry`. This exercises the success path (warning
        // emitted, parse succeeds).
        let body = r#"
old-entry = "src/main.tsx"
[wasm]
root_component = "App"
"#;
        // Use the test-only table since the prod table doesn't carry
        // `old-entry` (and won't — this is a synthetic example).
        let (rewritten, warnings) = apply_deprecated_remap(body, &p(), TEST_DEPRECATED).unwrap();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].key, "old-entry");
        assert_eq!(warnings[0].replacement, "wasm.entry");

        let cfg: ConfigFile = toml::from_str(&rewritten).unwrap();
        let wasm = cfg.wasm.unwrap();
        assert_eq!(wasm.entry, "src/main.tsx");
        assert_eq!(wasm.root_component, "App");
    }

    #[test]
    fn find_key_span_pinpoints_top_level_key() {
        let body = "first = 1\nsecond = 2\nthird = 3\n";
        let span = find_key_span(body, "second");
        assert_eq!(span.line, 2);
        assert_eq!(span.column, 1);
    }

    #[test]
    fn find_key_span_returns_zero_when_key_absent() {
        let body = "[wasm]\nentry = \"x\"\n";
        assert_eq!(find_key_span(body, "missing"), ConfigSpan::default());
    }

    #[test]
    fn empty_deprecated_table_is_pure_passthrough() {
        let body = "[wasm]\nentry = \"x\"\nroot_component = \"X\"\n";
        let (out, warnings) = apply_deprecated_remap(body, &p(), &[]).unwrap();
        assert_eq!(out, body);
        assert!(warnings.is_empty());
    }

    // ---- #1403: shared-with-JetConfig top-level sections ----

    #[test]
    fn accepts_cue_dogfood_shape_with_dev_alias_build_sections() {
        // Cue's 2026-05-08 jet.toml shape. Pre-#1403 this hit
        // `unknown field 'dev'` and the build aborted. The WASM
        // loader now tolerates `[dev]`, `[dev.proxy]`, `[alias]`,
        // and `[build]` as raw sections without re-validating them.
        let src = r#"
[wasm]
entry = "src/CueWasmApp.tsx"
root_component = "CueWasmApp"
root_props = [0, 1, 0]

[dev]
port = 3212

[dev.proxy]
"/api" = "http://localhost:3210"

[alias]
"@/" = "./src/"

[build]
out_dir = "../be/static"
"#;
        let cfg = WasmConfig::parse_str(src, &p()).expect("must parse");
        assert_eq!(cfg.entry, "src/CueWasmApp.tsx");
        assert_eq!(cfg.root_component, "CueWasmApp");
        assert_eq!(cfg.root_props.len(), 3);
    }

    #[test]
    fn accepts_pipeline_resolve_test_sections() {
        // Every section in JET_TOP_LEVEL_KEYS must round-trip
        // through the WASM loader, not just the Cue subset.
        let src = r#"
[wasm]
entry = "x"
root_component = "X"

[pipeline.build]
inputs = ["src/**"]

[resolve]
conditions = ["browser"]

[test]
include = ["tests/**"]
"#;
        let cfg = WasmConfig::parse_str(src, &p()).expect("must parse");
        assert_eq!(cfg.entry, "x");
    }

    #[test]
    fn unknown_section_outside_allowlist_still_rejected() {
        // `deny_unknown_fields` enforcement must still catch typos
        // or unsupported sections — only the canonical seven
        // (wasm, pipeline, dev, alias, build, resolve, test) pass.
        let src = r#"
[wasm]
entry = "x"
root_component = "X"

[bogus]
key = 1
"#;
        let err = WasmConfig::parse_str(src, &p()).unwrap_err();
        match err {
            ConfigError::UnknownKey { key, .. } => assert_eq!(key, "bogus"),
            other => panic!("expected UnknownKey on `bogus`, got {other:?}"),
        }
    }

    #[test]
    fn did_you_mean_suggests_dev_for_dvev_typo() {
        // With `dev` now in TOP_LEVEL_KEYS, the suggester should
        // pick it for a near-miss typo at the section level.
        let src = r#"
[wasm]
entry = "x"
root_component = "X"

[dvev]
port = 3212
"#;
        let err = WasmConfig::parse_str(src, &p()).unwrap_err();
        match err {
            ConfigError::UnknownKey {
                key, suggestion, ..
            } => {
                assert_eq!(key, "dvev");
                assert_eq!(suggestion.as_deref(), Some("dev"));
            }
            other => panic!("expected UnknownKey, got {other:?}"),
        }
    }
}

// Compatibility shim — keeps the old anyhow-only message path
// formatted the way the build pipeline used to expect.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
impl WasmConfig {
    /// Internal helper used by tests that want the legacy anyhow
    /// path's wrapping context. Not part of the public API.
    #[allow(dead_code)]
    fn legacy_anyhow_message(err: ConfigError) -> anyhow::Error {
        anyhow::Error::from(err)
    }
}

// Make sure `WasmConfig::load`'s `.context(...)` callers still see
// the same wrapping prose as before.
#[allow(dead_code)]
fn _typecheck_context_compat(root: &Path) -> anyhow::Result<WasmConfig> {
    WasmConfig::load(root).context("failed to read [wasm] from jet.toml")
}
// CODEGEN-END
