//! Watch mode for automatic re-analysis
//!
//! Provides file system watching with debouncing for incremental analysis.

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Default debounce duration in milliseconds
const DEFAULT_DEBOUNCE_MS: u64 = 300;

/// Events from the watch system
#[derive(Debug, Clone)]
pub enum WatchEvent {
    /// Files changed and need re-analysis
    FilesChanged(Vec<PathBuf>),
    /// Watch error occurred
    Error(String),
    /// Watcher started
    Started,
    /// Watcher stopped
    Stopped,
}

/// Configuration for the file watcher
#[derive(Debug, Clone)]
pub struct WatchConfig {
    /// Root directory to watch
    pub root: PathBuf,
    /// Debounce duration for rapid changes
    pub debounce: Duration,
    /// File extensions to watch (e.g., ["py", "pyi", "ts", "rs"])
    pub extensions: Vec<String>,
    /// Patterns to exclude from watching
    pub exclude_patterns: Vec<String>,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            debounce: Duration::from_millis(DEFAULT_DEBOUNCE_MS),
            extensions: vec![
                "py".to_string(),
                "pyi".to_string(),
                "ts".to_string(),
                "tsx".to_string(),
                "rs".to_string(),
            ],
            exclude_patterns: vec![
                "__pycache__".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                ".git".to_string(),
                ".venv".to_string(),
                "venv".to_string(),
            ],
        }
    }
}

impl WatchConfig {
    /// Create a new watch config for the given root directory
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            ..Default::default()
        }
    }

    /// Set the debounce duration
    pub fn with_debounce(mut self, debounce: Duration) -> Self {
        self.debounce = debounce;
        self
    }

    /// Set the file extensions to watch
    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = extensions;
        self
    }

    /// Check if a path should be excluded
    fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.exclude_patterns.iter().any(|p| path_str.contains(p))
    }

    /// Check if a path has a watched extension
    fn has_watched_extension(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy();
            self.extensions.iter().any(|e| e == ext_str.as_ref())
        } else {
            false
        }
    }

    /// Check if a path should be watched
    fn should_watch(&self, path: &Path) -> bool {
        !self.is_excluded(path) && self.has_watched_extension(path)
    }
}

/// File system watcher with debouncing
pub struct FileWatcher {
    /// Configuration
    config: WatchConfig,
    /// Event sender for communicating with the main thread
    event_tx: Sender<WatchEvent>,
    /// Event receiver
    event_rx: Receiver<WatchEvent>,
    /// The actual watcher (stored to keep it alive)
    _watcher: Option<RecommendedWatcher>,
    /// Background thread handle
    _thread: Option<JoinHandle<()>>,
    /// Pending changes (for debouncing)
    pending: Arc<Mutex<PendingChanges>>,
}

/// Tracks pending changes with debouncing
struct PendingChanges {
    /// Set of changed paths
    paths: HashSet<PathBuf>,
    /// Last change time
    last_change: Option<Instant>,
}

impl PendingChanges {
    fn new() -> Self {
        Self {
            paths: HashSet::new(),
            last_change: None,
        }
    }

    fn add(&mut self, path: PathBuf) {
        self.paths.insert(path);
        self.last_change = Some(Instant::now());
    }

    fn take(&mut self) -> Vec<PathBuf> {
        self.last_change = None;
        self.paths.drain().collect()
    }

    fn should_flush(&self, debounce: Duration) -> bool {
        if let Some(last) = self.last_change {
            last.elapsed() >= debounce && !self.paths.is_empty()
        } else {
            false
        }
    }
}

impl FileWatcher {
    /// Create a new file watcher with the given configuration
    pub fn new(config: WatchConfig) -> Result<Self, String> {
        let (event_tx, event_rx) = mpsc::channel();
        let pending = Arc::new(Mutex::new(PendingChanges::new()));

        Ok(Self {
            config,
            event_tx,
            event_rx,
            _watcher: None,
            _thread: None,
            pending,
        })
    }

