// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src-channels.md#schema
// CODEGEN-BEGIN

//! Focus channel — drives Tab × N and records the active-element trace
//! (R6). The live-browser implementation is delegated to the
//! [`BrowserSession::capture_focus_trace`] hook so tests can substitute
//! a deterministic stub trace.

use super::{Channel, ChannelArtifact, ChannelCtx, ChannelError};
use async_trait::async_trait;

/// @spec parity-dom-reference-runner.md#Dependency (FocusChannel)
pub struct FocusChannel;

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src-channels.md#schema
#[async_trait]
impl Channel for FocusChannel {
    fn name(&self) -> &'static str {
        "focus"
    }

    /// @spec parity-dom-reference-runner.md#Logic (ch_focus)
    async fn capture(&self, ctx: &mut ChannelCtx<'_>) -> Result<ChannelArtifact, ChannelError> {
        let trace = ctx
            .session
            .capture_focus_trace(ctx.manifest.tab_count)
            .await?;
        Ok(ChannelArtifact::Json {
            filename: "focus-trace.json".to_string(),
            value: serde_json::to_value(&trace)?,
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
    async fn focus_trace_has_tab_count_entries() {
        let mut session = StubBrowserSession::new();
        let manifest = FixtureManifest {
            name: "mui-button".into(),
            ime: false,
            tab_count: 5,
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
        let a = FocusChannel.capture(&mut ctx).await.unwrap();
        match a {
            ChannelArtifact::Json { filename, value } => {
                assert_eq!(filename, "focus-trace.json");
                let arr = value.as_array().unwrap();
                assert_eq!(arr.len(), 5);
                // shape check: every entry has the required keys.
                for entry in arr {
                    assert!(entry.get("step").is_some());
                    assert!(entry.get("selector").is_some());
                    assert!(entry.get("role").is_some());
                    assert!(entry.get("name").is_some());
                    assert!(entry.get("bounds").is_some());
                }
            }
            _ => panic!("expected json artifact"),
        }
    }
}
// CODEGEN-END
