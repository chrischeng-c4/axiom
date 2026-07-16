// HANDWRITE-BEGIN gap="missing-generator:ephemeral-k8s-session" tracker="#1693" reason="A disposable Apple-machine K3s session owns security-sensitive machine, credential, and child-process lifecycles. It must reconcile exact owned names, validate the inspected backing ID/IP, and fail closed on cleanup uncertainty; no generic generator owns that host-runtime protocol."
//! Headless, disposable local Kubernetes sessions over Apple Container.
//!
//! This is deliberately separate from crate::commands::cluster. Apple's current
//! machine restart path is not reliable enough for a persistent cluster backend,
//! while a one-boot guest can safely serve a single foreground agent command.

use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::net::IpAddr;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, ExitStatus, Output, Stdio};
use std::str::FromStr;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

use crate::commands::run::container_diagnostic_until;
use crate::{paths, sandbox};

mod port_forward;
mod port_forward_json;
mod session_exec;

/// Report whether the host has a standalone kubectl suitable for VAT's
/// Apple-Container K3s commands. This is read-only and rejects OrbStack's
/// compatibility binary by using the same resolver as the runtime path.
pub(crate) fn independent_kubectl_available() -> bool {
    port_forward::resolve_kubectl().is_ok()
}

const MACHINE_ASSET: &str = include_str!("../../assets/k8s/ephemeral-machine/Dockerfile");
const K3S_VERSION: &str = "v1.36.2+k3s1";
const K3S_KUBECONFIG: &str = "/etc/rancher/k3s/k3s.yaml";
const K3S_INSTALL_LOG: &str = "/var/log/vat-k3s-install.log";
// K3s can report its systemd service active before it has created a Node API
// object. `kubectl wait node --all` treats that empty list as an immediate
// error, so poll the actual Ready condition through the bounded outer command
// timeout instead of treating early startup as a terminal failure.
const K3S_READY_SCRIPT: &str = "set -eu; systemctl is-active --quiet k3s; deadline=$(( $(date +%s) + 180 )); while :; do if k3s kubectl get nodes --no-headers 2>/dev/null | awk '$2 == \"Ready\" { found=1 } END { exit !found }'; then k3s kubectl get nodes -o wide; exit 0; fi; if [ \"$(date +%s)\" -ge \"$deadline\" ]; then k3s kubectl get nodes -o wide || true; exit 1; fi; sleep 2; done";
const MACHINE_PREFIX: &str = "vat-k8s-ephemeral-";
const ACTIVE_SESSION_PREFIX: &str = "vat-k8s-session-";
const ACTIVE_SESSION_SCHEMA: &str = "vat.k8s.session.v1";
const MACHINE_CREATE_TIMEOUT: Duration = Duration::from_secs(90);
const MACHINE_READY_TIMEOUT: Duration = Duration::from_secs(90);
const GUEST_PRECHECK_TIMEOUT: Duration = Duration::from_secs(90);
const K3S_INSTALL_TIMEOUT: Duration = Duration::from_secs(300);
const K3S_READY_TIMEOUT: Duration = Duration::from_secs(240);
/// Bootstrap diagnostics are advisory evidence only: they must finish before
/// the exact owned-machine cleanup begins and never turn a failed bootstrap
/// into a longer-running recovery path.
const K3S_BOOTSTRAP_DIAGNOSTIC_TOTAL_TIMEOUT: Duration = Duration::from_secs(6);
const K3S_BOOTSTRAP_DIAGNOSTIC_PROBE_TIMEOUT: Duration = Duration::from_secs(1);
const HOST_API_TIMEOUT: Duration = Duration::from_secs(60);
const CLEANUP_TIMEOUT: Duration = Duration::from_secs(45);
const CLEANUP_RETRY_DELAY: Duration = Duration::from_secs(1);
const ACTIVE_SESSION_DEFAULT_TTL: Duration = Duration::from_secs(30 * 60);
const ACTIVE_SESSION_MIN_TTL: Duration = Duration::from_secs(60);
const ACTIVE_SESSION_MAX_TTL: Duration = Duration::from_secs(4 * 60 * 60);
/// Apple Container's local K3s guest currently runs on the host's ARM64
/// architecture. Keep the first local-image delivery contract deliberately
/// narrow instead of claiming cross-platform image emulation.
const K3S_GUEST_PLATFORM: &str = "linux/arm64";
/// The archive is transient, private, and bounded. Larger images need an
/// explicit future delivery contract rather than unbounded host-disk writes.
const K3S_IMAGE_ARCHIVE_MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const K3S_IMAGE_SAVE_TIMEOUT: Duration = Duration::from_secs(180);
const K3S_IMAGE_IMPORT_TIMEOUT: Duration = Duration::from_secs(180);

/// Parsed CLI inputs for vat k8s ephemeral run.
pub struct EphemeralRunArgs {
    pub image: Option<String>,
    pub command: Vec<String>,
}

/// Parsed CLI inputs for `vat k8s session create`.
pub struct ActiveSessionCreateArgs {
    pub image: Option<String>,
    /// A bounded lease such as `30m`, `2h`, or `900s`.
    pub ttl: String,
}

/// Parsed CLI inputs for `vat k8s session image load`.
pub struct ActiveSessionImageLoadArgs {
    /// Opaque id emitted by `vat k8s session create`.
    pub id: String,
    /// A locally present Apple Container image reference. Arbitrary tar files
    /// are intentionally not accepted by this first delivery contract.
    pub image: String,
    /// OCI platform selected from the locally inspected image.
    pub platform: String,
}

/// Parsed CLI inputs for one foreground loopback K3s Service port-forward.
pub struct ActiveSessionPortForwardArgs {
    /// Emit one bounded VAT JSON result after confirmed tunnel cleanup.
    pub json: bool,
    /// Opaque id emitted by vat k8s session create.
    pub id: String,
    /// One literal service/name selector. Pods and arbitrary resources are
    /// intentionally out of scope for the first agent-facing tunnel contract.
    pub resource: String,
    /// One numeric Service port to forward.
    pub remote_port: u16,
    /// Kubernetes namespace. The CLI defaults this to default.
    pub namespace: String,
    /// Loopback local port; zero requests a kubectl-selected ephemeral port.
    pub local_port: u16,
    /// One host-side assertion or test command after --.
    pub command: Vec<String>,
}

/// Build VAT's embedded systemd image into the Apple Container image store.
/// Building is explicit so ephemeral run never unexpectedly downloads a base
/// image before the agent's requested test command.
pub fn build_default_image() -> Result<ExitCode> {
    ensure_apple_container()?;
    let image = default_machine_image();
    if image_exists(&image)? {
        print_image_result("already_present", &image);
        return Ok(ExitCode::SUCCESS);
    }

    let context = tempfile::Builder::new()
        .prefix("vat-k8s-machine-image-")
        .tempdir()
        .context("create private temporary K3s machine-image context")?;
    restrict_dir(context.path())?;
    let dockerfile = context.path().join("Dockerfile");
    fs::write(&dockerfile, MACHINE_ASSET).with_context(|| {
        format!(
            "write embedded K3s machine Dockerfile at {}",
            dockerfile.display()
        )
    })?;
    restrict_file(&dockerfile)?;

    crate::commands::build::build_image(context.path(), &dockerfile, &image, &[])
        .context("build VAT's Apple Container K3s machine image")?;
    if !image_exists(&image)? {
        bail!(
            "Apple Container reported a successful K3s image build, but {image:?} is absent from its image store"
        );
    }
    print_image_result("built", &image);
    Ok(ExitCode::SUCCESS)
}

/// Create one disposable K3s node, run exactly one foreground host command with
/// a private kubeconfig, then remove credentials and the exact owned machine.
/// A cleanup uncertainty overrides the child exit result.
pub fn ephemeral_run(args: EphemeralRunArgs) -> Result<ExitCode> {
    let (program, program_args) = args
        .command
        .split_first()
        .context("vat k8s ephemeral run requires a host command after --")?;
    ensure_apple_container()?;

    let image = args.image.unwrap_or_else(default_machine_image);
    preflight_image(&image)?;
    let mut marker = create_session_marker(&image)?;
    let mut cleanup = MachineCleanup::new(marker.name.clone());

    let primary = run_ephemeral_session(&mut marker, &image, program, program_args, &mut cleanup);
    let machine_cleanup = cleanup.cleanup();
    let marker_cleanup = if machine_cleanup.is_ok() {
        remove_session_marker(&marker)
    } else {
        Ok(())
    };

    match (primary, machine_cleanup, marker_cleanup) {
        (_, Err(cleanup_error), _) => Err(cleanup_error).context(format!(
            "VAT could not reach a terminal-safe recovery state for owned Apple K3s machine {}; the recovery marker remains at {}. Run vat k8s ephemeral cleanup after resolving the runtime",
            marker.name,
            marker.path.display()
        )),
        (_, Ok(()), Err(marker_error)) => Err(marker_error).context(format!(
            "VAT removed owned machine {} but could not remove its non-secret recovery marker {}",
            marker.name,
            marker.path.display()
        )),
        (Ok(status), Ok(()), Ok(())) => {
            print_session_result(&marker, &image, status);
            Ok(exit_code(status))
        }
        (Err(error), Ok(()), Ok(())) => Err(error),
    }
}

/// Reconcile markers left by an interrupted VAT process. Only records whose
/// PID is no longer alive are eligible; active sessions are reported but never
/// touched. Every deletion is still exact-name-only and absence-confirmed.
pub fn cleanup_abandoned(json: bool) -> Result<ExitCode> {
    let directory = session_directory()?;
    if !directory.exists() {
        return print_cleanup_result(json, Vec::new(), Vec::new(), Vec::new());
    }
    ensure_apple_container()?;

    let mut removed = Vec::new();
    let mut active = Vec::new();
    let mut failed = Vec::new();
    for entry in fs::read_dir(&directory).with_context(|| {
        format!(
            "read ephemeral K3s session directory {}",
            directory.display()
        )
    })? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let marker = match read_session_marker(&path) {
            Ok(marker) => marker,
            Err(error) => {
                failed.push(format!("{}: {error:#}", path.display()));
                continue;
            }
        };
        if process_is_alive(marker.metadata.pid) {
            active.push(marker.name);
            continue;
        }

        let mut cleanup =
            MachineCleanup::recovery(marker.name.clone(), marker.metadata.create_uncertain);
        match cleanup
            .cleanup()
            .and_then(|()| remove_session_marker(&marker))
        {
            Ok(()) => removed.push(marker.name),
            Err(error) => failed.push(format!("{}: {error:#}", marker.name)),
        }
    }

    print_cleanup_result(json, removed, active, failed)
}

