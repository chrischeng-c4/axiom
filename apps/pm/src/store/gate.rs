use std::path::Path;
use std::time::Instant;
use anyhow::{bail, Context, Result};
use chrono::Utc;
use sha2::{Digest, Sha256};
use tokio::process::Command;

use crate::models::{GateRun, TaskStatus};
use crate::store::PmStore;

fn take_tail(s: &str, n: usize) -> String {
    let lines: Vec<&str> = s.lines().collect();
    if lines.len() <= n {
        s.to_string()
    } else {
        lines[lines.len() - n..].join("\n")
    }
}

/// Executes a verification command on the local machine, capturing exit status,
/// stdout, stderr, execution duration, and the current git HEAD commit hash.
/// Full logs are offloaded to `.pm/gates/<gate_id>.log` to keep state.json compact.
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

    // Offload full logs to .pm/gates/<id>.log
    let log_dir = work_dir.join(".pm").join("gates");
    let _ = tokio::fs::create_dir_all(&log_dir).await;
    let log_filename = format!("{}.log", id);
    let log_path_buf = log_dir.join(&log_filename);
    let relative_log_path = format!(".pm/gates/{}", log_filename);

    let full_log = format!(
        "=== GATE RUN: {} ===\nTask: {}\nCommand: {}\nExit code: {}\nDuration: {}ms\nHEAD commit: {}\nRun at: {}\n\n=== STDOUT ===\n{}\n\n=== STDERR ===\n{}\n",
        id, task_id, command, exit_code, duration_ms, head_commit, run_at, stdout, stderr
    );
    let _ = tokio::fs::write(&log_path_buf, full_log.as_bytes()).await;

    // Cryptographic SHA-256 digest over stdout + stderr
    let mut hasher = Sha256::new();
    hasher.update(stdout.as_bytes());
    hasher.update(b"\n---STDERR---\n");
    hasher.update(stderr.as_bytes());
    let output_sha256 = format!("{:x}", hasher.finalize());

    let stdout_tail = take_tail(&stdout, 40);
    let stderr_tail = take_tail(&stderr, 40);

    Ok(GateRun {
        id,
        task_id: task_id.to_string(),
        command: command.to_string(),
        exit_code,
        stdout_tail,
        stderr_tail,
        log_path: Some(relative_log_path),
        output_sha256: Some(output_sha256),
        duration_ms,
        head_commit,
        passed,
        run_at,
        stdout: None,
        stderr: None,
    })
}

