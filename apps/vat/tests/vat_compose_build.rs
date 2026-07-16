// HANDWRITE-BEGIN gap="missing-generator:e2e-test:compose-build-runtime-local" tracker="#1529" reason="Deterministic paired fake Docker and Apple Container stores prove compose build path resolution, runtime-local image ownership, scoped tags, build args, and fail-before-materialization behavior without requiring either host runtime."

//! Regression coverage for compose build artifacts.
//!
//! The test-local Docker and Apple Container executables each maintain a
//! separate image list. Their run command refuses an image built by the other
//! fake, which makes cross-store regressions observable without a real Docker
//! daemon or Apple Container installation.

use serde_json::Value;
use std::ffi::OsString;
use std::fs;
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

fn vat_bin() -> &'static str {
    env!("CARGO_BIN_EXE_vat")
}

#[derive(Clone, Copy)]
enum FakeStore {
    Docker,
    MicroVm,
}

impl FakeStore {
    fn runtime_arg(self) -> &'static str {
        match self {
            Self::Docker => "docker",
            Self::MicroVm => "micro-vm",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Docker => "docker",
            Self::MicroVm => "microvm",
        }
    }

    fn other(self) -> Self {
        match self {
            Self::Docker => Self::MicroVm,
            Self::MicroVm => Self::Docker,
        }
    }

    fn calls(self, fakes: &FakeRuntimes) -> String {
        match self {
            Self::Docker => fakes.docker_calls(),
            Self::MicroVm => fakes.container_calls(),
        }
    }
}

struct FakeRuntimes {
    bin: TempDir,
    state: TempDir,
    docker_log: PathBuf,
    container_log: PathBuf,
}

impl FakeRuntimes {
    fn new() -> Self {
        let bin = TempDir::new().expect("create fake runtime bin");
        let state = TempDir::new().expect("create fake runtime state");
        let docker_log = state.path().join("docker.log");
        let container_log = state.path().join("container.log");
        fs::write(&docker_log, "").expect("initialize Docker log");
        fs::write(&container_log, "").expect("initialize container log");
        write_executable(&bin.path().join("docker"), fake_docker_script());
        write_executable(&bin.path().join("container"), fake_container_script());
        Self {
            bin,
            state,
            docker_log,
            container_log,
        }
    }

    fn command(&self, vat_home: &Path, cwd: &Path) -> Command {
        let mut command = Command::new(vat_bin());
        command
            .current_dir(cwd)
            .env("VAT_HOME", vat_home)
            .env("PATH", path_with_fake_runtimes(self.bin.path()))
            .env("VAT_FAKE_RUNTIME_STATE", self.state.path())
            .env("VAT_FAKE_DOCKER_LOG", &self.docker_log)
            .env("VAT_FAKE_CONTAINER_LOG", &self.container_log);
        command
    }

    fn docker_calls(&self) -> String {
        fs::read_to_string(&self.docker_log).expect("read fake Docker log")
    }

    fn container_calls(&self) -> String {
        fs::read_to_string(&self.container_log).expect("read fake container log")
    }

    fn seed_image(&self, store: FakeStore, image: &str) {
        let images = self.state.path().join(match store {
            FakeStore::Docker => "docker-images",
            FakeStore::MicroVm => "container-images",
        });
        let mut content = fs::read_to_string(&images).unwrap_or_default();
        content.push_str(image);
        content.push('\n');
        fs::write(images, content).expect("seed fake runtime image");
    }
}

fn write_executable(path: &Path, contents: &str) {
    fs::write(path, contents).expect("write fake runtime");
    let mut permissions = fs::metadata(path)
        .expect("fake runtime metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("make fake runtime executable");
}

fn path_with_fake_runtimes(bin_dir: &Path) -> OsString {
    let mut paths = vec![bin_dir.to_path_buf()];
    if let Some(path) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&path));
    }
    std::env::join_paths(paths).expect("join fake runtime PATH")
}

fn fake_docker_script() -> &'static str {
    r#"#!/bin/sh
set -eu

printf '%s\n' "$*" >> "$VAT_FAKE_DOCKER_LOG"
images="$VAT_FAKE_RUNTIME_STATE/docker-images"

if [ "$#" -eq 0 ]; then
  exit 2
fi

