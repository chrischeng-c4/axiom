// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src-channels.md#schema
// CODEGEN-BEGIN

//! Accessibility channel — verbatim `Accessibility.getFullAXTree`
//! response, no filtering (R5).

use super::{Channel, ChannelArtifact, ChannelCtx, ChannelError};
use async_trait::async_trait;

/// @spec parity-dom-reference-runner.md#Dependency (A11yChannel)
pub struct A11yChannel;

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src-channels.md#schema
#[async_trait]
impl Channel for A11yChannel {
    fn name(&self) -> &'static str {
        "a11y"
    }

    /// @spec parity-dom-reference-runner.md#Logic (ch_a11y)
    async fn capture(&self, ctx: &mut ChannelCtx<'_>) -> Result<ChannelArtifact, ChannelError> {
        let tree = ctx.session.ax_full_tree().await?;
        Ok(ChannelArtifact::Json {
            filename: "a11y-tree.json".to_string(),
            value: tree,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channels::DeterministicPrng;
    use crate::manifest::FixtureManifest;
    use crate::runner::{BrowserKind, MatrixEntry, StubBrowserSession};
    use serde_json::json;

    #[tokio::test]
    async fn a11y_artifact_is_verbatim_axtree() {
        let mut session = StubBrowserSession::new();
        session.ax_tree = json!({"nodes": [{"nodeId": "1", "role": "button"}]});
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
        let a = A11yChannel.capture(&mut ctx).await.unwrap();
        match a {
            ChannelArtifact::Json { filename, value } => {
                assert_eq!(filename, "a11y-tree.json");
                // Verbatim — no filtering / mutation.
                assert_eq!(value, json!({"nodes": [{"nodeId": "1", "role": "button"}]}));
            }
            _ => panic!("expected json artifact"),
        }
    }
}
// CODEGEN-END
