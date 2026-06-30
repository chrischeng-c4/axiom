// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
// CODEGEN-BEGIN
//! Node.js worker process: type-strips a spec file, embeds the worker
//! runtime shim, and reads NDJSON events back from stdout.
//!
//! Phase 3 additions:
//! - `handle_expect_request` — dispatches `WireRequest` (QueryText/IsVisible/Screenshot)
//!   to the browser layer and writes a `WireResponse` back over stdin.
//! - `load_or_write_snapshot` — PNG snapshot read/write/compare for `toMatchSnapshot`.

use crate::browser::context::BrowserContext;
use crate::browser::page::Page;
use crate::browser::{Browser, LaunchOptions};
use crate::cdp_driver::{dispatch_page_request, parse_page_request, write_page_response};
use crate::test_runner::config::{LiveE2eConfig, RunnerConfig};
use crate::test_runner::discovery::SpecFile;
use crate::test_runner::reporter::{
    BrowserSessionReport, MultiReporter, Outcome, Summary, TestReport, TestStepReport,
};
use crate::test_runner::wire::{
    self, MatcherDiff, TestOutcome, WireRequest, WireResponse, WireTraceMode, WorkerEvent,
};
use crate::trace::buffer::{commit_trace, TraceBuffer};
use crate::trace::manifest::TraceOutcome;
use crate::transform::{TransformOptions, Transformer};
use anyhow::{Context, Result};
use base64::Engine as _;
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, Command};
use tokio::sync::Mutex;

/// Worker runtime source — embedded into the binary.
const WORKER_RUNTIME: &str = include_str!("../../data/runtime/test/index.js");

/// Worker runtime source for the page shim — embedded into the binary.
const PAGE_SHIM: &str = include_str!("../../data/runtime/test/page.js");

/// Matchers shim — polling Locator/Page matchers module imported by index.mjs.
const MATCHERS_SHIM: &str = include_str!("../../data/runtime/test/matchers.js");

