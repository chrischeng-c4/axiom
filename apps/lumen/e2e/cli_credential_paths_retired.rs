// HANDWRITE gap="missing-generator:e2e-test:cli-credential-retirement" tracker="2873" reason="Proving a path was deleted needs a scan of the shipped surface plus a live run under a fake kubectl; no generator emits either."
//! #2873: the CLI carries no credential.
//!
//! The three paths this file exists to keep deleted are the ones a caller
//! could reach without asking for them:
//!
//! 1. a flag that took a credential on the command line — visible in `ps`,
//!    in shell history, and in the CronJob's own `kubectl describe`;
//! 2. an environment variable read behind the flag's back, which every
//!    descendant of a wrapped command inherited;
//! 3. a `kubectl get secret` lookup behind *that*, which turned "run one
//!    query" into "hand me the fleet's shared credential".
//!
//! Each is asserted twice on purpose: once as absence from the surface a
//! caller reads (AC1, AC3) and once as behaviour of the running binary
//! (AC2, AC4, AC5). Absence alone is a grep that a future refactor can
//! satisfy while re-growing the path under a new name; behaviour alone
//! passes on a build whose help text still advertises the flag.
//!
//! Scope note: this gate reads `apps/lumen` only, and only the surface that
//! ships — source, examples, docs, manifests, generated clients, README. The
//! generated half is not a formality: the first thing this scan caught, once
//! widened, was a committed `clients/openapi.json` still publishing the
//! retired header recipe. The negative assertions living
//! in other tests (`spec_cli.rs`, `operator_render.rs`,
//! `operator_retired_credential_projection.rs`) name the retired strings on
//! purpose, so widening the scan to `e2e/` would make the gate fail on
//! its own siblings. #2875 owns the repository-wide residue gate and the
//! historical-evidence carve-out that requires.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(feature = "backup")]
use std::io::{BufRead, BufReader, Write};
#[cfg(feature = "backup")]
use std::net::{TcpListener, TcpStream};

/// Assembled at runtime so this file does not itself contain the string it
/// forbids — a residue gate that matches its own source is a gate nobody can
/// keep green.
fn needle(parts: &[&str]) -> String {
    parts.concat()
}

fn lumen_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every shipped text file under `apps/lumen`, excluding `e2e/` (see the
/// scope note above), `target/`, and anything not human-readable.
///
/// `clients/` earns its place on this list the hard way: `clients/openapi.json`
/// is a committed snapshot of `lumen spec --format openapi`, and it went on
/// telling readers to send `Authorization: Bearer <LUMEN_TOKEN>` for as long as
/// nobody regenerated it. A generated file is still a shipped file, and a stale
/// one is the most persuasive kind of wrong documentation — it looks
/// machine-produced. `external-contracts/` is deliberately absent: AC1 exempts
/// historical evidence, and evidence that no longer records what was actually
/// rejected is worthless.
fn shipped_files() -> Vec<PathBuf> {
    let root = lumen_dir();
    let mut out = Vec::new();
    for sub in [
        "src",
        "examples",
        "docs",
        "k8s",
        "tech-design",
        "clients",
        "scripts",
        "observability",
        "benches",
    ] {
        collect(&root.join(sub), &mut out);
    }
    for file in [
        "README.md",
        "CONTRIBUTING.md",
        "Cargo.toml",
        "aw.toml",
        "vat.toml",
        "compose.yaml",
        "llms.txt",
        "install.sh",
        "build.sh",
        "Dockerfile",
        "Dockerfile.release",
        "Dockerfile.test",
    ] {
        let path = root.join(file);
        if path.is_file() {
            out.push(path);
        }
    }
    out
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("rs" | "md" | "yaml" | "yml" | "toml" | "json" | "sh" | "txt")
        ) {
            out.push(path);
        }
    }
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

