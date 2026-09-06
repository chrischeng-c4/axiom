use std::path::Path;
use std::time::Instant;
use anyhow::{bail, Context, Result};
use chrono::Utc;
use tokio::process::Command;

use crate::models::{GateRun, TaskStatus};
use crate::store::PmStore;

/// Executes a verification command on the local machine, capturing exit status,
/// stdout, stderr, execution duration, and the current git HEAD commit hash.
pub async fn execute_gate(task_id: &str, command: &str, work_dir: &Path) -> Result<GateRun> {
    let start = Instant::now();

    // Get current git HEAD commit hash (if inside git repo)
    let head_commit = match Command::new("git")
        .args(["-c", "core.fsmonitor=false", "rev-parse", "HEAD"])
        .current_dir(work_dir)
        .output()
        .await
    {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        _ => "none".to_string(),
    };

    // Run the gate command in shell
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(work_dir)
        .output()
        .await
        .with_context(|| format!("Failed to execute gate command '{}'", command))?;

    let duration_ms = start.elapsed().as_millis() as u64;
    let exit_code = output.status.code().unwrap_or(-1);
    let passed = exit_code == 0;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let id = format!("gate_{}", Utc::now().timestamp_micros());
    let run_at = Utc::now().to_rfc3339();

    Ok(GateRun {
        id,
        task_id: task_id.to_string(),
        command: command.to_string(),
        exit_code,
        stdout,
        stderr,
        duration_ms,
        head_commit,
        passed,
        run_at,
    })
}

/// Verifies prerequisites (passing GateRun, zero unresolved blocking review comments)
/// and performs a local git merge into `target_branch`, marking the task as Done.
pub async fn verify_and_merge_task(
    store: &PmStore,
    task_id: &str,
    target_branch: &str,
    strategy: &str,
    work_dir: &Path,
) -> Result<String> {
    // 1. Validate prerequisites in a read lock
    let (task, latest_gate_id, latest_gate_duration_ms) = store
        .read(|state| {
            let task = match state.get_task(task_id) {
                Some(t) => t.clone(),
                None => bail!("Task '{}' not found", task_id),
            };

            if state.has_unresolved_blocking_reviews(task_id) {
                let blockers = state.list_review_comments(task_id, true);
                let count = blockers
                    .iter()
                    .filter(|b| b.severity == crate::models::ReviewSeverity::Blocking)
                    .count();
                bail!(
                    "Cannot merge task '{}': has {} unresolved blocking review comment(s). Please resolve them first.",
                    task_id,
                    count
                );
            }

            let latest_gate = match state.get_latest_gate_run(task_id) {
                Some(g) => g,
                None => bail!(
                    "Cannot merge task '{}': no gate run recorded. Run 'pm_verify_gate' first to verify changes.",
                    task_id
                ),
            };

            if !latest_gate.passed {
                bail!(
                    "Cannot merge task '{}': latest gate run '{}' failed with exit code {}. Verify and fix tests before merging.",
                    task_id,
                    latest_gate.id,
                    latest_gate.exit_code
                );
            }

            Ok((task, latest_gate.id.clone(), latest_gate.duration_ms))
        })
        .await?;

    // 2. Perform local git merge asynchronously outside any lock
    let merge_commit = perform_git_merge(&task.id, target_branch, strategy, work_dir).await?;

    // 3. Mutate task status to Done and record evidence in a write lock
    store
        .write(|state| {
            if let Some(t) = state.tasks.get_mut(task_id) {
                t.status = TaskStatus::Done;
                t.updated_at = Utc::now().to_rfc3339();
                let summary = format!(
                    "Merged into '{}' locally (commit: {}). Gate '{}' passed in {}ms.",
                    target_branch, merge_commit, latest_gate_id, latest_gate_duration_ms
                );
                t.result_summary = Some(summary);
            }
            Ok(())
        })
        .await?;

    Ok(merge_commit)
}

async fn perform_git_merge(
    task_id: &str,
    target_branch: &str,
    strategy: &str,
    work_dir: &Path,
) -> Result<String> {
    // Check if git is available in work_dir
    let git_check = Command::new("git")
        .args(["-c", "core.fsmonitor=false", "rev-parse", "--is-inside-work-tree"])
        .current_dir(work_dir)
        .output()
        .await;

    let is_git = match git_check {
        Ok(out) => out.status.success() && String::from_utf8_lossy(&out.stdout).trim() == "true",
        _ => false,
    };

    if !is_git {
        // In-memory or simulated directory
        return Ok(format!("mock_commit_{}", Utc::now().timestamp_millis()));
    }

    let current_branch_out = Command::new("git")
        .args(["-c", "core.fsmonitor=false", "branch", "--show-current"])
        .current_dir(work_dir)
        .output()
        .await
        .context("Failed to get current branch")?;

    let current_branch = String::from_utf8_lossy(&current_branch_out.stdout).trim().to_string();

    if current_branch == target_branch {
        // Already on target branch; return current HEAD
        let head_out = Command::new("git")
            .args(["-c", "core.fsmonitor=false", "rev-parse", "HEAD"])
            .current_dir(work_dir)
            .output()
            .await?;
        return Ok(String::from_utf8_lossy(&head_out.stdout).trim().to_string());
    }

    // Switch to target branch and merge
    let checkout = Command::new("git")
        .args(["-c", "core.fsmonitor=false", "checkout", target_branch])
        .current_dir(work_dir)
        .output()
        .await?;

    if !checkout.status.success() {
        bail!(
            "Failed to checkout target branch '{}': {}",
            target_branch,
            String::from_utf8_lossy(&checkout.stderr)
        );
    }

    let merge_arg = match strategy {
        "squash" => vec!["merge", "--squash", &current_branch],
        _ => vec!["merge", "--no-ff", &current_branch],
    };

    let merge = Command::new("git")
        .args(std::iter::once("-c").chain(std::iter::once("core.fsmonitor=false")).chain(merge_arg))
        .current_dir(work_dir)
        .output()
        .await?;

    if !merge.status.success() {
        // Abort merge on error
        let _ = Command::new("git")
            .args(["-c", "core.fsmonitor=false", "merge", "--abort"])
            .current_dir(work_dir)
            .output()
            .await;
        // Switch back
        let _ = Command::new("git")
            .args(["-c", "core.fsmonitor=false", "checkout", &current_branch])
            .current_dir(work_dir)
            .output()
            .await;
        bail!(
            "Failed to merge branch '{}' into '{}': {}",
            current_branch,
            target_branch,
            String::from_utf8_lossy(&merge.stderr)
        );
    }

    // If squash merge, commit the squashed changes
    if strategy == "squash" {
        let commit = Command::new("git")
            .args([
                "-c",
                "core.fsmonitor=false",
                "commit",
                "-m",
                &format!("feat: auto-merged task '{}' via pm local gate", task_id),
            ])
            .current_dir(work_dir)
            .output()
            .await?;

        if !commit.status.success() {
            bail!(
                "Failed to commit squashed merge: {}",
                String::from_utf8_lossy(&commit.stderr)
            );
        }
    }

    // Get final HEAD commit
    let final_head = Command::new("git")
        .args(["-c", "core.fsmonitor=false", "rev-parse", "HEAD"])
        .current_dir(work_dir)
        .output()
        .await?;

    let commit_hash = String::from_utf8_lossy(&final_head.stdout).trim().to_string();

    // Switch back to working branch
    let _ = Command::new("git")
        .args(["-c", "core.fsmonitor=false", "checkout", &current_branch])
        .current_dir(work_dir)
        .output()
        .await;

    Ok(commit_hash)
}