/// Run a single spec file to completion. Returns a partial Summary for this
/// file (aggregation happens in `test_runner::run`).
///
/// Browser lifecycle:
/// - If any test in the spec destructures `page`, the fixture registry sends a
///   `new_page` PageRequest before the test body runs. The Rust host lazily
///   launches Chromium on the first `new_page` request and keeps it alive for
///   the remainder of the spec. Visual open mode can instead connect to the
///   already-open review Chrome window and create the case page as a tab.
/// - Browser is closed after the worker loop exits (worker teardown).
///
// @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R5
pub async fn run_spec(
    spec: &SpecFile,
    config: &RunnerConfig,
    reporter: &MultiReporter,
) -> Result<Summary> {
    // 1. Transform the spec file (TS → JS). Skip type-stripping for
    //    already-JS files, but still normalize Jet's virtual test module.
    let transformed = transform_spec(&spec.path).context("Failed to type-strip spec")?;

    // 2. Write transformed spec + boot shim to a temp dir. The runtime is
    //    installed as a `node_modules/@jet/test/` package so specs (migrated
    //    off `@playwright/test` in Phase 5b) resolve the bare specifier via
    //    Node's standard ESM resolver.
    let tmp = tempfile::tempdir().context("Failed to create worker temp dir")?;
    let boot_path = tmp.path().join("__jet_boot.mjs");

    let shim_dir = tmp.path().join("node_modules").join("@jet").join("test");
    std::fs::create_dir_all(&shim_dir).context("Failed to create @jet/test shim dir")?;
    std::fs::write(
        shim_dir.join("package.json"),
        r#"{"name":"@jet/test","type":"module","main":"./index.mjs"}"#,
    )?;
    std::fs::write(shim_dir.join("index.mjs"), WORKER_RUNTIME)?;
    // Write page.js shim alongside index.mjs so it can be imported via "./page.js".
    std::fs::write(shim_dir.join("page.js"), PAGE_SHIM)?;
    // Write matchers.js shim so index.mjs's "./matchers.js" import resolves.
    std::fs::write(shim_dir.join("matchers.js"), MATCHERS_SHIM)?;

    // P4.5 Playwright compat shim — tests that haven't migrated yet import
    // from "@playwright/test". This shim re-exports every public symbol
    // from @jet/test so the common cases resolve without touching the
    // test's import lines.
    // @spec .aw/tech-design/projects/jet/logic/playwright-compat-shim.md#C1
    let pw_shim_dir = tmp
        .path()
        .join("node_modules")
        .join("@playwright")
        .join("test");
    std::fs::create_dir_all(&pw_shim_dir).context("Failed to create @playwright/test shim dir")?;
    std::fs::write(
        pw_shim_dir.join("package.json"),
        r#"{"name":"@playwright/test","type":"module","main":"./index.mjs"}"#,
    )?;
    std::fs::write(
        pw_shim_dir.join("index.mjs"),
        // Named + default re-export covers both `import { test } from "@playwright/test"`
        // and `import pw from "@playwright/test"` styles.
        r#"export * from "@jet/test";
import * as __jet from "@jet/test";
export default __jet;
"#,
    )?;

    let modules_dir = tmp.path().join("__jet_modules");
    let spec_path = {
        let mut emitter = TempModuleGraphEmitter::new(&modules_dir);
        emitter
            .emit(&spec.path, Some(transformed))
            .context("Failed to emit transformed spec module graph")?
    };
    std::fs::write(&boot_path, build_boot(&spec_path, spec, config))?;

    // 3. Spawn node with the boot script.
    //    stdin is piped so the Rust host can send WireResponse messages back
    //    for DOM-integrated matcher RPC calls (Phase 3) and PageResponse
    //    messages for page action RPCs (Phase 5).
    let mut child = Command::new("node")
        .arg(&boot_path)
        .current_dir(&config.project_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn node worker — is node on PATH?")?;

    let stdout = child
        .stdout
        .take()
        .context("Worker stdout was not captured")?;
    let stderr = child
        .stderr
        .take()
        .context("Worker stderr was not captured")?;
    // Stdin writer for sending WireResponse and PageResponse messages back.
    let stdin_writer: Option<Arc<Mutex<ChildStdin>>> =
        child.stdin.take().map(|stdin| Arc::new(Mutex::new(stdin)));

    // Snapshot directory slug: filename stem with non-alphanumerics collapsed.
    let spec_slug = spec_slug_for(&spec.path);
    let update_snapshots = config.update_snapshots;
    let headless = config.headless;

    // Active pages map: page_id (CDP target ID) → Page.
    // Shared between the new_page handler and the page action dispatcher.
    // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R5
    let pages: Arc<Mutex<HashMap<String, Arc<Page>>>> = Arc::new(Mutex::new(HashMap::new()));

    // Active browser-contexts map: browserContextId → BrowserContext. Only
    // user-created contexts live here — the implicit default context lives
    // inside `Browser`. Shared with the context handlers below.
    // @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R6
    let contexts: Arc<Mutex<HashMap<String, BrowserContext>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // Lazily-launched browser. Populated on the first `new_page` request.
    // Closed in the teardown block below.
    // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R5
    let browser: Arc<Mutex<Option<Browser>>> = Arc::new(Mutex::new(None));
    let browser_session: Arc<Mutex<Option<BrowserSessionReport>>> = Arc::new(Mutex::new(None));
    let mut network_event_pump_started = false;

    // 4. Drive the worker: read NDJSON from stdout, tail stderr into console
    //    events.
    //
    //    Each stdout line is tried in order:
    //      a) WorkerEvent (lifecycle, console, plan, test_start/end)
    //      b) PageRequest (page action RPC — Phase 5)
    //      c) WireRequest (DOM-matcher RPC — Phase 3)
    //      d) Unrecognised → surface as console output
    let reporter_ref = reporter;
    let mut summary = Summary::default();

    let stderr_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        let mut captured = Vec::new();
        while let Ok(Some(line)) = lines.next_line().await {
            captured.push(line);
        }
        captured
    });

    let mut stdout_lines = BufReader::new(stdout).lines();
    let mut checkpoint_state = LiveCheckpointState::default();
    let mut current_live_case_id: Option<String> = None;
    let mut current_live_case_title: Option<String> = None;
    let mut current_live_step_index: u64 = 0;
    // Per-test trace buffer + its test id. Populated on `TestStart` when
    // `config.trace.is_active()`, consumed on the matching `TestEnd`.
    // Out-of-order `TestStart` / `TestEnd` would surface here as a
    // stale buffer for the previous test — we just drop in that case
    // because the worker emits start/end strictly paired.
    // @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R1
    let mut current_trace: Option<(String, TraceBuffer)> = None;
    let mut step_reports_by_test: HashMap<String, Vec<TestStepReport>> = HashMap::new();
    while let Some(line) = stdout_lines.next_line().await? {
        // a) Try WorkerEvent first.
        if let Some(event) = wire::parse_line(&line) {
            reporter_ref.on_event(spec, &event);
            append_worker_live_event(config.live_e2e.as_ref(), spec, &event);
            match &event {
                WorkerEvent::TestStart { id, suite, name } => {
                    current_live_case_id = Some(id.clone());
                    current_live_case_title = Some(test_title_for_live(suite, name));
                    current_live_step_index = 0;
                    if config.trace.is_active() {
                        let title = test_title_for_live(suite, name);
                        let buf =
                            TraceBuffer::new(id.clone(), spec.path.display().to_string(), title);
                        current_trace = Some((id.clone(), buf));
                    } else {
                        current_trace = None;
                    }
                }
                WorkerEvent::TestEnd { .. } => {
                    current_live_case_id = None;
                    current_live_case_title = None;
                }
                WorkerEvent::StepEnd {
                    test_id,
                    step_id,
                    title,
                    outcome,
                    duration_ms,
                    error,
                    parent_step_id,
                } => {
                    step_reports_by_test
                        .entry(test_id.clone())
                        .or_default()
                        .push(TestStepReport {
                            id: step_id.clone(),
                            title: title.clone(),
                            outcome: outcome_from_wire(*outcome),
                            duration_ms: *duration_ms,
                            parent_step_id: parent_step_id.clone(),
                            error: error.clone().map(test_error_from_wire),
                        });
                }
                _ => {}
            }
            if let WorkerEvent::TestEnd {
                id,
                suite,
                name,
                outcome,
                duration_ms,
                error,
                artifacts,
                ..
            } = event
            {
                match outcome {
                    TestOutcome::Passed => summary.passed += 1,
                    TestOutcome::Failed => summary.failed += 1,
                    TestOutcome::Skipped => summary.skipped += 1,
                    TestOutcome::TimedOut => summary.failed += 1,
                }
                summary.duration_ms += duration_ms;
                // Commit the per-test trace buffer (if any) and surface
                // the on-disk path. Only the buffer matching the closing
                // test id is consumed; if the `TestStart`/`TestEnd` ids
                // mismatch we discard rather than misattribute.
                // @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R11
                let trace_buf = match current_trace.take() {
                    Some((buf_id, buf)) if buf_id == id => Some(buf),
                    _ => None,
                };
                let trace_path = commit_trace_for_report(
                    trace_buf,
                    outcome,
                    config.trace,
                    &config.trace_dir,
                    &spec_slug,
                    &id,
                )
                .unwrap_or_else(|e| {
                    eprintln!("[jet test] failed to commit trace for {id}: {e:#}");
                    None
                });
                // @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R4
                summary.reports.push(TestReport {
                    file: spec.path.clone(),
                    suite,
                    name,
                    outcome: outcome_from_wire(outcome),
                    duration_ms,
                    // @spec enhancement-html-reporter-for-native-test-runner-spec#R3
                    // @spec #2610 — attach source location parsed from stack
                    error: error.map(test_error_from_wire),
                    trace_path,
                    shard_index: None,
                    shard_total: None,
                    // @spec .aw/tech-design/projects/jet/logic/auto-artifacts.md#A1
                    artifacts: artifacts
                        .into_iter()
                        .map(std::path::PathBuf::from)
                        .collect(),
                    steps: step_reports_by_test.remove(&id).unwrap_or_default(),
                });
            }
            continue;
        }

        // b) Try PageRequest (Phase 5 page action RPC).
        // `NewPage`, `NewContext`, `CloseContext`, `ContextNewPage` are handled
        // here because they need access to the `Browser` / contexts map rather
        // than a per-page dispatch context.
        if let Some(page_req) = parse_page_request(&line) {
            use crate::cdp_driver::{PageRequest, PageResponse};
            let response = match &page_req {
                PageRequest::NewPage { req_id } => {
                    // Lazily launch the browser on the first new_page request.
                    // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R5
                    let req_id = *req_id;
                    match ensure_browser(
                        &browser,
                        &browser_session,
                        &spec.path,
                        headless,
                        config.browser_executable.clone(),
                        config.browser_ws_url.clone(),
                    )
                    .await
                    {
                        Err(e) => PageResponse::Error {
                            req_id,
                            message: format!("browser launch failed: {e}"),
                        },
                        Ok(()) => {
                            start_network_event_pump_if_needed(
                                &browser,
                                &pages,
                                stdin_writer.clone(),
                                &mut network_event_pump_started,
                            )
                            .await;
                            let browser_guard = browser.lock().await;
                            let browser_ref = browser_guard.as_ref().unwrap();
                            match browser_ref.new_page().await {
                                Ok(page) => match page.enable_network_events().await {
                                    Ok(()) => {
                                        let page_id = page.target_id().to_string();
                                        pages.lock().await.insert(page_id.clone(), Arc::new(page));
                                        PageResponse::NewPageResult { req_id, page_id }
                                    }
                                    Err(e) => PageResponse::Error {
                                        req_id,
                                        message: format!("Network.enable failed: {e}"),
                                    },
                                },
                                Err(e) => PageResponse::Error {
                                    req_id,
                                    message: format!("browser new_page failed: {e}"),
                                },
                            }
                        }
                    }
                }

                // @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R6
                PageRequest::NewContext { req_id } => {
                    let req_id = *req_id;
                    match ensure_browser(
                        &browser,
                        &browser_session,
                        &spec.path,
                        headless,
                        config.browser_executable.clone(),
                        config.browser_ws_url.clone(),
                    )
                    .await
                    {
                        Err(e) => PageResponse::Error {
                            req_id,
                            message: format!("browser launch failed: {e}"),
                        },
                        Ok(()) => {
                            start_network_event_pump_if_needed(
                                &browser,
                                &pages,
                                stdin_writer.clone(),
                                &mut network_event_pump_started,
                            )
                            .await;
                            let browser_guard = browser.lock().await;
                            let browser_ref = browser_guard.as_ref().unwrap();
                            match browser_ref.new_context().await {
                                Ok(ctx) => {
                                    let context_id = ctx.id().to_string();
                                    contexts.lock().await.insert(context_id.clone(), ctx);
                                    PageResponse::ContextResult { req_id, context_id }
                                }
                                Err(e) => PageResponse::Error {
                                    req_id,
                                    message: format!("browser new_context failed: {e}"),
                                },
                            }
                        }
                    }
                }

                // @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R6
                PageRequest::ContextNewPage { req_id, context_id } => {
                    let req_id = *req_id;
                    start_network_event_pump_if_needed(
                        &browser,
                        &pages,
                        stdin_writer.clone(),
                        &mut network_event_pump_started,
                    )
                    .await;
                    let contexts_guard = contexts.lock().await;
                    match contexts_guard.get(context_id) {
                        None => PageResponse::Error {
                            req_id,
                            message: format!("unknown browserContextId: {context_id}"),
                        },
                        Some(ctx) => match ctx.new_page().await {
                            Ok(page) => match page.enable_network_events().await {
                                Ok(()) => {
                                    let page_id = page.target_id().to_string();
                                    drop(contexts_guard);
                                    pages.lock().await.insert(page_id.clone(), Arc::new(page));
                                    PageResponse::NewPageResult { req_id, page_id }
                                }
                                Err(e) => PageResponse::Error {
                                    req_id,
                                    message: format!("Network.enable failed: {e}"),
                                },
                            },
                            Err(e) => PageResponse::Error {
                                req_id,
                                message: format!("context.new_page failed: {e}"),
                            },
                        },
                    }
                }

                // P3.2 storage-state variants — routed against the contexts map.
                // @spec .aw/tech-design/projects/jet/logic/storage-state.md#S1..S5
                PageRequest::ContextCookies { req_id, context_id } => {
                    let req_id = *req_id;
                    let guard = contexts.lock().await;
                    match guard.get(context_id) {
                        None => PageResponse::Error {
                            req_id,
                            message: format!("unknown browserContextId: {context_id}"),
                        },
                        Some(ctx) => match ctx.cookies().await {
                            Ok(cookies) => PageResponse::StorageStateResult {
                                req_id,
                                value: serde_json::Value::Array(cookies),
                            },
                            Err(e) => PageResponse::Error {
                                req_id,
                                message: format!("context.cookies failed: {e}"),
                            },
                        },
                    }
                }
                PageRequest::ContextAddCookies {
                    req_id,
                    context_id,
                    cookies,
                } => {
                    let req_id = *req_id;
                    let guard = contexts.lock().await;
                    match guard.get(context_id) {
                        None => PageResponse::Error {
                            req_id,
                            message: format!("unknown browserContextId: {context_id}"),
                        },
                        Some(ctx) => {
                            let arr = cookies.as_array().cloned().unwrap_or_default();
                            match ctx.add_cookies(arr).await {
                                Ok(()) => PageResponse::Ok { req_id },
                                Err(e) => PageResponse::Error {
                                    req_id,
                                    message: format!("context.add_cookies failed: {e}"),
                                },
                            }
                        }
                    }
                }
                PageRequest::ContextClearCookies { req_id, context_id } => {
                    let req_id = *req_id;
                    let guard = contexts.lock().await;
                    match guard.get(context_id) {
                        None => PageResponse::Error {
                            req_id,
                            message: format!("unknown browserContextId: {context_id}"),
                        },
                        Some(ctx) => match ctx.clear_cookies().await {
                            Ok(()) => PageResponse::Ok { req_id },
                            Err(e) => PageResponse::Error {
                                req_id,
                                message: format!("context.clear_cookies failed: {e}"),
                            },
                        },
                    }
                }
                PageRequest::ContextStorageState { req_id, context_id } => {
                    let req_id = *req_id;
                    let guard = contexts.lock().await;
                    match guard.get(context_id) {
                        None => PageResponse::Error {
                            req_id,
                            message: format!("unknown browserContextId: {context_id}"),
                        },
                        Some(ctx) => match ctx.storage_state().await {
                            Ok(state) => PageResponse::StorageStateResult {
                                req_id,
                                value: state,
                            },
                            Err(e) => PageResponse::Error {
                                req_id,
                                message: format!("context.storage_state failed: {e}"),
                            },
                        },
                    }
                }
                PageRequest::ContextSetStorageState {
                    req_id,
                    context_id,
                    state,
                } => {
                    let req_id = *req_id;
                    let guard = contexts.lock().await;
                    match guard.get(context_id) {
                        None => PageResponse::Error {
                            req_id,
                            message: format!("unknown browserContextId: {context_id}"),
                        },
                        Some(ctx) => match ctx.set_storage_state(state).await {
                            Ok(()) => PageResponse::Ok { req_id },
                            Err(e) => PageResponse::Error {
                                req_id,
                                message: format!("context.set_storage_state failed: {e}"),
                            },
                        },
                    }
                }

                // @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R6
                PageRequest::CloseContext { req_id, context_id } => {
                    let req_id = *req_id;
                    let maybe_ctx = contexts.lock().await.remove(context_id);
                    match maybe_ctx {
                        None => PageResponse::Error {
                            req_id,
                            message: format!("unknown browserContextId: {context_id}"),
                        },
                        Some(ctx) => match ctx.close().await {
                            Ok(()) => PageResponse::Ok { req_id },
                            Err(e) => PageResponse::Error {
                                req_id,
                                message: format!("context.close failed: {e}"),
                            },
                        },
                    }
                }

                other => {
                    // Route to the active page by page_id.
                    let page_id = page_req_id_str(other).map(|s| s.to_string());
                    // Extract close page_id before consuming page_req.
                    let close_page_id = if let PageRequest::Close { page_id: pid, .. } = other {
                        Some(pid.clone())
                    } else {
                        None
                    };
                    let live_step = live_step_from_page_request(other);
                    append_page_live_event(
                        config.live_e2e.as_ref(),
                        spec,
                        "page_action_started",
                        &live_step,
                        None,
                        current_live_case_id.as_deref(),
                        current_live_case_title.as_deref(),
                        current_live_step_index,
                    );
                    wait_for_live_checkpoint(
                        config.live_e2e.as_ref(),
                        &mut checkpoint_state,
                        current_live_case_id.as_deref().unwrap_or(""),
                        &live_step.title(),
                    )
                    .await;
                    current_live_step_index = current_live_step_index.saturating_add(1);
                    let pages_guard = pages.lock().await;
                    let page_opt = page_id.as_deref().and_then(|id| pages_guard.get(id));
                    if let (Some(page), Some(selector)) = (
                        if config.live_e2e.is_some() {
                            page_opt
                        } else {
                            None
                        },
                        live_step.selector.as_deref(),
                    ) {
                        let _ = highlight_selector_for_live_e2e(page.as_ref(), selector).await;
                    }
                    let req_id = page_req_id(&page_req);
                    let timeout_ms = page_req_timeout_ms(&page_req, config.timeout_ms);
                    let rpc_timeout = std::time::Duration::from_millis(timeout_ms.max(1));
                    let resp = match tokio::time::timeout(
                        rpc_timeout,
                        dispatch_page_request(page_req, page_opt.map(|p| p.as_ref())),
                    )
                    .await
                    {
                        Ok(resp) => resp,
                        Err(_) => PageResponse::Error {
                            req_id,
                            message: format_page_rpc_timeout_error(&live_step.action, timeout_ms),
                        },
                    };
                    let status = if matches!(resp, PageResponse::Error { .. }) {
                        "failed"
                    } else {
                        "passed"
                    };
                    append_page_live_event(
                        config.live_e2e.as_ref(),
                        spec,
                        "page_action_finished",
                        &live_step,
                        Some(status),
                        current_live_case_id.as_deref(),
                        current_live_case_title.as_deref(),
                        current_live_step_index.saturating_sub(1),
                    );
                    if let Some(live) = config.live_e2e.as_ref() {
                        let delay_ms = live_step_delay_ms(live);
                        if delay_ms > 0 {
                            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                        }
                    }
                    drop(pages_guard);
                    // Remove the page from the map when Close is received.
                    if let Some(pid) = close_page_id {
                        pages.lock().await.remove(&pid);
                    }
                    resp
                }
            };
            if let Some(writer) = stdin_writer.as_ref() {
                let mut writer = writer.lock().await;
                if let Err(err) = write_page_response(&mut *writer, response).await {
                    tracing::warn!(
                        target: "jet::test_runner::worker",
                        rpc_kind = "PageResponse",
                        error = %err,
                        "{}",
                        format_worker_stdin_write_warn("PageResponse", &err)
                    );
                }
            }
            continue;
        }

        // c) Try WireRequest (DOM-matcher RPC from the worker — Phase 3).
        if let Some(req) = wire::parse_request(&line) {
            if let WireRequest::LiveCheckpoint {
                req_id,
                test_id,
                title,
            } = req
            {
                wait_for_live_checkpoint(
                    config.live_e2e.as_ref(),
                    &mut checkpoint_state,
                    &test_id,
                    &title,
                )
                .await;
                if let Some(writer) = stdin_writer.as_ref() {
                    let mut writer = writer.lock().await;
                    if let Err(err) =
                        write_response(&mut *writer, WireResponse::LiveCheckpointResult { req_id })
                            .await
                    {
                        tracing::warn!(
                            target: "jet::test_runner::worker",
                            rpc_kind = "WireResponse::LiveCheckpointResult",
                            error = %err,
                            "{}",
                            format_worker_stdin_write_warn(
                                "WireResponse::LiveCheckpointResult",
                                &err,
                            )
                        );
                    }
                }
                continue;
            }
            // Use the first active page for DOM-matcher requests (page_id field
            // in WireRequest maps to whichever page is current — best-effort).
            let pages_guard = pages.lock().await;
            let active_page = pages_guard.values().next();
            let response = handle_expect_request(
                req,
                active_page.map(|page| page.as_ref()),
                &spec.path,
                &spec_slug,
                update_snapshots,
            )
            .await;
            drop(pages_guard);
            if let Some(writer) = stdin_writer.as_ref() {
                let mut writer = writer.lock().await;
                if let Err(err) = write_response(&mut *writer, response).await {
                    tracing::warn!(
                        target: "jet::test_runner::worker",
                        rpc_kind = "WireResponse",
                        error = %err,
                        "{}",
                        format_worker_stdin_write_warn("WireResponse", &err)
                    );
                }
            }
            continue;
        }

        // d) Unrecognised line — surface as console output.
        reporter_ref.on_event(
            spec,
            &WorkerEvent::Console {
                stream: wire::ConsoleStream::Stdout,
                message: line,
            },
        );
    }

    // 5. Wait for the process to exit.
    let status = child.wait().await.context("Worker process wait failed")?;
    let stderr_out = stderr_task.await.unwrap_or_default();

    // 6. Worker teardown: close browser if one was launched.
    // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R5
    if let Some(browser) = browser.lock().await.take() {
        let close_ok = browser.close().await.is_ok();
        if let Some(session) = browser_session.lock().await.as_mut() {
            session.close(close_ok, now_ms());
        }
    }
    if let Some(session) = browser_session.lock().await.take() {
        summary.browser_sessions.push(session);
    }

    if !status.success() && summary.reports.is_empty() {
        // Worker crashed without emitting any TestEnd — surface stderr tail
        // as a synthetic fatal report.
        let tail = stderr_out.join("\n");
        summary.failed += 1;
        summary.reports.push(TestReport {
            file: spec.path.clone(),
            suite: Vec::new(),
            name: "<worker crash>".to_string(),
            outcome: Outcome::Crashed,
            duration_ms: 0,
            error: Some(crate::test_runner::reporter::TestError {
                message: if tail.is_empty() {
                    format!("worker exited with {status}")
                } else {
                    tail.clone()
                },
                stack: None,
                diff: None,
                source_location: None,
            }),
            trace_path: None,
            shard_index: None,
            shard_total: None,
            artifacts: Vec::new(),
            steps: Vec::new(),
        });
    }

    Ok(summary)
}

#[derive(Default)]
struct LiveCheckpointState {
    next_token_seen: u64,
}

