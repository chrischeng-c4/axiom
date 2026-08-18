// HANDWRITE gap="missing-generator:e2e-test:cli-client-ksa-token" tracker="2878" reason="Proving where a credential is not requires running the real binary against a stand-in apiserver and a recording upstream, then reading the child's environment, argv and output; no generator primitive emits a three-party process test."
//! #2878: `lumen` mints its own short-lived token, and the child never sees it.
//!
//! The retired model handed a credential to whoever asked. The replacement
//! asks Kubernetes for one, for an account the caller named out loud, and then
//! spends it on the child's behalf. Two properties make that worth the extra
//! moving parts, and both are behavioural — neither can be read off the
//! source:
//!
//! 1. the request that goes to the apiserver names the flags the caller
//!    passed and Lumen's own audience (AC1);
//! 2. the token that comes back reaches the *server* and nowhere else — not
//!    the child's environment, not its argv, not anyone's stdout (AC4, AC6).
//!
//! So this file stands up three processes' worth of the real thing: a
//! stand-in apiserver that records what was asked of it, a recording upstream
//! standing in for a serving node, and the shipped binary between them. The
//! token the apiserver issues is a canary the CLI has no other way to obtain,
//! which is what makes the absence assertions mean something: any place it
//! turns up, it came from this path.
//!
//! What is *not* here: a Google-account kubeconfig and a GSA-backed kubeconfig
//! minting against a real GKE control plane (AC2), a live revoked grant
//! (AC5's second half), and a direct Google token being refused by a serving
//! node (AC7). Those need a cluster and belong to the GKE acceptance run;
//! the refresh clock and the revocation behaviour they exercise are unit-
//! tested in `service-auth`'s `k8s::token_request`.

#![cfg(all(unix, feature = "delegated-auth", feature = "backup"))]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;

/// The token the stand-in apiserver issues. The CLI has no other source for
/// it, so every assertion of the form "this string is absent" is an assertion
/// about this code path and not about the machine the test runs on.
const TOKEN_CANARY: &str = "canary-ksa-token-2878-must-not-escape";

/// Who the stand-in apiserver says the caller is. `lumen` learns this only by
/// asking (`SelfSubjectReview`), which is the point of R6: a denial names the
/// identity the cluster saw, not the one the CLI assumed.
const CALLER: &str = "chris@example.com";

const NAMESPACE: &str = "lumen-prod";
const CLIENT_SA: &str = "lumen-agent";

// ---------------------------------------------------------------------------
// A stand-in apiserver
// ---------------------------------------------------------------------------

/// One recorded HTTP request: the target, the body for a POST, and whatever
/// `Authorization` header came with it. The apiserver stand-in reads the first
/// two; the upstream stand-in reads the third.
#[derive(Debug, Clone)]
struct Recorded {
    method: String,
    path: String,
    body: String,
    authorization: String,
}

/// How the stand-in should answer a TokenRequest for a given ServiceAccount.
enum Grant {
    /// Issue `TOKEN_CANARY`, expiring far enough out that no refresh fires
    /// during the test.
    Issue { service_account: &'static str },
    /// Refuse everything, the way RBAC does when the caller holds no
    /// `serviceaccounts/token` create on the named object.
    RefuseAll,
}

struct FakeApiserver {
    port: u16,
    requests: mpsc::Receiver<Recorded>,
    _shutdown: TcpStream,
}

impl FakeApiserver {
    /// Bind on loopback and serve until the returned handle is dropped.
    ///
    /// Deliberately hand-rolled rather than built on a framework: this stands
    /// in for the *apiserver*, and the whole value of the test is that the
    /// bytes `lumen` sends are read by something that did not help write them.
    fn start(grant: Grant) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind stand-in apiserver");
        let port = listener.local_addr().expect("apiserver addr").port();
        let (tx, requests) = mpsc::channel();

        // A self-connection whose sender lives in the handle: dropping the
        // handle closes it, the accept loop notices, and the thread ends.
        let shutdown = TcpStream::connect(("127.0.0.1", port)).expect("open shutdown channel");
        let (sentinel, _) = listener.accept().expect("accept shutdown channel");
        drop(sentinel);

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { break };
                let Some(request) = read_request(&mut stream) else {
                    continue;
                };
                if tx.send(request.clone()).is_err() {
                    break;
                }
                let (status, body) = answer(&grant, &request);
                write_response(&mut stream, status, &body);
            }
        });

        Self {
            port,
            requests,
            _shutdown: shutdown,
        }
    }

    fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// Everything received so far. Called after the CLI has exited, so no
    /// request is still in flight.
    fn recorded(&self) -> Vec<Recorded> {
        self.requests.try_iter().collect()
    }
}

