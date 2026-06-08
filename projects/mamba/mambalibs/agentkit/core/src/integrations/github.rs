//! GitHub integration for issue tracking.

use super::{
    CommitFile, CreatedBranch, CreatedCommit, CreatedPullRequest, Issue, IssueComment, IssueFilter,
    IssueState, IssueSummary, PlatformIntegration, PostedComment, PullRequestParams,
};
use crate::error::{NovaError, NovaResult};
use crate::tools::{Tool, ToolParameter};
use async_trait::async_trait;
use mambalibs_http::client::{HttpClient, HttpMethod, RequestBuilder};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;

/// GitHub integration for fetching issues and pull requests.
pub struct GitHubIntegration {
    client: HttpClient,
    token: String,
    owner: String,
    repo: String,
}

impl GitHubIntegration {
    /// Create a new GitHub integration.
    pub fn new(
        token: impl Into<String>,
        owner: impl Into<String>,
        repo: impl Into<String>,
    ) -> NovaResult<Self> {
        let client = HttpClient::default_client()
            .map_err(|e| NovaError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            token: token.into(),
            owner: owner.into(),
            repo: repo.into(),
        })
    }

    /// Get the API base URL.
    fn api_url(&self, path: &str) -> String {
        format!(
            "https://api.github.com/repos/{}/{}/{}",
            self.owner, self.repo, path
        )
    }

    /// Make an authenticated GET request.
    async fn get<T: for<'de> Deserialize<'de>>(&self, url: &str) -> NovaResult<T> {
        let request = RequestBuilder::new(HttpMethod::Get, url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "cclab-nova");

        let response = self
            .client
            .execute_builder(request)
            .await
            .map_err(|e| NovaError::HttpError(format!("GitHub API request failed: {}", e)))?;

        if !response.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(NovaError::ApiError(format!(
                "GitHub API error {}: {}",
                response.status_code, body
            )));
        }

        response
            .json_as()
            .map_err(|e| NovaError::ApiError(format!("Failed to parse GitHub response: {}", e)))
    }

    /// Resolve a branch name or SHA to a commit SHA.
    async fn resolve_ref_sha(&self, git_ref: &str) -> NovaResult<String> {
        // If it already looks like a SHA (40 hex chars), use it directly.
        if git_ref.len() == 40 && git_ref.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(git_ref.to_string());
        }
        let url = self.api_url(&format!("git/refs/heads/{}", git_ref));
        let data: GitHubRef = self.get(&url).await?;
        Ok(data.object.sha)
    }

    /// Make an authenticated POST request.
    async fn post<T: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        body: serde_json::Value,
    ) -> NovaResult<T> {
        let request = RequestBuilder::new(HttpMethod::Post, url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "cclab-nova")
            .json_value(body);

        let response = self
            .client
            .execute_builder(request)
            .await
            .map_err(|e| NovaError::HttpError(format!("GitHub API request failed: {}", e)))?;

        if !response.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(NovaError::ApiError(format!(
                "GitHub API error {}: {}",
                response.status_code, body
            )));
        }

        response
            .json_as()
            .map_err(|e| NovaError::ApiError(format!("Failed to parse GitHub response: {}", e)))
    }

    /// Make an authenticated PATCH request (used for updating refs).
    async fn patch<T: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        body: serde_json::Value,
    ) -> NovaResult<T> {
        let request = RequestBuilder::new(HttpMethod::Patch, url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "cclab-nova")
            .json_value(body);

        let response = self
            .client
            .execute_builder(request)
            .await
            .map_err(|e| NovaError::HttpError(format!("GitHub API request failed: {}", e)))?;

        if !response.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(NovaError::ApiError(format!(
                "GitHub API error {}: {}",
                response.status_code, body
            )));
        }

        response
            .json_as()
            .map_err(|e| NovaError::ApiError(format!("Failed to parse GitHub response: {}", e)))
    }
}

#[async_trait]
impl PlatformIntegration for GitHubIntegration {
    fn name(&self) -> &str {
        "github"
    }

