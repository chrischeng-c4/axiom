//! Regression coverage for MicroVM-published host ports.
//!
//! The default tests use a small fake `container` executable plus a real local
//! TCP listener, so they prove the VAT lifecycle deterministically without
//! requiring Apple Container on every developer or CI host. The ignored test
//! is the opt-in real-host contract gate.

use std::ffi::OsString;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
#[cfg(unix)]
use std::os::fd::AsRawFd;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;
use tempfile::TempDir;

// HANDWRITE-BEGIN gap="vat-microvm-published-port-regression" tracker="#1526" reason="Add deterministic TCP reset, HTTP round-trip, failure-evidence, cleanup, and opt-in real Apple-container published-endpoint coverage."

/// The public MicroVM availability helper resolves `container` through PATH. This
/// serializes its one test-local PATH override without changing how command-level
/// regressions inject their fake binary into child processes.
static PATH_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

struct EnvVarGuard {
    key: &'static str,
    previous: Option<OsString>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: OsString) -> Self {
        let previous = std::env::var_os(key);
        std::env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(value) = &self.previous {
            std::env::set_var(self.key, value);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

fn vat_bin() -> &'static str {
    env!("CARGO_BIN_EXE_vat")
}

fn jsonl(stdout: &[u8]) -> Vec<Value> {
    String::from_utf8_lossy(stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("VAT should emit JSONL"))
        .collect()
}

fn result_event(events: &[Value]) -> &Value {
    events
        .iter()
        .find(|event| event["type"] == "result")
        .expect("missing result event")
}

fn fake_container_path(bin_dir: &Path) -> PathBuf {
    let script = bin_dir.join("container");
    fs::create_dir_all(bin_dir).expect("create fake-bin directory");
    fs::write(
        &script,
        r#"#!/bin/sh
if [ -n "${VAT_FAKE_CONTAINER_LOG:-}" ]; then
  printf '%s\n' "$*" >> "$VAT_FAKE_CONTAINER_LOG"
fi

case "${1:-}" in
  --version)
    echo "fake-container 1.0"
    ;;
  system)
    [ "${2:-}" = "status" ] || exit 2
    if [ -e "$(dirname "$0")/.vat-fake-system-status-hang" ]; then
      exec /bin/sleep 5
    fi
    if [ -e "$(dirname "$0")/.vat-fake-system-status-failure" ]; then
      exit 42
    fi
    ;;
  image)
    case "${2:-}" in
      inspect)
        if [ -e "$(dirname "$0")/.vat-fake-image-missing" ]; then
          exit 1
        fi
        printf '{"reference":"%s"}\n' "${3:-}"
        ;;
      pull)
        rm -f "$(dirname "$0")/.vat-fake-image-missing"
        ;;
      *)
        exit 2
        ;;
    esac
    ;;
  run)
    shift
    while [ "$#" -gt 0 ]; do
      if [ "$1" = "--name" ]; then
        printf '%s\n' "$2" > "$(dirname "$0")/.vat-fake-container-live-name"
        break
      fi
      shift
    done
    if [ -n "${VAT_FAKE_CONTAINER_RUN_FAILURE_GATE:-}" ]; then
      while [ ! -e "$VAT_FAKE_CONTAINER_RUN_FAILURE_GATE" ]; do
        /bin/sleep 0.01
      done
      /bin/sleep 0.05
      exit 17
    fi
    if [ "${VAT_FAKE_CONTAINER_RUN_DELAYED_FAILURE:-}" = "1" ]; then
      /bin/sleep 0.05
      exit 17
    fi
    exec /bin/sleep 30
    ;;
  list)
    if [ -e "$(dirname "$0")/.vat-fake-container-list-error" ]; then
      exit 71
    fi
    if [ -e "$(dirname "$0")/.vat-fake-container-list-empty" ]; then
      printf '[]\n'
      exit 0
    fi
    name=$(cat "$(dirname "$0")/.vat-fake-container-live-name" 2>/dev/null || true)
    if [ -n "$name" ]; then
      printf '[{"id":"%s"}]\n' "$name"
    else
      printf '[]\n'
    fi
    ;;
  inspect)
    if [ "${VAT_FAKE_CONTAINER_INSPECT_HANG:-}" = "1" ]; then
      exec /bin/sleep 5
    fi
    echo "guest_ip=10.0.0.2 guest_port=80"
    ;;
  rm)
    if [ "${VAT_FAKE_CONTAINER_RM_HANG:-}" = "1" ]; then
      exec /bin/sleep 5
    fi
    if [ "${VAT_FAKE_CONTAINER_RM_FAILURE:-}" = "1" ]; then
      exit 23
    fi
    if [ -e "$(dirname "$0")/.vat-fake-rm-failure" ]; then
      exit 23
    fi
    /bin/rm -f "$(dirname "$0")/.vat-fake-container-live-name"
    ;;
  *)
    exit 2
    ;;
esac
"#,
    )
    .expect("write fake container");
    let mut permissions = fs::metadata(&script)
        .expect("fake container metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).expect("make fake container executable");
    script
}

fn fake_docker_path(bin_dir: &Path) -> PathBuf {
    let script = bin_dir.join("docker");
    fs::write(
        &script,
        r#"#!/bin/sh
if [ -n "${VAT_FAKE_DOCKER_LOG:-}" ]; then
  printf '%s\n' "$*" >> "$VAT_FAKE_DOCKER_LOG"
fi
exit 97
"#,
    )
    .expect("write fake docker");
    let mut permissions = fs::metadata(&script)
        .expect("fake docker metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).expect("make fake docker executable");
    script
}

fn path_with_fake_container(bin_dir: &Path) -> OsString {
    let mut paths = vec![bin_dir.to_path_buf()];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").expect("PATH is set for test process"),
    ));
    std::env::join_paths(paths).expect("join test PATH")
}

fn write_project(project: &Path, port: u16, ready_http: Option<&str>, image: &str) {
    let ready_http = ready_http
        .map(|value| format!("ready_http = \"{value}\"\n"))
        .unwrap_or_default();
    fs::write(
        project.join("vat.toml"),
        format!(
            r#"version = 1

[workspace]
keep = "always"

[[services]]
id = "web"
image = "{image}"
runtime = "micro_vm"
container_port = 80
port = {port}
timeout_s = 2
{ready_http}
[[runners]]
id = "smoke"
requires = ["web"]
cmd = ["true"]
"#
        ),
    )
    .expect("write vat.toml");
}

fn write_microvm_preset_project(project: &Path, port: u16, runner_command: &str, timeout_s: u64) {
    fs::write(
        project.join("vat.toml"),
        format!(
            r#"version = 1

[workspace]
keep = "always"

[[services]]
id = "cache"
preset = "redis"
runtime = "micro_vm"
container_port = 6379
port = {port}
timeout_s = {timeout_s}

[[runners]]
id = "smoke"
requires = ["cache"]
cmd = ["/bin/sh", "-c", "{runner_command}"]
"#
        ),
    )
    .expect("write MicroVM preset vat.toml");
}

#[derive(Clone, Copy)]
enum CleanupRetentionTarget {
    Runner,
    Scenario,
}

impl CleanupRetentionTarget {
    fn label(self) -> &'static str {
        match self {
            Self::Runner => "runner keep=failed cleanup-retention regression",
            Self::Scenario => "scenario keep=failed cleanup-retention regression",
        }
    }

    fn command(self) -> Vec<&'static str> {
        match self {
            Self::Runner => vec!["run", "smoke"],
            Self::Scenario => vec!["run", "--scenario", "cleanup-scenario"],
        }
    }

    fn compose_project(self) -> &'static str {
        match self {
            Self::Runner => "runner-cleanup-retry",
            Self::Scenario => "scenario-cleanup-retry",
        }
    }
}