fn answer(grant: &Grant, request: &Recorded) -> (u16, String) {
    if request.path.contains("selfsubjectreviews") {
        return (
            201,
            format!(
                r#"{{"apiVersion":"authentication.k8s.io/v1","kind":"SelfSubjectReview",
                    "metadata":{{}},"status":{{"userInfo":{{"username":"{CALLER}",
                    "groups":["system:authenticated"]}}}}}}"#
            ),
        );
    }
    if !request.path.ends_with("/token") {
        return (404, r#"{"kind":"Status","code":404}"#.to_string());
    }
    match grant {
        Grant::Issue { service_account } if request.path.contains(service_account) => (
            201,
            format!(
                r#"{{"apiVersion":"authentication.k8s.io/v1","kind":"TokenRequest",
                    "metadata":{{}},
                    "spec":{{"audiences":["lumen.axiom.dev"],"expirationSeconds":600}},
                    "status":{{"token":"{TOKEN_CANARY}",
                    "expirationTimestamp":"2099-01-01T00:00:00Z"}}}}"#
            ),
        ),
        _ => {
            let name = request
                .path
                .rsplit('/')
                .nth(1)
                .unwrap_or("unknown")
                .to_string();
            (
                403,
                format!(
                    r#"{{"kind":"Status","apiVersion":"v1","status":"Failure","reason":"Forbidden",
                        "code":403,"message":"serviceaccounts \"{name}\" is forbidden: User
                        \"{CALLER}\" cannot create resource \"serviceaccounts/token\""}}"#
                ),
            )
        }
    }
}

/// A recording stand-in for a serving node. Answers every request 200 and
/// keeps the `Authorization` header it was sent, or the empty string.
struct RecordingUpstream {
    port: u16,
    headers: mpsc::Receiver<String>,
    _shutdown: TcpStream,
}

impl RecordingUpstream {
    /// Binds an ephemeral loopback port and reports it. The fake `kubectl`
    /// forwards the tunnelled port to this one, so nothing here has to guess a
    /// port number — and no two tests in this binary can pick the same one.
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind recording upstream");
        let port = listener.local_addr().expect("upstream addr").port();
        let (tx, headers) = mpsc::channel();
        let shutdown = TcpStream::connect(("127.0.0.1", port)).expect("open shutdown channel");
        let (sentinel, _) = listener.accept().expect("accept shutdown channel");
        drop(sentinel);

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { break };
                let Some(request) = read_request(&mut stream) else {
                    continue;
                };
                if tx.send(request.authorization).is_err() {
                    break;
                }
                write_response(&mut stream, 200, r#"{"collections":[]}"#);
            }
        });

        Self {
            port,
            headers,
            _shutdown: shutdown,
        }
    }

    fn port(&self) -> u16 {
        self.port
    }

    fn seen(&self) -> Vec<String> {
        self.headers.try_iter().collect()
    }
}

fn read_request(stream: &mut TcpStream) -> Option<Recorded> {
    let mut reader = BufReader::new(stream.try_clone().ok()?);
    let mut line = String::new();
    if reader.read_line(&mut line).ok()? == 0 {
        return None;
    }
    let mut parts = line.split_whitespace();
    let method = parts.next()?.to_string();
    // `kube` writes the query string even when there is nothing in it, so the
    // target arrives as `.../token?`. The query is not what any assertion here
    // is about; drop it so the recorded path is the resource path.
    let path = parts.next()?.split('?').next()?.to_string();

    let mut length = 0usize;
    let mut authorization = String::new();
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header).ok()? == 0 {
            break;
        }
        if header == "\r\n" || header == "\n" {
            break;
        }
        let lowered = header.to_ascii_lowercase();
        if let Some(rest) = lowered.strip_prefix("content-length:") {
            length = rest.trim().parse().unwrap_or(0);
        }
        if lowered.starts_with("authorization:") {
            authorization = header
                .split_once(':')
                .map(|(_, v)| v.trim().to_string())
                .unwrap_or_default();
        }
    }
    let mut body = vec![0u8; length];
    if length > 0 {
        reader.read_exact(&mut body).ok()?;
    }
    Some(Recorded {
        method,
        path,
        body: String::from_utf8_lossy(&body).into_owned(),
        authorization,
    })
}

fn write_response(stream: &mut TcpStream, status: u16, body: &str) {
    let head = format!(
        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\
         Connection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body.as_bytes());
    let _ = stream.flush();
}

// ---------------------------------------------------------------------------
// Fixtures on disk
// ---------------------------------------------------------------------------

