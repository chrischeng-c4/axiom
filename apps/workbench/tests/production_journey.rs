// HANDWRITE-BEGIN gap="missing-generator:unit-test:574fa347" tracker="pending-tracker" reason="Prove the real-PTY folder-to-cwd-to-Markdown/Git/AW journey and validate every retained manifest artifact and assertion."
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
    thread,
    time::{Duration, Instant},
};

use serde_json::json;
use workbench::{
    context::{
        provenance::{
            ContextProvenanceItem, ProvenanceAuthority, ProviderIdentity, SourceLocation,
            SourcePosition, SourceSpan,
        },
        ContextDocumentKind,
    },
    folder_shell::ShellState,
    native_agent_pty::{
        AgentKind, AgentLaunchCommand, PtyCommand, PtyLaunchError, PtyRuntime, PtySize,
    },
    production_journey::{render_journey_context, JourneySession, JourneySnapshot},
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
fn retained_production_evidence_manifest_is_complete() {
    exercise_real_journey(true);
    ensure_ui_evidence().unwrap_or_else(|error| panic!("{error}"));
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