/// AC1: no credential environment variable, no Google instance-identity
/// endpoint, and no Application Default Credentials handoff anywhere in the
/// surface `apps/lumen` ships.
///
/// These are spelled as exact strings rather than as a fuzzy "token" search
/// because `apps/lumen` legitimately contains the word: the GCS *destination
/// sink* in the storage topic authenticates to Google Cloud Storage, which is
/// a backup target, not a Lumen request identity. Conflating the two is how a
/// residue gate ends up either useless or permanently red.
#[test]
fn no_credential_env_var_or_google_identity_source_ships_in_lumen() {
    let forbidden = [
        needle(&["LUMEN_", "TOKEN"]),
        needle(&["LUMEN_BACKUP_", "TOKEN"]),
        needle(&["metadata.google", ".internal"]),
        needle(&["computeMeta", "data"]),
        needle(&["GOOGLE_APPLICATION_", "CREDENTIALS"]),
        needle(&["gcloud auth print-access-", "token"]),
    ];
    let mut hits = Vec::new();
    for path in shipped_files() {
        let body = read(&path);
        for (line_no, line) in body.lines().enumerate() {
            for f in &forbidden {
                if line.contains(f.as_str()) {
                    hits.push(format!(
                        "{}:{}: {}",
                        path.display(),
                        line_no + 1,
                        line.trim()
                    ));
                }
            }
        }
    }
    assert!(
        hits.is_empty(),
        "#2873 AC1: a retired credential source is still named in shipped lumen surface:\n{}",
        hits.join("\n")
    );
}

/// AC1, second half: the CLI binary has no way to *discover* a credential.
///
/// This assertion changed shape in #2878 and it is worth saying why, because
/// the change looks from a distance like a gate being relaxed.
///
/// #2873's version forbade attaching an identity at all — no header literal,
/// no `bearer_auth` call. That was the right assertion while the CLI had no
/// legitimate way to obtain one: any credential it could attach was a
/// credential it had found lying around. #2878 gives it a legitimate way, and
/// the property worth defending is no longer "attaches nothing" but "attaches
/// only what it just minted, for an account the caller named".
///
/// So the discovery paths stay banned outright, and the attachment is pinned
/// instead of forbidden: every call site must take the value from the `token`
/// parameter, whose type only a `TokenSource` can produce. A future edit that
/// reads a credential from anywhere else has to either name a banned symbol or
/// change the pinned call form — both visible in review, neither silent.
#[test]
fn the_cli_binary_can_only_attach_a_credential_it_minted() {
    let body = read(&lumen_dir().join("src/bin/lumen.rs"));

    // The discovery paths. Each of these found a credential somebody else had
    // stored; none may come back under any name.
    let forbidden = [
        needle(&["\"Authoriz", "ation\""]),
        needle(&["AUTHORIZ", "ATION"]),
        needle(&["resolve_token", "("]),
        needle(&["resolve_cr_tokens_secret", "("]),
        needle(&["\"get\", \"secret", "\""]),
    ];
    for f in &forbidden {
        assert!(
            !body.contains(f.as_str()),
            "#2873 AC1: `src/bin/lumen.rs` still contains `{f}` — the CLI must have no way to \
             find a request credential it did not mint"
        );
    }

    // The attachment. Pinned to the one legal form: the minted token, arriving
    // as a parameter.
    let attach = needle(&["bearer_", "auth("]);
    let legal = format!("{attach}token.expose())");
    let mut sites = 0usize;
    let mut from = 0usize;
    while let Some(rel) = body[from..].find(&attach) {
        let at = from + rel;
        sites += 1;
        assert!(
            body[at..].starts_with(&legal),
            "#2878 AC1: `src/bin/lumen.rs` attaches a credential at byte {at} in a form other \
             than `{legal}`; the only value the CLI may send is the one it minted through \
             TokenRequest:\n{}",
            body[at..].lines().next().unwrap_or_default()
        );
        from = at + attach.len();
    }
    assert!(
        sites > 0,
        "#2878 AC1: `src/bin/lumen.rs` no longer attaches a minted credential at all — if the \
         TokenRequest path was removed, this gate and #2878's tests should go with it rather \
         than passing vacuously"
    );

    // R3: the account is named per invocation. An `env =` on this flag would
    // make "which identity am I acting as" ambient again, which is the same
    // defect #2873 removed wearing different clothes.
    let flag = needle(&["client_", "sa: Option<String>"]);
    let decl = body
        .find(&flag)
        .unwrap_or_else(|| panic!("#2878 R3: `src/bin/lumen.rs` declares no `{flag}` field"));
    // The doc comment above the flag is prose with em dashes in it, so a fixed
    // byte offset can land inside a character; walk forward to a boundary.
    let mut start = decl.saturating_sub(400);
    while start < decl && !body.is_char_boundary(start) {
        start += 1;
    }
    let window = &body[start..decl];
    assert!(
        !window.contains("env ="),
        "#2878 R3: a `--client-sa` flag with an environment fallback lets the caller act as an \
         account nobody named in the command:\n{window}"
    );
}

