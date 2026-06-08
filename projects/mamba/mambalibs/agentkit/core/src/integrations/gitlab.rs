//! GitLab integration for issue tracking.

use super::{
    CommitFile, CreatedBranch, CreatedCommit, CreatedPullRequest, Issue, IssueComment, IssueFilter,
    IssueState, IssueSummary, PlatformIntegration, PostedComment, PullRequestParams,
};
use crate::error::{NovaError, NovaResult};
use crate::tools::{Tool, ToolParameter};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use mambalibs_http::client::{HttpClient, HttpMethod, RequestBuilder};
use serde::Deserialize;
use std::sync::Arc;

/// GitLab integration for fetching issues.
pub struct GitLabIntegration {
    client: HttpClient,
    token: String,
    base_url: String,
    project_id: String,
}

impl GitLabIntegration {
    /// Create a new GitLab integration.
    ///
    /// # Arguments
    /// * `token` - GitLab personal access token
    /// * `base_url` - GitLab instance URL (e.g., "https://gitlab.com")
    /// * `project_id` - Project ID or URL-encoded path (e.g., "group/project")
    pub fn new(
        token: impl Into<String>,
        base_url: impl Into<String>,
        project_id: impl Into<String>,
    ) -> NovaResult<Self> {
        let client = HttpClient::default_client()
            .map_err(|e| NovaError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        let base_url = base_url.into();
        let base_url = base_url.trim_end_matches('/').to_string();

        Ok(Self {
            client,
            token: token.into(),
            base_url,
            project_id: project_id.into(),
        })
    }

    /// Get the API base URL for the project.
    fn api_url(&self, path: &str) -> String {
        let encoded_project = urlencoding::encode(&self.project_id);
        format!(
            "{}/api/v4/projects/{}/{}",
            self.base_url, encoded_project, path
        )
    }

    /// Make an authenticated GET request.
    async fn get<T: for<'de> Deserialize<'de>>(&self, url: &str) -> NovaResult<T> {
        let request = RequestBuilder::new(HttpMethod::Get, url)
            .header("PRIVATE-TOKEN", &self.token)
            .header("Accept", "application/json");

        let response = self
            .client
            .execute_builder(request)
            .await
            .map_err(|e| NovaError::HttpError(format!("GitLab API request failed: {}", e)))?;

        if !response.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(NovaError::ApiError(format!(
                "GitLab API error {}: {}",
                response.status_code, body
            )));
        }

        response
            .json_as()
            .map_err(|e| NovaError::ApiError(format!("Failed to parse GitLab response: {}", e)))
    }

    /// Make an authenticated POST request.
    async fn post<T: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        body: serde_json::Value,
    ) -> NovaResult<T> {
        let request = RequestBuilder::new(HttpMethod::Post, url)
            .header("PRIVATE-TOKEN", &self.token)
            .json_value(body);

        let response = self
            .client
            .execute_builder(request)
            .await
            .map_err(|e| NovaError::HttpError(format!("GitLab API request failed: {}", e)))?;

        if !response.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(NovaError::ApiError(format!(
                "GitLab API error {}: {}",
                response.status_code, body
            )));
        }

        response
            .json_as()
            .map_err(|e| NovaError::ApiError(format!("Failed to parse GitLab response: {}", e)))
    }

    fn parse_state(state: &str) -> IssueState {
        match state {
            "opened" => IssueState::Open,
            "closed" => IssueState::Closed,
            _ => IssueState::Open,
        }
    }
}

#[async_trait]
impl PlatformIntegration for GitLabIntegration {
    fn name(&self) -> &str {
        "gitlab"
    }

    async fn get_issue(&self, id: &str) -> NovaResult<Issue> {
        let url = self.api_url(&format!("issues/{}", id));
        let gl_issue: GitLabIssue = self.get(&url).await?;

        let comments = self.get_comments(id).await?;

        Ok(Issue {
            id: gl_issue.iid.to_string(),
            title: gl_issue.title,
            body: gl_issue.description.unwrap_or_default(),
            state: Self::parse_state(&gl_issue.state),
            author: gl_issue.author.username,
            labels: gl_issue.labels,
            assignees: gl_issue
                .assignees
                .unwrap_or_default()
                .into_iter()
                .map(|a| a.username)
                .collect(),
            created_at: gl_issue.created_at,
            updated_at: gl_issue.updated_at,
            url: gl_issue.web_url,
            comments,
            metadata: serde_json::json!({
                "gitlab": {
                    "id": gl_issue.id,
                    "project_id": gl_issue.project_id,
                    "milestone": gl_issue.milestone.map(|m| m.title),
                    "weight": gl_issue.weight
                }
            }),
        })
    }

