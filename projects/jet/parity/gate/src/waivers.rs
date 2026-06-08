// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
// CODEGEN-BEGIN
//! Time-boxed waivers.

use std::path::Path;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::manifest::GateError;

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waiver {
    pub fixture_id: String,
    pub channel: String,
    /// Inclusive expiry date.
    pub expires_on: NaiveDate,
    pub reason: String,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Waivers {
    #[serde(default)]
    pub waivers: Vec<Waiver>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
impl Waivers {
    /// Parse a waivers TOML file. Missing file → empty set.
    pub fn parse(path: impl AsRef<Path>) -> Result<Self, GateError> {
        let path_ref = path.as_ref();
        if !path_ref.exists() {
            return Ok(Waivers::default());
        }
        let txt = std::fs::read_to_string(path_ref).map_err(|e| GateError::Io {
            path: path_ref.display().to_string(),
            source: e,
        })?;
        let w: Waivers = toml::from_str(&txt).map_err(|e| GateError::Toml {
            path: path_ref.display().to_string(),
            source: e,
        })?;
        Ok(w)
    }

    /// Returns the first unexpired waiver matching `(fixture_id, channel)`
    /// as of `now`.
    pub fn applies_to(
        &self,
        fixture_id: &str,
        channel: &str,
        now: DateTime<Utc>,
    ) -> Option<&Waiver> {
        let today = now.date_naive();
        self.waivers
            .iter()
            .find(|w| w.fixture_id == fixture_id && w.channel == channel && w.expires_on >= today)
    }
}
// CODEGEN-END
