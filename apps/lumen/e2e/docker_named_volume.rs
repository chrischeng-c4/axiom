//! Deterministic contract and smoke tests for Lumen Docker named-volume persistence.

use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use serde_yaml::{Mapping as YamlMapping, Value as YamlValue};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn repo_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().unwrap().parent().unwrap().into()
}

fn one(src: &str, from: &str, to: &str) -> String {
    assert_eq!(src.matches(from).count(), 1, "target {from:?} not unique");
    src.replacen(from, to, 1)
}

fn lines(content: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut buf = String::new();
    for raw in content.lines() {
        let raw = raw.trim_end();
        if raw.trim_start().starts_with('#') && buf.is_empty() {
            continue;
        }
        if let Some(part) = raw.strip_suffix('\\') {
            buf.push_str(part);
            buf.push(' ');
            continue;
        }
        buf.push_str(raw);
        let line = buf.trim();
        if !line.is_empty() && !line.starts_with('#') {
            lines.push(line.to_string());
        }
        buf.clear();
    }
    lines
}

fn is(line: &str, expected: &str) -> bool {
    line.split_whitespace()
        .next()
        .is_some_and(|word| word.eq_ignore_ascii_case(expected))
}

fn instruction_identity(line: &str, prefix: &str) -> bool {
    let line = line.to_ascii_uppercase();
    let prefix = prefix.to_ascii_uppercase();
    line == prefix
        || line.starts_with(&format!("{prefix}="))
        || line.starts_with(&format!("{prefix} "))
}

fn data_copy(line: &str) -> bool {
    let words: Vec<_> = line.split_whitespace().collect();
    is(line, "COPY")
        && (words.contains(&"/out/lumen-data/") || words.last() == Some(&"/var/lib/lumen/data/"))
}

macro_rules! reject {
    ($condition:expr, $reason:literal) => {
        if $condition {
            return Err($reason);
        }
    };
}

fn valid(content: &str, exp_seed: &str, exp_base: &str) -> Result<(), &'static str> {
    let instrs = lines(content);
    let froms: Vec<usize> = instrs
        .iter()
        .enumerate()
        .filter_map(|(i, line)| is(line, "FROM").then_some(i))
        .collect();
    reject!(froms.is_empty(), "no-stages");
    let seed_slice = froms
        .iter()
        .enumerate()
        .find(|(_, &i)| {
            let p: Vec<&str> = instrs[i].split_whitespace().collect();
            p.len() >= 4 && p[2].eq_ignore_ascii_case("AS") && p[3] == exp_seed
        })
        .map(|(idx, &i)| i..froms.get(idx + 1).copied().unwrap_or(instrs.len()))
        .ok_or("seed-layout")?;
    let seed = instrs[seed_slice].join("\n");
    reject!(
        ["0750", "/out/lumen-data", ".lumen-volume-seed"]
            .iter()
            .any(|r| !seed.contains(r)),
        "seed-layout"
    );
    let last_from = *froms.last().unwrap();
    reject!(
        instrs[last_from].split_whitespace().nth(1) != Some(exp_base),
        "final-base"
    );
    let b_copies = instrs[..last_from]
        .iter()
        .filter(|line| data_copy(line))
        .count();
    let mut copies = Vec::new();
    let mut ep_idx = None;
    for (idx, line) in instrs[last_from..].iter().enumerate() {
        let abs = last_from + idx;
        let words: Vec<&str> = line.split_whitespace().collect();
        if is(line, "USER") {
            let user = words.get(1).copied().unwrap_or("");
            reject!(!matches!(user, "65532" | "65532:65532"), "root-user");
        }
        if is(line, "ENTRYPOINT") && ep_idx.is_none() {
            ep_idx = Some(abs);
        }
        if data_copy(line) {
            copies.push((abs, line));
        }
    }
    let entrypoint = ep_idx.ok_or("missing-entrypoint")?;
    reject!(
        instrs
            .iter()
            .any(|line| instruction_identity(line, "ENV LUMEN_FSYNC")),
        "fsync-knob"
    );
    for (prefix, expected) in [
        ("ENV LUMEN_HOST", "ENV LUMEN_HOST=0.0.0.0"),
        (
            "ENV LUMEN_DATA_DIR",
            "ENV LUMEN_DATA_DIR=/var/lib/lumen/data",
        ),
        ("ENV LUMEN_PERSISTENCE", "ENV LUMEN_PERSISTENCE=segment"),
        ("ENV LUMEN_WAL", "ENV LUMEN_WAL=embedded"),
        ("VOLUME", "VOLUME [\"/var/lib/lumen/data\"]"),
    ] {
        let matches: Vec<_> = instrs
            .iter()
            .enumerate()
            .filter(|(_, line)| instruction_identity(line, prefix))
            .collect();
        reject!(matches.is_empty(), "missing-image-default");
        reject!(matches.len() != 1, "duplicate-image-default");
        let (index, actual) = matches[0];
        reject!(index < last_from, "builder-only-image-default");
        reject!(actual != expected, "invalid-image-default");
        reject!(index >= entrypoint, "late-image-default");
    }
    if copies.is_empty() {
        return if b_copies > 0 {
            Err("builder-only-copy")
        } else {
            Err("missing-copy")
        };
    }
    reject!(copies.len() > 1, "duplicate-copy");
    let (copy_idx, copy_line) = copies[0];
    reject!(copy_idx >= entrypoint, "copy-order");
    let tokens: Vec<&str> = copy_line.split_whitespace().collect();
    let exp_from = format!("--from={exp_seed}");
    reject!(tokens.get(1) != Some(&exp_from.as_str()), "copy-source");
    reject!(tokens.get(2) != Some(&"--chown=65532:65532"), "copy-flags");
    let late_flag = tokens
        .get(3..)
        .unwrap_or_default()
        .iter()
        .any(|t| t.starts_with("--"));
    reject!(late_flag, "copy-flags");
    reject!(
        tokens.len() != 5 || tokens[3] != "/out/lumen-data/",
        "copy-source"
    );
    reject!(tokens[4] != "/var/lib/lumen/data/", "copy-destination");
    Ok(())
}

