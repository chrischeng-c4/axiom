// HANDWRITE-BEGIN gap="missing-generator:ddd-context-map-contract" tracker="#2291" reason="The DDD context-map serialization model is hand-authored until an identity-schema generator owns this protocol."
//! Wire model for the DDD context-map contract.
//!
//! @spec apps/agentic-workflow/tech-design/core/logic/ddd-context-map-contract.md#logic

use serde::{Deserialize, Serialize};

pub const DDD_CONTEXT_MAP_SCHEMA: &str = "aw.ddd-context-map.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DddContextMap {
    pub schema_version: String,
    pub nodes: Vec<DddIdentity>,
    #[serde(default)]
    pub relations: Vec<DddRelation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DddIdentity {
    pub id: String,
    pub kind: DddIdentityKind,
    #[serde(default)]
    pub projections: DddProjection,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DddProjection {
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub markdown_anchors: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DddIdentityKind {
    Context,
    Aggregate,
    UseCase,
    Port,
    Adapter,
    Artifact,
}

impl DddIdentityKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::Aggregate => "aggregate",
            Self::UseCase => "use-case",
            Self::Port => "port",
            Self::Adapter => "adapter",
            Self::Artifact => "artifact",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DddRelation {
    pub from: String,
    pub kind: DddRelationKind,
    pub to: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DddRelationKind {
    Contains,
    Uses,
    DependsOn,
    Implements,
    Realizes,
}
// HANDWRITE-END
