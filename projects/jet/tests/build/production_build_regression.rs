// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#unit-test
// CODEGEN-BEGIN
//! Production-build regression gate for the representative React/MUI fixture.
//!
//! This intentionally drives the user-facing Jet CLI path:
//! `jet install --frozen-lockfile`, `jet build`, then `jet browser launch` on
//! the emitted `dist/index.html` over local HTTP. A plain build-complete line is not enough:
//! the built page must boot visibly and without browser console failures.

#[path = "../common/mod.rs"]
mod common;

use anyhow::{anyhow, Context, Result};
use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderValue, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde_json::{json, Value};
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::Duration;
use tokio::sync::oneshot;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("projects/")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn fixture_source() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("production-build-regression")
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src).with_context(|| format!("read {}", src.display()))? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).with_context(|| {
                format!("copy {} -> {}", src_path.display(), dst_path.display())
            })?;
        }
    }
    Ok(())
}

fn run_jet<I, S>(fixture: &Path, args: I) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(env!("CARGO_BIN_EXE_jet"))
        .args(args)
        .current_dir(fixture)
        .output()
        .context("run jet command")
}

fn write_output_artifacts(artifact_dir: &Path, phase: &str, output: &Output) -> Result<()> {
    fs::create_dir_all(artifact_dir)?;
    fs::write(
        artifact_dir.join(format!("{phase}.status.txt")),
        output.status.to_string(),
    )?;
    fs::write(
        artifact_dir.join(format!("{phase}.stdout.txt")),
        &output.stdout,
    )?;
    fs::write(
        artifact_dir.join(format!("{phase}.stderr.txt")),
        &output.stderr,
    )?;
    Ok(())
}

