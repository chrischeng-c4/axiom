// HANDWRITE-BEGIN gap="missing-generator:ephemeral-k8s-session-port-forward-json" tracker="#1693" reason="A foreground Service tunnel has a shared kubectl/host-child process group and bounded recovery lifecycle. Agent JSON capture must drain arbitrary host-child streams without delaying group cleanup or exposing unbounded output, which is runtime lifecycle policy rather than a generator primitive."
//! Bounded JSON capture for a credential-free host child behind a K3s tunnel.
//!
//! The tunnel owner stops and confirms the shared process group before this
//! module joins its pipe readers. A host descendant can inherit a pipe after
//! the direct child exits, so joining sooner could deadlock the JSON path and
//! prevent recovery cleanup.

use std::io::Read;
use std::process::{Child, ExitStatus};
use std::thread::{self, JoinHandle};

use anyhow::{Context, Result};

#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

const MAX_PORT_FORWARD_JSON_STREAM_CAPTURE_BYTES: usize = 64 * 1024;
const MAX_PORT_FORWARD_JSON_STREAM_VALUE_BYTES: usize = 64 * 1024;

#[cfg(test)]
static TEST_FAIL_READER_SPAWN_ON: AtomicUsize = AtomicUsize::new(0);

/// One bounded, decoded host-child stream suitable for a VAT JSON document.
pub(super) struct BoundedStream {
    pub(super) text: String,
    pub(super) truncated: bool,
    pub(super) utf8_lossy: bool,
}

/// The direct child and its concurrently-drained pipes. The caller keeps this
/// object alive through tunnel teardown, then joins it only after cleanup has
/// confirmed that inherited pipe holders cannot remain alive.
pub(super) struct CapturedHostChild {
    child: Child,
    stdout_reader: Option<JoinHandle<Result<BoundedStream>>>,
    stderr_reader: Option<JoinHandle<Result<BoundedStream>>>,
    setup_error: Option<String>,
}

pub(super) struct CapturedHostSnapshot {
    pub(super) status: ExitStatus,
    pub(super) stdout: BoundedStream,
    pub(super) stderr: BoundedStream,
}

impl CapturedHostChild {
    /// Capture setup deliberately never joins a partially-started reader or
    /// kills/reaps the shared-group child. If one reader cannot start, the
    /// caller retains this object, runs `stop_and_confirm`, then joins only
    /// after group cleanup has made inherited pipes safe to close.
    pub(super) fn start(mut child: Child) -> Self {
        let mut setup_error = None;
        let stdout_reader = match child.stdout.take() {
            Some(stdout) => match spawn_bounded_reader("stdout", stdout) {
                Ok(reader) => Some(reader),
                Err(error) => {
                    setup_error = Some(error.to_string());
                    None
                }
            },
            None => {
                setup_error = Some("host child did not expose stdout capture".to_string());
                None
            }
        };
        let stderr_reader = match child.stderr.take() {
            Some(stderr) => match spawn_bounded_reader("stderr", stderr) {
                Ok(reader) => Some(reader),
                Err(error) => {
                    if setup_error.is_none() {
                        setup_error = Some(error.to_string());
                    }
                    None
                }
            },
            None => {
                if setup_error.is_none() {
                    setup_error = Some("host child did not expose stderr capture".to_string());
                }
                None
            }
        };

        Self {
            child,
            stdout_reader,
            stderr_reader,
            setup_error,
        }
    }

    pub(super) fn capture_ready(&self) -> bool {
        self.setup_error.is_none()
    }

    pub(super) fn try_wait(&mut self) -> Result<Option<ExitStatus>> {
        self.child
            .try_wait()
            .context("poll port-forward JSON host child")
    }

    pub(super) fn child_mut(&mut self) -> &mut Child {
        &mut self.child
    }

    /// Join only after the caller has confirmed that the shared tunnel group
    /// is gone. This preserves cleanup priority over pipe EOF from a host
    /// descendant that inherited stdout or stderr.
    pub(super) fn finish_after_cleanup(
        mut self,
        status: ExitStatus,
    ) -> Result<CapturedHostSnapshot> {
        let (stdout, stderr) = self.join_readers_after_cleanup();
        if let Some(error) = self.setup_error.take() {
            let _ = stdout;
            let _ = stderr;
            return Err(anyhow::anyhow!(
                "K3s port-forward JSON capture setup failed after safe cleanup: {error}"
            ));
        }
        let stdout = stdout?.context("K3s port-forward JSON stdout reader is unavailable")?;
        let stderr = stderr?.context("K3s port-forward JSON stderr reader is unavailable")?;
        Ok(CapturedHostSnapshot {
            status,
            stdout,
            stderr,
        })
    }

