// HANDWRITE-BEGIN gap="sift-layered-deployment-renderer" tracker="1606" reason="Render Sift Dockerfile, CRD, operator, and instance artifacts from checked-in templates."
//! Deterministic offline deployment artifact rendering for Sift.

use anyhow::{bail, Result};

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

// <HANDWRITE gap="missing-generator:logic" tracker="pending-tracker" reason="logic section in deploy.rs is hand-written pending codegen support">
pub fn operator_yaml(namespace: &str) -> Result<String> {
    if namespace.trim().is_empty() {
        bail!("operator namespace must not be empty");
    }
    Ok(
        strip_ownership_markers(include_str!("../k8s/operator/operator.yaml"))
            .replace("namespace: sift-system", &format!("namespace: {namespace}")),
    )
}
// </HANDWRITE>

pub fn instance_yaml(profile: InstanceProfile) -> String {
    match profile {
        InstanceProfile::Dev => strip_ownership_markers(include_str!("../k8s/instances/dev.yaml")),
        InstanceProfile::Staging => r#"apiVersion: sift.axiom.dev/v1alpha1
kind: Sift
metadata: { name: sift, namespace: staging }
spec:
  image: ghcr.io/chrischeng-c4/axiom/sift:0.1.0
  replicasPerShard: 3
  voterCount: 3
  dataSize: 100Gi
  auth: required
"#
        .to_string(),
        InstanceProfile::Prod => r#"apiVersion: sift.axiom.dev/v1alpha1
kind: Sift
metadata: { name: sift, namespace: production }
spec:
  image: ghcr.io/chrischeng-c4/axiom/sift:0.1.0
  replicasPerShard: 3
  voterCount: 3
  dataSize: 500Gi
  auth: required
  backup:
    schedule: "0 * * * *"
    destination: REPLACE_ME__OFF_NODE_BACKUP_URI
    retentionSecs: 604800
"#
        .to_string(),
        InstanceProfile::Template => r#"apiVersion: sift.axiom.dev/v1alpha1
kind: Sift
metadata: { name: REPLACE_ME__NAME, namespace: REPLACE_ME__NAMESPACE }
spec:
  image: REPLACE_ME__IMAGE
  replicasPerShard: REPLACE_ME__REPLICAS
  voterCount: REPLACE_ME__VOTERS
  dataSize: REPLACE_ME__DATA_SIZE
  auth: required
"#
        .to_string(),
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
