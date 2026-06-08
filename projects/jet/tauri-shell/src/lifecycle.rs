// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
// CODEGEN-BEGIN
//! OS-window and process lifecycle pulses for the desktop target.
//!
//! @spec `.score/tech_design/projects/jet/logic/multi-target/desktop-runtime.md`
//!     §"Public surface" — "Window lifecycle (created, focused,
//!     minimized, suspended, resumed, closed)" is the desktop-only
//!     surface the web profile cannot express.
//! @issue #1242 — Slice 2c (this commit): the typed event enum +
//!     listener trait + in-process bus, all behind no `tauri` dep.
//!     Slice 2b will dispatch into [`LifecycleBus`] from inside the
//!     `tauri::Builder::on_window_event` / `on_run_event` hooks.
//!
//! The bus owns every listener and clones each event before
//! dispatching, so listeners can outlive a single event without
//! borrowing from the source. This shape is small enough to be
//! useful today (tests + headless harnesses can pump events through
//! the bus directly) and forward-compatible with the real Tauri
//! event-loop wiring landing in Slice 2b.

use std::sync::{Arc, Mutex};

/// One discrete OS-window or process pulse the desktop target
/// emits. The web profile has no analogue — these are the
/// degradation-rule **additions** the desktop capability matrix
/// notes (per `target-profiles.md` §"Desktop profile").
///
/// `WindowId` is opaque (the Tauri label string for the window the
/// pulse fired on) so multi-window support drops in without an
/// enum break — single-window apps see all events tagged
/// `WindowId("main")` (the default Tauri label).
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleEvent {
    /// A new OS window has been created and is ready to host a
    /// webview. Fired exactly once per window.
    WindowCreated { window: WindowId },
    /// Window gained keyboard focus.
    WindowFocused { window: WindowId },
    /// Window lost keyboard focus.
    WindowBlurred { window: WindowId },
    /// Window was minimized to the dock / taskbar.
    WindowMinimized { window: WindowId },
    /// Window was un-minimized (or restored from any non-normal
    /// state). Distinct from [`WindowFocused`](Self::WindowFocused)
    /// — restoration can happen without focus changing.
    WindowRestored { window: WindowId },
    /// Window was resized. Width/height in CSS pixels (the size the
    /// inner webview sees), matching how the web profile reports
    /// `resize` events.
    WindowResized {
        window: WindowId,
        width: u32,
        height: u32,
    },
    /// Window is closing. Listeners may still observe it, but no
    /// further per-window pulses will fire after this.
    WindowClosed { window: WindowId },
    /// Whole process is being suspended (macOS app-nap, Windows
    /// power state). Distinct from per-window minimize.
    Suspended,
    /// Process resumed from a prior [`Suspended`](Self::Suspended).
    Resumed,
}

/// Opaque OS-window identity. Wraps the Tauri window label (the
/// `&str` argument to `WindowBuilder::new`). Default Tauri apps
/// use `"main"`; multi-window apps assign their own labels.
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WindowId(pub String);

/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
impl WindowId {
    pub fn new(label: impl Into<String>) -> Self {
        Self(label.into())
    }

