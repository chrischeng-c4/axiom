// SPEC-MANAGED: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-session-proxy" tracker="#1288" reason="Session-mode proxy needs generator primitives that do not exist yet.">
//! One session's pipeline, following the TD Logic flowchart exactly:
//! admission -> backend connect (via `BackendPool::acquire_fresh()`,
//! WI #1289: session-mode is now capacity-bounded through the same shared
//! pool transaction-mode uses) -> frame-aware startup relay -> auth
//! passthrough (opaque, never persisted) -> ready relay -> bidirectional
//! relay until `Terminate`/EOF/`FrameError`, then `BackendPool::release`
//! with [`crate::pool::LeaseDisposition::Close`] on every exit path (session
//! mode never reuses its backend connection).
//!
//! This module never inspects password/SASL payload bytes semantically: a
//! tagged `'p'` frame is decoded generically as [`crate::wire::PasswordMessage`]
//! and re-encoded byte-identically regardless of whether it is really a
//! `PasswordMessage`/`SaslInitialResponse`/`SaslResponse` on the wire (all
//! three share tag `'p'` and, decoded generically, carry the same opaque
//! payload bytes) — so relaying is lossless without the proxy ever needing
//! to understand SASL/SCRAM semantics, and nothing is retained past the
//! single forward.

use bytes::BytesMut;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::pool::{BackendLease, BackendPool, LeaseDisposition, PoolError};
use crate::proxy::config::SessionProxyConfig;
use crate::proxy::error::{ProxyError, RejectionReason, SessionOutcome};
use crate::proxy::relay::{forward_frontend, read_startup, relay_until_ready, HandshakeOutcome};
use crate::wire::{FrameReader, FrontendMessage, Role};

/// Runs one accepted frontend connection through the full session-mode
/// proxy pipeline to a terminal [`SessionOutcome`]. Never panics: every
/// error path is turned into an outcome, the admission permit (if any was
/// acquired) is always released, and the backend lease (if any was
/// acquired) is always released with [`LeaseDisposition::Close`] before
/// returning.
pub async fn run_session(client: TcpStream, config: &SessionProxyConfig) -> SessionOutcome {
    let permit = match config.frontend_budget.try_acquire() {
        Ok(permit) => permit,
        Err(_) => {
            let mut client = client;
            write_rejection(&mut client, RejectionReason::FrontendBudgetExhausted).await;
            return SessionOutcome::RejectedSaturated;
        }
    };

    let lease = match connect_backend(&config.backend_pool).await {
        Ok(lease) => lease,
        Err(_) => {
            let mut client = client;
            write_rejection(&mut client, RejectionReason::BackendUnreachable).await;
            drop(permit);
            return SessionOutcome::RejectedBackendUnreachable;
        }
    };

    let outcome = drive_session(client, lease, &config.backend_pool, config).await;
    drop(permit);
    outcome
}

/// Always a brand-new backend connect (R3), now bounded by the shared
/// [`BackendPool`]'s capacity (WI #1289) instead of a raw `TcpStream::connect`.
async fn connect_backend(pool: &BackendPool) -> Result<BackendLease, PoolError> {
    pool.acquire_fresh().await
}

async fn write_rejection(client: &mut TcpStream, reason: RejectionReason) {
    if let Some(message) = reason.synthesized_error_response() {
        let mut buf = BytesMut::new();
        message.encode(&mut buf);
        let _ = client.write_all(&buf).await;
    }
    let _ = client.shutdown().await;
}

