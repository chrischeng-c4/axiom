// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
// CODEGEN-BEGIN
use anyhow::Result;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;
use tokio::sync::broadcast;

/// Debounce window in milliseconds.
/// Editors often save-then-rename which produces duplicate events.
const DEBOUNCE_MS: u128 = 50;

/// File watcher for detecting changes with debouncing.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    tx: broadcast::Sender<PathBuf>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
impl FileWatcher {
    pub fn new(root_dir: PathBuf) -> Result<Self> {
        let (tx, _) = broadcast::channel(100);
        let tx_clone = tx.clone();
        let last_seen: std::sync::Arc<Mutex<HashMap<PathBuf, Instant>>> =
            std::sync::Arc::new(Mutex::new(HashMap::new()));

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            match res {
                Ok(event) => {
                    for path in event.paths {
                        if should_ignore(&path) {
                            continue;
                        }

                        // Debounce: skip if same path was seen within DEBOUNCE_MS
                        let now = Instant::now();
                        {
                            let mut map = last_seen.lock().unwrap();
                            if let Some(prev) = map.get(&path) {
                                if now.duration_since(*prev).as_millis() < DEBOUNCE_MS {
                                    continue;
                                }
                            }
                            map.insert(path.clone(), now);

                            // Evict stale entries to prevent unbounded growth.
                            // Keep map under 10k entries by removing entries older than 60s.
                            if map.len() > 10_000 {
                                let cutoff = now
                                    .checked_sub(std::time::Duration::from_secs(60))
                                    .unwrap_or(now);
                                map.retain(|_, ts| *ts > cutoff);
                            }
                        }

                        let _ = tx_clone.send(path);
                    }
                }
                // GH #3127 — surface notify backend errors instead of
                // swallowing them. Without this the dev sees HMR
                // silently stop: inotify watch overflow, FSEvents
                // stream invalidation, ENFILE, or watch-root removal
                // all just kill event flow with zero diagnostic.
                Err(e) => {
                    let msg = format_watch_error(&e.to_string());
                    tracing::warn!(target: "jet::dev::watcher", "{msg}");
                }
            }
        })?;

        watcher.watch(&root_dir, RecursiveMode::Recursive)?;

        Ok(Self {
            _watcher: watcher,
            tx,
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<PathBuf> {
        self.tx.subscribe()
    }
}

/// Build the warning message emitted when the watcher backend yields
/// an error. Extracted so the formatting can be exercised by unit
/// tests without having to provoke a real `notify::Error` (which is
/// platform-specific and effectively non-constructible in tests).
///
/// GH #3127 — the message must name the underlying error AND point
/// the dev at the issue tag so the breadcrumb is searchable in
/// commit history / docs when it eventually surfaces.
fn format_watch_error(underlying: &str) -> String {
    format!(
        "file watcher backend error: {underlying}; some HMR updates may stop arriving (GH #3127)"
    )
}

fn should_ignore(path: &PathBuf) -> bool {
    let path_str = path.to_string_lossy();

    const IGNORE_PATTERNS: &[&str] = &[
        "node_modules",
        ".git",
        "dist",
        "build",
        ".jet-cache",
        "target",
        ".DS_Store",
    ];

    IGNORE_PATTERNS
        .iter()
        .any(|pattern| path_str.contains(pattern))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_ignore() {
        assert!(should_ignore(&PathBuf::from("node_modules/react/index.js")));
        assert!(should_ignore(&PathBuf::from(".git/config")));
        assert!(!should_ignore(&PathBuf::from("src/App.tsx")));
    }

    /// GH #3127 — the watcher backend error message must name the
    /// underlying error AND include the issue tag so the dev has a
    /// breadcrumb when HMR silently stops.
    #[test]
    fn watch_error_message_names_underlying_and_issue() {
        let msg = format_watch_error("EBADF: Bad file descriptor");
        assert!(
            msg.contains("EBADF: Bad file descriptor"),
            "underlying error must be preserved verbatim, got: {msg}"
        );
        assert!(
            msg.contains("GH #3127"),
            "must include searchable issue tag, got: {msg}"
        );
        assert!(
            msg.contains("file watcher"),
            "must name the failing component, got: {msg}"
        );
    }
}
// CODEGEN-END
