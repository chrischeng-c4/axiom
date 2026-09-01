//! Shared server substrate for the Axiom runtime.
//! @spec apps/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#logic
//!
//! This crate is intentionally below protocol crates. It owns the generic
//! lifecycle pieces that a raw TCP proxy, an HTTP dev server, and the
//! k8s-native service archetype all need: bind config, drain/readiness signals,
//! connection budgets, shutdown hooks, and metrics hooks. Protocol-specific
//! accept loops live above it in `server-tcp` and `server-http`.

pub mod config;
pub mod deadline;
pub mod drain;
pub mod hooks;
pub mod lifecycle;
pub mod limits;
pub mod metrics;
pub mod readiness;
pub mod signal;
pub mod task_supervisor;

pub use config::BindConfig;
pub use deadline::{DeadlineError, ShutdownDeadline};
pub use drain::{DrainController, DrainSignal, DrainState};
pub use hooks::{HookOutcome, HookStage, HookStatus, PhaseTiming, ShutdownContext, ShutdownReport};
pub use lifecycle::{
    LifecycleController, LifecycleDeadlineError, LifecycleError, LifecycleEventError,
    LifecycleEventSubscription, LifecycleObservation, LifecyclePhase, LifecycleSubscription,
    LifecycleSubscriptionError,
};
pub use limits::{ConnectionBudget, ConnectionLimitExceeded, ConnectionPermit};
pub use metrics::{ConnectionMetrics, NoopConnectionMetrics};
pub use readiness::Readiness;
pub use signal::{shutdown_with_drain, wait_shutdown_signal};
pub use task_supervisor::TaskSupervisor;
