// HANDWRITE-BEGIN gap="missing-generator:ephemeral-k8s-session-exec-json" tracker="#1693" reason="A leased K3s exec crosses private credential, exact backing-machine, operation-lock, and child-process boundaries. Its agent JSON capture must keep the lease proof through spawn while boundedly draining arbitrary child output, which is host-runtime lifecycle policy rather than a generic generator primitive."
//! Agent-safe execution in an active, ephemeral K3s lease.
//!
//! The text path preserves inherited foreground stdout/stderr. The JSON path
//! adds one bounded, single-document result for an agent that needs to run
//! ordinary `kubectl`, `helm`, or `kustomize` commands without parsing child
//! output followed by a separate terminal record.

use std::fs;
use std::io::Read;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, ExitStatus, Stdio};
use std::sync::{
    atomic::{AtomicI32, Ordering},
    mpsc::{self, Receiver},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use signal_hook::{
    consts::signal::{SIGINT, SIGTERM},
    iterator::{Handle as SignalHandle, Signals},
};

use super::{
    ensure_apple_container, exit_code, k8s_host_command, read_active_session,
    require_active_session_lease, require_private_file, sensitive_environment,
    validate_active_session_backing, verify_host_api, write_new_marker, ActiveSession,
    SessionKubeconfig,
};

const MAX_SESSION_EXEC_STREAM_CAPTURE_BYTES: u64 = 64 * 1024;
const MAX_SESSION_EXEC_JSON_STREAM_VALUE_BYTES: usize = 64 * 1024;
const MAX_SESSION_EXEC_TIMEOUT_SECONDS: u64 = 4 * 60 * 60;
const SESSION_EXEC_POLL_INTERVAL: Duration = Duration::from_millis(25);
const SESSION_EXEC_STOP_TIMEOUT: Duration = Duration::from_secs(2);
const SESSION_EXEC_MARKER: &str = "exec.json";
const SESSION_EXEC_MARKER_SCHEMA: &str = "vat.k8s.session.exec.v1";

/// Dispatch one leased-session host command. In the normal managed path every
/// invocation owns a private process group and is bounded by the remaining
/// lease (or a smaller explicit timeout). If the VAT parent crashes, recovery
/// deliberately retains an unauthenticated live group rather than claim it
/// was terminated. In JSON mode every successful child invocation produces
/// exactly one VAT-owned result document; raw child output is never replayed
/// around it.
pub(super) fn run(
    id: String,
    command: Vec<String>,
    json: bool,
    timeout_seconds: Option<u64>,
) -> Result<ExitCode> {
    let (program, program_args) = command
        .split_first()
        .context("vat k8s session exec requires a host command after --")?;
    let requested_timeout = timeout_seconds.map(parse_timeout_seconds).transpose()?;
    let prepared = prepare(&id)?;

    if json {
        run_json(prepared, program, program_args, requested_timeout)
    } else {
        run_text(prepared, program, program_args, requested_timeout)
    }
}

fn parse_timeout_seconds(seconds: u64) -> Result<Duration> {
    if !(1..=MAX_SESSION_EXEC_TIMEOUT_SECONDS).contains(&seconds) {
        bail!(
            "VAT's K3s session exec --timeout must be between 1 and {MAX_SESSION_EXEC_TIMEOUT_SECONDS} seconds"
        );
    }
    Ok(Duration::from_secs(seconds))
}

/// Keep the existing exact identity, private credential, and recovery checks
/// inside the private operation lock. A lease is rechecked after the lock and
/// recovery work, then again at the precise spawn boundary below: an API probe
/// can consume enough time for a bounded session to expire.
fn prepare(id: &str) -> Result<PreparedSessionExec> {
    let session = read_active_session(id)?;
    require_active_session_lease(&session)?;
    let operation = super::port_forward::SessionOperationLock::acquire(&session)?;
    require_active_session_lease(&session)?;
    super::port_forward::reconcile(&session, &operation)?;
    reconcile(&session, &operation)?;
    require_active_session_lease(&session)?;

    ensure_apple_container()?;
    let backing = validate_active_session_backing(&session)?;
    let credentials = SessionKubeconfig::open(&session.directory);
    credentials.validate().map_err(|_| {
        anyhow::anyhow!(
            "K3s session {} has unavailable or unsafe private credentials; VAT did not start a host command",
            session.metadata.id,
        )
    })?;
    let endpoint = backing.api_endpoint()?;
    verify_host_api(&credentials, &endpoint).map_err(|_| {
        anyhow::anyhow!(
            "K3s session {} API verification did not reach its exact owned K3s API; VAT did not start a host command",
            session.metadata.id,
        )
    })?;

    Ok(PreparedSessionExec {
        session,
        credentials,
        endpoint,
        operation,
    })
}

fn run_text(
    prepared: PreparedSessionExec,
    program: &str,
    program_args: &[String],
    requested_timeout: Option<Duration>,
) -> Result<ExitCode> {
    let PreparedSessionExec {
        session,
        credentials,
        endpoint,
        operation: _operation,
    } = prepared;
    let cancellation = SessionExecCancellation::new()?;
    let mut spawned = spawn_host_command(
        &session,
        program,
        program_args,
        &credentials,
        &endpoint,
        requested_timeout,
        &cancellation,
        Stdio::inherit(),
        Stdio::inherit(),
    )?;
    let outcome =
        wait_for_host_command(&mut spawned.child, program, spawned.deadline, &cancellation);
    let marker_cleanup = match &outcome {
        Ok(_) => spawned.marker.remove_after_group_absent(),
        Err(_) => Ok(()),
    };
    drop(cancellation);

    match (outcome, marker_cleanup) {
        (_, Err(cleanup_error)) => Err(cleanup_error).context(format!(
            "K3s session {} host command stopped but VAT retained its recovery marker",
            session.metadata.id,
        )),
        (Err(error), _) => Err(error),
        (Ok(SessionExecOutcome::Exited(status)), Ok(())) => {
            super::print_active_session_exec_result(&session, status);
            Ok(exit_code(status))
        }
        (Ok(SessionExecOutcome::TimedOut), Ok(())) => {
            let bound = if requested_timeout.is_some() {
                "--timeout"
            } else {
                "remaining lease TTL"
            };
            bail!(
                "K3s session {} host command exceeded its {bound}; VAT confirmed its owned process group stopped and emitted no terminal success record",
                session.metadata.id
            )
        }
        (Ok(SessionExecOutcome::Interrupted(signal)), Ok(())) => {
            Ok(ExitCode::from((128 + signal).clamp(0, 255) as u8))
        }
    }
}

fn run_json(
    prepared: PreparedSessionExec,
    program: &str,
    program_args: &[String],
    requested_timeout: Option<Duration>,
) -> Result<ExitCode> {
    let PreparedSessionExec {
        session,
        credentials,
        endpoint,
        operation: _operation,
    } = prepared;
    let cancellation = SessionExecCancellation::new()?;
    let mut spawned = spawn_host_command(
        &session,
        program,
        program_args,
        &credentials,
        &endpoint,
        requested_timeout,
        &cancellation,
        Stdio::piped(),
        Stdio::piped(),
    )?;
    let outcome = capture_child(&mut spawned.child, program, spawned.deadline, &cancellation);
    let marker_cleanup = match &outcome {
        Ok(_) => spawned.marker.remove_after_group_absent(),
        Err(_) => Ok(()),
    };
    drop(cancellation);

    match (outcome, marker_cleanup) {
        (_, Err(cleanup_error)) => Err(cleanup_error).context(format!(
            "K3s session {} JSON host command stopped but VAT retained its recovery marker",
            session.metadata.id,
        )),
        (Err(error), _) => Err(error),
        (Ok(CapturedSessionExecOutcome::Exited(snapshot)), Ok(())) => {
            println!("{}", json_result(&session, &snapshot));
            Ok(exit_code(snapshot.status))
        }
        (Ok(CapturedSessionExecOutcome::TimedOut), Ok(())) => {
            let bound = if requested_timeout.is_some() {
                "--timeout"
            } else {
                "remaining lease TTL"
            };
            bail!(
                "K3s session {} JSON host command exceeded its {bound}; VAT confirmed its owned process group stopped and emitted no partial JSON result",
                session.metadata.id
            )
        }
        (Ok(CapturedSessionExecOutcome::Interrupted(signal)), Ok(())) => {
            Ok(ExitCode::from((128 + signal).clamp(0, 255) as u8))
        }
    }
}

struct PreparedSessionExec {
    session: ActiveSession,
    credentials: SessionKubeconfig,
    endpoint: String,
    operation: super::port_forward::SessionOperationLock,
}

struct SpawnedHostCommand {
    child: Child,
    deadline: SessionExecDeadline,
    marker: SessionExecMarkerStorage,
}

#[derive(Clone, Copy)]
struct SessionExecDeadline {
    deadline: Instant,
}

impl SessionExecDeadline {
    fn begin(session: &ActiveSession, requested: Option<Duration>) -> Result<Self> {
        let remaining_ms = session
            .metadata
            .expires_unix_ms
            .checked_sub(super::unix_millis())
            .context("K3s session lease expired before its bounded host command could start")?;
        let remaining_ms = u64::try_from(remaining_ms)
            .context("K3s session remaining lease duration exceeds supported timeout range")?;
        let remaining = Duration::from_millis(remaining_ms);
        let requested = requested.unwrap_or(remaining);
        if requested > remaining {
            bail!(
                "K3s session {} has only {}ms remaining; --timeout {}s must not exceed its remaining lease TTL",
                session.metadata.id,
                remaining.as_millis(),
                requested.as_secs()
            );
        }
        let deadline = Instant::now()
            .checked_add(requested)
            .context("K3s session exec timeout deadline overflowed")?;
        Ok(Self { deadline })
    }
}

/// Spawn only after all slow proof work completes, then recheck the lease at
/// the last possible point before a child can inherit the private kubeconfig.
fn spawn_host_command(
    session: &ActiveSession,
    program: &str,
    args: &[String],
    credentials: &SessionKubeconfig,
    endpoint: &str,
    requested_timeout: Option<Duration>,
    cancellation: &SessionExecCancellation,
    stdout: Stdio,
    stderr: Stdio,
) -> Result<SpawnedHostCommand> {
    require_active_session_lease(session)?;
    if let Some(signal) = cancellation.received() {
        bail!("received signal {signal} before K3s session exec host command spawn");
    }
    let deadline = SessionExecDeadline::begin(session, requested_timeout)?;
    let mut command = k8s_host_command(program)?;
    command
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(stdout)
        .stderr(stderr);
    for key in sensitive_environment() {
        command.env_remove(key);
    }
    for (key, value) in credentials.environment(endpoint) {
        command.env(key, value);
    }
    set_owned_process_group(&mut command);
    require_session_exec_deadline(deadline)?;
    let mut marker = SessionExecMarkerStorage::create(session)?;
    if let Err(error) = require_active_session_lease(session) {
        return abort_unstarted_marker(marker, error);
    }
    if let Err(error) = require_session_exec_deadline(deadline) {
        return abort_unstarted_marker(marker, error);
    }
    if let Some(signal) = cancellation.received() {
        return abort_unstarted_marker(
            marker,
            anyhow::anyhow!(
                "received signal {signal} immediately before K3s session exec host command spawn"
            ),
        );
    }
    let child = command
        .spawn()
        .with_context(|| format!("run host command {program:?} against leased ephemeral K3s"));
    let mut child = match child {
        Ok(child) => child,
        Err(error) => return abort_unstarted_marker(marker, error),
    };
    if let Err(error) = marker.record_running(child.id()) {
        let cleanup = stop_owned_process_group(&mut child, "K3s session exec host command");
        if cleanup.is_ok() {
            let _ = marker.remove_after_group_absent();
        }
        return Err(error).context(
            "K3s session exec host command started but VAT could not record its private process group; no success is claimed",
        );
    }
    Ok(SpawnedHostCommand {
        child,
        deadline,
        marker,
    })
}

fn require_session_exec_deadline(deadline: SessionExecDeadline) -> Result<()> {
    if Instant::now() >= deadline.deadline {
        bail!("K3s session exec deadline elapsed before its host command could start");
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BoundedStream {
    text: String,
    truncated: bool,
    utf8_lossy: bool,
}

struct SessionExecSnapshot {
    status: ExitStatus,
    stdout: BoundedStream,
    stderr: BoundedStream,
}

enum SessionExecOutcome {
    Exited(ExitStatus),
    TimedOut,
    Interrupted(i32),
}

enum CapturedSessionExecOutcome {
    Exited(SessionExecSnapshot),
    TimedOut,
    Interrupted(i32),
}

/// Drain stdout and stderr concurrently. Retaining only bounded suffixes
/// keeps an output-heavy command from deadlocking on one full pipe or turning
/// a single agent response into unbounded memory use. The readers are
/// deliberately detached on timeout/interruption: a descendant outside VAT's
/// owned process group can retain a pipe, but must never extend this bounded
/// session operation or cause a partial JSON result.
fn capture_child(
    child: &mut Child,
    program: &str,
    deadline: SessionExecDeadline,
    cancellation: &SessionExecCancellation,
) -> Result<CapturedSessionExecOutcome> {
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            return cleanup_after_capture_setup_failure(
                child,
                anyhow::anyhow!("K3s session JSON exec child did not expose stdout capture"),
            );
        }
    };
    let stderr = match child.stderr.take() {
        Some(stderr) => stderr,
        None => {
            return cleanup_after_capture_setup_failure(
                child,
                anyhow::anyhow!("K3s session JSON exec child did not expose stderr capture"),
            );
        }
    };
    let stdout_reader = match start_bounded_reader(stdout, "stdout") {
        Ok(reader) => reader,
        Err(error) => return cleanup_after_capture_setup_failure(child, error),
    };
    let stderr_reader = match start_bounded_reader(stderr, "stderr") {
        Ok(reader) => reader,
        Err(error) => return cleanup_after_capture_setup_failure(child, error),
    };

    match wait_for_host_command(child, program, deadline, cancellation)? {
        SessionExecOutcome::TimedOut => Ok(CapturedSessionExecOutcome::TimedOut),
        SessionExecOutcome::Interrupted(signal) => {
            Ok(CapturedSessionExecOutcome::Interrupted(signal))
        }
        SessionExecOutcome::Exited(status) => {
            let Some(stdout) = receive_reader_until(&stdout_reader, deadline, "stdout")? else {
                return Ok(CapturedSessionExecOutcome::TimedOut);
            };
            let Some(stderr) = receive_reader_until(&stderr_reader, deadline, "stderr")? else {
                return Ok(CapturedSessionExecOutcome::TimedOut);
            };
            Ok(CapturedSessionExecOutcome::Exited(SessionExecSnapshot {
                status,
                stdout,
                stderr,
            }))
        }
    }
}

fn start_bounded_reader<R>(
    reader: R,
    stream: &'static str,
) -> Result<Receiver<Result<BoundedStream>>>
where
    R: Read + Send + 'static,
{
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::Builder::new()
        .name(format!("vat-k8s-session-exec-{stream}"))
        .spawn(move || {
            let _ = sender.send(bounded_stream(reader));
        })
        .with_context(|| format!("start bounded K3s session exec {stream} reader"))?;
    Ok(receiver)
}

fn receive_reader_until(
    reader: &Receiver<Result<BoundedStream>>,
    deadline: SessionExecDeadline,
    stream: &str,
) -> Result<Option<BoundedStream>> {
    let remaining = deadline.deadline.saturating_duration_since(Instant::now());
    match reader.recv_timeout(remaining) {
        Ok(result) => result
            .with_context(|| format!("capture bounded K3s session exec {stream}"))
            .map(Some),
        Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            bail!("bounded K3s session exec {stream} reader stopped before returning its capture")
        }
    }
}