    async fn list_issues(&self, filter: &IssueFilter) -> NovaResult<Vec<IssueSummary>> {
        let mut url = self.api_url("issues");
        let mut params = vec![];

        if let Some(state) = &filter.state {
            params.push(format!(
                "state={}",
                match state {
                    IssueState::Open => "opened",
                    IssueState::Closed => "closed",
                    _ => "all",
                }
            ));
        }

        if let Some(labels) = &filter.labels {
            params.push(format!("labels={}", labels.join(",")));
        }

        if let Some(assignee) = &filter.assignee {
            params.push(format!("assignee_username={}", assignee));
        }

        if let Some(query) = &filter.query {
            params.push(format!("search={}", urlencoding::encode(query)));
        }

        if let Some(limit) = filter.limit {
            params.push(format!("per_page={}", limit.min(100)));
        }

        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        let issues: Vec<GitLabIssue> = self.get(&url).await?;

        Ok(issues
            .into_iter()
            .map(|i| IssueSummary {
                id: i.iid.to_string(),
                title: i.title,
                state: Self::parse_state(&i.state),
                labels: i.labels,
                created_at: i.created_at,
                url: i.web_url,
            })
            .collect())
    }

    async fn get_comments(&self, issue_id: &str) -> NovaResult<Vec<IssueComment>> {
        let url = self.api_url(&format!("issues/{}/notes", issue_id));
        let notes: Vec<GitLabNote> = self.get(&url).await?;

        Ok(notes
            .into_iter()
            .filter(|n| !n.system) // Filter out system notes
            .map(|n| IssueComment {
                id: n.id.to_string(),
                author: n.author.username,
                body: n.body,
                created_at: n.created_at,
            })
            .collect())
    }

    async fn post_comment(&self, issue_id: &str, body: &str) -> NovaResult<PostedComment> {
        let url = self.api_url(&format!("issues/{}/notes", issue_id));
        let request_body = serde_json::json!({
            "body": body
        });

        let response: GitLabNoteResponse = self.post(&url, request_body).await?;

        // GitLab notes don't have a direct URL, construct it
        let note_url = format!(
            "{}/{}/-/issues/{}#note_{}",
            self.base_url, self.project_id, issue_id, response.id
        );

        Ok(PostedComment {
            id: response.id.to_string(),
            url: note_url,
        })
    }

    async fn create_branch(&self, branch_name: &str, from_ref: &str) -> NovaResult<CreatedBranch> {
        let url = self.api_url("repository/branches");
        let body = serde_json::json!({
            "branch": branch_name,
            "ref": from_ref
        });

        let branch: GitLabBranch = self.post(&url, body).await?;

        Ok(CreatedBranch {
            name: branch.name,
            sha: branch.commit.id,
        })
    }

    async fn create_commit(
        &self,
        branch: &str,
        message: &str,
        files: &[CommitFile],
    ) -> NovaResult<CreatedCommit> {
        let url = self.api_url("repository/commits");

        let actions: Vec<serde_json::Value> = files
            .iter()
            .map(|f| {
                serde_json::json!({
                    "action": "create",
                    "file_path": f.path,
                    "content": f.content
                })
            })
            .collect();

        let body = serde_json::json!({
            "branch": branch,
            "commit_message": message,
            "actions": actions
        });

        let commit: GitLabCommitResponse = self.post(&url, body).await?;

        let commit_url = format!(
            "{}/{}/-/commit/{}",
            self.base_url, self.project_id, commit.id
        );

        Ok(CreatedCommit {
            sha: commit.id,
            url: commit_url,
        })
    }

    async fn create_pull_request(
        &self,
        params: &PullRequestParams,
    ) -> NovaResult<CreatedPullRequest> {
        let url = self.api_url("merge_requests");
        let body = serde_json::json!({
            "title":         params.title,
            "description":   params.body,
            "source_branch": params.head,
            "target_branch": params.base
        });

        let mr: GitLabMergeRequest = self.post(&url, body).await?;

        Ok(CreatedPullRequest {
            id: mr.id.to_string(),
            url: mr.web_url,
            number: mr.iid,
        })
    }

    fn into_tools(self: Box<Self>) -> Vec<Box<dyn Tool>> {
        let integration = Arc::new(*self);

        vec![
            Box::new(GetGitLabIssueTool {
                integration: integration.clone(),
            }),
            Box::new(ListGitLabIssuesTool { integration }),
        ]
    }
}

// GitLab API response types

#[derive(Debug, Deserialize)]
struct GitLabIssue {
    id: u64,
    iid: u64,
    project_id: u64,
    title: String,
    description: Option<String>,
    state: String,
    author: GitLabUser,
    labels: Vec<String>,
    assignees: Option<Vec<GitLabUser>>,
    milestone: Option<GitLabMilestone>,
    weight: Option<u32>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    web_url: String,
}