    async fn get_issue(&self, id: &str) -> NovaResult<Issue> {
        let url = self.api_url(&format!("issues/{}", id));
        let gh_issue: GitHubIssue = self.get(&url).await?;

        // Fetch comments
        let comments = self.get_comments(id).await?;

        Ok(Issue {
            id: gh_issue.number.to_string(),
            title: gh_issue.title,
            body: gh_issue.body.unwrap_or_default(),
            state: match gh_issue.state.as_str() {
                "open" => IssueState::Open,
                "closed" => IssueState::Closed,
                _ => IssueState::Open,
            },
            author: gh_issue.user.login,
            labels: gh_issue.labels.into_iter().map(|l| l.name).collect(),
            assignees: gh_issue.assignees.into_iter().map(|a| a.login).collect(),
            created_at: gh_issue.created_at,
            updated_at: gh_issue.updated_at,
            url: gh_issue.html_url,
            comments,
            metadata: serde_json::json!({
                "github": {
                    "id": gh_issue.id,
                    "node_id": gh_issue.node_id,
                    "milestone": gh_issue.milestone.map(|m| m.title)
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
                    IssueState::Open => "open",
                    IssueState::Closed => "closed",
                    _ => "all",
                }
            ));
        }

        if let Some(labels) = &filter.labels {
            params.push(format!("labels={}", labels.join(",")));
        }

        if let Some(assignee) = &filter.assignee {
            params.push(format!("assignee={}", assignee));
        }

        if let Some(limit) = filter.limit {
            params.push(format!("per_page={}", limit.min(100)));
        }

        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        let issues: Vec<GitHubIssue> = self.get(&url).await?;

        Ok(issues
            .into_iter()
            .map(|i| IssueSummary {
                id: i.number.to_string(),
                title: i.title,
                state: match i.state.as_str() {
                    "open" => IssueState::Open,
                    "closed" => IssueState::Closed,
                    _ => IssueState::Open,
                },
                labels: i.labels.into_iter().map(|l| l.name).collect(),
                created_at: i.created_at,
                url: i.html_url,
            })
            .collect())
    }

    async fn get_comments(&self, issue_id: &str) -> NovaResult<Vec<IssueComment>> {
        let url = self.api_url(&format!("issues/{}/comments", issue_id));
        let comments: Vec<GitHubComment> = self.get(&url).await?;

        Ok(comments
            .into_iter()
            .map(|c| IssueComment {
                id: c.id.to_string(),
                author: c.user.login,
                body: c.body,
                created_at: c.created_at,
            })
            .collect())
    }

    async fn post_comment(&self, issue_id: &str, body: &str) -> NovaResult<PostedComment> {
        let url = self.api_url(&format!("issues/{}/comments", issue_id));
        let request_body = serde_json::json!({
            "body": body
        });

        let response: GitHubCommentResponse = self.post(&url, request_body).await?;

        Ok(PostedComment {
            id: response.id.to_string(),
            url: response.html_url,
        })
    }

    async fn create_branch(&self, branch_name: &str, from_ref: &str) -> NovaResult<CreatedBranch> {
        // Resolve from_ref to a SHA.
        let sha = self.resolve_ref_sha(from_ref).await?;

        let url = self.api_url("git/refs");
        let body = serde_json::json!({
            "ref": format!("refs/heads/{}", branch_name),
            "sha": sha
        });

        let _: serde_json::Value = self.post(&url, body).await?;

        Ok(CreatedBranch {
            name: branch_name.to_string(),
            sha,
        })
    }

    async fn create_commit(
        &self,
        branch: &str,
        message: &str,
        files: &[CommitFile],
    ) -> NovaResult<CreatedCommit> {
        // 1. Get the current commit SHA on the branch.
        let ref_url = self.api_url(&format!("git/refs/heads/{}", branch));
        let ref_data: GitHubRef = self.get(&ref_url).await?;
        let parent_sha = ref_data.object.sha.clone();

        // 2. Get the tree SHA of the parent commit.
        let commit_url = self.api_url(&format!("git/commits/{}", parent_sha));
        let parent_commit: GitHubCommitObject = self.get(&commit_url).await?;
        let base_tree_sha = parent_commit.tree.sha;

        // 3. Create blobs for each file.
        let mut tree_entries: Vec<serde_json::Value> = Vec::new();
        for file in files {
            let blob_url = self.api_url("git/blobs");
            let blob_body = serde_json::json!({
                "content": file.content,
                "encoding": "utf-8"
            });
            let blob: GitHubBlob = self.post(&blob_url, blob_body).await?;
            tree_entries.push(serde_json::json!({
                "path": file.path,
                "mode": "100644",
                "type": "blob",
                "sha": blob.sha
            }));
        }

        // 4. Create a new tree.
        let tree_url = self.api_url("git/trees");
        let tree_body = serde_json::json!({
            "base_tree": base_tree_sha,
            "tree": tree_entries
        });
        let new_tree: GitHubTree = self.post(&tree_url, tree_body).await?;

        // 5. Create the commit.
        let commits_url = self.api_url("git/commits");
        let commit_body = serde_json::json!({
            "message": message,
            "tree": new_tree.sha,
            "parents": [parent_sha]
        });
        let new_commit: GitHubCommitCreated = self.post(&commits_url, commit_body).await?;

        // 6. Update the branch ref.
        let patch_url = self.api_url(&format!("git/refs/heads/{}", branch));
        let patch_body = serde_json::json!({ "sha": new_commit.sha });
        self.patch::<serde_json::Value>(&patch_url, patch_body)
            .await?;

        Ok(CreatedCommit {
            sha: new_commit.sha,
            url: new_commit.html_url,
        })
    }

    async fn create_pull_request(
        &self,
        params: &PullRequestParams,
    ) -> NovaResult<CreatedPullRequest> {
        let url = self.api_url("pulls");
        let body = serde_json::json!({
            "title": params.title,
            "body":  params.body,
            "head":  params.head,
            "base":  params.base
        });

        let pr: GitHubPullRequest = self.post(&url, body).await?;

        Ok(CreatedPullRequest {
            id: pr.id.to_string(),
            url: pr.html_url,
            number: pr.number,
        })
    }

    fn into_tools(self: Box<Self>) -> Vec<Box<dyn Tool>> {
        let integration = Arc::new(*self);

        vec![
            Box::new(GetGitHubIssueTool {
                integration: integration.clone(),
            }),
            Box::new(ListGitHubIssuesTool { integration }),
        ]
    }
}

// GitHub API response types

#[derive(Debug, Deserialize)]
struct GitHubIssue {
    id: u64,
    node_id: String,
    number: u64,
    title: String,
    body: Option<String>,
    state: String,
    user: GitHubUser,
    labels: Vec<GitHubLabel>,
    assignees: Vec<GitHubUser>,
    milestone: Option<GitHubMilestone>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    html_url: String,
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    login: String,
}

#[derive(Debug, Deserialize)]
struct GitHubLabel {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GitHubMilestone {
    title: String,
}

#[derive(Debug, Deserialize)]
struct GitHubComment {
    id: u64,
    user: GitHubUser,
    body: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct GitHubCommentResponse {
    id: u64,
    html_url: String,
}

// Git database / source control response types

#[derive(Debug, Deserialize)]
struct GitHubRef {
    object: GitHubRefObject,
}

#[derive(Debug, Deserialize)]
struct GitHubRefObject {
    sha: String,
}

#[derive(Debug, Deserialize)]
struct GitHubCommitObject {
    tree: GitHubTreeRef,
}

#[derive(Debug, Deserialize)]
struct GitHubTreeRef {
    sha: String,
}

#[derive(Debug, Deserialize)]
struct GitHubBlob {
    sha: String,
}

#[derive(Debug, Deserialize)]
struct GitHubTree {
    sha: String,
}

#[derive(Debug, Deserialize)]
struct GitHubCommitCreated {
    sha: String,
    #[serde(default)]
    html_url: String,
}

#[derive(Debug, Deserialize)]
struct GitHubPullRequest {
    id: u64,
    number: u64,
    html_url: String,
}

// GitHub tools

struct GetGitHubIssueTool {
    integration: Arc<GitHubIntegration>,
}

#[async_trait]
impl Tool for GetGitHubIssueTool {
    fn name(&self) -> &str {
        "github_get_issue"
    }

