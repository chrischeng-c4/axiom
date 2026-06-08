// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/platform_sync/mod_facade_source.md#source
// CODEGEN-BEGIN
//! Platform Sync Service
//!
//! Syncs SDD change artifacts to GitHub/GitLab issues.
//! Issue numbers are stored in frontmatter and auto-updated after sync.
//!
//! Configuration: `.aw/config.toml`
//! ```toml
//! [platform]
//! type = "github"
//! repo = "owner/repo"
//!
//! [platform.auth]
//! envfile = ".env"
//! envfield = "GITHUB_TOKEN"
//! ```

mod config;
mod github;
pub mod payload;
mod types;

pub use config::PlatformConfig;
pub use github::GitHubProvider;
pub use payload::build_payload;
pub use types::*;

use crate::Result;
use std::path::PathBuf;
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/platform_sync/mod.md#schema
// CODEGEN-BEGIN
/// Platform sync service handle. Holds the project root path used
/// to resolve `.aw/config.toml` and change directories. All
/// behaviour is on the hand-written impl block.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/platform_sync/mod.md#schema
pub struct PlatformSyncService {
    /// Absolute path to the project root.
    project_root: PathBuf,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/platform_sync/mod_runtime_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/platform_sync/mod_runtime_source.md#source
impl PlatformSyncService {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    pub fn load_config(&self) -> Result<PlatformConfig> {
        PlatformConfig::load(&self.project_root)
    }

    /// Sync change to platform
    /// Returns SyncResult and writes issue numbers back to frontmatter
    pub async fn sync(&self, change_id: &str) -> Result<SyncResult> {
        let config = self.load_config()?;
        let change_dir = self.project_root.join(".aw/changes").join(change_id);

        if !change_dir.exists() {
            anyhow::bail!("Change '{}' not found", change_id);
        }

        // Build payload (reads existing issue numbers from frontmatter)
        let payload = payload::build_payload(&self.project_root, change_id)?;

        // Sync to platform
        let result = match config.platform_type.as_str() {
            "github" => {
                let provider = GitHubProvider::new(config).with_token(&self.project_root)?;
                provider.sync(&payload).await?
            }
            "gitlab" => anyhow::bail!("GitLab not yet implemented"),
            other => anyhow::bail!("Unsupported platform: {}", other),
        };

        // Write issue numbers back to frontmatter
        if let Some(issue_num) = result.issue_number {
            payload::write_issue_to_frontmatter(&change_dir.join("proposal.md"), issue_num)?;
        }

        for spec_result in &result.spec_results {
            if let Some(issue_num) = spec_result.issue_number {
                let spec_path = change_dir
                    .join("specs")
                    .join(format!("{}.md", spec_result.spec_id));
                if spec_path.exists() {
                    payload::write_issue_to_frontmatter(&spec_path, issue_num)?;
                }
            }
        }

        Ok(result)
    }
}
// CODEGEN-END
