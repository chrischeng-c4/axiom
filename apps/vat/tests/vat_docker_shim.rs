//! Regression coverage for VAT's opt-in docker-to-vat Apple Container shim.
//!
//! These tests execute the real VAT binary through a temporary docker symlink,
//! but substitute a tiny container executable. That proves raw argv dispatch,
//! fail-closed rejection, installer safety, and exit forwarding without
//! requiring an Apple Container runtime in CI.

#[cfg(unix)]
use std::ffi::CString;
use std::ffi::OsString;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;
use std::time::{Duration, Instant};

use tempfile::TempDir;

fn vat_bin() -> &'static str {
    env!("CARGO_BIN_EXE_vat")
}

fn write_fake_container(bin_dir: &Path) -> PathBuf {
    let script = bin_dir.join("container");
    fs::create_dir_all(bin_dir).expect("create fake container directory");
    fs::write(
        &script,
        r#"#!/bin/sh
if [ -n "$VAT_FAKE_CONTAINER_LOG" ]; then
  printf '%s\n' "$*" >> "$VAT_FAKE_CONTAINER_LOG"
fi
if [ -n "$VAT_FAKE_CONTAINER_EXIT" ]; then
  exit "$VAT_FAKE_CONTAINER_EXIT"
fi
exit 0
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

fn host_container_binary() -> PathBuf {
    std::env::split_paths(
        &std::env::var_os("PATH").expect("PATH is set for test process"),
    )
    .map(|directory| directory.join("container"))
    .find(|candidate| candidate.is_file())
    .expect("Apple Container CLI must be on PATH for the opted-in host E2E")
}

/// Record the exact Apple argv the real E2E uses without replacing the
/// runtime. This gives the test a high-entropy name + owner token to clean
/// only its own resource if an assertion fails after VAT has returned.
fn write_recording_container_proxy(bin_dir: &Path) -> PathBuf {
    let script = bin_dir.join("container");
    fs::create_dir_all(bin_dir).expect("create recording proxy directory");
    fs::write(
        &script,
        r#"#!/bin/sh
set -eu
printf '%s\n' "$*" >> "${VAT_REAL_CONTAINER_LOG:?VAT_REAL_CONTAINER_LOG is required}"
exec "${VAT_REAL_CONTAINER:?VAT_REAL_CONTAINER is required}" "$@"
"#,
    )
    .expect("write recording Apple Container proxy");
    let mut permissions = fs::metadata(&script)
        .expect("recording proxy metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).expect("make recording proxy executable");
    script
}

/// A strict Apple Container stats double. It accepts only VAT's canonical
/// non-streaming JSON argv, which lets the integration tests prove both that
/// rejected Docker-shaped flags never spawn and that the public stdout remains
/// Apple-native JSON rather than a VAT/Docker wrapper.
fn write_stats_fake_container(bin_dir: &Path) -> PathBuf {
    let script = bin_dir.join("container");
    fs::create_dir_all(bin_dir).expect("create fake container directory");
    fs::write(
        &script,
        r#"#!/bin/sh
set -eu

if [ -n "${VAT_FAKE_CONTAINER_LOG:-}" ]; then
  printf '%s\n' "$*" >> "$VAT_FAKE_CONTAINER_LOG"
fi

if [ "${1:-}" != "stats" ] \
  || [ "${2:-}" != "--format" ] \
  || [ "${3:-}" != "json" ] \
  || [ "${4:-}" != "--no-stream" ] \
  || [ -z "${5:-}" ]; then
  printf 'unexpected fake Apple Container stats argv: %s\n' "$*" >&2
  exit 64
fi

case "${VAT_FAKE_STATS_MODE:-valid}" in
  valid)
    printf '%s\n' '{"samples":[{"id":"agent-web","cpuUsageUsec":17,"memoryUsageBytes":42}]}'
    ;;
  nonzero)
    printf '%s\n' '{"samples":[{"id":"agent-web","cpuUsageUsec":17,"memoryUsageBytes":42}]}'
    printf '%s\n' 'fake Apple Container stats failure' >&2
    exit 47
    ;;
  malformed)
    printf '%s\n' 'raw-invalid-stats-marker'
    ;;
  oversized_valid)
    # Valid JSON whose opaque Apple payload exceeds VAT's 256 KiB capture
    # budget. The shim must drain it fully but suppress it rather than replay a
    # partial JSON document.
    printf '%s' '{"samples":["'
    dd if=/dev/zero bs=262145 count=1 2>/dev/null | tr '\000' x
    printf '%s\n' '"]}'
    ;;
  flood)
    # Drain both pipes concurrently: the background stderr producer and this
    # foreground stdout producer would deadlock a sequential reader.
    (dd if=/dev/zero bs=262145 count=1 2>/dev/null | tr '\000' e >&2) &
    dd if=/dev/zero bs=262145 count=1 2>/dev/null | tr '\000' o
    wait
    ;;
  *)
    printf 'unexpected fake stats mode: %s\n' "$VAT_FAKE_STATS_MODE" >&2
    exit 64
    ;;
esac
"#,
    )
    .expect("write fake stats container");
    let mut permissions = fs::metadata(&script)
        .expect("fake stats container metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).expect("make fake stats container executable");
    script
}

/// A strict Apple Container list double for the Docker-shaped agent inventory
/// surface. It knows no Docker schema: its native JSON payload is intentionally
/// opaque to the test and must be replayed unchanged only after VAT validates
/// the complete bounded document.
fn write_ps_json_fake_container(bin_dir: &Path) -> PathBuf {
    let script = bin_dir.join("container");
    fs::create_dir_all(bin_dir).expect("create fake container directory");
    fs::write(
        &script,
        r#"#!/bin/sh
set -eu

if [ -n "${VAT_FAKE_CONTAINER_LOG:-}" ]; then
  printf '%s\n' "$*" >> "$VAT_FAKE_CONTAINER_LOG"
fi

if [ "${1:-}" != "list" ] \
  || [ "${2:-}" != "--format" ] \
  || [ "${3:-}" != "json" ]; then
  printf 'unexpected fake Apple Container list argv: %s\n' "$*" >&2
  exit 64
fi

case "${4:-}" in
  '')
    ;;
  --all)
    [ -z "${5:-}" ] || {
      printf 'unexpected fake Apple Container list argv suffix: %s\n' "$*" >&2
      exit 64
    }
    ;;
  *)
    printf 'unexpected fake Apple Container list argv suffix: %s\n' "$*" >&2
    exit 64
    ;;
esac

case "${VAT_FAKE_PS_JSON_MODE:-valid}" in
  valid)
    printf '%s\n' '[{"id":"agent-web","state":"running","appleExtra":{"opaque":true}}]'
    ;;
  nonzero)
    printf '%s\n' '[{"id":"agent-web","state":"running","appleExtra":{"opaque":true}}]'
    printf '%s\n' 'fake Apple Container list failure' >&2
    exit 46
    ;;
  malformed)
    printf '%s\n' 'raw-invalid-list-marker'
    ;;
  oversized_valid)
    printf '%s' '[{"opaque":"'
    dd if=/dev/zero bs=262145 count=1 2>/dev/null | tr '\000' x
    printf '%s\n' '"}]'
    ;;
  flood)
    (dd if=/dev/zero bs=262145 count=1 2>/dev/null | tr '\000' e >&2) &
    dd if=/dev/zero bs=262145 count=1 2>/dev/null | tr '\000' o
    wait
    ;;
  *)
    printf 'unexpected fake ps JSON mode: %s\n' "$VAT_FAKE_PS_JSON_MODE" >&2
    exit 64
    ;;
esac
"#,
    )
    .expect("write fake ps JSON container");
    let mut permissions = fs::metadata(&script)
        .expect("fake ps JSON container metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).expect("make fake ps JSON container executable");
    script
}

/// A strict Apple Container image-list double. Its opaque JSON is a native
/// backend payload, so these tests can prove VAT validates/replays bytes
/// without inventing a Docker image schema or accepting image selectors.
fn write_images_json_fake_container(bin_dir: &Path) -> PathBuf {
    let script = bin_dir.join("container");
    fs::create_dir_all(bin_dir).expect("create fake container directory");
    fs::write(
        &script,
        r#"#!/bin/sh
set -eu

if [ -n "${VAT_FAKE_CONTAINER_LOG:-}" ]; then
  printf '%s\n' "$*" >> "$VAT_FAKE_CONTAINER_LOG"
fi

if [ "${1:-}" != "image" ] \
  || [ "${2:-}" != "list" ] \
  || [ "${3:-}" != "--format" ] \
  || [ "${4:-}" != "json" ] \
  || [ -n "${5:-}" ]; then
  printf 'unexpected fake Apple Container image list argv: %s\n' "$*" >&2
  exit 64
fi

case "${VAT_FAKE_IMAGES_JSON_MODE:-valid}" in
  valid)
    printf '%s\n' '[{"reference":"agent/web:latest","digest":"sha256:opaque","appleExtra":{"opaque":true}}]'
    ;;
  nonzero)
    printf '%s\n' '[{"reference":"agent/web:latest","digest":"sha256:opaque","appleExtra":{"opaque":true}}]'
    printf '%s\n' 'fake Apple Container image list failure' >&2
    exit 45
    ;;
  malformed)
    printf '%s\n' 'raw-invalid-image-list-marker'
    ;;
  oversized_valid)
    printf '%s' '[{"opaque":"'
    dd if=/dev/zero bs=262145 count=1 2>/dev/null | tr '\000' x
    printf '%s\n' '"}]'
    ;;
  flood)
    (dd if=/dev/zero bs=262145 count=1 2>/dev/null | tr '\000' e >&2) &
    dd if=/dev/zero bs=262145 count=1 2>/dev/null | tr '\000' o
    wait
    ;;
  *)
    printf 'unexpected fake images JSON mode: %s\n' "$VAT_FAKE_IMAGES_JSON_MODE" >&2
    exit 64
    ;;
esac
"#,
    )
    .expect("write fake images JSON container");
    let mut permissions = fs::metadata(&script)
        .expect("fake images JSON container metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).expect("make fake images JSON container executable");
    script
}

/// A strict Apple Container direct-inspect double. The backend itself has no
/// output-format flag; this proves VAT validates the Docker-shaped selector
/// and strips it before replaying the opaque native inspect document.
fn write_inspect_json_fake_container(bin_dir: &Path) -> PathBuf {
    let script = bin_dir.join("container");
    fs::create_dir_all(bin_dir).expect("create fake container directory");
    fs::write(
        &script,
        r#"#!/bin/sh
set -eu

if [ -n "${VAT_FAKE_CONTAINER_LOG:-}" ]; then
  printf '%s\n' "$*" >> "$VAT_FAKE_CONTAINER_LOG"
fi

if [ "${1:-}" != "inspect" ] \
  || [ "${2:-}" != "agent-web" ] \
  || [ -n "${3:-}" ]; then
  printf 'unexpected fake Apple Container inspect argv: %s\n' "$*" >&2
  exit 64
fi

case "${VAT_FAKE_INSPECT_JSON_MODE:-valid}" in
  valid)
    printf '%s\n' '[{"id":"agent-web","state":"running","appleExtra":{"opaque":true}}]'
    ;;
  nonzero)
    printf '%s\n' '[{"id":"agent-web","state":"running","appleExtra":{"opaque":true}}]'
    printf '%s\n' 'fake Apple Container inspect failure' >&2
    exit 44
    ;;
  malformed)
    printf '%s\n' 'raw-invalid-inspect-marker'
    ;;
  oversized_valid)
    printf '%s' '[{"opaque":"'
    dd if=/dev/zero bs=262145 count=1 2>/dev/null | tr '\000' x
    printf '%s\n' '"}]'
    ;;
  flood)
    (dd if=/dev/zero bs=262145 count=1 2>/dev/null | tr '\000' e >&2) &
    dd if=/dev/zero bs=262145 count=1 2>/dev/null | tr '\000' o
    wait
    ;;
  *)
    printf 'unexpected fake inspect JSON mode: %s\n' "$VAT_FAKE_INSPECT_JSON_MODE" >&2
    exit 64
    ;;
esac
"#,
    )
    .expect("write fake inspect JSON container");
    let mut permissions = fs::metadata(&script)
        .expect("fake inspect JSON container metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).expect("make fake inspect JSON container executable");
    script
}

/// A strict Apple Container image-inspect double. The backend itself has no
/// output-format flag; this proves VAT validates the Docker-shaped selector
/// and strips it before replaying the opaque native image document.
fn write_image_inspect_json_fake_container(bin_dir: &Path) -> PathBuf {
    let script = bin_dir.join("container");
    fs::create_dir_all(bin_dir).expect("create fake container directory");
    fs::write(
        &script,
        r#"#!/bin/sh
set -eu

if [ -n "${VAT_FAKE_CONTAINER_LOG:-}" ]; then
  printf '%s\n' "$*" >> "$VAT_FAKE_CONTAINER_LOG"
fi

if [ "${1:-}" != "image" ] \
  || [ "${2:-}" != "inspect" ] \
  || [ "${3:-}" != "alpine:3.20" ] \
  || [ -n "${4:-}" ]; then
  printf 'unexpected fake Apple Container image inspect argv: %s\n' "$*" >&2
  exit 64
fi

case "${VAT_FAKE_IMAGE_INSPECT_JSON_MODE:-valid}" in
  valid)
    printf '%s\n' '[{"configuration":{"name":"docker.io/library/alpine:3.20"},"descriptor":{"digest":"sha256:opaque"},"appleExtra":{"opaque":true}}]'
    ;;
  nonzero)
    printf '%s\n' '[{"configuration":{"name":"docker.io/library/alpine:3.20"},"descriptor":{"digest":"sha256:opaque"},"appleExtra":{"opaque":true}}]'
    printf '%s\n' 'fake Apple Container image inspect failure' >&2
    exit 42
    ;;
  malformed)
    printf '%s\n' 'raw-invalid-image-inspect-marker'
    ;;
  oversized_valid)
    printf '%s' '[{"opaque":"'
    dd if=/dev/zero bs=262145 count=1 2>/dev/null | tr '\000' x
    printf '%s\n' '"}]'
    ;;
  flood)
    (dd if=/dev/zero bs=262145 count=1 2>/dev/null | tr '\000' e >&2) &
    dd if=/dev/zero bs=262145 count=1 2>/dev/null | tr '\000' o
    wait
    ;;
  *)
    printf 'unexpected fake image inspect JSON mode: %s\n' "$VAT_FAKE_IMAGE_INSPECT_JSON_MODE" >&2
    exit 64
    ;;
esac
"#,
    )
    .expect("write fake image inspect JSON container");
    let mut permissions = fs::metadata(&script)
        .expect("fake image inspect JSON container metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions)
        .expect("make fake image inspect JSON container executable");
    script
}

/// A strict Apple Container logs double. Apple exposes logs as textual stdio,
/// so the fixture intentionally emits control bytes and invalid UTF-8 rather
/// than pretending that direct logs have a native JSON or stream-demux shape.
fn write_logs_json_fake_container(bin_dir: &Path) -> PathBuf {
    let script = bin_dir.join("container");
    fs::create_dir_all(bin_dir).expect("create fake container directory");
    fs::write(
        &script,
        r#"#!/bin/sh
set -eu

if [ -n "${VAT_FAKE_CONTAINER_LOG:-}" ]; then
  printf '%s\n' "$*" >> "$VAT_FAKE_CONTAINER_LOG"
fi

if [ "${1:-}" != "logs" ] \
  || [ "${2:-}" != "-n" ] \
  || [ "${3:-}" != "17" ] \
  || [ "${4:-}" != "agent-web" ] \
  || [ -n "${5:-}" ]; then
  printf 'unexpected fake Apple Container logs argv: %s\n' "$*" >&2
  exit 64
fi

case "${VAT_FAKE_LOGS_JSON_MODE:-valid}" in
  valid)
    printf 'line-one\ncontrol:\001\ninvalid:\377\n'
    printf 'diagnostic-control:\001 invalid:\377\n' >&2
    ;;
  nonzero)
    printf 'logs-before-failure\n'
    printf 'fake Apple Container logs failure\n' >&2
    exit 43
    ;;
  flood)
    (dd if=/dev/zero bs=65537 count=2 2>/dev/null | tr '\000' e >&2) &
    dd if=/dev/zero bs=65537 count=2 2>/dev/null | tr '\000' o
    wait
    ;;
  timeout)
    printf 'raw-timeout-log-must-not-be-wrapped\n'
    sleep 30
    ;;
  escaped_holder)
    [ -n "${VAT_LOGS_ESCAPED_PIPE_HOLDER_HELPER:-}" ] || {
      printf 'missing escaped pipe-holder helper\n' >&2
      exit 64
    }
    exec "$VAT_LOGS_ESCAPED_PIPE_HOLDER_HELPER" \
      --exact docker_logs_json_escaped_pipe_holder_helper --nocapture
    ;;
  *)
    printf 'unexpected fake logs mode: %s\n' "$VAT_FAKE_LOGS_JSON_MODE" >&2
    exit 64
    ;;
esac
"#,
    )
    .expect("write fake logs JSON container");
    let mut permissions = fs::metadata(&script)
        .expect("fake logs JSON container metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).expect("make fake logs JSON container executable");
    script
}

/// A strict Apple Container exec double. The direct JSON shim must strip the
/// Docker-only delimiter while retaining arbitrary stdout/stderr inside one
/// bounded VAT wrapper instead of replaying either child stream.
fn write_exec_json_fake_container(bin_dir: &Path) -> PathBuf {
    let script = bin_dir.join("container");
    fs::create_dir_all(bin_dir).expect("create fake container directory");
    fs::write(
        &script,
        r#"#!/bin/sh
set -eu

if [ -n "${VAT_FAKE_CONTAINER_LOG:-}" ]; then
  printf '%s\n' "$*" >> "${VAT_FAKE_CONTAINER_LOG}"
fi

if [ "${1:-}" != "exec" ] \
  || [ "${2:-}" != "agent-web" ] \
  || [ "${3:-}" != "fixture-command" ] \
  || [ "${4:-}" != "--literal" ] \
  || [ -n "${5:-}" ]; then
  printf 'unexpected fake Apple Container exec argv: %s\n' "$*" >&2
  exit 64
fi

case "${VAT_FAKE_EXEC_JSON_MODE:-valid}" in
  valid)
    printf 'exec-stdout\ncontrol:\001\ninvalid:\377\n'
    printf 'exec-stderr-control:\001 invalid:\377\n' >&2
    ;;
  nonzero)
    printf 'exec-before-failure\n'
    printf 'fake Apple Container exec failure\n' >&2
    exit 42
    ;;
  flood)
    (dd if=/dev/zero bs=65537 count=2 2>/dev/null | tr '\000' e >&2) &
    dd if=/dev/zero bs=65537 count=2 2>/dev/null | tr '\000' o
    wait
    ;;
  timeout)
    printf 'raw-timeout-exec-must-not-be-wrapped\n'
    sleep 30
    ;;
  *)
    printf 'unexpected fake exec mode: %s\n' "${VAT_FAKE_EXEC_JSON_MODE}" >&2
    exit 64
    ;;
esac
"#,
    )
    .expect("write fake exec JSON container");
    let mut permissions = fs::metadata(&script)
        .expect("fake exec JSON container metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).expect("make fake exec JSON container executable");
    script
}

/// A strict Apple Container build double. The Docker-facing JSON/deadline
/// selectors never reach it; this pins the one canonical public build argv
/// and exercises bounded text receipt behavior without building an image.
fn write_build_json_fake_container(bin_dir: &Path) -> PathBuf {
    let script = bin_dir.join("container");
    fs::create_dir_all(bin_dir).expect("create fake container directory");
    fs::write(
        &script,
        r#"#!/bin/sh
set -eu

if [ -n "${VAT_FAKE_CONTAINER_LOG:-}" ]; then
  printf '%s\n' "$*" >> "$VAT_FAKE_CONTAINER_LOG"
fi

if [ "${1:-}" != "build" ] \
  || [ "${2:-}" != "--tag" ] \
  || [ "${3:-}" != "fixture/image:latest" ] \
  || [ "${4:-}" != "--file" ] \
  || [ "${5:-}" != "Dockerfile.fixture" ] \
  || [ "${6:-}" != "--build-arg" ] \
  || [ "${7:-}" != "MODE=test" ] \
  || [ "${8:-}" != "--build-arg" ] \
  || [ "${9:-}" != "TOKEN=opaque" ] \
  || [ "${10:-}" != "--target" ] \
  || [ "${11:-}" != "release" ] \
  || [ "${12:-}" != "--platform" ] \
  || [ "${13:-}" != "linux/arm64" ] \
  || [ "${14:-}" != "--label" ] \
  || [ "${15:-}" != "io.cclab.vat.test=opaque" ] \
  || [ "${16:-}" != "${VAT_FAKE_BUILD_CONTEXT:?VAT_FAKE_BUILD_CONTEXT is required}" ] \
  || [ -n "${17:-}" ]; then
  printf 'unexpected fake Apple Container build argv: %s\n' "$*" >&2
  exit 64
fi

case "${VAT_FAKE_BUILD_JSON_MODE:-valid}" in
  valid)
    printf 'build-stdout\ncontrol:\001\ninvalid:\377\n'
    printf 'build-stderr-control:\001 invalid:\377\n' >&2
    ;;
  nonzero)
    printf 'build-before-failure\n'
    printf 'fake Apple Container build failure\n' >&2
    exit 40
    ;;
  malformed)
    printf 'build-malformed-control:\001 invalid:\377\n'
    printf 'build-malformed-stderr-control:\001 invalid:\377\n' >&2
    ;;
  oversized)
    dd if=/dev/zero bs=65537 count=2 2>/dev/null | tr '\000' o
    printf 'build-oversized-stderr\n' >&2
    ;;
  flood)
    (dd if=/dev/zero bs=65537 count=2 2>/dev/null | tr '\000' e >&2) &
    dd if=/dev/zero bs=65537 count=2 2>/dev/null | tr '\000' o
    wait
    ;;
  timeout)
    printf 'raw-timeout-build-must-not-be-wrapped\n'
    sleep 30
    ;;
  *)
    printf 'unexpected fake build JSON mode: %s\n' "$VAT_FAKE_BUILD_JSON_MODE" >&2
    exit 64
    ;;
esac
"#,
    )
    .expect("write fake build JSON container");
    let mut permissions = fs::metadata(&script)
        .expect("fake build JSON container metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).expect("make fake build JSON container executable");
    script
}

/// A strict Apple Container image-pull double. The Docker-facing
/// JSON/deadline selectors never reach it; arbitrary registry/client output
/// stays inside VAT's bounded wrapper rather than becoming raw shim output.
fn write_pull_json_fake_container(bin_dir: &Path) -> PathBuf {
    let script = bin_dir.join("container");
    fs::create_dir_all(bin_dir).expect("create fake container directory");
    fs::write(
        &script,
        r#"#!/bin/sh
set -eu

if [ -n "${VAT_FAKE_CONTAINER_LOG:-}" ]; then
  printf '%s\n' "$*" >> "$VAT_FAKE_CONTAINER_LOG"
fi

if [ "${1:-}" != "image" ] \
  || [ "${2:-}" != "pull" ] \
  || [ "${3:-}" != "fixture/image:latest" ] \
  || [ -n "${4:-}" ]; then
  printf 'unexpected fake Apple Container image pull argv: %s\n' "$*" >&2
  exit 64
fi

case "${VAT_FAKE_PULL_JSON_MODE:-valid}" in
  valid)
    printf 'pull-stdout\ncontrol:\001\ninvalid:\377\n'
    printf 'pull-stderr-control:\001 invalid:\377\n' >&2
    ;;
  nonzero)
    printf 'pull-before-failure\n'
    printf 'fake Apple Container image pull failure\n' >&2
    exit 41
    ;;
  malformed)
    printf 'pull-malformed-control:\001 invalid:\377\n'
    printf 'pull-malformed-stderr-control:\001 invalid:\377\n' >&2
    ;;
  oversized)
    dd if=/dev/zero bs=65537 count=2 2>/dev/null | tr '\000' o
    printf 'pull-oversized-stderr\n' >&2
    ;;
  flood)
    (dd if=/dev/zero bs=65537 count=2 2>/dev/null | tr '\000' e >&2) &
    dd if=/dev/zero bs=65537 count=2 2>/dev/null | tr '\000' o
    wait
    ;;
  timeout)
    printf 'raw-timeout-pull-must-not-be-wrapped\n'
    sleep 30
    ;;
  *)
    printf 'unexpected fake pull JSON mode: %s\n' "$VAT_FAKE_PULL_JSON_MODE" >&2
    exit 64
    ;;
esac
"#,
    )
    .expect("write fake pull JSON container");
    let mut permissions = fs::metadata(&script)
        .expect("fake pull JSON container metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).expect("make fake pull JSON container executable");
    script
}

/// Native-image fixture for the strict build E2E guards. It models Apple's
/// actual one-image array with labels nested in each OCI variant config, and
/// provides absence/ambiguity outcomes without touching a host image store.
fn write_image_owner_guard_fake_container(bin_dir: &Path) -> PathBuf {
    let script = bin_dir.join("container");
    fs::create_dir_all(bin_dir).expect("create fake image guard directory");
    fs::write(
        &script,
        r#"#!/bin/sh
set -e

if [ -n "$VAT_FAKE_CONTAINER_LOG" ]; then
  printf '%s\n' "$*" >> "$VAT_FAKE_CONTAINER_LOG"
fi

case "$1:$2" in
  image:inspect)
    [ -n "$3" ] && [ -z "$4" ] || {
      printf 'unexpected fake image inspect argv: %s\n' "$*" >&2
      exit 64
    }
    case "$3" in
      missing:latest)
        printf 'Error: image not found: %s\n' "$3" >&2
        exit 1
        ;;
      noisy-missing:latest)
        printf 'Error: image not found: %s\nunexpected extra diagnostic\n' "$3" >&2
        exit 1
        ;;
      present:latest|owned:latest)
        printf '%s\n' '[{"configuration":{"name":"owned:latest"},"variants":[{"config":{"config":{"Labels":{"io.cclab.vat.e2e-owner":"expected-owner"}}}}]}]'
        ;;
      partial:latest)
        printf '%s\n' '[{"configuration":{"name":"partial:latest"},"variants":[{"config":{"config":{"Labels":{"io.cclab.vat.e2e-owner":"expected-owner"}}}},{"config":{"config":{"Labels":{"io.cclab.vat.e2e-owner":"other-owner"}}}}]}]'
        ;;
      missing-label:latest)
        printf '%s\n' '[{"configuration":{"name":"missing-label:latest"},"variants":[{"config":{"config":{"Labels":{"io.cclab.vat.e2e-owner":"expected-owner"}}}},{"config":{"config":{}}}]}]'
        ;;
      legacy:latest)
        printf '%s\n' '[{"configuration":{"labels":{"io.cclab.vat.e2e-owner":"expected-owner"}}}]'
        ;;
      multi-image:latest)
        printf '%s\n' '[{"variants":[{"config":{"config":{"Labels":{"io.cclab.vat.e2e-owner":"expected-owner"}}}}]},{"variants":[{"config":{"config":{"Labels":{"io.cclab.vat.e2e-owner":"expected-owner"}}}}]}]'
        ;;
      malformed:latest)
        printf 'not native JSON\n'
        ;;
      uncertain:latest)
        printf 'Error: fake image backend unavailable\n' >&2
        exit 75
        ;;
      *)
        printf 'unexpected fake image guard tag: %s\n' "$3" >&2
        exit 64
        ;;
    esac
    ;;
  image:delete)
    [ "$3" = "owned:latest" ] && [ -z "$4" ] || {
      printf 'unexpected fake image delete argv: %s\n' "$*" >&2
      exit 64
    }
    ;;
  *)
    printf 'unexpected fake image guard command: %s\n' "$*" >&2
    exit 64
    ;;
esac
"#,
    )
    .expect("write fake image guard container");
    let mut permissions = fs::metadata(&script)
        .expect("fake image guard container metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).expect("make fake image guard container executable");
    script
}

fn strict_build_json_args(context: &Path, timeout_seconds: u64) -> Vec<String> {
    vec![
        "build".to_string(),
        "--format=json".to_string(),
        format!("--timeout={timeout_seconds}"),
        "--tag=fixture/image:latest".to_string(),
        "--file=Dockerfile.fixture".to_string(),
        "--build-arg=MODE=test".to_string(),
        "--build-arg=TOKEN=opaque".to_string(),
        "--target=release".to_string(),
        "--platform=linux/arm64".to_string(),
        "--label=io.cclab.vat.test=opaque".to_string(),
        context
            .to_str()
            .expect("UTF-8 fake build context")
            .to_string(),
    ]
}

fn strict_pull_json_args(timeout_seconds: u64) -> Vec<String> {
    vec![
        "pull".to_string(),
        "--format=json".to_string(),
        format!("--timeout={timeout_seconds}"),
        "fixture/image:latest".to_string(),
    ]
}

fn fake_build_context(root: &Path) -> (PathBuf, String) {
    let context = root.join("build-context");
    fs::create_dir_all(&context).expect("create fake build context");
    let canonical = fs::canonicalize(&context)
        .expect("canonicalize fake build context")
        .to_str()
        .expect("UTF-8 canonical fake build context")
        .to_string();
    (context, canonical)
}

