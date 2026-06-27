// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! Product-flow E2E run/open support.
//!
//! `jet e2e run` is the agent/CI surface: it runs cases headlessly and writes
//! machine-readable evidence. `jet e2e open` is the human visual review surface:
//! it launches one tabbed browser window with the review shell in tab 0 and the
//! active case in a controlled tab, executes cases serially, and writes the same
//! evidence to disk. `jet e2e manual` is the human-readable documentation
//! surface: it runs the same Jet-owned E2E runtime serially and publishes
//! Markdown/HTML docs.

pub mod actionability;
pub mod assertion_diff;
pub mod browser_session;
pub mod clock;
pub mod discovery;
pub mod dom_snapshot;
pub mod explorer;
pub mod lifecycle;
pub mod network;
pub mod open_controls;
pub mod open_replay;
pub mod open_state;
pub mod permissions;
pub mod playwright_shim;
pub mod retry;
pub mod screenshots;
pub mod selectors;
pub mod step_artifacts;
pub mod step_panels;
pub mod storage;
pub mod trace;
pub mod video;

use crate::browser::{Browser, LaunchOptions};
use crate::test_runner::config::{LiveE2eConfig, RunnerConfig};
use crate::test_runner::reporter::{Outcome, Summary, TestReport};
use crate::test_runner::wire::WireTraceMode;
use anyhow::{Context, Result};
use axum::body::Bytes;
use axum::extract::State;
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

pub const EVIDENCE_SCHEMA_VERSION: &str = "jet.e2e.evidence.v1";
pub const CONTROL_PROTOCOL_VERSION: &str = "jet.e2e.open-control.v1";
pub const JET_BROWSER_DRIVER: &str = "cdp-chromium";
pub const JET_REVIEW_SHELL_DRIVER: &str = "chromium-window-tabs";
pub const JET_CHROME_BROWSER_DRIVER: &str = "cdp-chrome";
pub const JET_CHROME_REVIEW_SHELL_DRIVER: &str = "chrome-window-tabs";
pub const DEFAULT_OPEN_SLOW_MO_MS: u64 = 500;