case "$1" in
  info)
    exit 0
    ;;
  build)
    tag=""
    shift
    while [ "$#" -gt 0 ]; do
      if [ "$1" = "-t" ]; then
        shift
        if [ "$#" -eq 0 ]; then
          exit 41
        fi
        tag="$1"
      fi
      shift
    done
    if [ -z "$tag" ]; then
      exit 42
    fi
    printf '%s\n' "$tag" >> "$images"
    ;;
  run)
    image=""
    for arg in "$@"; do
      image="$arg"
    done
    found=0
    if [ -f "$images" ]; then
      while IFS= read -r known; do
        if [ "$known" = "$image" ]; then
          found=1
          break
        fi
      done < "$images"
    fi
    if [ "$found" -ne 1 ]; then
      echo "Docker fake has no image $image" >&2
      exit 73
    fi
    exec /bin/sleep 30
    ;;
  rm)
    exit 0
    ;;
  container)
    if [ "$#" -ge 2 ] && [ "$2" = "ls" ]; then
      exit 0
    fi
    exit 2
    ;;
  *)
    exit 2
    ;;
esac
"#
}

fn fake_container_script() -> &'static str {
    r#"#!/bin/sh
set -eu

printf '%s\n' "$*" >> "$VAT_FAKE_CONTAINER_LOG"
images="$VAT_FAKE_RUNTIME_STATE/container-images"

if [ "$#" -eq 0 ]; then
  exit 2
fi

case "$1" in
  --version)
    echo "fake-container 1.0"
    ;;
  system)
    if [ "$#" -lt 2 ] || [ "$2" != "status" ]; then
      exit 2
    fi
    ;;
  build)
    tag=""
    shift
    while [ "$#" -gt 0 ]; do
      if [ "$1" = "-t" ]; then
        shift
        if [ "$#" -eq 0 ]; then
          exit 41
        fi
        tag="$1"
      fi
      shift
    done
    if [ -z "$tag" ]; then
      exit 42
    fi
    printf '%s\n' "$tag" >> "$images"
    ;;
  image)
    if [ "$#" -ne 3 ] || [ "$2" != "inspect" ]; then
      exit 2
    fi
    found=0
    if [ -f "$images" ]; then
      while IFS= read -r known; do
        if [ "$known" = "$3" ]; then
          found=1
          break
        fi
      done < "$images"
    fi
    if [ "$found" -ne 1 ]; then
      exit 1
    fi
    printf '{}\n'
    ;;
  run)
    image=""
    for arg in "$@"; do
      image="$arg"
    done
    found=0
    if [ -f "$images" ]; then
      while IFS= read -r known; do
        if [ "$known" = "$image" ]; then
          found=1
          break
        fi
      done < "$images"
    fi
    if [ "$found" -ne 1 ]; then
      echo "Apple Container fake has no image $image" >&2
      exit 73
    fi
    exec /bin/sleep 30
    ;;
  list)
    printf '[]\n'
    ;;
  inspect)
    printf '{}\n'
    ;;
  rm)
    exit 0
    ;;
  *)
    exit 2
    ;;
esac
"#
}

fn assert_success(output: &Output, operation: &str) {
    assert!(
        output.status.success(),
        "{operation} failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn import_with_fakes(
    fakes: &FakeRuntimes,
    vat_home: &Path,
    cwd: &Path,
    compose_file: &Path,
    project: &str,
    runtime: &str,
) -> Output {
    fakes
        .command(vat_home, cwd)
        .arg("compose")
        .arg("import")
        .arg(compose_file)
        .args(["--project", project, "--runtime", runtime])
        .output()
        .expect("run vat compose import")
}

fn write_dockerfile(path: &Path) {
    fs::write(path, "FROM scratch\n").expect("write Dockerfile");
}

fn write_single_build_service_compose(path: &Path, service_id: &str) {
    fs::write(
        path,
        format!("services:\n  \"{service_id}\":\n    build: .\n"),
    )
    .expect("write single-service compose file");
}

fn build_tags(calls: &str) -> Vec<String> {
    calls
        .lines()
        .filter(|line| line.starts_with("build "))
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            while let Some(word) = words.next() {
                if word == "-t" {
                    return words.next().map(str::to_string);
                }
            }
            None
        })
        .collect()
}

