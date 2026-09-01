//! Parse-time and pre-I/O coverage for standalone backup and restore.

#![cfg(unix)]

#[cfg(feature = "backup")]
use std::fs;
#[cfg(all(feature = "backup", feature = "delegated-auth"))]
use std::io::{BufRead, BufReader};
#[cfg(feature = "backup")]
use std::io::{Read, Write};
#[cfg(all(feature = "backup", feature = "delegated-auth"))]
use std::net::{TcpListener, TcpStream};
#[cfg(feature = "backup")]
use std::path::Path;
use std::process::{Command, Output};
#[cfg(feature = "backup")]
use std::thread::{self, JoinHandle};
#[cfg(all(feature = "backup", feature = "delegated-auth"))]
use std::time::{Duration, Instant};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_lumen"))
        .args(args)
        .output()
        .expect("run lumen")
}

fn failed(output: Output) {
    assert!(!output.status.success(), "unexpected success: {output:?}");
}

// This assertion helper is shared by disabled, partial, and full feature builds.
fn failed_with(output: Output, expected: &str) {
    assert!(!output.status.success(), "unexpected success: {output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected),
        "stderr did not contain {expected:?}: {stderr}"
    );
    assert!(
        !stderr.contains("No such file") && !stderr.contains("missing"),
        "{stderr}"
    );
}

#[cfg(feature = "backup")]
fn compose(path: &Path, port: &str, managed: bool) {
    let label = if managed { "true" } else { "false" };
    fs::write(
        path,
        format!(
            "services:\n  lumen:\n    image: ghcr.io/chrischeng-c4/lumen:0.4.31\n    ports:\n      - '{port}'\n    labels:\n      com.axiom.lumen.managed: '{label}'\n"
        ),
    )
    .unwrap();
}

#[cfg(feature = "backup")]
fn serve_once(status: u16, body: &'static [u8]) -> JoinHandle<Vec<u8>> {
    let listener = std::net::TcpListener::bind("127.0.0.1:7373").unwrap();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = vec![0; 16 * 1024];
        let size = stream.read(&mut request).unwrap();
        let reason = match status {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            _ => "Error",
        };
        let response = format!(
            "HTTP/1.1 {status} {reason}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
        stream.write_all(body).unwrap();
        request.truncate(size);
        request
    })
}

#[cfg(feature = "backup")]
#[test]
fn standalone_backup_restore_rejects_invalid_invocations_before_io() {
    failed(run(&["standalone", "backup", "--out", "x"]));
    failed(run(&[
        "standalone",
        "backup",
        "--compose",
        "a",
        "--gke",
        "b",
        "--out",
        "x",
    ]));
    failed(run(&[
        "standalone",
        "backup",
        "--gke",
        "b",
        "--name",
        "x",
        "--out",
        "x",
    ]));
    failed(run(&[
        "standalone",
        "restore",
        "--compose",
        "a",
        "--file",
        "b",
    ]));

    let dir = tempfile::tempdir().unwrap();
    let compose_path = dir.path().join("compose.yaml");
    let output_path = dir.path().join("backup.json");
    compose(&compose_path, "127.0.0.1:7374:7373", true);
    failed(run(&[
        "standalone",
        "backup",
        "--compose",
        compose_path.to_str().unwrap(),
        "--out",
        output_path.to_str().unwrap(),
    ]));
    compose(&compose_path, "127.0.0.1:7373:7373", false);
    failed(run(&[
        "standalone",
        "backup",
        "--compose",
        compose_path.to_str().unwrap(),
        "--out",
        output_path.to_str().unwrap(),
    ]));
    assert!(!output_path.exists());
}

