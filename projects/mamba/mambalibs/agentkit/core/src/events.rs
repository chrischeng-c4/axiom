//! Structured event log (#2058).
//!
//! A typed [`AgentEvent`] enum is emitted at every state change of an
//! [`Agent::run`](crate::Agent::run) — start, every LLM round-trip, every
//! tool dispatch, validation retries, and the final outcome.
//!
//! Events are delivered through an [`EventBus`], a thin wrapper around
//! `tokio::sync::broadcast` so an arbitrary number of subscribers (UI,
//! file log, OTel exporter, eval harness) can attach. The bus is
//! lock-free on the hot path and lossy under back-pressure — late
//! subscribers may observe `Err(Lagged(n))`, which surfaces a typed
//! gap count rather than silently dropping events.
//!
//! Events are `Serialize + Deserialize` so they round-trip through the
//! cross-surface JSON-RPC envelope from #2029.

// HANDWRITE-BEGIN reason: no rust-runtime generator for a typed event
// bus + per-event-shape codegen yet. Once a `events` section type
// lands that can emit the enum + serde plumbing from a YAML schema,
// swap to CODEGEN markers.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::error::{NovaError, NovaResult};

/// Channel capacity for [`EventBus`]. Picked large enough to absorb a
/// few thousand events between subscriber polls; subscribers that fall
/// further behind will receive `Err(Lagged(n))` rather than blocking
/// the publisher.
pub const EVENT_BUS_CAPACITY: usize = 1024;

/// Typed event emitted from `Agent::run`. Each variant carries the
/// minimum data a downstream consumer needs to reconstruct what
/// happened without re-reading the conversation transcript.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum AgentEvent {
    RunStarted {
        model: String,
        timestamp_ms: u64,
    },
    LlmCallStarted {
        turn: u32,
        message_count: usize,
    },
    LlmCallCompleted {
        turn: u32,
        tool_calls: usize,
    },
    ToolCallStarted {
        tool: String,
        call_id: String,
    },
    ToolCallCompleted {
        tool: String,
        call_id: String,
    },
    ToolCallFailed {
        tool: String,
        call_id: String,
        error: String,
    },
    ValidationFailed {
        revision: u32,
        error: String,
    },
    RunCompleted {
        turns_used: u32,
        revisions: u32,
    },
    RunFailed {
        turns_used: u32,
        revisions: u32,
        error: String,
    },
}

impl AgentEvent {
    pub fn run_started(model: impl Into<String>) -> Self {
        Self::RunStarted {
            model: model.into(),
            timestamp_ms: now_ms(),
        }
    }
}

/// Multi-producer / multi-consumer event bus over a typed event `E`.
/// Wraps `tokio::sync::broadcast` so the publishing side is fire-and-
/// forget (no `.await`) and N subscribers each get their own queue.
#[derive(Debug)]
pub struct EventBus<E>
where
    E: Clone + Send + 'static,
{
    sender: broadcast::Sender<E>,
}

impl<E> EventBus<E>
where
    E: Clone + Send + 'static,
{
    /// Create a fresh bus with the default [`EVENT_BUS_CAPACITY`].
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(EVENT_BUS_CAPACITY);
        Self { sender }
    }

    /// Create a fresh bus with an explicit per-subscriber capacity.
    /// Capacities below 1 are coerced to 1; `tokio::broadcast` panics
    /// on a zero capacity, so we normalise rather than propagate.
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity.max(1));
        Self { sender }
    }

    /// Publish an event to every active subscriber. Returns the number
    /// of subscribers that received the event; returns `Ok(0)` if no
    /// one is listening (no error — losing events without a subscriber
    /// is intentional).
    pub fn emit(&self, event: E) -> NovaResult<usize> {
        match self.sender.send(event) {
            Ok(n) => Ok(n),
            Err(broadcast::error::SendError(_)) => Ok(0),
        }
    }

    /// Subscribe to all future events. Each call returns a fresh
    /// receiver — clone of `EventBus` is intentionally not exposed
    /// because all consumers should go through `subscribe`.
    pub fn subscribe(&self) -> Subscriber<E> {
        Subscriber {
            inner: self.sender.subscribe(),
        }
    }

    /// Current number of active subscribers (for diagnostics / tests).
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl<E> Default for EventBus<E>
where
    E: Clone + Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Receiver side of an [`EventBus`].