fn valid_local_image_tag(tag: &str) -> bool {
    let Some((name, version)) = tag.split_once(':') else {
        return false;
    };
    !name.is_empty()
        && !version.is_empty()
        && name.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '.' | '_' | '-')
        })
        && version.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '.' | '_' | '-')
        })
}

fn assert_build_invocation(
    calls: &str,
    dockerfile: &Path,
    context: &Path,
    expected_args: &[&str],
) -> String {
    let prefix = format!("build -f {} -t ", dockerfile.display());
    let suffix = format!(" {}", context.display());
    let line = calls
        .lines()
        .find(|line| line.starts_with(&prefix) && line.ends_with(&suffix))
        .unwrap_or_else(|| {
            panic!(
                "missing build invocation for Dockerfile {} and context {}:\n{}",
                dockerfile.display(),
                context.display(),
                calls
            )
        });
    let body = line
        .strip_prefix(&prefix)
        .and_then(|value| value.strip_suffix(&suffix))
        .expect("matched build line retains tag segment");
    let mut words = body.split_whitespace();
    let tag = words.next().expect("build tag").to_string();
    let mut actual_args = Vec::new();
    while let Some(flag) = words.next() {
        assert_eq!(flag, "--build-arg", "unexpected build token in {line}");
        actual_args.push(words.next().expect("value after --build-arg"));
    }
    assert!(
        valid_local_image_tag(&tag),
        "build tag must be OCI-safe: {tag}"
    );
    assert_eq!(
        actual_args, expected_args,
        "build args differ for {dockerfile:?}: {line}"
    );
    tag
}

fn wait_for_compose_ready(
    fakes: &FakeRuntimes,
    vat_home: &Path,
    cwd: &Path,
    project: &str,
    calls: &str,
) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let output = fakes
            .command(vat_home, cwd)
            .args(["compose", "ps", project])
            .output()
            .expect("run compose ps");
        let latest_ps = output_text(&output);
        if output.status.success() && latest_ps.contains("is ready") {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "compose project {project} never became ready:\nlast ps output:\n{latest_ps}\nruntime calls:\n{}",
            calls
        );
        thread::sleep(Duration::from_millis(25));
    }
}

fn spawn_ready_listener() -> (u16, thread::JoinHandle<bool>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake runtime readiness port");
    let port = listener
        .local_addr()
        .expect("read fake runtime readiness address")
        .port();
    let server = thread::spawn(move || {
        listener
            .set_nonblocking(true)
            .expect("make fake readiness listener nonblocking");
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    // The MicroVM readiness probe must see an endpoint that
                    // stays open after the handshake; Docker's probe also
                    // succeeds against this listener.
                    let _stream = stream;
                    thread::sleep(Duration::from_millis(600));
                    return true;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        return false;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("accept fake readiness connection: {error}"),
            }
        }
    });
    (port, server)
}

