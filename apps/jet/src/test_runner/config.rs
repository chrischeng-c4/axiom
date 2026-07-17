// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
// CODEGEN-BEGIN
//! Test runner configuration. See TD §CLI + Config.

use crate::task_runner::config::WebServerConfig;
use crate::test_runner::wire::WireTraceMode;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Fully-resolved runner config — sourced from defaults, static Jest project
/// configuration, then explicit CLI selections.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    pub project_root: PathBuf,
    pub test_dir: PathBuf,
    pub test_match: Vec<String>,
    pub test_ignore: Vec<String>,
    pub timeout_ms: u64,
    /// Default poll timeout (ms) for async locator/page assertions
    /// (`expect(locator).toBeVisible()` and the rest) that lack a per-call
    /// `{ timeout }` override. Distinct from `timeout_ms` (the whole-test
    /// timeout). Mirrors Playwright's 5000ms web-first-assertion default;
    /// forwarded to the JS runtime via `jetConfig.expectTimeoutMs` in
    /// `worker::build_boot` and consumed by
    /// `matchers.js::setDefaultAssertionTimeout`. #1908
    pub expect_timeout_ms: u64,
    /// Number of parallel spec workers. 1 = serial (default).
    /// Activated from stub — now wired to WorkerPool.
    // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R1
    pub workers: usize,
    /// Active reporter list. Parsed from `--reporter=list,html` comma-split.
    // @spec enhancement-html-reporter-for-native-test-runner-spec#R5
    pub reporters: Vec<Reporter>,
    /// Output directory for the HTML reporter (default: `test-results/report/`).
    // @spec enhancement-html-reporter-for-native-test-runner-spec#R5
    pub report_dir: PathBuf,
    pub grep: Option<String>,
    pub update_snapshots: bool,
    pub only_files: Vec<PathBuf>,
    /// Trace capture mode (default: Off).
    // @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R1
    pub trace: WireTraceMode,
    /// Output directory for trace zip files when `trace` is not `Off`.
    /// Defaults to `<project_root>/test-results/traces/`. The worker commits
    /// one zip per test on `TestEnd` and surfaces the path on
    /// `TestReport.trace_path`.
    // @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R11
    pub trace_dir: PathBuf,
    /// Shard selection: `(i, N)` from `--shard=i/N`. `None` = run all specs.
    // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R3
    pub shard: Option<(u32, u32)>,
    /// Base URL for relative `page.goto` calls. Read from
    /// `jet.test.config.ts` project `use.baseURL`. `None` = absolute-only.
    // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R3
    pub base_url: Option<String>,
    /// Launch browser headless (default `true`). Read from
    /// `jet.test.config.ts` project `use.headless`.
    // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R5
    pub headless: bool,
    /// Optional browser executable override for browser-backed tests.
    /// `None` keeps Jet's default pinned Chromium-first resolver.
    pub browser_executable: Option<PathBuf>,
    /// Optional existing browser CDP endpoint. When set, the worker connects
    /// to that browser and opens test pages as tabs in its default window
    /// instead of launching another browser process.
    pub browser_ws_url: Option<String>,
    /// Web-server to spawn before the runner starts. Loaded from
    /// `jet.toml` `[test.web_server]`. When `Some`, `test_runner::run`
    /// boots the server, waits for readiness, runs the specs, and kills it.
    // @spec .aw/tech-design/projects/jet/logic/web-server.md#W2
    pub web_server: Option<WebServerConfig>,
    /// On test failure, snap a PNG of every active page into
    /// `auto_artifacts_dir/<sanitized-test-name>/page-<n>.png`. Default true.
    // @spec .aw/tech-design/projects/jet/logic/auto-artifacts.md#A2
    pub auto_artifacts: bool,
    /// Directory for failure artifacts. Defaults to
    /// `<project_root>/test-results/artifacts`.
    // @spec .aw/tech-design/projects/jet/logic/auto-artifacts.md#A3
    pub auto_artifacts_dir: PathBuf,
    /// Optional live E2E runner channel. When set, the worker streams plan,
    /// case, page-action, and console events to this JSONL file and blocks at
    /// per-case checkpoints when the control file requests pause/next mode.
    pub live_e2e: Option<LiveE2eConfig>,
    /// Declared test environment shape. Defaults to `Node`, unless a static
    /// root Jest config declares `testEnvironment: "jsdom"`.
    // @spec enhancement-define-component-and-browser-like-test-environment-boundary
    pub environment: TestEnvironment,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
