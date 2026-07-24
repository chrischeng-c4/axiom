// HANDWRITE-BEGIN gap="missing-generator:e2e-test:compose-full-cycle" tracker="#1484" reason="AC5: gated full up -d / ps / logs / down cycle test against a real container/docker backend, using a `container_available()` skip helper mirroring `vat_cluster.rs`'s Docker-gated pattern and `vat_sandbox_microvm.rs`'s container-gated tests -- new test file, hand-authored per this project's e2e-test convention."

//! Container-gated e2e tests for vat compose full lifecycle.
//!
//! Tests the full import/up/ps/logs/down cycle against a real Docker backend
//! by invoking the compiled `vat` binary directly, using a `docker_available()`
//! skip helper. Each invocation gets its own `VAT_HOME` tempdir so the test
//! never touches this repo's real `.vat` state.

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use std::ffi::OsString;
    use std::fs;
    #[cfg(unix)]
    use std::fs::OpenOptions;
    use std::net::TcpListener;
    #[cfg(unix)]
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};
    use tempfile::TempDir;

    fn vat_bin() -> &'static str {
        env!("CARGO_BIN_EXE_vat")
    }

    const REAL_DOCKER_E2E_REQUIRED: &str = "VAT_COMPOSE_REAL_DOCKER_E2E_REQUIRED";

    /// Check if Docker is available (skip test if not) -- the compose full
    /// cycle test runs services via `--runtime docker`.
    fn docker_available() -> bool {
        Command::new("docker")
            .arg("info")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn result_id(stdout: &[u8]) -> String {
        String::from_utf8_lossy(stdout)
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .find(|event| event["type"] == "result")
            .and_then(|event| event["id"].as_str().map(str::to_string))
            .expect("failed vat run must retain a result id")
    }

    fn registry_dir(vat_home: &TempDir, project: &str) -> std::path::PathBuf {
        vat_home.path().join("compose").join(project)
    }

    fn write_registry(registry: &Path, project: &str, status: &str, vat_id: Option<&str>) {
        fs::create_dir_all(registry).expect("create compose registry");
        fs::write(
            registry.join("project.json"),
            serde_json::to_vec(&serde_json::json!({
                "project": project,
                "vat_id": vat_id,
                "service_ids": ["web"],
                "status": status,
                "created_at": "2026-01-01T00:00:00Z",
            }))
            .expect("serialize compose registry"),
        )
        .expect("write compose registry");
    }

    /// Current compose records retain this marker after the transient token
    /// and launcher PID are cleared. Keep it separate from `write_registry`,
    /// whose omitted field intentionally models historic pre-protocol JSON.
    fn write_modern_registry(registry: &Path, project: &str, status: &str, vat_id: &str) {
        fs::create_dir_all(registry).expect("create compose registry");
        fs::write(
            registry.join("project.json"),
            serde_json::to_vec(&serde_json::json!({
                "project": project,
                "vat_id": vat_id,
                "handoff_protocol": 1,
                "service_ids": ["web"],
                "status": status,
                "created_at": "2026-01-01T00:00:00Z",
            }))
            .expect("serialize modern compose registry"),
        )
        .expect("write modern compose registry");
    }

    fn write_token_backed_starting_registry(registry: &Path, project: &str, token: &str) {
        fs::create_dir_all(registry).expect("create compose registry");
        fs::write(
            registry.join("project.json"),
            serde_json::to_vec(&serde_json::json!({
                "project": project,
                "vat_id": null,
                "startup_pid": null,
                "startup_token": token,
                "service_ids": ["web"],
                "status": "starting",
                "created_at": "2026-01-01T00:00:00Z",
            }))
            .expect("serialize token-backed compose registry"),
        )
        .expect("write token-backed compose registry");
    }

    fn write_expired_token_without_pid_registry(registry: &Path, project: &str, token: &str) {
        fs::create_dir_all(registry).expect("create compose registry");
        fs::write(
            registry.join("project.json"),
            serde_json::to_vec(&serde_json::json!({
                "project": project,
                "vat_id": null,
                "startup_pid": null,
                "startup_token": token,
                "startup_started_at": "2000-01-01T00:00:00Z",
                "service_ids": ["web"],
                "status": "starting",
                "created_at": "2026-01-01T00:00:00Z",
            }))
            .expect("serialize expired token compose registry"),
        )
        .expect("write expired token compose registry");
    }

    fn read_registry(registry: &Path) -> Value {
        serde_json::from_slice(&fs::read(registry.join("project.json")).expect("read registry"))
            .expect("parse registry")
    }

    /// A no-Docker detached fixture that stays alive long enough for a second
    /// `compose up -d` to exercise the active-run claim instead of racing a
    /// completed runner.
    fn write_long_lived_vat_toml(registry: &Path) {
        fs::create_dir_all(registry).expect("create compose registry");
        fs::write(
            registry.join("vat.toml"),
            r#"version = 1

[workspace]
keep = "always"

[[services]]
id = "web"
cmd = ["/bin/sh", "-c", "sleep 10"]
ready_cmd = ["true"]
timeout_s = 2

[[runners]]
id = "project.up"
requires = ["web"]
cmd = ["/bin/sh", "-c", "sleep 10"]
"#,
        )
        .expect("write long-lived vat.toml");
    }

    #[derive(Clone, Copy)]
    enum DockerCleanupFailure {
        Nonzero,
        Hang,
    }

    impl DockerCleanupFailure {
        fn marker_name(self) -> &'static str {
            match self {
                Self::Nonzero => ".vat-fake-docker-rm-failure",
                Self::Hang => ".vat-fake-docker-rm-hang",
            }
        }

        fn expected_detail(self) -> &'static str {
            match self {
                Self::Nonzero => "exited unsuccessfully",
                Self::Hang => "timed out",
            }
        }
    }

    #[derive(Clone, Copy)]
    enum DockerExactList {
        Empty,
        Matching,
        Error,
    }

    impl DockerExactList {
        fn marker_name(self) -> Option<&'static str> {
            match self {
                Self::Empty => Some(".vat-fake-docker-list-empty"),
                Self::Matching => None,
                Self::Error => Some(".vat-fake-docker-list-error"),
            }
        }

        fn project(self) -> &'static str {
            match self {
                Self::Empty => "docker-auto-remove-confirmed",
                Self::Matching => "docker-auto-remove-still-present",
                Self::Error => "docker-auto-remove-list-error",
            }
        }

        fn confirms_absence(self) -> bool {
            matches!(self, Self::Empty)
        }
    }

    fn fake_docker_path(bin_dir: &Path) -> PathBuf {
        let script = bin_dir.join("docker");
        fs::create_dir_all(bin_dir).expect("create fake Docker directory");
        fs::write(
            &script,
            r#"#!/bin/sh
if [ -n "${VAT_FAKE_DOCKER_LOG:-}" ]; then
  printf '%s\n' "$*" >> "$VAT_FAKE_DOCKER_LOG"
fi

case "${1:-}" in
  info)
    if [ -e "$(dirname "$0")/.vat-fake-docker-info-failure" ]; then
      exit 42
    fi
    ;;
  run)
    shift
    while [ "$#" -gt 0 ]; do
      if [ "$1" = "--name" ]; then
        printf '%s\n' "$2" > "$(dirname "$0")/.vat-fake-docker-live-name"
        break
      fi
      shift
    done
    exec /bin/sleep 30
    ;;
  container)
    [ "${2:-}" = "ls" ] || exit 2
    if [ -e "$(dirname "$0")/.vat-fake-docker-list-error" ]; then
      exit 71
    fi
    if [ -e "$(dirname "$0")/.vat-fake-docker-list-empty" ]; then
      exit 0
    fi
    cat "$(dirname "$0")/.vat-fake-docker-live-name" 2>/dev/null || true
    ;;
  inspect)
    if [ -e "$(dirname "$0")/.vat-fake-docker-inspect-absent" ]; then
      exit 1
    fi
    ;;
  rm)
    if [ -e "$(dirname "$0")/.vat-fake-docker-rm-hang" ]; then
      exec /bin/sleep 5
    fi
    if [ -e "$(dirname "$0")/.vat-fake-docker-rm-failure" ]; then
      exit 23
    fi
    ;;
  *)
    exit 2
    ;;
