//! Jira integration for issue tracking.

use super::{
    Issue, IssueComment, IssueFilter, IssueState, IssueSummary, PlatformIntegration, PostedComment,
};
use crate::error::{NovaError, NovaResult};
use crate::tools::{Tool, ToolParameter};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Utc};
use mambalibs_http::client::{HttpClient, HttpMethod, RequestBuilder};
use serde::Deserialize;
use std::sync::Arc;

/// Jira integration for fetching issues.
pub struct JiraIntegration {
    client: HttpClient,
    base_url: String,
    auth_header: String,
    project_key: Option<String>,
}

impl JiraIntegration {
    /// Create a new Jira integration using basic authentication.
    ///
    /// # Arguments
    /// * `base_url` - Jira instance URL (e.g., "https://company.atlassian.net")
    /// * `email` - User email for authentication
    /// * `api_token` - Jira API token
    /// * `project_key` - Optional default project key
    pub fn new(
        base_url: impl Into<String>,
        email: impl Into<String>,
        api_token: impl Into<String>,
        project_key: Option<String>,
    ) -> NovaResult<Self> {
        let client = HttpClient::default_client()
            .map_err(|e| NovaError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        let base_url = base_url.into();
        let base_url = base_url.trim_end_matches('/').to_string();

        let email = email.into();
        let api_token = api_token.into();
        let auth = format!("{}:{}", email, api_token);
        let auth_header = format!("Basic {}", BASE64.encode(auth));

        Ok(Self {
            client,
            base_url,
            auth_header,
            project_key,
        })
    }

    /// Get the API URL.
    fn api_url(&self, path: &str) -> String {
        format!("{}/rest/api/3/{}", self.base_url, path)
    }

    /// Make an authenticated GET request.
    async fn get<T: for<'de> Deserialize<'de>>(&self, url: &str) -> NovaResult<T> {
        let request = RequestBuilder::new(HttpMethod::Get, url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json");

        let response = self
            .client
            .execute_builder(request)
            .await
            .map_err(|e| NovaError::HttpError(format!("Jira API request failed: {}", e)))?;

        if !response.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(NovaError::ApiError(format!(
                "Jira API error {}: {}",
                response.status_code, body
            )));
        }

        response
            .json_as()
            .map_err(|e| NovaError::ApiError(format!("Failed to parse Jira response: {}", e)))
    }

    /// Make an authenticated POST request.
    async fn post<T: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        body: serde_json::Value,
    ) -> NovaResult<T> {
        let request = RequestBuilder::new(HttpMethod::Post, url)
            .header("Authorization", &self.auth_header)
            .json_value(body);

        let response = self
            .client
            .execute_builder(request)
            .await
            .map_err(|e| NovaError::HttpError(format!("Jira API request failed: {}", e)))?;

        if !response.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(NovaError::ApiError(format!(
                "Jira API error {}: {}",
                response.status_code, body
            )));
        }

