// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src-channels.md#schema
// CODEGEN-BEGIN

//! Pointer channel — draws 1000 seeded `(x, y)` coordinates via the
//! deterministic PRNG (R7) and asks the page host to evaluate
//! `elementFromPoint` + `getComputedStyle.cursor` at each one.

use super::{Channel, ChannelArtifact, ChannelCtx, ChannelError};
use async_trait::async_trait;

/// @spec parity-dom-reference-runner.md#Dependency (PointerChannel)
pub struct PointerChannel;

/// @spec parity-dom-reference-runner.md#Logic (determinism contract — pointer)
pub const POINTER_SAMPLES: usize = 1000;

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src-channels.md#schema
#[async_trait]
impl Channel for PointerChannel {
    fn name(&self) -> &'static str {
        "pointer"
    }

    /// @spec parity-dom-reference-runner.md#Logic (ch_pointer)
    async fn capture(&self, ctx: &mut ChannelCtx<'_>) -> Result<ChannelArtifact, ChannelError> {
        let (w, h) = ctx.session.page().viewport;
        let mut coords = Vec::with_capacity(POINTER_SAMPLES);
        for _ in 0..POINTER_SAMPLES {
            coords.push(ctx.prng.next_point(w, h));
        }
        let hits = ctx.session.capture_pointer_hits(&coords).await?;
        Ok(ChannelArtifact::Json {
            filename: "pointer-hitmap.json".to_string(),
            value: serde_json::to_value(&hits)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channels::DeterministicPrng;
    use crate::manifest::FixtureManifest;
    use crate::runner::{BrowserKind, BrowserSession, MatrixEntry, StubBrowserSession};

    #[tokio::test]
    async fn pointer_hitmap_has_1000_entries() {
        let mut session = StubBrowserSession::new();
        session.page_mut().viewport = (800, 600);
        let manifest = FixtureManifest {
            name: "mui-button".into(),
            ime: false,
            tab_count: 8,
        };
        let mut ctx = ChannelCtx {
            session: &mut session,
            manifest: &manifest,
            matrix: MatrixEntry {
                browser: BrowserKind::Chromium,
                dpr: 1.0,
            },
            prng: DeterministicPrng::from_fixture_name(&manifest.name),
        };
        let a = PointerChannel.capture(&mut ctx).await.unwrap();
        match a {
            ChannelArtifact::Json { filename, value } => {
                assert_eq!(filename, "pointer-hitmap.json");
                assert_eq!(value.as_array().unwrap().len(), 1000);
            }
            _ => panic!("expected json artifact"),
        }
    }

    #[tokio::test]
    async fn pointer_seed_is_fnv1a_of_fixture_name() {
        // Two runs with the same fixture name → identical coordinate list.
        let manifest = FixtureManifest {
            name: "mui-button".into(),
            ime: false,
            tab_count: 8,
        };

        async fn run_once(manifest: &FixtureManifest) -> Vec<serde_json::Value> {
            let mut session = StubBrowserSession::new();
            session.page_mut().viewport = (800, 600);
            let mut ctx = ChannelCtx {
                session: &mut session,
                manifest,
                matrix: MatrixEntry {
                    browser: BrowserKind::Chromium,
                    dpr: 1.0,
                },
                prng: DeterministicPrng::from_fixture_name(&manifest.name),
            };
            let a = PointerChannel.capture(&mut ctx).await.unwrap();
            match a {
                ChannelArtifact::Json { value, .. } => value.as_array().unwrap().clone(),
                _ => panic!(),
            }
        }
        let a = run_once(&manifest).await;
        let b = run_once(&manifest).await;
        assert_eq!(a, b);
    }
}
// CODEGEN-END
