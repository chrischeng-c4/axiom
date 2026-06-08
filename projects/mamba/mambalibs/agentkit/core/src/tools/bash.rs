use crate::error::{NovaError, NovaResult};
use crate::tools::tool::{Tool, ToolParameter};
use async_trait::async_trait;
use serde::Deserialize;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, warn};

/// Tool for executing shell commands
pub struct BashTool {
    default_timeout: Duration,
    max_output_size: usize,
    shell: String,
    working_dir: Option<String>,
}

impl BashTool {
    pub fn new() -> Self {
        Self {
            default_timeout: Duration::from_secs(120),
            max_output_size: 100_000,
            shell: "/bin/bash".to_string(),
            working_dir: None,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    pub fn with_max_output_size(mut self, size: usize) -> Self {
        self.max_output_size = size;
        self
    }

    pub fn with_working_dir(mut self, dir: impl Into<String>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    async fn execute_impl(&self, args: BashArgs) -> NovaResult<serde_json::Value> {
        let timeout_ms = args
            .timeout
            .unwrap_or(self.default_timeout.as_millis() as u64);
        let timeout_duration = Duration::from_millis(timeout_ms);

        debug!("Executing command: {}", args.command);

        let mut cmd = Command::new(&self.shell);
        cmd.arg("-c").arg(&args.command);

        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let child = cmd
            .spawn()
            .map_err(|e| NovaError::CommandFailed(format!("Failed to spawn command: {}", e)))?;

        let output = timeout(timeout_duration, child.wait_with_output())
            .await
            .map_err(|_| NovaError::CommandTimeout(timeout_ms / 1000))?
            .map_err(|e| NovaError::CommandFailed(format!("Command execution failed: {}", e)))?;

        let exit_code = output.status.code().unwrap_or(-1);
        let mut stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let mut stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let stdout_truncated = stdout.len() > self.max_output_size;
        let stderr_truncated = stderr.len() > self.max_output_size;

        if stdout_truncated {
            stdout.truncate(self.max_output_size);
            stdout.push_str("\n... [output truncated]");
        }

        if stderr_truncated {
            stderr.truncate(self.max_output_size);
            stderr.push_str("\n... [output truncated]");
        }

        Ok(serde_json::json!({
            "exit_code": exit_code,
            "stdout": stdout,
            "stderr": stderr,
            "success": exit_code == 0,
            "stdout_truncated": stdout_truncated,
            "stderr_truncated": stderr_truncated
        }))
    }
}

impl Default for BashTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct BashArgs {
    command: String,
    timeout: Option<u64>,
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Execute a bash command and return its output."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "command".to_string(),
                description: "The bash command to execute".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "timeout".to_string(),
                description: "Timeout in milliseconds".to_string(),
                required: false,
                parameter_type: "integer".to_string(),
            },
        ]
    }

    async fn execute(&self, arguments: serde_json::Value) -> NovaResult<serde_json::Value> {
        let args: BashArgs = serde_json::from_value(arguments)?;
        self.execute_impl(args).await
    }
}

/// Tool for executing bash commands with streaming output
pub struct StreamingBashTool {
    inner: BashTool,
}

impl StreamingBashTool {
    pub fn new() -> Self {
        Self {
            inner: BashTool::new(),
        }
    }

    pub async fn execute_streaming<F>(
        &self,
        command: &str,
        timeout_ms: Option<u64>,
        mut on_output: F,
    ) -> NovaResult<i32>
    where
        F: FnMut(StreamOutput) + Send,
    {
        let timeout_duration = Duration::from_millis(
            timeout_ms.unwrap_or(self.inner.default_timeout.as_millis() as u64),
        );

        let mut cmd = Command::new(&self.inner.shell);
        cmd.arg("-c").arg(command);

        if let Some(ref dir) = self.inner.working_dir {
            cmd.current_dir(dir);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| NovaError::CommandFailed(format!("Failed to spawn command: {}", e)))?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        let read_task = async {
            loop {
                tokio::select! {
                    result = stdout_reader.next_line() => {
                        match result {
                            Ok(Some(line)) => on_output(StreamOutput::Stdout(line)),
                            Ok(None) => break,
                            Err(e) => {
                                warn!("Error reading stdout: {}", e);
                                break;
                            }
                        }
                    }
                    result = stderr_reader.next_line() => {
                        match result {
                            Ok(Some(line)) => on_output(StreamOutput::Stderr(line)),
                            Ok(None) => {}
                            Err(e) => {
                                warn!("Error reading stderr: {}", e);
                            }
                        }
                    }
                }
            }
        };

        let wait_task = child.wait();

        let (_, status) = timeout(timeout_duration, async {
            tokio::join!(read_task, wait_task)
        })
        .await
        .map_err(|_| NovaError::CommandTimeout(timeout_duration.as_secs()))?;

        let status = status
            .map_err(|e| NovaError::CommandFailed(format!("Failed to wait for command: {}", e)))?;

        Ok(status.code().unwrap_or(-1))
    }
}

impl Default for StreamingBashTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Output from streaming bash execution
#[derive(Debug, Clone)]
pub enum StreamOutput {
    Stdout(String),
    Stderr(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bash_echo() {
        let tool = BashTool::new();
        let result = tool
            .execute(serde_json::json!({
                "command": "echo 'Hello, World!'"
            }))
            .await
            .unwrap();

        assert!(result["success"].as_bool().unwrap());
        assert!(result["stdout"].as_str().unwrap().contains("Hello, World!"));
    }

    #[tokio::test]
    async fn test_bash_exit_code() {
        let tool = BashTool::new();
        let result = tool
            .execute(serde_json::json!({
                "command": "exit 42"
            }))
            .await
            .unwrap();

        assert_eq!(result["exit_code"], 42);
        assert!(!result["success"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_streaming_bash() {
        let tool = StreamingBashTool::new();
        let mut lines = Vec::new();

        let exit_code = tool
            .execute_streaming(
                "echo 'line1' && echo 'line2' && echo 'line3'",
                Some(5000),
                |output| {
                    if let StreamOutput::Stdout(line) = output {
                        lines.push(line);
                    }
                },
            )
            .await
            .unwrap();

        assert_eq!(exit_code, 0);
        assert_eq!(lines.len(), 3);
    }
}
