// HANDWRITE-BEGIN gap="missing-generator:logic:f9ee58f0" tracker="pending-tracker" reason="New module, cfg(feature = backup): fetch_snapshot_bytes(base_url, token) GETs {base_url}/admin/backup via reqwest (Bearer when set, non-2xx bails with status + body); run_backup(base_url, token, dest, retention) hands the exact bytes to service_backup::run_backup_once against sink_from_destination — lumen src/backup.rs pattern minus the restore POST (relay restore is load_live merge, library-side)."
// TODO: hand-write content for `projects/relay/src/backup.rs`.
// HANDWRITE-END