fn require_success(output: Output, phase: &str, artifact_dir: &Path) -> Result<Output> {
    write_output_artifacts(artifact_dir, phase, &output)?;
    if output.status.success() {
        return Ok(output);
    }
    Err(anyhow!(
        "{phase} failed; artifacts={}\nstatus={}\nstdout={}\nstderr={}",
        artifact_dir.display(),
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

struct StaticDistServer {
    url: String,
    shutdown: Option<oneshot::Sender<()>>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests.md#unit-test
impl StaticDistServer {
    async fn spawn(fixture: &Path) -> Result<Self> {
        let dist = fixture.join("dist");
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .context("bind static dist server")?;
        let addr = listener
            .local_addr()
            .context("read static dist server addr")?;
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let app = Router::new()
            .fallback(get(serve_static_dist_request))
            .with_state(StaticDistState { dist });

        tokio::spawn(async move {
            let _ = axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await;
        });

        Ok(Self {
            url: format!("http://{addr}/"),
            shutdown: Some(shutdown_tx),
        })
    }

    fn url(&self) -> String {
        self.url.clone()
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests.md#unit-test
impl Drop for StaticDistServer {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
    }
}

#[derive(Clone)]
struct StaticDistState {
    dist: PathBuf,
}

async fn serve_static_dist_request(State(state): State<StaticDistState>, uri: Uri) -> Response {
    let path = uri
        .path()
        .split('?')
        .next()
        .unwrap_or(uri.path())
        .trim_start_matches('/');

    if path.contains("..") || path.starts_with('/') {
        return (StatusCode::BAD_REQUEST, "Bad request").into_response();
    }

    let rel = if path.is_empty() { "index.html" } else { path };
    let file = state.dist.join(rel);
    match tokio::fs::read(&file).await {
        Ok(body) => {
            let mut response = Response::new(Body::from(body));
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static(content_type(&file)),
            );
            response
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            (StatusCode::NOT_FOUND, "Not found").into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to read {}: {err}", file.display()),
        )
            .into_response(),
    }
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn spawn_jet_browser(fixture: &Path, url: &str) -> Result<Child> {
    Command::new(env!("CARGO_BIN_EXE_jet"))
        .args(["browser", "launch", url])
        .current_dir(fixture)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawn jet browser launch")
}

async fn wait_for_browser_session(fixture: &Path) -> Result<()> {
    let session = fixture.join(".jet/browser-session.json");
    for _ in 0..150 {
        if session.exists() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Err(anyhow!(
        "jet browser launch did not write {}",
        session.display()
    ))
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
            return Err(anyhow!("{context} exited with {status}\nstderr={stderr}"));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(anyhow!("{context} did not exit after shutdown request"))
}

fn browser_eval_json(fixture: &Path, expression: &str) -> Result<Value> {
    let output = require_success(
        run_jet(fixture, ["browser", "eval", expression])?,
        "browser-eval",
        &std::env::temp_dir().join("jet-production-build-regression"),
    )?;
    serde_json::from_slice(&output.stdout)
        .with_context(|| format!("parse browser eval output for {expression:?}"))
}

fn shutdown_browser(fixture: &Path, child: &mut Child, artifact_dir: &Path) -> Result<String> {
    let output = run_jet(fixture, ["browser", "shutdown"])?;
    write_output_artifacts(artifact_dir, "browser-shutdown", &output)?;
    if !output.status.success() {
        return Err(anyhow!(
            "jet browser shutdown failed; artifacts={}\nstdout={}\nstderr={}",
            artifact_dir.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    wait_child_exit(child, "jet browser launch")
}

fn dist_file_summary(fixture: &Path) -> Result<Value> {
    let dist = fixture.join("dist");
    let mut files = Vec::new();
    if dist.exists() {
        for entry in fs::read_dir(&dist)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                files.push(json!({
                    "file": path.file_name().and_then(|name| name.to_str()).unwrap_or(""),
                    "bytes": entry.metadata()?.len(),
                }));
            }
        }
    }
    files.sort_by(|a, b| a["file"].as_str().cmp(&b["file"].as_str()));
    Ok(json!(files))
}

fn production_bundle_text(fixture: &Path) -> Result<String> {
    let mut scripts = Vec::new();
    scripts.extend(dist_js_files(fixture)?);
    if scripts.is_empty() {
        return Err(anyhow!(
            "production build did not emit any JS bundle under {}",
            fixture.join("dist").display()
        ));
    }

    let mut text = String::new();
    for script in scripts {
        text.push_str(
            &fs::read_to_string(&script)
                .with_context(|| format!("read production JS bundle {}", script.display()))?,
        );
        text.push('\n');
    }
    Ok(text)
}

fn dist_js_files(fixture: &Path) -> Result<Vec<PathBuf>> {
    let dist = fixture.join("dist");
    let mut scripts = Vec::new();
    for entry in fs::read_dir(&dist).with_context(|| format!("read {}", dist.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("js") {
            scripts.push(path);
        }
    }
    scripts.sort();
    Ok(scripts)
}

fn assert_dist_js_parseable_with_node(fixture: &Path, artifact_dir: &Path) -> Result<()> {
    let scripts = dist_js_files(fixture)?;
    if scripts.is_empty() {
        return Err(anyhow!(
            "production build did not emit any JS bundle under {}",
            fixture.join("dist").display()
        ));
    }

    let mut report = Vec::new();
    for script in scripts {
        let output = Command::new("node")
            .arg("--check")
            .arg(&script)
            .output()
            .with_context(|| format!("node --check {}", script.display()))?;
        report.push(json!({
            "file": script.file_name().and_then(|name| name.to_str()).unwrap_or(""),
            "status": output.status.to_string(),
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
        }));
        if !output.status.success() {
            fs::write(
                artifact_dir.join("dist-js-syntax.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .context("write dist JS syntax report")?;
            return Err(anyhow!(
                "production JS bundle failed syntax check; artifacts={}",
                artifact_dir.display()
            ));
        }
    }

    fs::write(
        artifact_dir.join("dist-js-syntax.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .context("write dist JS syntax report")?;
    Ok(())
}

fn assert_no_react_refresh_in_production_bundle(fixture: &Path, artifact_dir: &Path) -> Result<()> {
    let bundle_text = production_bundle_text(fixture)?;
    fs::write(
        artifact_dir.join("dist-bundle-refresh-scan.txt"),
        format!(
            "bytes={}\ncontains_react_refresh={}\ncontains_refresh_reg={}\ncontains_refresh_sig={}\ncontains_enqueue_update={}\n",
            bundle_text.len(),
            bundle_text.contains("/@react-refresh") || bundle_text.contains("react-refresh"),
            bundle_text.contains("RefreshReg"),
            bundle_text.contains("RefreshSig"),
            bundle_text.contains("enqueueUpdate"),
        ),
    )
    .context("write production refresh scan artifact")?;

    for marker in [
        "/@react-refresh",
        "react-refresh",
        "RefreshReg",
        "RefreshSig",
        "enqueueUpdate",
    ] {
        assert!(
            !bundle_text.contains(marker),
            "production jet build must not ship dev React Refresh marker {marker:?}; artifacts={}",
            artifact_dir.display()
        );
    }
    Ok(())
}

#[test]
fn production_build_minify_emits_parseable_js_bundle() -> Result<()> {
    if !common::node_available() {
        common::fail_missing_prerequisite(format!(
            "need node for production minify syntax gate (node={})",
            common::node_available(),
        ));
    }

    let artifact_dir = std::env::temp_dir().join("jet-production-build-minify-syntax");
    let _ = fs::remove_dir_all(&artifact_dir);
    fs::create_dir_all(&artifact_dir).expect("create production minify artifact dir");

    let temp = tempfile::tempdir().expect("temp fixture");
    let fixture = temp.path().join("production-build-regression");
    copy_dir_all(&fixture_source(), &fixture).expect("copy production build fixture");
    fs::copy(
        workspace_root()
            .join("examples")
            .join("mui-visual-demo")
            .join("jet-lock.yaml"),
        fixture.join("jet-lock.yaml"),
    )
    .expect("copy frozen fixture lockfile");

    require_success(
        run_jet(&fixture, ["install", "--frozen-lockfile"])?,
        "install",
        &artifact_dir,
    )
    .expect("jet install --frozen-lockfile");

    require_success(
        run_jet(&fixture, ["build", "--minify", "--sourcemap", "none"])?,
        "build-minify",
        &artifact_dir,
    )
    .expect("jet build --minify");

    fs::write(
        artifact_dir.join("dist-files.json"),
        serde_json::to_vec_pretty(&dist_file_summary(&fixture).unwrap()).unwrap(),
    )
    .expect("write dist summary");
    assert_dist_js_parseable_with_node(&fixture, &artifact_dir)
        .expect("minified production JS must parse");
    assert_no_react_refresh_in_production_bundle(&fixture, &artifact_dir)
        .expect("minified production bundle must not include React Refresh runtime");

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn production_build_regression_fixture_boots_in_browser() -> Result<()> {
    if !common::node_available() || !common::chromium_available() {
        common::fail_missing_prerequisite(format!(
            "need node + Chromium (node={} chromium={})",
            common::node_available(),
            common::chromium_available(),
        ));
    }

    let artifact_dir = std::env::temp_dir().join("jet-production-build-regression");
    let _ = fs::remove_dir_all(&artifact_dir);
    fs::create_dir_all(&artifact_dir).expect("create production build artifact dir");

    let temp = tempfile::tempdir().expect("temp fixture");
    let fixture = temp.path().join("production-build-regression");
    copy_dir_all(&fixture_source(), &fixture).expect("copy production build fixture");
    fs::copy(
        workspace_root()
            .join("examples")
            .join("mui-visual-demo")
            .join("jet-lock.yaml"),
        fixture.join("jet-lock.yaml"),
    )
    .expect("copy frozen fixture lockfile");

    require_success(
        run_jet(&fixture, ["install", "--frozen-lockfile"])?,
        "install",
        &artifact_dir,
    )
    .expect("jet install --frozen-lockfile");
    assert!(
        fixture.join("node_modules/react/package.json").exists()
            && fixture
                .join("node_modules/@mui/material/package.json")
                .exists(),
        "install phase did not hydrate required React/MUI packages; artifacts={}",
        artifact_dir.display()
    );

    require_success(run_jet(&fixture, ["build"])?, "build", &artifact_dir).expect("jet build");
    assert!(
        fixture.join("dist/index.html").exists(),
        "build did not emit dist/index.html; dist={}",
        dist_file_summary(&fixture).unwrap()
    );
    fs::write(
        artifact_dir.join("dist-files.json"),
        serde_json::to_vec_pretty(&dist_file_summary(&fixture).unwrap()).unwrap(),
    )
    .expect("write dist summary");
    assert_no_react_refresh_in_production_bundle(&fixture, &artifact_dir)
        .expect("production bundle must not include React Refresh runtime");

    let server = StaticDistServer::spawn(&fixture)
        .await
        .expect("serve dist over local HTTP");
    let url = server.url();
    let mut browser =
        spawn_jet_browser(&fixture, &url).expect("jet browser launch dist/index.html");
    wait_for_browser_session(&fixture)
        .await
        .expect("browser session");

    let ready_expr = r##"(() => ({
      ready: document.body?.innerText?.includes("Jet production regression fixture") ?? false,
      href: location.href,
      state: document.readyState,
      text: document.body?.innerText ?? "",
      errors: window.__jetProductionEvents ?? [],
      styles: [...document.styleSheets].length,
      resources: performance.getEntriesByType("resource").map((entry) => entry.name),
      rootHtml: document.querySelector("#root")?.innerHTML?.slice(0, 1000) ?? ""
    }))()"##;
    let mut diagnostics = Value::Null;
    for _ in 0..120 {
        diagnostics = browser_eval_json(&fixture, ready_expr).unwrap_or(Value::Null);
        if diagnostics
            .get("ready")
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    fs::write(
        artifact_dir.join("browser-diagnostics.json"),
        serde_json::to_vec_pretty(&diagnostics).unwrap(),
    )
    .expect("write browser diagnostics");
    let browser_stderr =
        shutdown_browser(&fixture, &mut browser, &artifact_dir).expect("jet browser shutdown");
    fs::write(artifact_dir.join("browser.stderr.txt"), browser_stderr)
        .expect("write browser stderr");

    assert!(
        diagnostics
            .get("ready")
            .and_then(|value| value.as_bool())
            .unwrap_or(false),
        "production build did not visibly boot; artifacts={}\ndiagnostics={}",
        artifact_dir.display(),
        serde_json::to_string_pretty(&diagnostics).unwrap()
    );
    assert!(
        diagnostics
            .get("errors")
            .and_then(|value| value.as_array())
            .is_some_and(|errors| errors.is_empty()),
        "production build emitted browser errors; artifacts={}\ndiagnostics={}",
        artifact_dir.display(),
        serde_json::to_string_pretty(&diagnostics).unwrap()
    );
    Ok(())
}
// CODEGEN-END