#[cfg(feature = "backup")]
#[test]
fn compose_backup_and_restore_require_exact_status_and_validate_before_io() {
    let dir = tempfile::tempdir().unwrap();
    let compose_path = dir.path().join("compose.yaml");
    let output_path = dir.path().join("backup.json");
    let snapshot = br#"{"version":1,"collections":{}}"#;
    compose(&compose_path, "127.0.0.1:7373:7373", true);

    let server = serve_once(200, snapshot);
    let output = run(&[
        "standalone",
        "backup",
        "--compose",
        compose_path.to_str().unwrap(),
        "--out",
        output_path.to_str().unwrap(),
    ]);
    assert!(output.status.success(), "{output:?}");
    let request = server.join().unwrap();
    assert!(String::from_utf8_lossy(&request).starts_with("GET /admin/backup "));
    assert_eq!(fs::read(&output_path).unwrap(), snapshot);

    fs::write(&output_path, b"previous").unwrap();
    let server = serve_once(201, snapshot);
    let output = run(&[
        "standalone",
        "backup",
        "--compose",
        compose_path.to_str().unwrap(),
        "--out",
        output_path.to_str().unwrap(),
    ]);
    assert!(!output.status.success());
    server.join().unwrap();
    assert_eq!(fs::read(&output_path).unwrap(), b"previous");

    let sentinel_path = dir.path().join("sentinel.json");
    let symlink_path = dir.path().join("backup-link.json");
    fs::write(&sentinel_path, b"sentinel").unwrap();
    std::os::unix::fs::symlink(&sentinel_path, &symlink_path).unwrap();
    let server = serve_once(200, snapshot);
    let output = run(&[
        "standalone",
        "backup",
        "--compose",
        compose_path.to_str().unwrap(),
        "--out",
        symlink_path.to_str().unwrap(),
    ]);
    assert!(!output.status.success());
    server.join().unwrap();
    assert!(fs::symlink_metadata(&symlink_path)
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(fs::read(&sentinel_path).unwrap(), b"sentinel");

    let input_path = dir.path().join("input.json");
    fs::write(&input_path, snapshot).unwrap();
    let server = serve_once(204, &[]);
    let output = run(&[
        "standalone",
        "restore",
        "--compose",
        compose_path.to_str().unwrap(),
        "--file",
        input_path.to_str().unwrap(),
        "--replace",
    ]);
    assert!(output.status.success(), "{output:?}");
    let request = server.join().unwrap();
    assert!(String::from_utf8_lossy(&request).starts_with("POST /admin/restore "));

    let server = serve_once(200, b"response must not be printed");
    let output = run(&[
        "standalone",
        "restore",
        "--compose",
        compose_path.to_str().unwrap(),
        "--file",
        input_path.to_str().unwrap(),
        "--replace",
    ]);
    assert!(!output.status.success());
    server.join().unwrap();
    assert!(!String::from_utf8_lossy(&output.stderr).contains("response must not be printed"));

    fs::write(&input_path, b"not a snapshot").unwrap();
    let output = run(&[
        "standalone",
        "restore",
        "--compose",
        compose_path.to_str().unwrap(),
        "--file",
        input_path.to_str().unwrap(),
        "--replace",
    ]);
    assert!(!output.status.success());
}

#[cfg(not(feature = "backup"))]
#[test]
fn standalone_backup_restore_refuses_before_reading_inputs() {
    failed_with(
        run(&[
            "standalone",
            "backup",
            "--compose",
            "/definitely/missing/compose.yaml",
            "--out",
            "/definitely/missing/backup.json",
        ]),
        "standalone backup requires the `backup` feature",
    );
    failed_with(
        run(&[
            "standalone",
            "restore",
            "--compose",
            "/definitely/missing/compose.yaml",
            "--file",
            "/definitely/missing/backup.json",
            "--replace",
        ]),
        "standalone restore requires the `backup` feature",
    );
    failed(run(&[
        "standalone",
        "backup",
        "--gke",
        "/definitely/missing/lumen.yaml",
        "--name",
        "x",
        "--out",
        "/definitely/missing/backup.json",
    ]));
    failed(run(&[
        "standalone",
        "restore",
        "--gke",
        "/definitely/missing/lumen.yaml",
        "--file",
        "/definitely/missing/backup.json",
    ]));
}

#[cfg(all(feature = "backup", not(feature = "delegated-auth")))]
#[test]
fn gke_refuses_before_reading_config_or_snapshot_without_delegated_auth() {
    failed_with(
        run(&[
            "standalone",
            "backup",
            "--gke",
            "/definitely/missing/lumen.yaml",
            "--out",
            "/definitely/missing/backup.json",
        ]),
        "GKE backup requires the `delegated-auth` feature",
    );
    failed_with(
        run(&[
            "standalone",
            "restore",
            "--gke",
            "/definitely/missing/lumen.yaml",
            "--file",
            "/definitely/missing/backup.json",
            "--replace",
        ]),
        "GKE restore requires the `delegated-auth` feature",
    );
}

#[cfg(all(feature = "backup", feature = "delegated-auth"))]
#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    authorization: String,
    body: Vec<u8>,
}

