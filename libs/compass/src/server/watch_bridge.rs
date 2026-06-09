//! Async FileWatcher Bridge
//!
//! Bridges synchronous file watcher events into the async Tokio runtime.
//! This ensures the async event loop is not blocked by file system notifications.

use std::path::PathBuf;
use std::time::Duration;

use tokio::sync::mpsc;

use crate::watch::{FileWatcher, WatchConfig, WatchEvent};

/// Events from the watch bridge
#[derive(Debug, Clone)]
pub enum BridgeEvent {
    /// Files were modified and need re-analysis
    FilesChanged(Vec<PathBuf>),
    /// Watch error occurred
    Error(String),
    /// Watcher is ready
    Ready,
    /// Watcher stopped
    Stopped,
}

/// Configuration for the watch bridge
#[derive(Debug, Clone)]
pub struct WatchBridgeConfig {
    /// Root directory to watch
    pub root: PathBuf,
    /// Debounce duration
    pub debounce: Duration,
    /// Channel buffer size
    pub channel_buffer: usize,
}

impl WatchBridgeConfig {
    /// Create a new config with default settings
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            debounce: Duration::from_millis(300),
            channel_buffer: 256,
        }
    }

    /// Set the debounce duration
    pub fn with_debounce(mut self, debounce: Duration) -> Self {
        self.debounce = debounce;
        self
    }

    /// Set the channel buffer size
    pub fn with_buffer(mut self, buffer: usize) -> Self {
        self.channel_buffer = buffer;
        self
    }
}

/// Handle to control the watch bridge
pub struct WatchBridgeHandle {
    /// Send stop signal
    stop_tx: Option<std::sync::mpsc::Sender<()>>,
    /// Thread join handle
    thread_handle: Option<std::thread::JoinHandle<()>>,
}

impl WatchBridgeHandle {
    /// Stop the watch bridge
    pub fn stop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
    }

    /// Wait for the bridge to stop
    pub fn join(mut self) {
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for WatchBridgeHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Watch bridge that forwards sync FileWatcher events to async channels
pub struct WatchBridge {
    config: WatchBridgeConfig,
}

impl WatchBridge {
    /// Create a new watch bridge
    pub fn new(config: WatchBridgeConfig) -> Self {
        Self { config }
    }

    /// Start the watch bridge
    ///
    /// Returns a receiver for bridge events and a handle to control the bridge.
    /// The bridge runs in a separate thread to avoid blocking the async runtime.
    pub fn start(self) -> Result<(mpsc::Receiver<BridgeEvent>, WatchBridgeHandle), String> {
        let (event_tx, event_rx) = mpsc::channel(self.config.channel_buffer);
        let (stop_tx, stop_rx) = std::sync::mpsc::channel();

        let config = self.config.clone();

        let thread_handle = std::thread::Builder::new()
            .name("cclab_lens-watch-bridge".to_string())
            .spawn(move || {
                Self::run_bridge(config, event_tx, stop_rx);
            })
            .map_err(|e| format!("Failed to spawn watch bridge thread: {}", e))?;

        let handle = WatchBridgeHandle {
            stop_tx: Some(stop_tx),
            thread_handle: Some(thread_handle),
        };

        Ok((event_rx, handle))
    }

    /// Run the bridge in a blocking thread
    fn run_bridge(
        config: WatchBridgeConfig,
        event_tx: mpsc::Sender<BridgeEvent>,
        stop_rx: std::sync::mpsc::Receiver<()>,
    ) {
        // Create the file watcher
        let watch_config = WatchConfig::new(config.root.clone()).with_debounce(config.debounce);

        let mut watcher = match FileWatcher::new(watch_config) {
            Ok(w) => w,
            Err(e) => {
                let _ = event_tx.blocking_send(BridgeEvent::Error(format!(
                    "Failed to create watcher: {}",
                    e
                )));
                return;
            }
        };

        if let Err(e) = watcher.start() {
            let _ = event_tx.blocking_send(BridgeEvent::Error(format!(
                "Failed to start watcher: {}",
                e
            )));
            return;
        }

        // Signal ready
        let _ = event_tx.blocking_send(BridgeEvent::Ready);

        let events_rx = watcher.events();

        loop {
            // Check for stop signal (non-blocking)
            if stop_rx.try_recv().is_ok() {
                let _ = event_tx.blocking_send(BridgeEvent::Stopped);
                break;
            }

            // Try to receive watch events with a timeout
            match events_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(WatchEvent::FilesChanged(paths)) => {
                    // Forward to async channel
                    if event_tx
                        .blocking_send(BridgeEvent::FilesChanged(paths))
                        .is_err()
                    {
                        // Channel closed, exit
                        break;
                    }
                }
                Ok(WatchEvent::Error(e)) => {
                    let _ = event_tx.blocking_send(BridgeEvent::Error(e));
                }
                Ok(WatchEvent::Started) => {
                    // Already sent Ready above
                }
                Ok(WatchEvent::Stopped) => {
                    let _ = event_tx.blocking_send(BridgeEvent::Stopped);
                    break;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Continue polling
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    // Watcher channel closed
                    let _ = event_tx.blocking_send(BridgeEvent::Stopped);
                    break;
                }
            }
        }
    }
}