    /// The default label Tauri assigns when no `WindowBuilder` is
    /// used and the shell launches a single-window app.
    pub fn main() -> Self {
        Self("main".into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Subscriber to lifecycle pulses. Implemented by app code (and
/// the `jet-multi-target` desktop profile adapter once the Cue
/// demo lands in Slice 5) to react to OS-shell state changes the
/// pure-web profile can't observe.
///
/// `Send + Sync` so a listener can be shared across the Tauri
/// event-loop thread and any worker the substrate spins up.
pub trait LifecycleListener: Send + Sync {
    fn on_event(&self, event: &LifecycleEvent);
}

/// In-process event bus owning a list of listeners. The Tauri
/// event-loop hook (Slice 2b) calls [`LifecycleBus::publish`]
/// from inside `tauri::Builder::on_window_event`; until then,
/// tests drive the bus directly.
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
#[derive(Default, Clone)]
pub struct LifecycleBus {
    inner: Arc<Mutex<Vec<Arc<dyn LifecycleListener>>>>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
impl std::fmt::Debug for LifecycleBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `dyn LifecycleListener` isn't Debug, so we surface the
        // listener count instead — enough for diagnostic output
        // without forcing every listener to implement Debug.
        f.debug_struct("LifecycleBus")
            .field("listeners", &self.listener_count())
            .finish()
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
impl LifecycleBus {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a listener. Returned bus is the same `Arc`-shared
    /// instance — listeners attached on any clone are visible to
    /// every clone (this is `Arc<Mutex<_>>` shared state, not a
    /// per-handle copy).
    pub fn subscribe<L: LifecycleListener + 'static>(&self, listener: L) {
        self.inner.lock().unwrap().push(Arc::new(listener));
    }

    /// Number of currently-registered listeners. Cheap; used by
    /// tests + by the future Tauri wiring to skip dispatch when
    /// no one's listening.
    pub fn listener_count(&self) -> usize {
        self.inner.lock().unwrap().len()
    }

    /// Dispatch an event to every registered listener. Each
    /// listener sees the same `&LifecycleEvent` reference; they
    /// must clone if they want to retain it. Order matches
    /// subscription order.
    pub fn publish(&self, event: LifecycleEvent) {
        let listeners = self.inner.lock().unwrap().clone();
        for l in &listeners {
            l.on_event(&event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Listener fixture that records every event it receives.
    /// Used by tests to assert dispatch order + payload integrity
    /// without needing a real Tauri webview.
    struct RecordingListener {
        events: Mutex<Vec<LifecycleEvent>>,
    }

    impl RecordingListener {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                events: Mutex::new(Vec::new()),
            })
        }

        fn snapshot(&self) -> Vec<LifecycleEvent> {
            self.events.lock().unwrap().clone()
        }
    }

    impl LifecycleListener for Arc<RecordingListener> {
        fn on_event(&self, event: &LifecycleEvent) {
            self.events.lock().unwrap().push(event.clone());
        }
    }

    #[test]
    fn window_id_main_round_trip() {
        let w = WindowId::main();
        assert_eq!(w.as_str(), "main");
        assert_eq!(w, WindowId::new("main"));
    }

    #[test]
    fn lifecycle_event_is_clone_and_eq() {
        let a = LifecycleEvent::WindowCreated {
            window: WindowId::main(),
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn lifecycle_event_resize_carries_dimensions() {
        let e = LifecycleEvent::WindowResized {
            window: WindowId::main(),
            width: 1280,
            height: 800,
        };
        match e {
            LifecycleEvent::WindowResized { width, height, .. } => {
                assert_eq!((width, height), (1280, 800));
            }
            _ => panic!("expected WindowResized"),
        }
    }

    #[test]
    fn empty_bus_has_zero_listeners() {
        let bus = LifecycleBus::new();
        assert_eq!(bus.listener_count(), 0);
        // Publishing on an empty bus is a no-op (must not panic).
        bus.publish(LifecycleEvent::Suspended);
    }

    #[test]
    fn subscribe_increments_listener_count() {
        let bus = LifecycleBus::new();
        bus.subscribe(RecordingListener::new());
        bus.subscribe(RecordingListener::new());
        assert_eq!(bus.listener_count(), 2);
    }

    #[test]
    fn publish_fans_out_to_every_listener_in_order() {
        let bus = LifecycleBus::new();
        let a = RecordingListener::new();
        let b = RecordingListener::new();
        bus.subscribe(a.clone());
        bus.subscribe(b.clone());

        bus.publish(LifecycleEvent::WindowCreated {
            window: WindowId::main(),
        });
        bus.publish(LifecycleEvent::WindowFocused {
            window: WindowId::main(),
        });
        bus.publish(LifecycleEvent::Suspended);

        for listener in [&a, &b] {
            let events = listener.snapshot();
            assert_eq!(events.len(), 3);
            assert!(matches!(events[0], LifecycleEvent::WindowCreated { .. }));
            assert!(matches!(events[1], LifecycleEvent::WindowFocused { .. }));
            assert!(matches!(events[2], LifecycleEvent::Suspended));
        }
    }

    #[test]
    fn bus_clone_shares_listener_list() {
        let bus = LifecycleBus::new();
        let cloned = bus.clone();
        let listener = RecordingListener::new();
        cloned.subscribe(listener.clone());
        // Original sees the listener attached on the clone — Arc/Mutex
        // shared state, not per-handle vector.
        assert_eq!(bus.listener_count(), 1);
        bus.publish(LifecycleEvent::Resumed);
        assert_eq!(listener.snapshot(), vec![LifecycleEvent::Resumed]);
    }
}
// CODEGEN-END
