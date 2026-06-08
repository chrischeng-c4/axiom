// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
// CODEGEN-BEGIN

//! Runner — drives the §Logic state machine.
//!
//! The runner owns a [`BrowserSession`] trait object so unit tests can
//! substitute a [`StubBrowserSession`] without requiring a live Chromium.
//! The production implementation (`PlaywrightBrowserSession`) launches
//! Chromium via the `playwright` crate; per the issue scope, the live
//! harness is currently `unimplemented!()` and gated on the #2139
//! follow-up.

use crate::artifacts::{ArtifactBundle, ArtifactWriter};
use crate::channels::{
    a11y::A11yChannel, focus::FocusChannel, ime::ImeChannel, pixel::PixelChannel,
    pointer::PointerChannel, Channel, ChannelArtifact, ChannelCtx, ChannelError, DeterministicPrng,
};
use crate::manifest::{FixtureManifest, ManifestError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;

/// @spec parity-dom-reference-runner.md#Dependency (RunnerConfig)
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    pub artifact_root: PathBuf,
    pub shell_html: PathBuf,
    pub per_fixture_budget: Duration,
    pub viewport: (u32, u32),
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            artifact_root: PathBuf::from("artifacts"),
            shell_html: PathBuf::from("fixtures/__shell__/index.html"),
            per_fixture_budget: Duration::from_secs(8),
            viewport: (1024, 768),
        }
    }
}

/// @spec parity-dom-reference-runner.md#Dependency (BrowserSession — browser kind)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BrowserKind {
    Chromium,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
impl BrowserKind {
    pub fn as_str(self) -> &'static str {
        match self {
            BrowserKind::Chromium => "chromium",
        }
    }
}