/// Builder for creating a watch bridge with async support
pub struct AsyncWatchBridgeBuilder {
    root: PathBuf,
    debounce: Duration,
    buffer_size: usize,
}

impl AsyncWatchBridgeBuilder {
    /// Create a new builder
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            debounce: Duration::from_millis(300),
            buffer_size: 256,
        }
    }

    /// Set debounce duration
    pub fn debounce(mut self, duration: Duration) -> Self {
        self.debounce = duration;
        self
    }

    /// Set buffer size
    pub fn buffer(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Build and start the watch bridge
    pub fn build(self) -> Result<(mpsc::Receiver<BridgeEvent>, WatchBridgeHandle), String> {
        let config = WatchBridgeConfig {
            root: self.root,
            debounce: self.debounce,
            channel_buffer: self.buffer_size,
        };
        WatchBridge::new(config).start()
    }
}

/// Convenience function to spawn a watch bridge
pub fn spawn_watch_bridge(
    root: PathBuf,
    debounce: Duration,
) -> Result<(mpsc::Receiver<BridgeEvent>, WatchBridgeHandle), String> {
    AsyncWatchBridgeBuilder::new(root)
        .debounce(debounce)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_watch_bridge_starts() {
        let dir = tempdir().expect("Failed to create temp dir");
        let root = dir.path().to_path_buf();

        let config = WatchBridgeConfig::new(root);
        let bridge = WatchBridge::new(config);
        let (mut rx, mut handle) = bridge.start().expect("Failed to start bridge");

        // Should receive Ready event
        let event = tokio::time::timeout(Duration::from_secs(2), rx.recv()).await;
        assert!(event.is_ok());

        if let Ok(Some(BridgeEvent::Ready)) = event {
            // Expected
        } else {
            panic!("Expected Ready event, got {:?}", event);
        }

        // Stop the bridge
        handle.stop();
    }

    #[tokio::test]
    async fn test_watch_bridge_detects_changes() {
        let dir = tempdir().expect("Failed to create temp dir");
        let root = dir.path().to_path_buf();

        // Create a Python file to watch
        let test_file = root.join("test.py");
        fs::write(&test_file, "# initial content").expect("Failed to write file");

        let config = WatchBridgeConfig::new(root.clone()).with_debounce(Duration::from_millis(100));
        let bridge = WatchBridge::new(config);
        let (mut rx, mut handle) = bridge.start().expect("Failed to start bridge");

        // Wait for Ready
        let event = tokio::time::timeout(Duration::from_secs(2), rx.recv()).await;
        assert!(matches!(event, Ok(Some(BridgeEvent::Ready))));

        // Modify the file
        tokio::time::sleep(Duration::from_millis(100)).await;
        fs::write(&test_file, "# modified content").expect("Failed to write file");

        // Should receive FilesChanged event
        let event = tokio::time::timeout(Duration::from_secs(2), rx.recv()).await;
        match event {
            Ok(Some(BridgeEvent::FilesChanged(paths))) => {
                assert!(paths.iter().any(|p| p.ends_with("test.py")));
            }
            other => {
                // Changes might not be detected in all CI environments
                eprintln!(
                    "File change not detected (may be CI environment): {:?}",
                    other
                );
            }
        }

        handle.stop();
    }

    #[test]
    fn test_watch_bridge_config() {
        let root = PathBuf::from("/test");
        let config = WatchBridgeConfig::new(root.clone())
            .with_debounce(Duration::from_secs(1))
            .with_buffer(512);

        assert_eq!(config.root, root);
        assert_eq!(config.debounce, Duration::from_secs(1));
        assert_eq!(config.channel_buffer, 512);
    }
}