/// A stateful Apple Container double for the strict foreground run path. It
/// accepts only VAT-generated name/label argv, records inspectable ownership,
/// and makes absent cleanup prove the current Apple `container not found`
/// diagnostic rather than treating every nonzero inspect as absence.
fn write_run_json_fake_container(bin_dir: &Path) -> PathBuf {
    let script = bin_dir.join("container");
    fs::create_dir_all(bin_dir).expect("create fake container directory");
    fs::write(
        &script,
        r#"#!/bin/sh
set -eu

if [ -n "${VAT_FAKE_CONTAINER_LOG:-}" ]; then
  printf '%s\n' "$*" >> "$VAT_FAKE_CONTAINER_LOG"
fi

state="${VAT_FAKE_RUN_STATE:?VAT_FAKE_RUN_STATE is required}"
mode="${VAT_FAKE_RUN_JSON_MODE:-valid}"

case "${1:-}" in
  run)
    [ "${2:-}" = "--name" ] || {
      printf 'expected generated --name, got: %s\n' "$*" >&2
      exit 64
    }
    name="${3:-}"
    [ "${4:-}" = "--label" ] || {
      printf 'expected generated --label, got: %s\n' "$*" >&2
      exit 64
    }
    label="${5:-}"
    [ "${6:-}" = "fixture/image:latest" ] || {
      printf 'expected fixture image, got: %s\n' "$*" >&2
      exit 64
    }
    [ "${7:-}" = "fixture-command" ] || {
      printf 'expected fixture command, got: %s\n' "$*" >&2
      exit 64
    }
    [ "${8:-}" = "--literal" ] || {
      printf 'expected literal command arg, got: %s\n' "$*" >&2
      exit 64
    }
    [ -z "${9:-}" ] || {
      printf 'unexpected strict run argv suffix: %s\n' "$*" >&2
      exit 64
    }
    label_key=${label%%=*}
    label_value=${label#*=}
    case "$mode" in
      initial_not_found|inspect_uncertain)
        rm -f "$state"
        ;;
      label_mismatch)
        printf '{"configuration":{"labels":{"%s":"wrong-owner"}}}\n' "$label_key" > "$state"
        ;;
      *)
        printf '{"configuration":{"labels":{"%s":"%s"}}}\n' "$label_key" "$label_value" > "$state"
        ;;
    esac
    case "$mode" in
      valid|initial_not_found|inspect_uncertain|label_mismatch|delete_failure|delete_persists)
        printf 'run-stdout\ncontrol:\001\ninvalid:\377\n'
        printf 'run-stderr-control:\001 invalid:\377\n' >&2
        ;;
      nonzero)
        printf 'run-before-failure\n'
        printf 'fake Apple Container run failure\n' >&2
        exit 41
        ;;
      flood)
        (dd if=/dev/zero bs=65537 count=2 2>/dev/null | tr '\000' e >&2) &
        dd if=/dev/zero bs=65537 count=2 2>/dev/null | tr '\000' o
        wait
        ;;
      timeout)
        printf 'raw-timeout-run-must-not-be-wrapped\n'
        sleep 30
        ;;
      *)
        printf 'unexpected fake run JSON mode: %s\n' "$mode" >&2
        exit 64
        ;;
    esac
    ;;
  inspect)
    [ -n "${2:-}" ] && [ -z "${3:-}" ] || {
      printf 'unexpected fake inspect argv: %s\n' "$*" >&2
      exit 64
    }
    if [ "$mode" = "inspect_uncertain" ]; then
      printf 'Error: Apple Container backend unavailable for %s\n' "$2" >&2
      exit 75
    fi
    if [ -f "$state" ]; then
      cat "$state"
    else
      printf 'Error: container not found: %s\n' "$2" >&2
      exit 1
    fi
    ;;
  delete)
    [ "${2:-}" = "--force" ] && [ -n "${3:-}" ] && [ -z "${4:-}" ] || {
      printf 'unexpected fake delete argv: %s\n' "$*" >&2
      exit 64
    }
    case "$mode" in
      delete_failure)
        printf 'fake Apple Container delete failure\n' >&2
        exit 74
        ;;
      delete_persists)
        ;;
      *)
        rm -f "$state"
        ;;
    esac
    ;;
  *)
    printf 'unexpected fake Apple Container run fixture argv: %s\n' "$*" >&2
    exit 64
    ;;
esac
"#,
    )
    .expect("write fake run JSON container");
    let mut permissions = fs::metadata(&script)
        .expect("fake run JSON container metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).expect("make fake run JSON container executable");
    script
}

/// Re-exec helper for the escaped-pipe regression below. The direct `container
/// logs` root is this test binary; its child moves to a new session, retains
/// the inherited capture pipes, and waits only on a test-owned release marker.
/// The post-fork child path stays raw-libc-only so it is safe even when the
/// test harness has spawned worker threads.
#[cfg(unix)]
#[test]
fn docker_logs_json_escaped_pipe_holder_helper() {
    let Some(ready_path) = std::env::var_os("VAT_LOGS_ESCAPED_PIPE_HOLDER_READY_PATH") else {
        return;
    };
    let release_path = PathBuf::from(
        std::env::var_os("VAT_LOGS_ESCAPED_PIPE_HOLDER_RELEASE_PATH")
            .expect("escaped logs pipe-holder release path"),
    );
    let exited_path = PathBuf::from(
        std::env::var_os("VAT_LOGS_ESCAPED_PIPE_HOLDER_EXITED_PATH")
            .expect("escaped logs pipe-holder exited path"),
    );
    let release_path = CString::new(release_path.as_os_str().as_bytes())
        .expect("NUL-free escaped logs pipe-holder release path");
    let exited_path = CString::new(exited_path.as_os_str().as_bytes())
        .expect("NUL-free escaped logs pipe-holder exited path");
    let mut ready_pipe = [-1_i32; 2];
    assert_eq!(
        unsafe { libc::pipe(ready_pipe.as_mut_ptr()) },
        0,
        "create escaped logs pipe-holder readiness pipe"
    );
    let child_pid = unsafe { libc::fork() };
    assert!(child_pid >= 0, "fork escaped logs pipe holder");
    if child_pid == 0 {
        unsafe {
            libc::close(ready_pipe[0]);
        }
        let marker = if unsafe { libc::setsid() } >= 0 {
            b"1"
        } else {
            b"0"
        };
        unsafe {
            libc::write(
                ready_pipe[1],
                marker.as_ptr().cast::<libc::c_void>(),
                marker.len(),
            );
            libc::close(ready_pipe[1]);
        }
        if marker == b"1" {
            // This unique marker is the only cleanup authority. It avoids a
            // numeric PID signal that could be redirected after PID reuse.
            for _ in 0..1_000 {
                if unsafe { libc::access(release_path.as_ptr(), libc::F_OK) } == 0 {
                    unsafe {
                        let fd = libc::open(
                            exited_path.as_ptr(),
                            libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
                            0o600,
                        );
                        if fd >= 0 {
                            let _ = libc::write(fd, b"1".as_ptr().cast::<libc::c_void>(), 1);
                            libc::close(fd);
                        }
                        libc::_exit(0);
                    }
                }
                unsafe {
                    libc::usleep(10_000);
                }
            }
            unsafe {
                libc::_exit(0);
            }
        }
        unsafe {
            libc::_exit(1);
        }
    }

    unsafe {
        libc::close(ready_pipe[1]);
    }
    let mut marker = [0_u8; 1];
    let read = unsafe {
        libc::read(
            ready_pipe[0],
            marker.as_mut_ptr().cast::<libc::c_void>(),
            marker.len(),
        )
    };
    unsafe {
        libc::close(ready_pipe[0]);
    }
    assert_eq!(read, 1, "read escaped logs pipe-holder readiness marker");
    assert_eq!(
        marker, *b"1",
        "escaped logs pipe holder must create a session"
    );
    fs::write(ready_path, b"ready").expect("record escaped logs pipe-holder readiness");
}

/// A small Apple Container double that preserves the process shape VAT needs
/// for Compose lifecycle coverage: `run` remains alive until VAT stops its
/// child, while the metadata and exec verbs succeed as the public CLI would.
/// Readiness itself is supplied by test-owned loopback listeners, so this
/// stays deterministic and does not emulate an undocumented runtime network.
fn write_lifecycle_fake_container(bin_dir: &Path) -> PathBuf {
    let script = bin_dir.join("container");
    fs::create_dir_all(bin_dir).expect("create fake container directory");
    fs::write(
        &script,
        r#"#!/bin/sh
set -eu

if [ -n "${VAT_FAKE_CONTAINER_LOG:-}" ]; then
  printf '%s\n' "$*" >> "$VAT_FAKE_CONTAINER_LOG"
fi

case "${1:-}" in
  --version)
    printf 'fake-container 1.0\n'
    ;;
  system)
    [ "${2:-}" = "status" ] || exit 2
    ;;
  image)
    [ "${2:-}" = "inspect" ] || exit 2
    printf '{}\n'
    ;;
  run)
    exec /bin/sleep 30
    ;;
  exec)
    if [ "${3:-}" = "--" ]; then
      printf 'fake Apple Container exec must not receive Docker-only separator\n' >&2
      exit 64
    fi
    case "$*" in
      *vat-exec-json-nonzero*)
        printf 'exec-json-stdout-one\nexec-json-stdout-two\n'
        printf 'exec-json-stderr-one\nexec-json-stderr-two\n' >&2
        exit 23
        ;;
      *)
        printf 'fake-compose-exec'
        ;;
    esac
    ;;
  rm)
    ;;
  list)
    printf '[]\n'
    ;;
  inspect)
    printf '{}\n'
    ;;
  *)
    exit 2
    ;;
esac
"#,
    )
    .expect("write lifecycle fake container");
    let mut permissions = fs::metadata(&script)
        .expect("lifecycle fake container metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).expect("make lifecycle fake container executable");
    script
}

/// A lifecycle fake with deterministic distinct stdout/stderr lines so the
/// agent JSON log snapshot can prove capture-only, per-stream tail behavior
/// without a host socket or Apple Container runtime.
fn write_logging_lifecycle_fake_container(bin_dir: &Path) -> PathBuf {
    let script = bin_dir.join("container");
    fs::create_dir_all(bin_dir).expect("create fake container directory");
    fs::write(
        &script,
        r#"#!/bin/sh
set -eu

if [ -n "${VAT_FAKE_CONTAINER_LOG:-}" ]; then
  printf '%s\n' "$*" >> "$VAT_FAKE_CONTAINER_LOG"
fi

case "${1:-}" in
  --version)
    printf 'fake-container 1.0\n'
    ;;
  system)
    [ "${2:-}" = "status" ] || exit 2
    ;;
  image)
    [ "${2:-}" = "inspect" ] || exit 2
    printf '{}\n'
    ;;
  run)
    printf 'stdout-one\nstdout-two\nstdout-three'
    printf 'stderr-one\nstderr-two\nstderr-three' >&2
    exec /bin/sleep 30
    ;;
  exec)
    printf 'fake-compose-exec\n'
    ;;
  rm)
    ;;
  list)
    printf '[]\n'
    ;;
  inspect)
    printf '{}\n'
    ;;
  *)
    exit 2
    ;;
esac
"#,
    )
    .expect("write logging lifecycle fake container");
    let mut permissions = fs::metadata(&script)
        .expect("logging lifecycle fake container metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions)
        .expect("make logging lifecycle fake container executable");
    script
}

fn docker_shim(dir: &Path) -> PathBuf {
    let shim = dir.join("docker");
    symlink(vat_bin(), &shim).expect("create docker to vat shim");
    shim
}

fn path_with_prepend(dir: &Path) -> OsString {
    let mut paths = vec![dir.to_path_buf()];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").expect("PATH is set for test process"),
    ));
    std::env::join_paths(paths).expect("join test PATH")
}

fn install_real_shim(dir: &Path) -> PathBuf {
    let output = Command::new(vat_bin())
        .args([
            "docker",
            "install-shim",
            "--dir",
            dir.to_str().expect("UTF-8 temp path"),
        ])
        .output()
        .expect("install real docker shim");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    dir.join("docker")
}

struct RealContainerCleanup {
    name: String,
    owner_label: String,
    owner_token: String,
    active: bool,
}

impl Drop for RealContainerCleanup {
    fn drop(&mut self) {
        if self.active
            && real_container_has_owner_label(&self.name, &self.owner_label, &self.owner_token)
        {
            let _ = Command::new("container")
                .args(["delete", "--force", &self.name])
                .output();
        }
    }
}

/// A real-host test must not deliberately clean up an arbitrary name. Apple
/// Container has no conditional delete, so the best available guard is a
/// high-entropy name plus an inspect-time test-owner label immediately before
/// every emergency delete. If inspect is unavailable or the label does not
/// match, leak rather than delete a potentially foreign container.
fn real_container_has_owner_label(name: &str, label: &str, token: &str) -> bool {
    let Ok(output) = Command::new("container").args(["inspect", name]).output() else {
        return false;
    };
    if !output.status.success() {
        return false;
    }
    let Ok(document) = serde_json::from_slice::<serde_json::Value>(&output.stdout) else {
        return false;
    };
    let container = document
        .as_array()
        .and_then(|containers| containers.first())
        .unwrap_or(&document);
    container
        .get("configuration")
        .and_then(|configuration| configuration.get("labels"))
        .and_then(|labels| labels.get(label))
        .and_then(serde_json::Value::as_str)
        == Some(token)
}

struct RealImageCleanup {
    tag: String,
    active: bool,
}

impl Drop for RealImageCleanup {
    fn drop(&mut self) {
        if self.active {
            let _ = Command::new("container")
                .args(["image", "delete", &self.tag])
                .output();
        }
    }
}

/// Strict build-receipt E2Es must never use the legacy name-only image
/// cleanup guard above. Image tags are caller-selected and a nonzero build can
/// retain a partial/replaced tag, so each delete is authorized only by an
/// immediately preceding native image-inspect label match. Ambiguity leaks.
struct RealOwnedImageCleanup {
    container_binary: PathBuf,
    tag: String,
    owner_label: String,
    owner_token: String,
    active: bool,
}

impl Drop for RealOwnedImageCleanup {
    fn drop(&mut self) {
        if self.active {
            let _ = delete_real_owned_image(
                &self.container_binary,
                &self.tag,
                &self.owner_label,
                &self.owner_token,
            );
        }
    }
}

/// Apple Container image inspect is a one-image array. Its image labels are
/// per-platform OCI config data at variants[*].config.config.Labels, not in
/// the top-level configuration summary. A caller-selected tag is eligible
/// for cleanup only when every returned variant has the exact test label.
fn native_image_has_exact_owner_label(
    document: &serde_json::Value,
    label: &str,
    token: &str,
) -> bool {
    let Some(images) = document.as_array() else {
        return false;
    };
    if images.len() != 1 {
        return false;
    }
    let Some(variants) = images[0].get("variants").and_then(serde_json::Value::as_array) else {
        return false;
    };
    !variants.is_empty()
        && variants.iter().all(|variant| {
            variant
                .get("config")
                .and_then(|config| config.get("config"))
                .and_then(|config| config.get("Labels"))
                .and_then(|labels| labels.get(label))
                .and_then(serde_json::Value::as_str)
                == Some(token)
        })
}

/// The only accepted absence proof is the native CLI's exact, single-line
/// not-found diagnostic on stderr. A successful inspect, a malformed result,
/// or any other failure remains ambiguous and must abort before build.
fn native_image_inspect_proves_exact_absence(output: &Output, tag: &str) -> bool {
    !output.status.success()
        && output.stdout.is_empty()
        && output.stderr == format!("Error: image not found: {tag}\n").as_bytes()
}

fn real_image_tag_is_proven_absent(container_binary: &Path, tag: &str) -> bool {
    Command::new(container_binary)
        .args(["image", "inspect", tag])
        .output()
        .is_ok_and(|output| native_image_inspect_proves_exact_absence(&output, tag))
}

fn real_image_has_owner_label(
    container_binary: &Path,
    tag: &str,
    label: &str,
    token: &str,
) -> bool {
    let Ok(output) = Command::new(container_binary)
        .args(["image", "inspect", tag])
        .output()
    else {
        return false;
    };
    if !output.status.success() {
        return false;
    }
    let Ok(document) = serde_json::from_slice::<serde_json::Value>(&output.stdout) else {
        return false;
    };
    native_image_has_exact_owner_label(&document, label, token)
}

/// Apple Container does not expose a compare-and-delete primitive, so an
/// inspect-to-delete race remains unavoidable. The caller first uses a
/// high-entropy tag and exact absence proof; immediately before delete this
/// function rechecks every native image variant and leaks on all ambiguity.
fn delete_real_owned_image(
    container_binary: &Path,
    tag: &str,
    label: &str,
    token: &str,
) -> bool {
    if !real_image_has_owner_label(container_binary, tag, label, token) {
        return false;
    }
    Command::new(container_binary)
        .args(["image", "delete", tag])
        .output()
        .is_ok_and(|output| output.status.success())
}

/// Emergency cleanup for the real Compose compatibility probe. The normal test
/// path proves `docker compose down`; this guard only prevents an interrupted
/// ignored E2E from leaving its project-owned Apple container behind.
struct RealComposeCleanup {
    shim: PathBuf,
    vat_home: PathBuf,
    project: String,
    container_name: Option<String>,
    image_tag: Option<String>,
    active: bool,
}

impl Drop for RealComposeCleanup {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        let _ = Command::new(&self.shim)
            .env("VAT_HOME", &self.vat_home)
            .args(["compose", "-p", &self.project, "down"])
            .output();
        if let Some(name) = &self.container_name {
            let _ = Command::new("container")
                .args(["delete", "--force", name])
                .output();
        }
        if let Some(tag) = &self.image_tag {
            let _ = Command::new("container")
                .args(["image", "delete", tag])
                .output();
        }
    }
}

/// Emergency cleanup for the real, multi-service Compose probe. The regular
/// assertion path proves `docker compose down`; this guard only recovers the
/// two names derived from this test's own VAT registry if an ignored E2E is
/// interrupted. It deliberately never removes images, projects, or any
/// container whose exact test-owned name was not recorded first.
struct RealIndependentComposeCleanup {
    shim: PathBuf,
    vat_home: PathBuf,
    project: String,
    container_names: Vec<String>,
    active: bool,
}

impl Drop for RealIndependentComposeCleanup {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        let _ = Command::new(&self.shim)
            .env("VAT_HOME", &self.vat_home)
            .args(["compose", "-p", &self.project, "down"])
            .output();
        for name in &self.container_names {
            let _ = Command::new("container")
                .args(["delete", "--force", name])
                .output();
        }
    }
}

fn wait_for_http_ok(port: u16) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(20);
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let mut last = "host endpoint was not probed".to_string();
    while Instant::now() < deadline {
        match TcpStream::connect_timeout(&address, Duration::from_millis(500)) {
            Ok(mut stream) => {
                let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
                let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));
                match stream
                    .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                {
                    Ok(()) => {
                        let mut response = [0u8; 96];
                        match stream.read(&mut response) {
                            Ok(read) if read > 0 => {
                                let head = String::from_utf8_lossy(&response[..read]);
                                if head.starts_with("HTTP/1.0 2") || head.starts_with("HTTP/1.1 2")
                                {
                                    return Ok(());
                                }
                                last = format!("host endpoint returned {head:?}");
                            }
                            Ok(_) => last = "host endpoint closed before HTTP response".to_string(),
                            Err(error) => last = format!("host endpoint read failed: {error}"),
                        }
                    }
                    Err(error) => last = format!("host endpoint write failed: {error}"),
                }
            }
            Err(error) => last = format!("host endpoint connect failed: {error}"),
        }
        thread::sleep(Duration::from_millis(150));
    }
    Err(last)
}

fn wait_for_port_to_close(port: u16) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(15);
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let mut last = "host endpoint was not probed".to_string();
    while Instant::now() < deadline {
        match TcpStream::connect_timeout(&address, Duration::from_millis(500)) {
            Err(_) => return Ok(()),
            Ok(_) => last = "host endpoint still accepts TCP connections".to_string(),
        }
        thread::sleep(Duration::from_millis(150));
    }
    Err(last)
}

fn assert_port_closed_and_bindable(port: u16) {
    wait_for_port_to_close(port).unwrap_or_else(|error| {
        panic!("Docker Compose shim host port {port} remained usable after down: {error}")
    });
    let listener =
        TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, port))).unwrap_or_else(|error| {
            panic!("Docker Compose shim host port {port} was not bindable after down: {error}")
        });
    drop(listener);
}

fn reserve_two_unique_loopback_ports() -> (u16, u16) {
    let first = TcpListener::bind("127.0.0.1:0").expect("reserve first loopback host port");
    let first_port = first
        .local_addr()
        .expect("first reserved loopback address")
        .port();
    let second = TcpListener::bind("127.0.0.1:0").expect("reserve second loopback host port");
    let second_port = second
        .local_addr()
        .expect("second reserved loopback address")
        .port();
    assert_ne!(
        first_port, second_port,
        "concurrently reserved loopback ports must be unique"
    );
    // The strict profile requires explicit nonzero host ports, so release both
    // reservations immediately before VAT asks Apple Container to bind them.
    drop(second);
    drop(first);
    (first_port, second_port)
}

fn request_http_path(port: u16, path: &str) -> Result<(), String> {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(2))
        .map_err(|error| format!("connect {address} for {path}: {error}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("set read timeout for {path}: {error}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("set write timeout for {path}: {error}"))?;
    let request = format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .map_err(|error| format!("write {path}: {error}"))?;
    let mut response = [0u8; 128];
    let read = stream
        .read(&mut response)
        .map_err(|error| format!("read {path}: {error}"))?;
    let head = String::from_utf8_lossy(&response[..read]);
    if head.starts_with("HTTP/1.0 ") || head.starts_with("HTTP/1.1 ") {
        Ok(())
    } else {
        Err(format!(
            "{path} did not return an HTTP status line: {head:?}"
        ))
    }
}

fn wait_for_compose_log_marker(
    shim: &Path,
    vat_home: &Path,
    project: &str,
    service: &str,
    marker: &str,
) -> Output {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let output = Command::new(shim)
            .env("VAT_HOME", vat_home)
            .args(["compose", "-p", project, "logs", service])
            .output()
            .expect("read Docker-shaped Compose service logs");
        if output.status.success() && output_text(&output).contains(marker) {
            return output;
        }
        if Instant::now() >= deadline {
            panic!(
                "Compose service {service} logs never contained marker {marker:?}:\n{}",
                output_text(&output)
            );
        }
        thread::sleep(Duration::from_millis(150));
    }
}

/// This fake-runtime regression needs real loopback listeners because VAT's
/// MicroVM profile proves the published endpoint after the TCP handshake. A
/// few restricted CI sandboxes prohibit all local sockets; preserve the test
/// as an opt-in hard gate there rather than misreporting that environmental
/// policy as a Compose failure.
fn loopback_listener_or_skip(test: &str) -> Option<TcpListener> {
    match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => Some(listener),
        Err(error) if error.kind() == ErrorKind::PermissionDenied => {
            if std::env::var("VAT_DOCKER_COMPOSE_SHIM_LIFECYCLE_REQUIRED").as_deref() == Ok("1") {
                panic!("{test} requires loopback sockets, but this runner forbids them: {error}");
            }
            eprintln!("Skipping {test}: loopback sockets are unavailable ({error})");
            None
        }
        Err(error) => panic!("bind loopback listener for {test}: {error}"),
    }
}

fn wait_for_compose_ready(shim: &Path, vat_home: &Path, project: &str) -> Output {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let output = Command::new(shim)
            .env("VAT_HOME", vat_home)
            .args(["compose", "-p", project, "ps"])
            .output()
            .expect("poll docker compose ps through VAT shim");
        let text = format!(
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        if output.status.success() && text.contains("is ready") {
            return output;
        }
        if Instant::now() >= deadline {
            panic!("compose project {project} never became ready:\n{text}");
        }
        thread::sleep(Duration::from_millis(150));
    }
}

fn compose_vat_id(vat_home: &Path, project: &str) -> String {
    let record_path = vat_home.join("compose").join(project).join("project.json");
    let record = fs::read_to_string(&record_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", record_path.display()));
    let record: serde_json::Value = serde_json::from_str(&record)
        .unwrap_or_else(|error| panic!("parse {}: {error}", record_path.display()));
    record
        .get("vat_id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| {
            panic!(
                "{} did not retain a compose-owned vat_id: {record}",
                record_path.display()
            )
        })
        .to_string()
}

/// Extract the final public Compose result from mixed lifecycle output. The
/// source-build E2E must obtain ownership details here, never from VAT_HOME.
fn compose_shim_result(stdout: &[u8], command: &str) -> serde_json::Value {
    let text = String::from_utf8_lossy(stdout);
    text.lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .find(|value| {
            value.get("type").and_then(serde_json::Value::as_str) == Some("vat_docker_compose")
                && value.get("command").and_then(serde_json::Value::as_str) == Some(command)
        })
        .unwrap_or_else(|| panic!("missing docker compose {command} result in:\n{text}"))
}

fn output_text(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_shim_provenance_rejected(output: &Output, expected: &str) {
    assert!(
        !output.status.success(),
        "Docker-shaped lifecycle command unexpectedly succeeded:\n{}",
        output_text(output)
    );
    assert!(
        output_text(output).contains(expected),
        "Docker-shaped lifecycle rejection must fail closed on provenance `{expected}`:\n{}",
        output_text(output)
    );
}

#[test]
fn docker_shim_help_advertises_the_strict_native_and_vat_json_contracts() {
    let root = TempDir::new().expect("temp root");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    let shim = docker_shim(&shim_dir);

    let output = Command::new(&shim)
        .arg("--help")
        .output()
        .expect("show docker shim help");
    assert!(output.status.success(), "{}", output_text(&output));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("docker images --format json")
            && stdout.contains("docker image ls")
            && stdout.contains("docker image list"),
        "help must advertise only the documented strict image JSON forms: {stdout}"
    );
    assert!(
        stdout.contains("Apple Container JSON document unchanged")
            && stdout.contains("not Docker Engine image schema"),
        "help must preserve the native-JSON/no-Docker-schema boundary: {stdout}"
    );
    assert!(
        stdout.contains("docker inspect --format json CONTAINER")
            && stdout.contains("docker container inspect"),
        "help must advertise only the documented strict direct container inspect forms: {stdout}"
    );
    assert!(
        stdout.contains("not Docker Engine inspect schema")
            && stdout.contains("secret-redaction guarantee"),
        "help must preserve inspect's native-JSON and no-redaction boundary: {stdout}"
    );
    assert!(
        stdout.contains("docker logs --format json --tail LINES CONTAINER")
            && stdout.contains("docker container logs")
            && stdout.contains("container logs -n LINES CONTAINER"),
        "help must advertise only the bounded direct logs JSON forms and canonical Apple argv: {stdout}"
    );
    assert!(
        stdout.contains("vat.docker.logs.v1")
            && stdout.contains("apple_container_stdio")
            && stdout.contains("untrusted content")
            && stdout.contains("--follow, --boot"),
        "help must disclose the direct logs wrapper and no-streaming boundary: {stdout}"
    );
}

#[test]
fn docker_images_text_and_quiet_aliases_keep_the_preexisting_generic_translation() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");

    for args in [
        ["images"].as_slice(),
        ["images", "-q"].as_slice(),
        ["image", "ls"].as_slice(),
        ["image", "ls", "--quiet"].as_slice(),
        ["image", "list"].as_slice(),
        ["image", "list", "-q"].as_slice(),
    ] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &log)
            .args(args)
            .output()
            .expect("run text/quiet docker images alias through shim");
        assert!(
            output.status.success(),
            "args: {args:?}\n{}",
            output_text(&output)
        );
    }

    assert_eq!(
        fs::read_to_string(&log).expect("read text/quiet image list invocations"),
        "image list\nimage list -q\nimage list\nimage list --quiet\nimage list\nimage list -q\n",
        "no-format image aliases must stay on their pre-existing generic Apple Container path"
    );
}

#[test]
fn docker_inspect_text_aliases_keep_the_preexisting_generic_translation() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");

    for args in [
        ["inspect", "agent-web"].as_slice(),
        ["container", "inspect", "agent-web"].as_slice(),
    ] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &log)
            .args(args)
            .output()
            .expect("run unformatted docker inspect alias through shim");
        assert!(
            output.status.success(),
            "args: {args:?}\n{}",
            output_text(&output)
        );
    }

    assert_eq!(
        fs::read_to_string(&log).expect("read unformatted inspect invocations"),
        "inspect agent-web\ninspect agent-web\n",
        "unformatted direct/container inspect aliases must retain the pre-existing generic Apple Container path"
    );
}

#[test]
fn docker_logs_text_aliases_keep_the_preexisting_generic_translation() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");

    for args in [
        ["logs", "agent-web"].as_slice(),
        ["logs", "--tail", "5", "agent-web"].as_slice(),
        ["container", "logs", "agent-web"].as_slice(),
        ["container", "logs", "--tail=5", "agent-web"].as_slice(),
    ] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &log)
            .args(args)
            .output()
            .expect("run unformatted docker logs alias through shim");
        assert!(
            output.status.success(),
            "args: {args:?}\n{}",
            output_text(&output)
        );
    }

    assert_eq!(
        fs::read_to_string(&log).expect("read unformatted logs invocations"),
        "logs agent-web\nlogs -n 5 agent-web\nlogs agent-web\nlogs -n 5 agent-web\n",
        "unformatted direct/container logs aliases must retain the pre-existing generic Apple Container path"
    );
}

#[test]
fn shim_translates_common_run_to_fake_apple_container() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &log)
        .args([
            "run",
            "--name",
            "web",
            "-p",
            "127.0.0.1:8080:80",
            "-e",
            "MODE=test",
            "--cpus",
            "2",
            "nginx:alpine",
        ])
        .output()
        .expect("run docker shim");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(log).expect("read fake container log"),
        "run --name web -p 127.0.0.1:8080:80 -e MODE=test --cpus 2 nginx:alpine\n"
    );
}

#[test]
fn shim_rejects_engine_and_semantically_unsafe_flags_before_runtime() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");

    for args in [
        ["info"].as_slice(),
        ["run", "-p", "80", "nginx:alpine"].as_slice(),
        ["rm", "--all"].as_slice(),
        ["stop", "--all"].as_slice(),
        ["kill", "-a"].as_slice(),
        ["image", "prune"].as_slice(),
        ["network", "prune"].as_slice(),
        ["volume", "prune"].as_slice(),
        ["login", "--scheme", "http", "registry.example"].as_slice(),
        ["run", "-c", "1024", "nginx:alpine"].as_slice(),
        ["run", "--no-dns", "nginx:alpine"].as_slice(),
        ["build", "--progress", "plain", "."].as_slice(),
    ] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &log)
            .args(args)
            .output()
            .expect("run rejected docker shim command");
        assert!(!output.status.success(), "args: {args:?}");
    }

    assert!(
        !log.exists(),
        "unsupported commands must not invoke Apple Container: {}",
        log.display()
    );
}

#[test]
fn docker_stats_replays_one_valid_apple_native_json_document_with_canonical_runtime_argv() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_stats_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &log)
        .args([
            "stats",
            "--format=json",
            "--no-stream",
            "agent-web",
            "agent-db",
        ])
        .output()
        .expect("run strict docker stats shim");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.stdout,
        b"{\"samples\":[{\"id\":\"agent-web\",\"cpuUsageUsec\":17,\"memoryUsageBytes\":42}]}\n",
        "the public result must remain exactly Apple Container's JSON, not a VAT wrapper"
    );
    assert!(output.stderr.is_empty(), "unexpected stats stderr: {}", output_text(&output));
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("validated Apple stats JSON");
    assert_eq!(
        parsed["samples"][0]["id"],
        serde_json::json!("agent-web")
    );
    assert_eq!(
        fs::read_to_string(&log).expect("read fake stats invocation"),
        "stats --format json --no-stream agent-web agent-db\n",
        "the shim must normalize to the exact bounded Apple Container stats argv"
    );
}