/// Create a leased K3s session that agents can use across several explicit
/// `vat k8s session exec` calls. This is intentionally not a durable cluster:
/// VAT never restarts a stopped Apple machine, and lease expiry requires a later
/// `session cleanup` or `session delete` command to reclaim the owned machine.
pub fn session_create(args: ActiveSessionCreateArgs) -> Result<ExitCode> {
    let ttl = if args.ttl.trim().is_empty() {
        ACTIVE_SESSION_DEFAULT_TTL
    } else {
        parse_active_session_ttl(&args.ttl)?
    };
    ensure_apple_container()?;
    let image = args.image.unwrap_or_else(default_machine_image);
    preflight_image(&image)?;

    let mut session = create_active_session_marker(&image, ttl)?;
    let mut cleanup = MachineCleanup::new(session.metadata.name.clone());
    let bootstrap = bootstrap_active_session(&mut session, &image, &mut cleanup);

    match bootstrap {
        Ok(()) => {
            // The marker now owns the machine lifecycle. Do not let the
            // stack-scoped cleanup guard remove a successfully leased session.
            cleanup.disarm();
            print_active_session_created(&session, ttl);
            Ok(ExitCode::SUCCESS)
        }
        Err(error) => {
            let machine_cleanup = cleanup.cleanup();
            let storage_cleanup = if machine_cleanup.is_ok() {
                remove_active_session_storage(&session)
            } else {
                Ok(())
            };
            match (machine_cleanup, storage_cleanup) {
                (Err(cleanup_error), _) => Err(cleanup_error).context(format!(
                    "VAT could not reach a terminal-safe recovery state for leased Apple K3s session {}; its private marker remains at {}. Run `vat k8s session delete {}` after resolving the runtime",
                    session.metadata.id,
                    session.marker_path.display(),
                    session.metadata.id,
                )),
                (Ok(()), Err(storage_error)) => Err(storage_error).context(format!(
                    "VAT removed the owned machine for failed K3s session {} but could not remove its private credential directory {}",
                    session.metadata.id,
                    session.directory.display(),
                )),
                (Ok(()), Ok(())) => Err(error),
            }
        }
    }
}

/// Execute one host command against a still-valid leased K3s session. Every
/// child is bounded by the remaining lease; an explicit timeout may narrow it.
pub fn session_exec(
    id: String,
    command: Vec<String>,
    json: bool,
    timeout_seconds: Option<u64>,
) -> Result<ExitCode> {
    session_exec::run(id, command, json, timeout_seconds)
}

/// Forward exactly one active lease Service to 127.0.0.1 for one foreground
/// host child. The child receives endpoint metadata, never the kubeconfig.
pub fn session_port_forward(args: ActiveSessionPortForwardArgs) -> Result<ExitCode> {
    port_forward::run(args)
}

/// Import one locally verified Apple Container image into the K3s `k8s.io`
/// namespace of an active lease. The archive never leaves VAT's private
/// session storage and is removed from both host and guest before success is
/// reported. This deliberately accepts an image reference, not an arbitrary
/// host tarball, so VAT can inspect the exact local source before transfer.
pub fn session_image_load(args: ActiveSessionImageLoadArgs) -> Result<ExitCode> {
    let platform = require_k3s_guest_platform(&args.platform)?;
    let session = read_active_session(&args.id)?;
    require_active_session_lease(&session)?;
    let _operation = port_forward::SessionOperationLock::acquire(&session)?;
    port_forward::reconcile(&session, &_operation)?;
    session_exec::reconcile(&session, &_operation)?;
    ensure_apple_container()?;
    let backing = validate_active_session_backing(&session)?;
    let credentials = SessionKubeconfig::open(&session.directory);
    credentials.validate()?;
    let endpoint = backing.api_endpoint()?;
    verify_host_api(&credentials, &endpoint)?;

    let source = inspect_local_image_variant(&args.image, platform)?;
    let staging = PrivateImageStaging::new(&session.directory)?;
    let archive = staging.archive_path().to_path_buf();
    let guest_archive = guest_image_archive_path(&session.metadata.id);
    let mut guest_archive_may_exist = false;
    let transfer = (|| {
        save_local_image_archive(&args.image, platform, &archive)?;
        let rechecked = inspect_local_image_variant(&args.image, platform)?;
        if rechecked != source {
            bail!(
                "Apple Container image reference {:?} changed while VAT prepared its private archive; retry after the local image store is stable",
                args.image
            );
        }
        guest_archive_may_exist = true;
        copy_local_image_archive(&backing, &archive, &guest_archive)?;
        import_guest_image(&backing, &guest_archive, &source.canonical_reference)?;
        Ok::<(), anyhow::Error>(())
    })();
    let guest_cleanup = if guest_archive_may_exist {
        remove_guest_image_archive(&backing, &guest_archive)
    } else {
        Ok(())
    };
    let host_cleanup = staging.close();

    match (transfer, guest_cleanup, host_cleanup) {
        (Ok(()), Ok(()), Ok(())) => {
            print_active_session_image_loaded(&session, &args.image, &source);
            Ok(ExitCode::SUCCESS)
        }
        (_, Err(cleanup_error), _) => Err(cleanup_error).context(format!(
            "VAT could not remove the temporary image archive from the owned K3s guest for session {}; the lease remains active but VAT will not claim a completed image load. Delete the session with `vat k8s session delete {}` if guest cleanup cannot be recovered",
            session.metadata.id, session.metadata.id,
        )),
        (_, Ok(()), Err(cleanup_error)) => Err(cleanup_error).context(format!(
            "VAT could not remove the private host image archive for K3s session {}; the lease remains active but VAT will not claim a completed image load",
            session.metadata.id,
        )),
        (Err(error), Ok(()), Ok(())) => Err(error),
    }
}

/// Show lease state without revealing the private credential path or contents.
pub fn session_status(id: String) -> Result<ExitCode> {
    let session = read_active_session(&id)?;
    let expired = active_session_expired(&session.metadata);
    let machine_state = if session.metadata.state == "active" && !expired {
        ensure_apple_container()?;
        match inspect_machine_presence(&session.metadata.name, Duration::from_secs(15))? {
            MachinePresence::Present => "present",
            MachinePresence::Absent => "absent",
            MachinePresence::Unknown => "unknown",
        }
    } else {
        "not_checked"
    };
    print_active_session_status(
        &session,
        expired,
        machine_state,
        port_forward::status(&session),
        None,
    );
    Ok(ExitCode::SUCCESS)
}

/// Verify an active leased session's owned Kubernetes API without changing its
/// lifecycle. This intentionally does not reconcile a retained port-forward
/// marker: a recovery-required forward is already a state that needs an
/// explicit recovery operation, not a status probe that might race it.
pub fn session_status_verify_api(id: String) -> Result<ExitCode> {
    let session = read_active_session(&id)?;
    let expired = active_session_expired(&session.metadata);
    let initial_port_forward = port_forward::status(&session);

    if session.metadata.state != "active" || expired {
        print_active_session_status(
            &session,
            expired,
            "not_checked",
            initial_port_forward,
            Some(SessionApiVerification::NotChecked),
        );
        return Ok(ExitCode::SUCCESS);
    }

    // A retained or unreadable port-forward marker may represent a foreground
    // kubectl process or a recovery state. Do not acquire its operation lock,
    // inspect the machine, or touch credentials from an API-status request.
    if initial_port_forward != "none" {
        print_active_session_status(
            &session,
            false,
            "not_checked",
            initial_port_forward,
            Some(SessionApiVerification::NotChecked),
        );
        return Ok(ExitCode::SUCCESS);
    }

    // Serialize the check with exec, image load, and port-forward. This is the
    // existing private, CLOEXEC flock; it neither reconciles nor changes the
    // session marker.
    let _operation = port_forward::SessionOperationLock::acquire(&session)?;
    let port_forward = port_forward::status(&session);
    let expired = active_session_expired(&session.metadata);
    // The command may have waited behind a foreground session operation. Do
    // not use the expiry result from before the lock: a lease that crossed its
    // deadline while waiting must not start a new API verification.
    if expired {
        print_active_session_status(
            &session,
            true,
            "not_checked",
            port_forward,
            Some(SessionApiVerification::NotChecked),
        );
        return Ok(ExitCode::SUCCESS);
    }
    if port_forward != "none" {
        print_active_session_status(
            &session,
            false,
            "not_checked",
            port_forward,
            Some(SessionApiVerification::NotChecked),
        );
        return Ok(ExitCode::SUCCESS);
    }

    ensure_apple_container()?;
    let backing = validate_active_session_backing(&session)?;
    let credentials = SessionKubeconfig::open(&session.directory);
    credentials.validate().map_err(|_| {
        anyhow::anyhow!(
            "K3s session {} has unavailable or unsafe private credentials; VAT did not modify the lease",
            session.metadata.id,
        )
    })?;
    let endpoint = backing.api_endpoint()?;
    // Exact machine inspection can itself be bounded but nontrivial. Recheck
    // immediately before kubectl so a lease that expires during that work is
    // reported as expired without opening the API or changing session state.
    if active_session_expired(&session.metadata) {
        print_active_session_status(
            &session,
            true,
            "not_checked",
            "none",
            Some(SessionApiVerification::NotChecked),
        );
        return Ok(ExitCode::SUCCESS);
    }
    verify_host_api(&credentials, &endpoint).map_err(|_| {
        anyhow::anyhow!(
            "K3s session {} API verification did not reach its exact owned K3s API; VAT did not modify the lease",
            session.metadata.id,
        )
    })?;

    print_active_session_status(
        &session,
        false,
        "present",
        "none",
        Some(SessionApiVerification::Reachable),
    );
    Ok(ExitCode::SUCCESS)
}

/// Delete one exact owned leased session and its private credentials.
pub fn session_delete(id: String) -> Result<ExitCode> {
    let session = read_active_session(&id)?;
    let _operation = port_forward::SessionOperationLock::acquire(&session)?;
    port_forward::reconcile(&session, &_operation)?;
    session_exec::reconcile(&session, &_operation)?;
    ensure_apple_container()?;
    delete_active_session(&session)?;
    println!(
        "{}",
        serde_json::json!({
            "type": "vat_k8s_session_delete",
            "id": session.metadata.id,
            "machine": session.metadata.name,
            "cleanup": "confirmed",
            "terminal": "cleaned_up",
        })
    );
    Ok(ExitCode::SUCCESS)
}