#[derive(Debug)]
struct LivePageStep {
    action: String,
    page_id: Option<String>,
    selector: Option<String>,
    url: Option<String>,
    html: Option<String>,
    value: Option<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
impl LivePageStep {
    fn title(&self) -> String {
        match (&self.selector, &self.url) {
            (Some(selector), _) => format!("{} {}", self.action, selector),
            (_, Some(url)) => format!("{} {}", self.action, url),
            (_, _) if self.value.is_some() && self.action == "evaluate" => {
                format!(
                    "{} {}",
                    self.action,
                    self.value.as_deref().unwrap_or_default()
                )
            }
            _ => self.action.clone(),
        }
    }
}

fn append_worker_live_event(live: Option<&LiveE2eConfig>, spec: &SpecFile, event: &WorkerEvent) {
    let Some(live) = live else {
        return;
    };
    let value = match event {
        WorkerEvent::Plan { file, tests } => json!({
            "kind": "plan",
            "ts_ms": now_ms(),
            "spec": spec.path,
            "file": file,
            "tests": tests,
        }),
        WorkerEvent::TestStart { id, suite, name } => json!({
            "kind": "case_started",
            "ts_ms": now_ms(),
            "spec": spec.path,
            "case_id": id,
            "suite": suite,
            "name": name,
            "title": test_title_for_live(suite, name),
        }),
        WorkerEvent::StepStart {
            test_id,
            step_id,
            title,
            parent_step_id,
        } => json!({
            "kind": "step_started",
            "ts_ms": now_ms(),
            "spec": spec.path,
            "case_id": test_id,
            "step_id": step_id,
            "title": title,
            "parent_step_id": parent_step_id,
        }),
        WorkerEvent::StepEnd {
            test_id,
            step_id,
            title,
            outcome,
            duration_ms,
            error,
            parent_step_id,
        } => json!({
            "kind": "step_finished",
            "ts_ms": now_ms(),
            "spec": spec.path,
            "case_id": test_id,
            "step_id": step_id,
            "title": title,
            "outcome": format!("{:?}", outcome).to_lowercase(),
            "status": format!("{:?}", outcome).to_lowercase(),
            "duration_ms": duration_ms,
            "error": error,
            "parent_step_id": parent_step_id,
        }),
        WorkerEvent::TestEnd {
            id,
            suite,
            name,
            outcome,
            duration_ms,
            error,
            artifacts,
            ..
        } => json!({
            "kind": "case_finished",
            "ts_ms": now_ms(),
            "spec": spec.path,
            "case_id": id,
            "suite": suite,
            "name": name,
            "title": test_title_for_live(suite, name),
            "outcome": format!("{:?}", outcome).to_lowercase(),
            "duration_ms": duration_ms,
            "error": error,
            "artifacts": artifacts,
        }),
        WorkerEvent::Console { stream, message } => json!({
            "kind": "console",
            "ts_ms": now_ms(),
            "spec": spec.path,
            "stream": format!("{:?}", stream).to_lowercase(),
            "message": message,
        }),
        WorkerEvent::Fatal { message } => json!({
            "kind": "fatal",
            "ts_ms": now_ms(),
            "spec": spec.path,
            "message": message,
        }),
    };
    append_jsonl(&live.event_log, &value);
}

fn append_page_live_event(
    live: Option<&LiveE2eConfig>,
    spec: &SpecFile,
    kind: &str,
    step: &LivePageStep,
    status: Option<&str>,
    case_id: Option<&str>,
    case_title: Option<&str>,
    step_index: u64,
) {
    let Some(live) = live else {
        return;
    };
    let value = json!({
        "kind": kind,
        "ts_ms": now_ms(),
        "spec": spec.path,
        "action": step.action,
        "page_id": step.page_id,
        "selector": step.selector,
        "url": step.url,
        "html": step.html,
        "value": step.value,
        "status": status,
        "case_id": case_id,
        "case_title": case_title,
        "step_id": format!("cmd-{step_index:04}"),
        "title": step.title(),
    });
    append_jsonl(&live.event_log, &value);
}

fn live_step_from_page_request(req: &crate::cdp_driver::PageRequest) -> LivePageStep {
    use crate::cdp_driver::PageRequest;
    match req {
        PageRequest::Goto { page_id, url, .. } => LivePageStep {
            action: "goto".to_string(),
            page_id: Some(page_id.clone()),
            selector: None,
            url: Some(url.clone()),
            html: None,
            value: None,
        },
        PageRequest::Click {
            page_id, selector, ..
        } => LivePageStep {
            action: "click".to_string(),
            page_id: Some(page_id.clone()),
            selector: Some(selector.clone()),
            url: None,
            html: None,
            value: None,
        },
        PageRequest::Fill {
            page_id,
            selector,
            value,
            ..
        } => LivePageStep {
            action: "fill".to_string(),
            page_id: Some(page_id.clone()),
            selector: Some(selector.clone()),
            url: None,
            html: None,
            value: Some(value.clone()),
        },
        PageRequest::WaitForSelector {
            page_id, selector, ..
        } => LivePageStep {
            action: "wait_for_selector".to_string(),
            page_id: Some(page_id.clone()),
            selector: Some(selector.clone()),
            url: None,
            html: None,
            value: None,
        },
        PageRequest::SetContent { page_id, html, .. } => LivePageStep {
            action: "set_content".to_string(),
            page_id: Some(page_id.clone()),
            selector: None,
            url: None,
            html: Some(html.clone()),
            value: None,
        },
        PageRequest::Evaluate {
            page_id,
            expression,
            ..
        } => LivePageStep {
            action: "evaluate".to_string(),
            page_id: Some(page_id.clone()),
            selector: None,
            url: None,
            html: None,
            value: Some(compact_live_value(expression, 120)),
        },
        PageRequest::BoundingBox {
            page_id, selector, ..
        }
        | PageRequest::Hover {
            page_id, selector, ..
        }
        | PageRequest::LocatorPress {
            page_id, selector, ..
        } => LivePageStep {
            action: page_request_kind(req),
            page_id: Some(page_id.clone()),
            selector: Some(selector.clone()),
            url: None,
            html: None,
            value: None,
        },
        _ => LivePageStep {
            action: page_request_kind(req),
            page_id: page_req_id_str(req).map(str::to_string),
            selector: None,
            url: None,
            html: None,
            value: None,
        },
    }
}

fn compact_live_value(value: &str, max_chars: usize) -> String {
    let compact = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out = String::new();
    for ch in compact.chars().take(max_chars) {
        out.push(ch);
    }
    if compact.chars().count() > max_chars {
        out.push_str("...");
    }
    out
}

fn page_request_kind(req: &crate::cdp_driver::PageRequest) -> String {
    let raw = serde_json::to_value(req)
        .ok()
        .and_then(|v| v.get("kind").and_then(|k| k.as_str()).map(str::to_string));
    raw.unwrap_or_else(|| "page_request".to_string())
}

async fn highlight_selector_for_live_e2e(page: &Page, selector: &str) -> Result<()> {
    let selector_json = serde_json::to_string(selector)?;
    let expression = format!(
        r#"(() => {{
  const selector = {selector_json};
  let el = null;
  try {{ el = document.querySelector(selector); }} catch {{ return false; }}
  if (!el) return false;
  let marker = document.getElementById("__jet_e2e_selector_highlight");
  if (!marker) {{
    marker = document.createElement("div");
    marker.id = "__jet_e2e_selector_highlight";
    marker.style.position = "fixed";
    marker.style.zIndex = "2147483647";
    marker.style.pointerEvents = "none";
    marker.style.border = "3px solid #ff3366";
    marker.style.boxShadow = "0 0 0 4px rgba(255, 51, 102, .18)";
    marker.style.borderRadius = "4px";
    document.documentElement.appendChild(marker);
  }}
  const r = el.getBoundingClientRect();
  marker.style.left = `${{Math.max(0, r.left)}}px`;
  marker.style.top = `${{Math.max(0, r.top)}}px`;
  marker.style.width = `${{Math.max(0, r.width)}}px`;
  marker.style.height = `${{Math.max(0, r.height)}}px`;
  return true;
}})()"#
    );
    page.evaluate(&expression).await.map(|_| ())
}

async fn wait_for_live_checkpoint(
    live: Option<&LiveE2eConfig>,
    state: &mut LiveCheckpointState,
    test_id: &str,
    title: &str,
) {
    let Some(live) = live else {
        return;
    };
    append_jsonl(
        &live.event_log,
        &json!({
            "kind": "checkpoint_waiting",
            "ts_ms": now_ms(),
            "case_id": test_id,
            "title": title,
        }),
    );
    loop {
        let control = read_live_control(&live.control_path);
        if !control.paused {
            break;
        }
        if control.next_token > state.next_token_seen {
            state.next_token_seen = control.next_token;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    append_jsonl(
        &live.event_log,
        &json!({
            "kind": "checkpoint_released",
            "ts_ms": now_ms(),
            "case_id": test_id,
            "title": title,
        }),
    );
}

#[derive(Default, serde::Deserialize)]
struct LiveControlFile {
    #[serde(default)]
    paused: bool,
    #[serde(default)]
    speed_multiplier: u64,
    #[serde(default)]
    next_token: u64,
}

fn normalize_live_speed_multiplier(speed_multiplier: u64) -> u64 {
    match speed_multiplier {
        2 | 4 => speed_multiplier,
        _ => 1,
    }
}

fn live_step_delay_ms(live: &LiveE2eConfig) -> u64 {
    if live.step_delay_ms == 0 {
        return 0;
    }
    let speed_multiplier =
        normalize_live_speed_multiplier(read_live_control(&live.control_path).speed_multiplier);
    (live.step_delay_ms / speed_multiplier).max(1)
}

fn read_live_control(path: &Path) -> LiveControlFile {
    // GH #3194 — the prior `.ok().and_then(...).unwrap_or_default()` chain
    // silently coalesced both `read_to_string` and `from_str` errors into
    // `LiveControlFile::default()`. NotFound is the legitimate "no
    // control file yet" branch, but other IO errors and JSON parse
    // failures dropped pause/replay state from the UI without surfacing
    // anything — the user clicked "pause" and the runner kept charging.
    let body = match std::fs::read_to_string(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return LiveControlFile::default();
        }
        Err(e) => {
            tracing::warn!(
                target: "jet::test_runner::live",
                "failed to read live control file {:?}: {e}; treating as no-op default \
                 (pause/replay commands from the UI will NOT take effect this tick) (GH #3194)",
                path
            );
            return LiveControlFile::default();
        }
    };
    match serde_json::from_str(&body) {
        Ok(ctrl) => ctrl,
        Err(e) => {
            tracing::warn!(
                target: "jet::test_runner::live",
                "failed to parse live control file {:?}: {e}; treating as no-op default \
                 (pause/replay commands from the UI will NOT take effect this tick) (GH #3194)",
                path
            );
            LiveControlFile::default()
        }
    }
}

fn append_jsonl(path: &Path, value: &serde_json::Value) {
    // GH #3191 — the prior implementation silently dropped failures from
    // each of `create_dir_all`, `serde_json::to_string`, `open`, and
    // `writeln!`. The live event log feeds the cclab UI's progress
    // stream; a silent write failure meant the UI froze on stale data
    // with no diagnostic. Surface every failure regime via
    // tracing::warn! so the underlying breakage is debuggable.
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            tracing::warn!(
                target: "jet::test_runner::live",
                "failed to create live event log dir {:?}: {e}; live event for {:?} dropped (GH #3191)",
                parent,
                path
            );
            return;
        }
    }
    let line = match serde_json::to_string(value) {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!(
                target: "jet::test_runner::live",
                "failed to serialize live event for {:?}: {e}; event dropped (GH #3191)",
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
                target: "jet::test_runner::live",
                "failed to open live event log {:?}: {e}; event dropped: {line} (GH #3191)",
                path
            );
            return;
        }
    };
    if let Err(e) = writeln!(file, "{line}") {
        tracing::warn!(
            target: "jet::test_runner::live",
            "failed to write live event to {:?}: {e}; event dropped: {line} (GH #3191)",
            path
        );
    }
}

fn test_title_for_live(suite: &[String], name: &str) -> String {
    if suite.is_empty() {
        name.to_string()
    } else {
        format!("{} > {}", suite.join(" > "), name)
    }
}

fn now_ms() -> u64 {
    // GH #3685 — was `.unwrap_or(0)` which silently stamped every live e2e
    // event with `ts_ms: 0` whenever the host wall clock drifted before
    // UNIX_EPOCH, collapsing the timeline, zeroing case durations, and
    // tripping stall detection without any breadcrumb. Route through
    // `safe_worker_now_ms` so the warn names the live-event-log symptom
    // and points at the host clock.
    let (ms, warn) = safe_worker_now_ms(SystemTime::now());
    if let Some(msg) = warn {
        tracing::warn!(target: "jet::test_runner::worker", "{}", msg);
    }
    ms
}

/// Format the warn message emitted when `SystemTime::now().duration_since(UNIX_EPOCH)`
/// fails inside the worker live-event timestamp path. Extracted so unit tests
/// can pin the user-facing text (issue tag, symptom names, host-clock fix
/// pointer) without spinning up a tracing subscriber.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
pub(crate) fn format_safe_worker_now_ms_warn(err: &std::time::SystemTimeError) -> String {
    format!(
        "GH #3685 jet::test_runner::worker now_ms: host wall clock is before UNIX_EPOCH \
         ({err}); falling back to ts_ms=0 on this live e2e event. Downstream symptoms: \
         live event timeline collapses to 1970-01-01, case_finished - case_started \
         durations report as 0 ms, and live-runner stall detection misfires because \
         `now() - last_event_ts` reads as seconds-since-1970. Fix the host clock \
         (NTP / container time / faketime); this is not a jet bug."
    )
}

/// Worker-side counterpart to the cross-module `safe_*_now_ms` family
/// (e2e #3669, trace::buffer #3673, browser_cli::session #3677,
/// dev_server #3680). Returns `(0, Some(warn))` on clock-before-epoch so
/// the caller can decide where/how to surface the warn — here, via a
/// `tracing::warn!` tagged at `jet::test_runner::worker`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
pub(crate) fn safe_worker_now_ms(now: SystemTime) -> (u64, Option<String>) {
    match now.duration_since(UNIX_EPOCH) {
        Ok(dur) => (dur.as_millis() as u64, None),
        Err(err) => (0, Some(format_safe_worker_now_ms_warn(&err))),
    }
}

/// Extract the page_id string from a PageRequest (for routing to active page).
fn page_req_id_str(req: &crate::cdp_driver::PageRequest) -> Option<&str> {
    use crate::cdp_driver::PageRequest;
    match req {
        PageRequest::Goto { page_id, .. }
        | PageRequest::Click { page_id, .. }
        | PageRequest::Fill { page_id, .. }
        | PageRequest::WaitForSelector { page_id, .. }
        | PageRequest::WaitForLoadState { page_id, .. }
        | PageRequest::Evaluate { page_id, .. }
        | PageRequest::Url { page_id, .. }
        | PageRequest::Close { page_id, .. }
        | PageRequest::GetText { page_id, .. }
        | PageRequest::GetAttribute { page_id, .. }
        // Phase-6 parity variants
        | PageRequest::Title { page_id, .. }
        | PageRequest::SetViewportSize { page_id, .. }
        | PageRequest::Screenshot { page_id, .. }
        | PageRequest::GoBack { page_id, .. }
        | PageRequest::GoForward { page_id, .. }
        | PageRequest::Reload { page_id, .. }
        | PageRequest::KeyboardPress { page_id, .. }
        | PageRequest::KeyboardType { page_id, .. }
        | PageRequest::MouseEvent { page_id, .. }
        | PageRequest::SetContent { page_id, .. }
        | PageRequest::Content { page_id, .. }
        | PageRequest::BoundingBox { page_id, .. }
        | PageRequest::Hover { page_id, .. }
        | PageRequest::LocatorPress { page_id, .. }
        | PageRequest::SubscribeEvent { page_id, .. }
        | PageRequest::RemoveEventListener { page_id, .. } => Some(page_id.as_str()),
        // NewPage and B3 context variants do not carry a page_id — they are
        // routed directly in the worker loop rather than via dispatch_page_request.
        PageRequest::NewPage { .. }
        | PageRequest::NewContext { .. }
        | PageRequest::CloseContext { .. }
        | PageRequest::ContextNewPage { .. }
        // P3.2 storage-state variants also lack a page_id — they target a
        // context by browserContextId and are routed directly in worker.rs.
        | PageRequest::ContextCookies { .. }
        | PageRequest::ContextAddCookies { .. }
        | PageRequest::ContextClearCookies { .. }
        | PageRequest::ContextStorageState { .. }
        | PageRequest::ContextSetStorageState { .. } => None,
    }
}

