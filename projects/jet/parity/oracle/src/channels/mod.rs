// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src-channels.md#schema
// CODEGEN-BEGIN

//! Channel trait — uniform `capture` interface for the five oracle
//! channels (pixel, a11y, focus, pointer, ime).
//!
//! Per §Logic, channel order is fixed and earlier channels must not
//! mutate state visible to later channels.

use crate::manifest::FixtureManifest;
use crate::runner::{BrowserSession, MatrixEntry};
use async_trait::async_trait;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod a11y;
pub mod focus;
pub mod ime;
pub mod pixel;
pub mod pointer;

/// @spec parity-dom-reference-runner.md#Dependency (ChannelArtifact)
///
/// Payload returned by a [`Channel`]'s `capture` step. The Runner forwards
/// this through the [`ArtifactWriter`](crate::artifacts::ArtifactWriter)
/// to disk under the deterministic per-fixture path layout (R3).
#[derive(Debug, Clone)]
pub enum ChannelArtifact {
    Png {
        filename: String,
        bytes: Vec<u8>,
    },
    Json {
        filename: String,
        value: serde_json::Value,
    },
}

/// @spec parity-dom-reference-runner.md#Dependency (Channel trait)
#[derive(Debug, Error)]
pub enum ChannelError {
    #[error("channel io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("channel cdp error: {0}")]
    Cdp(String),
    #[error("channel json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("channel not implemented (live-browser harness gated on #2139 follow-up): {0}")]
    NotImplemented(&'static str),
}

/// @spec parity-dom-reference-runner.md#Dependency (ChannelCtx)
///
/// The bundle of shared state every channel needs: a mutable handle to
/// the browser session (which exposes the CDP client + page host), the
/// fixture manifest, the matrix entry (browser/DPR), and a deterministic
/// PRNG seeded from the fixture name.
pub struct ChannelCtx<'a> {
    pub session: &'a mut dyn BrowserSession,
    pub manifest: &'a FixtureManifest,
    pub matrix: MatrixEntry,
    pub prng: DeterministicPrng,
}

/// @spec parity-dom-reference-runner.md#Dependency (Channel trait)
#[async_trait]
pub trait Channel: Send + Sync {
    /// Identifier for this channel (used for logging + artifact naming).
    fn name(&self) -> &'static str;

    /// Drive the channel against `ctx` and return the artifact payload.
    async fn capture(&self, ctx: &mut ChannelCtx<'_>) -> Result<ChannelArtifact, ChannelError>;
}

/// @spec parity-dom-reference-runner.md#Dependency (DeterministicPrng)
///
/// Wraps `Xoshiro256PlusPlus` seeded from `fnv1a64(fixture_name)`.
#[derive(Debug, Clone)]
pub struct DeterministicPrng {
    inner: Xoshiro256PlusPlus,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src-channels.md#schema
impl DeterministicPrng {
    /// @spec parity-dom-reference-runner.md#Logic (determinism contract — PRNG)
    pub fn from_fixture_name(name: &str) -> Self {
        let seed = fnv1a64(name.as_bytes());
        let mut seed_bytes = [0u8; 32];
        seed_bytes[..8].copy_from_slice(&seed.to_le_bytes());
        seed_bytes[8..16].copy_from_slice(&seed.to_le_bytes());
        seed_bytes[16..24].copy_from_slice(&seed.to_le_bytes());
        seed_bytes[24..32].copy_from_slice(&seed.to_le_bytes());
        Self {
            inner: Xoshiro256PlusPlus::from_seed(seed_bytes),
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        use rand_xoshiro::rand_core::RngCore;
        self.inner.next_u64()
    }

    /// Draw a uniform point in `[0, w) × [0, h)`.
    pub fn next_point(&mut self, w: u32, h: u32) -> (u32, u32) {
        let x = (self.next_u64() % w.max(1) as u64) as u32;
        let y = (self.next_u64() % h.max(1) as u64) as u32;
        (x, y)
    }
}

/// @spec parity-dom-reference-runner.md#Logic (PRNG seed)
///
/// 64-bit FNV-1a hash. Spec mandates this exact derivation.
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Hit-map entry recorded by [`pointer::PointerChannel`] (R7).
/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src-channels.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PointerHit {
    pub x: u32,
    pub y: u32,
    pub target_selector: String,
    pub computed_cursor: String,
}

/// Focus-trace entry recorded by [`focus::FocusChannel`] (R6).
/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src-channels.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusEntry {
    pub step: u32,
    pub selector: String,
    pub role: String,
    pub name: String,
    pub bounds: [f64; 4],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a64_matches_known_vector() {
        // FNV-1a 64 known vector: hash of "foobar" = 0x85944171f73967e8
        assert_eq!(fnv1a64(b"foobar"), 0x85944171f73967e8);
    }

    #[test]
    fn prng_is_deterministic_per_fixture_name() {
        let mut a = DeterministicPrng::from_fixture_name("mui-button");
        let mut b = DeterministicPrng::from_fixture_name("mui-button");
        for _ in 0..16 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn prng_differs_across_fixture_names() {
        let mut a = DeterministicPrng::from_fixture_name("mui-button");
        let mut b = DeterministicPrng::from_fixture_name("mui-text-field");
        // first 16 draws are unlikely to all collide.
        let any_diff = (0..16).any(|_| a.next_u64() != b.next_u64());
        assert!(any_diff);
    }

    #[test]
    fn prng_point_stays_in_viewport() {
        let mut p = DeterministicPrng::from_fixture_name("x");
        for _ in 0..256 {
            let (x, y) = p.next_point(800, 600);
            assert!(x < 800);
            assert!(y < 600);
        }
    }
}
// CODEGEN-END