/// Reclaim only expired leases and abandoned creations. There is no background
/// daemon, so an active lease is never removed merely because its creator PID
/// exited after a successful `session create`.
pub fn session_cleanup(json: bool) -> Result<ExitCode> {
    let directory = active_session_directory()?;
    if !directory.exists() {
        return print_active_session_cleanup_result(json, Vec::new(), Vec::new(), Vec::new());
    }
    ensure_apple_container()?;

    let mut removed = Vec::new();
    let mut active = Vec::new();
    let mut failed = Vec::new();
    for entry in fs::read_dir(&directory)
        .with_context(|| format!("read leased K3s session directory {}", directory.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !entry.file_type()?.is_dir() {
            failed.push(format!("{}: expected a session directory", path.display()));
            continue;
        }
        let id = match path.file_name().and_then(|name| name.to_str()) {
            Some(id) => id.to_string(),
            None => {
                failed.push(format!("{}: non-UTF-8 session directory", path.display()));
                continue;
            }
        };
        let session = match read_active_session(&id) {
            Ok(session) => session,
            Err(error) => {
                failed.push(format!("{}: {error:#}", path.display()));
                continue;
            }
        };
        let reclaimable = active_session_expired(&session.metadata)
            || (session.metadata.state == "creating"
                && !process_is_alive(session.metadata.creator_pid));
        if !reclaimable {
            active.push(session.metadata.id);
            continue;
        }
        let operation = match port_forward::SessionOperationLock::acquire(&session) {
            Ok(operation) => operation,
            Err(error) => {
                active.push(session.metadata.id);
                let _ = error;
                continue;
            }
        };
        if let Err(error) = port_forward::reconcile(&session, &operation) {
            failed.push(format!("{}: {error:#}", session.metadata.id));
            drop(operation);
            continue;
        }
        if let Err(error) = session_exec::reconcile(&session, &operation) {
            failed.push(format!("{}: {error:#}", session.metadata.id));
            drop(operation);
            continue;
        }
        match delete_active_session(&session) {
            Ok(()) => removed.push(session.metadata.id),
            Err(error) => failed.push(format!("{}: {error:#}", session.metadata.id)),
        }
    }
    print_active_session_cleanup_result(json, removed, active, failed)
}

fn print_active_session_created(session: &ActiveSession, ttl: Duration) {
    let id = &session.metadata.id;
    println!(
        "{}",
        serde_json::json!({
            "type": "vat_k8s_session",
            "id": id,
            "machine": session.metadata.name,
            "image": session.metadata.image,
            "state": "active",
            "ttl_seconds": ttl.as_secs(),
            "expires_unix_ms": session.metadata.expires_unix_ms,
            "next": active_session_exec_next(id),
        })
    );
}

/// Put an additive text-mode terminal record on its own line even when an
/// inherited foreground child wrote its last byte without a trailing newline.
/// JSON modes do not use this helper: they own stdout exclusively and emit one
/// document without replaying child streams.
pub(super) fn print_terminal_record(record: serde_json::Value) {
    println!();
    println!("{record}");
}

fn print_active_session_exec_result(session: &ActiveSession, status: ExitStatus) {
    print_terminal_record(serde_json::json!({
        "type": "vat_k8s_session_exec",
        "id": session.metadata.id,
        "state": "active",
        "child_exit_code": status.code().unwrap_or(1),
        "next": format!("vat k8s session status {}", session.metadata.id),
    }));
}

fn print_active_session_image_loaded(
    session: &ActiveSession,
    requested_image: &str,
    source: &LocalImageVariant,
) {
    println!(
        "{}",
        serde_json::json!({
            "type": "vat_k8s_session_image_load",
            "id": session.metadata.id,
            "state": "active",
            "image": requested_image,
            "canonical_image": source.canonical_reference,
            "platform": source.platform,
            "source_digest": source.source_digest,
            "variant_digest": source.variant_digest,
            "next": active_session_exec_next(&session.metadata.id),
        })
    );
}

fn print_active_session_status(
    session: &ActiveSession,
    expired: bool,
    machine_state: &str,
    port_forward: &str,
    api_verification: Option<SessionApiVerification>,
) {
    let id = &session.metadata.id;
    let state = if expired {
        "expired"
    } else {
        session.metadata.state.as_str()
    };
    let next = if expired || session.metadata.state != "active" {
        format!("vat k8s session cleanup")
    } else {
        active_session_exec_next(id)
    };
    let mut result = serde_json::json!({
        "type": "vat_k8s_session_status",
        "id": id,
        "machine": session.metadata.name,
        "state": state,
        "machine_state": machine_state,
        "port_forward": port_forward,
        "image": session.metadata.image,
        "expires_unix_ms": session.metadata.expires_unix_ms,
        "remaining_seconds": active_session_remaining_seconds(&session.metadata),
        "next": next,
    });
    if let Some(api_verification) = api_verification {
        let result = result
            .as_object_mut()
            .expect("K3s session status JSON result is an object");
        result.insert(
            "api_checked".to_string(),
            serde_json::Value::Bool(api_verification.checked()),
        );
        result.insert(
            "api_state".to_string(),
            serde_json::Value::String(api_verification.state().to_string()),
        );
    }
    println!("{result}");
}

#[derive(Clone, Copy)]
enum SessionApiVerification {
    Reachable,
    NotChecked,
}

impl SessionApiVerification {
    const fn checked(self) -> bool {
        matches!(self, Self::Reachable)
    }

    const fn state(self) -> &'static str {
        match self {
            Self::Reachable => "reachable",
            Self::NotChecked => "not_checked",
        }
    }
}