#[test]
fn docker_stats_rejects_streaming_templates_and_unknown_flags_before_runtime() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_stats_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");

    for args in [
        ["stats", "--format", "json", "agent-web"].as_slice(),
        ["stats", "--no-stream", "agent-web"].as_slice(),
        ["stats", "--stream", "--format", "json", "agent-web"].as_slice(),
        ["stats", "--no-stream", "--format", "{{.CPUPerc}}", "agent-web"].as_slice(),
        ["stats", "--no-stream", "--format=json", "--all", "agent-web"].as_slice(),
        ["stats", "--no-stream", "--format=json", "agent-web", "--all"].as_slice(),
        ["stats", "--no-stream", "--format=json", "--format=json", "agent-web"].as_slice(),
        ["stats", "--no-stream", "--format=json"].as_slice(),
    ] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &log)
            .args(args)
            .output()
            .expect("run rejected strict docker stats argv");
        assert!(
            !output.status.success(),
            "non-exact Docker stats argv must fail: {args:?}\n{}",
            output_text(&output)
        );
    }
    assert!(
        !log.exists(),
        "rejected stats argv must not start Apple Container: {}",
        log.display()
    );
}

#[test]
fn docker_stats_preserves_valid_native_json_and_the_child_nonzero_exit() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_stats_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_STATS_MODE", "nonzero")
        .args(["stats", "--no-stream", "--format", "json", "agent-web"])
        .output()
        .expect("run failing strict docker stats shim");

    assert_eq!(output.status.code(), Some(47), "{}", output_text(&output));
    assert_eq!(
        output.stdout,
        b"{\"samples\":[{\"id\":\"agent-web\",\"cpuUsageUsec\":17,\"memoryUsageBytes\":42}]}\n",
        "a nonzero Apple Container stats result must not be wrapped or have its valid JSON suppressed"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("fake Apple Container stats failure"),
        "the bounded native stderr diagnostic should survive the child failure: {}",
        output_text(&output)
    );
}

#[test]
fn docker_stats_suppresses_malformed_child_stdout() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_stats_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_STATS_MODE", "malformed")
        .args(["stats", "--no-stream", "--format", "json", "agent-web"])
        .output()
        .expect("run malformed strict docker stats shim");

    assert_eq!(output.status.code(), Some(1), "{}", output_text(&output));
    assert!(
        output.stdout.is_empty(),
        "malformed Apple stats stdout must not leak onto the agent JSON surface: {}",
        output_text(&output)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("was not one valid JSON document"), "{stderr}");
    assert!(
        !stderr.contains("raw-invalid-stats-marker"),
        "the malformed child stdout must stay suppressed: {stderr}"
    );
}

#[test]
fn docker_stats_suppresses_a_valid_apple_payload_that_exceeds_the_capture_limit() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_stats_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_STATS_MODE", "oversized_valid")
        .args(["stats", "--no-stream", "--format", "json", "agent-web"])
        .output()
        .expect("run oversized strict docker stats shim");

    assert_eq!(output.status.code(), Some(1));
    assert!(
        output.stdout.is_empty(),
        "a valid but oversized Apple payload must be suppressed rather than truncated"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("output exceeded VAT's bounded capture limit"),
        "the cap failure must remain explicit without replaying child stdout"
    );
}

#[test]
fn docker_stats_drains_bounded_stdout_and_stderr_floods_without_replaying_stdout() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_stats_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_STATS_MODE", "flood")
        .args(["stats", "--no-stream", "--format", "json", "agent-web"])
        .output()
        .expect("run flood strict docker stats shim");

    assert_eq!(output.status.code(), Some(1));
    assert!(
        output.stdout.is_empty(),
        "flooded child stdout must never be replayed as an agent JSON result"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("stderr exceeded VAT's bounded capture limit"),
        "a concurrent stderr flood must be bounded rather than deadlock"
    );
    assert!(
        output.stderr.len() < 300 * 1024,
        "captured stderr must stay bounded, got {} bytes",
        output.stderr.len()
    );
}

#[test]
fn docker_ps_json_replays_one_valid_apple_native_document_for_direct_and_list_aliases() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_ps_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");

    for args in [
        ["ps", "--format=json"].as_slice(),
        ["container", "ls", "--all", "--format", "json"].as_slice(),
        ["container", "list", "-a", "--format=json"].as_slice(),
    ] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &log)
            .args(args)
            .output()
            .expect("run strict docker ps JSON shim");
        assert!(
            output.status.success(),
            "args: {args:?}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            output.stdout,
            b"[{\"id\":\"agent-web\",\"state\":\"running\",\"appleExtra\":{\"opaque\":true}}]\n",
            "the agent inventory must remain exact Apple Container JSON, not a VAT or Docker wrapper"
        );
        assert!(
            output.stderr.is_empty(),
            "unexpected Docker ps JSON stderr: {}",
            output_text(&output)
        );
        let parsed: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("validated Apple list JSON");
        assert_eq!(parsed[0]["appleExtra"]["opaque"], serde_json::json!(true));
    }
    assert_eq!(
        fs::read_to_string(&log).expect("read fake list invocations"),
        "list --format json\nlist --format json --all\nlist --format json --all\n",
        "the shim must normalize only the documented JSON inventory aliases to exact Apple Container argv"
    );
}

#[test]
fn docker_ps_json_rejects_templates_filters_quiet_positionals_and_unknown_flags_before_runtime() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_ps_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");

    for args in [
        ["ps", "--format", "table {{.ID}}"].as_slice(),
        ["ps", "--format=json", "--quiet"].as_slice(),
        ["ps", "--format=json", "-q"].as_slice(),
        ["ps", "--filter", "status=running", "--format=json"].as_slice(),
        ["ps", "--format=json", "--all", "-a"].as_slice(),
        ["ps", "--format=json", "agent-web"].as_slice(),
        ["ps", "agent-web", "--format=json"].as_slice(),
        ["ps", "--format=json", "--unknown"].as_slice(),
        ["container", "ls", "--format=json", "--quiet"].as_slice(),
        ["container", "list", "--format=json", "agent-web"].as_slice(),
        ["container", "ps", "--format=json"].as_slice(),
    ] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &log)
            .args(args)
            .output()
            .expect("run rejected strict docker ps JSON argv");
        assert!(
            !output.status.success(),
            "non-exact Docker ps JSON argv must fail: {args:?}\n{}",
            output_text(&output)
        );
    }
    assert!(
        !log.exists(),
        "rejected Docker ps JSON argv must not start Apple Container: {}",
        log.display()
    );
}

#[test]
fn docker_ps_json_preserves_valid_native_json_and_the_child_nonzero_exit() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_ps_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_PS_JSON_MODE", "nonzero")
        .args(["ps", "--all", "--format", "json"])
        .output()
        .expect("run failing strict docker ps JSON shim");

    assert_eq!(output.status.code(), Some(46), "{}", output_text(&output));
    assert_eq!(
        output.stdout,
        b"[{\"id\":\"agent-web\",\"state\":\"running\",\"appleExtra\":{\"opaque\":true}}]\n",
        "a nonzero Apple Container list result must retain its valid native JSON bytes"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("fake Apple Container list failure"),
        "bounded native list stderr should survive the child failure: {}",
        output_text(&output)
    );
}

#[test]
fn docker_ps_json_suppresses_malformed_or_oversized_native_output_without_deadlocking() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_ps_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);

    for mode in ["malformed", "oversized_valid", "flood"] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_PS_JSON_MODE", mode)
            .args(["ps", "--format=json"])
            .output()
            .expect("run malformed or bounded strict docker ps JSON shim");
        assert_eq!(
            output.status.code(),
            Some(1),
            "mode={mode}: {}",
            output_text(&output)
        );
        assert!(
            output.stdout.is_empty(),
            "mode={mode}: invalid or capped Apple list stdout must not be replayed"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        match mode {
            "malformed" => {
                assert!(
                    stderr.contains("was not one valid JSON document"),
                    "{stderr}"
                );
                assert!(
                    !stderr.contains("raw-invalid-list-marker"),
                    "malformed child stdout must stay suppressed: {stderr}"
                );
            }
            "oversized_valid" => {
                assert!(
                    stderr.contains("output exceeded VAT's bounded capture limit"),
                    "{stderr}"
                );
            }
            "flood" => {
                assert!(
                    stderr.contains("stderr exceeded VAT's bounded capture limit"),
                    "{stderr}"
                );
                assert!(
                    output.stderr.len() < 300 * 1024,
                    "captured list stderr must stay bounded, got {} bytes",
                    output.stderr.len()
                );
            }
            _ => unreachable!("test modes are fixed"),
        }
    }
}

#[test]
fn docker_images_json_replays_one_valid_apple_native_document_for_direct_and_image_group_aliases() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_images_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");

    for args in [
        ["images", "--format=json"].as_slice(),
        ["image", "ls", "--format", "json"].as_slice(),
        ["image", "list", "--format=json"].as_slice(),
    ] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &log)
            .args(args)
            .output()
            .expect("run strict docker images JSON shim");
        assert!(
            output.status.success(),
            "args: {args:?}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            output.stdout,
            b"[{\"reference\":\"agent/web:latest\",\"digest\":\"sha256:opaque\",\"appleExtra\":{\"opaque\":true}}]\n",
            "the image inventory must remain exact Apple Container JSON, not a VAT or Docker wrapper"
        );
        assert!(
            output.stderr.is_empty(),
            "unexpected Docker images JSON stderr: {}",
            output_text(&output)
        );
        let parsed: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("validated Apple image list JSON");
        assert_eq!(parsed[0]["appleExtra"]["opaque"], serde_json::json!(true));
    }
    assert_eq!(
        fs::read_to_string(&log).expect("read fake image-list invocations"),
        "image list --format json\nimage list --format json\nimage list --format json\n",
        "the shim must normalize only documented image JSON aliases to exact Apple Container argv"
    );
}

#[test]
fn docker_images_json_rejects_templates_filters_quiet_positionals_and_unknown_flags_before_runtime()
{
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_images_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");

    for args in [
        ["images", "--format", "table {{.Repository}}"].as_slice(),
        ["images", "--format", "yaml"].as_slice(),
        ["images", "--format=toml"].as_slice(),
        ["images", "--format=json", "--quiet"].as_slice(),
        ["images", "--format=json", "-q"].as_slice(),
        ["images", "--filter", "dangling=true", "--format=json"].as_slice(),
        ["images", "--format=json", "demo:latest"].as_slice(),
        ["images", "demo:latest", "--format=json"].as_slice(),
        ["images", "--format=json", "--all"].as_slice(),
        ["images", "--format=json", "--"].as_slice(),
        ["image", "ls", "--format=json", "--no-trunc"].as_slice(),
        ["image", "list", "--format=json", "--unknown"].as_slice(),
        ["image", "images", "--format=json"].as_slice(),
    ] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &log)
            .args(args)
            .output()
            .expect("run rejected strict docker images JSON argv");
        assert!(
            !output.status.success(),
            "non-exact Docker images JSON argv must fail: {args:?}\n{}",
            output_text(&output)
        );
    }
    assert!(
        !log.exists(),
        "rejected Docker images JSON argv must not start Apple Container: {}",
        log.display()
    );
}

#[test]
fn docker_images_json_preserves_valid_native_json_and_the_child_nonzero_exit() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_images_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_IMAGES_JSON_MODE", "nonzero")
        .args(["images", "--format", "json"])
        .output()
        .expect("run failing strict docker images JSON shim");

    assert_eq!(output.status.code(), Some(45), "{}", output_text(&output));
    assert_eq!(
        output.stdout,
        b"[{\"reference\":\"agent/web:latest\",\"digest\":\"sha256:opaque\",\"appleExtra\":{\"opaque\":true}}]\n",
        "a nonzero Apple Container image-list result must retain its valid native JSON bytes"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("fake Apple Container image list failure"),
        "bounded native image-list stderr should survive the child failure: {}",
        output_text(&output)
    );
}

#[test]
fn docker_images_json_suppresses_malformed_or_oversized_native_output_without_deadlocking() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_images_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);

    for mode in ["malformed", "oversized_valid", "flood"] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_IMAGES_JSON_MODE", mode)
            .args(["images", "--format=json"])
            .output()
            .expect("run malformed or bounded strict docker images JSON shim");
        assert_eq!(
            output.status.code(),
            Some(1),
            "mode={mode}: {}",
            output_text(&output)
        );
        assert!(
            output.stdout.is_empty(),
            "mode={mode}: invalid or capped Apple image-list stdout must not be replayed"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        match mode {
            "malformed" => {
                assert!(
                    stderr.contains("was not one valid JSON document"),
                    "{stderr}"
                );
                assert!(
                    !stderr.contains("raw-invalid-image-list-marker"),
                    "malformed child stdout must stay suppressed: {stderr}"
                );
            }
            "oversized_valid" => {
                assert!(
                    stderr.contains("output exceeded VAT's bounded capture limit"),
                    "{stderr}"
                );
            }
            "flood" => {
                assert!(
                    stderr.contains("stderr exceeded VAT's bounded capture limit"),
                    "{stderr}"
                );
                assert!(
                    output.stderr.len() < 300 * 1024,
                    "captured image-list stderr must stay bounded, got {} bytes",
                    output.stderr.len()
                );
            }
            _ => unreachable!("test modes are fixed"),
        }
    }
}

#[test]
fn docker_inspect_json_replays_one_valid_apple_native_document_for_direct_and_container_aliases() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_inspect_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");

    for args in [
        ["inspect", "--format=json", "agent-web"].as_slice(),
        ["container", "inspect", "--format", "json", "agent-web"].as_slice(),
    ] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &log)
            .args(args)
            .output()
            .expect("run strict docker inspect JSON shim");
        assert!(
            output.status.success(),
            "args: {args:?}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            output.stdout,
            b"[{\"id\":\"agent-web\",\"state\":\"running\",\"appleExtra\":{\"opaque\":true}}]\n",
            "the direct inspect result must remain exact Apple Container JSON, not a VAT or Docker wrapper"
        );
        assert!(
            output.stderr.is_empty(),
            "unexpected Docker inspect JSON stderr: {}",
            output_text(&output)
        );
        let parsed: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("validated Apple inspect JSON");
        assert_eq!(parsed[0]["appleExtra"]["opaque"], serde_json::json!(true));
    }
    assert_eq!(
        fs::read_to_string(&log).expect("read fake inspect invocations"),
        "inspect agent-web\ninspect agent-web\n",
        "the shim must strip its Docker selector and normalize only documented inspect aliases to exact Apple Container argv"
    );
}

#[test]
fn docker_inspect_json_rejects_object_selectors_templates_and_nonexact_args_before_runtime() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_inspect_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");

    for args in [
        ["inspect", "--format", "table {{.Id}}", "agent-web"].as_slice(),
        ["inspect", "--format", "yaml", "agent-web"].as_slice(),
        ["inspect", "--format=toml", "agent-web"].as_slice(),
        ["inspect", "--format={{.Id}}", "agent-web"].as_slice(),
        [
            "inspect",
            "--type",
            "container",
            "--format=json",
            "agent-web",
        ]
        .as_slice(),
        ["inspect", "--size", "--format=json", "agent-web"].as_slice(),
        [
            "inspect",
            "--filter",
            "name=agent-web",
            "--format=json",
            "agent-web",
        ]
        .as_slice(),
        ["inspect", "--format=json", "agent-web", "agent-db"].as_slice(),
        ["inspect", "agent-web", "--format=json"].as_slice(),
        ["inspect", "--format=json", "--"].as_slice(),
        ["inspect", "--format=json", "-agent-web"].as_slice(),
        ["inspect", "--format=json", "agent/web"].as_slice(),
        ["inspect", "--format=json"].as_slice(),
        [
            "container",
            "inspect",
            "--format=json",
            "--unknown",
            "agent-web",
        ]
        .as_slice(),
        [
            "container",
            "inspect",
            "--format=json",
            "agent-web",
            "--format=json",
        ]
        .as_slice(),
        ["image", "inspect", "--format=json", "agent-web"].as_slice(),
    ] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &log)
            .args(args)
            .output()
            .expect("run rejected strict docker inspect JSON argv");
        assert!(
            !output.status.success(),
            "non-exact Docker inspect JSON argv must fail: {args:?}\n{}",
            output_text(&output)
        );
    }
    assert!(
        !log.exists(),
        "rejected Docker inspect JSON argv must not start Apple Container: {}",
        log.display()
    );
}

#[test]
fn docker_inspect_json_preserves_valid_native_json_and_the_child_nonzero_exit() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_inspect_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_INSPECT_JSON_MODE", "nonzero")
        .args(["inspect", "--format", "json", "agent-web"])
        .output()
        .expect("run failing strict docker inspect JSON shim");

    assert_eq!(output.status.code(), Some(44), "{}", output_text(&output));
    assert_eq!(
        output.stdout,
        b"[{\"id\":\"agent-web\",\"state\":\"running\",\"appleExtra\":{\"opaque\":true}}]\n",
        "a nonzero Apple Container inspect result must retain its valid native JSON bytes"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("fake Apple Container inspect failure"),
        "bounded native inspect stderr should survive the child failure: {}",
        output_text(&output)
    );
}

#[test]
fn docker_inspect_json_suppresses_malformed_or_oversized_native_output_without_deadlocking() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_inspect_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);

    for mode in ["malformed", "oversized_valid", "flood"] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_INSPECT_JSON_MODE", mode)
            .args(["inspect", "--format=json", "agent-web"])
            .output()
            .expect("run malformed or bounded strict docker inspect JSON shim");
        assert_eq!(
            output.status.code(),
            Some(1),
            "mode={mode}: {}",
            output_text(&output)
        );
        assert!(
            output.stdout.is_empty(),
            "mode={mode}: invalid or capped Apple inspect stdout must not be replayed"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        match mode {
            "malformed" => {
                assert!(
                    stderr.contains("was not one valid JSON document"),
                    "{stderr}"
                );
                assert!(
                    !stderr.contains("raw-invalid-inspect-marker"),
                    "malformed child stdout must stay suppressed: {stderr}"
                );
            }
            "oversized_valid" => {
                assert!(
                    stderr.contains("output exceeded VAT's bounded capture limit"),
                    "{stderr}"
                );
            }
            "flood" => {
                assert!(
                    stderr.contains("stderr exceeded VAT's bounded capture limit"),
                    "{stderr}"
                );
                assert!(
                    output.stderr.len() < 300 * 1024,
                    "captured inspect stderr must stay bounded, got {} bytes",
                    output.stderr.len()
                );
            }
            _ => unreachable!("test modes are fixed"),
        }
    }
}

#[test]
fn docker_image_inspect_json_replays_one_valid_apple_native_document() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_image_inspect_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &log)
        .args(["image", "inspect", "--format=json", "alpine:3.20"])
        .output()
        .expect("run strict docker image inspect JSON shim");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.stdout,
        b"[{\"configuration\":{\"name\":\"docker.io/library/alpine:3.20\"},\"descriptor\":{\"digest\":\"sha256:opaque\"},\"appleExtra\":{\"opaque\":true}}]\n",
        "the direct image inspect result must remain exact Apple Container JSON, not a VAT or Docker wrapper"
    );
    assert!(
        output.stderr.is_empty(),
        "unexpected Docker image inspect JSON stderr: {}",
        output_text(&output)
    );
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("validated Apple image inspect JSON");
    assert_eq!(parsed[0]["appleExtra"]["opaque"], serde_json::json!(true));
    assert_eq!(
        fs::read_to_string(&log).expect("read fake image inspect invocation"),
        "image inspect alpine:3.20\n",
        "the shim must strip its Docker selector and invoke only exact Apple Container image inspect argv"
    );
}

#[test]
fn docker_image_inspect_json_rejects_templates_options_extra_refs_misordering_and_separator_before_runtime(
) {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_image_inspect_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");

    for args in [
        [
            "image",
            "inspect",
            "--format",
            "table {{.Id}}",
            "alpine:3.20",
        ]
        .as_slice(),
        ["image", "inspect", "--format", "yaml", "alpine:3.20"].as_slice(),
        ["image", "inspect", "--format=toml", "alpine:3.20"].as_slice(),
        ["image", "inspect", "--format={{.Id}}", "alpine:3.20"].as_slice(),
        [
            "image",
            "inspect",
            "--platform",
            "linux/arm64",
            "--format=json",
            "alpine:3.20",
        ]
        .as_slice(),
        [
            "image",
            "inspect",
            "--format=json",
            "alpine:3.20",
            "busybox:latest",
        ]
        .as_slice(),
        ["image", "inspect", "alpine:3.20", "--format=json"].as_slice(),
        ["image", "inspect", "--format=json", "--", "alpine:3.20"].as_slice(),
        ["image", "inspect", "--format=json", "alpine:3.20", "--"].as_slice(),
        ["image", "inspect", "--format=json", "-alpine:3.20"].as_slice(),
        ["image", "inspect", "--format=json"].as_slice(),
        [
            "image",
            "inspect",
            "--format=json",
            "alpine:3.20",
            "--format=json",
        ]
        .as_slice(),
        [
            "image",
            "inspect",
            "--format=json",
            "--unknown",
            "alpine:3.20",
        ]
        .as_slice(),
    ] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &log)
            .args(args)
            .output()
            .expect("run rejected strict docker image inspect JSON argv");
        assert!(
            !output.status.success(),
            "non-exact Docker image inspect JSON argv must fail: {args:?}\n{}",
            output_text(&output)
        );
    }
    assert!(
        !log.exists(),
        "rejected Docker image inspect JSON argv must not start Apple Container: {}",
        log.display()
    );
}

#[test]
fn docker_image_inspect_json_preserves_valid_native_json_and_the_child_nonzero_exit() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_image_inspect_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_IMAGE_INSPECT_JSON_MODE", "nonzero")
        .args(["image", "inspect", "--format", "json", "alpine:3.20"])
        .output()
        .expect("run failing strict docker image inspect JSON shim");

    assert_eq!(output.status.code(), Some(42), "{}", output_text(&output));
    assert_eq!(
        output.stdout,
        b"[{\"configuration\":{\"name\":\"docker.io/library/alpine:3.20\"},\"descriptor\":{\"digest\":\"sha256:opaque\"},\"appleExtra\":{\"opaque\":true}}]\n",
        "a nonzero Apple Container image inspect result must retain its valid native JSON bytes"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("fake Apple Container image inspect failure"),
        "bounded native image inspect stderr should survive the child failure: {}",
        output_text(&output)
    );
}

#[test]
fn docker_image_inspect_json_suppresses_malformed_or_oversized_native_output_without_deadlocking() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_image_inspect_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);

    for mode in ["malformed", "oversized_valid", "flood"] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_IMAGE_INSPECT_JSON_MODE", mode)
            .args(["image", "inspect", "--format=json", "alpine:3.20"])
            .output()
            .expect("run malformed or bounded strict docker image inspect JSON shim");
        assert_eq!(
            output.status.code(),
            Some(1),
            "mode={mode}: {}",
            output_text(&output)
        );
        assert!(
            output.stdout.is_empty(),
            "mode={mode}: invalid or capped Apple image inspect stdout must not be replayed"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        match mode {
            "malformed" => {
                assert!(
                    stderr.contains("was not one valid JSON document"),
                    "{stderr}"
                );
                assert!(
                    !stderr.contains("raw-invalid-image-inspect-marker"),
                    "malformed child stdout must stay suppressed: {stderr}"
                );
            }
            "oversized_valid" => {
                assert!(
                    stderr.contains("output exceeded VAT's bounded capture limit"),
                    "{stderr}"
                );
            }
            "flood" => {
                assert!(
                    stderr.contains("stderr exceeded VAT's bounded capture limit"),
                    "{stderr}"
                );
                assert!(
                    output.stderr.len() < 300 * 1024,
                    "captured image inspect stderr must stay bounded, got {} bytes",
                    output.stderr.len()
                );
            }
            _ => unreachable!("test modes are fixed"),
        }
    }
}

#[test]
fn docker_logs_json_wraps_one_bounded_apple_stdio_snapshot_for_direct_and_container_aliases() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_logs_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");

    for args in [
        ["logs", "--format=json", "--tail=17", "agent-web"].as_slice(),
        [
            "container",
            "logs",
            "--tail",
            "17",
            "--format",
            "json",
            "agent-web",
        ]
        .as_slice(),
    ] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &log)
            .args(args)
            .output()
            .expect("run strict docker logs JSON shim");
        assert!(
            output.status.success(),
            "args: {args:?}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            output.stderr.is_empty(),
            "bounded child diagnostics must live in the one JSON wrapper: {}",
            output_text(&output)
        );
        let stdout = String::from_utf8(output.stdout).expect("wrapper stdout is UTF-8 JSON");
        assert!(
            stdout.contains("\\u0001"),
            "control bytes must be escaped in the JSON wrapper: {stdout}"
        );
        let result: serde_json::Value =
            serde_json::from_str(&stdout).expect("one valid direct logs VAT JSON document");
        assert_eq!(result["schema"], "vat.docker.logs.v1");
        assert_eq!(result["format"], "vat_json");
        assert_eq!(result["backend"], "apple-container");
        assert_eq!(result["container"], "agent-web");
        assert_eq!(result["requested_tail_lines"], 17);
        assert_eq!(result["source"], "apple-container-stdio");
        assert_eq!(result["outcome"], "observed");
        assert_eq!(result["child_exit_code"], 0);
        assert_eq!(result["runtime_invoked"], true);
        assert_eq!(result["secret_redaction_guaranteed"], false);
        assert_eq!(result["untrusted_log_content"], true);
        assert_eq!(
            result["next"], "docker inspect --format json agent-web",
            "the wrapper next command must use the supported strict inspect form"
        );
        let stdio = result["apple_container_stdio"]
            .as_str()
            .expect("Apple stdio payload string");
        assert!(stdio.contains("line-one"), "{stdio:?}");
        assert!(stdio.contains("control:\u{1}"), "{stdio:?}");
        assert!(stdio.contains("invalid:\u{FFFD}"), "{stdio:?}");
        assert_eq!(result["apple_container_stdio_utf8_lossy"], true);
        assert_eq!(result["apple_container_stdio_truncated"], false);
        let diagnostics = result["diagnostic_stderr"]
            .as_str()
            .expect("bounded diagnostic string");
        assert!(
            diagnostics.contains("diagnostic-control:\u{1}"),
            "{diagnostics:?}"
        );
        assert_eq!(result["diagnostic_stderr_utf8_lossy"], true);
        assert_eq!(result["diagnostic_stderr_truncated"], false);
        assert!(
            result.get("stdout").is_none() && result.get("stderr").is_none(),
            "direct Apple logs must not claim Docker stdout/stderr demultiplexing: {result}"
        );
    }

    assert_eq!(
        fs::read_to_string(&log).expect("read fake logs invocations"),
        "logs -n 17 agent-web\nlogs -n 17 agent-web\n",
        "the selector must be stripped and both aliases normalized to exact Apple logs argv"
    );
}

#[test]
fn docker_logs_json_rejects_streaming_boot_templates_and_nonexact_args_before_runtime() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_logs_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");

    for args in [
        [
            "logs",
            "--format",
            "table {{.ID}}",
            "--tail",
            "17",
            "agent-web",
        ]
        .as_slice(),
        ["logs", "--format=yaml", "--tail=17", "agent-web"].as_slice(),
        ["logs", "--format=toml", "--tail=17", "agent-web"].as_slice(),
        ["logs", "--format=json", "--tail=all", "agent-web"].as_slice(),
        ["logs", "--format=json", "--tail=0", "agent-web"].as_slice(),
        ["logs", "--format=json", "--tail=1001", "agent-web"].as_slice(),
        ["logs", "--format=json", "--tail=+17", "agent-web"].as_slice(),
        [
            "logs",
            "--format=json",
            "--tail=17",
            "--follow",
            "agent-web",
        ]
        .as_slice(),
        ["logs", "-f", "--format=json", "--tail=17", "agent-web"].as_slice(),
        ["logs", "--boot", "--format=json", "--tail=17", "agent-web"].as_slice(),
        [
            "logs",
            "--format=json",
            "--tail=17",
            "--timestamps",
            "agent-web",
        ]
        .as_slice(),
        [
            "logs",
            "--format=json",
            "--tail=17",
            "--details",
            "agent-web",
        ]
        .as_slice(),
        [
            "logs",
            "--format=json",
            "--tail=17",
            "--since",
            "now",
            "agent-web",
        ]
        .as_slice(),
        [
            "logs",
            "--format=json",
            "--tail=17",
            "--until",
            "now",
            "agent-web",
        ]
        .as_slice(),
        ["logs", "--format=json", "-n", "17", "agent-web"].as_slice(),
        [
            "logs",
            "--format=json",
            "--format=json",
            "--tail=17",
            "agent-web",
        ]
        .as_slice(),
        [
            "logs",
            "--format=json",
            "--tail=17",
            "--tail=18",
            "agent-web",
        ]
        .as_slice(),
        ["logs", "agent-web", "--format=json", "--tail=17"].as_slice(),
        [
            "logs",
            "--format=json",
            "--tail=17",
            "agent-web",
            "agent-db",
        ]
        .as_slice(),
        [
            "logs",
            "--format=json",
            "--tail=17",
            "agent-web",
            "--follow",
        ]
        .as_slice(),
        ["logs", "--format=json", "--tail=17", "--", "agent-web"].as_slice(),
        ["logs", "--format=json", "--tail=17", "agent/web"].as_slice(),
        ["logs", "--format=json", "--tail=17", "-agent-web"].as_slice(),
        [
            "container",
            "logs",
            "--format=json",
            "--tail=17",
            "--unknown",
            "agent-web",
        ]
        .as_slice(),
    ] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &log)
            .args(args)
            .output()
            .expect("run rejected strict docker logs JSON argv");
        assert!(
            !output.status.success(),
            "non-exact Docker logs JSON argv must fail: {args:?}\n{}",
            output_text(&output)
        );
    }
    assert!(
        !log.exists(),
        "rejected Docker logs JSON argv must not start Apple Container: {}",
        log.display()
    );
}

