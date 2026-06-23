//! loom — DAG workflow scheduler (control plane).
//!
//! loom composes per-task lifecycles into a dynamic DAG. It is a *control
//! plane*: it owns sharded, strongly-consistent workflow/DAG state and
//! coordinates execution over two deliberately simple primitives —
//! **relay** (broker: publish / lease / ack) and **keep** (store: input and
//! result payloads) — without ever sitting in the data path.
//!
//! Boundaries (see `README.md`, issues #106 / #165 / #164):
//! - client → loom (submit / status / result-ref) + keep (payload, direct). Not relay.
//! - worker → relay (lease / ack / heartbeat) + keep (payload). Not loom.
//! - loom  → relay (publish task, observe acks) + keep (read result-ref). Never touches workers.
//!
//! Payload bytes never traverse loom: it issues keep refs / scoped URLs and the
//! parties transfer directly (claim-check).

pub mod model;
pub mod runner;
pub mod canvas;
pub mod fairness;
pub mod scheduler;
pub mod store;
pub mod raft;
pub mod relay_client;
pub mod keep_client;

pub mod controller;
pub mod worker;
pub mod runtask;
pub mod jobcontroller;
