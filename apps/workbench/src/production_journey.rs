// HANDWRITE-BEGIN gap="missing-generator:logic:271042a0" tracker="pending-tracker" reason="Assemble real PTY launch/IO/poll/resize/terminate, OSC7 cwd, bounded transcript, renderer requests, recoverable agent errors, and serializable desktop snapshots."
use std::{
    io::Read,
    path::PathBuf,
    sync::{
        mpsc::{self, Receiver, TryRecvError},
        Mutex,
    },
    thread,
};

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    context::{ContextDocument, ContextRequest, RendererRegistry},
    cwd_context::ActiveCwdContext,
    native_agent_pty::{
        AgentKind, AgentLaunchCommand, PtyCommand, PtyLaunchError, PtyResult, PtyRuntime,
        PtySession, PtySize,
    },
};

pub const MAX_TRANSCRIPT_BYTES: usize = 512 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JourneySnapshot {
    pub agent: String,
    pub running: bool,
    pub process_id: Option<u32>,
    pub exit_code: Option<u32>,
    pub active_cwd: String,
    pub cwd_source: String,
    pub transcript: String,
}

pub struct JourneySession {
    agent_label: String,
    pty: PtySession,
    output: Receiver<Vec<u8>>,
    active_cwd: ActiveCwdContext,
    cwd_from_osc7: bool,
    transcript: Vec<u8>,
    exit_code: Option<u32>,
}

impl JourneySession {
    pub fn spawn_agent(
        runtime: &PtyRuntime,
        command: &AgentLaunchCommand,
        size: PtySize,
    ) -> PtyResult<Self> {
        let active_cwd = ActiveCwdContext::new(&command.cwd).map_err(|_| {
            PtyLaunchError::InvalidWorkingDirectory {
                cwd: command.cwd.clone(),
            }
        })?;
        let pty = runtime.spawn_agent(command, size)?;
        Self::from_pty(command.agent.label(), pty, active_cwd)
    }

    pub fn spawn_command(
        label: impl Into<String>,
        command: &PtyCommand,
        size: PtySize,
    ) -> PtyResult<Self> {
        let active_cwd = ActiveCwdContext::new(&command.cwd).map_err(|_| {
            PtyLaunchError::InvalidWorkingDirectory {
                cwd: command.cwd.clone(),
            }
        })?;
        let pty = PtyRuntime::default().spawn(command, size)?;
        Self::from_pty(label, pty, active_cwd)
    }