// ---------------------------------------------------------------------------
// AC3 — nothing in the shipped agent-facing text tells a caller to send a
// Google credential to Lumen.
// ---------------------------------------------------------------------------

fn lumen_output(args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_lumen"))
        .args(args)
        .output()
        .unwrap_or_else(|err| panic!("run lumen {args:?}: {err}"));
    let mut combined = String::from_utf8_lossy(&out.stdout).into_owned();
    combined.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.success(), combined)
}

/// AC3: every `lumen llm` topic and every `--help` page is free of an
/// instruction to present Application Default Credentials, a Google access
/// token, or a Google ID token to Lumen — and free of the retired credential
/// flag, which is the form such an instruction always took.
#[test]
fn no_llm_topic_or_help_page_directs_a_google_credential_at_lumen() {
    const HELP: [&[&str]; 12] = [
        &["--help"],
        &["query", "--help"],
        &["query", "collections", "--help"],
        &["query", "collections", "list", "--help"],
        &["query", "search", "--help"],
        &["connect", "--help"],
        &["backup", "--help"],
        &["dump", "--help"],
        &["load", "--help"],
        &["spec", "--help"],
        &["llm", "--help"],
        &["k8s", "--help"],
    ];
    let forbidden = [
        needle(&["LUMEN_", "TOKEN"]),
        needle(&["gcloud auth print-access-", "token"]),
        needle(&["GOOGLE_APPLICATION_", "CREDENTIALS"]),
        needle(&["metadata.google", ".internal"]),
        needle(&["token-", "registry"]),
    ];

    let mut pages: Vec<(String, String)> = Vec::new();
    let (ok, outline) = lumen_output(&["llm", "--topic", "outline", "--format", "json"]);
    assert!(ok, "`lumen llm --topic outline` failed:\n{outline}");
    let manifest: serde_json::Value =
        serde_json::from_str(&outline).expect("LLM outline JSON parses");
    let mut topics = vec!["outline".to_string()];
    topics.extend(
        manifest["tasks"]
            .as_array()
            .expect("LLM outline has tasks")
            .iter()
            .map(|task| {
                task["topic"]
                    .as_str()
                    .expect("LLM task topic is a string")
                    .to_string()
            }),
    );
    for topic in topics {
        let (ok, text) = lumen_output(&["llm", "--topic", &topic]);
        assert!(ok, "`lumen llm --topic {topic}` failed:\n{text}");
        pages.push((format!("llm --topic {topic}"), text));
    }
    for args in HELP {
        let (_, text) = lumen_output(args);
        pages.push((format!("lumen {}", args.join(" ")), text));
    }

    let bare_token_flag = needle(&["--", "token"]);
    for (label, text) in &pages {
        for f in &forbidden {
            assert!(
                !text.contains(f.as_str()),
                "#2873 AC3: `{label}` still names `{f}`:\n{text}"
            );
        }
        // The flag itself, not every flag that starts with it. `--token-file`
        // (#2877) takes a *path* to a projected ServiceAccount token; `--token`
        // takes the material. The difference is the whole point — an argument
        // is visible in the pod spec, in `ps`, and in shell history — so the
        // one safe spelling is allowed by name and everything else that starts
        // `--token` is refused, including a bare `--token` and any new suffix
        // nobody reviewed.
        let mut from = 0;
        while let Some(rel) = text[from..].find(&bare_token_flag) {
            let end = from + rel + bare_token_flag.len();
            assert!(
                text[end..].starts_with("-file"),
                "#2873 AC3: `{label}` names a `{bare_token_flag}…` flag that is not \
                 `{bare_token_flag}-file`, the only spelling that takes a path instead of \
                 a credential:\n{text}"
            );
            from = end;
        }
    }
}

