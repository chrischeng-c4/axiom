use serde::{Deserialize, Serialize};

use super::feature::Priority;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    InReview,
    Done,
    Blocked,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Todo
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Todo => write!(f, "todo"),
            Self::InProgress => write!(f, "in_progress"),
            Self::InReview => write!(f, "in_review"),
            Self::Done => write!(f, "done"),
            Self::Blocked => write!(f, "blocked"),
        }
    }
}

impl TaskStatus {
    pub fn parse_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "todo" => Some(Self::Todo),
            "in_progress" | "inprogress" => Some(Self::InProgress),
            "in_review" | "inreview" => Some(Self::InReview),
            "done" => Some(Self::Done),
            "blocked" => Some(Self::Blocked),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub feature_id: Option<String>,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: Priority,
    pub assignee: Option<String>,
    #[serde(default)]
    pub blocked_by: Vec<String>,
    pub result_summary: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
