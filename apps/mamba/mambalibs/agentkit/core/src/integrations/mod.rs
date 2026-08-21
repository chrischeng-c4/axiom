//! Platform integrations for issue tracking and project management.
//!
//! This module provides integrations with external platforms like GitHub,
//! GitLab, and Jira for fetching issues and project information.

mod github;
mod gitlab;
mod jira;

pub use github::GitHubIntegration;
pub use gitlab::GitLabIntegration;
pub use jira::JiraIntegration;

use crate::error::{NovaError, NovaResult};
use crate::tools::Tool;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// An issue from an external platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Issue ID or number.
    pub id: String,
    /// Issue title.
    pub title: String,
    /// Issue description/body.
    pub body: String,
    /// Issue state (open, closed, etc.).
    pub state: IssueState,
    /// Issue author.
    pub author: String,
    /// Labels/tags.
    pub labels: Vec<String>,
    /// Assigned users.
    pub assignees: Vec<String>,
    /// When the issue was created.
    pub created_at: DateTime<Utc>,
    /// When the issue was last updated.
    pub updated_at: DateTime<Utc>,
    /// URL to the issue.
    pub url: String,
    /// Comments on the issue.
    pub comments: Vec<IssueComment>,
    /// Platform-specific metadata.
    pub metadata: serde_json::Value,
}

/// Issue state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueState {
    Open,
    Closed,
    InProgress,
    Resolved,
    Wontfix,
}

impl Default for IssueState {
    fn default() -> Self {
        Self::Open
    }
}

/// A comment on an issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueComment {
    /// Comment ID.
    pub id: String,
    /// Comment author.
    pub author: String,
    /// Comment body.
    pub body: String,
    /// When the comment was created.
    pub created_at: DateTime<Utc>,
}

/// Summary information about an issue for listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueSummary {
    /// Issue ID or number.
    pub id: String,
    /// Issue title.
    pub title: String,
    /// Issue state.
    pub state: IssueState,
    /// Labels.
    pub labels: Vec<String>,
    /// When the issue was created.
    pub created_at: DateTime<Utc>,
    /// URL to the issue.
    pub url: String,
}

impl From<&Issue> for IssueSummary {
    fn from(issue: &Issue) -> Self {
        Self {
            id: issue.id.clone(),
            title: issue.title.clone(),
            state: issue.state,
            labels: issue.labels.clone(),
            created_at: issue.created_at,
            url: issue.url.clone(),
        }
    }
}

/// Filter for listing issues.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IssueFilter {
    /// Filter by state.
    pub state: Option<IssueState>,
    /// Filter by labels (all must match).
    pub labels: Option<Vec<String>>,
    /// Filter by assignee.
    pub assignee: Option<String>,
    /// Search query.
    pub query: Option<String>,
    /// Maximum number of results.
    pub limit: Option<usize>,
}

impl IssueFilter {
    /// Create a new empty filter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by state.
    pub fn with_state(mut self, state: IssueState) -> Self {
        self.state = Some(state);
        self
    }

    /// Filter by labels.
    pub fn with_labels(mut self, labels: Vec<String>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Filter by assignee.
    pub fn with_assignee(mut self, assignee: impl Into<String>) -> Self {
        self.assignee = Some(assignee.into());
        self
    }

    /// Search query.
    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    /// Maximum results.
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Result of posting a comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostedComment {
    /// The comment ID assigned by the platform.
    pub id: String,
    /// URL to the comment.
    pub url: String,
}

// ============================================================
// Source control types
// ============================================================

/// A file to include in a commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitFile {
    /// Repository-relative path (e.g. `src/lib.rs`).
    pub path: String,
    /// Full file content (UTF-8).
    pub content: String,
}

/// Result of creating a branch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedBranch {
    /// Branch name.
    pub name: String,
    /// Commit SHA the branch points at.
    pub sha: String,
}

/// Result of creating a commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedCommit {
    /// Commit SHA.
    pub sha: String,
    /// URL to the commit on the platform.
    pub url: String,
}

/// Parameters for opening a pull / merge request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestParams {
    /// PR/MR title.
    pub title: String,
    /// PR/MR body / description.
    pub body: String,
    /// Source branch (head).
    pub head: String,
    /// Target branch (base).
    pub base: String,
}

/// Result of creating a pull / merge request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedPullRequest {
    /// Platform-assigned ID string.
    pub id: String,
    /// URL to view the PR/MR.
    pub url: String,
    /// PR/MR number on the platform.
    pub number: u64,
}