// ---------------------------------------------------------------------------
// AC2 — `lumen connect` hands its child a URL and nothing else, and reaches
// Kubernetes exactly once, for the port-forward.
// ---------------------------------------------------------------------------

/// A canary the real code has no way to obtain. It is served by the fake
/// `kubectl` for every invocation *except* `port-forward`, so if the CLI ever
/// re-grows a Secret lookup the canary lands in the child's environment and
/// the assertions below fail with the exact string that leaked.
#[allow(dead_code)]
const KUBECTL_CANARY: &str = "canary-fleet-credential-2873";

/// Writes a fake `kubectl` into `dir` that logs every invocation, serves the
/// canary to any lookup, and — for `port-forward` — binds the requested local
/// port so the CLI's readiness probe succeeds.
#[cfg(unix)]
fn install_fake_kubectl(dir: &Path, log: &Path, python: &str) {
    let script = format!(
        r#"#!/bin/sh
printf '%s\n' "$*" >> "{log}"
for a in "$@"; do
  if [ "$a" = "port-forward" ]; then
    for last in "$@"; do :; done
    port=${{last%%:*}}
    exec {python} -c '
import socket, sys, time
s = socket.socket()
s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(("127.0.0.1", int(sys.argv[1])))
s.listen(8)
while True:
    try:
        c, _ = s.accept()
        c.close()
    except Exception:
        time.sleep(0.05)
' "$port"
  fi
done
printf '{{"canary":"{canary}"}}\n'
exit 0
"#,
        log = log.display(),
        canary = KUBECTL_CANARY,
        python = python,
    );
    let path = dir.join("kubectl");
    std::fs::write(&path, script).expect("write fake kubectl");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .expect("chmod fake kubectl");
    }
}

/// The wrapped command: a python script that records its own environment and
/// argv, so the test can diff them against what `lumen connect` was given.
#[cfg(unix)]
fn install_env_recorder(dir: &Path) -> PathBuf {
    let path = dir.join("record_env.py");
    std::fs::write(
        &path,
        r#"import json, os, sys
json.dump({"env": dict(os.environ), "argv": sys.argv[1:]}, open(sys.argv[1], "w"))
"#,
    )
    .expect("write env recorder");
    path
}

/// Probes candidate python3 interpreters under the cleared test environment and
/// returns the first that exits 0. If none succeed, panics naming every candidate
/// and its exit code.
#[cfg(unix)]
fn resolve_python3(env: &[(String, String)]) -> String {
    let candidates = [
        "python3",
        "/usr/bin/python3",
        "/usr/local/bin/python3",
        "/opt/homebrew/bin/python3",
    ];
    let mut failures = Vec::new();
    for candidate in candidates {
        let mut cmd = Command::new(candidate);
        cmd.env_clear();
        for (k, v) in env {
            cmd.env(k, v);
        }
        cmd.args(["-c", "import sys; sys.exit(0)"]);
        match cmd.status() {
            Ok(status) if status.success() => return candidate.to_string(),
            Ok(status) => failures.push(format!("{candidate} exited {status}")),
            Err(err) => failures.push(format!("{candidate} failed to spawn: {err}")),
        }
    }
    panic!(
        "no candidate python3 interpreter succeeded under the cleared test environment; tried:\n{}",
        failures.join("\n")
    );
}

