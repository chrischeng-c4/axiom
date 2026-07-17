// SPEC-MANAGED: apps/pgpool/tech-design/logic/p0-dense-buffer-readiness-reactor.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-readiness-reactor" tracker="#1753" reason="The transaction reactor owns a socket-readiness state machine not yet expressible by the service generator.">
//! Transaction-pooling readiness reactor.
//!
//! The runtime will be deliberately thin: all ownership transitions live in
//! [`state`], which makes the reset and pipeline boundary testable without a
//! scheduler, a socket, or an incidental Tokio wakeup.

mod runtime;
pub(crate) mod state;

pub(crate) use runtime::TransactionReactor;
// </HANDWRITE>
