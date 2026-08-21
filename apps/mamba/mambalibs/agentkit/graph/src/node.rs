//! Node abstraction — a named, async, state-transforming step.
//!
//! Nodes are the atomic unit of computation in a `Graph<State>`. Each node
//! takes ownership of the running state, applies its transformation, and
//! returns the new state. Errors are typed via `agent::NovaError`.

// HANDWRITE-BEGIN reason: same gap as the parent lib — no async graph-engine
// generator yet (see lib.rs).

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use agent::NovaResult;
use async_trait::async_trait;

/// Reference-counted, type-erased node handle stored inside a [`Graph`].
pub type NodeRef<State> = Arc<dyn Node<State>>;

/// Async, named state-transforming step.
#[async_trait]
pub trait Node<State>: Send + Sync
where
    State: Send + 'static,
{
    /// Stable identifier used in edge declarations and run traces.
    fn name(&self) -> &str;

    /// Transform the running state. Ownership is taken so the implementor
    /// can mutate freely without exposing locking concerns; whatever is
    /// returned becomes the new graph state.
    async fn call(&self, state: State) -> NovaResult<State>;
}

/// Closure-backed [`Node`] for the common case where the user just wants to
/// hand the graph a function. See [`FnNode::new`].
pub struct FnNode<State> {
    name: String,
    handler: Arc<
        dyn Fn(State) -> Pin<Box<dyn Future<Output = NovaResult<State>> + Send>>
            + Send
            + Sync
            + 'static,
    >,
}

impl<State> FnNode<State>
where
    State: Send + 'static,
{
    /// Build a node from an async closure `Fn(State) -> Future<NovaResult<State>>`.
    pub fn new<F, Fut>(name: impl Into<String>, handler: F) -> Self
    where
        F: Fn(State) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = NovaResult<State>> + Send + 'static,
    {
        let handler = Arc::new(move |state: State| {
            let fut = handler(state);
            Box::pin(fut) as Pin<Box<dyn Future<Output = NovaResult<State>> + Send>>
        });
        Self {
            name: name.into(),
            handler,
        }
    }
}

#[async_trait]
impl<State> Node<State> for FnNode<State>
where
    State: Send + 'static,
{
    fn name(&self) -> &str {
        &self.name
    }

    async fn call(&self, state: State) -> NovaResult<State> {
        (self.handler)(state).await
    }
}

// HANDWRITE-END
