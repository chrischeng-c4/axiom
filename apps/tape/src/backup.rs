// HANDWRITE-BEGIN gap="missing-generator:logic:adf117ff" tracker="pending-tracker" reason="Tape's backup module only preserves the public names for the shared authenticated admin-snapshot fetch/upload contract; journal snapshot bytes and restore semantics remain Tape-owned."
//! `tape backup` (WI #1329): fetch a consistent snapshot from a running
//! node's `GET /admin/backup` endpoint and hand the exact bytes to a
//! `libs/service-backup` destination sink. This module owns NO transport or snapshot
//! logic — the endpoint serves the same `raft::snapshot_bytes` serialization
//! (the whole [`crate::TapeJournal`] + applied index) the raft state
//! machine's own `snapshot`/`restore` round-trip; this is transport +
//! shipping only (relay #1209's `src/backup.rs` pattern verbatim).
//!
//! Restore is the existing raft-side `TapeStateMachine::restore` merge path
//! (loaded offline/out of band); no restore CLI verb is added here, matching
//! relay's scope.

pub use service_backup::{
    fetch_admin_snapshot as fetch_snapshot_bytes, run_admin_snapshot_backup as run_backup,
};

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use service_backup::BackupDestination;

    use crate::server::{router, AppState};
    use crate::TapeJournal;

    async fn start_server() -> (SocketAddr, AppState) {
        let mut journal = TapeJournal::default();
        journal.append("orders", None, serde_json::json!({"n": 1}), Some(100));
        let state = AppState::new(journal, None);
        let app = router(state.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(service_http::serve(
            listener,
            app,
            std::future::pending::<()>(),
        ));
        (addr, state)
    }

    /// R2: `run_backup` fetches the live `/admin/backup` bytes unmodified and
    /// hands them to the destination sink — the artifact on disk is exactly
    /// `raft::snapshot_bytes` for the same journal.
    #[tokio::test]
    async fn run_backup_ships_fetched_bytes_to_sink() {
        let (addr, state) = start_server().await;
        let dir = tempfile::tempdir().unwrap();
        let dest =
            BackupDestination::from_uri(&format!("file://{}", dir.path().display())).unwrap();

        let result = super::run_backup(
            &format!("http://{addr}"),
            None,
            &dest,
            &service_backup::RetentionPolicy::default(),
        )
        .await
        .unwrap();

        assert!(result.object.bytes > 0);
        let artifact = std::fs::read(dir.path().join(&result.object.key)).unwrap();
        let expected = crate::raft::snapshot_bytes(&state.journal_handle(), 0).unwrap();
        assert_eq!(&artifact[..], &expected[..]);
    }
}
// HANDWRITE-END
