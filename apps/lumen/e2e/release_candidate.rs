//! Static and local-fixture oracle for the run-scoped release candidate.
use serde_json::{json, Value};
use serde_yaml::Value as Yaml;
use std::{
    fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

#[derive(Debug, PartialEq, Eq)]
struct Finding(&'static str);

const ACTIONS: &[&str] = &[
    "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1",
    "dtolnay/rust-toolchain@4360b52568e2003a75bf9bc1d59f33a8e3fc893c",
    "Swatinem/rust-cache@6323deb102c322ba6fcbdcafc7e3dddab59af2b6",
    "actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02",
    "actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093",
    "astral-sh/setup-uv@c771a70e6277c0a99b617c7a806ffedaca235ff9",
    "ruby/setup-ruby@95ef2b042f9d7a56d8268cba8559e2842e2ad01b",
    "docker/setup-qemu-action@c7c53464625b32c7a7e944ae62b3e17d2b600130",
    "docker/setup-buildx-action@8d2750c68a42422c14e847fe6c8ac0403b4cbd6f",
    "docker/login-action@c94ce9fb468520275223c153574b00df6fe4bcc9",
    "docker/build-push-action@53b7df96c91f9c12dcc8a07bcb9ccacbed38856a",
    "sigstore/cosign-installer@6f9f17788090df1f26f669e9d70d6ae9567deba6",
    "actions/attest@1e69f48acb82d1966a394da916b4c1698aa569d6",
    "anchore/sbom-action@e22c389904149dbc22b58101806040fa8d37a610",
];
const WORKFLOW_BYTES_SHA256: &str =
    "15773dfc479290ea61048dbee01e36ce8ad02adea603c91e9f0c010bda29a6d7";
const RELEASE_PERF_GATE: &str =
    "cargo test --release --locked -p lumen --test perf_gate -- --ignored --test-threads=1 --nocapture";
const VERIFIER_BYTES_SHA256: &str =
    "4fa31b498bab56f7d46e1f7b630893cf509607c8444b5b498e38438fd54529f7";

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut child = Command::new("shasum")
        .args(["-a", "256"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(bytes).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "shasum failed");
    String::from_utf8(output.stdout)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned()
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .into()
}
fn key(name: &str) -> Yaml {
    Yaml::String(name.into())
}
fn field<'a>(value: &'a Yaml, name: &str) -> Option<&'a Yaml> {
    value.as_mapping()?.get(&key(name))
}
fn job<'a>(workflow: &'a Yaml, name: &str) -> Option<&'a Yaml> {
    field(field(workflow, "jobs")?, name)
}
fn strings(value: Option<&Yaml>) -> Vec<&str> {
    match value {
        Some(Yaml::String(value)) => vec![value],
        Some(Yaml::Sequence(values)) => values.iter().filter_map(Yaml::as_str).collect(),
        _ => Vec::new(),
    }
}
fn require(ok: bool, code: &'static str) -> Result<(), Finding> {
    ok.then_some(()).ok_or(Finding(code))
}
fn replace_once(source: &str, from: &str, to: &str) -> String {
    assert_eq!(
        source.matches(from).count(),
        1,
        "mutation target {from:?} is not unique"
    );
    let changed = source.replacen(from, to, 1);
    assert_ne!(changed, source, "mutation did not change bytes");
    changed
}

fn replace_occurrence(source: &str, from: &str, to: &str, occurrence: usize) -> String {
    let offsets = source
        .match_indices(from)
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    assert!(
        occurrence < offsets.len(),
        "mutation target {from:?} occurrence {occurrence} is missing"
    );
    let mut changed = source.to_owned();
    let start = offsets[occurrence];
    changed.replace_range(start..start + from.len(), to);
    assert_ne!(changed, source, "mutation did not change bytes");
    changed
}

fn validate_uv_setup(workflow: &Yaml) -> Result<(), Finding> {
    const UV_SETUP: &str = "astral-sh/setup-uv@c771a70e6277c0a99b617c7a806ffedaca235ff9";
    const GATE_NAME: &str = "Run required Lumen product gates without GKE";
    let steps = field(
        job(workflow, "verify-candidate").ok_or(Finding("UV_SETUP"))?,
        "steps",
    )
    .and_then(Yaml::as_sequence)
    .ok_or(Finding("UV_SETUP"))?;
    let setup_indices: Vec<usize> = steps
        .iter()
        .enumerate()
        .filter_map(|(index, step)| {
            field(step, "uses")
                .and_then(Yaml::as_str)
                .and_then(|uses| uses.strip_prefix("astral-sh/setup-uv@"))
                .map(|_| index)
        })
        .collect();
    require(setup_indices.len() == 1, "UV_SETUP")?;
    let gate_indices: Vec<usize> = steps
        .iter()
        .enumerate()
        .filter_map(|(index, step)| {
            (field(step, "name").and_then(Yaml::as_str) == Some(GATE_NAME)).then_some(index)
        })
        .collect();
    require(gate_indices.len() == 1, "UV_SETUP")?;
    require(setup_indices[0] < gate_indices[0], "UV_SETUP")?;

    let setup = steps[setup_indices[0]]
        .as_mapping()
        .ok_or(Finding("UV_SETUP"))?;
    require(setup.len() == 2, "UV_SETUP")?;
    require(
        setup.get(&key("uses")).and_then(Yaml::as_str) == Some(UV_SETUP),
        "UV_SETUP",
    )?;
    let with = setup
        .get(&key("with"))
        .and_then(Yaml::as_mapping)
        .ok_or(Finding("UV_SETUP"))?;
    require(with.len() == 2, "UV_SETUP")?;
    require(
        with.get(&key("version")).and_then(Yaml::as_str) == Some("0.12.1"),
        "UV_SETUP",
    )?;
    require(
        with.get(&key("enable-cache")).and_then(Yaml::as_bool) == Some(false),
        "UV_SETUP",
    )?;
    Ok(())
}