esac
"#,
        )
        .expect("write fake Docker");
        let mut permissions = fs::metadata(&script)
            .expect("fake Docker metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script, permissions).expect("make fake Docker executable");
        script
    }

    fn path_with_fake_docker(bin_dir: &Path) -> OsString {
        let mut paths = vec![bin_dir.to_path_buf()];
        paths.extend(std::env::split_paths(
            &std::env::var_os("PATH").expect("PATH is set for test process"),
        ));
        std::env::join_paths(paths).expect("join fake Docker PATH")
    }

    fn fake_docker_vat_command(
        registry: &Path,
        vat_home: &TempDir,
        bin_dir: &Path,
        log: &Path,
    ) -> Command {
        let mut command = Command::new(vat_bin());
        command
            .current_dir(registry)
            .env("VAT_HOME", vat_home.path())
            .env("VAT_FAKE_DOCKER_LOG", log)
            .env("PATH", path_with_fake_docker(bin_dir));
        command
    }

    fn write_docker_compose_vat_toml(registry: &Path, port: u16) {
        fs::create_dir_all(registry).expect("create compose registry");
        fs::write(
            registry.join("vat.toml"),
            format!(
                r#"version = 1

[workspace]
keep = "always"

[[services]]
id = "web"
image = "fake:image"
runtime = "docker"
container_port = 80
port = {port}
timeout_s = 2

[[runners]]
id = "project.up"
requires = ["web"]
cmd = ["/bin/sh", "-c", "sleep 10"]
"#
            ),
        )
        .expect("write Docker compose vat.toml");
    }

    fn write_ready_then_exit_vat_toml(registry: &Path, marker: &Path) {
        fs::create_dir_all(registry).expect("create compose registry");
        fs::write(
            registry.join("vat.toml"),
            format!(
                r#"version = 1

[workspace]
keep = "always"

[[services]]
id = "web"
cmd = ["/bin/sh", "-c", "sleep 1; touch '{marker}'"]
ready_cmd = ["true"]
timeout_s = 2

[[runners]]
id = "project.up"
requires = ["web"]
cmd = ["/bin/sh", "-c", "sleep 10"]
"#,
                marker = marker.display(),
            ),
        )
        .expect("write ready-then-exit vat.toml");
    }

    /// Seed a retained VAT with a completed runner, then let the regression
    /// fixture project the narrow lifecycle snapshots compose must reconcile.
    /// Keeping creation on the public CLI gives the fixture a schema-valid
    /// `meta.json` while avoiding a live service process or external runtime.
    fn seed_completed_runner_vat(vat_home: &TempDir, registry: &Path, project: &str) -> String {
        fs::create_dir_all(registry).expect("create compose registry");
        fs::write(
            registry.join("vat.toml"),
            r#"version = 1

[workspace]
keep = "always"

[[runners]]
id = "project.up"
cmd = ["true"]
"#,
        )
        .expect("write completed-runner vat.toml");

        let output = Command::new(vat_bin())
            .current_dir(registry)
            .env("VAT_HOME", vat_home.path())
            .args(["run", "project.up", "--name", project, "--keep", "always"])
            .output()
            .expect("seed completed runner VAT");
        assert!(
            output.status.success(),
            "seed runner failed: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        result_id(&output.stdout)
    }

    /// Rewrite only the terminal-edge evidence under test. The runner is
    /// deliberately recorded as exited before the VAT lifecycle itself: this
    /// is the real teardown persistence window that must not free a compose
    /// registry while a tracked service can still own a published host port.
    fn write_reconcile_snapshot(
        vat_home: &TempDir,
        vat_id: &str,
        vat_state: &str,
        service_status: &str,
        cleanup_error: Option<&str>,
    ) {
        let meta_path = vat_home.path().join("vats").join(vat_id).join("meta.json");
        let mut meta: Value = serde_json::from_slice(&fs::read(&meta_path).expect("read VAT meta"))
            .expect("parse VAT meta");
        meta["status"] = match vat_state {
            "running" => serde_json::json!({ "state": "running" }),
            "exited" => serde_json::json!({ "state": "exited", "code": 0 }),
            other => panic!("unsupported fixture VAT state `{other}`"),
        };

        let (pid, exit_code) = match service_status {
            "ready" => (serde_json::json!(4242), Value::Null),
            "exited" => (Value::Null, serde_json::json!(0)),
            other => panic!("unsupported fixture service status `{other}`"),
        };
        let runner = serde_json::json!({
            "id": "project.up",
            "command": ["true"],
            "status": "exited",
            "exit_code": 0,
            "stdout_log": "",
            "stderr_log": "",
        });
        let service = serde_json::json!({
            "id": "web",
            "command": ["/bin/sh", "-c", "sleep 60"],
            "status": service_status,
            "host": "127.0.0.1",
            "port": 8080,
            "owned_by_vat": true,
            "pid": pid,
            "exit_code": exit_code,
            "cleanup_error": cleanup_error,
            "stdout_log": "",
            "stderr_log": "",
        });
        let test_run = meta
            .get_mut("test_run")
            .and_then(Value::as_object_mut)
            .expect("seed VAT must have runner evidence");
        test_run.insert("runner".to_string(), runner.clone());
        test_run.insert("runners".to_string(), serde_json::json!([runner]));
        test_run.insert("services".to_string(), serde_json::json!([service]));
        fs::write(
            &meta_path,
            serde_json::to_vec_pretty(&meta).expect("serialize lifecycle snapshot"),
        )
        .expect("write lifecycle snapshot");
    }

    fn wait_for_compose_ready(vat_home: &TempDir, project: &str) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let output = Command::new(vat_bin())
                .env("VAT_HOME", vat_home.path())
                .args(["compose", "ps", project])
                .output()
                .expect("compose ps");
            let stdout = String::from_utf8_lossy(&output.stdout);
            if output.status.success() && stdout.contains("is ready") {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "compose project `{project}` never became ready:\nstdout: {stdout}\nstderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    fn wait_for_path(path: &Path, description: &str) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !path.exists() {
            assert!(
                Instant::now() < deadline,
                "{description} did not appear at {}",
                path.display()
            );
            std::thread::sleep(Duration::from_millis(25));
        }
    }

    /// Wait until another compose invocation owns the shared advisory lock.
    /// Observing flock contention is more reliable than watching the detached
    /// stop-request file, whose consumer removes it immediately.
    #[cfg(unix)]
    fn wait_for_lifecycle_claim(registry: &Path) {
        let lock_path = registry.join("startup.lock");
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&lock_path)
                .expect("open compose startup lock");
            let locked = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
            if locked != 0 {
                let error = std::io::Error::last_os_error();
                assert!(
                    error.raw_os_error() == Some(libc::EWOULDBLOCK)
                        || error.raw_os_error() == Some(libc::EAGAIN),
                    "unexpected compose lock probe error: {error}"
                );
                return;
            }
            let unlocked = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_UN) };
            assert_eq!(unlocked, 0, "release lifecycle lock probe");
            assert!(
                Instant::now() < deadline,
                "compose down never acquired its lifecycle claim"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[cfg(not(unix))]
    fn wait_for_lifecycle_claim(_registry: &Path) {
        panic!("this local process lifecycle regression requires Unix flock")
    }

    fn vat_state(vat_home: &TempDir, vat_id: &str) -> Value {
        let output = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["state", vat_id, "--compact"])
            .output()
            .expect("vat state");
        assert!(
            output.status.success(),
            "vat state failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).expect("parse vat state")
    }

    fn down_compose(vat_home: &TempDir, project: &str) {
        let output = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "down", project])
            .output()
            .expect("compose down");
        assert!(
            output.status.success(),
            "compose down failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn docker_cleanup_error_is_retained_until_retry(failure: DockerCleanupFailure) {
        let vat_home = TempDir::new().expect("VAT_HOME");
        let fake_bin = TempDir::new().expect("fake Docker bin");
        let fake_log = fake_bin.path().join("docker.log");
        fake_docker_path(fake_bin.path());
        let failure_marker = fake_bin.path().join(failure.marker_name());
        fs::write(&failure_marker, b"force Docker cleanup failure")
            .expect("mark fake Docker cleanup failure");

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake Docker readiness port");
        let port = listener
            .local_addr()
            .expect("fake Docker readiness address")
            .port();
        let project = match failure {
            DockerCleanupFailure::Nonzero => "docker-cleanup-nonzero",
            DockerCleanupFailure::Hang => "docker-cleanup-hang",
        };
        let registry = registry_dir(&vat_home, project);
        write_docker_compose_vat_toml(&registry, port);
        write_registry(&registry, project, "imported", None);

        let up = fake_docker_vat_command(&registry, &vat_home, fake_bin.path(), &fake_log)
            .args(["compose", "up", "--project", project, "--detach"])
            .output()
            .expect("compose up with fake Docker");
        assert!(
            up.status.success(),
            "compose up failed: stdout={} stderr={}",
            String::from_utf8_lossy(&up.stdout),
            String::from_utf8_lossy(&up.stderr)
        );
        wait_for_compose_ready(&vat_home, project);
        let vat_id = read_registry(&registry)["vat_id"]
            .as_str()
            .expect("active Docker VAT id")
            .to_string();

        let started = Instant::now();
        let failed_down = fake_docker_vat_command(&registry, &vat_home, fake_bin.path(), &fake_log)
            .args(["compose", "down", project])
            .output()
            .expect("compose down with failed Docker cleanup");
        if matches!(failure, DockerCleanupFailure::Hang) {
            assert!(
                started.elapsed() < Duration::from_secs(4),
                "hung Docker cleanup escaped its bounded teardown: {:?}",
                started.elapsed()
            );
        }
        assert!(
            !failed_down.status.success(),
            "compose down must retain an unconfirmed Docker cleanup: stdout={} stderr={}",
            String::from_utf8_lossy(&failed_down.stdout),
            String::from_utf8_lossy(&failed_down.stderr)
        );

        let retained = read_registry(&registry);
        assert_eq!(retained["vat_id"], vat_id, "registry: {retained}");
        assert_ne!(retained["status"], "imported", "registry: {retained}");
        let failed_state = vat_state(&vat_home, &vat_id);
        let failed_service = failed_state["test_run"]["services"]
            .as_array()
            .and_then(|services| services.iter().find(|service| service["id"] == "web"))
            .expect("persisted Docker service");
        let cleanup_error = failed_service["cleanup_error"]
            .as_str()
            .expect("unconfirmed Docker cleanup must be persisted");
        assert!(
            cleanup_error.contains("docker rm -f")
                && cleanup_error.contains(failure.expected_detail()),
            "state: {failed_state}"
        );

        let ps = fake_docker_vat_command(&registry, &vat_home, fake_bin.path(), &fake_log)
            .args(["compose", "ps", project])
            .output()
            .expect("compose ps after failed Docker cleanup");
        assert!(
            !ps.status.success(),
            "compose ps must not project a cleanup-unconfirmed Docker binding as ready"
        );
        let up_again = fake_docker_vat_command(&registry, &vat_home, fake_bin.path(), &fake_log)
            .args(["compose", "up", "--project", project, "--detach"])
            .output()
            .expect("compose up after failed Docker cleanup");
        assert!(
            !up_again.status.success(),
            "compose up must not replace a cleanup-unconfirmed Docker binding"
        );
        let still_retained = read_registry(&registry);
        assert_eq!(
            still_retained["vat_id"], vat_id,
            "registry: {still_retained}"
        );
        assert_ne!(
            still_retained["status"], "imported",
            "registry: {still_retained}"
        );

        fs::remove_file(&failure_marker).expect("repair fake Docker cleanup");
        let retry = fake_docker_vat_command(&registry, &vat_home, fake_bin.path(), &fake_log)
            .args(["compose", "down", project])
            .output()
            .expect("retry Docker cleanup through compose down");
        assert!(
            retry.status.success(),
            "Docker cleanup retry failed: stdout={} stderr={}",
            String::from_utf8_lossy(&retry.stdout),
            String::from_utf8_lossy(&retry.stderr)
        );
        let released = read_registry(&registry);
        assert_eq!(released["status"], "imported", "registry: {released}");
        assert!(released["vat_id"].is_null(), "registry: {released}");
        let recovered_state = vat_state(&vat_home, &vat_id);
        let recovered_service = recovered_state["test_run"]["services"]
            .as_array()
            .and_then(|services| services.iter().find(|service| service["id"] == "web"))
            .expect("recovered Docker service");
        assert!(
            recovered_service["cleanup_error"].is_null(),
            "successful Docker cleanup retry must clear evidence: {recovered_state}"
        );

        let calls = fs::read_to_string(&fake_log).expect("fake Docker command log");
        let removals = calls
            .lines()
            .filter(|line| line.starts_with("rm -f "))
            .count();
        assert_eq!(
            removals, 2,
            "expected initial Docker cleanup plus one retry: {calls}"
        );
        drop(listener);
    }

    /// A nonzero `docker rm -f` is normal with `docker run --rm` when the
    /// foreground child has already removed itself. Compose may release the
    /// binding only after a successful exact-name list is empty. A matching
    /// name or a list/object error is not absence evidence.
    fn docker_failed_rm_with_exact_name_list(result: DockerExactList) {
        let vat_home = TempDir::new().expect("VAT_HOME");
        let fake_bin = TempDir::new().expect("fake Docker bin");
        let fake_log = fake_bin.path().join("docker.log");
        fake_docker_path(fake_bin.path());
        fs::write(
            fake_bin.path().join(".vat-fake-docker-rm-failure"),
            b"simulate --rm auto-removal before explicit cleanup",
        )
        .expect("mark fake Docker rm failure");
        if let Some(marker) = result.marker_name() {
            fs::write(
                fake_bin.path().join(marker),
                b"configure fake Docker exact-name list proof",
            )
            .expect("configure fake Docker exact-name list proof");
        }

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake Docker readiness port");
        let port = listener
            .local_addr()
            .expect("fake Docker readiness address")
            .port();
        let project = result.project();
        let registry = registry_dir(&vat_home, project);
        write_docker_compose_vat_toml(&registry, port);
        write_registry(&registry, project, "imported", None);

        let up = fake_docker_vat_command(&registry, &vat_home, fake_bin.path(), &fake_log)
            .args(["compose", "up", "--project", project, "--detach"])
            .output()
            .expect("compose up with fake Docker");
        assert!(
            up.status.success(),
            "compose up failed: stdout={} stderr={}",
            String::from_utf8_lossy(&up.stdout),
            String::from_utf8_lossy(&up.stderr)
        );
        wait_for_compose_ready(&vat_home, project);
        let vat_id = read_registry(&registry)["vat_id"]
            .as_str()
            .expect("active Docker VAT id")
            .to_string();

        let down = fake_docker_vat_command(&registry, &vat_home, fake_bin.path(), &fake_log)
            .args(["compose", "down", project])
            .output()
            .expect("compose down after fake Docker auto-removal");
        let lifecycle = vat_state(&vat_home, &vat_id);
        let service = lifecycle["test_run"]["services"]
            .as_array()
            .and_then(|services| services.iter().find(|service| service["id"] == "web"))
            .expect("persisted Docker service");
        let record = read_registry(&registry);

        if result.confirms_absence() {
            assert!(
                down.status.success(),
                "an empty exact-name Docker list must permit compose down: stdout={} stderr={}",
                String::from_utf8_lossy(&down.stdout),
                String::from_utf8_lossy(&down.stderr)
            );
            assert_eq!(record["status"], "imported", "registry: {record}");
            assert!(record["vat_id"].is_null(), "registry: {record}");
            assert!(
                service["cleanup_error"].is_null(),
                "confirmed Docker auto-removal must not persist cleanup evidence: {lifecycle}"
            );
        } else {
            assert!(
                !down.status.success(),
                "missing exact absence proof must keep cleanup unconfirmed: stdout={} stderr={}",
                String::from_utf8_lossy(&down.stdout),
                String::from_utf8_lossy(&down.stderr)
            );
            assert_eq!(record["vat_id"], vat_id, "registry: {record}");
            assert_ne!(record["status"], "imported", "registry: {record}");
            assert!(
                service["cleanup_error"]
                    .as_str()
                    .is_some_and(|error| error.contains("docker rm -f")),
                "a matching name or list/object error must retain Docker cleanup evidence: {lifecycle}"
            );
        }

        let calls = fs::read_to_string(&fake_log).expect("fake Docker command log");
        let cleanup_start = calls
            .lines()
            .position(|line| line.starts_with("rm -f "))
            .expect("Docker cleanup command");
        let cleanup_calls: Vec<_> = calls.lines().skip(cleanup_start).collect();
        assert!(
            cleanup_calls
                .iter()
                .any(|line| line.starts_with("container ls ")),
            "failed rm must list the exact Docker name: {calls}"
        );
        drop(listener);
    }

    // <HANDWRITE gap="vat-compose-detached-status-regression" tracker="#1526" reason="Update full-cycle compose assertions for evidence-based starting and ready semantics without changing Docker runtime behavior.">
    #[test]
    fn test_compose_full_cycle_up_down() {
        if !docker_available() {
            assert_ne!(
                std::env::var(REAL_DOCKER_E2E_REQUIRED).as_deref(),
                Ok("1"),
                "{REAL_DOCKER_E2E_REQUIRED}=1 requires a working Docker daemon"
            );
            eprintln!("Skipping test: docker not available");
            return;
        }

        let tmpdir = TempDir::new().unwrap();
        let vat_home = TempDir::new().unwrap();
        let project = format!("vattest{}", std::process::id());

        let compose = r#"
version: '3'
services:
  web:
    image: nginx:1.27-alpine
    ports:
      - "80"
"#;
        let compose_file = tmpdir.path().join("docker-compose.yml");
        fs::write(&compose_file, compose).unwrap();

        // import
        let output = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .arg("compose")
            .arg("import")
            .arg(&compose_file)
            .args(["--project", &project, "--runtime", "docker"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "compose import failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // up -d
        let output = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "up", "--project", &project, "--detach"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "compose up --detach failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let up_json: serde_json::Value =
            serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
                panic!(
                    "compose up did not print JSON: {e}\n{}",
                    String::from_utf8_lossy(&output.stdout)
                )
            });
        assert!(
            matches!(up_json["status"].as_str(), Some("starting" | "ready")),
            "up_json: {up_json}"
        );
        assert_ne!(up_json["status"], "started", "up_json: {up_json}");
        assert!(
            up_json["vat_id"].is_string() || up_json["vat_id"].is_null(),
            "up_json: {up_json}"
        );

        // ps -- poll briefly since the service becomes visible only once the
        // detached `vat run` has persisted its first ServiceRunRecord.
        let deadline = Instant::now() + Duration::from_secs(30);
        let ps_text = loop {
            let output = Command::new(vat_bin())
                .env("VAT_HOME", vat_home.path())
                .args(["compose", "ps", &project])
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "compose ps failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let text = String::from_utf8_lossy(&output.stdout).into_owned();
            if (text.contains("ready") && text.contains("web")) || Instant::now() >= deadline {
                break text;
            }
            std::thread::sleep(Duration::from_millis(500));
        };
        assert!(
            ps_text.contains("ready") && ps_text.contains("web"),
            "compose ps never reached a ready `web` service: {ps_text}"
        );

        // logs (must not error, even if nginx hasn't written much yet).
        let output = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "logs", &project, "web"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "compose logs failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // down
        let output = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "down", &project])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "compose down failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // A second `down` must fail cleanly -- the durable import record has
        // no active VAT binding after the first down.
        let output = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "down", &project])
            .output()
            .unwrap();
        assert!(!output.status.success());
    }
    // </HANDWRITE>

    #[test]
    fn test_compose_terminal_ps_resets_active_run_and_preserves_import_for_retry() {
        let vat_home = TempDir::new().unwrap();
        let project = "terminal-retry";
        let registry = vat_home.path().join("compose").join(project);
        fs::create_dir_all(&registry).unwrap();
        fs::write(
            registry.join("vat.toml"),
            r#"version = 1

[workspace]
keep = "always"

[[services]]
id = "web"
cmd = ["/bin/sh", "-c", "exit 1"]
ready_cmd = ["/bin/sh", "-c", "exit 1"]
timeout_s = 1

[[runners]]
id = "project.up"
requires = ["web"]
cmd = ["true"]
"#,
        )
        .unwrap();

        let registry_path = registry.join("project.json");
        fs::write(
            &registry_path,
            serde_json::to_vec(&serde_json::json!({
                "project": project,
                "vat_id": null,
                "service_ids": ["web"],
                "status": "starting",
                "created_at": "2026-01-01T00:00:00Z",
            }))
            .unwrap(),
        )
        .unwrap();
        let concurrent_up = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "up", "--project", project, "--detach"])
            .output()
            .unwrap();
        assert!(!concurrent_up.status.success());
        assert!(
            String::from_utf8_lossy(&concurrent_up.stderr).contains("already starting"),
            "stderr: {}",
            String::from_utf8_lossy(&concurrent_up.stderr)
        );
        let down_while_starting = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "down", project])
            .output()
            .unwrap();
        assert!(!down_while_starting.status.success());
        assert!(
            registry_path.exists(),
            "down must retain the starting registry until it can identify the child"
        );

        let failed_run = Command::new(vat_bin())
            .current_dir(&registry)
            .env("VAT_HOME", vat_home.path())
            .args(["run", "project.up", "--name", project, "--keep", "always"])
            .output()
            .unwrap();
        assert!(
            !failed_run.status.success(),
            "fixture service must fail before readiness"
        );
        let vat_id = result_id(&failed_run.stdout);
        fs::write(
            &registry_path,
            serde_json::to_vec(&serde_json::json!({
                "project": project,
                "vat_id": vat_id,
                "service_ids": ["web"],
                "status": "starting",
                "created_at": "2026-01-01T00:00:00Z",
            }))
            .unwrap(),
        )
        .unwrap();

        let terminal_ps = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "ps", project])
            .output()
            .unwrap();
        assert!(!terminal_ps.status.success());
        let terminal_stderr = String::from_utf8_lossy(&terminal_ps.stderr);
        assert!(
            terminal_stderr.contains(&format!("vat state {vat_id}")),
            "stderr: {terminal_stderr}"
        );
        assert!(
            vat_home
                .path()
                .join("vats")
                .join(&vat_id)
                .join("meta.json")
                .exists(),
            "terminal compose state must retain VAT evidence"
        );

        let reset: Value = serde_json::from_slice(&fs::read(&registry_path).unwrap()).unwrap();
        assert_eq!(reset["status"], "imported", "registry: {reset}");
        assert!(reset["vat_id"].is_null(), "registry: {reset}");
        assert_eq!(reset["service_ids"], serde_json::json!(["web"]));

        let imported_ps = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "ps", project])
            .output()
            .unwrap();
        assert!(
            imported_ps.status.success(),
            "imported ps failed: {}",
            String::from_utf8_lossy(&imported_ps.stderr)
        );
        assert!(
            String::from_utf8_lossy(&imported_ps.stdout).contains("is imported"),
            "stdout: {}",
            String::from_utf8_lossy(&imported_ps.stdout)
        );

        let retry = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "up", "--project", project, "--detach"])
            .output()
            .unwrap();
        assert!(
            !retry.status.success(),
            "the deliberately failing retried service must not report a false startup success"
        );
        let retry_stderr = String::from_utf8_lossy(&retry.stderr);
        assert!(
            retry_stderr.contains("service `web`") || retry_stderr.contains("startup failed"),
            "stderr: {retry_stderr}"
        );
        let retried: Value = serde_json::from_slice(&fs::read(&registry_path).unwrap()).unwrap();
        assert_eq!(retried["status"], "imported", "registry: {retried}");
        assert_eq!(retried["service_ids"], serde_json::json!(["web"]));
    }

    #[test]
    fn test_compose_detached_up_uses_atomic_claim_and_creates_one_run() {
        let vat_home = TempDir::new().expect("VAT_HOME");
        let project = "atomic-claim";
        let registry = registry_dir(&vat_home, project);
        write_long_lived_vat_toml(&registry);
        write_registry(&registry, project, "imported", None);

        // Start two independent CLI processes without an artificial delay. The
        // winner may release its advisory claim before the loser runs, in
        // which case the durable `starting` record must still reject the loser.
        let first = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "up", "--project", project, "--detach"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn first compose up");
        let second = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "up", "--project", project, "--detach"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn second compose up");
        let first_output = first.wait_with_output().expect("wait first compose up");
        let second_output = second.wait_with_output().expect("wait second compose up");
        let outputs = [&first_output, &second_output];
        assert_eq!(
            outputs
                .iter()
                .filter(|output| output.status.success())
                .count(),
            1,
            "exactly one detached up may win:\nfirst stderr: {}\nsecond stderr: {}",
            String::from_utf8_lossy(&first_output.stderr),
            String::from_utf8_lossy(&second_output.stderr)
        );
        let loser = outputs
            .iter()
            .find(|output| !output.status.success())
            .expect("one detached up loser");
        let loser_stderr = String::from_utf8_lossy(&loser.stderr);
        assert!(
            loser_stderr.contains("already being started")
                || loser_stderr.contains("already starting")
                || loser_stderr.contains("has a lifecycle operation in progress"),
            "unexpected concurrent up failure: {loser_stderr}"
        );

        wait_for_compose_ready(&vat_home, project);
        let record = read_registry(&registry);
        let vat_id = record["vat_id"].as_str().expect("active vat id");
        assert!(matches!(
            record["status"].as_str(),
            Some("starting" | "ready")
        ));
        assert!(
            vat_home
                .path()
                .join("vats")
                .join(vat_id)
                .join("meta.json")
                .exists(),
            "the registry must name the one created VAT"
        );
        let vat_count = fs::read_dir(vat_home.path().join("vats"))
            .expect("read VAT store")
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .count();
        assert_eq!(vat_count, 1, "concurrent up created more than one VAT");
        down_compose(&vat_home, project);
        let meta: Value = serde_json::from_slice(
            &fs::read(vat_home.path().join("vats").join(vat_id).join("meta.json"))
                .expect("read stopped VAT meta"),
        )
        .expect("parse stopped VAT meta");
        assert_eq!(meta["status"]["state"], "exited", "meta: {meta}");
        assert!(
            meta["test_run"]["services"]
                .as_array()
                .expect("service records")
                .iter()
                .all(|service| service["status"] == "exited"),
            "compose registry must not reset before VAT persisted service cleanup: {meta}"
        );
    }

    #[test]
    fn test_compose_up_is_rejected_while_down_holds_lifecycle_claim() {
        let vat_home = TempDir::new().expect("VAT_HOME");
        let project = "down-claim";
        let registry = registry_dir(&vat_home, project);
        write_long_lived_vat_toml(&registry);
        write_registry(&registry, project, "imported", None);

        let up = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "up", "--project", project, "--detach"])
            .output()
            .expect("compose up");
        assert!(
            up.status.success(),
            "compose up failed: stdout={} stderr={}",
            String::from_utf8_lossy(&up.stdout),
            String::from_utf8_lossy(&up.stderr)
        );
        wait_for_compose_ready(&vat_home, project);

        let down = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "down", project])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn compose down");
        // `down` owns the lock from its first registry read through service
        // shutdown and final reset. Probe the actual flock rather than its
        // transient stop-request file, which the detached child consumes
        // immediately.
        wait_for_lifecycle_claim(&registry);

        let concurrent_up = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "up", "--project", project, "--detach"])
            .output()
            .expect("concurrent compose up");
        assert!(
            !concurrent_up.status.success(),
            "compose up must not overlap a down lifecycle operation"
        );
        assert!(
            String::from_utf8_lossy(&concurrent_up.stderr)
                .contains("has a lifecycle operation in progress"),
            "stderr: {}",
            String::from_utf8_lossy(&concurrent_up.stderr)
        );

        let down = down.wait_with_output().expect("wait compose down");
        assert!(
            down.status.success(),
            "compose down failed: stdout={} stderr={}",
            String::from_utf8_lossy(&down.stdout),
            String::from_utf8_lossy(&down.stderr)
        );
        let released = read_registry(&registry);
        assert_eq!(released["status"], "imported", "registry: {released}");
        assert!(released["vat_id"].is_null(), "registry: {released}");
    }

    #[test]
    fn test_compose_retains_nonzero_docker_cleanup_until_retry() {
        docker_cleanup_error_is_retained_until_retry(DockerCleanupFailure::Nonzero);
    }

    #[test]
    fn test_compose_retains_hung_docker_cleanup_until_retry() {
        docker_cleanup_error_is_retained_until_retry(DockerCleanupFailure::Hang);
    }

    #[test]
    fn test_compose_accepts_docker_auto_remove_after_empty_exact_name_list() {
        docker_failed_rm_with_exact_name_list(DockerExactList::Empty);
    }

    #[test]
    fn test_compose_retains_docker_auto_remove_when_exact_name_list_matches() {
        docker_failed_rm_with_exact_name_list(DockerExactList::Matching);
    }

    #[test]
    fn test_compose_retains_docker_auto_remove_when_exact_name_list_errors() {
        docker_failed_rm_with_exact_name_list(DockerExactList::Error);
    }

    #[test]
    fn test_compose_detached_early_child_failure_resets_to_imported() {
        let vat_home = TempDir::new().expect("VAT_HOME");
        let project = "early-child-failure";
        let registry = registry_dir(&vat_home, project);
        fs::create_dir_all(&registry).expect("create registry");
        // The detached child reaches `vat run` but cannot parse its config, so
        // it exits before creating any VAT state for the parent to discover.
        fs::write(registry.join("vat.toml"), "version = not-a-number\n")
            .expect("write malformed vat.toml");
        write_registry(&registry, project, "imported", None);

        let output = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "up", "--project", project, "--detach"])
            .output()
            .expect("compose up with malformed detached child");
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("before creating VAT evidence"),
            "stderr: {stderr}"
        );

        let record = read_registry(&registry);
        assert_eq!(record["status"], "imported", "registry: {record}");
        assert!(record["vat_id"].is_null(), "registry: {record}");
        assert!(record.get("startup_pid").is_none(), "registry: {record}");
        // Unix keeps the advisory lock inode for crash-safe reuse, but this
        // second invocation proves the lock itself was released rather than
        // becoming a permanent create_new-style stale lock.
        let retry = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "up", "--project", project, "--detach"])
            .output()
            .expect("retry compose up after early child failure");
        assert!(!retry.status.success());
        assert!(
            !String::from_utf8_lossy(&retry.stderr).contains("already being started"),
            "stale startup lock blocked retry: {}",
            String::from_utf8_lossy(&retry.stderr)
        );
        let ps = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "ps", project])
            .output()
            .expect("compose ps after early child failure");
        assert!(ps.status.success());
        assert!(
            String::from_utf8_lossy(&ps.stdout).contains("is imported"),
            "stdout: {}",
            String::from_utf8_lossy(&ps.stdout)
        );
    }

    #[test]
    fn test_compose_up_reclaims_expired_token_without_launcher_pid() {
        let vat_home = TempDir::new().expect("VAT_HOME");
        let project = "expired-token-recovery";
        let registry = registry_dir(&vat_home, project);
        write_long_lived_vat_toml(&registry);
        // A parent that died before spawn left a token but no child PID. The
        // deliberately old timestamp makes this a deterministic abandoned
        // handoff, rather than sleeping through the two-second grace window.
        write_expired_token_without_pid_registry(&registry, project, "abandoned-token");

        let output = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "up", "--project", project, "--detach"])
            .output()
            .expect("reclaim abandoned detached startup");
        assert!(
            output.status.success(),
            "expired token did not recover: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        wait_for_compose_ready(&vat_home, project);
        let record = read_registry(&registry);
        let vat_id = record["vat_id"].as_str().expect("replacement VAT id");
        assert_ne!(
            record["startup_token"], "abandoned-token",
            "registry: {record}"
        );
        assert!(
            record["startup_token"].is_null()
                && record["startup_pid"].is_null()
                && record["startup_started_at"].is_null(),
            "replacement handoff must clear abandoned transient state: {record}"
        );
        let vat_count = fs::read_dir(vat_home.path().join("vats"))
            .expect("read VAT store")
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .count();
        assert_eq!(vat_count, 1, "abandoned token spawned duplicate VATs");
        assert!(
            vat_home
                .path()
                .join("vats")
                .join(vat_id)
                .join("meta.json")
                .exists(),
            "replacement VAT is not durable"
        );
        down_compose(&vat_home, project);
    }

    #[test]
    fn test_compose_down_marks_already_exited_ready_service_terminal() {
        let vat_home = TempDir::new().expect("VAT_HOME");
        let project = "ready-service-already-exited";
        let registry = registry_dir(&vat_home, project);
        let marker = registry.join("web-exited.marker");
        write_ready_then_exit_vat_toml(&registry, &marker);
        write_registry(&registry, project, "imported", None);

        let up = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "up", "--project", project, "--detach"])
            .output()
            .expect("compose up");
        assert!(
            up.status.success(),
            "compose up failed: stdout={} stderr={}",
            String::from_utf8_lossy(&up.stdout),
            String::from_utf8_lossy(&up.stderr)
        );
        wait_for_compose_ready(&vat_home, project);
        let vat_id = read_registry(&registry)["vat_id"]
            .as_str()
            .expect("active VAT id")
            .to_string();

        // The service had already been recorded Ready, then exits naturally
        // before down asks the VAT parent to stop the runner tree.
        wait_for_path(&marker, "naturally exited service marker");
        let started = Instant::now();
        down_compose(&vat_home, project);
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "down waited for a stale Ready service after it had already exited: {:?}",
            started.elapsed()
        );

        let state = vat_state(&vat_home, &vat_id);
        assert_eq!(state["status"]["state"], "exited", "state: {state}");
        let service = state["test_run"]["services"]
            .as_array()
            .and_then(|services| services.iter().find(|service| service["id"] == "web"))
            .expect("persisted web service");
        assert_eq!(service["status"], "exited", "state: {state}");
        assert!(service["pid"].is_null(), "state: {state}");
        assert!(service["cleanup_error"].is_null(), "state: {state}");

        let reset = read_registry(&registry);
        assert_eq!(reset["status"], "imported", "registry: {reset}");
        assert!(reset["vat_id"].is_null(), "registry: {reset}");
    }

    #[test]
    fn test_compose_reconcile_retains_running_vat_until_teardown_is_terminal_and_confirmed() {
        let vat_home = TempDir::new().expect("VAT_HOME");
        let project = "runner-teardown-window";
        let registry = registry_dir(&vat_home, project);
        let vat_id = seed_completed_runner_vat(&vat_home, &registry, project);
        write_registry(&registry, project, "ready", Some(&vat_id));

        // `run_configured` can persist an exited runner before it has torn
        // down the Ready service. VAT itself is still Running, so `ps` must
        // retain the binding and a concurrent up must not reuse port 8080.
        write_reconcile_snapshot(&vat_home, &vat_id, "running", "ready", None);
        let interim_ps = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "ps", project])
            .output()
            .expect("compose ps during runner teardown");
        assert!(
            interim_ps.status.success(),
            "running VAT with a live compose service must not be reset: stdout={} stderr={}",
            String::from_utf8_lossy(&interim_ps.stdout),
            String::from_utf8_lossy(&interim_ps.stderr)
        );
        let held = read_registry(&registry);
        assert_eq!(held["vat_id"], vat_id, "registry: {held}");
        assert!(
            matches!(
                held["status"].as_str(),
                Some("starting" | "ready" | "stopping")
            ),
            "registry: {held}"
        );

        let concurrent_up = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "up", "--project", project, "--detach"])
            .output()
            .expect("concurrent compose up during teardown");
        assert!(
            !concurrent_up.status.success(),
            "compose up must not overlap the retained service lifecycle"
        );
        assert_eq!(
            read_registry(&registry)["vat_id"],
            vat_id,
            "concurrent up must not clear or replace the active binding"
        );

        // An exited VAT with a failed cleanup is likewise non-resettable: the
        // service process may still exist even though runner work is terminal.
        write_reconcile_snapshot(
            &vat_home,
            &vat_id,
            "exited",
            "exited",
            Some("container rm -f web timed out"),
        );
        let cleanup_unconfirmed = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "ps", project])
            .output()
            .expect("compose ps with cleanup error");
        assert!(!cleanup_unconfirmed.status.success());
        assert_eq!(
            read_registry(&registry)["vat_id"],
            vat_id,
            "cleanup-unconfirmed state must retain the binding"
        );

        // Only the final snapshot — VAT exited, service terminal, and no
        // cleanup error — can surface a terminal lifecycle and free the
        // imported compose project for another up.
        write_reconcile_snapshot(&vat_home, &vat_id, "exited", "exited", None);
        let terminal_ps = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "ps", project])
            .output()
            .expect("compose ps after terminal cleanup");
        assert!(
            !terminal_ps.status.success(),
            "terminal evidence should tell the caller to inspect the retained VAT"
        );
        let reset = read_registry(&registry);
        assert_eq!(reset["status"], "imported", "registry: {reset}");
        assert!(reset["vat_id"].is_null(), "registry: {reset}");
    }

    #[test]
    fn test_compose_retains_bound_registry_when_vat_meta_is_torn() {
        let vat_home = TempDir::new().expect("VAT_HOME");
        let project = "torn-vat-evidence";
        let registry = registry_dir(&vat_home, project);
        let vat_id = seed_completed_runner_vat(&vat_home, &registry, project);
        write_registry(&registry, project, "ready", Some(&vat_id));

        // This models a process observing a torn/partially-replaced evidence
        // file. It is not terminal evidence: `ps` and a concurrent `up` must
        // leave the binding intact rather than free a possibly live port.
        let meta_path = vat_home.path().join("vats").join(&vat_id).join("meta.json");
        let torn_meta = b"{\"status\":";
        fs::write(&meta_path, torn_meta).expect("write torn VAT meta");
        let vat_count_before = fs::read_dir(vat_home.path().join("vats"))
            .expect("read VAT store")
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .count();

        let ps = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "ps", project])
            .output()
            .expect("compose ps with torn VAT evidence");
        assert!(
            !ps.status.success(),
            "torn VAT evidence must not be projected as terminal"
        );
        assert!(
            String::from_utf8_lossy(&ps.stderr).contains("temporarily unavailable"),
            "stderr: {}",
            String::from_utf8_lossy(&ps.stderr)
        );
        let after_ps = read_registry(&registry);
        assert_eq!(after_ps["vat_id"], vat_id, "registry: {after_ps}");
        assert_eq!(after_ps["status"], "ready", "registry: {after_ps}");

        let up = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "up", "--project", project, "--detach"])
            .output()
            .expect("compose up with torn VAT evidence");
        assert!(
            !up.status.success(),
            "a bound project with torn VAT evidence must not create another run"
        );
        assert!(
            String::from_utf8_lossy(&up.stderr).contains("temporarily unavailable"),
            "stderr: {}",
            String::from_utf8_lossy(&up.stderr)
        );
        let after_up = read_registry(&registry);
        assert_eq!(after_up["vat_id"], vat_id, "registry: {after_up}");
        assert_eq!(after_up["status"], "ready", "registry: {after_up}");
        let vat_count_after = fs::read_dir(vat_home.path().join("vats"))
            .expect("read VAT store")
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .count();
        assert_eq!(vat_count_after, vat_count_before, "up created a new VAT");
        assert_eq!(fs::read(&meta_path).expect("read torn VAT meta"), torn_meta);
    }

    #[test]
    fn test_compose_retains_modern_handoff_binding_when_vat_meta_is_missing() {
        let vat_home = TempDir::new().expect("VAT_HOME");
        let project = "modern-missing-vat-evidence";
        let vat_id = "missing-current-vat";
        let registry = registry_dir(&vat_home, project);
        write_long_lived_vat_toml(&registry);
        write_modern_registry(&registry, project, "ready", vat_id);

        // Unlike an omitted historic marker, this record was created by the
        // durable token-owned protocol. Even a definitive NotFound cannot
        // prove the current service set has stopped, so both readers and a
        // concurrent launcher must retain the exact binding.
        assert!(
            !vat_home
                .path()
                .join("vats")
                .join(vat_id)
                .join("meta.json")
                .exists(),
            "the fixture must start with missing VAT metadata"
        );
        let vat_count_before = fs::read_dir(vat_home.path().join("vats"))
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .filter(|entry| entry.path().is_dir())
                    .count()
            })
            .unwrap_or(0);

        let ps = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "ps", project])
            .output()
            .expect("compose ps with missing modern VAT metadata");
        assert!(
            !ps.status.success(),
            "modern missing evidence must not be projected as terminal"
        );
        assert!(
            String::from_utf8_lossy(&ps.stderr).contains("temporarily unavailable"),
            "stderr: {}",
            String::from_utf8_lossy(&ps.stderr)
        );
        let after_ps = read_registry(&registry);
        assert_eq!(after_ps["vat_id"], vat_id, "registry: {after_ps}");
        assert_eq!(after_ps["handoff_protocol"], 1, "registry: {after_ps}");
        assert_eq!(after_ps["status"], "ready", "registry: {after_ps}");

        let up = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "up", "--project", project, "--detach"])
            .output()
            .expect("compose up with missing modern VAT metadata");
        assert!(
            !up.status.success(),
            "modern missing evidence must not permit a replacement VAT"
        );
        assert!(
            String::from_utf8_lossy(&up.stderr).contains("temporarily unavailable"),
            "stderr: {}",
            String::from_utf8_lossy(&up.stderr)
        );
        let after_up = read_registry(&registry);
        assert_eq!(after_up["vat_id"], vat_id, "registry: {after_up}");
        assert_eq!(after_up["handoff_protocol"], 1, "registry: {after_up}");
        assert_eq!(after_up["status"], "ready", "registry: {after_up}");
        let vat_count_after = fs::read_dir(vat_home.path().join("vats"))
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .filter(|entry| entry.path().is_dir())
                    .count()
            })
            .unwrap_or(0);
        assert_eq!(vat_count_after, vat_count_before, "up created a new VAT");
    }

    #[test]
    fn test_detached_child_publishes_vat_id_without_parent_poller() {
        let vat_home = TempDir::new().expect("VAT_HOME");
        let project = "child-owned-handoff";
        let registry = registry_dir(&vat_home, project);
        let token = "parent-poller-is-absent";
        write_long_lived_vat_toml(&registry);
        // Deliberately do not invoke `vat compose up -d`: this emulates a
        // parent that disappeared after persisting its token but before its
        // 10-second discovery poll could write a VAT id. The re-exec-style
        // child itself must perform the durable handoff.
        write_token_backed_starting_registry(&registry, project, token);

        let mut child = Command::new(vat_bin())
            .current_dir(&registry)
            .env("VAT_HOME", vat_home.path())
            .env("VAT_COMPOSE_PROJECT", project)
            .env("VAT_COMPOSE_STARTUP_TOKEN", token)
            .args(["run", "project.up", "--name", project, "--keep", "always"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn detached-style child");

        let deadline = Instant::now() + Duration::from_secs(8);
        let (vat_id, handoff) = loop {
            let handoff = read_registry(&registry);
            if let Some(vat_id) = handoff["vat_id"].as_str() {
                break (vat_id.to_string(), handoff);
            }
            assert!(
                Instant::now() < deadline,
                "child did not publish a VAT id after parent loss: {handoff}"
            );
            std::thread::sleep(Duration::from_millis(50));
        };
        assert_eq!(handoff["status"], "starting", "registry: {handoff}");
        assert!(
            handoff["startup_pid"].is_null() && handoff["startup_token"].is_null(),
            "child must clear transient handoff fields after publishing its VAT id: {handoff}"
        );
        assert!(
            vat_home
                .path()
                .join("vats")
                .join(&vat_id)
                .join("meta.json")
                .exists(),
            "child-published VAT id must refer to durable state"
        );

        // `ps` owns the readiness projection, then `down` must request
        // cleanup from the VAT parent rather than signaling the historical
        // launcher PID from the registry.
        wait_for_compose_ready(&vat_home, project);
        down_compose(&vat_home, project);
        let _ = child.wait().expect("reap detached-style child");

        let reset = read_registry(&registry);
        assert_eq!(reset["status"], "imported", "registry: {reset}");
        assert!(reset["vat_id"].is_null(), "registry: {reset}");
    }

    #[test]
    fn test_token_handoff_never_binds_same_name_unrelated_vat() {
        let vat_home = TempDir::new().expect("VAT_HOME");
        let project = "token-owned-binding";
        let token = "only-the-compose-child-may-bind";
        let registry = registry_dir(&vat_home, project);
        write_long_lived_vat_toml(&registry);
        write_token_backed_starting_registry(&registry, project, token);
        fs::write(registry.join("ordinary-vat.sh"), "#!/bin/sh\nexit 0\n")
            .expect("write ordinary VAT workspace script");

        // This ordinary direct run deliberately shares the compose project's
        // human name. It is retained in the global VAT store before the real
        // handoff child begins, so name/time discovery would incorrectly bind
        // it if it remained part of the compose startup protocol.
        let unrelated = Command::new(vat_bin())
            .current_dir(&registry)
            .env("VAT_HOME", vat_home.path())
            .args([
                "run",
                "--name",
                project,
                "--json",
                "--",
                "sh",
                "ordinary-vat.sh",
            ])
            .output()
            .expect("ordinary same-name vat run");
        assert!(
            unrelated.status.success(),
            "ordinary VAT run failed: stdout={} stderr={}",
            String::from_utf8_lossy(&unrelated.stdout),
            String::from_utf8_lossy(&unrelated.stderr)
        );
        let unrelated_id: String = serde_json::from_slice::<Value>(&unrelated.stdout)
            .expect("ordinary VAT JSON")
            .get("id")
            .and_then(Value::as_str)
            .expect("ordinary VAT id")
            .to_string();
        assert!(
            vat_home
                .path()
                .join("vats")
                .join(&unrelated_id)
                .join("meta.json")
                .exists(),
            "ordinary same-name VAT must be retained"
        );
        let before_child = read_registry(&registry);
        assert!(before_child["vat_id"].is_null(), "registry: {before_child}");
        assert_eq!(
            before_child["startup_token"], token,
            "registry: {before_child}"
        );

        let mut child = Command::new(vat_bin())
            .current_dir(&registry)
            .env("VAT_HOME", vat_home.path())
            .env("VAT_COMPOSE_PROJECT", project)
            .env("VAT_COMPOSE_STARTUP_TOKEN", token)
            .args(["run", "project.up", "--name", project, "--keep", "always"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn token-owned compose child");

        let deadline = Instant::now() + Duration::from_secs(8);
        let token_owner_id = loop {
            let record = read_registry(&registry);
            if let Some(vat_id) = record["vat_id"].as_str() {
                break vat_id.to_string();
            }
            assert!(
                Instant::now() < deadline,
                "token-owned child did not publish a VAT id: {record}"
            );
            std::thread::sleep(Duration::from_millis(25));
        };
        assert_ne!(
            token_owner_id, unrelated_id,
            "compose registry must never bind a same-name ordinary VAT"
        );
        assert!(
            vat_home
                .path()
                .join("vats")
                .join(&token_owner_id)
                .join("meta.json")
                .exists(),
            "registry must point at the durable token-owner VAT"
        );

        wait_for_compose_ready(&vat_home, project);
        down_compose(&vat_home, project);
        let _ = child.wait().expect("reap token-owned compose child");

        let reset = read_registry(&registry);
        assert_eq!(reset["status"], "imported", "registry: {reset}");
        assert!(reset["vat_id"].is_null(), "registry: {reset}");
        assert!(
            vat_home
                .path()
                .join("vats")
                .join(unrelated_id)
                .join("meta.json")
                .exists(),
            "compose cleanup must not touch the unrelated same-name VAT"
        );
    }

    #[test]
    fn test_mismatched_handoff_token_refuses_before_vat_creation() {
        let vat_home = TempDir::new().expect("VAT_HOME");
        let project = "mismatched-token";
        let registry = registry_dir(&vat_home, project);
        write_long_lived_vat_toml(&registry);
        write_token_backed_starting_registry(&registry, project, "expected-token");

        let output = Command::new(vat_bin())
            .current_dir(&registry)
            .env("VAT_HOME", vat_home.path())
            .env("VAT_COMPOSE_PROJECT", project)
            .env("VAT_COMPOSE_STARTUP_TOKEN", "wrong-token")
            .args(["run", "project.up", "--name", project, "--keep", "always"])
            .output()
            .expect("run with mismatched compose handoff token");
        assert!(
            !output.status.success(),
            "a mismatched handoff token must not start a VAT"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("refusing to create an untracked VAT"),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let record = read_registry(&registry);
        assert!(record["vat_id"].is_null(), "registry: {record}");
        assert_eq!(
            record["startup_token"], "expected-token",
            "registry: {record}"
        );
        assert!(record["startup_pid"].is_null(), "registry: {record}");
        let vat_count = fs::read_dir(vat_home.path().join("vats"))
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .filter(|entry| entry.path().is_dir())
                    .count()
            })
            .unwrap_or(0);
        assert_eq!(
            vat_count, 0,
            "mismatched handoff must fail before creating a VAT or starting services"
        );
    }

    #[test]
    fn test_compose_legacy_started_and_running_records_normalize_then_recover() {
        let vat_home = TempDir::new().expect("VAT_HOME");
        let project = "legacy-status";
        let registry = registry_dir(&vat_home, project);
        write_long_lived_vat_toml(&registry);

        // A legacy `started` record with no VAT id is still an active launch,
        // not permission to erase the record and spawn a duplicate run.
        write_registry(&registry, project, "started", None);
        let active_legacy = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "up", "--project", project, "--detach"])
            .output()
            .expect("compose up with legacy started record");
        assert!(!active_legacy.status.success());
        assert!(
            String::from_utf8_lossy(&active_legacy.stderr).contains("already starting"),
            "stderr: {}",
            String::from_utf8_lossy(&active_legacy.stderr)
        );
        let normalized = read_registry(&registry);
        assert_eq!(normalized["status"], "starting", "registry: {normalized}");
        assert!(normalized["vat_id"].is_null(), "registry: {normalized}");

        // A legacy `running` record whose VAT was already lost is terminal, so
        // compose must reset that binding and create exactly one fresh run.
        write_registry(&registry, project, "running", Some("missing-legacy-vat"));
        let recovered = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "up", "--project", project, "--detach"])
            .output()
            .expect("compose up recovering legacy running record");
        assert!(
            recovered.status.success(),
            "legacy recovery failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&recovered.stdout),
            String::from_utf8_lossy(&recovered.stderr)
        );
        wait_for_compose_ready(&vat_home, project);
        let refreshed = read_registry(&registry);
        assert_ne!(
            refreshed["vat_id"], "missing-legacy-vat",
            "registry: {refreshed}"
        );
        assert!(refreshed["vat_id"].is_string(), "registry: {refreshed}");
        assert!(matches!(
            refreshed["status"].as_str(),
            Some("starting" | "ready")
        ));
        down_compose(&vat_home, project);
    }

    #[test]
    fn test_compose_import_expands_services() {
        let tmpdir = TempDir::new().unwrap();

        let compose = r#"
version: '3'
services:
  api:
    image: myapi:v1
    ports:
      - "3000"
    environment:
      - DEBUG=true
    depends_on:
      - db
  db:
    image: postgres:13
    ports:
      - "5432"
"#;
        let compose_file = tmpdir.path().join("docker-compose.yml");
        fs::write(&compose_file, compose).unwrap();

        // Parse and expand.
        let parsed = vat::compose::parse(&compose_file).unwrap();
        let expanded =
            vat::compose::expand(&parsed, "test", vat::config::ServiceRuntime::Auto).unwrap();

        assert_eq!(expanded.len(), 2);
        let api = expanded.iter().find(|s| s.id == "api").unwrap();
        assert_eq!(api.image, Some("myapi:v1".to_string()));
        assert_eq!(api.requires, vec!["db".to_string()]);

        let db = expanded.iter().find(|s| s.id == "db").unwrap();
        assert_eq!(db.image, Some("postgres:13".to_string()));
        assert_eq!(db.container_port, Some(5432));
    }

    #[test]
    fn test_compose_rejects_unsupported_network_key() {
        let tmpdir = TempDir::new().unwrap();

        let compose = r#"
services:
  app:
    image: myapp:v1
    networks:
      - custom-network
"#;
        let compose_file = tmpdir.path().join("docker-compose.yml");
        fs::write(&compose_file, compose).unwrap();

        let result = vat::compose::parse(&compose_file);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("networks"));
    }

    #[test]
    fn test_compose_rejects_unsupported_secrets_key() {
        let tmpdir = TempDir::new().unwrap();

        let compose = r#"
services:
  app:
    image: myapp:v1
    secrets:
      - db_password
"#;
        let compose_file = tmpdir.path().join("docker-compose.yml");
        fs::write(&compose_file, compose).unwrap();

        let result = vat::compose::parse(&compose_file);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("secrets"));
    }

    #[test]
    fn test_compose_materialize_sets_runtime() {
        let tmpdir = TempDir::new().unwrap();

        let service = vat::config::ServiceConfig {
            id: "test".to_string(),
            requires: Vec::new(),
            cmd: Vec::new(),
            preset: None,
            image: Some("test:v1".to_string()),
            container_port: Some(8080),
            image_env: Default::default(),
            runtime: vat::config::ServiceRuntime::Docker,
            cluster: None,
            external: None,
            k8s_version: None,
            nodes: None,
            spec: None,
            version: None,
            port: vat::config::PortSpec::Auto(String::new()),
            seed: Vec::new(),
            export: Default::default(),
            ready_http: None,
            ready_cmd: Vec::new(),
            timeout_s: 300,
            volumes: Vec::new(),
        };

        let vat_toml = tmpdir.path().join("vat.toml");
        vat::compose::materialize(&[service], &vat_toml).unwrap();

        let content = fs::read_to_string(&vat_toml).unwrap();
        assert!(content.contains("runtime = \"docker\""));
    }
}
// HANDWRITE-END
