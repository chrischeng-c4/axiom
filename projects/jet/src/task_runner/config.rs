// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
// CODEGEN-BEGIN
//! jet.toml configuration parser.
//!
//! Defines the full configuration schema for `jet.toml`, including:
//! - `pipeline`   — task pipeline definitions (unchanged)
//! - `dev`        — dev server settings (port, proxy map)
//! - `alias`      — module path aliases (overrides tsconfig.json paths)
//! - `build`      — production build settings (out_dir)
//! - `resolve`    — module resolution settings (conditions)

use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::wasm_build::config::WasmConfig;

/// Top-level jet configuration.
///
/// `deny_unknown_fields` is the wedge for #1233 R4: a typo at the
/// section level (e.g. `[devv]` instead of `[dev]`, or any of the
/// historical "I thought this was a section" misses) now fails the
/// `jet dev` / `jet build` startup parse rather than being silently
/// dropped on the floor. Sub-sections are still permissive — the
/// per-section `deny_unknown_fields` rollout (Slice 7+) hardens
/// inside-the-section keys; this slice closes the most common
/// silent-misconfig surface first.
/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
#[derive(Debug, Clone, Deserialize, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct JetConfig {
    /// Task pipeline definitions: task_name → TaskDef.
    #[serde(default)]
    pub pipeline: HashMap<String, TaskDef>,

    /// Dev server settings.
    #[serde(default)]
    pub dev: DevConfig,

    /// Module path alias map: alias_prefix → target path.
    ///
    /// Overrides matching entries from `tsconfig.json` `compilerOptions.paths`.
    /// Example: `"@/" = "./src/"`.
    #[serde(default)]
    pub alias: HashMap<String, String>,

    /// Production build settings.
    #[serde(default)]
    pub build: JetBuildConfig,

    /// Library-build settings (`jet build --lib`). Optional because most
    /// projects ship an app, not a publishable library; absent → app-mode
    /// build. Surfaced here so `[lib]` is a recognized top-level section
    /// under `deny_unknown_fields`.
    /// @issue #170
    #[serde(default)]
    pub lib: Option<LibConfig>,

    /// Module resolution settings.
    ///
    /// Controls which `exports` conditions are tried when resolving package
    /// entries.  Absent → resolver uses default `["import", "browser", "default"]`.
    // @spec .aw/changes/enhancement-resolver-conditional-exports-import-require-browse/specs/enhancement-resolver-conditional-exports-import-require-browse-spec.md#R4
    #[serde(default)]
    pub resolve: ResolveConfig,

    /// Test-runner settings.
    // @spec .aw/tech-design/projects/jet/logic/web-server.md#W1
    #[serde(default)]
    pub test: TestConfig,

    /// WASM-build (`jet build --wasm`) settings. Optional because not
    /// every project uses canvas/WASM; absent → standard JS bundle path.
    /// Surfaced here so `[wasm]` is a recognized top-level section under
    /// `deny_unknown_fields` — the strict-validation entry point for
    /// `[wasm]` keys remains `wasm_build::config::WasmConfig::load_typed`
    /// (Slice 2 of #1233).
    #[serde(default)]
    pub wasm: Option<WasmConfig>,
}

/// `[test]` section of `jet.toml`.
// @spec .aw/tech-design/projects/jet/logic/web-server.md#W1
#[derive(Debug, Clone, Deserialize, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TestConfig {
    /// Web-server spawned before the runner starts and killed after it ends.
    #[serde(default)]
    pub web_server: Option<WebServerConfig>,
}

/// `[test.web_server]` section. Spawned as a subprocess during `jet test`;
/// the runner waits for the port (or `url`) to accept connections before
/// dispatching any specs.
// @spec .aw/tech-design/projects/jet/logic/web-server.md#W2
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebServerConfig {
    /// Shell command to spawn. Run under `sh -c` so pipes/env work.
    pub command: String,
    /// TCP port to poll on `127.0.0.1`. Ignored when `url` is set.
    pub port: Option<u16>,
    /// HTTP URL to poll with a GET request. Takes precedence over `port`.
    pub url: Option<String>,
    /// Readiness deadline in ms. Default 60000.
    #[serde(default = "default_web_server_timeout")]
    pub timeout_ms: u64,
    /// If true and the port (or URL) is already accepting connections,
    /// skip spawning and leave the external server alone. Default true.
    #[serde(default = "default_true")]
    pub reuse_existing: bool,
    /// Working directory for `command` (relative to project root). Default
    /// is the project root.
    pub cwd: Option<String>,
}