#[test]
fn docker_logs_json_wraps_ordinary_child_failure_and_preserves_its_exit_code() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_logs_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_LOGS_JSON_MODE", "nonzero")
        .args(["logs", "--format", "json", "--tail", "17", "agent-web"])
        .output()
        .expect("run failing strict docker logs JSON shim");

    assert_eq!(output.status.code(), Some(43), "{}", output_text(&output));
    assert!(
        output.stderr.is_empty(),
        "ordinary backend diagnostics must remain in the wrapper: {}",
        output_text(&output)
    );
    let result: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one nonzero logs wrapper");
    assert_eq!(result["outcome"], "failed");
    assert_eq!(result["child_exit_code"], 43);
    assert_eq!(result["apple_container_stdio"], "logs-before-failure\n");
    assert!(
        result["diagnostic_stderr"]
            .as_str()
            .is_some_and(|value| value.contains("fake Apple Container logs failure")),
        "backend diagnostic must be bounded inside the wrapper: {result}"
    );
}

#[test]
fn docker_logs_json_bounds_dual_stream_floods_and_fails_closed_on_timeout() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_logs_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);

    let flood = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_LOGS_JSON_MODE", "flood")
        .args(["logs", "--format=json", "--tail=17", "agent-web"])
        .output()
        .expect("run bounded direct logs flood");
    assert!(flood.status.success(), "{}", output_text(&flood));
    assert!(
        flood.stderr.is_empty(),
        "flood diagnostics must remain inside the wrapper: {}",
        output_text(&flood)
    );
    assert!(
        flood.stdout.len() < 160 * 1024,
        "the total JSON wrapper must stay bounded, got {} bytes",
        flood.stdout.len()
    );
    let flood_result: serde_json::Value =
        serde_json::from_slice(&flood.stdout).expect("bounded flood wrapper JSON");
    for field in ["apple_container_stdio", "diagnostic_stderr"] {
        let text = flood_result[field].as_str().expect("bounded stream string");
        assert!(
            serde_json::to_vec(text)
                .expect("serialize bounded stream")
                .len()
                <= 64 * 1024,
            "{field} must respect the serialized JSON string cap"
        );
    }
    assert_eq!(flood_result["apple_container_stdio_truncated"], true);
    assert_eq!(flood_result["diagnostic_stderr_truncated"], true);

    let started = Instant::now();
    let timeout = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_LOGS_JSON_MODE", "timeout")
        .args(["logs", "--format=json", "--tail=17", "agent-web"])
        .output()
        .expect("run timed-out direct logs snapshot");
    assert_eq!(timeout.status.code(), Some(1), "{}", output_text(&timeout));
    assert!(
        timeout.stdout.is_empty(),
        "a timeout must fail closed without a partial logs wrapper: {}",
        output_text(&timeout)
    );
    let stderr = String::from_utf8_lossy(&timeout.stderr);
    assert!(
        stderr.contains("Apple Container logs observation timed out"),
        "timeout must retain the direct logs operation label: {stderr}"
    );
    assert!(
        !stderr.contains("raw-timeout-log-must-not-be-wrapped"),
        "raw child logs must not leak after timeout: {stderr}"
    );
    assert!(
        started.elapsed() < Duration::from_secs(8),
        "the direct logs deadline and bounded cleanup must prevent a long wait: {:?}",
        started.elapsed()
    );
}

#[test]
fn docker_exec_json_wraps_one_bounded_command_snapshot_for_direct_and_container_aliases() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_exec_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");

    for args in [
        [
            "exec",
            "--format=json",
            "--timeout=2",
            "agent-web",
            "--",
            "fixture-command",
            "--literal",
        ]
        .as_slice(),
        [
            "container",
            "exec",
            "--timeout",
            "2",
            "--format",
            "json",
            "agent-web",
            "--",
            "fixture-command",
            "--literal",
        ]
        .as_slice(),
    ] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &log)
            .args(args)
            .output()
            .expect("run strict docker exec JSON shim");
        assert!(
            output.status.success(),
            "args: {args:?}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            output.stderr.is_empty(),
            "bounded child diagnostics must live in the one JSON wrapper: {}",
            output_text(&output)
        );
        let stdout = String::from_utf8(output.stdout).expect("wrapper stdout is UTF-8 JSON");
        assert!(
            stdout.contains("\\u0001"),
            "control bytes must be escaped in the JSON wrapper: {stdout}"
        );
        let result: serde_json::Value =
            serde_json::from_str(&stdout).expect("one valid direct exec VAT JSON document");
        assert_eq!(result["schema"], "vat.docker.exec.v1");
        assert_eq!(result["format"], "vat_json");
        assert_eq!(result["backend"], "apple-container");
        assert_eq!(result["container"], "agent-web");
        assert_eq!(result["requested_timeout_seconds"], 2);
        assert_eq!(
            result["timeout_scope"],
            "host-container-client-observation"
        );
        assert_eq!(result["source"], "apple-container-exec");
        assert_eq!(result["outcome"], "completed");
        assert_eq!(result["child_exit_code"], 0);
        assert_eq!(result["runtime_invoked"], true);
        assert_eq!(result["secret_redaction_guaranteed"], false);
        assert_eq!(result["untrusted_command_output"], true);
        assert_eq!(
            result["next"], "docker inspect --format json agent-web",
            "the wrapper next command must use the supported strict inspect form"
        );
        let stdout = result["stdout"].as_str().expect("bounded stdout string");
        assert!(stdout.contains("exec-stdout"), "{stdout:?}");
        assert!(stdout.contains("control:\u{1}"), "{stdout:?}");
        assert!(stdout.contains("invalid:\u{FFFD}"), "{stdout:?}");
        assert_eq!(result["stdout_utf8_lossy"], true);
        assert_eq!(result["stdout_truncated"], false);
        let stderr = result["stderr"].as_str().expect("bounded stderr string");
        assert!(stderr.contains("exec-stderr-control:\u{1}"), "{stderr:?}");
        assert_eq!(result["stderr_utf8_lossy"], true);
        assert_eq!(result["stderr_truncated"], false);
    }

    assert_eq!(
        fs::read_to_string(&log).expect("read fake exec invocations"),
        "exec agent-web fixture-command --literal\nexec agent-web fixture-command --literal\n",
        "the selector must be stripped, both aliases normalized, and the Docker delimiter stripped before canonical Apple argv"
    );
}

#[test]
fn docker_exec_json_rejects_nonexact_args_before_runtime_and_keeps_raw_commands_raw() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_exec_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");

    for args in [
        [
            "exec",
            "--format=json",
            "--timeout=0",
            "agent-web",
            "--",
            "fixture-command",
            "--literal",
        ]
        .as_slice(),
        [
            "exec",
            "--format=json",
            "--timeout=1201",
            "agent-web",
            "--",
            "fixture-command",
            "--literal",
        ]
        .as_slice(),
        [
            "exec",
            "--format=table",
            "--timeout=2",
            "agent-web",
            "--",
            "fixture-command",
            "--literal",
        ]
        .as_slice(),
        [
            "exec",
            "--format=json",
            "--timeout=2",
            "agent-web",
            "fixture-command",
        ]
        .as_slice(),
        [
            "exec",
            "--format=json",
            "--timeout=2",
            "agent-web",
            "--",
        ]
        .as_slice(),
        [
            "exec",
            "-it",
            "--format=json",
            "--timeout=2",
            "agent-web",
            "--",
            "fixture-command",
        ]
        .as_slice(),
        [
            "exec",
            "--env",
            "A=B",
            "--format=json",
            "--timeout=2",
            "agent-web",
            "--",
            "fixture-command",
        ]
        .as_slice(),
        [
            "container",
            "exec",
            "--timeout",
            "2",
            "--format",
            "json",
            "--workdir",
            "/tmp",
            "agent-web",
            "--",
            "fixture-command",
        ]
        .as_slice(),
    ] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &log)
            .args(args)
            .output()
            .expect("run rejected strict docker exec JSON argv");
        assert!(
            !output.status.success(),
            "non-exact Docker exec JSON argv must fail: {args:?}\n{}",
            output_text(&output)
        );
    }
    assert!(
        !log.exists(),
        "rejected Docker exec JSON argv must not start Apple Container: {}",
        log.display()
    );

    let raw_bin = root.path().join("raw-bin");
    write_fake_container(&raw_bin);
    let raw_log = root.path().join("raw-container.log");
    let raw = Command::new(&shim)
        .env("PATH", path_with_prepend(&raw_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &raw_log)
        .args([
            "exec",
            "agent-web",
            "--",
            "fixture-command",
            "--format",
            "json",
        ])
        .output()
        .expect("run raw exec command containing format-looking command args");
    assert!(
        raw.status.success(),
        "a raw exec command after its literal separator must not be intercepted: {}",
        output_text(&raw)
    );
    assert!(
        raw.stdout.is_empty() && raw.stderr.is_empty(),
        "raw exec must keep inherited child stdio rather than emit a VAT JSON wrapper: {}",
        output_text(&raw)
    );
    assert_eq!(
        fs::read_to_string(&raw_log).expect("read raw exec invocation"),
        "exec agent-web -- fixture-command --format json\n",
        "format-looking arguments after the literal raw-command separator must preserve the generic raw exec argv"
    );
}

#[test]
fn docker_exec_json_wraps_ordinary_child_failure_and_preserves_its_exit_code() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_exec_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_EXEC_JSON_MODE", "nonzero")
        .args([
            "exec",
            "--format",
            "json",
            "--timeout",
            "2",
            "agent-web",
            "--",
            "fixture-command",
            "--literal",
        ])
        .output()
        .expect("run failing strict docker exec JSON shim");

    assert_eq!(output.status.code(), Some(42), "{}", output_text(&output));
    assert!(
        output.stderr.is_empty(),
        "ordinary backend diagnostics must remain in the wrapper: {}",
        output_text(&output)
    );
    let result: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one nonzero exec wrapper");
    assert_eq!(result["schema"], "vat.docker.exec.v1");
    assert_eq!(result["outcome"], "failed");
    assert_eq!(result["child_exit_code"], 42);
    assert_eq!(result["stdout"], "exec-before-failure\n");
    assert!(
        result["stderr"]
            .as_str()
            .is_some_and(|value| value.contains("fake Apple Container exec failure")),
        "backend diagnostic must be bounded inside the wrapper: {result}"
    );
}

#[test]
fn docker_exec_json_bounds_dual_stream_floods_and_fails_closed_on_timeout() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_exec_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);

    let flood = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_EXEC_JSON_MODE", "flood")
        .args([
            "exec",
            "--format=json",
            "--timeout=2",
            "agent-web",
            "--",
            "fixture-command",
            "--literal",
        ])
        .output()
        .expect("run bounded direct exec flood");
    assert!(flood.status.success(), "{}", output_text(&flood));
    assert!(
        flood.stderr.is_empty(),
        "flood diagnostics must remain inside the wrapper: {}",
        output_text(&flood)
    );
    assert!(
        flood.stdout.len() < 160 * 1024,
        "the total JSON wrapper must stay bounded, got {} bytes",
        flood.stdout.len()
    );
    let flood_result: serde_json::Value =
        serde_json::from_slice(&flood.stdout).expect("bounded flood wrapper JSON");
    for field in ["stdout", "stderr"] {
        let text = flood_result[field].as_str().expect("bounded stream string");
        assert!(
            serde_json::to_vec(text)
                .expect("serialize bounded stream")
                .len()
                <= 64 * 1024,
            "{field} must respect the serialized JSON string cap"
        );
    }
    assert_eq!(flood_result["stdout_truncated"], true);
    assert_eq!(flood_result["stderr_truncated"], true);

    let started = Instant::now();
    let timeout = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_EXEC_JSON_MODE", "timeout")
        .args([
            "exec",
            "--format=json",
            "--timeout=1",
            "agent-web",
            "--",
            "fixture-command",
            "--literal",
        ])
        .output()
        .expect("run timed-out direct exec snapshot");
    assert_eq!(timeout.status.code(), Some(1), "{}", output_text(&timeout));
    assert!(
        timeout.stdout.is_empty(),
        "a client-observation timeout must fail closed without a partial exec wrapper: {}",
        output_text(&timeout)
    );
    let stderr = String::from_utf8_lossy(&timeout.stderr);
    assert!(
        stderr.contains("Apple Container exec client observation timed out"),
        "timeout must retain the host client operation label: {stderr}"
    );
    assert!(
        !stderr.contains("raw-timeout-exec-must-not-be-wrapped"),
        "raw child output must not leak after timeout: {stderr}"
    );
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "the direct exec client deadline and bounded cleanup must prevent a long wait: {:?}",
        started.elapsed()
    );
}

#[test]
fn docker_pull_json_emits_one_bounded_nonowning_image_receipt() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_pull_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &log)
        .args(strict_pull_json_args(2))
        .output()
        .expect("run strict Docker pull JSON receipt");

    assert!(output.status.success(), "{}", output_text(&output));
    assert!(
        output.stderr.is_empty(),
        "bounded pull diagnostics must stay in one receipt: {}",
        output_text(&output)
    );
    let stdout = String::from_utf8(output.stdout).expect("receipt stdout is UTF-8 JSON");
    assert!(
        stdout.contains("\\u0001"),
        "control bytes must be escaped in the JSON receipt: {stdout}"
    );
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("one strict Docker pull VAT JSON receipt");
    assert_eq!(result["schema"], "vat.docker.pull.v1");
    assert_eq!(result["format"], "vat_json");
    assert_eq!(result["type"], "vat_docker_pull");
    assert_eq!(result["command"], "pull");
    assert_eq!(result["backend"], "apple-container");
    assert_eq!(result["image"], "fixture/image:latest");
    assert_eq!(result["requested_timeout_seconds"], 2);
    assert_eq!(result["timeout_scope"], "host-container-client-observation");
    assert_eq!(result["source"], "apple-container-image-pull");
    assert_eq!(result["outcome"], "completed");
    assert_eq!(result["child_exit_code"], 0);
    assert_eq!(result["runtime_invoked"], true);
    assert_eq!(result["image_lifecycle"], "not_owned_no_auto_cleanup");
    assert_eq!(result["cleanup_attempted"], false);
    assert_eq!(result["registry_management_implemented"], false);
    assert_eq!(result["provenance_verified"], false);
    assert_eq!(result["digest_verified"], false);
    assert_eq!(result["platform_verified"], false);
    assert_eq!(result["freshness_verified"], false);
    assert_eq!(result["image_state_verified"], false);
    assert_eq!(result["ownership_verified"], false);
    assert_eq!(result["security_verified"], false);
    assert_eq!(result["secret_redaction_guaranteed"], false);
    assert_eq!(result["cancellation_guaranteed"], false);
    assert_eq!(result["download_completion_guaranteed"], false);
    assert_eq!(result["rollback_guaranteed"], false);
    assert_eq!(result["untrusted_pull_output"], true);
    assert!(
        result["stdout"]
            .as_str()
            .is_some_and(|value| value.contains("pull-stdout") && value.contains("invalid:\u{FFFD}")),
        "bounded stdout must retain fake child content: {result}"
    );
    assert_eq!(result["stdout_utf8_lossy"], true);
    assert_eq!(result["stderr_utf8_lossy"], true);
    assert_eq!(
        result["next"],
        "docker image inspect --format json 'fixture/image:latest'"
    );
    assert!(
        result.get("terminal").is_none(),
        "successful pull receipt hands off to exact image inspect rather than declaring terminal completion"
    );
    assert!(
        !result.to_string().contains("--timeout"),
        "the VAT-only deadline selector must not be copied into the receipt"
    );

    assert_eq!(
        fs::read_to_string(&log).expect("read fake image pull invocation"),
        "image pull fixture/image:latest\n",
        "the strict receipt must strip only its Docker-facing selectors and use the public Apple image pull argv"
    );
}

#[test]
fn docker_pull_json_preserves_raw_direct_and_image_group_pull_paths() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_pull_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");

    for args in [
        ["pull", "fixture/image:latest"].as_slice(),
        ["image", "pull", "fixture/image:latest"].as_slice(),
    ] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &log)
            .args(args)
            .output()
            .expect("run pre-existing raw Docker pull path");
        assert!(output.status.success(), "args={args:?}: {}", output_text(&output));
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("pull-stdout"),
            "raw pull stdout must retain child stdio rather than receive the strict VAT receipt: {}",
            output_text(&output)
        );
        assert!(
            !String::from_utf8_lossy(&output.stdout).contains("vat.docker.pull.v1"),
            "raw pull must remain outside the strict receipt surface: {}",
            output_text(&output)
        );
    }
    assert_eq!(
        fs::read_to_string(&log).expect("read raw pull invocations"),
        "image pull fixture/image:latest\nimage pull fixture/image:latest\n",
        "unformatted direct and image-group pulls must preserve their generic Apple translation"
    );
}

#[test]
fn docker_pull_json_preserves_a_normal_child_nonzero_receipt_without_image_handoff() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_pull_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_PULL_JSON_MODE", "nonzero")
        .args(strict_pull_json_args(2))
        .output()
        .expect("run failing strict Docker pull JSON receipt");

    assert_eq!(output.status.code(), Some(41), "{}", output_text(&output));
    assert!(output.stderr.is_empty(), "{}", output_text(&output));
    let result: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one nonzero pull receipt");
    assert_eq!(result["schema"], "vat.docker.pull.v1");
    assert_eq!(result["outcome"], "failed");
    assert_eq!(result["child_exit_code"], 41);
    assert_eq!(result["terminal"], "pull_failed");
    assert_eq!(result["next"], "docker --help");
    assert_eq!(result["stdout"], "pull-before-failure\n");
    assert!(
        result["stderr"]
            .as_str()
            .is_some_and(|value| value.contains("fake Apple Container image pull failure")),
        "bounded stderr must retain failure diagnostic: {result}"
    );
    assert!(
        !result["next"]
            .as_str()
            .is_some_and(|next| next.contains("fixture/image:latest")),
        "failed pull must not hand off to an image whose transfer/state is unverified: {result}"
    );
}

#[test]
fn docker_pull_json_retains_lossy_and_bounded_text_receipts_without_deadlocking() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_pull_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);

    for mode in ["malformed", "oversized", "flood"] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_PULL_JSON_MODE", mode)
            .args(strict_pull_json_args(2))
            .output()
            .expect("run lossy or bounded strict Docker pull receipt");
        assert!(
            output.status.success(),
            "mode={mode}: {}",
            output_text(&output)
        );
        assert!(
            output.stderr.is_empty(),
            "mode={mode}: child diagnostics must stay inside the receipt: {}",
            output_text(&output)
        );
        assert!(
            output.stdout.len() < 160 * 1024,
            "mode={mode}: bounded pull receipt must remain finite, got {} bytes",
            output.stdout.len()
        );
        let result: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("bounded pull receipt JSON");
        assert_eq!(result["schema"], "vat.docker.pull.v1");
        assert_eq!(result["outcome"], "completed");
        match mode {
            "malformed" => {
                assert_eq!(result["stdout_utf8_lossy"], true);
                assert_eq!(result["stderr_utf8_lossy"], true);
                assert!(
                    result["stdout"]
                        .as_str()
                        .is_some_and(|stdout| stdout.contains("invalid:\u{FFFD}")),
                    "malformed text must be represented lossily inside receipt: {result}"
                );
            }
            "oversized" => {
                assert_eq!(result["stdout_truncated"], true);
                assert_eq!(result["stderr_truncated"], false);
            }
            "flood" => {
                assert_eq!(result["stdout_truncated"], true);
                assert_eq!(result["stderr_truncated"], true);
            }
            _ => unreachable!("test modes are fixed"),
        }
        for field in ["stdout", "stderr"] {
            let text = result[field].as_str().expect("bounded stream string");
            assert!(
                serde_json::to_vec(text)
                    .expect("serialize bounded stream")
                    .len()
                    <= 64 * 1024,
                "mode={mode}: {field} must respect the serialized JSON string cap"
            );
        }
    }
}

#[test]
fn docker_pull_json_rejects_nonexact_shapes_before_runtime_and_fails_closed_on_timeout() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_pull_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");

    for args in [
        ["pull", "--format=json", "--timeout=2"].as_slice(),
        ["pull", "--format=table", "--timeout=2", "fixture/image:latest"].as_slice(),
        ["pull", "--format=json", "--timeout=0", "fixture/image:latest"].as_slice(),
        ["pull", "--format=json", "--timeout=1201", "fixture/image:latest"].as_slice(),
        [
            "pull",
            "--format=json",
            "--format=json",
            "--timeout=2",
            "fixture/image:latest",
        ]
        .as_slice(),
        [
            "pull",
            "--format=json",
            "--timeout=2",
            "--timeout=3",
            "fixture/image:latest",
        ]
        .as_slice(),
        [
            "pull",
            "--format=json",
            "--timeout=2",
            "--all",
            "fixture/image:latest",
        ]
        .as_slice(),
        [
            "pull",
            "--format=json",
            "--timeout=2",
            "--",
            "fixture/image:latest",
        ]
        .as_slice(),
        ["pull", "--format=json", "--timeout=2", "-fixture/image:latest"].as_slice(),
        [
            "pull",
            "--format=json",
            "--timeout=2",
            "https://registry.example/fixture/image:latest",
        ]
        .as_slice(),
        [
            "pull",
            "--format=json",
            "--timeout=2",
            "git@registry.example:fixture/image",
        ]
        .as_slice(),
        [
            "pull",
            "fixture/image:latest",
            "--format=json",
            "--timeout=2",
        ]
        .as_slice(),
        [
            "pull",
            "--format=json",
            "fixture/image:latest",
            "--timeout=2",
        ]
        .as_slice(),
        [
            "pull",
            "--format=json",
            "--timeout=2",
            "fixture/image:latest",
            "other:latest",
        ]
        .as_slice(),
    ] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &log)
            .args(args)
            .output()
            .expect("run rejected strict Docker pull JSON argv");
        assert!(
            !output.status.success(),
            "non-exact Docker pull JSON argv must fail: {args:?}\n{}",
            output_text(&output)
        );
    }
    assert!(
        !log.exists(),
        "rejected strict Docker pull JSON argv must not start Apple Container: {}",
        log.display()
    );

    let started = Instant::now();
    let timeout = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_PULL_JSON_MODE", "timeout")
        .args(strict_pull_json_args(1))
        .output()
        .expect("run timed-out strict Docker pull receipt");
    assert_eq!(timeout.status.code(), Some(1), "{}", output_text(&timeout));
    assert!(
        timeout.stdout.is_empty(),
        "a client-observation timeout must fail closed without a partial pull receipt: {}",
        output_text(&timeout)
    );
    let stderr = String::from_utf8_lossy(&timeout.stderr);
    assert!(
        stderr.contains("Apple Container pull client observation timed out"),
        "timeout must retain the host pull client operation label: {stderr}"
    );
    assert!(
        !stderr.contains("raw-timeout-pull-must-not-be-wrapped"),
        "raw timed-out child output must not leak: {stderr}"
    );
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "the direct pull client deadline and bounded cleanup must prevent a long wait: {:?}",
        started.elapsed()
    );
}

#[test]
fn docker_build_json_emits_one_bounded_retained_image_receipt() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_build_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let (context, canonical_context) = fake_build_context(root.path());
    let log = root.path().join("container.log");

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &log)
        .env("VAT_FAKE_BUILD_CONTEXT", &canonical_context)
        .args(strict_build_json_args(&context, 2))
        .output()
        .expect("run strict Docker build JSON receipt");

    assert!(output.status.success(), "{}", output_text(&output));
    assert!(
        output.stderr.is_empty(),
        "bounded build diagnostics must stay in one receipt: {}",
        output_text(&output)
    );
    let stdout = String::from_utf8(output.stdout).expect("receipt stdout is UTF-8 JSON");
    assert!(
        stdout.contains("\\u0001"),
        "control bytes must be escaped in the JSON receipt: {stdout}"
    );
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("one strict Docker build VAT JSON receipt");
    assert_eq!(result["schema"], "vat.docker.build.v1");
    assert_eq!(result["format"], "vat_json");
    assert_eq!(result["type"], "vat_docker_build");
    assert_eq!(result["command"], "build");
    assert_eq!(result["backend"], "apple-container");
    assert_eq!(result["tag"], "fixture/image:latest");
    assert_eq!(result["context"], canonical_context);
    assert_eq!(result["context_kind"], "existing_local_directory");
    assert_eq!(result["dockerfile"], "Dockerfile.fixture");
    assert_eq!(result["requested_timeout_seconds"], 2);
    assert_eq!(result["timeout_scope"], "host-container-client-observation");
    assert_eq!(result["source"], "apple-container-build");
    assert_eq!(result["outcome"], "completed");
    assert_eq!(result["child_exit_code"], 0);
    assert_eq!(result["runtime_invoked"], true);
    assert_eq!(result["image_lifecycle"], "retained_no_auto_cleanup");
    assert_eq!(result["partial_or_replaced_image_cleanup_attempted"], false);
    assert_eq!(result["docker_engine_api_implemented"], false);
    assert_eq!(result["provenance_verified"], false);
    assert_eq!(result["ownership_verified"], false);
    assert_eq!(result["readiness_verified"], false);
    assert_eq!(result["security_verified"], false);
    assert_eq!(result["secret_redaction_guaranteed"], false);
    assert_eq!(result["cancellation_guaranteed"], false);
    assert_eq!(result["rollback_guaranteed"], false);
    assert_eq!(result["untrusted_build_arguments"], true);
    assert_eq!(result["untrusted_build_labels"], true);
    assert_eq!(result["untrusted_build_output"], true);
    assert!(
        result["stdout"]
            .as_str()
            .is_some_and(|value| value.contains("build-stdout") && value.contains("invalid:\u{FFFD}")),
        "bounded stdout must retain fake child content: {result}"
    );
    assert_eq!(result["stdout_utf8_lossy"], true);
    assert_eq!(result["stderr_utf8_lossy"], true);
    assert_eq!(
        result["next"],
        "docker image inspect --format json 'fixture/image:latest'"
    );
    assert!(
        result.get("terminal").is_none(),
        "successful build receipt hands off to exact image inspect rather than declaring terminal completion"
    );
    assert!(
        !result.to_string().contains("TOKEN=opaque"),
        "opaque build args must not be copied into the receipt"
    );

    assert_eq!(
        fs::read_to_string(&log).expect("read fake build invocation"),
        format!(
            "build --tag fixture/image:latest --file Dockerfile.fixture --build-arg MODE=test --build-arg TOKEN=opaque --target release --platform linux/arm64 --label io.cclab.vat.test=opaque {canonical_context}\n"
        ),
        "the strict receipt must strip only its Docker-facing selectors and use canonical local context"
    );
}

#[test]
fn docker_build_json_preserves_a_normal_child_nonzero_receipt_without_stale_image_handoff() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_build_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let (context, canonical_context) = fake_build_context(root.path());

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_BUILD_CONTEXT", &canonical_context)
        .env("VAT_FAKE_BUILD_JSON_MODE", "nonzero")
        .args(strict_build_json_args(&context, 2))
        .output()
        .expect("run failing strict Docker build JSON receipt");

    assert_eq!(output.status.code(), Some(40), "{}", output_text(&output));
    assert!(output.stderr.is_empty(), "{}", output_text(&output));
    let result: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one nonzero build receipt");
    assert_eq!(result["schema"], "vat.docker.build.v1");
    assert_eq!(result["outcome"], "failed");
    assert_eq!(result["child_exit_code"], 40);
    assert_eq!(result["terminal"], "build_failed");
    assert_eq!(result["next"], "docker --help");
    assert_eq!(result["stdout"], "build-before-failure\n");
    assert!(
        result["stderr"]
            .as_str()
            .is_some_and(|value| value.contains("fake Apple Container build failure")),
        "bounded stderr must retain failure diagnostic: {result}"
    );
    assert!(
        !result["next"]
            .as_str()
            .is_some_and(|next| next.contains("fixture/image:latest")),
        "failed build must not hand off to possibly stale/replaced image inspect: {result}"
    );
}

#[test]
fn docker_build_json_retains_lossy_and_bounded_text_receipts_without_deadlocking() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_build_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let (context, canonical_context) = fake_build_context(root.path());

    for mode in ["malformed", "oversized", "flood"] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_BUILD_CONTEXT", &canonical_context)
            .env("VAT_FAKE_BUILD_JSON_MODE", mode)
            .args(strict_build_json_args(&context, 2))
            .output()
            .expect("run lossy or bounded strict Docker build receipt");
        assert!(
            output.status.success(),
            "mode={mode}: {}",
            output_text(&output)
        );
        assert!(
            output.stderr.is_empty(),
            "mode={mode}: child diagnostics must stay inside the receipt: {}",
            output_text(&output)
        );
        assert!(
            output.stdout.len() < 160 * 1024,
            "mode={mode}: bounded build receipt must remain finite, got {} bytes",
            output.stdout.len()
        );
        let result: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("bounded build receipt JSON");
        assert_eq!(result["schema"], "vat.docker.build.v1");
        assert_eq!(result["outcome"], "completed");
        match mode {
            "malformed" => {
                assert_eq!(result["stdout_utf8_lossy"], true);
                assert_eq!(result["stderr_utf8_lossy"], true);
                assert!(
                    result["stdout"]
                        .as_str()
                        .is_some_and(|stdout| stdout.contains("invalid:\u{FFFD}")),
                    "malformed text must be represented lossily inside receipt: {result}"
                );
            }
            "oversized" => {
                assert_eq!(result["stdout_truncated"], true);
                assert_eq!(result["stderr_truncated"], false);
            }
            "flood" => {
                assert_eq!(result["stdout_truncated"], true);
                assert_eq!(result["stderr_truncated"], true);
            }
            _ => unreachable!("test modes are fixed"),
        }
        for field in ["stdout", "stderr"] {
            let text = result[field].as_str().expect("bounded stream string");
            assert!(
                serde_json::to_vec(text)
                    .expect("serialize bounded stream")
                    .len()
                    <= 64 * 1024,
                "mode={mode}: {field} must respect the serialized JSON string cap"
            );
        }
    }
}