#[cfg(all(feature = "backup", feature = "delegated-auth"))]
fn read_http_request(stream: &TcpStream) -> HttpRequest {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut first = String::new();
    if reader.read_line(&mut first).unwrap() == 0 {
        return HttpRequest {
            method: String::new(),
            path: String::new(),
            authorization: String::new(),
            body: Vec::new(),
        };
    }
    let mut content_length = 0usize;
    let mut authorization = String::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap() == 0 {
            break;
        }
        if line == "\r\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse().unwrap();
            } else if name.eq_ignore_ascii_case("authorization") {
                authorization = value.trim().to_string();
            }
        }
    }
    let mut body = vec![0; content_length];
    reader.read_exact(&mut body).unwrap();
    let mut fields = first.split_whitespace();
    let path = fields.next().unwrap_or_default();
    HttpRequest {
        method: path.into(),
        path: fields
            .next()
            .unwrap_or_default()
            .split('?')
            .next()
            .unwrap_or_default()
            .into(),
        authorization,
        body,
    }
}

#[cfg(all(feature = "backup", feature = "delegated-auth"))]
fn write_http_response(stream: &mut TcpStream, status: u16, body: &[u8]) {
    let reason = match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        _ => "Error",
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes()).unwrap();
    stream.write_all(body).unwrap();
}

#[cfg(all(feature = "backup", feature = "delegated-auth"))]
const GKE_CANARY: &str = "canary-gke-backup-token-must-not-escape";

#[cfg(all(feature = "backup", feature = "delegated-auth"))]
fn fake_apiserver() -> (String, JoinHandle<HttpRequest>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_http_request(&stream);
        let response = br#"{"apiVersion":"authentication.k8s.io/v1","kind":"TokenRequest","spec":{"audiences":[],"expirationSeconds":600},"status":{"token":"canary-gke-backup-token-must-not-escape","expirationTimestamp":"2099-01-01T00:00:00Z"}}"#;
        write_http_response(&mut stream, 201, response);
        request
    });
    (address, handle)
}

#[cfg(all(feature = "backup", feature = "delegated-auth"))]
fn fake_lumen(status: u16, body: Vec<u8>) -> (u16, JoinHandle<HttpRequest>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        for accepted in listener.incoming() {
            let mut stream = accepted.unwrap();
            let request = read_http_request(&stream);
            if request.method.is_empty() {
                continue;
            }
            write_http_response(&mut stream, status, &body);
            return request;
        }
        panic!("fake Lumen listener closed before a request");
    });
    (port, handle)
}

#[cfg(all(feature = "backup", feature = "delegated-auth"))]
fn write_gke_config(path: &Path) {
    fs::write(
        path,
        "name: oracle\nnamespace: lumen-prod\nnodePool: pool\ncpu: 1\nmemory: 1Gi\nstorageSize: 20Gi\nstorageClass: premium-rwo\nallowedServiceAccounts:\n  - client/app\n",
    )
    .unwrap();
}

#[cfg(all(feature = "backup", feature = "delegated-auth"))]
fn write_kubeconfig(path: &Path, server: &str) {
    fs::write(
        path,
        format!(
            "apiVersion: v1\nkind: Config\ncurrent-context: fake\nclusters:\n- name: fake\n  cluster:\n    server: {server}\ncontexts:\n- name: fake\n  context:\n    cluster: fake\n    user: caller\nusers:\n- name: caller\n  user:\n    token: caller-only\n"
        ),
    )
    .unwrap();
}

