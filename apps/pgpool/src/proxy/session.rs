// SPEC-MANAGED: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-session-proxy" tracker="#1288" reason="Session-mode proxy needs generator primitives that do not exist yet.">
//! One session's pipeline, following the TD Logic flowchart exactly:
//! admission -> backend connect (bounded by `backend_connect_timeout`) ->
//! frame-aware startup relay -> auth passthrough (opaque, never persisted)
//! -> ready relay -> bidirectional relay until `Terminate`/EOF/`FrameError`.
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
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::proxy::config::SessionProxyConfig;
use crate::proxy::error::{ProxyError, RejectionReason, SessionOutcome};
use crate::wire::{BackendMessage, FrameReader, FrontendMessage, Role, WireMessage};

/// Runs one accepted frontend connection through the full session-mode
/// proxy pipeline to a terminal [`SessionOutcome`]. Never panics: every
/// error path is turned into an outcome, the admission permit (if any was
/// acquired) is always released, and both sockets are always closed before
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

    let backend = match connect_backend(config).await {
        Ok(backend) => backend,
        Err(_) => {
            let mut client = client;
            write_rejection(&mut client, RejectionReason::BackendUnreachable).await;
            drop(permit);
            return SessionOutcome::RejectedBackendUnreachable;
        }
    };

    let outcome = drive_session(client, backend, config).await;
    drop(permit);
    outcome
}

async fn connect_backend(config: &SessionProxyConfig) -> Result<TcpStream, ProxyError> {
    let addr = format!("{}:{}", config.backend.host, config.backend.port);
    let connect = TcpStream::connect(&addr);
    match tokio::time::timeout(config.backend_connect_timeout, connect).await {
        Ok(Ok(stream)) => Ok(stream),
        Ok(Err(error)) => Err(ProxyError::Io(error.to_string())),
        Err(_) => Err(ProxyError::Io(format!(
            "backend connect to {addr} timed out after {:?}",
            config.backend_connect_timeout
        ))),
    }
}

async fn write_rejection(client: &mut TcpStream, reason: RejectionReason) {
    if let Some(message) = reason.synthesized_error_response() {
        let mut buf = BytesMut::new();
        message.encode(&mut buf);
        let _ = client.write_all(&buf).await;
    }
    let _ = client.shutdown().await;
}

/// Which side of the pre-established handshake ended: forward progress to
/// `ReadyForQuery`, or a backend `ErrorResponse` before it (`auth_result` in
/// the TD Logic flowchart).
enum PreEstablished {
    Ready,
    Rejected,
}

async fn drive_session(
    client: TcpStream,
    backend: TcpStream,
    config: &SessionProxyConfig,
) -> SessionOutcome {
    let (mut client_read, mut client_write) = client.into_split();
    let (mut backend_read, mut backend_write) = backend.into_split();
    let mut frontend_reader = FrameReader::new(Role::Frontend, &config.wire);
    let mut backend_reader = FrameReader::new(Role::Backend, &config.wire);

    let startup =
        match read_startup(&mut client_read, &mut client_write, &mut frontend_reader).await {
            Ok(startup) => startup,
            Err(_) => return SessionOutcome::EstablishedClosedError,
        };
    if forward_frontend(&mut backend_write, &FrontendMessage::Startup(startup))
        .await
        .is_err()
    {
        return SessionOutcome::EstablishedClosedError;
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
        Ok(PreEstablished::Rejected) => return SessionOutcome::RejectedAuthFailed,
        Ok(PreEstablished::Ready) => {}
        Err(_) => return SessionOutcome::EstablishedClosedError,
    }

    bidi_relay(
        client_read,
        client_write,
        backend_read,
        backend_write,
        frontend_reader,
        backend_reader,
    )
    .await
}

/// Reads frontend frames until the real `StartupMessage` arrives.
/// `SSLRequest` is legitimate and precedes it: TLS is out of scope for this
/// slice, so it is refused with the protocol's single `'N'` byte (never
/// forwarded to the backend) and the client is expected to retry in
/// cleartext.
async fn read_startup(
    client_read: &mut (impl AsyncRead + Unpin),
    client_write: &mut (impl AsyncWrite + Unpin),
    reader: &mut FrameReader,
) -> Result<crate::wire::StartupMessage, ProxyError> {
    loop {
        match read_frame(client_read, reader).await? {
            Some(WireMessage::Frontend(FrontendMessage::Ssl(_))) => {
                client_write
                    .write_all(b"N")
                    .await
                    .map_err(|error| ProxyError::Io(error.to_string()))?;
            }
            Some(WireMessage::Frontend(FrontendMessage::Startup(startup))) => return Ok(startup),
            Some(WireMessage::Frontend(_)) => {
                return Err(ProxyError::Io(
                    "unexpected frontend message before startup".to_string(),
                ));
            }
            Some(WireMessage::Backend(_)) => {
                unreachable!("frontend-role reader only emits Frontend frames")
            }
            None => return Err(ProxyError::Io("client closed before startup".to_string())),
        }
    }
}

