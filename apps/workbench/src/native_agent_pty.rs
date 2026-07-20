// HANDWRITE-BEGIN gap="missing-generator:logic:c4f5b6e3" tracker="pending-tracker" reason="Define provider command construction, recoverable binary resolution, and the real native PTY session lifecycle."
//! Native agent command construction and real PTY lifecycle.

use std::env;
use std::ffi::OsString;
use std::fmt;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty};
pub use portable_pty::{ExitStatus, PtySize};

use crate::cwd_context::{CWD_TELEMETRY_ENV, CWD_TELEMETRY_PROTOCOL};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Native agent providers supported by Workbench.
///
/// Workbench constructs their CLI command but does not own vendor session or
/// history state.
///
/// @spec apps/workbench/tech-design/interfaces/cli/launch-native-claude-code-codex-and-agy-clis-through-a-real-pty.md#logic
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AgentKind {
    #[default]
    ClaudeCode,
    Codex,
    Agy,
}

impl AgentKind {
    pub const ALL: [Self; 3] = [Self::ClaudeCode, Self::Codex, Self::Agy];

    pub const fn program(self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude",
            Self::Codex => "codex",
            Self::Agy => "agy",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::ClaudeCode => "Claude Code",
            Self::Codex => "Codex",
            Self::Agy => "AGY",
        }
    }
}

/// Inspectable command produced by one native agent adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentLaunchCommand {
    pub agent: AgentKind,
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub cwd: PathBuf,
}

impl AgentLaunchCommand {
    pub fn for_kind(agent: AgentKind, cwd: impl Into<PathBuf>) -> Self {
        Self {
            agent,
            program: PathBuf::from(agent.program()),
            args: Vec::new(),
            cwd: cwd.into(),
        }
    }

    pub fn as_pty_command(&self) -> PtyCommand {
        PtyCommand {
            program: self.program.clone(),
            args: self.args.clone(),
            cwd: self.cwd.clone(),
        }
    }
}

/// Provider-neutral command accepted by the PTY runtime.
///
/// Tests use this type with a deterministic local shell; production converts
/// an `AgentLaunchCommand` into it without changing program, args, or cwd.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtyCommand {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub cwd: PathBuf,
}

impl PtyCommand {
    pub fn new(program: impl Into<PathBuf>, cwd: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            cwd: cwd.into(),
        }
    }

    pub fn arg(mut self, arg: impl Into<OsString>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }
}

/// Typed, recoverable failures from the native PTY boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PtyLaunchError {
    UnavailableBinary {
        program: PathBuf,
    },
    InvalidWorkingDirectory {
        cwd: PathBuf,
    },
    SessionClosed,
    Operation {
        operation: &'static str,
        message: String,
    },
}

impl PtyLaunchError {
    fn operation(operation: &'static str, error: impl fmt::Display) -> Self {
        Self::Operation {
            operation,
            message: error.to_string(),
        }
    }
}

impl fmt::Display for PtyLaunchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnavailableBinary { program } => write!(
                formatter,
                "{} is unavailable; install it or select another native agent",
                program.display()
            ),
            Self::InvalidWorkingDirectory { cwd } => {
                write!(formatter, "{} is not a launchable directory", cwd.display())
            }
            Self::SessionClosed => formatter.write_str("the PTY session is already closed"),
            Self::Operation { operation, message } => {
                write!(formatter, "PTY {operation} failed: {message}")
            }
        }
    }
}

impl std::error::Error for PtyLaunchError {}

pub type PtyResult<T> = Result<T, PtyLaunchError>;

/// Program resolver and PTY factory.
///
/// A custom search path supports deterministic unavailable-binary tests without
/// mutating global process environment.
#[derive(Debug, Clone)]
pub struct PtyRuntime {
    search_path: Option<OsString>,
}

impl Default for PtyRuntime {
    fn default() -> Self {
        Self {
            search_path: env::var_os("PATH"),
        }
    }
}

impl PtyRuntime {
    pub fn with_search_path(search_path: impl Into<OsString>) -> Self {
        Self {
            search_path: Some(search_path.into()),
        }
    }

    pub fn spawn_agent(
        &self,
        command: &AgentLaunchCommand,
        size: PtySize,
    ) -> PtyResult<PtySession> {
        self.spawn(&command.as_pty_command(), size)
    }

    pub fn spawn(&self, command: &PtyCommand, size: PtySize) -> PtyResult<PtySession> {
        if !command.cwd.is_dir() {
            return Err(PtyLaunchError::InvalidWorkingDirectory {
                cwd: command.cwd.clone(),
            });
        }
        let program = self.resolve_program(&command.program).ok_or_else(|| {
            PtyLaunchError::UnavailableBinary {
                program: command.program.clone(),
            }
        })?;

        let pair = native_pty_system()
            .openpty(size)
            .map_err(|error| PtyLaunchError::operation("allocation", error))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|error| PtyLaunchError::operation("writer creation", error))?;