fn cleanup_after_capture_setup_failure<T>(child: &mut Child, error: anyhow::Error) -> Result<T> {
    match stop_owned_process_group(child, "K3s session exec JSON host command") {
        Ok(_) => Err(error),
        Err(cleanup_error) => Err(cleanup_error).context(format!(
            "K3s session exec JSON capture setup failed and VAT could not confirm process-group cleanup: {error:#}"
        )),
    }
}

fn wait_for_host_command(
    child: &mut Child,
    program: &str,
    deadline: SessionExecDeadline,
    cancellation: &SessionExecCancellation,
) -> Result<SessionExecOutcome> {
    let label = "K3s session exec host command";
    loop {
        let exited = child_has_exited_without_reap(child).with_context(|| {
            format!("observe host command {program:?} without reaping its process-group leader")
        });
        let exited = match exited {
            Ok(exited) => exited,
            Err(error) => return cleanup_after_wait_failure(child, label, error),
        };
        if exited {
            return stop_owned_process_group(child, label).map(SessionExecOutcome::Exited);
        }
        if let Some(signal) = cancellation.received() {
            stop_owned_process_group(child, label)?;
            return Ok(SessionExecOutcome::Interrupted(signal));
        }
        if Instant::now() >= deadline.deadline {
            stop_owned_process_group(child, label)?;
            return Ok(SessionExecOutcome::TimedOut);
        }
        let sleep = SESSION_EXEC_POLL_INTERVAL
            .min(deadline.deadline.saturating_duration_since(Instant::now()));
        if sleep.is_zero() {
            continue;
        }
        thread::sleep(sleep);
    }
}