/// Relays the auth passthrough loop and the ready-report frames that follow
/// it: every `Authentication*`/`ParameterStatus`/`BackendKeyData` frame from
/// the backend is forwarded to the client verbatim, and every backend
/// challenge that expects exactly one client reply
/// (`AuthenticationCleartextPassword`/`Md5Password`/`Sasl`/`SaslContinue`)
/// is answered by relaying the client's next frame back to the backend
/// unchanged. `AuthenticationSaslFinal` does not expect a client reply.
/// Ends at `ReadyForQuery` (session established) or an `ErrorResponse`
/// arriving before it (auth/startup rejected; forwarded to the client
/// verbatim, never synthesized).
async fn relay_until_ready(
    client_read: &mut (impl AsyncRead + Unpin),
    client_write: &mut (impl AsyncWrite + Unpin),
    backend_read: &mut (impl AsyncRead + Unpin),
    backend_write: &mut (impl AsyncWrite + Unpin),
    frontend_reader: &mut FrameReader,
    backend_reader: &mut FrameReader,
) -> Result<PreEstablished, ProxyError> {
    loop {
        let backend_msg = match read_frame(backend_read, backend_reader).await? {
            Some(WireMessage::Backend(msg)) => msg,
            Some(WireMessage::Frontend(_)) => {
                unreachable!("backend-role reader only emits Backend frames")
            }
            None => return Err(ProxyError::Io("backend closed before ready".to_string())),
        };

        forward_backend(client_write, &backend_msg).await?;

        match backend_msg {
            BackendMessage::ErrorResponse(_) => return Ok(PreEstablished::Rejected),
            BackendMessage::ReadyForQuery(_) => return Ok(PreEstablished::Ready),
            BackendMessage::AuthenticationCleartextPassword(_)
            | BackendMessage::AuthenticationMd5Password(_)
            | BackendMessage::AuthenticationSasl(_)
            | BackendMessage::AuthenticationSaslContinue(_) => {
                let client_msg = match read_frame(client_read, frontend_reader).await? {
                    Some(WireMessage::Frontend(msg)) => msg,
                    Some(WireMessage::Backend(_)) => {
                        unreachable!("frontend-role reader only emits Frontend frames")
                    }
                    None => return Err(ProxyError::Io("client closed during auth".to_string())),
                };
                forward_frontend(backend_write, &client_msg).await?;
            }
            // AuthenticationOk, AuthenticationSaslFinal, NoticeResponse,
            // ParameterStatus, BackendKeyData: forwarded above, no client
            // reply expected, keep waiting for ReadyForQuery.
            _ => {}
        }
    }
}