fn print_active_session_cleanup_result(
    json: bool,
    removed: Vec<String>,
    active: Vec<String>,
    failed: Vec<String>,
) -> Result<ExitCode> {
    let success = failed.is_empty();
    let result = serde_json::json!({
        "type": "vat_k8s_session_cleanup",
        "removed": removed,
        "active": active,
        "failed": failed,
        "next": if success {
            "vat k8s session create --ttl 30m"
        } else {
            "vat k8s session cleanup"
        },
    });
    if json {
        println!("{}", serde_json::to_string(&result)?);
    } else {
        println!("{}", serde_json::to_string_pretty(&result)?);
    }
    Ok(if success {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn active_session_exec_next(id: &str) -> String {
    format!("vat k8s session exec {id} -- kubectl get nodes")
}

fn parse_active_session_ttl(source: &str) -> Result<Duration> {
    let source = source.trim();
    let (number, multiplier) = match source.chars().last() {
        Some('s') => (&source[..source.len() - 1], 1_u64),
        Some('m') => (&source[..source.len() - 1], 60_u64),
        Some('h') => (&source[..source.len() - 1], 60 * 60),
        Some(character) if character.is_ascii_digit() => (source, 1_u64),
        _ => bail!(
            "invalid K3s session TTL {source:?}; use a positive whole number with s, m, or h (for example 30m)"
        ),
    };
    let seconds = number
        .parse::<u64>()
        .with_context(|| format!("parse K3s session TTL {source:?}"))?
        .checked_mul(multiplier)
        .context("K3s session TTL overflows seconds")?;
    let ttl = Duration::from_secs(seconds);
    if ttl < ACTIVE_SESSION_MIN_TTL || ttl > ACTIVE_SESSION_MAX_TTL {
        bail!(
            "K3s session TTL must be between {}s and {}s (received {}s)",
            ACTIVE_SESSION_MIN_TTL.as_secs(),
            ACTIVE_SESSION_MAX_TTL.as_secs(),
            ttl.as_secs()
        );
    }
    Ok(ttl)
}

fn print_cleanup_result(
    json: bool,
    removed: Vec<String>,
    active: Vec<String>,
    failed: Vec<String>,
) -> Result<ExitCode> {
    let success = failed.is_empty();
    let result = serde_json::json!({
        "type": "vat_k8s_ephemeral_cleanup",
        "removed": removed,
        "active": active,
        "failed": failed,
        "next": if success {
            "vat k8s ephemeral run -- kubectl get nodes"
        } else {
            "vat k8s ephemeral cleanup"
        },
    });
    if json {
        println!("{}", serde_json::to_string(&result)?);
    } else {
        println!("{}", serde_json::to_string_pretty(&result)?);
    }
    Ok(if success {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn print_image_result(status: &str, image: &str) {
    println!(
        "{}",
        serde_json::json!({
            "type": "vat_k8s_ephemeral_image",
            "status": status,
            "image": image,
            "next": "vat k8s ephemeral run -- kubectl get nodes",
        })
    );
}

fn print_session_result(marker: &SessionMarker, image: &str, status: ExitStatus) {
    print_terminal_record(serde_json::json!({
        "type": "vat_k8s_ephemeral_result",
        "machine": marker.name.as_str(),
        "image": image,
        "child_exit_code": status.code().unwrap_or(1),
        "cleanup": "confirmed",
        "terminal": "cleaned_up",
    }));
}

/// Allocate the exact owned machine name, preserving the recovery marker's
/// conservative create state until the caller atomically records success.
fn create_owned_machine(name: &str, image: &str, cleanup: &mut MachineCleanup) -> Result<()> {
    cleanup.mark_create_attempted();
    let create_args = strings(&[
        "machine",
        "create",
        "--name",
        name,
        "--home-mount",
        "none",
        "--cpus",
        "2",
        "--memory",
        "4G",
        image,
    ]);
    let created = match run_bounded("container", &create_args, MACHINE_CREATE_TIMEOUT, &[], &[]) {
        Ok(output) => output,
        Err(error) => {
            // A client-side timeout can still mean Apple's daemon allocated the
            // exact name. Keep the marker uncertain until a later recovery can
            // establish a terminal backend state.
            cleanup.record_create_result(false);
            return Err(error).context("create owned Apple K3s machine");
        }
    };
    cleanup.record_create_result(created.status.success());
    if !created.status.success() {
        bail!(
            "Apple Container failed to create owned K3s machine {name}: {}",
            command_failure(&created)
        );
    }
    Ok(())
}

fn run_ephemeral_session(
    marker: &mut SessionMarker,
    image: &str,
    program: &str,
    program_args: &[String],
    cleanup: &mut MachineCleanup,
) -> Result<ExitStatus> {
    create_owned_machine(&marker.name, image, cleanup)?;
    mark_session_create_confirmed(marker)?;

    let backing = wait_for_backing_container(&marker.name)?;
    bootstrap_k3s_guest(&marker.name, &backing)?;

    let credentials = PrivateKubeconfig::new()?;
    let session_result = (|| {
        copy_kubeconfig(&backing, &credentials)?;
        verify_host_api(&credentials, &backing.api_endpoint()?)?;
        let status = run_host_command(
            program,
            program_args,
            &credentials,
            &backing.api_endpoint()?,
        )?;
        Ok::<ExitStatus, anyhow::Error>(status)
    })();
    let credential_cleanup = credentials.close();

    match (session_result, credential_cleanup) {
        (_, Err(cleanup_error)) => Err(cleanup_error).context(
            "VAT could not remove the private K3s kubeconfig/cache before machine cleanup",
        ),
        (Err(error), Ok(())) => Err(error),
        (Ok(status), Ok(())) => Ok(status),
    }
}

fn ensure_apple_container() -> Result<()> {
    if !sandbox::microvm::available() {
        bail!(
            "Apple Container CLI not found on PATH; install Apple's container CLI (for example brew install container) and retry"
        );
    }
    if !sandbox::microvm::system_up() {
        sandbox::microvm::ensure_system_started(Duration::from_secs(30))
            .map_err(anyhow::Error::msg)?;
    }
    Ok(())
}

fn default_machine_image() -> String {
    let digest = blake3::hash(MACHINE_ASSET.as_bytes()).to_hex();
    format!("local/vat-k8s-ephemeral:asset-{}", &digest[..12])
}

fn preflight_image(image: &str) -> Result<()> {
    if image_exists(image)? {
        return Ok(());
    }
    if image == default_machine_image() {
        bail!(
            "VAT's ephemeral K3s machine image {image:?} is absent from Apple Container. Build it explicitly with vat k8s ephemeral image build, then retry"
        );
    }
    bail!(
        "requested ephemeral K3s machine image {image:?} is absent from Apple Container; build or pull it into the Apple Container image store before retrying"
    )
}

fn image_exists(image: &str) -> Result<bool> {
    let args = strings(&["image", "inspect", image]);
    Ok(
        run_bounded("container", &args, Duration::from_secs(30), &[], &[])?
            .status
            .success(),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalImageVariant {
    canonical_reference: String,
    platform: String,
    source_digest: String,
    variant_digest: String,
}

/// Private transient staging under a validated 0700 session directory. The
/// directory's random name is never printed, and callers must explicitly close
/// it so cleanup failures turn into a visible error instead of Drop best effort.
struct PrivateImageStaging {
    directory: TempDir,
    archive: PathBuf,
}

impl PrivateImageStaging {
    fn new(session_directory: &Path) -> Result<Self> {
        require_private_directory(session_directory, "leased K3s session storage")?;
        let directory = tempfile::Builder::new()
            .prefix("image-load-")
            .tempdir_in(session_directory)
            .context("create private K3s local-image staging directory")?;
        restrict_dir(directory.path())?;
        Ok(Self {
            archive: directory.path().join("image.oci.tar"),
            directory,
        })
    }

    fn archive_path(&self) -> &Path {
        &self.archive
    }

    fn close(self) -> Result<()> {
        self.directory
            .close()
            .context("remove private K3s local-image staging directory")
    }
}

fn require_k3s_guest_platform(platform: &str) -> Result<&'static str> {
    if platform == K3S_GUEST_PLATFORM {
        Ok(K3S_GUEST_PLATFORM)
    } else {
        bail!(
            "VAT's current Apple K3s image-load path supports only {K3S_GUEST_PLATFORM:?}; it will not claim cross-platform image delivery for requested platform {platform:?}"
        );
    }
}

fn inspect_local_image_variant(image: &str, platform: &str) -> Result<LocalImageVariant> {
    require_safe_image_reference(image, "K3s local-image reference")?;
    let args = strings(&["image", "inspect", image]);
    let output = require_success(
        "inspect local Apple Container image before K3s delivery",
        run_bounded("container", &args, Duration::from_secs(30), &[], &[])?,
    )?;
    parse_local_image_variant(&output.stdout, image, platform)
}

fn parse_local_image_variant(
    source: &[u8],
    requested_image: &str,
    platform: &str,
) -> Result<LocalImageVariant> {
    let document: serde_json::Value = serde_json::from_slice(source)
        .context("Apple Container image inspect did not return JSON")?;
    let records = document
        .as_array()
        .context("Apple Container image inspect did not return an image array")?;
    if records.len() != 1 {
        bail!(
            "Apple Container image inspect for {requested_image:?} returned {} records; VAT requires exactly one locally verified image",
            records.len()
        );
    }
    let record = &records[0];
    let canonical_reference = record
        .pointer("/configuration/name")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .context("Apple Container image inspect did not return configuration.name")?
        .to_string();
    require_safe_image_reference(
        &canonical_reference,
        "Apple Container image inspect configuration.name",
    )?;
    // Apple Container's top-level `id` is a bare hex string in current
    // releases, while the OCI descriptor carries the stable digest form. Use
    // that descriptor rather than assuming a Docker-style `sha256:` ID.
    let source_digest = parse_sha256_digest(
        record
            .pointer("/configuration/descriptor/digest")
            .and_then(serde_json::Value::as_str)
            .context("Apple Container image inspect did not return an OCI descriptor digest")?,
        "Apple Container image inspect OCI descriptor",
    )?;
    let (os, architecture) = platform.split_once('/').with_context(|| {
        format!("VAT internal image platform {platform:?} is not os/architecture")
    })?;
    let variants = record
        .get("variants")
        .and_then(serde_json::Value::as_array)
        .context("Apple Container image inspect did not return variants")?;
    let matches: Vec<&serde_json::Value> = variants
        .iter()
        .filter(|variant| {
            variant
                .pointer("/platform/os")
                .and_then(serde_json::Value::as_str)
                == Some(os)
                && variant
                    .pointer("/platform/architecture")
                    .and_then(serde_json::Value::as_str)
                    == Some(architecture)
        })
        .collect();
    if matches.len() != 1 {
        bail!(
            "local Apple Container image {requested_image:?} has {} variants for {platform:?}; VAT requires exactly one guest-compatible variant",
            matches.len()
        );
    }
    let variant_digest = parse_sha256_digest(
        matches[0]
            .get("digest")
            .and_then(serde_json::Value::as_str)
            .context("Apple Container image inspect did not return a variant digest")?,
        "Apple Container image inspect variant",
    )?;
    Ok(LocalImageVariant {
        canonical_reference,
        platform: platform.to_string(),
        source_digest,
        variant_digest,
    })
}

fn require_safe_image_reference(reference: &str, label: &str) -> Result<()> {
    if reference.is_empty()
        || reference.starts_with('-')
        || reference
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        bail!(
            "{label} must be a non-empty, whitespace-free image reference that does not start with '-'"
        );
    }
    Ok(())
}

fn parse_sha256_digest(value: &str, label: &str) -> Result<String> {
    let Some(raw) = value.strip_prefix("sha256:") else {
        bail!("{label} must be a sha256 digest");
    };
    if raw.len() != 64 || !raw.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("{label} must contain exactly 64 hexadecimal sha256 characters");
    }
    Ok(format!("sha256:{raw}"))
}

fn save_local_image_archive(image: &str, platform: &str, archive: &Path) -> Result<()> {
    let output = archive
        .to_str()
        .context("private K3s image archive path is not UTF-8")?;
    let args = strings(&[
        "image",
        "save",
        "--platform",
        platform,
        "--output",
        output,
        image,
    ]);
    require_success(
        "save locally inspected Apple Container image as a private OCI archive",
        run_bounded("container", &args, K3S_IMAGE_SAVE_TIMEOUT, &[], &[])?,
    )?;
    let metadata = fs::symlink_metadata(archive)
        .with_context(|| format!("inspect private K3s image archive {}", archive.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!(
            "Apple Container image save did not produce a regular private archive at {}",
            archive.display()
        );
    }
    if metadata.len() == 0 {
        bail!("Apple Container image save produced an empty OCI archive");
    }
    if metadata.len() > K3S_IMAGE_ARCHIVE_MAX_BYTES {
        bail!(
            "Apple Container image archive is {} bytes, exceeding VAT's bounded {} byte K3s image-delivery limit",
            metadata.len(),
            K3S_IMAGE_ARCHIVE_MAX_BYTES,
        );
    }
    restrict_file(archive)?;
    require_private_file(archive, "private K3s image archive")
}

fn guest_image_archive_path(session_id: &str) -> String {
    format!("/tmp/vat-k8s-image-{session_id}.oci.tar")
}

fn copy_local_image_archive(
    backing: &BackingContainer,
    archive: &Path,
    guest_archive: &str,
) -> Result<()> {
    let source = archive
        .to_str()
        .context("private K3s image archive path is not UTF-8")?;
    let destination = format!("{}:{guest_archive}", backing.id);
    let args = strings(&["copy", source, &destination]);
    require_success(
        "copy private OCI image archive into the exact owned Apple K3s guest",
        run_bounded("container", &args, K3S_IMAGE_SAVE_TIMEOUT, &[], &[])?,
    )?;
    Ok(())
}

fn import_guest_image(
    backing: &BackingContainer,
    guest_archive: &str,
    canonical_reference: &str,
) -> Result<()> {
    // `guest_archive` is generated from a validated VAT id and both values are
    // passed as positional shell parameters, never interpolated into the script.
    const IMPORT_SCRIPT: &str = "set -eu; archive=\"$1\"; image=\"$2\"; test -f \"$archive\"; k3s ctr -n k8s.io images import \"$archive\"; k3s ctr -n k8s.io images inspect \"$image\" >/dev/null";
    let args = strings(&[
        "exec",
        &backing.id,
        "sh",
        "-ec",
        IMPORT_SCRIPT,
        "vat-k8s-image-load",
        guest_archive,
        canonical_reference,
    ]);
    require_success(
        "import locally verified image into the owned K3s k8s.io namespace",
        run_bounded("container", &args, K3S_IMAGE_IMPORT_TIMEOUT, &[], &[])?,
    )?;
    Ok(())
}

fn remove_guest_image_archive(backing: &BackingContainer, guest_archive: &str) -> Result<()> {
    let args = strings(&["exec", &backing.id, "rm", "-f", "--", guest_archive]);
    require_success(
        "remove temporary OCI image archive from the exact owned Apple K3s guest",
        run_bounded("container", &args, Duration::from_secs(30), &[], &[])?,
    )?;
    Ok(())
}

#[derive(Debug, Clone)]
struct BackingContainer {
    id: String,
    ip_address: String,
}

impl BackingContainer {
    fn api_endpoint(&self) -> Result<String> {
        let ip = IpAddr::from_str(&self.ip_address).with_context(|| {
            format!(
                "Apple Container returned invalid IP address {:?} for the owned machine",
                self.ip_address
            )
        })?;
        Ok(match ip {
            IpAddr::V4(_) => format!("https://{ip}:6443"),
            IpAddr::V6(_) => format!("https://[{ip}]:6443"),
        })
    }
}

fn wait_for_backing_container(name: &str) -> Result<BackingContainer> {
    let deadline = Instant::now() + MACHINE_READY_TIMEOUT;
    let last_error = loop {
        let args = strings(&["machine", "inspect", name]);
        let observed = match run_bounded("container", &args, Duration::from_secs(15), &[], &[]) {
            Ok(output) if output.status.success() => {
                match parse_backing_container(&output.stdout) {
                    Ok(backing) => return Ok(backing),
                    Err(error) => error.to_string(),
                }
            }
            Ok(output) => command_failure(&output),
            Err(error) => error.to_string(),
        };
        if Instant::now() >= deadline {
            break observed;
        }
        thread::sleep(Duration::from_millis(500));
    };
    bail!(
        "Apple Container did not expose a running backing container ID and IP for owned machine {name:?} within {} seconds: {last_error}",
        MACHINE_READY_TIMEOUT.as_secs()
    );
}

fn parse_backing_container(source: &[u8]) -> Result<BackingContainer> {
    let value: serde_json::Value = serde_json::from_slice(source)
        .context("Apple Container machine inspect did not return JSON")?;
    let record = match &value {
        serde_json::Value::Array(records) => records
            .first()
            .context("Apple Container machine inspect returned no records")?,
        serde_json::Value::Object(_) => &value,
        _ => bail!("Apple Container machine inspect returned neither an object nor an array"),
    };
    let status = record
        .get("status")
        .and_then(serde_json::Value::as_str)
        .context("Apple Container machine inspect did not return status")?;
    if !status.eq_ignore_ascii_case("running") {
        bail!("Apple Container machine inspect returned status {status:?}, not running");
    }
    let id = record
        .get("containerId")
        .and_then(serde_json::Value::as_str)
        .filter(|id| !id.is_empty())
        .context("Apple Container machine inspect did not return a backing containerId")?;
    let ip_address = record
        .get("ipAddress")
        .and_then(serde_json::Value::as_str)
        .filter(|ip| !ip.is_empty())
        .context("Apple Container machine inspect did not return an ipAddress")?;
    IpAddr::from_str(ip_address).with_context(|| {
        format!("Apple Container returned invalid machine ipAddress {ip_address:?}")
    })?;
    Ok(BackingContainer {
        id: id.to_string(),
        ip_address: ip_address.to_string(),
    })
}

fn guest_preflight(backing: &BackingContainer) -> Result<()> {
    let systemd = strings(&[
        "exec",
        &backing.id,
        "sh",
        "-ec",
        "set -eu; test \"$(cat /proc/1/comm)\" = systemd; state=\"$(systemctl is-system-running --wait || true)\"; test \"$state\" = running || test \"$state\" = degraded",
    ]);
    require_success(
        "verify systemd in the owned Apple K3s guest",
        run_bounded("container", &systemd, GUEST_PRECHECK_TIMEOUT, &[], &[])?,
    )?;
    let root = strings(&["exec", &backing.id, "id", "-u"]);
    let root = require_success(
        "verify root execution in the owned Apple K3s guest",
        run_bounded("container", &root, GUEST_PRECHECK_TIMEOUT, &[], &[])?,
    )?;
    if String::from_utf8_lossy(&root.stdout).trim() != "0" {
        bail!("owned Apple K3s guest does not expose root execution through container exec");
    }
    Ok(())
}

/// Run the three guest bootstrap stages once for both ephemeral and leased
/// sessions. If any stage fails, collect bounded evidence while the exact
/// owned machine is still available; the caller retains sole responsibility
/// for its existing exact-machine cleanup sequence.
fn bootstrap_k3s_guest(machine_name: &str, backing: &BackingContainer) -> Result<()> {
    (|| -> Result<()> {
        guest_preflight(backing)?;
        install_k3s(backing)?;
        wait_for_k3s_ready(backing)?;
        Ok(())
    })()
    .map_err(|error| {
        let diagnostics = collect_k3s_bootstrap_diagnostics(machine_name, backing);
        format_k3s_bootstrap_failure(error, diagnostics)
    })
}

/// Keep the failed bootstrap as the error source while rendering its root
/// cause before advisory diagnostics. `anyhow::Context` would print the
/// diagnostic context first under `{err:#}`, burying the actionable failure.
fn format_k3s_bootstrap_failure(primary: anyhow::Error, diagnostics: String) -> anyhow::Error {
    anyhow::Error::new(K3sBootstrapFailure {
        primary,
        diagnostics,
    })
}

#[derive(Debug)]
struct K3sBootstrapFailure {
    primary: anyhow::Error,
    diagnostics: String,
}

impl std::fmt::Display for K3sBootstrapFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:#}\n{}", self.primary, self.diagnostics)
    }
}

impl std::error::Error for K3sBootstrapFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.primary.as_ref())
    }
}

/// Gather only fixed, read-only diagnostics from a guest before it has exposed
/// a VAT kubeconfig to a host child. Each probe is bounded independently and
/// by the shared deadline, so failure evidence cannot delay exact cleanup.
fn collect_k3s_bootstrap_diagnostics(machine_name: &str, backing: &BackingContainer) -> String {
    const GUEST_SYSTEM_DIAGNOSTIC: &str = "set -eu; systemctl status k3s --no-pager --full || true; journalctl -b -u k3s --no-pager -n 24 || true";

    let deadline = Instant::now() + K3S_BOOTSTRAP_DIAGNOSTIC_TOTAL_TIMEOUT;
    let install_log = format!(
        "set -eu; if test -r {K3S_INSTALL_LOG}; then tail -n 32 {K3S_INSTALL_LOG}; else printf '%s\\n' 'vat-k3s-install log unavailable'; fi"
    );
    let mut diagnostics = Vec::with_capacity(6);
    let guest_install = [
        "exec",
        backing.id.as_str(),
        "sh",
        "-ec",
        install_log.as_str(),
    ];
    append_k3s_bootstrap_diagnostic(
        &mut diagnostics,
        deadline,
        "guest_install_log",
        &guest_install,
    );
    let guest_system = [
        "exec",
        backing.id.as_str(),
        "sh",
        "-ec",
        GUEST_SYSTEM_DIAGNOSTIC,
    ];
    append_k3s_bootstrap_diagnostic(
        &mut diagnostics,
        deadline,
        "guest_k3s_system",
        &guest_system,
    );
    let backing_logs = ["logs", backing.id.as_str()];
    append_k3s_bootstrap_diagnostic(
        &mut diagnostics,
        deadline,
        "backing_container_logs",
        &backing_logs,
    );
    let machine_logs = ["machine", "logs", machine_name];
    append_k3s_bootstrap_diagnostic(
        &mut diagnostics,
        deadline,
        "machine_boot_log",
        &machine_logs,
    );
    let machine_inspect = ["machine", "inspect", machine_name];
    append_k3s_bootstrap_diagnostic(
        &mut diagnostics,
        deadline,
        "machine_inspect",
        &machine_inspect,
    );
    let system_status = ["system", "status"];
    append_k3s_bootstrap_diagnostic(
        &mut diagnostics,
        deadline,
        "container_system_status",
        &system_status,
    );

    format!(
        "K3s bootstrap diagnostics (best effort, bounded before owned-machine cleanup):\n{}",
        diagnostics.join("\n")
    )
}

fn append_k3s_bootstrap_diagnostic(
    diagnostics: &mut Vec<String>,
    deadline: Instant,
    label: &str,
    args: &[&str],
) {
    let now = Instant::now();
    if now >= deadline {
        diagnostics.push(format!(
            "{label}: skipped because diagnostic budget expired"
        ));
        return;
    }
    let per_probe_deadline = now + K3S_BOOTSTRAP_DIAGNOSTIC_PROBE_TIMEOUT;
    let probe_deadline = if per_probe_deadline < deadline {
        per_probe_deadline
    } else {
        deadline
    };
    diagnostics.push(format!(
        "{label}: {}",
        container_diagnostic_until(args, probe_deadline)
    ));
}

fn install_k3s(backing: &BackingContainer) -> Result<()> {
    let endpoint = backing.api_endpoint()?;
    let install = k3s_install_script(endpoint_host(&endpoint)?);
    let args = strings(&["exec", &backing.id, "sh", "-ec", &install]);
    require_success(
        "install pinned K3s in the owned Apple guest",
        run_bounded("container", &args, K3S_INSTALL_TIMEOUT, &[], &[])?,
    )?;
    Ok(())
}

fn k3s_install_script(tls_san: &str) -> String {
    format!(
        "set -eu; log={K3S_INSTALL_LOG}; rm -f \"$log\"; {{ printf '%s\\n' 'vat-k3s-install: fetch-script'; curl -fsSL https://get.k3s.io -o /tmp/install-k3s.sh; printf '%s\\n' 'vat-k3s-install: run-installer'; INSTALL_K3S_VERSION={K3S_VERSION} INSTALL_K3S_EXEC='server --write-kubeconfig-mode 0600 --tls-san {}' sh /tmp/install-k3s.sh; printf '%s\\n' 'vat-k3s-install: version'; k3s --version; printf '%s\\n' 'vat-k3s-install: complete'; }} >> \"$log\" 2>&1; tail -n 32 \"$log\"",
        tls_san
    )
}

fn endpoint_host(endpoint: &str) -> Result<&str> {
    let without_scheme = endpoint
        .strip_prefix("https://")
        .context("K3s endpoint is missing https scheme")?;
    let host = without_scheme
        .strip_suffix(":6443")
        .context("K3s endpoint is missing API port")?;
    Ok(host.trim_start_matches('[').trim_end_matches(']'))
}

fn wait_for_k3s_ready(backing: &BackingContainer) -> Result<()> {
    let args = strings(&["exec", &backing.id, "sh", "-ec", K3S_READY_SCRIPT]);
    require_success(
        "wait for the owned K3s node to become Ready",
        run_bounded("container", &args, K3S_READY_TIMEOUT, &[], &[])?,
    )?;
    Ok(())
}

struct PrivateKubeconfig {
    directory: Option<TempDir>,
    root: PathBuf,
    kubeconfig: PathBuf,
    cache: PathBuf,
    home: PathBuf,
}

impl PrivateKubeconfig {
    fn new() -> Result<Self> {
        let directory = tempfile::Builder::new()
            .prefix("vat-k8s-ephemeral-")
            .tempdir()
            .context("create private K3s credential directory")?;
        restrict_dir(directory.path())?;
        let root = directory.path().to_path_buf();
        let cache = root.join("kubectl-cache");
        let home = root.join("home");
        fs::create_dir(&cache).with_context(|| format!("create {}", cache.display()))?;
        fs::create_dir(&home).with_context(|| format!("create {}", home.display()))?;
        restrict_dir(&cache)?;
        restrict_dir(&home)?;
        Ok(Self {
            directory: Some(directory),
            kubeconfig: root.join("kubeconfig"),
            root,
            cache,
            home,
        })
    }

    fn environment(&self, endpoint: &str) -> Vec<(OsString, OsString)> {
        vec![
            (
                OsString::from("KUBECONFIG"),
                self.kubeconfig.clone().into_os_string(),
            ),
            (
                OsString::from("VAT_K8S_CACHE_DIR"),
                self.cache.clone().into_os_string(),
            ),
            (
                OsString::from("VAT_K8S_API_SERVER"),
                OsString::from(endpoint),
            ),
            (OsString::from("VAT_K8S_EPHEMERAL"), OsString::from("1")),
            (OsString::from("HOME"), self.home.clone().into_os_string()),
        ]
    }

    fn close(mut self) -> Result<()> {
        let directory = self
            .directory
            .take()
            .context("private K3s credential directory was already closed")?;
        let root = self.root.clone();
        directory.close().with_context(|| {
            format!("remove private K3s credential directory {}", root.display())
        })?;
        if root.exists() {
            bail!(
                "private K3s credential directory {} remains after cleanup",
                root.display()
            );
        }
        Ok(())
    }
}

/// Private credential material for either a one-shot session or an explicit
/// leased session. The common host-command path deliberately accepts only this
/// narrow view, so it never learns where a durable marker is stored.
trait KubeconfigAccess {
    fn kubeconfig_path(&self) -> &Path;
    fn cache_path(&self) -> &Path;
    fn environment(&self, endpoint: &str) -> Vec<(OsString, OsString)>;
}

impl KubeconfigAccess for PrivateKubeconfig {
    fn kubeconfig_path(&self) -> &Path {
        &self.kubeconfig
    }

    fn cache_path(&self) -> &Path {
        &self.cache
    }

    fn environment(&self, endpoint: &str) -> Vec<(OsString, OsString)> {
        PrivateKubeconfig::environment(self, endpoint)
    }
}

/// Credential material retained only for the life of a validated leased
/// session. It is rooted beneath the session's 0700 directory and is checked
/// for symlink/permission surprises before any later host command receives it.
struct SessionKubeconfig {
    root: PathBuf,
    kubeconfig: PathBuf,
    cache: PathBuf,
    home: PathBuf,
}

impl SessionKubeconfig {
    fn create(session_directory: &Path) -> Result<Self> {
        let credentials = Self::open(session_directory);
        fs::create_dir(&credentials.root).with_context(|| {
            format!(
                "create leased K3s credential directory {}",
                credentials.root.display()
            )
        })?;
        restrict_dir(&credentials.root)?;
        fs::create_dir(&credentials.cache)
            .with_context(|| format!("create {}", credentials.cache.display()))?;
        fs::create_dir(&credentials.home)
            .with_context(|| format!("create {}", credentials.home.display()))?;
        restrict_dir(&credentials.cache)?;
        restrict_dir(&credentials.home)?;
        Ok(credentials)
    }

    fn open(session_directory: &Path) -> Self {
        let root = session_directory.join("credentials");
        Self {
            kubeconfig: root.join("kubeconfig"),
            cache: root.join("kubectl-cache"),
            home: root.join("home"),
            root,
        }
    }

    fn validate(&self) -> Result<()> {
        require_private_directory(&self.root, "leased K3s credential directory")?;
        require_private_file(&self.kubeconfig, "leased K3s kubeconfig")?;
        require_private_directory(&self.cache, "leased K3s kubectl cache")?;
        require_private_directory(&self.home, "leased K3s HOME")
    }

    fn environment(&self, endpoint: &str) -> Vec<(OsString, OsString)> {
        kubeconfig_environment(&self.kubeconfig, &self.cache, &self.home, endpoint)
    }
}

impl KubeconfigAccess for SessionKubeconfig {
    fn kubeconfig_path(&self) -> &Path {
        &self.kubeconfig
    }

    fn cache_path(&self) -> &Path {
        &self.cache
    }

    fn environment(&self, endpoint: &str) -> Vec<(OsString, OsString)> {
        SessionKubeconfig::environment(self, endpoint)
    }
}

fn kubeconfig_environment(
    kubeconfig: &Path,
    cache: &Path,
    home: &Path,
    endpoint: &str,
) -> Vec<(OsString, OsString)> {
    vec![
        (
            OsString::from("KUBECONFIG"),
            kubeconfig.to_path_buf().into_os_string(),
        ),
        (
            OsString::from("VAT_K8S_CACHE_DIR"),
            cache.to_path_buf().into_os_string(),
        ),
        (
            OsString::from("VAT_K8S_API_SERVER"),
            OsString::from(endpoint),
        ),
        (OsString::from("VAT_K8S_EPHEMERAL"), OsString::from("1")),
        (OsString::from("HOME"), home.to_path_buf().into_os_string()),
    ]
}

fn copy_kubeconfig(backing: &BackingContainer, credentials: &impl KubeconfigAccess) -> Result<()> {
    let source = format!("{}:{K3S_KUBECONFIG}", backing.id);
    let destination = credentials.kubeconfig_path().to_string_lossy().into_owned();
    let args = strings(&["copy", &source, &destination]);
    require_success(
        "copy private kubeconfig from the exact owned Apple guest",
        run_bounded("container", &args, MACHINE_READY_TIMEOUT, &[], &[])?,
    )?;
    rewrite_kubeconfig_server(credentials.kubeconfig_path(), &backing.api_endpoint()?)?;
    restrict_file(credentials.kubeconfig_path())?;
    Ok(())
}

fn rewrite_kubeconfig_server(path: &Path, endpoint: &str) -> Result<()> {
    const LOOPBACK_ENDPOINT: &str = "https://127.0.0.1:6443";
    let source = fs::read_to_string(path)
        .with_context(|| format!("read private copied kubeconfig {}", path.display()))?;
    let matches = source.matches(LOOPBACK_ENDPOINT).count();
    if matches != 1 {
        bail!(
            "private copied kubeconfig must contain exactly one {LOOPBACK_ENDPOINT} endpoint, found {matches}"
        );
    }
    fs::write(path, source.replacen(LOOPBACK_ENDPOINT, endpoint, 1))
        .with_context(|| format!("rewrite private kubeconfig endpoint {}", path.display()))?;
    Ok(())
}

fn verify_host_api(credentials: &impl KubeconfigAccess, endpoint: &str) -> Result<()> {
    let kubectl = port_forward::resolve_kubectl()?;
    let kubectl = kubectl
        .to_str()
        .context("resolved independent kubectl path is not UTF-8")?;
    let kubeconfig = credentials.kubeconfig_path().to_string_lossy().into_owned();
    let cache = credentials.cache_path().to_string_lossy().into_owned();
    let args = strings(&[
        "--kubeconfig",
        &kubeconfig,
        "--cache-dir",
        &cache,
        "--request-timeout=20s",
        "get",
        "nodes",
        "-o",
        "json",
    ]);
    require_success(
        "verify host kubectl reaches the owned disposable K3s API",
        run_bounded(
            kubectl,
            &args,
            HOST_API_TIMEOUT,
            &sensitive_environment(),
            &credentials.environment(endpoint),
        )?,
    )?;
    Ok(())
}

fn run_host_command(
    program: &str,
    args: &[String],
    credentials: &impl KubeconfigAccess,
    endpoint: &str,
) -> Result<ExitStatus> {
    let mut command = k8s_host_command(program)?;
    command
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    for key in sensitive_environment() {
        command.env_remove(key);
    }
    for (key, value) in credentials.environment(endpoint) {
        command.env(key, value);
    }
    command
        .status()
        .with_context(|| format!("run host command {program:?} against ephemeral K3s"))
}

/// Build one host command whose ordinary `kubectl` resolution is independent
/// of OrbStack. `kubectl` itself uses the canonical standalone binary, while
/// other tools (for example helm or a shell wrapper) receive a PATH whose
/// first directory contains that same binary. Explicit OrbStack kubectl paths
/// fail closed instead of silently preserving an undeclared desktop-runtime
/// dependency.
pub(super) fn k8s_host_command(program: &str) -> Result<Command> {
    let kubectl = port_forward::resolve_kubectl()?;
    let executable = resolve_k8s_host_program(program, &kubectl)?;
    let kubectl_directory = kubectl
        .parent()
        .context("resolved independent kubectl has no parent directory")?;
    let inherited_path = std::env::var_os("PATH")
        .context("PATH is not available to prepare the K3s host command")?;
    let mut paths = vec![kubectl_directory.to_path_buf()];
    paths.extend(std::env::split_paths(&inherited_path));
    let path = std::env::join_paths(paths)
        .context("join PATH with the independent kubectl directory for K3s host command")?;
    let mut command = Command::new(executable);
    command.env("PATH", path);
    Ok(command)
}

fn resolve_k8s_host_program(program: &str, kubectl: &Path) -> Result<PathBuf> {
    if program == "kubectl" {
        return Ok(kubectl.to_path_buf());
    }

    let candidate = Path::new(program);
    if candidate.file_name() == Some(OsStr::new("kubectl")) {
        if let Ok(resolved) = fs::canonicalize(candidate) {
            if port_forward::is_orbstack_managed_path(&resolved) {
                bail!(
                    "VAT refuses explicit OrbStack kubectl path {}; install an independent kubectl and invoke `kubectl` through PATH instead",
                    resolved.display()
                );
            }
        }
    }
    Ok(candidate.to_path_buf())
}

fn sensitive_environment() -> [&'static str; 10] {
    [
        "KUBECONFIG",
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "NO_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
        "no_proxy",
        "KUBECTL_PLUGINS_PATH",
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionMetadata {
    schema: String,
    name: String,
    image: String,
    pid: u32,
    created_unix_ms: u128,
    /// A marker starts uncertain because a killed/timed-out `machine create`
    /// client does not prove Apple's daemon stopped allocating the exact name.
    /// It becomes false only after the client reports success and the marker is
    /// atomically updated. Older markers default to the conservative state.
    #[serde(default = "default_create_uncertain")]
    create_uncertain: bool,
}

fn default_create_uncertain() -> bool {
    true
}

#[derive(Debug, Clone)]
struct SessionMarker {
    metadata: SessionMetadata,
    name: String,
    path: PathBuf,
}

/// Durable-on-disk state for a leased (not restartable) agent K3s session.
/// The credential itself is intentionally stored only in the adjacent 0700
/// directory, never serialized into this marker or printed to stdout.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ActiveSessionMetadata {
    schema: String,
    id: String,
    name: String,
    image: String,
    creator_pid: u32,
    created_unix_ms: u128,
    expires_unix_ms: u128,
    #[serde(default = "default_create_uncertain")]
    create_uncertain: bool,
    state: String,
    #[serde(default)]
    backing_id: Option<String>,
    #[serde(default)]
    api_endpoint: Option<String>,
}

#[derive(Debug, Clone)]
struct ActiveSession {
    metadata: ActiveSessionMetadata,
    directory: PathBuf,
    marker_path: PathBuf,
}

fn create_active_session_marker(image: &str, ttl: Duration) -> Result<ActiveSession> {
    let root = active_session_directory()?;
    fs::create_dir_all(&root)
        .with_context(|| format!("create leased K3s session directory {}", root.display()))?;
    restrict_dir(&root)?;

    let created_unix_ms = unix_millis();
    let ttl_ms = ttl.as_millis();
    let expires_unix_ms = created_unix_ms
        .checked_add(ttl_ms)
        .context("K3s session lease expiry overflows milliseconds")?;
    for attempt in 0..8 {
        let id = unique_active_session_id(attempt);
        let directory = root.join(&id);
        match fs::create_dir(&directory) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("create leased K3s session storage {}", directory.display())
                });
            }
        }
        if let Err(error) = restrict_dir(&directory) {
            let _ = fs::remove_dir(&directory);
            return Err(error);
        }
        let metadata = ActiveSessionMetadata {
            schema: ACTIVE_SESSION_SCHEMA.to_string(),
            id: id.clone(),
            name: active_session_machine_name(&id),
            image: image.to_string(),
            creator_pid: std::process::id(),
            created_unix_ms,
            expires_unix_ms,
            create_uncertain: true,
            state: "creating".to_string(),
            backing_id: None,
            api_endpoint: None,
        };
        let marker_path = directory.join("session.json");
        match write_new_marker(&marker_path, &metadata) {
            Ok(()) => {
                return Ok(ActiveSession {
                    metadata,
                    directory,
                    marker_path,
                });
            }
            Err(error) => {
                let _ = fs::remove_dir_all(&directory);
                return Err(error).with_context(|| {
                    format!("write leased K3s session marker {}", marker_path.display())
                });
            }
        }
    }
    bail!("could not allocate a unique VAT leased K3s session id")
}

