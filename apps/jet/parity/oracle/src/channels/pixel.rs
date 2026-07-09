// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src-channels.md#schema
// CODEGEN-BEGIN

//! Pixel channel — captures a viewport screenshot and emits a PNG.
//! Filename is `<fixture>-<browser>-<dpr>.png` per R4 (the runner passes
//! the pre-computed name in via the constructor).

use super::{Channel, ChannelArtifact, ChannelCtx, ChannelError};
use async_trait::async_trait;

/// @spec parity-dom-reference-runner.md#Dependency (PixelChannel)
pub struct PixelChannel {
    /// Pre-computed `<fixture>-<browser>-<dpr>.png` filename (R4).
    pub filename: String,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src-channels.md#schema
impl PixelChannel {
    pub fn new(filename: String) -> Self {
        Self { filename }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src-channels.md#schema
#[async_trait]
impl Channel for PixelChannel {
    fn name(&self) -> &'static str {
        "pixel"
    }

    /// @spec parity-dom-reference-runner.md#Logic (ch_pixel)
    async fn capture(&self, ctx: &mut ChannelCtx<'_>) -> Result<ChannelArtifact, ChannelError> {
        let bytes = ctx.session.screenshot().await?;
        Ok(ChannelArtifact::Png {
            filename: self.filename.clone(),
            bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channels::DeterministicPrng;
    use crate::manifest::FixtureManifest;
    use crate::runner::{BrowserKind, MatrixEntry, StubBrowserSession};

    #[tokio::test]
    async fn pixel_filename_matches_spec_r4() {
        let mut session = StubBrowserSession::new();
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
        let pix = PixelChannel::new("mui-button-chromium-1.0.png".into());
        let artifact = pix.capture(&mut ctx).await.unwrap();
        match artifact {
            ChannelArtifact::Png { filename, bytes } => {
                assert_eq!(filename, "mui-button-chromium-1.0.png");
                assert_eq!(
                    &bytes[..8],
                    &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
                );
            }
            _ => panic!("expected png artifact"),
        }
    }
}
// CODEGEN-END