fn validate_cloud_free_acceptance_gates(workflow: &Yaml) -> Result<(), Finding> {
    const RUBY: &str = "ruby/setup-ruby@95ef2b042f9d7a56d8268cba8559e2842e2ad01b";
    let steps = field(
        job(workflow, "verify-candidate").ok_or(Finding("CLOUD_FREE_GATES"))?,
        "steps",
    )
    .and_then(Yaml::as_sequence)
    .ok_or(Finding("CLOUD_FREE_GATES"))?;
    let index_of = |name: &str| {
        steps
            .iter()
            .enumerate()
            .filter(|(_, step)| field(step, "name").and_then(Yaml::as_str) == Some(name))
            .map(|(index, _)| index)
            .collect::<Vec<_>>()
    };
    let uv = steps
        .iter()
        .enumerate()
        .filter(|(_, step)| {
            field(step, "uses").and_then(Yaml::as_str)
                == Some("astral-sh/setup-uv@c771a70e6277c0a99b617c7a806ffedaca235ff9")
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    require(uv.len() == 1, "CLOUD_FREE_GATES")?;
    let ruby = steps
        .iter()
        .enumerate()
        .filter(|(_, step)| field(step, "uses").and_then(Yaml::as_str) == Some(RUBY))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    require(ruby.len() == 1, "CLOUD_FREE_GATES")?;
    let ruby_step = &steps[ruby[0]];
    require(
        ruby_step.as_mapping().map(|map| map.len()) == Some(2)
            && field(ruby_step, "with")
                .and_then(Yaml::as_mapping)
                .map(|map| map.len())
                == Some(2)
            && field(ruby_step, "with")
                .and_then(|with| field(with, "ruby-version"))
                .and_then(Yaml::as_str)
                == Some("3.3.6")
            && field(ruby_step, "with")
                .and_then(|with| field(with, "bundler"))
                .and_then(Yaml::as_str)
                == Some("none"),
        "CLOUD_FREE_GATES",
    )?;
    validate_exact_run_step(
        workflow,
        "verify-candidate",
        "Install verified Terraform 1.9.4",
        &["name", "shell", "run"],
        &[
            "set -euo pipefail",
            "curl -fsSL https://releases.hashicorp.com/terraform/1.9.4/terraform_1.9.4_linux_amd64.zip -o /tmp/terraform.zip",
            "echo '6e9b2cc741875ab906d800af3134b076489f049565e0a1dbdb6deacd91f5054c  /tmp/terraform.zip' | sha256sum -c -",
            "unzip -oq /tmp/terraform.zip -d /tmp/terraform-bin",
            "sudo install -m 0755 /tmp/terraform-bin/terraform /usr/local/bin/terraform",
        ],
    )?;
    validate_exact_run_step(
        workflow,
        "verify-candidate",
        "Install verified kubectl v1.37.0",
        &["name", "shell", "run"],
        &[
            "set -euo pipefail",
            "curl -fsSL https://dl.k8s.io/release/v1.37.0/bin/linux/amd64/kubectl -o /tmp/kubectl",
            "echo '6129359f4e1f3848a5572ccb0b26cf28b8ca08cef38c95a765b2f64a2c961a2f  /tmp/kubectl' | sha256sum -c -",
            "chmod +x /tmp/kubectl",
            "sudo install -m 0755 /tmp/kubectl /usr/local/bin/kubectl",
        ],
    )?;
    validate_exact_run_step(
        workflow,
        "verify-candidate",
        "Run cloud-free Terraform acceptance gate",
        &["name", "shell", "run"],
        &["bash terraform/lumen-standalone-gke/scripts/check.sh"],
    )?;
    validate_exact_run_step(
        workflow,
        "verify-candidate",
        "Run cloud-free Kustomize acceptance gate",
        &["name", "shell", "run"],
        &["bash kustomize/lumen-standalone-acceptance/tests/contract.sh"],
    )?;
    let names = [
        "Install verified Terraform 1.9.4",
        "Install verified kubectl v1.37.0",
        "Run cloud-free Terraform acceptance gate",
        "Run cloud-free Kustomize acceptance gate",
        "Run required Lumen product gates without GKE",
    ];
    let ordered = names.iter().map(|name| index_of(name)).collect::<Vec<_>>();
    require(
        ordered.iter().all(|indices| indices.len() == 1),
        "CLOUD_FREE_GATES",
    )?;
    let mut previous = uv[0];
    for indices in ordered {
        require(previous < indices[0], "CLOUD_FREE_GATES")?;
        previous = indices[0];
    }
    require(ruby[0] == uv[0] + 1, "CLOUD_FREE_GATES")?;
    Ok(())
}

fn validate_libraries_job(workflow: &Yaml) -> Result<(), Finding> {
    let library_job = job(workflow, "verify-libraries").ok_or(Finding("LIBRARIES"))?;
    let library_map = library_job.as_mapping().ok_or(Finding("LIBRARIES"))?;
    require(
        library_map.len() == 5
            && ["name", "needs", "runs-on", "permissions", "steps"]
                .iter()
                .all(|name| library_map.contains_key(&key(name))),
        "LIBRARIES",
    )?;
    require(
        field(library_job, "name").and_then(Yaml::as_str)
            == Some("verify service and Raft library gates"),
        "LIBRARIES",
    )?;
    require(
        field(library_job, "runs-on").and_then(Yaml::as_str) == Some("ubuntu-latest"),
        "LIBRARIES",
    )?;
    let steps = field(library_job, "steps")
        .and_then(Yaml::as_sequence)
        .ok_or(Finding("LIBRARIES"))?;
    require(steps.len() == 2, "LIBRARIES")?;
    require(
        field(&steps[0], "uses").and_then(Yaml::as_str)
            == Some("actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1"),
        "LIBRARIES",
    )?;
    let checkout_step = steps[0].as_mapping().ok_or(Finding("LIBRARIES"))?;
    require(
        checkout_step.len() == 2
            && checkout_step.contains_key(&key("uses"))
            && checkout_step.contains_key(&key("with")),
        "LIBRARIES",
    )?;
    let checkout_with = field(&steps[0], "with")
        .and_then(Yaml::as_mapping)
        .ok_or(Finding("LIBRARIES"))?;
    require(
        checkout_with.len() == 2
            && checkout_with.contains_key(&key("ref"))
            && checkout_with.contains_key(&key("fetch-depth")),
        "LIBRARIES",
    )?;
    require(
        checkout_with.get(&key("ref")).and_then(Yaml::as_str)
            == Some("${{ needs.identity.outputs.commit }}"),
        "LIBRARIES",
    )?;
    require(
        checkout_with
            .get(&key("fetch-depth"))
            .and_then(Yaml::as_i64)
            == Some(0),
        "LIBRARIES",
    )?;
    let run_step = steps[1].as_mapping().ok_or(Finding("LIBRARIES"))?;
    require(
        run_step.len() == 3
            && ["name", "shell", "run"]
                .iter()
                .all(|name| run_step.contains_key(&key(name))),
        "LIBRARIES",
    )?;
    require(
        field(&steps[1], "shell").and_then(Yaml::as_str) == Some("bash"),
        "LIBRARIES",
    )?;
    require(
        field(&steps[1], "name").and_then(Yaml::as_str)
            == Some("Run required service and Raft library gates without GKE"),
        "LIBRARIES",
    )?;
    let run = field(&steps[1], "run")
        .and_then(Yaml::as_str)
        .ok_or(Finding("LIBRARIES"))?;
    require(
        exact_shell_lines(
            run,
            &[
                "set -euo pipefail",
                "cargo test -p service-k8s",
                "cargo test -p storage-durable",
                "cargo test -p service-backup --features http-client",
                "bash scripts/raft-implementor-build.sh",
                "cargo test -p raft-runtime",
                "python3 scripts/meta/test_readme_contract.py",
                "python3 scripts/meta/test_project_docs_contract.py",
                "python3 scripts/meta/project_docs_contract.py check apps/lumen libs/service-k8s --format json",
                "git -c core.fsmonitor=false diff --check",
            ],
        ),
        "LIBRARIES",
    )?;
    Ok(())
}

fn exact_shell_lines(content: &str, expected: &[&str]) -> bool {
    shell_logical_lines(content) == expected
}

fn strip_shell_comment(raw: &str) -> String {
    let mut output = String::with_capacity(raw.len());
    let mut quote = None;
    let mut escaped = false;
    let mut token_start = true;
    for ch in raw.chars() {
        if escaped {
            output.push(ch);
            escaped = false;
            token_start = ch.is_whitespace();
            continue;
        }
        match quote {
            Some('\'') => {
                output.push(ch);
                if ch == '\'' {
                    quote = None;
                }
                token_start = false;
            }
            Some('"') => {
                output.push(ch);
                if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    quote = None;
                }
                token_start = false;
            }
            None if ch == '\\' => {
                output.push(ch);
                escaped = true;
                token_start = false;
            }
            None if ch == '\'' || ch == '"' => {
                output.push(ch);
                quote = Some(ch);
                token_start = false;
            }
            None if ch == '#' && token_start => break,
            None => {
                output.push(ch);
                token_start = ch.is_whitespace();
            }
            Some(other) => unreachable!("unsupported shell quote delimiter: {other}"),
        }
    }
    output
}

fn has_unescaped_continuation(line: &str) -> bool {
    let slash_count = line.chars().rev().take_while(|ch| *ch == '\\').count();
    slash_count % 2 == 1
}

fn shell_logical_lines(content: &str) -> Vec<String> {
    let mut logical = Vec::new();
    let mut pending = String::new();
    for raw in content.lines() {
        let active = strip_shell_comment(raw);
        let line = active.trim();
        if line.is_empty() {
            continue;
        }
        let continued = has_unescaped_continuation(line);
        let part = if continued {
            line.strip_suffix('\\').expect("continuation suffix")
        } else {
            line
        };
        if !pending.is_empty() {
            pending.push(' ');
        }
        pending.push_str(part.trim_end());
        if !continued {
            logical.push(std::mem::take(&mut pending));
        }
    }
    if !pending.is_empty() {
        logical.push(pending);
    }
    logical
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ShellToken {
    Word(String, bool),
    Separator,
}

fn shell_tokens(line: &str) -> Vec<ShellToken> {
    let mut tokens = Vec::new();
    let mut word = String::new();
    let mut quoted = false;
    let mut quote = None;
    let mut escaped = false;
    let mut token_start = true;
    let mut chars = line.chars().peekable();
    let flush_word = |tokens: &mut Vec<ShellToken>, word: &mut String, quoted: &mut bool| {
        if !word.is_empty() {
            tokens.push(ShellToken::Word(std::mem::take(word), *quoted));
            *quoted = false;
        }
    };
    while let Some(ch) = chars.next() {
        if escaped {
            word.push(ch);
            escaped = false;
            token_start = false;
            continue;
        }
        match quote {
            Some('\'') => {
                if ch == '\'' {
                    quote = None;
                } else {
                    word.push(ch);
                }
                token_start = false;
            }
            Some('"') => {
                if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    quote = None;
                } else {
                    word.push(ch);
                }
                token_start = false;
            }
            None if ch == '\\' => {
                escaped = true;
                token_start = false;
            }
            None if ch == '\'' || ch == '"' => {
                quote = Some(ch);
                quoted = true;
                token_start = false;
            }
            None if ch == '#' && token_start => break,
            None if ch.is_whitespace() => {
                flush_word(&mut tokens, &mut word, &mut quoted);
                token_start = true;
            }
            None if ch == ';' || ch == '|' || ch == '&' => {
                flush_word(&mut tokens, &mut word, &mut quoted);
                if (ch == '|' || ch == '&') && chars.peek() == Some(&ch) {
                    chars.next();
                }
                tokens.push(ShellToken::Separator);
                token_start = true;
            }
            None => {
                word.push(ch);
                token_start = false;
            }
            Some(other) => unreachable!("unsupported shell quote delimiter: {other}"),
        }
    }
    flush_word(&mut tokens, &mut word, &mut quoted);
    tokens
}

fn is_gcloud_command(word: &str) -> bool {
    word == "gcloud" || word.ends_with("/gcloud")
}

fn is_shell_c_invocation(words: &[String], index: usize) -> bool {
    let word = words[index].as_str();
    let shell = word == "bash" || word == "sh" || word.ends_with("/bash") || word.ends_with("/sh");
    shell && words[index + 1..].iter().any(|word| word == "-c")
}

fn active_shell_c_invocation(tokens: &[ShellToken]) -> bool {
    tokens.iter().enumerate().any(|(index, token)| {
        let ShellToken::Word(shell, false) = token else {
            return false;
        };
        (shell == "bash" || shell == "sh" || shell.ends_with("/bash") || shell.ends_with("/sh"))
            && tokens[index + 1..]
                .iter()
                .any(|token| matches!(token, ShellToken::Word(option, false) if option == "-c"))
    })
}

fn command_segment_executes_gcloud(words: &[String]) -> bool {
    let mut index = 0;
    while index < words.len() {
        let word = words[index].as_str();
        if [
            "if", "then", "do", "done", "else", "elif", "fi", "for", "while", "case", "esac", "{",
            "}", "!",
        ]
        .contains(&word)
            || word.contains('=')
        {
            index += 1;
            continue;
        }
        if word == "sudo" {
            index += 1;
            while index < words.len() {
                let option = words[index].as_str();
                if option == "--" {
                    index += 1;
                    break;
                }
                if !option.starts_with('-') {
                    break;
                }
                index += if ["-u", "-g", "-h", "-p", "-r", "-t", "-C"].contains(&option) {
                    2
                } else {
                    1
                };
            }
            continue;
        }
        if word == "env" {
            index += 1;
            while index < words.len() {
                let option = words[index].as_str();
                if option == "--" {
                    index += 1;
                    break;
                }
                if option.contains('=') {
                    index += 1;
                    continue;
                }
                if option.starts_with('-') {
                    index += if ["-u", "-C"].contains(&option) { 2 } else { 1 };
                    continue;
                }
                break;
            }
            continue;
        }
        if word == "command" {
            index += 1;
            let mut lookup_only = false;
            while index < words.len() && words[index].starts_with('-') {
                lookup_only |= words[index] == "-v" || words[index] == "-V";
                index += 1;
            }
            if lookup_only {
                return false;
            }
            continue;
        }
        if word == "time" {
            index += 1;
            while index < words.len() && words[index].starts_with('-') {
                index += 1;
            }
            continue;
        }
        if (word == "bash" || word == "sh" || word.ends_with("/bash") || word.ends_with("/sh"))
            && is_shell_c_invocation(words, index)
        {
            return true;
        }
        return is_gcloud_command(word);
    }
    false
}

fn logical_line_executes_gcloud(line: &str) -> bool {
    let mut segment = Vec::new();
    for token in shell_tokens(line) {
        match token {
            ShellToken::Word(word, quoted) => segment.push(ShellToken::Word(word, quoted)),
            ShellToken::Separator => {
                if active_shell_c_invocation(&segment)
                    || command_segment_executes_gcloud(
                        &segment
                            .iter()
                            .filter_map(|token| match token {
                                ShellToken::Word(word, _) => Some(word.clone()),
                                ShellToken::Separator => None,
                            })
                            .collect::<Vec<_>>(),
                    )
                {
                    return true;
                }
                segment.clear();
            }
        }
    }
    active_shell_c_invocation(&segment)
        || command_segment_executes_gcloud(
            &segment
                .iter()
                .filter_map(|token| match token {
                    ShellToken::Word(word, _) => Some(word.clone()),
                    ShellToken::Separator => None,
                })
                .collect::<Vec<_>>(),
        )
}

fn validate_no_gcloud_execution(workflow: &Yaml) -> Result<(), Finding> {
    let jobs = field(workflow, "jobs")
        .and_then(Yaml::as_mapping)
        .ok_or(Finding("CANDIDATE_ONLY"))?;
    for job in jobs.values() {
        let Some(steps) = field(job, "steps").and_then(Yaml::as_sequence) else {
            continue;
        };
        for step in steps {
            let Some(run) = field(step, "run").and_then(Yaml::as_str) else {
                continue;
            };
            if shell_logical_lines(run)
                .iter()
                .any(|line| logical_line_executes_gcloud(line))
            {
                return Err(Finding("CANDIDATE_ONLY"));
            }
        }
    }
    Ok(())
}

fn named_step<'a>(workflow: &'a Yaml, job_name: &str, step_name: &str) -> Option<&'a Yaml> {
    let steps = field(job(workflow, job_name)?, "steps")?.as_sequence()?;
    let matches = steps
        .iter()
        .filter(|step| field(step, "name").and_then(Yaml::as_str) == Some(step_name))
        .collect::<Vec<_>>();
    (matches.len() == 1).then(|| matches[0])
}

