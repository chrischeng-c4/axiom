// SPEC-MANAGED: projects/lumen/tech-design/interfaces/cli/lumen-issue-search-view-create-shared-cli-standard.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:lumen-cli-convention" tracker="standardize-gap-projects-lumen-tests-cli-convention-rs" reason="CLI convention smoke test for the shared llm/upgrade/issue surface until the test generator owns binary-help assertions."
use lumen::spec::llm_outline_md;
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

fn run_lumen(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_lumen"))
        .args(args)
        .output()
        .unwrap_or_else(|err| panic!("run lumen {args:?}: {err}"));

    assert!(
        output.status.success(),
        "lumen {args:?} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("lumen stdout is utf8")
}

fn outline_llm_topic_commands() -> Vec<String> {
    llm_outline_md()
        .lines()
        .filter_map(|line| {
            let start = line.find("`lumen llm --topic ")?;
            let command = &line[start + 1..];
            let end = command.find('`')?;
            Some(command[..end].to_string())
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

/// #1095: direct SnapshotV1 movement verbs are visible alongside `backup`.
/// @spec projects/lumen/tech-design/interfaces/cli/lumen-cli-add-dump-load-export-import-snapshot-verbs.md#unit-test
#[test]
fn help_ships_snapshot_data_movement_verbs() {
    let help = run_lumen(&["--help"]);
    for command in ["dump", "export", "load", "import", "backup"] {
        assert!(help.contains(command), "missing `{command}` in:\n{help}");
    }

    for command in ["dump", "export"] {
        let help = run_lumen(&[command, "--help"]);
        for expected in ["--url", "--out", "--token", "SnapshotV1"] {
            assert!(
                help.contains(expected),
                "missing `{expected}` in `lumen {command} --help`:\n{help}"
            );
        }
    }

    for command in ["load", "import"] {
        let help = run_lumen(&[command, "--help"]);
        for expected in ["--url", "--file", "--token", "/admin/restore"] {
            assert!(
                help.contains(expected),
                "missing `{expected}` in `lumen {command} --help`:\n{help}"
            );
        }
    }
}

/// #824: every topic command shown by `llm_outline_md()` must parse through
/// the actual lumen binary.
/// @spec projects/lumen/tech-design/interfaces/cli/self-docs-teach-positional-lumen-llm-topic-but-the-cli-only-acce.md#unit-test
#[test]
fn llm_outline_advertised_topic_commands_parse() {
    let commands = outline_llm_topic_commands();
    assert_eq!(
        commands.len(),
        6,
        "outline should advertise the six detail topics: {commands:?}"
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
fn issue_help_lists_search_view_create_comment() {
    let help = run_lumen(&["issue", "--help"]);
    for command in ["search", "view", "create", "comment"] {
        assert!(help.contains(command), "missing `{command}` in:\n{help}");
    }
}

/// #931: issue comment is the shared cli-std follow-up path; dry-run must be
/// offline-testable and show the reopen/comment preview without mutating GitHub.
/// @spec projects/lumen/tech-design/interfaces/cli/lumen-cli-add-issue-comment-auto-reopen-follow-up.md#unit-test
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

/// #963: file-writing modes end with exactly one `next: <command>` tail line
/// (shape `^next: \S`); stream-to-stdout modes (no `--out`) emit none, and
/// the streamed artifact bytes stay untouched. Offline, no network/server.
#[test]
fn chainable_output_next_line_file_writing_vs_stream() {
    let dir = tempfile::tempdir().expect("tempdir");

    // k8s crd render --out: `next: kubectl apply -f <out>`.
    let crd_out = dir.path().join("crd.yaml");
    let out = run_lumen(&["k8s", "crd", "render", "--out", crd_out.to_str().unwrap()]);
    assert_next_line_is_last(&out, "k8s crd render --out");
    assert!(out.contains(&format!("next: kubectl apply -f {}", crd_out.display())));

    // k8s crd render (stream to stdout): no `next:` line, artifact untouched.
    let streamed = run_lumen(&["k8s", "crd", "render"]);
    assert_no_next_line(&streamed, "k8s crd render (stream)");
    assert!(streamed.contains("kind: CustomResourceDefinition"));

    // k8s operator render --out: same `kubectl apply -f` shape.
    let operator_out = dir.path().join("operator.yaml");
    let out = run_lumen(&[
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
    let out = run_lumen(&[
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
    let out = run_lumen(&[
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
    let out = run_lumen(&[
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
// HANDWRITE-END
