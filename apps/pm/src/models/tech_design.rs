use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TechDesignStatus {
    Draft,
    InReview,
    Approved,
    Superseded,
}

impl Default for TechDesignStatus {
    fn default() -> Self {
        Self::Draft
    }
}

impl std::fmt::Display for TechDesignStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::InReview => write!(f, "in_review"),
            Self::Approved => write!(f, "approved"),
            Self::Superseded => write!(f, "superseded"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TechDesign {
    pub id: String,
    pub project_id: String,
    pub prd_id: Option<String>,
    pub title: String,
    pub version: String,
    pub status: TechDesignStatus,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}