fn assert_runtime_local_build_and_run(store: FakeStore, runtime: &str) {
    let fakes = FakeRuntimes::new();
    let vat_home = TempDir::new().expect("create VAT_HOME");
    let project_dir = TempDir::new().expect("create compose project");
    let unrelated_cwd = TempDir::new().expect("create unrelated caller cwd");
    let (port, readiness_server) = spawn_ready_listener();
    let project = format!("runtime-{}-{}", store.label(), runtime);
    let compose_file = project_dir.path().join("docker-compose.yml");
    let dockerfile = project_dir.path().join("Dockerfile");
    write_dockerfile(&dockerfile);
    fs::write(
        &compose_file,
        format!("services:\n  web:\n    build: .\n    ports:\n      - \"{port}:80\"\n"),
    )
    .expect("write compose file");

    let imported = import_with_fakes(
        &fakes,
        vat_home.path(),
        unrelated_cwd.path(),
        &compose_file,
        &project,
        runtime,
    );
    assert_success(&imported, "compose import");

    let context = fs::canonicalize(project_dir.path()).expect("canonical build context");
    let dockerfile = fs::canonicalize(&dockerfile).expect("canonical Dockerfile");
    let selected_before_up = store.calls(&fakes);
    let tag = assert_build_invocation(&selected_before_up, &dockerfile, &context, &[]);
    let other_before_up = store.other().calls(&fakes);
    assert!(
        !other_before_up
            .lines()
            .any(|line| line.starts_with("build ")),
        "unselected {} builder must not build the image:\n{}",
        store.other().label(),
        other_before_up
    );

    let up = fakes
        .command(vat_home.path(), unrelated_cwd.path())
        .args(["compose", "up", "--project", &project, "--detach"])
        .output()
        .expect("run compose up");
    assert_success(&up, "compose up");

    wait_for_compose_ready(
        &fakes,
        vat_home.path(),
        unrelated_cwd.path(),
        &project,
        &store.calls(&fakes),
    );

    let selected_after_up = store.calls(&fakes);
    assert!(
        selected_after_up
            .lines()
            .any(|line| line.starts_with("run ") && line.ends_with(&tag)),
        "selected {} runtime did not run its own built tag:\n{}",
        store.label(),
        selected_after_up
    );
    if matches!(store, FakeStore::MicroVm) {
        assert!(
            selected_after_up
                .lines()
                .any(|line| line == format!("image inspect {tag}")),
            "Apple Container must inspect the just-built local image before running it:\n{selected_after_up}",
        );
        assert!(
            !selected_after_up
                .lines()
                .any(|line| line == format!("image pull {tag}")),
            "a just-built Apple Container image must not be pulled from a registry:\n{selected_after_up}",
        );
    }
    let other_after_up = store.other().calls(&fakes);
    assert!(
        !other_after_up
            .lines()
            .any(|line| line.starts_with("build ") || line.starts_with("run ")),
        "unselected {} runtime must not build or run this service:\n{}",
        store.other().label(),
        other_after_up
    );

    let down = fakes
        .command(vat_home.path(), unrelated_cwd.path())
        .args(["compose", "down", &project])
        .output()
        .expect("run compose down");
    assert_success(&down, "compose down");
    assert!(
        readiness_server.join().expect("join fake readiness server"),
        "VAT never probed the fake runtime endpoint"
    );
}

#[test]
fn compose_build_paths_are_canonical_and_relative_to_the_source_file() {
    let fakes = FakeRuntimes::new();
    let vat_home = TempDir::new().expect("create VAT_HOME");
    let compose_root = TempDir::new().expect("create compose root");
    let unrelated_cwd = TempDir::new().expect("create unrelated caller cwd");
    let nested_context = compose_root.path().join("nested-context");
    fs::create_dir_all(&nested_context).expect("create nested context");
    write_dockerfile(&compose_root.path().join("Dockerfile"));
    let custom_dockerfile = compose_root.path().join("Custom.Dockerfile");
    write_dockerfile(&custom_dockerfile);
    let compose_file = compose_root.path().join("docker-compose.yml");
    fs::write(
        &compose_file,
        r#"services:
  short:
    build: .
  mapping:
    build:
      context: nested-context
      dockerfile: Custom.Dockerfile
      args:
        BETA: two
        ALPHA: one
"#,
    )
    .expect("write compose file");

    let imported = import_with_fakes(
        &fakes,
        vat_home.path(),
        unrelated_cwd.path(),
        &compose_file,
        "source-paths",
        "docker",
    );
    assert_success(&imported, "source-relative compose import");

    let source = fs::canonicalize(compose_root.path()).expect("canonical compose root");
    let nested = fs::canonicalize(&nested_context).expect("canonical nested context");
    let default_dockerfile =
        fs::canonicalize(source.join("Dockerfile")).expect("canonical default Dockerfile");
    let custom_dockerfile =
        fs::canonicalize(&custom_dockerfile).expect("canonical custom Dockerfile");
    let calls = fakes.docker_calls();
    let short_tag = assert_build_invocation(&calls, &default_dockerfile, &source, &[]);
    let mapping_tag = assert_build_invocation(
        &calls,
        &custom_dockerfile,
        &nested,
        &["ALPHA=one", "BETA=two"],
    );
    assert_ne!(
        short_tag, mapping_tag,
        "separate services in one project need distinct tags"
    );
    assert!(
        fakes.container_calls().trim().is_empty(),
        "Docker import must not invoke the Apple Container builder:\n{}",
        fakes.container_calls()
    );
}

#[test]
fn compose_docker_builds_and_runs_from_docker_store() {
    assert_runtime_local_build_and_run(FakeStore::Docker, FakeStore::Docker.runtime_arg());
}