/// A kubeconfig pointing at the stand-in, with a bearer user. What matters is
/// that the *caller's* credential is this one — and that it is not the thing
/// that reaches Lumen.
fn write_kubeconfig(dir: &Path, apiserver: &str) -> PathBuf {
    let path = dir.join("kubeconfig");
    std::fs::write(
        &path,
        format!(
            r#"apiVersion: v1
kind: Config
current-context: stand-in
clusters:
- name: stand-in
  cluster:
    server: {apiserver}
contexts:
- name: stand-in
  context:
    cluster: stand-in
    user: caller
users:
- name: caller
  user:
    token: caller-kubeconfig-credential-2878
"#
        ),
    )
    .expect("write kubeconfig");
    path
}

/// A `kubectl` that records what it was asked to do and, for `port-forward`,
/// actually forwards: it binds the local port itself and pumps every byte to
/// the in-process upstream named by `LUMEN_TEST_UPSTREAM_PORT`.
///
/// Forwarding for real is what makes the ordering real rather than assumed.
/// `lumen connect` waits for the local port to accept, and the only thing that
/// opens it is this process — so a run that got as far as talking to the
/// upstream provably ran `kubectl` first, and the recorded log is complete by
/// then. Binding the upstream directly on the forwarded port instead would
/// leave the port open before `lumen` started, and the invocation record would
/// be a race.
fn install_fake_kubectl(dir: &Path, log: &Path) {
    let forwarder = dir.join("port_forwarder.py");
    std::fs::write(
        &forwarder,
        r#"import socket, sys, threading
local, upstream = int(sys.argv[1]), int(sys.argv[2])
server = socket.socket()
server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
server.bind(("127.0.0.1", local))
server.listen(64)

def pump(src, dst):
    try:
        while True:
            chunk = src.recv(65536)
            if not chunk:
                break
            dst.sendall(chunk)
    except OSError:
        pass
    finally:
        try:
            dst.shutdown(socket.SHUT_WR)
        except OSError:
            pass

while True:
    near, _ = server.accept()
    try:
        far = socket.create_connection(("127.0.0.1", upstream))
    except OSError:
        near.close()
        continue
    for a, b in ((near, far), (far, near)):
        threading.Thread(target=pump, args=(a, b), daemon=True).start()
"#,
    )
    .expect("write port forwarder");

    let script = format!(
        r#"#!/bin/sh
printf '%s\n' "$*" >> "{log}"
forwarding=
mapping=
for a in "$@"; do
  case "$a" in
    port-forward) forwarding=1 ;;
    *:*) mapping="$a" ;;
  esac
done
if [ -n "$forwarding" ]; then
  # `lumen connect` nulls this process's stderr, so a forwarder that died on
  # startup would otherwise surface only as a bare 30s readiness timeout. Keep
  # its diagnosis next to the invocation log the assertions already read.
  exec python3 "{forwarder}" "${{mapping%%:*}}" "$LUMEN_TEST_UPSTREAM_PORT" 2>>"{log}.forwarder"
fi
exit 0
"#,
        log = log.display(),
        forwarder = forwarder.display()
    );
    let path = dir.join("kubectl");
    std::fs::write(&path, script).expect("write fake kubectl");
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
        .expect("chmod fake kubectl");
}

/// The wrapped command: records its own environment and argv, then spends the
/// URL it was handed on one request.
fn install_child(dir: &Path) -> PathBuf {
    let path = dir.join("child.py");
    std::fs::write(
        &path,
        r#"import json, os, sys, urllib.request
url = os.environ["LUMEN_URL"]
try:
    body = urllib.request.urlopen(url + "/collections", timeout=10).read().decode()
except Exception as exc:
    body = "error: %s" % exc
json.dump({"env": dict(os.environ), "argv": sys.argv[1:], "body": body},
          open(sys.argv[1], "w"))
"#,
    )
    .expect("write child");
    path
}

// ---------------------------------------------------------------------------
// AC1 + AC4 + AC6
// ---------------------------------------------------------------------------

