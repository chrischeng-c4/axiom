// HANDWRITE-BEGIN gap="missing-generator:logic:cbe7b900" tracker="pending-tracker" reason="Decode bounded OSC 7 frames and own validated ephemeral active-cwd context updates."
//! Explicit PTY cwd telemetry and ephemeral active-context state.

use std::fmt;
use std::path::{Path, PathBuf};

use url::Url;

const OSC7_PREFIX: &[u8] = b"\x1b]7;";
const OSC_BEL: u8 = 0x07;
const MAX_PENDING_BYTES: usize = 8 * 1024;

pub const CWD_TELEMETRY_ENV: &str = "WORKBENCH_CWD_TELEMETRY";
pub const CWD_TELEMETRY_PROTOCOL: &str = "osc7-file-uri-v1";

/// Source disclosed with every accepted active cwd change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CwdTelemetrySource {
    Osc7,
}

/// One validated active-context transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CwdContextUpdate {
    pub path: PathBuf,
    pub source: CwdTelemetrySource,
}

/// Errors constructing the initial authoritative context or an encoded frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CwdContextError {
    InvalidDirectory { path: PathBuf },
    CannotEncodeFileUri { path: PathBuf },
}

impl fmt::Display for CwdContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDirectory { path } => {
                write!(formatter, "{} is not an existing directory", path.display())
            }
            Self::CannotEncodeFileUri { path } => {
                write!(
                    formatter,
                    "{} cannot be encoded as a local file URI",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for CwdContextError {}

/// Streaming OSC 7 decoder that never interprets ordinary terminal text.
///
/// @spec apps/workbench/tech-design/logic/synchronize-authoritative-pty-cwd-into-workbench-active-context.md#logic
#[derive(Debug, Default)]
pub struct CwdTelemetryDecoder {
    pending: Vec<u8>,
}

impl CwdTelemetryDecoder {
    /// Feed arbitrary PTY output bytes and return complete OSC 7 URI payloads.
    pub fn push(&mut self, chunk: &[u8]) -> Vec<String> {
        self.pending.extend_from_slice(chunk);
        let mut frames = Vec::new();

        loop {
            let Some(prefix_start) = find_bytes(&self.pending, OSC7_PREFIX) else {
                self.retain_possible_prefix();
                break;
            };
            if prefix_start > 0 {
                self.pending.drain(..prefix_start);
            }

            let payload_start = OSC7_PREFIX.len();
            let Some((payload_end, consumed)) = find_osc_terminator(&self.pending, payload_start)
            else {
                if self.pending.len() > MAX_PENDING_BYTES {
                    self.pending.clear();
                }
                break;
            };

            if let Ok(payload) = std::str::from_utf8(&self.pending[payload_start..payload_end]) {
                frames.push(payload.to_string());
            }
            self.pending.drain(..consumed);
        }

        frames
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    pub const fn max_pending_bytes() -> usize {
        MAX_PENDING_BYTES
    }

    fn retain_possible_prefix(&mut self) {
        let keep = (1..OSC7_PREFIX.len())
            .rev()
            .find(|length| {
                self.pending.len() >= *length
                    && OSC7_PREFIX.starts_with(&self.pending[self.pending.len() - *length..])
            })
            .unwrap_or(0);
        if keep == 0 {
            self.pending.clear();
        } else {
            self.pending.drain(..self.pending.len() - keep);
        }
    }
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn find_osc_terminator(bytes: &[u8], start: usize) -> Option<(usize, usize)> {
    let mut index = start;
    while index < bytes.len() {
        if bytes[index] == OSC_BEL {
            return Some((index, index + 1));
        }
        if bytes[index] == 0x1b && bytes.get(index + 1) == Some(&b'\\') {
            return Some((index, index + 2));
        }
        index += 1;
    }
    None
}

/// Ephemeral cwd context derived only from validated telemetry frames.
///
/// This type deliberately has no reference to the registered folder store.
///
/// @spec apps/workbench/tech-design/logic/synchronize-authoritative-pty-cwd-into-workbench-active-context.md#logic
#[derive(Debug)]
pub struct ActiveCwdContext {
    current: PathBuf,
    decoder: CwdTelemetryDecoder,
}

impl ActiveCwdContext {
    pub fn new(initial: impl AsRef<Path>) -> Result<Self, CwdContextError> {
        let current = canonical_directory(initial.as_ref())?;
        Ok(Self {
            current,
            decoder: CwdTelemetryDecoder::default(),
        })
    }

    pub fn current(&self) -> &Path {
        &self.current
    }

    /// Consume raw PTY output and apply every valid changed directory in order.
    pub fn push_output(&mut self, chunk: &[u8]) -> Vec<CwdContextUpdate> {
        let mut updates = Vec::new();
        for candidate in self.decoder.push(chunk) {
            let Some(path) = decode_local_directory_uri(&candidate) else {
                continue;
            };
            if path == self.current {
                continue;
            }
            self.current = path.clone();
            updates.push(CwdContextUpdate {
                path,
                source: CwdTelemetrySource::Osc7,
            });
        }
        updates
    }

    pub fn pending_telemetry_bytes(&self) -> usize {
        self.decoder.pending_len()
    }
}

fn canonical_directory(path: &Path) -> Result<PathBuf, CwdContextError> {
    let canonical = path
        .canonicalize()
        .map_err(|_| CwdContextError::InvalidDirectory {
            path: path.to_path_buf(),
        })?;
    if canonical.is_dir() {
        Ok(canonical)
    } else {
        Err(CwdContextError::InvalidDirectory { path: canonical })
    }
}

fn decode_local_directory_uri(candidate: &str) -> Option<PathBuf> {
    let uri = Url::parse(candidate).ok()?;
    if uri.scheme() != "file" {
        return None;
    }
    if uri
        .host_str()
        .is_some_and(|host| !host.is_empty() && !host.eq_ignore_ascii_case("localhost"))
    {
        return None;
    }
    let path = uri.to_file_path().ok()?;
    canonical_directory(&path).ok()
}

/// Encode one canonical local directory using the protocol consumed above.
pub fn cwd_telemetry_frame(path: impl AsRef<Path>) -> Result<String, CwdContextError> {
    let canonical = canonical_directory(path.as_ref())?;
    let mut uri = Url::from_directory_path(&canonical).map_err(|()| {
        CwdContextError::CannotEncodeFileUri {
            path: canonical.clone(),
        }
    })?;
    uri.set_host(Some("localhost"))
        .map_err(|_| CwdContextError::CannotEncodeFileUri {
            path: canonical.clone(),
        })?;
    Ok(format!("\x1b]7;{uri}\x07"))
}

// HANDWRITE-END
