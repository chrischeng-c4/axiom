// HANDWRITE-BEGIN gap="missing-generator:unit-test:574fa347" tracker="pending-tracker" reason="Prove the real-PTY folder-to-cwd-to-Markdown/Git/AW journey and validate every retained manifest artifact and assertion."
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::OnceLock,
    thread,
    time::{Duration, Instant},
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use serde_json::{json, Value};
#[cfg(unix)]
use tauri::{
    ipc::{CallbackFn, InvokeBody},
    test::{get_ipc_response, mock_builder, mock_context, noop_assets, MockRuntime, INVOKE_KEY},
    webview::InvokeRequest,
    WebviewWindow, WebviewWindowBuilder,
};
use workbench::{
    context::{
        provenance::{
            ContextProvenanceItem, ProvenanceAuthority, ProviderIdentity, SourceLocation,
            SourcePosition, SourceSpan,
        },
        ContextDocumentKind,
    },
    folder_shell::{FolderShellStore, ShellState},
    native_agent_pty::{
        AgentKind, AgentLaunchCommand, PtyCommand, PtyLaunchError, PtyRuntime, PtySize,
    },
    production_journey::{
        render_journey_context, JourneySession, JourneySnapshot, ProductionJourneyStore,
    },
};

const PRODUCTION_COMMAND: &str = "cargo test -p workbench --test production_journey -- --nocapture";

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("apps/workbench lives below repository root")
        .to_path_buf()
}

fn evidence_root() -> PathBuf {
    repository_root().join("apps/workbench/evidence/production-journey/v1")
}

fn size() -> PtySize {
    PtySize {
        rows: 28,
        cols: 100,
        pixel_width: 0,
        pixel_height: 0,
    }
}

fn run_git(root: &Path, arguments: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .status()
        .expect("git command starts");
    assert!(status.success(), "git {} failed", arguments.join(" "));
}

fn poll_until(
    session: &mut JourneySession,
    predicate: impl Fn(&JourneySnapshot) -> bool,
) -> JourneySnapshot {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut latest = session.poll().expect("poll journey");
    while Instant::now() < deadline {
        if predicate(&latest) {
            return latest;
        }
        thread::sleep(Duration::from_millis(20));
        latest = session.poll().expect("poll journey");
    }
    panic!("journey condition timed out: {latest:?}");
}

