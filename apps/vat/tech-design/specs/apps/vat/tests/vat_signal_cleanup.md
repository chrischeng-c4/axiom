---
id: apps-vat-tests-vat-signal-cleanup-rs
summary: Lossless rust-source-unit coverage for `apps/vat/tests/vat_signal_cleanup.rs`.
fill_sections: [rust-source-unit, changes]
---

# Fillback apps/vat/tests/vat_signal_cleanup.rs

## Source
<!-- type: rust-source-unit lang: rust -->

```rust
#![cfg(unix)]

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::Value;

fn vat_bin() -> &'static str {
    env!("CARGO_BIN_EXE_vat")
}

fn write_executable(path: &Path, source: &str) {
    fs::write(path, source).expect("write fixture executable");
    let mut permissions = fs::metadata(path).expect("fixture metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("make fixture executable");
}

fn read_pid(path: &Path) -> Option<u32> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

fn process_exists(pid: u32) -> bool {
    let result = unsafe { libc::kill(pid as libc::pid_t, 0) };
    result == 0
        || std::io::Error::last_os_error()
            .raw_os_error()
            .is_some_and(|error| error == libc::EPERM)
}

fn assert_process_group_absent(pgid: u32) {
    let result = unsafe { libc::kill(-(pgid as libc::pid_t), 0) };
    assert_eq!(result, -1, "owned process group {pgid} is still reachable");
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ESRCH),
        "owned process group {pgid} absence was not proven"
    );
}

fn wait_until(mut predicate: impl FnMut() -> bool, label: &str) {
    assert!(
        wait_until_for(Duration::from_secs(10), &mut predicate),
        "timed out waiting for {label}"
    );
}

fn wait_until_for(timeout: Duration, predicate: &mut impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if predicate() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    false
}

fn vat_log_snapshot(vat_home: &Path) -> String {
    let Some((meta_path, meta)) = read_single_meta(vat_home) else {
        return "no readable VAT metadata".to_string();
    };
    let logs_dir = meta_path.parent().expect("VAT directory").join("logs");
    let logs = fs::read_dir(logs_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| {
            format!(
                "{}={:?}",
                entry.file_name().to_string_lossy(),
                fs::read_to_string(entry.path()).unwrap_or_else(|error| error.to_string())
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    format!("metadata={meta}; logs={logs}")
}

struct TestProcessGuard {
    child: Child,
    group_leader_markers: Vec<PathBuf>,
    pid_markers: Vec<PathBuf>,
    disarmed: bool,
}

impl TestProcessGuard {
    fn new(child: Child, group_leader_markers: Vec<PathBuf>, pid_markers: Vec<PathBuf>) -> Self {
        Self {
            child,
            group_leader_markers,
            pid_markers,
            disarmed: false,
        }
    }

    fn disarm(&mut self) {
        self.disarmed = true;
    }
}

impl Drop for TestProcessGuard {
    fn drop(&mut self) {
        if self.disarmed {
            return;
        }
        // Stop VAT first so the failed test has only this guard as cleanup
        // owner, then kill only PGIDs/PIDs written by the test fixtures.
        let _ = unsafe { libc::kill(self.child.id() as libc::pid_t, libc::SIGKILL) };
        for marker in &self.group_leader_markers {
            if let Some(pgid) = read_pid(marker) {
                let _ = unsafe { libc::kill(-(pgid as libc::pid_t), libc::SIGKILL) };
            }
        }
        for marker in &self.pid_markers {
            if let Some(pid) = read_pid(marker) {
                let _ = unsafe { libc::kill(pid as libc::pid_t, libc::SIGKILL) };
            }
        }
        let _ = self.child.wait();
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            let any_live = self
                .pid_markers
                .iter()
                .filter_map(|marker| read_pid(marker))
                .any(process_exists);
            if !any_live {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

struct ExternalListenerGuard {
    listener: TcpListener,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl ExternalListenerGuard {
    fn endpoint(&self) -> std::net::SocketAddr {
        self.listener.local_addr().expect("external endpoint")
    }
}

impl Drop for ExternalListenerGuard {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn read_single_meta(vat_home: &Path) -> Option<(PathBuf, Value)> {
    let vats = vat_home.join("vats");
    let entries = fs::read_dir(vats)
        .ok()?
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    if entries.len() != 1 {
        return None;
    }
    let path = entries[0].path().join("meta.json");
    let value = serde_json::from_slice(&fs::read(&path).ok()?).ok()?;
    Some((path, value))
}

fn start_external_listener() -> ExternalListenerGuard {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind unrelated listener");
    listener
        .set_nonblocking(true)
        .expect("set unrelated listener nonblocking");
    let server = listener.try_clone().expect("clone unrelated listener");
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop);
    let thread = std::thread::spawn(move || {
        while !thread_stop.load(Ordering::Acquire) {
            match server.accept() {
                Ok((mut stream, _)) => {
                    let _ = stream.write_all(b"external-ok");
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(_) => return,
            }
        }
    });
    ExternalListenerGuard {
        listener,
        stop,
        thread: Some(thread),
    }
}

fn assert_external_listener_replies(endpoint: std::net::SocketAddr) {
    let mut stream = TcpStream::connect_timeout(&endpoint, Duration::from_secs(1))
        .expect("unrelated listener must remain reachable");
    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .expect("set external read timeout");
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).expect("read external reply");
    assert_eq!(bytes, b"external-ok");
}

fn run_signal_cleanup_case(first_signal: i32, second_signal: i32, expected_exit: i32) {
    let project = tempfile::tempdir().expect("project tempdir");
    let vat_home = tempfile::tempdir().expect("VAT_HOME tempdir");
    let markers = tempfile::tempdir().expect("marker tempdir");
    let service_leader = markers.path().join("service-leader.pid");
    let service_descendant = markers.path().join("service-descendant.pid");
    let runner_leader = markers.path().join("runner-leader.pid");
    let runner_descendant = markers.path().join("runner-descendant.pid");

    write_executable(
        &project.path().join("stubborn_server.py"),
        r#"#!/usr/bin/env python3
import os, signal, socket, sys
signal.signal(signal.SIGINT, signal.SIG_IGN)
signal.signal(signal.SIGTERM, signal.SIG_IGN)
sock = socket.socket()
sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
sock.bind(("127.0.0.1", int(sys.argv[1])))
sock.listen(16)
with open(sys.argv[2], "w") as marker:
    marker.write(str(os.getpid()))
while True:
    conn, _ = sock.accept()
    conn.close()
"#,
    );
    write_executable(
        &project.path().join("stubborn_runner.py"),
        r#"#!/usr/bin/env python3
import os, signal, sys, time
signal.signal(signal.SIGINT, signal.SIG_IGN)
signal.signal(signal.SIGTERM, signal.SIG_IGN)
with open(sys.argv[1], "w") as marker:
    marker.write(str(os.getpid()))
while True:
    time.sleep(1)
"#,
    );

    let external_listener = start_external_listener();
    let external_endpoint = external_listener.endpoint();
    fs::write(
        project.path().join("vat.toml"),
        format!(
            r#"
version = 1
default_runner = "interrupt"

[workspace]
base = "."
workdir = "."
keep = "never"

[env]
SERVICE_LEADER = "{}"
SERVICE_DESCENDANT = "{}"
RUNNER_LEADER = "{}"
RUNNER_DESCENDANT = "{}"

[[services]]
id = "external"
external = {{ host = "127.0.0.1", port = {} }}
timeout_s = 5

[[services]]
id = "owned"
cmd = ["sh", "-c", "trap '' INT TERM; printf '%s' $$ > \"$SERVICE_LEADER\"; python3 \"$VAT_CONFIG_ROOT/stubborn_server.py\" \"$1\" \"$SERVICE_DESCENDANT\" & descendant=$!; wait \"$descendant\"", "--", "{{port}}"]
timeout_s = 5

[[runners]]
id = "interrupt"
requires = ["external", "owned"]
cmd = ["sh", "-c", "trap '' INT TERM; printf '%s' $$ > \"$RUNNER_LEADER\"; python3 \"$VAT_CONFIG_ROOT/stubborn_runner.py\" \"$RUNNER_DESCENDANT\" & descendant=$!; wait \"$descendant\""]
"#,
            service_leader.display(),
            service_descendant.display(),
            runner_leader.display(),
            runner_descendant.display(),
            external_endpoint.port(),
        ),
    )
    .expect("write signal vat.toml");

    let vat = Command::new(vat_bin())
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .arg("run")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn vat run");
    let mut vat = TestProcessGuard::new(
        vat,
        vec![service_leader.clone(), runner_leader.clone()],
        vec![
            service_leader.clone(),
            service_descendant.clone(),
            runner_leader.clone(),
            runner_descendant.clone(),
        ],
    );

    let mut observed_port = None;
    let mut ready = || {
        let Some((_, meta)) = read_single_meta(vat_home.path()) else {
            return false;
        };
        let owned = meta["test_run"]["services"]
            .as_array()
            .and_then(|services| services.iter().find(|service| service["id"] == "owned"));
        let runner_running = meta["test_run"]["runner"]["status"] == "running"
            && meta["test_run"]["runner"]["pid"].as_u64().is_some();
        let owned_ready = owned.is_some_and(|service| {
            observed_port = service["port"].as_u64().map(|port| port as u16);
            service["status"] == "ready" && service["pid"].as_u64().is_some()
        });
        owned_ready
            && runner_running
            && read_pid(&service_leader).is_some()
            && read_pid(&service_descendant).is_some()
            && read_pid(&runner_leader).is_some()
            && read_pid(&runner_descendant).is_some()
    };
    assert!(
        wait_until_for(Duration::from_secs(20), &mut ready),
        "timed out waiting for owned service and runner readiness: {}",
        vat_log_snapshot(vat_home.path())
    );

    let owned_port = observed_port.expect("owned service port");
    assert_external_listener_replies(external_endpoint);
    let vat_pid = vat.child.id();
    assert_eq!(
        unsafe { libc::kill(vat_pid as libc::pid_t, first_signal) },
        0
    );
    std::thread::sleep(Duration::from_millis(25));
    assert_eq!(
        unsafe { libc::kill(vat_pid as libc::pid_t, second_signal) },
        0,
        "second signal must reach the live VAT process so first-arrival precedence is proven"
    );

    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = vat.child.try_wait().expect("poll vat run") {
            break status;
        }
        if Instant::now() >= deadline {
            panic!("vat run did not finish bounded signal cleanup");
        }
        std::thread::sleep(Duration::from_millis(25));
    };
    assert_eq!(status.code(), Some(expected_exit));

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    vat.child
        .stdout
        .take()
        .expect("vat stdout")
        .read_to_end(&mut stdout)
        .expect("read vat stdout");
    vat.child
        .stderr
        .take()
        .expect("vat stderr")
        .read_to_end(&mut stderr)
        .expect("read vat stderr");

    let (meta_path, meta) = read_single_meta(vat_home.path()).expect("terminal VAT metadata");
    assert_eq!(meta["status"]["state"], "interrupted", "{meta}");
    assert_eq!(meta["status"]["signal"], first_signal, "{meta}");
    assert!(meta["status"]["reason"]
        .as_str()
        .is_some_and(|reason| reason.contains(if first_signal == libc::SIGINT {
            "SIGINT"
        } else {
            "SIGTERM"
        })));
    let runner = &meta["test_run"]["runner"];
    assert_eq!(runner["status"], "interrupted", "{meta}");
    assert!(runner["pid"].is_null(), "{meta}");
    assert_eq!(runner["exit_code"], expected_exit, "{meta}");
    let runners = meta["test_run"]["runners"]
        .as_array()
        .expect("runner evidence list");
    assert!(!runners.is_empty(), "{meta}");
    assert!(
        runners.iter().all(|runner| {
            runner["status"] == "interrupted"
                && runner["pid"].is_null()
                && runner["exit_code"] == expected_exit
        }),
        "{meta}"
    );
    let services = meta["test_run"]["services"]
        .as_array()
        .expect("service evidence");
    let owned = services
        .iter()
        .find(|service| service["id"] == "owned")
        .expect("owned service evidence");
    assert_eq!(owned["status"], "interrupted", "{owned}");
    assert!(owned["pid"].is_null(), "{owned}");
    let external = services
        .iter()
        .find(|service| service["id"] == "external")
        .expect("external service evidence");
    assert_eq!(external["owned_by_vat"], false, "{external}");
    assert!(external["pid"].is_null(), "{external}");

    for path in [
        &service_leader,
        &service_descendant,
        &runner_leader,
        &runner_descendant,
    ] {
        let pid = read_pid(path).expect("owned pid marker");
        wait_until(|| !process_exists(pid), &format!("owned pid {pid} absence"));
    }
    assert_process_group_absent(read_pid(&service_leader).expect("service PGID marker"));
    assert_process_group_absent(read_pid(&runner_leader).expect("runner PGID marker"));
    TcpListener::bind(("127.0.0.1", owned_port))
        .unwrap_or_else(|error| panic!("owned service port {owned_port} did not rebind: {error}"));
    assert_external_listener_replies(external_endpoint);
    vat.disarm();

    let vat_id = meta["id"].as_str().expect("VAT id");
    let state = Command::new(vat_bin())
        .env("VAT_HOME", vat_home.path())
        .args(["state", vat_id, "--compact"])
        .output()
        .expect("inspect interrupted state");
    assert!(state.status.success());
    let projected: Value = serde_json::from_slice(&state.stdout).expect("state JSON");
    assert_eq!(projected["status"]["state"], "interrupted");
    assert_eq!(projected["status"]["signal"], first_signal);

    let listed = Command::new(vat_bin())
        .env("VAT_HOME", vat_home.path())
        .arg("ls")
        .output()
        .expect("list interrupted VAT");
    assert!(listed.status.success());
    let listed = String::from_utf8_lossy(&listed.stdout);
    assert!(listed.contains(vat_id), "{listed}");
    assert!(
        listed.contains(if first_signal == libc::SIGINT {
            "interrupted:SIGINT"
        } else {
            "interrupted:SIGTERM"
        }),
        "{listed}"
    );

    for execute in [false, true] {
        let mut gc_command = Command::new(vat_bin());
        gc_command
            .env("VAT_HOME", vat_home.path())
            .args(["gc", "--json", "--keep-last", "0"]);
        if execute {
            gc_command.arg("--execute");
        }
        let gc = gc_command.output().expect("inspect GC classification");
        assert!(gc.status.success());
        let report: Value = serde_json::from_slice(&gc.stdout).expect("GC JSON");
        let entry = report["entries"]
            .as_array()
            .and_then(|entries| entries.iter().find(|entry| entry["id"] == vat_id))
            .expect("interrupted GC entry");
        assert_eq!(entry["candidate"], false, "{entry}");
        assert_eq!(entry["reason"], "interrupted_retained", "{entry}");
        assert!(entry["status"]
            .as_str()
            .is_some_and(|status| status.starts_with("interrupted:")));
        assert!(meta_path.exists(), "GC must retain interrupted evidence");
    }

    let events = String::from_utf8_lossy(&stdout);
    assert!(events.contains("\"code\":\"run_interrupted\""), "{events}");
    assert!(
        events.contains(&format!("\"signal\":{first_signal}")),
        "{events}"
    );
    assert!(
        stderr.is_empty(),
        "unexpected vat stderr: {}\nmetadata: {}",
        String::from_utf8_lossy(&stderr),
        meta_path.display()
    );
}

fn run_direct_signal_case(first_signal: i32, second_signal: i32, expected_exit: i32) {
    let project = tempfile::tempdir().expect("direct project tempdir");
    let vat_home = tempfile::tempdir().expect("direct VAT_HOME tempdir");
    let markers = tempfile::tempdir().expect("direct marker tempdir");
    let leader = markers.path().join("direct-leader.pid");
    let descendant = markers.path().join("direct-descendant.pid");
    write_executable(
        &project.path().join("stubborn_runner.py"),
        r#"#!/usr/bin/env python3
import os, signal, sys, time
signal.signal(signal.SIGINT, signal.SIG_IGN)
signal.signal(signal.SIGTERM, signal.SIG_IGN)
with open(sys.argv[1], "w") as marker:
    marker.write(str(os.getpid()))
while True:
    time.sleep(1)
"#,
    );

    let vat = Command::new(vat_bin())
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .env("DIRECT_LEADER", &leader)
        .env("DIRECT_DESCENDANT", &descendant)
        .args([
            "run",
            "--",
            "sh",
            "-c",
            "trap '' INT TERM; printf '%s' $$ > \"$DIRECT_LEADER\"; python3 stubborn_runner.py \"$DIRECT_DESCENDANT\" & child=$!; wait \"$child\"",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn direct vat run");
    let mut vat = TestProcessGuard::new(
        vat,
        vec![leader.clone()],
        vec![leader.clone(), descendant.clone()],
    );

    wait_until(
        || {
            read_single_meta(vat_home.path()).is_some()
                && read_pid(&leader).is_some()
                && read_pid(&descendant).is_some()
        },
        "direct owned process group",
    );
    let vat_pid = vat.child.id();
    assert_eq!(
        unsafe { libc::kill(vat_pid as libc::pid_t, first_signal) },
        0
    );
    std::thread::sleep(Duration::from_millis(25));
    assert_eq!(
        unsafe { libc::kill(vat_pid as libc::pid_t, second_signal) },
        0,
        "second signal must reach direct-mode VAT"
    );

    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = vat.child.try_wait().expect("poll direct vat run") {
            break status;
        }
        if Instant::now() >= deadline {
            panic!("direct vat run did not finish bounded signal cleanup");
        }
        std::thread::sleep(Duration::from_millis(25));
    };
    assert_eq!(status.code(), Some(expected_exit));

    let (_, meta) = read_single_meta(vat_home.path()).expect("direct terminal metadata");
    assert_eq!(meta["status"]["state"], "interrupted", "{meta}");
    assert_eq!(meta["status"]["signal"], first_signal, "{meta}");
    assert_eq!(meta["last_run"]["exit_code"], expected_exit, "{meta}");
    assert!(meta["last_run"]["finished_at"].as_str().is_some(), "{meta}");
    for path in [&leader, &descendant] {
        let pid = read_pid(path).expect("direct owned pid marker");
        wait_until(
            || !process_exists(pid),
            &format!("direct owned pid {pid} absence"),
        );
    }
    assert_process_group_absent(read_pid(&leader).expect("direct PGID marker"));
    vat.disarm();
}

#[test]
fn sigint_cleans_owned_groups_and_persists_interrupted_state() {
    run_signal_cleanup_case(libc::SIGINT, libc::SIGTERM, 130);
}

#[test]
fn sigterm_cleans_owned_groups_and_persists_interrupted_state() {
    run_signal_cleanup_case(libc::SIGTERM, libc::SIGINT, 143);
}

#[test]
fn direct_mode_sigint_and_sigterm_persist_interrupted_state() {
    for (first, second, exit) in [
        (libc::SIGINT, libc::SIGTERM, 130),
        (libc::SIGTERM, libc::SIGINT, 143),
    ] {
        run_direct_signal_case(first, second, exit);
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: "apps/vat/tests/vat_signal_cleanup.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Lossless rust-source-unit ownership created from explicit file fillback.
```