#[test]
fn compose_microvm_builds_and_runs_from_apple_container_store() {
    assert_runtime_local_build_and_run(FakeStore::MicroVm, FakeStore::MicroVm.runtime_arg());
}

#[test]
fn compose_auto_builds_and_runs_from_docker_store_deterministically() {
    assert_runtime_local_build_and_run(FakeStore::Docker, "auto");
}

#[test]
fn compose_mapping_build_args_reach_the_selected_microvm_builder() {
    let fakes = FakeRuntimes::new();
    let vat_home = TempDir::new().expect("create VAT_HOME");
    let compose_root = TempDir::new().expect("create compose root");
    let unrelated_cwd = TempDir::new().expect("create unrelated caller cwd");
    let dockerfile = compose_root.path().join("Dockerfile");
    write_dockerfile(&dockerfile);
    let compose_file = compose_root.path().join("docker-compose.yml");
    fs::write(
        &compose_file,
        r#"services:
  web:
    build:
      context: .
      dockerfile: Dockerfile
      args:
        VERSION: "1.2.3"
        DEBUG: "true"
"#,
    )
    .expect("write compose file");

    let imported = import_with_fakes(
        &fakes,
        vat_home.path(),
        unrelated_cwd.path(),
        &compose_file,
        "args-microvm",
        "micro-vm",
    );
    assert_success(&imported, "MicroVM args compose import");

    let context = fs::canonicalize(compose_root.path()).expect("canonical context");
    let dockerfile = fs::canonicalize(&dockerfile).expect("canonical Dockerfile");
    let calls = fakes.container_calls();
    assert_build_invocation(
        &calls,
        &dockerfile,
        &context,
        &["DEBUG=true", "VERSION=1.2.3"],
    );
    assert!(
        fakes.docker_calls().trim().is_empty(),
        "MicroVM import must not invoke Docker:\n{}",
        fakes.docker_calls()
    );
}

#[test]
fn compose_build_tags_are_project_scoped_sanitized_and_deterministic() {
    let fakes = FakeRuntimes::new();
    let vat_home = TempDir::new().expect("create VAT_HOME");
    let compose_root = TempDir::new().expect("create compose root");
    let unrelated_cwd = TempDir::new().expect("create unrelated caller cwd");
    let dockerfile = compose_root.path().join("Dockerfile");
    write_dockerfile(&dockerfile);
    let compose_file = compose_root.path().join("docker-compose.yml");
    let identities = [
        ("a--b", "Web API!"),
        ("a-b", "Web API!"),
        ("é", "Web API!"),
        ("project", "Web API!"),
        ("a", "b-c"),
        ("a-b", "c"),
        ("a--b", "Web API!"),
    ];

    for (project, service_id) in identities {
        write_single_build_service_compose(&compose_file, service_id);
        let imported = import_with_fakes(
            &fakes,
            vat_home.path(),
            unrelated_cwd.path(),
            &compose_file,
            project,
            "docker",
        );
        assert_success(&imported, "project-scoped tag compose import");
    }

    let tags = build_tags(&fakes.docker_calls());
    assert_eq!(tags.len(), 7, "one build tag per import: {tags:?}");
    assert!(
        tags.iter().all(|tag| valid_local_image_tag(tag)),
        "generated tags must be OCI-safe: {tags:?}"
    );
    assert_ne!(
        tags[0], tags[1],
        "a--b and a-b must not share a service image tag: {tags:?}"
    );
    assert_ne!(
        tags[2], tags[3],
        "Unicode project names must not collapse onto ASCII project names: {tags:?}"
    );
    assert_ne!(
        tags[4], tags[5],
        "a/b-c and a-b/c must not share a delimiter-ambiguous tag: {tags:?}"
    );
    assert_eq!(
        tags[0], tags[6],
        "identical project and service input must remain deterministic: {tags:?}"
    );
}

