//! Graph runtime publishes a typed event stream when an event bus is
//! attached (#2058 — graph half).

use std::sync::Arc;

use agent::NovaError;
use agentkit_graph::{Graph, GraphEvent, GraphEventBus};

#[tokio::test(flavor = "current_thread")]
async fn linear_walk_emits_typed_event_stream() {
    let bus: Arc<GraphEventBus> = Arc::new(GraphEventBus::new());
    let mut sub = bus.subscribe();

    let mut g: Graph<u32> = Graph::new();
    g.add_fn("inc", |s: u32| async move { Ok(s + 1) }).unwrap();
    g.add_fn("double", |s: u32| async move { Ok(s * 2) }).unwrap();
    g.add_edge("inc", "double").unwrap();
    g.set_entry("inc").unwrap();
    g.events(bus.clone());

    let out = g.run(3).await.unwrap();
    assert_eq!(out, 8);

    let events = sub.try_drain();
    let names: Vec<&'static str> = events
        .iter()
        .map(|e| match e {
            GraphEvent::WalkStarted { .. } => "walk_started",
            GraphEvent::NodeStarted { .. } => "node_started",
            GraphEvent::NodeCompleted { .. } => "node_completed",
            GraphEvent::NodeFailed { .. } => "node_failed",
            GraphEvent::WalkCompleted { .. } => "walk_completed",
            GraphEvent::WalkFailed { .. } => "walk_failed",
        })
        .collect();
    assert_eq!(
        names,
        vec![
            "walk_started",
            "node_started",
            "node_completed",
            "node_started",
            "node_completed",
            "walk_completed",
        ]
    );
}

#[tokio::test(flavor = "current_thread")]
async fn node_error_emits_node_failed_and_walk_failed() {
    let bus: Arc<GraphEventBus> = Arc::new(GraphEventBus::new());
    let mut sub = bus.subscribe();

    let mut g: Graph<u32> = Graph::new();
    g.add_fn("ok", |s: u32| async move { Ok(s + 1) }).unwrap();
    g.add_fn("boom", |_: u32| async move {
        Err(NovaError::LLMError("boom".into()))
    })
    .unwrap();
    g.add_edge("ok", "boom").unwrap();
    g.set_entry("ok").unwrap();
    g.events(bus.clone());

    let err = g.run(0).await.unwrap_err();
    assert!(matches!(err, NovaError::LLMError(_)));

    let events = sub.try_drain();
    assert!(events
        .iter()
        .any(|e| matches!(e, GraphEvent::NodeFailed { node, .. } if node == "boom")));
    assert!(matches!(events.last(), Some(GraphEvent::WalkFailed { .. })));
}