fn cleanup_after_wait_failure<T>(
    child: &mut Child,
    label: &str,
    error: anyhow::Error,
) -> Result<T> {
    match stop_owned_process_group(child, label) {
        Ok(_) => Err(error),
        Err(cleanup_error) => Err(cleanup_error).context(format!(
            "could not confirm {label} cleanup after a wait failure: {error:#}"
        )),
    }
}

fn set_owned_process_group(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProcessGroupSignalOutcome {
    DeliveredOrGone,
    PermissionPartial,
}

/// Stop a direct, still-owned group leader. Keeping it unreaped until after
/// group signalling pins the numeric PGID, so VAT never signals a recycled
/// group while an ordinary background descendant remains in the owned group.
fn stop_owned_process_group(child: &mut Child, label: &str) -> Result<ExitStatus> {
    let pgid = child.id();
    let term = signal_process_group_outcome(pgid, libc::SIGTERM, label)?;
    thread::sleep(SESSION_EXEC_POLL_INTERVAL);
    let kill = signal_process_group_outcome(pgid, libc::SIGKILL, label)?;
    let permission_partial = matches!(term, ProcessGroupSignalOutcome::PermissionPartial)
        || matches!(kill, ProcessGroupSignalOutcome::PermissionPartial);
    if permission_partial {
        kill_owned_child_if_running(child, label)?;
    }

    let deadline = Instant::now() + SESSION_EXEC_STOP_TIMEOUT;
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .context("poll K3s session exec host child after KILL")?
        {
            break status;
        }
        if Instant::now() >= deadline {
            if permission_partial {
                bail!(
                    "{label} did not exit after a partially-permitted group TERM/KILL and direct owned-child KILL"
                );
            }
            bail!("{label} did not exit after TERM and KILL");
        }
        thread::sleep(SESSION_EXEC_POLL_INTERVAL);
    };

    let group_deadline = Instant::now() + SESSION_EXEC_STOP_TIMEOUT;
    while process_group_exists(pgid)? && Instant::now() < group_deadline {
        thread::sleep(SESSION_EXEC_POLL_INTERVAL);
    }
    if !process_group_exists(pgid)? {
        return Ok(status);
    }
    if permission_partial {
        bail!(
            "{label} direct child exited after partially-permitted group TERM/KILL and direct owned-child KILL, but process group {pgid} remains visible"
        );
    }
    bail!("{label} process group {pgid} remains after TERM and KILL");
}

