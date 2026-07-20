//! Dev and Ops helper logic — rendering Dockerfiles and Kubernetes manifests.

use std::path::Path;

/// Write the generated manifest to stdout or a file, and print a `next:` helper command.
pub fn write_or_print(
    out: Option<&Path>,
    default_file: &str,
    body: &str,
    next: impl FnOnce(&Path) -> String,
) -> anyhow::Result<()> {
    if let Some(path) = out {
        let target = if path.extension().is_some() {
            path.to_path_buf()
        } else {
            path.join(default_file)
        };
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&target, body)?;
        println!("wrote {}", target.display());
        println!("next: {}", next(&target));
    } else {
        print!("{body}");
    }
    Ok(())
}

fn strip_ownership_markers(input: &str) -> String {
    let mut out = String::new();
    let mut skipping = false;
    for line in input.lines() {
        if line.contains("SPEC-MANAGED:") || line.contains("CODEGEN-BEGIN") {
            skipping = true;
            continue;
        }
        if line.contains("CODEGEN-END") {
            skipping = false;
            continue;
        }
        if !skipping {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

pub fn render_source_dockerfile() -> String {
    strip_ownership_markers(include_str!("../Dockerfile"))
}

pub fn render_release_dockerfile(version: Option<&str>) -> String {
    let tag = normalize_beam_tag(version);
    let version = tag.trim_start_matches("beam@");
    let template = strip_ownership_markers(include_str!("../Dockerfile.release"));
    let mut out = String::new();
    for line in template.lines() {
        if line.starts_with("#   docker build -f apps/beam/Dockerfile.release -t beam:") {
            out.push_str(&format!(
                "#   docker build -f apps/beam/Dockerfile.release -t beam:{version} \\"
            ));
        } else if line.starts_with("#     --build-arg BEAM_VERSION=") {
            out.push_str(&format!("#     --build-arg BEAM_VERSION={tag} ."));
        } else if line.starts_with("ARG BEAM_VERSION=") {
            out.push_str(&format!("ARG BEAM_VERSION={tag}"));
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

fn normalize_beam_tag(version: Option<&str>) -> String {
    let raw = version
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(env!("CARGO_PKG_VERSION"))
        .trim();
    if raw.starts_with("beam@") {
        raw.to_string()
    } else {
        format!("beam@{raw}")
    }
}

pub fn dockerfile_next_command(
    variant_release: bool,
    version: Option<&str>,
    target: &Path,
) -> String {
    if !variant_release {
        format!("docker build -f {} -t beam:dev .", target.display())
    } else {
        let tag = normalize_beam_tag(version);
        let ver = tag.trim_start_matches("beam@");
        format!(
            "docker build -f {} -t beam:{ver} --build-arg BEAM_VERSION={tag} .",
            target.display()
        )
    }
}

pub fn render_crd_yaml() -> String {
    strip_ownership_markers(include_str!("../k8s/operator/crd.yaml"))
}

pub fn render_operator_yaml(namespace: &str) -> String {
    let rbac = replace_operator_namespace(
        include_str!("../k8s/operator/rbac.yaml"),
        namespace,
    );
    let deploy = replace_operator_namespace(
        include_str!("../k8s/operator/deployment.yaml"),
        namespace,
    );
    let mut out = String::new();
    out.push_str(&rbac);
    out.push_str("\n---\n");
    out.push_str(&deploy);
    out
}

fn replace_operator_namespace(input: &str, namespace: &str) -> String {
    let mut out = String::new();
    for line in input.lines() {
        if line.contains("beam-system") {
            out.push_str(&line.replace("beam-system", namespace));
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

// <HANDWRITE gap="missing-generator:logic" tracker="pending-tracker" reason="logic section in dx.rs is hand-written pending codegen support">
pub fn render_instance_yaml(
    profile: &str,
    name: &str,
    namespace: &str,
    image: Option<&str>,
) -> String {
    // Generate a simple Kubernetes Deployment + Service representing a Beam instance.
    let image = image.unwrap_or("beam:latest");
    format!(
        r#"apiVersion: v1
kind: Service
metadata:
  name: {name}
  namespace: {namespace}
  labels:
    app: {name}
    profile: {profile}
spec:
  ports:
    - port: 7373
      targetPort: 7373
      name: http
  selector:
    app: {name}
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {name}
  namespace: {namespace}
  labels:
    app: {name}
spec:
  replicas: 1
  selector:
    matchLabels:
      app: {name}
  template:
    metadata:
      labels:
        app: {name}
    spec:
      containers:
        - name: beam
          image: {image}
          ports:
            - containerPort: 7373
          env:
            - name: BEAM_PORT
              value: "7373"
            - name: BEAM_HOST
              value: "0.0.0.0"
"#
    )
}
// </HANDWRITE>