fn page_req_id(req: &crate::cdp_driver::PageRequest) -> u64 {
    use crate::cdp_driver::PageRequest;
    match req {
        PageRequest::NewPage { req_id }
        | PageRequest::Goto { req_id, .. }
        | PageRequest::Click { req_id, .. }
        | PageRequest::Fill { req_id, .. }
        | PageRequest::WaitForSelector { req_id, .. }
        | PageRequest::WaitForLoadState { req_id, .. }
        | PageRequest::Evaluate { req_id, .. }
        | PageRequest::Url { req_id, .. }
        | PageRequest::Close { req_id, .. }
        | PageRequest::GetText { req_id, .. }
        | PageRequest::GetAttribute { req_id, .. }
        | PageRequest::Title { req_id, .. }
        | PageRequest::SetViewportSize { req_id, .. }
        | PageRequest::Screenshot { req_id, .. }
        | PageRequest::GoBack { req_id, .. }
        | PageRequest::GoForward { req_id, .. }
        | PageRequest::Reload { req_id, .. }
        | PageRequest::KeyboardPress { req_id, .. }
        | PageRequest::KeyboardType { req_id, .. }
        | PageRequest::MouseEvent { req_id, .. }
        | PageRequest::SetContent { req_id, .. }
        | PageRequest::Content { req_id, .. }
        | PageRequest::BoundingBox { req_id, .. }
        | PageRequest::Hover { req_id, .. }
        | PageRequest::LocatorPress { req_id, .. }
        | PageRequest::SubscribeEvent { req_id, .. }
        | PageRequest::RemoveEventListener { req_id, .. }
        | PageRequest::NewContext { req_id }
        | PageRequest::CloseContext { req_id, .. }
        | PageRequest::ContextNewPage { req_id, .. }
        | PageRequest::ContextCookies { req_id, .. }
        | PageRequest::ContextAddCookies { req_id, .. }
        | PageRequest::ContextClearCookies { req_id, .. }
        | PageRequest::ContextStorageState { req_id, .. }
        | PageRequest::ContextSetStorageState { req_id, .. } => *req_id,
    }
}

fn page_req_timeout_ms(req: &crate::cdp_driver::PageRequest, default_timeout_ms: u64) -> u64 {
    use crate::cdp_driver::PageRequest;
    match req {
        PageRequest::Evaluate {
            timeout_ms: Some(timeout_ms),
            ..
        }
        | PageRequest::Close {
            timeout_ms: Some(timeout_ms),
            ..
        }
        | PageRequest::Screenshot {
            timeout_ms: Some(timeout_ms),
            ..
        } => *timeout_ms,
        _ => default_timeout_ms,
    }
}

fn format_page_rpc_timeout_error(action: &str, timeout_ms: u64) -> String {
    format!(
        "page action `{action}` timed out after {timeout_ms}ms while waiting for the browser RPC to finish"
    )
}

#[derive(Debug, Clone)]
struct PendingNetworkRequest {
    url: String,
    method: String,
}

#[derive(Debug, Clone)]
struct PendingNetworkResponse {
    session_id: Option<String>,
    url: String,
    method: String,
    status: u16,
    resource_type: String,
}

async fn start_network_event_pump_if_needed(
    browser: &Arc<Mutex<Option<Browser>>>,
    pages: &Arc<Mutex<HashMap<String, Arc<Page>>>>,
    stdin_writer: Option<Arc<Mutex<ChildStdin>>>,
    started: &mut bool,
) {
    if *started {
        return;
    }
    let Some(stdin_writer) = stdin_writer else {
        return;
    };
    let mut browser_guard = browser.lock().await;
    let Some(browser_ref) = browser_guard.as_mut() else {
        return;
    };
    let Some(mut events_rx) = browser_ref.take_event_receiver() else {
        return;
    };
    let root_session = browser_ref.root_session();
    drop(browser_guard);

    let pages = pages.clone();
    tokio::spawn(async move {
        let mut requests: HashMap<String, PendingNetworkRequest> = HashMap::new();
        let mut responses: HashMap<String, PendingNetworkResponse> = HashMap::new();
        while let Some(event) = events_rx.recv().await {
            match event.method.as_str() {
                "Network.requestWillBeSent" => {
                    let Some(request_id) = event.params["requestId"].as_str() else {
                        continue;
                    };
                    let key = network_event_key(event.session_id.as_deref(), request_id);
                    let request = &event.params["request"];
                    let url = request["url"].as_str().unwrap_or_default().to_string();
                    let method = request["method"].as_str().unwrap_or("GET").to_string();
                    requests.insert(key, PendingNetworkRequest { url, method });
                }
                "Network.responseReceived" => {
                    let Some(request_id) = event.params["requestId"].as_str() else {
                        continue;
                    };
                    let key = network_event_key(event.session_id.as_deref(), request_id);
                    let response = &event.params["response"];
                    let request = requests
                        .get(&key)
                        .cloned()
                        .unwrap_or(PendingNetworkRequest {
                            url: response["url"].as_str().unwrap_or_default().to_string(),
                            method: "GET".to_string(),
                        });
                    let status = response["status"]
                        .as_u64()
                        .and_then(|n| u16::try_from(n).ok())
                        .unwrap_or(0);
                    responses.insert(
                        key,
                        PendingNetworkResponse {
                            session_id: event.session_id.clone(),
                            url: response["url"]
                                .as_str()
                                .unwrap_or(request.url.as_str())
                                .to_string(),
                            method: request.method,
                            status,
                            resource_type: event.params["type"]
                                .as_str()
                                .unwrap_or_default()
                                .to_string(),
                        },
                    );
                }
                "Network.loadingFinished" => {
                    let Some(request_id) = event.params["requestId"].as_str() else {
                        continue;
                    };
                    let key = network_event_key(event.session_id.as_deref(), request_id);
                    let Some(response) = responses.remove(&key) else {
                        requests.remove(&key);
                        continue;
                    };
                    if !should_forward_network_response(&response) {
                        requests.remove(&key);
                        continue;
                    }
                    let pages_for_body = pages.clone();
                    let writer_for_body = stdin_writer.clone();
                    let root_session_for_body = root_session.clone();
                    let request_id_for_body = request_id.to_string();
                    tokio::spawn(async move {
                        let page_id = page_id_for_cdp_session(
                            &pages_for_body,
                            response.session_id.as_deref(),
                        )
                        .await;
                        let Some(page_id) = page_id else {
                            return;
                        };
                        let body = network_response_body(
                            &root_session_for_body,
                            response.session_id.as_deref(),
                            &request_id_for_body,
                        )
                        .await
                        .unwrap_or_default();
                        let payload = json!({
                            "url": response.url,
                            "status": response.status,
                            "body": body,
                            "request": {
                                "url": response.url,
                                "method": response.method,
                            },
                        });
                        let event_response = crate::cdp_driver::PageResponse::Event {
                            page_id,
                            event: "response".to_string(),
                            payload,
                        };
                        let mut writer = writer_for_body.lock().await;
                        if let Err(err) = write_page_response(&mut *writer, event_response).await {
                            tracing::warn!(
                                target: "jet::test_runner::worker",
                                rpc_kind = "PageResponse::Event",
                                error = %err,
                                "{}",
                                format_worker_stdin_write_warn("PageResponse::Event", &err)
                            );
                        }
                    });
                    requests.remove(&key);
                }
                "Network.loadingFailed" => {
                    if let Some(request_id) = event.params["requestId"].as_str() {
                        let key = network_event_key(event.session_id.as_deref(), request_id);
                        requests.remove(&key);
                        responses.remove(&key);
                    }
                }
                _ => {}
            }
        }
    });
    *started = true;
}

fn should_forward_network_response(response: &PendingNetworkResponse) -> bool {
    matches!(response.resource_type.as_str(), "Fetch" | "XHR") || response.url.contains("/api/")
}

fn network_event_key(session_id: Option<&str>, request_id: &str) -> String {
    format!("{}:{request_id}", session_id.unwrap_or(""))
}

async fn page_id_for_cdp_session(
    pages: &Arc<Mutex<HashMap<String, Arc<Page>>>>,
    session_id: Option<&str>,
) -> Option<String> {
    let session_id = session_id?;
    let pages = pages.lock().await;
    pages.iter().find_map(|(page_id, page)| {
        (page.session().session_id() == Some(session_id)).then(|| page_id.clone())
    })
}

async fn network_response_body(
    root_session: &crate::browser::cdp::CdpSession,
    session_id: Option<&str>,
    request_id: &str,
) -> Result<String> {
    let session = match session_id {
        Some(session_id) => root_session.child_session(session_id.to_string()),
        None => root_session.clone(),
    };
    let value = session
        .send(
            "Network.getResponseBody",
            json!({ "requestId": request_id }),
        )
        .await?;
    let body = value["body"].as_str().unwrap_or_default();
    if value["base64Encoded"].as_bool().unwrap_or(false) {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(body)
            .context("decode Network.getResponseBody base64")?;
        return Ok(String::from_utf8_lossy(&bytes).into_owned());
    }
    Ok(body.to_string())
}

/// Lazily launch the browser on first use. Returns `Ok(())` on success (the
/// browser is now populated inside the `Arc<Mutex<Option<Browser>>>`), or the
/// launcher error otherwise so the calling handler can surface it to JS.
// @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R6
async fn ensure_browser(
    browser: &Arc<Mutex<Option<Browser>>>,
    browser_session: &Arc<Mutex<Option<BrowserSessionReport>>>,
    spec_path: &Path,
    headless: bool,
    executable: Option<PathBuf>,
    browser_ws_url: Option<String>,
) -> Result<()> {
    let mut guard = browser.lock().await;
    if guard.is_some() {
        return Ok(());
    }
    let (mut session, launched) = if let Some(ws_url) = browser_ws_url {
        let mut session = BrowserSessionReport::connecting_with_driver(
            spec_path.to_path_buf(),
            headless,
            now_ms(),
            "cdp-shared-window",
        );
        let connected = match Browser::connect_to_default_context(&ws_url).await {
            Ok(connected) => connected,
            Err(err) => {
                let message = format!("{err:#}");
                session.fail(message, now_ms());
                *browser_session.lock().await = Some(session);
                return Err(err);
            }
        };
        (session, connected)
    } else {
        let browser_driver = if executable.is_some() {
            "chrome"
        } else {
            "chromium"
        };
        let mut session = BrowserSessionReport::launching_with_driver(
            spec_path.to_path_buf(),
            headless,
            now_ms(),
            browser_driver,
        );
        let launched = match Browser::launch(LaunchOptions {
            executable,
            headless,
            ..Default::default()
        })
        .await
        {
            Ok(launched) => launched,
            Err(err) => {
                let message = format!("{err:#}");
                session.fail(message, now_ms());
                *browser_session.lock().await = Some(session);
                return Err(err);
            }
        };
        (session, launched)
    };
    session.ready(
        launched.process_id(),
        launched.ws_url().to_string(),
        now_ms(),
    );
    *browser_session.lock().await = Some(session);
    *guard = Some(launched);
    Ok(())
}

// ── Phase 3: expect RPC handlers ─────────────────────────────────────────────

