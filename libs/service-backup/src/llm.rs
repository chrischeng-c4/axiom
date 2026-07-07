//! LLM topic provider for the shared service-backup contract.

/// Agent-facing topic describing backup destinations, sinks, and seed fetches.
pub const TOPIC: cli_std::llm::Topic = cli_std::llm::Topic {
    id: "service-backup",
    summary: "Shared backup destination, policy, sink, runner, and bootstrap-object contract.",
    body: r#"# service-backup shared topic

## Ownership boundary
The service owns snapshot consistency, snapshot bytes, restore semantics, and
the admin or CLI endpoint that produces the snapshot. `service-backup` owns the
transport contract around those bytes: `BackupDestination`, `RetentionPolicy`,
`BackupSink`, `LocalFsSink`, optional S3 support, `run_backup_once`, and
`fetch_backup_object`.

Operator code or a CronJob should schedule and transport backups; it should not
serialize service state itself.

## Destination contract
Use exact destination URIs:

```text
file:///mnt/backups/service
s3://bucket/prefix
gs://bucket/prefix
```

`file://` is always available. `s3://` requires this crate's `s3` feature.
`gs://` parses and round-trips as schema-compatible input, but fails loudly
until a real GCS sink is implemented.

## Restore and bootstrap
`fetch_backup_object` reads an exact `file://` or `s3://` object URI for
restore or empty-PVC bootstrap. It is a cold seed path, not live replica
synchronization.
"#,
};

/// Return the shared backup topic for CLI composition.
pub fn topic() -> &'static cli_std::llm::Topic {
    &TOPIC
}

#[cfg(test)]
mod tests {
    #[test]
    fn llm_topic_is_nonempty() {
        let topic = super::topic();
        assert_eq!(topic.id, "service-backup");
        assert!(topic.body.contains("BackupDestination"));
        assert!(topic.body.contains("fetch_backup_object"));
    }
}
