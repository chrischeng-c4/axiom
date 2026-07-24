// HANDWRITE-BEGIN gap="missing-generator:ephemeral-k8s-session-port-forward" tracker="#1693" reason="A foreground Service tunnel coordinates private credentials, an owned process group, exact cleanup, and bounded host-child output; no generator owns that lifecycle."
//! Foreground, loopback-only host access to a Service in a leased K3s session.
//!
//! A port-forward is deliberately not a daemon. VAT owns it only while one
//! host child runs, keeps Kubernetes credentials on the kubectl side of the
//! boundary, and records enough non-secret state to fail closed after an
//! interrupted parent process.

use std::ffi::{CString, OsStr};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, ExitStatus, Stdio};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use signal_hook::consts::signal::{SIGINT, SIGTERM};
use signal_hook::iterator::{Handle as SignalHandle, Signals};

use super::{
    active_session_expired, ensure_apple_container, exit_code, require_active_session_lease,
    require_private_directory, require_private_file, restrict_dir, run_bounded,
    sensitive_environment, strings, validate_active_session_backing, verify_host_api,
    write_new_marker, ActiveSession, ActiveSessionPortForwardArgs, KubeconfigAccess,
    SessionKubeconfig,
};

const PORT_FORWARD_SCHEMA: &str = "vat.k8s.session.port-forward.v2";
const LEGACY_PORT_FORWARD_SCHEMA: &str = "vat.k8s.session.port-forward.v1";
const PORT_FORWARD_MARKER: &str = "port-forward.json";
const PORT_FORWARD_DIRECTORY: &str = "port-forward";
const PORT_FORWARD_READY_TIMEOUT: Duration = Duration::from_secs(30);
const PORT_FORWARD_STOP_TIMEOUT: Duration = Duration::from_secs(5);
const PORT_FORWARD_POLL_INTERVAL: Duration = Duration::from_millis(100);
const PORT_FORWARD_LOG_LIMIT: usize = 4_096;
const PORT_FORWARD_TOKEN_PREFIX: &str = "vat-pf-";
const PORT_FORWARD_TOKEN_BYTES: usize = 16;

/// An exclusive lock for one leased-session operation.
///
/// The 0600 lock file is retained after a process exits, while flock
/// serializes active operations without a background daemon.
pub(super) struct SessionOperationLock {
    file: File,
}

impl SessionOperationLock {
    pub(super) fn acquire(session: &ActiveSession) -> Result<Self> {
        let path = session.directory.join("operation.lock");
        let file = open_private_lock(&path)?;

        let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        if result != 0 {
            let error = std::io::Error::last_os_error();
            if error
                .raw_os_error()
                .is_some_and(|code| code == libc::EWOULDBLOCK || code == libc::EAGAIN)
            {
                bail!(
                    "K3s session {} is busy with another VAT operation; wait for it to finish and run vat k8s session status {}",
                    session.metadata.id,
                    session.metadata.id,
                );
            }
            return Err(error)
                .with_context(|| format!("lock leased K3s operation {}", path.display()));
        }

        Ok(Self { file })
    }
}

impl Drop for SessionOperationLock {
    fn drop(&mut self) {
        unsafe {
            libc::flock(self.file.as_raw_fd(), libc::LOCK_UN);
        }
    }
}

fn open_private_lock(path: &Path) -> Result<File> {
    let path = CString::new(path.as_os_str().as_bytes())
        .context("leased K3s operation lock path contains an unexpected NUL byte")?;
    // The lock must never survive into kubectl or the untrusted foreground
    // host child. If the VAT parent is killed, an inherited flock would make
    // the next invocation fail "busy" before it could reconcile the marker.
    let create_flags =
        libc::O_RDWR | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC;
    let mut created = false;
    let descriptor = unsafe { libc::open(path.as_ptr(), create_flags, 0o600) };
    let descriptor = if descriptor >= 0 {
        created = true;
        descriptor
    } else {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::EEXIST) {
            return Err(error).with_context(|| "create private leased K3s operation lock");
        }
        let descriptor = unsafe {
            libc::open(
                path.as_ptr(),
                libc::O_RDWR | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if descriptor < 0 {
            return Err(std::io::Error::last_os_error()).with_context(|| {
                "open existing private leased K3s operation lock without following symlinks"
            });
        }
        descriptor
    };
    // SAFETY: libc::open returned a live owned descriptor above.
    let file = unsafe { File::from_raw_fd(descriptor) };
    if created {
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .context("restrict newly created leased K3s operation lock")?;
    }
    let metadata = file
        .metadata()
        .context("inspect opened leased K3s operation lock")?;
    if !metadata.is_file() {
        bail!("leased K3s operation lock is not a regular file");
    }
    if metadata.permissions().mode() & 0o077 != 0 {
        bail!("leased K3s operation lock is not private (expected mode 0600)");
    }
    Ok(file)
}

#[derive(Debug, Clone)]
struct PortForwardRequest {
    json: bool,
    id: String,
    resource: String,
    namespace: String,
    remote_port: u16,
    requested_local_port: u16,
    command: Vec<String>,
}

impl TryFrom<ActiveSessionPortForwardArgs> for PortForwardRequest {
    type Error = anyhow::Error;

    fn try_from(args: ActiveSessionPortForwardArgs) -> Result<Self> {
        let resource = parse_service_resource(&args.resource)?;
        if !valid_dns_label(&args.namespace) {
            bail!(
                "invalid Kubernetes namespace {:?}; use one lowercase DNS label with internal hyphens only",
                args.namespace
            );
        }
        if args.remote_port == 0 {
            bail!("Kubernetes port-forward remote port must be in 1..=65535");
        }
        if args.command.is_empty() {
            bail!("vat k8s session port-forward run requires a host command after --");
        }
        Ok(Self {
            json: args.json,
            id: args.id,
            resource,
            namespace: args.namespace,
            remote_port: args.remote_port,
            requested_local_port: args.local_port,
            command: args.command,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortForwardMarker {
    schema: String,
    session_id: String,
    owner_pid: u32,
    token: String,
    state: String,
    resource: String,
    namespace: String,
    remote_port: u16,
    requested_local_port: u16,
    #[serde(default)]
    local_port: Option<u16>,
    kubectl: String,
    cache_dir: String,
    #[serde(default)]
    kubectl_pid: Option<u32>,
    #[serde(default)]
    pgid: Option<u32>,
    /// Once set, a host command may be running in kubectl's private process
    /// group. If the group leader cannot later be authenticated, recovery
    /// retains the marker rather than assume a host descendant stopped.
    #[serde(default)]
    host_started: bool,
}

/// The pre-CSPRNG marker format is intentionally read only for stale-storage
/// cleanup. Its token is not strong enough to authenticate a live process, so
/// VAT must never derive a signal target from one of these records.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyPortForwardMarker {
    schema: String,
    session_id: String,
    owner_pid: u32,
    token: String,
    state: String,
    cache_dir: String,
    #[serde(default)]
    pgid: Option<u32>,
}

enum StoredPortForwardMarker {
    Current(PortForwardMarker),
    Legacy(LegacyPortForwardMarker),
}

struct PortForwardStorage {
    marker_path: PathBuf,
    cache_directory: PathBuf,
    host_home: PathBuf,
    marker: PortForwardMarker,
}

impl PortForwardStorage {
    fn create(
        session: &ActiveSession,
        request: &PortForwardRequest,
        kubectl: &Path,
    ) -> Result<Self> {
        let token = fresh_token()?;
        if !valid_token(&token) {
            bail!("VAT generated an invalid private K3s port-forward token");
        }
        let root = session.directory.join(PORT_FORWARD_DIRECTORY);
        match fs::symlink_metadata(&root) {
            Ok(_) => require_private_directory(&root, "leased K3s port-forward directory")?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(&root).with_context(|| {
                    format!("create K3s port-forward directory {}", root.display())
                })?;
                restrict_dir(&root)?;
            }
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("inspect K3s port-forward directory {}", root.display())
                });
            }
        }
        let token_directory = root.join(&token);
        fs::create_dir(&token_directory).with_context(|| {
            format!(
                "create private K3s port-forward directory {}",
                token_directory.display()
            )
        })?;
        restrict_dir(&token_directory)?;
        let cache_directory = token_directory.join("cache");
        let host_home = token_directory.join("home");
        let preparation = (|| -> Result<()> {
            fs::create_dir(&cache_directory)
                .with_context(|| format!("create {}", cache_directory.display()))?;
            fs::create_dir(&host_home)
                .with_context(|| format!("create {}", host_home.display()))?;
            restrict_dir(&cache_directory)?;
            restrict_dir(&host_home)?;
            Ok(())
        })();
        if let Err(error) = preparation {
            let _ = fs::remove_dir_all(&token_directory);
            return Err(error);
        }

        let marker_path = marker_path(session);
        let kubectl = kubectl
            .to_str()
            .context("resolved kubectl path is not UTF-8")?
            .to_string();
        let cache_dir = cache_directory
            .to_str()
            .context("private K3s port-forward cache path is not UTF-8")?
            .to_string();
        let marker = PortForwardMarker {
            schema: PORT_FORWARD_SCHEMA.to_string(),
            session_id: session.metadata.id.clone(),
            owner_pid: std::process::id(),
            token,
            state: "starting".to_string(),
            resource: request.resource.clone(),
            namespace: request.namespace.clone(),
            remote_port: request.remote_port,
            requested_local_port: request.requested_local_port,
            local_port: None,
            kubectl,
            cache_dir,
            kubectl_pid: None,
            pgid: None,
            host_started: false,
        };
        if let Err(error) = write_new_marker(&marker_path, &marker) {
            let _ = fs::remove_dir_all(&token_directory);
            return Err(error).with_context(|| {
                format!("write K3s port-forward marker {}", marker_path.display())
            });
        }
        Ok(Self {
            marker_path,
            cache_directory,
            host_home,
            marker,
        })
    }

    fn update_marker(&self) -> Result<()> {
        super::replace_session_marker(&self.marker_path, &self.marker)
            .context("atomically update K3s port-forward recovery marker")
    }
}

