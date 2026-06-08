// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/backends/mod.md#source
// CODEGEN-BEGIN
//! Issue storage backend implementations.

pub mod github;
pub mod gitlab;
pub mod local;

pub use github::GitHubBackend;
pub use gitlab::GitLabBackend;
pub use local::LocalBackend;
// CODEGEN-END