/// Parsed response from a clarification comment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParsedClarificationResponse {
    /// Selected checkbox options (labels).
    pub selected_options: Vec<String>,
    /// Free-form text reply (if any).
    pub reply_text: Option<String>,
}

/// Option for a clarification question.
#[derive(Debug, Clone)]
pub struct ClarificationQuestionOption {
    /// The option label.
    pub label: String,
    /// Whether this is the recommended option.
    pub recommended: bool,
}

/// Format a clarification question as a markdown comment.
///
/// Creates a comment with checkboxes that users can select.
pub fn format_clarification_comment(
    question: &str,
    options: &[ClarificationQuestionOption],
    multi_select: bool,
) -> String {
    let mut comment = String::new();
    comment.push_str("## 🤖 需要您的輸入\n\n");
    comment.push_str(&format!("**問題**: {}\n\n", question));

    if !options.is_empty() {
        if multi_select {
            comment.push_str("請勾選（可多選）:\n");
        } else {
            comment.push_str("請勾選（單選）:\n");
        }

        for opt in options {
            if opt.recommended {
                comment.push_str(&format!("- [ ] {} ⭐ 推薦\n", opt.label));
            } else {
                comment.push_str(&format!("- [ ] {}\n", opt.label));
            }
        }
        comment.push('\n');
    }

    comment.push_str("---\n");
    comment.push_str("*如果以上都不適合，請直接回覆說明。*\n");

    comment
}

/// Parse a markdown comment to extract selected checkboxes and reply text.
///
/// Handles both edited checkbox selections (`[x]`) and reply comments.
pub fn parse_clarification_response(comment_body: &str) -> ParsedClarificationResponse {
    let mut response = ParsedClarificationResponse::default();

    let lines: Vec<&str> = comment_body.lines().collect();
    let mut in_checkbox_section = false;
    let mut reply_lines = Vec::new();

    for line in lines {
        let trimmed = line.trim();

        // Check for checked checkbox: - [x] or - [X]
        if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            in_checkbox_section = true;
            // Extract the label (remove checkbox, recommended marker, and trim)
            let label = trimmed[5..]
                .trim()
                .trim_end_matches("⭐ 推薦")
                .trim_end_matches("⭐ Recommended")
                .trim_end_matches("(Recommended)")
                .trim_end_matches("(推薦)")
                .trim();
            if !label.is_empty() {
                response.selected_options.push(label.to_string());
            }
        } else if trimmed.starts_with("- [ ]") {
            // Unchecked checkbox, just mark we're in checkbox section
            in_checkbox_section = true;
        } else if !trimmed.is_empty()
            && !trimmed.starts_with("##")
            && !trimmed.starts_with("**")
            && !trimmed.starts_with("---")
            && !trimmed.starts_with("*如果")
            && !trimmed.starts_with("請勾選")
        {
            // This is reply text (not part of the template)
            if !in_checkbox_section || !trimmed.starts_with('-') {
                reply_lines.push(trimmed);
            }
        }
    }

    if !reply_lines.is_empty() {
        response.reply_text = Some(reply_lines.join("\n"));
    }

    response
}

/// Trait for platform integrations.
///
/// Platform integrations provide access to external issue tracking,
/// project management, and source control systems.
#[async_trait]
pub trait PlatformIntegration: Send + Sync {
    /// Get the platform name.
    fn name(&self) -> &str;

    /// Get a single issue by ID.
    async fn get_issue(&self, id: &str) -> NovaResult<Issue>;

    /// List issues matching a filter.
    async fn list_issues(&self, filter: &IssueFilter) -> NovaResult<Vec<IssueSummary>>;

    /// Get comments for an issue.
    async fn get_comments(&self, issue_id: &str) -> NovaResult<Vec<IssueComment>>;

    /// Post a comment to an issue.
    async fn post_comment(&self, issue_id: &str, body: &str) -> NovaResult<PostedComment>;

    // ---- Source control ----

    /// Create a new branch from `from_ref` (branch name or commit SHA).
    ///
    /// Default implementation returns [`NovaError::PlatformError`].
    /// Override in platforms that support source control.
    async fn create_branch(
        &self,
        _branch_name: &str,
        _from_ref: &str,
    ) -> NovaResult<CreatedBranch> {
        Err(NovaError::PlatformError(format!(
            "{} does not support source control",
            self.name()
        )))
    }