/// Run one loopback Service forward and one foreground host child.
pub(super) fn run(args: ActiveSessionPortForwardArgs) -> Result<ExitCode> {
    let request = PortForwardRequest::try_from(args)?;
    if request.json {
        return run_json(request);
    }
    run_text(request)
}

/// The historic foreground path keeps inheriting host stdout/stderr exactly as
/// before. JSON capture is deliberately isolated so adding it cannot change a
/// human's existing tunnel invocation.
fn run_text(request: PortForwardRequest) -> Result<ExitCode> {
    let session = super::read_active_session(&request.id)?;
    require_active_session_lease(&session)?;
    let _operation = SessionOperationLock::acquire(&session)?;
    reconcile(&session, &_operation)?;
    super::session_exec::reconcile(&session, &_operation)?;
    ensure_apple_container()?;
    let backing = validate_active_session_backing(&session)?;
    let credentials = SessionKubeconfig::open(&session.directory);
    credentials.validate()?;
    let endpoint = backing.api_endpoint()?;
    verify_host_api(&credentials, &endpoint)?;

    let cancellation = SignalCancellation::new()?;
    let mut forward = match ActivePortForward::start(
        &session,
        &request,
        &credentials,
        &endpoint,
        &cancellation,
    ) {
        Ok(forward) => forward,
        Err(error) => {
            if let Some(signal) = cancellation.received() {
                return Ok(ExitCode::from((128 + signal).clamp(0, 255) as u8));
            }
            return Err(error);
        }
    };
    let host_result = run_host_child(&session, &request, &mut forward, &cancellation);
    let cleanup = forward.stop_and_confirm(&session);
    drop(cancellation);

    match (host_result, cleanup) {
        (_, Err(cleanup_error)) => Err(cleanup_error).context(format!(
            "VAT could not confirm cleanup of the loopback K3s port-forward for session {}; its non-secret recovery marker remains and must be reconciled before another session operation",
            session.metadata.id,
        )),
        (Err(error), Ok(())) => Err(error),
        (Ok(HostOutcome::Exited(status)), Ok(())) => {
            print_result(&session, &request, forward.local_port(), Some(status), None);
            Ok(exit_code(status))
        }
        (Ok(HostOutcome::Interrupted(signal)), Ok(())) => {
            print_result(&session, &request, forward.local_port(), None, Some(signal));
            Ok(ExitCode::from((128 + signal).clamp(0, 255) as u8))
        }
    }
}

/// Agent JSON mode owns exactly one terminal document, but only after the
/// shared kubectl/host process group and its recovery marker are confirmed
/// gone. Its public errors intentionally collapse VAT-owned setup, validation,
/// API, tunnel, and cleanup details: those chains can contain private session
/// paths or kubeconfig/cache material. Opaque credential-free child output is
/// instead preserved faithfully inside a successful result document.
fn run_json(request: PortForwardRequest) -> Result<ExitCode> {
    let id = request.id.clone();
    run_json_inner(request).map_err(|_| {
        anyhow::anyhow!(
            "K3s session {id} port-forward JSON execution did not produce a safe result; no host-output document was emitted. Run vat k8s session status {id}"
        )
    })
}

fn run_json_inner(request: PortForwardRequest) -> Result<ExitCode> {
    let session = super::read_active_session(&request.id)?;
    require_active_session_lease_json(&session)?;
    // Both foreground operation kinds retain this lock through group cleanup
    // and recovery-marker removal; allowing another lifecycle operation
    // mid-cleanup would weaken the durable recovery proof.
    let _operation = SessionOperationLock::acquire(&session)?;
    require_active_session_lease_json(&session)?;
    reconcile(&session, &_operation)?;
    super::session_exec::reconcile(&session, &_operation)?;
    require_active_session_lease_json(&session)?;
    ensure_apple_container()?;
    let backing = validate_active_session_backing(&session)?;
    let credentials = SessionKubeconfig::open(&session.directory);
    credentials.validate()?;
    let endpoint = backing.api_endpoint()?;
    verify_host_api(&credentials, &endpoint)?;
    // API verification may consume most or all of a short lease. This check
    // is intentionally silent: JSON mode must not emit a helper record before
    // the only permitted terminal document after cleanup.
    require_active_session_lease_json(&session)?;

    let cancellation = SignalCancellation::new()?;
    let mut forward = match ActivePortForward::start(
        &session,
        &request,
        &credentials,
        &endpoint,
        &cancellation,
    ) {
        Ok(forward) => forward,
        Err(error) => {
            if let Some(signal) = cancellation.received() {
                return Ok(ExitCode::from((128 + signal).clamp(0, 255) as u8));
            }
            return Err(error);
        }
    };
    let host_result = run_host_child_json(&session, &request, &mut forward, &cancellation);
    let cleanup = forward.stop_and_confirm(&session);
    drop(cancellation);

    match (host_result, cleanup) {
        // Do not join pipe readers or emit JSON when the exact owned group or
        // private marker/storage remains. A descendant could still hold the
        // pipes, and success must never race recovery.
        (_, Err(cleanup_error)) => Err(cleanup_error).context(format!(
            "VAT could not confirm cleanup of the loopback K3s port-forward for session {}; its non-secret recovery marker remains and must be reconciled before another session operation",
            session.metadata.id,
        )),
        (Err(error), Ok(())) => Err(error),
        (Ok(JsonHostOutcome::Exited { status, child }), Ok(())) => {
            let snapshot = child.finish_after_cleanup(status)?;
            print_json_result(&session, &request, forward.local_port(), &snapshot);
            Ok(exit_code(snapshot.status))
        }
        (Ok(JsonHostOutcome::Interrupted { signal, child }), Ok(())) => {
            child.discard_after_cleanup()?;
            Ok(ExitCode::from((128 + signal).clamp(0, 255) as u8))
        }
        (Ok(JsonHostOutcome::CaptureUnavailable { child }), Ok(())) => {
            child.discard_after_cleanup()?;
            bail!("K3s port-forward JSON capture was unavailable; no result document was emitted")
        }
    }
}

/// The general lease helper emits an error JSON record for text-oriented
/// callers. Agent JSON tunnel mode instead needs a silent predicate so a
/// failed invocation cannot produce stdout before its tunnel cleanup outcome.
fn require_active_session_lease_json(session: &ActiveSession) -> Result<()> {
    if session.metadata.state != "active" {
        bail!("K3s session is not active for JSON port-forward");
    }
    if active_session_expired(&session.metadata) {
        bail!("K3s session lease expired before JSON port-forward could continue");
    }
    Ok(())
}

enum JsonHostOutcome {
    Exited {
        status: ExitStatus,
        child: super::port_forward_json::CapturedHostChild,
    },
    Interrupted {
        signal: i32,
        child: super::port_forward_json::CapturedHostChild,
    },
    CaptureUnavailable {
        child: super::port_forward_json::CapturedHostChild,
    },
}

struct ActivePortForward {
    child: Child,
    storage: PortForwardStorage,
    logs: Receiver<ForwardLogLine>,
    readers: Vec<JoinHandle<()>>,
    diagnostics: String,
    // Once group shutdown starts, `stop_child_process_group` may reap the
    // direct kubectl leader. Do not retry a naked PGID from Drop afterwards:
    // an OS may have recycled it. The durable marker intentionally remains
    // when storage cleanup cannot be confirmed.
    group_shutdown_started: bool,
    stopped: bool,
}