fn kill_owned_child_if_running(child: &mut Child, label: &str) -> Result<()> {
    if child_has_exited_without_reap(child)? {
        return Ok(());
    }
    match child.kill() {
        Ok(()) => Ok(()),
        Err(error) if error.raw_os_error() == Some(libc::ESRCH) => Ok(()),
        Err(error) => Err(error).with_context(|| format!("send direct KILL to {label}")),
    }
}

fn child_has_exited_without_reap(child: &Child) -> Result<bool> {
    let mut info = unsafe { std::mem::zeroed::<libc::siginfo_t>() };
    let result = unsafe {
        libc::waitid(
            libc::P_PID,
            child.id() as libc::id_t,
            &mut info,
            libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
        )
    };
    if result != 0 {
        return Err(std::io::Error::last_os_error())
            .context("observe K3s session exec exit without reaping its process-group leader");
    }
    Ok(unsafe { info.si_pid() } != 0)
}

fn process_group_exists(pgid: u32) -> Result<bool> {
    let result = unsafe { libc::kill(-(pgid as libc::pid_t), 0) };
    if result == 0 {
        return Ok(true);
    }
    let error = std::io::Error::last_os_error();
    match error.raw_os_error() {
        Some(libc::ESRCH) => Ok(false),
        Some(libc::EPERM) => Ok(true),
        _ => Err(error).with_context(|| format!("inspect K3s session exec process group {pgid}")),
    }
}