fn exercise_real_journey(write_evidence: bool) -> JourneySnapshot {
    let fixture = tempfile::tempdir().expect("production fixture");
    let nested = fixture.path().join("nested");
    fs::create_dir(&nested).expect("nested cwd");
    fs::write(nested.join("aw.toml"), "[project]\nname = \"fixture\"\n").expect("AW activation");
    fs::write(nested.join("README.md"), "# Baseline\n").expect("baseline Markdown");
    fs::write(
        nested.join("tech-design.md"),
        "---\nid: fixture-td\nfill_sections: [logic]\n---\n\n# Fixture TD\n\n## Logic\n\nCanonical source.\n",
    )
    .expect("typed AW fixture");
    run_git(fixture.path(), &["init", "--quiet"]);
    run_git(
        fixture.path(),
        &["config", "user.email", "workbench@example.invalid"],
    );
    run_git(fixture.path(), &["config", "user.name", "Workbench Test"]);
    run_git(fixture.path(), &["add", "."]);
    run_git(fixture.path(), &["commit", "--quiet", "-m", "baseline"]);
    fs::write(
        nested.join("README.md"),
        "# Workbench production fixture\n\nRead-only Markdown context.\n",
    )
    .expect("modified Markdown");
    let markdown_before = fs::read(nested.join("README.md")).expect("before Markdown");
    let td_before = fs::read(nested.join("tech-design.md")).expect("before TD");

    let mut folders = ShellState::default();
    let selected = folders
        .register_path(fixture.path())
        .expect("register root");
    folders.select(&selected.id).expect("select root");
    assert_eq!(
        folders.selected_launch_path(),
        fixture
            .path()
            .canonicalize()
            .ok()
            .and_then(|path| path.to_str().map(str::to_owned))
            .as_deref()
    );

    let script = concat!(
        "cd \"$1\"; ",
        "printf '\\033]7;file://localhost%s\\007' \"$PWD\"; ",
        "printf 'READY:%s\\n' \"$PWD\"; ",
        "IFS= read -r line; ",
        "printf 'ECHO:%s\\n' \"$line\"; ",
        "stty size; exit 0"
    );
    let command = PtyCommand::new("/bin/sh", fixture.path()).args([
        "-c",
        script,
        "workbench-production-fixture",
        nested.to_str().expect("UTF-8 fixture path"),
    ]);
    let mut session = JourneySession::spawn_command("Fixture agent", &command, size())
        .expect("real production PTY");
    session.resize(42, 132).expect("resize production PTY");
    let ready = poll_until(&mut session, |snapshot| {
        snapshot.transcript.contains("READY:")
    });
    assert_eq!(ready.cwd_source, "OSC 7");
    assert_eq!(
        Path::new(&ready.active_cwd),
        nested.canonicalize().expect("canonical nested cwd")
    );
    session
        .send_input(b"show production context\n")
        .expect("terminal input");
    let complete = poll_until(&mut session, |snapshot| {
        !snapshot.running
            && snapshot.transcript.contains("ECHO:show production context")
            && snapshot.transcript.contains("42 132")
    });
    assert_eq!(complete.exit_code, Some(0));

    let nested_text = nested.to_str().expect("UTF-8 nested path").to_owned();
    let markdown = render_journey_context(nested_text.clone(), Some("README.md".to_owned()))
        .expect("Markdown context");
    let git = render_journey_context(nested_text.clone(), None).expect("Git context");
    let typed =
        render_journey_context(nested_text, Some("tech-design.md".to_owned())).expect("AW context");
    assert_eq!(markdown.kind, ContextDocumentKind::Markdown);
    assert_eq!(git.kind, ContextDocumentKind::Git);
    assert_eq!(typed.kind, ContextDocumentKind::AwTyped);
    assert!(markdown.body_html.contains("Workbench production fixture"));
    assert!(git.body_html.contains("README.md"));
    assert!(typed.body_html.contains("Fixture TD"));
    assert!(typed
        .navigation
        .iter()
        .any(|navigation| navigation.path == Path::new("tech-design.md")));

    let provenance = ContextProvenanceItem::extracted(
        ProviderIdentity::new("aw-typed", "AW typed renderer"),
        SourceLocation::with_span(
            "tech-design.md",
            SourceSpan::new(SourcePosition::new(7, 1), SourcePosition::new(8, 1)),
        ),
    )
    .resolve(&nested);
    assert_eq!(provenance.authority, ProvenanceAuthority::Canonical);
    assert!(provenance.badge.contains("canonical source"));
    assert_eq!(markdown_before, fs::read(nested.join("README.md")).unwrap());
    assert_eq!(td_before, fs::read(nested.join("tech-design.md")).unwrap());

    if write_evidence {
        let evidence = evidence_root();
        fs::create_dir_all(&evidence).expect("evidence directory");
        fs::write(evidence.join("pty-transcript.txt"), &complete.transcript)
            .expect("PTY transcript evidence");
        let context = json!({
            "schemaVersion": "workbench.production-context.evidence.v1",
            "generatedBy": PRODUCTION_COMMAND,
            "renderers": {
                "markdown": markdown.renderer_id,
                "git": git.renderer_id,
                "aw": typed.renderer_id,
            },
            "sourceNavigation": typed.navigation,
            "provenance": {
                "authority": format!("{:?}", provenance.authority).to_ascii_lowercase(),
                "badge": provenance.badge,
                "sourceCount": provenance.sources.len(),
            },
        });
        fs::write(
            evidence.join("context-summary.json"),
            serde_json::to_vec_pretty(&context).expect("context evidence JSON"),
        )
        .expect("context summary evidence");
    }

    complete
}