impl ActivePortForward {
    fn start(
        session: &ActiveSession,
        request: &PortForwardRequest,
        credentials: &SessionKubeconfig,
        endpoint: &str,
        cancellation: &SignalCancellation,
    ) -> Result<Self> {
        let kubectl = resolve_kubectl()?;
        let mut storage = PortForwardStorage::create(session, request, &kubectl)?;
        if let Some(signal) = cancellation.received() {
            let cleanup = remove_storage(session, &storage.marker);
            return match cleanup {
                Ok(()) => bail!("received signal {signal} before K3s port-forward started"),
                Err(error) => Err(error).context(format!(
                    "received signal {signal} before K3s port-forward started and could not remove its recovery marker"
                )),
            };
        }

        let mapping = port_mapping(request.requested_local_port, request.remote_port);
        let kubeconfig = credentials.kubeconfig_path().to_string_lossy().into_owned();
        let cache = storage.cache_directory.to_string_lossy().into_owned();
        let mut command = Command::new(&storage.marker.kubectl);
        command
            .args([
                "--kubeconfig",
                &kubeconfig,
                "--cache-dir",
                &cache,
                "--request-timeout=20s",
                "--namespace",
                &request.namespace,
                "port-forward",
                "--address",
                "127.0.0.1",
                &request.resource,
                &mapping,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for key in sensitive_environment() {
            command.env_remove(key);
        }
        for (key, value) in credentials.environment(endpoint) {
            command.env(key, value);
        }
        set_process_group(&mut command);
        if request.json {
            // Storage/marker setup deliberately precedes this exact spawn
            // boundary so recovery can see a crash-safe starting record. If
            // the lease crossed its TTL while setup ran, remove that record
            // and never launch credentialed kubectl.
            let lease = require_active_session_lease_json(session);
            let signal = cancellation.received();
            if let Err(error) = lease {
                let cleanup = remove_storage(session, &storage.marker);
                return match cleanup {
                    Ok(()) => Err(error),
                    Err(cleanup_error) => Err(cleanup_error).context(
                        "K3s JSON port-forward lease expired before kubectl spawn and VAT could not remove its recovery marker",
                    ),
                };
            }
            if let Some(signal) = signal {
                let cleanup = remove_storage(session, &storage.marker);
                return match cleanup {
                    Ok(()) => bail!(
                        "received signal {signal} immediately before K3s JSON port-forward kubectl spawn"
                    ),
                    Err(cleanup_error) => Err(cleanup_error).context(format!(
                        "received signal {signal} before K3s JSON port-forward kubectl spawn and VAT could not remove its recovery marker"
                    )),
                };
            }
        }
        let mut child = command.spawn().with_context(|| {
            format!(
                "start loopback K3s port-forward {} in namespace {}",
                request.resource, request.namespace
            )
        })?;
        storage.marker.state = "running".to_string();
        storage.marker.kubectl_pid = Some(child.id());
        storage.marker.pgid = Some(child.id());
        if let Err(error) = storage.update_marker() {
            let stop = stop_child_process_group(&mut child, "kubectl port-forward");
            if stop.is_ok() {
                let _ = remove_storage(session, &storage.marker);
            }
            return Err(error).context(
                "kubectl port-forward started but VAT could not record its process identity; no success is claimed",
            );
        }

        let stdout = child
            .stdout
            .take()
            .context("kubectl port-forward stdout was not captured")?;
        let stderr = child
            .stderr
            .take()
            .context("kubectl port-forward stderr was not captured")?;
        let (sender, logs) = mpsc::channel();
        let readers = vec![
            spawn_log_reader(stdout, ForwardLogStream::Stdout, sender.clone()),
            spawn_log_reader(stderr, ForwardLogStream::Stderr, sender),
        ];
        let mut forward = Self {
            child,
            storage,
            logs,
            readers,
            diagnostics: String::new(),
            group_shutdown_started: false,
            stopped: false,
        };
        match forward.wait_for_ready(session, request, cancellation) {
            Ok(local_port) => forward.with_ready_local_port(local_port),
            Err(error) => match forward.stop_and_confirm(session) {
                Ok(()) => Err(error),
                Err(cleanup_error) => Err(cleanup_error).context(format!(
                    "K3s port-forward did not become ready and VAT could not confirm cleanup: {error:#}"
                )),
            },
        }
    }

    fn with_ready_local_port(mut self, local_port: u16) -> Result<Self> {
        self.storage.marker.local_port = Some(local_port);
        self.storage.update_marker()?;
        Ok(self)
    }

    fn begin_host_child(&mut self) -> Result<u32> {
        let pgid = self
            .storage
            .marker
            .pgid
            .context("ready K3s port-forward is missing its private process group")?;
        // Persist this before spawning the host command. A crash in the small
        // spawn window can leave a conservative marker behind, but must never
        // convince recovery that no host process could exist.
        self.storage.marker.host_started = true;
        self.storage.update_marker()?;
        Ok(pgid)
    }

    fn wait_for_ready(
        &mut self,
        session: &ActiveSession,
        request: &PortForwardRequest,
        cancellation: &SignalCancellation,
    ) -> Result<u16> {
        let deadline = Instant::now() + PORT_FORWARD_READY_TIMEOUT;
        loop {
            if let Some(signal) = cancellation.received() {
                bail!("received signal {signal} while waiting for K3s port-forward readiness");
            }
            if active_session_expired(&session.metadata) {
                bail!(
                    "K3s session {} expired before its port-forward became ready",
                    session.metadata.id
                );
            }
            match self.logs.recv_timeout(PORT_FORWARD_POLL_INTERVAL) {
                Ok(line) => {
                    self.record_diagnostic(&line);
                    if let Some(local_port) = parse_ready_line(&line.text, request.remote_port) {
                        if request.requested_local_port != 0
                            && request.requested_local_port != local_port
                        {
                            bail!(
                                "kubectl reported loopback port {local_port}, but VAT requested {}",
                                request.requested_local_port
                            );
                        }
                        self.ensure_running()?;
                        return Ok(local_port);
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {}
            }
            self.ensure_running()?;
            if Instant::now() >= deadline {
                bail!(
                    "K3s port-forward {} in namespace {} did not report loopback readiness within {} seconds{}",
                    request.resource,
                    request.namespace,
                    PORT_FORWARD_READY_TIMEOUT.as_secs(),
                    self.diagnostic_suffix(),
                );
            }
        }
    }

    fn ensure_running(&mut self) -> Result<()> {
        if child_has_exited_without_reap(&self.child)? {
            bail!(
                "kubectl port-forward exited before the host command completed{}",
                self.diagnostic_suffix(),
            );
        }
        Ok(())
    }

    fn local_port(&self) -> u16 {
        self.storage
            .marker
            .local_port
            .expect("port-forward result only prints after readiness")
    }

    fn stop_and_confirm(&mut self, session: &ActiveSession) -> Result<()> {
        if self.stopped {
            return Ok(());
        }
        let pgid = self
            .storage
            .marker
            .pgid
            .context("running K3s port-forward is missing its private process group")?;
        // Set this before signalling: even an error path may have reaped the
        // leader, at which point Drop is no longer allowed to signal its
        // numeric process group again.
        self.group_shutdown_started = true;
        stop_child_process_group(&mut self.child, "kubectl port-forward")?;
        // This foreground path still owns the unreaped direct kubectl child,
        // and `stop_child_process_group` has already confirmed the exact
        // process group is gone. Do not invoke a separate `ps` lookup here:
        // recovery needs argv authentication because it has no Child handle,
        // but a live owner can prove group death without that ambient process
        // inspection (which may itself be unavailable in a constrained host).
        if process_group_exists(pgid)? {
            bail!(
                "kubectl port-forward private process group remains visible after cleanup; refusing to remove the recovery marker"
            );
        }
        for reader in self.readers.drain(..) {
            let _ = reader.join();
        }
        remove_storage(session, &self.storage.marker)?;
        self.stopped = true;
        Ok(())
    }

    fn record_diagnostic(&mut self, line: &ForwardLogLine) {
        let source = match line.stream {
            ForwardLogStream::Stdout => "stdout",
            ForwardLogStream::Stderr => "stderr",
        };
        if self.diagnostics.len() >= PORT_FORWARD_LOG_LIMIT {
            return;
        }
        let rendered = format!(" {source}:{}", line.text.trim());
        let remaining = PORT_FORWARD_LOG_LIMIT - self.diagnostics.len();
        self.diagnostics
            .push_str(&rendered[..rendered.len().min(remaining)]);
    }

    fn diagnostic_suffix(&self) -> String {
        if self.diagnostics.is_empty() {
            String::new()
        } else {
            format!("; kubectl output:{}", self.diagnostics)
        }
    }
}

impl Drop for ActivePortForward {
    fn drop(&mut self) {
        // While this object has not begun shutdown it still owns the direct,
        // unreaped kubectl child, which pins its process-group identity. Once
        // shutdown starts, only explicit recovery with marker authentication
        // may act further; retrying here could hit a recycled PGID.
        if !self.stopped && !self.group_shutdown_started {
            let _ = stop_child_process_group(&mut self.child, "kubectl port-forward");
        }
    }
}

enum HostOutcome {
    Exited(ExitStatus),
    Interrupted(i32),
}

fn run_host_child(
    session: &ActiveSession,
    request: &PortForwardRequest,
    forward: &mut ActivePortForward,
    cancellation: &SignalCancellation,
) -> Result<HostOutcome> {
    let (program, program_args) = request
        .command
        .split_first()
        .context("vat k8s session port-forward run requires a host command after --")?;
    forward.ensure_running()?;
    let local_port = forward.local_port();
    let mut command = Command::new(program);
    command
        .args(program_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .env("HOME", &forward.storage.host_home)
        .env("VAT_K8S_PORT_FORWARD_HOST", "127.0.0.1")
        .env("VAT_K8S_PORT_FORWARD_PORT", local_port.to_string())
        .env(
            "VAT_K8S_PORT_FORWARD_ADDR",
            format!("127.0.0.1:{local_port}"),
        )
        .env("VAT_K8S_PORT_FORWARD_RESOURCE", &request.resource)
        .env("VAT_K8S_PORT_FORWARD_NAMESPACE", &request.namespace);
    for key in sensitive_environment() {
        command.env_remove(key);
    }
    for key in [
        "VAT_HOME",
        "VAT_K8S_CACHE_DIR",
        "VAT_K8S_API_SERVER",
        "VAT_K8S_EPHEMERAL",
    ] {
        command.env_remove(key);
    }
    // Host work joins the already-recorded kubectl process group. This is
    // intentionally not a separate group: the owned forward group is the
    // one identity that crash recovery can authenticate before signalling.
    join_process_group(&mut command, forward.begin_host_child()?);
    let mut child = command.spawn().with_context(|| {
        format!(
            "run host command {program:?} through loopback K3s port-forward {}",
            request.resource
        )
    })?;
    loop {
        if let Some(status) = child.try_wait().context("poll port-forward host child")? {
            return Ok(HostOutcome::Exited(status));
        }
        if let Some(signal) = cancellation.received() {
            stop_host_child(&mut child, "port-forward host child")?;
            return Ok(HostOutcome::Interrupted(signal));
        }
        if active_session_expired(&session.metadata) {
            stop_host_child(&mut child, "port-forward host child")?;
            bail!(
                "K3s session {} lease expired while its port-forward host command was running",
                session.metadata.id
            );
        }
        if let Err(error) = forward.ensure_running() {
            let _ = stop_host_child(&mut child, "port-forward host child");
            return Err(error).context("K3s port-forward ended before its host command");
        }
        thread::sleep(PORT_FORWARD_POLL_INTERVAL);
    }
}

/// The JSON equivalent of `run_host_child` preserves the credential-free host
/// boundary and TTL/cancellation polling, while capturing both streams without
/// replaying them. It intentionally does not join those readers: a descendant
/// in kubectl's shared process group can inherit them, and only
/// `stop_and_confirm` is allowed to decide that cleanup completed first.
fn run_host_child_json(
    session: &ActiveSession,
    request: &PortForwardRequest,
    forward: &mut ActivePortForward,
    cancellation: &SignalCancellation,
) -> Result<JsonHostOutcome> {
    let (program, program_args) = request
        .command
        .split_first()
        .context("vat k8s session port-forward run requires a host command after --")?;
    forward.ensure_running()?;
    require_active_session_lease_json(session)?;
    if let Some(signal) = cancellation.received() {
        bail!("received signal {signal} immediately before K3s port-forward host command start");
    }
    let local_port = forward.local_port();
    let mut command = Command::new(program);
    command
        .args(program_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("HOME", &forward.storage.host_home)
        .env("VAT_K8S_PORT_FORWARD_HOST", "127.0.0.1")
        .env("VAT_K8S_PORT_FORWARD_PORT", local_port.to_string())
        .env(
            "VAT_K8S_PORT_FORWARD_ADDR",
            format!("127.0.0.1:{local_port}"),
        )
        .env("VAT_K8S_PORT_FORWARD_RESOURCE", &request.resource)
        .env("VAT_K8S_PORT_FORWARD_NAMESPACE", &request.namespace);
    for key in sensitive_environment() {
        command.env_remove(key);
    }
    for key in [
        "VAT_HOME",
        "VAT_K8S_CACHE_DIR",
        "VAT_K8S_API_SERVER",
        "VAT_K8S_EPHEMERAL",
    ] {
        command.env_remove(key);
    }

    // This is deliberately the last pre-spawn proof point. The marker records
    // `host_started` immediately before spawn so a crash cannot make recovery
    // assume no host descendant exists; the session lock stays held until its
    // group and private marker have both been confirmed gone.
    forward.ensure_running()?;
    require_active_session_lease_json(session)?;
    if let Some(signal) = cancellation.received() {
        bail!("received signal {signal} immediately before K3s port-forward host command spawn");
    }
    join_process_group(&mut command, forward.begin_host_child()?);
    let child = command.spawn().with_context(|| {
        format!(
            "run host command {program:?} through loopback K3s port-forward {}",
            request.resource
        )
    })?;
    let mut child = super::port_forward_json::CapturedHostChild::start(child);
    if !child.capture_ready() {
        // A partial reader setup must not leave this direct host child as a
        // zombie after `stop_and_confirm` reaps only kubectl. Do not join its
        // surviving reader here: descendants in kubectl's group may still
        // hold that pipe, and outer cleanup owns their termination.
        stop_host_child(
            child.child_mut(),
            "port-forward JSON host child after capture setup failure",
        )?;
        return Ok(JsonHostOutcome::CaptureUnavailable { child });
    }
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(JsonHostOutcome::Exited { status, child });
        }
        if let Some(signal) = cancellation.received() {
            stop_host_child(child.child_mut(), "port-forward JSON host child")?;
            return Ok(JsonHostOutcome::Interrupted { signal, child });
        }
        if active_session_expired(&session.metadata) {
            stop_host_child(child.child_mut(), "port-forward JSON host child")?;
            bail!(
                "K3s session {} lease expired while its port-forward JSON host command was running",
                session.metadata.id
            );
        }
        if let Err(error) = forward.ensure_running() {
            let _ = stop_host_child(child.child_mut(), "port-forward JSON host child");
            return Err(error).context("K3s port-forward ended before its JSON host command");
        }
        thread::sleep(PORT_FORWARD_POLL_INTERVAL);
    }
}

fn print_json_result(
    session: &ActiveSession,
    request: &PortForwardRequest,
    local_port: u16,
    snapshot: &super::port_forward_json::CapturedHostSnapshot,
) {
    println!(
        "{}",
        serde_json::json!({
            "schema": "vat.k8s.session.port-forward.v1",
            "format": "vat_json",
            "type": "vat_k8s_session_port_forward",
            "id": session.metadata.id,
            "state": "active",
            "resource": request.resource,
            "namespace": request.namespace,
            "local_host": "127.0.0.1",
            "local_port": local_port,
            "remote_port": request.remote_port,
            "child_exit_code": json_child_exit_code(snapshot.status),
            "child_signal": snapshot.status.signal(),
            "stdout": snapshot.stdout.text,
            "stderr": snapshot.stderr.text,
            "stdout_truncated": snapshot.stdout.truncated,
            "stderr_truncated": snapshot.stderr.truncated,
            "stdout_utf8_lossy": snapshot.stdout.utf8_lossy,
            "stderr_utf8_lossy": snapshot.stderr.utf8_lossy,
            "api_verified": true,
            "runtime_invoked": true,
            "session_record_mutated": false,
            "cleanup": "confirmed",
            "cleanup_confirmed": true,
            "port_forward": "none",
            "next": format!("vat k8s session status --verify-api {}", session.metadata.id),
        })
    );
}

fn json_child_exit_code(status: ExitStatus) -> i32 {
    status
        .code()
        .or_else(|| status.signal().map(|signal| 128 + signal))
        .unwrap_or(1)
}

fn print_result(
    session: &ActiveSession,
    request: &PortForwardRequest,
    local_port: u16,
    child: Option<ExitStatus>,
    interrupted_signal: Option<i32>,
) {
    super::print_terminal_record(serde_json::json!({
        "type": "vat_k8s_session_port_forward",
        "id": session.metadata.id,
        "state": "active",
        "resource": request.resource,
        "namespace": request.namespace,
        "local_host": "127.0.0.1",
        "local_port": local_port,
        "remote_port": request.remote_port,
        "child_exit_code": child.as_ref().and_then(|status| status.code()),
        "child_signal": child.as_ref().and_then(|status| status.signal()),
        "interrupted_signal": interrupted_signal,
        "cleanup": "confirmed",
        "next": format!("vat k8s session status {}", session.metadata.id),
    }));
}

#[derive(Clone, Copy)]
enum ForwardLogStream {
    Stdout,
    Stderr,
}

struct ForwardLogLine {
    stream: ForwardLogStream,
    text: String,
}

fn spawn_log_reader<R>(
    reader: R,
    stream: ForwardLogStream,
    sender: mpsc::Sender<ForwardLogLine>,
) -> JoinHandle<()>
where
    R: std::io::Read + Send + 'static,
{
    thread::spawn(move || {
        for line in BufReader::new(reader).lines() {
            match line {
                Ok(text) => {
                    if sender.send(ForwardLogLine { stream, text }).is_err() {
                        return;
                    }
                }
                Err(_) => return,
            }
        }
    })
}

fn parse_service_resource(source: &str) -> Result<String> {
    let Some(name) = source.strip_prefix("service/") else {
        bail!(
            "K3s port-forward only accepts a Service selector of the form service/<name>; got {source:?}"
        );
    };
    if !valid_dns_label(name) {
        bail!("invalid K3s Service selector {source:?}; use service/<lowercase-dns-label>");
    }
    Ok(format!("service/{name}"))
}

fn valid_dns_label(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 63
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && value
            .as_bytes()
            .first()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && value
            .as_bytes()
            .last()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}

fn valid_token(value: &str) -> bool {
    let Some(hex) = value.strip_prefix(PORT_FORWARD_TOKEN_PREFIX) else {
        return false;
    };
    hex.len() == PORT_FORWARD_TOKEN_BYTES * 2
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_legacy_token(value: &str) -> bool {
    // v1 used a predictable, timestamp-shaped token. It is accepted only as
    // a private storage component after the owner is dead and its recorded
    // process group is already absent; it is never an authentication proof.
    value.starts_with(PORT_FORWARD_TOKEN_PREFIX)
        && value.len() > PORT_FORWARD_TOKEN_PREFIX.len()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn fresh_token() -> Result<String> {
    let mut bytes = [0_u8; PORT_FORWARD_TOKEN_BYTES];
    getrandom::fill(&mut bytes).map_err(|error| {
        anyhow::anyhow!("read OS CSPRNG for K3s port-forward recovery token: {error}")
    })?;
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut token = String::with_capacity(PORT_FORWARD_TOKEN_PREFIX.len() + bytes.len() * 2);
    token.push_str(PORT_FORWARD_TOKEN_PREFIX);
    for byte in bytes {
        token.push(HEX[(byte >> 4) as usize] as char);
        token.push(HEX[(byte & 0x0f) as usize] as char);
    }
    Ok(token)
}

fn port_mapping(local_port: u16, remote_port: u16) -> String {
    if local_port == 0 {
        format!(":{remote_port}")
    } else {
        format!("{local_port}:{remote_port}")
    }
}

fn parse_ready_line(line: &str, remote_port: u16) -> Option<u16> {
    let rest = line.trim().strip_prefix("Forwarding from 127.0.0.1:")?;
    let (local, remote) = rest.split_once(" -> ")?;
    if remote.trim() != remote_port.to_string() {
        return None;
    }
    let local = local.parse::<u16>().ok()?;
    (local != 0).then_some(local)
}

fn marker_path(session: &ActiveSession) -> PathBuf {
    session.directory.join(PORT_FORWARD_MARKER)
}

fn read_marker(session: &ActiveSession) -> Result<StoredPortForwardMarker> {
    let path = marker_path(session);
    let bytes = fs::read(&path)
        .with_context(|| format!("read leased K3s port-forward marker {}", path.display()))?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse leased K3s port-forward marker {}", path.display()))?;
    let schema = value
        .get("schema")
        .and_then(serde_json::Value::as_str)
        .context("leased K3s port-forward marker is missing its schema")?;
    match schema {
        PORT_FORWARD_SCHEMA => {
            let marker: PortForwardMarker = serde_json::from_value(value).with_context(|| {
                format!(
                    "parse current leased K3s port-forward marker {}",
                    path.display()
                )
            })?;
            validate_current_marker(session, &marker)?;
            Ok(StoredPortForwardMarker::Current(marker))
        }
        LEGACY_PORT_FORWARD_SCHEMA => {
            let marker: LegacyPortForwardMarker =
                serde_json::from_value(value).with_context(|| {
                    format!(
                        "parse legacy leased K3s port-forward marker {}",
                        path.display()
                    )
                })?;
            validate_legacy_marker(session, &marker)?;
            Ok(StoredPortForwardMarker::Legacy(marker))
        }
        _ => bail!("leased K3s port-forward marker is not a valid VAT-owned record"),
    }
}

fn validate_current_marker(session: &ActiveSession, marker: &PortForwardMarker) -> Result<()> {
    if marker.schema != PORT_FORWARD_SCHEMA
        || marker.session_id != session.metadata.id
        || marker.owner_pid == 0
        || !valid_token(&marker.token)
        || parse_service_resource(&marker.resource).is_err()
        || !valid_dns_label(&marker.namespace)
        || marker.remote_port == 0
        || !matches!(marker.state.as_str(), "starting" | "running" | "cleaning")
        || marker.kubectl.is_empty()
        || marker.cache_dir.is_empty()
    {
        bail!("leased K3s port-forward marker is not a valid VAT-owned record");
    }
    if marker.local_port.is_some_and(|port| port == 0) {
        bail!("leased K3s port-forward marker has an invalid local port");
    }
    match marker.state.as_str() {
        "starting"
            if marker.kubectl_pid.is_some() || marker.pgid.is_some() || marker.host_started =>
        {
            bail!("starting K3s port-forward marker has an impossible process state");
        }
        "running" => {
            let kubectl_pid = marker
                .kubectl_pid
                .filter(|pid| *pid != 0)
                .context("running K3s port-forward marker is missing kubectl pid")?;
            let pgid = marker
                .pgid
                .filter(|pid| *pid != 0)
                .context("running K3s port-forward marker is missing process group")?;
            if kubectl_pid != pgid {
                bail!(
                    "running K3s port-forward marker has mismatched kubectl pid and process group"
                );
            }
        }
        "cleaning" => {}
        _ => unreachable!("marker state was validated above"),
    }
    let (expected_root, expected_token_directory, expected_cache) =
        storage_paths(session, &marker)?;
    if marker.state == "cleaning" {
        require_private_directory_if_present(&expected_root, "leased K3s port-forward directory")?;
        require_private_directory_if_present(
            &expected_token_directory,
            "leased K3s port-forward token directory",
        )?;
        require_private_directory_if_present(
            &expected_cache,
            "leased K3s port-forward cache directory",
        )?;
    } else {
        require_private_directory(&expected_root, "leased K3s port-forward directory")?;
        require_private_directory(
            &expected_token_directory,
            "leased K3s port-forward token directory",
        )?;
        require_private_directory(&expected_cache, "leased K3s port-forward cache directory")?;
    }
    Ok(())
}

fn validate_legacy_marker(session: &ActiveSession, marker: &LegacyPortForwardMarker) -> Result<()> {
    if marker.schema != LEGACY_PORT_FORWARD_SCHEMA
        || marker.session_id != session.metadata.id
        || marker.owner_pid == 0
        || !valid_legacy_token(&marker.token)
        || !matches!(marker.state.as_str(), "starting" | "running" | "cleaning")
        || marker.cache_dir.is_empty()
    {
        bail!("legacy K3s port-forward marker is not a valid VAT-owned record");
    }
    if marker.pgid.is_some_and(|pgid| pgid == 0) {
        bail!("legacy K3s port-forward marker has an invalid process group");
    }
    let (expected_root, expected_token_directory, expected_cache) =
        legacy_storage_paths(session, marker)?;
    if marker.state == "cleaning" {
        require_private_directory_if_present(&expected_root, "leased K3s port-forward directory")?;
        require_private_directory_if_present(
            &expected_token_directory,
            "leased K3s port-forward token directory",
        )?;
        require_private_directory_if_present(
            &expected_cache,
            "leased K3s port-forward cache directory",
        )?;
    } else {
        require_private_directory(&expected_root, "leased K3s port-forward directory")?;
        require_private_directory(
            &expected_token_directory,
            "leased K3s port-forward token directory",
        )?;
        require_private_directory(&expected_cache, "leased K3s port-forward cache directory")?;
    }
    Ok(())
}

fn storage_paths(
    session: &ActiveSession,
    marker: &PortForwardMarker,
) -> Result<(PathBuf, PathBuf, PathBuf)> {
    let expected_root = session.directory.join(PORT_FORWARD_DIRECTORY);
    let expected_token_directory = expected_root.join(&marker.token);
    let expected_cache = expected_token_directory.join("cache");
    if Path::new(&marker.cache_dir) != expected_cache {
        bail!("leased K3s port-forward marker cache path is outside its private token directory");
    }
    Ok((expected_root, expected_token_directory, expected_cache))
}

fn legacy_storage_paths(
    session: &ActiveSession,
    marker: &LegacyPortForwardMarker,
) -> Result<(PathBuf, PathBuf, PathBuf)> {
    let expected_root = session.directory.join(PORT_FORWARD_DIRECTORY);
    let expected_token_directory = expected_root.join(&marker.token);
    let expected_cache = expected_token_directory.join("cache");
    if Path::new(&marker.cache_dir) != expected_cache {
        bail!("legacy K3s port-forward marker cache path is outside its private token directory");
    }
    Ok((expected_root, expected_token_directory, expected_cache))
}

fn require_private_directory_if_present(path: &Path, label: &str) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => {
            require_private_directory(path, label)?;
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| format!("inspect {label} {}", path.display())),
    }
}

fn remove_storage(session: &ActiveSession, marker: &PortForwardMarker) -> Result<()> {
    let path = marker_path(session);
    if marker.state != "cleaning" {
        require_private_file(&path, "leased K3s port-forward marker")?;
        let mut cleaning = marker.clone();
        // Persist a marker that remains readable even after its private cache
        // disappears. If a later unlink fails, reconcile can finish cleanup
        // instead of leaving an unparsable, permanently blocking marker.
        cleaning.state = "cleaning".to_string();
        super::replace_session_marker(&path, &cleaning)
            .context("record durable K3s port-forward cleanup phase")?;
        return finish_cleaning_storage(session, &cleaning);
    }
    finish_cleaning_storage(session, marker)
}

fn remove_legacy_storage(session: &ActiveSession, marker: &LegacyPortForwardMarker) -> Result<()> {
    let path = marker_path(session);
    if marker.state != "cleaning" {
        require_private_file(&path, "legacy leased K3s port-forward marker")?;
        let mut cleaning = marker.clone();
        // Preserve a v1-shaped, but durable, tombstone. This marker is still
        // never usable for process authentication; it merely makes storage
        // cleanup retryable if an unlink fails after the cache disappears.
        cleaning.state = "cleaning".to_string();
        super::replace_session_marker(&path, &cleaning)
            .context("record durable legacy K3s port-forward cleanup phase")?;
        return finish_legacy_storage(session, &cleaning);
    }
    finish_legacy_storage(session, marker)
}

fn finish_cleaning_storage(session: &ActiveSession, marker: &PortForwardMarker) -> Result<()> {
    let (expected_root, expected_token_directory, _) = storage_paths(session, marker)?;
    finish_storage_paths(session, &expected_root, &expected_token_directory)
}

fn finish_legacy_storage(session: &ActiveSession, marker: &LegacyPortForwardMarker) -> Result<()> {
    let (expected_root, expected_token_directory, _) = legacy_storage_paths(session, marker)?;
    finish_storage_paths(session, &expected_root, &expected_token_directory)
}

fn finish_storage_paths(
    session: &ActiveSession,
    expected_root: &Path,
    expected_token_directory: &Path,
) -> Result<()> {
    let path = marker_path(session);
    if require_private_directory_if_present(
        expected_token_directory,
        "leased K3s port-forward token directory",
    )? {
        fs::remove_dir_all(expected_token_directory).with_context(|| {
            format!(
                "remove private K3s port-forward storage {}",
                expected_token_directory.display()
            )
        })?;
        match fs::symlink_metadata(expected_token_directory) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Ok(_) => {
                bail!(
                    "private K3s port-forward storage {} remains after cleanup",
                    expected_token_directory.display()
                );
            }
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "inspect private K3s port-forward storage {} after cleanup",
                        expected_token_directory.display()
                    )
                });
            }
        }
    }
    if require_private_directory_if_present(expected_root, "leased K3s port-forward directory")? {
        if fs::read_dir(expected_root)
            .with_context(|| {
                format!(
                    "read K3s port-forward directory {}",
                    expected_root.display()
                )
            })?
            .next()
            .is_some()
        {
            bail!(
                "leased K3s port-forward directory {} contains unexpected residual state; cleanup marker is retained",
                expected_root.display()
            );
        }
        fs::remove_dir(expected_root).with_context(|| {
            format!(
                "remove empty K3s port-forward directory {}",
                expected_root.display()
            )
        })?;
    }
    match fs::symlink_metadata(&path) {
        Ok(_) => {
            require_private_file(&path, "leased K3s port-forward marker")?;
            fs::remove_file(&path)
                .with_context(|| format!("remove K3s port-forward marker {}", path.display()))?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error)
                .with_context(|| format!("inspect K3s port-forward marker {}", path.display()));
        }
    }
    Ok(())
}