/// AC2 (with R2 and R4): the child of `lumen connect` receives exactly one
/// new environment variable, it is a URL, and Kubernetes was contacted once —
/// for the port-forward and for nothing else.
#[cfg(unix)]
#[test]
fn connect_hands_the_child_a_url_and_never_reads_a_secret() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bin = tmp.path().join("bin");
    std::fs::create_dir_all(&bin).expect("create fake bin dir");
    let kubectl_log = tmp.path().join("kubectl.log");

    // A deterministic, minimal environment: `env_clear` makes the "delta is
    // exactly one variable" assertion below exact instead of approximate.
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let parent_env: Vec<(String, String)> = vec![
        ("PATH".into(), path),
        (
            "HOME".into(),
            std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()),
        ),
    ];
    let python = resolve_python3(&parent_env);
    install_fake_kubectl(&bin, &kubectl_log, &python);
    let recorder = install_env_recorder(tmp.path());
    let child_out = tmp.path().join("child.json");

    // The baseline: the same recorder, the same environment, launched
    // directly. Diffing against this rather than against `parent_env`
    // attributes only what `lumen connect` added — a `python3` that is a
    // version-manager shim injects several variables of its own, and blaming
    // those on the CLI would make this assertion a coin flip per machine.
    let baseline_out = tmp.path().join("baseline.json");
    let mut baseline_cmd = Command::new(&python);
    baseline_cmd.env_clear();
    for (k, v) in &parent_env {
        baseline_cmd.env(k, v);
    }
    let baseline_status = baseline_cmd
        .args([recorder.to_str().unwrap(), baseline_out.to_str().unwrap()])
        .status()
        .expect("run baseline recorder");
    assert!(baseline_status.success(), "baseline recorder failed");
    let baseline: serde_json::Value =
        serde_json::from_str(&read(&baseline_out)).expect("baseline recorded its environment");
    let baseline_keys: BTreeSet<String> = baseline["env"]
        .as_object()
        .expect("baseline env is an object")
        .keys()
        .cloned()
        .collect();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_lumen"));
    cmd.env_clear();
    for (k, v) in &parent_env {
        cmd.env(k, v);
    }
    cmd.args([
        "connect",
        "--plaintext",
        "--namespace",
        "lumen-acceptance",
        "--cr",
        "search",
        "--",
        &python,
        recorder.to_str().unwrap(),
        child_out.to_str().unwrap(),
    ]);
    let out = cmd.output().expect("run lumen connect");
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(
        out.status.success(),
        "`lumen connect` failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );

    // R2: exactly one kubectl invocation, and it is the port-forward. A
    // Secret lookup would appear here as a second line before it.
    let log = read(&kubectl_log);
    let invocations: Vec<&str> = log.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(
        invocations.len(),
        1,
        "#2873 AC2/R2: `lumen connect` contacted kubectl {} times; only the port-forward is \
         allowed:\n{log}",
        invocations.len()
    );
    assert!(
        invocations[0].contains("port-forward"),
        "#2873 AC2/R2: the single kubectl invocation is not the port-forward: {}",
        invocations[0]
    );

    let recorded: serde_json::Value =
        serde_json::from_str(&read(&child_out)).expect("child recorded its environment");
    let child_env = recorded["env"]
        .as_object()
        .expect("recorded env is an object");

    // AC2: the child's environment gained exactly `LUMEN_URL`, and lost
    // nothing.
    let child_keys: BTreeSet<String> = child_env.keys().cloned().collect();
    let gained: Vec<&String> = child_keys.difference(&baseline_keys).collect();
    assert_eq!(
        gained,
        vec!["LUMEN_URL"],
        "#2873 AC2: `lumen connect` gave its child {gained:?}; the only variable it may add is \
         LUMEN_URL"
    );
    let lost: Vec<&String> = baseline_keys.difference(&child_keys).collect();
    assert!(
        lost.is_empty(),
        "#2873 AC2: `lumen connect` dropped {lost:?} from its child's environment"
    );
    let url = child_env["LUMEN_URL"].as_str().unwrap_or_default();
    assert!(
        url.starts_with("http://127.0.0.1:"),
        "#2873 AC2: LUMEN_URL should be the local end of the port-forward, got {url:?}"
    );

    // AC2: no credential reached the child by any route — not in its
    // environment, not on its command line, and not through lumen's own
    // output.
    let argv = recorded["argv"].to_string();
    let env_dump = serde_json::to_string(child_env).expect("serialize child env");
    for (label, haystack) in [
        ("child environment", env_dump.as_str()),
        ("child argv", argv.as_str()),
        ("lumen stdout", stdout.as_str()),
        ("lumen stderr", stderr.as_str()),
    ] {
        assert!(
            !haystack.contains(KUBECTL_CANARY),
            "#2873 AC2: the fake kubectl's canary credential reached the {label}, which means a \
             Secret lookup ran:\n{haystack}"
        );
    }

    // R4: the caller is told, on stderr, that the forwarded connection
    // carries no identity. Silence here is how a 401 becomes a mystery.
    assert!(
        stderr.contains("no credential"),
        "#2873 R4: `lumen connect` must say on stderr that the connection is unauthenticated, \
         got:\n{stderr}"
    );
}