#[test]
fn docker_build_json_rejects_nonexact_shapes_before_runtime_and_fails_closed_on_timeout() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_build_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let (context, canonical_context) = fake_build_context(root.path());
    let context_text = context
        .to_str()
        .expect("UTF-8 fake build context")
        .to_string();
    let file_context = root.path().join("not-a-directory");
    fs::write(&file_context, "not a directory").expect("write fake file context");
    let file_context = file_context
        .to_str()
        .expect("UTF-8 fake file context")
        .to_string();
    let log = root.path().join("container.log");

    let mut invalid_cases = vec![
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--progress=plain".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--quiet".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--pull".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--no-cache".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--unknown".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--timeout=3".to_string(),
            "--tag=fixture/image:latest".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:one".to_string(),
            "--tag=fixture/image:two".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--file=Dockerfile.one".to_string(),
            "--file=Dockerfile.two".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--target=one".to_string(),
            "--target=two".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--platform=linux/arm64".to_string(),
            "--platform=linux/amd64".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "-".to_string(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--help".to_string(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "https://example.invalid/repo.git".to_string(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            file_context,
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            context_text.clone(),
            "--label=late=x".to_string(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            context_text.clone(),
            "another-context".to_string(),
        ],
    ];
    invalid_cases.extend([
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--file".to_string(),
            "--help".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--file=--help".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--target".to_string(),
            "--help".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--target=--help".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--platform".to_string(),
            "--".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--platform=--help".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--build-arg".to_string(),
            "--TOKEN=opaque".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--build-arg=--TOKEN=opaque".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--label".to_string(),
            "--owner=opaque".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--label=--owner=opaque".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--file=Dockerfile\u{1}".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--build-arg=MODE\u{1}=opaque".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            "--label=owner=opaque\u{1}".to_string(),
            context_text.clone(),
        ],
        vec![
            "build".to_string(),
            "--format=json".to_string(),
            "--timeout=2".to_string(),
            "--tag=fixture/image:latest".to_string(),
            format!("{context_text}\u{1}"),
        ],
    ]);
    for args in invalid_cases {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &log)
            .env("VAT_FAKE_BUILD_CONTEXT", &canonical_context)
            .args(&args)
            .output()
            .expect("run rejected strict Docker build JSON argv");
        assert!(
            !output.status.success(),
            "non-exact Docker build JSON argv must fail: {args:?}\n{}",
            output_text(&output)
        );
    }
    assert!(
        !log.exists(),
        "rejected strict Docker build JSON argv must not start Apple Container: {}",
        log.display()
    );

    let started = Instant::now();
    let timeout = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_BUILD_CONTEXT", &canonical_context)
        .env("VAT_FAKE_BUILD_JSON_MODE", "timeout")
        .args(strict_build_json_args(&context, 1))
        .output()
        .expect("run timed-out strict Docker build receipt");
    assert_eq!(timeout.status.code(), Some(1), "{}", output_text(&timeout));
    assert!(
        timeout.stdout.is_empty(),
        "a client-observation timeout must fail closed without a partial build receipt: {}",
        output_text(&timeout)
    );
    let stderr = String::from_utf8_lossy(&timeout.stderr);
    assert!(
        stderr.contains("Apple Container build client observation timed out"),
        "timeout must retain the host build client operation label: {stderr}"
    );
    assert!(
        !stderr.contains("raw-timeout-build-must-not-be-wrapped"),
        "raw timed-out child output must not leak: {stderr}"
    );
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "the direct build client deadline and bounded cleanup must prevent a long wait: {:?}",
        started.elapsed()
    );
}

#[test]
fn native_image_owner_guard_requires_exact_apple_metadata_and_absence_preflight() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let fake_container = write_image_owner_guard_fake_container(&fake_bin);
    let owner_label = "io.cclab.vat.e2e-owner";
    let owner_token = "expected-owner";

    assert!(
        real_image_tag_is_proven_absent(&fake_container, "missing:latest"),
        "only the exact native not-found diagnostic may authorize a new tag"
    );
    for tag in [
        "present:latest",
        "noisy-missing:latest",
        "uncertain:latest",
    ] {
        assert!(
            !real_image_tag_is_proven_absent(&fake_container, tag),
            "successful or uncertain native image inspect must fail closed: {tag}"
        );
    }

    assert!(
        real_image_has_owner_label(
            &fake_container,
            "owned:latest",
            owner_label,
            owner_token
        ),
        "the actual Apple variants[*].config.config.Labels shape must prove ownership"
    );
    for tag in [
        "partial:latest",
        "missing-label:latest",
        "legacy:latest",
        "multi-image:latest",
        "malformed:latest",
    ] {
        assert!(
            !real_image_has_owner_label(&fake_container, tag, owner_label, owner_token),
            "missing/mismatched variants or a non-single-image document must fail closed: {tag}"
        );
    }
    assert!(
        !delete_real_owned_image(&fake_container, "partial:latest", owner_label, owner_token),
        "an ambiguous native image document must never authorize a delete"
    );
    assert!(
        delete_real_owned_image(&fake_container, "owned:latest", owner_label, owner_token),
        "a delete must re-inspect the exact label in every native variant"
    );
}

#[test]
fn docker_run_json_wraps_one_bounded_ephemeral_snapshot_and_proves_exact_cleanup() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_run_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let state = root.path().join("run-state.json");
    let log = root.path().join("container.log");

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &log)
        .env("VAT_FAKE_RUN_STATE", &state)
        .args([
            "run",
            "--timeout",
            "2",
            "--format=json",
            "fixture/image:latest",
            "fixture-command",
            "--literal",
        ])
        .output()
        .expect("run strict Docker run JSON shim");

    assert!(output.status.success(), "{}", output_text(&output));
    assert!(
        output.stderr.is_empty(),
        "bounded run diagnostics must stay in one wrapper: {}",
        output_text(&output)
    );
    let stdout = String::from_utf8(output.stdout).expect("wrapper stdout is UTF-8 JSON");
    assert!(
        stdout.contains("\\u0001"),
        "control bytes must be JSON escaped: {stdout}"
    );
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("one strict Docker run VAT JSON document");
    assert_eq!(result["schema"], "vat.docker.run.v1");
    assert_eq!(result["format"], "vat_json");
    assert_eq!(result["backend"], "apple-container");
    assert_eq!(result["image"], "fixture/image:latest");
    assert!(
        result["generated_container_name"]
            .as_str()
            .is_some_and(|name| name.starts_with("vat-docker-run-")),
        "wrapper must expose the generated exact name needed to verify cleanup: {result}"
    );
    assert_eq!(result["requested_timeout_seconds"], 2);
    assert_eq!(result["timeout_scope"], "host-container-client-observation");
    assert_eq!(result["source"], "apple-container-run");
    assert_eq!(result["outcome"], "completed");
    assert_eq!(result["child_exit_code"], 0);
    assert_eq!(result["runtime_invoked"], true);
    assert_eq!(result["cleanup"], "confirmed_absent");
    assert_eq!(result["terminal"], "cleaned_up");
    assert_eq!(result["secret_redaction_guaranteed"], false);
    assert_eq!(result["untrusted_command_output"], true);
    assert!(
        result["stdout"]
            .as_str()
            .is_some_and(|value| value.contains("run-stdout") && value.contains("invalid:\u{FFFD}")),
        "bounded stdout must retain fake child content: {result}"
    );
    assert!(
        result["stderr"]
            .as_str()
            .is_some_and(|value| value.contains("run-stderr-control:\u{1}")),
        "bounded stderr must retain fake child content: {result}"
    );
    assert!(!state.exists(), "owner-checked cleanup must remove fake state");

    let calls = fs::read_to_string(&log).expect("read fake run calls");
    let lines = calls.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 4, "strict run + inspect/delete/inspect: {calls}");
    let run = lines[0].split_whitespace().collect::<Vec<_>>();
    assert_eq!(run[0], "run");
    assert_eq!(run[1], "--name");
    let name = run[2];
    assert!(
        name.starts_with("vat-docker-run-")
            && name.len() == "vat-docker-run-".len() + 32
            && name["vat-docker-run-".len()..]
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
        "generated run name must be a high-entropy hex identifier: {name}"
    );
    assert_eq!(run[3], "--label");
    assert!(
        run[4].starts_with("io.cclab.vat.docker-run-owner=vat-run-")
            && !run.iter().any(|argument| *argument == "--rm"),
        "strict run must own its label and avoid auto-removal before inspect validation: {}",
        lines[0]
    );
    assert_eq!(run[5], "fixture/image:latest");
    assert_eq!(run[6], "fixture-command");
    assert_eq!(run[7], "--literal");
    assert_eq!(lines[1], format!("inspect {name}"));
    assert_eq!(lines[2], format!("delete --force {name}"));
    assert_eq!(lines[3], format!("inspect {name}"));
}

#[test]
fn docker_run_json_rejects_caller_lifecycle_options_before_runtime() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_run_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let state = root.path().join("run-state.json");
    let log = root.path().join("container.log");

    for args in [
        [
            "run",
            "--format=json",
            "--timeout=2",
            "--detach",
            "fixture/image:latest",
        ]
        .as_slice(),
        [
            "run",
            "--timeout",
            "2",
            "--format",
            "json",
            "--name",
            "caller-selected",
            "fixture/image:latest",
        ]
        .as_slice(),
        [
            "run",
            "--format=json",
            "--timeout=2",
            "--label",
            "caller=owned",
            "fixture/image:latest",
        ]
        .as_slice(),
        [
            "run",
            "--format=json",
            "--timeout=2",
            "--publish",
            "127.0.0.1:8080:80",
            "fixture/image:latest",
        ]
        .as_slice(),
        [
            "run",
            "--format=json",
            "--timeout=2",
            "--network",
            "host",
            "fixture/image:latest",
        ]
        .as_slice(),
        [
            "run",
            "--format=json",
            "--timeout=2",
            "--mount",
            "type=tmpfs,target=/work",
            "fixture/image:latest",
        ]
        .as_slice(),
        [
            "run",
            "--format=json",
            "--timeout=2",
            "--env",
            "SECRET=value",
            "fixture/image:latest",
        ]
        .as_slice(),
        [
            "run",
            "--format=json",
            "--timeout=0",
            "fixture/image:latest",
        ]
        .as_slice(),
        [
            "run",
            "--format=json",
            "--timeout=1201",
            "fixture/image:latest",
        ]
        .as_slice(),
        [
            "run",
            "--format=json",
            "--timeout=2",
            "--",
            "fixture/image:latest",
        ]
        .as_slice(),
        [
            "run",
            "--format=json",
            "--timeout=2",
            "fixture/image:latest",
            "--",
            "fixture-command",
        ]
        .as_slice(),
    ] {
        let output = Command::new(&shim)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &log)
            .env("VAT_FAKE_RUN_STATE", &state)
            .args(args)
            .output()
            .expect("run rejected strict Docker run JSON argv");
        assert!(
            !output.status.success(),
            "non-exact Docker run JSON argv must fail: {args:?}\n{}",
            output_text(&output)
        );
    }
    assert!(
        !log.exists(),
        "rejected strict Docker run JSON argv must not start Apple Container: {}",
        log.display()
    );
    assert!(
        !state.exists(),
        "rejected strict Docker run JSON argv must not create cleanup state"
    );
}

#[test]
fn docker_run_json_preserves_a_child_nonzero_exit_only_after_cleanup() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_run_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let state = root.path().join("run-state.json");

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_RUN_STATE", &state)
        .env("VAT_FAKE_RUN_JSON_MODE", "nonzero")
        .args([
            "run",
            "--format",
            "json",
            "--timeout",
            "2",
            "fixture/image:latest",
            "fixture-command",
            "--literal",
        ])
        .output()
        .expect("run failing strict Docker run JSON shim");

    assert_eq!(output.status.code(), Some(41), "{}", output_text(&output));
    assert!(output.stderr.is_empty(), "{}", output_text(&output));
    let result: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one nonzero run wrapper");
    assert_eq!(result["schema"], "vat.docker.run.v1");
    assert_eq!(result["outcome"], "failed");
    assert_eq!(result["child_exit_code"], 41);
    assert_eq!(result["cleanup"], "confirmed_absent");
    assert_eq!(result["terminal"], "cleaned_up");
    assert_eq!(result["stdout"], "run-before-failure\n");
    assert!(
        result["stderr"]
            .as_str()
            .is_some_and(|value| value.contains("fake Apple Container run failure")),
        "bounded stderr must retain failure diagnostic: {result}"
    );
    assert!(!state.exists(), "nonzero child must still be owner-cleaned");
}

#[test]
fn docker_run_json_timeout_cleans_owned_state_without_partial_wrapper() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_run_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let state = root.path().join("run-state.json");

    let started = Instant::now();
    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_RUN_STATE", &state)
        .env("VAT_FAKE_RUN_JSON_MODE", "timeout")
        .args([
            "run",
            "--format=json",
            "--timeout=1",
            "fixture/image:latest",
            "fixture-command",
            "--literal",
        ])
        .output()
        .expect("run timed-out strict Docker run JSON shim");

    assert_eq!(output.status.code(), Some(1), "{}", output_text(&output));
    assert!(
        output.stdout.is_empty(),
        "a timed-out run must not expose a partial wrapper: {}",
        output_text(&output)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Apple Container run client observation timed out"),
        "timeout must retain the bounded run client label: {stderr}"
    );
    assert!(
        !stderr.contains("raw-timeout-run-must-not-be-wrapped"),
        "raw timed-out child output must not leak: {stderr}"
    );
    assert!(
        !state.exists(),
        "observed timeout must owner-clean the generated fake container"
    );
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "timeout plus bounded cleanup must not hang: {:?}",
        started.elapsed()
    );
}

#[test]
fn docker_run_json_accepts_only_explicit_not_found_and_refuses_label_mismatch_cleanup() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_run_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);

    let absent_state = root.path().join("absent-state.json");
    let absent_log = root.path().join("absent-container.log");
    let absent = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &absent_log)
        .env("VAT_FAKE_RUN_STATE", &absent_state)
        .env("VAT_FAKE_RUN_JSON_MODE", "initial_not_found")
        .args([
            "run",
            "--format=json",
            "--timeout=2",
            "fixture/image:latest",
            "fixture-command",
            "--literal",
        ])
        .output()
        .expect("run strict Docker run with already-absent fake state");
    assert!(absent.status.success(), "{}", output_text(&absent));
    let absent_result: serde_json::Value =
        serde_json::from_slice(&absent.stdout).expect("absent cleanup wrapper");
    assert_eq!(absent_result["cleanup"], "confirmed_absent");
    let absent_calls = fs::read_to_string(&absent_log).expect("read absent cleanup calls");
    assert_eq!(
        absent_calls.lines().count(),
        2,
        "only run + exact not-found inspect may occur when state is already absent: {absent_calls}"
    );
    assert!(
        !absent_calls.contains("delete --force"),
        "an explicit not-found diagnostic must not authorize a blind delete: {absent_calls}"
    );

    let mismatch_state = root.path().join("mismatch-state.json");
    let mismatch_log = root.path().join("mismatch-container.log");
    let mismatch = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &mismatch_log)
        .env("VAT_FAKE_RUN_STATE", &mismatch_state)
        .env("VAT_FAKE_RUN_JSON_MODE", "label_mismatch")
        .args([
            "run",
            "--format=json",
            "--timeout=2",
            "fixture/image:latest",
            "fixture-command",
            "--literal",
        ])
        .output()
        .expect("run strict Docker run with mismatched owner label");
    assert_eq!(mismatch.status.code(), Some(1), "{}", output_text(&mismatch));
    assert!(
        mismatch.stdout.is_empty(),
        "cleanup uncertainty must suppress the success wrapper: {}",
        output_text(&mismatch)
    );
    assert!(
        String::from_utf8_lossy(&mismatch.stderr).contains("exact owner label did not verify"),
        "label mismatch must report the fail-closed reason: {}",
        output_text(&mismatch)
    );
    assert!(
        mismatch_state.exists(),
        "label mismatch must leak rather than delete an unverified state"
    );
    let mismatch_calls = fs::read_to_string(&mismatch_log).expect("read mismatch calls");
    assert!(
        !mismatch_calls.contains("delete --force"),
        "a mismatched label must never authorize name-only delete: {mismatch_calls}"
    );

    let uncertain_state = root.path().join("uncertain-state.json");
    let uncertain = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_RUN_STATE", &uncertain_state)
        .env("VAT_FAKE_RUN_JSON_MODE", "inspect_uncertain")
        .args([
            "run",
            "--format=json",
            "--timeout=2",
            "fixture/image:latest",
            "fixture-command",
            "--literal",
        ])
        .output()
        .expect("run strict Docker run with uncertain fake inspect");
    assert_eq!(uncertain.status.code(), Some(1), "{}", output_text(&uncertain));
    assert!(uncertain.stdout.is_empty(), "{}", output_text(&uncertain));
    assert!(
        String::from_utf8_lossy(&uncertain.stderr)
            .contains("did not return its explicit container-not-found diagnostic"),
        "arbitrary inspect failure must not be accepted as absence: {}",
        output_text(&uncertain)
    );
}

#[cfg(unix)]
struct EscapedLogsPipeHolderRelease {
    release_path: PathBuf,
    exited_path: PathBuf,
    released: bool,
}

#[cfg(unix)]
impl EscapedLogsPipeHolderRelease {
    fn new(release_path: PathBuf, exited_path: PathBuf) -> Self {
        Self {
            release_path,
            exited_path,
            released: false,
        }
    }

    fn release(&mut self) {
        if !self.released {
            fs::write(&self.release_path, b"release escaped logs pipe holder")
                .expect("release escaped logs pipe holder");
            self.released = true;
        }

        let deadline = Instant::now() + Duration::from_secs(2);
        while !self.exited_path.exists() {
            assert!(
                Instant::now() < deadline,
                "escaped logs pipe holder did not exit after its test-owned release"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }
}

#[cfg(unix)]
impl Drop for EscapedLogsPipeHolderRelease {
    fn drop(&mut self) {
        if !self.released {
            let _ = fs::write(&self.release_path, b"release escaped logs pipe holder");
        }
    }
}

#[cfg(unix)]
#[test]
fn docker_logs_json_fails_closed_when_an_escaped_pipe_holder_outlives_the_root() {
    let root = TempDir::new().expect("escaped logs timeout tempdir");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_logs_json_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let ready_path = root.path().join("escaped-pipe-holder.ready");
    let release_path = root.path().join("release-escaped-pipe-holder");
    let exited_path = root.path().join("escaped-pipe-holder.exited");
    let mut release = EscapedLogsPipeHolderRelease::new(release_path.clone(), exited_path);
    let helper = std::env::current_exe().expect("current VAT integration-test binary");

    let started = Instant::now();
    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_LOGS_JSON_MODE", "escaped_holder")
        .env("VAT_LOGS_ESCAPED_PIPE_HOLDER_HELPER", helper)
        .env("VAT_LOGS_ESCAPED_PIPE_HOLDER_READY_PATH", &ready_path)
        .env("VAT_LOGS_ESCAPED_PIPE_HOLDER_RELEASE_PATH", &release_path)
        .env(
            "VAT_LOGS_ESCAPED_PIPE_HOLDER_EXITED_PATH",
            &release.exited_path,
        )
        .args(["logs", "--format=json", "--tail=17", "agent-web"])
        .output()
        .expect("run escaped-pipe strict Docker logs JSON shim");

    assert_eq!(output.status.code(), Some(1), "{}", output_text(&output));
    assert!(
        ready_path.exists(),
        "the helper must prove its escaped pipe holder started: {}",
        output_text(&output)
    );
    assert!(
        output.stdout.is_empty(),
        "an escaped pipe holder must fail closed without any partial vat.docker.logs.v1 wrapper: {}",
        output_text(&output)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("escaped pipe holder was not joined")
            || stderr.contains("capture readers were detached"),
        "escaped holder failure must disclose detached capture readers: {stderr}"
    );
    assert!(
        !stderr.contains("running 1 test") && !stderr.contains("vat.docker.logs.v1"),
        "escaped child output and a partial wrapper must not be replayed: {stderr}"
    );
    assert!(
        started.elapsed() < Duration::from_secs(8),
        "the direct logs deadline and bounded cleanup must prevent a hang: {:?}",
        started.elapsed()
    );

    release.release();
}

#[test]
fn compose_profile_rejects_lossy_file_before_runtime() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");
    let compose = root.path().join("compose.yml");
    fs::write(
        &compose,
        r#"services:
  web:
    image: nginx:alpine
    build: .
    ports:
      - "18080:80"
"#,
    )
    .expect("write lossy compose profile");

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &log)
        .args([
            "compose",
            "-f",
            compose.to_str().expect("UTF-8 compose path"),
            "-p",
            "agent-web",
            "up",
            "-d",
        ])
        .output()
        .expect("run rejected docker compose shim command");

    assert!(
        !output.status.success(),
        "lossy profile must fail: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !log.exists(),
        "strict compose preflight must not invoke Apple Container: {}",
        log.display()
    );
}

#[test]
fn docker_compose_dry_run_validates_strict_profiles_without_runtime_or_registry_state() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let vat_home = root.path().join("vat-home");
    let container_log = root.path().join("container.log");

    let cases = [
        (
            "single-image",
            "services:\n  web:\n    image: nginx:1.27-alpine\n    ports: [\"18081:80\"]\n",
            false,
            "strict-single-image-v1",
        ),
        (
            "single-build",
            "services:\n  web:\n    build: .\n    ports: [\"18082:80\"]\n",
            true,
            "strict-single-build-v1",
        ),
        (
            "host-facing",
            "x-vat-compose-profile: host-facing-independent-v1\nservices:\n  docs:\n    image: nginx:1.27-alpine\n    ports: [\"18083:80\"]\n  inspector:\n    image: nginx:1.27-alpine\n    ports: [\"18084:80\"]\n",
            false,
            "host-facing-independent-v1",
        ),
    ];
    for (name, source, build, expected_profile) in cases {
        let compose = root.path().join(format!("{name}.yml"));
        fs::write(&compose, source).expect("write strict dry-run fixture");
        let project = format!("dry-run-{name}");
        let mut command = Command::new(&shim);
        command
            .env("VAT_HOME", &vat_home)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &container_log)
            .args([
                "compose",
                "--dry-run",
                "-f",
                compose.to_str().expect("UTF-8 compose path"),
                "-p",
                &project,
                "up",
                "-d",
            ]);
        if build {
            command.arg("--build");
        }
        let output = command.output().expect("run Docker-shaped dry run");
        assert!(
            output.status.success(),
            "{name} dry run failed:\n{}",
            output_text(&output)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(
            stdout.lines().count(),
            1,
            "dry run must emit exactly one agent document: {stdout}"
        );
        let result: serde_json::Value =
            serde_json::from_str(stdout.trim()).expect("parse dry-run JSON");
        assert_eq!(
            result.get("schema").and_then(serde_json::Value::as_str),
            Some("vat.docker-compose.preflight.v1")
        );
        assert_eq!(result.get("dry_run"), Some(&serde_json::Value::Bool(true)));
        assert_eq!(result.get("build"), Some(&serde_json::Value::Bool(build)));
        assert_eq!(
            result.get("validated"),
            Some(&serde_json::Value::Bool(true))
        );
        assert_eq!(
            result.get("profile").and_then(serde_json::Value::as_str),
            Some(expected_profile)
        );
        assert_eq!(
            result.get("runtime_started"),
            Some(&serde_json::Value::Bool(false))
        );
        assert_eq!(
            result.get("registry_written"),
            Some(&serde_json::Value::Bool(false))
        );
        assert_eq!(
            result.get("image_built"),
            Some(&serde_json::Value::Bool(false))
        );
        assert_eq!(
            result.get("launch_revalidates"),
            Some(&serde_json::Value::Bool(true))
        );
        let launch_argv = result["launch_argv"].as_array().expect("launch argv");
        assert_eq!(
            launch_argv.last().and_then(serde_json::Value::as_str),
            Some(if build { "--build" } else { "-d" }),
            "build flag must be explicit in the returned launch argv: {result}"
        );
        assert_eq!(launch_argv.len(), if build { 9 } else { 8 });
        assert!(
            !launch_argv.iter().any(|argument| argument == "--dry-run"),
            "next launch must not repeat the non-stateful preflight flag: {result}"
        );
        assert!(
            result.get("topology").is_none(),
            "preflight must not claim runtime topology: {result}"
        );
    }

    let invalid = root.path().join("build-without-flag.yml");
    fs::write(
        &invalid,
        "services:\n  web:\n    build: .\n    ports: [\"18085:80\"]\n",
    )
    .expect("write mismatched build fixture");
    let rejected = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &container_log)
        .args([
            "compose",
            "--dry-run",
            "-f",
            invalid.to_str().expect("UTF-8 invalid compose path"),
            "-p",
            "dry-run-invalid",
            "up",
            "-d",
        ])
        .output()
        .expect("reject mismatched dry-run profile");
    assert!(
        !rejected.status.success() && output_text(&rejected).contains("may not use build"),
        "dry run must enforce the same image/build profile split:\n{}",
        output_text(&rejected)
    );
    assert!(
        !container_log.exists(),
        "dry run must not invoke Apple Container: {}",
        container_log.display()
    );
    assert!(
        !vat_home.join("compose").exists(),
        "dry run must not import or write a Compose registry"
    );
}

#[test]
fn docker_compose_dry_run_returns_canonical_source_for_cross_cwd_revalidation() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    let source_dir = root.path().join("preflight-source");
    let other_dir = root.path().join("different-cwd");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    fs::create_dir_all(&source_dir).expect("create preflight source directory");
    fs::create_dir_all(&other_dir).expect("create alternate working directory");
    write_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let vat_home = root.path().join("vat-home");
    let container_log = root.path().join("container.log");
    let source = source_dir.join("compose.yml");
    let unrelated_same_name = other_dir.join("compose.yml");
    let valid_profile =
        "services:\n  web:\n    image: nginx:1.27-alpine\n    ports: [\"18086:80\"]\n";
    fs::write(&source, valid_profile).expect("write preflight source profile");
    fs::write(&unrelated_same_name, valid_profile).expect("write unrelated same-name profile");

    let preflight = Command::new(&shim)
        .current_dir(&source_dir)
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &container_log)
        .args([
            "compose",
            "--dry-run",
            "-f",
            "compose.yml",
            "-p",
            "dry-run-canonical",
            "up",
            "-d",
        ])
        .output()
        .expect("run relative-path Docker-shaped dry run");
    assert!(
        preflight.status.success(),
        "relative dry run failed:\n{}",
        output_text(&preflight)
    );
    let result: serde_json::Value =
        serde_json::from_slice(&preflight.stdout).expect("parse relative dry-run JSON");
    let launch_argv = result["launch_argv"]
        .as_array()
        .expect("launch argv")
        .iter()
        .map(|argument| {
            argument
                .as_str()
                .expect("launch argv must contain only strings")
                .to_string()
        })
        .collect::<Vec<_>>();
    let canonical_source = fs::canonicalize(&source).expect("canonical preflight source");
    assert_eq!(
        launch_argv.get(3).map(String::as_str),
        canonical_source.to_str(),
        "the returned launch must retain the preflighted source identity"
    );

    // A same-named valid file exists in the new cwd, while the exact source
    // that passed preflight is now invalid. The returned launch must fail on
    // that canonical source before it can import or invoke Apple Container.
    fs::write(
        &source,
        "services:\n  web:\n    image: nginx:1.27-alpine\n    ports: [\"18086:80\"]\n    networks: [private]\n",
    )
    .expect("invalidate exact preflight source");
    let relaunched = Command::new(&shim)
        .current_dir(&other_dir)
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &container_log)
        .args(&launch_argv[1..])
        .output()
        .expect("revalidate canonical launch from another cwd");
    assert!(
        !relaunched.status.success() && output_text(&relaunched).contains("network"),
        "launch must revalidate the exact canonical source, not the alternate cwd file:\n{}",
        output_text(&relaunched)
    );
    assert!(
        !container_log.exists(),
        "cross-cwd failed revalidation must not invoke Apple Container"
    );
    assert!(
        !vat_home.join("compose").exists(),
        "cross-cwd failed revalidation must not write a Compose registry"
    );
}