fn signal_process_group_outcome(
    pgid: u32,
    signal: i32,
    label: &str,
) -> Result<ProcessGroupSignalOutcome> {
    let result = unsafe { libc::kill(-(pgid as libc::pid_t), signal) };
    if result == 0 {
        return Ok(ProcessGroupSignalOutcome::DeliveredOrGone);
    }
    let error = std::io::Error::last_os_error();
    match error.raw_os_error() {
        Some(libc::ESRCH) => Ok(ProcessGroupSignalOutcome::DeliveredOrGone),
        Some(libc::EPERM) => Ok(ProcessGroupSignalOutcome::PermissionPartial),
        _ => Err(error)
            .with_context(|| format!("send signal {signal} to {label} process group {pgid}")),
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Serialize)]
struct SessionExecMarker {
    schema: String,
    session_id: String,
    owner_pid: u32,
    state: String,
    #[serde(default)]
    pgid: Option<u32>,
}

struct SessionExecMarkerStorage {
    path: PathBuf,
    marker: SessionExecMarker,
}

impl SessionExecMarkerStorage {
    fn create(session: &ActiveSession) -> Result<Self> {
        let path = session_exec_marker_path(session);
        let marker = SessionExecMarker {
            schema: SESSION_EXEC_MARKER_SCHEMA.to_string(),
            session_id: session.metadata.id.clone(),
            owner_pid: std::process::id(),
            state: "starting".to_string(),
            pgid: None,
        };
        write_new_marker(&path, &marker)
            .with_context(|| format!("write private K3s session exec marker {}", path.display()))?;
        Ok(Self { path, marker })
    }

