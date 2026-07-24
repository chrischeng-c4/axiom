// HANDWRITE-BEGIN gap="missing-generator:e2e-test:ephemeral-k8s-command" tracker="#1693" reason="The Apple Container local-Kubernetes session needs deterministic process-level proof that raw command parsing, exact-machine cleanup, private kubeconfig injection, and host-child exit forwarding work together without a live runtime."

//! Process-level regression tests for VAT's bounded Apple Container K3s command.

use std::ffi::OsString;
use std::fs::{self, File};
use std::net::TcpStream;
use std::os::fd::AsRawFd;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;
use tempfile::TempDir;

fn vat_bin() -> &'static str {
    env!("CARGO_BIN_EXE_vat")
}

fn path_with(bin: &Path) -> OsString {
    let mut paths = vec![bin.to_path_buf()];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").expect("PATH is available"),
    ));
    std::env::join_paths(paths).expect("join test PATH")
}

fn make_executable(path: &Path, source: &str) {
    fs::write(path, source).expect("write script");
    let mut permissions = fs::metadata(path).expect("script metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("make script executable");
}

fn write_fake_runtime(bin: &Path) {
    fs::create_dir_all(bin).expect("create fake bin");
    make_executable(
        &bin.join("container"),
        r#"#!/bin/sh
state="$VAT_FAKE_K8S_STATE"
container_log="$VAT_FAKE_CONTAINER_LOG"
backing_id="${VAT_FAKE_K8S_BACKING_ID:-owned-backing}"
backing_ip="${VAT_FAKE_K8S_BACKING_IP:-192.168.64.17}"
case "$1:$2" in
  system:status)
    if [ "$VAT_FAKE_K3S_DIAGNOSTIC_MODE" = "fail" ] && [ -e "$state.bootstrap-failed" ]; then
      echo "simulated container system diagnostic failure" >&2
      exit 76
    fi
    printf 'fake container system status\n'
    exit 0
    ;;
  image:inspect)
    if [ "$3" = "fixture/systemd:k3s" ]; then
      exit 0
    fi
    if [ "$VAT_FAKE_K8S_IMAGE_MODE" = "missing" ]; then
      echo "fake image not found: $3" >&2
      exit 1
    fi
    printf '[{"configuration":{"name":"docker.io/library/alpine:3.20","descriptor":{"digest":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}},"id":"source-digest","variants":[{"digest":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","platform":{"os":"linux","architecture":"arm64","variant":"v8"}}]}]\n'
    exit 0
    ;;
  image:save)
    archive=""
    shift 2
    while [ "$#" -gt 0 ]; do
      case "$1" in
        --output)
          archive="$2"
          shift 2
          ;;
        *)
          shift
          ;;
      esac
    done
    test -n "$archive"
    printf 'fake-oci-archive\n' > "$archive"
    printf 'image-save:%s\n' "$archive" >> "$container_log"
    exit 0
    ;;
  machine:create)
    if [ "$VAT_FAKE_K8S_CREATE_MODE" = "uncertain" ]; then
      : > "$state"
      echo "simulated client-side create failure after allocation" >&2
      exit 1
    fi
    : > "$state"
    exit 0
    ;;
  machine:inspect)
    if [ "$VAT_FAKE_K3S_DIAGNOSTIC_MODE" = "fail" ] \
      && [ -e "$state.bootstrap-failed" ] \
      && [ ! -e "$state.diagnostic-machine-inspect-failed" ]; then
      : > "$state.diagnostic-machine-inspect-failed"
      echo "simulated machine inspect diagnostic failure" >&2
      exit 75
    fi
    if [ -e "$state" ]; then
      printf '[{"status":"running","containerId":"%s","ipAddress":"%s"}]\n' "$backing_id" "$backing_ip"
      exit 0
    fi
    echo "notFound: \"container machine with ID $3 not found\"" >&2
    exit 1
    ;;
  machine:delete)
    rm -f "$state"
    exit 0
    ;;
  machine:logs)
    if [ "$VAT_FAKE_K3S_DIAGNOSTIC_MODE" = "fail" ] && [ -e "$state.bootstrap-failed" ]; then
      echo "simulated machine log diagnostic failure" >&2
      exit 74
    fi
    printf 'fake machine boot log evidence\n'
    exit 0
    ;;
  logs:owned-backing)
    if [ "$VAT_FAKE_K3S_DIAGNOSTIC_MODE" = "fail" ] && [ -e "$state.bootstrap-failed" ]; then
      echo "simulated backing log diagnostic failure" >&2
      exit 73
    fi
    printf 'fake backing container log evidence\n'
    exit 0
    ;;
  exec:owned-backing)
    printf 'exec:%s\n' "$*" >> "$container_log"
    case "$*" in
      *INSTALL_K3S_VERSION*)
        if [ "$VAT_FAKE_K3S_INSTALL_MODE" = "fail" ]; then
          : > "$state.bootstrap-failed"
          echo "simulated pinned K3s installation failure" >&2
          exit 71
        fi
        ;;
      *vat-k3s-install.log*|*systemctl\ status\ k3s*)
        if [ "$VAT_FAKE_K3S_DIAGNOSTIC_MODE" = "fail" ] && [ -e "$state.bootstrap-failed" ]; then
          echo "simulated guest diagnostic failure" >&2
          exit 72
        fi
        ;;
    esac
    if [ "$3" = "id" ]; then
      printf '0\n'
    fi
    exit 0
    ;;
  copy:owned-backing:/etc/rancher/k3s/k3s.yaml)
    printf 'clusters:\n- cluster:\n    server: https://127.0.0.1:6443\n' > "$3"
    exit 0
    ;;
  copy:*)
    case "$3" in
      owned-backing:/tmp/vat-k8s-image-*.oci.tar)
        test -f "$2"
        printf 'image-copy:%s|%s\n' "$2" "$3" >> "$container_log"
        exit 0
        ;;
    esac
    ;;
