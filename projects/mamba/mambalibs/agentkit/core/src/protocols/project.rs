//! ProjectProtocol — domain contract for a project/repository.

use serde::{Deserialize, Serialize};

/// Hosting platform for a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    GitHub,
    GitLab,
    Jira,
    /// Any platform not covered by the variants above.
    #[serde(untagged)]
    Other(String),
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::GitHub => write!(f, "github"),
            Platform::GitLab => write!(f, "gitlab"),
            Platform::Jira => write!(f, "jira"),
            Platform::Other(s) => write!(f, "{}", s),
        }
    }
}

/// Domain contract for a project/repository.
///
/// Used by `RestructureCodebaseAgent` and other agents that operate
/// at the project level.  Consumers (Conductor, etc.) map their ORM
/// models to/from this type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectProtocol {
    /// Unique project identifier.
    pub id: String,
    /// Human-readable project name.
    pub name: String,
    /// URL to the repository (e.g. `https://github.com/org/repo`).
    pub repo_url: String,
    /// Hosting platform.
    pub platform: Platform,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_protocol_roundtrip() {
        let project = ProjectProtocol {
            id: "proj-001".to_string(),
            name: "cclab-agent".to_string(),
            repo_url: "https://github.com/org/cclab-agent".to_string(),
            platform: Platform::GitHub,
        };

        let json = serde_json::to_string(&project).unwrap();
        let decoded: ProjectProtocol = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.id, "proj-001");
        assert_eq!(decoded.platform, Platform::GitHub);
    }

    #[test]
    fn test_platform_display() {
        assert_eq!(Platform::GitHub.to_string(), "github");
        assert_eq!(Platform::GitLab.to_string(), "gitlab");
        assert_eq!(
            Platform::Other("bitbucket".to_string()).to_string(),
            "bitbucket"
        );
    }
}
