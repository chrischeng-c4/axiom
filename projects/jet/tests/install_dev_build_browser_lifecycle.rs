// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! CLI lifecycle matrix for `jet install`, `jet dev`, `jet build`, and `jet browser`.
//!
//! The fixture is generated into a temp project so the test can exercise real
//! Jet-hosted React DOM and WASM flows without leaving build artifacts in the
//! repository checkout.

mod common;

use anyhow::{anyhow, Context, Result};
use jet::dev_session::{DevSession, DevSessionMode};
use serde_json::{json, Value};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::Duration;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(300);

const STATE_EXPR: &str = r#"(() => ({
  text: document.body?.innerText ?? '',
  visualRoot: !!document.querySelector('#visual-root'),
  rootHtml: document.querySelector('#root')?.innerHTML?.slice(0, 1000) ?? '',
  debugBridge: typeof window.__jet_debug,
  debugElementTree: (() => {
    try {
      return typeof window.__jet_debug === 'object'
        ? window.__jet_debug.elementTree()
        : null;
    } catch (error) {
      return { error: String(error) };
    }
  })(),
  debugPaintOps: (() => {
    try {
      return typeof window.__jet_debug === 'object'
        ? window.__jet_debug.paintOps().slice(0, 20)
        : [];
    } catch (error) {
      return [{ error: String(error) }];
    }
  })(),
  console: window.__jetLifecycleConsole ?? [],
  failedResources: performance.getEntriesByType('resource')
    .filter((entry) => entry.responseStatus && entry.responseStatus >= 400)
    .filter((entry) => !entry.name.endsWith('/favicon.ico'))
    .map((entry) => ({ name: entry.name, status: entry.responseStatus }))
    .slice(-40)
}))()"#;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn jet_cli_lifecycle_matrix_uses_shutdown_commands_and_persists_failure_artifacts() {
    common::require_full_wasm_e2e_env();

    let tmp = write_lifecycle_react_fixture().expect("write lifecycle React fixture");
    let fixture = tmp.path();
    let artifact_dir = std::env::temp_dir().join(format!(
        "jet-install-dev-build-browser-lifecycle-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&artifact_dir);
    fs::create_dir_all(&artifact_dir).expect("create lifecycle artifact dir");

    let result = run_lifecycle_matrix(fixture, &artifact_dir).await;
    if let Err(err) = &result {
        write_json_artifact(
            &artifact_dir,
            "failure.json",
            &json!({
                "error": err.to_string(),
                "artifact_dir": artifact_dir.display().to_string(),
            }),
        )
        .ok();
    }
    result.unwrap_or_else(|err| panic!("{err:#}\nartifacts: {}", artifact_dir.display()));
}

async fn run_lifecycle_matrix(fixture: &Path, artifact_dir: &Path) -> Result<()> {
    require_success(
        run_jet(fixture, &["install"], artifact_dir, "install")?,
        artifact_dir,
        "install",
    )?;

    require_success(
        run_jet(
            fixture,
            &["build", "--output", "dist-dom-lifecycle"],
            artifact_dir,
            "build-dom",
        )?,
        artifact_dir,
        "build-dom",
    )?;
    ensure_file(
        fixture.join("dist-dom-lifecycle/index.html"),
        "DOM build index.html",
    )?;

    require_success(
        run_jet(
            fixture,
            &[
                "build",
                "--wasm",
                "--debug",
                "--output",
                "dist-wasm-lifecycle",
            ],
            artifact_dir,
            "build-wasm-debug",
        )?,
        artifact_dir,
        "build-wasm-debug",
    )?;
    ensure_file(
        fixture.join("dist-wasm-lifecycle/app_bg.wasm"),
        "WASM debug build artifact",
    )?;

    let dom_state =
        exercise_dev_and_browser(fixture, artifact_dir, "dom", DevSessionMode::Dom, |port| {
            vec!["dev".to_string(), "-p".to_string(), port.to_string()]
        })
        .await?;
    assert_rendered_state(&dom_state, false, "DOM dev/browser");

    let wasm_state = exercise_dev_and_browser(
        fixture,
        artifact_dir,
        "wasm",
        DevSessionMode::WasmDebug,
        |port| {
            vec![
                "dev".to_string(),
                "--wasm".to_string(),
                "--debug".to_string(),
                "-p".to_string(),
                port.to_string(),
            ]
        },
    )
    .await?;
    assert_rendered_state(&wasm_state, true, "WASM dev/browser");

    Ok(())
}

async fn exercise_dev_and_browser<F>(
    fixture: &Path,
    artifact_dir: &Path,
    stem: &'static str,
    expected_mode: DevSessionMode,
    dev_args: F,
) -> Result<Value>
where
    F: FnOnce(u16) -> Vec<String>,
{
    let port = free_port().await;
    let url = format!("http://127.0.0.1:{port}/");
    let dev_phase = format!("{stem}-dev");
    let browser_phase = format!("{stem}-browser");

    let mut dev = ManagedJetChild::spawn(
        fixture,
        dev_args(port),
        ManagedKind::Dev,
        dev_phase.clone(),
        artifact_dir,
    )?;
    let session = wait_for_dev_session(fixture, expected_mode, port).await?;
    write_json_artifact(
        artifact_dir,
        &format!("{stem}-dev-session.json"),
        &serde_json::to_value(&session)?,
    )?;
    wait_for_http(&url).await?;

    let mut browser = ManagedJetChild::spawn(
        fixture,
        vec!["browser".into(), "launch".into(), url.clone()],
        ManagedKind::Browser,
        browser_phase.clone(),
        artifact_dir,
    )?;
    wait_for_browser_session(fixture).await?;
    let state = wait_for_rendered_state(fixture, artifact_dir, stem).await?;
    write_json_artifact(artifact_dir, &format!("{stem}-state.json"), &state)?;
    require_success(
        run_jet(
            fixture,
            &[
                "browser",
                "screenshot",
                "-o",
                &artifact_dir
                    .join(format!("{stem}.png"))
                    .display()
                    .to_string(),
            ],
            artifact_dir,
            &format!("{stem}-screenshot"),
        )?,
        artifact_dir,
        &format!("{stem}-screenshot"),
    )?;

    browser.shutdown(artifact_dir)?;
    dev.shutdown(artifact_dir)?;
    assert!(
        jet::browser_cli::session::read(fixture).is_err(),
        "{stem}: browser session file should be cleared after `jet browser shutdown`"
    );
    assert!(
        jet::dev_session::read(fixture).is_err(),
        "{stem}: dev session file should be cleared after `jet dev shutdown`"
    );
    Ok(state)
}

fn assert_rendered_state(state: &Value, expect_debug_bridge: bool, context: &str) {
    let text = state.get("text").and_then(|v| v.as_str()).unwrap_or("");
    if expect_debug_bridge {
        let debug_state = serde_json::to_string(state).unwrap_or_else(|_| state.to_string());
        assert!(
            debug_state.contains("Jet lifecycle fixture")
                && debug_state.contains("Lifecycle Primary"),
            "{context}: debug tree did not include fixture labels: {state}"
        );
        assert!(
            state
                .get("debugPaintOps")
                .and_then(|v| v.as_array())
                .map(|items| !items.is_empty())
                .unwrap_or(false),
            "{context}: paint ops were empty: {state}"
        );
    } else {
        assert!(
            text.contains("Jet lifecycle fixture") && text.contains("Lifecycle Primary"),
            "{context}: text snapshot did not include fixture labels: {state}"
        );
        assert_eq!(
            state.get("visualRoot").and_then(|v| v.as_bool()),
            Some(true),
            "{context}: visual root missing: {state}"
        );
    }
    assert_eq!(
        state.get("debugBridge").and_then(|v| v.as_str()),
        Some(if expect_debug_bridge {
            "object"
        } else {
            "undefined"
        }),
        "{context}: debug bridge state mismatch: {state}"
    );
    assert!(
        state
            .get("failedResources")
            .and_then(|v| v.as_array())
            .map(|items| items.is_empty())
            .unwrap_or(false),
        "{context}: browser reported failed resources: {state}"
    );
}

fn write_lifecycle_react_fixture() -> Result<tempfile::TempDir> {
    let scratch = workspace_root().join("target/jet-lifecycle-fixtures");
    fs::create_dir_all(&scratch)
        .with_context(|| format!("create fixture scratch {}", scratch.display()))?;
    let tmp = tempfile::Builder::new()
        .prefix("jet-lifecycle-react-")
        .tempdir_in(&scratch)
        .with_context(|| format!("create temp fixture in {}", scratch.display()))?;
    fs::write(
        tmp.path().join("package.json"),
        r#"{
  "name": "jet-lifecycle-react-fixture",
  "version": "0.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "jet dev",
    "dev:wasm": "jet dev --wasm --debug",
    "build": "jet build",
    "build:wasm": "jet build --wasm --debug"
  },
  "dependencies": {
    "react": "^18.3.1",
    "react-dom": "^18.3.1"
  }
}
"#,
    )?;
    fs::write(
        tmp.path().join("jet.config.toml"),
        r#"[wasm]
entry = "src/LifecycleFixture.tsx"
root_component = "LifecycleFixture"
root_props = ["Lifecycle Primary"]
"#,
    )?;
    fs::write(
        tmp.path().join("index.html"),
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>Jet Lifecycle Fixture</title>
</head>
<body>
  <div id="root"></div>
  <script type="module" src="/src/main.tsx"></script>
</body>
</html>
"#,
    )?;
    fs::create_dir_all(tmp.path().join("src"))?;
    fs::write(
        tmp.path().join("src/LifecycleFixture.tsx"),
        r#"import React from "react";

interface LifecycleFixtureProps {
  label: string;
}

export function LifecycleFixture({ label }: LifecycleFixtureProps) {
  return (
    <main id="visual-root">
      <h1>Jet lifecycle fixture</h1>
      <button id="primary">{label}</button>
    </main>
  );
}
"#,
    )?;
    fs::write(
        tmp.path().join("src/main.tsx"),
        r#"import React from "react";
import { createRoot } from "react-dom/client";
import { LifecycleFixture } from "./LifecycleFixture";

createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <LifecycleFixture label="Lifecycle Primary" />
  </React.StrictMode>
);
"#,
    )?;
    inject_console_capture(&tmp.path().join("index.html"))?;
    Ok(tmp)
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("projects/")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn inject_console_capture(index_html: &Path) -> Result<()> {
    let html = fs::read_to_string(index_html)?;
    let capture = r#"<script>
window.__jetLifecycleConsole = [];
for (const name of ["log", "warn", "error"]) {
  const original = console[name].bind(console);
  console[name] = (...args) => {
    window.__jetLifecycleConsole.push({
      level: name,
      text: args.map((arg) => String(arg)).join(" ")
    });
    return original(...args);
  };
}
</script>"#;
    let updated = html.replacen("</head>", &format!("{capture}\n</head>"), 1);
    fs::write(index_html, updated)?;
    Ok(())
}