#[test]
fn docker_compose_post_verbs_fail_closed_for_generic_and_unknown_provenance() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let vat_home = root.path().join("vat-home");
    let container_log = root.path().join("container.log");
    let compose = root.path().join("generic-compose.yml");
    let project = "generic-direct-import";
    fs::write(
        &compose,
        r#"services:
  web:
    image: nginx:1.27-alpine
    ports: ["18080:80"]
"#,
    )
    .expect("write generic compose file");

    let imported = Command::new(vat_bin())
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .args([
            "compose",
            "import",
            compose.to_str().expect("UTF-8 compose path"),
            "--project",
            project,
            "--runtime",
            "micro-vm",
        ])
        .output()
        .expect("generic vat compose import");
    assert!(
        imported.status.success(),
        "generic import failed:\n{}",
        output_text(&imported)
    );

    let post_verbs = vec![
        vec!["compose", "-p", project, "ps"],
        vec!["compose", "-p", project, "ps", "--format", "json"],
        vec!["compose", "-p", project, "logs", "web"],
        vec![
            "compose", "-p", project, "logs", "--format", "json", "--tail", "2", "web",
        ],
        vec!["compose", "-p", project, "exec", "-T", "web", "--", "true"],
        vec![
            "compose",
            "-p",
            project,
            "exec",
            "-T",
            "--format=json",
            "web",
            "--",
            "true",
        ],
        vec!["compose", "-p", project, "down"],
    ];
    for args in &post_verbs {
        let output = Command::new(&shim)
            .env("VAT_HOME", &vat_home)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &container_log)
            .args(args)
            .output()
            .expect("Docker-shaped post verb against generic import");
        assert_shim_provenance_rejected(&output, "no Docker shim profile provenance");
    }

    let record_path = vat_home.join("compose").join(project).join("project.json");
    let mut record: serde_json::Value =
        serde_json::from_slice(&fs::read(&record_path).expect("read generic compose registry"))
            .expect("parse generic compose registry");
    record["docker_shim_profile"] = serde_json::Value::String("unknown-profile-v1".to_string());
    fs::write(
        &record_path,
        serde_json::to_vec_pretty(&record).expect("serialize unknown provenance registry"),
    )
    .expect("write unknown provenance registry");

    for args in &post_verbs {
        let output = Command::new(&shim)
            .env("VAT_HOME", &vat_home)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &container_log)
            .args(args)
            .output()
            .expect("Docker-shaped post verb against unknown provenance");
        assert_shim_provenance_rejected(&output, "unsupported Docker shim provenance");
    }
    assert!(
        !container_log.exists(),
        "generic/unknown provenance must reject JSON exec before any Apple Container invocation"
    );

    for args in [
        vec!["compose", "up", "--project", project, "--detach"],
        vec!["compose", "ps", project],
        vec!["compose", "logs", project, "web"],
    ] {
        let output = Command::new(vat_bin())
            .env("VAT_HOME", &vat_home)
            .env("PATH", path_with_prepend(&fake_bin))
            .args(args)
            .output()
            .expect("generic VAT lifecycle command against unknown provenance");
        assert!(
            !output.status.success()
                && output_text(&output).contains("was imported by VAT's Docker shim profile"),
            "generic VAT lifecycle must reject unknown provenance:\n{}",
            output_text(&output)
        );
    }
    let generic_import = Command::new(vat_bin())
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .args([
            "compose",
            "import",
            compose.to_str().expect("UTF-8 compose path"),
            "--project",
            project,
            "--runtime",
            "micro-vm",
        ])
        .output()
        .expect("generic import against unknown provenance");
    assert!(
        !generic_import.status.success()
            && output_text(&generic_import).contains("generic import will not adopt"),
        "generic import must not adopt unknown provenance:\n{}",
        output_text(&generic_import)
    );

    let generic_down = Command::new(vat_bin())
        .env("VAT_HOME", &vat_home)
        .args(["compose", "down", project])
        .output()
        .expect("generic down cleanup for unknown provenance");
    assert!(
        generic_down.status.success(),
        "generic down must release inactive unknown provenance registry:\n{}",
        output_text(&generic_down)
    );
    assert!(
        !record_path.exists(),
        "registry-only unknown-profile cleanup must remove project.json"
    );
    assert!(
        vat_home
            .join("compose")
            .join(project)
            .join("vat.toml")
            .exists(),
        "registry-only unknown-profile cleanup must retain vat.toml for diagnosis"
    );

    let active_project = "unknown-active-profile";
    let active_registry = vat_home.join("compose").join(active_project);
    fs::create_dir_all(&active_registry).expect("create active unknown registry");
    let active_record = active_registry.join("project.json");
    fs::write(
        &active_record,
        serde_json::to_vec_pretty(&serde_json::json!({
            "project": active_project,
            "vat_id": "future-profile-bound-vat",
            "docker_shim_profile": "unknown-profile-v1",
            "handoff_protocol": 1,
            "service_ids": ["web"],
            "status": "ready",
            "created_at": "2026-07-14T00:00:00Z",
        }))
        .expect("serialize active unknown registry"),
    )
    .expect("write active unknown registry");
    let active_down = Command::new(vat_bin())
        .env("VAT_HOME", &vat_home)
        .args(["compose", "down", active_project])
        .output()
        .expect("generic down active unknown provenance");
    assert!(
        !active_down.status.success()
            && output_text(&active_down).contains("may still own runtime resources"),
        "active unknown profile must remain fail-closed:\n{}",
        output_text(&active_down)
    );
    assert!(
        active_record.exists(),
        "active unknown profile registry must remain retained after rejected cleanup"
    );

    let handoff_project = "unknown-imported-handoff";
    let handoff_registry = vat_home.join("compose").join(handoff_project);
    fs::create_dir_all(&handoff_registry).expect("create unknown handoff registry");
    let handoff_record = handoff_registry.join("project.json");
    fs::write(
        &handoff_record,
        serde_json::to_vec_pretty(&serde_json::json!({
            "project": handoff_project,
            "vat_id": null,
            "docker_shim_profile": "unknown-profile-v1",
            "handoff_protocol": 1,
            "startup_pid": 12345,
            "startup_token": "future-profile-handoff",
            "startup_started_at": "2026-07-14T00:00:00Z",
            "service_ids": ["web"],
            "status": "imported",
            "created_at": "2026-07-14T00:00:00Z",
        }))
        .expect("serialize unknown in-flight handoff registry"),
    )
    .expect("write unknown in-flight handoff registry");
    let handoff_down = Command::new(vat_bin())
        .env("VAT_HOME", &vat_home)
        .args(["compose", "down", handoff_project])
        .output()
        .expect("generic down unknown in-flight handoff");
    assert!(
        !handoff_down.status.success()
            && output_text(&handoff_down).contains("in-flight startup handoff"),
        "unknown imported handoff must remain fail-closed:\n{}",
        output_text(&handoff_down)
    );
    assert!(
        handoff_record.exists(),
        "unknown imported handoff registry must remain retained after rejected cleanup"
    );
}

#[test]
fn docker_compose_ps_json_is_one_document_for_a_known_inactive_profile() {
    let root = TempDir::new().expect("temp root");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    let shim = docker_shim(&shim_dir);
    let vat_home = root.path().join("vat-home");
    let project = "json-inactive-profile";
    let registry = vat_home.join("compose").join(project);
    fs::create_dir_all(&registry).expect("create seeded registry");
    fs::write(
        registry.join("project.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "project": project,
            "vat_id": null,
            "docker_shim_profile": "strict-single-image-v1",
            "handoff_protocol": 1,
            "service_ids": ["web"],
            "status": "imported",
            "created_at": "2026-07-14T00:00:00Z",
        }))
        .expect("serialize known inactive shim registry"),
    )
    .expect("write known inactive shim registry");

    let output = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .args(["compose", "-p", project, "ps", "--format=json"])
        .output()
        .expect("run VAT-native JSON compose ps");
    assert!(
        output.status.success(),
        "VAT-native JSON ps failed:\n{}",
        output_text(&output)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.lines().count(),
        1,
        "JSON mode must not emit the inactive human explanation: {stdout}"
    );
    let result: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("parse one VAT-native ps document");
    assert_eq!(
        result.get("schema").and_then(serde_json::Value::as_str),
        Some("vat.docker-compose.ps.v1")
    );
    assert_eq!(
        result.get("format").and_then(serde_json::Value::as_str),
        Some("vat_json")
    );
    assert_eq!(
        result.get("profile").and_then(serde_json::Value::as_str),
        Some("strict-single-image-v1")
    );
    assert_eq!(
        result.get("topology"),
        Some(&serde_json::json!({
            "phase": "inactive",
            "ready": false,
            "services": [{
                "name": "web",
                "state": "inactive",
            }],
        }))
    );
    assert!(
        result["topology"]["services"][0].get("endpoint").is_none(),
        "inactive JSON must not claim a routable endpoint"
    );
}

#[test]
fn docker_compose_logs_json_is_one_bounded_capture_snapshot_without_runtime_call() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_logging_lifecycle_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let vat_home = root.path().join("vat-home");
    let container_log = root.path().join("container.log");
    let project = "logs-json-snapshot";
    let compose = root.path().join("compose.yml");
    fs::write(
        &compose,
        "services:\n  web:\n    image: nginx:1.27-alpine\n    ports: [\"18087:80\"]\n",
    )
    .expect("write compose fixture");

    let up = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &container_log)
        .args([
            "compose",
            "-f",
            compose.to_str().expect("UTF-8 compose path"),
            "-p",
            project,
            "up",
            "-d",
        ])
        .output()
        .expect("launch logging Compose fixture");
    assert!(
        up.status.success(),
        "logging fixture up failed:\n{}",
        output_text(&up)
    );
    let raw_logs = wait_for_compose_log_marker(&shim, &vat_home, project, "web", "stdout-three");
    assert!(
        output_text(&raw_logs).contains("stderr-three"),
        "text logs must preserve the historic raw stream surface"
    );
    let raw_logs_stdout = String::from_utf8_lossy(&raw_logs.stdout);
    assert!(
        raw_logs_stdout.contains("stderr-three\n{"),
        "text log replay must put its terminal handoff on a separate line: {raw_logs_stdout}"
    );
    let raw_logs_result = compose_shim_result(&raw_logs.stdout, "logs");
    assert_eq!(raw_logs_result.get("terminal"), Some(&serde_json::json!("observed")));
    let calls_before_snapshot =
        fs::read_to_string(&container_log).expect("read lifecycle calls before log snapshot");

    let logs = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &container_log)
        .args([
            "compose",
            "-p",
            project,
            "logs",
            "--format=json",
            "--tail=2",
            "web",
        ])
        .output()
        .expect("read one JSON Compose log snapshot");
    assert!(
        logs.status.success(),
        "compose logs --format json failed:\n{}",
        output_text(&logs)
    );
    let stdout = String::from_utf8_lossy(&logs.stdout);
    assert_eq!(
        stdout.lines().count(),
        1,
        "VAT-native JSON logs must not emit raw log text or an additive second result: {stdout}"
    );
    let result: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("parse one VAT-native logs document");
    assert_eq!(
        result.get("schema").and_then(serde_json::Value::as_str),
        Some("vat.docker-compose.logs.v1")
    );
    assert_eq!(
        result.get("format").and_then(serde_json::Value::as_str),
        Some("vat_json")
    );
    assert_eq!(result.get("service"), Some(&serde_json::json!("web")));
    assert_eq!(result.get("tail_lines"), Some(&serde_json::json!(2)));
    assert_eq!(
        result.get("stdout"),
        Some(&serde_json::json!("stdout-two\nstdout-three"))
    );
    assert_eq!(
        result.get("stderr"),
        Some(&serde_json::json!("stderr-two\nstderr-three"))
    );
    for key in ["stdout_truncated", "stderr_truncated"] {
        assert_eq!(result.get(key), Some(&serde_json::Value::Bool(true)));
    }
    for key in [
        "stdout_utf8_lossy",
        "stderr_utf8_lossy",
        "runtime_invoked",
        "compose_record_mutated",
    ] {
        assert_eq!(result.get(key), Some(&serde_json::Value::Bool(false)));
    }
    assert_eq!(
        result.get("capture_only"),
        Some(&serde_json::Value::Bool(true))
    );
    assert_eq!(
        result.get("next").and_then(serde_json::Value::as_str),
        Some("docker compose -p logs-json-snapshot ps --format json")
    );
    assert!(
        result.get("topology").is_none(),
        "logs do not prove endpoints or ready topology"
    );
    assert_eq!(
        fs::read_to_string(&container_log).expect("read lifecycle calls after log snapshot"),
        calls_before_snapshot,
        "JSON log snapshot must not call Apple Container"
    );

    let cleanup_deadline = Instant::now() + Duration::from_secs(5);
    let down = loop {
        let output = Command::new(&shim)
            .env("VAT_HOME", &vat_home)
            .env("PATH", path_with_prepend(&fake_bin))
            .env("VAT_FAKE_CONTAINER_LOG", &container_log)
            .args(["compose", "-p", project, "down"])
            .output()
            .expect("clean up logging Compose fixture");
        if output.status.success() || Instant::now() >= cleanup_deadline {
            break output;
        }
        thread::sleep(Duration::from_millis(50));
    };
    assert!(
        down.status.success(),
        "logging fixture down failed:\n{}",
        output_text(&down)
    );
}

#[test]
fn generic_vat_lifecycle_rejects_known_shim_provenance_and_reimport_clears_it() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let vat_home = root.path().join("vat-home");
    let project = "known-shim-provenance";
    let registry = vat_home.join("compose").join(project);
    fs::create_dir_all(&registry).expect("create seeded registry");
    let vat_toml = registry.join("vat.toml");
    fs::write(
        &vat_toml,
        r#"version = 1

[workspace]
keep = "always"

[[services]]
id = "web"
image = "fake:seeded"
runtime = "docker"
container_port = 80
port = 18080
timeout_s = 2

[[runners]]
id = "project.up"
requires = ["web"]
cmd = ["sleep", "2147483647"]
"#,
    )
    .expect("write same-ID seeded vat.toml");
    let record_path = registry.join("project.json");
    fs::write(
        &record_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "project": project,
            "vat_id": null,
            "docker_shim_profile": "strict-single-image-v1",
            "handoff_protocol": 1,
            "service_ids": ["web"],
            "status": "imported",
            "created_at": "2026-07-14T00:00:00Z",
        }))
        .expect("serialize seeded shim registry"),
    )
    .expect("write seeded shim registry");

    // Keep exactly the same service identity while editing the materialized
    // config. Generic up must reject profile provenance before it could reach
    // any runtime backend.
    let seeded = fs::read_to_string(&vat_toml).expect("read seeded vat.toml");
    fs::write(&vat_toml, seeded.replace("fake:seeded", "fake:user-edited"))
        .expect("write same-ID user edit");

    let generic_lifecycle = vec![
        vec!["compose", "up", "--project", project, "--detach"],
        vec!["compose", "ps", project],
        vec!["compose", "logs", project, "web"],
        vec!["compose", "down", project],
    ];
    for args in generic_lifecycle {
        let output = Command::new(vat_bin())
            .env("VAT_HOME", &vat_home)
            .env("PATH", path_with_prepend(&fake_bin))
            .args(args)
            .output()
            .expect("generic VAT lifecycle against known shim provenance");
        assert!(
            !output.status.success()
                && output_text(&output).contains("was imported by VAT's Docker shim profile"),
            "generic lifecycle must reject known shim provenance:\n{}",
            output_text(&output)
        );
    }

    let source = root.path().join("fresh-compose.yml");
    fs::write(
        &source,
        r#"services:
  web:
    image: nginx:1.27-alpine
    ports: ["18080:80"]
"#,
    )
    .expect("write fresh generic compose source");
    let reimport = Command::new(vat_bin())
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .args([
            "compose",
            "import",
            source.to_str().expect("UTF-8 compose path"),
            "--project",
            project,
            "--runtime",
            "micro-vm",
        ])
        .output()
        .expect("generic re-import known shim provenance");
    assert!(
        reimport.status.success(),
        "generic re-import must explicitly clear known shim provenance:\n{}",
        output_text(&reimport)
    );
    let reimported: serde_json::Value =
        serde_json::from_slice(&fs::read(&record_path).expect("read re-imported registry"))
            .expect("parse re-imported registry");
    assert!(
        reimported.get("docker_shim_profile").is_none(),
        "generic re-import must clear provenance rather than adopt it: {reimported}"
    );
    let generic_ps = Command::new(vat_bin())
        .env("VAT_HOME", &vat_home)
        .args(["compose", "ps", project])
        .output()
        .expect("generic ps after re-import");
    assert!(
        generic_ps.status.success(),
        "generic lifecycle must own re-imported project:\n{}",
        output_text(&generic_ps)
    );
    let shim_ps = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .args(["compose", "-p", project, "ps"])
        .output()
        .expect("shim ps after generic re-import");
    assert_shim_provenance_rejected(&shim_ps, "no Docker shim profile provenance");
}

#[test]
fn compose_host_facing_independent_profile_runs_two_services_through_the_shim() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_lifecycle_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let log = root.path().join("container.log");
    let vat_home = root.path().join("vat-home");

    // The fake runtime deliberately does not bind host ports; these listeners
    // model Apple Container's loopback-published endpoint so VAT's MicroVM
    // readiness proof sees two distinct live endpoints.
    let Some(docs_listener) =
        loopback_listener_or_skip("host-facing Docker Compose shim lifecycle regression")
    else {
        return;
    };
    let docs_port = docs_listener
        .local_addr()
        .expect("docs loopback endpoint address")
        .port();
    let Some(inspector_listener) =
        loopback_listener_or_skip("host-facing Docker Compose shim lifecycle regression")
    else {
        return;
    };
    let inspector_port = inspector_listener
        .local_addr()
        .expect("inspector loopback endpoint address")
        .port();
    assert_ne!(
        docs_port, inspector_port,
        "independent services need unique ports"
    );

    let project = format!("local-tools-{docs_port}");
    let compose = root.path().join("compose.yml");
    fs::write(
        &compose,
        format!(
            "x-vat-compose-profile: host-facing-independent-v1\nservices:\n  docs:\n    image: nginx:1.27-alpine\n    ports: [\"{docs_port}:80\"]\n    environment:\n      MODE: docs\n  inspector:\n    image: nginx:1.27-alpine\n    ports: [\"{inspector_port}:80\"]\n    environment:\n      MODE: inspect\n"
        ),
    )
    .expect("write independent host-facing compose profile");

    let up = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &log)
        .args([
            "compose",
            "-f",
            compose.to_str().expect("UTF-8 compose path"),
            "-p",
            &project,
            "up",
            "-d",
        ])
        .output()
        .expect("run independent host-facing Compose through VAT shim");
    assert!(
        up.status.success(),
        "compose up stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&up.stdout),
        String::from_utf8_lossy(&up.stderr)
    );
    let up_result = compose_shim_result(&up.stdout, "up");
    assert_eq!(
        up_result.get("profile").and_then(serde_json::Value::as_str),
        Some("host-facing-independent-v1")
    );
    assert_eq!(
        up_result
            .get("service_name_dns")
            .and_then(serde_json::Value::as_bool),
        Some(false)
    );
    assert_eq!(
        up_result
            .get("host_loopback_only")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let ps = wait_for_compose_ready(&shim, &vat_home, &project);
    let ps_stdout = String::from_utf8_lossy(&ps.stdout);
    assert!(
        ps_stdout.contains("docs\tReady"),
        "compose ps:\n{ps_stdout}"
    );
    assert!(
        ps_stdout.contains("inspector\tReady"),
        "compose ps:\n{ps_stdout}"
    );
    let ps_result = compose_shim_result(&ps.stdout, "ps");
    assert_eq!(
        ps_result.get("profile").and_then(serde_json::Value::as_str),
        Some("host-facing-independent-v1"),
        "known shim ps must retain its exact profile in final JSON: {ps_result}"
    );
    assert_eq!(
        ps_result
            .get("service_name_dns")
            .and_then(serde_json::Value::as_bool),
        Some(false)
    );
    assert_eq!(
        ps_result
            .get("host_loopback_only")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    let expected_topology = serde_json::json!({
        "phase": "ready",
        "ready": true,
        "services": [
            {
                "name": "docs",
                "state": "ready",
                "endpoint": format!("127.0.0.1:{docs_port}"),
            },
            {
                "name": "inspector",
                "state": "ready",
                "endpoint": format!("127.0.0.1:{inspector_port}"),
            },
        ],
    });
    assert_eq!(
        ps_result.get("topology"),
        Some(&expected_topology),
        "two-service host-facing ps must provide only its exact VAT-proven loopback endpoints in registry service order: {ps_result}"
    );

    let ps_json = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &log)
        .args(["compose", "-p", &project, "ps", "--format", "json"])
        .output()
        .expect("read one VAT-native JSON compose topology");
    assert!(
        ps_json.status.success(),
        "compose ps --format json:\n{}",
        output_text(&ps_json)
    );
    let ps_json_stdout = String::from_utf8_lossy(&ps_json.stdout);
    assert_eq!(
        ps_json_stdout.lines().count(),
        1,
        "VAT-native JSON ps must not prepend a human table or append another record: {ps_json_stdout}"
    );
    let ps_json_result: serde_json::Value =
        serde_json::from_str(ps_json_stdout.trim()).expect("parse VAT-native compose ps JSON");
    assert_eq!(
        ps_json_result
            .get("schema")
            .and_then(serde_json::Value::as_str),
        Some("vat.docker-compose.ps.v1")
    );
    assert_eq!(
        ps_json_result
            .get("format")
            .and_then(serde_json::Value::as_str),
        Some("vat_json")
    );
    assert_eq!(
        ps_json_result
            .get("profile")
            .and_then(serde_json::Value::as_str),
        Some("host-facing-independent-v1")
    );
    assert_eq!(
        ps_json_result.get("service_name_dns"),
        Some(&serde_json::Value::Bool(false))
    );
    assert_eq!(
        ps_json_result.get("host_loopback_only"),
        Some(&serde_json::Value::Bool(true))
    );
    assert_eq!(
        ps_json_result.get("topology"),
        Some(&expected_topology),
        "machine-readable ps must reuse the same claim-held VAT topology proof"
    );

    let vat_id = compose_vat_id(&vat_home, &project);
    let docs_name = format!("{vat_id}-docs");
    let inspector_name = format!("{vat_id}-inspector");
    let calls_after_up = fs::read_to_string(&log).expect("read fake container lifecycle calls");
    for (name, port) in [(&docs_name, docs_port), (&inspector_name, inspector_port)] {
        assert!(
            calls_after_up.contains(&format!("run --rm --name {name} -p 127.0.0.1:{port}:80")),
            "missing exact independent MicroVM launch for {name}:\n{calls_after_up}"
        );
    }
    assert_eq!(
        calls_after_up
            .lines()
            .filter(|line| line.starts_with("run --rm --name "))
            .count(),
        2,
        "the host-facing profile must launch exactly its two services:\n{calls_after_up}"
    );

    let record_path = vat_home.join("compose").join(&project).join("project.json");
    let record: serde_json::Value = serde_json::from_slice(
        &fs::read(&record_path).expect("read active host-facing Compose registry"),
    )
    .expect("parse active host-facing Compose registry");
    assert_eq!(
        record
            .get("docker_shim_profile")
            .and_then(serde_json::Value::as_str),
        Some("host-facing-independent-v1"),
        "shim import must retain exact profile provenance: {record}"
    );

    // The generic lifecycle must reject every operation surface before it can
    // touch a shim-owned project. In particular, the `up` rejection is before
    // a new Apple Container launch, even though a same-ID vat.toml would be
    // accepted by the ordinary generic identity gate.
    for args in [
        vec!["compose", "up", "--project", &project, "--detach"],
        vec!["compose", "ps", &project],
        vec!["compose", "logs", &project, "docs"],
        vec!["compose", "down", &project],
    ] {
        let output = Command::new(vat_bin())
            .env("VAT_HOME", &vat_home)
            .env("PATH", path_with_prepend(&fake_bin))
            .args(args)
            .output()
            .expect("generic VAT lifecycle command against shim project");
        assert!(
            !output.status.success()
                && output_text(&output).contains("was imported by VAT's Docker shim profile"),
            "generic VAT lifecycle must reject shim provenance:\n{}",
            output_text(&output)
        );
    }
    assert_eq!(
        fs::read_to_string(&log).expect("read lifecycle calls after generic rejection"),
        calls_after_up,
        "generic lifecycle rejection must happen before another runtime operation"
    );

    let exec = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &log)
        .args([
            "compose",
            "-p",
            &project,
            "exec",
            "-T",
            "docs",
            "--",
            "--version",
        ])
        .output()
        .expect("exec one independent host-facing service through VAT shim");
    assert!(
        exec.status.success(),
        "compose exec stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&exec.stdout),
        String::from_utf8_lossy(&exec.stderr)
    );
    let exec_stdout = String::from_utf8_lossy(&exec.stdout);
    assert!(
        exec_stdout.contains("fake-compose-exec")
            && exec_stdout.contains("\"outcome\":\"completed\""),
        "compose exec must forward fake child output and handoff JSON: {exec_stdout}"
    );
    assert!(
        exec_stdout.contains("fake-compose-exec\n{"),
        "compose exec must put its terminal handoff on a separate line: {exec_stdout}"
    );
    assert_eq!(
        compose_shim_result(&exec.stdout, "exec").get("outcome"),
        Some(&serde_json::json!("completed"))
    );
    let record_before_exec_json: serde_json::Value = serde_json::from_slice(
        &fs::read(&record_path).expect("read shim registry before JSON exec"),
    )
    .expect("parse shim registry before JSON exec");
    let calls_before_exec_json =
        fs::read_to_string(&log).expect("read fake calls before JSON exec");
    let exec_json = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &log)
        .args([
            "compose",
            "-p",
            &project,
            "exec",
            "-T",
            "--format=json",
            "docs",
            "--",
            "--env",
            "vat-exec-json-nonzero",
        ])
        .output()
        .expect("exec one independent host-facing service through VAT JSON shim");
    assert_eq!(
        exec_json.status.code(),
        Some(23),
        "JSON exec must preserve the fake child nonzero exit:\n{}",
        output_text(&exec_json)
    );
    let exec_json_stdout = String::from_utf8_lossy(&exec_json.stdout);
    assert_eq!(
        exec_json_stdout.lines().count(),
        1,
        "JSON exec must emit one document without replaying raw child stdout: {exec_json_stdout}"
    );
    assert!(
        exec_json.stderr.is_empty(),
        "JSON exec must capture raw child stderr in its document:\n{}",
        output_text(&exec_json)
    );
    let exec_json_result: serde_json::Value = serde_json::from_str(exec_json_stdout.trim())
        .expect("parse one VAT-native compose exec JSON document");
    assert_eq!(
        exec_json_result
            .get("schema")
            .and_then(serde_json::Value::as_str),
        Some("vat.docker-compose.exec.v1")
    );
    assert_eq!(
        exec_json_result.get("format"),
        Some(&serde_json::json!("vat_json"))
    );
    assert_eq!(
        exec_json_result.get("outcome"),
        Some(&serde_json::json!("failed"))
    );
    assert_eq!(
        exec_json_result.get("child_exit_code"),
        Some(&serde_json::json!(23))
    );
    assert_eq!(
        exec_json_result.get("stdout"),
        Some(&serde_json::json!(
            "exec-json-stdout-one\nexec-json-stdout-two\n"
        ))
    );
    assert_eq!(
        exec_json_result.get("stderr"),
        Some(&serde_json::json!(
            "exec-json-stderr-one\nexec-json-stderr-two\n"
        ))
    );
    for key in [
        "stdout_truncated",
        "stderr_truncated",
        "stdout_utf8_lossy",
        "stderr_utf8_lossy",
        "compose_record_mutated",
    ] {
        assert_eq!(
            exec_json_result.get(key),
            Some(&serde_json::Value::Bool(false)),
            "JSON exec field `{key}`"
        );
    }
    assert_eq!(
        exec_json_result.get("runtime_invoked"),
        Some(&serde_json::Value::Bool(true))
    );
    assert_eq!(
        exec_json_result.get("profile"),
        Some(&serde_json::json!("host-facing-independent-v1"))
    );
    assert!(
        exec_json_result.get("topology").is_none(),
        "exec JSON must not reopen endpoint topology after its spawn proof"
    );
    let record_after_exec_json: serde_json::Value = serde_json::from_slice(
        &fs::read(&record_path).expect("read shim registry after JSON exec"),
    )
    .expect("parse shim registry after JSON exec");
    for key in ["vat_id", "docker_shim_profile", "service_ids", "status"] {
        assert_eq!(
            record_after_exec_json.get(key),
            record_before_exec_json.get(key),
            "JSON exec must not widen or mutate Compose registry field `{key}`"
        );
    }
    let calls_after_exec_json = fs::read_to_string(&log).expect("read fake calls after JSON exec");
    assert!(
    calls_after_exec_json.starts_with(&calls_before_exec_json)
        && calls_after_exec_json
            .contains(&format!("exec {docs_name} --env vat-exec-json-nonzero")),
        "JSON exec must pass option-looking argv directly after the Apple Container name:\n{calls_after_exec_json}"
    );
    let inspector_exec = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &log)
        .args([
            "compose",
            "-p",
            &project,
            "exec",
            "-T",
            "inspector",
            "--",
            "true",
        ])
        .output()
        .expect("exec the second independent host-facing service through VAT shim");
    assert!(
        inspector_exec.status.success(),
        "inspector compose exec stdout:\n{}",
        output_text(&inspector_exec)
    );
    let calls_after_exec = fs::read_to_string(&log).expect("read fake container exec calls");
    for name in [&docs_name, &inspector_name] {
        assert!(
            calls_after_exec.contains(&format!("exec {name} ")),
            "compose exec must target exact VAT-owned MicroVM `{name}`:\n{calls_after_exec}"
        );
    }
    assert!(
        calls_after_exec.contains(&format!("exec {docs_name} --version")),
        "text exec must pass option-looking command argv directly after the Apple Container name:\n{calls_after_exec}"
    );

    let down = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &log)
        .args(["compose", "-p", &project, "down"])
        .output()
        .expect("down independent host-facing Compose through VAT shim");
    assert!(
        down.status.success(),
        "compose down stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&down.stdout),
        String::from_utf8_lossy(&down.stderr)
    );
    let calls_after_down = fs::read_to_string(&log).expect("read fake container cleanup calls");
    assert!(
        calls_after_down.contains(&format!("rm -f {docs_name}"))
            && calls_after_down.contains(&format!("rm -f {inspector_name}")),
        "compose down must remove both independent MicroVMs:\n{calls_after_down}"
    );
    let record: serde_json::Value = serde_json::from_slice(
        &fs::read(&record_path).expect("read released host-facing Compose registry"),
    )
    .expect("parse released host-facing Compose registry");
    assert_eq!(
        record.get("status").and_then(serde_json::Value::as_str),
        Some("imported")
    );
    assert!(
        record.get("vat_id").is_some_and(serde_json::Value::is_null),
        "compose down must release the project registry: {record}"
    );
    let inactive_shim_ps = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .args(["compose", "-p", &project, "ps"])
        .output()
        .expect("shim ps after shim-owned down");
    assert!(
        inactive_shim_ps.status.success()
            && output_text(&inactive_shim_ps).contains("does not retain its source Compose file")
            && output_text(&inactive_shim_ps).contains(&format!(
                "same validated `-f <compose-file>` and `-p {project}`"
            )),
        "inactive shim project must give a truthful source-file-required restart instruction:\n{}",
        output_text(&inactive_shim_ps)
    );

    // A normal user is allowed to edit a generated config while the project
    // is inactive, but that must not turn a Docker-shaped record into a
    // generic launch. Keep the exact service IDs and change image references
    // so this passes the generic identity gate; provenance still blocks it
    // before runtime. The following strict shim up must re-import and replace
    // that edit before it starts its next profile-owned run.
    let vat_toml = vat_home.join("compose").join(&project).join("vat.toml");
    let materialized = fs::read_to_string(&vat_toml).expect("read materialized profile config");
    let edited = materialized.replace("nginx:1.27-alpine", "locally-edited:latest");
    assert_ne!(
        edited, materialized,
        "expected literal profile images in vat.toml"
    );
    fs::write(&vat_toml, &edited).expect("write same-ID user-edited vat.toml");

    let generic_up = Command::new(vat_bin())
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .args(["compose", "up", "--project", &project, "--detach"])
        .output()
        .expect("generic up after same-ID user edit");
    assert!(
        !generic_up.status.success()
            && output_text(&generic_up).contains("was imported by VAT's Docker shim profile"),
        "generic up must reject shim provenance before runtime after a same-ID edit:\n{}",
        output_text(&generic_up)
    );
    assert!(
        !fs::read_to_string(&log)
            .expect("read lifecycle calls after same-ID rejection")
            .contains("locally-edited:latest"),
        "generic provenance rejection must precede runtime invocation"
    );

    let second_up = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &log)
        .args([
            "compose",
            "-f",
            compose.to_str().expect("UTF-8 compose path"),
            "-p",
            &project,
            "up",
            "-d",
        ])
        .output()
        .expect("second strict shim up replaces same-ID edit");
    assert!(
        second_up.status.success(),
        "second strict shim up failed:\n{}",
        output_text(&second_up)
    );
    assert!(
        !fs::read_to_string(&vat_toml)
            .expect("read re-imported profile config")
            .contains("locally-edited:latest"),
        "strict shim import must replace a same-ID generic config edit"
    );
    let _second_ps = wait_for_compose_ready(&shim, &vat_home, &project);
    let second_down = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_LOG", &log)
        .args(["compose", "-p", &project, "down"])
        .output()
        .expect("second strict shim down");
    assert!(
        second_down.status.success(),
        "second strict shim down failed:\n{}",
        output_text(&second_down)
    );

    // Explicit generic re-import transfers ownership out of the shim. That
    // clears provenance, makes generic `vat compose ps` available again, and
    // causes Docker-shaped post verbs to fail closed instead of attaching to a
    // generic record merely because it reuses the same project ID.
    let reimport = Command::new(vat_bin())
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .args([
            "compose",
            "import",
            compose.to_str().expect("UTF-8 compose path"),
            "--project",
            &project,
            "--runtime",
            "micro-vm",
        ])
        .output()
        .expect("generic re-import clears shim provenance");
    assert!(
        reimport.status.success(),
        "generic re-import failed:\n{}",
        output_text(&reimport)
    );
    let reimported_record: serde_json::Value =
        serde_json::from_slice(&fs::read(&record_path).expect("read generic re-import registry"))
            .expect("parse generic re-import registry");
    assert!(
        reimported_record.get("docker_shim_profile").is_none(),
        "generic re-import must clear Docker shim provenance: {reimported_record}"
    );
    let generic_ps = Command::new(vat_bin())
        .env("VAT_HOME", &vat_home)
        .args(["compose", "ps", &project])
        .output()
        .expect("generic ps after explicit re-import");
    assert!(
        generic_ps.status.success(),
        "generic ps must own explicitly re-imported project:\n{}",
        output_text(&generic_ps)
    );
    let shim_ps_after_reimport = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .args(["compose", "-p", &project, "ps"])
        .output()
        .expect("shim ps after generic re-import");
    assert_shim_provenance_rejected(&shim_ps_after_reimport, "no Docker shim profile provenance");

    // Keep both test-owned listeners alive through VAT's teardown. They are
    // dropped only after the project has released all fake runtime children.
    drop((docs_listener, inspector_listener));
}