#[derive(Debug, Clone)]
pub struct LiveE2eConfig {
    pub event_log: PathBuf,
    pub control_path: PathBuf,
    /// Delay after each live page command so the human runner can see the
    /// controlled target progress step by step.
    pub step_delay_ms: u64,
}

/// Test environment kind for `jet test`.
///
/// `jet test` covers three test shapes — unit, component, and
/// frontend-integration — while product-flow E2E lives behind `jet e2e`.
/// This enum lets a project or CLI declare the environment a test expects.
/// `Dom` uses the project's installed `jsdom` package; `Component` remains a
/// future real-renderer boundary.
// @spec enhancement-define-component-and-browser-like-test-environment-boundary
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestEnvironment {
    /// Plain Node.js sandbox — no DOM globals, no browser APIs. Covers
    /// unit tests and pure-logic frontend tests. (Default.)
    Node,
    /// Browser-like DOM backed by the project's installed `jsdom` package.
    Dom,
    /// Component-mount DOM (Vitest browser / Playwright CT-style).
    /// Reserved for component tests that need a real renderer. Not yet
    /// implemented — selecting this fails fast with a clear error.
    Component,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
impl TestEnvironment {
    /// Parse a CLI string into a `TestEnvironment`.
    ///
    /// Accepts: `node`, `dom`, `jsdom` (alias for `dom`),
    /// `happy-dom` (alias for `dom`), `component`. Case-insensitive.
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.trim().to_lowercase().as_str() {
            "node" => Ok(Self::Node),
            "dom" | "jsdom" | "happy-dom" | "jest-environment-jsdom" => Ok(Self::Dom),
            "component" => Ok(Self::Component),
            other => Err(format!(
                "unknown --env value: {other}. expected one of: node, dom, component"
            )),
        }
    }

    /// Return `Ok(())` if the runner can execute this environment, or a clear
    /// error naming the unsupported surface.
    pub fn ensure_supported(&self) -> Result<(), String> {
        match self {
            Self::Node | Self::Dom => Ok(()),
            Self::Component => Err(
                "test environment `component` is not yet implemented in the native jet test \
                 runner. use `--env=node` for unit and pure-logic tests, or `jet e2e` for \
                 product-flow cases that need a real browser."
                    .to_string(),
            ),
        }
    }

    /// Select a CLI environment without discarding a project-level jsdom
    /// declaration. `--env node` is Jet's compatibility fallback, so a static
    /// Jest `testEnvironment: "jsdom"` remains authoritative; explicit
    /// browser-like selections still replace the project default.
    pub fn with_cli_request(self, requested: &str) -> Result<Self, String> {
        let requested = Self::parse(requested)?;
        if requested == Self::Node && self == Self::Dom {
            Ok(Self::Dom)
        } else {
            Ok(requested)
        }
    }
}

/// Reporter kinds selectable via `--reporter=<kind>`.
// @spec enhancement-html-reporter-for-native-test-runner-spec#R5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reporter {
    /// Terminal summary reporter (default).
    Term,
    /// JSON file reporter (writes `.jet/test-results.json`).
    Json,
    /// HTML report reporter.
    // @spec enhancement-html-reporter-for-native-test-runner-spec#R5
    Html,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