const CC_BASE: &str = "gcr.io/distroless/cc-debian12:nonroot";
const STATIC_BASE: &str = "gcr.io/distroless/static-debian12:nonroot";
const RELEASE_STATIC_BASE: &str = "gcr.io/distroless/static-debian12:nonroot@sha256:afa5c872c891853ca7fcf1f12c3edb23f7eeef36189728842dd51042ff57f7ab";
const COPY: &str =
    "COPY --from=builder --chown=65532:65532 /out/lumen-data/ /var/lib/lumen/data/\n";

#[test]
fn test_checked_in_and_rendered_dockerfiles_satisfy_contract() {
    let root = repo_root();
    let mut discovered: Vec<String> = std::fs::read_dir(root.join("apps/lumen"))
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|name| name.starts_with("Dockerfile"))
        .map(|name| format!("apps/lumen/{name}"))
        .collect();
    discovered.sort();
    assert_eq!(
        discovered,
        [
            "apps/lumen/Dockerfile",
            "apps/lumen/Dockerfile.release",
            "apps/lumen/Dockerfile.test",
        ]
    );
    let targets = [
        ("apps/lumen/Dockerfile", "builder", CC_BASE),
        ("apps/lumen/Dockerfile.release", "seed", RELEASE_STATIC_BASE),
        ("apps/lumen/Dockerfile.test", "seed", STATIC_BASE),
    ];
    for (rel, seed, base) in targets {
        let content = std::fs::read_to_string(root.join(rel)).unwrap();
        valid(&content, seed, base).unwrap();
    }
    for (v, seed, base) in [
        ("source", "builder", CC_BASE),
        ("release", "seed", RELEASE_STATIC_BASE),
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_lumen"))
            .args(["dockerfile", "render", "--variant", v])
            .output()
            .unwrap();
        assert!(out.status.success());
        valid(&String::from_utf8_lossy(&out.stdout), seed, base).unwrap();
    }
}

#[test]
fn test_release_dockerfile_rejects_mutable_final_base() {
    let root = repo_root();
    let content = std::fs::read_to_string(root.join("apps/lumen/Dockerfile.release")).unwrap();
    let mutated = one(&content, RELEASE_STATIC_BASE, STATIC_BASE);
    assert_eq!(
        valid(&mutated, "seed", RELEASE_STATIC_BASE),
        Err("final-base")
    );
}