#[derive(Debug, Deserialize)]
struct GitLabUser {
    username: String,
}

#[derive(Debug, Deserialize)]
struct GitLabMilestone {
    title: String,
}

#[derive(Debug, Deserialize)]
struct GitLabNote {
    id: u64,
    author: GitLabUser,
    body: String,
    system: bool,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct GitLabNoteResponse {
    id: u64,
}

// Source control response types

#[derive(Debug, Deserialize)]
struct GitLabBranchCommit {
    id: String,
}

#[derive(Debug, Deserialize)]
struct GitLabBranch {
    name: String,
    commit: GitLabBranchCommit,
}

#[derive(Debug, Deserialize)]
struct GitLabCommitResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct GitLabMergeRequest {
    id: u64,
    iid: u64,
    web_url: String,
}

// GitLab tools

struct GetGitLabIssueTool {
    integration: Arc<GitLabIntegration>,
}

#[async_trait]
impl Tool for GetGitLabIssueTool {
    fn name(&self) -> &str {
        "gitlab_get_issue"
    }

    fn description(&self) -> &str {
        "Get details of a GitLab issue including comments."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "issue_iid".to_string(),
            description: "The issue IID (project-specific ID)".to_string(),
            required: true,
            parameter_type: "string".to_string(),
        }]
    }

    async fn execute(&self, arguments: serde_json::Value) -> NovaResult<serde_json::Value> {
        #[derive(Deserialize)]
        struct Args {
            issue_iid: String,
        }

        let args: Args = serde_json::from_value(arguments)?;
        let issue = self.integration.get_issue(&args.issue_iid).await?;

        Ok(serde_json::to_value(issue)?)
    }
}

struct ListGitLabIssuesTool {
    integration: Arc<GitLabIntegration>,
}

#[async_trait]
impl Tool for ListGitLabIssuesTool {
    fn name(&self) -> &str {
        "gitlab_list_issues"
    }

    fn description(&self) -> &str {
        "List GitLab issues with optional filters."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "state".to_string(),
                description: "Filter by state: opened, closed, or all".to_string(),
                required: false,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "labels".to_string(),
                description: "Comma-separated list of labels".to_string(),
                required: false,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "assignee".to_string(),
                description: "Filter by assignee username".to_string(),
                required: false,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "search".to_string(),
                description: "Search query".to_string(),
                required: false,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "limit".to_string(),
                description: "Maximum number of issues to return".to_string(),
                required: false,
                parameter_type: "integer".to_string(),
            },
        ]
    }

    async fn execute(&self, arguments: serde_json::Value) -> NovaResult<serde_json::Value> {
        #[derive(Deserialize)]
        struct Args {
            state: Option<String>,
            labels: Option<String>,
            assignee: Option<String>,
            search: Option<String>,
            limit: Option<usize>,
        }

        let args: Args = serde_json::from_value(arguments)?;

        let mut filter = IssueFilter::new();

        if let Some(state) = args.state {
            filter = filter.with_state(match state.as_str() {
                "opened" | "open" => IssueState::Open,
                "closed" => IssueState::Closed,
                _ => IssueState::Open,
            });
        }

        if let Some(labels) = args.labels {
            filter = filter.with_labels(labels.split(',').map(|s| s.trim().to_string()).collect());
        }

        if let Some(assignee) = args.assignee {
            filter = filter.with_assignee(assignee);
        }

        if let Some(search) = args.search {
            filter = filter.with_query(search);
        }

        if let Some(limit) = args.limit {
            filter = filter.with_limit(limit);
        }

        let issues = self.integration.list_issues(&filter).await?;

        Ok(serde_json::json!({
            "issues": issues,
            "count": issues.len()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gitlab_integration_creation() {
        let integration = GitLabIntegration::new("token", "https://gitlab.com", "group/project");
        assert!(integration.is_ok());
    }

    #[test]
    fn test_gitlab_api_url() {
        let integration =
            GitLabIntegration::new("token", "https://gitlab.com", "group/project").unwrap();
        let url = integration.api_url("issues/1");
        assert_eq!(
            url,
            "https://gitlab.com/api/v4/projects/group%2Fproject/issues/1"
        );
    }

    #[test]
    fn test_gitlab_api_url_trailing_slash() {
        let integration =
            GitLabIntegration::new("token", "https://gitlab.com/", "group/project").unwrap();
        let url = integration.api_url("issues");
        assert!(url.starts_with("https://gitlab.com/api/v4"));
    }
}
