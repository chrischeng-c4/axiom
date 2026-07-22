// HANDWRITE-BEGIN gap="sift-layered-deployment-renderer" tracker="1606" reason="Render Sift Dockerfile, CRD, operator, and instance artifacts from checked-in templates."
//! Deterministic offline deployment artifact rendering for Sift.

use anyhow::{bail, Result};

pub const DEFAULT_OPERATOR_IMAGE: &str = "ghcr.io/chrischeng-c4/axiom/sift:0.1.0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DockerfileVariant {
    Source,
    Release,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstanceProfile {
    Dev,
    Staging,
    Prod,
    Template,
}

pub fn dockerfile(variant: DockerfileVariant, version: Option<&str>) -> Result<String> {
    match variant {
        DockerfileVariant::Source => Ok(strip_ownership_markers(include_str!("../Dockerfile"))),
        DockerfileVariant::Release => {
            let version = version
                .map(|version| version.trim_start_matches("sift@"))
                .unwrap_or(env!("CARGO_PKG_VERSION"));
            if version.is_empty() {
                bail!("release Dockerfile version must not be empty");
            }
            Ok(
                strip_ownership_markers(include_str!("../Dockerfile.release")).replace(
                    "SIFT_VERSION=REPLACE_ME",
                    &format!("SIFT_VERSION={version}"),
                ),
            )
        }
    }
}

pub fn crd_yaml() -> String {
    strip_ownership_markers(include_str!("../k8s/crd/sift.yaml"))
}

// <HANDWRITE gap="missing-generator:logic" tracker="1675" reason="Render validated Sift collector DaemonSet assets beside the existing operator layer.">
pub fn operator_yaml(namespace: &str) -> Result<String> {
    operator_yaml_with_image(namespace, DEFAULT_OPERATOR_IMAGE)
}

pub fn operator_yaml_with_image(namespace: &str, image: &str) -> Result<String> {
    validate_namespace(namespace)?;
    validate_image(image)?;
    Ok(
        strip_ownership_markers(include_str!("../k8s/operator/operator.yaml"))
            .replace("sift-system", namespace)
            .replace(DEFAULT_OPERATOR_IMAGE, image),
    )
}

pub fn collector_yaml(namespace: &str, image: &str) -> Result<String> {
    validate_manifest_value("collector namespace", namespace)?;
    validate_manifest_value("collector image", image)?;
    Ok(
        strip_ownership_markers(include_str!("../k8s/collector/daemonset.yaml"))
            .replace("REPLACE_NAMESPACE", namespace)
            .replace("ghcr.io/chrischeng-c4/axiom/sift:0.1.0", image),
    )
}

fn validate_manifest_value(name: &str, value: &str) -> Result<()> {
    if value.trim().is_empty()
        || value.len() > 512
        || value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        bail!("{name} must be a nonempty single bounded token");
    }
    Ok(())
}

fn validate_namespace(namespace: &str) -> Result<()> {
    if namespace.is_empty()
        || namespace.len() > 63
        || namespace.starts_with('-')
        || namespace.ends_with('-')
        || namespace.chars().any(|character| {
            !character.is_ascii_lowercase() && !character.is_ascii_digit() && character != '-'
        })
    {
        bail!("operator namespace must be a valid lowercase DNS label");
    }
    Ok(())
}

fn validate_image(image: &str) -> Result<()> {
    validate_manifest_value("operator image", image)?;
    if image.chars().any(|character| {
        !character.is_ascii_alphanumeric()
            && !matches!(character, '.' | '_' | '-' | '/' | ':' | '@')
    }) {
        bail!("operator image contains characters outside an OCI image reference");
    }
    Ok(())
}
// </HANDWRITE>

pub fn instance_yaml(profile: InstanceProfile) -> String {
    match profile {
        InstanceProfile::Dev => strip_ownership_markers(include_str!("../k8s/instances/dev.yaml")),
        InstanceProfile::Staging => {
            strip_ownership_markers(include_str!("../k8s/overlays/staging/sift.yaml"))
        }
        InstanceProfile::Prod => {
            strip_ownership_markers(include_str!("../k8s/overlays/prod/sift.yaml"))
        }
        InstanceProfile::Template => {
            strip_ownership_markers(include_str!("../k8s/overlays/template/sift.yaml"))
        }
    }
}

fn strip_ownership_markers(body: &str) -> String {
    body.lines()
        .filter(|line| !line.contains("HANDWRITE-BEGIN") && !line.contains("HANDWRITE-END"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

// HANDWRITE-END