// ---------------------------------------------------------------------------
// AC4 — a request that needs an identity fails as 401, without inventing one.
// ---------------------------------------------------------------------------

/// A one-shot HTTP server that records the request head and answers 401.
/// Returns the bound port and a handle yielding the recorded request.
#[cfg(feature = "backup")]
fn spawn_denying_server() -> (u16, std::thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind denying server");
    let port = listener.local_addr().expect("local addr").port();
    let handle = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let head = read_request_head(&mut stream);
        let body = br#"{"error":"unauthorized"}"#;
        let response = format!(
            "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.write_all(body);
        let _ = stream.flush();
        head
    });
    (port, handle)
}

#[cfg(feature = "backup")]
fn read_request_head(stream: &mut TcpStream) -> String {
    let mut reader = BufReader::new(stream);
    let mut head = String::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        let done = line == "\r\n" || line == "\n";
        head.push_str(&line);
        if done {
            break;
        }
    }
    head
}

/// AC4: `lumen query` sends no `Authorization` header — with a credential-
/// shaped variable sitting in its environment, which is exactly the condition
/// under which the retired fallback used to fire — and it reports the 401 it
/// gets back instead of retrying with something it found lying around.
#[cfg(feature = "backup")]
#[test]
fn query_sends_no_authorization_header_and_surfaces_the_401() {
    let (port, server) = spawn_denying_server();
    let out = Command::new(env!("CARGO_BIN_EXE_lumen"))
        .args([
            "query",
            "collections",
            "list",
            "--url",
            &format!("http://127.0.0.1:{port}"),
        ])
        // Both spellings the retired code path used to read. Neither may
        // reach the wire.
        .env(needle(&["LUMEN_", "TOKEN"]), KUBECTL_CANARY)
        .env(needle(&["LUMEN_BACKUP_", "TOKEN"]), KUBECTL_CANARY)
        .output()
        .expect("run lumen query");

    let head = server.join().expect("denying server thread");
    let lowered = head.to_lowercase();
    assert!(
        !lowered.contains("authorization:"),
        "#2873 AC4: `lumen query` sent an Authorization header:\n{head}"
    );
    assert!(
        !head.contains(KUBECTL_CANARY),
        "#2873 AC4: a credential from the environment reached the wire:\n{head}"
    );

    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(
        !out.status.success(),
        "#2873 AC4: a 401 must fail the command, not be swallowed\nstdout:\n{stdout}"
    );
    assert!(
        stderr.contains("401"),
        "#2873 AC4: the failure must name the status the server actually returned, got:\n{stderr}"
    );
}