    /// Start watching for file changes
    pub fn start(&mut self) -> Result<(), String> {
        let root = self.config.root.clone();
        let config = self.config.clone();
        let pending = Arc::clone(&self.pending);
        let event_tx = self.event_tx.clone();

        // Create the notify watcher
        let pending_for_watcher = Arc::clone(&pending);
        let config_for_watcher = config.clone();

        let watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    // Filter for relevant events (create, modify, remove)
                    match event.kind {
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                            for path in event.paths {
                                if config_for_watcher.should_watch(&path) {
                                    if let Ok(mut pending) = pending_for_watcher.lock() {
                                        pending.add(path);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    // Log error but continue
                    eprintln!("Watch error: {:?}", e);
                }
            }
        })
        .map_err(|e| format!("Failed to create watcher: {}", e))?;

        self._watcher = Some(watcher);

        // Start watching the root directory
        if let Some(ref mut watcher) = self._watcher {
            watcher
                .watch(&root, RecursiveMode::Recursive)
                .map_err(|e| format!("Failed to watch directory: {}", e))?;
        }

        // Start the debounce thread
        let debounce = config.debounce;
        let pending_for_thread = Arc::clone(&pending);
        let event_tx_for_thread = event_tx.clone();

        let thread = thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(50)); // Check every 50ms

                let should_flush = {
                    if let Ok(pending) = pending_for_thread.lock() {
                        pending.should_flush(debounce)
                    } else {
                        false
                    }
                };

                if should_flush {
                    let paths = {
                        if let Ok(mut pending) = pending_for_thread.lock() {
                            pending.take()
                        } else {
                            vec![]
                        }
                    };

                    if !paths.is_empty() {
                        let _ = event_tx_for_thread.send(WatchEvent::FilesChanged(paths));
                    }
                }
            }
        });

        self._thread = Some(thread);

        let _ = self.event_tx.send(WatchEvent::Started);
        Ok(())
    }

    /// Get the event receiver for handling watch events
    pub fn events(&self) -> &Receiver<WatchEvent> {
        &self.event_rx
    }

    /// Manually trigger analysis for specific paths
    pub fn trigger_analysis(&self, paths: Vec<PathBuf>) {
        let _ = self.event_tx.send(WatchEvent::FilesChanged(paths));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch_config_default() {
        let config = WatchConfig::default();
        assert_eq!(config.debounce, Duration::from_millis(DEFAULT_DEBOUNCE_MS));
        assert!(config.extensions.contains(&"py".to_string()));
        assert!(config.extensions.contains(&"ts".to_string()));
        assert!(config.extensions.contains(&"rs".to_string()));
    }

    #[test]
    fn test_watch_config_exclusions() {
        let config = WatchConfig::default();
        assert!(config.is_excluded(Path::new("/project/__pycache__/foo.py")));
        assert!(config.is_excluded(Path::new("/project/node_modules/pkg/index.ts")));
        assert!(!config.is_excluded(Path::new("/project/src/main.py")));
    }

    #[test]
    fn test_watch_config_extensions() {
        let config = WatchConfig::default();
        assert!(config.has_watched_extension(Path::new("test.py")));
        assert!(config.has_watched_extension(Path::new("test.ts")));
        assert!(config.has_watched_extension(Path::new("test.rs")));
        assert!(!config.has_watched_extension(Path::new("test.txt")));
        assert!(!config.has_watched_extension(Path::new("test.json")));
    }

    #[test]
    fn test_should_watch() {
        let config = WatchConfig::default();
        assert!(config.should_watch(Path::new("/project/src/main.py")));
        assert!(!config.should_watch(Path::new("/project/__pycache__/main.pyc")));
        assert!(!config.should_watch(Path::new("/project/src/README.md")));
    }

    #[test]
    fn test_pending_changes() {
        let mut pending = PendingChanges::new();
        assert!(!pending.should_flush(Duration::from_millis(100)));

        pending.add(PathBuf::from("/test/file.py"));
        assert!(!pending.should_flush(Duration::from_millis(100)));

        // Can't easily test timing without sleeping in tests
        // but the structure is correct
    }
}