impl Reporter {
    /// Parse a comma-separated list of reporter kind names.
    ///
    /// Accepts `"term"`, `"json"`, `"html"` (case-insensitive, with `"list"`
    /// as an alias for `"term"`).
    ///
    /// Returns `Err` with the unrecognised name on failure.
    // @spec enhancement-html-reporter-for-native-test-runner-spec#R5
    pub fn parse_list(s: &str) -> Result<Vec<Self>, String> {
        let mut out = Vec::new();
        for part in s.split(',') {
            match part.trim().to_lowercase().as_str() {
                "term" | "list" => out.push(Reporter::Term),
                "json" => out.push(Reporter::Json),
                "html" => out.push(Reporter::Html),
                other => return Err(format!("unknown reporter: {other}")),
            }
        }
        Ok(out)
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
impl RunnerConfig {
    pub fn default_for_root(project_root: &Path) -> Result<Self> {
        let root = project_root
            .canonicalize()
            .with_context(|| format!("Invalid project root: {}", project_root.display()))?;
        let test_results = root.join("test-results");
        let environment = project_test_environment(&root);
        Ok(Self {
            test_dir: root.clone(),
            // Default HTML report dir relative to project root.
            // @spec enhancement-html-reporter-for-native-test-runner-spec#R5
            report_dir: test_results.join("report"),
            // @spec .aw/tech-design/projects/jet/logic/auto-artifacts.md#A3
            auto_artifacts_dir: test_results.join("artifacts"),
            // @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R11
            trace_dir: test_results.join("traces"),
            live_e2e: None,
            project_root: root,
            test_match: vec![
                "**/*.spec.ts".to_string(),
                "**/*.spec.tsx".to_string(),
                "**/*.test.ts".to_string(),
                "**/*.test.tsx".to_string(),
                "**/*.spec.js".to_string(),
                "**/*.test.js".to_string(),
                "**/*.spec.mjs".to_string(),
                "**/*.test.mjs".to_string(),
                "**/*.spec.cjs".to_string(),
                "**/*.test.cjs".to_string(),
            ],
            test_ignore: vec![
                "**/node_modules/**".to_string(),
                "**/.jet/**".to_string(),
                "**/.jet-test-*/**".to_string(),
                "**/dist/**".to_string(),
                "**/target/**".to_string(),
                "**/.git/**".to_string(),
            ],
            timeout_ms: 30_000,
            // Playwright web-first-assertion default (#1908).
            expect_timeout_ms: 5_000,
            // Default to logical CPU count; fall back to 1 if unavailable.
            // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R1
            workers: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1),
            reporters: vec![Reporter::Term, Reporter::Json],
            grep: None,
            update_snapshots: false,
            only_files: Vec::new(),
            trace: WireTraceMode::Off,
            shard: None,
            base_url: None,
            headless: true,
            browser_executable: None,
            browser_ws_url: None,
            web_server: None,
            // @spec .aw/tech-design/projects/jet/logic/auto-artifacts.md#A2
            auto_artifacts: true,
            // @spec enhancement-define-component-and-browser-like-test-environment-boundary
            environment,
        })
    }
}

const JEST_CONFIG_FILENAMES: &[&str] = &[
    "jest.config.ts",
    "jest.config.js",
    "jest.config.cjs",
    "jest.config.mjs",
    "jest.config.json",
    "jest.preset.ts",
    "jest.preset.js",
    "jest.preset.cjs",
    "jest.preset.mjs",
    "package.json",
];

fn project_test_environment(root: &Path) -> TestEnvironment {
    JEST_CONFIG_FILENAMES
        .iter()
        .filter_map(|filename| {
            std::fs::read_to_string(root.join(filename))
                .ok()
                .map(|source| (*filename, source))
        })
        .find_map(|(filename, source)| {
            if filename == "package.json" {
                jest_environment_from_package_json(&source)
            } else {
                jest_environment_from_source(&source)
            }
        })
        .unwrap_or(TestEnvironment::Node)
}

fn jest_environment_from_package_json(source: &str) -> Option<TestEnvironment> {
    let package: serde_json::Value = serde_json::from_str(source).ok()?;
    let value = package.get("jest")?.get("testEnvironment")?.as_str()?;
    TestEnvironment::parse(value).ok()
}

fn jest_environment_from_source(source: &str) -> Option<TestEnvironment> {
    let mut search_start = 0usize;
    const KEY: &str = "testEnvironment";
    while let Some(offset) = source[search_start..].find(KEY) {
        let start = search_start + offset;
        if source[..start]
            .chars()
            .next_back()
            .is_some_and(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
        {
            search_start = start + KEY.len();
            continue;
        }
        let after_key = &source[start + KEY.len()..];
        if after_key
            .chars()
            .next()
            .is_some_and(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
        {
            search_start = start + KEY.len();
            continue;
        }
        let Some(separator) = after_key.find(|ch| ch == ':' || ch == '=') else {
            break;
        };
        if !after_key[..separator].trim().is_empty() {
            search_start = start + KEY.len();
            continue;
        }
        let value = after_key[separator + 1..].trim_start();
        let Some(quote) = value.chars().next() else {
            break;
        };
        if quote != '\'' && quote != '"' {
            search_start = start + KEY.len();
            continue;
        }
        let quoted = &value[quote.len_utf8()..];
        let Some(end) = quoted.find(quote) else {
            search_start = start + KEY.len();
            continue;
        };
        if let Ok(environment) = TestEnvironment::parse(&quoted[..end]) {
            return Some(environment);
        }
        search_start = start + KEY.len();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn default_config_has_spec_patterns() {
        let tmp = TempDir::new().unwrap();
        let cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        assert!(cfg.test_match.iter().any(|p| p.contains("spec.ts")));
        assert!(cfg.test_match.iter().any(|p| p.contains("spec.cjs")));
        assert!(cfg.test_ignore.iter().any(|p| p.contains("node_modules")));
        assert!(cfg.test_ignore.iter().any(|p| p.contains(".jet-test-")));
        assert_eq!(cfg.timeout_ms, 30_000);
        // #1908: assertion poll timeout defaults to Playwright's 5000ms and
        // is distinct from the whole-test timeout_ms above.
        assert_eq!(cfg.expect_timeout_ms, 5_000);
        assert!(cfg.workers >= 1);
    }

    #[test]
    fn invalid_root_errors() {
        let result = RunnerConfig::default_for_root(Path::new("/definitely/does/not/exist/xyz"));
        assert!(result.is_err());
    }

    #[test]
    fn default_trace_dir_is_under_test_results() {
        let tmp = TempDir::new().unwrap();
        let cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        let expected = tmp
            .path()
            .canonicalize()
            .unwrap()
            .join("test-results")
            .join("traces");
        assert_eq!(cfg.trace_dir, expected);
    }

    #[test]
    fn default_environment_is_node() {
        let tmp = TempDir::new().unwrap();
        let cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        assert_eq!(cfg.environment, TestEnvironment::Node);
    }

    #[test]
    fn root_jest_jsdom_config_selects_dom_environment() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("jest.config.ts"),
            r#"export default { testEnvironment: "jsdom" };"#,
        )
        .unwrap();

        let cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        assert_eq!(cfg.environment, TestEnvironment::Dom);
    }

    #[test]
    fn package_jest_jsdom_config_selects_dom_environment() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("package.json"),
            r#"{"jest":{"testEnvironment":"jsdom"}}"#,
        )
        .unwrap();

        let cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        assert_eq!(cfg.environment, TestEnvironment::Dom);
    }

