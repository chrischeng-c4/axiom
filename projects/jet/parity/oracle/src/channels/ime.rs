// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src-channels.md#schema
// CODEGEN-BEGIN

//! IME channel — for `ime: true` fixtures, drives the fixed CJK pinyin
//! script via CDP and captures the composition events; for non-IME
//! fixtures, writes `{"events": []}` (R3 says missing artifact = failure;
//! R8 says non-IME fixtures still emit a well-formed empty trace).

use super::{Channel, ChannelArtifact, ChannelCtx, ChannelError};
use async_trait::async_trait;
use serde_json::json;

/// @spec parity-dom-reference-runner.md#Dependency (ImeChannel)
pub struct ImeChannel;

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src-channels.md#schema
#[async_trait]
impl Channel for ImeChannel {
    fn name(&self) -> &'static str {
        "ime"
    }

    /// @spec parity-dom-reference-runner.md#Logic (ime_q / ch_ime / ime_empty)
    async fn capture(&self, ctx: &mut ChannelCtx<'_>) -> Result<ChannelArtifact, ChannelError> {
        let events = if ctx.manifest.ime {
            ctx.session.capture_ime_trace().await?
        } else {
            Vec::new()
        };
        Ok(ChannelArtifact::Json {
            filename: "ime-trace.json".to_string(),
            value: json!({ "events": events }),
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
    async fn non_ime_fixture_writes_empty_events() {
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
        let a = ImeChannel.capture(&mut ctx).await.unwrap();
        match a {
            ChannelArtifact::Json { filename, value } => {
                assert_eq!(filename, "ime-trace.json");
                assert_eq!(value, json!({"events": []}));
            }
            _ => panic!("expected json artifact"),
        }
    }

    #[tokio::test]
    async fn ime_fixture_captures_session_events() {
        let mut session = StubBrowserSession::new();
        session.ime_events = vec![
            json!({"type": "compositionstart", "data": ""}),
            json!({"type": "compositionupdate", "data": "ni"}),
            json!({"type": "compositionend", "data": "你"}),
            json!({"type": "input", "data": "你"}),
        ];
        let manifest = FixtureManifest {
            name: "mui-text-field".into(),
            ime: true,
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
        let a = ImeChannel.capture(&mut ctx).await.unwrap();
        match a {
            ChannelArtifact::Json { value, .. } => {
                let events = value.get("events").unwrap().as_array().unwrap();
                assert_eq!(events.len(), 4);
                assert_eq!(events[0]["type"], "compositionstart");
            }
            _ => panic!("expected json"),
        }
    }
}
// CODEGEN-END
