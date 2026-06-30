// HANDWRITE-BEGIN gap="missing-generator:logic:cf1699b3" tracker="pending-tracker" reason="Add the first opt-in per-session queue. CAP_SESSION_ID enables local session state, profiled no-observe commands enqueue background jobs, and observe commands drain prior same-session jobs."
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{bail, Context, Result};

const SESSION_ENV: &str = "CAP_SESSION_ID";
const QUEUE_DIR_ENV: &str = "CAP_SESSION_QUEUE_DIR";
const DEFAULT_WAIT_MS: u64 = 30_000;

/// @spec projects/cap/tech-design/logic/add-per-session-queue-with-observe-command-barriers.md#logic
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueDecision {
    ContinueSynchronously,
    Queued { job_id: String },
    PriorFailure { job_id: String, exit_code: i32 },
}

/// @spec projects/cap/tech-design/logic/add-per-session-queue-with-observe-command-barriers.md#logic
impl QueueDecision {
    pub fn exit_code(&self) -> Option<ExitCode> {
        match self {
            Self::ContinueSynchronously => None,
            Self::Queued { .. } => Some(ExitCode::SUCCESS),
            Self::PriorFailure { exit_code, .. } => Some(exit_code_from_i32(*exit_code)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WaitPolicy {
    NoObserve,
    Observe,
    Synchronous,
}

#[derive(Debug)]
struct JobRecord {
    id: String,
    command: String,
    status_path: PathBuf,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
}

/// @spec projects/cap/tech-design/logic/add-per-session-queue-with-observe-command-barriers.md#logic
pub fn handle_command_string(command: &str) -> Result<QueueDecision> {
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    handle_command_string_with_io(command, &mut stdout, &mut stderr)
}

fn handle_command_string_with_io(
    command: &str,
    stdout: &mut dyn io::Write,
    stderr: &mut dyn io::Write,
) -> Result<QueueDecision> {
    let Some(session_id) = session_id() else {
        return Ok(QueueDecision::ContinueSynchronously);
    };
    let Some(words) = split_profiled_words(command) else {
        return Ok(QueueDecision::ContinueSynchronously);
    };
    match classify(&words) {
        WaitPolicy::NoObserve => enqueue_job(&session_id, command, &words, stdout),
        WaitPolicy::Observe => drain_session(&session_id, stderr),
        WaitPolicy::Synchronous => Ok(QueueDecision::ContinueSynchronously),
    }
}

fn session_id() -> Option<String> {
    env::var(SESSION_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(|value| sanitize_session(&value))
}

fn split_profiled_words(command: &str) -> Option<Vec<String>> {
    if command.chars().any(|ch| {
        matches!(
            ch,
            '|' | '&' | ';' | '<' | '>' | '(' | ')' | '$' | '`' | '"' | '\'' | '\n'
        )
    }) {
        return None;
    }
    let words = command
        .split_whitespace()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    (!words.is_empty()).then_some(words)
}

fn classify(words: &[String]) -> WaitPolicy {
    let Some(program) = words.first().map(|word| basename(word)) else {
        return WaitPolicy::Synchronous;
    };
    match program.as_str() {
        "touch" if words.len() > 1 && words[1..].iter().all(|arg| !arg.starts_with('-')) => {
            WaitPolicy::NoObserve
        }
        "ls" | "cat" | "grep" | "find" => WaitPolicy::Observe,
        _ => WaitPolicy::Synchronous,
    }
}

fn enqueue_job(
    session_id: &str,
    command: &str,
    words: &[String],
    stdout: &mut dyn io::Write,
) -> Result<QueueDecision> {
    let dir = session_dir(session_id)?;
    let id = next_job_id();
    let status_path = dir.join(format!("{id}.status"));
    let stdout_path = dir.join(format!("{id}.stdout"));
    let stderr_path = dir.join(format!("{id}.stderr"));
    let record_path = dir.join(format!("{id}.job"));
    let script = format!(
        "{} > {} 2> {}; code=$?; printf '%s\\n' \"$code\" > {}",
        shell_join(words),
        shell_quote_path(&stdout_path),
        shell_quote_path(&stderr_path),
        shell_quote_path(&status_path),
    );

    let child = Command::new("sh")
        .arg("-c")
        .arg(script)
        .spawn()
        .with_context(|| format!("enqueue queued cap job for {command}"))?;

    fs::write(
        &record_path,
        format!(
            "id={id}\npid={}\ncommand={command}\nstatus={}\nstdout={}\nstderr={}\n",
            child.id(),
            status_path.display(),
            stdout_path.display(),
            stderr_path.display()
        ),
    )?;
    writeln!(stdout, "cap queue: queued job {id}")?;
    Ok(QueueDecision::Queued { job_id: id })
}

fn drain_session(session_id: &str, stderr: &mut dyn io::Write) -> Result<QueueDecision> {
    let dir = session_dir(session_id)?;
    let mut records = fs::read_dir(&dir)?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "job"))
        .collect::<Vec<_>>();
    records.sort();
    for record_path in records {
        let record = read_job_record(&record_path)?;
        wait_for_status(&record.status_path)?;
        let exit_code = fs::read_to_string(&record.status_path)
            .ok()
            .and_then(|text| text.trim().parse::<i32>().ok())
            .unwrap_or(1);
        if exit_code != 0 {
            let prior_stderr = fs::read_to_string(&record.stderr_path).unwrap_or_default();
            writeln!(
                stderr,
                "cap queue: prior job {} failed with exit {}: {}",
                record.id, exit_code, record.command
            )?;
            if !prior_stderr.trim().is_empty() {
                write!(stderr, "{prior_stderr}")?;
            }
            cleanup_job(&record_path, &record);
            return Ok(QueueDecision::PriorFailure {
                job_id: record.id,
                exit_code,
            });
        }
        cleanup_job(&record_path, &record);
    }
    Ok(QueueDecision::ContinueSynchronously)
}

fn read_job_record(path: &Path) -> Result<JobRecord> {
    let text = fs::read_to_string(path)?;
    let mut id = String::new();
    let mut command = String::new();
    let mut status_path = PathBuf::new();
    let mut stdout_path = PathBuf::new();
    let mut stderr_path = PathBuf::new();
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "id" => id = value.to_string(),
            "command" => command = value.to_string(),
            "status" => status_path = PathBuf::from(value),
            "stdout" => stdout_path = PathBuf::from(value),
            "stderr" => stderr_path = PathBuf::from(value),
            _ => {}
        }
    }
    if id.is_empty() || status_path.as_os_str().is_empty() {
        bail!("invalid cap queue job record: {}", path.display());
    }
    Ok(JobRecord {
        id,
        command,
        status_path,
        stdout_path,
        stderr_path,
    })
}

fn wait_for_status(status_path: &Path) -> Result<()> {
    let deadline = Instant::now() + Duration::from_millis(DEFAULT_WAIT_MS);
    while !status_path.exists() {
        if Instant::now() >= deadline {
            bail!(
                "timed out waiting for queued cap job {}",
                status_path.display()
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}

fn cleanup_job(record_path: &Path, record: &JobRecord) {
    let _ = fs::remove_file(record_path);
    let _ = fs::remove_file(&record.status_path);
    let _ = fs::remove_file(&record.stdout_path);
    let _ = fs::remove_file(&record.stderr_path);
}

fn session_dir(session_id: &str) -> Result<PathBuf> {
    let root = env::var_os(QUEUE_DIR_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("cap-session-queue"));
    let dir = root.join(session_id);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn next_job_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{millis}-{}", std::process::id())
}

fn sanitize_session(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn basename(program: &str) -> String {
    Path::new(program)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(program)
        .to_string()
}

fn shell_join(words: &[String]) -> String {
    words
        .iter()
        .map(|word| shell_quote(word))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_quote_path(path: &Path) -> String {
    shell_quote(&path.display().to_string())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn exit_code_from_i32(code: i32) -> ExitCode {
    if (0..=255).contains(&code) {
        ExitCode::from(code as u8)
    } else {
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    struct EnvGuard {
        _guard: std::sync::MutexGuard<'static, ()>,
        old_session: Option<String>,
        old_dir: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn new(session: &str, dir: &Path) -> Self {
            let guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
            let old_session = env::var(SESSION_ENV).ok();
            let old_dir = env::var_os(QUEUE_DIR_ENV);
            env::set_var(SESSION_ENV, session);
            env::set_var(QUEUE_DIR_ENV, dir);
            Self {
                _guard: guard,
                old_session,
                old_dir,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.old_session {
                Some(value) => env::set_var(SESSION_ENV, value),
                None => env::remove_var(SESSION_ENV),
            }
            match &self.old_dir {
                Some(value) => env::set_var(QUEUE_DIR_ENV, value),
                None => env::remove_var(QUEUE_DIR_ENV),
            }
        }
    }

    #[test]
    fn session_queue_queued_touch_returns_metadata_and_observe_drains() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let _env = EnvGuard::new("queue-touch", temp.path());
        let touched = temp.path().join("created.txt");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let queued = handle_command_string_with_io(
            &format!("touch {}", touched.display()),
            &mut stdout,
            &mut stderr,
        )?;
        assert!(matches!(queued, QueueDecision::Queued { .. }));
        assert!(String::from_utf8_lossy(&stdout).contains("cap queue: queued job"));

        let observe = handle_command_string_with_io(
            &format!("ls {}", touched.display()),
            &mut stdout,
            &mut stderr,
        )?;
        assert_eq!(observe, QueueDecision::ContinueSynchronously);
        assert!(touched.exists(), "observe barrier should wait for touch");
        assert!(session_dir("queue-touch")?.read_dir()?.next().is_none());
        Ok(())
    }

    #[test]
    fn session_queue_observe_reports_prior_failure() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let _env = EnvGuard::new("queue-failure", temp.path());
        let missing_parent = temp.path().join("missing").join("file.txt");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let queued = handle_command_string_with_io(
            &format!("touch {}", missing_parent.display()),
            &mut stdout,
            &mut stderr,
        )?;
        assert!(matches!(queued, QueueDecision::Queued { .. }));

        let observe = handle_command_string_with_io("ls .", &mut stdout, &mut stderr)?;
        assert!(matches!(observe, QueueDecision::PriorFailure { .. }));
        let text = String::from_utf8_lossy(&stderr);
        assert!(text.contains("prior job"));
        assert!(text.contains("failed"));
        Ok(())
    }

    #[test]
    fn session_queue_unknown_command_remains_synchronous() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let _env = EnvGuard::new("queue-unknown", temp.path());
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let decision = handle_command_string_with_io("echo hello", &mut stdout, &mut stderr)?;
        assert_eq!(decision, QueueDecision::ContinueSynchronously);
        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
        Ok(())
    }

    #[test]
    fn session_queue_disabled_without_session_id() -> Result<()> {
        let guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let old_session = env::var(SESSION_ENV).ok();
        env::remove_var(SESSION_ENV);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let decision = handle_command_string_with_io("touch file", &mut stdout, &mut stderr)?;
        assert_eq!(decision, QueueDecision::ContinueSynchronously);
        if let Some(value) = old_session {
            env::set_var(SESSION_ENV, value);
        }
        drop(guard);
        Ok(())
    }
}

// HANDWRITE-END