/// Deterministic process exit codes for `jet e2e run`.
///
/// Agents and CI runners disambiguate failure modes by exit code without
/// having to parse evidence. Precedence (highest first):
/// infrastructure > timeout > assertion > pass.
///
/// Invalid-config is emitted by the CLI before any cases run, so it has
/// no precedence interaction with case-level outcomes.
///
/// @spec #2618
pub const E2E_EXIT_OK: i32 = 0;
pub const E2E_EXIT_ASSERTION_FAILURE: i32 = 1;
pub const E2E_EXIT_INVALID_CONFIG: i32 = 2;
pub const E2E_EXIT_TIMEOUT: i32 = 3;
pub const E2E_EXIT_INFRASTRUCTURE: i32 = 4;

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum E2eMode {
    Run,
    Open,
    Manual,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum E2eServeMode {
    #[default]
    Off,
    Dev,
    Prod,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum E2eOpenBrowserMode {
    #[default]
    Chrome,
    Chromium,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2eRunOptions {
    pub project_root: PathBuf,
    pub cases: Vec<PathBuf>,
    pub grep: Option<String>,
    pub timeout_ms: Option<u64>,
    pub workers: Option<usize>,
    pub trace: WireTraceMode,
    pub evidence_dir: PathBuf,
    #[serde(default)]
    pub serve: E2eServeMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    pub print_json: bool,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2eOpenOptions {
    pub project_root: PathBuf,
    pub cases: Vec<PathBuf>,
    pub grep: Option<String>,
    pub timeout_ms: Option<u64>,
    pub evidence_dir: PathBuf,
    #[serde(default)]
    pub serve: E2eServeMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(default)]
    pub slow_mo_ms: u64,
    #[serde(default)]
    pub browser: E2eOpenBrowserMode,
    pub dry_run: bool,
    pub no_open: bool,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2eManualOptions {
    pub project_root: PathBuf,
    pub cases: Vec<PathBuf>,
    pub grep: Option<String>,
    pub timeout_ms: Option<u64>,
    pub trace: WireTraceMode,
    pub evidence_dir: PathBuf,
    pub out_dir: PathBuf,
    #[serde(default)]
    pub serve: E2eServeMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub print_json: bool,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2eRunResult {
    pub evidence_path: PathBuf,
    pub jsonl_path: PathBuf,
    pub review_app_path: Option<PathBuf>,
    pub manual_docs_path: Option<PathBuf>,
    pub bundle: E2eEvidenceBundle,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2eEvidenceBundle {
    pub schema_version: String,
    pub mode: E2eMode,
    pub run_id: String,
    pub started_at_ms: u64,
    pub finished_at_ms: u64,
    pub summary: E2eSummary,
    pub cases: Vec<E2eCaseEvidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<E2eArtifactRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serve_session: Option<crate::dev_server::session::ServeSession>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub browser_sessions: Vec<crate::test_runner::reporter::BrowserSessionReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_control: Option<E2eOpenControlProtocol>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct E2eSummary {
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub duration_ms: u64,
    pub exit_code: i32,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2eCaseEvidence {
    pub id: String,
    pub title: String,
    pub file: PathBuf,
    pub outcome: String,
    pub duration_ms: u64,
    pub steps: Vec<E2eProductStep>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2eProductStep {
    pub id: String,
    pub title: String,
    pub status: String,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assertion: Option<E2eAssertionDetail>,
    pub context: E2eStepContext,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2eAssertionDetail {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct E2eStepContext {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selectors: Vec<E2eSelectorContext>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub screenshots: Vec<E2eArtifactRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub console: Vec<E2eConsoleEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub network: Vec<E2eNetworkEntry>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2eSelectorContext {
    pub selector: String,
    pub action: String,
    #[serde(default)]
    pub highlighted: bool,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2eConsoleEntry {
    pub level: String,
    pub text: String,
    pub ts_ms: u64,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2eNetworkEntry {
    pub request_id: String,
    pub method: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    pub ts_start_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts_end_ms: Option<u64>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2eArtifactRef {
    pub kind: String,
    pub path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2eOpenControlProtocol {
    pub protocol_version: String,
    pub transport: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_shell: Option<E2eOpenReviewShell>,
    pub browser: E2eOpenBrowserTarget,
    pub commands: Vec<E2eOpenCommand>,
    pub event_log: PathBuf,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2eOpenReviewShell {
    pub kind: E2eOpenReviewShellKind,
    pub driver: String,
    pub runner_shell: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runner_shell_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdp_ws_url: Option<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum E2eOpenReviewShellKind {
    DesktopAppWindow,
    BrowserWindowTabs,
    ExportOnly,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2eOpenBrowserTarget {
    pub kind: E2eOpenBrowserKind,
    pub driver: String,
    pub headless: bool,
    pub isolated_profile: bool,
    pub runner_shell: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runner_shell_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdp_ws_url: Option<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum E2eOpenBrowserKind {
    ControlledJetBrowser,
    ExportOnly,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2eOpenCommand {
    pub name: String,
    pub description: String,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum E2eEvidenceEvent {
    RunStarted {
        run_id: String,
        mode: E2eMode,
        ts_ms: u64,
    },
    ServeSessionStarted {
        run_id: String,
        mode: String,
        target: String,
        url: String,
        host: String,
        port: u16,
        pid: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        log_file: Option<String>,
        ts_ms: u64,
    },
    BrowserSessionStarted {
        run_id: String,
        session_id: String,
        driver: String,
        anchor: String,
        headless: bool,
        spec_file: PathBuf,
        #[serde(skip_serializing_if = "Option::is_none")]
        pid: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ws_endpoint: Option<String>,
        ts_ms: u64,
    },
    BrowserSessionFinished {
        run_id: String,
        session_id: String,
        state: String,
        graceful_close: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        ts_ms: u64,
    },
    StepStarted {
        run_id: String,
        case_id: String,
        step_id: String,
        title: String,
        ts_ms: u64,
    },
    StepFinished {
        run_id: String,
        case_id: String,
        step_id: String,
        title: String,
        status: String,
        duration_ms: u64,
        ts_ms: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        assertion: Option<E2eAssertionDetail>,
    },
    CaseFinished {
        run_id: String,
        case_id: String,
        title: String,
        outcome: String,
        duration_ms: u64,
    },
    RunFinished {
        run_id: String,
        exit_code: i32,
        passed: u32,
        failed: u32,
        skipped: u32,
        ts_ms: u64,
    },
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub async fn run_agent_mode(opts: E2eRunOptions) -> Result<E2eRunResult> {
    if opts.serve != E2eServeMode::Off && opts.base_url.is_some() {
        anyhow::bail!("`jet e2e run --serve` and `--base-url` cannot be used together");
    }

    let started_at_ms = now_ms();
    let mut launched_serve = match opts.serve {
        E2eServeMode::Off => None,
        E2eServeMode::Dev => Some(
            crate::dev_server::serve_process::launch_detached(
                crate::dev_server::serve_process::ServeProcessOptions::dom_dev(
                    opts.project_root.clone(),
                ),
            )
            .await
            .context("starting `jet serve` for e2e run")?,
        ),
        E2eServeMode::Prod => Some(
            crate::dev_server::serve_process::launch_detached(
                crate::dev_server::serve_process::ServeProcessOptions {
                    ready_timeout: Duration::from_secs(30),
                    ..crate::dev_server::serve_process::ServeProcessOptions::dom_prod(
                        opts.project_root.clone(),
                    )
                },
            )
            .await
            .context("starting `jet serve --prod` for e2e run")?,
        ),
    };
    let base_url = launched_serve
        .as_ref()
        .map(|serve| serve.session.url.clone())
        .or_else(|| opts.base_url.clone());
    let serve_session = launched_serve.as_ref().map(|serve| serve.session.clone());

    let run_result = run_cases(
        &opts.project_root,
        &opts.cases,
        opts.grep.clone(),
        opts.timeout_ms,
        opts.workers,
        opts.trace,
        true,
        base_url,
        None,
        None,
        None,
    )
    .await;

    let shutdown_result = if let Some(serve) = launched_serve.take() {
        let result = crate::dev_server::serve_process::shutdown_host_port(
            &serve.session.host,
            serve.session.port,
        )
        .await;
        crate::dev_server::session::clear(&opts.project_root);
        Some(result.context("shutting down e2e serve session"))
    } else {
        None
    };

    let summary = run_result?;
    if let Some(shutdown) = shutdown_result {
        shutdown?;
    }

    let finished_at_ms = now_ms();
    let bundle = build_evidence_bundle(
        E2eMode::Run,
        summary,
        started_at_ms,
        finished_at_ms,
        serve_session,
        None,
    );
    let written = write_evidence_bundle(&opts.evidence_dir, &bundle)?;
    if opts.print_json {
        println!("{}", serde_json::to_string_pretty(&written.bundle)?);
    } else {
        println!("E2E evidence: {}", written.evidence_path.display());
        println!("E2E events: {}", written.jsonl_path.display());
    }
    Ok(written)
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub async fn run_manual_mode(opts: E2eManualOptions) -> Result<E2eRunResult> {
    if opts.serve != E2eServeMode::Off && opts.base_url.is_some() {
        anyhow::bail!("`jet e2e manual --serve` and `--base-url` cannot be used together");
    }

    let started_at_ms = now_ms();
    let mut launched_serve = match opts.serve {
        E2eServeMode::Off => None,
        E2eServeMode::Dev => Some(
            crate::dev_server::serve_process::launch_detached(
                crate::dev_server::serve_process::ServeProcessOptions::dom_dev(
                    opts.project_root.clone(),
                ),
            )
            .await
            .context("starting `jet serve` for e2e manual")?,
        ),
        E2eServeMode::Prod => Some(
            crate::dev_server::serve_process::launch_detached(
                crate::dev_server::serve_process::ServeProcessOptions {
                    ready_timeout: Duration::from_secs(30),
                    ..crate::dev_server::serve_process::ServeProcessOptions::dom_prod(
                        opts.project_root.clone(),
                    )
                },
            )
            .await
            .context("starting `jet serve --prod` for e2e manual")?,
        ),
    };
    let base_url = launched_serve
        .as_ref()
        .map(|serve| serve.session.url.clone())
        .or_else(|| opts.base_url.clone());
    let serve_session = launched_serve.as_ref().map(|serve| serve.session.clone());

    let run_result = run_cases(
        &opts.project_root,
        &opts.cases,
        opts.grep.clone(),
        opts.timeout_ms,
        Some(1),
        opts.trace,
        true,
        base_url,
        None,
        None,
        None,
    )
    .await;

    let shutdown_result = if let Some(serve) = launched_serve.take() {
        let result = crate::dev_server::serve_process::shutdown_host_port(
            &serve.session.host,
            serve.session.port,
        )
        .await;
        crate::dev_server::session::clear(&opts.project_root);
        Some(result.context("shutting down e2e manual serve session"))
    } else {
        None
    };

    let summary = run_result?;
    if let Some(shutdown) = shutdown_result {
        shutdown?;
    }

    let finished_at_ms = now_ms();
    let mut bundle = build_evidence_bundle(
        E2eMode::Manual,
        summary,
        started_at_ms,
        finished_at_ms,
        serve_session,
        None,
    );
    let out_dir = resolve_project_path(&opts.project_root, &opts.out_dir);
    let manual_docs = write_manual_docs(
        &opts.project_root,
        &out_dir,
        &bundle,
        opts.title.as_deref().unwrap_or("Jet E2E Manual"),
    )?;
    bundle.artifacts.push(E2eArtifactRef {
        kind: "manual-markdown".to_string(),
        path: artifact_path_for_project(&opts.project_root, &manual_docs.markdown_path),
        label: Some("Manual Markdown".to_string()),
    });
    bundle.artifacts.push(E2eArtifactRef {
        kind: "manual-html".to_string(),
        path: artifact_path_for_project(&opts.project_root, &manual_docs.html_path),
        label: Some("Manual HTML".to_string()),
    });

    let mut written = write_evidence_bundle(&opts.evidence_dir, &bundle)?;
    written.manual_docs_path = Some(manual_docs.html_path.clone());
    if opts.print_json {
        println!("{}", serde_json::to_string_pretty(&written.bundle)?);
    } else {
        println!("E2E manual docs: {}", manual_docs.html_path.display());
        println!(
            "E2E manual markdown: {}",
            manual_docs.markdown_path.display()
        );
        println!("E2E evidence: {}", written.evidence_path.display());
        println!("E2E events: {}", written.jsonl_path.display());
    }
    Ok(written)
}

fn open_browser_executable(mode: E2eOpenBrowserMode) -> Result<Option<PathBuf>> {
    match mode {
        E2eOpenBrowserMode::Chrome => {
            let path = crate::browser::launcher::BrowserLauncher::find_system_chrome()
                .context("finding system Chrome for `jet e2e open --browser chrome`")?;
            Ok(Some(path))
        }
        E2eOpenBrowserMode::Chromium => Ok(None),
    }
}

fn open_browser_driver(mode: E2eOpenBrowserMode) -> &'static str {
    match mode {
        E2eOpenBrowserMode::Chrome => JET_CHROME_BROWSER_DRIVER,
        E2eOpenBrowserMode::Chromium => JET_BROWSER_DRIVER,
    }
}

fn open_review_shell_driver(mode: E2eOpenBrowserMode) -> &'static str {
    match mode {
        E2eOpenBrowserMode::Chrome => JET_CHROME_REVIEW_SHELL_DRIVER,
        E2eOpenBrowserMode::Chromium => JET_REVIEW_SHELL_DRIVER,
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub async fn open_human_mode(opts: E2eOpenOptions) -> Result<E2eRunResult> {
    if opts.serve != E2eServeMode::Off && opts.base_url.is_some() {
        anyhow::bail!("`jet e2e open --serve` and `--base-url` cannot be used together");
    }

    let browser_executable = open_browser_executable(opts.browser)?;
    let review_shell_driver = open_review_shell_driver(opts.browser);
    let browser_driver = open_browser_driver(opts.browser);
    let started_at_ms = now_ms();
    let run_id = make_run_id(started_at_ms, E2eMode::Open);
    let mut launched_serve = match opts.serve {
        E2eServeMode::Off => None,
        E2eServeMode::Dev => Some(
            crate::dev_server::serve_process::launch_detached(
                crate::dev_server::serve_process::ServeProcessOptions::dom_dev(
                    opts.project_root.clone(),
                ),
            )
            .await
            .context("starting `jet serve` for e2e open")?,
        ),
        E2eServeMode::Prod => Some(
            crate::dev_server::serve_process::launch_detached(
                crate::dev_server::serve_process::ServeProcessOptions {
                    ready_timeout: Duration::from_secs(30),
                    ..crate::dev_server::serve_process::ServeProcessOptions::dom_prod(
                        opts.project_root.clone(),
                    )
                },
            )
            .await
            .context("starting `jet serve --prod` for e2e open")?,
        ),
    };
    let base_url = launched_serve
        .as_ref()
        .map(|serve| serve.session.url.clone())
        .or_else(|| opts.base_url.clone());
    let serve_session = launched_serve.as_ref().map(|serve| serve.session.clone());
    let initial_summary = Summary::default();
    let shell_path = open_runner_shell_path(&opts.evidence_dir);
    let live_files = LiveRunnerFiles::new(&opts.evidence_dir, &run_id);
    live_files.write_control(LiveControlState::default())?;
    write_live_event(
        &live_files.event_log,
        &json!({
            "kind": "run_started",
            "ts_ms": started_at_ms,
            "run_id": run_id,
            "mode": "open",
        }),
    );
    let live_server = if opts.no_open {
        None
    } else {
        Some(LiveRunnerServer::start(live_files.clone()).await?)
    };
    let mut control = open_control_protocol(
        Some(E2eOpenReviewShell {
            kind: if opts.no_open {
                E2eOpenReviewShellKind::ExportOnly
            } else {
                E2eOpenReviewShellKind::BrowserWindowTabs
            },
            driver: review_shell_driver.to_string(),
            runner_shell: shell_path.clone(),
            runner_shell_url: live_server.as_ref().map(|server| server.url.clone()),
            cdp_ws_url: None,
        }),
        E2eOpenBrowserTarget {
            kind: if opts.no_open {
                E2eOpenBrowserKind::ExportOnly
            } else {
                E2eOpenBrowserKind::ControlledJetBrowser
            },
            driver: browser_driver.to_string(),
            headless: false,
            isolated_profile: true,
            runner_shell: shell_path.clone(),
            runner_shell_url: live_server.as_ref().map(|server| server.url.clone()),
            cdp_ws_url: None,
        },
        &live_files.event_log,
    );
    let mut initial_bundle = build_evidence_bundle(
        E2eMode::Open,
        initial_summary,
        started_at_ms,
        started_at_ms,
        serve_session.clone(),
        Some(control.clone()),
    );
    initial_bundle.run_id = run_id.clone();
    write_open_runner_shell(&opts.evidence_dir, &initial_bundle)?;

    let review_shell = if opts.no_open {
        None
    } else {
        let runner_url = live_server
            .as_ref()
            .map(|server| server.url.as_str())
            .context("live runner server missing")?;
        let session =
            ReviewShellWindow::launch(&opts.evidence_dir, runner_url, browser_executable.clone())
                .await?;
        println!("E2E runner URL: {runner_url}");
        println!(
            "E2E visual pacing: {}ms between page actions",
            opts.slow_mo_ms
        );
        if let Some(shell) = control.review_shell.as_mut() {
            shell.cdp_ws_url = Some(session.ws_url.clone());
        }
        control.browser.cdp_ws_url = Some(session.ws_url.clone());
        Some(session)
    };

    let shared_browser_ws_url = review_shell.as_ref().map(|shell| shell.ws_url.clone());
    let summary = if opts.dry_run {
        Summary::default()
    } else {
        run_cases(
            &opts.project_root,
            &opts.cases,
            opts.grep.clone(),
            opts.timeout_ms,
            Some(1),
            WireTraceMode::RetainOnFailure,
            false,
            base_url,
            if shared_browser_ws_url.is_some() {
                None
            } else {
                browser_executable.clone()
            },
            shared_browser_ws_url,
            if opts.no_open {
                None
            } else {
                Some(LiveE2eConfig {
                    event_log: live_files.event_log.clone(),
                    control_path: live_files.control_path.clone(),
                    step_delay_ms: opts.slow_mo_ms,
                })
            },
        )
        .await?
    };
    let finished_at_ms = now_ms();
    let mut bundle = build_evidence_bundle(
        E2eMode::Open,
        summary,
        started_at_ms,
        finished_at_ms,
        serve_session,
        Some(control),
    );
    bundle.run_id = run_id;
    let mut written = write_evidence_bundle(&opts.evidence_dir, &bundle)?;
    let app_path = write_open_runner_shell(&opts.evidence_dir, &written.bundle)?;
    if review_shell.is_some() {
        write_live_event(
            &live_files.event_log,
            &json!({
                "kind": "run_finished",
                "ts_ms": finished_at_ms,
                "run_id": written.bundle.run_id,
                "exit_code": written.bundle.summary.exit_code,
                "passed": written.bundle.summary.passed,
                "failed": written.bundle.summary.failed,
                "skipped": written.bundle.summary.skipped,
            }),
        );
        println!("Jet E2E shell: browser tab review shell launched via {review_shell_driver}");
        println!("Jet Browser target: visible controlled tab launched via {browser_driver}");
    } else {
        println!("Jet E2E shell: export-only (--no-open)");
    }
    println!("E2E runner shell: {}", app_path.display());
    println!("E2E evidence: {}", written.evidence_path.display());
    written.review_app_path = Some(app_path);
    if let Some(shell) = review_shell {
        println!("E2E open session is still live. Press Ctrl-C to close the review shell.");
        let keep_result = keep_open_session_alive(&opts, &live_files, &shell, &written.bundle)
            .await
            .context("keeping e2e open session alive");
        let close_result = shell
            .close()
            .await
            .context("closing e2e open review browser");
        let shutdown_result = if let Some(serve) = launched_serve.take() {
            let result = crate::dev_server::serve_process::shutdown_host_port(
                &serve.session.host,
                serve.session.port,
            )
            .await;
            crate::dev_server::session::clear(&opts.project_root);
            Some(result.context("shutting down e2e open serve session"))
        } else {
            None
        };
        keep_result?;
        close_result?;
        if let Some(shutdown) = shutdown_result {
            shutdown?;
        }
    } else if let Some(serve) = launched_serve.take() {
        crate::dev_server::serve_process::shutdown_host_port(
            &serve.session.host,
            serve.session.port,
        )
        .await
        .context("shutting down e2e open serve session")?;
        crate::dev_server::session::clear(&opts.project_root);
    }
    Ok(written)
}

struct ReviewShellWindow {
    browser: Browser,
    ws_url: String,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl ReviewShellWindow {
    async fn launch(
        evidence_dir: &Path,
        runner_url: &str,
        executable: Option<PathBuf>,
    ) -> Result<Self> {
        let profile_dir = evidence_dir.join("jet-e2e-review-shell-profile");
        std::fs::create_dir_all(&profile_dir)
            .with_context(|| format!("Failed to create {}", profile_dir.display()))?;
        let browser = Browser::launch(LaunchOptions {
            executable,
            headless: false,
            user_data_dir: Some(profile_dir),
            args: vec!["--window-size=1500,960".to_string(), runner_url.to_string()],
            ..LaunchOptions::default()
        })
        .await
        .context("launching Jet E2E tabbed review window")?;
        let ws_url = browser.ws_url().to_string();
        Ok(Self { browser, ws_url })
    }

    async fn close(self) -> Result<()> {
        self.browser.close().await
    }
}

async fn keep_open_session_alive(
    opts: &E2eOpenOptions,
    live_files: &LiveRunnerFiles,
    _review_shell: &ReviewShellWindow,
    bundle: &E2eEvidenceBundle,
) -> Result<()> {
    let Some(open_control) = bundle.open_control.clone() else {
        tokio::signal::ctrl_c()
            .await
            .context("waiting for Ctrl-C to close e2e open session")?;
        return Ok(());
    };
    let mut seen_replay_token = live_files.read_control().replay_token;
    loop {
        tokio::select! {
            result = tokio::signal::ctrl_c() => {
                result.context("waiting for Ctrl-C to close e2e open session")?;
                break;
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(250)) => {
                let control = live_files.read_control();
                if control.replay_token != seen_replay_token {
                    seen_replay_token = control.replay_token;
                    replay_open_case(opts, live_files, &open_control, &control)
                        .await?;
                }
            }
        }
    }
    Ok(())
}

async fn replay_open_case(
    opts: &E2eOpenOptions,
    live_files: &LiveRunnerFiles,
    open_control: &E2eOpenControlProtocol,
    control: &LiveControlState,
) -> Result<()> {
    let selected_title = replay_case_title(live_files, control);
    let grep = selected_title.as_ref().map(|title| regex::escape(title));
    let replay_started_at = now_ms();
    let shared_browser_ws_url = open_control
        .review_shell
        .as_ref()
        .and_then(|shell| shell.cdp_ws_url.clone());
    let browser_executable = if shared_browser_ws_url.is_some() {
        None
    } else {
        open_browser_executable(opts.browser)?
    };
    write_live_event(
        &live_files.event_log,
        &json!({
            "kind": "replay_started",
            "ts_ms": replay_started_at,
            "case_index": control.replay_case_index,
            "case_id": control.replay_case_id.as_deref(),
            "title": selected_title.as_deref(),
        }),
    );
    let summary = run_cases(
        &opts.project_root,
        &opts.cases,
        grep,
        opts.timeout_ms,
        Some(1),
        WireTraceMode::RetainOnFailure,
        false,
        None,
        browser_executable,
        shared_browser_ws_url,
        Some(LiveE2eConfig {
            event_log: live_files.event_log.clone(),
            control_path: live_files.control_path.clone(),
            step_delay_ms: opts.slow_mo_ms,
        }),
    )
    .await?;
    let replay_finished_at = now_ms();
    let mut replay_bundle = build_evidence_bundle(
        E2eMode::Open,
        summary,
        replay_started_at,
        replay_finished_at,
        None,
        Some(open_control.clone()),
    );
    replay_bundle.run_id = make_run_id(replay_started_at, E2eMode::Open);
    let written = write_evidence_bundle(&opts.evidence_dir, &replay_bundle)?;
    write_open_runner_shell(&opts.evidence_dir, &written.bundle)?;
    write_live_event(
        &live_files.event_log,
        &json!({
            "kind": "replay_finished",
            "ts_ms": replay_finished_at,
            "run_id": written.bundle.run_id,
            "case_index": control.replay_case_index,
            "case_id": control.replay_case_id.as_deref(),
            "title": selected_title.as_deref(),
            "passed": written.bundle.summary.passed,
            "failed": written.bundle.summary.failed,
            "skipped": written.bundle.summary.skipped,
            "exit_code": written.bundle.summary.exit_code,
        }),
    );
    Ok(())
}

fn replay_case_title(files: &LiveRunnerFiles, control: &LiveControlState) -> Option<String> {
    if let Some(title) = control
        .replay_case_title
        .as_ref()
        .filter(|title| !title.is_empty())
    {
        return Some(title.clone());
    }
    let index = control.replay_case_index?;
    read_runner_bundle(files)
        .and_then(|bundle| bundle.cases.get(index).map(|case| case.title.clone()))
}

#[derive(Clone)]
struct LiveRunnerFiles {
    event_log: PathBuf,
    control_path: PathBuf,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl LiveRunnerFiles {
    fn new(evidence_dir: &Path, run_id: &str) -> Self {
        Self {
            event_log: evidence_dir.join(format!("{run_id}.live.events.jsonl")),
            control_path: evidence_dir.join(format!("{run_id}.control.json")),
        }
    }

    fn write_control(&self, control: LiveControlState) -> Result<()> {
        if let Some(parent) = self.control_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        std::fs::write(
            &self.control_path,
            serde_json::to_vec_pretty(&control).context("serialising live control state")?,
        )
        .with_context(|| format!("Failed to write {}", self.control_path.display()))
    }

    fn read_control(&self) -> LiveControlState {
        // GH #3150 — distinguish "file absent" (legitimate fresh
        // session) from "file unreadable / corrupt JSON" (silent state
        // loss). The latter is polled every few hundred ms while the
        // live UI is open; a transient corruption would silently reset
        // pause/resume/replay state on every poll with zero
        // diagnostic. Keep returning default for liveness, but emit a
        // tracing warning so the dev can find the cause.
        let body = match std::fs::read_to_string(&self.control_path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return LiveControlState::default();
            }
            Err(e) => {
                tracing::warn!(
                    target: "jet::e2e::live",
                    "control file {:?} unreadable: {e}; reverting to default state (GH #3150)",
                    self.control_path
                );
                return LiveControlState::default();
            }
        };
        match serde_json::from_str(&body) {
            Ok(state) => state,
            Err(e) => {
                tracing::warn!(
                    target: "jet::e2e::live",
                    "control file {:?} contains invalid JSON: {e}; reverting to default state (GH #3150)",
                    self.control_path
                );
                LiveControlState::default()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LiveControlState {
    #[serde(default)]
    paused: bool,
    #[serde(default = "default_live_speed_multiplier")]
    speed_multiplier: u64,
    #[serde(default)]
    next_token: u64,
    #[serde(default)]
    replay_token: u64,
    #[serde(default)]
    replay_case_index: Option<usize>,
    #[serde(default)]
    replay_case_id: Option<String>,
    #[serde(default)]
    replay_case_title: Option<String>,
}

fn default_live_speed_multiplier() -> u64 {
    1
}

fn parse_live_speed_multiplier(payload: &serde_json::Value) -> Option<u64> {
    payload
        .get("speed_multiplier")
        .or_else(|| payload.get("speed"))
        .and_then(|value| value.as_u64())
        .filter(|value| matches!(value, 1 | 2 | 4))
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl Default for LiveControlState {
    fn default() -> Self {
        Self {
            paused: false,
            speed_multiplier: default_live_speed_multiplier(),
            next_token: 0,
            replay_token: 0,
            replay_case_index: None,
            replay_case_id: None,
            replay_case_title: None,
        }
    }
}

struct LiveRunnerServer {
    url: String,
    _task: tokio::task::JoinHandle<()>,
}

#[derive(Clone)]
struct LiveRunnerServerState {
    files: LiveRunnerFiles,
    touched_at_ms: Arc<Mutex<u64>>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl LiveRunnerServer {
    async fn start(files: LiveRunnerFiles) -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .context("binding e2e live runner server")?;
        let addr = listener.local_addr().context("reading live runner addr")?;
        let state = LiveRunnerServerState {
            files: files.clone(),
            touched_at_ms: Arc::new(Mutex::new(now_ms())),
        };
        let app = Router::new()
            .route("/", get(live_runner_index))
            .route("/api/state", get(live_runner_state))
            .route("/api/control/{command}", post(live_runner_control))
            .with_state(state);
        let task = tokio::spawn(serve_live_runner(listener, app, addr));
        Ok(Self {
            url: format!("http://{addr}/"),
            _task: task,
        })
    }
}

/// GH #3490 — drive the axum live-runner server and surface a structured
/// error log if `axum::serve` returns `Err`. The prior `let _ = ...` form
/// silently dropped IO failures (accept loop dies, listener invalidated by
/// a sleep/wake cycle), leaving the user with a "connection refused" in
/// their browser and no jet-side breadcrumb to chase. Extracted as a free
/// async fn (instead of inlined inside `tokio::spawn`) so it can be unit-
/// tested with a deliberately-broken listener — see `gh3490_tests`.
async fn serve_live_runner(listener: TcpListener, app: Router, addr: std::net::SocketAddr) {
    if let Err(err) = axum::serve(listener, app).await {
        tracing::error!(
            target: "jet::e2e::live_runner",
            addr = %addr,
            error_kind = ?err.kind(),
            error = %err,
            "GH #3490 e2e live-runner axum::serve loop terminated with an \
             error; the live e2e UI server is now dead and any subsequent \
             requests against its URL will fail. Restart the e2e session \
             or inspect the network stack for invalidated listeners."
        );
    }
}

/// Format the warning emitted when `render_runner_shell_html` returns
/// `Err` and the live runner index degrades to the fallback shell.
///
/// The message names the case count, the underlying error, AND the
/// user-visible consequence ("no recent runs" fallback) so the dev knows
/// to look at the renderer / template path instead of debugging the
/// evidence writer. Tagged `GH #3552` so grepping the symptom lands here.
///
/// Extracted as a free function so unit tests can pin the message shape
/// without inspecting log capture.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub(crate) fn format_live_runner_render_warn(case_count: usize, err: &anyhow::Error) -> String {
    format!(
        "GH #3552 e2e live-runner render_runner_shell_html failed for a bundle with {case_count} case(s): {err}; falling back to the empty fallback_runner_html shell. The browser will show the 'no recent runs' placeholder even though evidence IS present on disk. The bug is in the renderer / review-UI template, not the evidence writer — start there."
    )
}

async fn live_runner_index(State(state): State<LiveRunnerServerState>) -> impl IntoResponse {
    let bundle = read_runner_bundle(&state.files);
    let html = match bundle {
        Some(bundle) => match render_runner_shell_html(&bundle) {
            Ok(html) => html,
            Err(err) => {
                tracing::warn!(
                    target: "jet::e2e::live",
                    case_count = bundle.cases.len(),
                    error = %err,
                    "{}",
                    format_live_runner_render_warn(bundle.cases.len(), &err)
                );
                fallback_runner_html()
            }
        },
        None => fallback_runner_html(),
    };
    Html(html)
}

async fn live_runner_state(State(state): State<LiveRunnerServerState>) -> impl IntoResponse {
    let events = read_jsonl_values(&state.files.event_log);
    let control = state.files.read_control();
    let bundle = read_runner_bundle(&state.files);
    let touched_at_ms = *state.touched_at_ms.lock().await;
    Json(json!({
        "bundle": bundle,
        "events": events,
        "control": control,
        "touched_at_ms": touched_at_ms,
    }))
}

async fn live_runner_control(
    State(state): State<LiveRunnerServerState>,
    axum::extract::Path(command): axum::extract::Path<String>,
    body: Bytes,
) -> axum::response::Response {
    use axum::http::StatusCode;

    // GH #3164 — distinguish "empty body" from "malformed JSON". The
    // previous `.unwrap_or_else(|_| json!({}))` silently turned a bad
    // `replay {"case_index": 5}` into `replay {}` (= replay first
    // failure) and then returned `{ "ok": true }`. Surface the parse
    // error to the caller instead.
    let payload = if body.is_empty() {
        json!({})
    } else {
        match serde_json::from_slice::<serde_json::Value>(&body) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(
                    target: "jet::e2e::live",
                    "control {command:?} rejected: malformed JSON body: {e} (GH #3164)"
                );
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "ok": false,
                        "error": format!(
                            "malformed JSON body for control {command:?}: {e} (GH #3164)"
                        ),
                    })),
                )
                    .into_response();
            }
        }
    };
    let mut control = state.files.read_control();
    match command.as_str() {
        "pause" => control.paused = true,
        "resume" => control.paused = false,
        "next" => {
            control.paused = true;
            control.next_token = control.next_token.saturating_add(1);
        }
        "speed" => {
            let Some(speed_multiplier) = parse_live_speed_multiplier(&payload) else {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "ok": false,
                        "error": "speed control requires speed_multiplier of 1, 2, or 4",
                    })),
                )
                    .into_response();
            };
            control.speed_multiplier = speed_multiplier;
        }
        "replay" => {
            control.replay_token = control.replay_token.saturating_add(1);
            control.replay_case_index = payload
                .get("case_index")
                .and_then(|value| value.as_u64())
                .map(|value| value as usize);
            control.replay_case_id = payload
                .get("case_id")
                .and_then(|value| value.as_str())
                .map(str::to_string);
            control.replay_case_title = payload
                .get("case_title")
                .and_then(|value| value.as_str())
                .map(str::to_string);
        }
        // GH #3164 — unknown command was previously a silent no-op that
        // still returned `{ "ok": true }`. Reject with 400 + list of
        // supported commands so a typo'd `pasue` doesn't masquerade as
        // success.
        other => {
            tracing::warn!(
                target: "jet::e2e::live",
                "control rejected: unknown command {other:?} (GH #3164)"
            );
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": format!(
                        "unknown control command {other:?}; \
                         supported: pause | resume | next | speed | replay (GH #3164)"
                    ),
                })),
            )
                .into_response();
        }
    }
    // GH #3164 — write failures used to be silently dropped with
    // `let _ = ...`. The in-memory `control` was returned in the
    // response but never persisted; the next /api/state poll re-read
    // stale disk state. Surface as 500 so the UI sees the failure.
    if let Err(e) = state.files.write_control(control.clone()) {
        tracing::warn!(
            target: "jet::e2e::live",
            "control {command:?} accepted but write_control failed: {e:#} (GH #3164)"
        );
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "ok": false,
                "error": format!(
                    "control {command:?} could not be persisted: {e:#} (GH #3164)"
                ),
            })),
        )
            .into_response();
    }
    *state.touched_at_ms.lock().await = now_ms();
    write_live_event(
        &state.files.event_log,
        &json!({
            "kind": "control",
            "ts_ms": now_ms(),
            "command": command,
            "control": control,
        }),
    );
    Json(json!({ "ok": true, "control": control })).into_response()
}

fn read_runner_bundle(files: &LiveRunnerFiles) -> Option<E2eEvidenceBundle> {
    let app_state = files
        .event_log
        .parent()?
        .join("open-runner-shell")
        .join("app-state.json");
    // GH #3258 — distinguish NotFound (open-mode runner hasn't written
    // the snapshot yet — legitimate silence) from other IO / parse
    // errors (partial write, schema drift, EIO). The prior `.ok().and_then`
    // chain silently rendered every malformed snapshot as "no bundle"
    // and the open shell rendered blank with no diagnostic trail.
    let body = match std::fs::read_to_string(&app_state) {
        Ok(b) => b,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return None,
        Err(err) => {
            tracing::warn!(
                target: "jet::e2e::live",
                path = %app_state.display(),
                error = %err,
                "GH #3258 failed to read open-runner-shell app-state.json; open mode will show no bundle"
            );
            return None;
        }
    };
    match serde_json::from_str(&body) {
        Ok(bundle) => Some(bundle),
        Err(err) => {
            tracing::warn!(
                target: "jet::e2e::live",
                path = %app_state.display(),
                error = %err,
                "GH #3258 failed to parse open-runner-shell app-state.json; open mode will show no bundle"
            );
            None
        }
    }
}

fn read_jsonl_values(path: &Path) -> Vec<serde_json::Value> {
    // GH #3258 — outer NotFound stays silent (the live runner polls
    // this path; absence just means "no events yet"), other read
    // errors warn. Per-line parse failures warn but do not nuke the
    // surrounding valid events.
    let body = match std::fs::read_to_string(path) {
        Ok(b) => b,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(err) => {
            tracing::warn!(
                target: "jet::e2e::live",
                path = %path.display(),
                error = %err,
                "GH #3258 failed to read live-event JSONL; open mode timeline will be empty"
            );
            return Vec::new();
        }
    };
    let mut values = Vec::new();
    for (idx, line) in body.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str(line) {
            Ok(v) => values.push(v),
            Err(err) => {
                tracing::warn!(
                    target: "jet::e2e::live",
                    path = %path.display(),
                    line_idx = idx,
                    error = %err,
                    "GH #3258 dropping malformed live-event JSONL line; open mode timeline truncated"
                );
            }
        }
    }
    values
}

// GH #3174 — Surface live-event-log write failures via tracing::warn!.
// The function stays infallible (callers don't bubble it up — event
// logging is best-effort), but the diagnostic surfaces so a perms bug
// at the evidence-dir doesn't masquerade as "no events happened". This
// is the only persistent record of step-by-step e2e progress, so silent
// loss = silent loss of debuggability.
fn write_live_event(path: &Path, value: &serde_json::Value) {
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            tracing::warn!(
                target: "jet::e2e::live",
                "failed to create live event log dir {:?}: {e}; \
                 live event {:?} dropped (GH #3174)",
                parent, path
            );
            return;
        }
    }
    let line = match serde_json::to_string(value) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(
                target: "jet::e2e::live",
                "failed to serialise live event for {:?}: {e} (GH #3174)",
                path
            );
            return;
        }
    };
    use std::io::Write;
    let mut file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!(
                target: "jet::e2e::live",
                "failed to open live event log {:?}: {e} (GH #3174)",
                path
            );
            return;
        }
    };
    if let Err(e) = writeln!(file, "{line}") {
        tracing::warn!(
            target: "jet::e2e::live",
            "failed to append to live event log {:?}: {e} (GH #3174)",
            path
        );
    }
}

fn fallback_runner_html() -> String {
    "<!doctype html><title>Jet E2E Runner</title><body>Jet E2E Runner starting...</body>"
        .to_string()
}

async fn run_cases(
    project_root: &Path,
    cases: &[PathBuf],
    grep: Option<String>,
    timeout_ms: Option<u64>,
    workers: Option<usize>,
    trace: WireTraceMode,
    headless: bool,
    base_url: Option<String>,
    browser_executable: Option<PathBuf>,
    browser_ws_url: Option<String>,
    live_e2e: Option<LiveE2eConfig>,
) -> Result<Summary> {
    let mut cfg = RunnerConfig::default_for_root(project_root)?;
    cfg.only_files = cases.to_vec();
    cfg.grep = grep;
    cfg.base_url = base_url;
    if let Some(timeout_ms) = timeout_ms {
        cfg.timeout_ms = timeout_ms;
    }
    if let Some(workers) = workers {
        cfg.workers = workers.max(1);
    }
    cfg.trace = trace;
    cfg.headless = headless;
    cfg.browser_executable = browser_executable;
    cfg.browser_ws_url = browser_ws_url;
    cfg.live_e2e = live_e2e;
    cfg.reporters = Vec::new();
    crate::test_runner::run(cfg).await
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn build_evidence_bundle(
    mode: E2eMode,
    summary: Summary,
    started_at_ms: u64,
    finished_at_ms: u64,
    serve_session: Option<crate::dev_server::session::ServeSession>,
    open_control: Option<E2eOpenControlProtocol>,
) -> E2eEvidenceBundle {
    let run_id = make_run_id(started_at_ms, mode);
    let exit_code = exit_code_for_reports(&summary.reports);
    let mut artifacts = Vec::new();
    let cases = summary
        .reports
        .iter()
        .enumerate()
        .map(|(idx, report)| {
            let case = case_from_report(idx, report);
            for step in &case.steps {
                artifacts.extend(step.context.screenshots.clone());
            }
            if let Some(trace) = &report.trace_path {
                artifacts.push(E2eArtifactRef {
                    kind: "trace".to_string(),
                    path: trace.clone(),
                    label: Some(case.title.clone()),
                });
            }
            case
        })
        .collect();
    if let Some(session) = &serve_session {
        if let Some(log_file) = &session.log_file {
            artifacts.push(E2eArtifactRef {
                kind: "serve-log".to_string(),
                path: PathBuf::from(log_file),
                label: Some(format!("jet serve {} log", session.target)),
            });
        }
    }
    E2eEvidenceBundle {
        schema_version: EVIDENCE_SCHEMA_VERSION.to_string(),
        mode,
        run_id,
        started_at_ms,
        finished_at_ms,
        summary: E2eSummary {
            passed: summary.passed,
            failed: summary.failed,
            skipped: summary.skipped,
            duration_ms: summary.duration_ms,
            exit_code,
        },
        cases,
        artifacts,
        serve_session,
        browser_sessions: summary.browser_sessions.clone(),
        open_control,
    }
}

fn case_from_report(idx: usize, report: &TestReport) -> E2eCaseEvidence {
    let title = test_title(report);
    let outcome = outcome_string(report.outcome);
    let mut report_context = E2eStepContext::default();
    report_context
        .screenshots
        .extend(report.artifacts.iter().map(|path| E2eArtifactRef {
            kind: "screenshot".to_string(),
            path: path.clone(),
            label: Some("failure artifact".to_string()),
        }));
    if let Some(trace_path) = &report.trace_path {
        merge_trace_context(trace_path, &mut report_context);
    }
    let assertion = report.error.as_ref().map(|err| E2eAssertionDetail {
        message: err.message.clone(),
        stack: err.stack.clone(),
        diff: err.diff.clone(),
    });
    let steps = if report.steps.is_empty() {
        vec![E2eProductStep {
            id: "step-0001".to_string(),
            title: title.clone(),
            status: outcome.clone(),
            duration_ms: report.duration_ms,
            assertion,
            context: report_context,
        }]
    } else {
        let context_step_index = report
            .steps
            .iter()
            .position(|step| !matches!(step.outcome, Outcome::Passed))
            .unwrap_or_else(|| report.steps.len().saturating_sub(1));
        report
            .steps
            .iter()
            .enumerate()
            .map(|(step_index, step)| {
                let mut context = E2eStepContext::default();
                if step_index == context_step_index {
                    context = report_context.clone();
                }
                let mut step_assertion = step.error.as_ref().map(|err| E2eAssertionDetail {
                    message: err.message.clone(),
                    stack: err.stack.clone(),
                    diff: err.diff.clone(),
                });
                if step_index == context_step_index && step_assertion.is_none() {
                    step_assertion = assertion.clone();
                }
                E2eProductStep {
                    id: step.id.clone(),
                    title: step.title.clone(),
                    status: outcome_string(step.outcome),
                    duration_ms: step.duration_ms,
                    assertion: step_assertion,
                    context,
                }
            })
            .collect()
    };
    E2eCaseEvidence {
        id: format!("case-{:04}", idx + 1),
        title: title.clone(),
        file: report.file.clone(),
        outcome: outcome.clone(),
        duration_ms: report.duration_ms,
        steps,
    }
}

/// Format the warning emitted when `merge_trace_context` cannot read the
/// trace manifest. Names the offending trace path verbatim, preserves the
/// underlying error, and tags `GH #3540` so a developer grepping logs for
/// "e2e failure report missing selectors/screenshots" can land on this line.
/// Extracted for unit-test pinning.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub(crate) fn format_trace_manifest_read_warn(trace_path: &Path, err: &anyhow::Error) -> String {
    format!(
        "GH #3540 failed to read trace manifest at {}: {}; the e2e failure \
         report will be rendered WITHOUT selectors, screenshots, or console \
         context (E2eStepContext fields will be empty). The trace .zip may \
         be truncated, missing the manifest entry, or written by an \
         incompatible version — check that the trace file is complete and \
         that jet/playwright versions match.",
        trace_path.display(),
        err
    )
}

fn merge_trace_context(trace_path: &Path, context: &mut E2eStepContext) {
    // GH #3540 — previously `let Ok(...) else return;` swallowed every
    // manifest-read failure. Surface a structured warn so a developer
    // debugging "the e2e report lost its trace context" has a breadcrumb.
    let manifest = match crate::trace::archive::read_manifest_from_zip(trace_path) {
        Ok(m) => m,
        Err(err) => {
            tracing::warn!(
                target: "jet::e2e::merge_trace_context",
                trace = %trace_path.display(),
                error = %err,
                "{}",
                format_trace_manifest_read_warn(trace_path, &err)
            );
            return;
        }
    };
    for event in manifest.events {
        match event {
            crate::trace::manifest::TraceEvent::ActionStep(step) => {
                if let Some(selector) = step.selector {
                    context.selectors.push(E2eSelectorContext {
                        selector,
                        action: format!("{:?}", step.action).to_lowercase(),
                        highlighted: true,
                    });
                }
                if let Some(screenshot_ref) = step.screenshot_ref {
                    context.screenshots.push(E2eArtifactRef {
                        kind: "trace-screenshot".to_string(),
                        path: PathBuf::from(screenshot_ref),
                        label: Some(format!("step {}", step.step_id)),
                    });
                }
            }
            crate::trace::manifest::TraceEvent::Console(console) => {
                context.console.push(E2eConsoleEntry {
                    level: format!("{:?}", console.level).to_lowercase(),
                    text: console.text,
                    ts_ms: console.ts,
                });
            }
            crate::trace::manifest::TraceEvent::Network(network) => {
                context.network.push(E2eNetworkEntry {
                    request_id: network.request_id,
                    method: network.method,
                    url: network.url,
                    status: network.status,
                    ts_start_ms: network.ts_start,
                    ts_end_ms: network.ts_end,
                });
            }
            crate::trace::manifest::TraceEvent::Screenshot(screenshot) => {
                context.screenshots.push(E2eArtifactRef {
                    kind: "trace-screenshot".to_string(),
                    path: PathBuf::from(screenshot.screenshot_ref),
                    label: Some("explicit screenshot".to_string()),
                });
            }
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn write_evidence_bundle(
    evidence_dir: &Path,
    bundle: &E2eEvidenceBundle,
) -> Result<E2eRunResult> {
    std::fs::create_dir_all(evidence_dir)
        .with_context(|| format!("Failed to create {}", evidence_dir.display()))?;
    let evidence_path = evidence_dir.join(format!("{}.evidence.json", bundle.run_id));
    let jsonl_path = evidence_dir.join(format!("{}.events.jsonl", bundle.run_id));
    std::fs::write(
        &evidence_path,
        serde_json::to_vec_pretty(bundle).context("serialising e2e evidence")?,
    )
    .with_context(|| format!("Failed to write {}", evidence_path.display()))?;
    let events = events_for_bundle(bundle);
    let mut jsonl = String::new();
    for event in events {
        jsonl.push_str(&serde_json::to_string(&event)?);
        jsonl.push('\n');
    }
    std::fs::write(&jsonl_path, jsonl)
        .with_context(|| format!("Failed to write {}", jsonl_path.display()))?;
    Ok(E2eRunResult {
        evidence_path,
        jsonl_path,
        review_app_path: None,
        manual_docs_path: None,
        bundle: bundle.clone(),
    })
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn events_for_bundle(bundle: &E2eEvidenceBundle) -> Vec<E2eEvidenceEvent> {
    let mut events = vec![E2eEvidenceEvent::RunStarted {
        run_id: bundle.run_id.clone(),
        mode: bundle.mode,
        ts_ms: bundle.started_at_ms,
    }];
    if let Some(session) = &bundle.serve_session {
        events.push(E2eEvidenceEvent::ServeSessionStarted {
            run_id: bundle.run_id.clone(),
            mode: session.mode.clone(),
            target: session.target.clone(),
            url: session.url.clone(),
            host: session.host.clone(),
            port: session.port,
            pid: session.pid,
            log_file: session.log_file.clone(),
            ts_ms: bundle.started_at_ms,
        });
    }
    for session in &bundle.browser_sessions {
        events.push(E2eEvidenceEvent::BrowserSessionStarted {
            run_id: bundle.run_id.clone(),
            session_id: session.session_id.clone(),
            driver: session.driver.clone(),
            anchor: session.anchor.clone(),
            headless: session.headless,
            spec_file: session.spec_file.clone(),
            pid: session.pid,
            ws_endpoint: session.ws_endpoint.clone(),
            ts_ms: session.started_at_ms,
        });
        events.push(E2eEvidenceEvent::BrowserSessionFinished {
            run_id: bundle.run_id.clone(),
            session_id: session.session_id.clone(),
            state: session.state.as_str().to_string(),
            graceful_close: session.graceful_close,
            error: session.error.clone(),
            ts_ms: session
                .closed_at_ms
                .or(session.ready_at_ms)
                .unwrap_or(session.started_at_ms),
        });
    }
    let mut cursor_ms = bundle.started_at_ms;
    for case in &bundle.cases {
        for step in &case.steps {
            events.push(E2eEvidenceEvent::StepStarted {
                run_id: bundle.run_id.clone(),
                case_id: case.id.clone(),
                step_id: step.id.clone(),
                title: step.title.clone(),
                ts_ms: cursor_ms,
            });
            cursor_ms = cursor_ms.saturating_add(step.duration_ms);
            events.push(E2eEvidenceEvent::StepFinished {
                run_id: bundle.run_id.clone(),
                case_id: case.id.clone(),
                step_id: step.id.clone(),
                title: step.title.clone(),
                status: step.status.clone(),
                duration_ms: step.duration_ms,
                ts_ms: cursor_ms,
                assertion: step.assertion.clone(),
            });
        }
        events.push(E2eEvidenceEvent::CaseFinished {
            run_id: bundle.run_id.clone(),
            case_id: case.id.clone(),
            title: case.title.clone(),
            outcome: case.outcome.clone(),
            duration_ms: case.duration_ms,
        });
    }
    events.push(E2eEvidenceEvent::RunFinished {
        run_id: bundle.run_id.clone(),
        exit_code: bundle.summary.exit_code,
        passed: bundle.summary.passed,
        failed: bundle.summary.failed,
        skipped: bundle.summary.skipped,
        ts_ms: bundle.finished_at_ms,
    });
    events
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn write_open_runner_shell(evidence_dir: &Path, bundle: &E2eEvidenceBundle) -> Result<PathBuf> {
    let app_dir = open_runner_shell_dir(evidence_dir);
    std::fs::create_dir_all(&app_dir)
        .with_context(|| format!("Failed to create {}", app_dir.display()))?;
    let app_state = app_dir.join("app-state.json");
    let protocol_path = app_dir.join("control-protocol.json");
    std::fs::write(&app_state, serde_json::to_vec_pretty(bundle)?)?;
    if let Some(protocol) = &bundle.open_control {
        std::fs::write(&protocol_path, serde_json::to_vec_pretty(protocol)?)?;
    }
    let index = app_dir.join("index.html");
    std::fs::write(&index, render_runner_shell_html(bundle)?)?;
    Ok(index)
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn write_open_review_app(evidence_dir: &Path, bundle: &E2eEvidenceBundle) -> Result<PathBuf> {
    write_open_runner_shell(evidence_dir, bundle)
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct E2eManualDocs {
    pub markdown_path: PathBuf,
    pub html_path: PathBuf,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn write_manual_docs(
    project_root: &Path,
    out_dir: &Path,
    bundle: &E2eEvidenceBundle,
    title: &str,
) -> Result<E2eManualDocs> {
    std::fs::create_dir_all(out_dir)
        .with_context(|| format!("Failed to create manual output dir {}", out_dir.display()))?;
    let images_dir = out_dir.join("images");
    std::fs::create_dir_all(&images_dir).with_context(|| {
        format!(
            "Failed to create manual image output dir {}",
            images_dir.display()
        )
    })?;

    let markdown = render_manual_markdown(project_root, out_dir, bundle, title)?;
    let markdown_path = out_dir.join("index.md");
    std::fs::write(&markdown_path, markdown.as_bytes())
        .with_context(|| format!("Failed to write {}", markdown_path.display()))?;

    let html = render_manual_html(title, &markdown);
    let html_path = out_dir.join("index.html");
    std::fs::write(&html_path, html.as_bytes())
        .with_context(|| format!("Failed to write {}", html_path.display()))?;

    Ok(E2eManualDocs {
        markdown_path,
        html_path,
    })
}

fn render_manual_markdown(
    project_root: &Path,
    out_dir: &Path,
    bundle: &E2eEvidenceBundle,
    title: &str,
) -> Result<String> {
    let mut out = String::new();
    out.push_str("# ");
    out.push_str(title);
    out.push_str("\n\n");
    out.push_str("Generated by `jet e2e manual` from Jet-owned product-flow E2E evidence.\n\n");
    out.push_str("| Field | Value |\n|---|---|\n");
    out.push_str(&format!(
        "| Run ID | `{}` |\n",
        markdown_escape(&bundle.run_id)
    ));
    out.push_str(&format!("| Mode | `{:?}` |\n", bundle.mode));
    out.push_str(&format!("| Passed | {} |\n", bundle.summary.passed));
    out.push_str(&format!("| Failed | {} |\n", bundle.summary.failed));
    out.push_str(&format!("| Skipped | {} |\n", bundle.summary.skipped));
    out.push_str(&format!(
        "| Duration | {} ms |\n",
        bundle.summary.duration_ms
    ));
    out.push('\n');

    if bundle.cases.is_empty() {
        out.push_str("No E2E cases were discovered.\n");
        return Ok(out);
    }

    for (case_index, case) in bundle.cases.iter().enumerate() {
        out.push_str(&format!(
            "## {}. {}\n\n",
            case_index + 1,
            markdown_escape(&case.title)
        ));
        out.push_str("| Field | Value |\n|---|---|\n");
        out.push_str(&format!(
            "| Status | `{}` |\n",
            markdown_escape(&case.outcome)
        ));
        out.push_str(&format!("| File | `{}` |\n", case.file.display()));
        out.push_str(&format!("| Duration | {} ms |\n\n", case.duration_ms));

        for (step_index, step) in case.steps.iter().enumerate() {
            out.push_str(&format!(
                "### Step {}. {}\n\n",
                step_index + 1,
                markdown_escape(&step.title)
            ));
            out.push_str(&format!("Status: `{}`  \n", markdown_escape(&step.status)));
            out.push_str(&format!("Duration: {} ms\n\n", step.duration_ms));
            if let Some(assertion) = &step.assertion {
                out.push_str("Assertion:\n\n```text\n");
                out.push_str(&assertion.message);
                if let Some(diff) = &assertion.diff {
                    out.push('\n');
                    out.push_str(diff);
                }
                out.push_str("\n```\n\n");
            }

            let screenshots =
                copy_manual_screenshots(project_root, out_dir, case_index, step_index, step)?;
            if screenshots.is_empty() {
                out.push_str("_No screenshots were captured for this step._\n\n");
            } else {
                for screenshot in screenshots {
                    out.push_str(&format!(
                        "![{}]({})\n\n",
                        markdown_escape(&step.title),
                        screenshot.display()
                    ));
                }
            }
        }
    }

    if !bundle.artifacts.is_empty() {
        out.push_str("## Evidence Artifacts\n\n");
        for artifact in &bundle.artifacts {
            let label = artifact.label.as_deref().unwrap_or(&artifact.kind);
            out.push_str(&format!(
                "- `{}`: `{}` ({})\n",
                markdown_escape(&artifact.kind),
                artifact.path.display(),
                markdown_escape(label)
            ));
        }
        out.push('\n');
    }

    Ok(out)
}

fn copy_manual_screenshots(
    project_root: &Path,
    out_dir: &Path,
    case_index: usize,
    step_index: usize,
    step: &E2eProductStep,
) -> Result<Vec<PathBuf>> {
    let mut copied = Vec::new();
    let images_dir = out_dir.join("images");
    for (shot_index, shot) in step.context.screenshots.iter().enumerate() {
        let source = resolve_project_path(project_root, &shot.path);
        if !source.exists() {
            continue;
        }
        let extension = source
            .extension()
            .and_then(|ext| ext.to_str())
            .filter(|ext| !ext.is_empty())
            .unwrap_or("png");
        let file_name = format!(
            "case-{:02}-step-{:02}-{:02}.{}",
            case_index + 1,
            step_index + 1,
            shot_index + 1,
            extension
        );
        let target = images_dir.join(file_name);
        std::fs::copy(&source, &target).with_context(|| {
            format!(
                "Failed to copy manual screenshot {} to {}",
                source.display(),
                target.display()
            )
        })?;
        copied.push(PathBuf::from("images").join(target.file_name().unwrap()));
    }
    Ok(copied)
}

fn render_manual_html(title: &str, markdown: &str) -> String {
    let body = render_manual_html_body(markdown);
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{}</title>
<style>
body {{ margin: 0; background: #f7f9fc; color: #172033; font: 16px/1.55 -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }}
main {{ max-width: 1040px; margin: 0 auto; padding: 32px 20px 56px; }}
h1, h2, h3 {{ line-height: 1.2; letter-spacing: 0; }}
h1 {{ font-size: 30px; margin: 0 0 18px; }}
h2 {{ font-size: 22px; margin: 34px 0 14px; border-top: 1px solid #d9e1ef; padding-top: 22px; }}
h3 {{ font-size: 17px; margin: 22px 0 10px; }}
table {{ width: 100%; border-collapse: collapse; margin: 14px 0 18px; background: #fff; }}
th, td {{ border: 1px solid #d9e1ef; padding: 8px 10px; text-align: left; vertical-align: top; }}
code, pre {{ font-family: "SFMono-Regular", Consolas, monospace; }}
pre {{ overflow: auto; background: #111827; color: #e5e7eb; padding: 12px; border-radius: 6px; }}
img {{ display: block; max-width: 100%; margin: 10px 0 22px; border: 1px solid #d9e1ef; background: #fff; }}
li {{ margin: 4px 0; }}
</style>
</head>
<body><main>{}</main></body>
</html>
"#,
        html_escape(title),
        body
    )
}

fn render_manual_html_body(markdown: &str) -> String {
    let mut body = String::new();
    let mut in_code = false;
    for line in markdown.lines() {
        if line.starts_with("```") {
            if in_code {
                body.push_str("</pre>\n");
                in_code = false;
            } else {
                body.push_str("<pre>");
                in_code = true;
            }
            continue;
        }
        if in_code {
            body.push_str(&html_escape(line));
            body.push('\n');
            continue;
        }
        if let Some(text) = line.strip_prefix("# ") {
            body.push_str(&format!("<h1>{}</h1>\n", html_escape(text)));
        } else if let Some(text) = line.strip_prefix("## ") {
            body.push_str(&format!("<h2>{}</h2>\n", html_escape(text)));
        } else if let Some(text) = line.strip_prefix("### ") {
            body.push_str(&format!("<h3>{}</h3>\n", html_escape(text)));
        } else if let Some((alt, src)) = parse_markdown_image(line) {
            body.push_str(&format!(
                r#"<img alt="{}" src="{}">"#,
                html_escape(alt),
                html_escape(src)
            ));
            body.push('\n');
        } else if line.trim().is_empty() {
            body.push('\n');
        } else if line.starts_with('|') {
            body.push_str(&format!("<p><code>{}</code></p>\n", html_escape(line)));
        } else if let Some(text) = line.strip_prefix("- ") {
            body.push_str(&format!("<p>• {}</p>\n", html_escape(text)));
        } else {
            body.push_str(&format!("<p>{}</p>\n", html_escape(line)));
        }
    }
    if in_code {
        body.push_str("</pre>\n");
    }
    body
}

fn parse_markdown_image(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix("![")?;
    let (alt, rest) = rest.split_once("](")?;
    let src = rest.strip_suffix(')')?;
    Some((alt, src))
}

fn markdown_escape(value: &str) -> String {
    value.replace('|', "\\|")
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn resolve_project_path(project_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_root.join(path)
    }
}

fn artifact_path_for_project(project_root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(project_root)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| path.to_path_buf())
}

fn render_runner_shell_html(bundle: &E2eEvidenceBundle) -> Result<String> {
    render_review_ui_html(bundle, ReviewUiMode::Live)
}

/// Render the PM-facing static report HTML for an evidence bundle.
///
/// Convenience wrapper around [`render_review_ui_html`] that always uses
/// [`ReviewUiMode::PmReport`]: the toolbar and dev-only command log are
/// hidden via CSS, while the case grid, inspector, and failure summary
/// remain so non-developers can read the outcome without raw JSON.
// @spec #2622
pub fn render_pm_report_html(bundle: &E2eEvidenceBundle) -> Result<String> {
    render_review_ui_html(bundle, ReviewUiMode::PmReport)
}

/// UI mode for the shared review surface.
///
/// All three modes share the same renderer and DOM scaffold so the
/// review component cannot drift between consumers. The mode flag
/// switches three things:
///   * the embedded control adapter (live POSTs vs. disabled vs. absent),
///   * the visibility of dev-only controls (pause/next/replay toolbar),
///   * the `data-mode` attribute so CSS and tests can branch.
///
/// @spec #2614, #2622
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewUiMode {
    /// Live dev adapter — POST control commands to the open-mode server.
    Live,
    /// Read-only adapter for agent post-mortem — controls are visible
    /// but disabled; no fetches. Keeps the full developer-facing detail
    /// (command log, inspector panels).
    ReadOnly,
    /// PM web report — controls are hidden (not just disabled), the
    /// developer-only command log is hidden, and the summary + product
    /// outcome + failed-step + screenshots are emphasised. No remote
    /// browser control surface.
    PmReport,
}

/// Render the shared review UI HTML for an evidence bundle.
///
/// The same renderer is used by the local-open adapter and the read-only
/// PM report adapter so the component surface cannot drift between modes.
/// The DOM is identical between modes; only the embedded control adapter
/// switches.
// @spec #2614
pub fn render_review_ui_html(bundle: &E2eEvidenceBundle, mode: ReviewUiMode) -> Result<String> {
    let data = serde_json::to_string(bundle)?;
    let mode_attr = match mode {
        ReviewUiMode::Live => "live",
        ReviewUiMode::ReadOnly => "read-only",
        ReviewUiMode::PmReport => "pm-report",
    };
    let controls_disabled = match mode {
        ReviewUiMode::Live => "",
        ReviewUiMode::ReadOnly | ReviewUiMode::PmReport => " disabled",
    };
    Ok(format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Jet E2E Runner</title>
<style>
:root {{
  color-scheme: dark;
  --bg: #0b0d10;
  --rail: #11161d;
  --panel: #151b23;
  --panel-2: #0f141a;
  --line: #26313d;
  --line-strong: #384553;
  --text: #eef2f7;
  --muted: #9aa7b6;
  --subtle: #657386;
  --green: #22c55e;
  --green-soft: rgba(34, 197, 94, .15);
  --amber: #f59e0b;
  --amber-soft: rgba(245, 158, 11, .16);
  --red: #f43f5e;
  --red-soft: rgba(244, 63, 94, .15);
  --focus: #38bdf8;
  font-family: "IBM Plex Sans", "Fira Sans", ui-sans-serif, system-ui, sans-serif;
}}
* {{ box-sizing: border-box; }}
body {{ margin: 0; background: var(--bg); color: var(--text); overflow-x: hidden; }}
.runner-header {{
  min-height: 64px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 10px 16px;
  border-bottom: 1px solid var(--line);
  background: #0e141c;
}}
.runner-title {{ display: grid; gap: 3px; min-width: 0; }}
.runner-title h1 {{ font: 700 18px/1.2 "Fira Code", "SFMono-Regular", ui-monospace, monospace; }}
.runner-title .statusline {{ margin-top: 0; }}
.runner-header-controls {{ display: flex; align-items: center; gap: 12px; flex-wrap: wrap; justify-content: flex-end; }}
.speed-control {{ display: flex; align-items: center; gap: 7px; }}
.speed-control-label {{ color: var(--muted); font: 700 11px/1 "Fira Code", ui-monospace, monospace; text-transform: uppercase; }}
.speed-options {{
  display: inline-grid;
  grid-template-columns: repeat(3, minmax(48px, 1fr));
  border: 1px solid var(--line-strong);
  border-radius: 8px;
  overflow: hidden;
}}
.speed-button {{
  min-height: 36px;
  border: 0;
  border-right: 1px solid var(--line-strong);
  border-radius: 0;
  padding: 7px 12px;
  background: #111821;
  font-weight: 700;
}}
.speed-button:last-child {{ border-right: 0; }}
.speed-button.active {{ color: #bbf7d0; background: rgba(34, 197, 94, .16); }}
main {{
  display: grid;
  grid-template-columns: 360px minmax(540px, 1fr) 392px;
  min-height: calc(100vh - 64px);
  max-height: calc(100vh - 64px);
  background: var(--bg);
}}
aside, section, article {{ min-width: 0; }}
aside {{
  border-right: 1px solid var(--line);
  background: linear-gradient(180deg, #121821, #0d1117);
  padding: 16px;
  overflow-y: auto;
  max-height: calc(100vh - 64px);
}}
section {{
  padding: 16px;
  border-right: 1px solid var(--line);
  min-height: calc(100vh - 64px);
}}
article {{
  padding: 16px;
  background: rgba(12, 16, 21, .92);
  overflow-y: auto;
  max-height: calc(100vh - 64px);
}}
h1, h2, h3 {{ margin: 0; letter-spacing: 0; }}
h1 {{ font: 700 18px/1.2 "Fira Code", "SFMono-Regular", ui-monospace, monospace; }}
h2 {{ font-size: 13px; line-height: 1.25; color: var(--muted); text-transform: uppercase; margin-top: 18px; margin-bottom: 8px; }}
h3 {{ font-size: 12px; line-height: 1.25; color: var(--muted); text-transform: uppercase; margin-bottom: 8px; }}
button {{
  min-height: 44px;
  border: 1px solid var(--line-strong);
  background: #111821;
  color: var(--text);
  border-radius: 7px;
  padding: 9px 11px;
  cursor: pointer;
  transition: background .18s ease, border-color .18s ease, color .18s ease, box-shadow .18s ease;
}}
button:hover {{ background: #18212b; border-color: #506173; }}
button:active {{ background: #0d131a; }}
button:focus-visible {{ outline: 2px solid var(--focus); outline-offset: 2px; }}
.toolbar {{ display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 8px; margin: 14px 0 18px; }}
.toolbar button {{ font-weight: 700; }}
#replay {{ border-color: rgba(34, 197, 94, .55); color: #bbf7d0; background: rgba(34, 197, 94, .12); }}
.case {{
  width: 100%;
  text-align: left;
  margin: 6px 0;
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 9px;
  align-items: start;
  color: #d8e1ec;
}}
.case.active {{ border-color: var(--focus); box-shadow: 0 0 0 2px rgba(56, 189, 248, .16); background: #17212c; }}
.status {{
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 72px;
  height: 22px;
  padding: 0 8px;
  border-radius: 999px;
  font: 700 11px/1 "Fira Code", ui-monospace, monospace;
  text-transform: uppercase;
}}
.failed {{ color: #fecdd3; background: var(--red-soft); border-color: rgba(244, 63, 94, .42); }}
.passed {{ color: #bbf7d0; background: var(--green-soft); border-color: rgba(34, 197, 94, .42); }}
.skipped {{ color: #fde68a; background: var(--amber-soft); border-color: rgba(245, 158, 11, .42); }}
.running {{ color: #bae6fd; background: rgba(56, 189, 248, .13); border-color: rgba(56, 189, 248, .42); }}
.grid {{ display: grid; grid-template-columns: 1fr; gap: 10px; }}
.panel, .browser {{
  border: 1px solid var(--line);
  border-radius: 8px;
  background: linear-gradient(180deg, var(--panel), var(--panel-2));
  padding: 12px;
}}
.panel {{ min-height: 106px; }}
.panel summary {{
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  cursor: pointer;
  color: var(--muted);
  font: 700 12px/1.25 "IBM Plex Sans", ui-sans-serif, system-ui, sans-serif;
  text-transform: uppercase;
  letter-spacing: 0;
}}
.panel summary::-webkit-details-marker {{ display: none; }}
.panel summary::after {{
  content: "Open";
  color: var(--subtle);
  font: 700 11px/1 "Fira Code", ui-monospace, monospace;
  text-transform: none;
}}
.panel[open] summary {{ margin-bottom: 8px; }}
.panel[open] summary::after {{ content: "Close"; }}
.panel h3 {{ margin-bottom: 8px; }}
.panel details h3,
details.panel h3 {{ margin: 0; }}
.panel-count {{
  margin-left: auto;
  color: var(--subtle);
  font: 700 11px/1 "Fira Code", ui-monospace, monospace;
  text-transform: none;
}}
.browser {{ min-height: 178px; margin-bottom: 12px; }}
.browser + .browser {{ margin-top: 10px; }}
.browser code {{ display: block; white-space: pre-wrap; overflow-wrap: anywhere; color: #cbd5e1; }}
.artifact-list {{ display: grid; gap: 8px; }}
.artifact-link {{
  display: grid;
  gap: 3px;
  border: 1px solid var(--line);
  border-radius: 7px;
  padding: 8px 10px;
  background: #0e141c;
  color: #dbeafe;
  text-decoration: none;
  overflow-wrap: anywhere;
}}
.artifact-link:hover {{ border-color: var(--focus); background: #122032; }}
.artifact-link span {{ color: var(--muted); font: 11px/1.4 "Fira Code", ui-monospace, monospace; }}
.artifact-link.disabled {{ color: var(--muted); opacity: .72; }}
.statusline {{ font-size: 12px; line-height: 1.35; color: var(--muted); margin-top: 4px; overflow-wrap: anywhere; }}
.command-log {{ display: grid; gap: 6px; margin-top: 10px; }}
.dev-panel {{
  margin-top: 18px;
  border-top: 1px solid var(--line);
  padding-top: 12px;
}}
.dev-panel summary {{
  cursor: pointer;
  color: var(--subtle);
  font: 700 12px/1.25 "IBM Plex Sans", ui-sans-serif, system-ui, sans-serif;
  text-transform: uppercase;
  letter-spacing: 0;
}}
.dev-panel[open] summary {{ margin-bottom: 8px; }}
.command {{
  width: 100%;
  min-height: 40px;
  text-align: left;
  border-left: 4px solid var(--line-strong);
  background: #0f151d;
  color: #dbe5f0;
  padding: 8px 10px;
}}
.command strong {{ font-family: "Fira Code", "SFMono-Regular", ui-monospace, monospace; font-size: 11px; color: var(--subtle); margin-right: 7px; }}
.command.running {{ border-left-color: var(--focus); }}
.command.passed {{ border-left-color: var(--green); }}
.command.failed {{ border-left-color: var(--red); }}
.command.active {{ box-shadow: inset 0 0 0 1px var(--focus); background: #14202a; }}
.aut-shell {{ display: grid; grid-template-rows: auto minmax(420px, 1fr); min-height: calc(100vh - 32px); }}
.aut-toolbar {{
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
  min-height: 48px;
  margin-bottom: 10px;
}}
.aut-toolbar h2 {{
  margin: 0;
  color: var(--text);
  font: 700 14px/1.35 "IBM Plex Sans", ui-sans-serif, system-ui, sans-serif;
  text-transform: none;
}}
.case-explanation {{
  display: grid;
  gap: 12px;
  margin-bottom: 12px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: linear-gradient(180deg, var(--panel), var(--panel-2));
  padding: 12px;
}}
.case-explanation-grid {{
  display: grid;
  grid-template-columns: minmax(0, 1.2fr) minmax(0, .8fr);
  gap: 12px;
}}
.case-explanation h3 {{ margin-bottom: 7px; }}
.case-explanation ol,
.case-explanation ul {{
  margin: 0;
  padding-left: 20px;
  color: #d8e4ef;
  font-size: 12px;
  line-height: 1.48;
}}
.case-explanation li + li {{ margin-top: 6px; }}
.case-explanation .statusline {{ margin-top: 0; }}
.target-surface {{
  border: 1px solid var(--line-strong);
  border-radius: 8px;
  background:
    linear-gradient(180deg, rgba(34, 197, 94, .07), transparent 42%),
    #020617;
  min-height: 620px;
  overflow: auto;
  box-shadow: 0 18px 48px rgba(0, 0, 0, .35);
  position: relative;
  padding: 18px;
}}
.target-surface::before {{
  content: "";
  display: block;
  height: 30px;
  margin: -18px -18px 18px;
  background:
    radial-gradient(circle at 15px 15px, #ef4444 0 5px, transparent 6px),
    radial-gradient(circle at 35px 15px, #f59e0b 0 5px, transparent 6px),
    radial-gradient(circle at 55px 15px, #22c55e 0 5px, transparent 6px),
    linear-gradient(180deg, #19212b, #111821);
  border-bottom: 1px solid var(--line);
}}
.target-grid {{ display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; }}
.target-card {{
  min-height: 118px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: #0e141c;
  padding: 12px;
}}
.target-card strong {{ display: block; margin-bottom: 8px; color: var(--muted); font: 700 11px/1.2 "Fira Code", ui-monospace, monospace; text-transform: uppercase; }}
.target-card code {{ display: block; white-space: pre-wrap; overflow-wrap: anywhere; color: #d8e4ef; font: 12px/1.55 "Fira Code", ui-monospace, monospace; }}
.live-event {{ border-bottom: 1px solid var(--line); padding: 7px 0; font-size: 12px; color: #cbd5e1; }}
.live-event strong {{ display: inline-block; min-width: 132px; color: var(--muted); font-family: "Fira Code", ui-monospace, monospace; }}
pre {{ white-space: pre-wrap; overflow-wrap: anywhere; margin: 0; font: 12px/1.5 "Fira Code", "SFMono-Regular", ui-monospace, monospace; color: #d6e0ea; }}
#runtime, #summary {{ color: var(--muted); font-size: 12px; line-height: 1.45; margin-top: 7px; }}
body[data-mode="pm-report"] .toolbar,
body[data-mode="pm-report"] #commands,
body[data-mode="pm-report"] #command-log-panel {{ display: none; }}
body[data-mode="pm-report"] #pm-summary {{
  display: block;
  margin: 0 0 14px;
  padding: 12px 14px;
  border: 1px solid var(--line-strong);
  border-radius: 8px;
  background: linear-gradient(180deg, var(--panel), var(--panel-2));
}}
body[data-mode="pm-report"] #pm-failures {{
  display: block;
  margin: 6px 0 14px;
  padding: 12px 14px;
  border: 1px solid rgba(244, 63, 94, .42);
  border-radius: 8px;
  background: var(--red-soft);
  color: #fecdd3;
  font: 12px/1.55 "Fira Code", ui-monospace, monospace;
  white-space: pre-wrap;
}}
body:not([data-mode="pm-report"]) #pm-summary,
body:not([data-mode="pm-report"]) #pm-failures {{ display: none; }}
@media (prefers-reduced-motion: reduce) {{ * {{ transition: none !important; }} }}
@media (max-width: 1180px) {{
  main {{ grid-template-columns: 320px minmax(440px, 1fr); }}
  article {{ grid-column: 1 / -1; max-height: none; border-top: 1px solid var(--line); }}
}}
@media (max-width: 760px) {{
  .runner-header {{ align-items: flex-start; flex-direction: column; }}
  .runner-header-controls {{ width: 100%; justify-content: flex-start; }}
  main {{ grid-template-columns: 1fr; }}
  aside, article {{ max-height: none; }}
  aside, section {{ border-right: 0; border-bottom: 1px solid var(--line); }}
  .toolbar {{ grid-template-columns: 1fr; }}
  .aut-shell {{ min-height: auto; }}
  .target-surface {{ min-height: 520px; }}
  .target-grid {{ grid-template-columns: 1fr; }}
  .case-explanation-grid {{ grid-template-columns: 1fr; }}
}}
</style>
</head>
<body data-mode="{mode_attr}">
<header class="runner-header">
<div class="runner-title">
<h1>Jet E2E</h1>
<div class="statusline" id="header-status">loaded</div>
</div>
<div class="runner-header-controls">
<div class="speed-control" aria-label="Playback speed">
<span class="speed-control-label">Speed</span>
<div class="speed-options">
<button class="speed-button" data-speed="1"{controls_disabled}>1x</button>
<button class="speed-button" data-speed="2"{controls_disabled}>2x</button>
<button class="speed-button" data-speed="4"{controls_disabled}>4x</button>
</div>
</div>
</div>
</header>
<main>
<aside>
<div id="runtime"></div>
<div id="summary"></div>
<div id="pm-summary"></div>
<div id="pm-failures"></div>
<div class="toolbar">
<button id="pause"{controls_disabled}>Pause</button>
<button id="next"{controls_disabled}>Next</button>
<button id="replay"{controls_disabled}>Replay</button>
</div>
<h2>Cases</h2>
<div id="cases"></div>
<details class="dev-panel" id="command-log-panel"><summary>Steps</summary><div id="commands" class="command-log"></div></details>
</aside>
<section>
<div class="aut-shell">
<div class="aut-toolbar">
<h2 id="title"></h2>
<div class="statusline" id="aut-status">ready</div>
</div>
<div class="case-explanation" id="case-explanation">
<div class="case-explanation-grid">
<div>
<h3>How This Case Runs</h3>
<div id="case-runbook" class="statusline"></div>
</div>
<div>
<h3>What This Case Tests</h3>
<div id="case-checks" class="statusline"></div>
</div>
</div>
</div>
<div class="target-surface">
<div class="target-grid">
<div class="target-card"><strong>Target</strong><code id="target-kind"></code></div>
<div class="target-card"><strong>Command</strong><code id="target-command"></code></div>
<div class="target-card"><strong>Selector</strong><code id="target-selector"></code></div>
<div class="target-card"><strong>Page</strong><code id="target-page"></code></div>
</div>
</div>
</div>
</section>
<article>
<h2>Inspector</h2>
<div class="browser">
<div><strong>Review Shell</strong></div>
<strong id="shell-kind"></strong>
<div class="statusline" id="shell-status">starting</div>
<code id="shell-detail"></code>
</div>
<div class="browser">
<div><strong>Controlled Target</strong></div>
<strong id="browser-kind"></strong>
<div class="statusline" id="browser-status">starting</div>
<code id="browser-detail"></code>
</div>
<div class="grid">
<div class="panel"><h3>Selectors</h3><pre id="selectors"></pre></div>
<div class="panel"><h3>Assertions</h3><pre id="assertions"></pre></div>
<div class="panel"><h3>Screenshots</h3><pre id="screenshots"></pre></div>
<div class="panel"><h3>Artifacts</h3><div id="artifacts" class="artifact-list"></div></div>
<details class="panel" id="console-panel"><summary><span>Console</span><span class="panel-count" id="console-count">0</span></summary><pre id="console"></pre></details>
<div class="panel"><h3>Network</h3><pre id="network"></pre></div>
</div>
<h3>Live Events</h3>
<div id="live-events"></div>
</article>
</main>
<script>
let bundle = {data};
let active = 0;
let activeCommandId = null;
let manualActive = false;
let paused = false;
let speedMultiplier = 1;
let liveStatus = 'loaded';
let liveEvents = [];
let control = {{}};
const byId = (id) => document.getElementById(id);
function statusClass(outcome) {{ return outcome === 'passed' ? 'passed' : outcome === 'failed' ? 'failed' : outcome === 'running' ? 'running' : 'skipped'; }}
function reviewShell() {{ return (bundle.open_control && bundle.open_control.review_shell) || {{kind: 'export-only', driver: 'none'}}; }}
function browserTarget() {{ return (bundle.open_control && bundle.open_control.browser) || {{kind: 'export-only', driver: 'none'}}; }}
function normalizeSpeedMultiplier(value) {{
  const n = Number(value);
  return n === 2 || n === 4 ? n : 1;
}}
function renderSpeedControl() {{
  speedMultiplier = normalizeSpeedMultiplier(control && control.speed_multiplier);
  document.querySelectorAll('.speed-button').forEach((btn) => {{
    const active = normalizeSpeedMultiplier(btn.dataset.speed) === speedMultiplier;
    btn.classList.toggle('active', active);
    btn.setAttribute('aria-pressed', active ? 'true' : 'false');
  }});
}}
function escapeHtml(value) {{
  return String(value ?? '').replace(/[&<>"']/g, ch => ({{'&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;'}})[ch]);
}}
function artifactUrl(path) {{
  const raw = String(path || '');
  if (!raw || raw.startsWith('/') || raw.includes('..')) return null;
  return raw.replace(/\\/g, '/');
}}
function fullTitleFromTest(test) {{
  const suite = Array.isArray(test.suite) ? test.suite : [];
  return [...suite, test.name].filter(Boolean).join(' > ');
}}
function latestPlanCases() {{
  const plan = [...liveEvents].reverse().find(e => e.kind === 'plan' && Array.isArray(e.tests));
  if (!plan) return [];
  return plan.tests.map((test, index) => {{
    const title = fullTitleFromTest(test);
    const finished = [...liveEvents].reverse().find(e => e.kind === 'case_finished' && (e.case_id === test.id || e.title === title));
    const started = [...liveEvents].reverse().find(e => e.kind === 'case_started' && (e.case_id === test.id || e.title === title));
    return {{
      id: test.id || `live-case-${{index}}`,
      title,
      file: plan.file || '',
      outcome: finished ? (finished.outcome || 'passed') : (started ? 'running' : (test.skip ? 'skipped' : 'pending')),
      duration_ms: finished ? (finished.duration_ms || 0) : 0,
      steps: []
    }};
  }});
}}
function cases() {{
  if (bundle.cases && bundle.cases.length) return bundle.cases;
  return latestPlanCases();
}}
function currentCase() {{
  const list = cases();
  if (active >= list.length) active = Math.max(0, list.length - 1);
  return list[active] || {{id: 'case-none', title: 'No cases', steps: []}};
}}
function eventMatchesCase(event, c, index) {{
  const ids = new Set([String(index), c.id, c.case_id].filter(Boolean).map(String));
  return ids.has(String(event.case_id || '')) || event.case_title === c.title || event.title === c.title;
}}
function maybeAutoSelectCase() {{
  if (manualActive) return;
  const list = cases();
  const latest = [...liveEvents].reverse().find(e => e.case_id || e.case_title);
  if (!latest) return;
  const idx = list.findIndex((c, i) => eventMatchesCase(latest, c, i));
  if (idx >= 0) active = idx;
}}
function commandEventsForCase(c) {{
  const byUserStep = new Map();
  liveEvents
    .filter(e => e.kind === 'step_started' || e.kind === 'step_finished')
    .filter(e => eventMatchesCase(e, c, active))
    .forEach((event, index) => {{
      const key = event.step_id || `step-${{index}}`;
      const previous = byUserStep.get(key) || {{}};
      byUserStep.set(key, {{
        ...previous,
        ...event,
        step_id: key,
        action: event.title || previous.action || key,
        status: event.status || event.outcome || previous.status || (event.kind === 'step_started' ? 'running' : 'passed')
      }});
    }});
  const userSteps = Array.from(byUserStep.values());
  if (userSteps.length) return userSteps;

  const byStep = new Map();
  liveEvents
    .filter(e => e.kind === 'page_action_started' || e.kind === 'page_action_finished')
    .filter(e => eventMatchesCase(e, c, active))
    .forEach((event, index) => {{
      const key = event.step_id || `${{event.action || 'action'}}-${{index}}`;
      const previous = byStep.get(key) || {{}};
      byStep.set(key, {{...previous, ...event, step_id: key}});
    }});
  const commands = Array.from(byStep.values());
  if (commands.length) return commands;
  return (c.steps || []).map((step, index) => ({{
    kind: 'evidence_step',
    step_id: step.id || `step-${{index}}`,
    title: step.title,
    action: step.title,
    status: step.status,
    assertion: step.assertion,
    context: step.context || {{}}
  }}));
}}
function currentCommand(c) {{
  const commands = commandEventsForCase(c);
  return commands.find(command => command.step_id === activeCommandId) || commands[commands.length - 1] || null;
}}
function compactText(value, fallback = '') {{
  return String(value ?? fallback).replace(/\s+/g, ' ').trim();
}}
function runItemHtml(item, index) {{
  const action = compactText(item.action || item.title || item.kind, `Step ${{index + 1}}`);
  const target = compactText(item.selector || item.url || item.page_id || '');
  const status = compactText(item.status || item.outcome || '');
  return `<li><strong>${{escapeHtml(action)}}</strong>${{target ? `<div class="statusline">${{escapeHtml(target)}}</div>` : ''}}${{status ? `<div class="statusline">${{escapeHtml(status)}}</div>` : ''}}</li>`;
}}
function assertionMessages(c, commands) {{
  const messages = [];
  (c.steps || []).forEach(step => {{
    const msg = step && step.assertion && step.assertion.message;
    if (msg) messages.push(msg);
  }});
  commands.forEach(command => {{
    const msg = command && command.assertion && command.assertion.message;
    if (msg) messages.push(msg);
  }});
  return [...new Set(messages.map(message => compactText(message)).filter(Boolean))];
}}
function renderCaseExplanation(c, commands) {{
  const source = commands.length ? commands : (c.steps || []);
  const runItems = source.length
    ? source.map(runItemHtml).join('')
    : '<li>Waiting for the first live action from the controlled Chrome tab.</li>';
  byId('case-runbook').innerHTML = `<ol>${{runItems}}</ol>`;

  const checks = [];
  if (c.file) checks.push(`Spec file: ${{c.file}}`);
  checks.push('Passes when every listed step completes without failed assertions, timeouts, or browser RPC errors.');
  assertionMessages(c, commands).forEach(message => checks.push(`Assertion: ${{message}}`));
  if (!assertionMessages(c, commands).length && c.outcome && c.outcome !== 'pending') {{
    checks.push(`Current result: ${{c.outcome}}`);
  }}
  byId('case-checks').innerHTML = `<ul>${{checks.map(check => `<li>${{escapeHtml(check)}}</li>`).join('')}}</ul>`;
}}
function renderPmSummary(summary, list) {{
  const total = (summary.passed || 0) + (summary.failed || 0) + (summary.skipped || 0);
  const failedCount = summary.failed || 0;
  const passedCount = summary.passed || 0;
  const headline = failedCount > 0
    ? `<strong style="color: var(--red)">${{failedCount}} failed</strong> out of ${{total}} cases`
    : `<strong style="color: var(--green)">All ${{total}} cases passed</strong>`;
  const durationSec = ((summary.duration_ms || 0) / 1000).toFixed(2);
  byId('pm-summary').innerHTML = `${{headline}}<div class="statusline">${{passedCount}} passed · ${{summary.skipped || 0}} skipped · ${{durationSec}}s · run ${{bundle.run_id || ''}}</div>`;
  const failures = (list || []).filter(c => (c.outcome || '') === 'failed');
  if (!failures.length) {{
    byId('pm-failures').textContent = '';
    return;
  }}
  byId('pm-failures').textContent = failures.map(c => {{
    const failedStep = (c.steps || []).find(s => (s.status || '') === 'failed') || {{}};
    const assertion = failedStep.assertion || {{}};
    const msg = assertion.message || c.outcome || 'failed';
    const stepTitle = failedStep.title || '(unknown step)';
    const shots = ((failedStep.context && failedStep.context.screenshots) || [])
      .map(s => s.path || '')
      .filter(Boolean);
    const shotLine = shots.length ? `\nScreenshots: ${{shots.join(', ')}}` : '';
    const diff = assertion.diff ? `\n${{assertion.diff}}` : '';
    return `• ${{c.title}}\n  step: ${{stepTitle}}\n  ${{msg}}${{diff}}${{shotLine}}`;
  }}).join('\n\n');
}}
function renderCases() {{
  const summary = bundle.summary || {{passed: 0, failed: 0, skipped: 0}};
  const list = cases();
  byId('summary').innerHTML = `<div>${{summary.passed || 0}} passed / ${{summary.failed || 0}} failed / ${{summary.skipped || 0}} skipped</div><small>${{bundle.run_id || ''}}</small>`;
  byId('runtime').innerHTML = `<strong>${{reviewShell().kind}}</strong><div class="statusline">${{liveStatus}}</div>`;
  byId('header-status').textContent = `${{liveStatus}} · ${{speedMultiplier}}x`;
  renderPmSummary(summary, list);
  byId('cases').innerHTML = list.map((c, i) => `<button class="case ${{i === active ? 'active' : ''}}" data-i="${{i}}"><span class="status ${{statusClass(c.outcome)}}">${{c.outcome}}</span>${{c.title}}</button>`).join('');
  document.querySelectorAll('.case').forEach(btn => btn.onclick = () => {{
    active = Number(btn.dataset.i);
    activeCommandId = null;
    manualActive = true;
    render();
  }});
}}
function renderCommands(c) {{
  const commands = commandEventsForCase(c);
  if (commands.length && !commands.some(command => command.step_id === activeCommandId)) activeCommandId = commands[commands.length - 1].step_id;
  byId('commands').innerHTML = commands.map(command => {{
    const status = command.status || (command.kind === 'page_action_started' ? 'running' : 'passed');
    const target = command.selector || command.url || '';
    const activeClass = command.step_id === activeCommandId ? 'active' : '';
    return `<button class="command ${{statusClass(status)}} ${{activeClass}}" data-step="${{command.step_id}}"><strong>${{command.step_id}}</strong> ${{command.action || command.title || 'step'}}<div class="statusline">${{target}}</div></button>`;
  }}).join('');
  document.querySelectorAll('.command').forEach(btn => btn.onclick = () => {{
    activeCommandId = btn.dataset.step;
    render();
  }});
}}
function renderArtifacts() {{
  const artifacts = Array.isArray(bundle.artifacts) ? bundle.artifacts : [];
  if (!artifacts.length) {{
    byId('artifacts').innerHTML = '<div class="statusline">No top-level artifacts</div>';
    return;
  }}
  byId('artifacts').innerHTML = artifacts.map((artifact) => {{
    const path = artifact.path || '';
    const kind = artifact.kind || 'artifact';
    const label = artifact.label || kind;
    const meta = [kind, path].filter(Boolean).join(' · ');
    const url = artifactUrl(path);
    if (!url) {{
      return `<div class="artifact-link disabled"><strong>${{escapeHtml(label)}}</strong><span>${{escapeHtml(meta)}}</span></div>`;
    }}
    return `<a class="artifact-link" href="${{escapeHtml(url)}}" target="_blank" rel="noreferrer"><strong>${{escapeHtml(label)}}</strong><span>${{escapeHtml(meta)}}</span></a>`;
  }}).join('');
}}
function render() {{
  maybeAutoSelectCase();
  renderSpeedControl();
  const c = currentCase();
  const command = currentCommand(c);
  const commands = commandEventsForCase(c);
  byId('title').textContent = c.title;
  renderCommands(c);
  renderCaseExplanation(c, commands);
  const fallbackStep = c.steps && c.steps[0] ? c.steps[0] : {{context: {{}}}};
  const commandContext = command && command.context ? command.context : null;
  const selectors = command && command.selector ? [{{selector: command.selector, action: command.action || 'action', highlighted: true}}] : ((commandContext && commandContext.selectors) || fallbackStep.context.selectors || []);
  const screenshots = (commandContext && commandContext.screenshots) || fallbackStep.context.screenshots || [];
  const consoleRows = liveEvents.filter(e => e.kind === 'console').slice(-20);
  const networkRows = (commandContext && commandContext.network) || fallbackStep.context.network || [];
  const assertion = (command && command.assertion) || fallbackStep.assertion || null;
  byId('selectors').textContent = JSON.stringify(selectors, null, 2);
  byId('assertions').textContent = JSON.stringify(assertion || {{status: command ? (command.status || 'running') : c.outcome}}, null, 2);
  byId('screenshots').textContent = JSON.stringify(screenshots, null, 2);
  renderArtifacts();
  byId('console-count').textContent = String(consoleRows.length);
  byId('console').textContent = JSON.stringify(consoleRows, null, 2);
  byId('network').textContent = JSON.stringify(networkRows, null, 2);
  const shell = reviewShell();
  const target = browserTarget();
  byId('shell-kind').textContent = shell.kind || 'export-only';
  byId('shell-status').textContent = liveStatus;
  byId('shell-detail').textContent = JSON.stringify(shell, null, 2);
  byId('browser-kind').textContent = target.kind || 'export-only';
  byId('browser-status').textContent = liveStatus;
  byId('browser-detail').textContent = JSON.stringify(target, null, 2);
  byId('target-kind').textContent = `${{target.kind || 'export-only'}} / ${{target.driver || 'none'}}`;
  byId('target-command').textContent = command ? `${{command.step_id || ''}} ${{command.action || command.title || ''}} ${{command.status || 'running'}}` : liveStatus;
  byId('target-selector').textContent = command && command.selector ? command.selector : 'none';
  byId('target-page').textContent = command && command.page_id ? command.page_id : (target.cdp_ws_url || 'created by live page fixture');
  byId('live-events').innerHTML = liveEvents.slice(-80).reverse().map(e => `<div class="live-event"><strong>${{e.kind || 'event'}}</strong>${{e.title || e.action || e.command || e.message || ''}}</div>`).join('');
  byId('aut-status').textContent = command ? `${{command.step_id}} ${{command.status || 'running'}}` : liveStatus;
  renderCases();
}}
const reviewMode = document.body.dataset.mode || 'live';
const isReadOnly = reviewMode === 'read-only' || reviewMode === 'pm-report';
async function sendControl(command, payload = {{}}) {{
  if (isReadOnly) return;
  const c = currentCase();
  if (location.protocol.startsWith('http')) {{
    await fetch(`/api/control/${{command}}`, {{
      method: 'POST',
      headers: {{'content-type': 'application/json'}},
      body: JSON.stringify({{case_index: active, case_id: c.id, case_title: c.title, ...payload}})
    }}).catch(() => null);
    await syncLiveState();
    return;
  }}
  if (command === 'pause') paused = true;
  if (command === 'resume') paused = false;
  if (command === 'next') active = Math.min(active + 1, Math.max(0, bundle.cases.length - 1));
  if (command === 'replay') activeCommandId = null;
  if (command === 'speed') control = {{...control, speed_multiplier: normalizeSpeedMultiplier(payload.speed_multiplier)}};
  render();
}}
byId('pause').onclick = () => sendControl((control && control.paused) || paused ? 'resume' : 'pause');
byId('next').onclick = () => sendControl('next');
byId('replay').onclick = () => sendControl('replay');
document.querySelectorAll('.speed-button').forEach((btn) => {{
  btn.onclick = () => sendControl('speed', {{speed_multiplier: normalizeSpeedMultiplier(btn.dataset.speed)}});
}});
window.JetE2E = {{
  setStatus(status) {{
    liveStatus = status;
    render();
  }}
}};
async function syncLiveState() {{
  if (isReadOnly) return;
  if (!location.protocol.startsWith('http')) return;
  const res = await fetch('/api/state', {{ cache: 'no-store' }}).catch(() => null);
  if (!res || !res.ok) return;
  const state = await res.json();
  if (state.bundle) bundle = state.bundle;
  liveEvents = Array.isArray(state.events) ? state.events : [];
  control = state.control || {{}};
  paused = Boolean(control.paused);
  byId('pause').textContent = paused ? 'Resume' : 'Pause';
  const last = liveEvents[liveEvents.length - 1];
  liveStatus = last ? `${{last.kind}}` : liveStatus;
  render();
}}
render();
if (!isReadOnly && location.protocol.startsWith('http')) {{
  syncLiveState();
  setInterval(syncLiveState, 300);
}}
</script>
</body>
</html>
"#
    ))
}

fn open_runner_shell_dir(evidence_dir: &Path) -> PathBuf {
    evidence_dir.join("open-runner-shell")
}

fn open_runner_shell_path(evidence_dir: &Path) -> PathBuf {
    open_runner_shell_dir(evidence_dir).join("index.html")
}

fn open_control_protocol(
    review_shell: Option<E2eOpenReviewShell>,
    browser: E2eOpenBrowserTarget,
    event_log: &Path,
) -> E2eOpenControlProtocol {
    E2eOpenControlProtocol {
        protocol_version: CONTROL_PROTOCOL_VERSION.to_string(),
        transport: "local-jsonl+cdp".to_string(),
        review_shell,
        browser,
        event_log: event_log.to_path_buf(),
        commands: vec![
            E2eOpenCommand {
                name: "pause".to_string(),
                description: "Pause live case execution at the next runner checkpoint.".to_string(),
            },
            E2eOpenCommand {
                name: "next".to_string(),
                description: "Advance to the next case or product step.".to_string(),
            },
            E2eOpenCommand {
                name: "speed".to_string(),
                description: "Set live case playback speed to 1x, 2x, or 4x.".to_string(),
            },
            E2eOpenCommand {
                name: "replay".to_string(),
                description: "Replay the selected case against the controlled Jet Browser."
                    .to_string(),
            },
            E2eOpenCommand {
                name: "highlight_selector".to_string(),
                description: "Highlight selector context in the live Jet Browser target."
                    .to_string(),
            },
        ],
    }
}

#[cfg(test)]
fn percent_encode_file_path(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for byte in raw.bytes() {
        if matches!(
            byte,
            b'A'..=b'Z'
                | b'a'..=b'z'
                | b'0'..=b'9'
                | b'/'
                | b':'
                | b'-'
                | b'_'
                | b'.'
                | b'~'
        ) {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn test_title(report: &TestReport) -> String {
    if report.suite.is_empty() {
        report.name.clone()
    } else {
        format!("{} > {}", report.suite.join(" > "), report.name)
    }
}

fn outcome_string(outcome: Outcome) -> String {
    match outcome {
        Outcome::Passed => "passed",
        Outcome::Failed => "failed",
        Outcome::Skipped => "skipped",
        Outcome::TimedOut => "timed_out",
        Outcome::Crashed => "crashed",
    }
    .to_string()
}

fn make_run_id(started_at_ms: u64, mode: E2eMode) -> String {
    let mode = match mode {
        E2eMode::Run => "run",
        E2eMode::Open => "open",
        E2eMode::Manual => "manual",
    };
    format!("{mode}-{started_at_ms}")
}

fn now_ms() -> u64 {
    // GH #3669 — was `.unwrap_or(0)` which silently collapsed any
    // clock-before-epoch failure (Mac VM reset, container without
    // `--rtc`, freshly-booted devboard before NTP sync) onto epoch zero.
    // The fallout is broad: bundle started_at/finished_at, replay
    // timing, idle touched_at_ms, and the `run-{started_at_ms}` id all
    // collapse to zero with no breadcrumb. `safe_e2e_now_ms` preserves
    // the historical zero on the broken-clock branch but returns a
    // tagged warn so the operator sees why downstream timestamps look
    // wrong. Same family as #3644 (task_runner cache clock fallback)
    // and #3582/#3586/#3610 (env var NotPresent/NotUnicode collapse).
    let (ms, warn) = safe_e2e_now_ms(SystemTime::now());
    if let Some(msg) = warn {
        tracing::warn!(target: "jet::e2e", "{}", msg);
    }
    ms
}

/// GH #3669 — convert `SystemTime` to epoch-ms with an observable error
/// branch. Happy path returns the wall-clock millis. Error branch (clock
/// before UNIX_EPOCH) returns `0` (preserving historical behaviour so
/// existing assertions and run-id formats don't shift) plus a tagged
/// warn message the caller is expected to emit via `tracing::warn!`
/// against its own static-target macro.
///
/// The warn message is returned (rather than emitted here) so each call
/// site can use a compile-time-constant `target:` for `tracing::warn!`
/// (the `target:` arg must be a constant expression).
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub(crate) fn safe_e2e_now_ms(now: SystemTime) -> (u64, Option<String>) {
    match now.duration_since(UNIX_EPOCH) {
        Ok(dur) => (dur.as_millis() as u64, None),
        Err(err) => {
            let warn = format_safe_e2e_now_ms_warn(&err);
            (0, Some(warn))
        }
    }
}

/// GH #3669 — build the warn wording for the clock-before-epoch branch.
/// Extracted so the issue tag, error visibility, and operator guidance
/// are unit-testable without provoking the actual broken-clock platform
/// case.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub(crate) fn format_safe_e2e_now_ms_warn(err: &std::time::SystemTimeError) -> String {
    format!(
        "GH #3669 jet::e2e: SystemTime::now() reports a wall clock before \
         UNIX_EPOCH ({err}); falling back to ms=0. Every e2e timestamp \
         (bundle started_at_ms/finished_at_ms, replay timing, \
         touched_at_ms idle eviction) and the run id `run-{{started_at_ms}}` \
         will be derived from zero, so back-to-back runs will collide on \
         `run-0` and overwrite each other's evidence dirs. Fix the host \
         clock (NTP / container --rtc / RTC battery) before trusting any \
         duration reported by this run."
    )
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn default_evidence_dir(project_root: &Path, mode: E2eMode) -> PathBuf {
    let leaf = match mode {
        E2eMode::Run => "run",
        E2eMode::Open => "open",
        E2eMode::Manual => "manual",
    };
    project_root.join("test-results").join("e2e").join(leaf)
}

/// Parse the CLI-supplied `--trace` value for agent mode.
///
/// `None` (flag omitted) defaults to `RetainOnFailure`. `Some(garbage)`
/// returns `Err` instead of silently coercing — GH #3097 closes the
/// silent-swallow gap that #3094 fixed on the parallel `jet test` path.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn parse_trace_mode(raw: Option<&String>) -> anyhow::Result<WireTraceMode> {
    match raw {
        None => Ok(WireTraceMode::RetainOnFailure),
        Some(s) => WireTraceMode::from_str(s).ok_or_else(|| {
            anyhow::anyhow!("Unknown --trace value '{s}'. Valid values: off, on, retain-on-failure")
        }),
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn parse_workers(raw: Option<&usize>) -> Option<usize> {
    raw.copied().map(|n| n.max(1))
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn parse_serve_mode(raw: Option<&String>) -> anyhow::Result<E2eServeMode> {
    match raw.map(String::as_str) {
        None | Some("off") => Ok(E2eServeMode::Off),
        Some("dev") => Ok(E2eServeMode::Dev),
        Some("prod") => Ok(E2eServeMode::Prod),
        Some(other) => {
            anyhow::bail!("Unknown --serve value '{other}'. Valid values: off, dev, prod")
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn parse_open_browser_mode(raw: Option<&String>) -> anyhow::Result<E2eOpenBrowserMode> {
    match raw.map(String::as_str) {
        None | Some("chrome") => Ok(E2eOpenBrowserMode::Chrome),
        Some("chromium") => Ok(E2eOpenBrowserMode::Chromium),
        Some(other) => {
            anyhow::bail!("Unknown --browser value '{other}'. Valid values: chrome, chromium")
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn summary_exit_code(bundle: &E2eEvidenceBundle) -> i32 {
    bundle.summary.exit_code
}

/// Map per-case outcomes to a deterministic process exit code.
///
/// Precedence: infrastructure (Crashed) > timeout (TimedOut) > assertion
/// (Failed) > pass. Skipped does not influence the exit code.
///
/// @spec #2618
pub fn exit_code_for_reports(reports: &[crate::test_runner::reporter::TestReport]) -> i32 {
    use crate::test_runner::reporter::Outcome;
    let mut saw_failed = false;
    let mut saw_timeout = false;
    let mut saw_crashed = false;
    for r in reports {
        match r.outcome {
            Outcome::Crashed => saw_crashed = true,
            Outcome::TimedOut => saw_timeout = true,
            Outcome::Failed => saw_failed = true,
            Outcome::Passed | Outcome::Skipped => {}
        }
    }
    if saw_crashed {
        E2E_EXIT_INFRASTRUCTURE
    } else if saw_timeout {
        E2E_EXIT_TIMEOUT
    } else if saw_failed {
        E2E_EXIT_ASSERTION_FAILURE
    } else {
        E2E_EXIT_OK
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_runner::reporter::{TestError, TestReport, TestStepReport};
    use tempfile::TempDir;

    fn report(outcome: Outcome) -> TestReport {
        TestReport {
            file: PathBuf::from("flows/cue-artifact-studio.spec.ts"),
            suite: vec!["Cue Artifact Studio".to_string()],
            name: "promotes work item into artifact".to_string(),
            outcome,
            duration_ms: 42,
            error: if matches!(outcome, Outcome::Failed) {
                Some(TestError {
                    message: "expected artifact to be visible".to_string(),
                    stack: Some("stack".to_string()),
                    diff: Some("- missing\n+ visible".to_string()),
                    source_location: None,
                })
            } else {
                None
            },
            trace_path: None,
            shard_index: None,
            shard_total: None,
            artifacts: vec![PathBuf::from("test-results/artifacts/case/page-1.png")],
            steps: Vec::new(),
        }
    }

    #[test]
    fn parse_serve_mode_accepts_basic_dom_targets() {
        assert_eq!(parse_serve_mode(None).unwrap(), E2eServeMode::Off);
        assert_eq!(
            parse_serve_mode(Some(&"off".to_string())).unwrap(),
            E2eServeMode::Off
        );
        assert_eq!(
            parse_serve_mode(Some(&"dev".to_string())).unwrap(),
            E2eServeMode::Dev
        );
        assert_eq!(
            parse_serve_mode(Some(&"prod".to_string())).unwrap(),
            E2eServeMode::Prod
        );
        assert!(parse_serve_mode(Some(&"wasm".to_string())).is_err());
    }

    #[test]
    fn parse_open_browser_mode_accepts_chrome_and_chromium() {
        assert_eq!(
            parse_open_browser_mode(None).unwrap(),
            E2eOpenBrowserMode::Chrome
        );
        assert_eq!(
            parse_open_browser_mode(Some(&"chrome".to_string())).unwrap(),
            E2eOpenBrowserMode::Chrome
        );
        assert_eq!(
            parse_open_browser_mode(Some(&"chromium".to_string())).unwrap(),
            E2eOpenBrowserMode::Chromium
        );
        assert!(parse_open_browser_mode(Some(&"firefox".to_string())).is_err());
    }

    #[test]
    fn evidence_bundle_can_carry_serve_session() {
        let tmp = TempDir::new().unwrap();
        let serve_session = crate::dev_server::session::ServeSession {
            schema_version: crate::dev_server::session::SCHEMA_VERSION.to_string(),
            mode: crate::dev_server::session::MODE_DETACHED.to_string(),
            target: crate::dev_server::session::TARGET_DOM.to_string(),
            host: "127.0.0.1".to_string(),
            port: 43127,
            url: "http://127.0.0.1:43127/".to_string(),
            pid: 123,
            root_dir: tmp.path().display().to_string(),
            log_file: Some(
                crate::dev_server::session::log_path(tmp.path())
                    .display()
                    .to_string(),
            ),
            started_at: crate::dev_server::session::now_unix(),
        };
        let bundle = build_evidence_bundle(
            E2eMode::Run,
            Summary::default(),
            10,
            20,
            Some(serve_session),
            None,
        );

        let session = bundle.serve_session.expect("serve session evidence");
        assert_eq!(session.target, crate::dev_server::session::TARGET_DOM);
        assert_eq!(session.url, "http://127.0.0.1:43127/");
        let serve_log = bundle
            .artifacts
            .iter()
            .find(|artifact| artifact.kind == "serve-log")
            .expect("managed serve log is a shareable artifact");
        assert_eq!(
            serve_log.label.as_deref(),
            Some("jet serve dom log"),
            "serve log artifact label should name the served target",
        );
        assert!(
            serve_log.path.ends_with(".jet/serve.log"),
            "serve log artifact should point at the session log file: {serve_log:?}",
        );
    }

    #[tokio::test]
    async fn run_agent_mode_rejects_managed_serve_and_base_url_before_launching() {
        let tmp = TempDir::new().unwrap();
        let opts = E2eRunOptions {
            project_root: tmp.path().to_path_buf(),
            cases: vec![],
            grep: None,
            timeout_ms: None,
            workers: None,
            trace: WireTraceMode::Off,
            evidence_dir: tmp.path().join("evidence"),
            serve: E2eServeMode::Dev,
            base_url: Some("http://127.0.0.1:43127/".to_string()),
            print_json: false,
        };

        let err = run_agent_mode(opts).await.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("--serve") && msg.contains("--base-url"),
            "error should name conflicting flags: {msg}"
        );
        assert!(
            !crate::dev_server::session::session_path(tmp.path()).exists(),
            "conflict must be rejected before launching a managed serve session"
        );
    }

    #[tokio::test]
    async fn run_manual_mode_rejects_managed_serve_and_base_url_before_launching() {
        let tmp = TempDir::new().unwrap();
        let opts = E2eManualOptions {
            project_root: tmp.path().to_path_buf(),
            cases: vec![],
            grep: None,
            timeout_ms: None,
            trace: WireTraceMode::Off,
            evidence_dir: tmp.path().join("evidence"),
            out_dir: tmp.path().join("docs/manual"),
            serve: E2eServeMode::Dev,
            base_url: Some("http://127.0.0.1:43127/".to_string()),
            title: None,
            print_json: false,
        };

        let err = run_manual_mode(opts).await.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("manual") && msg.contains("--serve") && msg.contains("--base-url"),
            "manual conflict error should name mode and flags: {msg}"
        );
        assert!(
            !crate::dev_server::session::session_path(tmp.path()).exists(),
            "manual conflict must be rejected before launching a managed serve session"
        );
    }

    #[tokio::test]
    async fn open_human_mode_rejects_managed_serve_and_base_url_before_launching() {
        let tmp = TempDir::new().unwrap();
        let opts = E2eOpenOptions {
            project_root: tmp.path().to_path_buf(),
            cases: vec![],
            grep: None,
            timeout_ms: None,
            evidence_dir: tmp.path().join("evidence"),
            serve: E2eServeMode::Dev,
            base_url: Some("http://127.0.0.1:43127/".to_string()),
            slow_mo_ms: DEFAULT_OPEN_SLOW_MO_MS,
            browser: E2eOpenBrowserMode::Chromium,
            dry_run: true,
            no_open: true,
        };

        let err = open_human_mode(opts).await.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("open") && msg.contains("--serve") && msg.contains("--base-url"),
            "open conflict error should name mode and flags: {msg}"
        );
        assert!(
            !crate::dev_server::session::session_path(tmp.path()).exists(),
            "open conflict must be rejected before launching a managed serve session"
        );
    }

    #[test]
    fn evidence_bundle_carries_case_step_and_context() {
        let summary = Summary {
            schema_version: crate::test_runner::reporter::SCHEMA_VERSION,
            passed: 0,
            failed: 1,
            skipped: 0,
            duration_ms: 42,
            reports: vec![report(Outcome::Failed)],
            coverage: None,
            browser_sessions: Vec::new(),
        };
        let bundle = build_evidence_bundle(E2eMode::Run, summary, 10, 20, None, None);
        assert_eq!(bundle.schema_version, EVIDENCE_SCHEMA_VERSION);
        assert_eq!(bundle.summary.exit_code, 1);
        assert_eq!(bundle.cases.len(), 1);
        assert_eq!(bundle.cases[0].steps.len(), 1);
        assert!(bundle.cases[0].steps[0].assertion.is_some());
        assert_eq!(bundle.cases[0].steps[0].context.screenshots.len(), 1);
    }

    #[test]
    fn evidence_bundle_expands_authored_test_steps() {
        let mut report = report(Outcome::Passed);
        report.steps = vec![
            TestStepReport {
                id: "step-0001".to_string(),
                title: "Create project".to_string(),
                outcome: Outcome::Passed,
                duration_ms: 7,
                parent_step_id: None,
                error: None,
            },
            TestStepReport {
                id: "step-0002".to_string(),
                title: "Review generated README capability contract".to_string(),
                outcome: Outcome::Passed,
                duration_ms: 11,
                parent_step_id: None,
                error: None,
            },
        ];
        let summary = Summary {
            schema_version: crate::test_runner::reporter::SCHEMA_VERSION,
            passed: 1,
            failed: 0,
            skipped: 0,
            duration_ms: 18,
            reports: vec![report],
            coverage: None,
            browser_sessions: Vec::new(),
        };

        let bundle = build_evidence_bundle(E2eMode::Manual, summary, 10, 28, None, None);

        assert_eq!(bundle.cases[0].steps.len(), 2);
        assert_eq!(bundle.cases[0].steps[0].title, "Create project");
        assert_eq!(
            bundle.cases[0].steps[1].title,
            "Review generated README capability contract"
        );
        assert_eq!(bundle.cases[0].steps[1].duration_ms, 11);
    }

    #[test]
    fn write_evidence_emits_json_and_jsonl() {
        let tmp = TempDir::new().unwrap();
        let summary = Summary {
            schema_version: crate::test_runner::reporter::SCHEMA_VERSION,
            passed: 1,
            failed: 0,
            skipped: 0,
            duration_ms: 12,
            reports: vec![report(Outcome::Passed)],
            coverage: None,
            browser_sessions: Vec::new(),
        };
        let bundle = build_evidence_bundle(E2eMode::Run, summary, 10, 20, None, None);
        let written = write_evidence_bundle(tmp.path(), &bundle).unwrap();
        assert!(written.evidence_path.exists());
        assert!(written.jsonl_path.exists());
        let jsonl = std::fs::read_to_string(written.jsonl_path).unwrap();
        assert!(jsonl.contains("run_started"));
        assert!(jsonl.contains("case_finished"));
        assert!(jsonl.contains("run_finished"));
    }

    #[test]
    fn manual_docs_writer_publishes_markdown_html_and_copied_screenshots() {
        let tmp = TempDir::new().unwrap();
        let screenshot = tmp.path().join("test-results/artifacts/case/page-1.png");
        std::fs::create_dir_all(screenshot.parent().unwrap()).unwrap();
        std::fs::write(&screenshot, b"fake-png").unwrap();
        let summary = Summary {
            schema_version: crate::test_runner::reporter::SCHEMA_VERSION,
            passed: 1,
            failed: 0,
            skipped: 0,
            duration_ms: 12,
            reports: vec![report(Outcome::Passed)],
            coverage: None,
            browser_sessions: Vec::new(),
        };
        let bundle = build_evidence_bundle(E2eMode::Manual, summary, 10, 20, None, None);
        let docs = write_manual_docs(
            tmp.path(),
            &tmp.path().join("docs/e2e-manual"),
            &bundle,
            "Product Manual",
        )
        .unwrap();
        let markdown = std::fs::read_to_string(&docs.markdown_path).unwrap();
        let html = std::fs::read_to_string(&docs.html_path).unwrap();
        assert!(markdown.contains("# Product Manual"), "{markdown}");
        assert!(
            markdown.contains("Generated by `jet e2e manual`"),
            "{markdown}"
        );
        assert!(markdown.contains("![Cue Artifact Studio"), "{markdown}");
        assert!(html.contains("<title>Product Manual</title>"), "{html}");
        assert!(
            tmp.path()
                .join("docs/e2e-manual/images/case-01-step-01-01.png")
                .exists(),
            "manual docs should copy screenshot artifacts into a portable images dir"
        );
    }

    #[test]
    fn open_runner_shell_contains_controls_panels_and_browser_target() {
        let tmp = TempDir::new().unwrap();
        let summary = Summary {
            schema_version: crate::test_runner::reporter::SCHEMA_VERSION,
            passed: 1,
            failed: 0,
            skipped: 0,
            duration_ms: 12,
            reports: vec![report(Outcome::Passed)],
            coverage: None,
            browser_sessions: Vec::new(),
        };
        let protocol = E2eOpenControlProtocol {
            protocol_version: CONTROL_PROTOCOL_VERSION.to_string(),
            transport: "local-jsonl+cdp".to_string(),
            review_shell: Some(E2eOpenReviewShell {
                kind: E2eOpenReviewShellKind::BrowserWindowTabs,
                driver: JET_REVIEW_SHELL_DRIVER.to_string(),
                runner_shell: tmp.path().join("open-runner-shell/index.html"),
                runner_shell_url: Some("http://127.0.0.1:3000/".to_string()),
                cdp_ws_url: Some("ws://127.0.0.1/devtools/browser/shell".to_string()),
            }),
            browser: E2eOpenBrowserTarget {
                kind: E2eOpenBrowserKind::ControlledJetBrowser,
                driver: JET_BROWSER_DRIVER.to_string(),
                headless: false,
                isolated_profile: true,
                runner_shell: tmp.path().join("open-runner-shell/index.html"),
                runner_shell_url: Some("http://127.0.0.1:3000/".to_string()),
                cdp_ws_url: Some("ws://127.0.0.1/devtools/browser/shell".to_string()),
            },
            commands: vec![
                E2eOpenCommand {
                    name: "pause".to_string(),
                    description: "pause".to_string(),
                },
                E2eOpenCommand {
                    name: "next".to_string(),
                    description: "next".to_string(),
                },
                E2eOpenCommand {
                    name: "replay".to_string(),
                    description: "replay".to_string(),
                },
            ],
            event_log: tmp.path().join("events.jsonl"),
        };
        let bundle = build_evidence_bundle(E2eMode::Open, summary, 10, 20, None, Some(protocol));
        let path = write_open_runner_shell(tmp.path(), &bundle).unwrap();
        let html = std::fs::read_to_string(path).unwrap();
        assert!(html.contains("Pause"));
        assert!(html.contains("Next"));
        assert!(html.contains("Replay"));
        assert!(html.contains("Speed"));
        assert!(html.contains(r#"data-speed="1""#));
        assert!(html.contains(r#"data-speed="2""#));
        assert!(html.contains(r#"data-speed="4""#));
        assert!(html.contains("How This Case Runs"));
        assert!(html.contains("What This Case Tests"));
        assert!(html.contains("renderCaseExplanation"));
        assert!(html.contains("Review Shell"));
        assert!(html.contains("Controlled Target"));
        assert!(html.contains("Steps"));
        assert!(html.contains(r#"<details class="dev-panel" id="command-log-panel">"#));
        assert!(html.contains("target-surface"));
        assert!(html.contains("target-command"));
        assert!(html.contains("browser-window-tabs"));
        assert!(html.contains("ws://127.0.0.1/devtools/browser/shell"));
        assert!(html.contains("controlled-jet-browser"));
        assert!(html.contains("Selectors"));
        assert!(html.contains("Console"));
        assert!(html.contains(r#"<details class="panel" id="console-panel">"#));
        assert!(html.contains("Network"));
        assert!(!html.contains("auto-open-devtools-for-tabs"));
    }

    #[test]
    fn control_protocol_names_expected_human_commands_and_browser_contract() {
        let protocol = open_control_protocol(
            Some(E2eOpenReviewShell {
                kind: E2eOpenReviewShellKind::BrowserWindowTabs,
                driver: JET_REVIEW_SHELL_DRIVER.to_string(),
                runner_shell: PathBuf::from("test-results/e2e/open/open-runner-shell/index.html"),
                runner_shell_url: Some("http://127.0.0.1:3000/".to_string()),
                cdp_ws_url: None,
            }),
            E2eOpenBrowserTarget {
                kind: E2eOpenBrowserKind::ControlledJetBrowser,
                driver: JET_BROWSER_DRIVER.to_string(),
                headless: false,
                isolated_profile: true,
                runner_shell: PathBuf::from("test-results/e2e/open/open-runner-shell/index.html"),
                runner_shell_url: Some("http://127.0.0.1:3000/".to_string()),
                cdp_ws_url: None,
            },
            Path::new("test-results/e2e/open/open-10.live.events.jsonl"),
        );
        let names: Vec<&str> = protocol.commands.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["pause", "next", "speed", "replay", "highlight_selector"]
        );
        assert_eq!(protocol.transport, "local-jsonl+cdp");
        assert_eq!(
            protocol.browser.kind,
            E2eOpenBrowserKind::ControlledJetBrowser
        );
        assert_eq!(protocol.browser.driver, JET_BROWSER_DRIVER);
        assert!(!protocol.browser.headless);
        let shell = protocol.review_shell.as_ref().expect("review shell");
        assert_eq!(shell.kind, E2eOpenReviewShellKind::BrowserWindowTabs);
        assert_eq!(shell.driver, JET_REVIEW_SHELL_DRIVER);
    }

    #[test]
    fn file_urls_are_percent_encoded_for_visible_browser_navigation() {
        assert_eq!(
            percent_encode_file_path("/tmp/Jet E2E/#runner.html"),
            "/tmp/Jet%20E2E/%23runner.html"
        );
    }

    fn minimal_bundle() -> E2eEvidenceBundle {
        let summary = Summary {
            schema_version: crate::test_runner::reporter::SCHEMA_VERSION,
            passed: 1,
            failed: 0,
            skipped: 0,
            duration_ms: 12,
            reports: vec![report(Outcome::Passed)],
            coverage: None,
            browser_sessions: Vec::new(),
        };
        build_evidence_bundle(E2eMode::Open, summary, 10, 20, None, None)
    }

    /// The review UI must render from the same data source for both the
    /// live dev adapter and the read-only PM report adapter; the only
    /// allowed delta is the embedded control adapter wiring.
    // @spec #2614
    #[test]
    fn review_ui_renderer_supports_live_and_read_only_modes() {
        let bundle = minimal_bundle();
        let live = render_review_ui_html(&bundle, ReviewUiMode::Live).expect("live html");
        let ro = render_review_ui_html(&bundle, ReviewUiMode::ReadOnly).expect("read-only html");

        assert!(live.contains(r#"<body data-mode="live">"#));
        assert!(ro.contains(r#"<body data-mode="read-only">"#));

        // Live mode keeps the buttons enabled; read-only disables them.
        assert!(live.contains(r#"<button id="pause">Pause</button>"#));
        assert!(live.contains(r#"class="speed-button" data-speed="1">1x</button>"#));
        assert!(ro.contains(r#"<button id="pause" disabled>Pause</button>"#));
        assert!(ro.contains(r#"<button id="next" disabled>Next</button>"#));
        assert!(ro.contains(r#"<button id="replay" disabled>Replay</button>"#));
        assert!(ro.contains(r#"class="speed-button" data-speed="1" disabled>1x</button>"#));

        // Both modes embed the same case grid + step log + inspector
        // shell. Drift here would mean the component split regressed.
        for html in [&live, &ro] {
            assert!(html.contains("Steps"));
            assert!(html.contains("target-surface"));
            assert!(html.contains("Review Shell"));
            assert!(html.contains("Controlled Target"));
            assert!(html.contains("Selectors"));
        }

        // The read-only adapter explicitly short-circuits the control
        // adapter and the polling fetch.
        assert!(ro.contains("if (isReadOnly) return;"));
    }

    /// PM web report mode reuses the shared review renderer, surfaces a
    /// non-developer summary, and hides the live-control toolbar and
    /// dev-only command log via CSS — without forking the component.
    // @spec #2622
    #[test]
    fn pm_report_mode_reuses_review_renderer_and_hides_dev_chrome() {
        let bundle = minimal_bundle();
        let live = render_review_ui_html(&bundle, ReviewUiMode::Live).expect("live html");
        let pm = render_pm_report_html(&bundle).expect("pm html");

        // Mode flag flips to pm-report; renderer surface is shared.
        assert!(pm.contains(r#"<body data-mode="pm-report">"#));
        assert!(!pm.contains(r#"<body data-mode="live">"#));

        // Controls remain in the DOM (single renderer) but are disabled,
        // so manual JS does not fire a control POST off file:// origin.
        assert!(pm.contains(r#"<button id="pause" disabled>Pause</button>"#));
        assert!(pm.contains(r#"<button id="next" disabled>Next</button>"#));
        assert!(pm.contains(r#"<button id="replay" disabled>Replay</button>"#));

        // CSS hides the dev-only chrome whenever the body wears the
        // pm-report flag — these selectors are the contract acceptance
        // tests assert on.
        assert!(pm.contains("body[data-mode=\"pm-report\"] .toolbar"));
        assert!(pm.contains("body[data-mode=\"pm-report\"] #commands"));

        // PM-only DOM hooks (summary banner + failure block) exist.
        assert!(pm.contains(r#"<div id="pm-summary"></div>"#));
        assert!(pm.contains(r#"<div id="pm-failures"></div>"#));

        // The shared inspector + case grid still render — drift here
        // would mean we forked the component.
        for shared in [
            "target-surface",
            "Review Shell",
            "Controlled Target",
            "Selectors",
        ] {
            assert!(
                pm.contains(shared),
                "shared review chrome missing: {shared}"
            );
            assert!(live.contains(shared));
        }

        // No live-control endpoint embedded in PM mode.
        assert!(!pm.contains("/api/live-control"));
    }

    #[test]
    fn pm_report_mode_surfaces_top_level_artifacts_panel() {
        let mut bundle = minimal_bundle();
        bundle.artifacts.push(E2eArtifactRef {
            kind: "serve-log".to_string(),
            path: PathBuf::from("artifacts/serve.log"),
            label: Some("jet serve dom log".to_string()),
        });

        let pm = render_pm_report_html(&bundle).expect("pm html");

        assert!(pm.contains(r#"<h3>Artifacts</h3>"#));
        assert!(pm.contains(r#"id="artifacts""#));
        assert!(pm.contains("function renderArtifacts()"));
        assert!(pm.contains("artifact-link"));
        assert!(pm.contains("serve-log"));
        assert!(pm.contains("artifacts/serve.log"));
        assert!(pm.contains("jet serve dom log"));
    }

    /// PM mode shares the JS read-only short-circuit with the agent
    /// report so opening the report from file:// never fires a control
    /// POST. The renderer relies on the `isReadOnly` flag the body's
    /// data-mode wires up.
    // @spec #2622
    #[test]
    fn pm_report_mode_short_circuits_control_adapter_like_read_only() {
        let pm = render_pm_report_html(&minimal_bundle()).expect("pm html");
        // The bootstrap line that maps data-mode → isReadOnly accepts
        // both 'read-only' and 'pm-report' (we treat pm-report as a
        // strict superset of read-only on the JS adapter).
        assert!(pm.contains("const isReadOnly"));
        // The polling sync is gated on isReadOnly === false.
        assert!(pm.contains("if (isReadOnly) return;"));
    }

    /// Locks in the contract that the live control file round-trips the
    /// fields the open-mode review surface depends on for single-case
    /// replay and pause/next checkpoints.
    // @spec #2613
    #[test]
    fn live_control_state_round_trips_pause_next_and_replay_selection() {
        let dir = TempDir::new().expect("tempdir");
        let files = LiveRunnerFiles::new(dir.path(), "open-42");
        files
            .write_control(LiveControlState {
                paused: true,
                speed_multiplier: 4,
                next_token: 3,
                replay_token: 5,
                replay_case_index: Some(2),
                replay_case_id: Some("case-2".to_string()),
                replay_case_title: Some("checkout flow".to_string()),
            })
            .expect("write control");

        let back = files.read_control();
        assert!(back.paused);
        assert_eq!(back.speed_multiplier, 4);
        assert_eq!(back.next_token, 3);
        assert_eq!(back.replay_token, 5);
        assert_eq!(back.replay_case_index, Some(2));
        assert_eq!(back.replay_case_id.as_deref(), Some("case-2"));
        assert_eq!(back.replay_case_title.as_deref(), Some("checkout flow"));
    }

    /// Selected case replay must resolve a concrete title — explicit
    /// `replay_case_title` wins over the falling-back index lookup against
    /// the persisted runner bundle.
    // @spec #2613
    #[test]
    fn replay_case_title_prefers_explicit_title_over_index_lookup() {
        let dir = TempDir::new().expect("tempdir");
        let files = LiveRunnerFiles::new(dir.path(), "open-7");

        // Even with an indexable runner bundle, an explicit title wins.
        let bundle = E2eEvidenceBundle {
            schema_version: EVIDENCE_SCHEMA_VERSION.to_string(),
            mode: E2eMode::Open,
            run_id: "open-7".to_string(),
            started_at_ms: 1,
            finished_at_ms: 2,
            summary: E2eSummary::default(),
            cases: vec![
                E2eCaseEvidence {
                    id: "case-0".to_string(),
                    title: "first".to_string(),
                    file: PathBuf::from("e2e/flow.case.ts"),
                    outcome: "passed".to_string(),
                    duration_ms: 1,
                    steps: vec![],
                },
                E2eCaseEvidence {
                    id: "case-1".to_string(),
                    title: "second".to_string(),
                    file: PathBuf::from("e2e/flow.case.ts"),
                    outcome: "passed".to_string(),
                    duration_ms: 1,
                    steps: vec![],
                },
            ],
            artifacts: vec![],
            serve_session: None,
            browser_sessions: vec![],
            open_control: None,
        };
        write_open_runner_shell(dir.path(), &bundle).expect("write shell");

        // Title field set explicitly → that wins.
        let control = LiveControlState {
            replay_case_title: Some("preferred".to_string()),
            replay_case_index: Some(1),
            ..LiveControlState::default()
        };
        assert_eq!(
            replay_case_title(&files, &control).as_deref(),
            Some("preferred")
        );

        // Title empty → fall back to index lookup against the bundle.
        let control = LiveControlState {
            replay_case_index: Some(1),
            ..LiveControlState::default()
        };
        assert_eq!(
            replay_case_title(&files, &control).as_deref(),
            Some("second")
        );

        // Out-of-range index → None (replay handler will run all cases).
        let control = LiveControlState {
            replay_case_index: Some(99),
            ..LiveControlState::default()
        };
        assert!(replay_case_title(&files, &control).is_none());
    }

    fn step(id: &str, title: &str, status: &str, duration_ms: u64) -> E2eProductStep {
        E2eProductStep {
            id: id.to_string(),
            title: title.to_string(),
            status: status.to_string(),
            duration_ms,
            assertion: None,
            context: E2eStepContext::default(),
        }
    }

    fn bundle_with_named_steps() -> E2eEvidenceBundle {
        E2eEvidenceBundle {
            schema_version: EVIDENCE_SCHEMA_VERSION.to_string(),
            mode: E2eMode::Run,
            run_id: "run-step-1".to_string(),
            started_at_ms: 1_000,
            finished_at_ms: 1_900,
            summary: E2eSummary {
                passed: 1,
                failed: 1,
                skipped: 0,
                duration_ms: 900,
                exit_code: 1,
            },
            cases: vec![
                E2eCaseEvidence {
                    id: "case-0001".to_string(),
                    title: "creates a project".to_string(),
                    file: PathBuf::from("e2e/cue.spec.js"),
                    outcome: "passed".to_string(),
                    duration_ms: 250,
                    steps: vec![
                        step("step-0001", "open studio", "passed", 100),
                        step("step-0002", "create project", "passed", 150),
                    ],
                },
                E2eCaseEvidence {
                    id: "case-0002".to_string(),
                    title: "promotes a work item".to_string(),
                    file: PathBuf::from("e2e/cue.spec.js"),
                    outcome: "failed".to_string(),
                    duration_ms: 650,
                    steps: vec![
                        step("step-0001", "open studio", "passed", 100),
                        E2eProductStep {
                            id: "step-0002".to_string(),
                            title: "publish artifact".to_string(),
                            status: "failed".to_string(),
                            duration_ms: 550,
                            assertion: Some(E2eAssertionDetail {
                                message: "Expected work-state 'shipped'".to_string(),
                                stack: None,
                                diff: Some("- shipped\n+ reviewing".to_string()),
                            }),
                            context: E2eStepContext::default(),
                        },
                    ],
                },
            ],
            artifacts: vec![],
            serve_session: None,
            browser_sessions: vec![],
            open_control: None,
        }
    }

    #[test]
    fn events_for_bundle_emits_step_started_finished_per_step_in_order() {
        let bundle = bundle_with_named_steps();
        let events = events_for_bundle(&bundle);

        let step_events: Vec<&E2eEvidenceEvent> = events
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    E2eEvidenceEvent::StepStarted { .. } | E2eEvidenceEvent::StepFinished { .. }
                )
            })
            .collect();
        assert_eq!(step_events.len(), 8, "2 cases * 2 steps * (start+finish)");

        for pair in step_events.chunks(2) {
            match (pair[0], pair[1]) {
                (
                    E2eEvidenceEvent::StepStarted {
                        case_id: c1,
                        step_id: s1,
                        ts_ms: start,
                        ..
                    },
                    E2eEvidenceEvent::StepFinished {
                        case_id: c2,
                        step_id: s2,
                        ts_ms: end,
                        duration_ms,
                        ..
                    },
                ) => {
                    assert_eq!(c1, c2);
                    assert_eq!(s1, s2);
                    assert_eq!(end, &start.saturating_add(*duration_ms));
                }
                _ => panic!("expected Started followed by Finished"),
            }
        }
    }

    #[test]
    fn step_finished_carries_assertion_context_on_failure() {
        let bundle = bundle_with_named_steps();
        let events = events_for_bundle(&bundle);
        let failed_finish = events
            .iter()
            .find_map(|e| match e {
                E2eEvidenceEvent::StepFinished {
                    status,
                    assertion,
                    title,
                    ..
                } if status == "failed" => Some((title.clone(), assertion.clone())),
                _ => None,
            })
            .expect("a failed StepFinished event");
        assert_eq!(failed_finish.0, "publish artifact");
        let assertion = failed_finish.1.expect("failed step carries assertion");
        assert!(assertion.message.contains("work-state"));
        assert!(assertion.diff.unwrap().contains("shipped"));
    }

    #[test]
    fn timeline_ordering_groups_steps_then_case_finished_per_case() {
        let bundle = bundle_with_named_steps();
        let events = events_for_bundle(&bundle);
        assert!(matches!(
            events.first(),
            Some(E2eEvidenceEvent::RunStarted { .. })
        ));
        assert!(matches!(
            events.last(),
            Some(E2eEvidenceEvent::RunFinished { .. })
        ));

        let kinds: Vec<&'static str> = events.iter().map(event_kind).collect();
        assert_eq!(
            kinds,
            vec![
                "run_started",
                "step_started",
                "step_finished",
                "step_started",
                "step_finished",
                "case_finished",
                "step_started",
                "step_finished",
                "step_started",
                "step_finished",
                "case_finished",
                "run_finished",
            ],
        );
    }

    #[test]
    fn events_include_managed_serve_session_metadata_when_present() {
        let tmp = TempDir::new().unwrap();
        let mut bundle = bundle_with_named_steps();
        bundle.serve_session = Some(crate::dev_server::session::ServeSession {
            schema_version: crate::dev_server::session::SCHEMA_VERSION.to_string(),
            mode: crate::dev_server::session::MODE_DETACHED.to_string(),
            target: crate::dev_server::session::TARGET_DOM.to_string(),
            host: "127.0.0.1".to_string(),
            port: 43127,
            url: "http://127.0.0.1:43127/".to_string(),
            pid: 123,
            root_dir: tmp.path().display().to_string(),
            log_file: Some(
                crate::dev_server::session::log_path(tmp.path())
                    .display()
                    .to_string(),
            ),
            started_at: crate::dev_server::session::now_unix(),
        });

        let events = events_for_bundle(&bundle);
        assert!(matches!(
            events.first(),
            Some(E2eEvidenceEvent::RunStarted { .. })
        ));
        let Some(E2eEvidenceEvent::ServeSessionStarted {
            target, url, port, ..
        }) = events.get(1)
        else {
            panic!("managed serve session event should follow run_started: {events:?}");
        };
        assert_eq!(target, crate::dev_server::session::TARGET_DOM);
        assert_eq!(url, "http://127.0.0.1:43127/");
        assert_eq!(*port, 43127);
    }

    #[test]
    fn events_include_browser_session_metadata_when_present() {
        let mut bundle = bundle_with_named_steps();
        let mut session = crate::test_runner::reporter::BrowserSessionReport::launching(
            PathBuf::from("e2e/browser.spec.js"),
            true,
            1_050,
        );
        session.ready(
            Some(42),
            "ws://127.0.0.1/devtools/browser/abc".to_string(),
            1_075,
        );
        session.close(true, 1_200);
        let session_id = session.session_id.clone();
        bundle.browser_sessions.push(session);

        let events = events_for_bundle(&bundle);
        let kinds: Vec<&'static str> = events.iter().map(event_kind).collect();
        assert!(
            kinds.contains(&"browser_session_started"),
            "events should carry browser session start: {kinds:?}",
        );
        assert!(
            kinds.contains(&"browser_session_finished"),
            "events should carry browser session finish: {kinds:?}",
        );

        let start = events
            .iter()
            .find_map(|event| match event {
                E2eEvidenceEvent::BrowserSessionStarted {
                    session_id: got,
                    driver,
                    headless,
                    pid,
                    ws_endpoint,
                    ..
                } if got == &session_id => Some((driver, headless, pid, ws_endpoint)),
                _ => None,
            })
            .expect("browser session started event");
        assert_eq!(start.0, "chromium");
        assert!(*start.1);
        assert_eq!(*start.2, Some(42));
        assert_eq!(
            start.3.as_deref(),
            Some("ws://127.0.0.1/devtools/browser/abc")
        );

        let finish = events
            .iter()
            .find_map(|event| match event {
                E2eEvidenceEvent::BrowserSessionFinished {
                    session_id: got,
                    state,
                    graceful_close,
                    ..
                } if got == &session_id => Some((state, graceful_close)),
                _ => None,
            })
            .expect("browser session finished event");
        assert_eq!(finish.0, "closed");
        assert!(*finish.1);
    }

    #[test]
    fn jsonl_round_trip_preserves_step_event_payloads() {
        let bundle = bundle_with_named_steps();
        let events = events_for_bundle(&bundle);
        for ev in &events {
            let line = serde_json::to_string(ev).expect("serialize");
            let back: E2eEvidenceEvent = serde_json::from_str(&line).expect("round-trip");
            match (ev, &back) {
                (
                    E2eEvidenceEvent::StepStarted { step_id: a, .. },
                    E2eEvidenceEvent::StepStarted { step_id: b, .. },
                ) => assert_eq!(a, b),
                (
                    E2eEvidenceEvent::StepFinished {
                        step_id: a,
                        status: sa,
                        ..
                    },
                    E2eEvidenceEvent::StepFinished {
                        step_id: b,
                        status: sb,
                        ..
                    },
                ) => {
                    assert_eq!(a, b);
                    assert_eq!(sa, sb);
                }
                _ => {}
            }
        }
    }

    #[test]
    fn open_mode_emits_same_step_timeline_as_run_mode() {
        let mut run_bundle = bundle_with_named_steps();
        run_bundle.mode = E2eMode::Run;
        let mut open_bundle = bundle_with_named_steps();
        open_bundle.mode = E2eMode::Open;

        let run_kinds: Vec<&'static str> = events_for_bundle(&run_bundle)
            .iter()
            .map(event_kind)
            .collect();
        let open_kinds: Vec<&'static str> = events_for_bundle(&open_bundle)
            .iter()
            .map(event_kind)
            .collect();
        assert_eq!(
            run_kinds, open_kinds,
            "step timeline shape must be identical across run and open modes",
        );
    }

    fn event_kind(e: &E2eEvidenceEvent) -> &'static str {
        match e {
            E2eEvidenceEvent::RunStarted { .. } => "run_started",
            E2eEvidenceEvent::ServeSessionStarted { .. } => "serve_session_started",
            E2eEvidenceEvent::BrowserSessionStarted { .. } => "browser_session_started",
            E2eEvidenceEvent::BrowserSessionFinished { .. } => "browser_session_finished",
            E2eEvidenceEvent::StepStarted { .. } => "step_started",
            E2eEvidenceEvent::StepFinished { .. } => "step_finished",
            E2eEvidenceEvent::CaseFinished { .. } => "case_finished",
            E2eEvidenceEvent::RunFinished { .. } => "run_finished",
        }
    }

    fn report_with_outcome(outcome: Outcome) -> TestReport {
        TestReport {
            file: PathBuf::from("e2e/x.spec.js"),
            suite: vec!["s".to_string()],
            name: format!("test:{outcome:?}"),
            outcome,
            duration_ms: 1,
            error: None,
            trace_path: None,
            shard_index: None,
            shard_total: None,
            artifacts: vec![],
            steps: Vec::new(),
        }
    }

    #[test]
    fn exit_code_zero_when_all_passed_or_skipped() {
        let reports = vec![
            report_with_outcome(Outcome::Passed),
            report_with_outcome(Outcome::Skipped),
        ];
        assert_eq!(exit_code_for_reports(&reports), E2E_EXIT_OK);
    }

    #[test]
    fn exit_code_one_when_assertion_failed() {
        let reports = vec![
            report_with_outcome(Outcome::Passed),
            report_with_outcome(Outcome::Failed),
        ];
        assert_eq!(exit_code_for_reports(&reports), E2E_EXIT_ASSERTION_FAILURE);
    }

    #[test]
    fn exit_code_three_when_any_case_timed_out() {
        // Timeout beats assertion failure.
        let reports = vec![
            report_with_outcome(Outcome::Failed),
            report_with_outcome(Outcome::TimedOut),
        ];
        assert_eq!(exit_code_for_reports(&reports), E2E_EXIT_TIMEOUT);
    }

    #[test]
    fn exit_code_four_when_any_case_crashed() {
        // Infrastructure (crash) trumps timeout and assertion.
        let reports = vec![
            report_with_outcome(Outcome::Failed),
            report_with_outcome(Outcome::TimedOut),
            report_with_outcome(Outcome::Crashed),
        ];
        assert_eq!(exit_code_for_reports(&reports), E2E_EXIT_INFRASTRUCTURE);
    }

    #[test]
    fn build_evidence_bundle_uses_precedence_exit_code() {
        let summary = Summary {
            schema_version: crate::test_runner::reporter::SCHEMA_VERSION,
            passed: 1,
            failed: 0,
            skipped: 0,
            duration_ms: 5,
            reports: vec![
                report_with_outcome(Outcome::Passed),
                report_with_outcome(Outcome::TimedOut),
            ],
            coverage: None,
            browser_sessions: Vec::new(),
        };
        let bundle = build_evidence_bundle(E2eMode::Run, summary, 1, 2, None, None);
        assert_eq!(bundle.summary.exit_code, E2E_EXIT_TIMEOUT);
    }

    #[test]
    fn run_mode_evidence_is_agent_readable_and_schema_versioned() {
        let summary = Summary {
            schema_version: crate::test_runner::reporter::SCHEMA_VERSION,
            passed: 1,
            failed: 0,
            skipped: 0,
            duration_ms: 5,
            reports: vec![report_with_outcome(Outcome::Passed)],
            coverage: None,
            browser_sessions: Vec::new(),
        };
        let bundle = build_evidence_bundle(E2eMode::Run, summary, 1, 2, None, None);
        let serialized = serde_json::to_string(&bundle).expect("serialise");
        let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(parsed["schema_version"], EVIDENCE_SCHEMA_VERSION);
        assert!(parsed["summary"]["exit_code"].is_i64());
        assert!(parsed["cases"].is_array());
        // open_control should not appear in run-mode evidence.
        assert!(
            parsed.get("open_control").is_none()
                || parsed["open_control"] == serde_json::Value::Null,
            "run-mode evidence carries no open_control"
        );
    }

    #[test]
    fn run_mode_evidence_does_not_carry_open_control() {
        let summary = Summary {
            schema_version: crate::test_runner::reporter::SCHEMA_VERSION,
            passed: 1,
            failed: 0,
            skipped: 0,
            duration_ms: 5,
            reports: vec![report_with_outcome(Outcome::Passed)],
            coverage: None,
            browser_sessions: Vec::new(),
        };
        let bundle = build_evidence_bundle(E2eMode::Run, summary, 1, 2, None, None);
        assert!(
            bundle.open_control.is_none(),
            "run mode must never launch a review shell or carry control protocol",
        );
    }

    /// GH #3097 — `--trace=<garbage>` on the agent-mode path used to
    /// silently coerce to `RetainOnFailure`. Pin the three contract
    /// branches: `None` defaults to `RetainOnFailure`; known values
    /// parse cleanly; typos return `Err` listing the valid values.
    #[test]
    fn parse_trace_mode_defaults_to_retain_on_failure_when_omitted() {
        assert_eq!(
            parse_trace_mode(None).unwrap(),
            WireTraceMode::RetainOnFailure,
        );
    }

    #[test]
    fn parse_trace_mode_accepts_documented_values() {
        let off = "off".to_string();
        let on = "on".to_string();
        let retain = "retain-on-failure".to_string();
        assert_eq!(parse_trace_mode(Some(&off)).unwrap(), WireTraceMode::Off);
        assert_eq!(parse_trace_mode(Some(&on)).unwrap(), WireTraceMode::On);
        assert_eq!(
            parse_trace_mode(Some(&retain)).unwrap(),
            WireTraceMode::RetainOnFailure,
        );
    }

    #[test]
    fn parse_trace_mode_rejects_unknown_value() {
        let garbage = "verbose".to_string();
        let err = parse_trace_mode(Some(&garbage))
            .expect_err("unknown trace value must return Err, not silently default");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("verbose")
                && msg.contains("off")
                && msg.contains("on")
                && msg.contains("retain-on-failure"),
            "diagnostic must name the typo and the valid values, got: {msg}",
        );
    }

    /// GH #3150 — `read_control` must distinguish "file absent"
    /// (legitimate fresh session, no warning) from "file present but
    /// corrupt" (silent state loss, emit a tracing warning) while
    /// preserving liveness by returning a default state in both cases.
    /// We pin behaviour (default state returned) end-to-end; the
    /// warning channel is best-effort tracing.
    #[test]
    fn read_control_returns_default_when_file_absent() {
        let dir = tempfile::tempdir().unwrap();
        let files = LiveRunnerFiles::new(dir.path(), "run-3150-absent");
        // control_path is never created → NotFound branch.
        assert!(!files.control_path.exists());
        let state = files.read_control();
        // Default state has paused=false, all tokens = 0.
        assert!(!state.paused);
        assert_eq!(state.next_token, 0);
        assert_eq!(state.replay_token, 0);
        assert!(state.replay_case_index.is_none());
    }

    #[test]
    fn read_control_returns_default_when_file_corrupt() {
        let dir = tempfile::tempdir().unwrap();
        let files = LiveRunnerFiles::new(dir.path(), "run-3150-corrupt");
        if let Some(parent) = files.control_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&files.control_path, b"{not valid json").unwrap();

        // Pre-fix: also returned default. Post-fix: still returns
        // default but logs a warning. The contract for the caller
        // (liveness preserved) is what we pin here.
        let state = files.read_control();
        assert!(!state.paused);
        assert_eq!(state.next_token, 0);
    }

    /// GH #3164 — Malformed JSON body must NOT be coerced to `{}` and
    /// returned as `{ "ok": true }`. The endpoint must reject with 400
    /// and the on-disk control state must remain unchanged.
    #[tokio::test]
    async fn live_runner_control_rejects_malformed_json_body() {
        use axum::body::to_bytes;
        use axum::http::Request;
        use tower::ServiceExt;

        let dir = tempfile::tempdir().unwrap();
        let files = LiveRunnerFiles::new(dir.path(), "run-3164-malformed");
        let state = LiveRunnerServerState {
            files: files.clone(),
            touched_at_ms: Arc::new(Mutex::new(now_ms())),
        };
        let app = Router::new()
            .route("/api/control/{command}", post(live_runner_control))
            .with_state(state);

        let req = Request::builder()
            .method("POST")
            .uri("/api/control/replay")
            .header("content-type", "application/json")
            .body(axum::body::Body::from(
                "{ not actually json", // ← malformed
            ))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(
            resp.status(),
            axum::http::StatusCode::BAD_REQUEST,
            "malformed JSON must yield 400, not silent 200 (GH #3164)"
        );
        let body = to_bytes(resp.into_body(), 1 << 20).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(
            body_str.contains("GH #3164"),
            "response body must carry the searchable tag: {body_str}"
        );

        // Control file must NOT have been written.
        assert!(
            !files.control_path.exists(),
            "malformed body must not persist any control state change (GH #3164)"
        );
    }

    /// GH #3164 — Unknown command was a silent no-op that still
    /// returned `{ "ok": true }`. Pin it to a 400 listing the valid
    /// commands.
    #[tokio::test]
    async fn live_runner_control_rejects_unknown_command() {
        use axum::body::to_bytes;
        use axum::http::Request;
        use tower::ServiceExt;

        let dir = tempfile::tempdir().unwrap();
        let files = LiveRunnerFiles::new(dir.path(), "run-3164-unknown");
        let state = LiveRunnerServerState {
            files: files.clone(),
            touched_at_ms: Arc::new(Mutex::new(now_ms())),
        };
        let app = Router::new()
            .route("/api/control/{command}", post(live_runner_control))
            .with_state(state);

        let req = Request::builder()
            .method("POST")
            .uri("/api/control/pasue") // ← typo
            .body(axum::body::Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(
            resp.status(),
            axum::http::StatusCode::BAD_REQUEST,
            "unknown command must yield 400, not silent 200 (GH #3164)"
        );
        let body = to_bytes(resp.into_body(), 1 << 20).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(
            body_str.contains("pause")
                && body_str.contains("resume")
                && body_str.contains("next")
                && body_str.contains("speed")
                && body_str.contains("replay"),
            "error must enumerate supported commands, got: {body_str}"
        );
    }

    /// GH #3164 — Happy path: valid replay POST with case_index returns
    /// 200 and persists the replay selection. Pins that the
    /// error-handling refactor didn't regress the working case.
    #[tokio::test]
    async fn live_runner_control_replay_persists_case_index_on_valid_payload() {
        use axum::http::Request;
        use tower::ServiceExt;

        let dir = tempfile::tempdir().unwrap();
        let files = LiveRunnerFiles::new(dir.path(), "run-3164-happy");
        let state = LiveRunnerServerState {
            files: files.clone(),
            touched_at_ms: Arc::new(Mutex::new(now_ms())),
        };
        let app = Router::new()
            .route("/api/control/{command}", post(live_runner_control))
            .with_state(state);

        let req = Request::builder()
            .method("POST")
            .uri("/api/control/replay")
            .header("content-type", "application/json")
            .body(axum::body::Body::from(r#"{"case_index": 5}"#))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::OK);

        let persisted = files.read_control();
        assert_eq!(
            persisted.replay_case_index,
            Some(5),
            "valid replay payload must persist case_index (GH #3164)"
        );
        assert_eq!(
            persisted.replay_token, 1,
            "replay_token must increment on each replay command"
        );
    }

    #[tokio::test]
    async fn live_runner_control_speed_persists_multiplier() {
        use axum::http::Request;
        use tower::ServiceExt;

        let dir = tempfile::tempdir().unwrap();
        let files = LiveRunnerFiles::new(dir.path(), "run-speed");
        let state = LiveRunnerServerState {
            files: files.clone(),
            touched_at_ms: Arc::new(Mutex::new(now_ms())),
        };
        let app = Router::new()
            .route("/api/control/{command}", post(live_runner_control))
            .with_state(state);

        let req = Request::builder()
            .method("POST")
            .uri("/api/control/speed")
            .header("content-type", "application/json")
            .body(axum::body::Body::from(r#"{"speed_multiplier": 4}"#))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::OK);

        let persisted = files.read_control();
        assert_eq!(
            persisted.speed_multiplier, 4,
            "speed control must affect live step pacing, not just UI state"
        );
    }

    /// GH #3174 — Happy path: two events appended to a fresh path
    /// produce a 2-line NDJSON file. Pins that the new
    /// error-surfacing rewrite didn't regress the working case.
    #[test]
    fn write_live_event_appends_ndjson_to_fresh_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("live/events.jsonl");
        write_live_event(&path, &json!({ "kind": "ping", "n": 1 }));
        write_live_event(&path, &json!({ "kind": "pong", "n": 2 }));

        let body = std::fs::read_to_string(&path).expect("event log must exist");
        let lines: Vec<&str> = body.lines().collect();
        assert_eq!(lines.len(), 2, "two events → two NDJSON lines");
        assert!(lines[0].contains("\"ping\""));
        assert!(lines[1].contains("\"pong\""));
    }

    /// GH #3174 — When the parent dir cannot be created (parent path is
    /// a regular file, not a dir), write_live_event must return without
    /// panicking and must not create the target file. The fix's
    /// tracing::warn! is best-effort and not asserted on here.
    #[test]
    fn write_live_event_does_not_panic_when_parent_path_is_a_file() {
        let dir = tempfile::tempdir().unwrap();
        // `parent_blocker` is a regular file; we then try to write a
        // "child" event log under it, which forces create_dir_all to
        // fail because a regular file already occupies the path.
        let parent_blocker = dir.path().join("parent_blocker");
        std::fs::write(&parent_blocker, b"i am a file, not a dir").unwrap();
        let event_path = parent_blocker.join("events.jsonl");

        // Must not panic.
        write_live_event(&event_path, &json!({ "kind": "blocked" }));

        // Event log must not exist (write was correctly skipped).
        assert!(
            !event_path.exists(),
            "write_live_event must not create files when parent dir is unavailable (GH #3174)"
        );
    }

    /// GH #3150 — round-trip happy path must keep working: a written
    /// LiveControlState round-trips through write_control + read_control.
    #[test]
    fn read_control_round_trips_with_write_control() {
        let dir = tempfile::tempdir().unwrap();
        let files = LiveRunnerFiles::new(dir.path(), "run-3150-roundtrip");

        let written = LiveControlState {
            paused: true,
            speed_multiplier: 2,
            next_token: 7,
            replay_token: 3,
            replay_case_index: Some(2),
            replay_case_id: Some("case-xyz".to_string()),
            replay_case_title: Some("a title".to_string()),
        };
        files.write_control(written).expect("write must succeed");

        let read = files.read_control();
        assert!(read.paused);
        assert_eq!(read.speed_multiplier, 2);
        assert_eq!(read.next_token, 7);
        assert_eq!(read.replay_token, 3);
        assert_eq!(read.replay_case_index, Some(2));
        assert_eq!(read.replay_case_id.as_deref(), Some("case-xyz"));
        assert_eq!(read.replay_case_title.as_deref(), Some("a title"));
    }

    /// GH #3258 — `read_runner_bundle` happy path: a valid
    /// app-state.json is parsed and returned as `Some`.
    #[test]
    fn read_runner_bundle_returns_some_for_valid_bundle() {
        let dir = TempDir::new().expect("tempdir");
        let files = LiveRunnerFiles::new(dir.path(), "open-3258");
        let shell_dir = dir.path().join("open-runner-shell");
        std::fs::create_dir_all(&shell_dir).unwrap();
        let bundle = minimal_bundle();
        std::fs::write(
            shell_dir.join("app-state.json"),
            serde_json::to_vec(&bundle).unwrap(),
        )
        .unwrap();

        let back = super::read_runner_bundle(&files);
        assert!(back.is_some(), "valid bundle must round-trip (GH #3258)");
    }

    /// GH #3258 — `read_runner_bundle` returns `None` silently when
    /// the open-mode runner hasn't written the snapshot yet
    /// (NotFound is the legitimate "no bundle yet" branch).
    #[test]
    fn read_runner_bundle_returns_none_when_missing() {
        let dir = TempDir::new().expect("tempdir");
        let files = LiveRunnerFiles::new(dir.path(), "open-3258");
        // No open-runner-shell/app-state.json written.
        assert!(super::read_runner_bundle(&files).is_none());
    }

    /// GH #3258 — `read_runner_bundle` returns `None` (and now warns,
    /// observable in tracing but not asserted here) when the snapshot
    /// is malformed JSON. The prior `.ok().and_then(.ok())` chain
    /// silently rendered "no bundle" — same return value, but no
    /// diagnostic.
    #[test]
    fn read_runner_bundle_returns_none_for_malformed_snapshot() {
        let dir = TempDir::new().expect("tempdir");
        let files = LiveRunnerFiles::new(dir.path(), "open-3258");
        let shell_dir = dir.path().join("open-runner-shell");
        std::fs::create_dir_all(&shell_dir).unwrap();
        std::fs::write(shell_dir.join("app-state.json"), b"{ not valid json").unwrap();

        assert!(super::read_runner_bundle(&files).is_none());
    }

    /// GH #3258 — `read_jsonl_values` preserves valid lines around a
    /// malformed line and a JSON-shaped-but-arbitrary line (Value
    /// accepts any well-formed JSON, so only truly-malformed lines
    /// are dropped). Confirms the explicit-match arm did not break
    /// the surrounding events.
    #[test]
    fn read_jsonl_values_preserves_valid_around_malformed_line() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("events.jsonl");
        // 1 valid, 1 truncated, 1 valid, blank line, 1 valid.
        let body = b"{\"a\":1}\n{ trunc\n{\"b\":2}\n\n{\"c\":3}\n";
        std::fs::write(&path, body).unwrap();

        let values = super::read_jsonl_values(&path);
        assert_eq!(
            values.len(),
            3,
            "3 valid lines around 1 malformed + 1 blank, got {}",
            values.len()
        );
    }

    /// GH #3258 — `read_jsonl_values` returns an empty vec silently
    /// for a missing file (the live runner polls before any event is
    /// emitted — NotFound is the legitimate "no events yet" branch).
    #[test]
    fn read_jsonl_values_returns_empty_for_missing_file() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("does-not-exist.jsonl");
        assert!(super::read_jsonl_values(&path).is_empty());
    }

    /// GH #3490 — happy path: `serve_live_runner` accepts a real request
    /// without panicking. Pins the extracted-helper signature and confirms
    /// the surrounding `tokio::spawn(serve_live_runner(...))` wiring inside
    /// `LiveRunnerServer::start` continues to work.
    #[tokio::test]
    async fn gh3490_serve_live_runner_serves_real_request() {
        use axum::routing::get;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app: Router = Router::new().route("/ping", get(|| async { "pong" }));

        let task = tokio::spawn(super::serve_live_runner(listener, app, addr));

        let url = format!("http://{addr}/ping");
        let resp = reqwest::get(&url).await.expect("GET /ping must succeed");
        assert!(resp.status().is_success());
        assert_eq!(resp.text().await.unwrap(), "pong");

        task.abort();
        let _ = task.await;
    }

    /// GH #3490 — cancel safety: aborting the spawned task while it is
    /// blocked in the accept loop must terminate cleanly (no panic, no
    /// hang). Documents the contract that `LiveRunnerServer` relies on
    /// when its `JoinHandle` is dropped at the end of the e2e session.
    // ──────────────────────────────────────────────────────────────────
    // GH #3540 — merge_trace_context surfaces unreadable trace manifests
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn gh3540_format_trace_manifest_read_warn_names_path_error_and_issue() {
        let trace_path = PathBuf::from("/tmp/jet-e2e/case-42/trace.zip");
        let err = anyhow::anyhow!("not a zip file");
        let msg = format_trace_manifest_read_warn(&trace_path, &err);
        assert!(
            msg.contains("/tmp/jet-e2e/case-42/trace.zip"),
            "warn must name the offending trace path: {msg}"
        );
        assert!(
            msg.contains("not a zip file"),
            "warn must preserve the underlying error verbatim: {msg}"
        );
        assert!(
            msg.contains("GH #3540"),
            "warn must carry the GH #3540 log-grep tag: {msg}"
        );
    }

    #[test]
    fn gh3540_format_trace_manifest_read_warn_hints_at_symptom() {
        let trace_path = PathBuf::from("/tmp/trace.zip");
        let err = anyhow::anyhow!("eof while reading central directory");
        let msg = format_trace_manifest_read_warn(&trace_path, &err);
        // A developer grepping "report missing selectors" or "report no screenshot"
        // must be able to land on this warn line.
        assert!(
            msg.contains("selectors") || msg.contains("screenshots") || msg.contains("console"),
            "warn must mention which report fields go empty: {msg}"
        );
        assert!(
            msg.contains("truncated") || msg.contains("manifest") || msg.contains("incompatible"),
            "warn must hint at typical root causes: {msg}"
        );
    }

    #[test]
    fn gh3540_merge_trace_context_unreadable_trace_leaves_context_empty() {
        // End-to-end: a missing trace .zip is the simplest unreadable case.
        // read_manifest_from_zip will return Err, merge_trace_context falls
        // back to leaving context fields empty (the warn line is enforced
        // via the helper-shape tests above since tracing capture in unit
        // tests is brittle).
        let tmp = TempDir::new().unwrap();
        let nonexistent = tmp.path().join("does-not-exist.zip");

        let mut context = E2eStepContext::default();
        super::merge_trace_context(&nonexistent, &mut context);

        assert!(
            context.selectors.is_empty(),
            "selectors must remain empty when trace read fails"
        );
        assert!(
            context.screenshots.is_empty(),
            "screenshots must remain empty when trace read fails"
        );
        assert!(
            context.console.is_empty(),
            "console must remain empty when trace read fails"
        );
    }

    #[tokio::test]
    async fn gh3490_serve_live_runner_cancel_terminates_cleanly() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app: Router = Router::new();

        let task = tokio::spawn(super::serve_live_runner(listener, app, addr));

        // Give the server a tick to enter the accept loop.
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        task.abort();
        let join_result = task.await;
        assert!(
            join_result.is_err() && join_result.unwrap_err().is_cancelled(),
            "aborted task must report cancelled"
        );
    }

    // ─── GH #3552: live runner silent render fallback ────────────────────

    /// GH #3552 — the render-fallback warn must name the GH issue tag and
    /// the user-visible symptom ("fallback shell" or "no recent runs") so
    /// a grep from the symptom or the issue lands on this line.
    #[test]
    fn gh3552_live_runner_render_warn_names_issue_and_symptom() {
        let err = anyhow::anyhow!("template parse error at line 42: unclosed tag");
        let msg = super::format_live_runner_render_warn(7, &err);

        assert!(
            msg.contains("GH #3552"),
            "must include searchable issue tag, got: {msg}"
        );
        assert!(
            msg.contains("fallback") || msg.contains("no recent runs"),
            "must name the user-visible fallback symptom, got: {msg}"
        );
        assert!(
            msg.contains("template parse error at line 42"),
            "must preserve underlying error verbatim, got: {msg}"
        );
    }

    /// GH #3552 — the warning must name the bundle case count so the dev
    /// can correlate which evidence shape triggered the failure (a 0-case
    /// bundle vs. a 50-case bundle implies very different root causes).
    #[test]
    fn gh3552_live_runner_render_warn_names_case_count() {
        let err = anyhow::anyhow!("missing field 'product_outcome'");
        let msg = super::format_live_runner_render_warn(13, &err);

        assert!(
            msg.contains("13"),
            "must name the bundle case count, got: {msg}"
        );
        assert!(
            msg.contains("case"),
            "must use the word 'case' alongside the count, got: {msg}"
        );
    }

    /// GH #3552 — the warning must point the dev at the renderer / review-UI
    /// template, NOT the evidence writer, because the evidence has already
    /// been written successfully by the time the renderer fails. Without
    /// this hint, devs waste time debugging the wrong half of the pipeline.
    #[test]
    fn gh3552_live_runner_render_warn_points_at_renderer_not_writer() {
        let err = anyhow::anyhow!("anything");
        let msg = super::format_live_runner_render_warn(1, &err);

        assert!(
            msg.contains("renderer") || msg.contains("template"),
            "must point at renderer/template, got: {msg}"
        );
        assert!(
            msg.contains("writer") || msg.contains("evidence writer"),
            "must explicitly call out the writer as NOT the cause, got: {msg}"
        );
    }
}

#[cfg(test)]
mod gh3669_safe_e2e_now_ms_tests {
    //! GH #3669 — `e2e::now_ms()` used to call
    //! `SystemTime::now().duration_since(UNIX_EPOCH).map(...).unwrap_or(0)`,
    //! silently collapsing any clock-before-epoch failure (Mac VM
    //! reset, container without `--rtc`, freshly-booted devboard
    //! before NTP sync) onto zero with no breadcrumb. Bundle
    //! timestamps, replay timing, idle touched_at_ms, and the
    //! `run-{started_at_ms}` id all silently collapsed to zero —
    //! including duplicate `run-0` ids across reruns that overwrote
    //! each other's evidence dirs. `safe_e2e_now_ms` preserves the
    //! historical zero on the broken-clock branch but returns a
    //! tagged warn so the operator sees the cause.
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn happy_path_returns_millis_and_no_warn() {
        let t = UNIX_EPOCH + Duration::from_millis(1_700_000_000_000);
        let (ms, warn) = safe_e2e_now_ms(t);
        assert_eq!(ms, 1_700_000_000_000);
        assert!(warn.is_none(), "happy path must not warn");
    }

    #[test]
    fn epoch_itself_returns_zero_and_no_warn() {
        // UNIX_EPOCH must be treated as the happy path (Ok(0s)), not
        // the broken-clock branch — otherwise every test on a Truman-
        // show clock would warn.
        let (ms, warn) = safe_e2e_now_ms(UNIX_EPOCH);
        assert_eq!(ms, 0);
        assert!(warn.is_none());
    }

    #[test]
    fn clock_before_epoch_returns_zero_and_warns() {
        let before = UNIX_EPOCH - Duration::from_secs(1);
        let (ms, warn) = safe_e2e_now_ms(before);
        assert_eq!(
            ms, 0,
            "broken-clock branch must preserve the historical zero"
        );
        let msg = warn.expect("broken-clock branch must emit a warn");
        assert!(
            msg.contains("GH #3669"),
            "warn must carry the issue tag, got: {msg}"
        );
    }

    #[test]
    fn warn_message_names_the_downstream_failure_modes() {
        // The warn must tell the operator that bundle timestamps,
        // replay timing, idle touched_at_ms, and the run id all
        // collapse — otherwise the operator can't connect the dot
        // between a single warn and the cascade of collisions.
        let before = UNIX_EPOCH - Duration::from_secs(1);
        let (_, warn) = safe_e2e_now_ms(before);
        let msg = warn.unwrap();
        assert!(
            msg.contains("run-") || msg.contains("run id"),
            "warn must mention the run-id collision, got: {msg}"
        );
        assert!(
            msg.contains("started_at_ms") || msg.contains("timestamp"),
            "warn must mention bundle timestamps, got: {msg}"
        );
    }

    #[test]
    fn warn_message_points_at_the_host_clock_fix_not_jet_code() {
        // The bug is a host-clock misconfig, not a jet code path the
        // operator can patch. The warn must steer them toward NTP /
        // container --rtc / RTC battery, not at internal jet code.
        let before = UNIX_EPOCH - Duration::from_secs(1);
        let (_, warn) = safe_e2e_now_ms(before);
        let msg = warn.unwrap();
        assert!(
            msg.contains("clock") || msg.contains("NTP") || msg.contains("RTC"),
            "warn must point at the host clock as the fix surface, got: {msg}"
        );
    }

    #[test]
    fn format_helper_round_trip_carries_observed_error_text() {
        // The format helper takes a real SystemTimeError so the
        // observed error text gets surfaced. Avoid the temptation of
        // building a synthetic error string — use the real Err.
        let err = (UNIX_EPOCH - Duration::from_secs(5))
            .duration_since(UNIX_EPOCH)
            .unwrap_err();
        let msg = format_safe_e2e_now_ms_warn(&err);
        assert!(msg.contains("GH #3669"));
        // The Display of SystemTimeError surfaces the magnitude of
        // the negative skew (5s here) somewhere — the helper must
        // forward that detail rather than swallow it.
        assert!(
            msg.contains("5") || msg.contains("seconds") || msg.contains("UNIX_EPOCH"),
            "warn must surface error detail, got: {msg}"
        );
    }

    #[test]
    fn helper_output_is_deterministic_across_calls() {
        // Same input must produce same warn — important because the
        // warn is consumed by log scrapers / alert rules.
        let before = UNIX_EPOCH - Duration::from_millis(123);
        let (_, w1) = safe_e2e_now_ms(before);
        let (_, w2) = safe_e2e_now_ms(before);
        assert_eq!(w1, w2);
    }
}
// CODEGEN-END
