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
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

use crate::proxy::error::ProxyError;
use crate::wire::{
    BackendKeyData, BackendMessage, FrameReader, FrontendMessage, RelayFrame, RelayFrameKind,
    TransactionStatus, WireMessage,
};

/// Which side of a pre-established handshake ended: forward progress to
/// `ReadyForQuery`, or a backend `ErrorResponse` before it.
pub(crate) enum HandshakeOutcome {
    Ready {
        startup_replay: Option<Vec<BackendMessage>>,
    },
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
                let n = reader
                    .read_from(stream)
                    .await
                    .map_err(|error| ProxyError::Io(error.to_string()))?;
                if n == 0 {
                    return Ok(None);
                }
            }
            Err(error) => return Err(ProxyError::Wire(error)),
        }
    }
}

/// Reads a fully validated relay frame while retaining only the ownership
/// facts transaction pooling needs. The reader still enforces the normal
/// frame bounds and structural protocol validation; it simply avoids
/// materializing typed result rows and strings that will be forwarded
/// verbatim and discarded.
pub(crate) async fn read_relay_frame_with_raw(
    stream: &mut (impl AsyncRead + Unpin),
    reader: &mut FrameReader,
) -> Result<Option<RelayFrame>, ProxyError> {
    loop {
        match reader.next_relay_frame_with_raw() {
            Ok(Some(frame)) => return Ok(Some(frame)),
            Ok(None) => {
                let n = reader
                    .read_from(stream)
                    .await
                    .map_err(|error| ProxyError::Io(error.to_string()))?;
                if n == 0 {
                    return Ok(None);
                }
            }
            Err(error) => return Err(ProxyError::Wire(error)),
        }
    }
}

/// Consecutive backend frames that were already complete in the reader's
/// buffer. `bytes` owns a concatenated copy only when more than one frame is
/// present; the ordinary single-frame path retains its original raw slice.
/// A malformed suffix is recorded after its valid prefix so that callers can
/// forward exactly the frames they would have relayed one by one before
/// closing the connection.
pub(crate) struct BackendRelayBatch {
    first: bytes::Bytes,
    combined: Option<BytesMut>,
    pub(crate) ready: Option<TransactionStatus>,
    pub(crate) terminal_error: bool,
    #[cfg(test)]
    frame_count: usize,
}

impl BackendRelayBatch {
    fn new(frame: RelayFrame) -> Self {
        let ready = backend_ready(frame.kind);
        Self {
            first: frame.bytes,
            combined: None,
            ready,
            terminal_error: false,
            #[cfg(test)]
            frame_count: 1,
        }
    }

    fn push(&mut self, frame: RelayFrame) {
        let ready = backend_ready(frame.kind);
        match &mut self.combined {
            Some(combined) => combined.extend_from_slice(&frame.bytes),
            None => {
                let mut combined = BytesMut::with_capacity(self.first.len() + frame.bytes.len());
                combined.extend_from_slice(&self.first);
                combined.extend_from_slice(&frame.bytes);
                self.combined = Some(combined);
            }
        }
        self.ready = ready.or(self.ready);
        #[cfg(test)]
        {
            self.frame_count += 1;
        }
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        self.combined
            .as_deref()
            .unwrap_or_else(|| self.first.as_ref())
    }

    #[cfg(test)]
    fn frame_count(&self) -> usize {
        self.frame_count
    }
}

fn backend_ready(kind: RelayFrameKind) -> Option<TransactionStatus> {
    match kind {
        RelayFrameKind::Other => None,
        RelayFrameKind::BackendReady(status) => Some(status),
        RelayFrameKind::FrontendTerminate => {
            unreachable!("backend-role reader cannot emit a frontend termination frame")
        }
    }
}

