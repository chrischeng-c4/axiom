// HANDWRITE-BEGIN gap="missing-generator:unit-test:cbb497d5" tracker="#2483" reason="Drift gate for the #2483 stale-backup-destination-docs class: pins tape's README, deployment-handoff runbook, `tape backup --help`, and the new LLM_BACKUP_TOPICS sectioned topic (#2494) to service_backup::SUPPORTED_SCHEMES so any of the three doc surfaces #2483 found stale (or a fourth) fails CI the moment it drifts from what the linked service-backup build actually accepts."
//! Backup-destination doc/CLI-help/LLM-topic drift gate (#2483, #2494).
//!
//! `service_backup::SUPPORTED_SCHEMES` (`libs/service-backup/src/
//! destination.rs`) is the single fact source for which backup destination
//! URI schemes Tape accepts and whether each has a live sink linked into
//! this build. #2483 found three hand-copied `file://`+`s3://`-only claims
//! (CLI help, `docs/deployment-handoff.md`, `README.md`) that had drifted
//! stale against the unconditionally-shipped `gs://` scheme; these tests
//! pin every doc surface plus Tape's own sectioned LLM topic
//! (`tape::spec::LLM_BACKUP_TOPICS`, #2494) to that fact source so the same
//! drift class can't silently reopen.

use std::path::PathBuf;

fn tape_bin() -> &'static str {
    env!("CARGO_BIN_EXE_tape")
}

fn manifest_doc(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[test]
fn readme_backup_sections_list_every_supported_scheme() {
    let readme = manifest_doc("README.md");
    for info in service_backup::SUPPORTED_SCHEMES {
        assert!(
            readme.contains(info.scheme),
            "README.md is missing backup destination scheme `{}` from service_backup::SUPPORTED_SCHEMES — #2483 drift is back",
            info.scheme
        );
    }
}

#[test]
fn deployment_handoff_lists_every_supported_scheme() {
    let handoff = manifest_doc("docs/deployment-handoff.md");
    for info in service_backup::SUPPORTED_SCHEMES {
        assert!(
            handoff.contains(info.scheme),
            "docs/deployment-handoff.md is missing backup destination scheme `{}` from service_backup::SUPPORTED_SCHEMES — #2483 drift is back",
            info.scheme
        );
    }
}

#[test]
fn backup_help_lists_every_supported_scheme() {
    let output = std::process::Command::new(tape_bin())
        .args(["backup", "--help"])
        .output()
        .expect("run tape backup --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for info in service_backup::SUPPORTED_SCHEMES {
        assert!(
            stdout.contains(info.scheme),
            "`tape backup --help` is missing scheme `{}` from service_backup::SUPPORTED_SCHEMES — #2483 drift is back",
            info.scheme
        );
    }
}

#[test]
fn file_and_gs_sinks_are_unconditional_per_supported_schemes() {
    // Locks the exact fact #2483 was filed over: `gs://` always ships a
    // live sink; only `s3://` depends on a linked cargo feature. Docs that
    // call `gs://` "unconditional"/"always linked" are asserting this.
    let by_scheme = |scheme: &str| {
        service_backup::SUPPORTED_SCHEMES
            .iter()
            .find(|s| s.scheme == scheme)
            .unwrap_or_else(|| panic!("service_backup::SUPPORTED_SCHEMES has no `{scheme}` entry"))
    };
    assert!(by_scheme("file://").sink_available);
    assert!(by_scheme("gs://").sink_available);
}

#[test]
fn tape_llm_backup_sectioned_topic_conforms() {
    cli_std::llm::assert_topics_render(tape::spec::LLM_BACKUP_TOPICS);
}

#[test]
fn tape_llm_backup_topic_renders_every_supported_scheme_via_render_sectioned() {
    let rendered = cli_std::llm::render_sectioned(
        "tape",
        env!("CARGO_PKG_VERSION"),
        tape::spec::LLM_BACKUP_TOPICS,
        "backup",
        cli_std::llm::Format::Md,
    )
    .expect("tape::spec::LLM_BACKUP_TOPICS renders its `backup` topic");
    for info in service_backup::SUPPORTED_SCHEMES {
        assert!(
            rendered.contains(info.scheme),
            "tape::spec::LLM_BACKUP_TOPICS rendered output is missing scheme `{}`",
            info.scheme
        );
    }
}
// HANDWRITE-END