const FIXTURE: &str = "\
FROM debian:bookworm-slim AS builder\n\
RUN mkdir -p -m 0750 /out/lumen-data && touch /out/lumen-data/.lumen-volume-seed\n\
FROM gcr.io/distroless/cc-debian12:nonroot\n\
COPY --from=builder --chown=65532:65532 /out/lumen-data/ /var/lib/lumen/data/\n\
ENV LUMEN_HOST=0.0.0.0\n\
ENV LUMEN_DATA_DIR=/var/lib/lumen/data\n\
ENV LUMEN_PERSISTENCE=segment\n\
ENV LUMEN_WAL=embedded\n\
VOLUME [\"/var/lib/lumen/data\"]\n\
ENTRYPOINT [\"/usr/local/bin/lumen\"]\n";

#[test]
fn test_source_dockerfile_negative_mutations() {
    enum M {
        R(&'static str, &'static str),
        C(&'static str, &'static str),
        F(&'static str),
        S(&'static str),
        Missing,
        Builder,
        Duplicate,
        Late,
    }
    use M::*;
    const SRC: &str = "/out/lumen-data/";
    const DST: &str = "/var/lib/lumen/data/";
    const OWN: &str = "--chown=65532:65532";
    const ORDER: &str = "--chown=65532:65532 /out/lumen-data/";
    const MOVED: &str = "/out/lumen-data/ --chown=65532:65532";
    const ALT: &str = "COPY /out/other/ /var/lib/lumen/data/\n";
    const SEED: &str =
        "RUN mkdir -p -m 0750 /out/lumen-data && touch /out/lumen-data/.lumen-volume-seed";
    let cases = [
        (R("0750", "0777"), "seed-layout"),
        (R(".lumen-volume-seed", ".other"), "seed-layout"),
        (R(SEED, "RUN touch /out/other/.seed"), "seed-layout"),
        (C("--from=builder", "--from=wrong"), "copy-source"),
        (C(SRC, "/out/other/"), "copy-source"),
        (C(DST, "/var/lib/other/"), "copy-destination"),
        (C(OWN, "--chown=0:0"), "copy-flags"),
        (C("--chown=65532:65532 ", ""), "copy-flags"),
        (C(OWN, "--chown=65532:65532 --chmod=0000"), "copy-flags"),
        (C(ORDER, MOVED), "copy-flags"),
        (C(DST, "/var/lib/lumen/data/ /extra/"), "copy-source"),
        (R(CC_BASE, "debian:bookworm-slim"), "final-base"),
        (F("ENV LUMEN_DATA_DIR=bad"), "duplicate-image-default"),
        (F("ENV LUMEN_PERSISTENCE=bad"), "duplicate-image-default"),
        (F("ENV LUMEN_WAL=bad"), "duplicate-image-default"),
        (F("USER root"), "root-user"),
        (F("USER root:root"), "root-user"),
        (F("USER 0"), "root-user"),
        (F("USER 0:0"), "root-user"),
        (F("USER 0:65532"), "root-user"),
        (F("USER 12345"), "root-user"),
        (F("VOLUME /data"), "duplicate-image-default"),
        (S("VOLUME /out/d"), "duplicate-image-default"),
        (S("volume \\\n /out/d"), "duplicate-image-default"),
        (Missing, "missing-copy"),
        (Builder, "builder-only-copy"),
        (Duplicate, "duplicate-copy"),
        (Late, "copy-order"),
    ];
    for (mutation, why) in cases {
        let no_copy = || one(FIXTURE, COPY, "");
        let content = match mutation {
            R(from, to) => one(FIXTURE, from, to),
            C(from, to) => one(FIXTURE, COPY, &COPY.replace(from, to)),
            F(line) => one(FIXTURE, "ENTRYPOINT", &format!("{line}\nENTRYPOINT")),
            S(line) => one(FIXTURE, "AS builder\n", &format!("AS builder\n{line}\n")),
            Missing => no_copy(),
            Builder => one(&no_copy(), "AS builder\n", &format!("AS builder\n{COPY}")),
            Duplicate => one(FIXTURE, COPY, &format!("{COPY}{ALT}")),
            Late => one(&no_copy(), "ENTRYPOINT", &format!("ENTRYPOINT\n{COPY}")),
        };
        assert_eq!(valid(&content, "builder", CC_BASE), Err(why));
    }

    for (exact, wrong) in [
        ("ENV LUMEN_HOST=0.0.0.0", "ENV LUMEN_HOST=127.0.0.1"),
        (
            "ENV LUMEN_DATA_DIR=/var/lib/lumen/data",
            "ENV LUMEN_DATA_DIR=/data",
        ),
        (
            "ENV LUMEN_PERSISTENCE=segment",
            "ENV LUMEN_PERSISTENCE=cbor",
        ),
        ("ENV LUMEN_WAL=embedded", "ENV LUMEN_WAL=auto"),
        ("VOLUME [\"/var/lib/lumen/data\"]", "VOLUME [\"/data\"]"),
    ] {
        let exact_line = format!("{exact}\n");
        let missing = one(FIXTURE, &exact_line, "");
        assert_eq!(
            valid(&missing, "builder", CC_BASE),
            Err("missing-image-default")
        );
        assert_eq!(
            valid(
                &one(FIXTURE, &exact_line, &format!("{exact}\n{exact}\n")),
                "builder",
                CC_BASE,
            ),
            Err("duplicate-image-default")
        );
        assert_eq!(
            valid(&one(FIXTURE, exact, wrong), "builder", CC_BASE),
            Err("invalid-image-default")
        );
        assert_eq!(
            valid(
                &one(
                    &missing,
                    "ENTRYPOINT [\"/usr/local/bin/lumen\"]\n",
                    &format!("ENTRYPOINT [\"/usr/local/bin/lumen\"]\n{exact}\n"),
                ),
                "builder",
                CC_BASE,
            ),
            Err("late-image-default")
        );
        assert_eq!(
            valid(
                &one(
                    &missing,
                    "FROM debian:bookworm-slim AS builder\n",
                    &format!("FROM debian:bookworm-slim AS builder\n{exact}\n"),
                ),
                "builder",
                CC_BASE,
            ),
            Err("builder-only-image-default")
        );
    }
    assert_eq!(
        valid(
            &one(
                FIXTURE,
                "ENTRYPOINT [\"/usr/local/bin/lumen\"]\n",
                "ENV LUMEN_FSYNC=always\nENTRYPOINT [\"/usr/local/bin/lumen\"]\n",
            ),
            "builder",
            CC_BASE,
        ),
        Err("fsync-knob")
    );
}

fn docker(args: &[&str]) -> Result<String, String> {
    let out = Command::new("docker")
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!(
            "docker {args:?} failed ({}): {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn logs(name: &str) -> String {
    match Command::new("docker").args(["logs", name]).output() {
        Ok(out) => format!(
            "STDOUT:\n{}\nSTDERR:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
        Err(err) => format!("read logs: {err}"),
    }
}

fn need(ok: bool, name: &str, message: String) {
    if !ok {
        panic!("{message}\nlogs:\n{}", logs(name));
    }
}

fn is_not_found(kind: &str, stderr: &[u8]) -> bool {
    let text = String::from_utf8_lossy(stderr).to_ascii_lowercase();
    matches!(kind, "container" | "image" | "volume") && text.contains(&format!("no such {kind}"))
}

fn label(kind: &str, target: &str) -> Result<Option<String>, String> {
    let fmt = if kind == "volume" {
        "{{index .Labels \"lumen.run_id\"}}"
    } else {
        "{{index .Config.Labels \"lumen.run_id\"}}"
    };
    let out = Command::new("docker")
        .args([kind, "inspect", "--format", fmt, target])
        .output()
        .map_err(|e| format!("inspect {kind} {target}: {e}"))?;
    if !out.status.success() {
        if is_not_found(kind, &out.stderr) {
            return Ok(None);
        }
        return Err(format!(
            "inspect {kind} {target}: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(Some(
        String::from_utf8_lossy(&out.stdout).trim().to_string(),
    ))
}

fn assert_absent(kind: &str, name: &str) {
    assert_eq!(
        label(kind, name),
        Ok(None),
        "{kind} {name} already exists or cannot be inspected"
    );
}

struct Guard {
    run_id: String,
    items: Vec<(&'static str, String)>,
}

impl Guard {
    fn new(run_id: String) -> Self {
        Self {
            run_id,
            items: Vec::new(),
        }
    }

    fn cleanup(&self) -> Result<(), String> {
        let mut errs = Vec::new();
        for (kind, name) in self.items.iter().rev() {
            match label(kind, name) {
                Ok(Some(label)) if label == self.run_id => {
                    if *kind == "container" {
                        let _ = Command::new("docker")
                            .args(["container", "stop", "--time=1", name])
                            .output();
                    }
                    match Command::new("docker")
                        .args([kind, "rm", "-f", name])
                        .output()
                    {
                        Ok(out) if out.status.success() || is_not_found(kind, &out.stderr) => {}
                        Ok(out) => errs.push(format!(
                            "rm {kind} {name}: {}",
                            String::from_utf8_lossy(&out.stderr)
                        )),
                        Err(err) => errs.push(format!("rm {kind} {name}: {err}")),
                    }
                }
                Ok(None) => {}
                Ok(Some(other)) => errs.push(format!(
                    "refusing to clean {kind} {name}: label mismatch {other:?}"
                )),
                Err(e) => errs.push(format!("label check {kind} {name}: {e}")),
            }
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs.join("; "))
        }
    }

    fn verify_cleaned(&self) {
        for (kind, name) in &self.items {
            assert_eq!(
                label(kind, name),
                Ok(None),
                "{kind} {name} still exists or inspection failed"
            );
        }
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

const COMMON_ENV: [&str; 4] = ["-e", "LUMEN_AUTH=off", "-e", "LUMEN_GRACE_SECS=1"];

fn candidate_image() -> String {
    let image = std::env::var("LUMEN_DOCKER_CANDIDATE_IMAGE")
        .expect("set LUMEN_DOCKER_CANDIDATE_IMAGE to the verified candidate digest");
    validate_candidate_image(&image).unwrap_or_else(|reason| panic!("{reason}: {image}"));
    image
}

fn validate_candidate_image(image: &str) -> Result<(), &'static str> {
    let digest = image
        .strip_prefix("ghcr.io/chrischeng-c4/lumen@sha256:")
        .ok_or("candidate image must use canonical GHCR repository and @sha256 digest")?;
    if digest.len() != 64 {
        return Err("candidate sha256 must contain 64 hex digits");
    }
    if !digest
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("candidate sha256 must be lowercase hexadecimal");
    }
    Ok(())
}

#[test]
fn live_gate_accepts_only_the_exact_candidate_digest_shape() {
    let valid = format!(
        "ghcr.io/chrischeng-c4/lumen@sha256:{}",
        "0123456789abcdef".repeat(4)
    );
    assert_eq!(validate_candidate_image(&valid), Ok(()));
    for invalid in [
        "ghcr.io/chrischeng-c4/lumen:0.4.29".to_owned(),
        "ghcr.io/other/lumen@sha256:".to_owned() + &"0".repeat(64),
        "ghcr.io/chrischeng-c4/lumen@sha256:abc".to_owned(),
        format!("ghcr.io/chrischeng-c4/lumen@sha256:{}", "A".repeat(64)),
        format!("ghcr.io/chrischeng-c4/lumen@sha256:{}", "g".repeat(64)),
    ] {
        assert!(
            validate_candidate_image(&invalid).is_err(),
            "invalid candidate accepted: {invalid}"
        );
    }
}

async fn start(
    guard: &mut Guard,
    client: &Client,
    name: &str,
    img: &str,
    vol: Option<&str>,
) -> (String, u16) {
    assert_absent("container", name);
    let run_label = format!("lumen.run_id={}", guard.run_id);
    let mut args = vec![
        "run",
        "-d",
        "--name",
        name,
        "--label",
        &run_label,
        "-p",
        "127.0.0.1::7373",
    ];
    args.extend(COMMON_ENV);
    let mount_arg;
    if let Some(v) = vol {
        mount_arg = format!("type=volume,src={v},dst=/var/lib/lumen/data");
        args.extend(["--mount", &mount_arg]);
    }
    args.push(img);
    let cid = docker(&args).unwrap_or_else(|e| panic!("run {name}: {e}"));
    guard.items.push(("container", cid.clone()));
    let mapping = docker(&["port", &cid, "7373"]).unwrap_or_else(|e| panic!("port {name}: {e}"));
    let port: u16 = mapping
        .strip_prefix("127.0.0.1:")
        .filter(|p| !p.contains('\n'))
        .expect("one exact 127.0.0.1 mapping")
        .parse()
        .expect("valid port");
    let url = format!("http://127.0.0.1:{port}/healthz");
    let (start, timeout) = (Instant::now(), Duration::from_secs(15));
    while start.elapsed() < timeout {
        if client
            .get(&url)
            .send()
            .await
            .is_ok_and(|r| r.status().is_success())
        {
            return (cid, port);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("container {name} health timeout\nlogs:\n{}", logs(&cid));
}

async fn post(
    client: &Client,
    cid: &str,
    path: &str,
    port: u16,
    body: Value,
) -> (StatusCode, Value) {
    let url = format!("http://127.0.0.1:{port}{path}");
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .unwrap_or_else(|e| panic!("post {url}: {e}\nlogs:\n{}", logs(cid)));
    let status = resp.status();
    let bytes = resp.bytes().await.unwrap_or_else(|e| {
        panic!(
            "read {url} {status}: body=<unavailable>; error={e}\nlogs:\n{}",
            logs(cid)
        )
    });
    let json = serde_json::from_slice(&bytes).unwrap_or_else(|e| {
        panic!(
            "json {url} {status}: body={:?}; error={e}\nlogs:\n{}",
            String::from_utf8_lossy(&bytes),
            logs(cid)
        )
    });
    (status, json)
}

type S<'a> = (
    (&'a str, Option<&'a str>),
    (&'a str, &'a str),
    (Option<&'a str>, Option<&'a str>),
);
type L<'a> = (
    ([&'a str; 2], Option<&'a str>),
    (&'a str, &'a str, &'a str),
    bool,
    StopMode,
);

#[derive(Clone, Copy)]
enum StopMode {
    Graceful,
    HardKill,
}

async fn step(
    guard: &mut Guard,
    client: &Client,
    img: &str,
    s: S<'_>,
    stop_mode: StopMode,
) -> String {
    let ((name, vol), (coll, value), (write_id, expected_id)) = s;
    let (id, port) = start(guard, client, name, img, vol).await;
    if let Some(doc_id) = write_id {
        let put = client
            .put(format!("http://127.0.0.1:{port}/collections/{coll}"))
            .json(&json!({ "fields": { "k": { "type": "keyword" } } }))
            .send()
            .await
            .unwrap_or_else(|e| panic!("put {coll}: {e}\nlogs:\n{}", logs(&id)));
        let ps = put.status();
        need(ps.is_success(), &id, format!("put {coll}: {ps}"));
        let (st, _) = post(
            client,
            &id,
            &format!("/collections/{coll}/index"),
            port,
            json!({ "items": [{ "external_id": doc_id, "field": "k", "value": value }] }),
        )
        .await;
        need(st.is_success(), &id, format!("index {coll}: {st}"));
    }
    let (st, body) = post(
        client,
        &id,
        &format!("/collections/{coll}/search"),
        port,
        json!({ "query": { "term": { "field": "k", "value": value } }, "limit": 10 }),
    )
    .await;
    let hit = body
        .pointer("/hits/0/external_id")
        .and_then(|value| value.as_str());
    let total = body.get("total").and_then(Value::as_u64);
    if let Some(expected) = expected_id {
        need(
            st.is_success() && hit == Some(expected) && total == Some(1),
            &id,
            format!("search {st} hit={hit:?} total={total:?}"),
        );
    } else {
        need(
            st.as_u16() == 404,
            &id,
            format!("search expected 404, got {st}"),
        );
    }
    let stop = match stop_mode {
        StopMode::Graceful => Command::new("docker")
            .args(["stop", "--time=5", &id])
            .output()
            .expect("docker stop"),
        StopMode::HardKill => Command::new("docker")
            .args(["kill", "--signal=KILL", &id])
            .output()
            .expect("docker kill"),
    };
    need(
        stop.status.success(),
        &id,
        format!("terminate {name} failed"),
    );
    let code =
        docker(&["inspect", "--format", "{{.State.ExitCode}}", &id]).expect("inspect exit code");
    let expected_code = match stop_mode {
        StopMode::Graceful => "0",
        StopMode::HardKill => "137",
    };
    need(
        code == expected_code,
        &id,
        format!("{name} exit code {code}, expected {expected_code}"),
    );
    let rm = Command::new("docker")
        .args(["rm", &id])
        .output()
        .expect("docker rm");
    let rm_err = String::from_utf8_lossy(&rm.stderr);
    need(rm.status.success(), &id, format!("rm {name}: {rm_err}"));
    id
}

async fn pair(guard: &mut Guard, client: &Client, img: &str, lane: L<'_>) {
    let ((suffixes, vol), (coll, doc_id, value), survives, first_stop) = lane;
    let names = suffixes.map(|suffix| format!("{}-{suffix}", guard.run_id));
    let a = (
        (&*names[0], vol),
        (coll, value),
        (Some(doc_id), Some(doc_id)),
    );
    let b = (
        (&*names[1], vol),
        (coll, value),
        (None, survives.then_some(doc_id)),
    );
    let first = step(guard, client, img, a, first_stop).await;
    let second = step(guard, client, img, b, StopMode::Graceful).await;
    assert_ne!(
        first, second,
        "replacement container must have a distinct ID"
    );
}

#[ignore]
#[tokio::test]
async fn digest_bound_volume_survives_replacement_and_hard_kill_but_anonymous_volume_does_not() {
    docker(&["info"]).expect("docker unavailable");
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let run_id = format!("lumen-test-{epoch}");
    let mut guard = Guard::new(run_id.clone());
    let image = candidate_image();
    let run_label = format!("lumen.run_id={run_id}");
    let user = docker(&["inspect", "--format", "{{.Config.User}}", &image]).expect("inspect user");
    assert!(
        matches!(user.as_str(), "65532" | "65532:65532"),
        "user must be 65532, got {user:?}"
    );
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::CONNECTION, "close".parse().unwrap());
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .default_headers(headers)
        .no_proxy()
        .build()
        .expect("client");
    let vol = format!("{run_id}-vol");
    assert_absent("volume", &vol);
    docker(&["volume", "create", "--label", &run_label, &vol]).expect("create volume");
    guard.items.push(("volume", vol.clone()));
    let (coll_p, id_p, val_p) = ("coll_p", "doc_p", format!("val_p_{run_id}"));
    let persistent = (
        (["ca", "cb"], Some(&*vol)),
        (coll_p, id_p, &*val_p),
        true,
        StopMode::Graceful,
    );
    pair(&mut guard, &client, &image, persistent).await;
    let (coll_k, id_k, val_k) = ("coll_k", "doc_k", format!("val_k_{run_id}"));
    let hard_kill = (
        (["ce", "cf"], Some(&*vol)),
        (coll_k, id_k, &*val_k),
        true,
        StopMode::HardKill,
    );
    pair(&mut guard, &client, &image, hard_kill).await;
    let (coll_e, id_e, val_e) = ("coll_e", "doc_e", format!("val_e_{run_id}"));
    let ephemeral = (
        (["cc", "cd"], None),
        (coll_e, id_e, &*val_e),
        false,
        StopMode::Graceful,
    );
    pair(&mut guard, &client, &image, ephemeral).await;
    guard.cleanup().expect("cleanup must succeed");
    guard.verify_cleaned();
}

struct ComposeGuard {
    file: PathBuf,
    project: String,
}

impl ComposeGuard {
    fn command(&self, args: &[&str]) -> std::process::Output {
        Command::new("docker")
            .args(["compose", "--project-name", &self.project, "--file"])
            .arg(&self.file)
            .args(args)
            .output()
            .expect("run docker compose")
    }

    fn success(&self, args: &[&str]) -> String {
        let output = self.command(args);
        assert!(
            output.status.success(),
            "docker compose {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    }
}

impl Drop for ComposeGuard {
    fn drop(&mut self) {
        let _ = self.command(&["down", "--volumes", "--remove-orphans"]);
    }
}

fn yaml_key(value: &str) -> YamlValue {
    YamlValue::String(value.into())
}

fn compose_volumes(project: &str) -> Vec<String> {
    let project_filter = format!("label=com.docker.compose.project={project}");
    let output = Command::new("docker")
        .args([
            "volume",
            "ls",
            "--filter",
            &project_filter,
            "--filter",
            "label=com.docker.compose.volume=lumen-data",
            "--format",
            "{{.Name}}",
        ])
        .output()
        .expect("list Compose volumes");
    assert!(
        output.status.success(),
        "list Compose volumes: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

fn set_compose_candidate_and_run_labels(file: &std::path::Path, image: &str, run_id: &str) {
    let mut root: YamlValue = serde_yaml::from_slice(&fs::read(file).unwrap()).unwrap();
    let root = root.as_mapping_mut().unwrap();
    let services = root
        .get_mut(&yaml_key("services"))
        .and_then(YamlValue::as_mapping_mut)
        .unwrap();
    let service = services
        .get_mut(&yaml_key("lumen"))
        .and_then(YamlValue::as_mapping_mut)
        .unwrap();
    service.insert(yaml_key("image"), yaml_key(image));
    let labels = service
        .get_mut(&yaml_key("labels"))
        .and_then(YamlValue::as_mapping_mut)
        .unwrap();
    labels.insert(yaml_key("lumen.run_id"), yaml_key(run_id));
    let environment = service
        .get_mut(&yaml_key("environment"))
        .and_then(YamlValue::as_mapping_mut)
        .unwrap();
    environment.insert(yaml_key("LUMEN_GRACE_SECS"), yaml_key("1"));

    let volumes = root
        .get_mut(&yaml_key("volumes"))
        .and_then(YamlValue::as_mapping_mut)
        .unwrap();
    let volume = volumes
        .get_mut(&yaml_key("lumen-data"))
        .and_then(YamlValue::as_mapping_mut)
        .unwrap();
    let mut volume_labels = YamlMapping::new();
    volume_labels.insert(yaml_key("lumen.run_id"), yaml_key(run_id));
    volume.insert(yaml_key("labels"), YamlValue::Mapping(volume_labels));
    fs::write(file, serde_yaml::to_string(&root).unwrap()).unwrap();
}

async fn wait_fixed_compose_port(client: &Client, cid: &str) {
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(15) {
        if client
            .get("http://127.0.0.1:7373/healthz")
            .send()
            .await
            .is_ok_and(|response| response.status().is_success())
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("Compose container health timeout\n{}", logs(cid));
}

#[ignore]
#[tokio::test]
async fn digest_bound_compose_down_keeps_volume_and_down_v_deletes_it() {
    docker(&["info"]).expect("docker unavailable");
    let image = candidate_image();
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let run_id = format!("lumen-compose-test-{epoch}");
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("compose.yaml");
    let patch = Command::new(env!("CARGO_BIN_EXE_lumen"))
        .args(["standalone", "compose", "patch", "--file"])
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        patch.status.success(),
        "patch failed: {}",
        String::from_utf8_lossy(&patch.stderr)
    );
    set_compose_candidate_and_run_labels(&file, &image, &run_id);
    let compose = ComposeGuard {
        file,
        project: run_id.clone(),
    };
    assert!(compose_volumes(&run_id).is_empty());

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .no_proxy()
        .build()
        .unwrap();
    compose.success(&["up", "-d"]);
    let first = compose.success(&["ps", "--quiet", "lumen"]);
    let volumes = compose_volumes(&run_id);
    assert_eq!(volumes.len(), 1, "expected one Lumen Compose volume");
    let volume = volumes[0].clone();
    assert_eq!(label("container", &first), Ok(Some(run_id.clone())));
    assert_eq!(label("volume", &volume), Ok(Some(run_id.clone())));
    wait_fixed_compose_port(&client, &first).await;
    let put = client
        .put("http://127.0.0.1:7373/collections/compose_durable")
        .json(&json!({"fields":{"k":{"type":"keyword"}}}))
        .send()
        .await
        .unwrap();
    assert!(put.status().is_success());
    let (status, _) = post(
        &client,
        &first,
        "/collections/compose_durable/index",
        7373,
        json!({"items":[{"external_id":"doc","field":"k","value":"survives"}]}),
    )
    .await;
    assert!(status.is_success());

    compose.success(&["down"]);
    assert_eq!(label("volume", &volume), Ok(Some(run_id.clone())));
    compose.success(&["up", "-d"]);
    let second = compose.success(&["ps", "--quiet", "lumen"]);
    assert_ne!(first, second);
    wait_fixed_compose_port(&client, &second).await;
    let (status, body) = post(
        &client,
        &second,
        "/collections/compose_durable/search",
        7373,
        json!({"query":{"term":{"field":"k","value":"survives"}},"limit":10}),
    )
    .await;
    assert!(status.is_success(), "search after down: {status} {body}");
    assert_eq!(body["total"], 1);

    compose.success(&["down", "-v"]);
    assert_eq!(label("volume", &volume), Ok(None));
    compose.success(&["up", "-d"]);
    let third = compose.success(&["ps", "--quiet", "lumen"]);
    wait_fixed_compose_port(&client, &third).await;
    let (status, _) = post(
        &client,
        &third,
        "/collections/compose_durable/search",
        7373,
        json!({"query":{"term":{"field":"k","value":"survives"}},"limit":10}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