fn write_successful_cleanup_retention_project(
    project: &Path,
    port: u16,
    target: CleanupRetentionTarget,
) {
    let scenario = match target {
        CleanupRetentionTarget::Runner => "",
        CleanupRetentionTarget::Scenario => {
            r#"
[[scenarios]]
id = "cleanup-scenario"
app = "web"
runner = "smoke"
"#
        }
    };
    fs::write(
        project.join("vat.toml"),
        format!(
            r#"version = 1

[workspace]
keep = "failed"

[[services]]
id = "web"
image = "fake:image"
runtime = "micro_vm"
container_port = 80
port = {port}
timeout_s = 2

[[runners]]
id = "smoke"
requires = ["web"]
cmd = ["true"]
{scenario}"#
        ),
    )
    .expect("write successful cleanup-retention vat.toml");
}

fn write_compose_retry_registry(vat_home: &Path, project: &str, vat_id: &str) -> PathBuf {
    let registry = vat_home.join("compose").join(project);
    fs::create_dir_all(&registry).expect("create compose retry registry");
    fs::write(
        registry.join("project.json"),
        serde_json::to_vec(&serde_json::json!({
            "project": project,
            "vat_id": vat_id,
            "service_ids": ["web"],
            "status": "ready",
            "created_at": "2026-01-01T00:00:00Z",
        }))
        .expect("serialize compose retry registry"),
    )
    .expect("write compose retry registry");
    registry
}

fn write_compose_microvm_project(
    vat_home: &Path,
    project: &str,
    port: u16,
    runner_command: &str,
) -> PathBuf {
    let registry = vat_home.join("compose").join(project);
    fs::create_dir_all(&registry).expect("create compose registry");
    fs::write(
        registry.join("vat.toml"),
        format!(
            r#"version = 1

[workspace]
keep = "always"

[[services]]
id = "web"
image = "fake:image"
runtime = "micro_vm"
container_port = 80
port = {port}
timeout_s = 2

[[runners]]
id = "project.up"
requires = ["web"]
cmd = ["/bin/sh", "-c", "{runner_command}"]
"#,
        ),
    )
    .expect("write compose MicroVM vat.toml");
    fs::write(
        registry.join("project.json"),
        serde_json::to_vec(&serde_json::json!({
            "project": project,
            "vat_id": null,
            "service_ids": ["web"],
            "status": "imported",
            "created_at": "2026-01-01T00:00:00Z",
        }))
        .expect("serialize compose registry"),
    )
    .expect("write compose registry");
    registry
}

fn fake_vat_command(project: &Path, vat_home: &Path, bin_dir: &Path, fake_log: &Path) -> Command {
    let mut command = Command::new(vat_bin());
    command
        .current_dir(project)
        .env("VAT_HOME", vat_home)
        .env("VAT_FAKE_CONTAINER_LOG", fake_log)
        .env("PATH", path_with_fake_container(bin_dir));
    command
}

fn state(vat_home: &Path, id: &str) -> Value {
    let output = Command::new(vat_bin())
        .env("VAT_HOME", vat_home)
        .args(["state", id, "--compact"])
        .output()
        .expect("vat state");
    assert!(
        output.status.success(),
        "vat state failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("VAT state JSON")
}

fn wait_for_compose_microvm_ready(vat_home: &Path, registry: &Path) -> String {
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        let record = fs::read(registry.join("project.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
        if let Some(record) = record {
            if let Some(vat_id) = record["vat_id"].as_str() {
                let lifecycle = state(vat_home, vat_id);
                let ready = lifecycle["test_run"]["services"]
                    .as_array()
                    .is_some_and(|services| {
                        services
                            .iter()
                            .any(|service| service["id"] == "web" && service["status"] == "ready")
                    });
                if ready {
                    return vat_id.to_string();
                }
            }
        }
        assert!(
            Instant::now() < deadline,
            "detached compose MicroVM service never became ready"
        );
        thread::sleep(Duration::from_millis(25));
    }
}

fn wait_for_compose_cleanup_error(vat_home: &Path, registry: &Path) -> (Value, String, Value) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let record = fs::read(registry.join("project.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
        if let Some(record) = record {
            if let Some(vat_id) = record["vat_id"].as_str().map(str::to_owned) {
                let meta = fs::read(vat_home.join("vats").join(&vat_id).join("meta.json"))
                    .ok()
                    .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
                if let Some(meta) = meta {
                    let has_cleanup_error =
                        meta["test_run"]["services"]
                            .as_array()
                            .is_some_and(|services| {
                                services.iter().any(|service| {
                                    service["cleanup_error"]
                                        .as_str()
                                        .is_some_and(|error| !error.is_empty())
                                })
                            });
                    if has_cleanup_error {
                        return (record, vat_id.clone(), state(vat_home, &vat_id));
                    }
                }
            }
        }
        assert!(
            Instant::now() < deadline,
            "detached compose startup never persisted its MicroVM cleanup failure"
        );
        thread::sleep(Duration::from_millis(25));
    }
}

fn loopback_listener(test: &str) -> Option<TcpListener> {
    match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => Some(listener),
        Err(err) if err.kind() == ErrorKind::PermissionDenied => {
            if std::env::var("VAT_MICROVM_LOOPBACK_REQUIRED").as_deref() == Ok("1") {
                panic!("{test} requires loopback sockets, but this runner forbids them: {err}");
            }
            eprintln!("Skipping {test}: loopback sockets are unavailable ({err})");
            None
        }
        Err(err) => panic!("bind loopback for {test}: {err}"),
    }
}

fn accept_before_deadline(listener: &TcpListener, test: &str) -> std::io::Result<TcpStream> {
    accept_before_timeout(listener, test, Duration::from_secs(4))
}

fn accept_before_timeout(
    listener: &TcpListener,
    test: &str,
    timeout: Duration,
) -> std::io::Result<TcpStream> {
    listener.set_nonblocking(true)?;
    let deadline = Instant::now() + timeout;
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                stream.set_nonblocking(false)?;
                return Ok(stream);
            }
            Err(err) if err.kind() == ErrorKind::WouldBlock => {
                if Instant::now() >= deadline {
                    return Err(std::io::Error::new(
                        ErrorKind::TimedOut,
                        format!("{test} did not receive a readiness connection"),
                    ));
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(err) => return Err(err),
        }
    }
}

#[cfg(unix)]
#[test]
fn accepted_test_stream_is_returned_in_blocking_mode() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback listener");
    let address = listener.local_addr().expect("loopback listener address");
    let _client = TcpStream::connect(address).expect("queue loopback client");
    let stream = accept_before_timeout(
        &listener,
        "accepted test stream blocking-mode regression",
        Duration::from_secs(1),
    )
    .expect("accept queued loopback client");

    let flags = unsafe { libc::fcntl(stream.as_raw_fd(), libc::F_GETFL) };
    assert_ne!(
        flags,
        -1,
        "read accepted stream flags: {}",
        std::io::Error::last_os_error()
    );
    assert_eq!(
        flags & libc::O_NONBLOCK,
        0,
        "accepted helper stream must be blocking"
    );
}