    /// Create a single commit on `branch` containing all `files`.
    ///
    /// Default implementation returns [`NovaError::PlatformError`].
    async fn create_commit(
        &self,
        _branch: &str,
        _message: &str,
        _files: &[CommitFile],
    ) -> NovaResult<CreatedCommit> {
        Err(NovaError::PlatformError(format!(
            "{} does not support source control",
            self.name()
        )))
    }

    /// Open a pull request (GitHub) or merge request (GitLab).
    ///
    /// Default implementation returns [`NovaError::PlatformError`].
    async fn create_pull_request(
        &self,
        _params: &PullRequestParams,
    ) -> NovaResult<CreatedPullRequest> {
        Err(NovaError::PlatformError(format!(
            "{} does not support pull requests",
            self.name()
        )))
    }

    /// Convert this integration into a set of tools.
    fn into_tools(self: Box<Self>) -> Vec<Box<dyn Tool>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_filter_builder() {
        let filter = IssueFilter::new()
            .with_state(IssueState::Open)
            .with_labels(vec!["bug".to_string()])
            .with_limit(10);

        assert_eq!(filter.state, Some(IssueState::Open));
        assert_eq!(filter.labels, Some(vec!["bug".to_string()]));
        assert_eq!(filter.limit, Some(10));
    }

    #[test]
    fn test_issue_summary_from_issue() {
        let issue = Issue {
            id: "123".to_string(),
            title: "Test Issue".to_string(),
            body: "Description".to_string(),
            state: IssueState::Open,
            author: "user".to_string(),
            labels: vec!["bug".to_string()],
            assignees: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            url: "https://example.com/issue/123".to_string(),
            comments: vec![],
            metadata: serde_json::json!({}),
        };

        let summary = IssueSummary::from(&issue);
        assert_eq!(summary.id, "123");
        assert_eq!(summary.title, "Test Issue");
        assert_eq!(summary.state, IssueState::Open);
    }

    #[test]
    fn test_format_clarification_comment() {
        let options = vec![
            ClarificationQuestionOption {
                label: "OAuth 2.0".to_string(),
                recommended: true,
            },
            ClarificationQuestionOption {
                label: "API Key".to_string(),
                recommended: false,
            },
        ];

        let comment = format_clarification_comment("Which auth method?", &options, false);

        assert!(comment.contains("🤖 需要您的輸入"));
        assert!(comment.contains("Which auth method?"));
        assert!(comment.contains("- [ ] OAuth 2.0 ⭐ 推薦"));
        assert!(comment.contains("- [ ] API Key"));
        assert!(comment.contains("單選"));
    }

    #[test]
    fn test_format_clarification_comment_multiselect() {
        let options = vec![
            ClarificationQuestionOption {
                label: "Feature A".to_string(),
                recommended: false,
            },
            ClarificationQuestionOption {
                label: "Feature B".to_string(),
                recommended: true,
            },
        ];

        let comment = format_clarification_comment("Which features?", &options, true);
        assert!(comment.contains("可多選"));
    }

    #[test]
    fn test_parse_clarification_response_checkbox() {
        let comment = r#"## 🤖 需要您的輸入

**問題**: Which auth method?

請勾選（單選）:
- [ ] OAuth 2.0 ⭐ 推薦
- [x] API Key

---
*如果以上都不適合，請直接回覆說明。*
"#;

        let response = parse_clarification_response(comment);
        assert_eq!(response.selected_options, vec!["API Key"]);
        assert!(response.reply_text.is_none());
    }

    #[test]
    fn test_parse_clarification_response_with_reply() {
        let comment = "I think we should use JWT tokens instead.";

        let response = parse_clarification_response(comment);
        assert!(response.selected_options.is_empty());
        assert_eq!(
            response.reply_text,
            Some("I think we should use JWT tokens instead.".to_string())
        );
    }

    #[test]
    fn test_parse_clarification_response_multiple_selected() {
        let comment = r#"- [x] Feature A
- [x] Feature B
- [ ] Feature C

Additional context here."#;

        let response = parse_clarification_response(comment);
        assert_eq!(response.selected_options.len(), 2);
        assert!(response.selected_options.contains(&"Feature A".to_string()));
        assert!(response.selected_options.contains(&"Feature B".to_string()));
        assert_eq!(
            response.reply_text,
            Some("Additional context here.".to_string())
        );
    }
}
