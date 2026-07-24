// HANDWRITE-BEGIN gap="missing-generator:e2e-test:compose-import-fixtures" tracker="#1484" reason="AC2/AC7: pure fixture-based expansion-shape assertions and one assertion per R3 hard-reject key, requiring no container/docker binary -- new test file, hand-authored per this project's e2e-test convention (mirrors vat_build.rs's split between a pure and a gated test file)."

//! Pure fixture-based tests for vat compose import.
//!
//! Tests parse/expand/materialize with no container/docker required,
//! and validates the exact error text for all R3 hard-reject keys.

#[cfg(test)]
mod tests {
    use vat::compose;
    use vat::config::ServiceRuntime;
    use std::fs;
    use tempfile::TempDir;

    fn write_compose(tmpdir: &TempDir, content: &str) -> std::path::PathBuf {
        let file = tmpdir.path().join("docker-compose.yml");
        fs::write(&file, content).unwrap();
        file
    }

    #[test]
    fn test_import_basic_image_service() {
        let tmpdir = TempDir::new().unwrap();
        let compose_file = write_compose(
            &tmpdir,
            r#"
services:
  web:
    image: nginx:latest
    ports:
      - "80:80"
"#,
        );

        let file = compose::parse(&compose_file).unwrap();
        let services = compose::expand(&file, "test", ServiceRuntime::Auto).unwrap();

        assert_eq!(services.len(), 1);
        assert_eq!(services[0].id, "web");
        assert_eq!(services[0].image, Some("nginx:latest".to_string()));
        assert_eq!(services[0].container_port, Some(80));
    }

    #[test]
    fn test_expand_bare_ports_container_form() {
        let tmpdir = TempDir::new().unwrap();
        let compose_file = write_compose(
            &tmpdir,
            r#"
services:
  app:
    image: myapp:v1
    ports:
      - "3000"
"#,
        );

        let file = compose::parse(&compose_file).unwrap();
        let services = compose::expand(&file, "test", ServiceRuntime::Auto).unwrap();

        assert_eq!(services[0].container_port, Some(3000));
        match &services[0].port {
            vat::config::PortSpec::Auto(_) => {},
            _ => panic!("expected Auto port"),
        }
    }

    #[test]
    fn test_parse_rejects_bare_environment_key_list() {
        let tmpdir = TempDir::new().unwrap();
        let compose_file = write_compose(
            &tmpdir,
            r#"
services:
  app:
    image: test:v1
    environment:
      - BARE_KEY
"#,
        );

        let result = compose::parse(&compose_file);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("bare key"));
    }

    #[test]
    fn test_parse_rejects_bare_environment_key_map() {
        let tmpdir = TempDir::new().unwrap();
        let compose_file = write_compose(
            &tmpdir,
            r#"
services:
  app:
    image: test:v1
    environment:
      BARE_KEY: null
"#,
        );

        let result = compose::parse(&compose_file);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("bare key") || err_msg.contains("null value"));
    }

    #[test]
    fn test_parse_rejects_deploy_key() {
        let tmpdir = TempDir::new().unwrap();
        let compose_file = write_compose(
            &tmpdir,
            r#"
services:
  app:
    image: test:v1
    deploy:
      resources:
        limits:
          cpus: '0.5'
"#,
        );

        let result = compose::parse(&compose_file);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("deploy") && err_msg.contains("unsupported key"));
    }

    #[test]
    fn test_parse_rejects_healthcheck_key() {
        let tmpdir = TempDir::new().unwrap();
        let compose_file = write_compose(
            &tmpdir,
            r#"
services:
  app:
    image: test:v1
    healthcheck:
      test: ["CMD", "curl", "localhost"]
"#,
        );

        let result = compose::parse(&compose_file);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("healthcheck"));
    }

    #[test]
    fn test_expand_derives_requires_from_depends_on() {
        let tmpdir = TempDir::new().unwrap();
        let compose_file = write_compose(
            &tmpdir,
            r#"
services:
  db:
    image: postgres:latest
  api:
    image: myapi:v1
    depends_on:
      - db
"#,
        );

        let file = compose::parse(&compose_file).unwrap();
        let services = compose::expand(&file, "test", ServiceRuntime::Auto).unwrap();

        let api_service = services.iter().find(|s| s.id == "api").unwrap();
        assert_eq!(api_service.requires, vec!["db"]);
    }

    #[test]
    fn test_materialize_writes_synthesized_runner() {
        let tmpdir = TempDir::new().unwrap();
        let compose_file = write_compose(
            &tmpdir,
            r#"
services:
  web:
    image: nginx:latest
  api:
    image: myapi:v1
"#,
        );

        let file = compose::parse(&compose_file).unwrap();
        let services = compose::expand(&file, "test", ServiceRuntime::Auto).unwrap();
        let vat_toml = tmpdir.path().join("vat.toml");

        compose::materialize(&services, &vat_toml).unwrap();

        let content = fs::read_to_string(&vat_toml).unwrap();
        assert!(content.contains("[[services]]"), "Missing [[services]] section");
        assert!(content.contains("id = \"web\""), "Missing web service");
        assert!(content.contains("id = \"api\""), "Missing api service");
        assert!(content.contains("[[runners]]"), "Missing [[runners]] section");
        assert!(content.contains("id = \"project.up\""), "Missing project.up runner");
        assert!(content.contains("requires") && content.contains("web") && content.contains("api"),
                "Missing requires array with services: {}", content);
    }

    #[test]
    fn test_parse_accepts_x_extension_keys() {
        let tmpdir = TempDir::new().unwrap();
        let compose_file = write_compose(
            &tmpdir,
            r#"
services:
  app:
    image: test:v1
    x-custom: value
version: '3'
x-toplevel: custom
"#,
        );

        let result = compose::parse(&compose_file);
        assert!(result.is_ok());
    }
}
// HANDWRITE-END