    fn from_pty(
        label: impl Into<String>,
        pty: PtySession,
        active_cwd: ActiveCwdContext,
    ) -> PtyResult<Self> {
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
            agent_label: label.into(),
            pty,
            output,
            active_cwd,
            cwd_from_osc7: false,
            transcript: Vec::new(),
            exit_code: None,
        })
    }

    pub fn send_input(&mut self, input: &[u8]) -> PtyResult<()> {
        self.pty.write_all(input)
    }

    pub fn resize(&self, rows: u16, cols: u16) -> PtyResult<()> {
        self.pty.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
    }

    pub fn interrupt(&mut self) -> PtyResult<()> {
        self.pty.interrupt()
    }

    pub fn terminate(&mut self) -> PtyResult<JourneySnapshot> {
        let status = self.pty.terminate()?;
        self.exit_code = Some(status.exit_code());
        self.drain_output();
        Ok(self.snapshot())
    }

    pub fn poll(&mut self) -> PtyResult<JourneySnapshot> {
        self.drain_output();
        if self.exit_code.is_none() {
            if let Some(status) = self.pty.try_wait()? {
                self.exit_code = Some(status.exit_code());
                self.drain_output();
            }
        }
        Ok(self.snapshot())
    }

    pub fn snapshot(&self) -> JourneySnapshot {
        JourneySnapshot {
            agent: self.agent_label.clone(),
            running: self.exit_code.is_none(),
            process_id: self.pty.process_id(),
            exit_code: self.exit_code,
            active_cwd: self.active_cwd.current().display().to_string(),
            cwd_source: if self.cwd_from_osc7 {
                "OSC 7".to_owned()
            } else {
                "Launch folder".to_owned()
            },
            transcript: String::from_utf8_lossy(&self.transcript).into_owned(),
        }
    }

    fn drain_output(&mut self) {
        loop {
            match self.output.try_recv() {
                Ok(chunk) => {
                    if !self.active_cwd.push_output(&chunk).is_empty() {
                        self.cwd_from_osc7 = true;
                    }
                    self.transcript.extend_from_slice(&chunk);
                    if self.transcript.len() > MAX_TRANSCRIPT_BYTES {
                        let excess = self.transcript.len() - MAX_TRANSCRIPT_BYTES;
                        self.transcript.drain(..excess);
                    }
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }
    }
}

pub struct ProductionJourneyStore {
    runtime: PtyRuntime,
    session: Mutex<Option<JourneySession>>,
}

impl Default for ProductionJourneyStore {
    fn default() -> Self {
        Self::with_runtime(PtyRuntime::default())
    }
}

impl ProductionJourneyStore {
    /// Construct the production command store with an explicit PTY resolver.
    ///
    /// The desktop host uses the process PATH. External-contract tests inject
    /// only a deterministic agent executable while retaining the real PTY and
    /// Tauri command boundary.
    pub fn with_runtime(runtime: PtyRuntime) -> Self {
        Self {
            runtime,
            session: Mutex::new(None),
        }
    }

    fn with_session<T>(
        &self,
        action: impl FnOnce(&mut JourneySession) -> PtyResult<T>,
    ) -> Result<T, String> {
        let mut session = self
            .session
            .lock()
            .map_err(|_| "Production journey session lock is poisoned".to_owned())?;
        let session = session
            .as_mut()
            .ok_or_else(|| "No native agent session is active".to_owned())?;
        action(session).map_err(|error| error.to_string())
    }
}

#[tauri::command]
pub fn launch_journey_agent(
    agent: String,
    cwd: String,
    store: State<'_, ProductionJourneyStore>,
) -> Result<JourneySnapshot, String> {
    let kind = parse_agent_kind(&agent)?;
    let command = AgentLaunchCommand::for_kind(kind, PathBuf::from(cwd));
    let session = JourneySession::spawn_agent(
        &store.runtime,
        &command,
        PtySize {
            rows: 28,
            cols: 100,
            pixel_width: 0,
            pixel_height: 0,
        },
    )
    .map_err(|error| error.to_string())?;
    let snapshot = session.snapshot();
    *store
        .session
        .lock()
        .map_err(|_| "Production journey session lock is poisoned".to_owned())? = Some(session);
    Ok(snapshot)
}

#[tauri::command]
pub fn poll_journey_agent(
    store: State<'_, ProductionJourneyStore>,
) -> Result<JourneySnapshot, String> {
    store.with_session(JourneySession::poll)
}

#[tauri::command]
pub fn send_journey_input(
    input: String,
    store: State<'_, ProductionJourneyStore>,
) -> Result<JourneySnapshot, String> {
    store.with_session(|session| {
        session.send_input(input.as_bytes())?;
        if !input.ends_with('\n') {
            session.send_input(b"\n")?;
        }
        session.poll()
    })
}

#[tauri::command]
pub fn resize_journey_agent(
    rows: u16,
    cols: u16,
    store: State<'_, ProductionJourneyStore>,
) -> Result<JourneySnapshot, String> {
    if rows == 0 || cols == 0 {
        return Err("Terminal rows and columns must be non-zero".to_owned());
    }
    store.with_session(|session| {
        session.resize(rows, cols)?;
        session.poll()
    })
}

#[tauri::command]
pub fn interrupt_journey_agent(
    store: State<'_, ProductionJourneyStore>,
) -> Result<JourneySnapshot, String> {
    store.with_session(|session| {
        session.interrupt()?;
        session.poll()
    })
}

#[tauri::command]
pub fn terminate_journey_agent(
    store: State<'_, ProductionJourneyStore>,
) -> Result<JourneySnapshot, String> {
    store.with_session(JourneySession::terminate)
}

#[tauri::command]
pub fn render_journey_context(
    root: String,
    target: Option<String>,
) -> Result<ContextDocument, String> {
    let request = match target {
        Some(target) => ContextRequest::file(&root, target),
        None => ContextRequest::workspace(&root),
    }
    .map_err(|error| error.to_string())?;
    Ok(RendererRegistry::production().render(&request))
}

fn parse_agent_kind(value: &str) -> Result<AgentKind, String> {
    match value {
        "claude" => Ok(AgentKind::ClaudeCode),
        "codex" => Ok(AgentKind::Codex),
        "agy" => Ok(AgentKind::Agy),
        _ => Err(format!("Unknown native agent {value:?}")),
    }
}
// HANDWRITE-END
