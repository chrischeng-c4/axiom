//! Analysis tools for the AnalystAgent.
//!
//! These tools support requirements analysis workflows:
//! - AskUserTool: Pause execution to gather user input
//! - TakeNoteTool: Store findings in session state
//! - WebSearchTool: Search the web for information
//! - WebFetchTool: Fetch and parse web content

use crate::error::{NovaError, NovaResult};
use crate::storage::{Finding, FindingSeverity, SessionState};
use crate::tools::tool::{Tool, ToolParameter};
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Tool for asking questions to the user.
///
/// When this tool is called, it signals that the agent needs user input
/// before continuing. The agent loop should pause and wait for a response.
pub struct AskUserTool;

impl AskUserTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AskUserTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct AskUserArgs {
    question: String,
    context: Option<String>,
    options: Option<Vec<String>>,
}

#[async_trait]
impl Tool for AskUserTool {
    fn name(&self) -> &str {
        "ask_user"
    }

    fn description(&self) -> &str {
        "Ask the user a question. Use this when you need clarification or additional information."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "question".to_string(),
                description: "The question to ask the user".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "context".to_string(),
                description: "Optional context to help the user understand the question"
                    .to_string(),
                required: false,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "options".to_string(),
                description: "Optional list of suggested answers".to_string(),
                required: false,
                parameter_type: "array".to_string(),
            },
        ]
    }

    async fn execute(&self, arguments: serde_json::Value) -> NovaResult<serde_json::Value> {
        let args: AskUserArgs = serde_json::from_value(arguments)?;

        // Return a structured response that the agent loop can interpret
        Ok(serde_json::json!({
            "status": "pending",
            "type": "user_input_required",
            "question": args.question,
            "context": args.context,
            "options": args.options
        }))
    }
}

/// Tool for taking notes during analysis.
///
/// Notes are stored in the session state and can be used to build
/// the final analysis report.
pub struct TakeNoteTool {
    session: Arc<RwLock<SessionState>>,
}

impl TakeNoteTool {
    /// Create a new TakeNoteTool with a shared session state.
    pub fn new(session: Arc<RwLock<SessionState>>) -> Self {
        Self { session }
    }
}

#[derive(Debug, Deserialize)]
struct TakeNoteArgs {
    content: String,
    category: Option<String>,
}

#[async_trait]
impl Tool for TakeNoteTool {
    fn name(&self) -> &str {
        "take_note"
    }

    fn description(&self) -> &str {
        "Record a note or observation during analysis. Notes are saved to the session."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "content".to_string(),
                description: "The note content to record".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "category".to_string(),
                description: "Optional category (e.g., 'requirement', 'risk', 'question')"
                    .to_string(),
                required: false,
                parameter_type: "string".to_string(),
            },
        ]
    }

    async fn execute(&self, arguments: serde_json::Value) -> NovaResult<serde_json::Value> {
        let args: TakeNoteArgs = serde_json::from_value(arguments)?;

        let mut session = self.session.write().await;
        session.add_note(&args.content, args.category.clone());
        let note_count = session.notes.len();

        Ok(serde_json::json!({
            "success": true,
            "content": args.content,
            "category": args.category,
            "total_notes": note_count
        }))
    }
}

/// Tool for recording findings during analysis.
pub struct RecordFindingTool {
    session: Arc<RwLock<SessionState>>,
}

impl RecordFindingTool {
    /// Create a new RecordFindingTool with a shared session state.
    pub fn new(session: Arc<RwLock<SessionState>>) -> Self {
        Self { session }
    }
}

#[derive(Debug, Deserialize)]
struct RecordFindingArgs {
    title: String,
    description: String,
    severity: String,
    sources: Option<Vec<String>>,
}

#[async_trait]
impl Tool for RecordFindingTool {
    fn name(&self) -> &str {
        "record_finding"
    }

    fn description(&self) -> &str {
        "Record a finding or conclusion from the analysis."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "title".to_string(),
                description: "Short title for the finding".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "description".to_string(),
                description: "Detailed description of the finding".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "severity".to_string(),
                description: "Severity level: info, low, medium, high, critical".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "sources".to_string(),
                description: "Optional list of source references".to_string(),
                required: false,
                parameter_type: "array".to_string(),
            },
        ]
    }

    async fn execute(&self, arguments: serde_json::Value) -> NovaResult<serde_json::Value> {
        let args: RecordFindingArgs = serde_json::from_value(arguments)?;

        let severity = match args.severity.to_lowercase().as_str() {
            "info" => FindingSeverity::Info,
            "low" => FindingSeverity::Low,
            "medium" => FindingSeverity::Medium,
            "high" => FindingSeverity::High,
            "critical" => FindingSeverity::Critical,
            _ => {
                return Err(NovaError::InvalidArguments(format!(
                    "Invalid severity: {}. Must be one of: info, low, medium, high, critical",
                    args.severity
                )));
            }
        };

        let mut finding = Finding::new(&args.title, &args.description, severity);
        if let Some(sources) = args.sources {
            for source in sources {
                finding = finding.with_source(source);
            }
        }

        let mut session = self.session.write().await;
        session.add_finding(finding);
        let finding_count = session.findings.len();

        Ok(serde_json::json!({
            "success": true,
            "title": args.title,
            "severity": args.severity,
            "total_findings": finding_count
        }))
    }
}

