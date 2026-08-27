//! Deterministic contract test for Lumen standalone container and binary defaults.

use std::path::PathBuf;

const EXPECTED_DOCKERFILES: &[&str] = &[
    "apps/lumen/Dockerfile",
    "apps/lumen/Dockerfile.release",
    "apps/lumen/Dockerfile.test",
];

fn repo_root() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.parent()
        .and_then(|p| p.parent())
        .map(PathBuf::from)
        .unwrap_or(dir)
}

fn discover_dockerfiles() -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(repo_root().join("apps/lumen")) {
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if !file_type.is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("Dockerfile") {
                files.push(format!("apps/lumen/{name}"));
            }
        }
    }
    files.sort();
    files
}

fn replace_exact(source: &str, target: &str, replacement: &str) -> String {
    let count = source.matches(target).count();
    assert_eq!(
        count, 1,
        "expected target {target:?} to occur exactly once, found {count}"
    );
    source.replacen(target, replacement, 1)
}

fn insert_after_first_from(source: &str, instruction: &str) -> String {
    let mut output = String::with_capacity(source.len() + instruction.len() + 1);
    let mut inserted = false;
    for line in source.split_inclusive('\n') {
        output.push_str(line);
        if !inserted && line.trim_start().to_ascii_uppercase().starts_with("FROM ") {
            output.push_str(instruction);
            output.push('\n');
            inserted = true;
        }
    }
    assert!(inserted, "fixture has no FROM instruction");
    output
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockerfileValidationError {
    InventoryMismatch(Vec<String>, Vec<String>),
    NoStagesFound(String),
    MissingFinalStageInstruction(String, String),
    BuilderStageOnly(String, String),
    DuplicateInstruction(String, String, usize),
    MissingEntrypoint(String),
    PlacedAfterEntrypoint(String, String),
    ForbiddenHostFlagInEntrypoint(String),
    ForbiddenHostFlagInCmd(String),
    InvalidInstruction(String, String, String),
    ForbiddenFsyncKnob(String),
}

pub fn validate_inventory(discovered: &[String]) -> Result<(), DockerfileValidationError> {
    let expected: Vec<String> = EXPECTED_DOCKERFILES.iter().map(|s| s.to_string()).collect();
    if discovered != expected.as_slice() {
        return Err(DockerfileValidationError::InventoryMismatch(
            expected,
            discovered.to_vec(),
        ));
    }
    Ok(())
}

pub fn validate_dockerfile_content(
    path: &str,
    content: &str,
) -> Result<(), DockerfileValidationError> {
    let error_path = path.to_owned();
    let mut stages: Vec<Vec<(usize, &str)>> = Vec::new();
    let mut cur: Vec<(usize, &str)> = Vec::new();
    let mut in_stage = false;

    for (idx, line) in content.lines().enumerate() {
        let (num, trimmed) = (idx + 1, line.trim());
        let is_from =
            trimmed.starts_with("FROM ") || trimmed.to_ascii_uppercase().starts_with("FROM ");
        if is_from {
            if in_stage {
                stages.push(std::mem::take(&mut cur));
            }
            in_stage = true;
        }
        if in_stage {
            cur.push((num, line));
        }
    }
    if in_stage && !cur.is_empty() {
        stages.push(cur);
    }
    if stages.is_empty() {
        return Err(DockerfileValidationError::NoStagesFound(error_path));
    }

    let last_s = stages.len() - 1;
    let entrypoints: Vec<(usize, &str)> = stages[last_s]
        .iter()
        .filter(|(_, l)| l.trim().starts_with("ENTRYPOINT"))
        .copied()
        .collect();

    if entrypoints.is_empty() {
        return Err(DockerfileValidationError::MissingEntrypoint(error_path));
    }

    for (_, ep_line) in &entrypoints {
        if ep_line.contains("--host") {
            return Err(DockerfileValidationError::ForbiddenHostFlagInEntrypoint(
                error_path.clone(),
            ));
        }
    }

    let (last_ep_num, _) = entrypoints.last().unwrap();
    if stages.iter().flatten().any(|(_, line)| {
        line.trim()
            .strip_prefix("ENV ")
            .is_some_and(|value| value.starts_with("LUMEN_FSYNC"))
    }) {
        return Err(DockerfileValidationError::ForbiddenFsyncKnob(
            error_path.clone(),
        ));
    }

    const REQUIRED: [(&str, &str); 5] = [
        ("ENV LUMEN_HOST", "ENV LUMEN_HOST=0.0.0.0"),
        (
            "ENV LUMEN_DATA_DIR",
            "ENV LUMEN_DATA_DIR=/var/lib/lumen/data",
        ),
        ("ENV LUMEN_PERSISTENCE", "ENV LUMEN_PERSISTENCE=segment"),
        ("ENV LUMEN_WAL", "ENV LUMEN_WAL=embedded"),
        ("VOLUME", "VOLUME [\"/var/lib/lumen/data\"]"),
    ];
    for (prefix, expected) in REQUIRED {
        let matches: Vec<_> = stages
            .iter()
            .enumerate()
            .flat_map(|(stage, lines)| {
                lines.iter().filter_map(move |(line_number, line)| {
                    let line = line.trim();
                    (line == prefix
                        || line.starts_with(&format!("{prefix} "))
                        || line.starts_with(&format!("{prefix}=")))
                    .then_some((stage, *line_number, line))
                })
            })
            .collect();
        if matches.is_empty() {
            return Err(DockerfileValidationError::MissingFinalStageInstruction(
                error_path.clone(),
                expected.into(),
            ));
        }
        if matches.len() > 1 {
            return Err(DockerfileValidationError::DuplicateInstruction(
                error_path.clone(),
                expected.into(),
                matches.len(),
            ));
        }
        let (stage, line_number, actual) = matches[0];
        if stage != last_s {
            return Err(DockerfileValidationError::BuilderStageOnly(
                error_path.clone(),
                expected.into(),
            ));
        }
        if actual != expected {
            return Err(DockerfileValidationError::InvalidInstruction(
                error_path.clone(),
                prefix.into(),
                actual.into(),
            ));
        }
        if line_number >= *last_ep_num {
            return Err(DockerfileValidationError::PlacedAfterEntrypoint(
                error_path.clone(),
                expected.into(),
            ));
        }
    }

    if stages[last_s]
        .iter()
        .any(|(_, l)| l.trim().starts_with("CMD") && l.contains("--host"))
    {
        return Err(DockerfileValidationError::ForbiddenHostFlagInCmd(
            error_path,
        ));
    }
    Ok(())
}

fn for_each_dockerfile(f: impl Fn(&str, &str)) {
    let root = repo_root();
    for &rel_path in EXPECTED_DOCKERFILES {
        let content = std::fs::read_to_string(root.join(rel_path))
            .unwrap_or_else(|err| panic!("read {rel_path}: {err}"));
        f(rel_path, &content);
    }
}

#[test]
fn test_checked_in_dockerfiles_satisfy_contract() {
    let discovered = discover_dockerfiles();
    validate_inventory(&discovered).expect("inventory must match expected");
    for_each_dockerfile(|path, content| {
        validate_dockerfile_content(path, content)
            .unwrap_or_else(|err| panic!("validation failed for {path}: {err:?}"));
    });
}

#[test]
fn test_negative_fixture_inventory_mismatch() {
    let mut missing: Vec<String> = EXPECTED_DOCKERFILES.iter().map(|s| s.to_string()).collect();
    missing.pop();
    assert!(matches!(
        validate_inventory(&missing),
        Err(DockerfileValidationError::InventoryMismatch(_, _))
    ));

    let mut extra = missing;
    extra.extend([
        "apps/lumen/Dockerfile.test".into(),
        "apps/lumen/Dockerfile.extra".into(),
    ]);
    assert!(matches!(
        validate_inventory(&extra),
        Err(DockerfileValidationError::InventoryMismatch(_, _))
    ));
}

#[test]
fn test_negative_fixture_no_stages_found() {
    let no_from = "# syntax=docker/dockerfile:1\nENV LUMEN_HOST=0.0.0.0\n\
        ENTRYPOINT [\"/usr/local/bin/lumen\"]\nCMD [\"serve\"]";
    assert_eq!(
        validate_dockerfile_content("apps/lumen/Dockerfile", no_from),
        Err(DockerfileValidationError::NoStagesFound(
            "apps/lumen/Dockerfile".to_string()
        ))
    );
}

#[test]
fn test_negative_fixtures_per_file() {
    for_each_dockerfile(|path, content| {
        let p = path.to_string();
        let check = |mutated: String, err: DockerfileValidationError| {
            assert_eq!(validate_dockerfile_content(path, &mutated), Err(err));
        };

        check(
            replace_exact(content, "ENTRYPOINT [\"/usr/local/bin/lumen\"]\n", ""),
            DockerfileValidationError::MissingEntrypoint(p.clone()),
        );
        check(
            replace_exact(
                content,
                "ENTRYPOINT [\"/usr/local/bin/lumen\"]\n",
                "ENTRYPOINT [\"/usr/local/bin/lumen\"]\nENTRYPOINT [\"/usr/local/bin/lumen\", \"--host\", \"0.0.0.0\"]\n",
            ),
            DockerfileValidationError::ForbiddenHostFlagInEntrypoint(p.clone()),
        );
        check(
            replace_exact(
                content,
                "CMD [\"serve\"]",
                "CMD [\"serve\", \"--host\", \"0.0.0.0\"]",
            ),
            DockerfileValidationError::ForbiddenHostFlagInCmd(p.clone()),
        );
        check(
            replace_exact(
                content,
                "ENTRYPOINT [\"/usr/local/bin/lumen\"]\n",
                "ENV LUMEN_FSYNC=always\nENTRYPOINT [\"/usr/local/bin/lumen\"]\n",
            ),
            DockerfileValidationError::ForbiddenFsyncKnob(p.clone()),
        );

        let required = [
            (
                "ENV LUMEN_HOST",
                "ENV LUMEN_HOST=0.0.0.0",
                "ENV LUMEN_HOST=127.0.0.1",
            ),
            (
                "ENV LUMEN_DATA_DIR",
                "ENV LUMEN_DATA_DIR=/var/lib/lumen/data",
                "ENV LUMEN_DATA_DIR=/data",
            ),
            (
                "ENV LUMEN_PERSISTENCE",
                "ENV LUMEN_PERSISTENCE=segment",
                "ENV LUMEN_PERSISTENCE=cbor",
            ),
            (
                "ENV LUMEN_WAL",
                "ENV LUMEN_WAL=embedded",
                "ENV LUMEN_WAL=auto",
            ),
            (
                "VOLUME",
                "VOLUME [\"/var/lib/lumen/data\"]",
                "VOLUME [\"/data\"]",
            ),
        ];
        for (prefix, exact, wrong) in required {
            let exact_line = format!("{exact}\n");
            let missing = replace_exact(content, &exact_line, "");
            check(
                missing.clone(),
                DockerfileValidationError::MissingFinalStageInstruction(p.clone(), exact.into()),
            );
            check(
                replace_exact(content, &exact_line, &format!("{exact}\n{exact}\n")),
                DockerfileValidationError::DuplicateInstruction(p.clone(), exact.into(), 2),
            );
            check(
                replace_exact(content, exact, wrong),
                DockerfileValidationError::InvalidInstruction(
                    p.clone(),
                    prefix.into(),
                    wrong.into(),
                ),
            );
            check(
                replace_exact(
                    &missing,
                    "ENTRYPOINT [\"/usr/local/bin/lumen\"]\n",
                    &format!("ENTRYPOINT [\"/usr/local/bin/lumen\"]\n{exact}\n"),
                ),
                DockerfileValidationError::PlacedAfterEntrypoint(p.clone(), exact.into()),
            );
            check(
                insert_after_first_from(&missing, exact),
                DockerfileValidationError::BuilderStageOnly(p.clone(), exact.into()),
            );
        }
    });
}

#[tokio::test]
async fn test_bare_process_defaults_to_localhost() {
    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind free port");
        listener.local_addr().expect("local addr").port()
    };

    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_lumen"));
    cmd.args(["serve", "--port", &port.to_string()])
        .env_remove("LUMEN_HOST")
        .env_remove("RUST_LOG")
        .env("LUMEN_AUTH", "off")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    struct ChildGuard(Option<std::process::Child>);
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            if let Some(mut child) = self.0.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }

    let child = cmd.spawn().expect("failed to spawn lumen serve");
    let mut guard = ChildGuard(Some(child));

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(500))
        .no_proxy()
        .build()
        .expect("build reqwest client");

    let url = format!("http://127.0.0.1:{port}/healthz");
    let (start, timeout) = (
        std::time::Instant::now(),
        std::time::Duration::from_secs(15),
    );
    let mut healthy = false;

    while start.elapsed() < timeout {
        if client
            .get(&url)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
        {
            healthy = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    let mut child = guard.0.take().expect("child exists");
    let _ = child.kill();
    let output = child.wait_with_output().expect("wait for child output");
    let logs = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        healthy,
        "lumen serve did not answer /healthz within deadline\nlogs:\n{logs}"
    );

    let expected_addr = format!("127.0.0.1:{port}");
    let record_matches = logs
        .lines()
        .any(|l| l.contains("lumen serve listening") && l.contains(&expected_addr));
    assert!(
        record_matches,
        "single log line must contain 'lumen serve listening' and '{expected_addr}', logs:\n{logs}"
    );
}
