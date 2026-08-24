//! The shipped CLI's help surface: which command groups exist, how they are
//! spelled, and which of their outputs can be chained into the next verb.
//!
//! Every assertion runs the built binary and reads its real output. Help text
//! is a published interface — scripts and agents parse it — so a rename that
//! only a doc comment records is a breaking change nobody measured.
//!
//! The chainable assertions carry weight the help-text ones cannot. A verb can
//! be listed, spelled correctly, and still emit output that no downstream verb
//! is able to consume.
//!
//! ## Contracts inherited from the retired EC shells
//!
//! These 5 sentences were the whole of the `// Contract:` comment in 5 AW-EC shells
//! under `apps/lumen/e2e/`, each of which ran `cargo test -p lumen --test
//! cli_convention` in a subprocess and asserted the child's exit status. `cargo test -p
//! lumen` already runs this target directly, so the shells added a second, nested run
//! and nothing else. They were deleted on 2026-08-20 with the EC machinery they
//! belonged to, and the sentence is the only thing they held that nothing else did.
//! Each line below is prefixed with the EC id its shell was filed under.
//!
//! - `lumen-claim-chainable-artifact-render-follow-ups` — Artifact render commands
//!   write exactly one runnable trailing `next:` line when writing files and keep
//!   stream mode raw.
//! - `lumen-claim-chainable-backup-export-import-next` — Backup, export, and import
//!   helpers emit machine-readable next or terminal markers through the built binary.
//! - `lumen-claim-chainable-issue-upgrade-terminal` — Shared issue dry-run paths and
//!   upgrade check terminate with explicit `next: done` markers.
//! - `lumen-claim-cli-standard-issue-surface` — The shared issue group exposes search,
//!   view, create, and comment under `lumen issue`.
//! - `lumen-claim-cli-standard-upgrade-check` — The shared upgrade check surface
//!   remains available and emits an explicit terminal marker.
// HANDWRITE-BEGIN gap="missing-generator:unit-test:lumen-cli-convention" tracker="standardize-gap-projects-lumen-tests-cli-convention-rs" reason="CLI convention smoke test for the shared llm/upgrade/issue surface until the test generator owns binary-help assertions."
use cli_std::chainable::assert_chainable;
use serde_json::Value;
use std::process::Command;

/// #963: the deterministic `next:` tail line. Exactly one line, matching
/// `^next: \S`, and it must be the very last line of stdout.
fn assert_next_line_is_last(stdout: &str, context: &str) {
    let lines: Vec<&str> = stdout.lines().collect();
    let last = lines
        .last()
        .unwrap_or_else(|| panic!("{context}: empty stdout"));
    assert!(
        last.starts_with("next: ") && last.len() > "next: ".len(),
        "{context}: last line must match `^next: \\S`, got {last:?} in:\n{stdout}"
    );
    let next_lines: Vec<&&str> = lines.iter().filter(|l| l.starts_with("next: ")).collect();
    assert_eq!(
        next_lines.len(),
        1,
        "{context}: expected exactly one `next:` line, got {next_lines:?} in:\n{stdout}"
    );
}

fn assert_no_next_line(stdout: &str, context: &str) {
    assert!(
        !stdout.lines().any(|l| l.starts_with("next: ")),
        "{context}: stream mode must not emit a `next:` line in:\n{stdout}"
    );
}