/// Tool for searching the web.
pub struct WebSearchTool {
    http_client: mambalibs_http::client::HttpClient,
    max_results: usize,
}

impl WebSearchTool {
    /// Create a new WebSearchTool.
    pub fn new() -> NovaResult<Self> {
        let http_client = mambalibs_http::client::HttpClient::default_client()
            .map_err(|e| NovaError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            http_client,
            max_results: 10,
        })
    }

    /// Set the maximum number of results to return.
    pub fn with_max_results(mut self, max: usize) -> Self {
        self.max_results = max;
        self
    }
}

impl Default for WebSearchTool {
    fn default() -> Self {
        Self::new().expect("Failed to create WebSearchTool")
    }
}

#[derive(Debug, Deserialize)]
struct WebSearchArgs {
    query: String,
    max_results: Option<usize>,
}

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the web for information. Returns a list of relevant results."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "query".to_string(),
                description: "The search query".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "max_results".to_string(),
                description: "Maximum number of results to return".to_string(),
                required: false,
                parameter_type: "integer".to_string(),
            },
        ]
    }

    async fn execute(&self, arguments: serde_json::Value) -> NovaResult<serde_json::Value> {
        let args: WebSearchArgs = serde_json::from_value(arguments)?;
        let max_results = args.max_results.unwrap_or(self.max_results);

        // Use DuckDuckGo HTML search (no API key required)
        let url = format!(
            "https://html.duckduckgo.com/html/?q={}",
            urlencoding::encode(&args.query)
        );

        let response = self
            .http_client
            .get(&url)
            .await
            .map_err(|e| NovaError::HttpError(format!("Search request failed: {}", e)))?;

        let body = response
            .text()
            .map_err(|e| NovaError::HttpError(format!("Failed to read response: {}", e)))?;

        // Parse search results from HTML
        let results = parse_duckduckgo_results(&body, max_results);

        Ok(serde_json::json!({
            "query": args.query,
            "results": results,
            "count": results.len()
        }))
    }
}

/// Parse DuckDuckGo HTML search results.
fn parse_duckduckgo_results(html: &str, max_results: usize) -> Vec<serde_json::Value> {
    let mut results = Vec::new();

    // Simple regex-based parsing for result links and snippets
    // DuckDuckGo HTML uses class="result__a" for links and "result__snippet" for descriptions
    let link_re = regex::Regex::new(r#"class="result__a"[^>]*href="([^"]+)"[^>]*>([^<]+)</a>"#)
        .expect("Invalid regex");
    let snippet_re =
        regex::Regex::new(r#"class="result__snippet"[^>]*>([^<]+)</a>"#).expect("Invalid regex");

    let links: Vec<_> = link_re.captures_iter(html).collect();
    let snippets: Vec<_> = snippet_re.captures_iter(html).collect();

    for (i, link_cap) in links.iter().take(max_results).enumerate() {
        let url = link_cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let title = link_cap
            .get(2)
            .map(|m| html_escape::decode_html_entities(m.as_str()).to_string())
            .unwrap_or_default();

        let snippet = snippets
            .get(i)
            .and_then(|cap| cap.get(1))
            .map(|m| html_escape::decode_html_entities(m.as_str()).to_string())
            .unwrap_or_default();

        if !url.is_empty() {
            results.push(serde_json::json!({
                "title": title,
                "url": url,
                "snippet": snippet
            }));
        }
    }

    results
}

/// Tool for fetching and parsing web content.
pub struct WebFetchTool {
    http_client: mambalibs_http::client::HttpClient,
    max_content_length: usize,
}

impl WebFetchTool {
    /// Create a new WebFetchTool.
    pub fn new() -> NovaResult<Self> {
        let http_client = mambalibs_http::client::HttpClient::default_client()
            .map_err(|e| NovaError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            http_client,
            max_content_length: 100_000, // 100KB default
        })
    }

    /// Set the maximum content length to fetch.
    pub fn with_max_content_length(mut self, max: usize) -> Self {
        self.max_content_length = max;
        self
    }
}

impl Default for WebFetchTool {
    fn default() -> Self {
        Self::new().expect("Failed to create WebFetchTool")
    }
}

#[derive(Debug, Deserialize)]
struct WebFetchArgs {
    url: String,
    extract_text: Option<bool>,
}

#[async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }

    fn description(&self) -> &str {
        "Fetch content from a URL. Can extract text from HTML pages."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "url".to_string(),
                description: "The URL to fetch".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "extract_text".to_string(),
                description: "If true, extract plain text from HTML (default: true)".to_string(),
                required: false,
                parameter_type: "boolean".to_string(),
            },
        ]
    }

    async fn execute(&self, arguments: serde_json::Value) -> NovaResult<serde_json::Value> {
        let args: WebFetchArgs = serde_json::from_value(arguments)?;
        let extract_text = args.extract_text.unwrap_or(true);

        let response = self
            .http_client
            .get(&args.url)
            .await
            .map_err(|e| NovaError::HttpError(format!("Fetch request failed: {}", e)))?;

        let status = response.status_code;
        let content_type = response.header("content-type").map(|s| s.to_string());

        let body = response
            .text()
            .map_err(|e| NovaError::HttpError(format!("Failed to read response: {}", e)))?;

        // Truncate if too long
        let body = if body.len() > self.max_content_length {
            format!(
                "{}... [truncated, {} bytes total]",
                &body[..self.max_content_length],
                body.len()
            )
        } else {
            body
        };

        // Extract text from HTML if requested
        let content = if extract_text && content_type.as_deref().unwrap_or("").contains("text/html")
        {
            extract_text_from_html(&body)
        } else {
            body
        };

        Ok(serde_json::json!({
            "url": args.url,
            "status": status,
            "content_type": content_type,
            "content": content,
            "length": content.len()
        }))
    }
}