/// Two concurrent legs, each decoding + re-encoding + forwarding frames
/// until the client sends `Terminate`, either leg hits EOF, or either leg's
/// `FrameError` ends that leg without forwarding the offending bytes.
async fn bidi_relay(
    mut client_read: tokio::net::tcp::OwnedReadHalf,
    mut client_write: tokio::net::tcp::OwnedWriteHalf,
    mut backend_read: tokio::net::tcp::OwnedReadHalf,
    mut backend_write: tokio::net::tcp::OwnedWriteHalf,
    mut frontend_reader: FrameReader,
    mut backend_reader: FrameReader,
) -> SessionOutcome {
    let client_to_backend = async {
        loop {
            match read_frame(&mut client_read, &mut frontend_reader).await {
                Ok(Some(WireMessage::Frontend(FrontendMessage::Terminate(terminate)))) => {
                    let _ = forward_frontend(
                        &mut backend_write,
                        &FrontendMessage::Terminate(terminate),
                    )
                    .await;
                    return Ok(());
                }
                Ok(Some(WireMessage::Frontend(msg))) => {
                    forward_frontend(&mut backend_write, &msg).await?;
                }
                Ok(Some(WireMessage::Backend(_))) => {
                    unreachable!("frontend-role reader only emits Frontend frames")
                }
                Ok(None) => return Ok(()),
                Err(error) => return Err(error),
            }
        }
    };

    let backend_to_client = async {
        loop {
            match read_frame(&mut backend_read, &mut backend_reader).await {
                Ok(Some(WireMessage::Backend(msg))) => {
                    forward_backend(&mut client_write, &msg).await?;
                }
                Ok(Some(WireMessage::Frontend(_))) => {
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

/// Reads and decodes the next frame off `stream` via `reader`, awaiting more
/// bytes on `Ok(None)` (split/partial read) until a full frame is available.
/// Returns `Ok(None)` on clean EOF before any partial frame was buffered.
async fn read_frame(
    stream: &mut (impl AsyncRead + Unpin),
    reader: &mut FrameReader,
) -> Result<Option<WireMessage>, ProxyError> {
    loop {
        match reader.next_frame() {
            Ok(Some(message)) => return Ok(Some(message)),
            Ok(None) => {
                let mut buf = [0_u8; 8192];
                let n = stream
                    .read(&mut buf)
                    .await
                    .map_err(|error| ProxyError::Io(error.to_string()))?;
                if n == 0 {
                    return Ok(None);
                }
                reader.feed(&buf[..n]);
            }
            Err(error) => return Err(ProxyError::Wire(error)),
        }
    }
}

async fn forward_frontend(
    write: &mut (impl AsyncWrite + Unpin),
    msg: &FrontendMessage,
) -> Result<(), ProxyError> {
    let mut buf = BytesMut::new();
    msg.encode(&mut buf);
    write
        .write_all(&buf)
        .await
        .map_err(|error| ProxyError::Io(error.to_string()))
}

async fn forward_backend(
    write: &mut (impl AsyncWrite + Unpin),
    msg: &BackendMessage,
) -> Result<(), ProxyError> {
    let mut buf = BytesMut::new();
    msg.encode(&mut buf);
    write
        .write_all(&buf)
        .await
        .map_err(|error| ProxyError::Io(error.to_string()))
}
// </HANDWRITE>
// SPEC-MANAGED: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#logic
// CODEGEN-BEGIN
pub fn cli_serve_entry() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // SPEC-REF: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#pgpool-session-proxy-logic-flow-tcp_accept
    // TODO: Implement process step: tcp-server's serve_arc accepts a raw TCP connection and invokes SessionHandler::handle(stream, ConnectionContext) for every accepted socket
    todo!("process: tcp-server's serve_arc accepts a raw TCP connection and invokes SessionHandler::handle(stream, ConnectionContext) for every accepted socket");
    // Decision: SessionHandler's own ConnectionBudget::try_acquire() (RuntimePlan::frontend_budget) succeeds for this connection (R1)?
    if todo!("decision: SessionHandler's own ConnectionBudget::try_acquire() (RuntimePlan::frontend_budget) succeeds for this connection (R1)?") /* budget exhausted */ {
        return Err(todo!("error: Saturated: encode BackendMessage::ErrorResponse (SQLSTATE 53300 too_many_connections) and write it directly to the client stream, then close the socket without touching the backend or any other session (AC3)"));
    } else { /* permit acquired */
        // SPEC-REF: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#pgpool-session-proxy-logic-flow-connect_backend
        // TODO: Implement process step: Admitted, permit held for the session lifetime: TCP-connect to the configured backend endpoint (PGPOOL_BACKEND_ADDR/--backend-addr host:port, bounded by PGPOOL_BACKEND_CONNECT_TIMEOUT_MS/--backend-connect-timeout-ms) (R3)
        todo!("process: Admitted, permit held for the session lifetime: TCP-connect to the configured backend endpoint (PGPOOL_BACKEND_ADDR/--backend-addr host:port, bounded by PGPOOL_BACKEND_CONNECT_TIMEOUT_MS/--backend-connect-timeout-ms) (R3)");
        // Decision: Backend TCP connect succeeded within the connect timeout?
        if todo!("decision: Backend TCP connect succeeded within the connect timeout?") /* connect failed/timed out */ {
            return Err(todo!("error: Unreachable/timed out: encode BackendMessage::ErrorResponse (SQLSTATE 08006 connection_failure) to the client, release the ConnectionBudget permit, close the client socket"));
        } else { /* backend socket established */
            // SPEC-REF: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#pgpool-session-proxy-logic-flow-relay_startup
            // TODO: Implement process step: Frame-aware startup relay: a frontend-role FrameReader fed from the client stream decodes the client's untagged StartupMessage (or SSLRequest, rejected — TLS is out of scope), which is re-encoded and forwarded byte-identically to the backend stream (R2)
            todo!("process: Frame-aware startup relay: a frontend-role FrameReader fed from the client stream decodes the client's untagged StartupMessage (or SSLRequest, rejected — TLS is out of scope), which is re-encoded and forwarded byte-identically to the backend stream (R2)");
            // SPEC-REF: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#pgpool-session-proxy-logic-flow-relay_auth
            // TODO: Implement process step: Auth passthrough loop: alternately decode backend-role BackendMessage frames from the backend stream and frontend-role FrontendMessage frames from the client stream, re-encoding and forwarding every Authentication*/PasswordMessage/SaslInitialResponse/SaslResponse frame verbatim in both directions; pgpool treats password/SASL payload bytes as opaque relay data only and never persists them (R2, AC2)
            todo!("process: Auth passthrough loop: alternately decode backend-role BackendMessage frames from the backend stream and frontend-role FrontendMessage frames from the client stream, re-encoding and forwarding every Authentication*/PasswordMessage/SaslInitialResponse/SaslResponse frame verbatim in both directions; pgpool treats password/SASL payload bytes as opaque relay data only and never persists them (R2, AC2)");
            // Decision: Backend emits AuthenticationOk before any ErrorResponse?
            if todo!("decision: Backend emits AuthenticationOk before any ErrorResponse?") /* ErrorResponse (auth failed) */ {
                return Err(todo!("error: Backend emits ErrorResponse during startup/auth (bad credentials, SCRAM failure, ...): forward it to the client verbatim, release the permit, close both sides (no retry, no credential caching)"));
            } else { /* AuthenticationOk */
                // SPEC-REF: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#pgpool-session-proxy-logic-flow-relay_ready
                // TODO: Implement process step: Forward remaining backend startup messages (ParameterStatus*, BackendKeyData, ReadyForQuery) to the client; the backend-role FrameReader's transaction_status() records the initial TransactionStatus
                todo!("process: Forward remaining backend startup messages (ParameterStatus*, BackendKeyData, ReadyForQuery) to the client; the backend-role FrameReader's transaction_status() records the initial TransactionStatus");
                // SPEC-REF: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#pgpool-session-proxy-logic-flow-bidi_relay
                // TODO: Implement process step: Bidirectional relay: two concurrent tasks each decode WireMessage frames from one leg (frontend FrameReader on client->backend, backend FrameReader on backend->client) and re-encode+forward them to the other leg, until a frontend Terminate message, clean EOF, or a FrameError ends that leg (R2)
                todo!("process: Bidirectional relay: two concurrent tasks each decode WireMessage frames from one leg (frontend FrameReader on client->backend, backend FrameReader on backend->client) and re-encode+forward them to the other leg, until a frontend Terminate message, clean EOF, or a FrameError ends that leg (R2)");
                // Decision: Which condition ended the bidirectional relay?
                if todo!("decision: Which condition ended the bidirectional relay?") /* Terminate or clean client EOF */ {
                    todo!("terminal: Client sent Terminate (or closed cleanly): forward Terminate to the backend if not already sent, close the backend connection, release the ConnectionBudget permit — a clean session end");
                } else { /* backend EOF or FrameError */
                    return Err(todo!("error: Backend closed the connection, or a leg hit FrameError (malformed/oversized frame): close the client socket, release the permit; a FrameError never forwards the offending bytes, it only ends that leg"));
                }
            }
        }
        // SPEC-REF: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#pgpool-session-proxy-logic-flow-drain_interaction
        // TODO: Implement process step: Concurrently, DrainSignal (from ConnectionContext.drain) flips to Draining when the process receives SIGTERM/SIGINT: tcp-server's accept loop stops taking new connections immediately, while this session's bidi_relay keeps running unaffected until the client/backend end it or tcp_server's TcpServerConfig.drain_timeout elapses, at which point the task is abandoned (R4, AC4)
        todo!("process: Concurrently, DrainSignal (from ConnectionContext.drain) flips to Draining when the process receives SIGTERM/SIGINT: tcp-server's accept loop stops taking new connections immediately, while this session's bidi_relay keeps running unaffected until the client/backend end it or tcp_server's TcpServerConfig.drain_timeout elapses, at which point the task is abandoned (R4, AC4)");
    }
    // Terminal: backend_closed_or_error -> Backend closed the connection, or a leg hit FrameError (malformed/oversized frame): close the client socket, release the permit; a FrameError never forwards the offending bytes, it only ends that leg
    // Terminal: client_terminate -> Client sent Terminate (or closed cleanly): forward Terminate to the backend if not already sent, close the backend connection, release the ConnectionBudget permit — a clean session end
    // Terminal: reject_backend_unreachable -> Unreachable/timed out: encode BackendMessage::ErrorResponse (SQLSTATE 08006 connection_failure) to the client, release the ConnectionBudget permit, close the client socket
    // Terminal: reject_saturated -> Saturated: encode BackendMessage::ErrorResponse (SQLSTATE 53300 too_many_connections) and write it directly to the client stream, then close the socket without touching the backend or any other session (AC3)
    // Terminal: relay_error_from_backend -> Backend emits ErrorResponse during startup/auth (bad credentials, SCRAM failure, ...): forward it to the client verbatim, release the permit, close both sides (no retry, no credential caching)
}
// CODEGEN-END