fn read_active_session(id: &str) -> Result<ActiveSession> {
    if !valid_active_session_id(id) {
        bail!("invalid VAT K3s session id {id:?}");
    }
    let root = active_session_directory()?;
    let directory = root.join(id);
    let expected_directory = root.join(id);
    if directory != expected_directory {
        bail!("K3s session id does not resolve to its owned storage directory");
    }
    require_private_directory(&directory, "leased K3s session storage")?;
    let marker_path = directory.join("session.json");
    let bytes = fs::read(&marker_path)
        .with_context(|| format!("read leased K3s session marker {}", marker_path.display()))?;
    let metadata: ActiveSessionMetadata = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse leased K3s session marker {}", marker_path.display()))?;
    if metadata.schema != ACTIVE_SESSION_SCHEMA
        || metadata.id != id
        || !valid_active_session_id(&metadata.id)
        || metadata.name != active_session_machine_name(id)
        || !valid_active_machine_name(&metadata.name)
    {
        bail!("session marker is not a valid VAT-owned leased K3s record");
    }
    if marker_path.file_name().and_then(|name| name.to_str()) != Some("session.json") {
        bail!("leased K3s session marker has an unexpected filename");
    }
    if metadata.expires_unix_ms <= metadata.created_unix_ms {
        bail!("leased K3s session marker has a non-positive TTL");
    }
    match metadata.state.as_str() {
        "creating" if metadata.backing_id.is_none() && metadata.api_endpoint.is_none() => {}
        "active"
            if !metadata.create_uncertain
                && metadata
                    .backing_id
                    .as_deref()
                    .is_some_and(|id| !id.is_empty())
                && metadata
                    .api_endpoint
                    .as_deref()
                    .is_some_and(|endpoint| endpoint.starts_with("https://")) => {}
        _ => bail!("leased K3s session marker has an invalid lifecycle state"),
    }
    Ok(ActiveSession {
        metadata,
        directory,
        marker_path,
    })
}