/// Tool for posting clarification questions to an external platform.
///
/// This tool posts a formatted comment with checkbox options to the issue
/// and returns a status indicating the agent should pause for user input.
#[allow(dead_code)]
pub struct PostCommentTool<I: crate::integrations::PlatformIntegration + 'static> {
    integration: Arc<I>,
    session: Arc<RwLock<SessionState>>,
}

impl<I: crate::integrations::PlatformIntegration + 'static> PostCommentTool<I> {
    /// Create a new PostCommentTool.
    #[allow(dead_code)]
    pub fn new(integration: Arc<I>, session: Arc<RwLock<SessionState>>) -> Self {
        Self {
            integration,
            session,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct PostCommentArgs {
    issue_id: String,
    question: String,
    options: Option<Vec<PostCommentOption>>,
    multi_select: Option<bool>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, serde::Serialize)]
struct PostCommentOption {
    label: String,
    recommended: Option<bool>,
}

#[async_trait]
impl<I: crate::integrations::PlatformIntegration + 'static> Tool for PostCommentTool<I> {
    fn name(&self) -> &str {
        "post_comment"
    }

    fn description(&self) -> &str {
        "Post a clarification question to an issue with checkbox options. The agent will pause until the user responds."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "issue_id".to_string(),
                description: "The issue ID to post the comment to".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "question".to_string(),
                description: "The question to ask".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "options".to_string(),
                description: "List of options with label and optional recommended flag".to_string(),
                required: false,
                parameter_type: "array".to_string(),
            },
            ToolParameter {
                name: "multi_select".to_string(),
                description: "Whether multiple options can be selected (default: false)"
                    .to_string(),
                required: false,
                parameter_type: "boolean".to_string(),
            },
        ]
    }

    async fn execute(&self, arguments: serde_json::Value) -> NovaResult<serde_json::Value> {
        use crate::integrations::{format_clarification_comment, ClarificationQuestionOption};
        use crate::storage::{ClarificationOption, PendingClarification};
        use chrono::Utc;

        let args: PostCommentArgs = serde_json::from_value(arguments)?;
        let multi_select = args.multi_select.unwrap_or(false);

        // Convert options
        let options: Vec<ClarificationQuestionOption> = args
            .options
            .as_ref()
            .map(|opts| {
                opts.iter()
                    .map(|o| ClarificationQuestionOption {
                        label: o.label.clone(),
                        recommended: o.recommended.unwrap_or(false),
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Format the comment
        let comment_body = format_clarification_comment(&args.question, &options, multi_select);

        // Post the comment
        let posted = self
            .integration
            .post_comment(&args.issue_id, &comment_body)
            .await?;

        // Create pending clarification
        let pending = PendingClarification {
            platform: self.integration.name().to_string(),
            issue_id: args.issue_id.clone(),
            comment_id: posted.id.clone(),
            question: args.question.clone(),
            options: options
                .iter()
                .map(|o| ClarificationOption {
                    label: o.label.clone(),
                    recommended: o.recommended,
                })
                .collect(),
            multi_select,
            requested_at: Utc::now(),
        };

        // Update session state
        {
            let mut session = self.session.write().await;
            session.pause_for_clarification(pending);
        }

        Ok(serde_json::json!({
            "status": "user_input_required",
            "type": "clarification_posted",
            "platform": self.integration.name(),
            "issue_id": args.issue_id,
            "comment_id": posted.id,
            "comment_url": posted.url,
            "question": args.question,
            "options": args.options,
            "multi_select": multi_select
        }))
    }
}

/// Extract plain text from HTML content.
fn extract_text_from_html(html: &str) -> String {
    // Remove script and style elements
    let script_re = regex::Regex::new(r"(?is)<script[^>]*>.*?</script>").expect("Invalid regex");
    let style_re = regex::Regex::new(r"(?is)<style[^>]*>.*?</style>").expect("Invalid regex");
    let tag_re = regex::Regex::new(r"<[^>]+>").expect("Invalid regex");
    let whitespace_re = regex::Regex::new(r"\s+").expect("Invalid regex");

    let text = script_re.replace_all(html, "");
    let text = style_re.replace_all(&text, "");
    let text = tag_re.replace_all(&text, " ");
    let text = whitespace_re.replace_all(&text, " ");
    let text = html_escape::decode_html_entities(&text);

    text.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ask_user_tool() {
        let tool = AskUserTool::new();
        let result = tool
            .execute(serde_json::json!({
                "question": "What is the project scope?",
                "context": "I need to understand the requirements"
            }))
            .await
            .unwrap();

        assert_eq!(result["status"], "pending");
        assert_eq!(result["type"], "user_input_required");
        assert_eq!(result["question"], "What is the project scope?");
    }

    #[tokio::test]
    async fn test_take_note_tool() {
        let session = Arc::new(RwLock::new(SessionState::new("test")));
        let tool = TakeNoteTool::new(session.clone());

        let result = tool
            .execute(serde_json::json!({
                "content": "Important requirement found",
                "category": "requirement"
            }))
            .await
            .unwrap();

        assert!(result["success"].as_bool().unwrap());
        assert_eq!(result["total_notes"], 1);

        let session = session.read().await;
        assert_eq!(session.notes.len(), 1);
        assert_eq!(session.notes[0].content, "Important requirement found");
    }

    #[tokio::test]
    async fn test_record_finding_tool() {
        let session = Arc::new(RwLock::new(SessionState::new("test")));
        let tool = RecordFindingTool::new(session.clone());

        let result = tool
            .execute(serde_json::json!({
                "title": "Security Gap",
                "description": "Authentication is missing",
                "severity": "high",
                "sources": ["code review", "design doc"]
            }))
            .await
            .unwrap();

        assert!(result["success"].as_bool().unwrap());
        assert_eq!(result["total_findings"], 1);

        let session = session.read().await;
        assert_eq!(session.findings.len(), 1);
        assert_eq!(session.findings[0].title, "Security Gap");
        assert_eq!(session.findings[0].severity, FindingSeverity::High);
        assert_eq!(session.findings[0].sources.len(), 2);
    }

    #[tokio::test]
    async fn test_record_finding_invalid_severity() {
        let session = Arc::new(RwLock::new(SessionState::new("test")));
        let tool = RecordFindingTool::new(session);

        let result = tool
            .execute(serde_json::json!({
                "title": "Test",
                "description": "Test",
                "severity": "invalid"
            }))
            .await;

        assert!(result.is_err());
    }

    #[test]
    fn test_extract_text_from_html() {
        let html = r#"
            <html>
            <head>
                <script>var x = 1;</script>
                <style>body { color: red; }</style>
            </head>
            <body>
                <h1>Hello World</h1>
                <p>This is a &amp; test.</p>
            </body>
            </html>
        "#;

        let text = extract_text_from_html(html);
        assert!(text.contains("Hello World"));
        assert!(text.contains("This is a & test"));
        assert!(!text.contains("var x = 1"));
        assert!(!text.contains("color: red"));
    }

    #[test]
    fn test_parse_duckduckgo_results() {
        let html = r#"
            <a class="result__a" href="https://example.com">Example Title</a>
            <a class="result__snippet">This is the snippet text</a>
        "#;

        let results = parse_duckduckgo_results(html, 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["url"], "https://example.com");
        assert_eq!(results[0]["title"], "Example Title");
    }
}