        response
            .json_as()
            .map_err(|e| NovaError::ApiError(format!("Failed to parse Jira response: {}", e)))
    }

    /// Convert markdown to Jira's Atlassian Document Format.
    fn markdown_to_adf(markdown: &str) -> serde_json::Value {
        // Simple conversion: each line becomes a paragraph
        let paragraphs: Vec<serde_json::Value> = markdown
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                // Handle checkbox lines
                let content =
                    if line.trim().starts_with("- [ ]") || line.trim().starts_with("- [x]") {
                        line.to_string()
                    } else {
                        line.to_string()
                    };
                serde_json::json!({
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": content
                        }
                    ]
                })
            })
            .collect();

        serde_json::json!({
            "type": "doc",
            "version": 1,
            "content": paragraphs
        })
    }

    fn parse_state(status_category: &str) -> IssueState {
        match status_category.to_lowercase().as_str() {
            "to do" | "new" => IssueState::Open,
            "in progress" => IssueState::InProgress,
            "done" => IssueState::Closed,
            _ => IssueState::Open,
        }
    }

    /// Extract plain text from Jira's Atlassian Document Format.
    fn extract_text_from_adf(adf: &serde_json::Value) -> String {
        let mut text = String::new();

        if let Some(content) = adf.get("content").and_then(|v| v.as_array()) {
            for node in content {
                Self::extract_text_from_node(node, &mut text);
            }
        }

        text.trim().to_string()
    }

    fn extract_text_from_node(node: &serde_json::Value, text: &mut String) {
        if let Some(node_text) = node.get("text").and_then(|v| v.as_str()) {
            text.push_str(node_text);
        }

        if let Some(content) = node.get("content").and_then(|v| v.as_array()) {
            for child in content {
                Self::extract_text_from_node(child, text);
            }
        }

        // Add line breaks for block elements
        if let Some(node_type) = node.get("type").and_then(|v| v.as_str()) {
            if matches!(
                node_type,
                "paragraph" | "heading" | "bulletList" | "orderedList"
            ) {
                text.push('\n');
            }
        }
    }
}

#[async_trait]
impl PlatformIntegration for JiraIntegration {
    fn name(&self) -> &str {
        "jira"
    }

    async fn get_issue(&self, id: &str) -> NovaResult<Issue> {
        let url = self.api_url(&format!("issue/{}?expand=renderedFields", id));
        let jira_issue: JiraIssue = self.get(&url).await?;

        let comments = self.get_comments(id).await?;

        let body = jira_issue
            .fields
            .description
            .as_ref()
            .map(|d| Self::extract_text_from_adf(d))
            .unwrap_or_default();

        Ok(Issue {
            id: jira_issue.key.clone(),
            title: jira_issue.fields.summary,
            body,
            state: Self::parse_state(&jira_issue.fields.status.status_category.name),
            author: jira_issue
                .fields
                .reporter
                .map(|r| r.display_name)
                .unwrap_or_default(),
            labels: jira_issue.fields.labels,
            assignees: jira_issue
                .fields
                .assignee
                .map(|a| vec![a.display_name])
                .unwrap_or_default(),
            created_at: jira_issue.fields.created,
            updated_at: jira_issue.fields.updated,
            url: format!("{}/browse/{}", self.base_url, jira_issue.key),
            comments,
            metadata: serde_json::json!({
                "jira": {
                    "id": jira_issue.id,
                    "key": jira_issue.key,
                    "type": jira_issue.fields.issuetype.name,
                    "priority": jira_issue.fields.priority.map(|p| p.name),
                    "project": jira_issue.fields.project.key
                }
            }),
        })
    }

    async fn list_issues(&self, filter: &IssueFilter) -> NovaResult<Vec<IssueSummary>> {
        let mut jql_parts = vec![];

        if let Some(ref project_key) = self.project_key {
            jql_parts.push(format!("project = {}", project_key));
        }

        if let Some(state) = &filter.state {
            let status = match state {
                IssueState::Open => "statusCategory = \"To Do\"",
                IssueState::InProgress => "statusCategory = \"In Progress\"",
                IssueState::Closed => "statusCategory = \"Done\"",
                _ => "",
            };
            if !status.is_empty() {
                jql_parts.push(status.to_string());
            }
        }

        if let Some(labels) = &filter.labels {
            for label in labels {
                jql_parts.push(format!("labels = \"{}\"", label));
            }
        }

        if let Some(assignee) = &filter.assignee {
            jql_parts.push(format!("assignee = \"{}\"", assignee));
        }

        if let Some(query) = &filter.query {
            jql_parts.push(format!("text ~ \"{}\"", query));
        }

        let jql = if jql_parts.is_empty() {
            "ORDER BY created DESC".to_string()
        } else {
            format!("{} ORDER BY created DESC", jql_parts.join(" AND "))
        };

        let max_results = filter.limit.unwrap_or(50).min(100);
        let url = format!(
            "{}?jql={}&maxResults={}",
            self.api_url("search"),
            urlencoding::encode(&jql),
            max_results
        );

        let response: JiraSearchResponse = self.get(&url).await?;

        Ok(response
            .issues
            .into_iter()
            .map(|i| {
                let key = i.key.clone();
                IssueSummary {
                    id: i.key,
                    title: i.fields.summary,
                    state: Self::parse_state(&i.fields.status.status_category.name),
                    labels: i.fields.labels,
                    created_at: i.fields.created,
                    url: format!("{}/browse/{}", self.base_url, key),
                }
            })
            .collect())
    }