// ---------------------------------------------------------------------------
// AC5 — diagnostics are redaction-tested with a canary credential.
// ---------------------------------------------------------------------------

/// AC5: passing a credential where the retired flag used to be is rejected,
/// and the rejection does not echo the value.
///
/// This is the case a redaction test exists for: the argument parser is the
/// first thing to see an unknown flag, and the obvious error message —
/// "unexpected argument `--token=<value>`" — would write the credential into
/// the caller's terminal, their CI log, and their shell history in one go.
#[test]
fn rejecting_the_retired_credential_flag_never_echoes_its_value() {
    let canary = "canary-argv-credential-2873";
    let flag = needle(&["--", "token"]);
    let cases: Vec<Vec<String>> = vec![
        vec![
            "dump".into(),
            "--url".into(),
            "http://127.0.0.1:1".into(),
            flag.clone(),
            canary.into(),
        ],
        vec![
            "dump".into(),
            "--url".into(),
            "http://127.0.0.1:1".into(),
            format!("{flag}={canary}"),
        ],
        vec![
            "backup".into(),
            "--url".into(),
            "http://127.0.0.1:1".into(),
            "--dest".into(),
            "file:///tmp/none".into(),
            flag.clone(),
            canary.into(),
        ],
        vec![
            "query".into(),
            "collections".into(),
            "list".into(),
            "--url".into(),
            "http://127.0.0.1:1".into(),
            flag.clone(),
            canary.into(),
        ],
    ];

    for case in cases {
        let args: Vec<&str> = case.iter().map(String::as_str).collect();
        let out = Command::new(env!("CARGO_BIN_EXE_lumen"))
            .args(&args)
            .output()
            .unwrap_or_else(|err| panic!("run lumen {args:?}: {err}"));
        let mut combined = String::from_utf8_lossy(&out.stdout).into_owned();
        combined.push_str(&String::from_utf8_lossy(&out.stderr));
        assert!(
            !out.status.success(),
            "#2873 AC5: `lumen {}` must be rejected, not accepted:\n{combined}",
            args.join(" ")
        );
        assert!(
            !combined.contains(canary),
            "#2873 AC5: rejecting `lumen {}` echoed the credential value:\n{combined}",
            args.join(" ")
        );
    }
}

/// AC5, second half: a credential-shaped environment variable is not echoed
/// into diagnostics either. The failure path below is a connection refusal —
/// the most verbose error the query path can produce — so if any diagnostic
/// dumps the environment, this is where it shows up.
#[cfg(feature = "backup")]
#[test]
fn diagnostics_never_echo_a_credential_shaped_environment_variable() {
    let canary = "canary-env-credential-2873";
    // Port 1 is reserved and unbound: connect() fails immediately.
    let out = Command::new(env!("CARGO_BIN_EXE_lumen"))
        .args([
            "query",
            "collections",
            "list",
            "--url",
            "http://127.0.0.1:1",
        ])
        .env(needle(&["LUMEN_", "TOKEN"]), canary)
        .output()
        .expect("run lumen query");
    let mut combined = String::from_utf8_lossy(&out.stdout).into_owned();
    combined.push_str(&String::from_utf8_lossy(&out.stderr));
    assert!(
        !out.status.success(),
        "an unreachable URL must fail:\n{combined}"
    );
    assert!(
        !combined.contains(canary),
        "#2873 AC5: a credential-shaped environment variable was echoed into diagnostics:\n{combined}"
    );
}