/// The whole contract in one run: the CLI asks the apiserver for a token for
/// the account named on the command line, and then spends it *for* the child
/// rather than giving it to the child.
#[test]
fn connect_mints_for_the_named_account_and_spends_the_token_on_the_childs_behalf() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bin = tmp.path().join("bin");
    std::fs::create_dir_all(&bin).expect("create fake bin dir");

    let apiserver = FakeApiserver::start(Grant::Issue {
        service_account: CLIENT_SA,
    });
    let upstream = RecordingUpstream::start();
    let kubeconfig = write_kubeconfig(tmp.path(), &apiserver.url());
    let kubectl_log = tmp.path().join("kubectl.log");
    install_fake_kubectl(&bin, &kubectl_log);
    let child = install_child(tmp.path());
    let child_out = tmp.path().join("child.json");

    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = Command::new(env!("CARGO_BIN_EXE_lumen"))
        .env_clear()
        .env("PATH", &path)
        .env(
            "HOME",
            std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()),
        )
        .env("KUBECONFIG", &kubeconfig)
        .env("LUMEN_TEST_UPSTREAM_PORT", upstream.port().to_string())
        .args([
            "connect",
            "--plaintext",
            "--namespace",
            NAMESPACE,
            "--cr",
            "search",
            "--client-sa",
            CLIENT_SA,
            "--",
            "python3",
            child.to_str().unwrap(),
            child_out.to_str().unwrap(),
        ])
        .output()
        .expect("run lumen connect");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    let seen = apiserver.recorded();
    assert!(
        out.status.success(),
        "`lumen connect --client-sa` failed\nstdout:\n{stdout}\nstderr:\n{stderr}\n\
         apiserver saw: {:?}",
        seen.iter()
            .map(|r| format!("{} {}", r.method, r.path))
            .collect::<Vec<_>>()
    );

    // AC1: the request that actually went out names the namespace and the
    // ServiceAccount the caller passed, Lumen's audience, and 600 seconds.
    let mints: Vec<Recorded> = seen
        .into_iter()
        .filter(|r| r.path.ends_with("/token"))
        .collect();
    assert_eq!(
        mints.len(),
        1,
        "#2878 AC1: expected exactly one TokenRequest, saw {}: {:?}",
        mints.len(),
        mints.iter().map(|r| &r.path).collect::<Vec<_>>()
    );
    let mint = &mints[0];
    assert_eq!(mint.method, "POST");
    assert_eq!(
        mint.path,
        format!("/api/v1/namespaces/{NAMESPACE}/serviceaccounts/{CLIENT_SA}/token"),
        "#2878 AC1/R3: the TokenRequest went to a different object than the flags named"
    );
    let asked: serde_json::Value =
        serde_json::from_str(&mint.body).expect("#2878 AC1: the TokenRequest body is JSON");
    assert_eq!(
        asked["spec"]["audiences"],
        serde_json::json!(["lumen.axiom.dev"]),
        "#2878 AC1/R2: a token minted for another audience is one a serving node will refuse"
    );
    assert_eq!(
        asked["spec"]["expirationSeconds"],
        serde_json::json!(600),
        "#2878 AC1/R2: the requested lifetime is part of the contract, not a default"
    );

    // AC4: the server saw the minted token, exactly once, as a bearer.
    let seen = upstream.seen();
    assert_eq!(
        seen,
        vec![format!("Bearer {TOKEN_CANARY}")],
        "#2878 AC4: the proxy must attach the minted token to the child's request"
    );

    // AC4: what the child got was a loopback URL, and not the forwarded port —
    // it talked to the proxy, which is the only party holding the credential.
    let recorded: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&child_out).expect("child wrote its record"))
            .expect("child record is JSON");
    let env = recorded["env"].as_object().expect("child env is an object");
    let url = env["LUMEN_URL"].as_str().unwrap_or_default();
    assert!(
        url.starts_with("http://127.0.0.1:"),
        "#2878 AC4: the child's URL must be loopback, got {url:?}"
    );
    assert!(
        !url.ends_with(&format!(":{}", upstream.port())),
        "#2878 AC4: the child was pointed straight at the upstream, so nothing attached a \
         credential on its behalf: {url}"
    );
    assert!(
        recorded["body"]
            .as_str()
            .unwrap_or_default()
            .contains("collections"),
        "#2878 AC4: the child's request did not come back with the upstream's answer: {}",
        recorded["body"]
    );

    // AC6: the canary is in the one place it belongs and nowhere else.
    let child_argv = recorded["argv"].to_string();
    let child_env = serde_json::to_string(env).expect("serialize child env");
    for (label, haystack) in [
        ("child environment", child_env.as_str()),
        ("child argv", child_argv.as_str()),
        ("lumen stdout", stdout.as_str()),
        ("lumen stderr", stderr.as_str()),
    ] {
        assert!(
            !haystack.contains(TOKEN_CANARY),
            "#2878 AC6: the minted token reached the {label}:\n{haystack}"
        );
    }
    // The kubeconfig credential is the caller's, and it stops at the
    // apiserver. It has no business on the Lumen side of the connection.
    assert!(
        !seen
            .iter()
            .any(|h| h.contains("caller-kubeconfig-credential")),
        "#2878 R5: the caller's own kubeconfig credential reached the serving node: {seen:?}"
    );

    // AC6, the temp-file half: nothing under the working directory holds it.
    let mut leaked = Vec::new();
    scan_for(tmp.path(), TOKEN_CANARY, &mut leaked);
    assert!(
        leaked.is_empty(),
        "#2878 AC6: the minted token was written to disk: {leaked:?}"
    );

    // The port-forward is the only thing kubectl was used for; the token came
    // from the Kubernetes client, not from shelling out.
    let log = std::fs::read_to_string(&kubectl_log).unwrap_or_default();
    let invocations: Vec<&str> = log.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(
        invocations.len(),
        1,
        "#2878: kubectl was invoked {} times; only the port-forward is allowed:\n{log}",
        invocations.len()
    );
    assert!(
        invocations[0].contains("port-forward"),
        "{}",
        invocations[0]
    );
}

