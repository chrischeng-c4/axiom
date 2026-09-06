use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrdStatus {
    Draft,
    InReview,
    Approved,
    Deprecated,
}

impl Default for PrdStatus {
    fn default() -> Self {
        Self::Draft
    }
}

impl std::fmt::Display for PrdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::InReview => write!(f, "in_review"),
            Self::Approved => write!(f, "approved"),
            Self::Deprecated => write!(f, "deprecated"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Prd {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub version: String,
    pub status: PrdStatus,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}