/// Reconcile an interrupted foreground forward before another mutating session
/// operation. The caller must hold the session flock: after O_CLOEXEC, a
/// successful acquisition proves no live VAT operation owns this marker, and
/// avoids treating a recycled numeric owner PID as authoritative. The private
/// v2 cache token in observed argv is the only proof that permits signalling.
pub(super) fn reconcile(session: &ActiveSession, _operation: &SessionOperationLock) -> Result<()> {
    let path = marker_path(session);
    let marker = match fs::symlink_metadata(&path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!(
                    "leased K3s port-forward marker {} is not a real private file; VAT will not reuse or delete this session",
                    path.display()
                );
            }
            require_private_file(&path, "leased K3s port-forward marker")?;
            read_marker(session)?
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(error).with_context(|| {
                format!("inspect leased K3s port-forward marker {}", path.display())
            });
        }
    };

    match marker {
        StoredPortForwardMarker::Current(marker) => reconcile_current_marker(session, marker),
        StoredPortForwardMarker::Legacy(marker) => reconcile_legacy_marker(session, marker),
    }
}

fn reconcile_current_marker(session: &ActiveSession, marker: PortForwardMarker) -> Result<()> {
    // A cleaning tombstone is no longer a process-control record. Handle it
    // before owner PID liveness so PID reuse cannot strand a torn cleanup.
    if marker.state == "cleaning" {
        return finish_cleaning_storage(session, &marker);
    }
    let mut matches = marker_processes(&marker)?;
    if matches.is_empty() {
        // A parent can die between spawn and writing the child PID. Give a
        // just-starting kubectl one bounded chance to appear before declaring
        // the marker stale.
        thread::sleep(Duration::from_millis(200));
        matches = marker_processes(&marker)?;
    }
    if matches.is_empty() {
        if !marker.host_started {
            return remove_storage(session, &marker);
        }
        let pgid = marker
            .pgid
            .context("host-started K3s port-forward marker is missing its process group")?;
        if !process_group_exists(pgid)? {
            return remove_storage(session, &marker);
        }
        bail!(
            "K3s session {} recorded a host command in its port-forward process group, but VAT can no longer authenticate the kubectl group leader; recovery marker is retained rather than risking an unrelated process group",
            session.metadata.id,
        );
    }
    match matches.as_slice() {
        [] => unreachable!("empty port-forward matches returned above"),
        [observed]
            if observed.pid == observed.pgid
                && marker.pgid.is_none_or(|expected| expected == observed.pgid) =>
        {
            stop_recovered_process_group(observed.pgid, &marker)?;
            if !marker_processes(&marker)?.is_empty() || process_group_exists(observed.pgid)? {
                bail!(
                    "VAT terminated recovered K3s port-forward process group {}, but it remains visible in ps or as a live process group; refusing to remove the recovery marker",
                    observed.pgid,
                );
            }
            remove_storage(session, &marker)
        }
        [observed] => bail!(
            "recovered K3s port-forward process {} has unexpected process group {}; refusing to signal an unverified group",
            observed.pid,
            observed.pgid,
        ),
        _ => bail!(
            "multiple processes match the private K3s port-forward recovery token for session {}; VAT will not guess which process group to terminate",
            session.metadata.id,
        ),
    }
}