fn scan_for(dir: &Path, needle: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_for(&path, needle, out);
        } else if std::fs::read_to_string(&path)
            .map(|b| b.contains(needle))
            .unwrap_or(false)
        {
            out.push(path);
        }
    }
}

// ---------------------------------------------------------------------------
// R6 / AC3's local half
// ---------------------------------------------------------------------------

/// A caller who may mint one account's token and not another's gets told which
/// identity was refused, for which object, and the command that answers "may
/// I?" — and no credential appears in the message.
#[test]
fn a_refused_mint_names_the_caller_the_account_and_the_check_to_run() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bin = tmp.path().join("bin");
    std::fs::create_dir_all(&bin).expect("create fake bin dir");

    let apiserver = FakeApiserver::start(Grant::RefuseAll);
    let upstream = RecordingUpstream::start();
    let kubeconfig = write_kubeconfig(tmp.path(), &apiserver.url());
    install_fake_kubectl(&bin, &tmp.path().join("kubectl.log"));

    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = Command::new(env!("CARGO_BIN_EXE_lumen"))
        .env_clear()
        .env("PATH", &path)
        .env(
            "HOME",
            std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()),
        )
        .env("KUBECONFIG", &kubeconfig)
        .env("LUMEN_TEST_UPSTREAM_PORT", upstream.port().to_string())
        .args([
            "connect",
            "--plaintext",
            "--namespace",
            NAMESPACE,
            "--cr",
            "search",
            "--client-sa",
            "lumen-sibling",
            "--",
            "true",
        ])
        .output()
        .expect("run lumen connect");
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();

    assert!(
        !out.status.success(),
        "#2878 R6: a refused mint must fail the command rather than running the child \
         unauthenticated\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    for expected in [
        CALLER,
        "lumen-sibling",
        NAMESPACE,
        "kubectl auth can-i",
        "--subresource=token",
        "lumen k8s access render",
    ] {
        assert!(
            stderr.contains(expected),
            "#2878 R6: the denial does not name `{expected}`:\n{stderr}"
        );
    }
    assert!(
        !stderr.contains("caller-kubeconfig-credential"),
        "#2878 R6/AC6: the denial printed the caller's credential:\n{stderr}"
    );
    // The child must not have run: an unauthenticated fallback is precisely
    // the behaviour this phase exists to remove.
    assert!(
        stderr.contains("forbidden") || stderr.contains("cannot mint") || stderr.contains("denied"),
        "#2878 R6: the denial should read as a denial:\n{stderr}"
    );
}

// ---------------------------------------------------------------------------
// #3113 R6/R7, AC4: the tunnel is loopback, the identity is the Service's
// ---------------------------------------------------------------------------
//
// A port-forward's local end is `127.0.0.1`, and that is the whole reason
// these tests exist. It is tempting to conclude that a socket on the loopback
// interface needs no verification, or that the certificate should name
// `localhost` so that it does. Both conclusions end in the same place: a leaf
// that authenticates nothing, or a connection that authenticates the tunnel
// instead of what the tunnel reaches. So `lumen connect` addresses the
// Kubernetes Service by name and redirects only address resolution — and the
// assertions below are about what the *server* saw addressed to it, and about
// what the caller is told when the name does not check out.

/// The Service these tests reach. `<service>.<namespace>.svc` — the default
/// `lumen connect` derives, and one of the two names the operator requests.
const SERVICE_DNS: &str = "search.lumen-prod.svc";

/// A throwaway CA, standing in for the private pool #3109 provisions.
struct TestCa {
    cert: rcgen::Certificate,
    key: rcgen::KeyPair,
}

fn test_ca(common_name: &str) -> TestCa {
    let mut params = rcgen::CertificateParams::new(Vec::new()).expect("ca params");
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, common_name);
    params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    let key = rcgen::KeyPair::generate().expect("ca key");
    let cert = params.self_signed(&key).expect("self-sign ca");
    TestCa { cert, key }
}

/// A serving leaf for `dns`, signed by `ca`.
fn test_leaf(ca: &TestCa, dns: &str) -> (String, String) {
    let mut params =
        rcgen::CertificateParams::new(vec![dns.to_string()]).expect("leaf params");
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, dns);
    params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ServerAuth];
    let key = rcgen::KeyPair::generate().expect("leaf key");
    let cert = params.signed_by(&key, &ca.cert, &ca.key).expect("sign leaf");
    (cert.pem(), key.serialize_pem())
}

/// One request the TLS upstream answered.
#[derive(Debug, Clone)]
struct TlsSeen {
    path: String,
    host: String,
    authorization: String,
}