pub struct Subscriber<E>
where
    E: Clone + Send + 'static,
{
    inner: broadcast::Receiver<E>,
}

impl<E> Subscriber<E>
where
    E: Clone + Send + 'static,
{
    /// Await the next event. Returns `NovaError::ConfigError` if the
    /// bus has been dropped, or if the subscriber lagged behind by
    /// more than the bus capacity (the number of skipped events is
    /// embedded in the error message so callers can warn but keep
    /// reading).
    pub async fn recv(&mut self) -> NovaResult<E> {
        match self.inner.recv().await {
            Ok(event) => Ok(event),
            Err(broadcast::error::RecvError::Closed) => {
                Err(NovaError::ConfigError("event bus closed".into()))
            }
            Err(broadcast::error::RecvError::Lagged(n)) => Err(NovaError::ConfigError(format!(
                "event subscriber lagged — {n} events skipped"
            ))),
        }
    }

    /// Drain whatever is currently in the receiver buffer without
    /// awaiting. Useful for batch test assertions.
    pub fn try_drain(&mut self) -> Vec<E> {
        let mut out = Vec::new();
        loop {
            match self.inner.try_recv() {
                Ok(e) => out.push(e),
                Err(_) => return out,
            }
        }
    }
}

/// Convenience handle plumbed into `Agent` / `Graph` so they can emit
/// without taking ownership of the bus. Cloning is cheap.
pub type EventEmitter<E> = Arc<EventBus<E>>;

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn emit_delivers_to_all_subscribers() {
        let bus: EventBus<AgentEvent> = EventBus::new();
        let mut s1 = bus.subscribe();
        let mut s2 = bus.subscribe();

        assert_eq!(bus.subscriber_count(), 2);
        let delivered = bus
            .emit(AgentEvent::RunStarted {
                model: "m".into(),
                timestamp_ms: 0,
            })
            .unwrap();
        assert_eq!(delivered, 2);

        assert!(matches!(
            s1.recv().await.unwrap(),
            AgentEvent::RunStarted { .. }
        ));
        assert!(matches!(
            s2.recv().await.unwrap(),
            AgentEvent::RunStarted { .. }
        ));
    }

    #[tokio::test]
    async fn emit_without_subscribers_returns_zero() {
        let bus: EventBus<AgentEvent> = EventBus::new();
        let delivered = bus
            .emit(AgentEvent::RunCompleted {
                turns_used: 1,
                revisions: 0,
            })
            .unwrap();
        assert_eq!(delivered, 0);
    }

    #[tokio::test]
    async fn subscriber_lag_surfaces_typed_error() {
        let bus: EventBus<AgentEvent> = EventBus::with_capacity(2);
        let mut sub = bus.subscribe();
        for i in 0..5 {
            bus.emit(AgentEvent::LlmCallStarted {
                turn: i,
                message_count: 0,
            })
            .unwrap();
        }
        // We pushed 5 with capacity 2 — recv should report lagged.
        let err = sub.recv().await.unwrap_err();
        assert!(
            matches!(&err, NovaError::ConfigError(m) if m.contains("lagged")),
            "got: {err:?}"
        );
    }

    #[tokio::test]
    async fn closed_bus_surfaces_typed_error() {
        let bus: EventBus<AgentEvent> = EventBus::new();
        let mut sub = bus.subscribe();
        drop(bus);
        let err = sub.recv().await.unwrap_err();
        assert!(
            matches!(&err, NovaError::ConfigError(m) if m.contains("closed")),
            "got: {err:?}"
        );
    }

    #[test]
    fn agent_event_serde_round_trip() {
        let event = AgentEvent::ToolCallCompleted {
            tool: "echo".into(),
            call_id: "call_1".into(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"event\":\"tool_call_completed\""));
        let back: AgentEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back, event);
    }
}