    async fn get_comments(&self, issue_id: &str) -> NovaResult<Vec<IssueComment>> {
        let url = self.api_url(&format!("issue/{}/comment", issue_id));
        let response: JiraCommentResponse = self.get(&url).await?;

        Ok(response
            .comments
            .into_iter()
            .map(|c| {
                let body = Self::extract_text_from_adf(&c.body);
                IssueComment {
                    id: c.id,
                    author: c.author.display_name,
                    body,
                    created_at: c.created,
                }
            })
            .collect())
    }

    async fn post_comment(&self, issue_id: &str, body: &str) -> NovaResult<PostedComment> {
        let url = self.api_url(&format!("issue/{}/comment", issue_id));
        let adf_body = Self::markdown_to_adf(body);
        let request_body = serde_json::json!({
            "body": adf_body
        });

        let response: JiraCommentPostResponse = self.post(&url, request_body).await?;

        // Construct the comment URL
        let comment_url = format!(
            "{}/browse/{}?focusedCommentId={}",
            self.base_url, issue_id, response.id
        );

        Ok(PostedComment {
            id: response.id,
            url: comment_url,
        })
    }

    fn into_tools(self: Box<Self>) -> Vec<Box<dyn Tool>> {
        let integration = Arc::new(*self);

        vec![
            Box::new(GetJiraIssueTool {
                integration: integration.clone(),
            }),
            Box::new(ListJiraIssuesTool {
                integration: integration.clone(),
            }),
            Box::new(SearchJiraIssuesTool { integration }),
        ]
    }
}

// Jira API response types

#[derive(Debug, Deserialize)]
struct JiraIssue {
    id: String,
    key: String,
    fields: JiraFields,
}

#[derive(Debug, Deserialize)]
struct JiraFields {
    summary: String,
    description: Option<serde_json::Value>,
    status: JiraStatus,
    issuetype: JiraIssueType,
    priority: Option<JiraPriority>,
    project: JiraProject,
    labels: Vec<String>,
    reporter: Option<JiraUser>,
    assignee: Option<JiraUser>,
    created: DateTime<Utc>,
    updated: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JiraStatus {
    name: String,
    #[serde(rename = "statusCategory")]
    status_category: JiraStatusCategory,
}

#[derive(Debug, Deserialize)]
struct JiraStatusCategory {
    name: String,
}

#[derive(Debug, Deserialize)]
struct JiraIssueType {
    name: String,
}

#[derive(Debug, Deserialize)]
struct JiraPriority {
    name: String,
}

#[derive(Debug, Deserialize)]
struct JiraProject {
    key: String,
}

#[derive(Debug, Deserialize)]
struct JiraUser {
    #[serde(rename = "displayName")]
    display_name: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JiraSearchResponse {
    issues: Vec<JiraIssue>,
    total: u32,
}

#[derive(Debug, Deserialize)]
struct JiraCommentResponse {
    comments: Vec<JiraComment>,
}

#[derive(Debug, Deserialize)]
struct JiraCommentPostResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct JiraComment {
    id: String,
    author: JiraUser,
    body: serde_json::Value,
    created: DateTime<Utc>,
}

// Jira tools

struct GetJiraIssueTool {
    integration: Arc<JiraIntegration>,
}

#[async_trait]
impl Tool for GetJiraIssueTool {
    fn name(&self) -> &str {
        "jira_get_issue"
    }

