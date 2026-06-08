// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Visual regression guard for real MUI on React DOM vs Jet WASM.
//!
//! This intentionally uses a committed fixture under examples/ instead of the
//! simplified parity oracle. The DOM side must load real MUI packages installed
//! by `jet install`, render non-blank content in Chromium, and give Jet WASM a
//! comparable external page surface.

mod common;

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use std::ffi::OsStr;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::Duration;

use common::react_oracle::{
    screenshot_diff_message, screenshot_summaries_match, screenshot_summary_from_png,
};

const VISUAL_DIAGNOSTICS_EXPR: &str = r#"({
  visualRoot: !!document.querySelector('#visual-root'),
  debugBridge: !!window.__jet_debug,
  rootHtml: document.querySelector('#root')?.innerHTML?.slice(0, 800) ?? '',
  bodyHtml: document.body?.innerHTML?.slice(0, 800) ?? '',
  events: window.__jetVisualEvents ?? [],
  resourceCount: performance.getEntriesByType('resource').length,
  muiResources: performance.getEntriesByType('resource')
    .map((entry) => entry.name)
    .filter((name) => name.includes('@mui'))
    .slice(-40),
  failedResources: performance.getEntriesByType('resource')
    .filter((entry) => entry.responseStatus && entry.responseStatus >= 400)
    .map((entry) => ({ name: entry.name, status: entry.responseStatus }))
    .slice(-40)
})"#;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("projects/")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

async fn free_port() -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind free port");
    listener.local_addr().expect("local addr").port()
}

fn missing_mui_install_deps(fixture: &Path) -> Vec<&'static str> {
    let required = [
        "node_modules/react/package.json",
        "node_modules/react-dom/package.json",
        "node_modules/@mui/material/package.json",
    ];
    required
        .iter()
        .filter(|rel| !fixture.join(rel).exists())
        .copied()
        .collect()
}

fn require_mui_install(fixture: &Path) {
    let missing = missing_mui_install_deps(fixture);
    assert!(
        missing.is_empty(),
        "examples/mui-visual-demo dependencies are missing: {missing:?}. Run `cd examples/mui-visual-demo && jet install` before this visual regression gate."
    );
}

async fn wait_for_http(url: &str) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()?;
    for _ in 0..720 {
        if client.get(url).send().await.is_ok() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    Err(anyhow!("server did not become ready at {url}"))
}

fn run_jet_command<I, S>(fixture: &Path, args: I) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let exe = env!("CARGO_BIN_EXE_jet");
    Command::new(exe)
        .args(args)
        .current_dir(fixture)
        .output()
        .context("run jet command")
}

fn require_success(output: Output, context: &str) -> Result<Output> {
    if output.status.success() {
        return Ok(output);
    }
    Err(anyhow!(
        "{context} failed\nstatus={}\nstdout={}\nstderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

fn spawn_jet<I, S>(fixture: &Path, args: I, context: &str) -> Result<Child>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let exe = env!("CARGO_BIN_EXE_jet");
    Command::new(exe)
        .args(args)
        .current_dir(fixture)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| context.to_string())
}

fn spawn_jet_dev(fixture: &Path, port: u16, wasm: bool) -> Result<Child> {
    let exe = env!("CARGO_BIN_EXE_jet");
    let port = port.to_string();
    let mut command = Command::new(exe);
    if wasm {
        command.args(["dev", "--wasm", "--debug", "-p", port.as_str()]);
    } else {
        command.args(["dev", "-p", port.as_str()]);
    }
    command
        .current_dir(fixture)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| {
            if wasm {
                "spawn jet dev --wasm --debug for MUI fixture"
            } else {
                "spawn jet dev for React DOM MUI fixture"
            }
            .to_string()
        })
}

fn run_jet_install(fixture: &Path) -> Result<()> {
    require_success(
        run_jet_command(fixture, ["install", "--frozen-lockfile"])?,
        "jet install --frozen-lockfile for examples/mui-visual-demo",
    )?;
    Ok(())
}

fn read_child_stderr(child: &mut Child) -> String {
    let mut stderr = String::new();
    if let Some(mut pipe) = child.stderr.take() {
        let _ = pipe.read_to_string(&mut stderr);
    }
    stderr
}