fn default_web_server_timeout() -> u64 {
    60_000
}

/// Module resolution configuration block.
///
/// Deserialized from the `[resolve]` section of `jet.toml`.
///
/// Example:
/// ```toml
/// [resolve]
/// conditions = ["import", "node", "default"]
/// ```
// @spec .aw/changes/enhancement-resolver-conditional-exports-import-require-browse/specs/enhancement-resolver-conditional-exports-import-require-browse-spec.md#R4
#[derive(Debug, Clone, Deserialize, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolveConfig {
    /// Ordered export conditions for package.json `exports` resolution.
    ///
    /// When `None`, the resolver defaults to `["import", "browser", "default"]`.
    pub conditions: Option<Vec<String>>,
}

/// Dev server configuration block.
/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
#[derive(Debug, Clone, Deserialize, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DevConfig {
    /// TCP port the dev server listens on (default: 3000).
    pub port: Option<u16>,

    /// HTTP reverse proxy map: path prefix → target URL.
    ///
    /// Example:
    /// ```toml
    /// [dev.proxy]
    /// "/api" = "http://localhost:3200"
    /// "/mcp" = "http://localhost:3200"
    /// ```
    #[serde(default)]
    pub proxy: HashMap<String, String>,
}

/// Production build configuration block.
/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
#[derive(Debug, Clone, Deserialize, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct JetBuildConfig {
    /// Output directory for production build (relative to project root).
    ///
    /// Example: `out_dir = "../be/static"`.
    pub out_dir: Option<String>,
}

/// `[lib]` section of `jet.toml` — `jet build --lib` settings.
///
/// Every field is optional so `[lib]` can be present (opting the project into
/// library-build mode) with all behaviour defaulted. Entries default to the
/// `package.json` `exports`/`module`/`main` discovery; formats default to
/// `["esm"]`.
/// @issue #170
#[derive(Debug, Clone, Deserialize, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LibConfig {
    /// Explicit entry source paths. When `None`, entries are discovered from
    /// `package.json` `exports` (falling back to `module`/`main`).
    pub entry: Option<Vec<String>>,

    /// Output formats to emit, e.g. `["esm", "cjs"]`. When `None`, defaults
    /// to `["esm"]`.
    pub formats: Option<Vec<String>>,

    /// Force-externalize all bare package specifiers. Library builds always
    /// externalize `dependencies`/`peerDependencies`; this widens that to
    /// every bare import. Default `true` for library mode.
    pub externalize_all: Option<bool>,

    /// Output directory (relative to project root). When `None`, defaults to
    /// the `[build]` `out_dir`, then `dist`.
    pub out_dir: Option<String>,

    /// Preserve the internal module structure (one output per source module)
    /// instead of bundling each entry. Default `false`; currently a deferred
    /// follow-up in the bundler.
    pub preserve_modules: Option<bool>,
}

/// Single task definition within the pipeline.
/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskDef {
    /// Task dependencies. `^task` means cross-package dependency.
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// Glob patterns for input files that affect the cache key.
    #[serde(default)]
    pub inputs: Vec<String>,

    /// Glob patterns for output files to cache.
    #[serde(default)]
    pub outputs: Vec<String>,

    /// Whether this task is cacheable (default: true).
    #[serde(default = "default_true")]
    pub cache: bool,

    /// Whether this is a long-running task (e.g., dev server).
    /// Persistent tasks are never cached.
    #[serde(default)]
    pub persistent: bool,

    /// Environment variables that affect the cache key.
    #[serde(default)]
    pub env: Vec<String>,

    /// Shell command to execute for this task.
    ///
    /// Used for pipeline hooks such as `pipeline.tailwind`.
    /// Example: `command = "tailwindcss -i ./src/index.css -o ./src/output.css --watch"`.
    pub command: Option<String>,

    /// Whether this is a watch-mode task (used by pipeline hooks).
    #[serde(default)]
    pub watch: bool,
}

fn default_true() -> bool {
    true
}