fn mark_active_session_create_confirmed(session: &mut ActiveSession) -> Result<()> {
    if !session.metadata.create_uncertain {
        return Ok(());
    }
    let mut metadata = session.metadata.clone();
    metadata.create_uncertain = false;
    replace_session_marker(&session.marker_path, &metadata)?;
    session.metadata = metadata;
    Ok(())
}

fn mark_active_session_ready(
    session: &mut ActiveSession,
    backing: &BackingContainer,
) -> Result<()> {
    if session.metadata.create_uncertain {
        bail!("cannot activate a K3s session before machine create is confirmed");
    }
    let endpoint = backing.api_endpoint()?;
    let mut metadata = session.metadata.clone();
    metadata.state = "active".to_string();
    metadata.backing_id = Some(backing.id.clone());
    metadata.api_endpoint = Some(endpoint);
    replace_session_marker(&session.marker_path, &metadata)?;
    session.metadata = metadata;
    Ok(())
}

fn bootstrap_active_session(
    session: &mut ActiveSession,
    image: &str,
    cleanup: &mut MachineCleanup,
) -> Result<()> {
    create_owned_machine(&session.metadata.name, image, cleanup)?;
    mark_active_session_create_confirmed(session)?;
    let backing = wait_for_backing_container(&session.metadata.name)?;
    bootstrap_k3s_guest(&session.metadata.name, &backing)?;
    let credentials = SessionKubeconfig::create(&session.directory)?;
    copy_kubeconfig(&backing, &credentials)?;
    let endpoint = backing.api_endpoint()?;
    verify_host_api(&credentials, &endpoint)?;
    mark_active_session_ready(session, &backing)
}