#[cfg(all(feature = "backup", feature = "delegated-auth"))]
fn install_gke_kubectl(dir: &Path, log: &Path, pid_file: &Path, stall: bool) {
    let forwarder = dir.join("forwarder.py");
    fs::write(
        &forwarder,
        r#"import socket, sys, threading
local, upstream = int(sys.argv[1]), int(sys.argv[2])
server = socket.socket()
server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
server.bind(("127.0.0.1", local))
server.listen(8)
def pump(src, dst):
    try:
        while True:
            data = src.recv(65536)
            if not data: break
            dst.sendall(data)
    except OSError: pass
    finally:
        try: dst.shutdown(socket.SHUT_WR)
        except OSError: pass
while True:
    near, _ = server.accept()
    try: far = socket.create_connection(("127.0.0.1", upstream))
    except OSError:
        near.close(); continue
    threading.Thread(target=pump, args=(near, far), daemon=True).start()
    threading.Thread(target=pump, args=(far, near), daemon=True).start()
"#,
    )
    .unwrap();
    let mode = if stall { "1" } else { "0" };
    let script = format!(
        "#!/bin/sh\necho $$ > '{pid}'\nprintf '%s\\n' \"$*\" >> '{log}'\nif [ '{mode}' = 1 ]; then exec sleep 300; fi\nfor a in \"$@\"; do case \"$a\" in *:7373) mapping=\"$a\";; esac; done\nexec python3 '{forwarder}' \"${{mapping%%:*}}\" \"$LUMEN_TEST_UPSTREAM_PORT\"\n",
        pid = pid_file.display(),
        log = log.display(),
        mode = mode,
        forwarder = forwarder.display(),
    );
    let path = dir.join("kubectl");
    fs::write(&path, script).unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(all(feature = "backup", feature = "delegated-auth"))]
fn run_gke_command(
    tmp: &Path,
    bin_dir: &Path,
    kubeconfig: &Path,
    upstream_port: u16,
    args: &[&str],
) -> Command {
    let path = format!("{}:{}", bin_dir.display(), std::env::var("PATH").unwrap());
    let mut command = Command::new(env!("CARGO_BIN_EXE_lumen"));
    command
        .env_clear()
        .env("PATH", path)
        .env(
            "HOME",
            std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()),
        )
        .env("KUBECONFIG", kubeconfig)
        .env("LUMEN_TEST_UPSTREAM_PORT", upstream_port.to_string())
        .current_dir(tmp)
        .args(args);
    command
}

#[cfg(all(feature = "backup", feature = "delegated-auth"))]
fn assert_no_canary(path: &Path) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let child = entry.path();
        if child.is_dir() {
            assert_no_canary(&child);
        } else {
            let bytes = fs::read(&child).unwrap();
            assert!(
                !String::from_utf8_lossy(&bytes).contains(GKE_CANARY),
                "{}",
                child.display()
            );
        }
    }
}

#[cfg(all(feature = "backup", feature = "delegated-auth"))]
fn assert_process_stopped(pid_file: &Path) {
    let pid = fs::read_to_string(pid_file)
        .unwrap()
        .trim()
        .parse::<u32>()
        .unwrap();
    assert!(!Command::new("/bin/kill")
        .args(["-0", &pid.to_string()])
        .stderr(std::process::Stdio::null())
        .status()
        .unwrap()
        .success());
}

