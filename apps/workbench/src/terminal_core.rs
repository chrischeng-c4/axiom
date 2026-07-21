// HANDWRITE-BEGIN gap="missing-generator:logic:17a15d58" tracker="pending-tracker" reason="Own profiles, macOS account-default-shell resolution, safe tab ids, real PTY sessions, byte frames, cwd, lifecycle state, and isolation."
use std::{
    collections::BTreeMap,
    env,
    ffi::CStr,
    fmt,
    io::Read,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::Duration,
};

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use serde::{Deserialize, Serialize};

use crate::{
    cwd_context::ActiveCwdContext,
    native_agent_pty::{
        AgentKind, AgentLaunchCommand, PtyCommand, PtyLaunchError, PtyRuntime, PtySession,
        PtySize,
    },
};

pub const MAX_TERMINAL_TAB_ID_BYTES: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalProfile {
    Claude,
    Codex,
    Agy,
    Shell,
}

impl TerminalProfile {
    pub const DEFAULTS: [Self; 4] = [Self::Claude, Self::Codex, Self::Agy, Self::Shell];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Claude => "Claude Code",
            Self::Codex => "Codex",
            Self::Agy => "AGY",
            Self::Shell => "Shell",
        }
    }

    fn agent_kind(self) -> Option<AgentKind> {
        match self {
            Self::Claude => Some(AgentKind::ClaudeCode),
            Self::Codex => Some(AgentKind::Codex),
            Self::Agy => Some(AgentKind::Agy),
            Self::Shell => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalSnapshot {
    pub tab_id: String,
    pub profile: TerminalProfile,
    pub label: String,
    pub running: bool,
    pub process_id: Option<u32>,
    pub exit_code: Option<u32>,
    pub active_cwd: String,
    pub cwd_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalFrame {
    pub snapshot: TerminalSnapshot,
    pub sequence: u64,
    pub output_base64: String,
}

#[derive(Debug)]
pub enum TerminalCoreError {
    InvalidTabId,
    InvalidWorkingDirectory(PathBuf),
    UnavailableProgram(PathBuf),
    AlreadyRunning(String),
    MissingSession(String),
    Operation(String),
}

impl fmt::Display for TerminalCoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTabId => write!(
                formatter,
                "tab id must be 1-{MAX_TERMINAL_TAB_ID_BYTES} ASCII letters, digits, dots, dashes, or underscores"
            ),
            Self::InvalidWorkingDirectory(path) => {
                write!(formatter, "{} is not a launchable directory", path.display())
            }
            Self::UnavailableProgram(path) => write!(
                formatter,
                "{} is unavailable; install it or start another terminal profile",
                path.display()
            ),
            Self::AlreadyRunning(tab_id) => {
                write!(formatter, "terminal tab {tab_id:?} is already running")
            }
            Self::MissingSession(tab_id) => {
                write!(formatter, "terminal tab {tab_id:?} has not been started")
            }
            Self::Operation(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for TerminalCoreError {}

impl From<PtyLaunchError> for TerminalCoreError {
    fn from(error: PtyLaunchError) -> Self {
        match error {
            PtyLaunchError::UnavailableBinary { program } => Self::UnavailableProgram(program),
            PtyLaunchError::InvalidWorkingDirectory { cwd } => Self::InvalidWorkingDirectory(cwd),
            other => Self::Operation(other.to_string()),
        }
    }
}

struct TerminalSession {
    tab_id: String,
    profile: TerminalProfile,
    pty: PtySession,
    output: Receiver<Vec<u8>>,
    active_cwd: ActiveCwdContext,
    cwd_from_osc7: bool,
    exit_code: Option<u32>,
    sequence: u64,
}

impl TerminalSession {
    fn spawn(
        runtime: &PtyRuntime,
        tab_id: &str,
        profile: TerminalProfile,
        command: &PtyCommand,
        size: PtySize,
    ) -> Result<Self, TerminalCoreError> {
        let active_cwd = ActiveCwdContext::new(&command.cwd)
            .map_err(|_| TerminalCoreError::InvalidWorkingDirectory(command.cwd.clone()))?;
        let pty = runtime.spawn(command, size)?;
        let mut reader = pty.try_clone_reader()?;
        let (sender, output) = mpsc::channel();
        thread::spawn(move || {
            let mut buffer = [0_u8; 8192];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(length) => {
                        if sender.send(buffer[..length].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(Self {
            tab_id: tab_id.to_owned(),
            profile,
            pty,
            output,
            active_cwd,
            cwd_from_osc7: false,
            exit_code: None,
            sequence: 0,
        })
    }

    fn poll(&mut self) -> Result<TerminalFrame, TerminalCoreError> {
        let mut output = self.drain_output();
        if self.exit_code.is_none() {
            if let Some(status) = self.pty.try_wait()? {
                self.exit_code = Some(status.exit_code());
                thread::sleep(Duration::from_millis(10));
                output.extend(self.drain_output());
            }
        }
        if !output.is_empty() {
            self.sequence = self.sequence.saturating_add(1);
        }
        Ok(self.frame(output))
    }

    fn input(&mut self, bytes: &[u8]) -> Result<TerminalFrame, TerminalCoreError> {
        if self.exit_code.is_some() {
            return Err(TerminalCoreError::Operation(
                "cannot send input to an exited terminal session".to_owned(),
            ));
        }
        self.pty.write_all(bytes)?;
        self.poll()
    }

    fn resize(&mut self, rows: u16, cols: u16) -> Result<TerminalFrame, TerminalCoreError> {
        if rows == 0 || cols == 0 {
            return Err(TerminalCoreError::Operation(
                "terminal rows and columns must be non-zero".to_owned(),
            ));
        }
        self.pty.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        self.poll()
    }

    fn interrupt(&mut self) -> Result<TerminalFrame, TerminalCoreError> {
        if self.exit_code.is_none() {
            self.pty.interrupt()?;
        }
        self.poll()
    }

    fn terminate(&mut self) -> Result<TerminalFrame, TerminalCoreError> {
        if self.exit_code.is_none() {
            let status = self.pty.terminate()?;
            self.exit_code = Some(status.exit_code());
            thread::sleep(Duration::from_millis(10));
        }
        let output = self.drain_output();
        if !output.is_empty() {
            self.sequence = self.sequence.saturating_add(1);
        }
        Ok(self.frame(output))
    }

    fn frame(&self, output: Vec<u8>) -> TerminalFrame {
        TerminalFrame {
            snapshot: TerminalSnapshot {
                tab_id: self.tab_id.clone(),
                profile: self.profile,
                label: self.profile.label().to_owned(),
                running: self.exit_code.is_none(),
                process_id: self.pty.process_id(),
                exit_code: self.exit_code,
                active_cwd: self.active_cwd.current().display().to_string(),
                cwd_source: if self.cwd_from_osc7 {
                    "OSC 7".to_owned()
                } else {
                    "Launch folder".to_owned()
                },
            },
            sequence: self.sequence,
            output_base64: BASE64_STANDARD.encode(output),
        }
    }

    fn drain_output(&mut self) -> Vec<u8> {
        let mut drained = Vec::new();
        loop {
            match self.output.try_recv() {
                Ok(chunk) => {
                    if !self.active_cwd.push_output(&chunk).is_empty() {
                        self.cwd_from_osc7 = true;
                    }
                    drained.extend(chunk);
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }
        drained
    }
}

pub struct TerminalCore {
    runtime: PtyRuntime,
    default_shell: PathBuf,
    sessions: BTreeMap<String, TerminalSession>,
}

impl Default for TerminalCore {
    fn default() -> Self {
        Self::with_runtime_and_shell(PtyRuntime::default(), system_default_shell())
    }
}

impl TerminalCore {
    pub fn with_runtime_and_shell(
        runtime: PtyRuntime,
        default_shell: impl Into<PathBuf>,
    ) -> Self {
        Self {
            runtime,
            default_shell: default_shell.into(),
            sessions: BTreeMap::new(),
        }
    }

    pub fn default_shell(&self) -> &Path {
        &self.default_shell
    }

    pub fn launch(
        &mut self,
        tab_id: &str,
        profile: TerminalProfile,
        cwd: impl Into<PathBuf>,
        size: PtySize,
    ) -> Result<TerminalFrame, TerminalCoreError> {
        validate_tab_id(tab_id)?;
        if let Some(existing) = self.sessions.get_mut(tab_id) {
            if existing.poll()?.snapshot.running {
                return Err(TerminalCoreError::AlreadyRunning(tab_id.to_owned()));
            }
        }
        let cwd = cwd.into();
        let command = if let Some(kind) = profile.agent_kind() {
            AgentLaunchCommand::for_kind(kind, cwd).as_pty_command()
        } else {
            PtyCommand::new(self.default_shell.clone(), cwd)
        };
        let session = TerminalSession::spawn(&self.runtime, tab_id, profile, &command, size)?;
        let frame = session.frame(Vec::new());
        self.sessions.insert(tab_id.to_owned(), session);
        Ok(frame)
    }

    pub fn poll(&mut self, tab_id: &str) -> Result<TerminalFrame, TerminalCoreError> {
        self.session_mut(tab_id)?.poll()
    }

    pub fn input(
        &mut self,
        tab_id: &str,
        bytes: &[u8],
    ) -> Result<TerminalFrame, TerminalCoreError> {
        self.session_mut(tab_id)?.input(bytes)
    }

    pub fn resize(
        &mut self,
        tab_id: &str,
        rows: u16,
        cols: u16,
    ) -> Result<TerminalFrame, TerminalCoreError> {
        self.session_mut(tab_id)?.resize(rows, cols)
    }

    pub fn interrupt(&mut self, tab_id: &str) -> Result<TerminalFrame, TerminalCoreError> {
        self.session_mut(tab_id)?.interrupt()
    }

    pub fn terminate(&mut self, tab_id: &str) -> Result<TerminalFrame, TerminalCoreError> {
        self.session_mut(tab_id)?.terminate()
    }

    pub fn terminate_all(&mut self) {
        for session in self.sessions.values_mut() {
            let _ = session.terminate();
        }
    }

    fn session_mut(&mut self, tab_id: &str) -> Result<&mut TerminalSession, TerminalCoreError> {
        validate_tab_id(tab_id)?;
        self.sessions
            .get_mut(tab_id)
            .ok_or_else(|| TerminalCoreError::MissingSession(tab_id.to_owned()))
    }
}

impl Drop for TerminalCore {
    fn drop(&mut self) {
        self.terminate_all();
    }
}

pub fn validate_tab_id(tab_id: &str) -> Result<(), TerminalCoreError> {
    let valid = !tab_id.is_empty()
        && tab_id.len() <= MAX_TERMINAL_TAB_ID_BYTES
        && tab_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
    valid.then_some(()).ok_or(TerminalCoreError::InvalidTabId)
}

pub fn system_default_shell() -> PathBuf {
    account_default_shell()
        .or_else(|| env::var_os("SHELL").filter(|value| !value.is_empty()).map(PathBuf::from))
        .unwrap_or_else(platform_fallback_shell)
}

#[cfg(unix)]
fn account_default_shell() -> Option<PathBuf> {
    use std::os::unix::ffi::OsStringExt;

    let entry = unsafe { libc::getpwuid(libc::getuid()) };
    if entry.is_null() {
        return None;
    }
    let shell = unsafe { (*entry).pw_shell };
    if shell.is_null() {
        return None;
    }
    let bytes = unsafe { CStr::from_ptr(shell) }.to_bytes();
    (!bytes.is_empty()).then(|| PathBuf::from(std::ffi::OsString::from_vec(bytes.to_vec())))
}

#[cfg(not(unix))]
fn account_default_shell() -> Option<PathBuf> {
    env::var_os("COMSPEC")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

#[cfg(unix)]
fn platform_fallback_shell() -> PathBuf {
    PathBuf::from("/bin/sh")
}

#[cfg(windows)]
fn platform_fallback_shell() -> PathBuf {
    PathBuf::from("cmd.exe")
}

#[cfg(not(any(unix, windows)))]
fn platform_fallback_shell() -> PathBuf {
    PathBuf::from("sh")
}

<!-- marker: missing-generator:logic:17a15d58 path: apps/workbench/src/terminal_core.rs reason: Own profiles, macOS account-default-shell resolution, safe tab ids, real PTY sessions, byte frames, cwd, lifecycle state, and isolation. -->
// HANDWRITE-END