/// Dispatch a `WireRequest` from the JS worker to the browser layer and return
/// a `WireResponse`. Called from `run_spec` when a line of NDJSON from the
/// worker decodes as a `WireRequest` rather than a `WorkerEvent`.
///
/// `page` is `None` when no browser is active (headless: false not yet wired),
/// in which case an error response is returned so the matcher fails cleanly.
// @spec .aw/changes/enhancement-phase-3-dom-matchers-fixtures-for-native-test-runn/specs/enhancement-phase-3-dom-matchers-fixtures-for-native-test-runn-spec.md#R4
pub async fn handle_expect_request(
    req: WireRequest,
    page: Option<&Page>,
    spec_path: &Path,
    spec_slug: &str,
    update_snapshots: bool,
) -> WireResponse {
    match req {
        // @spec ...#R1
        WireRequest::QueryText {
            req_id,
            page_id: _,
            selector,
        } => {
            let Some(page) = page else {
                return WireResponse::Error {
                    req_id,
                    message: "No active browser page for expect.queryText".to_string(),
                    matcher_diff: None,
                };
            };
            match page.locator(&selector) {
                Err(e) => WireResponse::Error {
                    req_id,
                    message: format!("Invalid selector for queryText: {e}"),
                    matcher_diff: None,
                },
                Ok(locator) => match locator.text_content().await {
                    Ok(text) => WireResponse::TextResult { req_id, text },
                    Err(e) => WireResponse::Error {
                        req_id,
                        message: format!("text_content failed: {e}"),
                        matcher_diff: None,
                    },
                },
            }
        }

        // @spec ...#R2
        WireRequest::IsVisible {
            req_id,
            page_id: _,
            selector,
        } => {
            let Some(page) = page else {
                return WireResponse::Error {
                    req_id,
                    message: "No active browser page for expect.isVisible".to_string(),
                    matcher_diff: None,
                };
            };
            match page.locator(&selector) {
                Err(e) => WireResponse::Error {
                    req_id,
                    message: format!("Invalid selector for isVisible: {e}"),
                    matcher_diff: None,
                },
                Ok(locator) => match locator.is_visible().await {
                    Ok(visible) => WireResponse::VisibleResult { req_id, visible },
                    Err(e) => WireResponse::Error {
                        req_id,
                        message: format!("is_visible failed: {e}"),
                        matcher_diff: None,
                    },
                },
            }
        }

        // @spec ...#R3
        WireRequest::Screenshot { req_id, page_id: _ } => {
            let Some(page) = page else {
                return WireResponse::Error {
                    req_id,
                    message: "No active browser page for expect.screenshot".to_string(),
                    matcher_diff: None,
                };
            };
            match page.screenshot().await {
                Ok(bytes) => {
                    let data = base64::engine::general_purpose::STANDARD.encode(&bytes);
                    WireResponse::ScreenshotResult { req_id, data }
                }
                Err(e) => WireResponse::Error {
                    req_id,
                    message: format!("screenshot failed: {e}"),
                    matcher_diff: None,
                },
            }
        }

        // @spec ...#R3
        // @spec ...#R7
        // @spec ...#R8
        WireRequest::MatchSnapshot {
            req_id,
            page_id: _,
            snapshot_name,
        } => {
            let Some(page) = page else {
                return WireResponse::Error {
                    req_id,
                    message: "No active browser page for expect.matchSnapshot".to_string(),
                    matcher_diff: None,
                };
            };
            let bytes = match page.screenshot().await {
                Ok(b) => b,
                Err(e) => {
                    return WireResponse::Error {
                        req_id,
                        message: format!("screenshot failed: {e}"),
                        matcher_diff: None,
                    };
                }
            };
            match load_or_write_snapshot(
                spec_path,
                spec_slug,
                &snapshot_name,
                &bytes,
                update_snapshots,
            ) {
                Ok(None) => WireResponse::SnapshotResult { req_id },
                Ok(Some(diff)) => WireResponse::Error {
                    req_id,
                    message: format!(
                        "snapshot mismatch for {snapshot_name} (re-run with --update-snapshots to overwrite)"
                    ),
                    matcher_diff: Some(diff),
                },
                Err(e) => WireResponse::Error {
                    req_id,
                    message: format!("snapshot I/O failed: {e}"),
                    matcher_diff: None,
                },
            }
        }

        // @spec #2713
        WireRequest::MatchTextSnapshot {
            req_id,
            snapshot_name,
            content,
        } => match load_or_write_text_snapshot(
            spec_path,
            spec_slug,
            &snapshot_name,
            &content,
            update_snapshots,
        ) {
            Ok(None) => WireResponse::SnapshotResult { req_id },
            Ok(Some(diff)) => WireResponse::Error {
                req_id,
                message: format!(
                    "text snapshot mismatch for {snapshot_name} (re-run with --update-snapshots to overwrite)"
                ),
                matcher_diff: Some(diff),
            },
            Err(e) => WireResponse::Error {
                req_id,
                message: format!("text snapshot I/O failed: {e}"),
                matcher_diff: None,
            },
        },

        WireRequest::LiveCheckpoint { req_id, .. } => WireResponse::LiveCheckpointResult { req_id },
    }
}

/// Load or write a PNG snapshot for `toMatchSnapshot`.
///
/// Snapshot path: `<spec-dir>/__snapshots__/<spec-slug>/<name>.png`
///
/// Behaviour:
/// - File **absent** → write `actual_bytes` to disk and return `Ok(None)` (pass).
/// - File **present** and `update_snapshots == true` → overwrite, return `Ok(None)` (pass).
/// - File **present** and bytes match → `Ok(None)` (pass).
/// - File **present** and bytes differ → `Ok(Some(MatcherDiff))` (fail).
///
/// Returns `Err` only on I/O errors.
// @spec .aw/changes/enhancement-phase-3-dom-matchers-fixtures-for-native-test-runn/specs/enhancement-phase-3-dom-matchers-fixtures-for-native-test-runn-spec.md#R3
// @spec ...#R7
pub fn load_or_write_snapshot(
    spec_path: &Path,
    spec_slug: &str,
    snapshot_name: &str,
    actual_bytes: &[u8],
    update_snapshots: bool,
) -> Result<Option<MatcherDiff>> {
    let snap_dir: PathBuf = spec_path
        .parent()
        .unwrap_or(spec_path)
        .join("__snapshots__")
        .join(spec_slug);

    std::fs::create_dir_all(&snap_dir)
        .with_context(|| format!("Failed to create snapshot dir: {}", snap_dir.display()))?;

    let snap_file = snap_dir.join(format!("{snapshot_name}.png"));

    if !snap_file.exists() || update_snapshots {
        // First run or forced update — write and pass.
        std::fs::write(&snap_file, actual_bytes)
            .with_context(|| format!("Failed to write snapshot: {}", snap_file.display()))?;
        return Ok(None);
    }

    // Subsequent runs — compare exact bytes.
    let stored = std::fs::read(&snap_file)
        .with_context(|| format!("Failed to read snapshot: {}", snap_file.display()))?;

    if stored == actual_bytes {
        Ok(None)
    } else {
        // Return a diff that the reporter can display.
        Ok(Some(MatcherDiff {
            actual: format!("<PNG {} bytes>", actual_bytes.len()),
            expected: format!(
                "<PNG {} bytes stored at {}>",
                stored.len(),
                snap_file.display()
            ),
        }))
    }
}

/// Load or write a text snapshot for `toMatchTextSnapshot`.
///
/// Snapshot path: `<spec-dir>/__snapshots__/<spec-slug>/<name>.txt`
///
/// Behaviour mirrors `load_or_write_snapshot` but operates on UTF-8 text and
/// returns a unified line-based diff on mismatch rather than a byte-count
/// summary. A trailing newline is appended on write so the baseline files
/// edit cleanly under standard text editors; reads strip it before
/// comparison so round-tripping is byte-stable.
// @spec #2713
pub fn load_or_write_text_snapshot(
    spec_path: &Path,
    spec_slug: &str,
    snapshot_name: &str,
    actual: &str,
    update_snapshots: bool,
) -> Result<Option<MatcherDiff>> {
    let snap_dir: PathBuf = spec_path
        .parent()
        .unwrap_or(spec_path)
        .join("__snapshots__")
        .join(spec_slug);

    std::fs::create_dir_all(&snap_dir)
        .with_context(|| format!("Failed to create snapshot dir: {}", snap_dir.display()))?;

    let snap_file = snap_dir.join(format!("{snapshot_name}.txt"));
    let to_write = if actual.ends_with('\n') {
        actual.to_string()
    } else {
        format!("{actual}\n")
    };

    if !snap_file.exists() || update_snapshots {
        std::fs::write(&snap_file, to_write.as_bytes())
            .with_context(|| format!("Failed to write snapshot: {}", snap_file.display()))?;
        return Ok(None);
    }

    let stored = std::fs::read_to_string(&snap_file)
        .with_context(|| format!("Failed to read snapshot: {}", snap_file.display()))?;
    let expected = stored.strip_suffix('\n').unwrap_or(&stored);

    if expected == actual {
        Ok(None)
    } else {
        Ok(Some(MatcherDiff {
            actual: actual.to_string(),
            expected: expected.to_string(),
        }))
    }
}

/// Compute a filesystem-safe slug for a spec file — used as the snapshot
/// subdirectory name. Uses the file stem with non-alphanumerics collapsed to `-`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
pub fn spec_slug_for(spec_path: &Path) -> String {
    let stem = spec_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("spec");
    stem.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

/// Convert the wire-side `WireTraceMode` (used by the runner config and
/// the NDJSON protocol) into the buffer-side `TraceMode` accepted by
/// `commit_trace`. Both enums carry the same three variants; this is a
/// pure lowering with no semantic adjustment.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R1
pub(crate) fn wire_trace_mode_to_buffer_mode(
    mode: WireTraceMode,
) -> crate::trace::buffer::TraceMode {
    use crate::trace::buffer::TraceMode;
    match mode {
        WireTraceMode::Off => TraceMode::Off,
        WireTraceMode::On => TraceMode::On,
        WireTraceMode::RetainOnFailure => TraceMode::RetainOnFailure,
    }
}

/// Convert a worker `TestOutcome` into the manifest-side `TraceOutcome`.
/// `Skipped` cases never get a trace (they didn't run); the caller must
/// short-circuit before calling this helper.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R1
pub(crate) fn test_outcome_to_trace_outcome(outcome: TestOutcome) -> Option<TraceOutcome> {
    match outcome {
        TestOutcome::Passed => Some(TraceOutcome::Passed),
        TestOutcome::Failed => Some(TraceOutcome::Failed),
        TestOutcome::TimedOut => Some(TraceOutcome::TimedOut),
        TestOutcome::Skipped => None,
    }
}

/// Commit a per-test trace buffer for `TestEnd` and return the path that
/// should be surfaced on `TestReport.trace_path`. `None` means no zip
/// was written: either the buffer was absent (trace mode `Off`, or no
/// `TestStart` arrived), the mode was `RetainOnFailure` on a passing
/// test, or the test was `Skipped`.
///
/// `trace_dir` is created lazily — callers don't need to mkdir.
/// Output filename is `<spec_slug>__<test_id_sanitised>.zip`, joined
/// under `trace_dir`. Sanitisation matches `spec_slug_for`'s rule:
/// non-alphanumerics collapse to `-`.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R11
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R4
pub(crate) fn commit_trace_for_report(
    buffer: Option<TraceBuffer>,
    outcome: TestOutcome,
    mode: WireTraceMode,
    trace_dir: &Path,
    spec_slug: &str,
    test_id: &str,
) -> Result<Option<PathBuf>> {
    let Some(buffer) = buffer else {
        return Ok(None);
    };
    let Some(trace_outcome) = test_outcome_to_trace_outcome(outcome) else {
        return Ok(None);
    };
    let buffer_mode = wire_trace_mode_to_buffer_mode(mode);
    // `commit_trace` itself handles `RetainOnFailure` + `Passed` →
    // discard, so we can call it unconditionally here.
    std::fs::create_dir_all(trace_dir)
        .with_context(|| format!("creating trace dir: {}", trace_dir.display()))?;
    let test_slug = test_id_slug(test_id);
    let out_path = trace_dir.join(format!("{spec_slug}__{test_slug}.zip"));
    let written = commit_trace(buffer, trace_outcome, buffer_mode, &out_path)
        .with_context(|| format!("committing trace to {}", out_path.display()))?;
    Ok(written)
}

/// Filesystem-safe slug for a wire-level test id. Mirrors `spec_slug_for`'s
/// rule: non-alphanumerics → `-`, trim leading/trailing dashes.
fn test_id_slug(id: &str) -> String {
    let slug: String = id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let trimmed = slug.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "test".to_string()
    } else {
        trimmed
    }
}

fn outcome_from_wire(outcome: TestOutcome) -> Outcome {
    match outcome {
        TestOutcome::Passed => Outcome::Passed,
        TestOutcome::Failed => Outcome::Failed,
        TestOutcome::Skipped => Outcome::Skipped,
        TestOutcome::TimedOut => Outcome::TimedOut,
    }
}

fn test_error_from_wire(e: wire::TestError) -> crate::test_runner::reporter::TestError {
    let source_location = e
        .stack
        .as_deref()
        .and_then(crate::test_runner::reporter::SourceLocation::parse_from_stack);
    crate::test_runner::reporter::TestError {
        message: e.message,
        stack: e.stack,
        diff: e.diff,
        source_location,
    }
}

/// Encode a `WireResponse` as an NDJSON line and write it to `writer`.
/// Used in `run_spec` to send responses back to the worker via stdin.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
pub async fn write_response<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    response: WireResponse,
) -> Result<()> {
    let line = serde_json::to_string(&response).context("Failed to serialise WireResponse")?;
    writer.write_all(line.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    Ok(())
}

/// Format the warning emitted when a write to the worker subprocess's
/// stdin pipe fails during RPC response delivery. Names the RPC kind
/// ("WireResponse" or "PageResponse") and the underlying error, and
/// tags `GH #3546` so operators grepping "test runner stopped
/// responding" or "worker hung mid-spec" can land on this line.
/// Extracted for unit-test pinning.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
pub(crate) fn format_worker_stdin_write_warn(rpc_kind: &str, err: &anyhow::Error) -> String {
    format!(
        "GH #3546 worker stdin write failed for {rpc_kind}: {err}; the worker subprocess will NOT receive this RPC response. Typical cause: the worker has already exited or closed its stdin (BrokenPipe), in which case the next test will appear to hang or report 'no tests ran'. Check the worker process exit status and the surrounding stderr lines."
    )
}

/// Type-strip TypeScript → JavaScript using jet's existing transformer.
fn transform_spec(path: &Path) -> Result<String> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read spec: {}", path.display()))?;

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext == "js" || ext == "mjs" {
        return Ok(normalize_jet_test_virtual_imports(source));
    }

    let options = TransformOptions {
        source_maps: false,
        jsx_automatic: true,
        ..Default::default()
    };
    let transformer = Transformer::new(options);
    let result = transformer
        .transform_js(&source, path)
        .with_context(|| format!("Failed to type-strip {}", path.display()))?;
    Ok(normalize_jet_test_virtual_imports(result.code))
}

struct TempModuleGraphEmitter<'a> {
    out_dir: &'a Path,
    outputs: HashMap<PathBuf, PathBuf>,
    next_id: usize,
}

impl<'a> TempModuleGraphEmitter<'a> {
    fn new(out_dir: &'a Path) -> Self {
        Self {
            out_dir,
            outputs: HashMap::new(),
            next_id: 0,
        }
    }

    fn emit(&mut self, path: &Path, pretransformed: Option<String>) -> Result<PathBuf> {
        let canonical = path
            .canonicalize()
            .with_context(|| format!("canonicalizing test module {}", path.display()))?;
        if let Some(out) = self.outputs.get(&canonical) {
            return Ok(out.clone());
        }

        let out_path = self.out_dir.join(temp_module_file_name(path, self.next_id));
        self.next_id += 1;
        self.outputs.insert(canonical, out_path.clone());

        let transformed = match pretransformed {
            Some(source) => source,
            None => transform_spec(path)?,
        };
        let rewritten = rewrite_relative_test_imports(&transformed, path, self)
            .with_context(|| format!("rewriting relative imports in {}", path.display()))?;

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        std::fs::write(&out_path, rewritten)
            .with_context(|| format!("writing transformed test module {}", out_path.display()))?;
        Ok(out_path)
    }
}