#[cfg(all(feature = "backup", feature = "delegated-auth"))]
#[test]
fn gke_backup_mints_default_audience_and_forwards_once() {
    let tmp = tempfile::tempdir().unwrap();
    let bin_dir = tmp.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();
    let kubectl_log = tmp.path().join("kubectl.log");
    let pid_file = tmp.path().join("kubectl.pid");
    install_gke_kubectl(&bin_dir, &kubectl_log, &pid_file, false);
    let config = tmp.path().join("lumen.yaml");
    write_gke_config(&config);
    let kubeconfig = tmp.path().join("kubeconfig");
    let (api_url, api_thread) = fake_apiserver();
    write_kubeconfig(&kubeconfig, &api_url);
    let snapshot = br#"{"version":1,"collections":{}}"#.to_vec();
    let (upstream_port, lumen_thread) = fake_lumen(200, snapshot.clone());
    let output_path = tmp.path().join("backup.json");
    let output = run_gke_command(
        tmp.path(),
        &bin_dir,
        &kubeconfig,
        upstream_port,
        &[
            "standalone",
            "backup",
            "--gke",
            config.to_str().unwrap(),
            "--out",
            output_path.to_str().unwrap(),
        ],
    )
    .output()
    .unwrap();
    assert!(output.status.success(), "{output:?}");
    let token_request = api_thread.join().unwrap();
    assert_eq!(token_request.method, "POST");
    assert_eq!(
        token_request.path,
        "/api/v1/namespaces/lumen-prod/serviceaccounts/oracle-admin/token"
    );
    let token_body: serde_json::Value = serde_json::from_slice(&token_request.body).unwrap();
    assert_eq!(
        token_body["kind"], "TokenRequest",
        "the formal GKE backup flow must use only the renderer-owned admin KSA"
    );
    assert_eq!(token_body["spec"]["audiences"], serde_json::json!([]));
    assert_eq!(token_body["spec"]["expirationSeconds"], 600);
    let lumen_request = lumen_thread.join().unwrap();
    assert_eq!(lumen_request.method, "GET");
    assert_eq!(lumen_request.path, "/admin/backup");
    assert_eq!(lumen_request.authorization, format!("Bearer {GKE_CANARY}"));
    assert_eq!(fs::read(&output_path).unwrap(), snapshot);
    assert_process_stopped(&pid_file);
    let invocations: Vec<_> = fs::read_to_string(&kubectl_log)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    assert_eq!(invocations.len(), 1, "{invocations:?}");
    let args: Vec<_> = invocations[0].split_whitespace().collect();
    assert_eq!(
        &args[..4],
        ["port-forward", "-n", "lumen-prod", "svc/oracle"]
    );
    assert!(args[4].split_once(':').is_some_and(|(port, target)| {
        !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) && target == "7373"
    }));
    assert_no_canary(tmp.path());
    assert!(!String::from_utf8_lossy(&output.stdout).contains(GKE_CANARY));
    assert!(!String::from_utf8_lossy(&output.stderr).contains(GKE_CANARY));
}

#[cfg(all(feature = "backup", feature = "delegated-auth"))]
#[test]
fn gke_restore_validates_then_posts_original_snapshot_once() {
    let tmp = tempfile::tempdir().unwrap();
    let bin_dir = tmp.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();
    let kubectl_log = tmp.path().join("kubectl.log");
    let pid_file = tmp.path().join("kubectl.pid");
    install_gke_kubectl(&bin_dir, &kubectl_log, &pid_file, false);
    let config = tmp.path().join("lumen.yaml");
    write_gke_config(&config);
    let kubeconfig = tmp.path().join("kubeconfig");
    let (api_url, api_thread) = fake_apiserver();
    write_kubeconfig(&kubeconfig, &api_url);
    let snapshot = br#"{"version":1,"collections":{}}"#.to_vec();
    let input = tmp.path().join("backup.json");
    fs::write(&input, &snapshot).unwrap();
    let (upstream_port, lumen_thread) = fake_lumen(204, Vec::new());
    let output = run_gke_command(
        tmp.path(),
        &bin_dir,
        &kubeconfig,
        upstream_port,
        &[
            "standalone",
            "restore",
            "--gke",
            config.to_str().unwrap(),
            "--file",
            input.to_str().unwrap(),
            "--replace",
        ],
    )
    .output()
    .unwrap();
    assert!(output.status.success(), "{output:?}");
    let token_request = api_thread.join().unwrap();
    assert_eq!(token_request.method, "POST");
    assert_eq!(
        token_request.path,
        "/api/v1/namespaces/lumen-prod/serviceaccounts/oracle-admin/token"
    );
    let lumen_request = lumen_thread.join().unwrap();
    assert_eq!(lumen_request.method, "POST");
    assert_eq!(lumen_request.path, "/admin/restore");
    assert_eq!(lumen_request.authorization, format!("Bearer {GKE_CANARY}"));
    assert_eq!(lumen_request.body, snapshot);
    let invocations: Vec<_> = fs::read_to_string(&kubectl_log)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    assert_eq!(invocations.len(), 1, "{invocations:?}");
    let args: Vec<_> = invocations[0].split_whitespace().collect();
    assert_eq!(
        &args[..4],
        ["port-forward", "-n", "lumen-prod", "svc/oracle"]
    );
    assert!(args[4].split_once(':').is_some_and(|(port, target)| {
        !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) && target == "7373"
    }));
    assert_process_stopped(&pid_file);
    assert!(String::from_utf8_lossy(&output.stdout).contains("restore complete"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains(GKE_CANARY));
    assert!(!String::from_utf8_lossy(&output.stderr).contains(GKE_CANARY));
    assert_no_canary(tmp.path());
}