/// Top-level sections recognised by `JetConfig`. Source of truth for
/// the did-you-mean suggestion when a typo lands at section level.
/// Keep in lockstep with the struct fields above.
const JET_TOP_LEVEL_KEYS: &[&str] = &[
    "pipeline", "dev", "alias", "build", "resolve", "test", "wasm", "lib",
];

/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
impl JetConfig {
    pub fn resolve_conditions(&self) -> Option<&[String]> {
        self.resolve.conditions.as_deref()
    }

    /// Load configuration from jet.toml in the project root.
    ///
    /// On parse failure, the diagnostic includes:
    ///   * the file path,
    ///   * the 1-based line number from `toml::de::Error::span()` (when
    ///     present), and
    ///   * a did-you-mean suggestion against [`JET_TOP_LEVEL_KEYS`] when
    ///     `serde` reports an `unknown field` (the closed-precedent
    ///     surface from `bug-jet-dev-ignores-jet-config-yaml-dev-port-
    ///     and-dev-p` — #1233 R2 / R4).
    pub fn load(project_root: &Path) -> Result<Self> {
        let config_path = project_root.join("jet.toml");

        if !config_path.exists() {
            // Return empty config if no file exists
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read {}", config_path.display()))?;

        toml::from_str::<JetConfig>(&content)
            .map_err(|err| classify_jet_toml_error(err, &config_path, &content))
    }
}

/// Wrap a `toml::de::Error` from `JetConfig` parsing into an
/// `anyhow::Error` that includes a 1-based line number (when the
/// parser exposes a span) and a did-you-mean suggestion against the
/// known top-level sections (when `serde` reports an `unknown field`).
fn classify_jet_toml_error(err: toml::de::Error, path: &Path, body: &str) -> anyhow::Error {
    let msg = err.message().to_string();
    let line = err
        .span()
        .map(|range| line_for_byte_offset(body, range.start));

    if let Some(unknown) = extract_unknown_field(&msg) {
        let suggestion =
            crate::wasm_build::config::nearest_candidate(&unknown, JET_TOP_LEVEL_KEYS.to_vec());
        let where_ = match line {
            Some(l) if l > 0 => format!(" at line {l}"),
            _ => String::new(),
        };
        let hint = match suggestion {
            Some(s) => format!(" — did you mean `{s}`?"),
            None => String::new(),
        };
        return anyhow::anyhow!(
            "{}: unknown section `{}`{}{}",
            path.display(),
            unknown,
            where_,
            hint,
        );
    }

    let where_ = match line {
        Some(l) if l > 0 => format!(" at line {l}"),
        _ => String::new(),
    };
    anyhow::anyhow!("{}: {}{}", path.display(), msg, where_)
}

/// Pull the offending key out of serde's `unknown field \`XYZ\`,
/// expected one of \`a\`, \`b\`` style message. Mirrors the helper
/// in `wasm_build::config` — kept private here because that module's
/// is also private; promoting it would couple the two surfaces more
/// tightly than the wedge needs.
fn extract_unknown_field(msg: &str) -> Option<String> {
    let prefix = "unknown field `";
    let start = msg.find(prefix)? + prefix.len();
    let rest = &msg[start..];
    let end = rest.find('`')?;
    Some(rest[..end].to_string())
}

/// Convert a byte offset into a 1-based line number. Returns 0 if
/// the offset is past the end of the source. UTF-8 safe — counts
/// `\n` bytes only, which are the same byte index in any encoding
/// `toml::de::Error::span()` produces.
fn line_for_byte_offset(body: &str, byte_offset: usize) -> usize {
    if byte_offset > body.len() {
        return 0;
    }
    body[..byte_offset].bytes().filter(|b| *b == b'\n').count() + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jet_config_emits_json_schema_with_all_top_level_sections() {
        // Regression guard for #1233: every top-level `JetConfig` field
        // must appear in the schema so the JSON Schema artifact covers
        // the WHOLE surface — not just `[wasm]`. If a section is added
        // to `JetConfig` and missed here, the schema artifact silently
        // omits it and editors lose autocomplete on the new section.
        let schema = schemars::schema_for!(JetConfig);
        let props = schema
            .schema
            .object
            .as_ref()
            .expect("JetConfig schema should be an object")
            .properties
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        for expected in [
            "pipeline", "dev", "alias", "build", "resolve", "test", "wasm", "lib",
        ] {
            assert!(
                props.iter().any(|p| p == expected),
                "missing `{expected}` in JetConfig schema; got {props:?}",
            );
        }
    }

    #[test]
    fn test_parse_pipeline_config() {
        let toml_str = r#"
[pipeline.build]
depends_on = ["^build"]
outputs = ["dist/**"]
inputs = ["src/**", "package.json"]

[pipeline.test]
depends_on = ["build"]
outputs = []

[pipeline.lint]
outputs = []

[pipeline.dev]
cache = false
persistent = true
"#;

        let config: JetConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.pipeline.len(), 4);

        let build = &config.pipeline["build"];
        assert_eq!(build.depends_on, vec!["^build"]);
        assert_eq!(build.outputs, vec!["dist/**"]);
        assert!(build.cache); // default true

        let dev = &config.pipeline["dev"];
        assert!(!dev.cache);
        assert!(dev.persistent);
    }