fn temp_module_file_name(path: &Path, id: usize) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("module");
    let safe: String = stem
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect();
    let safe = safe.trim_matches('-');
    let safe = if safe.is_empty() { "module" } else { safe };
    format!("{id:04}-{safe}.mjs")
}

fn rewrite_relative_test_imports(
    source: &str,
    source_path: &Path,
    emitter: &mut TempModuleGraphEmitter<'_>,
) -> Result<String> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_javascript::LANGUAGE.into())
        .context("setting tree-sitter JavaScript language")?;
    let tree = parser
        .parse(source, None)
        .context("parsing transformed test module")?;
    let root = tree.root_node();

    let mut replacements = Vec::new();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        let kind = child.kind();
        if kind != "import_statement" && kind != "export_statement" {
            continue;
        }
        let Some((start, end)) = first_module_string_range(&child) else {
            continue;
        };
        let raw = &source[start..end];
        let spec = strip_module_quotes(raw);
        if !is_relative_or_absolute_specifier(&spec) {
            continue;
        }
        let Some(target) = resolve_test_relative_module(source_path, &spec)? else {
            continue;
        };
        let target_out = emitter.emit(&target, None)?;
        let rewritten = serde_json::to_string(&path_to_file_url(&target_out))
            .expect("file URL string serializes");
        replacements.push((start, end, rewritten));
    }

    if replacements.is_empty() {
        return Ok(source.to_string());
    }

    let mut out = String::with_capacity(source.len());
    let mut last = 0usize;
    for (start, end, replacement) in replacements {
        out.push_str(&source[last..start]);
        out.push_str(&replacement);
        last = end;
    }
    out.push_str(&source[last..]);
    Ok(out)
}

fn first_module_string_range(node: &tree_sitter::Node) -> Option<(usize, usize)> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "string" {
            return Some((child.start_byte(), child.end_byte()));
        }
    }
    None
}

fn strip_module_quotes(raw: &str) -> String {
    raw.trim()
        .trim_start_matches(['"', '\''])
        .trim_end_matches(['"', '\''])
        .to_string()
}

fn is_relative_or_absolute_specifier(spec: &str) -> bool {
    spec.starts_with("./") || spec.starts_with("../") || spec.starts_with('/')
}

fn resolve_test_relative_module(from: &Path, spec: &str) -> Result<Option<PathBuf>> {
    let base = if spec.starts_with('/') {
        PathBuf::from(spec)
    } else {
        let Some(parent) = from.parent() else {
            return Ok(None);
        };
        parent.join(spec)
    };

    if base.is_file() {
        return Ok(if is_test_source_module(&base) {
            Some(base)
        } else {
            None
        });
    }

    if matches!(
        base.extension().and_then(|e| e.to_str()),
        Some("js" | "jsx" | "mjs")
    ) {
        for ext in ["ts", "tsx"] {
            let candidate = base.with_extension(ext);
            if candidate.is_file() {
                return Ok(Some(candidate));
            }
        }
    }

    if base.extension().is_none() {
        for ext in ["ts", "tsx", "js", "jsx", "mjs", "cjs"] {
            let candidate = base.with_extension(ext);
            if candidate.is_file() {
                return Ok(Some(candidate));
            }
        }
        for ext in ["ts", "tsx", "js", "jsx", "mjs", "cjs"] {
            let candidate = base.join(format!("index.{ext}"));
            if candidate.is_file() {
                return Ok(Some(candidate));
            }
        }
    }

    Ok(None)
}

fn is_test_source_module(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs")
    )
}

fn normalize_jet_test_virtual_imports(source: String) -> String {
    if !source.contains("jet:test") {
        return source;
    }

    source
        .replace("\"jet:test\"", "\"@jet/test\"")
        .replace("'jet:test'", "'@jet/test'")
}

/// Build the boot ESM module that wires the runtime to the spec and writes
/// NDJSON to stdout. The runtime is resolved via the bare specifier
/// `@jet/test` so it shares a single module instance with specs that import
/// named exports from the same package. The spec file is imported via a
/// file URL so relative paths in its own code (if any) resolve correctly.
///
/// `jetConfig` is forwarded to the fixture registry so the default `page`
/// fixture can read `baseURL` and `headless` from the active project config.
///
// @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R3
// @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R5
fn build_boot(spec_path: &Path, spec: &SpecFile, config: &RunnerConfig) -> String {
    let spec_url = path_to_file_url(spec_path);
    let rel = spec.relative.display().to_string().replace('\\', "/");
    let timeout_ms = config.timeout_ms;
    let grep_js = config
        .grep
        .as_ref()
        .map(|g| format!("new RegExp({})", serde_json::to_string(g).unwrap()))
        .unwrap_or_else(|| "null".to_string());

    // Serialize jetConfig for the fixture registry.
    let base_url_js = config
        .base_url
        .as_deref()
        .map(|u| serde_json::to_string(u).unwrap())
        .unwrap_or_else(|| "null".to_string());
    let headless_js = if config.headless { "true" } else { "false" };
    let auto_artifacts_js = if config.auto_artifacts {
        "true"
    } else {
        "false"
    };
    let live_control_js = if config.live_e2e.is_some() {
        "true"
    } else {
        "false"
    };
    let artifacts_dir_js =
        serde_json::to_string(&config.auto_artifacts_dir.display().to_string()).unwrap();

    format!(
        r#"import {{ __jetRun }} from "@jet/test";
await __jetRun({{
  specUrl: {spec},
  file: {file},
  timeoutMs: {timeout},
  grep: {grep},
  jetConfig: {{ baseURL: {base_url}, headless: {headless} }},
  autoArtifacts: {auto_artifacts},
  artifactsDir: {artifacts_dir},
  liveControl: {live_control},
}});
"#,
        spec = serde_json::to_string(&spec_url).unwrap(),
        file = serde_json::to_string(&rel).unwrap(),
        timeout = timeout_ms,
        grep = grep_js,
        base_url = base_url_js,
        headless = headless_js,
        auto_artifacts = auto_artifacts_js,
        artifacts_dir = artifacts_dir_js,
        live_control = live_control_js,
    )
}