fn reconcile_legacy_marker(session: &ActiveSession, marker: LegacyPortForwardMarker) -> Result<()> {
    // As above, this is only a storage tombstone. A recycled v1 owner PID
    // must not prevent a retry after cache deletion already began.
    if marker.state == "cleaning" {
        return finish_legacy_storage(session, &marker);
    }
    let pgid = marker.pgid.context(
        "legacy K3s port-forward marker has no recorded process group; VAT will not remove it without manual inspection",
    )?;
    if process_group_exists(pgid)? {
        bail!(
            "legacy K3s port-forward marker for session {} records a live process group {pgid}; VAT will not signal a pre-CSPRNG identity. Stop that group manually, then retry session delete",
            session.metadata.id,
        );
    }
    // No signal is sent for v1. Its predictable token is insufficient to
    // authenticate a process, but an already-absent recorded group makes
    // private-storage cleanup safe and unblocks historical stale leases.
    remove_legacy_storage(session, &marker)
}

/// A status helper deliberately avoids mutating recovery state.
pub(super) fn status(session: &ActiveSession) -> &'static str {
    match fs::symlink_metadata(marker_path(session)) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => "none",
        Ok(_) => "recovery_required",
        Err(_) => "unknown",
    }
}

struct ObservedProcess {
    pid: u32,
    pgid: u32,
}