fn require_active_session_lease(session: &ActiveSession) -> Result<()> {
    if session.metadata.state != "active" {
        let next = "vat k8s session cleanup";
        println!(
            "{}",
            serde_json::json!({
                "type": "error",
                "code": "k8s_session_not_active",
                "id": session.metadata.id,
                "state": session.metadata.state,
                "next": next,
            })
        );
        bail!(
            "K3s session {} is not active; run `{next}` to reconcile it",
            session.metadata.id
        );
    }
    if active_session_expired(&session.metadata) {
        let next = "vat k8s session cleanup";
        println!(
            "{}",
            serde_json::json!({
                "type": "error",
                "code": "k8s_session_expired",
                "id": session.metadata.id,
                "expires_unix_ms": session.metadata.expires_unix_ms,
                "next": next,
            })
        );
        bail!(
            "K3s session {} has expired; run `{next}` or `vat k8s session delete {}` before creating a new lease",
            session.metadata.id,
            session.metadata.id,
        );
    }
    Ok(())
}

fn validate_active_session_backing(session: &ActiveSession) -> Result<BackingContainer> {
    let args = strings(&["machine", "inspect", &session.metadata.name]);
    let output = require_success(
        "inspect the exact owned leased Apple K3s machine",
        run_bounded("container", &args, Duration::from_secs(15), &[], &[])?,
    )?;
    let backing = parse_backing_container(&output.stdout)?;
    let expected_id = session
        .metadata
        .backing_id
        .as_deref()
        .context("active K3s session marker is missing its inspected backing container ID")?;
    if backing.id != expected_id {
        bail!(
            "Apple Container backing ID for K3s session {} changed from {:?} to {:?}; VAT will not reuse its private kubeconfig. Delete the session with `vat k8s session delete {}`",
            session.metadata.id,
            expected_id,
            backing.id,
            session.metadata.id,
        );
    }
    let endpoint = backing.api_endpoint()?;
    if session.metadata.api_endpoint.as_deref() != Some(endpoint.as_str()) {
        bail!(
            "Apple Container API endpoint for K3s session {} changed; VAT will not reuse its private kubeconfig. Delete the session with `vat k8s session delete {}`",
            session.metadata.id,
            session.metadata.id,
        );
    }
    Ok(backing)
}

fn delete_active_session(session: &ActiveSession) -> Result<()> {
    let mut cleanup = MachineCleanup::recovery(
        session.metadata.name.clone(),
        session.metadata.create_uncertain,
    );
    cleanup.cleanup()?;
    remove_active_session_storage(session)
}