fn ensure_ui_evidence() -> Result<(), String> {
    static RESULT: OnceLock<Result<(), String>> = OnceLock::new();
    RESULT
        .get_or_init(|| {
            let root = repository_root();
            let jet_evidence = tempfile::tempdir().map_err(|error| error.to_string())?;
            let output = Command::new("jet")
                .current_dir(&root)
                .arg("e2e")
                .arg("run")
                .arg("--trace")
                .arg("on")
                .arg("--timeout")
                .arg("60000")
                .arg("--workers")
                .arg("1")
                .arg("--evidence-dir")
                .arg(jet_evidence.path())
                .arg(root.join("apps/workbench/e2e/production-journey.spec.js"))
                .output()
                .map_err(|error| format!("launch Jet: {error}"))?;
            if !output.status.success() {
                return Err(format!(
                    "Jet production journey failed\nstdout:\n{}\nstderr:\n{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            Ok(())
        })
        .clone()
}

#[cfg(unix)]
fn invoke_json(
    webview: &WebviewWindow<MockRuntime>,
    command: &str,
    body: Value,
) -> Result<Value, String> {
    get_ipc_response(
        webview,
        InvokeRequest {
            cmd: command.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost"
                .parse()
                .expect("valid local Tauri origin"),
            body: InvokeBody::Json(body),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_owned(),
        },
    )
    .map(|body| body.deserialize::<Value>().expect("command JSON response"))
    .map_err(|error| error.to_string())
}

#[cfg(unix)]
fn poll_ipc_until(
    webview: &WebviewWindow<MockRuntime>,
    predicate: impl Fn(&Value) -> bool,
    message: &str,
) -> Value {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut latest =
        invoke_json(webview, "poll_journey_agent", json!({})).expect("poll production IPC journey");
    while Instant::now() < deadline {
        if predicate(&latest) {
            return latest;
        }
        thread::sleep(Duration::from_millis(10));
        latest = invoke_json(webview, "poll_journey_agent", json!({}))
            .expect("poll production IPC journey");
    }
    panic!("{message}: {latest}");
}

#[cfg(unix)]
fn process_is_alive(pid: u32) -> bool {
    Command::new("/bin/kill")
        .args(["-0", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

#[cfg(unix)]
fn peak_rss_kib() -> u64 {
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::zeroed();
    let result = unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) };
    assert_eq!(result, 0, "getrusage must expose peak RSS");
    let raw = unsafe { usage.assume_init() }.ru_maxrss.max(0) as u64;
    if cfg!(target_os = "macos") {
        raw / 1024
    } else {
        raw
    }
}

#[cfg(unix)]
fn exercise_production_ipc_boundary() {
    const STABILITY_CYCLES: usize = 12;
    const MAX_LAUNCH_TO_READY_MS: u128 = 2_000;
    const MAX_PEAK_RSS_KIB: u64 = 512 * 1024;

    let fixture = tempfile::tempdir().expect("production IPC fixture");
    let root = fixture
        .path()
        .canonicalize()
        .expect("canonical IPC fixture");
    let nested = root.join("nested");
    let binaries = root.join("bin");
    fs::create_dir_all(&nested).expect("nested IPC cwd");
    fs::create_dir_all(&binaries).expect("deterministic agent bin directory");
    fs::write(nested.join("aw.toml"), "[project]\nname = \"fixture\"\n").expect("AW activation");
    fs::write(
        nested.join("README.md"),
        "# Workbench IPC fixture\n\nCanonical Markdown context.\n",
    )
    .expect("Markdown IPC fixture");
    fs::write(
        nested.join("tech-design.md"),
        "---\nid: ipc-td\nfill_sections: [logic]\n---\n\n# IPC Tech design\n\n## Logic\n\nCanonical source.\n",
    )
    .expect("typed IPC fixture");
    run_git(&root, &["init", "--quiet"]);
    run_git(
        &root,
        &["config", "user.email", "workbench@example.invalid"],
    );
    run_git(&root, &["config", "user.name", "Workbench Test"]);
    run_git(&root, &["add", "."]);
    run_git(&root, &["commit", "--quiet", "-m", "ipc baseline"]);
    fs::write(
        nested.join("README.md"),
        "# Workbench IPC fixture\n\nRead-only modified Markdown context.\n",
    )
    .expect("modified IPC Markdown");

    let agent = binaries.join("claude");
    fs::write(
        &agent,
        r##"#!/bin/sh
cd "$PWD/nested" || exit 91
printf '\033]7;file://localhost%s\007' "$PWD"
printf 'READY:%s\n' "$PWD"
trap 'printf "INTERRUPTED\n"; exit 130' INT
while IFS= read -r line; do
  if [ "$line" = "__exit__" ]; then exit 0; fi
  printf 'ECHO:%s\n' "$line"
  stty size
done
"##,
    )
    .expect("deterministic agent executable");
    let mut permissions = fs::metadata(&agent).expect("agent metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&agent, permissions).expect("executable agent fixture");

    let mut shell = ShellState::default();
    let registered = shell.register_path(&root).expect("register IPC folder");
    shell.select(&registered.id).expect("select IPC folder");
    let app = workbench::configure_builder(
        mock_builder(),
        FolderShellStore::with_state(shell),
        ProductionJourneyStore::with_runtime(PtyRuntime::with_search_path(binaries.as_os_str())),
    )
    .build(mock_context(noop_assets()))
    .expect("build production Tauri IPC app");
    let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("build production IPC webview");

    let selected = invoke_json(&webview, "selected_launch_path", json!({}))
        .expect("selected path through production IPC");
    assert_eq!(selected, json!(root.to_string_lossy()));
    let unavailable = invoke_json(
        &webview,
        "launch_journey_agent",
        json!({"agent": "agy", "cwd": root.to_string_lossy()}),
    )
    .expect_err("missing AGY must be recoverable through production IPC");
    assert!(unavailable.contains("agy is unavailable"), "{unavailable}");

    let mut child_pids = Vec::new();
    let mut launch_to_ready_max_ms = 0_u128;
    let mut peak_transcript_bytes = 0_usize;
    let mut rendered_context = None;
    for cycle in 0..STABILITY_CYCLES {
        let started = Instant::now();
        let launched = invoke_json(
            &webview,
            "launch_journey_agent",
            json!({"agent": "claude", "cwd": root.to_string_lossy()}),
        )
        .expect("launch deterministic agent through production IPC");
        let pid = launched["processId"]
            .as_u64()
            .expect("production snapshot exposes child pid") as u32;
        child_pids.push(pid);
        let ready = poll_ipc_until(
            &webview,
            |snapshot| {
                snapshot["cwdSource"] == "OSC 7"
                    && snapshot["transcript"]
                        .as_str()
                        .is_some_and(|value| value.contains("READY:"))
            },
            "production IPC launch never became ready",
        );
        launch_to_ready_max_ms = launch_to_ready_max_ms.max(started.elapsed().as_millis());
        assert_eq!(Path::new(ready["activeCwd"].as_str().unwrap()), nested);

        invoke_json(
            &webview,
            "resize_journey_agent",
            json!({"rows": 42, "cols": 132}),
        )
        .expect("resize through production IPC");
        invoke_json(
            &webview,
            "send_journey_input",
            json!({"input": format!("cycle-{cycle}")}),
        )
        .expect("send through production IPC");
        let echoed = poll_ipc_until(
            &webview,
            |snapshot| {
                snapshot["transcript"].as_str().is_some_and(|value| {
                    value.contains(&format!("ECHO:cycle-{cycle}")) && value.contains("42 132")
                })
            },
            "production IPC input/resize round trip failed",
        );
        peak_transcript_bytes = peak_transcript_bytes.max(
            echoed["transcript"]
                .as_str()
                .expect("transcript string")
                .len(),
        );

        if cycle == 0 {
            let root_text = nested.to_string_lossy();
            let git = invoke_json(
                &webview,
                "render_journey_context",
                json!({"root": root_text, "target": null}),
            )
            .expect("Git context through production IPC");
            let markdown = invoke_json(
                &webview,
                "render_journey_context",
                json!({"root": root_text, "target": "README.md"}),
            )
            .expect("Markdown context through production IPC");
            let typed = invoke_json(
                &webview,
                "render_journey_context",
                json!({"root": root_text, "target": "tech-design.md"}),
            )
            .expect("AW context through production IPC");
            assert_eq!(git["rendererId"], "git");
            assert_eq!(markdown["rendererId"], "markdown");
            assert_eq!(typed["rendererId"], "aw-typed");
            assert!(typed["navigation"]
                .as_array()
                .is_some_and(|items| !items.is_empty()));
            rendered_context = Some(json!({
                "git": git["rendererId"],
                "markdown": markdown["rendererId"],
                "aw": typed["rendererId"],
                "sourceNavigation": typed["navigation"].as_array().map(Vec::len),
            }));
        }

        let complete = match cycle % 3 {
            0 => {
                invoke_json(&webview, "interrupt_journey_agent", json!({}))
                    .expect("interrupt through production IPC");
                poll_ipc_until(
                    &webview,
                    |snapshot| snapshot["running"] == false,
                    "interrupted production IPC child did not exit",
                )
            }
            1 => invoke_json(&webview, "terminate_journey_agent", json!({}))
                .expect("terminate through production IPC"),
            _ => {
                invoke_json(&webview, "send_journey_input", json!({"input": "__exit__"}))
                    .expect("normal exit through production IPC");
                poll_ipc_until(
                    &webview,
                    |snapshot| snapshot["running"] == false,
                    "normally exiting production IPC child did not exit",
                )
            }
        };
        assert_eq!(complete["running"], false);
        assert_eq!(complete["processId"], Value::Null);
        assert!(
            complete["transcript"].as_str().unwrap().len()
                <= workbench::production_journey::MAX_TRANSCRIPT_BYTES
        );
    }

    thread::sleep(Duration::from_millis(25));
    assert!(
        child_pids.iter().all(|pid| !process_is_alive(*pid)),
        "production IPC leaked a child process: {child_pids:?}"
    );
    let selected_after = invoke_json(&webview, "selected_launch_path", json!({}))
        .expect("selected path after stability cycles");
    assert_eq!(selected_after, json!(root.to_string_lossy()));

    let peak_rss_kib = peak_rss_kib();
    assert!(
        launch_to_ready_max_ms <= MAX_LAUNCH_TO_READY_MS,
        "launch-to-ready {launch_to_ready_max_ms}ms exceeded {MAX_LAUNCH_TO_READY_MS}ms"
    );
    assert!(
        peak_rss_kib <= MAX_PEAK_RSS_KIB,
        "peak RSS {peak_rss_kib} KiB exceeded {MAX_PEAK_RSS_KIB} KiB"
    );
    assert!(peak_transcript_bytes <= workbench::production_journey::MAX_TRANSCRIPT_BYTES);

    let evidence = json!({
        "schemaVersion": "workbench.production-ipc.evidence.v1",
        "generatedBy": PRODUCTION_COMMAND,
        "boundary": "browser IPC request -> production configure_builder handler -> real PTY -> renderer registry",
        "deterministicSubstitute": "agent executable only",
        "folderSelectionPreserved": true,
        "unavailableAgentRecovered": true,
        "cycles": STABILITY_CYCLES,
        "lifecycleModes": ["interrupt", "terminate", "normal-exit"],
        "noLeakedChildren": true,
        "launchToReadyMaxMs": launch_to_ready_max_ms,
        "launchToReadyLimitMs": MAX_LAUNCH_TO_READY_MS,
        "peakRssKib": peak_rss_kib,
        "peakRssLimitKib": MAX_PEAK_RSS_KIB,
        "peakTranscriptBytes": peak_transcript_bytes,
        "transcriptLimitBytes": workbench::production_journey::MAX_TRANSCRIPT_BYTES,
        "context": rendered_context.expect("production context evidence"),
    });
    fs::write(
        evidence_root().join("ipc-journey.json"),
        serde_json::to_vec_pretty(&evidence).expect("IPC evidence JSON"),
    )
    .expect("production IPC evidence");
}

#[cfg(unix)]
fn ensure_production_ipc_evidence() -> Result<(), String> {
    static RESULT: OnceLock<Result<(), String>> = OnceLock::new();
    RESULT
        .get_or_init(|| {
            exercise_production_ipc_boundary();
            Ok(())
        })
        .clone()
}

#[cfg(unix)]
fn merge_ipc_manifest() {
    let path = evidence_root().join("manifest.json");
    let mut manifest: Value =
        serde_json::from_slice(&fs::read(&path).expect("production UI manifest"))
            .expect("production manifest JSON");
    manifest["artifacts"]["ipcJourney"] = json!({
        "path": "ipc-journey.json",
        "mediaType": "application/json"
    });
    manifest["assertions"]["productionTauriIpc"] = json!({
        "passed": true,
        "artifacts": ["ipc-journey.json", "pty-transcript.txt", "context-summary.json"]
    });
    manifest["assertions"]["efficiencyLimits"] = json!({
        "passed": true,
        "artifacts": ["ipc-journey.json"]
    });
    manifest["assertions"]["stabilityCycles"] = json!({
        "passed": true,
        "artifacts": ["ipc-journey.json"]
    });
    fs::write(
        path,
        format!("{}\n", serde_json::to_string_pretty(&manifest).unwrap()),
    )
    .expect("merge production IPC manifest assertions");
}

#[cfg(unix)]
#[test]
fn real_pty_folder_cwd_and_artifact_journey() {
    let snapshot = exercise_real_journey(true);
    assert!(snapshot.transcript.len() <= workbench::production_journey::MAX_TRANSCRIPT_BYTES);
}

#[cfg(unix)]
#[test]
fn unavailable_agent_is_recoverable() {
    let empty_path = tempfile::tempdir().expect("empty search path");
    let cwd = tempfile::tempdir().expect("launch cwd");
    let runtime = PtyRuntime::with_search_path(empty_path.path().as_os_str());
    let unavailable = AgentLaunchCommand::for_kind(AgentKind::Agy, cwd.path());
    match JourneySession::spawn_agent(&runtime, &unavailable, size()) {
        Err(PtyLaunchError::UnavailableBinary { program }) => {
            assert_eq!(program, Path::new("agy"));
        }
        Err(error) => panic!("unexpected unavailable-agent error: {error}"),
        Ok(_) => panic!("AGY unexpectedly resolved in empty search path"),
    }

    fs::write(cwd.path().join("README.md"), "# Recovery context\n").expect("recovery Markdown");
    let command = PtyCommand::new("/bin/sh", cwd.path()).args(["-c", "printf 'RECOVERED\\n'"]);
    let mut recovered =
        JourneySession::spawn_command("Recovery fixture", &command, size()).expect("retry PTY");
    let snapshot = poll_until(&mut recovered, |snapshot| {
        !snapshot.running && snapshot.transcript.contains("RECOVERED")
    });
    assert_eq!(snapshot.exit_code, Some(0));
    let document = render_journey_context(
        cwd.path().to_string_lossy().into_owned(),
        Some("README.md".to_owned()),
    )
    .expect("context after failed agent");
    assert_eq!(document.kind, ContextDocumentKind::Markdown);
}

#[test]
fn production_ui_quality_journey_passes() {
    ensure_ui_evidence().unwrap_or_else(|error| panic!("{error}"));
    assert_png_dimensions(&evidence_root().join("desktop.png"), 1440, 900);
    assert_png_dimensions(&evidence_root().join("constrained.png"), 860, 720);
}

#[cfg(unix)]
#[test]
fn production_tauri_ipc_bridge_journey_passes() {
    ensure_production_ipc_evidence().unwrap_or_else(|error| panic!("{error}"));
    let evidence: Value = serde_json::from_slice(
        &fs::read(evidence_root().join("ipc-journey.json")).expect("IPC evidence"),
    )
    .expect("IPC evidence JSON");
    assert_eq!(evidence["deterministicSubstitute"], "agent executable only");
    assert_eq!(evidence["folderSelectionPreserved"], true);
    assert_eq!(evidence["unavailableAgentRecovered"], true);
    assert_eq!(evidence["noLeakedChildren"], true);
    assert_eq!(evidence["context"]["git"], "git");
    assert_eq!(evidence["context"]["markdown"], "markdown");
    assert_eq!(evidence["context"]["aw"], "aw-typed");
}

#[cfg(unix)]
#[test]
fn production_efficiency_limits_hold() {
    ensure_production_ipc_evidence().unwrap_or_else(|error| panic!("{error}"));
    let evidence: Value = serde_json::from_slice(
        &fs::read(evidence_root().join("ipc-journey.json")).expect("IPC evidence"),
    )
    .expect("IPC evidence JSON");
    assert!(
        evidence["launchToReadyMaxMs"].as_u64().unwrap()
            <= evidence["launchToReadyLimitMs"].as_u64().unwrap()
    );
    assert!(
        evidence["peakRssKib"].as_u64().unwrap() <= evidence["peakRssLimitKib"].as_u64().unwrap()
    );
}

#[cfg(unix)]
#[test]
fn production_stability_cycles_are_leak_free() {
    ensure_production_ipc_evidence().unwrap_or_else(|error| panic!("{error}"));
    let evidence: Value = serde_json::from_slice(
        &fs::read(evidence_root().join("ipc-journey.json")).expect("IPC evidence"),
    )
    .expect("IPC evidence JSON");
    assert_eq!(evidence["cycles"], 12);
    assert_eq!(evidence["noLeakedChildren"], true);
    assert!(
        evidence["peakTranscriptBytes"].as_u64().unwrap()
            <= evidence["transcriptLimitBytes"].as_u64().unwrap()
    );
    assert_eq!(
        evidence["lifecycleModes"],
        json!(["interrupt", "terminate", "normal-exit"])
    );
}

#[cfg(unix)]
#[test]
fn retained_production_evidence_manifest_is_complete() {
    exercise_real_journey(true);
    ensure_ui_evidence().unwrap_or_else(|error| panic!("{error}"));
    ensure_production_ipc_evidence().unwrap_or_else(|error| panic!("{error}"));
    merge_ipc_manifest();
    let evidence = evidence_root();
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(evidence.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(
        manifest["schemaVersion"],
        "workbench.production-journey.evidence.v1"
    );
    assert_eq!(manifest["workItem"], 2201);
    assert_eq!(manifest["command"], PRODUCTION_COMMAND);
    for assertion in [
        "folderAgentCwd",
        "markdownGitAwContext",
        "sourceNavigation",
        "placeholderFreePrimaryState",
        "keyboardAccessibility",
        "constrainedReadability",
        "unavailableAgentRecovery",
        "productionTauriIpc",
        "efficiencyLimits",
        "stabilityCycles",
    ] {
        assert_eq!(manifest["assertions"][assertion]["passed"], true);
        assert!(manifest["assertions"][assertion]["artifacts"]
            .as_array()
            .is_some_and(|artifacts| !artifacts.is_empty()));
    }
    for artifact in [
        "desktop.png",
        "constrained.png",
        "pty-transcript.txt",
        "context-summary.json",
        "ipc-journey.json",
    ] {
        assert!(evidence.join(artifact).is_file(), "missing {artifact}");
    }
    let transcript = fs::read_to_string(evidence.join("pty-transcript.txt")).unwrap();
    assert!(transcript.contains("ECHO:show production context"));
    let context: serde_json::Value =
        serde_json::from_slice(&fs::read(evidence.join("context-summary.json")).unwrap()).unwrap();
    assert_eq!(context["renderers"]["markdown"], "markdown");
    assert_eq!(context["renderers"]["git"], "git");
    assert_eq!(context["renderers"]["aw"], "aw-typed");
    assert_eq!(context["provenance"]["authority"], "canonical");

    let capability = include_str!("../CAPABILITIES.md");
    let external_contract =
        include_str!("../external-contracts/behavior/folder-agent-artifact-journey.md");
    assert!(capability.contains(PRODUCTION_COMMAND));
    assert!(external_contract.contains(PRODUCTION_COMMAND));
}

fn assert_png_dimensions(path: &Path, width: u32, height: u32) {
    let bytes = fs::read(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert_eq!(u32::from_be_bytes(bytes[16..20].try_into().unwrap()), width);
    assert_eq!(
        u32::from_be_bytes(bytes[20..24].try_into().unwrap()),
        height
    );
}
// HANDWRITE-END