    fn record_running(&mut self, pgid: u32) -> Result<()> {
        self.marker.state = "running".to_string();
        self.marker.pgid = Some(pgid);
        super::replace_session_marker(&self.path, &self.marker)
            .context("atomically record K3s session exec process group")
    }

    fn remove_unstarted(self) -> Result<()> {
        if self.marker.state != "starting" || self.marker.pgid.is_some() {
            bail!("K3s session exec marker is not an unstarted private record");
        }
        remove_exact_exec_marker(&self.path, &self.marker)
    }

    fn remove_after_group_absent(&self) -> Result<()> {
        let pgid = self
            .marker
            .pgid
            .filter(|pgid| *pgid != 0)
            .context("running K3s session exec marker is missing its process group")?;
        if process_group_exists(pgid)? {
            bail!(
                "K3s session exec private process group {pgid} remains visible; recovery marker is retained"
            );
        }
        remove_exact_exec_marker(&self.path, &self.marker)
    }
}

fn abort_unstarted_marker(
    marker: SessionExecMarkerStorage,
    error: anyhow::Error,
) -> Result<SpawnedHostCommand> {
    match marker.remove_unstarted() {
        Ok(()) => Err(error),
        Err(cleanup_error) => Err(cleanup_error).context(format!(
            "K3s session exec did not start a host command but could not remove its recovery marker: {error:#}"
        )),
    }
}

/// Reconcile a durable exec marker after the previous VAT parent has died.
/// Unlike the port-forward marker, an arbitrary host command has no stable,
/// token-bearing argv shape that lets us authenticate a recycled PID. A live
/// recorded group is therefore never signalled from recovery; it blocks all
/// lifecycle operations until a human verifies it. An absent recorded group is
/// safe to unlink because no process control is attempted.
pub(super) fn reconcile(
    session: &ActiveSession,
    _operation: &super::port_forward::SessionOperationLock,
) -> Result<()> {
    let path = session_exec_marker_path(session);
    match fs::symlink_metadata(&path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!(
                    "leased K3s session exec marker {} is not a real private file; VAT will not reuse or delete this session",
                    path.display()
                );
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(error).with_context(|| {
                format!("inspect leased K3s session exec marker {}", path.display())
            });
        }
    }
    let marker = read_exec_marker(&path, &session.metadata.id)?;
    match marker.state.as_str() {
        "starting" => bail!(
            "K3s session {} has an unconfirmed exec recovery marker; VAT will retain the session rather than assume a credentialed host command did not start",
            session.metadata.id,
        ),
        "running" => {
            let pgid = marker
                .pgid
                .filter(|pgid| *pgid != 0)
                .context("running K3s session exec marker is missing its process group")?;
            if process_group_exists(pgid)? {
                bail!(
                    "K3s session {} records a live exec process group {pgid}, but VAT cannot authenticate an arbitrary recovered host command; marker is retained and no signal was sent",
                    session.metadata.id,
                );
            }
            remove_exact_exec_marker(&path, &marker)
        }
        _ => unreachable!("exec marker state was validated"),
    }
}