fn indexed_step_by_id<'a>(
    workflow: &'a Yaml,
    job_name: &str,
    step_id: &str,
) -> Result<(usize, &'a Yaml), Finding> {
    let steps = field(job(workflow, job_name).ok_or(Finding("IMAGE"))?, "steps")
        .and_then(Yaml::as_sequence)
        .ok_or(Finding("IMAGE"))?;
    let matches = steps
        .iter()
        .enumerate()
        .filter(|(_, step)| field(step, "id").and_then(Yaml::as_str) == Some(step_id))
        .collect::<Vec<_>>();
    require(matches.len() == 1, "IMAGE")?;
    Ok(matches[0])
}

fn validate_candidate_image_outputs(workflow: &Yaml) -> Result<(), Finding> {
    const BUILD_PUSH: &str = "docker/build-push-action@53b7df96c91f9c12dcc8a07bcb9ccacbed38856a";
    let image_job = job(workflow, "ghcr-image-and-attest").ok_or(Finding("IMAGE"))?;
    let outputs = field(image_job, "outputs")
        .and_then(Yaml::as_mapping)
        .ok_or(Finding("IMAGE"))?;
    let expected_outputs = [
        ("image_repo", "${{ steps.tags.outputs.image_repo }}"),
        ("candidate_tag", "${{ steps.tags.outputs.candidate_tag }}"),
        ("root_digest", "${{ steps.push.outputs.digest }}"),
        (
            "amd64_digest",
            "${{ steps.platform_digests.outputs.amd64_digest }}",
        ),
        (
            "arm64_digest",
            "${{ steps.platform_digests.outputs.arm64_digest }}",
        ),
    ];
    require(outputs.len() == expected_outputs.len(), "IMAGE")?;
    for (name, expected) in expected_outputs {
        require(
            outputs.get(&key(name)).and_then(Yaml::as_str) == Some(expected),
            "IMAGE",
        )?;
    }

    let (tags_index, tags) = indexed_step_by_id(workflow, "ghcr-image-and-attest", "tags")?;
    let (push_index, push) = indexed_step_by_id(workflow, "ghcr-image-and-attest", "push")?;
    let (platform_index, platform) =
        indexed_step_by_id(workflow, "ghcr-image-and-attest", "platform_digests")?;
    require(
        tags_index < push_index && push_index < platform_index,
        "IMAGE",
    )?;
    require(
        field(tags, "name").and_then(Yaml::as_str)
            == Some("Resolve run-scoped candidate image identity"),
        "IMAGE",
    )?;
    require(
        field(push, "name").and_then(Yaml::as_str)
            == Some("Build and push only the candidate index"),
        "IMAGE",
    )?;
    require(
        field(push, "uses").and_then(Yaml::as_str) == Some(BUILD_PUSH),
        "IMAGE",
    )?;
    require(
        field(platform, "name").and_then(Yaml::as_str)
            == Some("Extract exact two platform child digests"),
        "IMAGE",
    )
}

fn validate_exact_run_step(
    workflow: &Yaml,
    job_name: &str,
    step_name: &str,
    expected_keys: &[&str],
    expected_lines: &[&str],
) -> Result<(), Finding> {
    let step = named_step(workflow, job_name, step_name).ok_or(Finding("GATE_COMMANDS"))?;
    let map = step.as_mapping().ok_or(Finding("GATE_COMMANDS"))?;
    require(
        map.len() == expected_keys.len()
            && expected_keys
                .iter()
                .all(|name| map.contains_key(&key(name))),
        "GATE_COMMANDS",
    )?;
    require(
        field(step, "shell").and_then(Yaml::as_str) == Some("bash"),
        "GATE_COMMANDS",
    )?;
    let run = field(step, "run")
        .and_then(Yaml::as_str)
        .ok_or(Finding("GATE_COMMANDS"))?;
    require(exact_shell_lines(run, expected_lines), "GATE_COMMANDS")
}

fn validate_candidate_and_kind_commands(workflow: &Yaml) -> Result<(), Finding> {
    validate_exact_run_step(
        workflow,
        "verify-candidate",
        "Verify full run-scoped candidate supply chain",
        &["name", "env", "shell", "run"],
        &[
            "set -euo pipefail",
            "apps/lumen/scripts/verify-release-candidate.sh --repo chrischeng-c4/axiom --version \"${{ needs.identity.outputs.version }}\" --commit \"${{ needs.identity.outputs.commit }}\" --run-id \"${{ github.run_id }}\" --run-attempt \"${{ github.run_attempt }}\" --manifest candidate/candidate-manifest.json --manifest-sidecar candidate/candidate-manifest.json.sha256 --artifacts-dir candidate --image \"${{ needs.ghcr-image-and-attest.outputs.image_repo }}@${{ needs.ghcr-image-and-attest.outputs.root_digest }}\" --candidate-tag \"${{ needs.ghcr-image-and-attest.outputs.candidate_tag }}\" --amd64-digest \"${{ needs.ghcr-image-and-attest.outputs.amd64_digest }}\" --arm64-digest \"${{ needs.ghcr-image-and-attest.outputs.arm64_digest }}\" --mode full",
        ],
    )?;
    let supply_chain = named_step(
        workflow,
        "verify-candidate",
        "Verify full run-scoped candidate supply chain",
    )
    .and_then(|step| field(step, "env"))
    .and_then(Yaml::as_mapping)
    .ok_or(Finding("GATE_COMMANDS"))?;
    require(
        supply_chain.len() == 1
            && supply_chain.get(&key("GH_TOKEN")).and_then(Yaml::as_str)
                == Some("${{ github.token }}"),
        "GATE_COMMANDS",
    )?;

    for (job_name, digest) in [
        (
            "kind-amd64",
            "${{ needs.ghcr-image-and-attest.outputs.amd64_digest }}",
        ),
        (
            "kind-arm64",
            "${{ needs.ghcr-image-and-attest.outputs.arm64_digest }}",
        ),
    ] {
        let command = format!(
            "LUMEN_E2E_MODE=operator LUMEN_E2E_IMAGE_MODE=prebuilt LUMEN_E2E_IMAGE=\"${{{{ needs.ghcr-image-and-attest.outputs.image_repo }}}}@${{{{ needs.ghcr-image-and-attest.outputs.root_digest }}}}\" LUMEN_E2E_EXPECTED_VERSION=\"${{{{ needs.identity.outputs.version }}}}\" LUMEN_E2E_EXPECTED_GIT_SHA=\"${{short_sha:0:8}}\" LUMEN_E2E_EXPECTED_RUNTIME_DIGEST=\"{digest}\" apps/lumen/scripts/kind-e2e.sh"
        );
        validate_exact_run_step(
            workflow,
            job_name,
            "Run prebuilt candidate kind e2e",
            &["name", "shell", "run"],
            &[
                "set -euo pipefail",
                "short_sha=\"${{ needs.identity.outputs.commit }}\"",
                &command,
            ],
        )?;
    }
    Ok(())
}

