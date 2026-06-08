// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/platform_sync/github_preamble_source.md#source
// CODEGEN-BEGIN
//! GitHub provider for platform sync
//!
//! Uses GitHub API when token is available, falls back to gh CLI otherwise.

use super::{PlatformConfig, SpecSyncResult, SyncPayload, SyncResult, SyncStatus};
use crate::Result;
use std::io::Write as _;
use std::process::Command;
use tempfile::NamedTempFile;
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/platform_sync/github.md#schema
// CODEGEN-BEGIN
/// GitHub platform-sync provider.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/platform_sync/github.md#schema
pub struct GitHubProvider {
    /// Platform configuration.
    config: PlatformConfig,
    /// Optional auth token.
    token: Option<String>,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/platform_sync/github_runtime_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/platform_sync/github_runtime_source.md#source
impl GitHubProvider {
    pub fn new(config: PlatformConfig) -> Self {
        Self {
            config,
            token: None,
        }
    }

    /// Initialize with token resolution
    pub fn with_token(mut self, project_root: &std::path::Path) -> Result<Self> {
        self.token = self.config.get_token(project_root)?;
        Ok(self)
    }

    /// Check if sync is possible (has token or gh CLI authenticated)
    pub fn can_sync(&self) -> bool {
        if self.token.is_some() {
            return true;
        }
        // Check gh auth status with hostname for GitHub Enterprise support
        let hostname = self.get_api_hostname();
        Command::new("gh")
            .args(["auth", "status", "--hostname", &hostname])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Get API hostname from repo config (supports GitHub Enterprise)
    fn get_api_hostname(&self) -> String {
        // Default to github.com, but could be extended to parse from config
        "github.com".to_string()
    }

    /// Sync payload to GitHub
    pub async fn sync(&self, payload: &SyncPayload) -> Result<SyncResult> {
        if !self.can_sync() {
            anyhow::bail!(
                "GitHub not authenticated. Set GITHUB_TOKEN in .env or run 'gh auth login'"
            );
        }

        let use_api = self.token.is_some();
        let token = self.token.clone().unwrap_or_default();

        // Step 1: Sync spec issues first
        let mut spec_results = Vec::new();
        let mut spec_links: Vec<(String, u64, String)> = Vec::new();

        for spec in &payload.specs {
            let result = if use_api {
                self.upsert_issue_api(
                    &token,
                    &spec.title,
                    &spec.body,
                    &spec.labels,
                    spec.existing_issue,
                )
                .await
            } else {
                self.upsert_issue_cli(&spec.title, &spec.body, &spec.labels, spec.existing_issue)
            };

            match result {
                Ok((status, num, url)) => {
                    spec_links.push((spec.spec_id.clone(), num, url.clone()));
                    spec_results.push(SpecSyncResult {
                        spec_id: spec.spec_id.clone(),
                        status,
                        issue_url: Some(url),
                        issue_number: Some(num),
                    });
                }
                Err(e) => {
                    spec_results.push(SpecSyncResult {
                        spec_id: spec.spec_id.clone(),
                        status: SyncStatus::Error,
                        issue_url: None,
                        issue_number: None,
                    });
                    eprintln!("Warning: Failed to sync spec {}: {}", spec.spec_id, e);
                }
            }
        }

        // Step 2: Update parent body with spec links
        let parent_body = if !spec_links.is_empty() {
            super::payload::update_body_with_spec_links(&payload.body, &spec_links)
        } else {
            payload.body.clone()
        };

        // Step 3: Sync parent issue
        let parent_result = if use_api {
            self.upsert_issue_api(
                &token,
                &payload.title,
                &parent_body,
                &payload.labels,
                payload.existing_issue,
            )
            .await
        } else {
            self.upsert_issue_cli(
                &payload.title,
                &parent_body,
                &payload.labels,
                payload.existing_issue,
            )
        };

        match parent_result {
            Ok((status, num, url)) => {
                let error_count = spec_results
                    .iter()
                    .filter(|r| r.status == SyncStatus::Error)
                    .count();
                let final_status = if error_count > 0 {
                    SyncStatus::Partial
                } else {
                    status
                };

                let message = if payload.specs.is_empty() {
                    format!("{:?} issue #{}", status, num)
                } else {
                    let created = spec_results
                        .iter()
                        .filter(|r| r.status == SyncStatus::Created)
                        .count();
                    let updated = spec_results
                        .iter()
                        .filter(|r| r.status == SyncStatus::Updated)
                        .count();
                    if error_count > 0 {
                        format!(
                            "{:?} #{} + {} specs ({} failed)",
                            status,
                            num,
                            payload.specs.len(),
                            error_count
                        )
                    } else {
                        format!(
                            "{:?} #{} + {} specs ({} created, {} updated)",
                            status,
                            num,
                            payload.specs.len(),
                            created,
                            updated
                        )
                    }
                };

                Ok(SyncResult {
                    status: final_status,
                    issue_url: Some(url),
                    issue_number: Some(num),
                    message,
                    spec_results,
                })
            }
            Err(e) => Err(e),
        }
    }

    /// Create or update issue via API
    async fn upsert_issue_api(
        &self,
        token: &str,
        title: &str,
        body: &str,
        labels: &[String],
        existing: Option<u64>,
    ) -> Result<(SyncStatus, u64, String)> {
        let client = reqwest::Client::new();

        if let Some(issue_num) = existing {
            // Update existing
            let url = format!(
                "https://api.github.com/repos/{}/issues/{}",
                self.config.repo, issue_num
            );
            let response = client
                .patch(&url)
                .header("Authorization", format!("Bearer {}", token))
                .header("User-Agent", "sdd")
                .header("Accept", "application/vnd.github+json")
                .json(&serde_json::json!({ "title": title, "body": body, "labels": labels }))
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                anyhow::bail!("GitHub API error ({}): {}", status, body);
            }

            let json: serde_json::Value = response.json().await?;
            let issue_url = json["html_url"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("No html_url"))?
                .to_string();
            return Ok((SyncStatus::Updated, issue_num, issue_url));
        }

        // Create new
        let url = format!("https://api.github.com/repos/{}/issues", self.config.repo);
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "sdd")
            .header("Accept", "application/vnd.github+json")
            .json(&serde_json::json!({ "title": title, "body": body, "labels": labels }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error ({}): {}", status, body);
        }

        let json: serde_json::Value = response.json().await?;
        let issue_number = json["number"]
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("No issue number"))?;
        let issue_url = json["html_url"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No html_url"))?
            .to_string();
        Ok((SyncStatus::Created, issue_number, issue_url))
    }