    fn description(&self) -> &str {
        "Get details of a GitHub issue including comments."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "issue_number".to_string(),
            description: "The issue number".to_string(),
            required: true,
            parameter_type: "string".to_string(),
        }]
    }

    async fn execute(&self, arguments: serde_json::Value) -> NovaResult<serde_json::Value> {
        #[derive(Deserialize)]
        struct Args {
            issue_number: String,
        }

        let args: Args = serde_json::from_value(arguments)?;
        let issue = self.integration.get_issue(&args.issue_number).await?;

        Ok(serde_json::to_value(issue)?)
    }
}

struct ListGitHubIssuesTool {
    integration: Arc<GitHubIntegration>,
}

#[async_trait]
impl Tool for ListGitHubIssuesTool {
    fn name(&self) -> &str {
        "github_list_issues"
    }

    fn description(&self) -> &str {
        "List GitHub issues with optional filters."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "state".to_string(),
                description: "Filter by state: open, closed, or all".to_string(),
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
            limit: Option<usize>,
        }

        let args: Args = serde_json::from_value(arguments)?;

        let mut filter = IssueFilter::new();

        if let Some(state) = args.state {
            filter = filter.with_state(match state.as_str() {
                "open" => IssueState::Open,
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
    fn test_github_integration_creation() {
        let integration = GitHubIntegration::new("token", "owner", "repo");
        assert!(integration.is_ok());
    }

    #[test]
    fn test_github_api_url() {
        let integration = GitHubIntegration::new("token", "owner", "repo").unwrap();
        let url = integration.api_url("issues/1");
        assert_eq!(url, "https://api.github.com/repos/owner/repo/issues/1");
    }
}
