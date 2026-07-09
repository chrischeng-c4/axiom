// HANDWRITE-BEGIN gap="missing-generator:logic:adf117ff" tracker="pending-tracker" reason="New module (feature backup): fetch_snapshot_bytes(base_url, token) GETs {base_url}/admin/backup via reqwest (Bearer when set, non-2xx bails with status+body); run_backup(base_url, token, dest, retention) hands the exact bytes to service_backup::run_backup_once against sink_from_destination -- relay's src/backup.rs pattern verbatim (transport + shipping only, no snapshot logic)."
// TODO: hand-write content for `apps/tape/src/backup.rs`.
// HANDWRITE-END