    /// Create or update issue via CLI
    fn upsert_issue_cli(
        &self,
        title: &str,
        body: &str,
        labels: &[String],
        existing: Option<u64>,
    ) -> Result<(SyncStatus, u64, String)> {
        let mut body_file = NamedTempFile::new()?;
        body_file.write_all(body.as_bytes())?;
        let body_path = body_file.path().to_string_lossy().to_string();

        if let Some(issue_num) = existing {
            // Update existing
            self.run_gh(&[
                "issue",
                "edit",
                &issue_num.to_string(),
                "--repo",
                &self.config.repo,
                "--title",
                title,
                "--body-file",
                &body_path,
            ])?;

            // Set labels via API
            if !labels.is_empty() {
                let labels_json = serde_json::json!(labels);
                let _ = self.run_gh(&[
                    "api",
                    &format!("repos/{}/issues/{}", self.config.repo, issue_num),
                    "-X",
                    "PATCH",
                    "-f",
                    &format!("labels={}", labels_json),
                ]);
            }

            let url = format!(
                "https://github.com/{}/issues/{}",
                self.config.repo, issue_num
            );
            return Ok((SyncStatus::Updated, issue_num, url));
        }

        // Create new
        let mut args = vec![
            "issue",
            "create",
            "--repo",
            &self.config.repo,
            "--title",
            title,
            "--body-file",
            &body_path,
        ];
        for label in labels {
            args.push("--label");
            args.push(label);
        }

        let output = self.run_gh(&args)?;
        Self::parse_issue_url(&output)
            .map(|(url, num)| (SyncStatus::Created, num, url))
            .ok_or_else(|| anyhow::anyhow!("Failed to parse issue URL: {}", output))
    }

    fn run_gh(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("gh").args(args).output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Sanitize stderr to avoid leaking tokens or sensitive auth info
            let sanitized = Self::sanitize_error_output(&stderr);
            anyhow::bail!("gh command failed: {}", sanitized);
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Sanitize error output to avoid leaking sensitive information
    fn sanitize_error_output(output: &str) -> String {
        // Remove potential token patterns (gh_*, gho_*, ghp_*, etc.)
        let token_pattern = regex::Regex::new(r"gh[opsu]_[A-Za-z0-9_]+").unwrap();
        let sanitized = token_pattern.replace_all(output, "[REDACTED]");

        // Remove Bearer token patterns
        let bearer_pattern = regex::Regex::new(r"Bearer\s+[A-Za-z0-9_\-\.]+").unwrap();
        let sanitized = bearer_pattern.replace_all(&sanitized, "Bearer [REDACTED]");

        sanitized.to_string()
    }

    fn parse_issue_url(output: &str) -> Option<(String, u64)> {
        let url = output.trim();
        url.rsplit('/')
            .next()?
            .parse()
            .ok()
            .map(|num| (url.to_string(), num))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_issue_url() {
        let (url, num) =
            GitHubProvider::parse_issue_url("https://github.com/owner/repo/issues/123").unwrap();
        assert_eq!(url, "https://github.com/owner/repo/issues/123");
        assert_eq!(num, 123);
    }

    #[test]
    fn test_sanitize_error_output() {
        // Should redact GitHub tokens
        let input = "Error: token ghp_abc123XYZ456 is invalid";
        let output = GitHubProvider::sanitize_error_output(input);
        assert_eq!(output, "Error: token [REDACTED] is invalid");

        // Should redact Bearer tokens
        let input = "Authorization: Bearer abc.def.ghi";
        let output = GitHubProvider::sanitize_error_output(input);
        assert_eq!(output, "Authorization: Bearer [REDACTED]");

        // Should handle multiple tokens
        let input = "tokens: ghp_first123 and gho_second456";
        let output = GitHubProvider::sanitize_error_output(input);
        assert_eq!(output, "tokens: [REDACTED] and [REDACTED]");

        // Should preserve non-sensitive content
        let input = "Error: repository not found";
        let output = GitHubProvider::sanitize_error_output(input);
        assert_eq!(output, "Error: repository not found");
    }
}
// CODEGEN-END