fn validate_gate_step_inventory(workflow: &Yaml) -> Result<(), Finding> {
    let expected: &[(&str, &[&str])] = &[
        (
            "verify-candidate",
            &[
                "uses:actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1",
                "Reclaim runner disk before the cargo gates",
                "uses:sigstore/cosign-installer@6f9f17788090df1f26f669e9d70d6ae9567deba6",
                "uses:docker/setup-buildx-action@8d2750c68a42422c14e847fe6c8ac0403b4cbd6f",
                "Log in to GHCR with read-only job access",
                "uses:actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093",
                "uses:actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093",
                "uses:actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093",
                "uses:actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093",
                "uses:actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093",
                "uses:actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093",
                "uses:actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093",
                "uses:astral-sh/setup-uv@c771a70e6277c0a99b617c7a806ffedaca235ff9",
                "uses:ruby/setup-ruby@95ef2b042f9d7a56d8268cba8559e2842e2ad01b",
                "Install verified Terraform 1.9.4",
                "Install verified kubectl v1.37.0",
                "Run cloud-free Terraform acceptance gate",
                "Run cloud-free Kustomize acceptance gate",
                "Run required Lumen product gates without GKE",
                "Verify full run-scoped candidate supply chain",
            ][..],
        ),
        (
            "kind-amd64",
            &[
                "uses:actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1",
                "Assert native x86_64 runner architecture",
                "Install verified kind v0.32.0",
                "Run prebuilt candidate kind e2e",
            ][..],
        ),
        (
            "kind-arm64",
            &[
                "uses:actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1",
                "Assert native aarch64 runner architecture",
                "Install verified kind v0.32.0",
                "Run prebuilt candidate kind e2e",
            ][..],
        ),
        (
            "result",
            &[
                "uses:actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1",
                "uses:actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093",
                "uses:actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093",
                "uses:actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093",
                "uses:actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093",
                "uses:actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093",
                "uses:actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093",
                "uses:actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093",
                "Verify exact preflight manifest sidecar",
                "Bind all successful job conclusions into final receipt",
                "Verify final receipt as local fixture only",
                "uses:actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02",
            ][..],
        ),
    ];
    for &(job_name, expected_names) in expected {
        let steps = field(
            job(workflow, job_name).ok_or(Finding("GATE_STEPS"))?,
            "steps",
        )
        .and_then(Yaml::as_sequence)
        .ok_or(Finding("GATE_STEPS"))?;
        let actual = steps
            .iter()
            .map(|step| {
                field(step, "name")
                    .and_then(Yaml::as_str)
                    .map(str::to_owned)
                    .or_else(|| {
                        field(step, "uses")
                            .and_then(Yaml::as_str)
                            .map(|uses| format!("uses:{uses}"))
                    })
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>();
        require(
            actual
                .iter()
                .map(String::as_str)
                .eq(expected_names.iter().copied()),
            "GATE_STEPS",
        )?;
    }
    Ok(())
}

fn validate_fail_closed_gate_conditions(workflow: &Yaml) -> Result<(), Finding> {
    for name in [
        "verify-candidate",
        "verify-libraries",
        "kind-amd64",
        "kind-arm64",
        "result",
    ] {
        let job = job(workflow, name).ok_or(Finding("CONDITIONS"))?;
        let job_map = job.as_mapping().ok_or(Finding("CONDITIONS"))?;
        for forbidden in ["if", "continue-on-error"] {
            require(!job_map.contains_key(&key(forbidden)), "CONDITIONS")?;
        }
        let steps = field(job, "steps")
            .and_then(Yaml::as_sequence)
            .ok_or(Finding("CONDITIONS"))?;
        for step in steps {
            let step = step.as_mapping().ok_or(Finding("CONDITIONS"))?;
            for forbidden in ["if", "continue-on-error"] {
                require(!step.contains_key(&key(forbidden)), "CONDITIONS")?;
            }
        }
    }
    Ok(())
}

fn validate_product_gate_partition(workflow: &Yaml, source: &str) -> Result<(), Finding> {
    let steps = field(
        job(workflow, "verify-candidate").ok_or(Finding("GATES"))?,
        "steps",
    )
    .and_then(Yaml::as_sequence)
    .ok_or(Finding("GATES"))?;
    let run = steps
        .iter()
        .find(|step| {
            field(step, "name").and_then(Yaml::as_str)
                == Some("Run required Lumen product gates without GKE")
        })
        .and_then(|step| field(step, "run"))
        .and_then(Yaml::as_str)
        .ok_or(Finding("GATES"))?;
    require(
        exact_shell_lines(
            run,
            &[
                "set -euo pipefail",
                "cargo test -p lumen --features operator --test capacity_catalog_client",
                "cargo test -p lumen --features operator --test capacity_catalog_contract",
                "cargo test -p lumen --test cli_convention",
                "cargo test -p lumen --test release_artifacts",
                "cargo test -p lumen --test release_candidate",
                "cargo test -p lumen",
                "cargo test -p lumen --features \"operator delegated-auth\"",
                RELEASE_PERF_GATE,
                "cargo test -p lumen --locked --features release --test release_feature_set",
                "cargo clean",
                "bash apps/lumen/scripts/standalone-container-smoke.sh bind",
                "LUMEN_STANDALONE_DURABLE_IMAGE=\"${{ needs.ghcr-image-and-attest.outputs.image_repo }}@${{ needs.ghcr-image-and-attest.outputs.root_digest }}\" bash apps/lumen/scripts/standalone-container-smoke.sh durable",
            ],
        ),
        "GATES",
    )?;
    for gate in [
        "cargo test -p service-k8s",
        "cargo test -p storage-durable",
        "cargo test -p service-backup --features http-client",
        "bash scripts/raft-implementor-build.sh",
        "cargo test -p raft-runtime",
        "python3 scripts/meta/test_readme_contract.py",
        "python3 scripts/meta/test_project_docs_contract.py",
        "python3 scripts/meta/project_docs_contract.py check apps/lumen libs/service-k8s --format json",
        "git -c core.fsmonitor=false diff --check",
    ] {
        require(!run.contains(gate), "LIBRARIES")?;
        require(source.matches(gate).count() == 1, "LIBRARIES")?;
    }
    Ok(())
}

fn perf_gate_inventory(source: &str) -> Vec<(String, bool)> {
    let mut attributes = Vec::new();
    let mut inventory = Vec::new();
    let mut in_block_comment = false;
    for raw in source.lines() {
        if in_block_comment {
            if raw.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }
        let line = raw.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if let Some(start) = line.find("/*") {
            if !line[start + 2..].contains("*/") {
                in_block_comment = true;
            }
            continue;
        }
        if line.starts_with("#[") {
            attributes.push(line.to_owned());
        } else if let Some(rest) = line.strip_prefix("fn ") {
            if attributes.iter().any(|attr| attr == "#[test]") {
                inventory.push((
                    rest.split('(').next().unwrap_or_default().to_owned(),
                    attributes.iter().any(|attr| attr.starts_with("#[ignore")),
                ));
            }
            attributes.clear();
        } else {
            attributes.clear();
        }
    }
    inventory
}

fn validate_perf_gate_source(source: &str) -> Result<(), Finding> {
    require(
        perf_gate_inventory(source)
            == vec![
                ("index_throughput_floor".into(), true),
                ("match_query_latency_floor".into(), true),
                ("term_query_latency_floor".into(), true),
                ("median_statistic_and_ignored_inventory".into(), false),
            ],
        "PERF_GATE",
    )
}

fn validate_workflow_semantics(source: &str, dockerfile: &str) -> Result<(), Finding> {
    let workflow: Yaml = serde_yaml::from_str(source).map_err(|_| Finding("YAML"))?;
    let events = field(&workflow, "on")
        .and_then(Yaml::as_mapping)
        .ok_or(Finding("TRIGGER"))?;
    require(events.len() == 1, "TRIGGER")?;
    let dispatch = events
        .get(&key("workflow_dispatch"))
        .ok_or(Finding("TRIGGER"))?;
    let inputs = field(dispatch, "inputs").ok_or(Finding("INPUTS"))?;
    for name in ["version", "commit"] {
        let input = field(inputs, name).ok_or(Finding("INPUTS"))?;
        require(
            field(input, "required").and_then(Yaml::as_bool) == Some(true),
            "INPUTS",
        )?;
        require(
            field(input, "type").and_then(Yaml::as_str) == Some("string"),
            "INPUTS",
        )?;
    }
    let concurrency = field(&workflow, "concurrency").ok_or(Finding("CONCURRENCY"))?;
    require(
        field(concurrency, "group").and_then(Yaml::as_str)
            == Some("lumen-release-candidate-${{ inputs.version }}"),
        "CONCURRENCY",
    )?;
    require(
        field(concurrency, "cancel-in-progress").and_then(Yaml::as_bool) == Some(false),
        "CONCURRENCY",
    )?;
    let names = [
        "identity",
        "build",
        "ghcr-image-and-attest",
        "manifest",
        "verify-candidate",
        "verify-libraries",
        "kind-amd64",
        "kind-arm64",
        "result",
    ];
    let jobs = field(&workflow, "jobs")
        .and_then(Yaml::as_mapping)
        .ok_or(Finding("JOBS"))?;
    require(
        jobs.len() == names.len() && names.iter().all(|name| job(&workflow, name).is_some()),
        "JOBS",
    )?;
    validate_candidate_image_outputs(&workflow)?;
    validate_no_gcloud_execution(&workflow)?;
    validate_uv_setup(&workflow)?;
    validate_cloud_free_acceptance_gates(&workflow)?;
    validate_libraries_job(&workflow)?;
    validate_product_gate_partition(&workflow, source)?;
    validate_perf_gate_source(&perf_gate_source())?;
    validate_fail_closed_gate_conditions(&workflow)?;
    validate_gate_step_inventory(&workflow)?;
    validate_candidate_and_kind_commands(&workflow)?;
    let graph = [
        ("identity", &[][..]),
        ("build", &["identity"][..]),
        ("ghcr-image-and-attest", &["identity", "build"][..]),
        (
            "manifest",
            &["identity", "build", "ghcr-image-and-attest"][..],
        ),
        (
            "verify-candidate",
            &["identity", "manifest", "ghcr-image-and-attest"][..],
        ),
        ("verify-libraries", &["identity"][..]),
        (
            "kind-amd64",
            &[
                "identity",
                "verify-candidate",
                "verify-libraries",
                "ghcr-image-and-attest",
            ][..],
        ),
        (
            "kind-arm64",
            &[
                "identity",
                "verify-candidate",
                "verify-libraries",
                "ghcr-image-and-attest",
            ][..],
        ),
        (
            "result",
            &[
                "identity",
                "build",
                "manifest",
                "ghcr-image-and-attest",
                "verify-candidate",
                "verify-libraries",
                "kind-amd64",
                "kind-arm64",
            ][..],
        ),
    ];
    for (name, expected) in graph {
        require(
            strings(field(job(&workflow, name).unwrap(), "needs")) == expected,
            "GRAPH",
        )?;
    }
    let permissions = [
        (
            "identity",
            "actions: read\ncontents: read\npull-requests: read",
        ),
        ("build", "contents: read"),
        (
            "ghcr-image-and-attest",
            "attestations: write\ncontents: read\nid-token: write\npackages: write",
        ),
        ("manifest", "contents: read"),
        (
            "verify-candidate",
            "attestations: read\ncontents: read\npackages: read",
        ),
        ("verify-libraries", "contents: read"),
        ("kind-amd64", "contents: read\npackages: read"),
        ("kind-arm64", "contents: read\npackages: read"),
        ("result", "contents: read"),
    ];
    for (name, expected) in permissions {
        let actual = field(job(&workflow, name).unwrap(), "permissions")
            .and_then(Yaml::as_mapping)
            .map(|map| {
                map.iter()
                    .filter_map(|(k, v)| Some(format!("{}: {}", k.as_str()?, v.as_str()?)))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let mut actual = actual;
        actual.sort();
        let mut expected = expected.split('\n').map(str::to_owned).collect::<Vec<_>>();
        expected.sort();
        require(actual == expected, "PERMISSIONS")?;
    }
    for line in source.lines().filter_map(|line| {
        let line = line.trim();
        line.strip_prefix("- uses: ")
            .or_else(|| line.strip_prefix("uses: "))
    }) {
        let action = line.split_whitespace().next().unwrap_or("");
        require(ACTIONS.contains(&action), "ACTION_PIN")?;
        let (_, sha) = action.rsplit_once('@').ok_or(Finding("ACTION_PIN"))?;
        require(
            sha.len() == 40 && sha.chars().all(|c| c.is_ascii_hexdigit()),
            "ACTION_PIN",
        )?;
    }
    for needle in [
        "candidate must dispatch main", "REQUESTED_COMMIT", "REQUESTED_VERSION",
        "git merge-base --is-ancestor", "expected one merged main PR",
        "git ls-remote --exit-code --tags origin", "the exact release tag already exists",
        "release_query=\"query", "the exact GitHub Release already exists",
        "cargo test -p lumen --features operator --test capacity_catalog_client",
        "cargo test -p lumen --features operator --test capacity_catalog_contract",
        "cargo test -p lumen --test cli_convention", "cargo test -p lumen --test release_artifacts",
        "cargo test -p lumen --test release_candidate", "cargo test -p lumen",
        "cargo test -p lumen --features \"operator delegated-auth\"",
        "cargo test -p lumen --locked --features release --test release_feature_set",
        "cargo clean",
        "bash apps/lumen/scripts/standalone-container-smoke.sh bind",
        "LUMEN_STANDALONE_DURABLE_IMAGE=\"${{ needs.ghcr-image-and-attest.outputs.image_repo }}@${{ needs.ghcr-image-and-attest.outputs.root_digest }}\" bash apps/lumen/scripts/standalone-container-smoke.sh durable",
        RELEASE_PERF_GATE,
        "cargo test -p service-k8s", "cargo test -p raft-runtime",
        "bash scripts/raft-implementor-build.sh", "git -c core.fsmonitor=false diff --check",
        "python3 scripts/meta/test_readme_contract.py", "project_docs_contract.py check apps/lumen libs/service-k8s",
        "--image \"${{ needs.ghcr-image-and-attest.outputs.image_repo }}@${{ needs.ghcr-image-and-attest.outputs.root_digest }}\"",
        "LUMEN_E2E_EXPECTED_RUNTIME_DIGEST", "final-candidate-manifest.json",
        "Verify final receipt as local fixture only", "--mode local",
        "ruby/setup-ruby@95ef2b042f9d7a56d8268cba8559e2842e2ad01b",
        "ruby-version: '3.3.6'", "terraform/1.9.4/terraform_1.9.4_linux_amd64.zip",
        "6e9b2cc741875ab906d800af3134b076489f049565e0a1dbdb6deacd91f5054c",
        "v1.37.0/bin/linux/amd64/kubectl",
        "6129359f4e1f3848a5572ccb0b26cf28b8ca08cef38c95a765b2f64a2c961a2f",
        "bash terraform/lumen-standalone-gke/scripts/check.sh",
        "bash kustomize/lumen-standalone-acceptance/tests/contract.sh",
    ] {
        require(source.contains(needle), "GATES")?;
    }
    for forbidden in [
        "gh release create",
        "gh release publish",
        "gh release upload",
        "imagetools create",
        ":latest",
        "gke-gcloud-auth-plugin",
        "git tag ",
    ] {
        require(!source.contains(forbidden), "CANDIDATE_ONLY")?;
    }
    require(
        source.contains(
            "candidate_tag=release-candidate-${{ github.run_id }}-${{ github.run_attempt }}",
        ),
        "IMAGE",
    )?;
    require(source.contains("org.opencontainers.image.url=https://github.com/${{ github.repository }}/actions/runs/${{ github.run_id }}/attempts/${{ github.run_attempt }}"), "IMAGE")?;
    require(
        source.contains(".manifests | type == \"array\" and length == 2"),
        "DIGESTS",
    )?;
    require(
        source.contains("[.manifests[].digest] | unique | length == 2"),
        "DIGESTS",
    )?;
    require(source.contains("sort == [\"amd64\",\"arm64\"]"), "DIGESTS")?;
    require(
        source.contains(
            "\"$root\" != \"$amd64\" && \"$root\" != \"$arm64\" && \"$amd64\" != \"$arm64\"",
        ),
        "DIGESTS",
    )?;
    require(source.matches("cosign sign --yes").count() == 1, "ATTEST")?;
    require(
        source.matches("uses: actions/attest@").count() == 3,
        "ATTEST",
    )?;
    require(
        source.matches("uses: anchore/sbom-action@").count() == 2,
        "ATTEST",
    )?;
    require(source.contains("name: lumen-candidate-${{ matrix.target }}-${{ github.run_id }}-${{ github.run_attempt }}"), "ARTIFACTS")?;
    for target in [
        "aarch64-apple-darwin",
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-unknown-linux-musl",
        "aarch64-unknown-linux-musl",
    ] {
        require(source.contains(&format!("target: {target}")), "ARTIFACTS")?;
    }
    require(
        source.contains("schema:\"cclab.lumen.candidate-manifest.v3\""),
        "MANIFEST",
    )?;
    for binding in [
        "--arg run_id",
        "--arg run_attempt",
        "--arg run_url",
        "--arg source_ref",
        "--arg workflow_ref",
        "--argjson pr_number",
        "--arg pr_url",
    ] {
        require(source.contains(binding), "MANIFEST")?;
    }
    require(
        source.contains(
            ". + {jobs:{identity:\"${{ needs.identity.result }}\",build:\"${{ needs.build.result }}\",manifest:\"${{ needs.manifest.result }}\",\"ghcr-image-and-attest\":\"${{ needs.ghcr-image-and-attest.result }}\",\"verify-candidate\":\"${{ needs.verify-candidate.result }}\",\"verify-libraries\":\"${{ needs.verify-libraries.result }}\",\"kind-amd64\":\"${{ needs.kind-amd64.result }}\",\"kind-arm64\":\"${{ needs.kind-arm64.result }}\",result:\"success\"}}",
        ),
        "MANIFEST",
    )?;
    require(source.contains("LUMEN_E2E_EXPECTED_RUNTIME_DIGEST=\"${{ needs.ghcr-image-and-attest.outputs.amd64_digest }}\""), "KIND")?;
    require(source.contains("LUMEN_E2E_EXPECTED_RUNTIME_DIGEST=\"${{ needs.ghcr-image-and-attest.outputs.arm64_digest }}\""), "KIND")?;
    require(
        source.contains("--manifest-sidecar candidate/final-candidate-manifest.json.sha256"),
        "MANIFEST",
    )?;
    validate_dockerfile(dockerfile)?;
    Ok(())
}

fn validate_workflow(source: &str, dockerfile: &str) -> Result<(), Finding> {
    validate_workflow_semantics(source, dockerfile)?;
    require(
        sha256_bytes(source.as_bytes()) == WORKFLOW_BYTES_SHA256,
        "WORKFLOW_BYTES",
    )
}

fn validate_dockerfile(source: &str) -> Result<(), Finding> {
    const DEBIAN: &str = "debian:bookworm-slim@sha256:88200866dfff7ea7f5cbcb6ec7c8a701889efe6fe859fe64d6990e4b07ea4171";
    const DISTROLESS: &str = "gcr.io/distroless/static-debian12:nonroot@sha256:afa5c872c891853ca7fcf1f12c3edb23f7eeef36189728842dd51042ff57f7ab";
    let froms = source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("FROM "))
        .map(|line| {
            line.split_whitespace()
                .take(3)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>();
    require(
        froms
            == vec![
                format!("{DEBIAN} AS seed"),
                format!("{DEBIAN} AS binary-source-fetch"),
                format!("{DEBIAN} AS binary-source-staged"),
                "binary-source-${SOURCE} AS binary-source".to_string(),
                DISTROLESS.to_string(),
            ],
        "BASE_IMAGE",
    )
}

fn validate_verifier(source: &str, mode: u32) -> Result<(), Finding> {
    require(mode & 0o777 == 0o755, "MODE")?;
    for needle in [
        "`local` validates synthetic files only. It is not candidate acceptance.",
        "`full` also validates the run-scoped GHCR image", "cclab.lumen.candidate-manifest.v3",
        "run_url == (\"https://github.com/\" + $repo + \"/actions/runs/\" + $run_id + \"/attempts/\" + $attempt)",
        ".source_ref == \"refs/heads/main\"", ".workflow_ref == $workflow_ref",
        ".jobs == {identity:\"success\",build:\"success\",manifest:\"success\",\"ghcr-image-and-attest\":\"success\",\"verify-candidate\":\"success\",\"verify-libraries\":\"success\",\"kind-amd64\":\"success\",\"kind-arm64\":\"success\",result:\"success\"}",
        "archive members changed", "archive binary is not executable", "invalid SPDX 2.3 SBOM",
        "predicate == $sbom[0]", "--certificate-identity \"$EXPECTED_CERT_ID\"",
        "--cert-oidc-issuer https://token.actions.githubusercontent.com",
        "expected_run_url=\"https://github.com/${REPO}/actions/runs/${RUN_ID}/attempts/${RUN_ATTEMPT}\"",
        "org.opencontainers.image.url", "archive_sha256", "sidecar_sha256", "spdx-${sbom}.json",
        "candidate tag is not scoped to this run attempt", "LOCAL FIXTURE ONLY: artifacts verified; this is not candidate acceptance.",
    ] {
        require(source.contains(needle), "VERIFIER")?;
    }
    for forbidden in [
        "gh release create",
        "gh release publish",
        "cosign sign",
        "imagetools create",
    ] {
        require(!source.contains(forbidden), "VERIFIER_SIDE_EFFECT")?;
    }
    require(
        sha256_bytes(source.as_bytes()) == VERIFIER_BYTES_SHA256,
        "VERIFIER_BYTES",
    )
}

fn expect_workflow(source: &str, from: &str, to: &str, code: &'static str) {
    let changed = replace_once(source, from, to);
    assert_eq!(
        validate_workflow(&changed, &dockerfile()).unwrap_err(),
        Finding(code)
    );
}
fn expect_verifier(source: &str, from: &str, to: &str, code: &'static str) {
    let changed = replace_once(source, from, to);
    assert_eq!(
        validate_verifier(&changed, 0o755).unwrap_err(),
        Finding(code)
    );
}
fn workflow() -> String {
    fs::read_to_string(root().join(".github/workflows/lumen-release-candidate.yml")).unwrap()
}
fn perf_gate_source() -> String {
    fs::read_to_string(root().join("apps/lumen/e2e/perf_gate.rs")).unwrap()
}
fn verifier() -> (String, u32) {
    let path = root().join("apps/lumen/scripts/verify-release-candidate.sh");
    (
        fs::read_to_string(&path).unwrap(),
        fs::metadata(path).unwrap().permissions().mode(),
    )
}
fn dockerfile() -> String {
    fs::read_to_string(root().join("apps/lumen/Dockerfile.release")).unwrap()
}

#[test]
fn cloud_free_gate_mutations_fail_without_hash_oracle() {
    let source = workflow();
    let assert_finding = |from: &str, to: &str, finding| {
        let changed = replace_once(&source, from, to);
        assert_eq!(
            validate_workflow_semantics(&changed, &dockerfile()).unwrap_err(),
            Finding(finding),
        );
    };
    assert_finding(
        "ruby/setup-ruby@95ef2b042f9d7a56d8268cba8559e2842e2ad01b",
        "ruby/setup-ruby@0000000000000000000000000000000000000000",
        "CLOUD_FREE_GATES",
    );
    assert_finding(
        "ruby-version: '3.3.6'",
        "ruby-version: '3.3.7'",
        "CLOUD_FREE_GATES",
    );
    assert_finding(
        "terraform/1.9.4/terraform_1.9.4_linux_amd64.zip",
        "terraform/1.9.5/terraform_1.9.5_linux_amd64.zip",
        "GATE_COMMANDS",
    );
    assert_finding(
        "6e9b2cc741875ab906d800af3134b076489f049565e0a1dbdb6deacd91f5054c",
        "0000000000000000000000000000000000000000000000000000000000000000",
        "GATE_COMMANDS",
    );
    assert_finding(
        "v1.37.0/bin/linux/amd64/kubectl",
        "v1.37.1/bin/linux/amd64/kubectl",
        "GATE_COMMANDS",
    );
    assert_finding(
        "6129359f4e1f3848a5572ccb0b26cf28b8ca08cef38c95a765b2f64a2c961a2f",
        "0000000000000000000000000000000000000000000000000000000000000000",
        "GATE_COMMANDS",
    );
    assert_finding(
        "bash terraform/lumen-standalone-gke/scripts/check.sh",
        "bash terraform/lumen-standalone-gke/scripts/other.sh",
        "GATE_COMMANDS",
    );
    assert_finding(
        "bash kustomize/lumen-standalone-acceptance/tests/contract.sh",
        "bash kustomize/lumen-standalone-acceptance/tests/other.sh",
        "GATE_COMMANDS",
    );
    let terraform_step = "      - name: Install verified Terraform 1.9.4\n        shell: bash\n        run: |\n          set -euo pipefail\n          curl -fsSL https://releases.hashicorp.com/terraform/1.9.4/terraform_1.9.4_linux_amd64.zip -o /tmp/terraform.zip\n          echo '6e9b2cc741875ab906d800af3134b076489f049565e0a1dbdb6deacd91f5054c  /tmp/terraform.zip' | sha256sum -c -\n          unzip -oq /tmp/terraform.zip -d /tmp/terraform-bin\n          sudo install -m 0755 /tmp/terraform-bin/terraform /usr/local/bin/terraform\n";
    let without_terraform = replace_once(&source, terraform_step, "");
    assert_eq!(
        validate_workflow_semantics(&without_terraform, &dockerfile()).unwrap_err(),
        Finding("GATE_COMMANDS"),
    );
    let terraform_start = source
        .find("      - name: Run cloud-free Terraform acceptance gate\n")
        .unwrap();
    let product_start = source
        .find("      - name: Run required Lumen product gates without GKE\n")
        .unwrap();
    assert!(terraform_start < product_start);
    let product_end = product_start
        + source[product_start..]
            .find("\n      - name: Verify full run-scoped candidate supply chain\n")
            .unwrap()
        + 1;
    let terraform_block = &source[terraform_start..product_start];
    let product_block = &source[product_start..product_end];
    let moved = format!(
        "{}{}{}{}",
        &source[..terraform_start],
        product_block,
        terraform_block,
        &source[product_end..]
    );
    assert_eq!(
        validate_workflow_semantics(&moved, &dockerfile()).unwrap_err(),
        Finding("CLOUD_FREE_GATES"),
    );
    assert_finding(
        "        run: bash terraform/lumen-standalone-gke/scripts/check.sh",
        "        run: '# bash terraform/lumen-standalone-gke/scripts/check.sh'",
        "GATE_COMMANDS",
    );
    assert_finding(
        "        run: bash kustomize/lumen-standalone-acceptance/tests/contract.sh",
        "        run: echo 'bash kustomize/lumen-standalone-acceptance/tests/contract.sh'",
        "GATE_COMMANDS",
    );
    assert_finding(
        "        run: bash terraform/lumen-standalone-gke/scripts/check.sh",
        "        run: if false; then bash terraform/lumen-standalone-gke/scripts/check.sh; fi",
        "GATE_COMMANDS",
    );
}

#[test]
fn live_candidate_contract_is_fail_closed() {
    let source = workflow();
    const DURABLE_GATE: &str = "LUMEN_STANDALONE_DURABLE_IMAGE=\"${{ needs.ghcr-image-and-attest.outputs.image_repo }}@${{ needs.ghcr-image-and-attest.outputs.root_digest }}\" bash apps/lumen/scripts/standalone-container-smoke.sh durable";
    for (replacement, finding) in [
        ("# LUMEN_STANDALONE_DURABLE_IMAGE=\"${{ needs.ghcr-image-and-attest.outputs.image_repo }}@${{ needs.ghcr-image-and-attest.outputs.root_digest }}\" bash apps/lumen/scripts/standalone-container-smoke.sh durable", "GATES"),
        ("echo 'LUMEN_STANDALONE_DURABLE_IMAGE=\"${{ needs.ghcr-image-and-attest.outputs.image_repo }}@${{ needs.ghcr-image-and-attest.outputs.root_digest }}\" bash apps/lumen/scripts/standalone-container-smoke.sh durable'", "GATES"),
        ("if false; then LUMEN_STANDALONE_DURABLE_IMAGE=\"${{ needs.ghcr-image-and-attest.outputs.image_repo }}@${{ needs.ghcr-image-and-attest.outputs.root_digest }}\" bash apps/lumen/scripts/standalone-container-smoke.sh durable; fi", "GATES"),
        ("LUMEN_STANDALONE_DURABLE_IMAGE=\"${{ needs.ghcr-image-and-attest.outputs.image_repo }}@${{ needs.ghcr-image-and-attest.outputs.amd64_digest }}\" bash apps/lumen/scripts/standalone-container-smoke.sh durable", "GATES"),
        ("LUMEN_STANDALONE_DURABLE_IMAGE=\"${{ needs.ghcr-image-and-attest.outputs.image_repo }}@${{ needs.ghcr-image-and-attest.outputs.arm64_digest }}\" bash apps/lumen/scripts/standalone-container-smoke.sh durable", "GATES"),
        ("LUMEN_STANDALONE_DURABLE_IMAGE=\"${{ needs.ghcr-image-and-attest.outputs.image_repo }}:latest\" bash apps/lumen/scripts/standalone-container-smoke.sh durable", "GATES"),
        ("LUMEN_STANDALONE_DURABLE_IMAGE=\"${{ needs.ghcr-image-and-attest.outputs.image_repo }}@${{ needs.ghcr-image-and-attest.outputs.root_digest }}\" bash apps/lumen/scripts/standalone-container-smoke.sh bind", "GATES"),
    ] {
        let changed = source.replace(DURABLE_GATE, replacement);
        assert_eq!(
            validate_workflow_semantics(&changed, &dockerfile()).unwrap_err(),
            Finding(finding)
        );
    }
    validate_workflow(&source, &dockerfile()).expect("candidate workflow contract");
    let (script, mode) = verifier();
    validate_verifier(&script, mode).expect("candidate verifier contract");
    assert_eq!(validate_verifier(&script, 0o644), Err(Finding("MODE")));
}

#[test]
fn candidate_source_mutations_fail_with_stable_categories() {
    let source = workflow();
    let root_digest_metadata = replace_once(
        &source,
        "root_digest: ${{ steps.push.outputs.digest }}",
        "root_digest: ${{ steps.push.outputs.metadata }}",
    );
    assert_eq!(
        validate_workflow_semantics(&root_digest_metadata, &dockerfile()).unwrap_err(),
        Finding("IMAGE")
    );
    let gcloud_anchor = "          echo \"workflow_ref=$WORKFLOW_REF\"\n";
    for (label, injection) in [
        ("gcloud tab", "          gcloud\tcontainer clusters list\n"),
        (
            "gcloud continuation",
            "          gcloud \\\n          container clusters list\n",
        ),
        (
            "gcloud executable path",
            "          /usr/local/bin/gcloud container clusters list\n",
        ),
        (
            "gcloud sudo wrapper",
            "          sudo -u root gcloud container clusters list\n",
        ),
        (
            "gcloud env wrapper",
            "          env gcloud container clusters list\n",
        ),
        (
            "gcloud command wrapper",
            "          command gcloud container clusters list\n",
        ),
        (
            "gcloud bash c wrapper",
            "          bash -c 'gcloud container clusters list'\n",
        ),
        (
            "gcloud sh c wrapper",
            "          sh -c 'gcloud container clusters list'\n",
        ),
        (
            "gcloud env path bash c wrapper",
            "          /usr/bin/env bash -c 'gcloud container clusters list'\n",
        ),
    ] {
        let changed = replace_once(
            &source,
            gcloud_anchor,
            &format!("{gcloud_anchor}{injection}"),
        );
        assert_eq!(
            validate_workflow_semantics(&changed, &dockerfile()).unwrap_err(),
            Finding("CANDIDATE_ONLY"),
            "{label} mutation passed"
        );
    }
    let same_name_step = replace_once(
        &source,
        "      - name: Log in to GHCR with read-only job access\n        uses: docker/login-action@c94ce9fb468520275223c153574b00df6fe4bcc9 # v3.7.0\n        with:\n          registry: ghcr.io\n          username: ${{ github.actor }}\n          password: ${{ github.token }}\n",
        "      - name: Log in to GHCR with read-only job access\n        shell: bash\n        run: |\n          # docker/login-action@c94ce9fb468520275223c153574b00df6fe4bcc9\n          printf '#!/usr/bin/env bash\\nexit 0\\n' > apps/lumen/scripts/verify-release-candidate.sh\n",
    );
    assert_ne!(same_name_step, source);
    assert_eq!(
        validate_workflow(&same_name_step, &dockerfile()).unwrap_err(),
        Finding("WORKFLOW_BYTES")
    );
    for (from, to, code) in [
        (
            "description: Exact Lumen semver without the lumen@ prefix.\n        required: true",
            "description: Exact Lumen semver without the lumen@ prefix.\n        required: false",
            "INPUTS",
        ),
        (
            "needs: [identity, manifest, ghcr-image-and-attest]",
            "needs: [identity, ghcr-image-and-attest]",
            "GRAPH",
        ),
        (
            "attestations: read\n      contents: read\n      packages: read",
            "attestations: read\n      contents: read",
            "PERMISSIONS",
        ),
        (
            "candidate_tag=release-candidate-${{ github.run_id }}-${{ github.run_attempt }}",
            "candidate_tag=lumen@${{ needs.identity.outputs.version }}",
            "IMAGE",
        ),
        ("--mode local", "--mode full", "GATES"),
        (
            "cargo test -p lumen --test release_candidate",
            "true",
            "GATES",
        ),
        (
            "dtolnay/rust-toolchain@4360b52568e2003a75bf9bc1d59f33a8e3fc893c",
            "dtolnay/rust-toolchain@stable",
            "ACTION_PIN",
        ),
        ("version: 0.12.1", "version: 0.12.2", "UV_SETUP"),
        (
            "imagetools inspect --raw",
            "imagetools create",
            "CANDIDATE_ONLY",
        ),
    ] {
        expect_workflow(&source, from, to, code);
    }
    for (name, replacement) in [
        (
            "missing release",
            RELEASE_PERF_GATE.replace("--release ", ""),
        ),
        ("missing locked", RELEASE_PERF_GATE.replace("--locked ", "")),
        (
            "missing ignored",
            RELEASE_PERF_GATE.replace("--ignored ", ""),
        ),
        (
            "missing test threads",
            RELEASE_PERF_GATE.replace("--test-threads=1 ", ""),
        ),
        (
            "missing nocapture",
            RELEASE_PERF_GATE.replace("--nocapture", ""),
        ),
        ("comment", format!("# {RELEASE_PERF_GATE}")),
        ("quoted prose", format!("echo '{RELEASE_PERF_GATE}'")),
        (
            "reordered command",
            RELEASE_PERF_GATE.replace("--release --locked", "--locked --release"),
        ),
        ("semicolon split", format!("{RELEASE_PERF_GATE}; true")),
        ("and split", format!("true && {RELEASE_PERF_GATE}")),
        ("if split", format!("if true; then {RELEASE_PERF_GATE}; fi")),
        ("eval split", format!("eval '{RELEASE_PERF_GATE}'")),
    ] {
        let changed = replace_once(&source, RELEASE_PERF_GATE, &replacement);
        assert_eq!(
            validate_workflow(&changed, &dockerfile()).unwrap_err(),
            Finding("GATES"),
            "{name} mutation passed",
        );
    }
    let perf_source = perf_gate_source();
    for occurrence in 0..3 {
        let changed = replace_occurrence(
            &perf_source,
            "#[ignore = \"coarse performance gate runs in the release candidate workflow\"]\n",
            "",
            occurrence,
        );
        assert_eq!(
            validate_perf_gate_source(&changed).unwrap_err(),
            Finding("PERF_GATE"),
            "missing ignore occurrence {occurrence} passed",
        );
    }
    let without_ignores = perf_source.replace(
        "#[ignore = \"coarse performance gate runs in the release candidate workflow\"]\n",
        "",
    );
    assert_eq!(
        validate_perf_gate_source(&without_ignores).unwrap_err(),
        Finding("PERF_GATE")
    );
    let statistic_ignored = replace_once(
        &perf_source,
        "#[test]\nfn median_statistic_and_ignored_inventory",
        "#[test]\n#[ignore]\nfn median_statistic_and_ignored_inventory",
    );
    assert_eq!(
        validate_perf_gate_source(&statistic_ignored).unwrap_err(),
        Finding("PERF_GATE")
    );
    let fourth_ignored = replace_once(
        &perf_source,
        "// CODEGEN-END",
        "#[test]\n#[ignore]\nfn extra_perf_row() {}\n// CODEGEN-END",
    );
    assert_eq!(
        validate_perf_gate_source(&fourth_ignored).unwrap_err(),
        Finding("PERF_GATE")
    );
    let comment_decoy = replace_occurrence(
        &perf_source,
        "#[ignore = \"coarse performance gate runs in the release candidate workflow\"]\n",
        "// #[ignore]\n",
        0,
    );
    assert_eq!(
        validate_perf_gate_source(&comment_decoy).unwrap_err(),
        Finding("PERF_GATE")
    );
    let block_comment_decoy = replace_occurrence(
        &perf_source,
        "#[ignore = \"coarse performance gate runs in the release candidate workflow\"]\n",
        "/*\n#[ignore]\nfn comment_decoy() {}\n*/\n",
        0,
    );
    assert_eq!(
        validate_perf_gate_source(&block_comment_decoy).unwrap_err(),
        Finding("PERF_GATE")
    );
    let changed = replace_occurrence(
        &source,
        "LUMEN_E2E_IMAGE=\"${{ needs.ghcr-image-and-attest.outputs.image_repo }}@${{ needs.ghcr-image-and-attest.outputs.root_digest }}\"",
        "LUMEN_E2E_IMAGE=\"${{ needs.ghcr-image-and-attest.outputs.image_repo }}@${{ needs.ghcr-image-and-attest.outputs.amd64_digest }}\"",
        0,
    );
    assert_eq!(
        validate_workflow(&changed, &dockerfile()).unwrap_err(),
        Finding("GATE_COMMANDS")
    );
    expect_workflow(
        &source,
        "LUMEN_E2E_EXPECTED_RUNTIME_DIGEST=\"${{ needs.ghcr-image-and-attest.outputs.amd64_digest }}\"",
        "LUMEN_E2E_EXPECTED_RUNTIME_DIGEST=\"${{ needs.ghcr-image-and-attest.outputs.arm64_digest }}\"",
        "GATE_COMMANDS",
    );
    expect_workflow(
        &source,
        "LUMEN_E2E_EXPECTED_RUNTIME_DIGEST=\"${{ needs.ghcr-image-and-attest.outputs.arm64_digest }}\"",
        "LUMEN_E2E_EXPECTED_RUNTIME_DIGEST=\"${{ needs.ghcr-image-and-attest.outputs.amd64_digest }}\"",
        "GATE_COMMANDS",
    );
    expect_workflow(
        &source,
        "schema:\"cclab.lumen.candidate-manifest.v3\"",
        "schema:\"cclab.lumen.candidate-manifest.v2\"",
        "MANIFEST",
    );
    expect_workflow(
        &source,
        "  verify-libraries:\n",
        "  verify-libraries-missing:\n",
        "JOBS",
    );
    expect_workflow(
        &source,
        "\n  kind-amd64:\n",
        "\n  extra-job:\n    name: extra\n    needs: [identity]\n    runs-on: ubuntu-latest\n    permissions:\n      contents: read\n    steps: []\n\n  kind-amd64:\n",
        "JOBS",
    );
    expect_workflow(
        &source,
        "\"verify-libraries\":\"${{ needs.verify-libraries.result }}\"",
        "\"verify-libraries\":\"failure\"",
        "MANIFEST",
    );
    expect_workflow(
        &source,
        "  kind-amd64:\n    name: kind e2e (amd64)\n    needs: [identity, verify-candidate, verify-libraries, ghcr-image-and-attest]",
        "  kind-amd64:\n    name: kind e2e (amd64)\n    needs: [identity, verify-candidate, ghcr-image-and-attest]",
        "GRAPH",
    );
    expect_workflow(&source, "cargo test -p service-k8s", "true", "LIBRARIES");
    expect_workflow(
        &source,
        "          cargo test -p service-k8s\n          cargo test -p storage-durable\n          cargo test -p service-backup --features http-client\n          bash scripts/raft-implementor-build.sh",
        "          bash scripts/raft-implementor-build.sh\n          cargo test -p service-k8s\n          cargo test -p storage-durable\n          cargo test -p service-backup --features http-client",
        "LIBRARIES",
    );
    expect_workflow(
        &source,
        "          cargo test -p service-k8s\n          cargo test -p storage-durable\n          cargo test -p service-backup --features http-client\n          bash scripts/raft-implementor-build.sh",
        "          cargo test -p service-k8s\n          cargo test -p storage-durable\n          cargo test -p service-backup --features http-client\n          cargo test -p service-k8s\n          bash scripts/raft-implementor-build.sh",
        "LIBRARIES",
    );
    for (from, to) in [
        (
            "      - name: Run required Lumen product gates without GKE\n        shell: bash",
            "      - name: Run required Lumen product gates without GKE\n        if: false\n        shell: bash",
        ),
        (
            "      - name: Verify full run-scoped candidate supply chain\n        env:",
            "      - name: Verify full run-scoped candidate supply chain\n        continue-on-error: true\n        env:",
        ),
        (
            "  kind-amd64:\n    name: kind e2e (amd64)",
            "  kind-amd64:\n    name: kind e2e (amd64)\n    if: always()",
        ),
        (
            "  kind-arm64:\n    name: kind e2e (arm64)",
            "  kind-arm64:\n    name: kind e2e (arm64)\n    continue-on-error: true",
        ),
        (
            "  result:\n    name: final candidate receipt",
            "  result:\n    name: final candidate receipt\n    if: always()",
        ),
        (
            "      - name: Bind all successful job conclusions into final receipt\n        shell: bash",
            "      - name: Bind all successful job conclusions into final receipt\n        if: false\n        shell: bash",
        ),
    ] {
        expect_workflow(&source, from, to, "CONDITIONS");
    }
    for occurrence in 0..2 {
        let changed = replace_occurrence(
            &source,
            "      - name: Run prebuilt candidate kind e2e\n        shell: bash",
            "      - name: Run prebuilt candidate kind e2e\n        if: false\n        shell: bash",
            occurrence,
        );
        assert_eq!(
            validate_workflow(&changed, &dockerfile()).unwrap_err(),
            Finding("CONDITIONS"),
            "kind YAML condition occurrence {occurrence} passed",
        );
    }
    for (job, gate, marker) in [
        (
            "verify-candidate",
            "Verify full run-scoped candidate supply chain",
            "apps/lumen/scripts/verify-release-candidate.sh \\",
        ),
        (
            "kind-amd64",
            "Run prebuilt candidate kind e2e",
            "apps/lumen/scripts/kind-e2e.sh",
        ),
        (
            "kind-arm64",
            "Run prebuilt candidate kind e2e",
            "apps/lumen/scripts/kind-e2e.sh",
        ),
    ] {
        let insertion = format!(
            "      - name: Unrecognized overwrite step\n        shell: bash\n        run: |\n          printf '#!/usr/bin/env bash\\nexit 0\\n' > {marker}\n"
        );
        let anchor = format!("      - name: {gate}\n");
        let occurrence = if job == "verify-candidate" {
            0
        } else if job == "kind-amd64" {
            0
        } else {
            1
        };
        let changed = replace_occurrence(
            &source,
            &anchor,
            &format!("{insertion}{anchor}"),
            occurrence,
        );
        assert_eq!(
            validate_workflow(&changed, &dockerfile()).unwrap_err(),
            Finding("GATE_STEPS"),
            "unrecognized step before {job} gate passed",
        );
    }
    expect_workflow(
        &source,
        "  verify-libraries:\n    name: verify service and Raft library gates\n    needs: [identity]\n    runs-on: ubuntu-latest\n    permissions:\n      contents: read\n    steps:\n      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1\n        with:\n          ref: ${{ needs.identity.outputs.commit }}",
        "  verify-libraries:\n    name: verify service and Raft library gates\n    needs: [identity]\n    runs-on: ubuntu-latest\n    permissions:\n      contents: read\n    steps:\n      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1\n        with:\n          ref: ${{ github.sha }}",
        "LIBRARIES",
    );
    for occurrence in 0..2 {
        let changed = replace_occurrence(
            &source,
            "          apps/lumen/scripts/kind-e2e.sh",
            "          if false; then apps/lumen/scripts/kind-e2e.sh; fi",
            occurrence,
        );
        assert_eq!(
            validate_workflow(&changed, &dockerfile()).unwrap_err(),
            Finding("GATE_COMMANDS"),
            "kind dead-branch occurrence {occurrence} passed",
        );
    }
    let supply_chain_dead = replace_occurrence(
        &source,
        "          apps/lumen/scripts/verify-release-candidate.sh \\",
        "          if false; then\n          apps/lumen/scripts/verify-release-candidate.sh \\",
        0,
    );
    let supply_chain_dead = replace_once(
        &supply_chain_dead,
        "            --mode full",
        "            --mode full\n          fi",
    );
    assert_eq!(
        validate_workflow(&supply_chain_dead, &dockerfile()).unwrap_err(),
        Finding("GATE_COMMANDS"),
        "candidate supply-chain dead branch passed",
    );
    let uv_setup = "      - uses: astral-sh/setup-uv@c771a70e6277c0a99b617c7a806ffedaca235ff9 # v9.0.0\n        with:\n          version: 0.12.1\n          enable-cache: false\n";
    let without_uv = replace_once(&source, uv_setup, "");
    let gate_following_step = "      - name: Verify full run-scoped candidate supply chain\n";
    let moved_uv = format!("{uv_setup}{gate_following_step}");
    let setup_after_gate = replace_once(&without_uv, gate_following_step, &moved_uv);
    assert_ne!(setup_after_gate, source);
    assert_eq!(
        validate_workflow(&setup_after_gate, &dockerfile()).unwrap_err(),
        Finding("UV_SETUP")
    );
    let (script, _) = verifier();
    let bypassed_supply_chain = replace_once(
        &script,
        "\n  verify_full_supply_chain\n",
        "\n  true # verify_full_supply_chain\n",
    );
    assert_eq!(
        validate_verifier(&bypassed_supply_chain, 0o755).unwrap_err(),
        Finding("VERIFIER_BYTES")
    );
    for (from, to, code) in [
        (
            "LOCAL FIXTURE ONLY: artifacts verified; this is not candidate acceptance.",
            "CANDIDATE ACCEPTED",
            "VERIFIER",
        ),
        (
            ".source_ref == \"refs/heads/main\"",
            ".source_ref == \"refs/heads/dev\"",
            "VERIFIER",
        ),
        (
            "candidate tag is not scoped to this run attempt",
            "candidate tag accepted",
            "VERIFIER",
        ),
        ("cosign verify", "cosign sign", "VERIFIER_SIDE_EFFECT"),
    ] {
        expect_verifier(&script, from, to, code);
    }
    let docker = dockerfile();
    let changed = docker.replacen("FROM debian:bookworm-slim@sha256:88200866dfff7ea7f5cbcb6ec7c8a701889efe6fe859fe64d6990e4b07ea4171", "FROM debian:bookworm-slim@sha256:0000000000000000000000000000000000000000000000000000000000000000", 1);
    assert_ne!(changed, docker);
    assert_eq!(validate_dockerfile(&changed), Err(Finding("BASE_IMAGE")));
    assert!(validate_dockerfile(&docker).is_ok());
}

fn sha(path: &Path) -> String {
    let output = Command::new("shasum")
        .args(["-a", "256"])
        .arg(path)
        .output()
        .unwrap();
    String::from_utf8(output.stdout)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .into()
}
fn write_manifest(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
    let digest = sha(path);
    fs::write(
        path.with_extension("json.sha256"),
        format!(
            "{digest}  {}\n",
            path.file_name().unwrap().to_string_lossy()
        ),
    )
    .unwrap();
}
fn local_fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let artifacts = dir.path();
    let stage = artifacts.join("stage");
    for target in [
        "aarch64-apple-darwin",
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-unknown-linux-musl",
        "aarch64-unknown-linux-musl",
    ] {
        let package = stage.join(format!("lumen-{target}"));
        fs::create_dir_all(&package).unwrap();
        let binary = package.join("lumen");
        fs::write(&binary, "#!/bin/sh\nprintf 'lumen 0.4.27\\n'\n").unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
        fs::write(package.join("README.md"), "fixture\n").unwrap();
        let archive = artifacts.join(format!("lumen-{target}.tar.gz"));
        assert!(Command::new("tar")
            .args([
                "-C",
                stage.to_str().unwrap(),
                "-czf",
                archive.to_str().unwrap(),
                &format!("lumen-{target}")
            ])
            .status()
            .unwrap()
            .success());
        let archive_sha = sha(&archive);
        fs::write(
            artifacts.join(format!("lumen-{target}.tar.gz.sha256")),
            format!("{archive_sha}  lumen-{target}.tar.gz\n"),
        )
        .unwrap();
    }
    for arch in ["amd64", "arm64"] {
        fs::write(
            artifacts.join(format!("spdx-{arch}.json")),
            r#"{"spdxVersion":"SPDX-2.3"}"#,
        )
        .unwrap();
    }
    let targets = [
        "aarch64-apple-darwin",
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-unknown-linux-musl",
        "aarch64-unknown-linux-musl",
    ];
    let artifacts_json: Vec<_> = targets.iter().map(|target| json!({"target":target,"archive":format!("lumen-{target}.tar.gz"),"archive_sha256":sha(&artifacts.join(format!("lumen-{target}.tar.gz"))),"sidecar":format!("lumen-{target}.tar.gz.sha256"),"sidecar_sha256":sha(&artifacts.join(format!("lumen-{target}.tar.gz.sha256")))})).collect();
    let manifest = json!({"schema":"cclab.lumen.candidate-manifest.v3","repository":"chrischeng-c4/axiom","workflow_path":".github/workflows/lumen-release-candidate.yml","workflow_id":42,"run_id":"7","run_attempt":"2","run_url":"https://github.com/chrischeng-c4/axiom/actions/runs/7/attempts/2","source_ref":"refs/heads/main","workflow_ref":"chrischeng-c4/axiom/.github/workflows/lumen-release-candidate.yml@refs/heads/main","commit":"0123456789012345678901234567890123456789","version":"0.4.27","tag":"lumen@0.4.27","candidate_tag":"release-candidate-7-2","pr":{"number":42,"url":"https://github.com/chrischeng-c4/axiom/pull/42"},"image":{"repository":"ghcr.io/chrischeng-c4/lumen","root_digest":format!("sha256:{}", "1".repeat(64)),"amd64_digest":format!("sha256:{}", "2".repeat(64)),"arm64_digest":format!("sha256:{}", "3".repeat(64))},"artifacts":artifacts_json,"sboms":{"amd64":{"file":"spdx-amd64.json","sha256":sha(&artifacts.join("spdx-amd64.json"))},"arm64":{"file":"spdx-arm64.json","sha256":sha(&artifacts.join("spdx-arm64.json"))}},"jobs":{"identity":"success","build":"success","manifest":"success","ghcr-image-and-attest":"success","verify-candidate":"success","verify-libraries":"success","kind-amd64":"success","kind-arm64":"success","result":"success"}});
    write_manifest(&artifacts.join("final-candidate-manifest.json"), &manifest);
    dir
}

fn replace_host_archive(dir: &Path, stage_name: &str, readme: bool, mode: u32, version: &str) {
    let target = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        _ => panic!("unsupported verifier host"),
    };
    let stage_root = dir.join(stage_name);
    let package = stage_root.join(format!("lumen-{target}"));
    fs::create_dir_all(&package).unwrap();
    let binary = package.join("lumen");
    fs::write(&binary, format!("#!/bin/sh\nprintf 'lumen {version}\\n'\n")).unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(mode)).unwrap();
    if readme {
        fs::write(package.join("README.md"), "fixture\n").unwrap();
    }
    let archive = dir.join(format!("lumen-{target}.tar.gz"));
    assert!(Command::new("tar")
        .args([
            "-C",
            stage_root.to_str().unwrap(),
            "-czf",
            archive.to_str().unwrap(),
            &format!("lumen-{target}")
        ])
        .status()
        .unwrap()
        .success());
    let archive_sidecar = dir.join(format!("lumen-{target}.tar.gz.sha256"));
    let archive_digest = sha(&archive);
    fs::write(
        &archive_sidecar,
        format!("{archive_digest}  lumen-{target}.tar.gz\n"),
    )
    .unwrap();
    let path = dir.join("final-candidate-manifest.json");
    let mut manifest: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let index = manifest["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .position(|item| item["target"] == target)
        .unwrap();
    manifest["artifacts"][index]["archive_sha256"] = json!(archive_digest);
    manifest["artifacts"][index]["sidecar_sha256"] = json!(sha(&archive_sidecar));
    write_manifest(&path, &manifest);
}

fn run_local(dir: &Path) -> Output {
    Command::new("bash")
        .arg(root().join("apps/lumen/scripts/verify-release-candidate.sh"))
        .args([
            "--repo",
            "chrischeng-c4/axiom",
            "--version",
            "0.4.27",
            "--commit",
            "0123456789012345678901234567890123456789",
            "--run-id",
            "7",
            "--run-attempt",
            "2",
            "--manifest",
        ])
        .arg(dir.join("final-candidate-manifest.json"))
        .args(["--manifest-sidecar"])
        .arg(dir.join("final-candidate-manifest.json.sha256"))
        .args(["--artifacts-dir"])
        .arg(dir)
        .args(["--mode", "local"])
        .output()
        .unwrap()
}

#[test]
fn local_final_receipt_and_negative_fixtures_are_executable() {
    assert!(run_local(local_fixture().path()).status.success());
    fn negative(name: &str, mutate: fn(&Path), needle: &str) {
        let dir = local_fixture();
        mutate(dir.path());
        let output = run_local(dir.path());
        assert!(!output.status.success(), "{name} mutation passed");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(needle),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    negative(
        "checksum",
        |dir| {
            let path = dir.join("final-candidate-manifest.json");
            let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            value["artifacts"][0]["archive_sha256"] = json!("0".repeat(64));
            write_manifest(&path, &value);
        },
        "archive checksum mismatch",
    );
    negative(
        "manifest sidecar",
        |dir| {
            fs::write(
                dir.join("final-candidate-manifest.json.sha256"),
                "0  final-candidate-manifest.json\n",
            )
            .unwrap();
        },
        "manifest sidecar does not bind",
    );
    negative(
        "SBOM",
        |dir| {
            fs::write(dir.join("spdx-amd64.json"), "{}\n").unwrap();
        },
        "SBOM checksum mismatch",
    );
    negative(
        "job result",
        |dir| {
            let path = dir.join("final-candidate-manifest.json");
            let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            value["jobs"]["kind-amd64"] = json!("failure");
            write_manifest(&path, &value);
        },
        "final candidate manifest does not bind",
    );
    negative(
        "run/ref",
        |dir| {
            let path = dir.join("final-candidate-manifest.json");
            let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            value["source_ref"] = json!("refs/heads/dev");
            write_manifest(&path, &value);
        },
        "candidate manifest bindings changed",
    );
    negative(
        "archive member",
        |dir| replace_host_archive(dir, "bad-stage", false, 0o755, "0.4.27"),
        "archive members changed",
    );
    negative(
        "executable mode",
        |dir| replace_host_archive(dir, "mode-stage", true, 0o644, "0.4.27"),
        "archive binary is not executable",
    );
    negative(
        "wrong version",
        |dir| replace_host_archive(dir, "version-stage", true, 0o755, "0.4.26"),
        "candidate binary version mismatch",
    );
}