fn remove_active_session_storage(session: &ActiveSession) -> Result<()> {
    let root = active_session_directory()?;
    let expected = root.join(&session.metadata.id);
    if session.directory != expected {
        bail!("leased K3s session storage does not match its validated id");
    }
    let metadata = fs::symlink_metadata(&session.directory).with_context(|| {
        format!(
            "inspect leased K3s session storage {}",
            session.directory.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!(
            "refusing to remove non-directory leased K3s session storage {}",
            session.directory.display()
        );
    }
    fs::remove_dir_all(&session.directory).with_context(|| {
        format!(
            "remove leased K3s credentials and marker {}",
            session.directory.display()
        )
    })?;
    if session.directory.exists() {
        bail!(
            "leased K3s session storage {} remains after cleanup",
            session.directory.display()
        );
    }
    Ok(())
}

fn active_session_directory() -> Result<PathBuf> {
    Ok(paths::root()?.join("k8s-sessions"))
}

fn unique_active_session_id(attempt: u8) -> String {
    let fresh = crate::id::fresh();
    let suffix = fresh.strip_prefix("vat-").unwrap_or(&fresh);
    format!("k8s-{suffix}-{attempt}")
}

fn active_session_machine_name(id: &str) -> String {
    format!("{ACTIVE_SESSION_PREFIX}{id}")
}

fn valid_active_session_id(id: &str) -> bool {
    id.starts_with("k8s-")
        && id.len() <= 96
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_active_machine_name(name: &str) -> bool {
    name.starts_with(ACTIVE_SESSION_PREFIX)
        && name.len() <= 120
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn active_session_expired(metadata: &ActiveSessionMetadata) -> bool {
    active_session_expired_at(metadata, unix_millis())
}

fn active_session_expired_at(metadata: &ActiveSessionMetadata, now_unix_ms: u128) -> bool {
    now_unix_ms >= metadata.expires_unix_ms
}

fn active_session_remaining_seconds(metadata: &ActiveSessionMetadata) -> u128 {
    let remaining_ms = metadata.expires_unix_ms.saturating_sub(unix_millis());
    (remaining_ms + 999) / 1000
}

fn create_session_marker(image: &str) -> Result<SessionMarker> {
    let directory = session_directory()?;
    fs::create_dir_all(&directory).with_context(|| {
        format!(
            "create ephemeral K3s session directory {}",
            directory.display()
        )
    })?;
    restrict_dir(&directory)?;

    for attempt in 0..8 {
        let name = unique_machine_name(attempt);
        let path = directory.join(format!("{name}.json"));
        let metadata = SessionMetadata {
            schema: "vat.k8s.ephemeral.session.v1".to_string(),
            name: name.clone(),
            image: image.to_string(),
            pid: std::process::id(),
            created_unix_ms: unix_millis(),
            create_uncertain: true,
        };
        match write_new_marker(&path, &metadata) {
            Ok(()) => {
                return Ok(SessionMarker {
                    metadata,
                    name,
                    path,
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("write ephemeral K3s session marker {}", path.display())
                });
            }
        }
    }
    bail!("could not allocate a unique VAT ephemeral K3s machine name")
}

fn read_session_marker(path: &Path) -> Result<SessionMarker> {
    let bytes =
        fs::read(path).with_context(|| format!("read session marker {}", path.display()))?;
    let metadata: SessionMetadata = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse session marker {}", path.display()))?;
    if metadata.schema != "vat.k8s.ephemeral.session.v1" || !valid_owned_name(&metadata.name) {
        bail!("session marker is not a valid VAT-owned ephemeral K3s record");
    }
    let expected = format!("{}.json", metadata.name);
    if path.file_name().and_then(|name| name.to_str()) != Some(expected.as_str()) {
        bail!("session marker filename does not match its owned machine name");
    }
    Ok(SessionMarker {
        name: metadata.name.clone(),
        metadata,
        path: path.to_path_buf(),
    })
}

fn write_new_marker<T: Serialize>(path: &Path, metadata: &T) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    serde_json::to_writer(&mut file, metadata).map_err(std::io::Error::other)?;
    file.write_all(b"\n")?;
    file.sync_all()
}

/// Mark that the `machine create` client returned success without opening a
/// window where a crash could erase the conservative recovery record. The
/// replacement is atomic: a later recovery sees either the original uncertain
/// marker or the confirmed one, never a truncated JSON file.
fn mark_session_create_confirmed(marker: &mut SessionMarker) -> Result<()> {
    if !marker.metadata.create_uncertain {
        return Ok(());
    }
    let mut metadata = marker.metadata.clone();
    metadata.create_uncertain = false;
    replace_session_marker(&marker.path, &metadata)?;
    marker.metadata = metadata;
    Ok(())
}

fn replace_session_marker<T: Serialize>(path: &Path, metadata: &T) -> Result<()> {
    let directory = path
        .parent()
        .context("ephemeral K3s session marker has no parent directory")?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .context("ephemeral K3s session marker has no UTF-8 filename")?;
    let temporary = directory.join(format!(".{file_name}.{}.tmp", crate::id::fresh()));
    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .with_context(|| format!("create temporary session marker {}", temporary.display()))?;
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .with_context(|| {
                format!("restrict temporary session marker {}", temporary.display())
            })?;
        serde_json::to_writer(&mut file, metadata)
            .context("serialize confirmed ephemeral K3s session marker")?;
        file.write_all(b"\n")
            .context("terminate confirmed ephemeral K3s session marker")?;
        file.sync_all()
            .with_context(|| format!("sync temporary session marker {}", temporary.display()))?;
        drop(file);
        fs::rename(&temporary, path).with_context(|| {
            format!(
                "atomically replace ephemeral K3s session marker {} from {}",
                path.display(),
                temporary.display()
            )
        })?;
        restrict_file(path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn remove_session_marker(marker: &SessionMarker) -> Result<()> {
    fs::remove_file(&marker.path).with_context(|| {
        format!(
            "remove ephemeral K3s session marker {}",
            marker.path.display()
        )
    })?;
    Ok(())
}

fn session_directory() -> Result<PathBuf> {
    Ok(paths::root()?.join("k8s-ephemeral"))
}

fn unique_machine_name(attempt: u8) -> String {
    format!(
        "{MACHINE_PREFIX}{}-{}-{attempt}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    )
}

fn unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn valid_owned_name(name: &str) -> bool {
    name.starts_with(MACHINE_PREFIX)
        && name.len() <= 120
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

#[cfg(unix)]
fn process_is_alive(pid: u32) -> bool {
    let pid = pid as libc::pid_t;
    unsafe {
        libc::kill(pid, 0) == 0
            || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
    }
}

#[cfg(not(unix))]
fn process_is_alive(_pid: u32) -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MachinePresence {
    Present,
    Absent,
    Unknown,
}

struct MachineCleanup {
    name: String,
    create_attempted: bool,
    create_succeeded: Option<bool>,
    cleanup_attempted: bool,
}

impl MachineCleanup {
    fn new(name: String) -> Self {
        Self {
            name,
            create_attempted: false,
            create_succeeded: None,
            cleanup_attempted: false,
        }
    }

    /// Reconcile a marker from a prior process. An interrupted or older
    /// marker with no explicit create confirmation remains deliberately
    /// uncertain even if one inspect currently reports it absent.
    fn recovery(name: String, create_uncertain: bool) -> Self {
        Self {
            name,
            create_attempted: true,
            create_succeeded: Some(!create_uncertain),
            cleanup_attempted: false,
        }
    }

    fn mark_create_attempted(&mut self) {
        self.create_attempted = true;
    }

    fn record_create_result(&mut self, succeeded: bool) {
        self.create_succeeded = Some(succeeded);
    }

    /// Transfer lifecycle ownership to a validated leased-session marker.
    /// The guard must never remove that machine as its creating CLI exits.
    fn disarm(&mut self) {
        self.cleanup_attempted = true;
    }

    fn cleanup(&mut self) -> Result<()> {
        if !self.create_attempted || self.cleanup_attempted {
            return Ok(());
        }
        self.cleanup_attempted = true;
        let started = Instant::now();
        let deadline = started + CLEANUP_TIMEOUT;
        let create_uncertain = self.create_succeeded != Some(true);
        let mut last_error = String::from("owned machine was not yet confirmed absent");

        while Instant::now() < deadline {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let timeout = remaining.min(Duration::from_secs(15));
            match inspect_machine_presence(&self.name, timeout) {
                Ok(MachinePresence::Absent) => {
                    if create_uncertain {
                        bail!(
                            "Apple Container never confirmed completion of `machine create` for {}; the exact name is currently absent, but VAT cannot prove a late allocation will not appear. The recovery marker is deliberately retained until the backend exposes a terminal create/cancellation state",
                            self.name
                        );
                    }
                    return Ok(());
                }
                Ok(MachinePresence::Present) => {
                    let args = strings(&["machine", "delete", &self.name]);
                    match run_bounded("container", &args, timeout, &[], &[]) {
                        Ok(output) if output.status.success() => {}
                        Ok(output) => last_error = command_failure(&output),
                        Err(error) => last_error = error.to_string(),
                    }
                }
                Ok(MachinePresence::Unknown) => {
                    last_error = "Apple Container could not establish exact owned-machine presence"
                        .to_string();
                }
                Err(error) => last_error = error.to_string(),
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }
            thread::sleep(CLEANUP_RETRY_DELAY.min(remaining));
        }
        bail!(
            "could not confirm absence of exact owned Apple K3s machine {} within {} seconds: {last_error}",
            self.name,
            CLEANUP_TIMEOUT.as_secs()
        )
    }
}

impl Drop for MachineCleanup {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

fn inspect_machine_presence(name: &str, timeout: Duration) -> Result<MachinePresence> {
    let args = strings(&["machine", "inspect", name]);
    let output = run_bounded("container", &args, timeout, &[], &[])?;
    if output.status.success() {
        return Ok(MachinePresence::Present);
    }
    let diagnostic = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
    .to_ascii_lowercase();
    if diagnostic.contains("notfound:")
        && diagnostic.contains("container machine")
        && diagnostic.contains(&name.to_ascii_lowercase())
    {
        Ok(MachinePresence::Absent)
    } else {
        Ok(MachinePresence::Unknown)
    }
}

fn run_bounded(
    program: &str,
    args: &[String],
    timeout: Duration,
    remove_env: &[&str],
    environment: &[(OsString, OsString)],
) -> Result<Output> {
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for key in remove_env {
        command.env_remove(key);
    }
    for (key, value) in environment {
        command.env(key, value);
    }
    let mut child = command
        .spawn()
        .with_context(|| format!("spawn {program:?} {}", args.join(" ")))?;
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                return child
                    .wait_with_output()
                    .with_context(|| format!("collect {program:?} {}", args.join(" ")));
            }
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(100)),
            Ok(None) => {
                let _ = child.kill();
                let output = child
                    .wait_with_output()
                    .with_context(|| format!("collect timed-out {program:?} {}", args.join(" ")))?;
                bail!(
                    "{program:?} {} timed out after {} seconds (exit {:?})",
                    args.join(" "),
                    timeout.as_secs(),
                    output.status.code()
                );
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error)
                    .with_context(|| format!("wait for {program:?} {}", args.join(" ")));
            }
        }
    }
}

fn require_success(label: &str, output: Output) -> Result<Output> {
    if output.status.success() {
        Ok(output)
    } else {
        bail!("{label} failed: {}", command_failure(&output));
    }
}

fn command_failure(output: &Output) -> String {
    let message = String::from_utf8_lossy(&output.stderr);
    let message = message.trim();
    if message.is_empty() {
        format!("exit status {:?}", output.status.code())
    } else {
        let mut compact = message.replace('\n', " ");
        compact.truncate(600);
        format!("exit status {:?}: {compact}", output.status.code())
    }
}

fn exit_code(status: ExitStatus) -> ExitCode {
    let code = status
        .code()
        .or_else(|| status.signal().map(|signal| 128 + signal))
        .unwrap_or(1);
    ExitCode::from(code.clamp(0, 255) as u8)
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn restrict_dir(path: &Path) -> Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .with_context(|| format!("restrict private directory {}", path.display()))?;
    Ok(())
}

fn restrict_file(path: &Path) -> Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .with_context(|| format!("restrict private file {}", path.display()))?;
    Ok(())
}

fn require_private_directory(path: &Path, label: &str) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect {label} {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("{label} {} is not a real directory", path.display());
    }
    if metadata.permissions().mode() & 0o077 != 0 {
        bail!(
            "{label} {} is not private (expected mode 0700)",
            path.display()
        );
    }
    Ok(())
}

fn require_private_file(path: &Path, label: &str) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect {label} {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("{label} {} is not a real file", path.display());
    }
    if metadata.permissions().mode() & 0o077 != 0 {
        bail!(
            "{label} {} is not private (expected mode 0600)",
            path.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;

    #[test]
    fn embedded_machine_image_tag_is_asset_derived_and_has_systemd_base() {
        let image = default_machine_image();
        assert!(image.starts_with("local/vat-k8s-ephemeral:asset-"));
        assert!(MACHINE_ASSET.contains("FROM ubuntu:24.04"));
        assert!(MACHINE_ASSET.contains("systemd"));
    }

    #[test]
    fn k3s_readiness_script_retries_empty_node_api_before_failing() {
        assert!(K3S_READY_SCRIPT.contains("deadline="));
        assert!(K3S_READY_SCRIPT.contains("get nodes --no-headers"));
        assert!(K3S_READY_SCRIPT.contains("$2 == \"Ready\""));
        assert!(!K3S_READY_SCRIPT.contains("kubectl wait --for"));
    }

    #[test]
    fn inspect_parser_requires_running_backing_id_and_ip() {
        let parsed = parse_backing_container(
            br#"[{"status":"running","containerId":"owned-id","ipAddress":"192.168.64.17"}]"#,
        )
        .expect("valid inspected machine");
        assert_eq!(parsed.id, "owned-id");
        assert_eq!(parsed.api_endpoint().unwrap(), "https://192.168.64.17:6443");
        assert!(parse_backing_container(
            br#"{"status":"stopped","containerId":"id","ipAddress":"192.168.64.17"}"#
        )
        .is_err());
        assert!(
            parse_backing_container(br#"{"status":"running","ipAddress":"192.168.64.17"}"#)
                .is_err()
        );
        assert!(parse_backing_container(
            br#"{"status":"running","containerId":"id","ipAddress":"not-an-ip"}"#
        )
        .is_err());
    }

    #[test]
    fn kubeconfig_rewrite_requires_exactly_one_loopback_endpoint() {
        let directory = tempfile::tempdir().unwrap();
        let config = directory.path().join("kubeconfig");
        fs::write(&config, "server: https://127.0.0.1:6443\n").unwrap();
        rewrite_kubeconfig_server(&config, "https://192.168.64.17:6443").unwrap();
        assert_eq!(
            fs::read_to_string(&config).unwrap(),
            "server: https://192.168.64.17:6443\n"
        );
        fs::write(
            &config,
            "a: https://127.0.0.1:6443\nb: https://127.0.0.1:6443\n",
        )
        .unwrap();
        assert!(rewrite_kubeconfig_server(&config, "https://192.168.64.17:6443").is_err());
    }

    #[test]
    fn private_credentials_limit_environment_to_the_foreground_child() {
        let credentials = PrivateKubeconfig::new().unwrap();
        let environment = credentials.environment("https://192.168.64.17:6443");
        assert!(environment
            .iter()
            .any(|(key, _)| key == &OsString::from("KUBECONFIG")));
        assert!(environment
            .iter()
            .any(|(key, _)| key == &OsString::from("VAT_K8S_CACHE_DIR")));
        assert!(environment.iter().any(|(key, value)| {
            key == &OsString::from("HOME") && value == &credentials.home.clone().into_os_string()
        }));
        credentials.close().unwrap();
    }

    #[test]
    fn session_markers_only_accept_generated_owned_names() {
        assert!(valid_owned_name("vat-k8s-ephemeral-123-456-0"));
        assert!(!valid_owned_name("other-machine"));
        assert!(!valid_owned_name("vat-k8s-ephemeral-../other"));
        assert!(!valid_owned_name("VAT-k8s-ephemeral-123"));
    }

    #[test]
    fn leased_session_ids_and_ttls_are_bounded_and_machine_safe() {
        assert!(valid_active_session_id("k8s-abc123-0"));
        assert!(!valid_active_session_id("../k8s-abc"));
        assert!(valid_active_machine_name("vat-k8s-session-k8s-abc123-0"));
        assert!(!valid_active_machine_name("vat-k8s-session-../other"));
        assert_eq!(
            parse_active_session_ttl("90s").unwrap(),
            Duration::from_secs(90)
        );
        assert_eq!(
            parse_active_session_ttl("2m").unwrap(),
            Duration::from_secs(120)
        );
        assert_eq!(
            parse_active_session_ttl("1h").unwrap(),
            Duration::from_secs(3600)
        );
        assert!(parse_active_session_ttl("59s").is_err());
        assert!(parse_active_session_ttl("5h").is_err());
        assert!(parse_active_session_ttl("soon").is_err());
    }

    #[test]
    fn leased_session_api_status_is_limited_to_non_sensitive_verification_states() {
        assert!(SessionApiVerification::Reachable.checked());
        assert_eq!(SessionApiVerification::Reachable.state(), "reachable");
        assert!(!SessionApiVerification::NotChecked.checked());
        assert_eq!(SessionApiVerification::NotChecked.state(), "not_checked");
    }

    #[test]
    fn api_status_expiry_recheck_rejects_a_lease_that_crossed_its_deadline() {
        let metadata = ActiveSessionMetadata {
            schema: ACTIVE_SESSION_SCHEMA.to_string(),
            id: "k8s-expiry-test-0".to_string(),
            name: "vat-k8s-session-k8s-expiry-test-0".to_string(),
            image: "fixture/systemd:k3s".to_string(),
            creator_pid: 1,
            created_unix_ms: 100,
            expires_unix_ms: 200,
            create_uncertain: false,
            state: "active".to_string(),
            backing_id: Some("owned-backing".to_string()),
            api_endpoint: Some("https://192.168.64.17:6443".to_string()),
        };
        assert!(!active_session_expired_at(&metadata, 199));
        assert!(active_session_expired_at(&metadata, 200));
    }

    #[test]
    fn local_image_delivery_requires_one_verified_arm64_variant() {
        let image = parse_local_image_variant(
            br#"[{"configuration":{"name":"docker.io/library/alpine:3.20","descriptor":{"digest":"sha256:1111111111111111111111111111111111111111111111111111111111111111"}},"id":"index","variants":[{"digest":"sha256:2222222222222222222222222222222222222222222222222222222222222222","platform":{"os":"linux","architecture":"arm64","variant":"v8"}}]}]"#,
            "alpine:3.20",
            "linux/arm64",
        )
        .expect("single ARM64 variant");
        assert_eq!(image.canonical_reference, "docker.io/library/alpine:3.20");
        assert_eq!(image.platform, "linux/arm64");
        assert_eq!(
            image.source_digest,
            "sha256:1111111111111111111111111111111111111111111111111111111111111111"
        );
        assert_eq!(
            image.variant_digest,
            "sha256:2222222222222222222222222222222222222222222222222222222222222222"
        );
        assert!(require_k3s_guest_platform("linux/arm64").is_ok());
        assert!(require_k3s_guest_platform("linux/amd64").is_err());
        assert!(
            parse_local_image_variant(
                br#"[{"configuration":{"name":"alpine:3.20","descriptor":{"digest":"sha256:index"}},"id":"index","variants":[{"digest":"sha256:amd64","platform":{"os":"linux","architecture":"amd64"}}]}]"#,
                "alpine:3.20",
                "linux/arm64",
            )
            .is_err()
        );
        assert!(
            parse_local_image_variant(
                br#"[{"configuration":{"name":"alpine:3.20","descriptor":{"digest":"sha256:index"}},"id":"index","variants":[{"digest":"sha256:first","platform":{"os":"linux","architecture":"arm64"}},{"digest":"sha256:second","platform":{"os":"linux","architecture":"arm64"}}]}]"#,
                "alpine:3.20",
                "linux/arm64",
            )
            .is_err()
        );
    }

    #[test]
    fn child_exit_status_is_preserved() {
        assert_eq!(exit_code(ExitStatus::from_raw(42 << 8)), ExitCode::from(42));
        assert_eq!(
            exit_code(ExitStatus::from_raw(libc::SIGTERM)),
            ExitCode::from(128 + libc::SIGTERM as u8)
        );
    }
}
// HANDWRITE-END
