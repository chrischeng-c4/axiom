//! Progress bar implementation backed by indicatif.

use crate::error::Result;
use std::time::Instant;

/// Re-export indicatif types with wrappers.
pub use indicatif::ProgressStyle;

/// A progress bar with rate/ETA tracking.
pub struct ProgressBar {
    inner: indicatif::ProgressBar,
    start_time: Instant,
    description: Option<String>,
}

impl ProgressBar {
    /// Create a new progress bar with a known total.
    pub fn new(total: u64) -> Self {
        let style = indicatif::ProgressStyle::default_bar()
            .template("{desc}{wide_bar} {pos}/{len} [{elapsed}<{eta}, {per_sec}]")
            .unwrap_or_else(|_| indicatif::ProgressStyle::default_bar());
        let inner = indicatif::ProgressBar::new(total);
        inner.set_style(style);
        Self {
            inner,
            start_time: Instant::now(),
            description: None,
        }
    }

    /// Create a spinner (unknown total).
    pub fn spinner() -> Self {
        let inner = indicatif::ProgressBar::new_spinner();
        Self {
            inner,
            start_time: Instant::now(),
            description: None,
        }
    }

    /// Increment progress by `n`.
    pub fn update(&self, n: u64) {
        self.inner.inc(n);
    }

    /// Set the description text.
    pub fn set_description(&mut self, desc: &str) {
        self.description = Some(desc.to_string());
        self.inner.set_prefix(desc.to_string());
    }

    /// Set postfix (additional info after the bar).
    pub fn set_postfix(&self, postfix: &str) {
        self.inner.set_message(postfix.to_string());
    }

    /// Get current position.
    pub fn position(&self) -> u64 {
        self.inner.position()
    }

    /// Get total length.
    pub fn length(&self) -> Option<u64> {
        self.inner.length()
    }

    /// Get elapsed time in seconds.
    pub fn elapsed_secs(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    /// Get rate (items per second).
    pub fn rate(&self) -> f64 {
        let elapsed = self.elapsed_secs();
        if elapsed > 0.0 {
            self.position() as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Finish the progress bar.
    pub fn finish(&self) {
        self.inner.finish();
    }

    /// Finish with a message.
    pub fn finish_with_message(&self, msg: &str) {
        self.inner.finish_with_message(msg.to_string());
    }

    /// Clear the progress bar.
    pub fn clear(&self) {
        self.inner.finish_and_clear();
    }

    /// Reset the bar.
    pub fn reset(&mut self) {
        self.inner.reset();
        self.start_time = Instant::now();
    }

    /// Set custom style template.
    pub fn set_style(&self, template: &str) -> Result<()> {
        let style = indicatif::ProgressStyle::default_bar().template(template)?;
        self.inner.set_style(style);
        Ok(())
    }
}

/// Manager for multiple concurrent progress bars.
pub struct MultiProgress {
    inner: indicatif::MultiProgress,
}

impl MultiProgress {
    /// Create a new multi-progress container.
    pub fn new() -> Self {
        Self {
            inner: indicatif::MultiProgress::new(),
        }
    }

    /// Add a progress bar to the container.
    pub fn add(&self, total: u64) -> ProgressBar {
        let bar = ProgressBar::new(total);
        let managed = self.inner.add(bar.inner.clone());
        ProgressBar {
            inner: managed,
            start_time: Instant::now(),
            description: None,
        }
    }

    /// Add a spinner to the container.
    pub fn add_spinner(&self) -> ProgressBar {
        let bar = ProgressBar::spinner();
        let managed = self.inner.add(bar.inner.clone());
        ProgressBar {
            inner: managed,
            start_time: Instant::now(),
            description: None,
        }
    }
}

impl Default for MultiProgress {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TqdmError;

    #[test]
    fn progress_bar_tracks_position_length_rate_and_reset() {
        let mut bar = ProgressBar::new(10);
        assert_eq!(bar.position(), 0);
        assert_eq!(bar.length(), Some(10));

        bar.update(3);
        assert_eq!(bar.position(), 3);
        assert!(bar.elapsed_secs() >= 0.0);
        assert!(bar.rate() >= 0.0);

        bar.set_description("loading");
        bar.set_postfix("phase=read");
        bar.reset();
        assert_eq!(bar.position(), 0);
        assert_eq!(bar.length(), Some(10));
        bar.finish();
    }

    #[test]
    fn spinner_supports_unknown_total_progress() {
        let bar = ProgressBar::spinner();
        assert_eq!(bar.length(), None);
        bar.update(1);
        assert_eq!(bar.position(), 1);
        bar.finish_with_message("done");
    }

    #[test]
    fn style_template_and_typed_error_contracts_are_stable() {
        let bar = ProgressBar::new(3);
        bar.set_style("{wide_bar} {pos}/{len}")
            .expect("valid style template should apply");

        let err = TqdmError::InvalidTemplate("missing bar".to_string());
        assert_eq!(err.to_string(), "Invalid style template: missing bar");
    }

    #[test]
    fn multi_progress_manages_bars_and_spinners() {
        let multi = MultiProgress::new();

        let bar = multi.add(5);
        bar.update(2);
        assert_eq!(bar.position(), 2);
        assert_eq!(bar.length(), Some(5));

        let spinner = multi.add_spinner();
        spinner.update(1);
        assert_eq!(spinner.length(), None);
        assert_eq!(spinner.position(), 1);

        bar.clear();
        spinner.clear();
    }
}