fn marker_processes(marker: &PortForwardMarker) -> Result<Vec<ObservedProcess>> {
    let candidates = marker_candidate_pids(marker)?;
    let mut observed = Vec::new();
    for pid in candidates {
        let pid = pid.to_string();
        let args = strings(&["-ww", "-p", &pid, "-o", "pid=,pgid=,command="]);
        let output = run_bounded("/bin/ps", &args, Duration::from_secs(5), &[], &[])?;
        if !output.status.success() {
            continue;
        }
        let expected_mapping = port_mapping(marker.requested_local_port, marker.remote_port);
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let mut fields = line.split_whitespace();
            let Some(pid) = fields.next().and_then(|value| value.parse::<u32>().ok()) else {
                continue;
            };
            let Some(pgid) = fields.next().and_then(|value| value.parse::<u32>().ok()) else {
                continue;
            };
            let command = fields.collect::<Vec<_>>().join(" ");
            if marker.kubectl_pid.is_none_or(|expected| expected == pid)
                && marker.pgid.is_none_or(|expected| expected == pgid)
                // `kubectl` may be a PATH wrapper that execs the real binary,
                // so its original canonical path is not a stable identity.
                // The CSPRNG-owned cache path plus the exact loopback forward
                // shape is the durable proof instead.
                && command.contains("port-forward")
                && command.contains("--address")
                && command.contains("127.0.0.1")
                && command.contains(&marker.resource)
                && command.contains(&expected_mapping)
                && command.contains(&marker.cache_dir)
            {
                observed.push(ObservedProcess { pid, pgid });
            }
        }
    }
    Ok(observed)
}