fn run_command(command: &mut Command, context: &str) -> String {
    let output = command
        .output()
        .unwrap_or_else(|err| panic!("run {context}: {err}"));

    assert!(
        output.status.success(),
        "{context} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("lumen stdout is utf8")
}

fn run_chainable_command(command: &mut Command, context: &str) -> String {
    let stdout = run_command(command, context);
    assert_chainable(&stdout)
        .unwrap_or_else(|err| panic!("{context}: expected chainable stdout, got {err}\n{stdout}"));
    stdout
}

fn run_lumen(args: &[&str]) -> String {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lumen"));
    command.args(args);
    run_command(&mut command, &format!("lumen {args:?}"))
}

fn run_lumen_chainable(args: &[&str]) -> String {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lumen"));
    command.args(args);
    run_chainable_command(&mut command, &format!("lumen {args:?}"))
}

fn outline_llm_topic_commands() -> Vec<String> {
    let output = run_lumen(&["llm", "--topic", "outline", "--format", "json"]);
    let document: Value = serde_json::from_str(&output).expect("LLM outline JSON parses");
    assert_eq!(document["protocol"], "cclab.llm.v2");
    document["tasks"]
        .as_array()
        .expect("outline includes typed tasks")
        .iter()
        .map(|task| {
            let topic = task["topic"].as_str().expect("task topic is a string");
            format!("lumen llm --topic {topic}")
        })
        .collect()
}

#[test]
fn help_ships_standard_issue_group_not_report_issue() {
    let help = run_lumen(&["--help"]);
    for command in ["llm", "upgrade", "issue"] {
        assert!(help.contains(command), "missing `{command}` in:\n{help}");
    }
    assert!(
        !help.contains("report-issue"),
        "deprecated report-issue command still appears in:\n{help}"
    );
}

#[test]
fn llm_help_names_the_current_task_registry_and_library_composition() {
    let help = run_lumen(&["llm", "--help"]);
    for current in [
        "agent-facing task topics",
        "run-standalone",
        "querying",
        "verify-release",
        "provider content stays owned by its library",
    ] {
        assert!(help.contains(current), "LLM help missing `{current}`:\n{help}");
    }

    for retired in ["workflow", "integration", "quickstart", "recipes"] {
        let advertised_prefix = format!("- {retired}:");
        let backticked_topic = format!("`{retired}`");
        let topic_flag = format!("--topic {retired}");
        assert!(
            !help
                .lines()
                .any(|line| line.trim_start().starts_with(&advertised_prefix))
                && !help.contains(&backticked_topic)
                && !help.contains(&topic_flag),
            "LLM help still advertises retired topic {retired}:\n{help}"
        );
    }
}

/// #1095: direct SnapshotV1 movement verbs are visible alongside `backup`.
#[test]
fn help_ships_snapshot_data_movement_verbs() {
    let help = run_lumen(&["--help"]);
    for command in ["dump", "export", "load", "import", "backup"] {
        assert!(help.contains(command), "missing `{command}` in:\n{help}");
    }

    for command in ["dump", "export"] {
        let help = run_lumen(&[command, "--help"]);
        for expected in ["--url", "--out", "SnapshotV1"] {
            assert!(
                help.contains(expected),
                "missing `{expected}` in `lumen {command} --help`:\n{help}"
            );
        }
        // #2873: the credential these verbs used to take on the command line
        // is gone. Asserted here as well as in the residue gate because help
        // text is what a caller copies, and a flag that still appears in
        // `--help` is a flag someone will keep passing.
        assert!(
            !help.contains("--token"),
            "`lumen {command} --help` still advertises a credential flag:\n{help}"
        );
    }

    for command in ["load", "import"] {
        let help = run_lumen(&[command, "--help"]);
        for expected in ["--url", "--file", "/admin/restore"] {
            assert!(
                help.contains(expected),
                "missing `{expected}` in `lumen {command} --help`:\n{help}"
            );
        }
        assert!(
            !help.contains("--token"),
            "`lumen {command} --help` still advertises a credential flag:\n{help}"
        );
    }
}

/// Every topic advertised by the typed task manifest must parse through the
/// actual lumen binary.
#[test]
fn llm_outline_advertised_topic_commands_parse() {
    let commands = outline_llm_topic_commands();
    assert_eq!(
        commands.len(),
        14,
        "outline should advertise the fourteen DX task topics: {commands:?}"
    );

    for command in commands {
        let parts: Vec<&str> = command.split_whitespace().collect();
        assert_eq!(
            parts.first(),
            Some(&"lumen"),
            "unexpected command: {command}"
        );
        run_lumen(&parts[1..]);
    }
}

#[test]
fn llm_v2_executes_only_fully_bound_advertised_commands() {
    let outline = outline_llm_topic_commands();
    for topic_command in outline {
        let parts: Vec<&str> = topic_command.split_whitespace().collect();
        let mut args = parts[1..].to_vec();
        args.extend(["--format", "json"]);
        let detail = run_lumen(&args);
        let value: Value = serde_json::from_str(&detail).expect("LLM detail JSON parses");
        assert!(
            value.get("next").is_none(),
            "task navigation cannot emit `next`"
        );
        for step in value["runbook"]["steps"].as_array().unwrap() {
            let inputs = step.get("inputs").and_then(Value::as_array);
            if let Some(command) = step.get("command").and_then(Value::as_str) {
                assert!(inputs.map_or(true, Vec::is_empty));
                let command_parts: Vec<&str> = command.split_whitespace().collect();
                assert_eq!(command_parts.first(), Some(&"lumen"));
                run_lumen(&command_parts[1..]);
            } else {
                assert!(step.get("command_template").is_some());
                assert!(inputs.is_some_and(|inputs| !inputs.is_empty()));
            }
        }
    }
}

#[test]
fn issue_help_lists_search_view_create_comment() {
    let help = run_lumen(&["issue", "--help"]);
    for command in ["search", "view", "create", "comment"] {
        assert!(help.contains(command), "missing `{command}` in:\n{help}");
    }
}

/// #931: issue comment is the shared cli-std follow-up path; dry-run must be
/// offline-testable and show the reopen/comment preview without mutating GitHub.
#[test]
fn issue_comment_help_and_dry_run_preview() {
    let help = run_lumen(&["issue", "comment", "--help"]);
    for expected in ["<NUMBER>", "--repo", "--dry-run", "--yes", "[MSG]"] {
        assert!(
            help.contains(expected),
            "missing `{expected}` in `lumen issue comment --help`:\n{help}"
        );
    }

    let preview = run_lumen(&["issue", "comment", "123", "--dry-run", "still", "broken"]);
    for expected in [
        "repo:  chrischeng-c4/axiom",
        "issue: #123",
        "state: open",
        "still broken",
        "## Diagnostics",
        "- lumen version:",
        "- os/arch:",
    ] {
        assert!(
            preview.contains(expected),
            "missing `{expected}` in dry-run preview:\n{preview}"
        );
    }
    assert_next_line_is_last(&preview, "issue comment --dry-run");
}

#[test]
fn issue_create_comment_and_upgrade_check_outputs_are_chainable() {
    let created = run_lumen_chainable(&["issue", "create", "--dry-run", "test", "message"]);
    for expected in [
        "repo:  chrischeng-c4/axiom",
        "title: lumen: test message",
        "next: done",
    ] {
        assert!(
            created.contains(expected),
            "missing `{expected}` in create preview:\n{created}"
        );
    }

    let commented =
        run_lumen_chainable(&["issue", "comment", "123", "--dry-run", "still", "broken"]);
    assert!(commented.contains("issue: #123"));
    assert!(commented.contains("next: done"));

    let checked = run_lumen_chainable(&["upgrade", "--check"]);
    for expected in ["current:", "latest:", "next: done"] {
        assert!(
            checked.contains(expected),
            "missing `{expected}` in upgrade check:\n{checked}"
        );
    }
    assert!(
        checked.contains("latest:  unavailable (this build has no `online` feature)")
            || checked.contains("(lumen@"),
        "upgrade check latest line is neither offline fallback nor online release result:\n{checked}"
    );
}

#[test]
fn help_ships_dockerfile_and_layered_k8s_groups() {
    let help = run_lumen(&["--help"]);
    for command in ["dockerfile", "k8s"] {
        assert!(help.contains(command), "missing `{command}` in:\n{help}");
    }

    let k8s_help = run_lumen(&["k8s", "--help"]);
    for layer in ["crd", "operator", "instance"] {
        assert!(
            k8s_help.contains(layer),
            "missing `{layer}` in:\n{k8s_help}"
        );
    }

    let operator_help = run_lumen(&["k8s", "operator", "--help"]);
    for command in ["run", "render"] {
        assert!(
            operator_help.contains(command),
            "missing `{command}` in:\n{operator_help}"
        );
    }
}

#[test]
fn dockerfile_render_release_sets_version_and_strips_markers() {
    let rendered = run_lumen(&[
        "dockerfile",
        "render",
        "--variant",
        "release",
        "--version",
        "9.9.9",
    ]);

    assert!(rendered.contains("ARG LUMEN_VERSION=lumen@9.9.9"));
    assert!(rendered.contains("-t lumen:9.9.9"));
    assert!(!rendered.contains("SPEC-MANAGED"));
    assert!(!rendered.contains("CODEGEN-BEGIN"));
    assert!(!rendered.contains("CODEGEN-END"));
}

#[test]
fn k8s_crd_render_is_offline() {
    let rendered = run_lumen(&["k8s", "crd", "render"]);

    assert!(rendered.contains("kind: CustomResourceDefinition"));
    assert!(rendered.contains("name: lumens.lumen.dev"));
}

#[test]
fn k8s_instance_render_prod_accepts_app_namespace_overrides() {
    let rendered = run_lumen(&[
        "k8s",
        "instance",
        "render",
        "--profile",
        "prod",
        "--namespace",
        "search-prod",
        "--name",
        "catalog",
        "--image",
        "registry.example/lumen:9.9.9",
    ]);

    for expected in [
        "kind: Lumen",
        "  name: catalog",
        "  namespace: search-prod",
        "  image: registry.example/lumen:9.9.9",
        "  auth: required",
        "  observability: true",
    ] {
        assert!(
            rendered.contains(expected),
            "missing `{expected}` in:\n{rendered}"
        );
    }
}

/// #2678: `spec.auth` fails closed, so every profile states its posture
/// explicitly rather than relying on a reader knowing the default.
///
/// #2872 inverted the second half of this check. A `required` profile used to
/// have to name a token source; now it must name none, because the fields that
/// did are gone from the schema and a CR carrying one is rejected outright.
#[test]
fn k8s_instance_render_every_profile_states_its_auth_posture() {
    for profile in ["dev", "staging", "prod", "template"] {
        let rendered = run_lumen(&["k8s", "instance", "render", "--profile", profile]);
        let auth = rendered
            .lines()
            .find_map(|line| line.strip_prefix("  auth: "))
            .unwrap_or_else(|| panic!("profile `{profile}` renders no `auth:` line:\n{rendered}"));

        assert!(
            matches!(auth, "required" | "disabled"),
            "profile `{profile}`: `auth: {auth}` is not a CRD enum value"
        );
        // A profile must name *no* token source. The retired fields are gone
        // from the schema (#2872), so a rendered CR that still carried one
        // would be rejected by the API server at `kubectl apply` — the check
        // that used to demand one now proves the opposite.
        for retired in [
            "tokensSecret",
            "tokensSecretProviderClass",
            "identities",
            "identityAudiences",
        ] {
            assert!(
                !rendered.contains(&format!("\n  {retired}:")),
                "profile `{profile}` renders retired field `{retired}`:\n{rendered}"
            );
        }
    }
}

/// #963: file-writing modes end with exactly one `next: <command>` tail line
/// (shape `^next: \S`); stream-to-stdout modes (no `--out`) emit none, and
/// the streamed artifact bytes stay untouched. Offline, no network/server.
#[test]
fn chainable_output_next_line_file_writing_vs_stream() {
    let dir = tempfile::tempdir().expect("tempdir");

    // k8s crd render --out: `next: kubectl apply -f <out>`.
    let crd_out = dir.path().join("crd.yaml");
    let out = run_lumen_chainable(&["k8s", "crd", "render", "--out", crd_out.to_str().unwrap()]);
    assert_next_line_is_last(&out, "k8s crd render --out");
    assert!(out.contains(&format!("next: kubectl apply -f {}", crd_out.display())));

    // k8s crd render (stream to stdout): no `next:` line, artifact untouched.
    let streamed = run_lumen(&["k8s", "crd", "render"]);
    assert_no_next_line(&streamed, "k8s crd render (stream)");
    assert!(streamed.contains("kind: CustomResourceDefinition"));

    // k8s operator render --out: same `kubectl apply -f` shape.
    let operator_out = dir.path().join("operator.yaml");
    let out = run_lumen_chainable(&[
        "k8s",
        "operator",
        "render",
        "--out",
        operator_out.to_str().unwrap(),
    ]);
    assert_next_line_is_last(&out, "k8s operator render --out");
    assert!(out.contains(&format!(
        "next: kubectl apply -f {}",
        operator_out.display()
    )));

    // k8s instance render --out: same `kubectl apply -f` shape.
    let instance_out = dir.path().join("lumen.yaml");
    let out = run_lumen_chainable(&[
        "k8s",
        "instance",
        "render",
        "--out",
        instance_out.to_str().unwrap(),
    ]);
    assert_next_line_is_last(&out, "k8s instance render --out");
    assert!(out.contains(&format!(
        "next: kubectl apply -f {}",
        instance_out.display()
    )));

    // dockerfile render --out (release variant): matching `docker build`.
    let dockerfile_out = dir.path().join("Dockerfile.release");
    let out = run_lumen_chainable(&[
        "dockerfile",
        "render",
        "--variant",
        "release",
        "--version",
        "9.9.9",
        "--out",
        dockerfile_out.to_str().unwrap(),
    ]);
    assert_next_line_is_last(&out, "dockerfile render --variant release --out");
    assert!(out.contains("next: docker build "));
    assert!(out.contains("-t lumen:9.9.9"));

    // dockerfile render (stream, source variant): no `next:` line, and the
    // rendered artifact bytes are exactly what was printed (no trailer mixed
    // in with the Dockerfile body).
    let streamed = run_lumen(&["dockerfile", "render", "--variant", "source"]);
    assert_no_next_line(&streamed, "dockerfile render (stream)");

    // spec gen --out: no stream mode exists (`--out` is required), so this
    // verb always ends with a `next:` pointer at the generated entrypoint.
    let ts_out = dir.path().join("ts-client");
    let out = run_lumen_chainable(&[
        "spec",
        "gen",
        "--lang",
        "ts",
        "--out",
        ts_out.to_str().unwrap(),
    ]);
    assert_next_line_is_last(&out, "spec gen --lang ts --out");
    assert!(out.contains(&format!("next: {}", ts_out.join("index.ts").display())));
}

#[cfg(feature = "backup")]
#[tokio::test]
async fn backup_export_import_outputs_are_chainable() {
    use serde_json::Value;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let snapshot = br#"{"snapshot_version":1,"collections":[]}"#.to_vec();

    Mock::given(method("GET"))
        .and(path("/admin/backup"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(snapshot.clone()))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/admin/restore"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let base = server.uri();
    let dir = tempfile::tempdir().expect("tempdir");
    let export_path = dir.path().join("snapshot.json");
    let export_path_str = export_path.display().to_string();

    let mut export = Command::new(env!("CARGO_BIN_EXE_lumen"));
    export
        .arg("export")
        .arg("--url")
        .arg(&base)
        .arg("--out")
        .arg(&export_path_str);
    let export_stdout = run_chainable_command(&mut export, "lumen export --out");
    let export_json: Value =
        serde_json::from_str(&export_stdout).expect("export output is valid JSON");
    assert_eq!(export_json["status"], "exported");
    assert_eq!(export_json["path"], export_path_str);
    assert_eq!(
        export_json["next"],
        format!(
            "lumen import --url {} --file {}",
            base,
            export_path.display()
        )
    );

    let backup_root = dir.path().join("backup-root");
    std::fs::create_dir_all(&backup_root).expect("create backup root");
    let backup_dest = format!("file://{}", backup_root.display());

    let mut backup = Command::new(env!("CARGO_BIN_EXE_lumen"));
    backup
        .arg("backup")
        .arg("--url")
        .arg(&base)
        .arg("--dest")
        .arg(&backup_dest);
    let backup_stdout = run_chainable_command(&mut backup, "lumen backup");
    let backup_json: Value =
        serde_json::from_str(&backup_stdout).expect("backup output is valid JSON");
    assert!(
        backup_json["next"]
            .as_str()
            .expect("backup next command")
            .contains("/admin/restore"),
        "backup follow-up should point at the restore endpoint: {backup_stdout}"
    );

    let mut import = Command::new(env!("CARGO_BIN_EXE_lumen"));
    import
        .arg("import")
        .arg("--url")
        .arg(&base)
        .arg("--file")
        .arg(&export_path_str);
    let import_stdout = run_chainable_command(&mut import, "lumen import --file");
    let import_json: Value =
        serde_json::from_str(&import_stdout).expect("import output is valid JSON");
    assert_eq!(import_json["status"], "restored");
    assert_eq!(import_json["next"], "done");
}
// HANDWRITE-END