fn path_to_file_url(p: &Path) -> String {
    // Minimal file:// URL. `node` accepts percent-encoded paths; we only
    // escape spaces for simplicity.
    let display = p.display().to_string();
    let escaped = display.replace(' ', "%20");
    if cfg!(windows) {
        format!("file:///{}", escaped.replace('\\', "/"))
    } else {
        format!("file://{}", escaped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn transform_spec_passes_through_js() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("a.spec.js");
        std::fs::write(&p, "console.log('ok');\n").unwrap();
        let out = transform_spec(&p).unwrap();
        assert!(out.contains("console.log"));
    }

    #[test]
    fn transform_spec_strips_types() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("a.spec.ts");
        std::fs::write(&p, "const x: number = 1;\n").unwrap();
        let out = transform_spec(&p).unwrap();
        assert!(out.contains("const x"));
        // `: number` annotation should be gone
        assert!(!out.contains(": number"));
    }

    #[test]
    fn worker_runtime_exposes_playwright_compatible_test_step() {
        assert!(
            WORKER_RUNTIME.contains("test.step"),
            "runtime must expose test.step for Playwright-style E2E specs"
        );
        assert!(
            WORKER_RUNTIME.contains("boundTest.step"),
            "extended test functions must inherit test.step"
        );
    }

    #[test]
    fn worker_runtime_test_step_emits_structured_step_events() {
        assert!(
            WORKER_RUNTIME.contains("kind: \"step_start\""),
            "test.step should emit a structured start event"
        );
        assert!(
            WORKER_RUNTIME.contains("kind: \"step_end\""),
            "test.step should emit a structured finish event"
        );
        assert!(
            WORKER_RUNTIME.contains("currentStepSeq"),
            "step ids should be stable and monotonic within each test"
        );
    }

    #[test]
    fn page_shim_uses_playwright_like_action_timeout() {
        assert!(
            PAGE_SHIM.contains("DEFAULT_ACTION_TIMEOUT_MS = 30000"),
            "locator actionability should allow real web-app startup latency"
        );
    }

    #[test]
    fn page_shim_evaluate_supports_playwright_serializable_arg() {
        assert!(
            PAGE_SHIM.contains("async evaluate(expression, arg)"),
            "page.evaluate should accept the Playwright-style serializable arg"
        );
        assert!(
            PAGE_SHIM.contains("_jetSerializeEvaluateArg(arg)"),
            "page.evaluate should pass the serialized arg into function calls"
        );
    }

    #[test]
    fn page_shim_get_by_role_supports_regex_name() {
        assert!(
            PAGE_SHIM.contains("name instanceof RegExp"),
            "getByRole should preserve Playwright-style regex names"
        );
        assert!(
            PAGE_SHIM.contains("nameRegex = new RegExp"),
            "role selector resolution should evaluate regex names in the page"
        );
        assert!(
            PAGE_SHIM.contains("role === 'heading'"),
            "role selector resolution should support implicit heading roles"
        );
        assert!(
            PAGE_SHIM.contains("h1, h2, h3, h4, h5, h6"),
            "heading role resolution should inspect native heading elements"
        );
        assert!(
            PAGE_SHIM.contains("arr = arr.filter(__visible)"),
            "getByRole should default to visible role candidates like Playwright"
        );
        assert!(
            PAGE_SHIM.contains("options && options.includeHidden"),
            "getByRole should retain an includeHidden escape hatch"
        );
    }

    #[test]
    fn page_shim_get_by_text_supports_regex_and_exact() {
        assert!(
            PAGE_SHIM.contains("function _jetTextSelector(text, options)"),
            "getByText should route through a selector builder"
        );
        assert!(
            PAGE_SHIM.contains("text instanceof RegExp"),
            "getByText should preserve Playwright-style regex text matchers"
        );
        assert!(
            PAGE_SHIM.contains("textExact="),
            "getByText should support exact text matching"
        );
        assert!(
            PAGE_SHIM.contains("options && options.exact"),
            "getByText should honor Playwright-style exact options"
        );
    }

    #[test]
    fn page_shim_supports_test_id_locators() {
        assert!(
            PAGE_SHIM.contains("getByTestId(testId)"),
            "page and locator should expose Playwright-style getByTestId"
        );
        assert!(
            PAGE_SHIM.contains("sel.indexOf('testid=') === 0"),
            "selector resolution should support data-testid lookups"
        );
    }

    #[test]
    fn page_shim_supports_label_locators() {
        assert!(
            PAGE_SHIM.contains("getByLabel(label)"),
            "page and locator should expose Playwright-style getByLabel"
        );
        assert!(
            PAGE_SHIM.contains("labelRegex="),
            "label selector resolution should support regex labels"
        );
        assert!(
            PAGE_SHIM.contains("aria-labelledby"),
            "label selector resolution should inspect accessible labels"
        );
    }

    #[test]
    fn page_shim_supports_response_waiters_and_request_get() {
        assert!(
            PAGE_SHIM.contains("async waitForResponse(predicate, opts)"),
            "page should expose Playwright-style waitForResponse"
        );
        assert!(
            PAGE_SHIM.contains("_notifyResponseWaiters(payload)"),
            "response events should resolve page.waitForResponse waiters"
        );
        assert!(
            PAGE_SHIM.contains("class JetApiRequestContext"),
            "page.request should expose a request context for API reads"
        );
        assert!(
            PAGE_SHIM.contains("frame()"),
            "response.request().frame().page() should be available for Playwright-compatible follow-up API calls"
        );
        assert!(
            PAGE_SHIM.contains("new JetResponse(payload, this)"),
            "response events should keep a reference to their originating page"
        );
    }

    #[test]
    fn worker_runtime_supports_locator_to_contain_text_matcher() {
        assert!(
            MATCHERS_SHIM.contains("export async function toContainTextLocator"),
            "matchers shim should expose locator-backed toContainText"
        );
        assert!(
            MATCHERS_SHIM.contains("containsPattern(text, expected)"),
            "toContainText should use substring/regex containment semantics"
        );
        assert!(
            WORKER_RUNTIME.contains("toContainTextLocator"),
            "worker runtime should import and attach toContainText"
        );
        assert!(
            WORKER_RUNTIME.contains("async toContainText(expected, opts)"),
            "expect(locator).toContainText should be available to specs"
        );
    }

    #[test]
    fn worker_runtime_locator_visibility_errors_name_locator() {
        assert!(
            PAGE_SHIM.contains("toString()"),
            "locator should expose a useful debug label"
        );
        assert!(
            MATCHERS_SHIM.contains("function describeLocator(locator)"),
            "matchers should format locator labels"
        );
        assert!(
            MATCHERS_SHIM.contains("Expected ${describeLocator(locator)} to be visible"),
            "toBeVisible timeout should identify the target locator"
        );
    }

    #[test]
    fn page_shim_locator_click_uses_dom_resolved_click() {
        assert!(
            PAGE_SHIM.contains("async _clickResolvedElement()"),
            "locator click should resolve an element before clicking"
        );
        assert!(
            PAGE_SHIM.contains("el.click();return true;"),
            "locator click should dispatch a DOM click on the resolved element"
        );
        assert!(
            PAGE_SHIM.contains("await this._clickResolvedElement()"),
            "locator click should use the resolved-element click helper"
        );
    }

    #[test]
    fn page_shim_locator_supports_scroll_into_view_if_needed() {
        assert!(
            PAGE_SHIM.contains("async scrollIntoViewIfNeeded(opts)"),
            "locator should expose Playwright-style scrollIntoViewIfNeeded"
        );
        assert!(
            PAGE_SHIM.contains("fullyVisible"),
            "scrollIntoViewIfNeeded should avoid unnecessary scrolling"
        );
        assert!(
            PAGE_SHIM.contains("el.scrollIntoView({block:'center',inline:'center'})"),
            "scrollIntoViewIfNeeded should center the resolved element when needed"
        );
    }

    #[test]
    fn worker_runtime_routes_page_events_by_page_id() {
        assert!(
            WORKER_RUNTIME.contains("pagesById: new Map()"),
            "worker runtime should keep page ids for async event dispatch"
        );
        assert!(
            WORKER_RUNTIME.contains("msg.kind === \"event\""),
            "worker runtime should dispatch PageResponse::Event messages"
        );
    }

    #[test]
    fn page_response_event_serializes_without_req_id() {
        let response = crate::cdp_driver::PageResponse::Event {
            page_id: "page-1".to_string(),
            event: "response".to_string(),
            payload: json!({ "status": 201 }),
        };
        let encoded = serde_json::to_string(&response).unwrap();
        assert!(
            encoded.contains(r#""kind":"event""#),
            "async page event should use event kind: {encoded}"
        );
        assert!(
            encoded.contains(r#""page_id":"page-1""#),
            "async page event should carry target page id: {encoded}"
        );
        assert!(
            !encoded.contains("req_id"),
            "async page events are not request/response RPCs: {encoded}"
        );
    }

    #[test]
    fn network_event_key_includes_cdp_session_id() {
        assert_eq!(
            network_event_key(Some("session-a"), "request-1"),
            "session-a:request-1"
        );
        assert_eq!(network_event_key(None, "request-1"), ":request-1");
    }

    #[test]
    fn network_event_pump_skips_static_response_bodies() {
        let script = PendingNetworkResponse {
            session_id: Some("session-a".to_string()),
            url: "http://127.0.0.1:5173/src/main.tsx".to_string(),
            method: "GET".to_string(),
            status: 200,
            resource_type: "Script".to_string(),
        };
        assert!(
            !should_forward_network_response(&script),
            "static scripts should not trigger Network.getResponseBody flooding"
        );
        let api = PendingNetworkResponse {
            resource_type: "Fetch".to_string(),
            url: "http://127.0.0.1:5173/api/projects".to_string(),
            ..script
        };
        assert!(should_forward_network_response(&api));
    }

    #[test]
    fn page_shim_locator_evaluate_uses_short_rpc_timeout() {
        assert!(
            PAGE_SHIM.contains("DEFAULT_LOCATOR_EVALUATE_RPC_TIMEOUT_MS = 60000"),
            "locator-internal evaluate RPCs should stay bounded while allowing slow hydration"
        );
        assert!(
            PAGE_SHIM.contains("timeout_ms: DEFAULT_LOCATOR_EVALUATE_RPC_TIMEOUT_MS"),
            "locator-internal evaluate requests should carry their per-RPC timeout"
        );
    }

    #[test]
    fn page_shim_close_uses_short_rpc_timeout() {
        assert!(
            PAGE_SHIM.contains("DEFAULT_PAGE_CLOSE_RPC_TIMEOUT_MS = 5000"),
            "page.close cleanup should not inherit long app-level timeouts"
        );
    }

    #[test]
    fn page_shim_failure_artifacts_use_short_screenshot_timeout() {
        assert!(
            WORKER_RUNTIME.contains("DEFAULT_FAILURE_SCREENSHOT_TIMEOUT_MS = 5000"),
            "failure screenshots should not inherit long app-level timeouts in visual mode"
        );
        assert!(
            PAGE_SHIM.contains("timeout_ms: opts && opts.timeout != null ? opts.timeout : null"),
            "page.screenshot timeout option should reach the Rust page request"
        );
    }

    #[test]
    fn screenshot_page_request_can_override_worker_timeout() {
        let req = crate::cdp_driver::PageRequest::Screenshot {
            req_id: 1,
            page_id: "page-1".to_string(),
            path: None,
            timeout_ms: Some(5_000),
        };
        assert_eq!(page_req_timeout_ms(&req, 300_000), 5_000);
    }

    #[test]
    fn evaluate_page_request_can_override_worker_timeout() {
        let req = crate::cdp_driver::PageRequest::Evaluate {
            req_id: 1,
            page_id: "page-1".to_string(),
            expression: "document.title".to_string(),
            timeout_ms: Some(5_000),
        };
        assert_eq!(page_req_timeout_ms(&req, 300_000), 5_000);
    }

    #[test]
    fn close_page_request_can_override_worker_timeout() {
        let req = crate::cdp_driver::PageRequest::Close {
            req_id: 1,
            page_id: "page-1".to_string(),
            timeout_ms: Some(5_000),
        };
        assert_eq!(page_req_timeout_ms(&req, 300_000), 5_000);
    }

    #[test]
    fn page_shim_stable_rect_allows_subpixel_layout_jitter() {
        assert!(
            PAGE_SHIM.contains("STABLE_RECT_EPSILON_PX = 1"),
            "locator Stable actionability should tolerate small browser layout jitter"
        );
        assert!(
            PAGE_SHIM.contains("function _rectsNear"),
            "locator Stable actionability should use a dedicated tolerant rect comparator"
        );
        assert!(
            !PAGE_SHIM.contains("prev[0] === cur[0]"),
            "locator Stable actionability must not require exact rect equality"
        );
    }

    #[test]
    fn page_shim_visibility_does_not_depend_on_offset_parent() {
        assert!(
            !PAGE_SHIM.contains("offsetParent === null"),
            "visibility should use style and client rects instead of offsetParent"
        );
        assert!(
            PAGE_SHIM.contains("el.getClientRects().length > 0"),
            "visibility should verify that the element has rendered client rects"
        );
    }

    #[test]
    fn page_rpc_timeout_error_names_action_and_budget() {
        let msg = format_page_rpc_timeout_error("evaluate", 120_000);
        assert!(msg.contains("evaluate"), "{msg}");
        assert!(msg.contains("120000ms"), "{msg}");
        assert!(msg.contains("browser RPC"), "{msg}");
    }

    #[test]
    fn compact_live_value_collapses_and_truncates_evaluate_expression() {
        let value = compact_live_value("  function() {\n  return document.body;\n}", 18);
        assert_eq!(value, "function() { retur...");
    }

    #[test]
    fn path_to_file_url_escapes_spaces() {
        let path = Path::new("/tmp/a b/c.js");
        let url = path_to_file_url(path);
        assert!(url.contains("a%20b"));
    }

    #[test]
    fn spec_slug_collapses_non_alnum() {
        assert_eq!(spec_slug_for(Path::new("/tmp/home.spec.ts")), "home-spec");
        assert_eq!(
            spec_slug_for(Path::new("/tmp/my test.spec.js")),
            "my-test-spec"
        );
        assert_eq!(spec_slug_for(Path::new("/tmp/plain")), "plain");
    }

    // @spec #2713 — text snapshot read/write/diff semantics.
    #[test]
    fn text_snapshot_writes_baseline_on_first_run() {
        let tmp = TempDir::new().unwrap();
        let spec = tmp.path().join("a.spec.ts");
        std::fs::write(&spec, "").unwrap();
        let res = load_or_write_text_snapshot(&spec, "a-spec", "g", "hello", false).unwrap();
        assert!(res.is_none(), "first run must pass");
        let baseline = tmp
            .path()
            .join("__snapshots__")
            .join("a-spec")
            .join("g.txt");
        assert_eq!(std::fs::read_to_string(baseline).unwrap(), "hello\n");
    }

    #[test]
    fn text_snapshot_matching_baseline_passes() {
        let tmp = TempDir::new().unwrap();
        let spec = tmp.path().join("a.spec.ts");
        std::fs::write(&spec, "").unwrap();
        // Seed.
        load_or_write_text_snapshot(&spec, "a-spec", "g", "hello", false).unwrap();
        // Re-run with same content.
        let res = load_or_write_text_snapshot(&spec, "a-spec", "g", "hello", false).unwrap();
        assert!(res.is_none(), "matching baseline must pass");
    }

    #[test]
    fn text_snapshot_mismatch_returns_diff_and_preserves_baseline() {
        let tmp = TempDir::new().unwrap();
        let spec = tmp.path().join("a.spec.ts");
        std::fs::write(&spec, "").unwrap();
        load_or_write_text_snapshot(&spec, "a-spec", "g", "stable", false).unwrap();

        let diff = load_or_write_text_snapshot(&spec, "a-spec", "g", "drift", false).unwrap();
        let diff = diff.expect("mismatch must return a MatcherDiff");
        assert_eq!(diff.expected, "stable");
        assert_eq!(diff.actual, "drift");
        // Baseline untouched.
        let baseline = tmp
            .path()
            .join("__snapshots__")
            .join("a-spec")
            .join("g.txt");
        assert_eq!(std::fs::read_to_string(baseline).unwrap(), "stable\n");
    }

    #[test]
    fn text_snapshot_update_rewrites_baseline_and_passes() {
        let tmp = TempDir::new().unwrap();
        let spec = tmp.path().join("a.spec.ts");
        std::fs::write(&spec, "").unwrap();
        load_or_write_text_snapshot(&spec, "a-spec", "g", "stale", false).unwrap();

        // update_snapshots = true → overwrite + pass even though content differs.
        let res = load_or_write_text_snapshot(&spec, "a-spec", "g", "fresh", true).unwrap();
        assert!(res.is_none(), "--update-snapshots must pass");
        let baseline = tmp
            .path()
            .join("__snapshots__")
            .join("a-spec")
            .join("g.txt");
        assert_eq!(std::fs::read_to_string(baseline).unwrap(), "fresh\n");
    }

    /// Pause/next live controls must round-trip through the on-disk
    /// control file the open-mode review surface writes. Locks in the
    /// shape expected by `wait_for_live_checkpoint`.
    // @spec #2613
    #[test]
    fn read_live_control_parses_pause_and_next_token() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("control.json");
        std::fs::write(
            &path,
            r#"{"paused": true, "speed_multiplier": 4, "next_token": 7, "replay_token": 2}"#,
        )
        .unwrap();
        let ctrl = read_live_control(&path);
        assert!(ctrl.paused);
        assert_eq!(ctrl.speed_multiplier, 4);
        assert_eq!(ctrl.next_token, 7);
    }

    #[test]
    fn live_step_delay_scales_with_control_speed_multiplier() {
        let tmp = TempDir::new().unwrap();
        let control_path = tmp.path().join("control.json");
        std::fs::write(&control_path, r#"{"speed_multiplier": 4}"#).unwrap();
        let live = LiveE2eConfig {
            event_log: tmp.path().join("events.jsonl"),
            control_path,
            step_delay_ms: 1000,
        };

        assert_eq!(live_step_delay_ms(&live), 250);
    }

    #[test]
    fn read_live_control_defaults_when_file_missing() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("missing.json");
        let ctrl = read_live_control(&path);
        assert!(!ctrl.paused);
        assert_eq!(ctrl.next_token, 0);
    }

    /// GH #3194 — Malformed control JSON (trailing comma — the canonical
    /// hand-edit / partial-write race) used to silently collapse into
    /// `LiveControlFile::default()` via `.ok().and_then(...)`. The user
    /// clicked "pause" in the UI and the runner kept charging through
    /// tests with no indication that the control transport broke.
    /// Post-fix: still returns Default for liveness (the runner cannot
    /// honor commands it cannot parse), but `tracing::warn!` surfaces
    /// the diagnostic so the developer can find the breakage.
    #[test]
    fn read_live_control_defaults_on_malformed_json() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("control.json");
        // Trailing comma — invalid JSON. Partial-write race produces
        // a similar shape.
        std::fs::write(&path, r#"{"paused": true, "next_token": 7,}"#).unwrap();

        let ctrl = read_live_control(&path);
        assert!(
            !ctrl.paused,
            "malformed control JSON must default to not-paused, not panic"
        );
        assert_eq!(ctrl.next_token, 0);
    }

    /// GH #3191 — Happy path: `append_jsonl` creates the parent dir,
    /// writes a newline-terminated JSON line, and appends correctly on
    /// repeat calls. Pins the contract the cclab UI's live-progress
    /// stream depends on.
    #[test]
    fn append_jsonl_writes_and_appends_lines() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nested").join("events.jsonl");
        let v1 = serde_json::json!({"kind": "start", "n": 1});
        let v2 = serde_json::json!({"kind": "result", "n": 2});

        append_jsonl(&path, &v1);
        append_jsonl(&path, &v2);

        let body = std::fs::read_to_string(&path).expect("file must exist after append");
        let mut lines = body.lines();
        assert_eq!(lines.next().unwrap(), r#"{"kind":"start","n":1}"#);
        assert_eq!(lines.next().unwrap(), r#"{"kind":"result","n":2}"#);
        assert!(lines.next().is_none());
    }

    /// GH #3191 — Failure path: an unreadable parent dir (where
    /// `create_dir_all` would race or fail) must not panic; the
    /// function returns silently after `tracing::warn!`. Pre-fix:
    /// `let _ = std::fs::create_dir_all(...)` already silently swallowed
    /// the error but then the next `open` would also silently fail —
    /// no log surface anywhere. Post-fix: returns without writing but
    /// surfaces the diagnostic via tracing.
    #[test]
    fn append_jsonl_does_not_panic_when_parent_is_a_file() {
        let tmp = TempDir::new().unwrap();
        // Make the would-be "parent" a FILE so that create_dir_all fails
        // with "Not a directory".
        let parent_as_file = tmp.path().join("blocked");
        std::fs::write(&parent_as_file, b"this is a file, not a dir").unwrap();
        let path = parent_as_file.join("events.jsonl");
        let v = serde_json::json!({"kind": "noop"});

        // No panic expected. The function returns ().
        append_jsonl(&path, &v);

        assert!(
            !path.exists(),
            "no jsonl file should appear under a non-directory parent"
        );
    }

    #[test]
    fn snapshot_first_run_writes_baseline() {
        let tmp = TempDir::new().unwrap();
        let spec = tmp.path().join("home.spec.ts");
        std::fs::write(&spec, "// empty").unwrap();
        let bytes = vec![1, 2, 3, 4];
        let out = load_or_write_snapshot(&spec, "home-spec", "hero", &bytes, false).unwrap();
        assert!(out.is_none(), "first run should pass");
        let stored = std::fs::read(
            tmp.path()
                .join("__snapshots__")
                .join("home-spec")
                .join("hero.png"),
        )
        .unwrap();
        assert_eq!(stored, bytes);
    }

    #[test]
    fn snapshot_matching_bytes_pass() {
        let tmp = TempDir::new().unwrap();
        let spec = tmp.path().join("home.spec.ts");
        std::fs::write(&spec, "// empty").unwrap();
        let bytes = vec![9, 9, 9];
        load_or_write_snapshot(&spec, "home-spec", "hero", &bytes, false).unwrap();
        let out = load_or_write_snapshot(&spec, "home-spec", "hero", &bytes, false).unwrap();
        assert!(out.is_none());
    }

    #[test]
    fn snapshot_mismatch_returns_diff() {
        let tmp = TempDir::new().unwrap();
        let spec = tmp.path().join("home.spec.ts");
        std::fs::write(&spec, "// empty").unwrap();
        load_or_write_snapshot(&spec, "home-spec", "hero", &[1, 2, 3], false).unwrap();
        let out = load_or_write_snapshot(&spec, "home-spec", "hero", &[4, 5, 6], false).unwrap();
        let diff = out.expect("bytes differ → expected Some(diff)");
        assert!(diff.actual.contains("3 bytes"));
        assert!(diff.expected.contains("3 bytes"));
    }

    #[test]
    fn snapshot_update_flag_overwrites() {
        let tmp = TempDir::new().unwrap();
        let spec = tmp.path().join("home.spec.ts");
        std::fs::write(&spec, "// empty").unwrap();
        load_or_write_snapshot(&spec, "home-spec", "hero", &[1, 2, 3], false).unwrap();
        let out = load_or_write_snapshot(&spec, "home-spec", "hero", &[4, 5, 6], true).unwrap();
        assert!(out.is_none(), "update=true should overwrite and pass");
        let stored = std::fs::read(
            tmp.path()
                .join("__snapshots__")
                .join("home-spec")
                .join("hero.png"),
        )
        .unwrap();
        assert_eq!(stored, vec![4, 5, 6]);
    }

    fn buf(test_id: &str) -> TraceBuffer {
        TraceBuffer::new(test_id, "/tmp/spec.ts", "fixture title")
    }

    #[test]
    fn test_id_slug_collapses_non_alphanumerics() {
        assert_eq!(test_id_slug("home > nav > opens"), "home---nav---opens");
        assert_eq!(test_id_slug("plain"), "plain");
        assert_eq!(test_id_slug(""), "test");
        assert_eq!(test_id_slug("///"), "test");
    }

    #[test]
    fn commit_trace_for_report_writes_zip_when_mode_on_and_test_passes() {
        let tmp = TempDir::new().unwrap();
        let trace_dir = tmp.path().join("traces");
        let path = commit_trace_for_report(
            Some(buf("case-a")),
            TestOutcome::Passed,
            WireTraceMode::On,
            &trace_dir,
            "home-spec",
            "case-a",
        )
        .expect("commit must succeed");
        let path = path.expect("On + Passed → Some(path)");
        assert!(
            path.exists(),
            "trace zip should exist at {}",
            path.display()
        );
        assert_eq!(
            path,
            trace_dir.join("home-spec__case-a.zip"),
            "deterministic filename"
        );
    }

    #[test]
    fn commit_trace_for_report_skips_zip_when_retain_on_failure_and_test_passes() {
        let tmp = TempDir::new().unwrap();
        let trace_dir = tmp.path().join("traces");
        let path = commit_trace_for_report(
            Some(buf("case-b")),
            TestOutcome::Passed,
            WireTraceMode::RetainOnFailure,
            &trace_dir,
            "home-spec",
            "case-b",
        )
        .expect("commit must succeed");
        assert!(path.is_none(), "retain-on-failure + Passed → no zip");
    }

    #[test]
    fn commit_trace_for_report_writes_zip_when_retain_on_failure_and_test_fails() {
        let tmp = TempDir::new().unwrap();
        let trace_dir = tmp.path().join("traces");
        let path = commit_trace_for_report(
            Some(buf("case-c")),
            TestOutcome::Failed,
            WireTraceMode::RetainOnFailure,
            &trace_dir,
            "home-spec",
            "case-c",
        )
        .expect("commit must succeed");
        let path = path.expect("retain-on-failure + Failed → Some(path)");
        assert!(path.exists());
    }

    #[test]
    fn commit_trace_for_report_writes_zip_when_test_times_out() {
        let tmp = TempDir::new().unwrap();
        let trace_dir = tmp.path().join("traces");
        let path = commit_trace_for_report(
            Some(buf("case-d")),
            TestOutcome::TimedOut,
            WireTraceMode::On,
            &trace_dir,
            "home-spec",
            "case-d",
        )
        .expect("commit must succeed");
        let path = path.expect("TimedOut counts as not-passed → zip written under On");
        assert!(path.exists());
    }

    #[test]
    fn commit_trace_for_report_returns_none_when_buffer_absent() {
        let tmp = TempDir::new().unwrap();
        let trace_dir = tmp.path().join("traces");
        let path = commit_trace_for_report(
            None,
            TestOutcome::Failed,
            WireTraceMode::On,
            &trace_dir,
            "home-spec",
            "case-e",
        )
        .expect("commit must succeed");
        assert!(path.is_none(), "no buffer → no trace_path");
        // Trace dir should not have been created when no work was done.
        assert!(!trace_dir.exists());
    }

    #[test]
    fn commit_trace_for_report_returns_none_when_test_skipped() {
        let tmp = TempDir::new().unwrap();
        let trace_dir = tmp.path().join("traces");
        let path = commit_trace_for_report(
            Some(buf("case-f")),
            TestOutcome::Skipped,
            WireTraceMode::On,
            &trace_dir,
            "home-spec",
            "case-f",
        )
        .expect("commit must succeed");
        assert!(path.is_none(), "skipped test → no trace_path");
    }

    #[test]
    fn wire_trace_mode_lowers_to_buffer_mode_variants() {
        use crate::trace::buffer::TraceMode;
        assert_eq!(
            wire_trace_mode_to_buffer_mode(WireTraceMode::Off),
            TraceMode::Off
        );
        assert_eq!(
            wire_trace_mode_to_buffer_mode(WireTraceMode::On),
            TraceMode::On
        );
        assert_eq!(
            wire_trace_mode_to_buffer_mode(WireTraceMode::RetainOnFailure),
            TraceMode::RetainOnFailure
        );
    }

    #[test]
    fn test_outcome_to_trace_outcome_maps_known_variants() {
        assert_eq!(
            test_outcome_to_trace_outcome(TestOutcome::Passed),
            Some(TraceOutcome::Passed)
        );
        assert_eq!(
            test_outcome_to_trace_outcome(TestOutcome::Failed),
            Some(TraceOutcome::Failed)
        );
        assert_eq!(
            test_outcome_to_trace_outcome(TestOutcome::TimedOut),
            Some(TraceOutcome::TimedOut)
        );
        assert_eq!(test_outcome_to_trace_outcome(TestOutcome::Skipped), None);
    }

    // ─── GH #3546 — worker stdin write failure surfacing ──────────────

    /// GH #3546 — fabricate a real `anyhow::Error` carrying an `io::Error`
    /// so the message-shape tests exercise the production code path.
    fn make_io_err(kind: std::io::ErrorKind, msg: &str) -> anyhow::Error {
        anyhow::Error::from(std::io::Error::new(kind, msg))
    }

    #[test]
    fn gh3546_format_worker_stdin_write_warn_names_kind_error_and_issue() {
        let err = make_io_err(std::io::ErrorKind::BrokenPipe, "pipe closed");
        let msg = format_worker_stdin_write_warn("WireResponse", &err);
        assert!(
            msg.contains("GH #3546"),
            "warning must carry the GH #3546 tag so operators grepping logs can land here: {msg}"
        );
        assert!(
            msg.contains("WireResponse"),
            "warning must name the RPC kind so operators can tell PageResponse-side vs WireResponse-side failures apart: {msg}"
        );
        assert!(
            msg.contains("pipe closed"),
            "warning must include the underlying io::Error so operators see the BrokenPipe / EPIPE detail: {msg}"
        );
    }

    #[test]
    fn gh3546_format_worker_stdin_write_warn_names_page_response_kind() {
        let err = make_io_err(std::io::ErrorKind::WriteZero, "wrote 0 bytes");
        let msg = format_worker_stdin_write_warn("PageResponse", &err);
        assert!(
            msg.contains("PageResponse"),
            "warning must name PageResponse when that RPC kind fails: {msg}"
        );
    }

    #[test]
    fn gh3546_format_worker_stdin_write_warn_hints_at_worker_exit_symptom() {
        let err = make_io_err(std::io::ErrorKind::BrokenPipe, "x");
        let msg = format_worker_stdin_write_warn("WireResponse", &err);
        assert!(
            msg.to_lowercase().contains("worker")
                && (msg.contains("BrokenPipe")
                    || msg.to_lowercase().contains("exited")
                    || msg.to_lowercase().contains("closed its stdin")),
            "warning must hint at the worker-exit symptom so operators searching for 'test runner stopped responding' or 'worker hung' find this line: {msg}"
        );
    }
}

#[cfg(test)]
mod gh3685_safe_worker_now_ms_tests {
    //! GH #3685 — `test_runner/worker.rs::now_ms()` used `.unwrap_or(0)` against
    //! `SystemTime::now().duration_since(UNIX_EPOCH)`, so every live e2e event
    //! (plan, case_started, case_finished, console, fatal, page step) would
    //! silently get `ts_ms: 0` whenever the host wall clock drifted before
    //! UNIX_EPOCH. The live-runner UI would then see a flat 1970 timeline,
    //! zero-duration cases, and false-positive stall detections — with no
    //! breadcrumb pointing at the host clock.
    //!
    //! These tests pin the safe-fallback helper and the warn message so the
    //! user-facing diagnostics survive future refactors.

    use super::{format_safe_worker_now_ms_warn, safe_worker_now_ms};
    use std::time::{Duration, UNIX_EPOCH};

    fn broken_clock_err() -> std::time::SystemTimeError {
        // Constructing a SystemTimeError directly is not in the public API,
        // so synthesize one by asking a pre-epoch SystemTime for its
        // duration_since(UNIX_EPOCH).
        (UNIX_EPOCH - Duration::from_secs(60))
            .duration_since(UNIX_EPOCH)
            .expect_err("pre-epoch SystemTime must produce SystemTimeError")
    }

    #[test]
    fn happy_path_returns_millis_and_no_warn() {
        let now = UNIX_EPOCH + Duration::from_millis(1_700_000_000_123);
        let (ms, warn) = safe_worker_now_ms(now);
        assert_eq!(ms, 1_700_000_000_123);
        assert!(
            warn.is_none(),
            "healthy clock must not produce a warn message"
        );
    }

    #[test]
    fn epoch_itself_returns_zero_and_no_warn() {
        let (ms, warn) = safe_worker_now_ms(UNIX_EPOCH);
        assert_eq!(ms, 0);
        assert!(
            warn.is_none(),
            "UNIX_EPOCH is a healthy clock, not a broken one — no warn"
        );
    }

    #[test]
    fn clock_before_epoch_returns_zero_and_warns_instead_of_silently_stamping_zero() {
        let now = UNIX_EPOCH - Duration::from_secs(3600);
        let (ms, warn) = safe_worker_now_ms(now);
        assert_eq!(ms, 0, "fallback value must be 0 (pre-existing behavior)");
        let msg = warn.expect(
            "clock-before-epoch must produce a warn message — the headline fix is that \
             we no longer silently stamp every live event with ts_ms=0",
        );
        assert!(
            msg.contains("GH #3685"),
            "warn message must carry the issue tag so future grep finds it: {msg}"
        );
    }

    #[test]
    fn warn_message_names_the_live_event_timeline_symptom() {
        let msg = format_safe_worker_now_ms_warn(&broken_clock_err());
        let lower = msg.to_lowercase();
        assert!(
            lower.contains("timeline")
                || lower.contains("live event")
                || lower.contains("1970")
                || lower.contains("ts_ms"),
            "warn must name the live-event-log symptom so operators searching for \
             'live runner shows 1970' or 'ts_ms=0' find this line: {msg}"
        );
        assert!(
            lower.contains("duration") || lower.contains("stall"),
            "warn must also name the case-duration or stall-detection symptom so \
             operators investigating either find this line: {msg}"
        );
    }

    #[test]
    fn warn_message_points_at_the_host_clock_fix_not_jet_code() {
        let msg = format_safe_worker_now_ms_warn(&broken_clock_err());
        let lower = msg.to_lowercase();
        assert!(
            lower.contains("host") && lower.contains("clock"),
            "warn must point at host clock as the fix: {msg}"
        );
        assert!(
            lower.contains("not a jet bug") || lower.contains("not jet"),
            "warn must explicitly disclaim this is a jet bug so operators don't \
             go hunting in jet source: {msg}"
        );
    }

    #[test]
    fn format_helper_round_trip_carries_observed_error_text() {
        let err = broken_clock_err();
        let msg = format_safe_worker_now_ms_warn(&err);
        // SystemTimeError's Display includes the duration the clock is "behind"
        // — surface that detail so operators can sanity-check how broken the
        // clock is.
        assert!(
            msg.contains(&err.to_string()),
            "warn must include the underlying SystemTimeError Display so operators \
             see how far behind the host clock is: msg={msg} err={err}"
        );
    }

    #[test]
    fn helper_output_is_deterministic_across_calls() {
        let now = UNIX_EPOCH + Duration::from_millis(42);
        let a = safe_worker_now_ms(now);
        let b = safe_worker_now_ms(now);
        assert_eq!(a, b, "same input must produce same output");

        let bad = UNIX_EPOCH - Duration::from_secs(10);
        let a = safe_worker_now_ms(bad);
        let b = safe_worker_now_ms(bad);
        assert_eq!(
            a, b,
            "broken-clock fallback must also be deterministic so warn messages don't churn"
        );
    }
}
// CODEGEN-END