/// Reads one backend relay frame, then immediately drains any following
/// complete validated frames that are already buffered from the same or an
/// earlier socket read. It never awaits another read while forming a batch,
/// and stops at `ReadyForQuery` so transaction ownership semantics remain
/// exactly at their existing boundary.
pub(crate) async fn read_backend_relay_batch_with_raw(
    stream: &mut (impl AsyncRead + Unpin),
    reader: &mut FrameReader,
) -> Result<Option<BackendRelayBatch>, ProxyError> {
    let Some(first) = read_relay_frame_with_raw(stream, reader).await? else {
        return Ok(None);
    };
    let mut batch = BackendRelayBatch::new(first);

    while batch.ready.is_none() {
        match reader.next_relay_frame_with_raw() {
            Ok(Some(frame)) => {
                batch.push(frame);
                if batch.ready.is_some() {
                    break;
                }
            }
            // The next bytes are incomplete (or absent), so forwarding this
            // valid prefix now cannot introduce a response delay.
            Ok(None) => break,
            // The old one-frame loop would have forwarded the valid prefix,
            // then failed reading this suffix. Preserve that observable order.
            Err(_) => {
                batch.terminal_error = true;
                break;
            }
        }
    }

    Ok(Some(batch))
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

/// Writes a frame exactly as it was validated from the opposite transport.
/// This intentionally does not bypass decoding: callers receive these bytes
/// only from [`read_relay_frame_with_raw`] on the transaction relay path.
pub(crate) async fn forward_raw(
    write: &mut (impl AsyncWrite + Unpin),
    bytes: &[u8],
) -> Result<(), ProxyError> {
    write
        .write_all(bytes)
        .await
        .map_err(|error| ProxyError::Io(error.to_string()))
}

/// Forwards an immediately available backend batch in one write. The batch
/// contains only structurally validated, ordered wire frames and never waits
/// for a buffering threshold.
pub(crate) async fn forward_backend_batch(
    write: &mut (impl AsyncWrite + Unpin),
    batch: &BackendRelayBatch,
) -> Result<(), ProxyError> {
    forward_raw(write, batch.bytes()).await
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
    // @spec apps/pgpool/tech-design/logic/trust-startup-replay-for-capped-transaction-pooling.md#logic
    // The replay is deliberately opt-in: any authentication frame requiring
    // a frontend response makes the whole handshake non-replayable. The
    // actual client receives its genuine protocol-ready frame sequence. The
    // cached copy replaces BackendKeyData with a synthetic zero key: pgpool
    // does not route CancelRequest and must never hand a later frontend a
    // key for another physical backend.
    let mut startup_replay = Vec::new();
    let mut saw_authentication_ok = false;
    let mut replayable = true;

    loop {
        let backend_msg = match read_frame(backend_read, backend_reader).await? {
            Some(WireMessage::Backend(msg)) => msg,
            Some(WireMessage::Frontend(_)) => {
                unreachable!("backend-role reader only emits Backend frames")
            }
            None => return Err(ProxyError::Io("backend closed before ready".to_string())),
        };

        forward_backend(client_write, &backend_msg).await?;

        match &backend_msg {
            BackendMessage::AuthenticationOk(_) => {
                saw_authentication_ok = true;
                if replayable {
                    startup_replay.push(backend_msg.clone());
                }
            }
            BackendMessage::AuthenticationCleartextPassword(_)
            | BackendMessage::AuthenticationMd5Password(_)
            | BackendMessage::AuthenticationSasl(_)
            | BackendMessage::AuthenticationSaslContinue(_) => {
                replayable = false;
                startup_replay.clear();
            }
            BackendMessage::BackendKeyData(_) if replayable => {
                startup_replay.push(BackendMessage::BackendKeyData(BackendKeyData {
                    process_id: 0,
                    secret_key: 0,
                }));
            }
            _ if replayable => startup_replay.push(backend_msg.clone()),
            _ => {}
        }

        match backend_msg {
            BackendMessage::ErrorResponse(_) => return Ok(HandshakeOutcome::Rejected),
            BackendMessage::ReadyForQuery(_) => {
                return Ok(HandshakeOutcome::Ready {
                    startup_replay: (replayable && saw_authentication_ok).then_some(startup_replay),
                });
            }
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

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use tokio::io::duplex;

    use super::read_backend_relay_batch_with_raw;
    use crate::wire::{
        BackendMessage, CommandComplete, FrameReader, ReadyForQuery, Role, TransactionStatus,
        WireCodecConfig,
    };

    fn encode(message: BackendMessage) -> BytesMut {
        let mut bytes = BytesMut::new();
        message.encode(&mut bytes);
        bytes
    }

    #[tokio::test]
    async fn batches_only_complete_buffered_backend_frames_until_ready() {
        let config = WireCodecConfig::default();
        let command = encode(BackendMessage::CommandComplete(CommandComplete {
            tag: "SELECT 1".to_string(),
        }));
        let ready = encode(BackendMessage::ReadyForQuery(ReadyForQuery {
            status: TransactionStatus::Idle,
        }));
        let mut reader = FrameReader::new(Role::Backend, &config);
        reader.feed(&command);
        reader.feed(&ready);
        let (_peer, mut stream) = duplex(1024);

        let batch = read_backend_relay_batch_with_raw(&mut stream, &mut reader)
            .await
            .expect("buffered frames decode")
            .expect("buffered frames form a batch");
        assert_eq!(batch.frame_count(), 2);
        assert_eq!(batch.ready, Some(TransactionStatus::Idle));
        assert!(!batch.terminal_error);
        assert_eq!(batch.bytes(), [command.as_ref(), ready.as_ref()].concat());
    }

    #[tokio::test]
    async fn leaves_an_incomplete_next_frame_for_the_normal_reader_path() {
        let config = WireCodecConfig::default();
        let command = encode(BackendMessage::CommandComplete(CommandComplete {
            tag: "SELECT 1".to_string(),
        }));
        let ready = encode(BackendMessage::ReadyForQuery(ReadyForQuery {
            status: TransactionStatus::Idle,
        }));
        let mut reader = FrameReader::new(Role::Backend, &config);
        reader.feed(&command);
        reader.feed(&ready[..4]);
        let (_peer, mut stream) = duplex(1024);

        let first = read_backend_relay_batch_with_raw(&mut stream, &mut reader)
            .await
            .expect("complete prefix decodes")
            .expect("complete prefix forms a batch");
        assert_eq!(first.frame_count(), 1);
        assert_eq!(first.bytes(), command.as_ref());
        assert_eq!(first.ready, None);

        reader.feed(&ready[4..]);
        let second = read_backend_relay_batch_with_raw(&mut stream, &mut reader)
            .await
            .expect("completed suffix decodes")
            .expect("completed suffix forms a batch");
        assert_eq!(second.frame_count(), 1);
        assert_eq!(second.bytes(), ready.as_ref());
        assert_eq!(second.ready, Some(TransactionStatus::Idle));
    }

    #[tokio::test]
    async fn forwards_valid_prefix_before_marking_a_malformed_suffix_terminal() {
        let config = WireCodecConfig::default();
        let command = encode(BackendMessage::CommandComplete(CommandComplete {
            tag: "SELECT 1".to_string(),
        }));
        let mut reader = FrameReader::new(Role::Backend, &config);
        reader.feed(&command);
        reader.feed(&[b'Z', 0, 0, 0, 5, b'Q']);
        let (_peer, mut stream) = duplex(1024);

        let batch = read_backend_relay_batch_with_raw(&mut stream, &mut reader)
            .await
            .expect("valid prefix remains relayable")
            .expect("valid prefix forms a batch");
        assert_eq!(batch.frame_count(), 1);
        assert_eq!(batch.bytes(), command.as_ref());
        assert!(batch.terminal_error);
    }
}
// </HANDWRITE>