fn session_exec_marker_path(session: &ActiveSession) -> PathBuf {
    session.directory.join(SESSION_EXEC_MARKER)
}

fn read_exec_marker(path: &Path, session_id: &str) -> Result<SessionExecMarker> {
    require_private_file(path, "leased K3s session exec marker")?;
    let bytes = fs::read(path)
        .with_context(|| format!("read leased K3s session exec marker {}", path.display()))?;
    let marker: SessionExecMarker = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse leased K3s session exec marker {}", path.display()))?;
    validate_exec_marker(session_id, &marker)?;
    Ok(marker)
}

fn validate_exec_marker(session_id: &str, marker: &SessionExecMarker) -> Result<()> {
    if marker.schema != SESSION_EXEC_MARKER_SCHEMA
        || marker.session_id != session_id
        || marker.owner_pid == 0
        || !matches!(marker.state.as_str(), "starting" | "running")
    {
        bail!("leased K3s session exec marker is not a valid VAT-owned record");
    }
    match marker.state.as_str() {
        "starting" if marker.pgid.is_none() => Ok(()),
        "running" if marker.pgid.is_some_and(|pgid| pgid != 0) => Ok(()),
        "starting" => bail!("starting K3s session exec marker has an impossible process group"),
        "running" => bail!("running K3s session exec marker is missing its process group"),
        _ => unreachable!("exec marker state was validated"),
    }
}

fn remove_exact_exec_marker(path: &Path, expected: &SessionExecMarker) -> Result<()> {
    let observed = read_exec_marker(path, &expected.session_id)?;
    if &observed != expected {
        bail!("K3s session exec recovery marker changed while VAT owned it; marker is retained");
    }
    fs::remove_file(path)
        .with_context(|| format!("remove K3s session exec marker {}", path.display()))?;
    Ok(())
}

struct SessionExecCancellation {
    receiver: Receiver<i32>,
    last_signal: Arc<AtomicI32>,
    handle: SignalHandle,
    thread: Option<JoinHandle<()>>,
}

impl SessionExecCancellation {
    fn new() -> Result<Self> {
        let mut signals = Signals::new([SIGINT, SIGTERM])
            .context("install scoped K3s session exec cancellation handlers")?;
        let handle = signals.handle();
        let (sender, receiver) = mpsc::channel();
        let last_signal = Arc::new(AtomicI32::new(0));
        let thread_last_signal = Arc::clone(&last_signal);
        let thread = thread::spawn(move || {
            for signal in signals.forever() {
                thread_last_signal.store(signal, Ordering::Relaxed);
                if sender.send(signal).is_err() {
                    return;
                }
            }
        });
        Ok(Self {
            receiver,
            last_signal,
            handle,
            thread: Some(thread),
        })
    }

    fn received(&self) -> Option<i32> {
        match self.receiver.try_recv() {
            Ok(signal) => {
                self.last_signal.store(signal, Ordering::Relaxed);
                Some(signal)
            }
            Err(_) => {
                let signal = self.last_signal.load(Ordering::Relaxed);
                (signal != 0).then_some(signal)
            }
        }
    }
}