#[test]
fn unavailable_selected_builder_fails_before_compose_materialization_with_remediation() {
    for (runtime, unavailable, remedy) in [
        ("docker", "Docker builder unavailable", "docker info"),
        (
            "micro-vm",
            "Apple Container CLI not found on PATH",
            "brew install container",
        ),
    ] {
        let vat_home = TempDir::new().expect("create VAT_HOME");
        let compose_root = TempDir::new().expect("create compose root");
        let unrelated_cwd = TempDir::new().expect("create unrelated caller cwd");
        let empty_bin = TempDir::new().expect("create empty PATH directory");
        let dockerfile = compose_root.path().join("Dockerfile");
        write_dockerfile(&dockerfile);
        let compose_file = compose_root.path().join("docker-compose.yml");
        fs::write(&compose_file, "services:\n  web:\n    build: .\n").expect("write compose file");
        let project = format!("unavailable-{}", runtime.replace('-', ""));

        let output = Command::new(vat_bin())
            .current_dir(unrelated_cwd.path())
            .env("VAT_HOME", vat_home.path())
            .env("PATH", empty_bin.path())
            .arg("compose")
            .arg("import")
            .arg(&compose_file)
            .args(["--project", &project, "--runtime", runtime])
            .output()
            .expect("run unavailable-builder compose import");
        assert!(
            !output.status.success(),
            "compose import must fail when {runtime} is unavailable"
        );
        let text = output_text(&output);
        assert!(
            text.contains(unavailable)
                && text.contains(remedy)
                && text.contains("vat compose import")
                && text.contains(&format!("--runtime {runtime}")),
            "unavailable runtime error must be actionable:\n{text}"
        );

        let registry = vat_home.path().join("compose").join(&project);
        assert!(
            !registry.join("vat.toml").exists(),
            "failed preflight must not materialize vat.toml"
        );
        assert!(
            !registry.join("project.json").exists(),
            "failed preflight must not create a compose record"
        );
    }
}

fn write_user_edited_replacement_vat_toml(registry: &Path, port: u16) {
    fs::write(
        registry.join("vat.toml"),
        format!(
            r#"version = 1

[workspace]
keep = "always"

# User-edited after the original compose import. It remains valid because the
# registry contract owns service identity, not a byte-for-byte config digest.
[[services]]
id = "replacement"
image = "fake:replacement"
runtime = "docker"
container_port = 80
port = {port}
timeout_s = 2

[[runners]]
id = "project.up"
requires = ["replacement"]
cmd = ["sleep", "2147483647"]
"#
        ),
    )
    .expect("write replacement vat.toml");
}

fn write_imported_registry(registry: &Path, project: &str, service_ids: &[&str]) {
    fs::write(
        registry.join("project.json"),
        serde_json::to_vec(&serde_json::json!({
            "project": project,
            "vat_id": null,
            "handoff_protocol": 1,
            "service_ids": service_ids,
            "status": "imported",
            "created_at": "2026-01-01T00:00:00Z",
        }))
        .expect("serialize compose registry"),
    )
    .expect("write compose registry");
}

#[test]
fn compose_up_rejects_stale_registry_before_starting_a_runtime() {
    let fakes = FakeRuntimes::new();
    let vat_home = TempDir::new().expect("create VAT_HOME");
    let caller = TempDir::new().expect("create caller directory");
    let project = "registry-config-mismatch";
    let registry = vat_home.path().join("compose").join(project);
    fs::create_dir_all(&registry).expect("create seeded compose registry");

    // This is a valid replacement config from a later import. The retained
    // registry describes a different service set, exactly the crash window a
    // split vat.toml/project.json publish must fail closed instead of launching.
    write_user_edited_replacement_vat_toml(&registry, 41880);
    write_imported_registry(&registry, project, &["old-service"]);

    let output = fakes
        .command(vat_home.path(), caller.path())
        .args(["compose", "up", "--project", project, "--detach"])
        .output()
        .expect("run compose up against stale registry");
    assert!(
        !output.status.success(),
        "compose up must reject a registry/config mismatch before launch"
    );
    let text = output_text(&output);
    assert!(
        text.contains("registry/config mismatch") && text.contains("vat compose import"),
        "stale registry rejection must explain remediation:\n{text}"
    );
    assert!(
        !fakes
            .docker_calls()
            .lines()
            .any(|line| line.starts_with("run ")),
        "stale registry must be rejected before Docker run:\n{}",
        fakes.docker_calls()
    );
    assert!(
        !fakes
            .container_calls()
            .lines()
            .any(|line| line.starts_with("run ")),
        "stale registry must be rejected before Apple Container run:\n{}",
        fakes.container_calls()
    );
}