fn marker_candidate_pids(marker: &PortForwardMarker) -> Result<Vec<u32>> {
    if let Some(pid) = marker.kubectl_pid.filter(|pid| *pid != 0) {
        return Ok(vec![pid]);
    }
    // Only an interrupted spawn can lack a recorded PID. Search by the random
    // token, then verify every candidate's complete command line below before
    // signal delivery. This avoids a slow unrestricted ps scan.
    let args = strings(&["-f", &marker.token]);
    let output = run_bounded("/usr/bin/pgrep", &args, Duration::from_secs(5), &[], &[])?;
    if output.status.code() == Some(1) {
        return Ok(Vec::new());
    }
    if !output.status.success() {
        bail!(
            "could not search for K3s port-forward recovery process identity: {}",
            super::command_failure(&output)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.trim().parse::<u32>().ok())
        .collect())
}

/// Resolve a standalone kubectl, never OrbStack's compatibility binary. The
/// leased K3s path must remain usable after OrbStack is uninstalled, so a
/// symlink resolved into `OrbStack.app` is not a valid host dependency even
/// though it can currently reach an API.
pub(super) fn resolve_kubectl() -> Result<PathBuf> {
    let path = std::env::var_os("PATH").context("PATH is not available to locate kubectl")?;
    resolve_kubectl_on_path(&path)
}

fn resolve_kubectl_on_path(path: &OsStr) -> Result<PathBuf> {
    let mut rejected_orbstack = Vec::new();
    for directory in std::env::split_paths(&path) {
        let candidate = directory.join("kubectl");
        let Ok(metadata) = fs::metadata(&candidate) else {
            continue;
        };
        if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
            continue;
        }
        let resolved = fs::canonicalize(&candidate)
            .with_context(|| format!("canonicalize kubectl path {}", candidate.display()))?;
        if is_orbstack_managed_path(&resolved) {
            rejected_orbstack.push(resolved);
            continue;
        }
        return Ok(resolved);
    }
    if !rejected_orbstack.is_empty() {
        let rejected = rejected_orbstack
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        bail!(
            "VAT refuses OrbStack-provided kubectl ({rejected}); install an independent kubectl, put it on PATH, then retry the K3s operation"
        );
    }
    bail!("kubectl was not found on PATH; install an independent kubectl before using VAT K3s")
}

pub(super) fn is_orbstack_managed_path(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case("orbstack.app")
    })
}

fn set_process_group(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

fn join_process_group(command: &mut Command, pgid: u32) {
    use std::os::unix::process::CommandExt;
    command.process_group(pgid as i32);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProcessGroupSignalOutcome {
    DeliveredOrGone,
    PermissionPartial,
}

fn stop_child_process_group(child: &mut Child, label: &str) -> Result<()> {
    stop_child_process_group_with(
        child,
        label,
        signal_process_group_outcome,
        process_group_exists,
        PORT_FORWARD_STOP_TIMEOUT,
        PORT_FORWARD_POLL_INTERVAL,
    )
}

/// Stop a direct, still-owned group leader. Darwin can report EPERM when one
/// member of a process group is not signalable, even though the group also
/// contains VAT's direct kubectl child. Keep the group proof strict, but use
/// that direct Child handle to reap the owned root before deciding whether the
/// group has actually disappeared.
fn stop_child_process_group_with<SignalGroup, GroupExists>(
    child: &mut Child,
    label: &str,
    signal_group: SignalGroup,
    group_exists: GroupExists,
    stop_timeout: Duration,
    poll_interval: Duration,
) -> Result<()>
where
    SignalGroup: Fn(u32, i32, &str) -> Result<ProcessGroupSignalOutcome>,
    GroupExists: Fn(u32) -> Result<bool>,
{
    let pgid = child.id();
    let term = signal_group(pgid, libc::SIGTERM, label)?;
    // Do not reap the direct group leader before this signal. Keeping it as
    // our unreaped child pins the process-group id, so KILL cannot be sent to
    // a recycled group while an ordinary host background descendant remains.
    thread::sleep(poll_interval);
    let kill = signal_group(pgid, libc::SIGKILL, label)?;
    let permission_partial = matches!(term, ProcessGroupSignalOutcome::PermissionPartial)
        || matches!(kill, ProcessGroupSignalOutcome::PermissionPartial);
    if permission_partial {
        kill_owned_child_if_running(child, label)?;
    }

    let deadline = Instant::now() + stop_timeout;
    let mut exited = false;
    while Instant::now() < deadline {
        if child
            .try_wait()
            .context("poll kubectl port-forward after KILL")?
            .is_some()
        {
            exited = true;
            break;
        }
        thread::sleep(poll_interval);
    }
    if !exited {
        if permission_partial {
            bail!(
                "{label} did not exit after a partially-permitted group TERM/KILL and direct owned-child KILL"
            );
        }
        bail!("{label} did not exit after TERM and KILL");
    }
    let group_deadline = Instant::now() + stop_timeout;
    while group_exists(pgid)? && Instant::now() < group_deadline {
        thread::sleep(poll_interval);
    }
    if !group_exists(pgid)? {
        return Ok(());
    }
    if permission_partial {
        bail!(
            "{label} direct child exited after partially-permitted group TERM/KILL and direct owned-child KILL, but process group {pgid} remains visible; recovery marker is retained"
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

fn stop_host_child(child: &mut Child, label: &str) -> Result<()> {
    if child
        .try_wait()
        .context("poll host child before cleanup")?
        .is_some()
    {
        return Ok(());
    }
    let result = unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGTERM) };
    if result != 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::ESRCH) {
            return Err(error).with_context(|| format!("send TERM to {label}"));
        }
    }
    let deadline = Instant::now() + PORT_FORWARD_STOP_TIMEOUT;
    while Instant::now() < deadline {
        if child
            .try_wait()
            .context("poll host child after TERM")?
            .is_some()
        {
            return Ok(());
        }
        thread::sleep(PORT_FORWARD_POLL_INTERVAL);
    }
    child
        .kill()
        .with_context(|| format!("send KILL to {label} after TERM timeout"))?;
    let _ = child
        .wait()
        .with_context(|| format!("reap {label} after KILL"))?;
    Ok(())
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
            .context("observe kubectl port-forward exit without reaping its process group leader");
    }
    Ok(unsafe { info.si_pid() } != 0)
}

