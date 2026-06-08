use crate::error::{NovaError, NovaResult};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Security policy for the coding agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub allowed_paths: Vec<PathBuf>,
    pub blocked_paths: Vec<PathBuf>,
    pub approval_required: HashSet<String>,
    pub blocked_commands: Vec<String>,
    pub shell_timeout: Duration,
    pub file_timeout: Duration,
    pub allow_absolute_paths: bool,
    pub working_directory: PathBuf,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            allowed_paths: vec![],
            blocked_paths: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/var"),
                PathBuf::from("/usr"),
                PathBuf::from("/bin"),
                PathBuf::from("/sbin"),
                PathBuf::from("/root"),
                PathBuf::from("/System"),
            ],
            approval_required: HashSet::from_iter([
                "write_file".to_string(),
                "bash".to_string(),
                "git_commit".to_string(),
            ]),
            blocked_commands: vec![
                "rm -rf /".to_string(),
                "rm -rf /*".to_string(),
                ":(){ :|:& };:".to_string(),
                "dd if=/dev/zero".to_string(),
                "mkfs".to_string(),
                "chmod 777 /".to_string(),
            ],
            shell_timeout: Duration::from_secs(120),
            file_timeout: Duration::from_secs(30),
            allow_absolute_paths: false,
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }
}

impl SecurityPolicy {
    /// Create a new security policy builder
    pub fn builder() -> SecurityPolicyBuilder {
        SecurityPolicyBuilder::default()
    }

    /// Create a permissive policy (for development/testing)
    pub fn permissive() -> Self {
        Self {
            allowed_paths: vec![],
            blocked_paths: vec![],
            approval_required: HashSet::new(),
            blocked_commands: vec![],
            shell_timeout: Duration::from_secs(600),
            file_timeout: Duration::from_secs(60),
            allow_absolute_paths: true,
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }

    /// Check if a path is allowed for file operations
    pub fn check_path(&self, path: &Path) -> NovaResult<()> {
        let normalized = self.normalize_path(path)?;

        for blocked in &self.blocked_paths {
            if normalized.starts_with(blocked) {
                return Err(NovaError::PathNotAllowed(format!(
                    "Path '{}' is in blocked directory '{}'",
                    normalized.display(),
                    blocked.display()
                )));
            }
        }

        if self.allowed_paths.is_empty() {
            return Ok(());
        }

        for allowed in &self.allowed_paths {
            if normalized.starts_with(allowed) {
                return Ok(());
            }
        }

        Err(NovaError::PathNotAllowed(format!(
            "Path '{}' is not in any allowed directory",
            normalized.display()
        )))
    }

    /// Check if a shell command is allowed
    pub fn check_command(&self, command: &str) -> NovaResult<()> {
        let cmd_lower = command.to_lowercase();

        for blocked in &self.blocked_commands {
            if cmd_lower.contains(&blocked.to_lowercase()) {
                return Err(NovaError::CommandNotAllowed(format!(
                    "Command contains blocked pattern: {}",
                    blocked
                )));
            }
        }

        Ok(())
    }

    /// Check if a tool requires approval
    pub fn requires_approval(&self, tool_name: &str) -> bool {
        self.approval_required.contains(tool_name)
    }

    fn normalize_path(&self, path: &Path) -> NovaResult<PathBuf> {
        let normalized = if path.is_absolute() {
            if !self.allow_absolute_paths && !path.starts_with(&self.working_directory) {
                return Err(NovaError::PathNotAllowed(format!(
                    "Absolute path '{}' outside working directory not allowed",
                    path.display()
                )));
            }
            path.to_path_buf()
        } else {
            self.working_directory.join(path)
        };

        Ok(normalized.canonicalize().unwrap_or(normalized))
    }
}

/// Builder for SecurityPolicy
#[derive(Default)]
pub struct SecurityPolicyBuilder {
    policy: SecurityPolicy,
}

impl SecurityPolicyBuilder {
    pub fn with_allowed_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.policy.allowed_paths = paths;
        self
    }

    pub fn with_blocked_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.policy.blocked_paths = paths;
        self
    }

    pub fn with_approval_required(mut self, tools: HashSet<String>) -> Self {
        self.policy.approval_required = tools;
        self
    }

    pub fn with_blocked_commands(mut self, commands: Vec<String>) -> Self {
        self.policy.blocked_commands = commands;
        self
    }

    pub fn with_shell_timeout(mut self, timeout: Duration) -> Self {
        self.policy.shell_timeout = timeout;
        self
    }

    pub fn with_working_directory(mut self, dir: PathBuf) -> Self {
        self.policy.working_directory = dir;
        self
    }

    pub fn allow_absolute_paths(mut self, allow: bool) -> Self {
        self.policy.allow_absolute_paths = allow;
        self
    }

    pub fn build(self) -> SecurityPolicy {
        self.policy
    }
}

/// Request for user approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub description: String,
    pub risks: Vec<String>,
}

/// User's response to an approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalResponse {
    Approved,
    Denied { reason: Option<String> },
    ApprovedWithModifications { modifications: serde_json::Value },
    AlwaysApprove,
    AlwaysDeny,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = SecurityPolicy::default();
        assert!(!policy.blocked_paths.is_empty());
        assert!(policy.approval_required.contains("write_file"));
        assert!(policy.approval_required.contains("bash"));
    }

    #[test]
    fn test_path_blocked() {
        let policy = SecurityPolicy::default();
        let result = policy.check_path(Path::new("/etc/passwd"));
        assert!(result.is_err());
    }

    #[test]
    fn test_command_blocked() {
        let policy = SecurityPolicy::default();
        let result = policy.check_command("rm -rf /");
        assert!(result.is_err());
    }

    #[test]
    fn test_permissive_policy() {
        let policy = SecurityPolicy::permissive();
        assert!(policy.blocked_paths.is_empty());
        assert!(policy.approval_required.is_empty());
    }

    #[test]
    fn test_requires_approval() {
        let policy = SecurityPolicy::default();
        assert!(policy.requires_approval("write_file"));
        assert!(policy.requires_approval("bash"));
        assert!(!policy.requires_approval("read_file"));
    }
}