/// A serving node standing where `kubectl port-forward` would, terminating TLS
/// with the real [`service_http::serve_tls`] listener over material laid out
/// the way `spec.servingTlsSecret` projects it.
///
/// Not a stub TLS socket: the point is that whatever `lumen connect` builds has
/// to satisfy the same listener production runs.
struct TlsUpstream {
    port: u16,
    seen: std::sync::Arc<std::sync::Mutex<Vec<TlsSeen>>>,
    // Held so the projected material outlives the listener reading it.
    _material: tempfile::TempDir,
    // Held, never sent on: dropping it is the shutdown edge.
    _shutdown: tokio::sync::oneshot::Sender<()>,
}

impl TlsUpstream {
    fn start(dns: &str, cert_pem: &str, key_pem: &str, ca_pem: &str) -> Self {
        let material = tempfile::tempdir().expect("material dir");
        let dir = material.path().to_path_buf();
        std::fs::write(dir.join("tls.crt"), cert_pem).expect("write cert");
        std::fs::write(dir.join("tls.key"), key_pem).expect("write key");
        std::fs::write(dir.join("ca.crt"), ca_pem).expect("write ca");

        let claimed = dns.to_string();
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let recorder = seen.clone();
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let (ready_tx, ready_rx) = mpsc::channel();

        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("upstream runtime");
            runtime.block_on(async move {
                lumen::tls::install_default_crypto_provider();
                let tls = lumen::tls::ServingTlsConfig {
                    cert: dir.join("tls.crt"),
                    key: dir.join("tls.key"),
                    ca: dir.join("ca.crt"),
                    dns_names: vec![claimed],
                }
                .reloadable()
                .expect("activate the serving leaf");
                let app = axum::Router::new().fallback(
                    move |request: axum::http::Request<axum::body::Body>| {
                        let recorder = recorder.clone();
                        async move {
                            let header = |name: &str| {
                                request
                                    .headers()
                                    .get(name)
                                    .and_then(|v| v.to_str().ok())
                                    .unwrap_or_default()
                                    .to_string()
                            };
                            recorder.lock().expect("record").push(TlsSeen {
                                path: request.uri().path().to_string(),
                                // HTTP/2 carries the authority in the pseudo-
                                // header the URI exposes; HTTP/1.1 in `Host`.
                                // Either way this is the name the connection
                                // was addressed to.
                                host: request
                                    .uri()
                                    .authority()
                                    .map(|a| a.host().to_string())
                                    .filter(|h| !h.is_empty())
                                    .unwrap_or_else(|| header("host")),
                                authorization: header("authorization"),
                            });
                            (axum::http::StatusCode::OK, r#"{"collections":[]}"#)
                        }
                    },
                );
                let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                    .await
                    .expect("bind tls upstream");
                let _ = ready_tx.send(listener.local_addr().expect("upstream addr").port());
                service_http::serve_tls(
                    listener,
                    app,
                    service_http::config_source(move || tls.server_config()),
                    async move {
                        let _ = shutdown_rx.await;
                    },
                )
                .await;
            });
        });

        let port = ready_rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("tls upstream binds");
        Self {
            port,
            seen,
            _material: material,
            _shutdown: shutdown_tx,
        }
    }

    fn port(&self) -> u16 {
        self.port
    }

    fn seen(&self) -> Vec<TlsSeen> {
        self.seen.lock().expect("read records").clone()
    }
}

/// Everything a `lumen connect` run needs on disk, in one place.
struct Fixture {
    tmp: tempfile::TempDir,
    bin: PathBuf,
    kubectl_log: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let tmp = tempfile::tempdir().expect("tempdir");
        let bin = tmp.path().join("bin");
        std::fs::create_dir_all(&bin).expect("create fake bin dir");
        let kubectl_log = tmp.path().join("kubectl.log");
        install_fake_kubectl(&bin, &kubectl_log);
        Self {
            tmp,
            bin,
            kubectl_log,
        }
    }

    fn run(
        &self,
        apiserver: &FakeApiserver,
        upstream_port: u16,
        args: &[&str],
    ) -> std::process::Output {
        let kubeconfig = write_kubeconfig(self.tmp.path(), &apiserver.url());
        let path = format!(
            "{}:{}",
            self.bin.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        Command::new(env!("CARGO_BIN_EXE_lumen"))
            .env_clear()
            .env("PATH", &path)
            .env(
                "HOME",
                std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()),
            )
            .env("KUBECONFIG", &kubeconfig)
            .env("LUMEN_TEST_UPSTREAM_PORT", upstream_port.to_string())
            .args(args)
            .output()
            .expect("run lumen connect")
    }
}

