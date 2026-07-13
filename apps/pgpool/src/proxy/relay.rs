// SPEC-MANAGED: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-session-proxy" tracker="#1288" reason="Session-mode proxy needs generator primitives that do not exist yet.">
//! Frame-level relay primitives shared by the session-mode proxy
//! (`crate::proxy::session`) and the transaction-mode pool handler
//! (`crate::pool::transaction`, WI #1289): generic frame read-with-retry,
//! verbatim frontend/backend forwarding, the startup reader, and the
//! auth-passthrough loop that drives one handshake to `ReadyForQuery` or a
//! pre-ready `ErrorResponse`. Extracted here (rather than duplicated) so
//! both pool modes share one lossless relay implementation.

use bytes::BytesMut;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::proxy::error::ProxyError;
use crate::wire::{BackendMessage, FrameReader, FrontendMessage, WireMessage};

/// Which side of a pre-established handshake ended: forward progress to
/// `ReadyForQuery`, or a backend `ErrorResponse` before it.
pub(crate) enum HandshakeOutcome {
    Ready,
    Rejected,
}

/// Reads and decodes the next frame off `stream` via `reader`, awaiting more
/// bytes on `Ok(None)` (split/partial read) until a full frame is available.
/// Returns `Ok(None)` on clean EOF before any partial frame was buffered.
pub(crate) async fn read_frame(
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

pub(crate) async fn forward_frontend(
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

pub(crate) async fn forward_backend(
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

/// Reads frontend frames until the real `StartupMessage` arrives.
/// `SSLRequest` is legitimate and precedes it: TLS is out of scope for this
/// slice, so it is refused with the protocol's single `'N'` byte (never
/// forwarded to the backend) and the client is expected to retry in
/// cleartext.
pub(crate) async fn read_startup(
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
/// Ends at `ReadyForQuery` (established) or an `ErrorResponse` arriving
/// before it (rejected; forwarded to the client verbatim, never
/// synthesized).
pub(crate) async fn relay_until_ready(
    client_read: &mut (impl AsyncRead + Unpin),
    client_write: &mut (impl AsyncWrite + Unpin),
    backend_read: &mut (impl AsyncRead + Unpin),
    backend_write: &mut (impl AsyncWrite + Unpin),
    frontend_reader: &mut FrameReader,
    backend_reader: &mut FrameReader,
) -> Result<HandshakeOutcome, ProxyError> {
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
            BackendMessage::ErrorResponse(_) => return Ok(HandshakeOutcome::Rejected),
            BackendMessage::ReadyForQuery(_) => return Ok(HandshakeOutcome::Ready),
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
// </HANDWRITE>