/// The MicroVM readiness probe deliberately accepts an idle, open TCP stream.
/// Keep the stale listener alive long enough for a failed `container run` child
/// to exit while VAT is probing it.
fn hold_connection_open_and_signal(
    listener: TcpListener,
    test: &'static str,
    signal: PathBuf,
    duration: Duration,
) -> thread::JoinHandle<Result<(), String>> {
    thread::spawn(move || {
        let _stream = accept_before_deadline(&listener, test).map_err(|err| err.to_string())?;
        fs::write(signal, b"stale listener accepted readiness connection")
            .map_err(|err| err.to_string())?;
        thread::sleep(duration);
        Ok(())
    })
}

/// A real-host cleanup assertion must not turn a broken runtime into an
/// indefinitely blocked test process.
fn container_status_bounded(
    args: &[&str],
    timeout: Duration,
) -> std::io::Result<Option<ExitStatus>> {
    let mut child = Command::new("container")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait()? {
            Some(status) => return Ok(Some(status)),
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Ok(None);
            }
            None => thread::sleep(Duration::from_millis(20)),
        }
    }
}

fn wait_for_redis_pong(port: u16) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(20);
    let address = format!("127.0.0.1:{port}");
    let mut last = "Redis endpoint was not probed".to_string();
    while Instant::now() < deadline {
        match TcpStream::connect(&address) {
            Ok(mut stream) => {
                let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
                let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));
                match stream.write_all(b"*1\r\n$4\r\nPING\r\n") {
                    Ok(()) => {
                        let mut response = [0u8; 64];
                        match stream.read(&mut response) {
                            Ok(read) if response[..read].starts_with(b"+PONG\r\n") => return Ok(()),
                            Ok(read) => {
                                last = format!(
                                    "Redis endpoint returned {:?}",
                                    String::from_utf8_lossy(&response[..read])
                                )
                            }
                            Err(error) => last = format!("Redis endpoint read failed: {error}"),
                        }
                    }
                    Err(error) => last = format!("Redis endpoint write failed: {error}"),
                }
            }
            Err(error) => last = format!("Redis endpoint connect failed: {error}"),
        }
        thread::sleep(Duration::from_millis(150));
    }
    Err(last)
}