esac
echo "unexpected fake container argv: $*" >&2
exit 99
"#,
    );
    make_executable(
        &bin.join("kubectl"),
        r#"#!/bin/sh
printf '%s|%s|%s\n' "$KUBECONFIG" "$VAT_K8S_CACHE_DIR" "$HOME" >> "$VAT_FAKE_KUBECTL_LOG"
test -f "$KUBECONFIG"
port_forward=0
cache=""
mapping=""
resource=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --cache-dir)
      cache="$2"
      shift 2
      ;;
    port-forward)
      port_forward=1
      shift
      ;;
    service/*)
      resource="$1"
      shift
      ;;
    :*|[0-9]*:[0-9]*)
      mapping="$1"
      shift
      ;;
    --namespace|--kubeconfig)
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done
if [ "$port_forward" = 1 ]; then
  test -n "$cache"
  test -d "$cache"
  test -n "$resource"
  test -n "$mapping"
  case "$mapping" in
    :*)
      local_port=""
      remote_port="$(printf '%s' "$mapping" | cut -c 2-)"
      ;;
    *)
      local_port="$(printf '%s' "$mapping" | cut -d: -f1)"
      remote_port="$(printf '%s' "$mapping" | cut -d: -f2)"
      ;;
  esac
  if [ -z "$local_port" ]; then
    local_port=38123
  fi
  printf 'forward-start:%s|%s|%s\n' "$resource" "$local_port" "$remote_port" >> "$VAT_FAKE_KUBECTL_LOG"
  printf 'Forwarding from 127.0.0.1:%s -> %s\n' "$local_port" "$remote_port"
  trap 'printf "forward-stop\n" >> "$VAT_FAKE_KUBECTL_LOG"; exit 0' TERM INT
  while :; do sleep 1; done
fi
if [ -n "${VAT_FAKE_KUBECTL_DELAY_SECONDS:-}" ]; then
  sleep "$VAT_FAKE_KUBECTL_DELAY_SECONDS"
fi
if [ "$VAT_FAKE_KUBECTL_MODE" = "fail" ]; then
  echo "simulated K3s API unavailable" >&2
  exit 79
fi
exit 0
"#,
    );
    make_executable(
        &bin.join("agent-child"),
        r#"#!/bin/sh
printf '%s|%s|%s|%s\n' "$KUBECONFIG" "$VAT_K8S_CACHE_DIR" "$VAT_K8S_API_SERVER" "$HOME" >> "$VAT_FAKE_CHILD_LOG"
test -f "$KUBECONFIG"
test -d "$VAT_K8S_CACHE_DIR"
test "$VAT_K8S_API_SERVER" = "https://192.168.64.17:6443"
exit "$VAT_FAKE_CHILD_EXIT"
"#,
    );
    make_executable(
        &bin.join("agent-json-child"),
        r#"#!/bin/sh
printf '%s|%s|%s|%s\n' "$KUBECONFIG" "$VAT_K8S_CACHE_DIR" "$VAT_K8S_API_SERVER" "$HOME" >> "$VAT_FAKE_CHILD_LOG"
test -f "$KUBECONFIG"
test -d "$VAT_K8S_CACHE_DIR"
test "$VAT_K8S_API_SERVER" = "https://192.168.64.17:6443"
printf '%s' "${VAT_FAKE_JSON_CHILD_STDOUT:-json-child-stdout}"
printf '%s' "${VAT_FAKE_JSON_CHILD_STDERR:-json-child-stderr}" >&2
exit "${VAT_FAKE_JSON_CHILD_EXIT:-0}"
"#,
    );
    make_executable(
        &bin.join("agent-timeout-child"),
        r#"#!/bin/sh
set -eu
test -f "$KUBECONFIG"
test -d "$VAT_K8S_CACHE_DIR"
printf '%s\n' "$$" > "$VAT_FAKE_EXEC_TIMEOUT_PID"
: > "$VAT_FAKE_EXEC_TIMEOUT_READY"
trap 'exit 0' TERM INT
while :; do sleep 1; done
"#,
    );
    make_executable(
        &bin.join("forward-child"),
        r#"#!/bin/sh
printf '%s|%s|%s|%s|%s|%s|%s|%s|%s\n' \
  "$KUBECONFIG" \
  "$VAT_K8S_CACHE_DIR" \
  "$VAT_K8S_API_SERVER" \
  "$VAT_K8S_EPHEMERAL" \
  "$VAT_HOME" \
  "$VAT_K8S_PORT_FORWARD_HOST" \
  "$VAT_K8S_PORT_FORWARD_PORT" \
  "$VAT_K8S_PORT_FORWARD_RESOURCE" \
  "$VAT_K8S_PORT_FORWARD_NAMESPACE" >> "$VAT_FAKE_FORWARD_CHILD_LOG"
test -z "$KUBECONFIG"
test -z "$VAT_K8S_CACHE_DIR"
test -z "$VAT_K8S_API_SERVER"
test -z "$VAT_K8S_EPHEMERAL"
test -z "$VAT_HOME"
test "$VAT_K8S_PORT_FORWARD_HOST" = "127.0.0.1"
test "$VAT_K8S_PORT_FORWARD_PORT" = "38123"
test "$VAT_K8S_PORT_FORWARD_RESOURCE" = "service/api"
test "$VAT_K8S_PORT_FORWARD_NAMESPACE" = "default"
test -n "$HOME"
printf '%s' "${VAT_FAKE_FORWARD_CHILD_STDOUT:-}"
exit "$VAT_FAKE_FORWARD_CHILD_EXIT"
"#,
    );
    make_executable(
        &bin.join("forward-json-child"),
        r#"#!/bin/sh
printf '%s|%s|%s|%s|%s|%s|%s|%s|%s\n' \
  "$KUBECONFIG" \
  "$VAT_K8S_CACHE_DIR" \
  "$VAT_K8S_API_SERVER" \
  "$VAT_K8S_EPHEMERAL" \
  "$VAT_HOME" \
  "$VAT_K8S_PORT_FORWARD_HOST" \
  "$VAT_K8S_PORT_FORWARD_PORT" \
  "$VAT_K8S_PORT_FORWARD_RESOURCE" \
  "$VAT_K8S_PORT_FORWARD_NAMESPACE" >> "$VAT_FAKE_FORWARD_JSON_CHILD_LOG"
test -z "$KUBECONFIG"
test -z "$VAT_K8S_CACHE_DIR"
test -z "$VAT_K8S_API_SERVER"
test -z "$VAT_K8S_EPHEMERAL"
test -z "$VAT_HOME"
test "$VAT_K8S_PORT_FORWARD_HOST" = "127.0.0.1"
test "$VAT_K8S_PORT_FORWARD_PORT" = "38123"
test "$VAT_K8S_PORT_FORWARD_RESOURCE" = "service/api"
test "$VAT_K8S_PORT_FORWARD_NAMESPACE" = "default"
test -n "$HOME"
printf '%s' "${VAT_FAKE_FORWARD_JSON_CHILD_STDOUT:-forward-json-stdout}"
printf '%s' "${VAT_FAKE_FORWARD_JSON_CHILD_STDERR:-forward-json-stderr}" >&2
exit "${VAT_FAKE_FORWARD_JSON_CHILD_EXIT:-0}"
"#,
    );
    make_executable(
        &bin.join("forward-wait-child"),
        r#"#!/bin/sh
while [ ! -e "$VAT_FAKE_FORWARD_RELEASE" ]; do sleep 1; done
exit 0
"#,
    );
    make_executable(
        &bin.join("forward-background-child"),
        r#"#!/bin/sh
set -eu
(
  trap '' TERM INT
  while :; do sleep 1; done
) &
pid=$!
printf '%s\n' "$pid" > "$VAT_FAKE_FORWARD_DESCENDANT_PID"
: > "$VAT_FAKE_FORWARD_DESCENDANT_READY"
exit 0
"#,
    );
    make_executable(
        &bin.join("forward-blocker"),
        r#"#!/bin/sh
set -eu
printf '%s\n' "$$" > "$VAT_FAKE_FORWARD_BLOCKER_PID"
: > "$VAT_FAKE_FORWARD_BLOCKER_READY"
while :; do sleep 1; done
"#,
    );
}

fn configure_fake_k8s_command<'a>(
    command: &'a mut Command,
    bin: &Path,
    vat_home: &Path,
    state: &Path,
    root: &Path,
) -> &'a mut Command {
    command
        .env("PATH", path_with(bin))
        .env("VAT_HOME", vat_home)
        .env("VAT_FAKE_K8S_STATE", state)
        .env("VAT_FAKE_KUBECTL_LOG", root.join("kubectl.log"))
        .env("VAT_FAKE_CHILD_LOG", root.join("child.log"))
        .env("VAT_FAKE_FORWARD_CHILD_LOG", root.join("forward-child.log"))
        .env(
            "VAT_FAKE_FORWARD_JSON_CHILD_LOG",
            root.join("forward-json-child.log"),
        )
        .env("VAT_FAKE_CONTAINER_LOG", root.join("container.log"))
}

fn create_fake_leased_session(bin: &Path, vat_home: &Path, state: &Path, root: &Path) -> String {
    let mut create = Command::new(vat_bin());
    let created = configure_fake_k8s_command(&mut create, bin, vat_home, state, root)
        .args([
            "k8s",
            "session",
            "create",
            "--image",
            "fixture/systemd:k3s",
            "--ttl",
            "10m",
        ])
        .output()
        .expect("create leased K3s session");
    assert!(
        created.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&created.stdout),
        String::from_utf8_lossy(&created.stderr)
    );
    serde_json::from_slice::<Value>(&created.stdout).expect("create JSON")["id"]
        .as_str()
        .expect("leased session id")
        .to_string()
}

fn delete_fake_leased_session(bin: &Path, vat_home: &Path, state: &Path, root: &Path, id: &str) {
    let mut delete = Command::new(vat_bin());
    let deleted = configure_fake_k8s_command(&mut delete, bin, vat_home, state, root)
        .args(["k8s", "session", "delete", id])
        .output()
        .expect("delete leased K3s session");
    assert!(
        deleted.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&deleted.stdout),
        String::from_utf8_lossy(&deleted.stderr)
    );
}

fn create_private_directory(path: &Path) {
    fs::create_dir(path).expect("create private directory");
    let mut permissions = fs::metadata(path)
        .expect("private directory metadata")
        .permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(path, permissions).expect("restrict private directory");
}

fn write_private_marker(path: &Path, marker: &Value) {
    fs::write(
        path,
        serde_json::to_vec(marker).expect("serialize port-forward marker"),
    )
    .expect("write port-forward marker");
    let mut permissions = fs::metadata(path)
        .expect("port-forward marker metadata")
        .permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(path, permissions).expect("restrict port-forward marker");
}

fn wait_for_path(path: &Path, label: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if path.exists() {
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("timed out waiting for {label} at {}", path.display());
}

fn wait_for_host_started_marker(path: &Path) -> Value {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if let Ok(bytes) = fs::read(path) {
            if let Ok(marker) = serde_json::from_slice::<Value>(&bytes) {
                if marker["host_started"] == true {
                    return marker;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!(
        "timed out waiting for host-started port-forward marker at {}",
        path.display()
    );
}

fn process_exists(pid: u32) -> bool {
    let result = unsafe { libc::kill(pid as libc::pid_t, 0) };
    if result == 0 {
        return true;
    }
    std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
}

fn process_group_exists(pgid: u32) -> bool {
    let result = unsafe { libc::kill(-(pgid as libc::pid_t), 0) };
    if result == 0 {
        return true;
    }
    std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
}

fn wait_for_process_exit(pid: u32, label: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if !process_exists(pid) {
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("{label} process {pid} remains after VAT reported cleanup");
}

fn wait_for_process_group_exit(pgid: u32, label: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if !process_group_exists(pgid) {
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("{label} process group {pgid} remains after VAT reported cleanup");
}

struct ProcessIdCleanup(Option<u32>);

impl ProcessIdCleanup {
    fn new(pid: u32) -> Self {
        Self(Some(pid))
    }

    fn disarm(&mut self) {
        self.0 = None;
    }
}

impl Drop for ProcessIdCleanup {
    fn drop(&mut self) {
        if let Some(pid) = self.0.take() {
            unsafe {
                libc::kill(pid as libc::pid_t, libc::SIGKILL);
            }
        }
    }
}

struct ProcessGroupCleanup(Option<u32>);

impl ProcessGroupCleanup {
    fn new(pgid: u32) -> Self {
        Self(Some(pgid))
    }

    fn disarm(&mut self) {
        self.0 = None;
    }
}

impl Drop for ProcessGroupCleanup {
    fn drop(&mut self) {
        if let Some(pgid) = self.0.take() {
            unsafe {
                libc::kill(-(pgid as libc::pid_t), libc::SIGKILL);
            }
        }
    }
}

/// Best-effort recovery only for the opt-in real-runtime tests. Product code
/// owns the normal exact cleanup proof; this guard keeps a failed assertion
/// from leaving a real leased K3s machine running while the test process exits.
struct RealLeasedSessionCleanup {
    vat_home: std::path::PathBuf,
    id: Option<String>,
}

impl RealLeasedSessionCleanup {
    fn new(vat_home: std::path::PathBuf, id: String) -> Self {
        Self {
            vat_home,
            id: Some(id),
        }
    }

    fn disarm(&mut self) {
        self.id = None;
    }
}

impl Drop for RealLeasedSessionCleanup {
    fn drop(&mut self) {
        let Some(id) = self.id.take() else {
            return;
        };
        let _ = Command::new(vat_bin())
            .env("VAT_HOME", &self.vat_home)
            .args(["k8s", "session", "delete", &id])
            .output();
    }
}

#[test]
fn ephemeral_session_injects_private_context_for_one_child_and_forwards_exit() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);

    let output = Command::new(vat_bin())
        .env("PATH", path_with(&bin))
        .env("VAT_HOME", root.path().join("vat-home"))
        .env("VAT_FAKE_K8S_STATE", root.path().join("machine-live"))
        .env("VAT_FAKE_KUBECTL_LOG", root.path().join("kubectl.log"))
        .env("VAT_FAKE_CHILD_LOG", root.path().join("child.log"))
        .env("VAT_FAKE_CHILD_EXIT", "42")
        .args([
            "k8s",
            "ephemeral",
            "run",
            "--image",
            "fixture/systemd:k3s",
            "--",
            "agent-child",
        ])
        .output()
        .expect("run VAT ephemeral K3s command");

    assert_eq!(
        output.status.code(),
        Some(42),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"type\":\"vat_k8s_ephemeral_result\""));
    assert!(stdout.contains("\"child_exit_code\":42"));
    assert!(stdout.contains("\"terminal\":\"cleaned_up\""));
    assert!(
        !root.path().join("machine-live").exists(),
        "exact owned machine must be deleted after child completion"
    );
    let child = fs::read_to_string(root.path().join("child.log")).expect("read child log");
    let fields: Vec<_> = child.trim().split('|').collect();
    assert_eq!(fields.len(), 4);
    assert!(fields[0].contains("vat-k8s-ephemeral-"));
    assert!(fields[1].contains("kubectl-cache"));
    assert_eq!(fields[2], "https://192.168.64.17:6443");
    assert!(fields[3].contains("vat-k8s-ephemeral-"));
    assert!(
        !Path::new(fields[0]).exists(),
        "private kubeconfig must be removed before VAT returns"
    );
    assert!(
        !root.path().join("vat-home").join("k8s-ephemeral").exists()
            || fs::read_dir(root.path().join("vat-home").join("k8s-ephemeral"))
                .expect("read session directory")
                .next()
                .is_none(),
        "successful cleanup removes the recovery marker"
    );
}

#[test]
fn failed_k3s_bootstrap_keeps_primary_error_and_diagnostics_without_blocking_cleanup() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");

    let mut create = Command::new(vat_bin());
    let failed = configure_fake_k8s_command(&mut create, &bin, &vat_home, &state, root.path())
        .env("VAT_FAKE_K3S_INSTALL_MODE", "fail")
        .env("VAT_FAKE_K3S_DIAGNOSTIC_MODE", "fail")
        .args([
            "k8s",
            "session",
            "create",
            "--image",
            "fixture/systemd:k3s",
            "--ttl",
            "10m",
        ])
        .output()
        .expect("fail leased K3s bootstrap");
    let diagnostic = format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&failed.stdout),
        String::from_utf8_lossy(&failed.stderr)
    );
    assert!(
        !failed.status.success(),
        "failed bootstrap must not report success: {diagnostic}"
    );
    assert!(
        diagnostic.contains("install pinned K3s in the owned Apple guest failed")
            && diagnostic.contains("simulated pinned K3s installation failure"),
        "the original install failure must remain visible after diagnostics: {diagnostic}"
    );
    let primary_error = diagnostic
        .find("install pinned K3s in the owned Apple guest failed")
        .expect("primary bootstrap error");
    let diagnostics_header = diagnostic
        .find("K3s bootstrap diagnostics")
        .expect("diagnostic header");
    assert!(
        primary_error < diagnostics_header,
        "the root bootstrap error must render before advisory diagnostics: {diagnostic}"
    );
    for label in [
        "guest_install_log:",
        "guest_k3s_system:",
        "backing_container_logs:",
        "machine_boot_log:",
        "machine_inspect:",
        "container_system_status:",
    ] {
        assert!(
            diagnostic.contains(label),
            "missing bounded bootstrap diagnostic {label:?}: {diagnostic}"
        );
    }
    assert!(
        diagnostic.contains("simulated guest diagnostic failure")
            && diagnostic.contains("simulated backing log diagnostic failure")
            && diagnostic.contains("simulated machine log diagnostic failure")
            && diagnostic.contains("simulated machine inspect diagnostic failure")
            && diagnostic.contains("simulated container system diagnostic failure"),
        "diagnostic probe failures must be reported without replacing cleanup: {diagnostic}"
    );
    assert!(
        !state.exists(),
        "exact owned machine must still be removed after failed diagnostics"
    );
    let sessions = vat_home.join("k8s-sessions");
    assert!(
        !sessions.exists()
            || fs::read_dir(&sessions)
                .expect("read leased session storage")
                .next()
                .is_none(),
        "failed bootstrap must remove private leased-session storage after exact cleanup"
    );
    let container_log = fs::read_to_string(root.path().join("container.log"))
        .expect("read fake container invocations");
    assert_eq!(
        container_log.matches("k3s --version").count(),
        1,
        "diagnostics must not rerun the potentially hung K3s version command: {container_log}"
    );
}

#[test]
fn leased_session_keeps_private_context_across_exec_then_exactly_deletes() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");

    let mut create = Command::new(vat_bin());
    let created = configure_fake_k8s_command(&mut create, &bin, &vat_home, &state, root.path())
        .args([
            "k8s",
            "session",
            "create",
            "--image",
            "fixture/systemd:k3s",
            "--ttl",
            "10m",
        ])
        .output()
        .expect("create leased K3s session");
    assert!(
        created.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&created.stdout),
        String::from_utf8_lossy(&created.stderr)
    );
    let created_json: Value = serde_json::from_slice(&created.stdout).expect("create JSON");
    assert_eq!(created_json["type"], "vat_k8s_session");
    assert_eq!(created_json["state"], "active");
    let id = created_json["id"]
        .as_str()
        .expect("leased session id")
        .to_string();
    assert!(
        state.exists(),
        "leased session must retain its exact machine"
    );

    let mut status = Command::new(vat_bin());
    let status = configure_fake_k8s_command(&mut status, &bin, &vat_home, &state, root.path())
        .args(["k8s", "session", "status", &id])
        .output()
        .expect("read leased K3s session status");
    assert!(status.status.success());
    let status_json: Value = serde_json::from_slice(&status.stdout).expect("status JSON");
    assert_eq!(status_json["id"], id);
    assert_eq!(status_json["state"], "active");
    assert_eq!(status_json["machine_state"], "present");
    assert!(
        status_json.get("api_checked").is_none() && status_json.get("api_state").is_none(),
        "status without --verify-api must retain its existing JSON contract: {status_json}"
    );

    let mut exec = Command::new(vat_bin());
    let executed = configure_fake_k8s_command(&mut exec, &bin, &vat_home, &state, root.path())
        .env("VAT_FAKE_CHILD_EXIT", "42")
        .args(["k8s", "session", "exec", &id, "--", "agent-child"])
        .output()
        .expect("execute leased K3s child");
    assert_eq!(
        executed.status.code(),
        Some(42),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&executed.stdout),
        String::from_utf8_lossy(&executed.stderr)
    );
    let terminal: Value = String::from_utf8_lossy(&executed.stdout)
        .lines()
        .rev()
        .find_map(|line| serde_json::from_str(line).ok())
        .expect("leased exec terminal JSON");
    assert_eq!(terminal["type"], "vat_k8s_session_exec");
    assert_eq!(terminal["state"], "active");
    assert!(state.exists(), "exec must not tear down an active lease");

    let child = fs::read_to_string(root.path().join("child.log")).expect("read child log");
    let fields: Vec<_> = child.trim().split('|').collect();
    assert_eq!(fields.len(), 4);
    assert!(
        Path::new(fields[0]).exists(),
        "leased kubeconfig remains for later exec"
    );
    assert!(fields[0].contains("k8s-sessions"));
    assert_eq!(fields[2], "https://192.168.64.17:6443");

    let mut delete = Command::new(vat_bin());
    let deleted = configure_fake_k8s_command(&mut delete, &bin, &vat_home, &state, root.path())
        .args(["k8s", "session", "delete", &id])
        .output()
        .expect("delete leased K3s session");
    assert!(
        deleted.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&deleted.stdout),
        String::from_utf8_lossy(&deleted.stderr)
    );
    let deleted_json: Value = serde_json::from_slice(&deleted.stdout).expect("delete JSON");
    assert_eq!(deleted_json["type"], "vat_k8s_session_delete");
    assert_eq!(deleted_json["terminal"], "cleaned_up");
    assert!(
        !state.exists(),
        "delete must remove the exact owned machine"
    );
    assert!(
        !Path::new(fields[0]).exists(),
        "delete must remove the persisted private kubeconfig"
    );
    assert!(
        !vat_home.join("k8s-sessions").join(&id).exists(),
        "delete must remove the session marker and cache directory"
    );
}

#[test]
fn leased_session_exec_json_emits_one_bounded_agent_document_and_preserves_child_exit() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");
    let id = create_fake_leased_session(&bin, &vat_home, &state, root.path());
    let marker_path = vat_home.join("k8s-sessions").join(&id).join("session.json");
    let marker_before = fs::read(&marker_path).expect("read session marker before JSON exec");

    let mut exec = Command::new(vat_bin());
    let output = configure_fake_k8s_command(&mut exec, &bin, &vat_home, &state, root.path())
        .env("VAT_FAKE_JSON_CHILD_STDOUT", "kubectl-json-output\\n")
        .env("VAT_FAKE_JSON_CHILD_STDERR", "kubectl-warning\\n")
        .env("VAT_FAKE_JSON_CHILD_EXIT", "42")
        .args([
            "k8s",
            "session",
            "exec",
            "--format",
            "json",
            &id,
            "--",
            "agent-json-child",
        ])
        .output()
        .expect("execute leased K3s JSON child");
    assert_eq!(
        output.status.code(),
        Some(42),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("JSON exec stdout is UTF-8");
    assert_eq!(
        stdout.lines().count(),
        1,
        "JSON exec must not replay raw child streams: {stdout:?}"
    );
    let json: Value = serde_json::from_str(&stdout).expect("one JSON exec document");
    assert_eq!(json["schema"], "vat.k8s.session.exec.v1");
    assert_eq!(json["format"], "vat_json");
    assert_eq!(json["type"], "vat_k8s_session_exec");
    assert_eq!(json["id"], id);
    assert_eq!(json["state"], "active");
    assert_eq!(json["child_exit_code"], 42);
    assert_eq!(json["stdout"], "kubectl-json-output\\n");
    assert_eq!(json["stderr"], "kubectl-warning\\n");
    assert_eq!(json["stdout_truncated"], false);
    assert_eq!(json["stderr_truncated"], false);
    assert_eq!(json["stdout_utf8_lossy"], false);
    assert_eq!(json["stderr_utf8_lossy"], false);
    assert_eq!(json["api_verified"], true);
    assert_eq!(json["runtime_invoked"], true);
    assert_eq!(json["session_record_mutated"], false);
    assert_eq!(
        json["next"],
        format!("vat k8s session status --verify-api {id}")
    );
    assert_eq!(
        fs::read(&marker_path).expect("read session marker after JSON exec"),
        marker_before,
        "JSON exec must not modify the active lease record"
    );
    assert!(
        !stdout.contains("k8s-sessions") && !stdout.contains("kubectl-cache"),
        "JSON exec must not expose private credential paths: {stdout}"
    );
    assert!(
        root.path().join("child.log").exists(),
        "JSON child must receive the private credentials without rendering them"
    );

    delete_fake_leased_session(&bin, &vat_home, &state, root.path(), &id);
}

#[test]
fn leased_session_exec_timeout_reaps_its_owned_group_and_removes_marker() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");
    let id = create_fake_leased_session(&bin, &vat_home, &state, root.path());
    let ready_path = root.path().join("exec-timeout.ready");
    let pid_path = root.path().join("exec-timeout.pid");
    let exec_marker = vat_home.join("k8s-sessions").join(&id).join("exec.json");

    let mut command = Command::new(vat_bin());
    let exec = configure_fake_k8s_command(&mut command, &bin, &vat_home, &state, root.path())
        .env("VAT_FAKE_EXEC_TIMEOUT_READY", &ready_path)
        .env("VAT_FAKE_EXEC_TIMEOUT_PID", &pid_path)
        .args([
            "k8s",
            "session",
            "exec",
            "--format",
            "json",
            "--timeout",
            "1",
            &id,
            "--",
            "agent-timeout-child",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start bounded leased K3s JSON exec");
    wait_for_path(&ready_path, "bounded exec child readiness");
    wait_for_path(&exec_marker, "bounded exec recovery marker");
    let pgid = fs::read_to_string(&pid_path)
        .expect("read bounded exec child pid")
        .trim()
        .parse::<u32>()
        .expect("parse bounded exec child pid");
    let marker: Value = serde_json::from_slice(
        &fs::read(&exec_marker).expect("read bounded exec recovery marker"),
    )
    .expect("parse bounded exec recovery marker");
    assert_eq!(marker["state"], "running");
    assert_eq!(marker["pgid"], pgid);
    let mut group_cleanup = ProcessGroupCleanup::new(pgid);

    let output = exec
        .wait_with_output()
        .expect("collect timed-out leased K3s JSON exec");
    assert!(
        !output.status.success(),
        "timeout must fail closed; stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "timed-out JSON exec must emit no partial result: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("exceeded its --timeout"),
        "timeout error must name the bounded execution contract: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    wait_for_process_group_exit(pgid, "timed-out K3s session exec");
    group_cleanup.disarm();
    assert!(
        !exec_marker.exists(),
        "VAT must remove the exec marker only after it confirms group cleanup"
    );

    delete_fake_leased_session(&bin, &vat_home, &state, root.path(), &id);
}

#[test]
fn leased_session_exec_without_timeout_is_lease_bound_and_interruptible() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");
    let id = create_fake_leased_session(&bin, &vat_home, &state, root.path());
    let ready_path = root.path().join("exec-default.ready");
    let pid_path = root.path().join("exec-default.pid");
    let exec_marker = vat_home.join("k8s-sessions").join(&id).join("exec.json");

    let mut command = Command::new(vat_bin());
    let exec = configure_fake_k8s_command(&mut command, &bin, &vat_home, &state, root.path())
        .env("VAT_FAKE_EXEC_TIMEOUT_READY", &ready_path)
        .env("VAT_FAKE_EXEC_TIMEOUT_PID", &pid_path)
        .args([
            "k8s",
            "session",
            "exec",
            "--format",
            "json",
            &id,
            "--",
            "agent-timeout-child",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start lease-bound K3s JSON exec without explicit timeout");
    wait_for_path(&ready_path, "default-bound exec child readiness");
    wait_for_path(&exec_marker, "default-bound exec recovery marker");
    let pgid = fs::read_to_string(&pid_path)
        .expect("read default-bound exec child pid")
        .trim()
        .parse::<u32>()
        .expect("parse default-bound exec child pid");
    let mut group_cleanup = ProcessGroupCleanup::new(pgid);
    let signal_result = unsafe { libc::kill(exec.id() as libc::pid_t, libc::SIGTERM) };
    assert_eq!(signal_result, 0, "send SIGTERM to VAT session exec parent");

    let output = exec
        .wait_with_output()
        .expect("collect interrupted leased K3s JSON exec");
    assert_eq!(
        output.status.code(),
        Some(128 + libc::SIGTERM),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "interrupted JSON exec must emit no terminal result: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    wait_for_process_group_exit(pgid, "interrupted K3s session exec");
    group_cleanup.disarm();
    assert!(
        !exec_marker.exists(),
        "default lease-bound exec must remove its marker after interruption cleanup"
    );

    delete_fake_leased_session(&bin, &vat_home, &state, root.path(), &id);
}

#[test]
fn leased_session_exec_holds_operation_lock_until_owned_group_cleanup() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");
    let id = create_fake_leased_session(&bin, &vat_home, &state, root.path());
    let ready_path = root.path().join("exec-lock.ready");
    let pid_path = root.path().join("exec-lock.pid");
    let exec_marker = vat_home.join("k8s-sessions").join(&id).join("exec.json");

    let mut command = Command::new(vat_bin());
    let exec = configure_fake_k8s_command(&mut command, &bin, &vat_home, &state, root.path())
        .env("VAT_FAKE_EXEC_TIMEOUT_READY", &ready_path)
        .env("VAT_FAKE_EXEC_TIMEOUT_PID", &pid_path)
        .args([
            "k8s",
            "session",
            "exec",
            "--format",
            "json",
            &id,
            "--",
            "agent-timeout-child",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start lock-owning leased K3s JSON exec");
    wait_for_path(&ready_path, "lock-owning exec child readiness");
    wait_for_path(&exec_marker, "lock-owning exec recovery marker");
    let pgid = fs::read_to_string(&pid_path)
        .expect("read lock-owning exec child pid")
        .trim()
        .parse::<u32>()
        .expect("parse lock-owning exec child pid");
    let mut group_cleanup = ProcessGroupCleanup::new(pgid);

    let mut delete = Command::new(vat_bin());
    let deleted = configure_fake_k8s_command(&mut delete, &bin, &vat_home, &state, root.path())
        .args(["k8s", "session", "delete", &id])
        .output()
        .expect("reject delete while leased exec owns the operation lock");
    assert!(!deleted.status.success());
    assert!(
        String::from_utf8_lossy(&deleted.stderr).contains("busy with another VAT operation"),
        "delete must not race a credentialed live exec: {}",
        String::from_utf8_lossy(&deleted.stderr)
    );
    assert!(
        state.exists() && exec_marker.exists() && process_group_exists(pgid),
        "busy delete must retain the machine, credentials marker, and live owned group"
    );

    let signal_result = unsafe { libc::kill(exec.id() as libc::pid_t, libc::SIGTERM) };
    assert_eq!(signal_result, 0, "interrupt lock-owning VAT session exec");
    let output = exec
        .wait_with_output()
        .expect("collect interrupted lock-owning leased K3s JSON exec");
    assert_eq!(output.status.code(), Some(128 + libc::SIGTERM));
    wait_for_process_group_exit(pgid, "lock-owning K3s session exec");
    group_cleanup.disarm();
    assert!(
        !exec_marker.exists(),
        "the marker may disappear only after the lock-owning exec cleans its group"
    );

    delete_fake_leased_session(&bin, &vat_home, &state, root.path(), &id);
}

#[test]
fn leased_session_exec_without_timeout_stops_at_remaining_lease_ttl() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");
    let id = create_fake_leased_session(&bin, &vat_home, &state, root.path());
    let session_marker = vat_home.join("k8s-sessions").join(&id).join("session.json");
    let mut session: Value = serde_json::from_slice(
        &fs::read(&session_marker).expect("read active lease marker"),
    )
    .expect("parse active lease marker");
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time after epoch")
        .as_millis();
    session["expires_unix_ms"] = serde_json::json!(now_ms + 1_500_u128);
    fs::write(
        &session_marker,
        serde_json::to_vec(&session).expect("serialize short active lease"),
    )
    .expect("write short active lease");
    let ready_path = root.path().join("exec-lease-default.ready");
    let pid_path = root.path().join("exec-lease-default.pid");
    let exec_marker = vat_home.join("k8s-sessions").join(&id).join("exec.json");

    let mut command = Command::new(vat_bin());
    let exec = configure_fake_k8s_command(&mut command, &bin, &vat_home, &state, root.path())
        .env("VAT_FAKE_EXEC_TIMEOUT_READY", &ready_path)
        .env("VAT_FAKE_EXEC_TIMEOUT_PID", &pid_path)
        .args([
            "k8s",
            "session",
            "exec",
            "--format",
            "json",
            &id,
            "--",
            "agent-timeout-child",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start default-TTL K3s JSON exec");
    wait_for_path(&ready_path, "default-TTL exec child readiness");
    wait_for_path(&exec_marker, "default-TTL exec recovery marker");
    let pgid = fs::read_to_string(&pid_path)
        .expect("read default-TTL exec child pid")
        .trim()
        .parse::<u32>()
        .expect("parse default-TTL exec child pid");
    let mut group_cleanup = ProcessGroupCleanup::new(pgid);

    let output = exec
        .wait_with_output()
        .expect("collect default-TTL leased K3s JSON exec");
    assert!(!output.status.success());
    assert!(
        output.stdout.is_empty(),
        "default-TTL JSON exec must emit no partial result: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("exceeded its remaining lease TTL"),
        "omitted timeout must use the remaining lease as its bound: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    wait_for_process_group_exit(pgid, "default-TTL K3s session exec");
    group_cleanup.disarm();
    assert!(
        !exec_marker.exists(),
        "default-TTL exec must remove its marker after confirmed group cleanup"
    );

    delete_fake_leased_session(&bin, &vat_home, &state, root.path(), &id);
}

#[test]
fn leased_session_exec_rejects_timeout_longer_than_remaining_lease_before_child_spawn() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");
    let id = create_fake_leased_session(&bin, &vat_home, &state, root.path());
    let session_marker = vat_home.join("k8s-sessions").join(&id).join("session.json");
    let mut marker: Value = serde_json::from_slice(
        &fs::read(&session_marker).expect("read active lease marker"),
    )
    .expect("parse active lease marker");
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time after epoch")
        .as_millis();
    marker["expires_unix_ms"] = serde_json::json!(now_ms + 30_000_u128);
    fs::write(
        &session_marker,
        serde_json::to_vec(&marker).expect("serialize short active lease"),
    )
    .expect("write short active lease");
    let ready_path = root.path().join("exec-too-long.ready");
    let pid_path = root.path().join("exec-too-long.pid");
    let exec_marker = vat_home.join("k8s-sessions").join(&id).join("exec.json");

    let mut command = Command::new(vat_bin());
    let output = configure_fake_k8s_command(&mut command, &bin, &vat_home, &state, root.path())
        .env("VAT_FAKE_EXEC_TIMEOUT_READY", &ready_path)
        .env("VAT_FAKE_EXEC_TIMEOUT_PID", &pid_path)
        .args([
            "k8s",
            "session",
            "exec",
            "--format",
            "json",
            "--timeout",
            "31",
            &id,
            "--",
            "agent-timeout-child",
        ])
        .output()
        .expect("reject overlong bounded K3s exec");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("must not exceed its remaining lease TTL"),
        "timeout must be rejected before spawn: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !ready_path.exists() && !pid_path.exists() && !exec_marker.exists(),
        "an overlong requested timeout must not create a child or durable exec marker"
    );

    delete_fake_leased_session(&bin, &vat_home, &state, root.path(), &id);
}

#[test]
fn leased_session_exec_recovery_marker_blocks_lifecycle_until_its_recorded_group_is_absent() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");
    let id = create_fake_leased_session(&bin, &vat_home, &state, root.path());
    let exec_marker = vat_home.join("k8s-sessions").join(&id).join("exec.json");

    let mut live_command = Command::new("/bin/sh");
    live_command
        .args(["-ec", "while :; do sleep 1; done"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    live_command.process_group(0);
    let mut live_group = live_command.spawn().expect("start live recovered exec group");
    let pgid = live_group.id();
    let mut group_cleanup = ProcessGroupCleanup::new(pgid);
    write_private_marker(
        &exec_marker,
        &serde_json::json!({
            "schema": "vat.k8s.session.exec.v1",
            "session_id": id,
            "owner_pid": std::process::id(),
            "state": "running",
            "pgid": pgid,
        }),
    );

    let mut forward = Command::new(vat_bin());
    let forward = configure_fake_k8s_command(&mut forward, &bin, &vat_home, &state, root.path())
        .args([
            "k8s",
            "session",
            "port-forward",
            "run",
            &id,
            "service/api",
            "8080",
            "--",
            "forward-child",
        ])
        .output()
        .expect("reject port-forward behind live exec marker");
    assert!(!forward.status.success());
    assert!(
        String::from_utf8_lossy(&forward.stderr).contains("cannot authenticate an arbitrary recovered host command"),
        "port-forward must fail closed on a live exec marker: {}",
        String::from_utf8_lossy(&forward.stderr)
    );
    assert!(
        !root.path().join("forward-child.log").exists(),
        "port-forward must not start a host child while an exec marker is live"
    );

    let mut delete = Command::new(vat_bin());
    let deleted = configure_fake_k8s_command(&mut delete, &bin, &vat_home, &state, root.path())
        .args(["k8s", "session", "delete", &id])
        .output()
        .expect("reject delete behind live exec marker");
    assert!(!deleted.status.success());
    assert!(state.exists(), "delete must retain the exact owned machine");
    assert!(exec_marker.exists(), "delete must retain the exec marker");

    let session_marker = vat_home.join("k8s-sessions").join(&id).join("session.json");
    let mut session: Value = serde_json::from_slice(
        &fs::read(&session_marker).expect("read active lease marker"),
    )
    .expect("parse active lease marker");
    let created_unix_ms = session["created_unix_ms"]
        .as_u64()
        .expect("created lease timestamp fits u64");
    session["expires_unix_ms"] = serde_json::json!(created_unix_ms + 1);
    fs::write(
        &session_marker,
        serde_json::to_vec(&session).expect("serialize expired lease"),
    )
    .expect("write expired lease");
    let mut cleanup = Command::new(vat_bin());
    let cleanup = configure_fake_k8s_command(&mut cleanup, &bin, &vat_home, &state, root.path())
        .args(["k8s", "session", "cleanup", "--json"])
        .output()
        .expect("reject cleanup behind live exec marker");
    assert!(!cleanup.status.success());
    let cleanup_json: Value = serde_json::from_slice(&cleanup.stdout).expect("cleanup JSON");
    assert!(
        cleanup_json["failed"]
            .as_array()
            .expect("failed cleanup list")
            .iter()
            .filter_map(Value::as_str)
            .any(|failure| failure.contains(&id)),
        "cleanup must retain a session with a live exec marker: {cleanup_json}"
    );
    assert!(
        process_group_exists(pgid),
        "recovery must not signal an unauthenticated live process group"
    );

    let kill_result = unsafe { libc::kill(-(pgid as libc::pid_t), libc::SIGKILL) };
    assert_eq!(kill_result, 0, "stop live recovered test process group");
    let _ = live_group.wait().expect("reap live recovered test group leader");
    wait_for_process_group_exit(pgid, "manually stopped recovered exec");
    group_cleanup.disarm();
    delete_fake_leased_session(&bin, &vat_home, &state, root.path(), &id);

    let starting_id = create_fake_leased_session(&bin, &vat_home, &state, root.path());
    let starting_marker = vat_home
        .join("k8s-sessions")
        .join(&starting_id)
        .join("exec.json");
    write_private_marker(
        &starting_marker,
        &serde_json::json!({
            "schema": "vat.k8s.session.exec.v1",
            "session_id": starting_id,
            "owner_pid": std::process::id(),
            "state": "starting",
            "pgid": null,
        }),
    );
    let mut starting_delete = Command::new(vat_bin());
    let starting_delete = configure_fake_k8s_command(
        &mut starting_delete,
        &bin,
        &vat_home,
        &state,
        root.path(),
    )
    .args(["k8s", "session", "delete", &starting_id])
    .output()
    .expect("reject delete behind unconfirmed exec marker");
    assert!(!starting_delete.status.success());
    assert!(
        starting_marker.exists() && state.exists(),
        "an unconfirmed pre-spawn marker must retain the session"
    );
    fs::remove_file(&starting_marker).expect("manual recovery removes unconfirmed test marker");
    delete_fake_leased_session(&bin, &vat_home, &state, root.path(), &starting_id);
}

#[test]
fn leased_session_exec_rejects_non_json_format_before_runtime() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");

    let mut exec = Command::new(vat_bin());
    let output = configure_fake_k8s_command(&mut exec, &bin, &vat_home, &state, root.path())
        .args([
            "k8s",
            "session",
            "exec",
            "--format",
            "table",
            "k8s-invalid-format-0",
            "--",
            "agent-json-child",
        ])
        .output()
        .expect("reject non-JSON K3s session exec format");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("possible values: json"),
        "non-JSON format must fail in clap before touching the runtime: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !state.exists() && !root.path().join("container.log").exists(),
        "invalid format must not start an Apple Container command"
    );
}

#[test]
fn leased_session_exec_json_rechecks_lease_after_api_probe_before_child_spawn() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");
    let id = create_fake_leased_session(&bin, &vat_home, &state, root.path());
    let kubectl_log = root.path().join("kubectl.log");
    fs::remove_file(&kubectl_log).expect("clear bootstrap kubectl log");
    let marker_path = vat_home.join("k8s-sessions").join(&id).join("session.json");
    let mut marker: Value = serde_json::from_slice(
        &fs::read(&marker_path).expect("read active lease marker"),
    )
    .expect("parse active lease marker");
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time after epoch")
        .as_millis();
    marker["expires_unix_ms"] = serde_json::json!(now_ms + 2_000_u128);
    fs::write(
        &marker_path,
        serde_json::to_vec(&marker).expect("serialize short active lease"),
    )
    .expect("write short active lease");

    let mut exec = Command::new(vat_bin());
    let output = configure_fake_k8s_command(&mut exec, &bin, &vat_home, &state, root.path())
        .env("VAT_FAKE_KUBECTL_DELAY_SECONDS", "3")
        .args([
            "k8s",
            "session",
            "exec",
            "--format",
            "json",
            &id,
            "--",
            "agent-json-child",
        ])
        .output()
        .expect("attempt JSON exec that crosses the lease TTL");
    assert!(
        !output.status.success(),
        "an API probe that crosses the lease TTL must not spawn a child"
    );
    assert!(
        kubectl_log.exists(),
        "the bounded owned-API probe must have occurred before the expiry recheck"
    );
    assert!(
        !root.path().join("child.log").exists(),
        "expired lease must not deliver private credentials to a child"
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("expiry error JSON");
    assert_eq!(json["type"], "error");
    assert_eq!(json["code"], "k8s_session_expired");
    assert_eq!(json["id"], id);

    delete_fake_leased_session(&bin, &vat_home, &state, root.path(), &id);
}

#[test]
fn leased_session_exec_masks_private_paths_when_credentials_or_api_probe_fail() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");

    let credential_id = create_fake_leased_session(&bin, &vat_home, &state, root.path());
    let credential_directory = vat_home.join("k8s-sessions").join(&credential_id);
    let credential_paths = [
        credential_directory.clone(),
        credential_directory.join("credentials/kubeconfig"),
        credential_directory.join("credentials/kubectl-cache"),
        credential_directory.join("credentials/home"),
    ];
    fs::remove_file(&credential_paths[1]).expect("remove private kubeconfig");
    let mut credential_exec = Command::new(vat_bin());
    let credential_failure = configure_fake_k8s_command(
        &mut credential_exec,
        &bin,
        &vat_home,
        &state,
        root.path(),
    )
    .args([
        "k8s",
        "session",
        "exec",
        "--format",
        "json",
        &credential_id,
        "--",
        "agent-json-child",
    ])
    .output()
    .expect("fail closed for missing private K3s credential");
    assert!(!credential_failure.status.success());
    let credential_rendered = format!(
        "{}\n{}",
        String::from_utf8_lossy(&credential_failure.stdout),
        String::from_utf8_lossy(&credential_failure.stderr)
    );
    for path in &credential_paths {
        assert!(
            !credential_rendered.contains(&path.display().to_string()),
            "credential failure must not expose private path {}: {credential_rendered}",
            path.display()
        );
    }
    assert!(
        !root.path().join("child.log").exists(),
        "invalid credentials must not reach a foreground child"
    );
    delete_fake_leased_session(&bin, &vat_home, &state, root.path(), &credential_id);

    let api_id = create_fake_leased_session(&bin, &vat_home, &state, root.path());
    let api_directory = vat_home.join("k8s-sessions").join(&api_id);
    let api_paths = [
        api_directory.clone(),
        api_directory.join("credentials/kubeconfig"),
        api_directory.join("credentials/kubectl-cache"),
        api_directory.join("credentials/home"),
    ];
    fs::remove_file(bin.join("kubectl")).expect("remove fake kubectl for spawn failure");
    let mut api_exec = Command::new(vat_bin());
    let api_failure = configure_fake_k8s_command(&mut api_exec, &bin, &vat_home, &state, root.path())
        .args([
            "k8s",
            "session",
            "exec",
            "--format",
            "json",
            &api_id,
            "--",
            "agent-json-child",
        ])
        .output()
        .expect("fail closed when private API probe cannot spawn kubectl");
    assert!(!api_failure.status.success());
    let api_rendered = format!(
        "{}\n{}",
        String::from_utf8_lossy(&api_failure.stdout),
        String::from_utf8_lossy(&api_failure.stderr)
    );
    for path in &api_paths {
        assert!(
            !api_rendered.contains(&path.display().to_string()),
            "API probe failure must not expose private path {}: {api_rendered}",
            path.display()
        );
    }
    assert!(
        !root.path().join("child.log").exists(),
        "failed API probe must not reach a foreground child"
    );
    delete_fake_leased_session(&bin, &vat_home, &state, root.path(), &api_id);
}

#[test]
fn leased_session_status_verify_api_checks_exact_owned_api_without_exposing_private_paths() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");
    let id = create_fake_leased_session(&bin, &vat_home, &state, root.path());
    let session_directory = vat_home.join("k8s-sessions").join(&id);
    let marker_path = session_directory.join("session.json");
    let marker_before = fs::read(&marker_path).expect("read session marker before API status");
    let kubeconfig = session_directory.join("credentials/kubeconfig");
    let cache = session_directory.join("credentials/kubectl-cache");
    let kubectl_log = root.path().join("kubectl.log");
    fs::remove_file(&kubectl_log).expect("clear bootstrap kubectl log");

    let mut status = Command::new(vat_bin());
    let status = configure_fake_k8s_command(&mut status, &bin, &vat_home, &state, root.path())
        .args(["k8s", "session", "status", "--verify-api", &id])
        .output()
        .expect("verify leased K3s API status");
    assert!(
        status.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&status.stdout),
        String::from_utf8_lossy(&status.stderr)
    );
    let json: Value = serde_json::from_slice(&status.stdout).expect("verified status JSON");
    assert_eq!(json["type"], "vat_k8s_session_status");
    assert_eq!(json["state"], "active");
    assert_eq!(json["machine_state"], "present");
    assert_eq!(json["api_checked"], true);
    assert_eq!(json["api_state"], "reachable");
    for private_key in ["kubeconfig", "cache", "credentials"] {
        assert!(
            json.get(private_key).is_none(),
            "verified status must not expose private {private_key}: {json}"
        );
    }
    let rendered = String::from_utf8_lossy(&status.stdout);
    for private_path in [&kubeconfig, &cache, &session_directory] {
        assert!(
            !rendered.contains(&private_path.display().to_string()),
            "verified status must not expose private path {}: {rendered}",
            private_path.display()
        );
    }
    assert_eq!(
        fs::read(&marker_path).expect("read session marker after API status"),
        marker_before,
        "API status must not mutate the leased-session marker"
    );
    assert!(state.exists(), "API status must retain the owned machine");
    assert!(
        kubeconfig.exists() && cache.exists(),
        "API status must retain private credentials"
    );
    assert_eq!(
        fs::read_to_string(&kubectl_log)
            .expect("read API probe log")
            .lines()
            .count(),
        1,
        "--verify-api must reuse one bounded kubectl API probe"
    );

    delete_fake_leased_session(&bin, &vat_home, &state, root.path(), &id);
}

#[test]
fn leased_session_status_verify_api_leaves_port_forward_recovery_unchecked() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");
    let id = create_fake_leased_session(&bin, &vat_home, &state, root.path());
    let session_directory = vat_home.join("k8s-sessions").join(&id);
    let forward_marker = session_directory.join("port-forward.json");
    write_private_marker(
        &forward_marker,
        &serde_json::json!({"retained": "test recovery marker"}),
    );
    let marker_before = fs::read(&forward_marker).expect("read retained forward marker");
    let kubectl_log = root.path().join("kubectl.log");
    let container_log = root.path().join("container.log");
    fs::remove_file(&kubectl_log).expect("clear bootstrap kubectl log");
    fs::remove_file(&container_log).expect("clear bootstrap container log");

    let mut status = Command::new(vat_bin());
    let status = configure_fake_k8s_command(&mut status, &bin, &vat_home, &state, root.path())
        .args(["k8s", "session", "status", "--verify-api", &id])
        .output()
        .expect("read recovery-required K3s API status");
    assert!(
        status.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&status.stdout),
        String::from_utf8_lossy(&status.stderr)
    );
    let json: Value = serde_json::from_slice(&status.stdout).expect("recovery status JSON");
    assert_eq!(json["port_forward"], "recovery_required");
    assert_eq!(json["machine_state"], "not_checked");
    assert_eq!(json["api_checked"], false);
    assert_eq!(json["api_state"], "not_checked");
    assert_eq!(
        fs::read(&forward_marker).expect("read retained marker after status"),
        marker_before,
        "API status must not reconcile or mutate a retained port-forward marker"
    );
    assert!(
        !session_directory.join("operation.lock").exists(),
        "recovery-required status must not acquire a competing operation lock"
    );
    assert!(
        !kubectl_log.exists() && !container_log.exists(),
        "recovery-required status must not invoke kubectl or Apple Container"
    );
    assert!(
        state.exists(),
        "recovery-required status must retain the lease"
    );

    fs::remove_file(&forward_marker).expect("remove test-only recovery marker");
    delete_fake_leased_session(&bin, &vat_home, &state, root.path(), &id);
}

#[test]
fn leased_session_status_verify_api_does_not_probe_an_expired_lease() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");
    let id = create_fake_leased_session(&bin, &vat_home, &state, root.path());
    let session_directory = vat_home.join("k8s-sessions").join(&id);
    let marker_path = session_directory.join("session.json");
    let mut marker: Value =
        serde_json::from_slice(&fs::read(&marker_path).expect("read active session marker"))
            .expect("parse active session marker");
    let created = marker["created_unix_ms"]
        .as_u64()
        .expect("session marker creation time");
    marker["expires_unix_ms"] = Value::from(created + 1);
    write_private_marker(&marker_path, &marker);
    let marker_before = fs::read(&marker_path).expect("read expired session marker");
    let kubectl_log = root.path().join("kubectl.log");
    let container_log = root.path().join("container.log");
    fs::remove_file(&kubectl_log).expect("clear bootstrap kubectl log");
    fs::remove_file(&container_log).expect("clear bootstrap container log");

    let mut status = Command::new(vat_bin());
    let status = configure_fake_k8s_command(&mut status, &bin, &vat_home, &state, root.path())
        .args(["k8s", "session", "status", "--verify-api", &id])
        .output()
        .expect("read expired K3s API status");
    assert!(
        status.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&status.stdout),
        String::from_utf8_lossy(&status.stderr)
    );
    let json: Value = serde_json::from_slice(&status.stdout).expect("expired status JSON");
    assert_eq!(json["state"], "expired");
    assert_eq!(json["machine_state"], "not_checked");
    assert_eq!(json["api_checked"], false);
    assert_eq!(json["api_state"], "not_checked");
    assert_eq!(
        fs::read(&marker_path).expect("read expired marker after status"),
        marker_before,
        "expired API status must not mutate the session marker"
    );
    assert!(
        !session_directory.join("operation.lock").exists(),
        "expired API status must not acquire the operation lock"
    );
    assert!(
        !kubectl_log.exists() && !container_log.exists(),
        "expired API status must not invoke kubectl or Apple Container"
    );
    assert!(
        state.exists(),
        "expired status must retain the lease for cleanup"
    );

    delete_fake_leased_session(&bin, &vat_home, &state, root.path(), &id);
}

#[test]
fn leased_session_status_verify_api_fails_closed_for_busy_unavailable_and_mismatched_leases() {
    for mode in ["busy", "unavailable", "identity_mismatch"] {
        let root = TempDir::new().expect("temp root");
        let bin = root.path().join("bin");
        write_fake_runtime(&bin);
        let vat_home = root.path().join("vat-home");
        let state = root.path().join("machine-live");
        let id = create_fake_leased_session(&bin, &vat_home, &state, root.path());
        let session_directory = vat_home.join("k8s-sessions").join(&id);
        let marker_path = session_directory.join("session.json");
        let marker_before =
            fs::read(&marker_path).expect("read session marker before failed API status");
        let kubeconfig = session_directory.join("credentials/kubeconfig");
        let cache = session_directory.join("credentials/kubectl-cache");

        let mut held_lock = None;
        let mut status = Command::new(vat_bin());
        let status = configure_fake_k8s_command(&mut status, &bin, &vat_home, &state, root.path());
        match mode {
            "busy" => {
                let lock_path = session_directory.join("operation.lock");
                let lock = File::create(&lock_path).expect("create private operation lock");
                let mut permissions = lock
                    .metadata()
                    .expect("operation lock metadata")
                    .permissions();
                permissions.set_mode(0o600);
                lock.set_permissions(permissions)
                    .expect("restrict operation lock");
                assert_eq!(
                    unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
                    0,
                    "hold fake leased-session operation lock"
                );
                held_lock = Some(lock);
            }
            "unavailable" => {
                status.env("VAT_FAKE_KUBECTL_MODE", "fail");
            }
            "identity_mismatch" => {
                status.env("VAT_FAKE_K8S_BACKING_ID", "unexpected-backing");
            }
            _ => unreachable!("test mode is fixed above"),
        }
        let status = status
            .args(["k8s", "session", "status", "--verify-api", &id])
            .output()
            .expect("fail closed leased K3s API status");
        assert!(
            !status.status.success(),
            "{mode} must fail closed: stdout={} stderr={}",
            String::from_utf8_lossy(&status.stdout),
            String::from_utf8_lossy(&status.stderr)
        );
        let output = format!(
            "{}{}",
            String::from_utf8_lossy(&status.stdout),
            String::from_utf8_lossy(&status.stderr)
        );
        for private_path in [&kubeconfig, &cache, &session_directory] {
            assert!(
                !output.contains(&private_path.display().to_string()),
                "{mode} error must not expose private path {}: {output}",
                private_path.display()
            );
        }
        assert_eq!(
            fs::read(&marker_path).expect("read marker after failed API status"),
            marker_before,
            "{mode} must not mutate the active session marker"
        );
        assert!(
            state.exists() && kubeconfig.exists() && cache.exists(),
            "{mode} must retain the active lease and private credentials"
        );

        if let Some(lock) = held_lock.take() {
            assert_eq!(
                unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_UN) },
                0,
                "release fake leased-session operation lock"
            );
        }
        delete_fake_leased_session(&bin, &vat_home, &state, root.path(), &id);
    }
}

#[test]
fn leased_session_port_forward_only_exposes_loopback_metadata_then_cleans_up() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");

    let mut create = Command::new(vat_bin());
    let created = configure_fake_k8s_command(&mut create, &bin, &vat_home, &state, root.path())
        .args([
            "k8s",
            "session",
            "create",
            "--image",
            "fixture/systemd:k3s",
            "--ttl",
            "10m",
        ])
        .output()
        .expect("create leased K3s session");
    assert!(
        created.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&created.stdout),
        String::from_utf8_lossy(&created.stderr)
    );
    let created: Value = serde_json::from_slice(&created.stdout).expect("create JSON");
    let id = created["id"]
        .as_str()
        .expect("leased session id")
        .to_string();

    let mut forward = Command::new(vat_bin());
    let forwarded = configure_fake_k8s_command(&mut forward, &bin, &vat_home, &state, root.path())
        .env("VAT_FAKE_FORWARD_CHILD_EXIT", "42")
        .env(
            "VAT_FAKE_FORWARD_CHILD_STDOUT",
            "forward-output-without-newline",
        )
        .args([
            "k8s",
            "session",
            "port-forward",
            "run",
            &id,
            "service/api",
            "8080",
            "--",
            "forward-child",
        ])
        .output()
        .expect("run leased K3s Service port-forward");
    assert_eq!(
        forwarded.status.code(),
        Some(42),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&forwarded.stdout),
        String::from_utf8_lossy(&forwarded.stderr)
    );
    let forwarded_stdout = String::from_utf8_lossy(&forwarded.stdout);
    assert!(
        forwarded_stdout.contains("forward-output-without-newline\n{"),
        "the terminal record must start on a new line after arbitrary child output: {forwarded_stdout}"
    );
    let terminal: Value = forwarded_stdout
        .lines()
        .rev()
        .find_map(|line| serde_json::from_str(line).ok())
        .expect("port-forward terminal JSON");
    assert_eq!(terminal["type"], "vat_k8s_session_port_forward");
    assert_eq!(terminal["resource"], "service/api");
    assert_eq!(terminal["local_host"], "127.0.0.1");
    assert_eq!(terminal["local_port"], 38123);
    assert_eq!(terminal["remote_port"], 8080);
    assert_eq!(terminal["child_exit_code"], 42);
    assert_eq!(terminal["cleanup"], "confirmed");
    assert!(
        state.exists(),
        "a foreground port-forward must retain its active lease"
    );

    let child =
        fs::read_to_string(root.path().join("forward-child.log")).expect("read forward child log");
    let fields: Vec<_> = child.trim().split('|').collect();
    assert_eq!(fields.len(), 9);
    assert!(
        fields[..5].iter().all(|value| value.is_empty()),
        "the host child must not receive kubeconfig, cache, API, K3s markers, or VAT_HOME: {child}"
    );
    assert_eq!(fields[5], "127.0.0.1");
    assert_eq!(fields[6], "38123");
    assert_eq!(fields[7], "service/api");
    assert_eq!(fields[8], "default");

    let kubectl = fs::read_to_string(root.path().join("kubectl.log")).expect("read kubectl log");
    assert!(
        kubectl.contains("forward-start:service/api|38123|8080"),
        "VAT must start exactly the requested Service tunnel: {kubectl}"
    );
    assert!(
        kubectl.contains("forward-stop"),
        "VAT must terminate the forward before reporting cleanup: {kubectl}"
    );
    let session_directory = vat_home.join("k8s-sessions").join(&id);
    assert!(
        !session_directory.join("port-forward.json").exists(),
        "successful foreground cleanup removes the recovery marker"
    );
    assert!(
        !session_directory.join("port-forward").exists(),
        "successful foreground cleanup removes its private cache and HOME"
    );

    let mut delete = Command::new(vat_bin());
    let deleted = configure_fake_k8s_command(&mut delete, &bin, &vat_home, &state, root.path())
        .args(["k8s", "session", "delete", &id])
        .output()
        .expect("delete leased K3s session");
    assert!(
        deleted.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&deleted.stdout),
        String::from_utf8_lossy(&deleted.stderr)
    );
    assert!(
        !state.exists(),
        "explicit delete remains the only route that removes the leased machine"
    );
}

#[test]
fn leased_session_port_forward_json_emits_one_bounded_agent_document_after_cleanup() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");
    let id = create_fake_leased_session(&bin, &vat_home, &state, root.path());

    let mut forward = Command::new(vat_bin());
    let forwarded = configure_fake_k8s_command(&mut forward, &bin, &vat_home, &state, root.path())
        .env("VAT_FAKE_FORWARD_JSON_CHILD_EXIT", "42")
        .args([
            "k8s",
            "session",
            "port-forward",
            "run",
            "--format",
            "json",
            &id,
            "service/api",
            "8080",
            "--",
            "forward-json-child",
        ])
        .output()
        .expect("run leased K3s Service port-forward JSON");
    assert_eq!(
        forwarded.status.code(),
        Some(42),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&forwarded.stdout),
        String::from_utf8_lossy(&forwarded.stderr)
    );
    let stdout = String::from_utf8_lossy(&forwarded.stdout);
    assert_eq!(
        stdout.lines().count(),
        1,
        "JSON mode must not replay raw child stdout around its one document: {stdout}"
    );
    let terminal: Value = serde_json::from_str(stdout.trim()).expect("single port-forward JSON");
    assert_eq!(terminal["schema"], "vat.k8s.session.port-forward.v1");
    assert_eq!(terminal["format"], "vat_json");
    assert_eq!(terminal["type"], "vat_k8s_session_port_forward");
    assert_eq!(terminal["id"], id);
    assert_eq!(terminal["state"], "active");
    assert_eq!(terminal["resource"], "service/api");
    assert_eq!(terminal["namespace"], "default");
    assert_eq!(terminal["local_host"], "127.0.0.1");
    assert_eq!(terminal["local_port"], 38123);
    assert_eq!(terminal["remote_port"], 8080);
    assert_eq!(terminal["child_exit_code"], 42);
    assert_eq!(terminal["stdout"], "forward-json-stdout");
    assert_eq!(terminal["stderr"], "forward-json-stderr");
    assert_eq!(terminal["stdout_truncated"], false);
    assert_eq!(terminal["stderr_truncated"], false);
    assert_eq!(terminal["stdout_utf8_lossy"], false);
    assert_eq!(terminal["stderr_utf8_lossy"], false);
    assert_eq!(terminal["api_verified"], true);
    assert_eq!(terminal["cleanup"], "confirmed");
    assert_eq!(terminal["cleanup_confirmed"], true);
    assert_eq!(terminal["port_forward"], "none");
    assert_eq!(
        terminal["next"],
        format!("vat k8s session status --verify-api {id}")
    );
    assert!(
        forwarded.stderr.is_empty(),
        "JSON mode must capture host stderr instead of replaying it: {}",
        String::from_utf8_lossy(&forwarded.stderr)
    );

    let child = fs::read_to_string(root.path().join("forward-json-child.log"))
        .expect("read JSON forward child log");
    let fields: Vec<_> = child.trim().split('|').collect();
    assert_eq!(fields.len(), 9);
    assert!(
        fields[..5].iter().all(|value| value.is_empty()),
        "the JSON host child must remain credential-free: {child}"
    );
    assert_eq!(fields[5], "127.0.0.1");
    assert_eq!(fields[6], "38123");
    assert_eq!(fields[7], "service/api");
    assert_eq!(fields[8], "default");

    let kubectl = fs::read_to_string(root.path().join("kubectl.log")).expect("read kubectl log");
    assert!(kubectl.contains("forward-stop"));
    let session_directory = vat_home.join("k8s-sessions").join(&id);
    assert!(
        !session_directory.join("port-forward.json").exists(),
        "success JSON is forbidden before recovery marker removal"
    );
    assert!(
        !session_directory.join("port-forward").exists(),
        "success JSON is forbidden before private tunnel storage removal"
    );
    delete_fake_leased_session(&bin, &vat_home, &state, root.path(), &id);
}

#[test]
fn leased_session_port_forward_json_masks_private_setup_and_api_failures_without_child_start() {
    for mode in ["credentials", "api"] {
        let root = TempDir::new().expect("temp root");
        let bin = root.path().join("bin");
        write_fake_runtime(&bin);
        let vat_home = root.path().join("vat-home");
        let state = root.path().join("machine-live");
        let id = create_fake_leased_session(&bin, &vat_home, &state, root.path());
        let session_directory = vat_home.join("k8s-sessions").join(&id);
        let kubeconfig = session_directory.join("credentials/kubeconfig");
        let cache = session_directory.join("credentials/kubectl-cache");
        let home = session_directory.join("credentials/home");
        if mode == "credentials" {
            fs::remove_file(&kubeconfig).expect("remove private kubeconfig for failure proof");
        }

        let mut forward = Command::new(vat_bin());
        let forward = configure_fake_k8s_command(&mut forward, &bin, &vat_home, &state, root.path());
        if mode == "api" {
            forward.env("VAT_FAKE_KUBECTL_MODE", "fail");
        }
        let failed = forward
            .args([
                "k8s",
                "session",
                "port-forward",
                "run",
                "--format",
                "json",
                &id,
                "service/api",
                "8080",
                "--",
                "forward-json-child",
            ])
            .output()
            .expect("run failed leased K3s port-forward JSON");
        assert!(
            !failed.status.success(),
            "{mode} must fail closed: stdout={} stderr={}",
            String::from_utf8_lossy(&failed.stdout),
            String::from_utf8_lossy(&failed.stderr)
        );
        assert!(
            failed.stdout.is_empty(),
            "{mode} must not emit a partial/success JSON document: {}",
            String::from_utf8_lossy(&failed.stdout)
        );
        let output = format!(
            "{}{}",
            String::from_utf8_lossy(&failed.stdout),
            String::from_utf8_lossy(&failed.stderr)
        );
        for private_path in [&session_directory, &kubeconfig, &cache, &home] {
            assert!(
                !output.contains(&private_path.display().to_string()),
                "{mode} JSON setup error must mask private VAT path {}: {output}",
                private_path.display()
            );
        }
        assert!(
            !root.path().join("forward-json-child.log").exists(),
            "{mode} must fail before a credential-free host child can start"
        );
        assert!(
            !session_directory.join("port-forward.json").exists(),
            "{mode} must not leave a port-forward recovery marker before child start"
        );
        delete_fake_leased_session(&bin, &vat_home, &state, root.path(), &id);
    }
}

#[test]
fn leased_session_port_forward_json_expired_lease_emits_no_helper_stdout() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");
    let id = create_fake_leased_session(&bin, &vat_home, &state, root.path());
    let session_directory = vat_home.join("k8s-sessions").join(&id);
    let marker_path = session_directory.join("session.json");
    let mut marker: Value =
        serde_json::from_slice(&fs::read(&marker_path).expect("read session marker"))
            .expect("parse session marker");
    let created = marker["created_unix_ms"]
        .as_u64()
        .expect("session marker creation time");
    marker["expires_unix_ms"] = Value::from(created + 1);
    write_private_marker(&marker_path, &marker);
    let kubectl_log = root.path().join("kubectl.log");
    let _ = fs::remove_file(&kubectl_log);

    let mut forward = Command::new(vat_bin());
    let expired = configure_fake_k8s_command(&mut forward, &bin, &vat_home, &state, root.path())
        .args([
            "k8s",
            "session",
            "port-forward",
            "run",
            "--format",
            "json",
            &id,
            "service/api",
            "8080",
            "--",
            "forward-json-child",
        ])
        .output()
        .expect("attempt expired JSON port-forward");
    assert!(!expired.status.success());
    assert!(
        expired.stdout.is_empty(),
        "JSON expiry must not emit a helper record before cleanup: {}",
        String::from_utf8_lossy(&expired.stdout)
    );
    assert!(
        !session_directory.join("port-forward.json").exists(),
        "expired JSON invocation must not create a tunnel marker"
    );
    assert!(
        !root.path().join("forward-json-child.log").exists(),
        "expired JSON invocation must not start a host child"
    );
    assert!(
        !kubectl_log.exists(),
        "expired JSON invocation must fail before any kubectl invocation"
    );
    delete_fake_leased_session(&bin, &vat_home, &state, root.path(), &id);
}

#[test]
fn leased_session_port_forward_json_rechecks_lease_after_api_verify_before_tunnel_spawn() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");
    let mut create = Command::new(vat_bin());
    let created = configure_fake_k8s_command(&mut create, &bin, &vat_home, &state, root.path())
        .args([
            "k8s",
            "session",
            "create",
            "--image",
            "fixture/systemd:k3s",
            "--ttl",
            "60s",
        ])
        .output()
        .expect("create short leased session");
    assert!(
        created.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&created.stdout),
        String::from_utf8_lossy(&created.stderr)
    );
    let id = serde_json::from_slice::<Value>(&created.stdout)
        .expect("short session JSON")["id"]
        .as_str()
        .expect("short leased session id")
        .to_string();
    let session_directory = vat_home.join("k8s-sessions").join(&id);
    let marker_path = session_directory.join("session.json");
    let mut marker: Value =
        serde_json::from_slice(&fs::read(&marker_path).expect("read short session marker"))
            .expect("parse short session marker");
    let short_expiry = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock after Unix epoch")
        .as_millis() as u64
        + 2_000;
    marker["expires_unix_ms"] = Value::from(short_expiry);
    write_private_marker(&marker_path, &marker);
    let kubectl_log = root.path().join("kubectl.log");
    let _ = fs::remove_file(&kubectl_log);

    let mut forward = Command::new(vat_bin());
    let expired = configure_fake_k8s_command(&mut forward, &bin, &vat_home, &state, root.path())
        .env("VAT_FAKE_KUBECTL_DELAY_SECONDS", "3")
        .args([
            "k8s",
            "session",
            "port-forward",
            "run",
            "--format",
            "json",
            &id,
            "service/api",
            "8080",
            "--",
            "forward-json-child",
        ])
        .output()
        .expect("attempt JSON port-forward that crosses the lease during API verify");
    assert!(
        !expired.status.success(),
        "a delayed API verify must fail before tunnel spawn"
    );
    assert!(
        expired.stdout.is_empty(),
        "post-API expiry must not emit a helper/terminal JSON document: {}",
        String::from_utf8_lossy(&expired.stdout)
    );
    let kubectl = fs::read_to_string(&kubectl_log).expect("read delayed API kubectl log");
    assert!(
        !kubectl.contains("forward-start"),
        "expired lease must not launch a tunnel after API verification: {kubectl}"
    );
    assert!(
        !root.path().join("forward-json-child.log").exists(),
        "post-API expiry must not start a host child"
    );
    assert!(
        !session_directory.join("port-forward.json").exists(),
        "post-API expiry must not retain a tunnel marker"
    );
    delete_fake_leased_session(&bin, &vat_home, &state, root.path(), &id);
}

#[test]
fn leased_session_port_forward_json_emits_no_document_when_cleanup_is_unconfirmed() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");
    let id = create_fake_leased_session(&bin, &vat_home, &state, root.path());
    let session_directory = vat_home.join("k8s-sessions").join(&id);
    let marker_path = session_directory.join("port-forward.json");
    let release_path = root.path().join("release-forward-child");
    let forward_stdout = root.path().join("cleanup-failure.stdout");
    let forward_stderr = root.path().join("cleanup-failure.stderr");

    let mut forward_command = Command::new(vat_bin());
    let mut forward =
        configure_fake_k8s_command(&mut forward_command, &bin, &vat_home, &state, root.path())
            .env("VAT_FAKE_FORWARD_RELEASE", &release_path)
            .args([
                "k8s",
                "session",
                "port-forward",
                "run",
                "--format",
                "json",
                &id,
                "service/api",
                "8080",
                "--",
                "forward-wait-child",
            ])
            .stdout(Stdio::from(
                File::create(&forward_stdout).expect("create cleanup-failure stdout file"),
            ))
            .stderr(Stdio::from(
                File::create(&forward_stderr).expect("create cleanup-failure stderr file"),
            ))
            .spawn()
            .expect("start JSON port-forward awaiting cleanup failure trigger");
    wait_for_host_started_marker(&marker_path);
    let residual = session_directory.join("port-forward/unexpected-residual");
    create_private_directory(&residual);
    fs::write(&release_path, b"release").expect("release JSON host child");
    let status = forward.wait().expect("reap cleanup-failed JSON port-forward");
    let stdout = fs::read_to_string(&forward_stdout).expect("read cleanup-failure stdout");
    let stderr = fs::read_to_string(&forward_stderr).expect("read cleanup-failure stderr");
    assert!(
        !status.success(),
        "cleanup failure must fail closed: stdout={stdout} stderr={stderr}"
    );
    assert!(
        stdout.trim().is_empty(),
        "VAT must not emit success JSON while cleanup is unconfirmed: {stdout}"
    );
    for private_path in [&session_directory, &marker_path, &residual] {
        assert!(
            !stderr.contains(&private_path.display().to_string()),
            "JSON cleanup error must mask private path {}: {stderr}",
            private_path.display()
        );
    }
    assert!(
        marker_path.exists(),
        "unconfirmed cleanup must retain the recovery marker instead of reporting success"
    );
    assert!(
        session_directory.join("port-forward").exists(),
        "unconfirmed cleanup must retain recoverable private tunnel storage"
    );

    fs::remove_dir(&residual).expect("remove test-only cleanup obstruction");
    delete_fake_leased_session(&bin, &vat_home, &state, root.path(), &id);
}

#[test]
fn leased_port_forward_json_cleans_background_pipe_descendants_before_joining_capture() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");
    let id = create_fake_leased_session(&bin, &vat_home, &state, root.path());
    let descendant_pid_path = root.path().join("forward-descendant.pid");
    let descendant_ready_path = root.path().join("forward-descendant.ready");

    let started = Instant::now();
    let mut forward = Command::new(vat_bin());
    let forwarded = configure_fake_k8s_command(&mut forward, &bin, &vat_home, &state, root.path())
        .env("VAT_FAKE_FORWARD_DESCENDANT_PID", &descendant_pid_path)
        .env("VAT_FAKE_FORWARD_DESCENDANT_READY", &descendant_ready_path)
        .args([
            "k8s",
            "session",
            "port-forward",
            "run",
            "--format",
            "json",
            &id,
            "service/api",
            "8080",
            "--",
            "forward-background-child",
        ])
        .output()
        .expect("run JSON port-forward with a background pipe descendant");
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "JSON capture must clean the inherited-pipe descendant instead of blocking before cleanup"
    );
    assert!(
        forwarded.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&forwarded.stdout),
        String::from_utf8_lossy(&forwarded.stderr)
    );
    let stdout = String::from_utf8_lossy(&forwarded.stdout);
    assert_eq!(stdout.lines().count(), 1, "JSON output must stay one document");
    let terminal: Value = serde_json::from_str(stdout.trim()).expect("background JSON result");
    assert_eq!(terminal["cleanup"], "confirmed");
    assert_eq!(terminal["stdout"], "");
    assert_eq!(terminal["stderr"], "");
    wait_for_path(
        &descendant_ready_path,
        "background JSON host descendant readiness",
    );
    let descendant_pid = fs::read_to_string(&descendant_pid_path)
        .expect("read JSON background descendant pid")
        .trim()
        .parse::<u32>()
        .expect("parse JSON background descendant pid");
    let mut descendant_cleanup = ProcessIdCleanup::new(descendant_pid);
    wait_for_process_exit(descendant_pid, "JSON background pipe descendant");
    descendant_cleanup.disarm();

    let session_directory = vat_home.join("k8s-sessions").join(&id);
    assert!(!session_directory.join("port-forward.json").exists());
    assert!(!session_directory.join("port-forward").exists());
    delete_fake_leased_session(&bin, &vat_home, &state, root.path(), &id);
}

#[test]
fn leased_port_forward_json_capture_setup_failure_reaps_host_before_group_cleanup() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");
    let id = create_fake_leased_session(&bin, &vat_home, &state, root.path());
    let descendant_pid_path = root.path().join("capture-failure-descendant.pid");
    let descendant_ready_path = root.path().join("capture-failure-descendant.ready");

    let started = Instant::now();
    let mut forward = Command::new(vat_bin());
    let failed = configure_fake_k8s_command(&mut forward, &bin, &vat_home, &state, root.path())
        .env("VAT_FAKE_FORWARD_DESCENDANT_PID", &descendant_pid_path)
        .env("VAT_FAKE_FORWARD_DESCENDANT_READY", &descendant_ready_path)
        .env("VAT_TEST_FAIL_PORT_FORWARD_JSON_READER", "stderr")
        .env(
            "VAT_TEST_FAIL_PORT_FORWARD_JSON_READER_READY",
            &descendant_ready_path,
        )
        .args([
            "k8s",
            "session",
            "port-forward",
            "run",
            "--format",
            "json",
            &id,
            "service/api",
            "8080",
            "--",
            "forward-background-child",
        ])
        .output()
        .expect("run JSON port-forward with forced capture setup failure");
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "capture setup failure must return through group cleanup instead of joining inherited pipes"
    );
    assert!(
        !failed.status.success(),
        "capture setup failure must fail closed: stdout={} stderr={}",
        String::from_utf8_lossy(&failed.stdout),
        String::from_utf8_lossy(&failed.stderr)
    );
    assert!(
        failed.stdout.is_empty(),
        "capture setup failure must not emit a partial/success JSON document: {}",
        String::from_utf8_lossy(&failed.stdout)
    );
    wait_for_path(
        &descendant_ready_path,
        "capture-failure background descendant readiness",
    );
    let descendant_pid = fs::read_to_string(&descendant_pid_path)
        .expect("read capture-failure background descendant pid")
        .trim()
        .parse::<u32>()
        .expect("parse capture-failure background descendant pid");
    let mut descendant_cleanup = ProcessIdCleanup::new(descendant_pid);
    wait_for_process_exit(
        descendant_pid,
        "capture-failure background pipe descendant after group cleanup",
    );
    descendant_cleanup.disarm();

    let session_directory = vat_home.join("k8s-sessions").join(&id);
    assert!(
        !session_directory.join("port-forward.json").exists(),
        "capture failure must still confirm marker cleanup after direct host reap"
    );
    assert!(
        !session_directory.join("port-forward").exists(),
        "capture failure must still confirm private tunnel storage cleanup"
    );
    let output = String::from_utf8_lossy(&failed.stderr);
    assert!(
        !output.contains(&session_directory.display().to_string()),
        "capture failure must mask private session paths: {output}"
    );
    delete_fake_leased_session(&bin, &vat_home, &state, root.path(), &id);
}

#[test]
fn leased_session_port_forward_rejects_non_json_format_before_runtime() {
    let rejected = Command::new(vat_bin())
        .args([
            "k8s",
            "session",
            "port-forward",
            "run",
            "--format",
            "table",
            "lease-id",
            "service/api",
            "8080",
            "--",
            "forward-json-child",
        ])
        .output()
        .expect("parse invalid port-forward JSON format");
    assert!(!rejected.status.success());
    let output = format!(
        "{}{}",
        String::from_utf8_lossy(&rejected.stdout),
        String::from_utf8_lossy(&rejected.stderr)
    );
    assert!(output.contains("invalid value 'table'"), "{output}");
    assert!(
        !output.contains("Apple Container") && !output.contains("kubectl"),
        "format validation must reject before the runtime surface: {output}"
    );
}

#[test]
fn session_delete_reconciles_a_stale_verified_port_forward_marker() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");

    let mut create = Command::new(vat_bin());
    let created = configure_fake_k8s_command(&mut create, &bin, &vat_home, &state, root.path())
        .args([
            "k8s",
            "session",
            "create",
            "--image",
            "fixture/systemd:k3s",
            "--ttl",
            "10m",
        ])
        .output()
        .expect("create leased K3s session");
    assert!(created.status.success());
    let created: Value = serde_json::from_slice(&created.stdout).expect("create JSON");
    let id = created["id"]
        .as_str()
        .expect("leased session id")
        .to_string();

    let session_directory = vat_home.join("k8s-sessions").join(&id);
    let forward_directory = session_directory.join("port-forward");
    let token = "vat-pf-0123456789abcdef0123456789abcdef";
    let token_directory = forward_directory.join(token);
    let cache_directory = token_directory.join("cache");
    fs::create_dir(&forward_directory).expect("create forward directory");
    fs::create_dir(&token_directory).expect("create token directory");
    fs::create_dir(&cache_directory).expect("create cache directory");
    for path in [&forward_directory, &token_directory, &cache_directory] {
        let mut permissions = fs::metadata(path)
            .expect("directory metadata")
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(path, permissions).expect("restrict private directory");
    }
    let marker = serde_json::json!({
        "schema": "vat.k8s.session.port-forward.v2",
        "session_id": id,
        "owner_pid": 999_999_999_u32,
        "token": token,
        "state": "running",
        "resource": "service/api",
        "namespace": "default",
        "remote_port": 8080,
        "requested_local_port": 0,
        "local_port": 38123,
        "kubectl": fs::canonicalize(bin.join("kubectl"))
            .expect("canonical fake kubectl")
            .to_string_lossy(),
        "cache_dir": cache_directory.to_string_lossy(),
        "kubectl_pid": 999_999_998_u32,
        "pgid": 999_999_998_u32,
    });
    let marker_path = session_directory.join("port-forward.json");
    fs::write(
        &marker_path,
        serde_json::to_vec(&marker).expect("serialize stale marker"),
    )
    .expect("write stale marker");
    let mut marker_permissions = fs::metadata(&marker_path)
        .expect("marker metadata")
        .permissions();
    marker_permissions.set_mode(0o600);
    fs::set_permissions(&marker_path, marker_permissions).expect("restrict stale marker");

    let mut delete = Command::new(vat_bin());
    let deleted = configure_fake_k8s_command(&mut delete, &bin, &vat_home, &state, root.path())
        .args(["k8s", "session", "delete", &id])
        .output()
        .expect("delete session with stale marker");
    assert!(
        deleted.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&deleted.stdout),
        String::from_utf8_lossy(&deleted.stderr)
    );
    assert!(
        !state.exists(),
        "session delete must still remove its exact owned machine after stale-forward reconciliation"
    );
    assert!(
        !session_directory.exists(),
        "session delete removes stale marker, private cache, and credentials"
    );
}

#[test]
fn session_delete_removes_owner_dead_v1_stale_marker_only_after_absent_group_check() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");
    let id = create_fake_leased_session(&bin, &vat_home, &state, root.path());

    let session_directory = vat_home.join("k8s-sessions").join(&id);
    let forward_directory = session_directory.join("port-forward");
    // This is the pre-CSPRNG v1 shape. It is deliberately not accepted as a
    // live process identity: a dead owner and an absent recorded group are the
    // only conditions that permit storage-only legacy cleanup.
    let token = "vat-pf-4242-1710000000000";
    let token_directory = forward_directory.join(token);
    let cache_directory = token_directory.join("cache");
    create_private_directory(&forward_directory);
    create_private_directory(&token_directory);
    create_private_directory(&cache_directory);
    let marker = serde_json::json!({
        "schema": "vat.k8s.session.port-forward.v1",
        "session_id": id,
        "owner_pid": 999_999_999_u32,
        "token": token,
        "state": "running",
        "resource": "service/api",
        "namespace": "default",
        "remote_port": 8080,
        "requested_local_port": 0,
        "local_port": 38123,
        "kubectl": fs::canonicalize(bin.join("kubectl"))
            .expect("canonical fake kubectl")
            .to_string_lossy(),
        "cache_dir": cache_directory.to_string_lossy(),
        "kubectl_pid": 999_999_998_u32,
        "pgid": 999_999_998_u32,
    });
    let marker_path = session_directory.join("port-forward.json");
    write_private_marker(&marker_path, &marker);

    assert!(
        !process_group_exists(999_999_998),
        "fixture only permits legacy cleanup when the recorded group is absent"
    );
    let mut delete = Command::new(vat_bin());
    let deleted = configure_fake_k8s_command(&mut delete, &bin, &vat_home, &state, root.path())
        .args(["k8s", "session", "delete", &id])
        .output()
        .expect("delete session with stale v1 marker");
    assert!(
        deleted.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&deleted.stdout),
        String::from_utf8_lossy(&deleted.stderr)
    );
    assert!(
        !state.exists(),
        "storage-only legacy reconciliation must still remove the exact owned machine"
    );
    assert!(
        !session_directory.exists(),
        "legacy stale cleanup removes the marker, private cache, credentials, and retained lock"
    );
}

#[test]
fn leased_port_forward_kills_term_ignoring_background_host_descendant_before_success() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");
    let id = create_fake_leased_session(&bin, &vat_home, &state, root.path());
    let descendant_pid_path = root.path().join("forward-descendant.pid");
    let descendant_ready_path = root.path().join("forward-descendant.ready");
    let forward_stdout = root.path().join("forward-background.stdout");
    let forward_stderr = root.path().join("forward-background.stderr");

    let mut forward = Command::new(vat_bin());
    let status = configure_fake_k8s_command(&mut forward, &bin, &vat_home, &state, root.path())
        .env("VAT_FAKE_FORWARD_DESCENDANT_PID", &descendant_pid_path)
        .env("VAT_FAKE_FORWARD_DESCENDANT_READY", &descendant_ready_path)
        .args([
            "k8s",
            "session",
            "port-forward",
            "run",
            &id,
            "service/api",
            "8080",
            "--",
            "forward-background-child",
        ])
        // The background descendant intentionally inherits these descriptors;
        // file stdio lets VAT exit without waiting for an inherited test pipe.
        .stdout(Stdio::from(
            File::create(&forward_stdout).expect("create port-forward stdout file"),
        ))
        .stderr(Stdio::from(
            File::create(&forward_stderr).expect("create port-forward stderr file"),
        ))
        .status()
        .expect("run port-forward with a background host descendant");
    let stdout = fs::read_to_string(&forward_stdout).expect("read port-forward stdout");
    let stderr = fs::read_to_string(&forward_stderr).expect("read port-forward stderr");
    assert!(status.success(), "stdout:\n{stdout}\nstderr:\n{stderr}");
    wait_for_path(
        &descendant_ready_path,
        "background host descendant readiness",
    );
    let descendant_pid = fs::read_to_string(&descendant_pid_path)
        .expect("read background descendant pid")
        .trim()
        .parse::<u32>()
        .expect("parse background descendant pid");
    let mut descendant_cleanup = ProcessIdCleanup::new(descendant_pid);
    let terminal: Value = stdout
        .lines()
        .rev()
        .find_map(|line| serde_json::from_str(line).ok())
        .expect("port-forward terminal JSON");
    assert_eq!(terminal["cleanup"], "confirmed");
    wait_for_process_exit(descendant_pid, "TERM-ignoring background host descendant");
    descendant_cleanup.disarm();

    let session_directory = vat_home.join("k8s-sessions").join(&id);
    assert!(
        !session_directory.join("port-forward.json").exists(),
        "VAT must not report terminal success while the recovery marker remains"
    );
    assert!(
        !session_directory.join("port-forward").exists(),
        "VAT must not report terminal success while private port-forward storage remains"
    );

    let mut delete = Command::new(vat_bin());
    let deleted = configure_fake_k8s_command(&mut delete, &bin, &vat_home, &state, root.path())
        .args(["k8s", "session", "delete", &id])
        .output()
        .expect("delete leased session after descendant cleanup");
    assert!(
        deleted.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&deleted.stdout),
        String::from_utf8_lossy(&deleted.stderr)
    );
}

#[test]
fn session_delete_recovers_sigkilled_owner_through_exec_wrapper_kubectl_and_releases_lock() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");
    let id = create_fake_leased_session(&bin, &vat_home, &state, root.path());

    let real_kubectl = bin.join("kubectl-real");
    fs::rename(bin.join("kubectl"), &real_kubectl).expect("preserve real fake kubectl");
    make_executable(
        &bin.join("kubectl"),
        r#"#!/bin/sh
printf 'wrapper-exec-port-forward\n' >> "$VAT_FAKE_KUBECTL_LOG"
exec "$VAT_FAKE_KUBECTL_REAL" "$@"
"#,
    );

    let session_directory = vat_home.join("k8s-sessions").join(&id);
    let marker_path = session_directory.join("port-forward.json");
    let blocker_pid_path = root.path().join("forward-blocker.pid");
    let blocker_ready_path = root.path().join("forward-blocker.ready");
    let forward_stdout = root.path().join("forward-blocker.stdout");
    let forward_stderr = root.path().join("forward-blocker.stderr");
    let mut forward_command = Command::new(vat_bin());
    let mut forward =
        configure_fake_k8s_command(&mut forward_command, &bin, &vat_home, &state, root.path())
            .env("VAT_FAKE_KUBECTL_REAL", &real_kubectl)
            .env("VAT_FAKE_FORWARD_BLOCKER_PID", &blocker_pid_path)
            .env("VAT_FAKE_FORWARD_BLOCKER_READY", &blocker_ready_path)
            .args([
                "k8s",
                "session",
                "port-forward",
                "run",
                &id,
                "service/api",
                "8080",
                "--",
                "forward-blocker",
            ])
            // The blocked host child must not keep a test pipe open after the VAT
            // parent is SIGKILLed, otherwise the recovery test itself can hang.
            .stdout(Stdio::from(
                File::create(&forward_stdout).expect("create blocker stdout file"),
            ))
            .stderr(Stdio::from(
                File::create(&forward_stderr).expect("create blocker stderr file"),
            ))
            .spawn()
            .expect("start blocked port-forward VAT parent");

    wait_for_path(&blocker_ready_path, "blocked host child readiness");
    let marker = wait_for_host_started_marker(&marker_path);
    let kubectl_pid = marker["kubectl_pid"]
        .as_u64()
        .expect("recorded kubectl pid") as u32;
    let pgid = marker["pgid"].as_u64().expect("recorded process group") as u32;
    let wrapper_kubectl = fs::canonicalize(bin.join("kubectl"))
        .expect("canonical wrapper kubectl")
        .to_string_lossy()
        .into_owned();
    assert_eq!(kubectl_pid, pgid, "kubectl must lead its owned group");
    assert_eq!(
        marker["kubectl"].as_str(),
        Some(wrapper_kubectl.as_str()),
        "marker records the wrapper even though it execs the real kubectl process"
    );
    assert!(
        session_directory.join("operation.lock").exists(),
        "port-forward owns the retained lock before the parent crash"
    );
    let blocker_pid = fs::read_to_string(&blocker_pid_path)
        .expect("read blocker pid")
        .trim()
        .parse::<u32>()
        .expect("parse blocker pid");
    let mut group_cleanup = ProcessGroupCleanup::new(pgid);

    forward.kill().expect("SIGKILL VAT port-forward parent");
    let parent_status = forward.wait().expect("reap SIGKILLed VAT parent");
    assert_eq!(
        parent_status.signal(),
        Some(libc::SIGKILL),
        "fixture must exercise abrupt parent death, not normal VAT cleanup"
    );
    assert!(
        process_exists(kubectl_pid) && process_exists(blocker_pid),
        "the recovery path must observe the still-live exec-wrapper group after parent death"
    );

    let mut delete = Command::new(vat_bin());
    let deleted = configure_fake_k8s_command(&mut delete, &bin, &vat_home, &state, root.path())
        .env("VAT_FAKE_KUBECTL_REAL", &real_kubectl)
        .args(["k8s", "session", "delete", &id])
        .output()
        .expect("recover SIGKILLed port-forward during session delete");
    let delete_stdout = String::from_utf8_lossy(&deleted.stdout);
    let delete_stderr = String::from_utf8_lossy(&deleted.stderr);
    assert!(
        deleted.status.success(),
        "stdout:\n{delete_stdout}\nstderr:\n{delete_stderr}"
    );
    wait_for_process_exit(kubectl_pid, "exec-wrapper kubectl");
    wait_for_process_exit(blocker_pid, "blocked host child");
    wait_for_process_group_exit(pgid, "recovered kubectl/host-child");
    group_cleanup.disarm();

    let kubectl_log =
        fs::read_to_string(root.path().join("kubectl.log")).expect("read wrapper kubectl log");
    assert!(
        kubectl_log.contains("wrapper-exec-port-forward"),
        "fixture must have exercised the exec wrapper: {kubectl_log}"
    );
    assert!(
        kubectl_log.contains("forward-start:service/api|38123|8080"),
        "recovery must target the authenticated Service tunnel: {kubectl_log}"
    );
    assert!(
        !state.exists(),
        "successful recovery delete removes the exact owned machine"
    );
    assert!(
        !session_directory.exists(),
        "a recovered delete must remove marker, storage, credentials, and retained operation lock"
    );
}

#[test]
fn session_delete_fails_closed_for_host_started_marker_with_unverified_live_group() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");
    let id = create_fake_leased_session(&bin, &vat_home, &state, root.path());

    let mut unrelated_command = Command::new("/bin/sh");
    unrelated_command
        .args(["-c", "trap '' TERM INT; while :; do sleep 1; done"])
        .process_group(0)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut unrelated = unrelated_command
        .spawn()
        .expect("start unrelated process-group leader");
    let unrelated_pgid = unrelated.id();
    let mut unrelated_cleanup = ProcessGroupCleanup::new(unrelated_pgid);
    assert!(
        process_group_exists(unrelated_pgid),
        "unrelated fixture group must be live before the recovery attempt"
    );

    let session_directory = vat_home.join("k8s-sessions").join(&id);
    let forward_directory = session_directory.join("port-forward");
    let token = "vat-pf-0123456789abcdef0123456789abcdef";
    let token_directory = forward_directory.join(token);
    let cache_directory = token_directory.join("cache");
    create_private_directory(&forward_directory);
    create_private_directory(&token_directory);
    create_private_directory(&cache_directory);
    let marker = serde_json::json!({
        "schema": "vat.k8s.session.port-forward.v2",
        "session_id": id,
        "owner_pid": 999_999_999_u32,
        "token": token,
        "state": "running",
        "resource": "service/api",
        "namespace": "default",
        "remote_port": 8080,
        "requested_local_port": 0,
        "local_port": 38123,
        "kubectl": fs::canonicalize(bin.join("kubectl"))
            .expect("canonical fake kubectl")
            .to_string_lossy(),
        "cache_dir": cache_directory.to_string_lossy(),
        "kubectl_pid": unrelated_pgid,
        "pgid": unrelated_pgid,
        "host_started": true,
    });
    let marker_path = session_directory.join("port-forward.json");
    write_private_marker(&marker_path, &marker);

    let mut delete = Command::new(vat_bin());
    let deleted = configure_fake_k8s_command(&mut delete, &bin, &vat_home, &state, root.path())
        .args(["k8s", "session", "delete", &id])
        .output()
        .expect("attempt delete with unverified live recovery group");
    let diagnostic = format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&deleted.stdout),
        String::from_utf8_lossy(&deleted.stderr)
    );
    assert!(
        !deleted.status.success(),
        "VAT must fail closed rather than signal an unverified process group: {diagnostic}"
    );
    assert!(
        diagnostic.contains("can no longer authenticate")
            || diagnostic.contains("cannot authenticate")
            || diagnostic.contains("recovery marker is retained"),
        "failure must explain the retained unverified recovery state: {diagnostic}"
    );
    assert!(
        process_exists(unrelated_pgid) && process_group_exists(unrelated_pgid),
        "VAT must not signal the unrelated process group"
    );
    assert!(
        state.exists(),
        "VAT must not delete the leased machine when recovery cannot authenticate its group"
    );
    assert!(
        marker_path.exists() && cache_directory.exists() && session_directory.exists(),
        "VAT must retain marker, cache, credentials, and session for explicit manual recovery"
    );

    unsafe {
        libc::kill(-(unrelated_pgid as libc::pid_t), libc::SIGKILL);
    }
    let _ = unrelated
        .wait()
        .expect("reap unrelated process-group leader");
    wait_for_process_group_exit(unrelated_pgid, "unrelated fixture");
    unrelated_cleanup.disarm();
}

#[test]
fn leased_session_imports_only_a_verified_local_image_then_removes_staging_archives() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");

    let mut create = Command::new(vat_bin());
    let created = configure_fake_k8s_command(&mut create, &bin, &vat_home, &state, root.path())
        .args([
            "k8s",
            "session",
            "create",
            "--image",
            "fixture/systemd:k3s",
            "--ttl",
            "10m",
        ])
        .output()
        .expect("create leased K3s session");
    assert!(
        created.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&created.stdout),
        String::from_utf8_lossy(&created.stderr)
    );
    let created: Value = serde_json::from_slice(&created.stdout).expect("create JSON");
    let id = created["id"]
        .as_str()
        .expect("leased session id")
        .to_string();

    let mut load = Command::new(vat_bin());
    let loaded = configure_fake_k8s_command(&mut load, &bin, &vat_home, &state, root.path())
        .args(["k8s", "session", "image", "load", &id, "alpine:3.20"])
        .output()
        .expect("load local image into leased K3s");
    assert!(
        loaded.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&loaded.stdout),
        String::from_utf8_lossy(&loaded.stderr)
    );
    let load_stdout = String::from_utf8_lossy(&loaded.stdout);
    let loaded: Value = serde_json::from_slice(&loaded.stdout).expect("image-load JSON");
    assert_eq!(loaded["type"], "vat_k8s_session_image_load");
    assert_eq!(loaded["id"], id);
    assert_eq!(loaded["state"], "active");
    assert_eq!(loaded["image"], "alpine:3.20");
    assert_eq!(loaded["canonical_image"], "docker.io/library/alpine:3.20");
    assert_eq!(loaded["platform"], "linux/arm64");
    assert_eq!(
        loaded["source_digest"],
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    );
    assert!(
        !load_stdout.contains("kubeconfig"),
        "result must not reveal private credential paths"
    );

    let log = fs::read_to_string(root.path().join("container.log")).expect("container log");
    let archive = log
        .lines()
        .find_map(|line| line.strip_prefix("image-save:"))
        .expect("fake image save path");
    assert!(
        !Path::new(archive).exists(),
        "private host OCI archive must be removed before success"
    );
    assert!(
        log.contains("image-copy:"),
        "image archive must be copied to guest"
    );
    assert!(
        log.contains("k3s ctr -n k8s.io images import"),
        "guest import must use K3s' k8s.io image namespace"
    );
    assert!(
        log.contains("rm -f -- /tmp/vat-k8s-image-"),
        "guest temporary OCI archive must be removed before success"
    );
    let session_dir = vat_home.join("k8s-sessions").join(&id);
    assert!(
        fs::read_dir(&session_dir)
            .expect("read active session storage")
            .all(|entry| !entry
                .expect("session entry")
                .file_name()
                .to_string_lossy()
                .starts_with("image-load-")),
        "active lease must not retain private image staging directories"
    );
    assert!(
        state.exists(),
        "image load must retain active leased machine"
    );

    let mut delete = Command::new(vat_bin());
    let deleted = configure_fake_k8s_command(&mut delete, &bin, &vat_home, &state, root.path())
        .args(["k8s", "session", "delete", &id])
        .output()
        .expect("delete leased K3s session");
    assert!(
        deleted.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&deleted.stdout),
        String::from_utf8_lossy(&deleted.stderr)
    );
    assert!(
        !state.exists(),
        "delete must remove the exact leased machine"
    );
}

#[test]
fn uncertain_machine_create_keeps_a_durable_recovery_marker() {
    let root = TempDir::new().expect("temp root");
    let bin = root.path().join("bin");
    write_fake_runtime(&bin);
    let vat_home = root.path().join("vat-home");
    let state = root.path().join("machine-live");

    let output = Command::new(vat_bin())
        .env("PATH", path_with(&bin))
        .env("VAT_HOME", &vat_home)
        .env("VAT_FAKE_K8S_STATE", &state)
        .env("VAT_FAKE_K8S_CREATE_MODE", "uncertain")
        .args([
            "k8s",
            "ephemeral",
            "run",
            "--image",
            "fixture/systemd:k3s",
            "--",
            "agent-child",
        ])
        .output()
        .expect("run VAT ephemeral K3s command with uncertain create");

    assert!(
        !output.status.success(),
        "uncertain create must fail closed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !state.exists(),
        "VAT still deletes an exact allocated machine before retaining recovery state"
    );

    let sessions = vat_home.join("k8s-ephemeral");
    let marker = fs::read_dir(&sessions)
        .expect("read retained sessions")
        .next()
        .expect("uncertain create marker")
        .expect("session entry")
        .path();
    let marker_json = fs::read_to_string(&marker).expect("read retained marker");
    assert!(
        marker_json.contains("\"create_uncertain\":true"),
        "marker must conservatively retain failed-create state: {marker_json}"
    );

    let recovery = Command::new(vat_bin())
        .env("PATH", path_with(&bin))
        .env("VAT_HOME", &vat_home)
        .env("VAT_FAKE_K8S_STATE", &state)
        .args(["k8s", "ephemeral", "cleanup", "--json"])
        .output()
        .expect("retry uncertain create recovery");
    assert!(
        !recovery.status.success(),
        "cleanup must keep an uncertain create marker instead of claiming terminal cleanup: stdout={} stderr={}",
        String::from_utf8_lossy(&recovery.stdout),
        String::from_utf8_lossy(&recovery.stderr)
    );
    assert!(
        marker.exists(),
        "uncertain create marker must persist until Apple exposes terminal create/cancellation state"
    );
}

#[test]
#[ignore = "real Apple Container K3s session; run only with VAT_K8S_EPHEMERAL_E2E_REQUIRED=1"]
fn apple_container_k3s_session_exposes_host_api_then_cleans_up() {
    if std::env::var("VAT_K8S_EPHEMERAL_E2E_REQUIRED").as_deref() != Ok("1") {
        eprintln!("VAT_K8S_EPHEMERAL_E2E_REQUIRED=1 is required; skipping real K3s session");
        return;
    }

    let root = TempDir::new().expect("temp VAT_HOME");
    let image = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "ephemeral", "image", "build"])
        .output()
        .expect("build ephemeral machine image");
    assert!(
        image.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&image.stdout),
        String::from_utf8_lossy(&image.stderr)
    );

    let output = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "ephemeral", "run", "--", "kubectl", "get", "nodes"])
        .output()
        .expect("run real ephemeral K3s session");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Ready"),
        "host command did not report a Ready K3s node"
    );
    let terminal: Value = stdout
        .lines()
        .rev()
        .find_map(|line| serde_json::from_str(line).ok())
        .expect("terminal JSON result line");
    assert_eq!(terminal["type"], "vat_k8s_ephemeral_result");
    assert_eq!(terminal["cleanup"], "confirmed");
    let machine = terminal["machine"]
        .as_str()
        .expect("owned machine name from terminal result");
    let inspect = Command::new("container")
        .args(["machine", "inspect", machine])
        .output()
        .expect("inspect exact cleaned-up machine");
    assert!(
        !inspect.status.success(),
        "terminal cleanup claimed success but exact machine still inspects: stdout={} stderr={}",
        String::from_utf8_lossy(&inspect.stdout),
        String::from_utf8_lossy(&inspect.stderr)
    );
    let inspect_diagnostic = format!(
        "{}\n{}",
        String::from_utf8_lossy(&inspect.stdout),
        String::from_utf8_lossy(&inspect.stderr)
    )
    .to_ascii_lowercase();
    assert!(
        inspect_diagnostic.contains("notfound") && inspect_diagnostic.contains(machine),
        "exact cleanup must prove Apple Container's not-found result: {inspect_diagnostic}"
    );

    let cleanup = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "ephemeral", "cleanup", "--json"])
        .output()
        .expect("verify recovery cleanup");
    assert!(cleanup.status.success());
    assert!(String::from_utf8_lossy(&cleanup.stdout).contains("\"failed\":[]"));
}

#[test]
#[ignore = "real Apple Container leased K3s session; run only with VAT_K8S_SESSION_E2E_REQUIRED=1"]
fn apple_container_k3s_leased_session_supports_multiple_host_commands_then_deletes() {
    if std::env::var("VAT_K8S_SESSION_E2E_REQUIRED").as_deref() != Ok("1") {
        eprintln!("VAT_K8S_SESSION_E2E_REQUIRED=1 is required; skipping leased K3s session");
        return;
    }

    let root = TempDir::new().expect("temp VAT_HOME");
    let image = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "ephemeral", "image", "build"])
        .output()
        .expect("build ephemeral machine image");
    assert!(
        image.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&image.stdout),
        String::from_utf8_lossy(&image.stderr)
    );

    let created = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "session", "create", "--ttl", "30m"])
        .output()
        .expect("create real leased K3s session");
    assert!(
        created.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&created.stdout),
        String::from_utf8_lossy(&created.stderr)
    );
    let created: Value = serde_json::from_slice(&created.stdout).expect("leased session JSON");
    assert_eq!(created["type"], "vat_k8s_session");
    assert_eq!(created["state"], "active");
    let id = created["id"]
        .as_str()
        .expect("leased session id")
        .to_string();
    let machine = created["machine"]
        .as_str()
        .expect("leased machine name")
        .to_string();
    let mut cleanup = RealLeasedSessionCleanup::new(root.path().to_path_buf(), id.clone());

    for command in [
        vec!["kubectl", "get", "nodes"],
        vec!["kubectl", "get", "namespaces"],
    ] {
        let output = Command::new(vat_bin())
            .env("VAT_HOME", root.path())
            .args(["k8s", "session", "exec", &id, "--"])
            .args(&command)
            .output()
            .expect("execute host command against real leased K3s session");
        assert!(
            output.status.success(),
            "command={command:?}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let terminal: Value = String::from_utf8_lossy(&output.stdout)
            .lines()
            .rev()
            .find_map(|line| serde_json::from_str(line).ok())
            .expect("leased session terminal JSON");
        assert_eq!(terminal["type"], "vat_k8s_session_exec");
        assert_eq!(terminal["state"], "active");
    }

    let json_exec = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args([
            "k8s",
            "session",
            "exec",
            "--format",
            "json",
            "--timeout",
            "30",
            &id,
            "--",
            "kubectl",
            "get",
            "nodes",
            "-o",
            "json",
        ])
        .output()
        .expect("execute JSON host command against real leased K3s session");
    assert!(
        json_exec.status.success(),
        "JSON exec stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&json_exec.stdout),
        String::from_utf8_lossy(&json_exec.stderr)
    );
    assert!(
        json_exec.stderr.is_empty(),
        "JSON exec must capture child stderr in its one VAT document: {}",
        String::from_utf8_lossy(&json_exec.stderr)
    );
    let json_exec_stdout = String::from_utf8_lossy(&json_exec.stdout);
    assert_eq!(
        json_exec_stdout.lines().count(),
        1,
        "JSON exec must emit exactly one VAT document: {json_exec_stdout}"
    );
    let json_exec_result: Value =
        serde_json::from_str(json_exec_stdout.trim()).expect("real leased JSON exec document");
    assert_eq!(json_exec_result["schema"], "vat.k8s.session.exec.v1");
    assert_eq!(json_exec_result["format"], "vat_json");
    assert_eq!(json_exec_result["type"], "vat_k8s_session_exec");
    assert_eq!(json_exec_result["id"], id);
    assert_eq!(json_exec_result["state"], "active");
    assert_eq!(json_exec_result["child_exit_code"], 0);
    assert_eq!(json_exec_result["api_verified"], true);
    assert_eq!(json_exec_result["runtime_invoked"], true);
    assert_eq!(json_exec_result["session_record_mutated"], false);
    assert!(
        json_exec_result["stdout"]
            .as_str()
            .is_some_and(|stdout| stdout.contains("\"items\"")),
        "JSON exec must retain the real kubectl response: {json_exec_result}"
    );
    let private_session_directory = root.path().join("k8s-sessions").join(&id);
    let private_kubeconfig = private_session_directory.join("credentials/kubeconfig");
    assert!(
        !json_exec_stdout.contains(&private_session_directory.display().to_string())
            && !json_exec_stdout.contains(&private_kubeconfig.display().to_string()),
        "JSON exec must not expose VAT private credential paths: {json_exec_stdout}"
    );

    let verified_status = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "session", "status", "--verify-api", &id])
        .output()
        .expect("verify real leased K3s API status");
    assert!(
        verified_status.status.success(),
        "verify-api stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&verified_status.stdout),
        String::from_utf8_lossy(&verified_status.stderr)
    );
    let verified_status: Value =
        serde_json::from_slice(&verified_status.stdout).expect("real verified status JSON");
    assert_eq!(verified_status["id"], id);
    assert_eq!(verified_status["state"], "active");
    assert_eq!(verified_status["api_checked"], true);
    assert_eq!(verified_status["api_state"], "reachable");

    let deleted = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "session", "delete", &id])
        .output()
        .expect("delete real leased K3s session");
    assert!(
        deleted.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&deleted.stdout),
        String::from_utf8_lossy(&deleted.stderr)
    );
    let deleted: Value = serde_json::from_slice(&deleted.stdout).expect("delete JSON");
    assert_eq!(deleted["terminal"], "cleaned_up");
    cleanup.disarm();

    let inspect = Command::new("container")
        .args(["machine", "inspect", &machine])
        .output()
        .expect("inspect exact cleaned-up leased machine");
    assert!(
        !inspect.status.success(),
        "leased session cleanup claimed success but exact machine remains: stdout={} stderr={}",
        String::from_utf8_lossy(&inspect.stdout),
        String::from_utf8_lossy(&inspect.stderr)
    );
    assert!(
        !root.path().join("k8s-sessions").join(&id).exists(),
        "leased-session credentials and marker must be gone after delete"
    );
}

#[test]
#[ignore = "real Apple Container local-image K3s contract; run only with VAT_K8S_LOCAL_IMAGE_E2E_REQUIRED=1"]
fn apple_container_k3s_lease_imports_local_image_without_registry_pull() {
    if std::env::var("VAT_K8S_LOCAL_IMAGE_E2E_REQUIRED").as_deref() != Ok("1") {
        eprintln!(
            "VAT_K8S_LOCAL_IMAGE_E2E_REQUIRED=1 is required; skipping real local-image K3s probe"
        );
        return;
    }

    let root = TempDir::new().expect("temp VAT_HOME");
    let image = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "ephemeral", "image", "build"])
        .output()
        .expect("build ephemeral machine image");
    assert!(
        image.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&image.stdout),
        String::from_utf8_lossy(&image.stderr)
    );

    let created = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "session", "create", "--ttl", "30m"])
        .output()
        .expect("create real leased K3s session");
    assert!(
        created.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&created.stdout),
        String::from_utf8_lossy(&created.stderr)
    );
    let created: Value = serde_json::from_slice(&created.stdout).expect("leased session JSON");
    let id = created["id"]
        .as_str()
        .expect("leased session id")
        .to_string();
    let machine = created["machine"]
        .as_str()
        .expect("leased machine name")
        .to_string();
    let mut cleanup = RealLeasedSessionCleanup::new(root.path().to_path_buf(), id.clone());

    // `alpine:3.20` is preloaded into the Apple Container store in this host
    // fixture. VAT never pulls it: session image load must inspect/save the
    // existing local reference and K3s must start the workload with Never.
    let loaded = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "session", "image", "load", &id, "alpine:3.20"])
        .output()
        .expect("load local Apple image into real K3s session");
    assert!(
        loaded.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&loaded.stdout),
        String::from_utf8_lossy(&loaded.stderr)
    );
    let loaded: Value = serde_json::from_slice(&loaded.stdout).expect("image-load JSON");
    assert_eq!(loaded["type"], "vat_k8s_session_image_load");
    assert_eq!(loaded["state"], "active");
    assert_eq!(loaded["platform"], "linux/arm64");

    let pod = format!("vat-local-image-{}", std::process::id());
    let run = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "session", "exec", &id, "--"])
        .args([
            "kubectl",
            "run",
            &pod,
            "--image=alpine:3.20",
            "--restart=Never",
            "--image-pull-policy=Never",
            "--command",
            "--",
            "/bin/sh",
            "-ec",
            "echo vat-local-image-load-ok",
        ])
        .output()
        .expect("run K3s pod from locally imported image");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let waited = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "session", "exec", &id, "--"])
        .args([
            "kubectl",
            "wait",
            "--for=jsonpath={.status.phase}=Succeeded",
            "--timeout=90s",
            &format!("pod/{pod}"),
        ])
        .output()
        .expect("wait for local-image pod completion");
    assert!(
        waited.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&waited.stdout),
        String::from_utf8_lossy(&waited.stderr)
    );

    let logs = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "session", "exec", &id, "--"])
        .args(["kubectl", "logs", &pod])
        .output()
        .expect("read local-image pod logs");
    assert!(
        logs.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&logs.stdout),
        String::from_utf8_lossy(&logs.stderr)
    );
    assert!(
        String::from_utf8_lossy(&logs.stdout).contains("vat-local-image-load-ok"),
        "local K3s workload did not execute its imported image"
    );

    let deleted = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "session", "delete", &id])
        .output()
        .expect("delete local-image leased K3s session");
    assert!(
        deleted.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&deleted.stdout),
        String::from_utf8_lossy(&deleted.stderr)
    );
    cleanup.disarm();

    let inspect = Command::new("container")
        .args(["machine", "inspect", &machine])
        .output()
        .expect("inspect exact cleaned-up local-image leased machine");
    assert!(
        !inspect.status.success(),
        "local-image session cleanup claimed success but exact machine remains: stdout={} stderr={}",
        String::from_utf8_lossy(&inspect.stdout),
        String::from_utf8_lossy(&inspect.stderr)
    );
    assert!(
        !root.path().join("k8s-sessions").join(&id).exists(),
        "local-image session credentials and marker must be gone after delete"
    );
}

#[test]
#[ignore = "real Apple Container K3s Service tunnel; run only with VAT_K8S_PORT_FORWARD_E2E_REQUIRED=1"]
fn apple_container_k3s_lease_port_forwards_local_service_to_one_credential_free_host_child() {
    if std::env::var("VAT_K8S_PORT_FORWARD_E2E_REQUIRED").as_deref() != Ok("1") {
        eprintln!(
            "VAT_K8S_PORT_FORWARD_E2E_REQUIRED=1 is required; skipping real K3s port-forward"
        );
        return;
    }

    let root = TempDir::new().expect("temp VAT_HOME");
    let image = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "ephemeral", "image", "build"])
        .output()
        .expect("build ephemeral machine image");
    assert!(
        image.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&image.stdout),
        String::from_utf8_lossy(&image.stderr)
    );

    let created = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "session", "create", "--ttl", "30m"])
        .output()
        .expect("create real leased K3s session");
    assert!(
        created.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&created.stdout),
        String::from_utf8_lossy(&created.stderr)
    );
    let created: Value = serde_json::from_slice(&created.stdout).expect("create JSON");
    let id = created["id"]
        .as_str()
        .expect("leased session id")
        .to_string();
    let machine = created["machine"]
        .as_str()
        .expect("leased machine name")
        .to_string();
    let mut cleanup = RealLeasedSessionCleanup::new(root.path().to_path_buf(), id.clone());

    let loaded = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "session", "image", "load", &id, "alpine:3.20"])
        .output()
        .expect("load local alpine image into leased K3s");
    assert!(
        loaded.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&loaded.stdout),
        String::from_utf8_lossy(&loaded.stderr)
    );

    let suffix = format!("{}", std::process::id());
    let pod = format!("vat-forward-http-{suffix}");
    let service = format!("vat-forward-service-{suffix}");
    let run = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "session", "exec", &id, "--"])
        .args([
            "kubectl",
            "run",
            &pod,
            "--image=alpine:3.20",
            "--restart=Never",
            "--image-pull-policy=Never",
            "--command",
            "--",
            "/bin/sh",
            "-ec",
            // The locally cached Alpine fixture has `nc` but not BusyBox's
            // optional `httpd` applet. Keep this server entirely within the
            // known local image surface, and serve one request per loop so
            // the later host-side tunnel assertion has a real HTTP response.
            "while :; do printf 'HTTP/1.1 200 OK\\r\\nContent-Length: 19\\r\\nConnection: close\\r\\n\\r\\nvat-port-forward-ok' | nc -l -p 8080; done",
        ])
        .output()
        .expect("run local HTTP pod");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let waited = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "session", "exec", &id, "--"])
        .args([
            "kubectl",
            "wait",
            "--for=condition=Ready",
            "--timeout=90s",
            &format!("pod/{pod}"),
        ])
        .output()
        .expect("wait for local HTTP pod");
    assert!(
        waited.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&waited.stdout),
        String::from_utf8_lossy(&waited.stderr)
    );

    // `Ready` without a readiness probe only proves the container process is
    // alive. Probe the actual fixture response before creating its Service so
    // a missing optional applet cannot turn into a misleading port-forward
    // data-plane failure later.
    let pod_response = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "session", "exec", &id, "--"])
        .args([
            "kubectl",
            "exec",
            &pod,
            "--",
            "wget",
            "-qO-",
            "http://127.0.0.1:8080/",
        ])
        .output()
        .expect("probe local HTTP pod response");
    assert!(
        pod_response.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&pod_response.stdout),
        String::from_utf8_lossy(&pod_response.stderr)
    );
    assert!(
        String::from_utf8_lossy(&pod_response.stdout).contains("vat-port-forward-ok"),
        "local HTTP pod did not serve the expected fixture response"
    );

    let exposed = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "session", "exec", &id, "--"])
        .args([
            "kubectl",
            "expose",
            &format!("pod/{pod}"),
            "--name",
            &service,
            "--port=8080",
            "--target-port=8080",
        ])
        .output()
        .expect("expose local HTTP Service");
    assert!(
        exposed.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&exposed.stdout),
        String::from_utf8_lossy(&exposed.stderr)
    );

    let forwarded = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args([
            "k8s",
            "session",
            "port-forward",
            "run",
            &id,
            &format!("service/{service}"),
            "8080",
            "--",
            "/bin/sh",
            "-ec",
            "test -z \"$KUBECONFIG\"; test -z \"$VAT_K8S_CACHE_DIR\"; test -z \"$VAT_K8S_API_SERVER\"; test -z \"$VAT_HOME\"; test \"$VAT_K8S_PORT_FORWARD_HOST\" = 127.0.0.1; curl -fsS \"http://$VAT_K8S_PORT_FORWARD_HOST:$VAT_K8S_PORT_FORWARD_PORT/\"",
        ])
        .output()
        .expect("port-forward Service to credential-free host child");
    assert!(
        forwarded.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&forwarded.stdout),
        String::from_utf8_lossy(&forwarded.stderr)
    );
    assert!(
        String::from_utf8_lossy(&forwarded.stdout).contains("vat-port-forward-ok"),
        "host child did not receive the local Service response"
    );
    let terminal: Value = String::from_utf8_lossy(&forwarded.stdout)
        .lines()
        .rev()
        .find_map(|line| serde_json::from_str(line).ok())
        .expect("port-forward terminal JSON");
    assert_eq!(terminal["type"], "vat_k8s_session_port_forward");
    assert_eq!(terminal["resource"], format!("service/{service}"));
    assert_eq!(terminal["cleanup"], "confirmed");
    let local_port = terminal["local_port"]
        .as_u64()
        .expect("selected local port") as u16;
    assert!(
        TcpStream::connect_timeout(
            &format!("127.0.0.1:{local_port}")
                .parse()
                .expect("loopback address"),
            Duration::from_millis(300),
        )
        .is_err(),
        "port-forward result claimed cleanup but loopback port {local_port} remains open"
    );

    let forwarded_json = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args([
            "k8s",
            "session",
            "port-forward",
            "run",
            "--format",
            "json",
            &id,
            &format!("service/{service}"),
            "8080",
            "--",
            "/bin/sh",
            "-ec",
            "test -z \"$KUBECONFIG\"; test -z \"$VAT_K8S_CACHE_DIR\"; test -z \"$VAT_K8S_API_SERVER\"; test -z \"$VAT_HOME\"; test \"$VAT_K8S_PORT_FORWARD_HOST\" = 127.0.0.1; curl -fsS \"http://$VAT_K8S_PORT_FORWARD_HOST:$VAT_K8S_PORT_FORWARD_PORT/\"",
        ])
        .output()
        .expect("JSON port-forward Service to credential-free host child");
    assert!(
        forwarded_json.status.success(),
        "JSON port-forward stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&forwarded_json.stdout),
        String::from_utf8_lossy(&forwarded_json.stderr)
    );
    assert!(
        forwarded_json.stderr.is_empty(),
        "JSON port-forward must capture host child stderr in its one VAT document: {}",
        String::from_utf8_lossy(&forwarded_json.stderr)
    );
    let forwarded_json_stdout = String::from_utf8_lossy(&forwarded_json.stdout);
    assert_eq!(
        forwarded_json_stdout.lines().count(),
        1,
        "JSON port-forward must emit exactly one VAT document: {forwarded_json_stdout}"
    );
    let forwarded_json_result: Value = serde_json::from_str(forwarded_json_stdout.trim())
        .expect("real JSON port-forward document");
    assert_eq!(
        forwarded_json_result["schema"],
        "vat.k8s.session.port-forward.v1"
    );
    assert_eq!(forwarded_json_result["format"], "vat_json");
    assert_eq!(
        forwarded_json_result["type"],
        "vat_k8s_session_port_forward"
    );
    assert_eq!(forwarded_json_result["id"], id);
    assert_eq!(
        forwarded_json_result["resource"],
        format!("service/{service}")
    );
    assert_eq!(forwarded_json_result["child_exit_code"], 0);
    assert_eq!(forwarded_json_result["api_verified"], true);
    assert_eq!(forwarded_json_result["runtime_invoked"], true);
    assert_eq!(forwarded_json_result["cleanup"], "confirmed");
    assert_eq!(forwarded_json_result["cleanup_confirmed"], true);
    assert_eq!(forwarded_json_result["port_forward"], "none");
    assert!(
        forwarded_json_result["stdout"]
            .as_str()
            .is_some_and(|stdout| stdout.contains("vat-port-forward-ok")),
        "JSON port-forward must retain the credential-free host response: {forwarded_json_result}"
    );
    assert!(
        !forwarded_json_stdout.contains("k8s-sessions")
            && !forwarded_json_stdout.contains("kubeconfig"),
        "JSON port-forward must not expose private credential paths: {forwarded_json_stdout}"
    );
    let json_local_port = forwarded_json_result["local_port"]
        .as_u64()
        .expect("JSON selected local port") as u16;
    assert!(
        TcpStream::connect_timeout(
            &format!("127.0.0.1:{json_local_port}")
                .parse()
                .expect("JSON loopback address"),
            Duration::from_millis(300),
        )
        .is_err(),
        "JSON port-forward claimed cleanup but loopback port {json_local_port} remains open"
    );

    let status = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "session", "status", &id])
        .output()
        .expect("status leased K3s session after port-forward");
    assert!(status.status.success());
    let status: Value = serde_json::from_slice(&status.stdout).expect("status JSON");
    assert_eq!(status["state"], "active");
    assert_eq!(status["port_forward"], "none");

    let deleted = Command::new(vat_bin())
        .env("VAT_HOME", root.path())
        .args(["k8s", "session", "delete", &id])
        .output()
        .expect("delete leased K3s session");
    assert!(
        deleted.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&deleted.stdout),
        String::from_utf8_lossy(&deleted.stderr)
    );
    cleanup.disarm();
    let inspect = Command::new("container")
        .args(["machine", "inspect", &machine])
        .output()
        .expect("inspect exact cleaned-up port-forward machine");
    assert!(
        !inspect.status.success(),
        "port-forward lease cleanup claimed success but exact machine remains: stdout={} stderr={}",
        String::from_utf8_lossy(&inspect.stdout),
        String::from_utf8_lossy(&inspect.stderr)
    );
}
// HANDWRITE-END
