// HANDWRITE-BEGIN gap="missing-generator:logic:f9ee58f0" tracker="pending-tracker" reason="Relay preserves public backup helper names while shared service-backup owns authenticated admin-snapshot transport and sink upload; Relay retains only its snapshot/restore semantics."
//! `relay backup` (WI #1209): fetch a consistent snapshot from a running
//! node's `GET /admin/backup` endpoint and hand the exact bytes to a
//! `libs/service-backup` destination sink. This module owns NO snapshot
//! logic — the endpoint serves the same `raft::snapshot_bytes` serialization
//! (`EngineSnapshot` = `dump_live` + applied index) the raft snapshotter
//! uses; this is transport + shipping only, meant to be driven by the
//! operator's optional backup CronJob (`spec.backup`, see
//! `service_k8s::render::backup_cron_job`) or invoked ad hoc via the CLI
//! (lumen #808 pattern).
//!
//! Restore is a library-side `load_live` MERGE: feed the artifact to
//! `crate::raft::load_snapshot_bytes` on a fresh node — idempotent per
//! `message_id`, leases/acks are node-local and not in the snapshot, so
//! restored work redelivers (at-least-once).

pub use service_backup::{
    fetch_admin_snapshot as fetch_snapshot_bytes, run_admin_snapshot_backup as run_backup,
};
// HANDWRITE-END