    /// A cancellation has no JSON success result, but once cleanup is known
    /// complete, join readers so they do not outlive the foreground VAT call.
    pub(super) fn discard_after_cleanup(mut self) -> Result<()> {
        let (stdout, stderr) = self.join_readers_after_cleanup();
        if let Some(error) = self.setup_error.take() {
            let _ = stdout;
            let _ = stderr;
            return Err(anyhow::anyhow!(
                "K3s port-forward JSON capture setup failed after safe cleanup: {error}"
            ));
        }
        let _ = stdout?.context("K3s port-forward JSON stdout reader is unavailable")?;
        let _ = stderr?.context("K3s port-forward JSON stderr reader is unavailable")?;
        Ok(())
    }

    /// Evaluate both joins before returning either error. This helper is only
    /// called after group cleanup, so a background descendant cannot keep a
    /// pipe open indefinitely and no reader is left unobserved on one error.
    fn join_readers_after_cleanup(
        &mut self,
    ) -> (Result<Option<BoundedStream>>, Result<Option<BoundedStream>>) {
        let stdout = match self.stdout_reader.take() {
            Some(reader) => join_reader(reader, "stdout").map(Some),
            None => Ok(None),
        };
        let stderr = match self.stderr_reader.take() {
            Some(reader) => join_reader(reader, "stderr").map(Some),
            None => Ok(None),
        };
        (stdout, stderr)
    }
}

fn spawn_bounded_reader<R>(stream: &str, reader: R) -> Result<JoinHandle<Result<BoundedStream>>>
where
    R: Read + Send + 'static,
{
    // Debug-only deterministic integration failpoint. It is intentionally
    // fail-closed and named TEST: it can only suppress a JSON result, never
    // enable credentials or change process-group ownership.
    if debug_reader_failpoint(stream) {
        return Err(anyhow::anyhow!(
            "debug test bounded K3s port-forward JSON {stream} reader startup failure"
        ));
    }
    #[cfg(test)]
    {
        let remaining = TEST_FAIL_READER_SPAWN_ON.load(Ordering::SeqCst);
        if remaining != 0 && TEST_FAIL_READER_SPAWN_ON.fetch_sub(1, Ordering::SeqCst) == 1 {
            return Err(anyhow::anyhow!(
                "test-only bounded K3s port-forward JSON {stream} reader startup failure"
            ));
        }
    }
    thread::Builder::new()
        .name(format!("vat-k8s-port-forward-json-{stream}"))
        .spawn(move || bounded_stream(reader))
        .with_context(|| format!("start bounded K3s port-forward JSON {stream} reader"))
}