        let mut builder = CommandBuilder::new(program);
        builder.args(command.args.iter());
        builder.cwd(&command.cwd);
        builder.env("TERM", "xterm-256color");
        builder.env("COLORTERM", "truecolor");
        builder.env(CWD_TELEMETRY_ENV, CWD_TELEMETRY_PROTOCOL);
        let child = pair.slave.spawn_command(builder).map_err(|error| {
            PtyLaunchError::operation(
                "child spawn",
                format!("{}: {error}", command.program.display()),
            )
        })?;
        drop(pair.slave);

        Ok(PtySession {
            master: pair.master,
            writer,
            child: Some(child),
            exit_status: None,
        })
    }

    fn resolve_program(&self, program: &Path) -> Option<PathBuf> {
        let has_path = program.is_absolute() || program.components().count() > 1;
        if has_path {
            return is_executable_file(program).then(|| program.to_path_buf());
        }
        let search_path = self.search_path.as_deref()?;
        env::split_paths(search_path)
            .map(|directory| directory.join(program))
            .find(|candidate| is_executable_file(candidate))
    }
}

fn is_executable_file(path: &Path) -> bool {
    let Ok(metadata) = path.metadata() else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// One live child process attached to a native pseudo-terminal.
pub struct PtySession {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Option<Box<dyn Child + Send + Sync>>,
    exit_status: Option<ExitStatus>,
}

impl PtySession {
    pub fn spawn(command: &PtyCommand, size: PtySize) -> PtyResult<Self> {
        PtyRuntime::default().spawn(command, size)
    }

    pub fn try_clone_reader(&self) -> PtyResult<Box<dyn Read + Send>> {
        self.master
            .try_clone_reader()
            .map_err(|error| PtyLaunchError::operation("reader clone", error))
    }

    pub fn write_all(&mut self, bytes: &[u8]) -> PtyResult<()> {
        self.writer
            .write_all(bytes)
            .and_then(|()| self.writer.flush())
            .map_err(|error| PtyLaunchError::operation("input", error))
    }

    pub fn interrupt(&mut self) -> PtyResult<()> {
        self.write_all(&[0x03])
    }

    pub fn resize(&self, size: PtySize) -> PtyResult<()> {
        self.master
            .resize(size)
            .map_err(|error| PtyLaunchError::operation("resize", error))
    }

    pub fn size(&self) -> PtyResult<PtySize> {
        self.master
            .get_size()
            .map_err(|error| PtyLaunchError::operation("size query", error))
    }

    pub fn process_id(&self) -> Option<u32> {
        self.child.as_ref().and_then(|child| child.process_id())
    }

    pub fn try_wait(&mut self) -> PtyResult<Option<ExitStatus>> {
        if let Some(status) = &self.exit_status {
            return Ok(Some(status.clone()));
        }
        let Some(child) = self.child.as_mut() else {
            return Ok(None);
        };
        let status = child
            .try_wait()
            .map_err(|error| PtyLaunchError::operation("exit poll", error))?;
        if let Some(status) = status {
            self.exit_status = Some(status.clone());
            self.child = None;
            Ok(Some(status))
        } else {
            Ok(None)
        }
    }

    pub fn wait(mut self) -> PtyResult<ExitStatus> {
        if let Some(status) = self.exit_status.clone() {
            return Ok(status);
        }
        let status = self
            .child
            .as_mut()
            .ok_or(PtyLaunchError::SessionClosed)?
            .wait()
            .map_err(|error| PtyLaunchError::operation("wait", error))?;
        self.exit_status = Some(status.clone());
        self.child = None;
        Ok(status)
    }

    pub fn terminate(&mut self) -> PtyResult<ExitStatus> {
        if let Some(status) = self.try_wait()? {
            return Ok(status);
        }
        let child = self.child.as_mut().ok_or(PtyLaunchError::SessionClosed)?;
        child
            .kill()
            .map_err(|error| PtyLaunchError::operation("termination", error))?;
        let status = child
            .wait()
            .map_err(|error| PtyLaunchError::operation("reap", error))?;
        self.exit_status = Some(status.clone());
        self.child = None;
        Ok(status)
    }
}

impl Drop for PtySession {
    fn drop(&mut self) {
        let Some(child) = self.child.as_mut() else {
            return;
        };
        if child.try_wait().ok().flatten().is_none() {
            let _ = child.kill();
        }
        let _ = child.wait();
        self.child = None;
    }
}

// HANDWRITE-END