#[test]
fn compose_up_rejects_missing_registry_before_starting_a_runtime() {
    let fakes = FakeRuntimes::new();
    let vat_home = TempDir::new().expect("create VAT_HOME");
    let caller = TempDir::new().expect("create caller directory");
    let project = "missing-compose-registry";
    let registry = vat_home.path().join("compose").join(project);
    fs::create_dir_all(&registry).expect("create seeded compose registry");
    write_user_edited_replacement_vat_toml(&registry, 41881);

    let output = fakes
        .command(vat_home.path(), caller.path())
        .args(["compose", "up", "--project", project, "--detach"])
        .output()
        .expect("run compose up against missing registry");
    assert!(
        !output.status.success(),
        "compose up must reject a missing registry before launch"
    );
    let text = output_text(&output);
    assert!(
        text.contains("project.json") && text.contains("vat compose import"),
        "missing registry rejection must explain remediation:\n{text}"
    );
    assert!(
        !fakes
            .docker_calls()
            .lines()
            .any(|line| line.starts_with("run ")),
        "missing registry must be rejected before Docker run:\n{}",
        fakes.docker_calls()
    );
}

#[test]
fn compose_up_accepts_user_edited_config_when_registry_service_ids_match() {
    let fakes = FakeRuntimes::new();
    let vat_home = TempDir::new().expect("create VAT_HOME");
    let caller = TempDir::new().expect("create caller directory");
    let project = "user-edited-matching-services";
    let registry = vat_home.path().join("compose").join(project);
    fs::create_dir_all(&registry).expect("create seeded compose registry");
    let (port, readiness_server) = spawn_ready_listener();
    write_user_edited_replacement_vat_toml(&registry, port);
    write_imported_registry(&registry, project, &["replacement"]);
    fakes.seed_image(FakeStore::Docker, "fake:replacement");

    let up = fakes
        .command(vat_home.path(), caller.path())
        .args(["compose", "up", "--project", project, "--detach"])
        .output()
        .expect("run compose up against user-edited config");
    assert_success(&up, "compose up with matching service ids");
    wait_for_compose_ready(
        &fakes,
        vat_home.path(),
        caller.path(),
        project,
        &fakes.docker_calls(),
    );
    assert!(
        fakes
            .docker_calls()
            .lines()
            .any(|line| line.starts_with("run ") && line.ends_with("fake:replacement")),
        "matching service ids must allow the user-edited config to start:\n{}",
        fakes.docker_calls()
    );

    let down = fakes
        .command(vat_home.path(), caller.path())
        .args(["compose", "down", project])
        .output()
        .expect("run compose down for user-edited config");
    assert_success(&down, "compose down with matching service ids");
    assert!(
        readiness_server.join().expect("join fake readiness server"),
        "VAT never probed the user-edited service endpoint"
    );
}

#[test]
fn image_only_import_succeeds_without_any_builder_on_path() {
    let vat_home = TempDir::new().expect("create VAT_HOME");
    let compose_root = TempDir::new().expect("create compose root");
    let unrelated_cwd = TempDir::new().expect("create unrelated caller cwd");
    let empty_bin = TempDir::new().expect("create empty PATH directory");
    let compose_file = compose_root.path().join("docker-compose.yml");
    fs::write(
        &compose_file,
        "services:\n  web:\n    image: nginx:1.27-alpine\n    ports:\n      - \"8080:80\"\n",
    )
    .expect("write image-only compose file");

    let output = Command::new(vat_bin())
        .current_dir(unrelated_cwd.path())
        .env("VAT_HOME", vat_home.path())
        .env("PATH", empty_bin.path())
        .arg("compose")
        .arg("import")
        .arg(&compose_file)
        .args(["--project", "image-only", "--runtime", "docker"])
        .output()
        .expect("run image-only compose import");
    assert_success(&output, "image-only compose import");

    let registry = vat_home.path().join("compose").join("image-only");
    let vat_toml =
        fs::read_to_string(registry.join("vat.toml")).expect("read materialized vat.toml");
    assert!(
        vat_toml.contains("image = \"nginx:1.27-alpine\""),
        "image-only import changed the source image:\n{vat_toml}"
    );
    let record: Value =
        serde_json::from_slice(&fs::read(registry.join("project.json")).expect("read registry"))
            .expect("parse registry");
    assert_eq!(record["status"], "imported");
}
// HANDWRITE-END