async fn free_port() -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind free port");
    listener.local_addr().expect("local addr").port()
}

async fn wait_for_http(url: &str) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()?;
    for _ in 0..360 {
        if client.get(url).send().await.is_ok() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    Err(anyhow!("server did not become ready at {url}"))
}

async fn wait_for_dev_session(
    fixture: &Path,
    expected_mode: DevSessionMode,
    expected_port: u16,
) -> Result<DevSession> {
    for _ in 0..360 {
        if let Ok(session) = jet::dev_session::read(fixture) {
            if session.mode == expected_mode && session.port == expected_port {
                return Ok(session);
            }
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    Err(anyhow!(
        "dev session did not appear for mode={expected_mode:?} port={expected_port}"
    ))
}

async fn wait_for_browser_session(fixture: &Path) -> Result<()> {
    let path = jet::browser_cli::session::session_path(fixture);
    for _ in 0..180 {
        if path.exists() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Err(anyhow!(
        "browser session did not appear at {}",
        path.display()
    ))
}

async fn wait_for_rendered_state(fixture: &Path, artifact_dir: &Path, stem: &str) -> Result<Value> {
    let mut last = None;
    for _ in 0..180 {
        match browser_eval_json(
            fixture,
            STATE_EXPR,
            artifact_dir,
            &format!("{stem}-state-probe"),
        ) {
            Ok(state) => {
                let serialized =
                    serde_json::to_string(&state).unwrap_or_else(|_| state.to_string());
                if serialized.contains("Jet lifecycle fixture")
                    && serialized.contains("Lifecycle Primary")
                {
                    return Ok(state);
                }
                last = Some(state);
            }
            Err(err) => {
                last = Some(json!({ "error": err.to_string() }));
            }
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    Err(anyhow!(
        "{stem}: fixture never rendered expected state; last_state={}",
        last.unwrap_or_else(|| json!(null))
    ))
}

fn browser_eval_json(
    fixture: &Path,
    expression: &str,
    artifact_dir: &Path,
    phase: &str,
) -> Result<Value> {
    let output = require_success(
        run_jet(
            fixture,
            &["browser", "eval", expression],
            artifact_dir,
            phase,
        )?,
        artifact_dir,
        phase,
    )?;
    serde_json::from_slice(&output.stdout).with_context(|| format!("parse {phase} JSON"))
}

fn ensure_file(path: PathBuf, label: &str) -> Result<()> {
    if path.is_file() {
        return Ok(());
    }
    Err(anyhow!("{label} missing at {}", path.display()))
}

fn run_jet(fixture: &Path, args: &[&str], artifact_dir: &Path, phase: &str) -> Result<Output> {
    write_json_artifact(
        artifact_dir,
        &format!("{phase}-command.json"),
        &json!({
            "cwd": fixture.display().to_string(),
            "args": args,
        }),
    )?;
    let mut child = Command::new(env!("CARGO_BIN_EXE_jet"))
        .args(args)
        .current_dir(fixture)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("run jet {phase}"))?;
    let started = std::time::Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            let (stdout, stderr) = read_child_logs(&mut child);
            write_command_logs(artifact_dir, phase, &stdout, &stderr)?;
            return Ok(Output {
                status,
                stdout,
                stderr,
            });
        }
        if started.elapsed() >= COMMAND_TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            let (stdout, stderr) = read_child_logs(&mut child);
            write_command_logs(artifact_dir, phase, &stdout, &stderr)?;
            write_json_artifact(
                artifact_dir,
                &format!("{phase}-timeout.json"),
                &json!({
                    "phase": phase,
                    "timeout_ms": COMMAND_TIMEOUT.as_millis(),
                    "stdout_log": format!("{phase}.stdout.log"),
                    "stderr_log": format!("{phase}.stderr.log"),
                }),
            )?;
            return Err(anyhow!(
                "jet {phase} timed out after {}s",
                COMMAND_TIMEOUT.as_secs()
            ));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn require_success(output: Output, artifact_dir: &Path, phase: &str) -> Result<Output> {
    if output.status.success() {
        return Ok(output);
    }
    write_json_artifact(
        artifact_dir,
        &format!("{phase}-failure.json"),
        &json!({
            "phase": phase,
            "status": output.status.to_string(),
            "stdout_log": format!("{phase}.stdout.log"),
            "stderr_log": format!("{phase}.stderr.log"),
        }),
    )?;
    Err(anyhow!(
        "jet {phase} failed: status={}\nstdout={}\nstderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

#[derive(Clone, Copy)]
enum ManagedKind {
    Dev,
    Browser,
}

struct ManagedJetChild {
    child: Child,
    fixture: PathBuf,
    kind: ManagedKind,
    phase: String,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
impl ManagedJetChild {
    fn spawn(
        fixture: &Path,
        args: Vec<String>,
        kind: ManagedKind,
        phase: String,
        artifact_dir: &Path,
    ) -> Result<Self> {
        write_json_artifact(
            artifact_dir,
            &format!("{phase}-command.json"),
            &json!({
                "cwd": fixture.display().to_string(),
                "args": args,
            }),
        )?;
        let child = Command::new(env!("CARGO_BIN_EXE_jet"))
            .args(&args)
            .current_dir(fixture)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("spawn {phase}"))?;
        Ok(Self {
            child,
            fixture: fixture.to_path_buf(),
            kind,
            phase,
        })
    }

    fn shutdown(&mut self, artifact_dir: &Path) -> Result<()> {
        let shutdown_phase = format!("{}-shutdown", self.phase);
        let args = match self.kind {
            ManagedKind::Dev => ["dev", "shutdown"].as_slice(),
            ManagedKind::Browser => ["browser", "shutdown"].as_slice(),
        };
        require_success(
            run_jet(&self.fixture, args, artifact_dir, &shutdown_phase)?,
            artifact_dir,
            &shutdown_phase,
        )?;
        self.wait_exit(artifact_dir)
    }

    fn wait_exit(&mut self, artifact_dir: &Path) -> Result<()> {
        for _ in 0..240 {
            if let Some(status) = self.child.try_wait()? {
                let (stdout, stderr) = read_child_logs(&mut self.child);
                write_command_logs(artifact_dir, &self.phase, &stdout, &stderr)?;
                if status.success() {
                    return Ok(());
                }
                return Err(anyhow!(
                    "{} exited unsuccessfully after shutdown: status={status}\nstdout={}\nstderr={}",
                    self.phase,
                    String::from_utf8_lossy(&stdout),
                    String::from_utf8_lossy(&stderr)
                ));
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        Err(anyhow!(
            "{} did not exit after Jet shutdown request",
            self.phase
        ))
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
impl Drop for ManagedJetChild {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(Some(_))) {
            return;
        }
        let args = match self.kind {
            ManagedKind::Dev => ["dev", "shutdown"].as_slice(),
            ManagedKind::Browser => ["browser", "shutdown"].as_slice(),
        };
        let _ = Command::new(env!("CARGO_BIN_EXE_jet"))
            .args(args)
            .current_dir(&self.fixture)
            .output();
        for _ in 0..30 {
            if matches!(self.child.try_wait(), Ok(Some(_))) {
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn read_child_logs(child: &mut Child) -> (Vec<u8>, Vec<u8>) {
    let mut stdout = Vec::new();
    if let Some(mut pipe) = child.stdout.take() {
        let _ = pipe.read_to_end(&mut stdout);
    }
    let mut stderr = Vec::new();
    if let Some(mut pipe) = child.stderr.take() {
        let _ = pipe.read_to_end(&mut stderr);
    }
    (stdout, stderr)
}

fn write_command_logs(
    artifact_dir: &Path,
    phase: &str,
    stdout: &[u8],
    stderr: &[u8],
) -> Result<()> {
    fs::write(artifact_dir.join(format!("{phase}.stdout.log")), stdout)?;
    fs::write(artifact_dir.join(format!("{phase}.stderr.log")), stderr)?;
    Ok(())
}

fn write_json_artifact(artifact_dir: &Path, name: &str, value: &Value) -> Result<()> {
    fs::create_dir_all(artifact_dir)?;
    fs::write(
        artifact_dir.join(name),
        serde_json::to_vec_pretty(value).context("serialize artifact JSON")?,
    )?;
    Ok(())
}
// CODEGEN-END
