// HANDWRITE-BEGIN gap="missing-generator:hand-written:9c45cd2d" tracker="standardize-gap-projects-mamba-src-pkgmgr-resolver-graph-rs" reason="ResolvedGraph, ResolvedNode, ResolutionError types. Codegen will eventually own these from the schema section once mamba-side schema codegen is wired."
//! Resolver output types — successful graph + structured failure.
//!
//! Schema source: `.aw/tech-design/projects/mamba/pkgmgr/resolver.md#schema`.

use serde::{Deserialize, Serialize};

use crate::pkgmanage::pkgmgr::types::FileHash;

use super::requirement::Requirement;

/// @spec .aw/tech-design/projects/mamba/pkgmgr/resolver.md#schema (ResolvedGraph)
///
/// Successful resolution output — full transitive closure. `nodes` is sorted by
/// `name` for byte-stable output (R5 / AC4); `roots` records the originally
/// requested distribution names so consumers can distinguish direct from
/// transitive dependencies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedGraph {
    pub nodes: Vec<ResolvedNode>,
    pub roots: Vec<String>,
}

/// @spec .aw/tech-design/projects/mamba/pkgmgr/resolver.md#schema (ResolvedNode)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedNode {
    pub name: String,
    pub version: String,
    pub files: Vec<FileHash>,
    pub requires: Vec<Requirement>,
}

/// @spec .aw/tech-design/projects/mamba/pkgmgr/resolver.md#schema (ResolutionError.kind)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionErrorKind {
    EmptyIntersection,
    NoCompatibleVersion,
    MissingPackage,
    MarkerExcludesAll,
    Cycle,
}

/// @spec .aw/tech-design/projects/mamba/pkgmgr/resolver.md#schema (ResolutionError)
///
/// Carries a PubGrub-style human-readable trace plus the set of package names
/// that participated in the conflict (for downstream UX / lockfile diagnostics).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionError {
    pub kind: ResolutionErrorKind,
    pub trace: String,
    pub involved: Vec<String>,
}

impl std::fmt::Display for ResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "resolution failed ({:?}): {}", self.kind, self.trace)
    }
}

impl std::error::Error for ResolutionError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pkgmanage::pkgmgr::resolver::specifier::{Op, VersionSpecifier};

    fn sample_node() -> ResolvedNode {
        ResolvedNode {
            name: "pkg-a".into(),
            version: "1.2.3".into(),
            files: vec![FileHash {
                algorithm: "sha256".into(),
                digest: "deadbeef".into(),
            }],
            requires: vec![Requirement {
                name: "pkg-b".into(),
                specifiers: vec![VersionSpecifier {
                    op: Op::Ge,
                    version: "3.0".into(),
                }],
                extras: vec!["test".into()],
                marker: Some("python_version >= '3.12'".into()),
            }],
        }
    }

    #[test]
    fn resolved_graph_roundtrip_serde() {
        let graph = ResolvedGraph {
            nodes: vec![sample_node()],
            roots: vec!["pkg-a".into()],
        };
        let json = serde_json::to_string(&graph).expect("serialize");
        let back: ResolvedGraph = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(graph, back);
        assert_eq!(back.roots, vec!["pkg-a".to_string()]);
        assert_eq!(back.nodes.len(), 1);
        assert_eq!(back.nodes[0].name, "pkg-a");
        assert_eq!(back.nodes[0].files[0].algorithm, "sha256");
        assert_eq!(back.nodes[0].requires[0].extras, vec!["test".to_string()]);
    }

    #[test]
    fn resolved_node_clone_eq() {
        let a = sample_node();
        let b = a.clone();
        assert_eq!(a, b);
        let debug = format!("{a:?}");
        assert!(debug.contains("pkg-a"));
    }

    #[test]
    fn resolution_error_kind_serde_snake_case() {
        let kinds = [
            (
                ResolutionErrorKind::EmptyIntersection,
                "\"empty_intersection\"",
            ),
            (
                ResolutionErrorKind::NoCompatibleVersion,
                "\"no_compatible_version\"",
            ),
            (ResolutionErrorKind::MissingPackage, "\"missing_package\""),
            (
                ResolutionErrorKind::MarkerExcludesAll,
                "\"marker_excludes_all\"",
            ),
            (ResolutionErrorKind::Cycle, "\"cycle\""),
        ];
        for (kind, expected) in kinds {
            let json = serde_json::to_string(&kind).expect("serialize kind");
            assert_eq!(json, expected);
            let back: ResolutionErrorKind = serde_json::from_str(&json).expect("deserialize kind");
            assert_eq!(back, kind);
        }
    }

    #[test]
    fn resolution_error_display_and_error_trait() {
        let err = ResolutionError {
            kind: ResolutionErrorKind::Cycle,
            trace: "A -> B -> A".into(),
            involved: vec!["A".into(), "B".into()],
        };
        let rendered = format!("{err}");
        assert!(rendered.contains("Cycle"));
        assert!(rendered.contains("A -> B -> A"));
        let as_err: &dyn std::error::Error = &err;
        assert!(as_err.source().is_none());
        assert_eq!(err.involved, vec!["A".to_string(), "B".to_string()]);
        let cloned = err.clone();
        assert_eq!(cloned, err);
    }
}
// HANDWRITE-END
