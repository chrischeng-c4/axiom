// HANDWRITE-BEGIN gap="missing-generator:e2e-test:compose-full-cycle" tracker="#1484" reason="AC5: gated full up -d / ps / logs / down cycle test against a real container/docker backend, using a `container_available()` skip helper mirroring `vat_cluster.rs`'s Docker-gated pattern and `vat_sandbox_microvm.rs`'s container-gated tests -- new test file, hand-authored per this project's e2e-test convention."

//! Container-gated e2e tests for vat compose full lifecycle.
//!
//! Tests the full import/up/ps/logs/down cycle against a real Docker backend
//! by invoking the compiled `vat` binary directly, using a `docker_available()`
//! skip helper. Each invocation gets its own `VAT_HOME` tempdir so the test
//! never touches this repo's real `.vat` state.

#[cfg(test)]
mod tests {
    use std::fs;
    use std::process::Command;
    use std::time::{Duration, Instant};
    use tempfile::TempDir;

    fn vat_bin() -> &'static str {
        env!("CARGO_BIN_EXE_vat")
    }

    /// Check if Docker is available (skip test if not) -- the compose full
    /// cycle test runs services via `--runtime docker`.
    fn docker_available() -> bool {
        Command::new("docker")
            .arg("info")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[test]
    fn test_compose_full_cycle_up_down() {
        if !docker_available() {
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
        let up_json: serde_json::Value = serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|e| panic!("compose up did not print JSON: {e}\n{}", String::from_utf8_lossy(&output.stdout)));
        assert_eq!(up_json["status"], "started", "up_json: {up_json}");
        assert!(up_json["vat_id"].is_string(), "up_json: {up_json}");

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
            if text.contains("web") || Instant::now() >= deadline {
                break text;
            }
            std::thread::sleep(Duration::from_millis(500));
        };
        assert!(
            ps_text.contains("web"),
            "compose ps never showed the `web` service: {ps_text}"
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

        // A second `down` must fail cleanly -- the registry entry is gone.
        let output = Command::new(vat_bin())
            .env("VAT_HOME", vat_home.path())
            .args(["compose", "down", &project])
            .output()
            .unwrap();
        assert!(!output.status.success());
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
        let expanded = vat::compose::expand(&parsed, "test", vat::config::ServiceRuntime::Auto).unwrap();

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