fn debug_reader_failpoint(stream: &str) -> bool {
    #[cfg(debug_assertions)]
    {
        if std::env::var("VAT_TEST_FAIL_PORT_FORWARD_JSON_READER").as_deref() != Ok(stream) {
            return false;
        }
        if let Some(ready) = std::env::var_os("VAT_TEST_FAIL_PORT_FORWARD_JSON_READER_READY") {
            let ready = std::path::PathBuf::from(ready);
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            while !ready.exists() && std::time::Instant::now() < deadline {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
        return true;
    }
    #[cfg(not(debug_assertions))]
    {
        let _ = stream;
        false
    }
}

fn join_reader(reader: JoinHandle<Result<BoundedStream>>, stream: &str) -> Result<BoundedStream> {
    reader
        .join()
        .map_err(|_| anyhow::anyhow!("bounded K3s port-forward JSON {stream} reader panicked"))?
        .with_context(|| format!("capture bounded K3s port-forward JSON {stream}"))
}

/// Drain arbitrary data in chunks and retain only the newest byte suffix.
/// Retention happens independently per stream so a full stderr pipe cannot
/// deadlock a host command that is still writing stdout.
fn bounded_stream(mut reader: impl Read) -> Result<BoundedStream> {
    let mut retained = Vec::with_capacity(MAX_PORT_FORWARD_JSON_STREAM_CAPTURE_BYTES);
    let mut chunk = [0_u8; 8 * 1024];
    let mut truncated = false;
    loop {
        let read = reader
            .read(&mut chunk)
            .context("read K3s port-forward JSON host-child stream")?;
        if read == 0 {
            break;
        }
        let bytes = &chunk[..read];
        if bytes.len() >= MAX_PORT_FORWARD_JSON_STREAM_CAPTURE_BYTES {
            retained.clear();
            retained.extend_from_slice(
                &bytes[bytes.len() - MAX_PORT_FORWARD_JSON_STREAM_CAPTURE_BYTES..],
            );
            truncated = true;
            continue;
        }
        let overflow = retained
            .len()
            .saturating_add(bytes.len())
            .saturating_sub(MAX_PORT_FORWARD_JSON_STREAM_CAPTURE_BYTES);
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

/// JSON escaping may expand a string past its raw byte length. Retain the
/// newest character-aligned suffix whose actual serialized string fits the
/// advertised per-stream budget.
fn cap_to_json_string_value(text: String) -> Result<(String, bool)> {
    if serialized_json_string_len(&text)? <= MAX_PORT_FORWARD_JSON_STREAM_VALUE_BYTES {
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
        if serialized_json_string_len(suffix)? <= MAX_PORT_FORWARD_JSON_STREAM_VALUE_BYTES {
            upper = middle;
        } else {
            lower = middle + 1;
        }
    }
    let suffix = text[boundaries[lower]..].to_string();
    debug_assert!(serialized_json_string_len(&suffix)
        .is_ok_and(|length| length <= MAX_PORT_FORWARD_JSON_STREAM_VALUE_BYTES));
    Ok((suffix, true))
}

pub(super) fn serialized_json_string_len(text: &str) -> Result<usize> {
    serde_json::to_vec(text)
        .map(|encoded| encoded.len())
        .context("serialize bounded K3s port-forward JSON stream")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::process::CommandExt;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    #[test]
    fn bounded_stream_preserves_latest_serializable_agent_suffix() {
        let mut bytes = vec![0xff; MAX_PORT_FORWARD_JSON_STREAM_CAPTURE_BYTES + 16];
        bytes.extend_from_slice(&[0, b'\n', b'e', b'n', b'd']);
        let snapshot = bounded_stream(std::io::Cursor::new(bytes))
            .expect("drain bounded port-forward JSON stream");
        assert!(snapshot.truncated);
        assert!(snapshot.utf8_lossy);
        assert!(snapshot.text.ends_with("\nend"));
        assert!(
            serialized_json_string_len(&snapshot.text).expect("serialize bounded text")
                <= MAX_PORT_FORWARD_JSON_STREAM_VALUE_BYTES,
            "lossy/control expansion must stay within the public JSON stream budget"
        );
    }

    #[test]
    fn partial_reader_setup_returns_before_shared_group_cleanup() {
        let mut command = Command::new("/bin/sh");
        command
            .args([
                "-c",
                "(trap '' TERM INT; while :; do sleep 1; done) & exit 0",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);
        let child = command
            .spawn()
            .expect("spawn host child with inherited-pipe descendant");
        let pgid = child.id();
        TEST_FAIL_READER_SPAWN_ON.store(2, Ordering::SeqCst);
        let started = Instant::now();
        let capture = CapturedHostChild::start(child);
        TEST_FAIL_READER_SPAWN_ON.store(0, Ordering::SeqCst);
        assert!(
            started.elapsed() < Duration::from_secs(1),
            "partial reader startup must not join an inherited pipe before caller cleanup"
        );
        assert!(
            !capture.capture_ready(),
            "the second reader startup failure must be retained for post-cleanup handling"
        );

        assert_eq!(
            unsafe { libc::kill(-(pgid as libc::pid_t), libc::SIGKILL) },
            0,
            "caller cleanup must stop the shared group before reader join"
        );
        let error = capture
            .discard_after_cleanup()
            .expect_err("post-cleanup reader setup failure must not yield a JSON success result");
        assert!(error.to_string().contains("capture setup failed"));
    }
}
// HANDWRITE-END