/// Verifies prerequisites (passing GateRun against current HEAD commit,
/// zero unresolved blocking review comments) and performs an isolated git merge
/// into `target_branch`, marking the task as Done and unblocking downstream tasks.
pub async fn verify_and_merge_task(
    store: &PmStore,
    task_id: &str,
    target_branch: &str,
    strategy: &str,
    work_dir: &Path,
) -> Result<String> {
    // 1. Verify that work_dir is a git worktree and check current HEAD commit
    let head_check = Command::new("git")
        .args(["-c", "core.fsmonitor=false", "rev-parse", "HEAD"])
        .current_dir(work_dir)
        .output()
        .await;

    let current_head = match head_check {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        _ => bail!(
            "Cannot merge task '{}': directory '{}' is not a git repository or has no HEAD commit",
            task_id,
            work_dir.display()
        ),
    };

    // 2. Validate prerequisites in a read lock
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

            if latest_gate.head_commit == "none" || latest_gate.head_commit != current_head {
                bail!(
                    "Cannot merge task '{}': latest gate run '{}' was executed against commit '{}', but current HEAD is '{}'. Please re-run 'pm_verify_gate' against current HEAD before merging.",
                    task_id,
                    latest_gate.id,
                    latest_gate.head_commit,
                    current_head
                );
            }

            Ok((task, latest_gate.id.clone(), latest_gate.duration_ms))
        })
        .await?;

    // 3. Perform local git merge in an isolated temporary worktree outside locks
    let merge_commit = perform_git_merge(&task.id, target_branch, strategy, work_dir).await?;

    // 4. Mutate task status to Done, unblock downstream tasks, and record evidence in write lock
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
            state.unblock_dependent_tasks(task_id);
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
        bail!(
            "Cannot perform git merge: '{}' is not inside a git working tree",
            work_dir.display()
        );
    }

    let current_branch_out = Command::new("git")
        .args(["-c", "core.fsmonitor=false", "branch", "--show-current"])
        .current_dir(work_dir)
        .output()
        .await
        .context("Failed to get current branch")?;

    let current_branch = String::from_utf8_lossy(&current_branch_out.stdout).trim().to_string();

    if current_branch == target_branch {
        // Already on target branch; return current HEAD directly
        let head_out = Command::new("git")
            .args(["-c", "core.fsmonitor=false", "rev-parse", "HEAD"])
            .current_dir(work_dir)
            .output()
            .await?;
        return Ok(String::from_utf8_lossy(&head_out.stdout).trim().to_string());
    }

    // Verify target_branch exists in refs/heads/
    let target_ref_check = Command::new("git")
        .args([
            "-c",
            "core.fsmonitor=false",
            "rev-parse",
            "--verify",
            &format!("refs/heads/{}", target_branch),
        ])
        .current_dir(work_dir)
        .output()
        .await?;

    if !target_ref_check.status.success() {
        bail!(
            "Target branch '{}' does not exist in local repository",
            target_branch
        );
    }

    // Use an isolated temporary worktree to merge so user's worktree/branch is completely untouched
    let tmp_worktree_dir = work_dir
        .join(".pm")
        .join("worktrees")
        .join(format!("merge_{}_{}", task_id, Utc::now().timestamp_micros()));

    if let Some(parent) = tmp_worktree_dir.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }

    // git worktree add --detach <tmp_worktree_dir> <target_branch>
    let add_worktree = Command::new("git")
        .args([
            "-c",
            "core.fsmonitor=false",
            "worktree",
            "add",
            "--detach",
            tmp_worktree_dir.to_str().unwrap(),
            target_branch,
        ])
        .current_dir(work_dir)
        .output()
        .await
        .context("Failed to create temporary worktree for merge")?;

    if !add_worktree.status.success() {
        bail!(
            "Failed to create temporary worktree at '{}': {}",
            tmp_worktree_dir.display(),
            String::from_utf8_lossy(&add_worktree.stderr)
        );
    }

    // Run merge inside temporary worktree
    let merge_arg = match strategy {
        "squash" => vec!["merge", "--squash", &current_branch],
        _ => vec![
            "merge",
            "--no-ff",
            "-m",
            "Merge task into target branch",
            &current_branch,
        ],
    };

    let merge = Command::new("git")
        .args([
            "-c",
            "core.fsmonitor=false",
            "-c",
            "user.name=pm",
            "-c",
            "user.email=pm@local",
        ])
        .args(merge_arg)
        .current_dir(&tmp_worktree_dir)
        .output()
        .await;

    let merge_result = match merge {
        Ok(out) if out.status.success() => {
            if strategy == "squash" {
                let commit = Command::new("git")
                    .args([
                        "-c",
                        "core.fsmonitor=false",
                        "-c",
                        "user.name=pm",
                        "-c",
                        "user.email=pm@local",
                        "commit",
                        "--allow-empty",
                        "-m",
                        &format!("feat: auto-merged task '{}' via pm local gate", task_id),
                    ])
                    .current_dir(&tmp_worktree_dir)
                    .output()
                    .await;

                match commit {
                    Ok(c_out) if c_out.status.success() => Ok(()),
                    Ok(c_out) => {
                        let err_msg = format!(
                            "{}\n{}",
                            String::from_utf8_lossy(&c_out.stdout).trim(),
                            String::from_utf8_lossy(&c_out.stderr).trim()
                        );
                        Err(anyhow::anyhow!("Failed to commit squashed merge: {}", err_msg.trim()))
                    }
                    Err(e) => Err(anyhow::anyhow!("Failed to run git commit: {}", e)),
                }
            } else {
                Ok(())
            }
        }
        Ok(out) => Err(anyhow::anyhow!(
            "Failed to merge branch '{}' into '{}': {}\n{}",
            current_branch,
            target_branch,
            String::from_utf8_lossy(&out.stdout).trim(),
            String::from_utf8_lossy(&out.stderr).trim()
        )),
        Err(e) => Err(anyhow::anyhow!("Failed to run git merge: {}", e)),
    };

    let final_commit = if merge_result.is_ok() {
        let head_out = Command::new("git")
            .args(["-c", "core.fsmonitor=false", "rev-parse", "HEAD"])
            .current_dir(&tmp_worktree_dir)
            .output()
            .await;

        match head_out {
            Ok(h_out) if h_out.status.success() => {
                let commit_hash = String::from_utf8_lossy(&h_out.stdout).trim().to_string();
                // Update target_branch ref to the new commit
                let _ = Command::new("git")
                    .args([
                        "-c",
                        "core.fsmonitor=false",
                        "update-ref",
                        &format!("refs/heads/{}", target_branch),
                        &commit_hash,
                    ])
                    .current_dir(work_dir)
                    .output()
                    .await;
                Ok(commit_hash)
            }
            _ => Err(anyhow::anyhow!("Failed to get final commit HEAD")),
        }
    } else {
        Err(merge_result.unwrap_err())
    };

    // Clean up temporary worktree
    let _ = Command::new("git")
        .args([
            "-c",
            "core.fsmonitor=false",
            "worktree",
            "remove",
            "--force",
            tmp_worktree_dir.to_str().unwrap(),
        ])
        .current_dir(work_dir)
        .output()
        .await;

    let _ = tokio::fs::remove_dir_all(&tmp_worktree_dir).await;

    final_commit
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Priority, ReviewComment, ReviewSeverity, Task};

    #[tokio::test]
    async fn test_execute_gate_creates_log_and_sha256() {
        let temp_dir = std::env::temp_dir().join(format!("pm_gate_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let run = execute_gate("task_log_1", "echo 'hello world from gate'", &temp_dir)
            .await
            .expect("execute_gate failed");

        assert_eq!(run.exit_code, 0);
        assert!(run.passed);
        assert!(run.stdout_display().contains("hello world from gate"));
        assert!(run.log_path.is_some());
        assert!(run.output_sha256.is_some());

        let rel_log = run.log_path.as_ref().unwrap();
        let log_file = temp_dir.join(rel_log);
        assert!(log_file.exists());
        let content = std::fs::read_to_string(&log_file).unwrap();
        assert!(content.contains("hello world from gate"));
        assert!(content.contains("=== GATE RUN:"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_verify_and_merge_rejects_non_git() {
        let temp_dir = std::env::temp_dir().join(format!("pm_nongit_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let store = PmStore::in_memory();
        let task_id = "task_non_git";

        store
            .write(|s| {
                s.upsert_task(Task {
                    id: task_id.to_string(),
                    project_id: "p1".to_string(),
                    feature_id: None,
                    title: "Non Git".to_string(),
                    description: "Desc".to_string(),
                    status: TaskStatus::Todo,
                    priority: Priority::Medium,
                    assignee: None,
                    blocked_by: vec![],
                    result_summary: None,
                    e2e_red_commit: None,
                    impl_red_commit: None,
                    created_at: "2026-09-06T00:00:00Z".to_string(),
                    updated_at: "2026-09-06T00:00:00Z".to_string(),
                });
                Ok(())
            })
            .await
            .unwrap();

        let res = verify_and_merge_task(&store, task_id, "main", "squash", &temp_dir).await;
        assert!(res.is_err());
        let err_msg = res.unwrap_err().to_string();
        assert!(err_msg.contains("not a git repository"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_verify_and_merge_rejects_commit_mismatch_and_reviews() {
        let store = PmStore::in_memory();
        let task_id = "task_guard";

        store
            .write(|s| {
                s.upsert_task(Task {
                    id: task_id.to_string(),
                    project_id: "p1".to_string(),
                    feature_id: None,
                    title: "Guarded Task".to_string(),
                    description: "Desc".to_string(),
                    status: TaskStatus::Todo,
                    priority: Priority::Medium,
                    assignee: None,
                    blocked_by: vec![],
                    result_summary: None,
                    e2e_red_commit: None,
                    impl_red_commit: None,
                    created_at: "2026-09-06T00:00:00Z".to_string(),
                    updated_at: "2026-09-06T00:00:00Z".to_string(),
                });
                s.upsert_review_comment(ReviewComment {
                    id: "rev_1".to_string(),
                    task_id: task_id.to_string(),
                    reviewer: "qa".to_string(),
                    content: "Blocker".to_string(),
                    severity: ReviewSeverity::Blocking,
                    file_path: None,
                    line_number: None,
                    resolved: false,
                    resolution_note: None,
                    created_at: "2026-09-06T00:00:00Z".to_string(),
                    updated_at: "2026-09-06T00:00:00Z".to_string(),
                });
                Ok(())
            })
            .await
            .unwrap();

        let fake_git_dir = std::env::temp_dir().join(format!("pm_guard_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&fake_git_dir);
        std::fs::create_dir_all(&fake_git_dir).unwrap();

        // Initialize a dummy git repo so it has a valid HEAD
        let _ = std::process::Command::new("git")
            .args(["-c", "core.fsmonitor=false", "init", "-b", "main"])
            .current_dir(&fake_git_dir)
            .output();
        let _ = std::process::Command::new("git")
            .args([
                "-c", "core.fsmonitor=false",
                "commit", "--allow-empty", "-m", "init",
                "--author", "Test <test@example.com>",
            ])
            .current_dir(&fake_git_dir)
            .output();

        // 1. Should fail on blocking review
        let err_review = verify_and_merge_task(&store, task_id, "main", "squash", &fake_git_dir).await;
        assert!(err_review.is_err());
        assert!(err_review.unwrap_err().to_string().contains("unresolved blocking review comment"));

        // Resolve review
        store.write(|s| {
            if let Some(c) = s.review_comments.get_mut("rev_1") {
                c.resolved = true;
            }
            Ok(())
        }).await.unwrap();

        // 2. Should fail on missing gate run
        let err_no_gate = verify_and_merge_task(&store, task_id, "main", "squash", &fake_git_dir).await;
        assert!(err_no_gate.is_err());
        assert!(err_no_gate.unwrap_err().to_string().contains("no gate run recorded"));

        // Record a gate run with outdated commit
        store.write(|s| {
            s.record_gate_run(GateRun {
                id: "gate_old".to_string(),
                task_id: task_id.to_string(),
                command: "echo test".to_string(),
                exit_code: 0,
                stdout_tail: "ok".to_string(),
                stderr_tail: "".to_string(),
                log_path: None,
                output_sha256: None,
                duration_ms: 10,
                head_commit: "0000000000000000000000000000000000000000".to_string(),
                passed: true,
                run_at: "2026-09-06T00:00:00Z".to_string(),
                stdout: None,
                stderr: None,
            });
            Ok(())
        }).await.unwrap();

        // 3. Should fail on commit mismatch
        let err_commit = verify_and_merge_task(&store, task_id, "main", "squash", &fake_git_dir).await;
        assert!(err_commit.is_err());
        let msg = err_commit.unwrap_err().to_string();
        assert!(msg.contains("was executed against commit"));
        assert!(msg.contains("0000000000000000000000000000000000000000"));

        let _ = std::fs::remove_dir_all(&fake_git_dir);
    }
}