    fn description(&self) -> &str {
        "Get details of a Jira issue including comments."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "issue_key".to_string(),
            description: "The issue key (e.g., PROJECT-123)".to_string(),
            required: true,
            parameter_type: "string".to_string(),
        }]
    }

    async fn execute(&self, arguments: serde_json::Value) -> NovaResult<serde_json::Value> {
        #[derive(Deserialize)]
        struct Args {
            issue_key: String,
        }

        let args: Args = serde_json::from_value(arguments)?;
        let issue = self.integration.get_issue(&args.issue_key).await?;

        Ok(serde_json::to_value(issue)?)
    }
}

struct ListJiraIssuesTool {
    integration: Arc<JiraIntegration>,
}

#[async_trait]
impl Tool for ListJiraIssuesTool {
    fn name(&self) -> &str {
        "jira_list_issues"
    }

    fn description(&self) -> &str {
        "List Jira issues with optional filters."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "status".to_string(),
                description: "Filter by status category: todo, inprogress, or done".to_string(),
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
                description: "Filter by assignee name".to_string(),
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
            status: Option<String>,
            labels: Option<String>,
            assignee: Option<String>,
            limit: Option<usize>,
        }

        let args: Args = serde_json::from_value(arguments)?;

        let mut filter = IssueFilter::new();

        if let Some(status) = args.status {
            filter = filter.with_state(match status.to_lowercase().as_str() {
                "todo" | "open" => IssueState::Open,
                "inprogress" | "in_progress" => IssueState::InProgress,
                "done" | "closed" => IssueState::Closed,
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

struct SearchJiraIssuesTool {
    integration: Arc<JiraIntegration>,
}

#[async_trait]
impl Tool for SearchJiraIssuesTool {
    fn name(&self) -> &str {
        "jira_search"
    }

    fn description(&self) -> &str {
        "Search Jira issues using text search."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "query".to_string(),
                description: "Search query text".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "limit".to_string(),
                description: "Maximum number of results".to_string(),
                required: false,
                parameter_type: "integer".to_string(),
            },
        ]
    }

    async fn execute(&self, arguments: serde_json::Value) -> NovaResult<serde_json::Value> {
        #[derive(Deserialize)]
        struct Args {
            query: String,
            limit: Option<usize>,
        }

        let args: Args = serde_json::from_value(arguments)?;

        let filter = IssueFilter::new()
            .with_query(args.query.clone())
            .with_limit(args.limit.unwrap_or(20));

        let issues = self.integration.list_issues(&filter).await?;

        Ok(serde_json::json!({
            "query": args.query,
            "issues": issues,
            "count": issues.len()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jira_integration_creation() {
        let integration = JiraIntegration::new(
            "https://company.atlassian.net",
            "user@example.com",
            "api_token",
            Some("PROJECT".to_string()),
        );
        assert!(integration.is_ok());
    }

    #[test]
    fn test_jira_api_url() {
        let integration = JiraIntegration::new(
            "https://company.atlassian.net",
            "user@example.com",
            "token",
            None,
        )
        .unwrap();

        let url = integration.api_url("issue/PROJECT-123");
        assert_eq!(
            url,
            "https://company.atlassian.net/rest/api/3/issue/PROJECT-123"
        );
    }

    #[test]
    fn test_extract_text_from_adf() {
        let adf = serde_json::json!({
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {"type": "text", "text": "Hello "},
                        {"type": "text", "text": "World"}
                    ]
                }
            ]
        });

        let text = JiraIntegration::extract_text_from_adf(&adf);
        assert!(text.contains("Hello World"));
    }

    #[test]
    fn test_parse_state() {
        assert_eq!(JiraIntegration::parse_state("To Do"), IssueState::Open);
        assert_eq!(
            JiraIntegration::parse_state("In Progress"),
            IssueState::InProgress
        );
        assert_eq!(JiraIntegration::parse_state("Done"), IssueState::Closed);
    }
}
