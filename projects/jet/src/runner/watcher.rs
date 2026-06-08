// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
// CODEGEN-BEGIN
//! File watcher for JIT watch mode.
//!
//! Watches source files for changes and triggers re-execution.
//! Uses the `notify` crate for cross-platform file system events.

use anyhow::Result;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// Debounced file watcher that coalesces rapid changes.
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub struct DebouncedWatcher {
    _watcher: RecommendedWatcher,
    rx: mpsc::Receiver<notify::Result<Event>>,
    debounce_ms: u64,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
impl DebouncedWatcher {
    /// Create a new debounced watcher for the given path.
    pub fn new(path: &Path, debounce_ms: u64) -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx)?;
        watcher.watch(path, RecursiveMode::Recursive)?;

        Ok(Self {
            _watcher: watcher,
            rx,
            debounce_ms,
        })
    }

    /// Wait for the next meaningful file change event.
    /// Returns the list of changed file paths.
    pub fn wait_for_change(&self) -> Result<Vec<std::path::PathBuf>> {
        let mut changed = Vec::new();
        let mut last_event = Instant::now();
        let debounce = Duration::from_millis(self.debounce_ms);

        loop {
            let timeout = if changed.is_empty() {
                Duration::from_secs(3600) // wait indefinitely
            } else {
                debounce.saturating_sub(last_event.elapsed())
            };

            match self.rx.recv_timeout(timeout) {
                Ok(Ok(event)) => {
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        for path in event.paths {
                            if is_source_file(&path) && !changed.contains(&path) {
                                changed.push(path);
                            }
                        }
                        last_event = Instant::now();
                    }
                }
                Ok(Err(e)) => {
                    tracing::warn!("Watch error: {:?}", e);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if !changed.is_empty() {
                        return Ok(changed);
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    anyhow::bail!("File watcher disconnected");
                }
            }
        }
    }
}

/// Check if a path is a JS/TS source file worth watching.
fn is_source_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| matches!(ext, "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_source_file() {
        assert!(is_source_file(&PathBuf::from("app.ts")));
        assert!(is_source_file(&PathBuf::from("app.tsx")));
        assert!(is_source_file(&PathBuf::from("app.jsx")));
        assert!(is_source_file(&PathBuf::from("app.js")));
        assert!(is_source_file(&PathBuf::from("app.mjs")));
        assert!(!is_source_file(&PathBuf::from("style.css")));
        assert!(!is_source_file(&PathBuf::from("data.json")));
    }
}
// CODEGEN-END
