use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum DefectSeverity {
    Cosmetic = 0,
    Minor = 1,
    Major = 2,
    Critical = 3,
}

impl Default for DefectSeverity {
    fn default() -> Self {
        Self::Major
    }
}

impl std::fmt::Display for DefectSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cosmetic => write!(f, "cosmetic"),
            Self::Minor => write!(f, "minor"),
            Self::Major => write!(f, "major"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

impl DefectSeverity {
    pub fn parse_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "cosmetic" => Some(Self::Cosmetic),
            "minor" => Some(Self::Minor),
            "major" => Some(Self::Major),
            "critical" => Some(Self::Critical),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DefectStatus {
    Open,
    InProgress,
    Resolved,
    WontFix,
}

impl Default for DefectStatus {
    fn default() -> Self {
        Self::Open
    }
}

impl std::fmt::Display for DefectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Resolved => write!(f, "resolved"),
            Self::WontFix => write!(f, "wont_fix"),
        }
    }
}

impl DefectStatus {
    pub fn parse_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "open" => Some(Self::Open),
            "in_progress" | "inprogress" => Some(Self::InProgress),
            "resolved" => Some(Self::Resolved),
            "wont_fix" | "wontfix" => Some(Self::WontFix),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Defect {
    pub id: String,
    pub project_id: String,
    pub feature_id: Option<String>,
    pub task_id: Option<String>,
    pub title: String,
    pub description: String,
    pub severity: DefectSeverity,
    pub status: DefectStatus,
    pub reproduction_steps: Option<String>,
    pub error_log: Option<String>,
    pub created_task_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