fn wait_child_exit(child: &mut Child, context: &str) -> Result<String> {
    for _ in 0..120 {
        if let Some(status) = child.try_wait()? {
            let stderr = read_child_stderr(child);
            if status.success() {
                return Ok(stderr);
            }
            return Err(anyhow!(
                "{context} exited unsuccessfully: status={status}\nstderr={}",
                truncate_for_failure(&stderr)
            ));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(anyhow!("{context} did not exit after Jet shutdown request"))
}

fn shutdown_jet_dev(fixture: &Path, port: u16, child: &mut Child) -> Result<String> {
    let _ = (fixture, port);
    if child.try_wait()?.is_none() {
        child.kill().context("kill jet dev process")?;
    }
    child.wait().context("wait for jet dev process")?;
    Ok(String::new())
}

fn truncate_for_failure(text: &str) -> String {
    const MAX: usize = 12_000;
    if text.len() <= MAX {
        return text.to_string();
    }
    format!("{}... <truncated {} bytes>", &text[..MAX], text.len() - MAX)
}

async fn wait_for_browser_session(fixture: &Path) -> Result<()> {
    let session_path = fixture.join(".jet/browser-session.json");
    for _ in 0..150 {
        if session_path.exists() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Err(anyhow!(
        "jet browser launch did not write {}",
        session_path.display()
    ))
}

fn jet_browser_eval_json(fixture: &Path, expression: &str) -> Result<Value> {
    let output = require_success(
        run_jet_command(fixture, ["browser", "eval", expression])?,
        "jet browser eval",
    )?;
    serde_json::from_slice(&output.stdout)
        .with_context(|| format!("parse jet browser eval output for {expression:?}"))
}

fn jet_browser_screenshot(fixture: &Path, out_path: &Path) -> Result<Vec<u8>> {
    let out = out_path.display().to_string();
    require_success(
        run_jet_command(fixture, ["browser", "screenshot", "-o", out.as_str()])?,
        "jet browser screenshot",
    )?;
    std::fs::read(out_path).with_context(|| format!("read {}", out_path.display()))
}

fn jet_browser_dom_capture(fixture: &Path, out_path: &Path) -> Value {
    let out = out_path.display().to_string();
    let output = run_jet_command(
        fixture,
        [
            "browser",
            "capture",
            "--surface",
            "dom",
            "--root-selector",
            "body",
            "--pretty",
            "-o",
            out.as_str(),
        ],
    );
    match output.and_then(|output| require_success(output, "jet browser capture --surface dom")) {
        Ok(_) => std::fs::read(out_path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_else(|| json!({ "error": "failed to read DOM capture output" })),
        Err(err) => json!({ "error": err.to_string() }),
    }
}
// CODEGEN-END

fn shutdown_jet_browser(fixture: &Path, child: &mut Child) -> Result<String> {
    require_success(
        run_jet_command(fixture, ["browser", "shutdown"])?,
        "jet browser shutdown",
    )?;
    wait_child_exit(child, "jet browser launch")
}

struct VisualSnapshot {
    body_text: String,
    diagnostics: Value,
    browser_capture: Value,
    png: Vec<u8>,
    screenshot_summary: Value,
    browser_stderr: String,
}

async fn page_snapshot(
    fixture: &Path,
    url: &str,
    artifact_dir: &Path,
    stem: &str,
) -> Result<VisualSnapshot> {
    let _ = std::fs::remove_file(fixture.join(".jet/browser-session.json"));
    let mut browser = spawn_jet(
        fixture,
        ["browser", "launch", url],
        "spawn jet browser launch for MUI fixture",
    )?;
    wait_for_browser_session(fixture).await?;
    let mut rendered = false;
    for _ in 0..300 {
        let ready = jet_browser_eval_json(
            fixture,
            "!!document.querySelector('#visual-root') && document.body.innerText.includes('MUI visual fixture')",
        )
        .unwrap_or(Value::Bool(false));
        if ready.as_bool().unwrap_or(false) {
            rendered = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let body_text = jet_browser_eval_json(fixture, "document.body.innerText")?
        .as_str()
        .unwrap_or_default()
        .to_string();
    let diagnostics = jet_browser_eval_json(fixture, VISUAL_DIAGNOSTICS_EXPR)?;
    let browser_capture = jet_browser_dom_capture(
        fixture,
        &artifact_dir.join(format!("{stem}-dom-capture.json")),
    );
    let screenshot = jet_browser_screenshot(fixture, &artifact_dir.join(format!("{stem}.png")))?;
    let screenshot_summary = screenshot_summary_from_png(&screenshot);
    let browser_stderr = shutdown_jet_browser(fixture, &mut browser)?;
    if !rendered {
        return Err(anyhow!(
            "{stem} page did not render #visual-root before snapshot\nbody={body_text:?}\ndiag={}",
            serde_json::to_string_pretty(&diagnostics).unwrap_or_else(|_| diagnostics.to_string())
        ));
    }
    Ok(VisualSnapshot {
        body_text,
        diagnostics,
        browser_capture,
        png: screenshot,
        screenshot_summary,
        browser_stderr,
    })
}

fn has_visible_mui_button_label(text: &str) -> bool {
    text.contains("MUI PRIMARY") || text.contains("MUI Primary")
}

fn write_visual_mismatch_artifacts(
    dom_png: &[u8],
    wasm_png: &[u8],
    diagnostics: &Value,
) -> Result<PathBuf> {
    let artifact_dir = std::env::temp_dir().join("jet-mui-visual-regression");
    std::fs::create_dir_all(&artifact_dir)?;
    std::fs::write(artifact_dir.join("react-dom.png"), dom_png)?;
    std::fs::write(artifact_dir.join("jet-wasm.png"), wasm_png)?;
    std::fs::write(
        artifact_dir.join("diagnostics.json"),
        serde_json::to_vec_pretty(diagnostics)?,
    )?;
    Ok(artifact_dir)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mui_visual_fixture_renders_on_react_dom_and_jet_wasm() {
    common::require_full_wasm_e2e_env();

    let fixture = workspace_root().join("examples/mui-visual-demo");
    let artifact_dir = std::env::temp_dir().join("jet-mui-visual-regression");
    std::fs::create_dir_all(&artifact_dir).expect("create visual artifact dir");

    if !missing_mui_install_deps(&fixture).is_empty() {
        run_jet_install(&fixture).expect("jet install for MUI fixture");
    }
    require_mui_install(&fixture);

    let dom_port = free_port().await;
    let dom_url = format!("http://127.0.0.1:{dom_port}/");
    let mut dom_server = spawn_jet_dev(&fixture, dom_port, false).expect("spawn jet dev");
    wait_for_http(&dom_url).await.expect("React DOM dev server");

    let dom_snapshot = page_snapshot(&fixture, &dom_url, &artifact_dir, "react-dom")
        .await
        .expect("React DOM page snapshot");
    let dom_server_stderr =
        shutdown_jet_dev(&fixture, dom_port, &mut dom_server).expect("jet dev shutdown for DOM");
    let dom_server_stderr = truncate_for_failure(&dom_server_stderr);
    let dom_text = dom_snapshot.body_text;
    let dom_diag = dom_snapshot.diagnostics;
    let dom_capture = dom_snapshot.browser_capture;
    let dom_png = dom_snapshot.png;
    let dom_screenshot = dom_snapshot.screenshot_summary;
    let dom_browser_stderr = truncate_for_failure(&dom_snapshot.browser_stderr);

    assert!(
        dom_text.contains("MUI visual fixture")
            && has_visible_mui_button_label(&dom_text)
            && dom_diag
                .get("visualRoot")
                .and_then(|value| value.as_bool())
                .unwrap_or(false),
        "React DOM MUI fixture rendered blank or incomplete.\nbody={dom_text:?}\ndiag={}\nbrowser_capture={}\njet_dev_stderr={dom_server_stderr}\njet_browser_stderr={dom_browser_stderr}",
        serde_json::to_string_pretty(&dom_diag).unwrap_or_else(|_| dom_diag.to_string()),
        serde_json::to_string_pretty(&dom_capture).unwrap_or_else(|_| dom_capture.to_string())
    );

    let wasm_port = free_port().await;
    let wasm_url = format!("http://127.0.0.1:{wasm_port}/");
    let mut wasm_server = spawn_jet_dev(&fixture, wasm_port, true)
        .expect("spawn jet dev --wasm --debug for MUI visual fixture");
    wait_for_http(&wasm_url).await.expect("Jet WASM dev server");

    let wasm_snapshot = page_snapshot(&fixture, &wasm_url, &artifact_dir, "jet-wasm")
        .await
        .expect("Jet WASM page snapshot");
    let wasm_server_stderr =
        shutdown_jet_dev(&fixture, wasm_port, &mut wasm_server).expect("jet dev shutdown for WASM");
    let wasm_server_stderr = truncate_for_failure(&wasm_server_stderr);
    let wasm_text = wasm_snapshot.body_text;
    let wasm_diag = wasm_snapshot.diagnostics;
    let wasm_capture = wasm_snapshot.browser_capture;
    let wasm_png = wasm_snapshot.png;
    let wasm_screenshot = wasm_snapshot.screenshot_summary;
    let wasm_browser_stderr = truncate_for_failure(&wasm_snapshot.browser_stderr);

    assert!(
        wasm_text.contains("MUI visual fixture") && has_visible_mui_button_label(&wasm_text),
        "Jet WASM MUI fixture rendered blank or incomplete.\nbody={wasm_text:?}\ndiag={}\nbrowser_capture={}\njet_dev_stderr={wasm_server_stderr}\njet_browser_stderr={wasm_browser_stderr}",
        serde_json::to_string_pretty(&wasm_diag).unwrap_or_else(|_| wasm_diag.to_string()),
        serde_json::to_string_pretty(&wasm_capture).unwrap_or_else(|_| wasm_capture.to_string())
    );
    if !screenshot_summaries_match(&dom_screenshot, &wasm_screenshot) {
        let diagnostics = json!({
            "react_dom": {
                "body": dom_text,
                "diagnostics": dom_diag,
                "browser_capture": dom_capture,
                "screenshot": dom_screenshot,
                "jet_browser_stderr": dom_browser_stderr,
                "jet_dev_stderr": dom_server_stderr,
            },
            "jet_wasm": {
                "body": wasm_text,
                "diagnostics": wasm_diag,
                "browser_capture": wasm_capture,
                "screenshot": wasm_screenshot,
                "jet_browser_stderr": wasm_browser_stderr,
                "jet_dev_stderr": wasm_server_stderr,
            }
        });
        let artifact_dir = write_visual_mismatch_artifacts(&dom_png, &wasm_png, &diagnostics).ok();
        panic!(
            "{}\nartifacts={}",
            screenshot_diff_message(
                "mui-visual-demo",
                "initial",
                &diagnostics["react_dom"]["screenshot"],
                &diagnostics["jet_wasm"]["screenshot"]
            ),
            artifact_dir
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<failed to write /tmp artifacts>".to_string())
        );
    }
}
