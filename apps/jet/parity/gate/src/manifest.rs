// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
// CODEGEN-BEGIN
//! `parity-gating.toml` schema + parser.

use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The five canonical parity channels (R1).
pub const CANONICAL_CHANNELS: &[&str] = &[
    "pixel",
    "ax-tree",
    "focus-order",
    "pointer-hit-map",
    "ime-trace",
];

/// Strongly-typed enum form used by `ChannelResult` consumers. The
/// manifest itself stores channels as strings so that unknown values
/// surface as a clean validation error rather than a Serde panic.
/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Channel {
    Pixel,
    AxTree,
    FocusOrder,
    PointerHitMap,
    ImeTrace,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
impl Channel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Channel::Pixel => "pixel",
            Channel::AxTree => "ax-tree",
            Channel::FocusOrder => "focus-order",
            Channel::PointerHitMap => "pointer-hit-map",
            Channel::ImeTrace => "ime-trace",
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
#[derive(Debug, Error)]
pub enum GateError {
    #[error("io error reading {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("toml parse error in {path}: {source}")]
    Toml {
        path: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("json parse error in {path}: {source}")]
    Json {
        path: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("manifest validation failed: {0}")]
    Invalid(String),
    #[error("refusing to overwrite {path} without --force")]
    WouldOverwrite { path: String },
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tolerance {
    /// Max per-pixel L2 delta in 0..=255.
    pub pixel_delta: u16,
    /// Max fraction of differing a11y nodes in 0.0..=1.0.
    pub a11y_diff_ratio: f64,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterSelector {
    pub id: String,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatingManifest {
    pub channels: Vec<String>,
    pub tolerance: Tolerance,
    pub blocking: bool,
    pub allow_waivers: bool,
    pub adapter: AdapterSelector,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
impl GatingManifest {
    pub fn parse(path: impl AsRef<Path>) -> Result<Self, GateError> {
        let path_ref = path.as_ref();
        let txt = std::fs::read_to_string(path_ref).map_err(|e| GateError::Io {
            path: path_ref.display().to_string(),
            source: e,
        })?;
        let m: GatingManifest = toml::from_str(&txt).map_err(|e| GateError::Toml {
            path: path_ref.display().to_string(),
            source: e,
        })?;
        m.validate()?;
        Ok(m)
    }

    fn validate(&self) -> Result<(), GateError> {
        if self.channels.is_empty() {
            return Err(GateError::Invalid("channels must not be empty".to_string()));
        }
        for c in &self.channels {
            if !CANONICAL_CHANNELS.contains(&c.as_str()) {
                return Err(GateError::Invalid(format!(
                    "unknown channel '{c}'; allowed: {:?}",
                    CANONICAL_CHANNELS
                )));
            }
        }
        if self.tolerance.pixel_delta > 255 {
            return Err(GateError::Invalid(format!(
                "tolerance.pixel_delta = {} is out of range 0..=255",
                self.tolerance.pixel_delta
            )));
        }
        let r = self.tolerance.a11y_diff_ratio;
        if !(0.0..=1.0).contains(&r) || r.is_nan() {
            return Err(GateError::Invalid(format!(
                "tolerance.a11y_diff_ratio = {r} is out of range 0.0..=1.0"
            )));
        }
        if self.adapter.id.trim().is_empty() {
            return Err(GateError::Invalid(
                "adapter.id must not be empty".to_string(),
            ));
        }
        Ok(())
    }
}
// CODEGEN-END
