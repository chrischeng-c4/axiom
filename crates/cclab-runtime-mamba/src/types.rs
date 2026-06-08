//! Opaque types for the `cclab-runtime-mamba` FFI layer.

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

// ── MbRouter ─────────────────────────────────────────────────────────────────

/// A single route entry consumed by [`MbRouter`].
#[derive(Debug, Clone)]
pub struct MbRouteEntry {
    pub method: String,
    pub path: String,
    pub handler_fn_ptr: usize,
}

/// Minimal Mamba-visible router shape consumed by `runtime.serve`.
#[derive(Debug, Clone)]
pub struct MbRouter {
    pub prefix: String,
    pub routes: Vec<MbRouteEntry>,
}

// ── MbServerHandle ────────────────────────────────────────────────────────────

/// An opaque handle to a running HTTP server instance.
///
/// Created by `mb_runtime_serve` after binding to the given host:port.
/// In this prototype the server is blocking (runs until ctrl-c), so the
/// handle is returned only after the server shuts down.
#[derive(Debug)]
pub struct MbServerHandle {
    /// Bound host address (e.g. `"0.0.0.0"`).
    pub host: String,
    /// Bound port.
    pub port: u16,
    /// Pointer to the `MbRouter` that was used to configure this server.
    /// Stored for introspection only — the server has already been started.
    pub router_ptr: usize,
}

// ── MbTask ────────────────────────────────────────────────────────────────────

/// An opaque handle to a spawned async task.
///
/// Tasks are spawned via `mb_runtime_spawn` onto the module-global Tokio
/// runtime.  The `is_done` flag is set atomically when the task completes.
#[derive(Debug)]
pub struct MbTask {
    /// Monotonically increasing task identifier.
    pub task_id: u64,
    /// Completion flag.
    pub is_done: AtomicBool,
}

impl MbTask {
    pub fn new(task_id: u64) -> Self {
        Self {
            task_id,
            is_done: AtomicBool::new(false),
        }
    }

    pub fn mark_done(&self) {
        self.is_done.store(true, Ordering::Release);
    }

    pub fn done(&self) -> bool {
        self.is_done.load(Ordering::Acquire)
    }
}