async fn drive_session(
    client: TcpStream,
    lease: BackendLease,
    pool: &BackendPool,
    config: &SessionProxyConfig,
) -> SessionOutcome {
    let BackendLease {
        id: backend_id,
        stream: backend,
        ..
    } = lease;
    let (mut client_read, mut client_write) = client.into_split();
    let (mut backend_read, mut backend_write) = backend.into_split();
    let mut frontend_reader = FrameReader::new(Role::Frontend, &config.wire);
    let mut backend_reader = FrameReader::new(Role::Backend, &config.wire);

    let outcome = 'pipeline: {
        let startup =
            match read_startup(&mut client_read, &mut client_write, &mut frontend_reader).await {
                Ok(startup) => startup,
                Err(_) => break 'pipeline SessionOutcome::EstablishedClosedError,
            };
        if forward_frontend(&mut backend_write, &FrontendMessage::Startup(startup))
            .await
            .is_err()
        {
            break 'pipeline SessionOutcome::EstablishedClosedError;
        }

        match relay_until_ready(
            &mut client_read,
            &mut client_write,
            &mut backend_read,
            &mut backend_write,
            &mut frontend_reader,
            &mut backend_reader,
        )
        .await
        {
            Ok(HandshakeOutcome::Rejected) => break 'pipeline SessionOutcome::RejectedAuthFailed,
            Ok(HandshakeOutcome::Ready) => {}
            Err(_) => break 'pipeline SessionOutcome::EstablishedClosedError,
        }

        bidi_relay(
            &mut client_read,
            &mut client_write,
            &mut backend_read,
            &mut backend_write,
            &mut frontend_reader,
            &mut backend_reader,
        )
        .await
    };

    // Session mode never reuses its backend connection (R2b): every exit
    // path releases with `Close`, freeing the pool capacity slot.
    if let Ok(stream) = backend_read.reunite(backend_write) {
        pool.release(backend_id, stream, LeaseDisposition::Close)
            .await;
    }
    outcome
}

/// Two concurrent legs, each decoding + re-encoding + forwarding frames
/// until the client sends `Terminate`, either leg hits EOF, or either leg's
/// `FrameError` ends that leg without forwarding the offending bytes.
async fn bidi_relay(
    client_read: &mut tokio::net::tcp::OwnedReadHalf,
    client_write: &mut tokio::net::tcp::OwnedWriteHalf,
    backend_read: &mut tokio::net::tcp::OwnedReadHalf,
    backend_write: &mut tokio::net::tcp::OwnedWriteHalf,
    frontend_reader: &mut FrameReader,
    backend_reader: &mut FrameReader,
) -> SessionOutcome {
    let client_to_backend = async {
        loop {
            match crate::proxy::relay::read_frame(client_read, frontend_reader).await {
                Ok(Some(crate::wire::WireMessage::Frontend(FrontendMessage::Terminate(
                    terminate,
                )))) => {
                    let _ = forward_frontend(backend_write, &FrontendMessage::Terminate(terminate))
                        .await;
                    return Ok(());
                }
                Ok(Some(crate::wire::WireMessage::Frontend(msg))) => {
                    forward_frontend(backend_write, &msg).await?;
                }
                Ok(Some(crate::wire::WireMessage::Backend(_))) => {
                    unreachable!("frontend-role reader only emits Frontend frames")
                }
                Ok(None) => return Ok(()),
                Err(error) => return Err(error),
            }
        }
    };

    let backend_to_client = async {
        loop {
            match crate::proxy::relay::read_frame(backend_read, backend_reader).await {
                Ok(Some(crate::wire::WireMessage::Backend(msg))) => {
                    crate::proxy::relay::forward_backend(client_write, &msg).await?;
                }
                Ok(Some(crate::wire::WireMessage::Frontend(_))) => {
                    unreachable!("backend-role reader only emits Backend frames")
                }
                Ok(None) => return Ok(()),
                Err(error) => return Err(error),
            }
        }
    };

    let result: Result<(), ProxyError> = tokio::select! {
        r = client_to_backend => r,
        r = backend_to_client => r,
    };

    match result {
        Ok(()) => SessionOutcome::EstablishedClosedClean,
        Err(_) => SessionOutcome::EstablishedClosedError,
    }
}
// </HANDWRITE>