/// @spec parity-dom-reference-runner.md#Logic (matrix entry; R4)
#[derive(Debug, Clone, Copy)]
pub struct MatrixEntry {
    pub browser: BrowserKind,
    pub dpr: f32,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
impl MatrixEntry {
    pub fn dpr_label(&self) -> String {
        // Stable label like "1.0" / "2.0" for the artifact path.
        format!("{:.1}", self.dpr)
    }
}

/// @spec parity-dom-reference-runner.md#Changes (runner.rs)
#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("manifest error: {0}")]
    Manifest(#[from] ManifestError),
    #[error("artifact error: {0}")]
    Artifact(#[from] crate::artifacts::ArtifactError),
    #[error("channel error in {channel}: {source}")]
    Channel {
        channel: &'static str,
        #[source]
        source: ChannelError,
    },
    #[error("browser launch failed: {0}")]
    LaunchFail(String),
    #[error("mount sentinel timeout after {0:?}")]
    MountTimeout(Duration),
    #[error("per-fixture wall-clock budget exceeded ({0:?})")]
    BudgetExceeded(Duration),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// @spec parity-dom-reference-runner.md#Dependency (PageHost)
///
/// The minimal page-host surface the channels need. In production this is
/// a Playwright `Page` handle; in tests it's `StubPageHost`.
#[derive(Debug, Default, Clone)]
pub struct PageHost {
    pub url: String,
    pub viewport: (u32, u32),
    pub mounted: bool,
}

/// @spec parity-dom-reference-runner.md#Dependency (BrowserSession trait)
///
/// Abstraction over the Chromium-driving harness. The production impl
/// (`PlaywrightBrowserSession`) wraps `playwright::api::Browser`. The
/// stub impl ([`StubBrowserSession`]) returns canned data so tests can
/// exercise the channel orchestration without a live browser.
#[async_trait]
pub trait BrowserSession: Send + Sync {
    fn browser_kind(&self) -> BrowserKind;
    fn page(&self) -> &PageHost;
    fn page_mut(&mut self) -> &mut PageHost;

    async fn launch(&mut self, dpr: f32, viewport: (u32, u32)) -> Result<(), RunnerError>;
    async fn navigate(&mut self, url: &str) -> Result<(), RunnerError>;
    async fn await_mount(&mut self, budget: Duration) -> Result<(), RunnerError>;
    async fn close(&mut self) -> Result<(), RunnerError>;

    /// @spec parity-dom-reference-runner.md#Dependency (PixelChannel -> ArtifactWriter)
    ///
    /// Take a viewport screenshot. Production: `page.screenshot()`.
    async fn screenshot(&mut self) -> Result<Vec<u8>, ChannelError>;

    /// @spec parity-dom-reference-runner.md#Dependency (A11yChannel invokes CDP)
    ///
    /// CDP `Accessibility.getFullAXTree` — verbatim response (R5).
    async fn ax_full_tree(&mut self) -> Result<serde_json::Value, ChannelError>;

    /// @spec parity-dom-reference-runner.md#Dependency (FocusChannel invokes CDP)
    ///
    /// CDP `Input.dispatchKeyEvent` for Tab × N. Returns the focus trace
    /// in the order produced by the channel (R6).
    async fn capture_focus_trace(
        &mut self,
        tab_count: u32,
    ) -> Result<Vec<crate::channels::FocusEntry>, ChannelError>;

    /// @spec parity-dom-reference-runner.md#Dependency (PointerChannel invokes PageHost)
    ///
    /// In-page `elementFromPoint` + `getComputedStyle.cursor` over the
    /// caller-supplied seeded coordinate list (R7).
    async fn capture_pointer_hits(
        &mut self,
        coords: &[(u32, u32)],
    ) -> Result<Vec<crate::channels::PointerHit>, ChannelError>;

    /// @spec parity-dom-reference-runner.md#Dependency (ImeChannel invokes CDP)
    ///
    /// CDP `Input.imeSetComposition` + `Input.insertText` for the fixed CJK
    /// script (R8). Returns the captured event list.
    async fn capture_ime_trace(&mut self) -> Result<Vec<serde_json::Value>, ChannelError>;
}

/// @spec parity-dom-reference-runner.md#Dependency (BrowserSession)
///
/// Production implementation (gated on the #2139 follow-up). Returns
/// `unimplemented!()` until the Playwright driver wiring lands.
pub struct PlaywrightBrowserSession {
    page: PageHost,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
impl PlaywrightBrowserSession {
    pub fn new() -> Self {
        Self {
            page: PageHost::default(),
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
impl Default for PlaywrightBrowserSession {
    fn default() -> Self {
        Self::new()
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
#[async_trait]
impl BrowserSession for PlaywrightBrowserSession {
    fn browser_kind(&self) -> BrowserKind {
        BrowserKind::Chromium
    }
    fn page(&self) -> &PageHost {
        &self.page
    }
    fn page_mut(&mut self) -> &mut PageHost {
        &mut self.page
    }

    async fn launch(&mut self, _dpr: f32, _viewport: (u32, u32)) -> Result<(), RunnerError> {
        unimplemented!("blocked on browser harness — issue #2139 follow-up")
    }
    async fn navigate(&mut self, _url: &str) -> Result<(), RunnerError> {
        unimplemented!("blocked on browser harness — issue #2139 follow-up")
    }
    async fn await_mount(&mut self, _budget: Duration) -> Result<(), RunnerError> {
        unimplemented!("blocked on browser harness — issue #2139 follow-up")
    }
    async fn close(&mut self) -> Result<(), RunnerError> {
        Ok(())
    }
    async fn screenshot(&mut self) -> Result<Vec<u8>, ChannelError> {
        unimplemented!("blocked on browser harness — issue #2139 follow-up")
    }
    async fn ax_full_tree(&mut self) -> Result<serde_json::Value, ChannelError> {
        unimplemented!("blocked on browser harness — issue #2139 follow-up")
    }
    async fn capture_focus_trace(
        &mut self,
        _tab_count: u32,
    ) -> Result<Vec<crate::channels::FocusEntry>, ChannelError> {
        unimplemented!("blocked on browser harness — issue #2139 follow-up")
    }
    async fn capture_pointer_hits(
        &mut self,
        _coords: &[(u32, u32)],
    ) -> Result<Vec<crate::channels::PointerHit>, ChannelError> {
        unimplemented!("blocked on browser harness — issue #2139 follow-up")
    }
    async fn capture_ime_trace(&mut self) -> Result<Vec<serde_json::Value>, ChannelError> {
        unimplemented!("blocked on browser harness — issue #2139 follow-up")
    }
}

/// @spec parity-dom-reference-runner.md#Dependency (BrowserSession — test substitution)
///
/// A canned-data implementation used by unit tests so they can exercise
/// the channel orchestration without a live Chromium.
#[derive(Debug, Clone)]
pub struct StubBrowserSession {
    page: PageHost,
    /// If false, `await_mount` will return `MountTimeout` — exercises T10.
    pub will_mount: bool,
    /// Fixed PNG bytes returned by `screenshot()`.
    pub screenshot_bytes: Vec<u8>,
    /// Fixed AX tree returned by `ax_full_tree()`.
    pub ax_tree: serde_json::Value,
    /// Fixed pointer hits (1000 entries expected per spec); the stub
    /// derives the hit list deterministically from the coord list.
    pub pointer_cursor: String,
    /// Fixed IME event payload list (empty for non-IME fixtures).
    pub ime_events: Vec<serde_json::Value>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
impl StubBrowserSession {
    pub fn new() -> Self {
        // Synthesise a deterministic 2x2 PNG so the pixel channel always
        // has something well-formed to ship.
        let img = image::RgbaImage::from_pixel(2, 2, image::Rgba([0, 0, 0, 255]));
        let mut bytes = Vec::new();
        {
            let mut cursor = std::io::Cursor::new(&mut bytes);
            image::DynamicImage::ImageRgba8(img)
                .write_to(&mut cursor, image::ImageFormat::Png)
                .unwrap();
        }
        Self {
            page: PageHost {
                url: String::new(),
                viewport: (1024, 768),
                mounted: false,
            },
            will_mount: true,
            screenshot_bytes: bytes,
            ax_tree: serde_json::json!({"nodes": []}),
            pointer_cursor: "auto".into(),
            ime_events: vec![],
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
impl Default for StubBrowserSession {
    fn default() -> Self {
        Self::new()
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
#[async_trait]
impl BrowserSession for StubBrowserSession {
    fn browser_kind(&self) -> BrowserKind {
        BrowserKind::Chromium
    }
    fn page(&self) -> &PageHost {
        &self.page
    }
    fn page_mut(&mut self) -> &mut PageHost {
        &mut self.page
    }
    async fn launch(&mut self, _dpr: f32, viewport: (u32, u32)) -> Result<(), RunnerError> {
        self.page.viewport = viewport;
        Ok(())
    }
    async fn navigate(&mut self, url: &str) -> Result<(), RunnerError> {
        self.page.url = url.to_string();
        Ok(())
    }
    async fn await_mount(&mut self, budget: Duration) -> Result<(), RunnerError> {
        if self.will_mount {
            self.page.mounted = true;
            Ok(())
        } else {
            Err(RunnerError::MountTimeout(budget))
        }
    }
    async fn close(&mut self) -> Result<(), RunnerError> {
        Ok(())
    }
    async fn screenshot(&mut self) -> Result<Vec<u8>, ChannelError> {
        Ok(self.screenshot_bytes.clone())
    }
    async fn ax_full_tree(&mut self) -> Result<serde_json::Value, ChannelError> {
        Ok(self.ax_tree.clone())
    }
    async fn capture_focus_trace(
        &mut self,
        tab_count: u32,
    ) -> Result<Vec<crate::channels::FocusEntry>, ChannelError> {
        Ok((0..tab_count)
            .map(|i| crate::channels::FocusEntry {
                step: i,
                selector: "<body>".into(),
                role: "generic".into(),
                name: String::new(),
                bounds: [0.0, 0.0, 0.0, 0.0],
            })
            .collect())
    }
    async fn capture_pointer_hits(
        &mut self,
        coords: &[(u32, u32)],
    ) -> Result<Vec<crate::channels::PointerHit>, ChannelError> {
        Ok(coords
            .iter()
            .map(|&(x, y)| crate::channels::PointerHit {
                x,
                y,
                target_selector: "body".into(),
                computed_cursor: self.pointer_cursor.clone(),
            })
            .collect())
    }
    async fn capture_ime_trace(&mut self) -> Result<Vec<serde_json::Value>, ChannelError> {
        Ok(self.ime_events.clone())
    }
}

/// @spec parity-dom-reference-runner.md#Dependency (Runner)
pub struct Runner {
    config: RunnerConfig,
    matrix: MatrixEntry,
    session: Box<dyn BrowserSession>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
impl Runner {
    /// @spec parity-dom-reference-runner.md#Logic (init)
    ///
    /// Build a runner backed by the production [`PlaywrightBrowserSession`].
    pub fn new(config: RunnerConfig, matrix: MatrixEntry) -> Self {
        Self {
            config,
            matrix,
            session: Box::new(PlaywrightBrowserSession::new()),
        }
    }

    /// @spec parity-dom-reference-runner.md#Logic (init — test substitution)
    pub fn with_session(
        config: RunnerConfig,
        matrix: MatrixEntry,
        session: Box<dyn BrowserSession>,
    ) -> Self {
        Self {
            config,
            matrix,
            session,
        }
    }

    /// @spec parity-dom-reference-runner.md#Logic
    ///
    /// Walks the lifecycle:
    /// `load -> launch -> navigate -> wait_paint -> ch_pixel -> ch_a11y
    ///  -> ch_focus -> ch_pointer -> {ch_ime | ime_empty} -> bundle -> teardown -> done`.
    pub async fn run(&mut self, fixture_path: &Path) -> Result<ArtifactBundle, RunnerError> {
        // load
        let manifest = FixtureManifest::from_file(fixture_path)?;

        // launch
        self.session
            .launch(self.matrix.dpr, self.config.viewport)
            .await?;
        // navigate
        let url = format!(
            "file://{}?fixture={}",
            self.config.shell_html.display(),
            fixture_path.display()
        );
        self.session.navigate(&url).await?;
        // wait_paint
        self.session
            .await_mount(self.config.per_fixture_budget)
            .await?;

        // Per-fixture artifact root: artifacts/<fixture>/<browser>-<dpr>/
        let root = self.config.artifact_root.join(&manifest.name).join(format!(
            "{}-{}",
            self.matrix.browser.as_str(),
            self.matrix.dpr_label()
        ));
        let mut writer = ArtifactWriter::new(root.clone())?;

        // The five channels run in fixed order. We pre-build them with the
        // pixel filename embedded so PixelChannel knows the R4 naming
        // convention `<fixture>-<browser>-<dpr>.png`.
        let pixel_filename = format!(
            "{}-{}-{}.png",
            manifest.name,
            self.matrix.browser.as_str(),
            self.matrix.dpr_label()
        );
        let prng = DeterministicPrng::from_fixture_name(&manifest.name);
        let mut ctx = ChannelCtx {
            session: self.session.as_mut(),
            manifest: &manifest,
            matrix: self.matrix,
            prng,
        };

        // ch_pixel
        let pixel = PixelChannel::new(pixel_filename.clone());
        run_channel(&pixel, &mut ctx, &mut writer).await?;
        // ch_a11y
        run_channel(&A11yChannel, &mut ctx, &mut writer).await?;
        // ch_focus
        run_channel(&FocusChannel, &mut ctx, &mut writer).await?;
        // ch_pointer
        run_channel(&PointerChannel, &mut ctx, &mut writer).await?;
        // ch_ime (or empty trace if non-IME — handled inside channel)
        run_channel(&ImeChannel, &mut ctx, &mut writer).await?;

        // bundle
        let sha = writer.into_sha256s();
        let bundle = ArtifactBundle {
            root_dir: root.clone(),
            pixel_png: root.join(&pixel_filename),
            a11y_json: root.join("a11y-tree.json"),
            focus_json: root.join("focus-trace.json"),
            pointer_json: root.join("pointer-hitmap.json"),
            ime_json: root.join("ime-trace.json"),
            sha256s: sha,
        };
        // teardown
        self.session.close().await?;
        Ok(bundle)
    }
}

async fn run_channel(
    channel: &dyn Channel,
    ctx: &mut ChannelCtx<'_>,
    writer: &mut ArtifactWriter,
) -> Result<(), RunnerError> {
    let artifact = channel
        .capture(ctx)
        .await
        .map_err(|source| RunnerError::Channel {
            channel: channel.name(),
            source,
        })?;
    match artifact {
        ChannelArtifact::Png { filename, bytes } => {
            writer.write_png(&filename, &bytes)?;
        }
        ChannelArtifact::Json { filename, value } => {
            writer.write_json(&filename, &value)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_dpr_label_stable() {
        let m = MatrixEntry {
            browser: BrowserKind::Chromium,
            dpr: 1.0,
        };
        assert_eq!(m.dpr_label(), "1.0");
        let m2 = MatrixEntry {
            browser: BrowserKind::Chromium,
            dpr: 2.0,
        };
        assert_eq!(m2.dpr_label(), "2.0");
    }

    #[tokio::test]
    async fn stub_session_round_trip() {
        let mut s = StubBrowserSession::new();
        s.launch(1.0, (800, 600)).await.unwrap();
        s.navigate("file:///x").await.unwrap();
        s.await_mount(Duration::from_secs(1)).await.unwrap();
        assert!(s.page().mounted);
        assert_eq!(s.page().url, "file:///x");
        let png = s.screenshot().await.unwrap();
        assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
    }

    #[tokio::test]
    async fn stub_mount_failure_yields_mount_timeout() {
        let mut s = StubBrowserSession::new();
        s.will_mount = false;
        let err = s.await_mount(Duration::from_millis(10)).await.unwrap_err();
        assert!(matches!(err, RunnerError::MountTimeout(_)));
    }
}
// CODEGEN-END
