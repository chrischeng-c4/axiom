// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-generate-gen.md#schema
// CODEGEN-BEGIN
//! Operations artifact emitters.
//!
//! Dockerfiles, docker ignore files, and Kustomize/Kubernetes YAML are already
//! declarative artifacts. The first generator shape preserves their content as
//! structured TD payload and emits the selected target verbatim.

/// Render one operations artifact from a runtime-image or deployment section.
///
/// The section payload is intentionally small and permissive:
///
/// ```yaml
/// runtime_image:
///   artifacts:
///     - path: Dockerfile
///       content: |
///         FROM python:3.12
/// ```
///
/// `deployment:` uses the same `artifacts:` list. A top-level `artifacts:` or
/// `files:` list is also accepted for hand-authored TDs.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/operations.md#source
pub fn emit_operations_artifact_from_yaml(
    yaml: &str,
    entry_path: &str,
    section: &str,
) -> Option<String> {
    let value: serde_yaml::Value = serde_yaml::from_str(yaml).ok()?;
    emit_operations_artifact_from_value(value, entry_path, section)
}

/// Render one operations artifact from an already parsed TD AST section value.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/operations.md#source
pub fn emit_operations_artifact_from_value(
    value: serde_yaml::Value,
    entry_path: &str,
    section: &str,
) -> Option<String> {
    let root = section_payload_root(&value, section).unwrap_or(&value);
    let artifacts = root
        .get("artifacts")
        .and_then(|v| v.as_sequence())
        .or_else(|| root.get("files").and_then(|v| v.as_sequence()))
        .or_else(|| value.get("artifacts").and_then(|v| v.as_sequence()))
        .or_else(|| value.get("files").and_then(|v| v.as_sequence()))?;

    for artifact in artifacts {
        let path = artifact.get("path").and_then(|v| v.as_str())?;
        if path != entry_path {
            continue;
        }
        let content = artifact
            .get("content")
            .or_else(|| artifact.get("body"))
            .and_then(|v| v.as_str())?;
        return Some(normalize_artifact_content(content));
    }
    None
}

fn section_payload_root<'a>(
    value: &'a serde_yaml::Value,
    section: &str,
) -> Option<&'a serde_yaml::Value> {
    let keys: &[&str] = match section {
        "runtime-image" => &["runtime_image", "runtime-image", "runtimeImage"],
        "deployment" => &["deployment"],
        _ => &[],
    };
    keys.iter().find_map(|key| value.get(*key))
}

fn normalize_artifact_content(content: &str) -> String {
    if content.is_empty() || content.ends_with('\n') {
        content.to_string()
    } else {
        format!("{content}\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_runtime_image_artifact_by_path() {
        let yaml = r#"
runtime_image:
  artifacts:
    - path: Dockerfile
      content: |
        FROM python:3.12
        CMD ["python"]
"#;

        let rendered =
            emit_operations_artifact_from_yaml(yaml, "Dockerfile", "runtime-image").unwrap();

        assert_eq!(rendered, "FROM python:3.12\nCMD [\"python\"]\n");
    }

    #[test]
    fn emits_deployment_artifact_by_path() {
        let yaml = r#"
deployment:
  artifacts:
    - path: kustomize/base/kustomization.yaml
      content: |
        resources:
          - deployment.yaml
"#;

        let rendered = emit_operations_artifact_from_yaml(
            yaml,
            "kustomize/base/kustomization.yaml",
            "deployment",
        )
        .unwrap();

        assert_eq!(rendered, "resources:\n  - deployment.yaml\n");
    }
}
// CODEGEN-END