/// R6/AC4: the socket is loopback, the certificate that satisfied the
/// connection names the Kubernetes Service, and the token still reaches only
/// the server.
#[test]
fn connect_verifies_the_service_identity_through_the_forwarded_socket() {
    let fixture = Fixture::new();
    let apiserver = FakeApiserver::start(Grant::Issue {
        service_account: CLIENT_SA,
    });
    let ca = test_ca("lumen-private-ca");
    let (cert, key) = test_leaf(&ca, SERVICE_DNS);
    let upstream = TlsUpstream::start(SERVICE_DNS, &cert, &key, &ca.cert.pem());
    let ca_file = fixture.tmp.path().join("ca.crt");
    std::fs::write(&ca_file, ca.cert.pem()).expect("write trust bundle");
    let child = install_child(fixture.tmp.path());
    let child_out = fixture.tmp.path().join("child.json");

    let out = fixture.run(
        &apiserver,
        upstream.port(),
        &[
            "connect",
            "--namespace",
            NAMESPACE,
            "--cr",
            "search",
            "--client-sa",
            CLIENT_SA,
            "--ca-file",
            ca_file.to_str().unwrap(),
            "--",
            "python3",
            child.to_str().unwrap(),
            child_out.to_str().unwrap(),
        ],
    );
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(
        out.status.success(),
        "#3113 R6: `lumen connect --ca-file` failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let seen = upstream.seen();
    assert!(
        seen.iter().any(|r| r.path == "/healthz"),
        "#3113 R7: the TLS preflight must happen before the child runs: {seen:?}"
    );
    let forwarded: Vec<&TlsSeen> = seen.iter().filter(|r| r.path == "/collections").collect();
    assert_eq!(
        forwarded.len(),
        1,
        "#3113 R6: expected the child's one request to arrive over TLS: {seen:?}"
    );
    assert_eq!(
        forwarded[0].authorization,
        format!("Bearer {TOKEN_CANARY}"),
        "#3113 R6: the token must still be attached by the proxy, over TLS"
    );

    // The identity half: every request was *addressed to* the Service, not to
    // the loopback address the packets went to. A `Host` of `127.0.0.1` would
    // mean SNI and hostname verification had been pointed at the tunnel.
    for record in &seen {
        assert_eq!(
            record.host, SERVICE_DNS,
            "#3113 R6: a request addressed something other than the Service: {record:?}"
        );
    }

    // AC4 is unchanged by TLS: the child is handed a loopback URL and no
    // credential, and the token appears nowhere it can read.
    let recorded: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&child_out).expect("child wrote its record"))
            .expect("child record is JSON");
    let env = recorded["env"].as_object().expect("child env is an object");
    let url = env["LUMEN_URL"].as_str().unwrap_or_default();
    assert!(
        url.starts_with("http://127.0.0.1:"),
        "#3113 AC4: the child's URL must be the local proxy, got {url:?}"
    );
    assert!(
        !url.contains(SERVICE_DNS),
        "#3113 AC4: the child was handed a name it cannot resolve: {url}"
    );
    for (label, haystack) in [
        ("child environment", serde_json::to_string(env).unwrap()),
        ("child argv", recorded["argv"].to_string()),
        ("lumen stderr", stderr.clone()),
    ] {
        assert!(
            !haystack.contains(TOKEN_CANARY),
            "#3113 AC4: the minted token reached the {label}:\n{haystack}"
        );
    }

    // Still one kubectl invocation: TLS added a trust file to read, not a
    // cluster round trip.
    let log = std::fs::read_to_string(&fixture.kubectl_log).unwrap_or_default();
    let invocations: Vec<&str> = log.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(invocations.len(), 1, "{log}");
    assert!(invocations[0].contains("port-forward"), "{}", invocations[0]);
}

/// Run `lumen connect --ca-file` against a deployment that will refuse it, and
/// return the stderr the caller sees. The wrapped command is `true`, so a run
/// that "succeeds" would prove the refusal did not happen.
fn refused_connect(ca_pem: &str, upstream_port: u16) -> String {
    let fixture = Fixture::new();
    let apiserver = FakeApiserver::start(Grant::Issue {
        service_account: CLIENT_SA,
    });
    let ca_file = fixture.tmp.path().join("ca.crt");
    std::fs::write(&ca_file, ca_pem).expect("write trust bundle");

    let out = fixture.run(
        &apiserver,
        upstream_port,
        &[
            "connect",
            "--namespace",
            NAMESPACE,
            "--cr",
            "search",
            "--client-sa",
            CLIENT_SA,
            "--ca-file",
            ca_file.to_str().unwrap(),
            "--",
            "true",
        ],
    );
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(
        !out.status.success(),
        "#3113 R7: the connection was accepted when it should have been refused:\n{stderr}"
    );
    // Whatever the reason, the answer is never to stop checking.
    for forbidden in [
        "insecure",
        "skip-tls-verify",
        "skip_verify",
        "danger_accept",
    ] {
        assert!(
            !stderr.to_ascii_lowercase().contains(forbidden),
            "#3113 R7: the diagnostic offered `{forbidden}` as a way out:\n{stderr}"
        );
    }
    stderr
}