impl Drop for SessionExecCancellation {
    fn drop(&mut self) {
        self.handle.close();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn bounded_stream(mut reader: impl Read) -> Result<BoundedStream> {
    let mut retained = Vec::with_capacity(MAX_SESSION_EXEC_STREAM_CAPTURE_BYTES as usize);
    let mut chunk = [0_u8; 8 * 1024];
    let mut truncated = false;
    loop {
        let read = reader
            .read(&mut chunk)
            .context("read K3s session exec child stream")?;
        if read == 0 {
            break;
        }
        let bytes = &chunk[..read];
        if bytes.len() >= MAX_SESSION_EXEC_STREAM_CAPTURE_BYTES as usize {
            retained.clear();
            retained.extend_from_slice(
                &bytes[bytes.len() - MAX_SESSION_EXEC_STREAM_CAPTURE_BYTES as usize..],
            );
            truncated = true;
            continue;
        }
        let overflow = retained
            .len()
            .saturating_add(bytes.len())
            .saturating_sub(MAX_SESSION_EXEC_STREAM_CAPTURE_BYTES as usize);
        if overflow > 0 {
            retained.drain(..overflow);
            truncated = true;
        }
        retained.extend_from_slice(bytes);
    }
    let (decoded, utf8_lossy) = match String::from_utf8_lossy(&retained) {
        std::borrow::Cow::Borrowed(text) => (text.to_string(), false),
        std::borrow::Cow::Owned(text) => (text, true),
    };
    let (text, json_truncated) = cap_to_json_string_value(decoded)?;
    Ok(BoundedStream {
        text,
        truncated: truncated || json_truncated,
        utf8_lossy,
    })
}

/// JSON escaping can expand a valid UTF-8 string well beyond its raw bytes.
/// Retain the newest character-aligned suffix whose actual serde JSON string
/// encoding fits the advertised cap.
fn cap_to_json_string_value(text: String) -> Result<(String, bool)> {
    if serialized_json_string_len(&text)? <= MAX_SESSION_EXEC_JSON_STREAM_VALUE_BYTES {
        return Ok((text, false));
    }

    let mut boundaries = text
        .char_indices()
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    boundaries.push(text.len());
    let mut lower = 0;
    let mut upper = boundaries.len() - 1;
    while lower < upper {
        let middle = lower + (upper - lower) / 2;
        let suffix = &text[boundaries[middle]..];
        if serialized_json_string_len(suffix)? <= MAX_SESSION_EXEC_JSON_STREAM_VALUE_BYTES {
            upper = middle;
        } else {
            lower = middle + 1;
        }
    }
    let suffix = text[boundaries[lower]..].to_string();
    debug_assert!(serialized_json_string_len(&suffix)
        .is_ok_and(|length| length <= MAX_SESSION_EXEC_JSON_STREAM_VALUE_BYTES));
    Ok((suffix, true))
}

fn serialized_json_string_len(text: &str) -> Result<usize> {
    serde_json::to_vec(text)
        .map(|encoded| encoded.len())
        .context("serialize bounded K3s session exec stream")
}

fn json_result(session: &ActiveSession, snapshot: &SessionExecSnapshot) -> serde_json::Value {
    serde_json::json!({
        "schema": "vat.k8s.session.exec.v1",
        "format": "vat_json",
        "type": "vat_k8s_session_exec",
        "id": session.metadata.id,
        "state": "active",
        "child_exit_code": child_exit_code(snapshot.status),
        "stdout": snapshot.stdout.text,
        "stderr": snapshot.stderr.text,
        "stdout_truncated": snapshot.stdout.truncated,
        "stderr_truncated": snapshot.stderr.truncated,
        "stdout_utf8_lossy": snapshot.stdout.utf8_lossy,
        "stderr_utf8_lossy": snapshot.stderr.utf8_lossy,
        "api_verified": true,
        "runtime_invoked": true,
        "session_record_mutated": false,
        "next": format!("vat k8s session status --verify-api {}", session.metadata.id),
    })
}

fn child_exit_code(status: ExitStatus) -> i32 {
    status
        .code()
        .or_else(|| status.signal().map(|signal| 128 + signal))
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_stream_preserves_the_latest_serializable_agent_snapshot() {
        let mut bytes = vec![0xff; MAX_SESSION_EXEC_STREAM_CAPTURE_BYTES as usize + 16];
        bytes.extend_from_slice(&[0, b'\n', b'e', b'n', b'd']);
        let snapshot = bounded_stream(std::io::Cursor::new(bytes))
            .expect("drain bounded K3s session exec stream");
        assert!(snapshot.truncated);
        assert!(snapshot.utf8_lossy);
        assert!(snapshot.text.ends_with("\nend"));
        assert!(
            serialized_json_string_len(&snapshot.text).expect("serialize bounded text")
                <= MAX_SESSION_EXEC_JSON_STREAM_VALUE_BYTES,
            "lossy/control expansion must stay within the public JSON stream budget"
        );
    }
}
// HANDWRITE-END
