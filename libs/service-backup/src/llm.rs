// CODEGEN-BEGIN
//! LLM topic provider for the shared service-backup contract.
//!
//! [`TOPIC`]/[`topic`] are the original static form and are unchanged —
//! every existing CLI consumer keeps compiling and behaving exactly as
//! before. [`SECTIONED_TOPICS`]/[`sectioned_topic`] are the parallel
//! `cli_std::llm::SectionedTopic` form (#2494): the destination-contract
//! block is a `TopicSection::Generated` section rendered from
//! `crate::SUPPORTED_SCHEMES` at call time instead of frozen into a
//! `&'static str`, so it can't drift from what
//! `BackupDestination::from_uri`/`sink_from_destination` actually accept in
//! this build. CLI composition can adopt either form; both describe the
//! same contract.

use crate::destination::SUPPORTED_SCHEMES;

/// Agent-facing topic describing backup destinations, sinks, and seed fetches.
pub const TOPIC: cli_std::llm::Topic = cli_std::llm::Topic {
    id: "service-backup",
    summary: "Shared backup destination, policy, sink, runner, and bootstrap-object contract.",
    body: r#"# service-backup shared topic

## Ownership boundary
The service owns snapshot consistency, snapshot bytes, restore semantics, and
the admin or CLI endpoint that produces the snapshot. `service-backup` owns the
transport contract around those bytes: `BackupDestination`, `RetentionPolicy`,
`BackupSink`, `LocalFsSink`, optional S3 support, `run_backup_once`,
`fetch_backup_object`, and (with `http-client`) the authenticated standard
`GET /admin/backup` fetch/upload path.

Operator code or a CronJob should schedule and transport backups; it should not
serialize service state itself.

## Destination contract
Use exact destination URIs:

```text
file:///mnt/backups/service
s3://bucket/prefix
gs://bucket/prefix
```

`file://` and `gs://` are always available. `s3://` requires this crate's `s3`
feature. GCS uses workload identity in production and
`STORAGE_EMULATOR_HOST` for Vat-backed local integration.

## Restore and bootstrap
`fetch_backup_object` reads an exact `file://`, `s3://`, or `gs://` object URI for
restore or empty-PVC bootstrap. It is a cold seed path, not live replica
synchronization.
"#,
};

/// Return the shared backup topic for CLI composition.
pub fn topic() -> &'static cli_std::llm::Topic {
    &TOPIC
}

/// Static prose companion to the `TopicSection::Generated` destination
/// section in [`SECTIONED_TOPICS`] — everything from [`TOPIC`]'s body
/// except the hand-copied scheme list, which the generated section below
/// derives from [`crate::SUPPORTED_SCHEMES`] instead.
const OWNERSHIP_BOUNDARY: &str = r#"# service-backup shared topic

## Ownership boundary
The service owns snapshot consistency, snapshot bytes, restore semantics, and
the admin or CLI endpoint that produces the snapshot. `service-backup` owns the
transport contract around those bytes: `BackupDestination`, `RetentionPolicy`,
`BackupSink`, `LocalFsSink`, optional S3 support, `run_backup_once`,
`fetch_backup_object`, and (with `http-client`) the authenticated standard
`GET /admin/backup` fetch/upload path.

Operator code or a CronJob should schedule and transport backups; it should not
serialize service state itself."#;

const RESTORE_AND_BOOTSTRAP: &str = r#"## Restore and bootstrap
`fetch_backup_object` reads an exact object URI (any scheme from the
destination contract above) for restore or empty-PVC bootstrap. It is a cold
seed path, not live replica synchronization."#;

/// Render the `## Destination contract` section from
/// [`crate::SUPPORTED_SCHEMES`] — the same table `BackupDestination::from_uri`
/// and `sink_from_destination` use — instead of a hand-copied scheme list
/// (#2494). `sink_available` reports this build's actual linked feature set
/// via `cfg!`, so a rebuild with a different feature set changes this
/// section's output without any hand edit.
fn destination_contract_section() -> String {
    let mut s = String::from(
        "## Destination contract\n\nSupported destination URI schemes in this build:\n\n",
    );
    for info in SUPPORTED_SCHEMES {
        let availability = if info.sink_available {
            "sink linked into this build"
        } else {
            "parses, but no sink linked — uploads fail loud until rebuilt with the adapter feature"
        };
        s.push_str(&format!(
            "- `{}` — {} ({availability})\n",
            info.scheme, info.description
        ));
    }
    s.push_str(
        "\nExample: `s3://bucket/prefix`, `gs://bucket/prefix`, `file:///mnt/backups/service`.\n",
    );
    s
}

/// [`cli_std::llm::SectionedTopic`] form of [`TOPIC`] (#2494). One topic per
/// slice element, matching the `&[SectionedTopic]` shape
/// `cli_std::llm::render_sectioned`/`assert_topics_render` expect.
pub const SECTIONED_TOPICS: &[cli_std::llm::SectionedTopic] = &[cli_std::llm::SectionedTopic {
    id: "service-backup",
    summary: "Shared backup destination, policy, sink, runner, and bootstrap-object contract.",
    sections: &[
        cli_std::llm::TopicSection::Prose(OWNERSHIP_BOUNDARY),
        cli_std::llm::TopicSection::Generated {
            id: "destination-contract",
            render: destination_contract_section,
        },
        cli_std::llm::TopicSection::Prose(RESTORE_AND_BOOTSTRAP),
    ],
}];

/// Return the shared backup topic in [`cli_std::llm::SectionedTopic`] form.
pub fn sectioned_topic() -> &'static cli_std::llm::SectionedTopic {
    &SECTIONED_TOPICS[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn llm_topic_is_nonempty() {
        let topic = super::topic();
        assert_eq!(topic.id, "service-backup");
        assert!(topic.body.contains("BackupDestination"));
        assert!(topic.body.contains("fetch_backup_object"));
        assert!(topic.body.contains("http-client"));
    }

    #[test]
    fn sectioned_topic_conforms() {
        cli_std::llm::assert_topics_render(SECTIONED_TOPICS);
    }

    #[test]
    fn sectioned_topic_destination_section_lists_every_supported_scheme() {
        let generated = destination_contract_section();
        for info in SUPPORTED_SCHEMES {
            assert!(
                generated.contains(info.scheme),
                "destination-contract section missing scheme {}",
                info.scheme
            );
        }
    }

    #[test]
    fn sectioned_topic_matches_static_topic_identity() {
        assert_eq!(sectioned_topic().id, TOPIC.id);
        assert_eq!(sectioned_topic().summary, TOPIC.summary);
    }
}
// CODEGEN-END
