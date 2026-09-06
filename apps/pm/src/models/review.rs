use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewSeverity {
    Praise,
    Suggestion,
    Blocking,
}

impl Default for ReviewSeverity {
    fn default() -> Self {
        Self::Blocking
    }
}

impl std::fmt::Display for ReviewSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Praise => write!(f, "praise"),
            Self::Suggestion => write!(f, "suggestion"),
            Self::Blocking => write!(f, "blocking"),
        }
    }
}

impl ReviewSeverity {
    pub fn parse_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "praise" => Some(Self::Praise),
            "suggestion" => Some(Self::Suggestion),
            "blocking" => Some(Self::Blocking),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewComment {
    pub id: String,
    pub task_id: String,
    pub reviewer: String,
    pub file_path: Option<String>,
    pub line_number: Option<usize>,
    pub severity: ReviewSeverity,
    pub content: String,
    pub resolved: bool,
    pub resolution_note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