/// R7: a bundle from another CA pool. The fix is to get the right bundle, and
/// the message says which two things to compare.
#[test]
fn an_unrelated_ca_is_refused_and_named_as_such() {
    let served = test_ca("the-fleets-ca");
    let (cert, key) = test_leaf(&served, SERVICE_DNS);
    let stranger = test_ca("someone-elses-ca");
    let upstream = TlsUpstream::start(SERVICE_DNS, &cert, &key, &served.cert.pem());
    let stderr = refused_connect(&stranger.cert.pem(), upstream.port());
    assert!(
        stderr.contains("not signed by"),
        "#3113 R7: an unrelated CA must be diagnosed as a trust mismatch:\n{stderr}"
    );
    assert!(
        stderr.contains("deployment administrator") || stderr.contains("public CA"),
        "#3113 R7/R4: the message must say the public CA is supplied separately:\n{stderr}"
    );
    let _ = upstream.seen();
}

/// R7: the right CA, the wrong Service. The message names the flag that fixes
/// it — and not a certificate for `localhost`, which would be valid against
/// every port-forward anyone opens.
#[test]
fn a_certificate_for_another_service_is_refused_by_name() {
    let ca = test_ca("the-fleets-ca");
    let (cert, key) = test_leaf(&ca, "other.lumen-prod.svc");
    let upstream = TlsUpstream::start("other.lumen-prod.svc", &cert, &key, &ca.cert.pem());
    let stderr = refused_connect(&ca.cert.pem(), upstream.port());
    assert!(
        stderr.contains(SERVICE_DNS) && stderr.contains("--server-name"),
        "#3113 R7: a name mismatch must name the expected identity and the flag that \
         overrides it:\n{stderr}"
    );
    let _ = upstream.seen();
    assert!(
        !stderr.contains("localhost") && !stderr.contains("127.0.0.1 certificate"),
        "#3113 R7: a localhost certificate is not a remediation path:\n{stderr}"
    );
}

/// R7: a fleet still serving cleartext is a deployment fact, and reads as one
/// rather than as a handshake failure.
#[test]
fn a_cleartext_fleet_is_diagnosed_rather_than_reported_as_a_handshake_failure() {
    let ca = test_ca("the-fleets-ca");
    let upstream = RecordingUpstream::start();
    let stderr = refused_connect(&ca.cert.pem(), upstream.port());
    assert!(
        stderr.contains("cleartext"),
        "#3113 R7: a plaintext far end must be named as one:\n{stderr}"
    );
    assert!(
        stderr.contains("--plaintext") && stderr.contains("servingTlsSecret"),
        "#3113 R7: both remediations — development opt-in and production issuance — must \
         appear:\n{stderr}"
    );
    let _ = upstream.seen();
}

/// R1/R7: cleartext is a decision, not a default, and there is no flag that
/// turns verification off.
#[test]
fn transport_must_be_chosen_and_verification_cannot_be_switched_off() {
    let fixture = Fixture::new();
    let apiserver = FakeApiserver::start(Grant::Issue {
        service_account: CLIENT_SA,
    });

    let neither = fixture.run(
        &apiserver,
        0,
        &[
            "connect",
            "--namespace",
            NAMESPACE,
            "--cr",
            "search",
            "--",
            "true",
        ],
    );
    let stderr = String::from_utf8_lossy(&neither.stderr).into_owned();
    assert!(
        !neither.status.success(),
        "#3113 R1: a run that names neither --ca-file nor --plaintext must not silently pick \
         cleartext:\n{stderr}"
    );
    assert!(
        stderr.contains("--ca-file") && stderr.contains("--plaintext"),
        "#3113 R1: the refusal must name both transports:\n{stderr}"
    );

    let skipped = fixture.run(
        &apiserver,
        0,
        &[
            "connect",
            "--namespace",
            NAMESPACE,
            "--cr",
            "search",
            "--insecure-skip-tls-verify",
            "--",
            "true",
        ],
    );
    assert!(
        !skipped.status.success(),
        "#3113 R7: `--insecure-skip-tls-verify` must not exist"
    );

    let help = fixture.run(&apiserver, 0, &["connect", "--help"]);
    let help = String::from_utf8_lossy(&help.stdout).to_ascii_lowercase();
    for forbidden in ["insecure", "skip-tls-verify", "no-verify"] {
        assert!(
            !help.contains(forbidden),
            "#3113 R7: `lumen connect --help` advertises `{forbidden}`"
        );
    }
}