fn stop_recovered_process_group(pgid: u32, marker: &PortForwardMarker) -> Result<()> {
    let label = "recovered kubectl port-forward";
    signal_process_group(pgid, libc::SIGTERM, label)?;
    // Recovery has no Child handle to pin an exited leader PID. Reauthenticate
    // the leader immediately before KILL; if it vanished after TERM, retain a
    // host-started marker rather than signal a naked, possibly recycled pgid.
    let matches = marker_processes(marker)?;
    let Some(observed) = matches
        .as_slice()
        .iter()
        .find(|observed| observed.pid == pgid && observed.pgid == pgid)
    else {
        if !process_group_exists(pgid)? {
            return Ok(());
        }
        if marker.host_started {
            bail!(
                "recovered kubectl port-forward leader {pgid} exited after TERM before VAT could safely KILL its host process group; recovery marker is retained"
            );
        }
        return Ok(());
    };
    if matches.len() != 1 || observed.pid != observed.pgid {
        bail!(
            "recovered K3s port-forward process identity changed before KILL; recovery marker is retained"
        );
    }
    signal_process_group(pgid, libc::SIGKILL, label)?;
    let deadline = Instant::now() + PORT_FORWARD_STOP_TIMEOUT;
    while Instant::now() < deadline {
        if marker_processes(marker)?.is_empty() && !process_group_exists(pgid)? {
            return Ok(());
        }
        thread::sleep(PORT_FORWARD_POLL_INTERVAL);
    }
    bail!(
        "could not confirm recovered kubectl port-forward process group {pgid} exited after TERM and KILL"
    )
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
        _ => Err(error).with_context(|| format!("inspect K3s port-forward process group {pgid}")),
    }
}

fn signal_process_group(pgid: u32, signal: i32, label: &str) -> Result<()> {
    match signal_process_group_outcome(pgid, signal, label)? {
        ProcessGroupSignalOutcome::DeliveredOrGone => Ok(()),
        ProcessGroupSignalOutcome::PermissionPartial => {
            Err(std::io::Error::from_raw_os_error(libc::EPERM))
                .with_context(|| format!("send signal {signal} to {label} process group {pgid}"))
        }
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

struct SignalCancellation {
    receiver: Receiver<i32>,
    last_signal: Arc<AtomicI32>,
    handle: SignalHandle,
    thread: Option<JoinHandle<()>>,
}

impl SignalCancellation {
    fn new() -> Result<Self> {
        let mut signals = Signals::new([SIGINT, SIGTERM])
            .context("install scoped K3s port-forward cancellation handlers")?;
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

impl Drop for SignalCancellation {
    fn drop(&mut self) {
        self.handle.close();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct TestProcessGroupCleanup(Option<u32>);

    impl TestProcessGroupCleanup {
        fn new(pgid: u32) -> Self {
            Self(Some(pgid))
        }
    }

    impl Drop for TestProcessGroupCleanup {
        fn drop(&mut self) {
            if let Some(pgid) = self.0.take() {
                unsafe {
                    libc::kill(-(pgid as libc::pid_t), libc::SIGKILL);
                }
            }
        }
    }

    fn owned_group_child(script: &str) -> Child {
        let mut command = Command::new("/bin/sh");
        command
            .args(["-ec", script])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        set_process_group(&mut command);
        command.spawn().expect("spawn owned test process group")
    }

    fn write_executable(path: &Path) {
        fs::write(path, "#!/bin/sh\nexit 0\n").expect("write fake kubectl");
        let mut permissions = fs::metadata(path)
            .expect("inspect fake kubectl")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("make fake kubectl executable");
    }

    #[test]
    fn partial_group_permission_reaps_the_owned_child_before_group_absence_success() {
        let mut child = owned_group_child("exec sleep 30");
        let attempts = Arc::new(AtomicUsize::new(0));
        let signal_attempts = Arc::clone(&attempts);

        stop_child_process_group_with(
            &mut child,
            "test kubectl port-forward",
            move |_, _, _| {
                signal_attempts.fetch_add(1, Ordering::SeqCst);
                Ok(ProcessGroupSignalOutcome::PermissionPartial)
            },
            |_| Ok(false),
            Duration::from_millis(200),
            Duration::from_millis(5),
        )
        .expect("direct owned-child fallback plus confirmed group absence");

        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        assert!(
            child
                .try_wait()
                .expect("inspect direct child after fallback")
                .is_some(),
            "the direct owned child must be reaped before cleanup succeeds"
        );
    }

    #[test]
    fn partial_group_permission_never_succeeds_while_the_group_remains_visible() {
        let mut child = owned_group_child("sleep 30 & wait");
        let _cleanup = TestProcessGroupCleanup::new(child.id());

        let error = stop_child_process_group_with(
            &mut child,
            "test kubectl port-forward",
            |_, _, _| Ok(ProcessGroupSignalOutcome::PermissionPartial),
            |_| Ok(true),
            Duration::from_millis(40),
            Duration::from_millis(5),
        )
        .expect_err("visible process group must retain recovery state");

        assert!(
            format!("{error:#}").contains("recovery marker is retained"),
            "partial permission must fail closed: {error:#}"
        );
        assert!(
            child
                .try_wait()
                .expect("inspect direct child after fallback")
                .is_some(),
            "the direct owned child must be reaped even when group proof fails"
        );
    }

    #[test]
    fn service_selectors_ports_and_readiness_are_strict() {
        assert_eq!(
            parse_service_resource("service/api").unwrap(),
            "service/api"
        );
        assert!(parse_service_resource("pod/api").is_err());
        assert!(parse_service_resource("service/API").is_err());
        assert!(valid_dns_label("agent-dev-1"));
        assert!(!valid_dns_label("agent_dev"));
        assert_eq!(port_mapping(0, 8080), ":8080");
        assert_eq!(port_mapping(31234, 8080), "31234:8080");
        assert_eq!(
            parse_ready_line("Forwarding from 127.0.0.1:31234 -> 8080", 8080),
            Some(31234)
        );
        assert_eq!(
            parse_ready_line("Forwarding from 127.0.0.1:31234 -> 8081", 8080),
            None
        );
        assert_eq!(
            parse_ready_line("Forwarding from [::1]:31234 -> 8080", 8080),
            None
        );
    }

    #[test]
    fn private_port_forward_tokens_are_path_safe() {
        assert!(valid_token("vat-pf-0123456789abcdef0123456789abcdef"));
        assert!(!valid_token("../vat-pf-0123456789abcdef0123456789abcdef"));
        assert!(!valid_token("vat-pf-0123456789abcdef0123456789abcdeg"));
        assert!(!valid_token("vat-pf-0123456789abcdef"));
    }

    #[test]
    fn legacy_tokens_are_storage_safe_but_not_current_identities() {
        let legacy = "vat-pf-4242-1710000000000";
        assert!(valid_legacy_token(legacy));
        assert!(!valid_token(legacy));
        assert!(!valid_legacy_token("../vat-pf-4242-1710000000000"));
        assert!(!valid_legacy_token("vat-pf-4242_1710000000000"));
    }

    #[test]
    fn private_port_forward_tokens_use_a_fixed_opaque_shape() {
        let first = fresh_token().expect("generate first token");
        let second = fresh_token().expect("generate second token");
        assert!(valid_token(&first));
        assert!(valid_token(&second));
        assert_ne!(first, second);
    }

    #[test]
    fn standalone_kubectl_resolution_skips_orbstack_and_rejects_it_as_the_only_candidate() {
        let root = tempfile::tempdir().expect("temporary kubectl resolver root");
        let orbstack = root
            .path()
            .join("OrbStack.app")
            .join("Contents")
            .join("MacOS")
            .join("xbin");
        let independent = root.path().join("standalone-kubectl");
        fs::create_dir_all(&orbstack).expect("create fake OrbStack kubectl directory");
        fs::create_dir_all(&independent).expect("create fake standalone kubectl directory");
        write_executable(&orbstack.join("kubectl"));
        write_executable(&independent.join("kubectl"));

        let path = std::env::join_paths([orbstack.as_path(), independent.as_path()])
            .expect("join fake kubectl PATH");
        assert_eq!(
            resolve_kubectl_on_path(path.as_os_str()).expect("choose standalone kubectl"),
            fs::canonicalize(independent.join("kubectl")).expect("canonical standalone kubectl")
        );

        let only_orbstack =
            std::env::join_paths([orbstack.as_path()]).expect("join OrbStack-only kubectl PATH");
        let error = resolve_kubectl_on_path(only_orbstack.as_os_str())
            .expect_err("OrbStack-only kubectl must fail closed");
        assert!(
            format!("{error:#}").contains("refuses OrbStack-provided kubectl"),
            "unexpected OrbStack-only resolver error: {error:#}"
        );
    }
}
// HANDWRITE-END