#[test]
fn compose_wait_returns_one_ready_runner_topology_result() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_lifecycle_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let vat_home = root.path().join("vat-home");
    let Some(listener) = loopback_listener_or_skip("Docker Compose --wait ready regression") else {
        return;
    };
    let port = listener.local_addr().expect("wait listener address").port();
    let project = format!("wait-ready-{port}");
    let compose = root.path().join("compose.yml");
    fs::write(
        &compose,
        format!("services:\n  web:\n    image: nginx:1.27-alpine\n    ports: [\"{port}:80\"]\n"),
    )
    .expect("write strict wait compose profile");

    let up = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .args([
            "compose",
            "-f",
            compose.to_str().expect("UTF-8 compose path"),
            "-p",
            &project,
            "up",
            "-d",
            "--wait",
            "--wait-timeout",
            "10",
        ])
        .output()
        .expect("run fake Docker Compose wait");
    assert!(up.status.success(), "compose wait:\n{}", output_text(&up));
    let result = compose_shim_result(&up.stdout, "up");
    assert_eq!(
        result.get("terminal").and_then(serde_json::Value::as_str),
        Some("wait_ready"),
        "wait success must be terminal for this command: {result}"
    );
    assert_eq!(
        result["wait"]
            .get("requested")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        result["wait"]
            .get("timeout_seconds")
            .and_then(serde_json::Value::as_u64),
        Some(10)
    );
    assert_eq!(
        result["wait"]
            .get("outcome")
            .and_then(serde_json::Value::as_str),
        Some("ready")
    );
    assert_eq!(
        result.get("topology"),
        Some(&serde_json::json!({
            "phase": "ready",
            "ready": true,
            "services": [{
                "name": "web",
                "state": "ready",
                "endpoint": format!("127.0.0.1:{port}"),
            }],
        })),
        "wait must return runner-proven topology, not a fresh host probe"
    );
    let structured_up_lines = String::from_utf8_lossy(&up.stdout)
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|value| {
            value.get("type").and_then(serde_json::Value::as_str) == Some("vat_docker_compose")
                && value.get("command").and_then(serde_json::Value::as_str) == Some("up")
        })
        .count();
    assert_eq!(
        structured_up_lines,
        1,
        "--wait must own exactly one final Docker-shaped up result: {}",
        String::from_utf8_lossy(&up.stdout)
    );

    let down = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .args(["compose", "-p", &project, "down"])
        .output()
        .expect("clean fake wait project");
    assert!(
        down.status.success(),
        "compose down:\n{}",
        output_text(&down)
    );
    drop(listener);
}

#[test]
fn compose_wait_timeout_retains_the_launch_until_later_ready_down_cleanup() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_lifecycle_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);
    let vat_home = root.path().join("vat-home");
    let Some(reservation) = loopback_listener_or_skip("Docker Compose --wait timeout regression")
    else {
        return;
    };
    let port = reservation.local_addr().expect("reserved wait port").port();
    // The fake container never binds ports. Releasing this reservation keeps
    // the runner in durable Starting state until the test deliberately makes
    // the same loopback endpoint available after wait has timed out.
    drop(reservation);
    let project = format!("wait-timeout-{port}");
    let compose = root.path().join("compose.yml");
    fs::write(
        &compose,
        format!("services:\n  web:\n    image: nginx:1.27-alpine\n    ports: [\"{port}:80\"]\n"),
    )
    .expect("write timeout compose profile");

    let wait_started = Instant::now();
    let up = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .args([
            "compose",
            "-f",
            compose.to_str().expect("UTF-8 compose path"),
            "-p",
            &project,
            "up",
            "-d",
            "--wait",
            "--wait-timeout=2",
        ])
        .output()
        .expect("run bounded fake wait without endpoint");
    let total_wall = wait_started.elapsed();
    assert_eq!(
        up.status.code(),
        Some(1),
        "compose wait:\n{}",
        output_text(&up)
    );
    assert!(
        total_wall < Duration::from_secs(8),
        "2s wait budget must not absorb historic 10s handoff/claim waits; total wall time (including fake runner startup) was {total_wall:?}"
    );
    let result = compose_shim_result(&up.stdout, "up");
    assert_eq!(
        result.get("terminal").and_then(serde_json::Value::as_str),
        Some("wait_failed")
    );
    assert_eq!(
        result["wait"]
            .get("outcome")
            .and_then(serde_json::Value::as_str),
        Some("timeout"),
        "a timeout is a command failure while retained runtime state remains: {result}"
    );
    assert!(
        result
            .get("topology")
            .and_then(|topology| topology.get("services"))
            .and_then(serde_json::Value::as_array)
            .is_none_or(|services| services
                .iter()
                .all(|service| service.get("endpoint").is_none())),
        "timed out wait must never leak an endpoint: {result}"
    );

    let record_path = vat_home.join("compose").join(&project).join("project.json");
    let record: serde_json::Value =
        serde_json::from_slice(&fs::read(&record_path).expect("read retained wait registry"))
            .expect("parse retained wait registry");
    assert!(
        record
            .get("launch_ticket")
            .and_then(serde_json::Value::as_str)
            .is_some(),
        "timeout must retain the same launch ticket rather than reset the project: {record}"
    );
    assert!(
        record
            .get("launch_generation")
            .and_then(serde_json::Value::as_u64)
            .is_some(),
        "timeout must retain a durable launch generation: {record}"
    );

    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))
        .expect("make retained fake endpoint available");
    let _ready = wait_for_compose_ready(&shim, &vat_home, &project);
    let down = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .env("PATH", path_with_prepend(&fake_bin))
        .args(["compose", "-p", &project, "down"])
        .output()
        .expect("clean retained fake wait project");
    assert!(
        down.status.success(),
        "compose down:\n{}",
        output_text(&down)
    );
    let cleaned: serde_json::Value =
        serde_json::from_slice(&fs::read(&record_path).expect("read cleaned wait registry"))
            .expect("parse cleaned wait registry");
    assert!(
        cleaned.get("launch_ticket").is_none(),
        "down must invalidate retained wait tickets: {cleaned}"
    );
    assert!(
        cleaned.get("vat_id").is_none_or(serde_json::Value::is_null),
        "down must clear the retained VAT binding: {cleaned}"
    );
    drop(listener);
}

#[test]
fn shim_preserves_container_exit_code() {
    let root = TempDir::new().expect("temp root");
    let fake_bin = root.path().join("fake-bin");
    let shim_dir = root.path().join("shim-bin");
    fs::create_dir_all(&shim_dir).expect("create shim directory");
    write_fake_container(&fake_bin);
    let shim = docker_shim(&shim_dir);

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&fake_bin))
        .env("VAT_FAKE_CONTAINER_EXIT", "42")
        .args(["ps", "-a"])
        .output()
        .expect("run docker shim");
    assert_eq!(output.status.code(), Some(42));
}

#[test]
fn installer_creates_only_its_own_symlink_and_refuses_foreign_paths() {
    let root = TempDir::new().expect("temp root");
    let install_dir = root.path().join("install");
    let output = Command::new(vat_bin())
        .args([
            "docker",
            "install-shim",
            "--dir",
            install_dir.to_str().expect("UTF-8 temp path"),
        ])
        .output()
        .expect("install shim");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let installed = install_dir.join("docker");
    assert!(installed.is_symlink(), "expected {}", installed.display());
    assert_eq!(
        fs::canonicalize(&installed).expect("resolve installed shim"),
        fs::canonicalize(vat_bin()).expect("resolve vat binary")
    );

    let foreign_dir = root.path().join("foreign");
    fs::create_dir_all(&foreign_dir).expect("create foreign directory");
    let foreign = foreign_dir.join("docker");
    fs::write(&foreign, "not VAT").expect("write foreign docker file");
    let rejected = Command::new(vat_bin())
        .args([
            "docker",
            "install-shim",
            "--dir",
            foreign_dir.to_str().expect("UTF-8 temp path"),
        ])
        .output()
        .expect("attempt foreign replacement");
    assert!(!rejected.status.success());
    assert_eq!(
        fs::read_to_string(foreign).expect("read foreign file"),
        "not VAT"
    );
}

#[test]
#[ignore = "real Apple Container Docker-command shim contract; run only with VAT_DOCKER_SHIM_E2E_REQUIRED=1"]
fn apple_container_docker_run_published_port_contract() {
    if std::env::var("VAT_DOCKER_SHIM_E2E_REQUIRED").as_deref() != Ok("1") {
        eprintln!("VAT_DOCKER_SHIM_E2E_REQUIRED=1 is required; skipping real Docker shim probe");
        return;
    }

    let root = TempDir::new().expect("temp root");
    let shim_dir = root.path().join("shim-bin");
    let shim = install_real_shim(&shim_dir);
    let port = TcpListener::bind("127.0.0.1:0")
        .expect("select currently free host port")
        .local_addr()
        .expect("selected host port")
        .port();
    let temp_nonce = root
        .path()
        .file_name()
        .and_then(|name| name.to_str())
        .expect("UTF-8 temporary directory name")
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect::<String>();
    assert!(
        !temp_nonce.is_empty(),
        "temporary directory name must yield a safe owner nonce"
    );
    let owner_token = format!("{}-{temp_nonce}", std::process::id());
    let owner_label = "io.cclab.vat.e2e-owner".to_string();
    let owner_label_value = format!("{owner_label}={owner_token}");
    let name = format!("vat-docker-shim-e2e-{owner_token}");
    let mut cleanup = RealContainerCleanup {
        name: name.clone(),
        owner_label: owner_label.clone(),
        owner_token: owner_token.clone(),
        active: false,
    };

    let output = Command::new(&shim)
        .args([
            "run",
            "-d",
            "--name",
            &name,
            "--label",
            &owner_label_value,
            "-p",
            &format!("127.0.0.1:{port}:80"),
            "nginx:alpine",
        ])
        .output()
        .expect("docker run through VAT shim");
    assert!(
        output.status.success(),
        "docker run stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        real_container_has_owner_label(&name, &owner_label, &owner_token),
        "the real test container must preserve its exact owner label before emergency cleanup is armed"
    );
    cleanup.active = true;

    wait_for_http_ok(port).unwrap_or_else(|error| {
        panic!("Docker shim published host endpoint never became HTTP-usable: {error}")
    });

    // These are deliberately opaque Apple Container documents, not Docker
    // Engine schemas. `ps` and `images` are global read-only inventory smoke
    // observations; inspect/stats/logs target this test-owned container before
    // the legacy text paths below prove their existing compatibility contract.
    for (label, args) in [
        ("ps", ["ps", "--format=json", "--all"].as_slice()),
        ("images", ["images", "--format=json"].as_slice()),
        (
            "inspect",
            ["inspect", "--format", "json", name.as_str()].as_slice(),
        ),
        (
            "stats",
            ["stats", "--no-stream", "--format=json", name.as_str()].as_slice(),
        ),
    ] {
        let observation = Command::new(&shim)
            .args(args)
            .output()
            .unwrap_or_else(|error| panic!("run real strict docker {label} JSON shim: {error}"));
        assert!(
            observation.status.success(),
            "real docker {label} JSON stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&observation.stdout),
            String::from_utf8_lossy(&observation.stderr)
        );
        serde_json::from_slice::<serde_json::Value>(&observation.stdout).unwrap_or_else(|error| {
            panic!(
                "real docker {label} JSON must be exactly one native Apple Container document ({error}): {}",
                String::from_utf8_lossy(&observation.stdout)
            )
        });
    }

    let logs_json = Command::new(&shim)
        .args(["logs", "--format=json", "--tail=100", &name])
        .output()
        .expect("read real bounded VAT docker logs JSON snapshot");
    assert!(
        logs_json.status.success(),
        "real docker logs JSON stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&logs_json.stdout),
        String::from_utf8_lossy(&logs_json.stderr)
    );
    let logs_json_result: serde_json::Value = serde_json::from_slice(&logs_json.stdout)
        .expect("parse one real bounded VAT docker logs JSON document");
    assert_eq!(logs_json_result["schema"], "vat.docker.logs.v1");
    assert_eq!(logs_json_result["format"], "vat_json");
    assert_eq!(logs_json_result["container"], name);
    assert_eq!(logs_json_result["requested_tail_lines"], 100);
    assert_eq!(logs_json_result["source"], "apple-container-stdio");
    assert_eq!(logs_json_result["backend"], "apple-container");
    assert_eq!(logs_json_result["outcome"], "observed");
    assert_eq!(logs_json_result["runtime_invoked"], true);
    assert_eq!(logs_json_result["untrusted_log_content"], true);
    assert_eq!(logs_json_result["secret_redaction_guaranteed"], false);
    assert!(
        logs_json_result["apple_container_stdio"].is_string(),
        "direct Apple logs must expose one opaque textual payload: {logs_json_result}"
    );
    assert_eq!(
        logs_json_result["next"],
        format!("docker inspect --format json {name}")
    );
    assert!(
        logs_json_result.get("stdout").is_none() && logs_json_result.get("stderr").is_none(),
        "direct Apple logs must not claim Docker stdout/stderr demultiplexing: {logs_json_result}"
    );

    let exec_marker = format!("vat-docker-exec-json-{owner_token}");
    let exec_command = format!(
        "printf %s {exec_marker}; printf %s {exec_marker} >&2",
    );
    let exec_json = Command::new(&shim)
        .args([
            "exec",
            "--format=json",
            "--timeout=10",
            &name,
            "--",
            "sh",
            "-ec",
            &exec_command,
        ])
        .output()
        .expect("run real bounded VAT docker exec JSON snapshot");
    assert!(
        exec_json.status.success(),
        "real docker exec JSON stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&exec_json.stdout),
        String::from_utf8_lossy(&exec_json.stderr)
    );
    let exec_json_result: serde_json::Value = serde_json::from_slice(&exec_json.stdout)
        .expect("parse one real bounded VAT docker exec JSON document");
    assert_eq!(exec_json_result["schema"], "vat.docker.exec.v1");
    assert_eq!(exec_json_result["format"], "vat_json");
    assert_eq!(exec_json_result["container"], name);
    assert_eq!(exec_json_result["requested_timeout_seconds"], 10);
    assert_eq!(
        exec_json_result["timeout_scope"],
        "host-container-client-observation"
    );
    assert_eq!(exec_json_result["outcome"], "completed");
    assert_eq!(exec_json_result["child_exit_code"], 0);
    assert_eq!(exec_json_result["runtime_invoked"], true);
    assert_eq!(exec_json_result["untrusted_command_output"], true);
    assert_eq!(exec_json_result["secret_redaction_guaranteed"], false);
    assert!(
        exec_json_result["stdout"]
            .as_str()
            .is_some_and(|stdout| stdout.contains(&exec_marker)),
        "real direct exec JSON must retain its stdout marker: {exec_json_result}"
    );
    assert!(
        exec_json_result["stderr"]
            .as_str()
            .is_some_and(|stderr| stderr.contains(&exec_marker)),
        "real direct exec JSON must retain its stderr marker: {exec_json_result}"
    );
    assert_eq!(
        exec_json_result["next"],
        format!("docker inspect --format json {name}")
    );

    let inspect = Command::new(&shim)
        .args(["inspect", &name])
        .output()
        .expect("docker inspect through VAT shim");
    assert!(
        inspect.status.success(),
        "inspect stderr: {}",
        String::from_utf8_lossy(&inspect.stderr)
    );
    let logs = Command::new(&shim)
        .args(["logs", &name])
        .output()
        .expect("docker logs through VAT shim");
    assert!(
        logs.status.success(),
        "logs stderr: {}",
        String::from_utf8_lossy(&logs.stderr)
    );
    // Normal completion deliberately uses the same owner-checked cleanup path
    // as unwinding. Apple Container offers no atomic conditional delete, so
    // this remains best effort and leaks rather than deleting on uncertainty.
    drop(cleanup);

    let absent = Command::new("container")
        .args(["inspect", &name])
        .output()
        .expect("confirm Apple Container cleanup");
    assert!(
        !absent.status.success(),
        "container {name} remained after owner-checked cleanup"
    );
}

#[test]
#[ignore = "real Apple Container strict ephemeral Docker run JSON contract; run only with VAT_DOCKER_RUN_JSON_E2E_REQUIRED=1"]
fn apple_container_docker_run_json_ephemeral_contract() {
    if std::env::var("VAT_DOCKER_RUN_JSON_E2E_REQUIRED").as_deref() != Ok("1") {
        eprintln!(
            "VAT_DOCKER_RUN_JSON_E2E_REQUIRED=1 is required; skipping real Docker run JSON probe"
        );
        return;
    }

    let root = TempDir::new().expect("temp root");
    let shim_dir = root.path().join("shim-bin");
    let shim = install_real_shim(&shim_dir);
    let proxy_bin = root.path().join("recording-container-bin");
    write_recording_container_proxy(&proxy_bin);
    let real_container = host_container_binary();
    let calls_path = root.path().join("real-container-calls.log");
    let marker = format!("vat-docker-run-json-{}", std::process::id());
    let command = format!("printf %s {marker}; printf %s {marker} >&2");

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&proxy_bin))
        .env("VAT_REAL_CONTAINER", &real_container)
        .env("VAT_REAL_CONTAINER_LOG", &calls_path)
        .args([
            "run",
            "--format=json",
            "--timeout=30",
            // This image is used by the existing local K3s probe and is
            // intentionally expected to be available in the local store.
            "alpine:3.20",
            "sh",
            "-ec",
            &command,
        ])
        .output()
        .expect("run real strict Docker run JSON shim");

    let calls = fs::read_to_string(&calls_path).unwrap_or_default();
    let run_line = calls
        .lines()
        .find(|line| line.starts_with("run --name "))
        .expect("real strict run must record the generated Apple argv");
    let run = run_line.split_whitespace().collect::<Vec<_>>();
    assert!(run.len() >= 7, "unexpected real strict run argv: {run_line}");
    assert_eq!(run[0], "run");
    assert_eq!(run[1], "--name");
    assert_eq!(run[3], "--label");
    let name = run[2].to_string();
    let (label, owner_token) = run[4]
        .split_once('=')
        .expect("generated real run label must have a value");
    assert_eq!(label, "io.cclab.vat.docker-run-owner");
    let mut cleanup = RealContainerCleanup {
        name: name.clone(),
        owner_label: label.to_string(),
        owner_token: owner_token.to_string(),
        active: real_container_has_owner_label(&name, label, owner_token),
    };

    assert!(
        output.status.success(),
        "real docker run JSON stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "the real strict run result must be one stdout JSON document: {}",
        output_text(&output)
    );
    let result: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("parse one real bounded VAT Docker run JSON document");
    assert_eq!(result["schema"], "vat.docker.run.v1");
    assert_eq!(result["format"], "vat_json");
    assert_eq!(result["backend"], "apple-container");
    assert_eq!(result["image"], "alpine:3.20");
    assert_eq!(result["requested_timeout_seconds"], 30);
    assert_eq!(result["timeout_scope"], "host-container-client-observation");
    assert_eq!(result["generated_container_name"], name);
    assert_eq!(result["cleanup"], "confirmed_absent");
    assert_eq!(result["terminal"], "cleaned_up");
    assert_eq!(result["outcome"], "completed");
    assert_eq!(result["child_exit_code"], 0);
    assert!(
        result["stdout"]
            .as_str()
            .is_some_and(|stdout| stdout.contains(&marker)),
        "real run wrapper must retain the stdout marker: {result}"
    );
    assert!(
        result["stderr"]
            .as_str()
            .is_some_and(|stderr| stderr.contains(&marker)),
        "real run wrapper must retain the stderr marker: {result}"
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains(owner_token),
        "the owner token must never be exposed in the agent result"
    );

    let absent = Command::new(&real_container)
        .args(["inspect", &name])
        .output()
        .expect("confirm exact real Docker run JSON cleanup");
    assert!(
        !absent.status.success(),
        "generated real container {name} remained after confirmed cleanup"
    );
    assert!(
        String::from_utf8_lossy(&absent.stderr)
            .to_ascii_lowercase()
            .contains(&format!("error: container not found: {}", name.to_ascii_lowercase())),
        "exact cleanup must use the current Apple not-found diagnostic: {}",
        String::from_utf8_lossy(&absent.stderr)
    );
    cleanup.active = false;
}

#[test]
#[ignore = "real Apple Container strict Docker image inspect JSON contract; run only with VAT_DOCKER_IMAGE_INSPECT_JSON_E2E_REQUIRED=1"]
fn apple_container_docker_image_inspect_json_contract() {
    if std::env::var("VAT_DOCKER_IMAGE_INSPECT_JSON_E2E_REQUIRED").as_deref() != Ok("1") {
        eprintln!(
            "VAT_DOCKER_IMAGE_INSPECT_JSON_E2E_REQUIRED=1 is required; skipping real Docker image inspect JSON probe"
        );
        return;
    }

    let root = TempDir::new().expect("temp root");
    let shim_dir = root.path().join("shim-bin");
    let shim = install_real_shim(&shim_dir);
    let proxy_bin = root.path().join("recording-container-bin");
    write_recording_container_proxy(&proxy_bin);
    let real_container = host_container_binary();
    let calls_path = root.path().join("real-container-calls.log");

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&proxy_bin))
        .env("VAT_REAL_CONTAINER", &real_container)
        .env("VAT_REAL_CONTAINER_LOG", &calls_path)
        .args(["image", "inspect", "--format=json", "alpine:3.20"])
        .output()
        .expect("run real strict Docker image inspect JSON shim");

    assert!(
        output.status.success(),
        "real docker image inspect JSON stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "the real strict image inspect result must be one stdout JSON document: {}",
        output_text(&output)
    );
    serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "real docker image inspect JSON must be exactly one native Apple Container document ({error}): {}",
            String::from_utf8_lossy(&output.stdout)
        )
    });
    assert_eq!(
        fs::read_to_string(&calls_path).expect("read real image inspect invocation"),
        "image inspect alpine:3.20\n",
        "the strict image inspect selector must be stripped before Apple Container starts"
    );
}

#[test]
#[ignore = "real Apple Container strict Docker pull JSON receipt contract; run only with VAT_DOCKER_PULL_JSON_E2E_REQUIRED=1"]
fn apple_container_docker_pull_json_receipt_contract() {
    if std::env::var("VAT_DOCKER_PULL_JSON_E2E_REQUIRED").as_deref() != Ok("1") {
        eprintln!(
            "VAT_DOCKER_PULL_JSON_E2E_REQUIRED=1 is required; skipping real Docker pull JSON receipt probe"
        );
        return;
    }

    let root = TempDir::new().expect("temp root");
    let shim_dir = root.path().join("shim-bin");
    let shim = install_real_shim(&shim_dir);
    let proxy_bin = root.path().join("recording-container-bin");
    write_recording_container_proxy(&proxy_bin);
    let real_container = host_container_binary();
    let calls_path = root.path().join("real-container-calls.log");

    // This uses a known local image but deliberately runs the real pull
    // client: a pull can still contact a registry or alter a shared image
    // store. The receipt is non-owning, so this E2E neither deletes nor
    // asserts ownership of `alpine:3.20` on success or failure.
    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&proxy_bin))
        .env("VAT_REAL_CONTAINER", &real_container)
        .env("VAT_REAL_CONTAINER_LOG", &calls_path)
        .args([
            "pull",
            "--format=json",
            "--timeout=120",
            "alpine:3.20",
        ])
        .output()
        .expect("run real strict Docker pull JSON receipt");

    assert!(
        output.status.success(),
        "real docker pull JSON stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "the real strict pull result must be one stdout JSON receipt: {}",
        output_text(&output)
    );
    let result: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one real VAT Docker pull JSON receipt");
    assert_eq!(result["schema"], "vat.docker.pull.v1");
    assert_eq!(result["format"], "vat_json");
    assert_eq!(result["backend"], "apple-container");
    assert_eq!(result["image"], "alpine:3.20");
    assert_eq!(result["requested_timeout_seconds"], 120);
    assert_eq!(result["timeout_scope"], "host-container-client-observation");
    assert_eq!(result["outcome"], "completed");
    assert_eq!(result["child_exit_code"], 0);
    assert_eq!(result["runtime_invoked"], true);
    assert_eq!(result["image_lifecycle"], "not_owned_no_auto_cleanup");
    assert_eq!(result["cleanup_attempted"], false);
    assert_eq!(result["registry_management_implemented"], false);
    assert_eq!(result["image_state_verified"], false);
    assert_eq!(result["secret_redaction_guaranteed"], false);
    assert_eq!(result["cancellation_guaranteed"], false);
    assert_eq!(result["download_completion_guaranteed"], false);
    assert_eq!(result["rollback_guaranteed"], false);
    assert_eq!(
        result["next"],
        "docker image inspect --format json 'alpine:3.20'"
    );
    assert!(
        result.get("terminal").is_none(),
        "a successful pull receipt must hand off to inspect instead of declaring image ownership/completion"
    );
    assert_eq!(
        fs::read_to_string(&calls_path).expect("read real image pull invocation"),
        "image pull alpine:3.20\n",
        "the strict pull selectors must be stripped before Apple Container starts"
    );
}

#[test]
#[ignore = "real Apple Container strict Docker build JSON receipt contract; run only with VAT_DOCKER_BUILD_JSON_E2E_REQUIRED=1"]
fn apple_container_docker_build_json_receipt_contract() {
    if std::env::var("VAT_DOCKER_BUILD_JSON_E2E_REQUIRED").as_deref() != Ok("1") {
        eprintln!(
            "VAT_DOCKER_BUILD_JSON_E2E_REQUIRED=1 is required; skipping real Docker build JSON receipt probe"
        );
        return;
    }

    let root = TempDir::new().expect("temp root");
    let shim_dir = root.path().join("shim-bin");
    let shim = install_real_shim(&shim_dir);
    let proxy_bin = root.path().join("recording-container-bin");
    write_recording_container_proxy(&proxy_bin);
    let real_container = host_container_binary();
    let calls_path = root.path().join("real-container-calls.log");
    let context = root.path().join("build-context");
    fs::create_dir_all(&context).expect("create real build context");
    let dockerfile = context.join("Dockerfile");
    fs::write(
        &dockerfile,
        "FROM alpine:3.20\nARG MESSAGE\nRUN test \"$MESSAGE\" = vat-docker-build-json-e2e\n",
    )
    .expect("write real build Dockerfile");
    let canonical_context = fs::canonicalize(&context)
        .expect("canonicalize real build context")
        .to_str()
        .expect("UTF-8 canonical real build context")
        .to_string();
    let temp_nonce = root
        .path()
        .file_name()
        .and_then(|name| name.to_str())
        .expect("UTF-8 temporary directory name")
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect::<String>();
    assert!(
        !temp_nonce.is_empty(),
        "temporary directory name must yield a safe owner nonce"
    );
    let owner_token = format!("{}-{temp_nonce}", std::process::id());
    let owner_label = "io.cclab.vat.e2e-owner".to_string();
    let owner_label_value = format!("{owner_label}={owner_token}");
    // Image repository components are conventionally lowercase; the tempdir
    // nonce may include uppercase ASCII even though it remains safe as an
    // owner-label value.
    let tag = format!(
        "vat-docker-build-json-{}",
        owner_token.to_ascii_lowercase()
    );
    let mut cleanup = RealOwnedImageCleanup {
        container_binary: real_container.clone(),
        tag: tag.clone(),
        owner_label: owner_label.clone(),
        owner_token: owner_token.clone(),
        active: false,
    };
    assert!(
        real_image_tag_is_proven_absent(&real_container, &tag),
        "the high-entropy build tag must have the exact native absence diagnostic before build; a successful or uncertain inspect must abort without cleanup"
    );
    // Apple Container has no conditional create/build. An inspect-to-build
    // race is therefore unavoidable; the high-entropy tag narrows it and the
    // owner-label check immediately before every cleanup delete remains the
    // authorization boundary. If another writer wins, cleanup leaks.
    cleanup.active = true;

    let output = Command::new(&shim)
        .env("PATH", path_with_prepend(&proxy_bin))
        .env("VAT_REAL_CONTAINER", &real_container)
        .env("VAT_REAL_CONTAINER_LOG", &calls_path)
        .args([
            "build",
            "--format=json",
            "--timeout=120",
            "--tag",
            &tag,
            "--file",
            dockerfile.to_str().expect("UTF-8 real Dockerfile path"),
            "--build-arg",
            "MESSAGE=vat-docker-build-json-e2e",
            "--label",
            &owner_label_value,
            context.to_str().expect("UTF-8 real build context"),
        ])
        .output()
        .expect("run real strict Docker build JSON receipt");

    assert!(
        output.status.success(),
        "real docker build JSON stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "the real strict build result must be one stdout JSON receipt: {}",
        output_text(&output)
    );
    let result: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one real VAT Docker build JSON receipt");
    assert_eq!(result["schema"], "vat.docker.build.v1");
    assert_eq!(result["format"], "vat_json");
    assert_eq!(result["backend"], "apple-container");
    assert_eq!(result["tag"], tag);
    assert_eq!(result["context"], canonical_context);
    assert_eq!(
        result["dockerfile"],
        dockerfile.to_str().expect("UTF-8 real Dockerfile path")
    );
    assert_eq!(result["requested_timeout_seconds"], 120);
    assert_eq!(result["timeout_scope"], "host-container-client-observation");
    assert_eq!(result["outcome"], "completed");
    assert_eq!(result["child_exit_code"], 0);
    assert_eq!(result["runtime_invoked"], true);
    assert_eq!(result["image_lifecycle"], "retained_no_auto_cleanup");
    assert_eq!(result["secret_redaction_guaranteed"], false);
    assert_eq!(result["cancellation_guaranteed"], false);
    assert_eq!(result["rollback_guaranteed"], false);
    assert_eq!(
        result["next"],
        format!("docker image inspect --format json '{tag}'")
    );
    assert!(
        real_image_has_owner_label(&real_container, &tag, &owner_label, &owner_token),
        "real strict build must preserve its exact test owner label before cleanup is authorized"
    );
    assert_eq!(
        fs::read_to_string(&calls_path).expect("read real build invocation"),
        format!(
            "build --tag {tag} --file {} --build-arg MESSAGE=vat-docker-build-json-e2e --label {owner_label_value} {canonical_context}\n",
            dockerfile.to_str().expect("UTF-8 real Dockerfile path")
        ),
        "strict build must strip only Docker-facing JSON/deadline selectors before Apple Container starts"
    );

    assert!(
        delete_real_owned_image(&real_container, &tag, &owner_label, &owner_token),
        "real strict build cleanup must re-inspect the exact owner label before deleting its image"
    );
    assert!(
        real_image_tag_is_proven_absent(&real_container, &tag),
        "real strict build image {tag} must return the exact native absence diagnostic after owner-checked cleanup"
    );
    cleanup.active = false;
}