    #[test]
    fn test_parse_dev_config() {
        let toml_str = r#"
[dev]
port = 3201

[dev.proxy]
"/api" = "http://localhost:3200"
"/webhook" = "http://localhost:3200"
"/mcp" = "http://localhost:3200"
"#;
        let config: JetConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.dev.port, Some(3201));
        assert_eq!(config.dev.proxy.len(), 3);
        assert_eq!(
            config.dev.proxy.get("/api").map(String::as_str),
            Some("http://localhost:3200")
        );
    }

    #[test]
    fn test_parse_alias_config() {
        let toml_str = r#"
[alias]
"@/" = "./src/"
"#;
        let config: JetConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.alias.len(), 1);
        assert_eq!(config.alias.get("@/").map(String::as_str), Some("./src/"));
    }

    #[test]
    fn test_parse_build_config() {
        let toml_str = r#"
[build]
out_dir = "../be/static"
"#;
        let config: JetConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.build.out_dir.as_deref(), Some("../be/static"));
    }

    #[test]
    fn test_parse_pipeline_hook_with_command() {
        let toml_str = r#"
[pipeline.tailwind]
command = "tailwindcss -i ./src/index.css -o ./src/output.css --watch"
persistent = true
cache = false
"#;
        let config: JetConfig = toml::from_str(toml_str).unwrap();
        let tailwind = &config.pipeline["tailwind"];
        assert_eq!(
            tailwind.command.as_deref(),
            Some("tailwindcss -i ./src/index.css -o ./src/output.css --watch")
        );
        assert!(tailwind.persistent);
        assert!(!tailwind.cache);
    }

    #[test]
    fn test_empty_config() {
        let toml_str = "[pipeline]";
        let config: JetConfig = toml::from_str(toml_str).unwrap();
        assert!(config.pipeline.is_empty());
    }

    #[test]
    fn test_load_nonexistent() {
        let dir = tempfile::tempdir().unwrap();
        let config = JetConfig::load(dir.path()).unwrap();
        assert!(config.pipeline.is_empty());
        assert!(config.dev.proxy.is_empty());
        assert!(config.alias.is_empty());
        assert!(config.build.out_dir.is_none());
    }

    #[test]
    fn test_task_def_defaults() {
        let toml_str = r#"
[pipeline.simple]
"#;
        let config: JetConfig = toml::from_str(toml_str).unwrap();
        let simple = &config.pipeline["simple"];
        assert!(simple.depends_on.is_empty());
        assert!(simple.inputs.is_empty());
        assert!(simple.outputs.is_empty());
        assert!(simple.cache);
        assert!(!simple.persistent);
        assert!(simple.env.is_empty());
        assert!(simple.command.is_none());
    }

    // ─── T9: config_override_conditions ──────────────────────────────────────────

    /// T9: When jet.config sets [resolve] conditions, those are used instead of default.
    // REQ: R4
    #[test]
    fn test_config_override_conditions() {
        let toml_str = r#"
[resolve]
conditions = ["import", "node", "default"]
"#;
        let config: JetConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(
            config.resolve.conditions,
            Some(vec![
                "import".to_string(),
                "node".to_string(),
                "default".to_string(),
            ]),
            "resolve.conditions from config must be deserialized correctly"
        );

        let conds = config.resolve_conditions().unwrap();
        assert_eq!(conds.len(), 3);
        assert_eq!(conds[0], "import");
        assert_eq!(conds[1], "node");
        assert_eq!(conds[2], "default");
    }

    /// When [resolve] section is absent, resolve_conditions() returns None
    /// (caller falls back to resolver default).
    // REQ: R4
    #[test]
    fn test_config_default_conditions_absent() {
        let config: JetConfig = JetConfig::default();
        assert!(
            config.resolve_conditions().is_none(),
            "No [resolve] section → resolve_conditions() returns None"
        );
    }

    /// #1233 R2 / R4 — Slice 6 wedge. A misspelled top-level section
    /// like `[devv]` (instead of `[dev]`) must fail the load rather
    /// than be silently dropped. This is the closed-precedent surface
    /// from `bug-jet-dev-ignores-jet-config-yaml-dev-port-and-dev-p`
    /// hardened at parse time. The current schema-known top-level
    /// sections (`pipeline`, `dev`, `alias`, `build`, `resolve`,
    /// `test`, `wasm`) are the only ones accepted.
    #[test]
    fn jet_config_rejects_unknown_top_level_section() {
        let toml_str = r#"
[devv]
port = 3000
"#;
        let err = toml::from_str::<JetConfig>(toml_str)
            .expect_err("deny_unknown_fields must reject `[devv]` (typo of `[dev]`)");
        let msg = err.to_string();
        assert!(
            msg.contains("devv") || msg.contains("unknown field"),
            "expected `unknown field` diagnostic mentioning `devv`, got: {msg}"
        );
    }

    /// #1233 — Slice 6 sanity. The `[wasm]` section that earlier
    /// `JetConfig::load` silently dropped (because the field didn't
    /// exist) now round-trips into the typed `wasm: Option<WasmConfig>`.
    #[test]
    fn jet_config_parses_wasm_section() {
        let toml_str = r#"
[wasm]
entry = "src/Counter.tsx"
root_component = "Counter"
root_props = [0]
"#;
        let config: JetConfig = toml::from_str(toml_str).unwrap();
        let wasm = config.wasm.expect("[wasm] section should parse");
        assert_eq!(wasm.entry, "src/Counter.tsx");
        assert_eq!(wasm.root_component, "Counter");
    }

    /// #1233 R2 — Slice 8. Typos *inside* a recognized section now
    /// fail too: `[dev]\nportt = 3000\n` is rejected by `DevConfig`'s
    /// `deny_unknown_fields`. The diagnostic from `JetConfig::load`
    /// surfaces the offending field name + line number, and the
    /// did-you-mean candidate set falls back to the section's
    /// `expected one of` list (here: `port`, `proxy`).
    #[test]
    fn jet_config_rejects_unknown_sub_section_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("jet.toml");
        std::fs::write(&path, "[dev]\nportt = 3000\n").unwrap();

        let err = JetConfig::load(dir.path())
            .expect_err("DevConfig deny_unknown_fields must reject `portt`");
        let msg = format!("{err}");
        assert!(
            msg.contains("portt"),
            "diagnostic must name the offending sub-key, got: {msg}"
        );
        assert!(
            msg.contains("line 2"),
            "diagnostic must include 1-based line number, got: {msg}"
        );
    }

    /// #2940 regression — the exact `[dev]` + `[dev.proxy]` shape
    /// reported in the bug must parse: `dev.port = Some(3212)` and one
    /// proxy entry. The bug was upstream of this parser (cli.rs was
    /// swallowing every load failure with `unwrap_or_default()`); this
    /// guard pins the contract this module is responsible for.
    #[test]
    fn jet_config_parses_artifact_studio_dev_block() {
        let toml_str = r#"
[dev]
port = 3212

[dev.proxy]
"/api" = "http://127.0.0.1:43219"
"#;
        let config: JetConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.dev.port, Some(3212));
        assert_eq!(config.dev.proxy.len(), 1);
        assert_eq!(
            config.dev.proxy.get("/api").map(String::as_str),
            Some("http://127.0.0.1:43219"),
        );
    }

    /// #3069 regression — `[pipeline.*]` from `jet.toml` must
    /// round-trip through `JetConfig::load` so `list_scripts()`
    /// (cli.rs:2192) can list them under "Pipeline tasks". The bug was
    /// upstream: cli.rs swallowed every load failure with
    /// `if let Ok(...)`. This guard pins the contract the parser is
    /// responsible for; the visible-error contract belongs in cli.rs.
    #[test]
    fn jet_config_load_parses_pipeline_section_from_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("jet.toml");
        std::fs::write(
            &path,
            r#"
[pipeline.build]
depends_on = ["^build"]

[pipeline.test]
depends_on = ["build"]
"#,
        )
        .unwrap();

        let config = JetConfig::load(dir.path()).expect("valid config must load");
        assert_eq!(config.pipeline.len(), 2);
        let build = config
            .pipeline
            .get("build")
            .expect("[pipeline.build] must round-trip");
        assert_eq!(build.depends_on, vec!["^build"]);
        let test = config
            .pipeline
            .get("test")
            .expect("[pipeline.test] must round-trip");
        assert_eq!(test.depends_on, vec!["build"]);
    }

    /// #3065 regression — `[test.web_server]` from `jet.toml` must
    /// round-trip through `JetConfig::load` so the `jet test` path
    /// (cli.rs:1671) can wire the preamble web server into the runner.
    /// The bug was upstream: cli.rs swallowed every load failure with
    /// `if let Ok(...)`. This guard pins the contract the parser is
    /// responsible for; the visible-error contract belongs in cli.rs.
    #[test]
    fn jet_config_load_parses_test_web_server_from_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("jet.toml");
        std::fs::write(
            &path,
            r#"
[test.web_server]
command = "node server.js"
port = 4321
timeout_ms = 30000
reuse_existing = true
"#,
        )
        .unwrap();

        let config = JetConfig::load(dir.path()).expect("valid config must load");
        let ws = config
            .test
            .web_server
            .expect("[test.web_server] must round-trip through JetConfig::load");
        assert_eq!(ws.command, "node server.js");
        assert_eq!(ws.port, Some(4321));
        assert_eq!(ws.timeout_ms, 30000);
        assert!(ws.reuse_existing);
    }

    /// #3061 regression — `[resolve.conditions]` and `[alias]` from
    /// `jet.toml` must round-trip through `JetConfig::load` so the
    /// `jet build` path (cli.rs:1395) can wire them into the bundler.
    /// The bug was upstream: cli.rs swallowed every load failure with
    /// `unwrap_or_default()`. This guard pins the contract the parser is
    /// responsible for; the visible-error contract belongs in cli.rs.
    #[test]
    fn jet_config_load_parses_resolve_and_alias_from_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("jet.toml");
        std::fs::write(
            &path,
            r#"
[alias]
"@/" = "./src/"

[resolve]
conditions = ["import", "node", "default"]
"#,
        )
        .unwrap();

        let config = JetConfig::load(dir.path()).expect("valid config must load");
        assert_eq!(
            config.alias.get("@/").map(String::as_str),
            Some("./src/"),
            "[alias] must round-trip through JetConfig::load"
        );
        let conds = config
            .resolve_conditions()
            .expect("[resolve.conditions] must round-trip through JetConfig::load");
        assert_eq!(conds, &["import", "node", "default"]);
    }

    /// #1233 R2 / R4 — Slice 7. `JetConfig::load` lifts the typo from
    /// `serde`'s `unknown field` message and runs did-you-mean against
    /// the known top-level sections. `[devv]` is one Levenshtein
    /// edit away from `dev`, well under the 2-character minimum
    /// threshold, so the hint must fire. The diagnostic also includes
    /// the file path and 1-based line number.
    #[test]
    fn jet_config_load_surfaces_did_you_mean_and_line_number() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("jet.toml");
        std::fs::write(&path, "# leading comment\n[devv]\nport = 3000\n").unwrap();

        let err =
            JetConfig::load(dir.path()).expect_err("JetConfig::load must reject `[devv]` typo");
        let msg = format!("{err}");
        assert!(
            msg.contains("devv"),
            "diagnostic must name the offending key, got: {msg}"
        );
        assert!(
            msg.contains("did you mean `dev`"),
            "diagnostic must include did-you-mean hint, got: {msg}"
        );
        assert!(
            msg.contains("line 2"),
            "diagnostic must include 1-based line number, got: {msg}"
        );
        assert!(
            msg.contains("jet.toml"),
            "diagnostic must include the file path, got: {msg}"
        );
    }
}
// CODEGEN-END