#[cfg(all(feature = "backup", feature = "delegated-auth"))]
#[test]
fn gke_restore_rejects_invalid_snapshot_before_config_or_network_io() {
    let tmp = tempfile::tempdir().unwrap();
    let bin_dir = tmp.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();
    let config = tmp.path().join("poison.yaml");
    fs::write(&config, "not: a valid standalone config: [").unwrap();
    let input = tmp.path().join("poison.json");
    fs::write(&input, b"not a SnapshotV1").unwrap();
    let output_path = tmp.path().join("should-not-exist.json");
    let kubectl_log = tmp.path().join("kubectl.log");
    let pid_file = tmp.path().join("kubectl.pid");
    let kubeconfig = tmp.path().join("no-kubeconfig");
    let output = run_gke_command(
        tmp.path(),
        &bin_dir,
        &kubeconfig,
        9,
        &[
            "standalone",
            "restore",
            "--gke",
            config.to_str().unwrap(),
            "--file",
            input.to_str().unwrap(),
            "--replace",
        ],
    )
    .output()
    .unwrap();
    failed_with(output, "decode SnapshotV1");
    assert!(!output_path.exists());
    assert!(!kubectl_log.exists());
    assert!(!pid_file.exists());
}

#[cfg(all(feature = "backup", feature = "delegated-auth"))]
#[test]
fn gke_backup_ctrl_c_reaps_stalled_port_forward() {
    let tmp = tempfile::tempdir().unwrap();
    let bin_dir = tmp.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();
    let kubectl_log = tmp.path().join("kubectl.log");
    let pid_file = tmp.path().join("kubectl.pid");
    install_gke_kubectl(&bin_dir, &kubectl_log, &pid_file, true);
    let config = tmp.path().join("lumen.yaml");
    write_gke_config(&config);
    let kubeconfig = tmp.path().join("kubeconfig");
    let (api_url, _api_thread) = fake_apiserver();
    write_kubeconfig(&kubeconfig, &api_url);
    let output_path = tmp.path().join("backup.json");
    let mut child = run_gke_command(
        tmp.path(),
        &bin_dir,
        &kubeconfig,
        9,
        &[
            "standalone",
            "backup",
            "--gke",
            config.to_str().unwrap(),
            "--out",
            output_path.to_str().unwrap(),
        ],
    )
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .spawn()
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    let pid = loop {
        if let Ok(text) = fs::read_to_string(&pid_file) {
            if let Ok(pid) = text.trim().parse::<u32>() {
                break pid;
            }
        }
        assert!(Instant::now() < deadline, "kubectl did not start");
        thread::sleep(Duration::from_millis(20));
    };
    Command::new("/bin/kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while child.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            let _ = child.kill();
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("interrupted"));
    assert!(!Command::new("/bin/kill")
        .args(["-0", &pid.to_string()])
        .stderr(std::process::Stdio::null())
        .status()
        .unwrap()
        .success());
    assert_no_canary(tmp.path());
}
