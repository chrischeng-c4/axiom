use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GateRun {
    pub id: String,
    pub task_id: String,
    pub command: String,
    pub exit_code: i32,
    #[serde(default)]
    pub stdout_tail: String,
    #[serde(default)]
    pub stderr_tail: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_sha256: Option<String>,
    pub duration_ms: u64,
    pub head_commit: String,
    pub passed: bool,
    pub run_at: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
}

impl GateRun {
    pub fn stdout_display(&self) -> &str {
        if !self.stdout_tail.is_empty() {
            &self.stdout_tail
        } else if let Some(ref s) = self.stdout {
            s.as_str()
        } else {
            ""
        }
    }

    pub fn stderr_display(&self) -> &str {
        if !self.stderr_tail.is_empty() {
            &self.stderr_tail
        } else if let Some(ref s) = self.stderr {
            s.as_str()
        } else {
            ""
        }
    }
}