#[test]
#[ignore = "real Apple Container strict Docker Compose profile; run only with VAT_DOCKER_COMPOSE_SHIM_E2E_REQUIRED=1"]
fn apple_container_docker_compose_strict_profile_contract() {
    if std::env::var("VAT_DOCKER_COMPOSE_SHIM_E2E_REQUIRED").as_deref() != Ok("1") {
        eprintln!(
            "VAT_DOCKER_COMPOSE_SHIM_E2E_REQUIRED=1 is required; skipping real Docker Compose shim probe"
        );
        return;
    }

    let root = TempDir::new().expect("temp root");
    let shim_dir = root.path().join("shim-bin");
    let shim = install_real_shim(&shim_dir);
    let vat_home = root.path().join("vat-home");
    let port = TcpListener::bind("127.0.0.1:0")
        .expect("select currently free host port")
        .local_addr()
        .expect("selected host port")
        .port();
    let project = format!("vat-compose-shim-{port}");
    let compose = root.path().join("compose.yml");
    fs::write(
        &compose,
        format!(
            "services:\n  web:\n    image: nginx:1.27-alpine\n    ports:\n      - \"{port}:80\"\n"
        ),
    )
    .expect("write strict compose profile");
    let mut cleanup = RealComposeCleanup {
        shim: shim.clone(),
        vat_home: vat_home.clone(),
        project: project.clone(),
        container_name: None,
        image_tag: None,
        active: true,
    };

    let up = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .args([
            "compose",
            "-f",
            compose.to_str().expect("UTF-8 compose path"),
            "-p",
            &project,
            "up",
            "-d",
        ])
        .output()
        .expect("docker compose up through VAT shim");
    assert!(
        up.status.success(),
        "compose up stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&up.stdout),
        String::from_utf8_lossy(&up.stderr)
    );
    assert!(
        String::from_utf8_lossy(&up.stdout).contains("vat_docker_compose"),
        "compose up must emit an agent-facing shim result: {}",
        String::from_utf8_lossy(&up.stdout)
    );
    let up_result = compose_shim_result(&up.stdout, "up");
    assert!(
        up_result.get("images").is_none() && up_result.get("cleanup_next").is_none(),
        "literal-image Compose up must not claim ownership of the caller image: {up_result}"
    );

    let vat_id = compose_vat_id(&vat_home, &project);
    let container_name = format!("{vat_id}-web");
    cleanup.container_name = Some(container_name.clone());

    let ps = wait_for_compose_ready(&shim, &vat_home, &project);
    assert!(
        ps.status.success(),
        "compose ps stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&ps.stdout),
        String::from_utf8_lossy(&ps.stderr)
    );
    assert!(
        String::from_utf8_lossy(&ps.stdout).contains("vat_docker_compose"),
        "compose ps must emit an agent-facing shim result: {}",
        String::from_utf8_lossy(&ps.stdout)
    );
    wait_for_http_ok(port).unwrap_or_else(|error| {
        panic!("Docker Compose shim published host endpoint never became HTTP-usable: {error}")
    });

    let exec = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .args([
            "compose",
            "-p",
            &project,
            "exec",
            "-T",
            "web",
            "--",
            "sh",
            "-ec",
            "printf vat-compose-exec-ok",
        ])
        .output()
        .expect("docker compose exec through VAT shim");
    assert!(
        exec.status.success(),
        "compose exec stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&exec.stdout),
        String::from_utf8_lossy(&exec.stderr)
    );
    let exec_stdout = String::from_utf8_lossy(&exec.stdout);
    assert!(
        exec_stdout.contains("vat-compose-exec-ok"),
        "compose exec must forward child stdout: {exec_stdout}"
    );
    assert!(
        exec_stdout.contains("\"command\":\"exec\""),
        "compose exec must emit an agent-facing shim result: {exec_stdout}"
    );
    assert!(
        exec_stdout.contains("vat-compose-exec-ok\n{"),
        "real Compose exec must put its terminal handoff on a separate line: {exec_stdout}"
    );
    assert_eq!(
        compose_shim_result(&exec.stdout, "exec").get("outcome"),
        Some(&serde_json::json!("completed"))
    );

    let failed_exec = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .args([
            "compose", "-p", &project, "exec", "-T", "web", "--", "sh", "-ec", "exit 37",
        ])
        .output()
        .expect("failing docker compose exec through VAT shim");
    assert_eq!(
        failed_exec.status.code(),
        Some(37),
        "compose exec must preserve child exit code; stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&failed_exec.stdout),
        String::from_utf8_lossy(&failed_exec.stderr)
    );
    let failed_exec_stdout = String::from_utf8_lossy(&failed_exec.stdout);
    assert!(
        failed_exec_stdout.contains("\"outcome\":\"failed\""),
        "failed compose exec must emit an agent-facing failure result: {failed_exec_stdout}"
    );
    assert!(
        failed_exec_stdout.contains("\"child_exit_code\":37"),
        "failed compose exec result must include the exact child exit code: {failed_exec_stdout}"
    );

    let logs = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .args(["compose", "-p", &project, "logs", "web"])
        .output()
        .expect("docker compose logs through VAT shim");
    assert!(
        logs.status.success(),
        "compose logs stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&logs.stdout),
        String::from_utf8_lossy(&logs.stderr)
    );

    let down = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .args(["compose", "-p", &project, "down"])
        .output()
        .expect("docker compose down through VAT shim");
    assert!(
        down.status.success(),
        "compose down stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&down.stdout),
        String::from_utf8_lossy(&down.stderr)
    );
    assert!(
        String::from_utf8_lossy(&down.stdout).contains("\"terminal\":\"cleaned_up\""),
        "compose down must emit the terminal cleanup result: {}",
        String::from_utf8_lossy(&down.stdout)
    );

    let absent = Command::new("container")
        .args(["inspect", &container_name])
        .output()
        .expect("confirm exact Apple Container compose cleanup");
    assert!(
        !absent.status.success(),
        "container {container_name} remained after docker compose down"
    );
    wait_for_port_to_close(port).unwrap_or_else(|error| {
        panic!("Docker Compose shim host port remained usable after down: {error}")
    });
    cleanup.active = false;
}

#[test]
#[ignore = "real Apple Container host-facing-independent-v1 Compose profile; run only with RUST_TEST_THREADS=1 VAT_DOCKER_COMPOSE_INDEPENDENT_SHIM_E2E_REQUIRED=1"]
fn apple_container_docker_compose_host_facing_independent_profile_contract() {
    if std::env::var("VAT_DOCKER_COMPOSE_INDEPENDENT_SHIM_E2E_REQUIRED").as_deref() != Ok("1") {
        eprintln!(
            "VAT_DOCKER_COMPOSE_INDEPENDENT_SHIM_E2E_REQUIRED=1 is required; skipping real independent Docker Compose shim probe"
        );
        return;
    }
    assert_eq!(
        std::env::var("RUST_TEST_THREADS").as_deref(),
        Ok("1"),
        "run this real dual-service Apple Container probe with RUST_TEST_THREADS=1"
    );

    let root = TempDir::new().expect("temp root");
    let shim_dir = root.path().join("shim-bin");
    let shim = install_real_shim(&shim_dir);
    let vat_home = root.path().join("vat-home");
    let (docs_port, inspector_port) = reserve_two_unique_loopback_ports();
    let project = format!("vat-compose-independent-shim-{docs_port}-{inspector_port}");
    let compose = root.path().join("compose.yml");
    fs::write(
        &compose,
        format!(
            "x-vat-compose-profile: host-facing-independent-v1\nservices:\n  docs:\n    image: nginx:1.27-alpine\n    ports:\n      - \"{docs_port}:80\"\n  inspector:\n    image: nginx:1.27-alpine\n    ports:\n      - \"{inspector_port}:80\"\n"
        ),
    )
    .expect("write real host-facing independent Compose profile");
    let mut cleanup = RealIndependentComposeCleanup {
        shim: shim.clone(),
        vat_home: vat_home.clone(),
        project: project.clone(),
        container_names: Vec::new(),
        active: true,
    };

    let up = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .args([
            "compose",
            "-f",
            compose.to_str().expect("UTF-8 compose path"),
            "-p",
            &project,
            "up",
            "-d",
            "--wait",
        ])
        .output()
        .expect("docker compose up -d --wait through VAT shim");
    assert!(
        up.status.success(),
        "independent compose up stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&up.stdout),
        String::from_utf8_lossy(&up.stderr)
    );
    let up_stdout = String::from_utf8_lossy(&up.stdout);
    let up_results = up_stdout
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|value| {
            value.get("type").and_then(serde_json::Value::as_str) == Some("vat_docker_compose")
                && value.get("command").and_then(serde_json::Value::as_str) == Some("up")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        up_results.len(),
        1,
        "--wait must emit one final Docker-shaped up result:\n{up_stdout}"
    );
    let up_result = &up_results[0];
    assert_eq!(
        up_result.get("profile").and_then(serde_json::Value::as_str),
        Some("host-facing-independent-v1")
    );
    assert_eq!(
        up_result
            .get("service_name_dns")
            .and_then(serde_json::Value::as_bool),
        Some(false),
        "the real probe must not imply unsupported Compose service-name DNS: {up_result}"
    );
    assert_eq!(
        up_result
            .get("host_loopback_only")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        up_result
            .get("wait")
            .and_then(|wait| wait.get("requested"))
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        up_result
            .get("wait")
            .and_then(|wait| wait.get("outcome"))
            .and_then(serde_json::Value::as_str),
        Some("ready"),
        "--wait must return VAT's final readiness observation: {up_result}"
    );
    let expected_topology = serde_json::json!({
        "phase": "ready",
        "ready": true,
        "services": [
            {
                "name": "docs",
                "state": "ready",
                "endpoint": format!("127.0.0.1:{docs_port}"),
            },
            {
                "name": "inspector",
                "state": "ready",
                "endpoint": format!("127.0.0.1:{inspector_port}"),
            },
        ],
    });
    assert_eq!(
        up_result.get("topology"),
        Some(&expected_topology),
        "--wait must expose only the final VAT-proven loopback topology: {up_result}"
    );

    let ps_json = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .args(["compose", "-p", &project, "ps", "--format=json"])
        .output()
        .expect("read real VAT-native JSON compose topology");
    assert!(
        ps_json.status.success(),
        "real compose ps --format json stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&ps_json.stdout),
        String::from_utf8_lossy(&ps_json.stderr)
    );
    let ps_json_stdout = String::from_utf8_lossy(&ps_json.stdout);
    assert_eq!(
        ps_json_stdout.lines().count(),
        1,
        "real JSON ps must not prepend a human table or append a second record: {ps_json_stdout}"
    );
    let ps_json_result: serde_json::Value =
        serde_json::from_str(ps_json_stdout.trim()).expect("parse real VAT-native compose ps JSON");
    assert_eq!(
        ps_json_result
            .get("schema")
            .and_then(serde_json::Value::as_str),
        Some("vat.docker-compose.ps.v1")
    );
    assert_eq!(
        ps_json_result
            .get("format")
            .and_then(serde_json::Value::as_str),
        Some("vat_json")
    );
    assert_eq!(
        ps_json_result.get("topology"),
        Some(&expected_topology),
        "real JSON ps must reuse the same provenance-validated topology proof"
    );

    for (service, port) in [("docs", docs_port), ("inspector", inspector_port)] {
        wait_for_http_ok(port).unwrap_or_else(|error| {
            panic!("real {service} loopback endpoint never became HTTP-usable: {error}")
        });
    }

    let vat_id = compose_vat_id(&vat_home, &project);
    let docs_name = format!("{vat_id}-docs");
    let inspector_name = format!("{vat_id}-inspector");
    cleanup.container_names = vec![docs_name.clone(), inspector_name.clone()];

    let docs_log_marker = format!("vat-compose-docs-log-{docs_port}");
    let inspector_log_marker = format!("vat-compose-inspector-log-{inspector_port}");
    request_http_path(docs_port, &format!("/{docs_log_marker}"))
        .unwrap_or_else(|error| panic!("send distinct docs log marker request: {error}"));
    request_http_path(inspector_port, &format!("/{inspector_log_marker}"))
        .unwrap_or_else(|error| panic!("send distinct inspector log marker request: {error}"));
    let docs_logs =
        wait_for_compose_log_marker(&shim, &vat_home, &project, "docs", &docs_log_marker);
    let inspector_logs = wait_for_compose_log_marker(
        &shim,
        &vat_home,
        &project,
        "inspector",
        &inspector_log_marker,
    );
    assert!(
        output_text(&docs_logs).contains(&docs_log_marker)
            && output_text(&inspector_logs).contains(&inspector_log_marker),
        "each service's Docker-shaped logs must preserve its own endpoint marker"
    );

    let docs_logs_json = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .args([
            "compose",
            "-p",
            &project,
            "logs",
            "--format=json",
            "--tail=100",
            "docs",
        ])
        .output()
        .expect("read one VAT-native JSON docs log snapshot");
    assert!(
        docs_logs_json.status.success(),
        "compose logs --format=json stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&docs_logs_json.stdout),
        String::from_utf8_lossy(&docs_logs_json.stderr)
    );
    let docs_logs_json_stdout = String::from_utf8_lossy(&docs_logs_json.stdout);
    assert_eq!(
        docs_logs_json_stdout.lines().count(),
        1,
        "real JSON logs must emit one VAT document without the raw text stream: {docs_logs_json_stdout}"
    );
    let docs_logs_result: serde_json::Value = serde_json::from_str(docs_logs_json_stdout.trim())
        .expect("parse real VAT-native Compose log JSON");
    assert_eq!(
        docs_logs_result
            .get("schema")
            .and_then(serde_json::Value::as_str),
        Some("vat.docker-compose.logs.v1")
    );
    assert_eq!(
        docs_logs_result.get("service"),
        Some(&serde_json::json!("docs"))
    );
    assert_eq!(
        docs_logs_result.get("tail_lines"),
        Some(&serde_json::json!(100))
    );
    assert!(
        ["stdout", "stderr"].iter().any(|stream| {
            docs_logs_result
                .get(*stream)
                .and_then(serde_json::Value::as_str)
                .is_some_and(|text| text.contains(&docs_log_marker))
        }),
        "the JSON snapshot must retain the exact docs marker: {docs_logs_result}"
    );
    assert_eq!(
        docs_logs_result.get("capture_only"),
        Some(&serde_json::Value::Bool(true))
    );
    assert_eq!(
        docs_logs_result.get("runtime_invoked"),
        Some(&serde_json::Value::Bool(false))
    );
    assert!(
        docs_logs_result.get("topology").is_none(),
        "a log read must not claim endpoint topology"
    );

    let docs_exec_marker = format!("vat-compose-docs-exec-{docs_port}");
    let docs_exec_command = format!("printf %s {docs_exec_marker}");
    let docs_exec = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .args([
            "compose",
            "-p",
            &project,
            "exec",
            "-T",
            "docs",
            "--",
            "sh",
            "-ec",
            &docs_exec_command,
        ])
        .output()
        .expect("Docker-shaped compose exec for docs");
    assert!(
        docs_exec.status.success() && output_text(&docs_exec).contains(&docs_exec_marker),
        "docs exec must forward its distinct marker:\n{}",
        output_text(&docs_exec)
    );
    let inspector_exec_marker = format!("vat-compose-inspector-exec-{inspector_port}");
    let inspector_exec_command = format!("printf %s {inspector_exec_marker}");
    let inspector_exec = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .args([
            "compose",
            "-p",
            &project,
            "exec",
            "-T",
            "inspector",
            "--",
            "sh",
            "-ec",
            &inspector_exec_command,
        ])
        .output()
        .expect("Docker-shaped compose exec for inspector");
    assert!(
        inspector_exec.status.success()
            && output_text(&inspector_exec).contains(&inspector_exec_marker),
        "inspector exec must forward its distinct marker:\n{}",
        output_text(&inspector_exec)
    );

    let docs_exec_json_marker = format!("vat-compose-docs-exec-json-{docs_port}");
    let docs_exec_json_command = format!(
        "printf %s {docs_exec_json_marker}; printf %s {docs_exec_json_marker} >&2"
    );
    let docs_exec_json = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .args([
            "compose",
            "-p",
            &project,
            "exec",
            "-T",
            "--format=json",
            "docs",
            "--",
            "sh",
            "-ec",
            &docs_exec_json_command,
        ])
        .output()
        .expect("Docker-shaped JSON compose exec for docs");
    assert!(
        docs_exec_json.status.success(),
        "docs JSON exec stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&docs_exec_json.stdout),
        String::from_utf8_lossy(&docs_exec_json.stderr)
    );
    assert!(
        docs_exec_json.stderr.is_empty(),
        "JSON exec must capture child stderr in its one VAT document:\n{}",
        output_text(&docs_exec_json)
    );
    let docs_exec_json_stdout = String::from_utf8_lossy(&docs_exec_json.stdout);
    assert_eq!(
        docs_exec_json_stdout.lines().count(),
        1,
        "real JSON exec must emit exactly one VAT document: {docs_exec_json_stdout}"
    );
    let docs_exec_json_result: serde_json::Value = serde_json::from_str(docs_exec_json_stdout.trim())
        .expect("parse one real VAT-native Compose exec JSON document");
    assert_eq!(
        docs_exec_json_result.get("schema"),
        Some(&serde_json::json!("vat.docker-compose.exec.v1"))
    );
    assert_eq!(
        docs_exec_json_result.get("format"),
        Some(&serde_json::json!("vat_json"))
    );
    assert_eq!(
        docs_exec_json_result.get("service"),
        Some(&serde_json::json!("docs"))
    );
    assert_eq!(
        docs_exec_json_result.get("outcome"),
        Some(&serde_json::json!("completed"))
    );
    assert_eq!(
        docs_exec_json_result.get("child_exit_code"),
        Some(&serde_json::json!(0))
    );
    assert!(
        docs_exec_json_result
            .get("stdout")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|stdout| stdout.contains(&docs_exec_json_marker)),
        "real JSON exec must retain its stdout marker: {docs_exec_json_result}"
    );
    assert!(
        docs_exec_json_result
            .get("stderr")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|stderr| stderr.contains(&docs_exec_json_marker)),
        "real JSON exec must retain its stderr marker: {docs_exec_json_result}"
    );
    assert_eq!(
        docs_exec_json_result.get("profile"),
        Some(&serde_json::json!("host-facing-independent-v1"))
    );
    assert_eq!(
        docs_exec_json_result.get("runtime_invoked"),
        Some(&serde_json::Value::Bool(true))
    );
    assert_eq!(
        docs_exec_json_result.get("compose_record_mutated"),
        Some(&serde_json::Value::Bool(false))
    );

    let down = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .args(["compose", "-p", &project, "down"])
        .output()
        .expect("Docker-shaped compose down for independent profile");
    assert!(
        down.status.success(),
        "independent compose down stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&down.stdout),
        String::from_utf8_lossy(&down.stderr)
    );
    assert!(
        String::from_utf8_lossy(&down.stdout).contains("\"terminal\":\"cleaned_up\""),
        "compose down must emit terminal cleanup evidence: {}",
        String::from_utf8_lossy(&down.stdout)
    );

    for name in [&docs_name, &inspector_name] {
        let absent = Command::new("container")
            .args(["inspect", name])
            .output()
            .expect("confirm exact Apple Container independent-service cleanup");
        assert!(
            !absent.status.success(),
            "container {name} remained after docker compose down"
        );
    }
    assert_port_closed_and_bindable(docs_port);
    assert_port_closed_and_bindable(inspector_port);

    let record_path = vat_home.join("compose").join(&project).join("project.json");
    let record: serde_json::Value = serde_json::from_slice(
        &fs::read(&record_path).expect("read released independent Compose registry"),
    )
    .expect("parse released independent Compose registry");
    assert_eq!(
        record.get("status").and_then(serde_json::Value::as_str),
        Some("imported")
    );
    assert!(
        record.get("vat_id").is_some_and(serde_json::Value::is_null),
        "compose down must release the independent profile registry: {record}"
    );
    cleanup.active = false;
}

#[test]
#[ignore = "real Apple Container strict Docker Compose source-build profile; run only with VAT_DOCKER_COMPOSE_BUILD_SHIM_E2E_REQUIRED=1"]
fn apple_container_docker_compose_strict_build_profile_contract() {
    if std::env::var("VAT_DOCKER_COMPOSE_BUILD_SHIM_E2E_REQUIRED").as_deref() != Ok("1") {
        eprintln!(
            "VAT_DOCKER_COMPOSE_BUILD_SHIM_E2E_REQUIRED=1 is required; skipping real Docker Compose build shim probe"
        );
        return;
    }

    let root = TempDir::new().expect("temp root");
    let shim_dir = root.path().join("shim-bin");
    let shim = install_real_shim(&shim_dir);
    let vat_home = root.path().join("vat-home");
    let port = TcpListener::bind("127.0.0.1:0")
        .expect("select currently free host port")
        .local_addr()
        .expect("selected host port")
        .port();
    let project = format!("vat-compose-build-shim-{port}");
    let dockerfile = root.path().join("Dockerfile");
    fs::write(
        &dockerfile,
        "FROM nginx:1.27-alpine\nRUN printf 'vat-compose-build-ok\\n' > /usr/share/nginx/html/index.html\n",
    )
    .expect("write strict Compose build Dockerfile");
    let compose = root.path().join("compose.yml");
    fs::write(
        &compose,
        format!(
            "services:\n  web:\n    build: .\n    ports:\n      - \"{port}:80\"\n    environment:\n      NGINX_ENTRYPOINT_QUIET_LOGS: \"1\"\n"
        ),
    )
    .expect("write strict Compose source-build profile");
    let mut cleanup = RealComposeCleanup {
        shim: shim.clone(),
        vat_home: vat_home.clone(),
        project: project.clone(),
        container_name: None,
        image_tag: None,
        active: true,
    };

    let up = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .args([
            "compose",
            "-f",
            compose.to_str().expect("UTF-8 compose path"),
            "-p",
            &project,
            "up",
            "-d",
            "--build",
        ])
        .output()
        .expect("docker compose source-build up through VAT shim");
    assert!(
        up.status.success(),
        "compose source-build up stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&up.stdout),
        String::from_utf8_lossy(&up.stderr)
    );
    assert!(
        String::from_utf8_lossy(&up.stdout).contains("vat_docker_compose"),
        "compose source-build up must emit an agent-facing shim result: {}",
        String::from_utf8_lossy(&up.stdout)
    );
    let up_result = compose_shim_result(&up.stdout, "up");
    let images = up_result
        .get("images")
        .and_then(serde_json::Value::as_array)
        .expect("source-build up result must expose VAT-owned images");
    assert_eq!(
        images.len(),
        1,
        "strict source-build up must expose exactly one image: {up_result}"
    );
    let image_tag = images[0]
        .as_str()
        .expect("source-build image tag must be a string")
        .to_string();
    let cleanup_next = up_result
        .get("cleanup_next")
        .and_then(serde_json::Value::as_str)
        .expect("source-build up result must expose an exact cleanup command")
        .to_string();
    assert_eq!(
        cleanup_next,
        format!("docker compose -p {project} down && docker image rm {image_tag}"),
        "source-build cleanup must remove only the exact public image"
    );
    cleanup.image_tag = Some(image_tag.clone());
    let image = Command::new("container")
        .args(["image", "inspect", &image_tag])
        .output()
        .expect("inspect exact source-build image");
    assert!(
        image.status.success(),
        "materialized Apple image {image_tag} is missing: {}",
        String::from_utf8_lossy(&image.stderr)
    );

    let vat_id = compose_vat_id(&vat_home, &project);
    let container_name = format!("{vat_id}-web");
    cleanup.container_name = Some(container_name.clone());

    let ps = wait_for_compose_ready(&shim, &vat_home, &project);
    assert!(
        ps.status.success(),
        "compose source-build ps stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&ps.stdout),
        String::from_utf8_lossy(&ps.stderr)
    );
    wait_for_http_ok(port).unwrap_or_else(|error| {
        panic!("Docker Compose source-build host endpoint never became HTTP-usable: {error}")
    });

    let exec = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .args([
            "compose",
            "-p",
            &project,
            "exec",
            "-T",
            "web",
            "--",
            "sh",
            "-ec",
            "test \"$(cat /usr/share/nginx/html/index.html)\" = vat-compose-build-ok",
        ])
        .output()
        .expect("docker compose exec against source-build profile");
    assert!(
        exec.status.success(),
        "source-build compose exec stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&exec.stdout),
        String::from_utf8_lossy(&exec.stderr)
    );

    let logs = Command::new(&shim)
        .env("VAT_HOME", &vat_home)
        .args(["compose", "-p", &project, "logs", "web"])
        .output()
        .expect("docker compose logs for source-build profile");
    assert!(
        logs.status.success(),
        "source-build compose logs stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&logs.stdout),
        String::from_utf8_lossy(&logs.stderr)
    );

    let cleanup_result = Command::new("/bin/sh")
        .arg("-c")
        .arg(&cleanup_next)
        .env("VAT_HOME", &vat_home)
        .env(
            "PATH",
            path_with_prepend(shim.parent().expect("shim parent directory")),
        )
        .output()
        .expect("run public source-build cleanup command");
    assert!(
        cleanup_result.status.success(),
        "public source-build cleanup stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&cleanup_result.stdout),
        String::from_utf8_lossy(&cleanup_result.stderr)
    );
    assert_eq!(
        compose_shim_result(&cleanup_result.stdout, "down")
            .get("terminal")
            .and_then(serde_json::Value::as_str),
        Some("cleaned_up"),
        "public cleanup must retain the compose down terminal result"
    );

    let absent = Command::new("container")
        .args(["inspect", &container_name])
        .output()
        .expect("confirm exact Apple Container source-build cleanup");
    assert!(
        !absent.status.success(),
        "container {container_name} remained after docker compose source-build down"
    );
    wait_for_port_to_close(port).unwrap_or_else(|error| {
        panic!("Docker Compose source-build host port remained usable after down: {error}")
    });

    let deleted_image = Command::new("container")
        .args(["image", "inspect", &image_tag])
        .output()
        .expect("confirm public cleanup removed exact source-build image");
    assert!(
        !deleted_image.status.success(),
        "public cleanup left source-build image {image_tag}: {}",
        String::from_utf8_lossy(&deleted_image.stderr)
    );
    cleanup.image_tag = None;
    cleanup.active = false;
}

#[test]
#[ignore = "real Apple Container Docker-command shim build contract; run only with VAT_DOCKER_SHIM_E2E_REQUIRED=1"]
fn apple_container_docker_build_contract() {
    if std::env::var("VAT_DOCKER_SHIM_E2E_REQUIRED").as_deref() != Ok("1") {
        eprintln!("VAT_DOCKER_SHIM_E2E_REQUIRED=1 is required; skipping real Docker shim build");
        return;
    }

    let root = TempDir::new().expect("temp root");
    let shim_dir = root.path().join("shim-bin");
    let shim = install_real_shim(&shim_dir);
    let context = root.path().join("build-context");
    fs::create_dir_all(&context).expect("create build context");
    fs::write(
        context.join("Dockerfile"),
        "FROM alpine:3.20\nARG MESSAGE=vat\nCMD [\"sh\", \"-c\", \"echo $MESSAGE\"]\n",
    )
    .expect("write Dockerfile");
    let tag = format!("vat-docker-shim-build-{}", std::process::id());
    let mut cleanup = RealImageCleanup {
        tag: tag.clone(),
        active: true,
    };

    let build = Command::new(&shim)
        .args([
            "build",
            "--file",
            context
                .join("Dockerfile")
                .to_str()
                .expect("UTF-8 Dockerfile path"),
            "--tag",
            &tag,
            "--build-arg",
            "MESSAGE=vat-docker-shim-e2e",
            context.to_str().expect("UTF-8 build context"),
        ])
        .output()
        .expect("docker build through VAT shim");
    assert!(
        build.status.success(),
        "docker build stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
    let inspect = Command::new(&shim)
        .args(["image", "inspect", &tag])
        .output()
        .expect("docker image inspect through VAT shim");
    assert!(
        inspect.status.success(),
        "image inspect stderr: {}",
        String::from_utf8_lossy(&inspect.stderr)
    );
    let remove = Command::new(&shim)
        .args(["image", "rm", &tag])
        .output()
        .expect("docker image rm through VAT shim");
    assert!(
        remove.status.success(),
        "image rm stderr: {}",
        String::from_utf8_lossy(&remove.stderr)
    );
    cleanup.active = false;
}