    #[test]
    fn node_cli_request_preserves_root_jest_jsdom_environment() {
        assert_eq!(
            TestEnvironment::Dom.with_cli_request("node").unwrap(),
            TestEnvironment::Dom
        );
        assert_eq!(
            TestEnvironment::Dom.with_cli_request("component").unwrap(),
            TestEnvironment::Component
        );
    }

    #[test]
    fn test_environment_parses_known_kinds() {
        assert_eq!(
            TestEnvironment::parse("node").unwrap(),
            TestEnvironment::Node
        );
        assert_eq!(
            TestEnvironment::parse("Node").unwrap(),
            TestEnvironment::Node
        );
        assert_eq!(TestEnvironment::parse("dom").unwrap(), TestEnvironment::Dom);
        assert_eq!(
            TestEnvironment::parse("jsdom").unwrap(),
            TestEnvironment::Dom
        );
        assert_eq!(
            TestEnvironment::parse("happy-dom").unwrap(),
            TestEnvironment::Dom
        );
        assert_eq!(
            TestEnvironment::parse("jest-environment-jsdom").unwrap(),
            TestEnvironment::Dom
        );
        assert_eq!(
            TestEnvironment::parse("component").unwrap(),
            TestEnvironment::Component
        );
        assert!(TestEnvironment::parse("e2e").is_err());
        assert!(TestEnvironment::parse("browser").is_err());
    }

    #[test]
    fn supported_and_unsupported_environments_are_explicit() {
        assert!(TestEnvironment::Node.ensure_supported().is_ok());
        assert!(TestEnvironment::Dom.ensure_supported().is_ok());
        let comp_err = TestEnvironment::Component.ensure_supported().unwrap_err();
        assert!(comp_err.contains("component"));
        assert!(comp_err.contains("--env=node"));
    }
}
// CODEGEN-END