fn run_microvm_preset_without_docker_fallback(image_missing: bool) {
    let project = TempDir::new().expect("project");
    let vat_home = TempDir::new().expect("VAT_HOME");
    let fake_bin = TempDir::new().expect("fake bin");
    let fake_container_log = fake_bin.path().join("container.log");
    let fake_docker_log = fake_bin.path().join("docker.log");
    fake_container_path(fake_bin.path());
    fake_docker_path(fake_bin.path());
    if image_missing {
        fs::write(
            fake_bin.path().join(".vat-fake-image-missing"),
            b"missing image",
        )
        .expect("mark fake Apple image missing");
    }

    let Some(listener) = loopback_listener("MicroVM preset routing regression") else {
        return;
    };
    let port = listener
        .local_addr()
        .expect("readiness listener address")
        .port();
    let ready_server = hold_connection_open_and_signal(
        listener,
        "MicroVM preset routing regression",
        project.path().join("readiness-accepted"),
        Duration::from_millis(500),
    );
    write_microvm_preset_project(project.path(), port, "true", 2);

    let output = fake_vat_command(
        project.path(),
        vat_home.path(),
        fake_bin.path(),
        &fake_container_log,
    )
    .env("VAT_FAKE_DOCKER_LOG", &fake_docker_log)
    .args(["run", "smoke"])
    .output()
    .expect("run MicroVM preset through VAT");
    ready_server
        .join()
        .expect("readiness server thread")
        .expect("readiness server");
    assert!(
        output.status.success(),
        "MicroVM preset run must succeed without Docker: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let events = jsonl(&output.stdout);
    let result = result_event(&events);
    let vat_id = result["id"].as_str().expect("result VAT id");
    let lifecycle = state(vat_home.path(), vat_id);
    let service = lifecycle["test_run"]["services"]
        .as_array()
        .and_then(|services| services.iter().find(|service| service["id"] == "cache"))
        .expect("persisted cache service evidence");
    assert_eq!(
        service["prepare_mode"], "container_run",
        "state: {lifecycle}"
    );
    assert!(service["docker_name"].is_null(), "state: {lifecycle}");
    let name = service["microvm_name"]
        .as_str()
        .expect("persisted Apple Container service name");

    let container_calls =
        fs::read_to_string(&fake_container_log).expect("read fake container command log");
    let inspect = format!("image inspect redis:7");
    assert!(
        container_calls.contains(&inspect),
        "MicroVM preset must inspect the Apple Container image store, calls:\n{container_calls}"
    );
    if image_missing {
        let pull = "image pull redis:7";
        let inspect_at = container_calls.find(&inspect).expect("image inspect call");
        let pull_at = container_calls.find(pull).expect("image pull call");
        let run_at = container_calls
            .find("run --rm --name")
            .expect("container run call");
        assert!(
            inspect_at < pull_at && pull_at < run_at,
            "missing Apple image must inspect, pull, then run in that order: {container_calls}"
        );
    } else {
        assert!(
            !container_calls.contains("image pull redis:7"),
            "cached Apple image must not be pulled again: {container_calls}"
        );
    }
    assert!(
        container_calls.contains(&format!(
            "run --rm --name {name} -p 127.0.0.1:{port}:6379 redis:7"
        )),
        "expected Apple Container Redis argv, calls:\n{container_calls}"
    );
    assert!(
        container_calls.contains(&format!("rm -f {name}")),
        "expected exact Apple Container cleanup, calls:\n{container_calls}"
    );
    assert!(
        !fake_docker_log.exists(),
        "a MicroVM preset must not invoke Docker: {}",
        fake_docker_log.display()
    );
}

#[test]
fn microvm_preset_uses_apple_container_without_docker_fallback() {
    run_microvm_preset_without_docker_fallback(false);
}

#[test]
fn microvm_preset_pulls_missing_apple_image_before_running() {
    run_microvm_preset_without_docker_fallback(true);
}

fn successful_cleanup_error_is_retained_for_retry(target: CleanupRetentionTarget) {
    let project = TempDir::new().expect("project");
    let vat_home = TempDir::new().expect("VAT_HOME");
    let fake_bin = TempDir::new().expect("fake bin");
    let fake_log = fake_bin.path().join("container.log");
    fake_container_path(fake_bin.path());

    let Some(listener) = loopback_listener(target.label()) else {
        return;
    };
    let port = listener
        .local_addr()
        .expect("readiness listener address")
        .port();
    let ready_server = hold_connection_open_and_signal(
        listener,
        target.label(),
        project.path().join("readiness-accepted"),
        Duration::from_millis(500),
    );
    write_successful_cleanup_retention_project(project.path(), port, target);

    // The service and target both succeed. Only its VAT-owned MicroVM removal
    // fails, which must make the lifecycle result fail and retain evidence
    // rather than silently discarding the retry target under `keep = failed`.
    let output = fake_vat_command(project.path(), vat_home.path(), fake_bin.path(), &fake_log)
        .env("VAT_FAKE_CONTAINER_RM_FAILURE", "1")
        .args(target.command())
        .output()
        .expect("successful VAT run with failed MicroVM cleanup");
    ready_server
        .join()
        .expect("readiness server thread")
        .expect("readiness server");
    assert!(
        !output.status.success(),
        "unconfirmed MicroVM cleanup must fail the lifecycle result: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let events = jsonl(&output.stdout);
    let result = result_event(&events);
    assert!(
        events
            .iter()
            .any(|event| event["code"] == "microvm_cleanup_unconfirmed"),
        "events: {events:#?}"
    );
    assert_eq!(result["ok"], false, "result: {result}");
    assert_eq!(result["exit_code"], -1, "result: {result}");
    assert_eq!(
        result["state"], "kept",
        "an unconfirmed MicroVM cleanup must override keep=failed removal: {result}"
    );
    let vat_id = result["id"].as_str().expect("result VAT id").to_string();
    assert!(
        vat_home
            .path()
            .join("vats")
            .join(&vat_id)
            .join("meta.json")
            .exists(),
        "VAT cleanup retry evidence was removed despite cleanup_error"
    );

    let failed_state = state(vat_home.path(), &vat_id);
    assert_eq!(
        failed_state["status"]["state"], "exited",
        "state: {failed_state}"
    );
    if matches!(target, CleanupRetentionTarget::Scenario) {
        assert_eq!(
            failed_state["test_run"]["scenario"]["id"], "cleanup-scenario",
            "state: {failed_state}"
        );
    }
    let failed_service = failed_state["test_run"]["services"]
        .as_array()
        .and_then(|services| services.iter().find(|service| service["id"] == "web"))
        .expect("persisted MicroVM service");
    assert_eq!(
        failed_state["test_run"]["runners"][0]["status"], "exited",
        "the target completed before cleanup failed: {failed_state}"
    );
    assert_eq!(
        failed_state["test_run"]["runners"][0]["exit_code"], 0,
        "the target succeeded before cleanup failed: {failed_state}"
    );
    assert_eq!(failed_service["status"], "failed", "state: {failed_state}");
    assert!(
        failed_service["cleanup_error"]
            .as_str()
            .unwrap_or_default()
            .contains("exited unsuccessfully"),
        "state: {failed_state}"
    );

    // Model the public compose retry path over the retained run. The repaired
    // fake runtime now lets `compose down` invoke retry_unconfirmed_microvm_cleanup,
    // clear the evidence, and release its binding.
    let registry = write_compose_retry_registry(vat_home.path(), target.compose_project(), &vat_id);
    let retry = fake_vat_command(project.path(), vat_home.path(), fake_bin.path(), &fake_log)
        .args(["compose", "down", target.compose_project()])
        .output()
        .expect("retry retained MicroVM cleanup through compose down");
    assert!(
        retry.status.success(),
        "cleanup retry failed: stdout={} stderr={}",
        String::from_utf8_lossy(&retry.stdout),
        String::from_utf8_lossy(&retry.stderr)
    );
    let recovered_state = state(vat_home.path(), &vat_id);
    let recovered_service = recovered_state["test_run"]["services"]
        .as_array()
        .and_then(|services| services.iter().find(|service| service["id"] == "web"))
        .expect("recovered MicroVM service");
    assert!(
        recovered_service["cleanup_error"].is_null(),
        "successful retry must clear cleanup evidence: {recovered_state}"
    );
    let released: Value =
        serde_json::from_slice(&fs::read(registry.join("project.json")).expect("read registry"))
            .expect("parse released registry");
    assert_eq!(released["status"], "imported", "registry: {released}");
    assert!(released["vat_id"].is_null(), "registry: {released}");

    let fake_calls = fs::read_to_string(&fake_log).expect("fake container command log");
    let removals = fake_calls
        .lines()
        .filter(|line| line.starts_with("rm -f "))
        .count();
    assert_eq!(
        removals, 2,
        "expected failed cleanup plus one successful retry: {fake_calls}"
    );
}

#[test]
fn hung_container_system_status_fails_microvm_startup_probe_within_bound() {
    // Exercise the public startup gate directly instead of invoking `vat run`,
    // whose user-facing retry budget is intentionally 30 seconds. A stuck
    // `container system status` must be killed by the one-second probe and
    // surface as a bounded startup failure.
    let _path_lock = PATH_ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("PATH test lock");
    let fake_bin = TempDir::new().expect("fake bin");
    fake_container_path(fake_bin.path());
    fs::write(
        fake_bin.path().join(".vat-fake-system-status-hang"),
        b"hang status",
    )
    .expect("mark fake status command as hung");
    let _path = EnvVarGuard::set("PATH", path_with_fake_container(fake_bin.path()));

    let started = Instant::now();
    let error = vat::sandbox::microvm::ensure_system_started(Duration::from_millis(25))
        .expect_err("a hung container system status must fail the MicroVM startup probe");
    assert!(
        error.contains("container system did not respond within"),
        "unexpected startup error: {error}"
    );
    assert!(
        started.elapsed() < Duration::from_secs(3),
        "hung `container system status` bypassed the bounded MicroVM startup probe: {:?}",
        started.elapsed()
    );
}

#[test]
fn published_tcp_reset_fails_closed_with_evidence_and_owned_cleanup() {
    let project = TempDir::new().expect("project");
    let vat_home = TempDir::new().expect("VAT_HOME");
    let fake_bin = TempDir::new().expect("fake bin");
    let fake_log = fake_bin.path().join("container.log");
    fake_container_path(fake_bin.path());

    let Some(listener) = loopback_listener("published TCP reset regression") else {
        return;
    };
    let port = listener
        .local_addr()
        .expect("reset listener address")
        .port();
    let reset_server = thread::spawn(move || -> Result<(), String> {
        let stream = accept_before_deadline(&listener, "published TCP reset regression")
            .map_err(|err| err.to_string())?;
        stream
            .shutdown(Shutdown::Both)
            .map_err(|err| err.to_string())
    });
    write_project(project.path(), port, None, "fake:image");

    let output = fake_vat_command(project.path(), vat_home.path(), fake_bin.path(), &fake_log)
        .args(["run", "smoke", "--keep", "always"])
        .output()
        .expect("vat run");
    reset_server
        .join()
        .expect("reset server thread")
        .expect("reset server");

    assert!(
        !output.status.success(),
        "reset endpoint unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let events = jsonl(&output.stdout);
    let endpoint_failure = events
        .iter()
        .find(|event| event["code"] == "microvm_published_endpoint_unusable")
        .expect("MicroVM endpoint failure event");
    assert_eq!(endpoint_failure["service"], "web");
    assert_eq!(
        endpoint_failure["host_endpoint"],
        format!("127.0.0.1:{port}")
    );
    assert!(
        endpoint_failure["runtime_evidence"]
            .as_str()
            .unwrap_or_default()
            .contains("fake-container"),
        "event: {endpoint_failure}"
    );
    assert!(
        endpoint_failure["inspect_evidence"]
            .as_str()
            .unwrap_or_default()
            .contains("guest_ip=10.0.0.2"),
        "event: {endpoint_failure}"
    );

    let result = result_event(&events);
    assert_eq!(result["ok"], false, "result: {result}");
    let id = result["id"].as_str().expect("retained vat id");
    let state = state(vat_home.path(), id);
    let service = state["test_run"]["services"]
        .as_array()
        .and_then(|services| services.iter().find(|service| service["id"] == "web"))
        .expect("persisted web service");
    assert_eq!(service["status"], "failed", "state: {state}");
    let name = service["microvm_name"]
        .as_str()
        .expect("persisted VAT-owned MicroVM name");
    assert!(
        service["readiness_error"]
            .as_str()
            .unwrap_or_default()
            .contains("MicroVM published endpoint"),
        "state: {state}"
    );

    let fake_calls = fs::read_to_string(&fake_log).expect("fake container command log");
    assert!(
        fake_calls.contains(&format!(
            "run --rm --name {name} -p 127.0.0.1:{port}:80 fake:image"
        )),
        "calls:\n{fake_calls}"
    );
    assert!(
        fake_calls.contains(&format!("inspect {name}")),
        "calls:\n{fake_calls}"
    );
    assert!(
        fake_calls.contains(&format!("rm -f {name}")),
        "calls:\n{fake_calls}"
    );
    let removal_calls: Vec<_> = fake_calls
        .lines()
        .filter(|line| line.starts_with("rm "))
        .map(str::to_string)
        .collect();
    assert_eq!(removal_calls, vec![format!("rm -f {name}")]);
}

#[test]
fn stale_published_listener_cannot_mask_a_failed_microvm_child() {
    let project = TempDir::new().expect("project");
    let vat_home = TempDir::new().expect("VAT_HOME");
    let fake_bin = TempDir::new().expect("fake bin");
    let fake_log = fake_bin.path().join("container.log");
    let failure_gate = fake_bin.path().join("run-failure.gate");
    fake_container_path(fake_bin.path());

    let Some(listener) = loopback_listener("stale published listener regression") else {
        return;
    };
    let port = listener
        .local_addr()
        .expect("stale listener address")
        .port();
    // The fake runtime waits for this listener to accept VAT's readiness
    // connection, then exits during the deliberately idle TCP probe. A stale
    // host listener must therefore not make the MicroVM appear ready.
    let stale_server = hold_connection_open_and_signal(
        listener,
        "stale published listener regression",
        failure_gate.clone(),
        Duration::from_millis(500),
    );
    write_project(project.path(), port, None, "fake:image");

    let output = fake_vat_command(project.path(), vat_home.path(), fake_bin.path(), &fake_log)
        .env("VAT_FAKE_CONTAINER_RUN_FAILURE_GATE", &failure_gate)
        .args(["run", "smoke", "--keep", "always"])
        .output()
        .expect("vat run");
    stale_server
        .join()
        .expect("stale listener thread")
        .expect("stale listener");

    assert!(
        !output.status.success(),
        "stale listener incorrectly let an exited MicroVM child pass readiness:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let events = jsonl(&output.stdout);
    let endpoint_failure = events
        .iter()
        .find(|event| event["code"] == "microvm_published_endpoint_unusable")
        .expect("MicroVM endpoint failure event");
    assert!(
        endpoint_failure["reason"]
            .as_str()
            .unwrap_or_default()
            .contains("process exited"),
        "event: {endpoint_failure}"
    );
    assert!(
        !events
            .iter()
            .any(|event| event["type"] == "ready" && event["service"] == "web"),
        "an exited MicroVM must never emit a ready event: {events:#?}"
    );

    let id = result_event(&events)["id"]
        .as_str()
        .expect("retained vat id");
    let state = state(vat_home.path(), id);
    let service = state["test_run"]["services"]
        .as_array()
        .and_then(|services| services.iter().find(|service| service["id"] == "web"))
        .expect("persisted web service");
    assert_eq!(service["status"], "failed", "state: {state}");
    assert_eq!(service["exit_code"], 17, "state: {state}");
    assert!(
        service["readiness_error"]
            .as_str()
            .unwrap_or_default()
            .contains("process exited"),
        "state: {state}"
    );
    let name = service["microvm_name"]
        .as_str()
        .expect("persisted VAT-owned MicroVM name");
    let fake_calls = fs::read_to_string(&fake_log).expect("fake container command log");
    assert!(
        fake_calls.contains(&format!("rm -f {name}")),
        "calls:\n{fake_calls}"
    );
}

#[test]
fn hung_microvm_cleanup_is_bounded_and_persists_terminal_evidence() {
    let project = TempDir::new().expect("project");
    let vat_home = TempDir::new().expect("VAT_HOME");
    let fake_bin = TempDir::new().expect("fake bin");
    let fake_log = fake_bin.path().join("container.log");
    fake_container_path(fake_bin.path());

    let Some(listener) = loopback_listener("hung MicroVM cleanup regression") else {
        return;
    };
    let port = listener
        .local_addr()
        .expect("reset listener address")
        .port();
    let reset_server = thread::spawn(move || -> Result<Instant, String> {
        let stream = accept_before_timeout(
            &listener,
            "hung MicroVM cleanup regression",
            Duration::from_secs(10),
        )
        .map_err(|err| err.to_string())?;
        let readiness_accepted_at = Instant::now();
        stream
            .shutdown(Shutdown::Both)
            .map_err(|err| err.to_string())?;
        Ok(readiness_accepted_at)
    });
    write_project(project.path(), port, None, "fake:image");

    let output = fake_vat_command(project.path(), vat_home.path(), fake_bin.path(), &fake_log)
        .env("VAT_FAKE_CONTAINER_RM_HANG", "1")
        .args(["run", "smoke", "--keep", "always"])
        .output()
        .expect("vat run");
    let process_exited_at = Instant::now();
    let readiness_accepted_at = match reset_server.join().expect("reset server thread") {
        Ok(accepted_at) => accepted_at,
        Err(error) => {
            let fake_calls = fs::read_to_string(&fake_log)
                .unwrap_or_else(|read_error| format!("<fake log unavailable: {read_error}>"));
            panic!(
                "reset server: {error}; VAT stdout:\n{}\nVAT stderr:\n{}\nfake container calls:\n{fake_calls}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
        }
    };
    let cleanup_elapsed = process_exited_at.saturating_duration_since(readiness_accepted_at);

    assert!(
        !output.status.success(),
        "cleanup fixture unexpectedly succeeded"
    );
    assert!(
        cleanup_elapsed < Duration::from_secs(4),
        "a hung `container rm -f` blocked failure persistence for {cleanup_elapsed:?} after readiness"
    );
    let events = jsonl(&output.stdout);
    assert!(
        events
            .iter()
            .any(|event| event["code"] == "microvm_published_endpoint_unusable"),
        "events: {events:#?}"
    );
    let id = result_event(&events)["id"]
        .as_str()
        .expect("retained vat id");
    let state = state(vat_home.path(), id);
    let service = state["test_run"]["services"]
        .as_array()
        .and_then(|services| services.iter().find(|service| service["id"] == "web"))
        .expect("persisted web service");
    assert_eq!(service["status"], "failed", "state: {state}");
    assert!(
        service["readiness_error"].is_string(),
        "failure persistence lost endpoint evidence: {state}"
    );
    let name = service["microvm_name"]
        .as_str()
        .expect("persisted VAT-owned MicroVM name");
    let fake_calls = fs::read_to_string(&fake_log).expect("fake container command log");
    assert!(
        fake_calls.contains(&format!("rm -f {name}")),
        "calls:\n{fake_calls}"
    );
}

#[test]
fn failed_microvm_cleanup_is_persisted_as_terminal_evidence() {
    let project = TempDir::new().expect("project");
    let vat_home = TempDir::new().expect("VAT_HOME");
    let fake_bin = TempDir::new().expect("fake bin");
    let fake_log = fake_bin.path().join("container.log");
    fake_container_path(fake_bin.path());

    let Some(listener) = loopback_listener("nonzero MicroVM cleanup regression") else {
        return;
    };
    let port = listener
        .local_addr()
        .expect("reset listener address")
        .port();
    let reset_server = thread::spawn(move || -> Result<(), String> {
        let stream = accept_before_deadline(&listener, "nonzero MicroVM cleanup regression")
            .map_err(|err| err.to_string())?;
        stream
            .shutdown(Shutdown::Both)
            .map_err(|err| err.to_string())
    });
    write_project(project.path(), port, None, "fake:image");

    let output = fake_vat_command(project.path(), vat_home.path(), fake_bin.path(), &fake_log)
        .env("VAT_FAKE_CONTAINER_RM_FAILURE", "1")
        .args(["run", "smoke", "--keep", "always"])
        .output()
        .expect("vat run");
    reset_server
        .join()
        .expect("reset server thread")
        .expect("reset server");

    assert!(
        !output.status.success(),
        "cleanup failure fixture unexpectedly succeeded"
    );
    let events = jsonl(&output.stdout);
    let id = result_event(&events)["id"]
        .as_str()
        .expect("retained vat id");
    let state = state(vat_home.path(), id);
    let service = state["test_run"]["services"]
        .as_array()
        .and_then(|services| services.iter().find(|service| service["id"] == "web"))
        .expect("persisted web service");
    assert_eq!(service["status"], "failed", "state: {state}");
    let name = service["microvm_name"]
        .as_str()
        .expect("persisted MicroVM name");
    assert!(
        service["readiness_error"]
            .as_str()
            .unwrap_or_default()
            .contains("MicroVM published endpoint"),
        "endpoint evidence must remain separate from teardown evidence: {state}"
    );
    let cleanup_error = service["cleanup_error"]
        .as_str()
        .expect("cleanup failure must be retained separately from endpoint evidence");
    assert!(
        cleanup_error.contains(&format!(
            "MicroVM cleanup `container rm -f {name}` did not finish"
        )),
        "state: {state}"
    );
    assert!(
        cleanup_error.contains("exited unsuccessfully") && cleanup_error.contains("23"),
        "state: {state}"
    );

    let fake_calls = fs::read_to_string(&fake_log).expect("fake container command log");
    assert!(
        fake_calls.contains(&format!("rm -f {name}")),
        "calls:\n{fake_calls}"
    );
}

#[test]
fn successful_runner_keep_failed_retains_unconfirmed_microvm_cleanup_for_retry() {
    successful_cleanup_error_is_retained_for_retry(CleanupRetentionTarget::Runner);
}

#[test]
fn successful_scenario_keep_failed_retains_unconfirmed_microvm_cleanup_for_retry() {
    successful_cleanup_error_is_retained_for_retry(CleanupRetentionTarget::Scenario);
}

#[test]
fn compose_retains_cleanup_error_until_microvm_removal_retries() {
    let vat_home = TempDir::new().expect("VAT_HOME");
    let fake_bin = TempDir::new().expect("fake bin");
    let fake_log = fake_bin.path().join("container.log");
    fake_container_path(fake_bin.path());
    let fail_cleanup = fake_bin.path().join(".vat-fake-rm-failure");
    fs::write(&fail_cleanup, b"fail the first cleanup").expect("mark cleanup failure");

    let Some(listener) = loopback_listener("compose MicroVM cleanup retry regression") else {
        return;
    };
    let port = listener
        .local_addr()
        .expect("reset listener address")
        .port();
    let reset_server = thread::spawn(move || -> Result<(), String> {
        let stream = accept_before_timeout(
            &listener,
            "compose MicroVM cleanup retry regression",
            Duration::from_secs(10),
        )
        .map_err(|err| err.to_string())?;
        stream
            .shutdown(Shutdown::Both)
            .map_err(|err| err.to_string())
    });
    let project = "compose-cleanup-retry";
    let registry = write_compose_microvm_project(vat_home.path(), project, port, "true");

    let up = fake_vat_command(&registry, vat_home.path(), fake_bin.path(), &fake_log)
        .args(["compose", "up", "--project", project, "--detach"])
        .output()
        .expect("compose up");
    if let Err(error) = reset_server.join().expect("reset server thread") {
        let fake_calls = fs::read_to_string(&fake_log)
            .unwrap_or_else(|read_error| format!("<fake log unavailable: {read_error}>"));
        panic!(
            "reset server: {error}; compose up stdout:\n{}\ncompose up stderr:\n{}\nfake container calls:\n{fake_calls}",
            String::from_utf8_lossy(&up.stdout),
            String::from_utf8_lossy(&up.stderr),
        );
    }
    // The parent may observe the child after it publishes the VAT id but
    // before endpoint cleanup completes. In that valid detached handoff race,
    // `up` returns `starting`; the durable cleanup evidence must still retain
    // the binding and make subsequent lifecycle commands fail closed.
    if !up.status.success() {
        assert!(
            String::from_utf8_lossy(&up.stderr).contains("cleanup is unconfirmed"),
            "stderr: {}",
            String::from_utf8_lossy(&up.stderr)
        );
    }

    let (retained, vat_id, failed_state) =
        wait_for_compose_cleanup_error(vat_home.path(), &registry);
    assert_ne!(retained["status"], "imported", "registry: {retained}");
    let failed_service = failed_state["test_run"]["services"]
        .as_array()
        .and_then(|services| services.iter().find(|service| service["id"] == "web"))
        .expect("persisted MicroVM service");
    assert!(
        failed_service["cleanup_error"]
            .as_str()
            .unwrap_or_default()
            .contains("exited unsuccessfully"),
        "state: {failed_state}"
    );

    let ps = fake_vat_command(&registry, vat_home.path(), fake_bin.path(), &fake_log)
        .args(["compose", "ps", project])
        .output()
        .expect("compose ps");
    assert!(
        !ps.status.success(),
        "cleanup-unconfirmed project must not project as imported or ready"
    );
    assert!(
        String::from_utf8_lossy(&ps.stderr).contains("cleanup is unconfirmed"),
        "stderr: {}",
        String::from_utf8_lossy(&ps.stderr)
    );
    let still_retained: Value =
        serde_json::from_slice(&fs::read(registry.join("project.json")).expect("read registry"))
            .expect("parse retained registry");
    assert_eq!(
        still_retained["vat_id"], vat_id,
        "registry: {still_retained}"
    );
    assert_ne!(
        still_retained["status"], "imported",
        "registry: {still_retained}"
    );

    // The runtime is repaired out-of-band. `compose down` must retry only
    // the persisted cleanup operation, then release the binding after confirmation.
    fs::remove_file(&fail_cleanup).expect("repair fake MicroVM cleanup");
    let down = fake_vat_command(&registry, vat_home.path(), fake_bin.path(), &fake_log)
        .args(["compose", "down", project])
        .output()
        .expect("compose down cleanup retry");
    assert!(
        down.status.success(),
        "cleanup retry failed: stdout={} stderr={}",
        String::from_utf8_lossy(&down.stdout),
        String::from_utf8_lossy(&down.stderr)
    );
    let released: Value =
        serde_json::from_slice(&fs::read(registry.join("project.json")).expect("read registry"))
            .expect("parse released registry");
    assert_eq!(released["status"], "imported", "registry: {released}");
    assert!(released["vat_id"].is_null(), "registry: {released}");

    let recovered_state = state(vat_home.path(), &vat_id);
    let recovered_service = recovered_state["test_run"]["services"]
        .as_array()
        .and_then(|services| services.iter().find(|service| service["id"] == "web"))
        .expect("persisted MicroVM service");
    assert!(
        recovered_service["cleanup_error"].is_null(),
        "successful retry must clear cleanup evidence: {recovered_state}"
    );
    let fake_calls = fs::read_to_string(&fake_log).expect("fake container command log");
    let removals = fake_calls
        .lines()
        .filter(|line| line.starts_with("rm -f "))
        .count();
    assert_eq!(
        removals, 2,
        "expected initial cleanup and one retry: {fake_calls}"
    );
}

#[derive(Clone, Copy)]
enum MicroVmExactList {
    Empty,
    Matching,
    Error,
}

impl MicroVmExactList {
    fn marker_name(self) -> Option<&'static str> {
        match self {
            Self::Empty => Some(".vat-fake-container-list-empty"),
            Self::Matching => None,
            Self::Error => Some(".vat-fake-container-list-error"),
        }
    }

    fn project(self) -> &'static str {
        match self {
            Self::Empty => "microvm-auto-remove-confirmed",
            Self::Matching => "microvm-auto-remove-still-present",
            Self::Error => "microvm-auto-remove-list-error",
        }
    }

    fn confirms_absence(self) -> bool {
        matches!(self, Self::Empty)
    }
}

/// Model a `container run --rm` child that removed itself before VAT's explicit
/// `rm -f` command. Only a successful JSON list that lacks the persisted exact
/// id may clear the compose binding; a matching id or list error is retained.
fn compose_microvm_failed_rm_with_exact_id_list(result: MicroVmExactList) {
    let workspace = TempDir::new().expect("workspace");
    let vat_home = TempDir::new().expect("VAT_HOME");
    let fake_bin = TempDir::new().expect("fake bin");
    let fake_log = fake_bin.path().join("container.log");
    fake_container_path(fake_bin.path());
    fs::write(
        fake_bin.path().join(".vat-fake-rm-failure"),
        b"simulate --rm auto-removal before explicit cleanup",
    )
    .expect("mark fake MicroVM rm failure");
    if let Some(marker) = result.marker_name() {
        fs::write(
            fake_bin.path().join(marker),
            b"configure fake MicroVM exact-name list proof",
        )
        .expect("configure fake MicroVM exact-name list proof");
    }

    let Some(listener) = loopback_listener("compose MicroVM exact-name list regression") else {
        return;
    };
    let port = listener
        .local_addr()
        .expect("MicroVM readiness listener address")
        .port();
    let ready_server = hold_connection_open_and_signal(
        listener,
        "compose MicroVM exact-name list regression",
        workspace.path().join("readiness-accepted"),
        Duration::from_secs(2),
    );
    let project = result.project();
    let registry = write_compose_microvm_project(vat_home.path(), project, port, "sleep 10");

    let up = fake_vat_command(&registry, vat_home.path(), fake_bin.path(), &fake_log)
        .args(["compose", "up", "--project", project, "--detach"])
        .output()
        .expect("compose up with fake MicroVM");
    assert!(
        up.status.success(),
        "compose up failed: stdout={} stderr={}",
        String::from_utf8_lossy(&up.stdout),
        String::from_utf8_lossy(&up.stderr)
    );
    let vat_id = wait_for_compose_microvm_ready(vat_home.path(), &registry);

    let down = fake_vat_command(&registry, vat_home.path(), fake_bin.path(), &fake_log)
        .args(["compose", "down", project])
        .output()
        .expect("compose down after fake MicroVM auto-removal");
    ready_server
        .join()
        .expect("MicroVM readiness server thread")
        .expect("MicroVM readiness server");

    let lifecycle = state(vat_home.path(), &vat_id);
    let service = lifecycle["test_run"]["services"]
        .as_array()
        .and_then(|services| services.iter().find(|service| service["id"] == "web"))
        .expect("persisted MicroVM service");
    let record: Value =
        serde_json::from_slice(&fs::read(registry.join("project.json")).expect("read registry"))
            .expect("parse registry");

    if result.confirms_absence() {
        assert!(
            down.status.success(),
            "an empty exact-id MicroVM list must permit compose down: stdout={} stderr={}",
            String::from_utf8_lossy(&down.stdout),
            String::from_utf8_lossy(&down.stderr)
        );
        assert_eq!(record["status"], "imported", "registry: {record}");
        assert!(record["vat_id"].is_null(), "registry: {record}");
        assert!(
            service["cleanup_error"].is_null(),
            "confirmed MicroVM auto-removal must not persist cleanup evidence: {lifecycle}"
        );
    } else {
        assert!(
            !down.status.success(),
            "a matching id or list error must retain MicroVM cleanup: stdout={} stderr={}",
            String::from_utf8_lossy(&down.stdout),
            String::from_utf8_lossy(&down.stderr)
        );
        assert_eq!(record["vat_id"], vat_id, "registry: {record}");
        assert_ne!(record["status"], "imported", "registry: {record}");
        assert!(
            service["cleanup_error"]
                .as_str()
                .is_some_and(|error| error.contains("container rm -f")),
            "a matching id or list error must retain MicroVM cleanup evidence: {lifecycle}"
        );
    }

    let calls = fs::read_to_string(&fake_log).expect("fake container command log");
    let cleanup_start = calls
        .lines()
        .position(|line| line.starts_with("rm -f "))
        .expect("MicroVM cleanup command");
    assert!(
        calls
            .lines()
            .skip(cleanup_start)
            .any(|line| line.starts_with("list --all --format json")),
        "failed rm must list the exact MicroVM id: {calls}"
    );
}

#[test]
fn compose_accepts_microvm_auto_remove_after_empty_exact_id_list() {
    compose_microvm_failed_rm_with_exact_id_list(MicroVmExactList::Empty);
}

#[test]
fn compose_retains_microvm_auto_remove_when_exact_id_list_matches() {
    compose_microvm_failed_rm_with_exact_id_list(MicroVmExactList::Matching);
}

#[test]
fn compose_retains_microvm_auto_remove_when_exact_id_list_errors() {
    compose_microvm_failed_rm_with_exact_id_list(MicroVmExactList::Error);
}

#[test]
fn published_endpoint_diagnostic_timeout_still_persists_and_cleans_up() {
    let project = TempDir::new().expect("project");
    let vat_home = TempDir::new().expect("VAT_HOME");
    let fake_bin = TempDir::new().expect("fake bin");
    let fake_log = fake_bin.path().join("container.log");
    fake_container_path(fake_bin.path());

    let Some(listener) = loopback_listener("published endpoint diagnostic timeout") else {
        return;
    };
    let port = listener
        .local_addr()
        .expect("reset listener address")
        .port();
    let reset_server = thread::spawn(move || -> Result<(), String> {
        let stream = accept_before_deadline(&listener, "published endpoint diagnostic timeout")
            .map_err(|err| err.to_string())?;
        stream
            .shutdown(Shutdown::Both)
            .map_err(|err| err.to_string())
    });
    write_project(project.path(), port, None, "fake:image");

    let started = Instant::now();
    let output = fake_vat_command(project.path(), vat_home.path(), fake_bin.path(), &fake_log)
        .env("VAT_FAKE_CONTAINER_INSPECT_HANG", "1")
        .args(["run", "smoke", "--keep", "always"])
        .output()
        .expect("vat run");
    let elapsed = started.elapsed();
    reset_server
        .join()
        .expect("reset server thread")
        .expect("reset server");

    assert!(
        !output.status.success(),
        "diagnostic fixture unexpectedly succeeded"
    );
    assert!(
        elapsed < Duration::from_secs(4),
        "a hung inspect blocked fail-closed cleanup for {elapsed:?}"
    );
    let events = jsonl(&output.stdout);
    let endpoint_failure = events
        .iter()
        .find(|event| event["code"] == "microvm_published_endpoint_unusable")
        .expect("MicroVM endpoint failure event");
    assert!(
        endpoint_failure["inspect_evidence"]
            .as_str()
            .is_some_and(|evidence| evidence.contains("timed out after") && evidence.contains("ms")),
        "event: {endpoint_failure}"
    );
    assert_eq!(endpoint_failure["diagnostic_budget_ms"], 1_000);
    let id = result_event(&events)["id"]
        .as_str()
        .expect("retained vat id");
    let state = state(vat_home.path(), id);
    let name = state["test_run"]["services"]
        .as_array()
        .and_then(|services| services.iter().find(|service| service["id"] == "web"))
        .and_then(|service| service["microvm_name"].as_str())
        .expect("persisted MicroVM name");
    let fake_calls = fs::read_to_string(&fake_log).expect("fake container command log");
    assert!(
        fake_calls.contains(&format!("inspect {name}")),
        "calls:\n{fake_calls}"
    );
    assert!(
        fake_calls.contains(&format!("rm -f {name}")),
        "calls:\n{fake_calls}"
    );
}

#[test]
fn configured_microvm_http_readiness_uses_allocated_published_port() {
    let project = TempDir::new().expect("project");
    let vat_home = TempDir::new().expect("VAT_HOME");
    let fake_bin = TempDir::new().expect("fake bin");
    let fake_log = fake_bin.path().join("container.log");
    fake_container_path(fake_bin.path());

    let Some(listener) = loopback_listener("published HTTP readiness regression") else {
        return;
    };
    let port = listener.local_addr().expect("HTTP listener address").port();
    let http_server = thread::spawn(move || -> Result<(), String> {
        let mut stream = accept_before_deadline(&listener, "published HTTP readiness regression")
            .map_err(|err| err.to_string())?;
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .map_err(|err| err.to_string())?;
        let mut request = Vec::new();
        loop {
            let mut chunk = [0u8; 256];
            let read = stream.read(&mut chunk).map_err(|err| err.to_string())?;
            if read == 0 {
                return Err("HTTP readiness client closed before request".to_string());
            }
            request.extend_from_slice(&chunk[..read]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }
        let request = String::from_utf8_lossy(&request);
        if !request.starts_with("GET /ready HTTP/1.1") {
            return Err(format!("unexpected request: {request}"));
        }
        stream
            .write_all(b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .map_err(|err| err.to_string())
    });
    write_project(
        project.path(),
        port,
        Some("http://{host}:{port}/ready"),
        "fake:image",
    );

    let output = fake_vat_command(project.path(), vat_home.path(), fake_bin.path(), &fake_log)
        .args(["run", "smoke", "--keep", "always"])
        .output()
        .expect("vat run");
    http_server
        .join()
        .expect("HTTP server thread")
        .expect("HTTP server");

    assert!(
        output.status.success(),
        "HTTP MicroVM readiness failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let events = jsonl(&output.stdout);
    assert!(
        events
            .iter()
            .any(|event| event["type"] == "ready" && event["service"] == "web"),
        "events: {events:#?}"
    );
    assert_eq!(result_event(&events)["ok"], true);
}

#[test]
#[ignore = "real Apple-container MicroVM preset contract; run only with VAT_MICROVM_PRESET_E2E_REQUIRED=1"]
fn apple_container_redis_microvm_preset_contract() {
    if std::env::var("VAT_MICROVM_PRESET_E2E_REQUIRED").as_deref() != Ok("1") {
        eprintln!(
            "VAT_MICROVM_PRESET_E2E_REQUIRED=1 is required; skipping real MicroVM preset probe"
        );
        return;
    }

    let project = TempDir::new().expect("project");
    let vat_home = TempDir::new().expect("VAT_HOME");
    let port_listener = TcpListener::bind("127.0.0.1:0").expect("reserve host port");
    let port = port_listener
        .local_addr()
        .expect("reserved host port")
        .port();
    // The Apple Container port forward must own this socket.  Keeping the
    // probing listener alive here makes the real lifecycle test fail before
    // Redis ever has a chance to accept connections.
    drop(port_listener);
    // Apple Container cold starts include guest boot and can take several
    // seconds even with the Redis image cached; keep this real-host contract
    // bounded, but do not encode a false two-second readiness guarantee.
    write_microvm_preset_project(project.path(), port, "sleep 5", 30);

    let child = Command::new(vat_bin())
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .args(["run", "smoke", "--keep", "always"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn real MicroVM preset run");
    let ping = wait_for_redis_pong(port);
    let output = child
        .wait_with_output()
        .expect("wait for real MicroVM preset run");
    ping.unwrap_or_else(|error| {
        panic!(
            "MicroVM Redis preset never returned RESP PONG through its host port: {error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    assert!(
        output.status.success(),
        "MicroVM Redis preset stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let events = jsonl(&output.stdout);
    let result = result_event(&events);
    assert_eq!(result["ok"], true, "result: {result}");
    let vat_id = result["id"].as_str().expect("retained VAT id");
    let lifecycle = state(vat_home.path(), vat_id);
    let service = lifecycle["test_run"]["services"]
        .as_array()
        .and_then(|services| services.iter().find(|service| service["id"] == "cache"))
        .expect("persisted Redis preset service");
    assert_eq!(
        service["prepare_mode"], "container_run",
        "state: {lifecycle}"
    );
    assert!(service["docker_name"].is_null(), "state: {lifecycle}");
    let name = service["microvm_name"]
        .as_str()
        .expect("persisted Apple Container Redis name");
    let inspect = container_status_bounded(&["inspect", name], Duration::from_secs(2))
        .expect("inspect exact Redis cleanup")
        .unwrap_or_else(|| panic!("cleanup inspection for MicroVM Redis `{name}` timed out"));
    assert!(
        !inspect.success(),
        "MicroVM Redis `{name}` remained after VAT cleanup"
    );
}

#[test]
#[ignore = "real Apple-container published endpoint contract; run only with VAT_MICROVM_E2E_REQUIRED=1"]
fn apple_container_published_endpoint_contract() {
    if std::env::var("VAT_MICROVM_E2E_REQUIRED").as_deref() != Ok("1") {
        eprintln!("VAT_MICROVM_E2E_REQUIRED=1 is required; skipping real-host probe");
        return;
    }

    let project = TempDir::new().expect("project");
    let vat_home = TempDir::new().expect("VAT_HOME");
    let port_listener = TcpListener::bind("127.0.0.1:0").expect("reserve host port");
    let port = port_listener
        .local_addr()
        .expect("reserved host port")
        .port();
    // Release the reservation before VAT asks Apple Container to publish it.
    drop(port_listener);
    write_project(
        project.path(),
        port,
        Some("http://{host}:{port}/"),
        "nginx:alpine",
    );

    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .args(["run", "smoke", "--keep", "always"])
        .output()
        .expect("real vat run");
    let events = jsonl(&output.stdout);
    if output.status.success() {
        assert_eq!(result_event(&events)["ok"], true);
    } else {
        let endpoint_failure = events
            .iter()
            .find(|event| event["code"] == "microvm_published_endpoint_unusable")
            .unwrap_or_else(|| {
                panic!(
                    "real host failures must fail closed with endpoint evidence:\nstdout:\n{}\nstderr:\n{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                )
            });
        let result = result_event(&events);
        let id = result["id"].as_str().expect("retained failure vat id");
        let state = state(vat_home.path(), id);
        let service = state["test_run"]["services"]
            .as_array()
            .and_then(|services| services.iter().find(|service| service["id"] == "web"))
            .expect("persisted MicroVM service");
        assert!(
            matches!(service["status"].as_str(), Some("failed" | "timeout")),
            "state: {state}"
        );
        let name = service["microvm_name"]
            .as_str()
            .expect("persisted MicroVM name");
        let inspect = container_status_bounded(&["inspect", name], Duration::from_secs(2))
            .expect("inspect cleanup")
            .unwrap_or_else(|| panic!("cleanup inspection for failed MicroVM `{name}` timed out"));
        assert!(
            !inspect.success(),
            "failed MicroVM `{name}` remained after VAT cleanup; endpoint event: {endpoint_failure}"
        );
    }
}
// HANDWRITE-END
