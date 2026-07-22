// HANDWRITE-BEGIN gap="missing-generator:unit-test:5f79c0a3" tracker="#2157" reason="Add a deterministic structural regression test for the exact 19 capability refs, including primary role and full coverage. generator gap: missing-generator:test:capability-td-linkage (#2157)."
// @spec apps/tape/tech-design/semantic/backfill-primary-td-linkage-for-completed-capability-claims.md#unit-test

const TD: &str = include_str!(
    "../tech-design/semantic/backfill-primary-td-linkage-for-completed-capability-claims.md"
);

const EXPECTED_REFS: [(&str, &str); 19] = [
    ("long-running-stability", "repeated-raft-restart-endurance"),
    ("long-running-stability", "bounded-http-replay-soak"),
    ("security-hardening", "topic-replay-security-boundary"),
    (
        "security-hardening",
        "opt-in-server-ingress-network-policy",
    ),
    (
        "subscription-delivery-resources",
        "pull-subscription-cursor-contract",
    ),
    (
        "retention-and-backfill",
        "retention-window-and-backfill-contract",
    ),
    (
        "http2-api-list",
        "service-http-shell-h2c-serve-standard-endpoints",
    ),
    (
        "http2-api-list",
        "backup-service-tls-spec-gen-clients",
    ),
    (
        "standard-operational-endpoints",
        "service-http-shell-h2c-serve-standard-endpoints",
    ),
    (
        "observability",
        "prometheus-operator-scrape-alert-component",
    ),
    ("ec-gates-configured", "crate-smoke-gate"),
    (
        "ec-gates-configured",
        "tape-vat-meter-guard-ec-gates-observability",
    ),
    ("ec-gates-configured", "shared-otlp-trace-export"),
    (
        "kubernetes-native-deployment",
        "operator-kind-pvc-restart-replay",
    ),
    ("backup-restore", "exact-journal-snapshot-backup"),
    ("backup-restore", "fresh-pvc-cold-recovery-seed"),
    (
        "replica-sync-bootstrap",
        "raft-log-existing-pvc-sync",
    ),
    (
        "replica-sync-bootstrap",
        "empty-pvc-external-backup-seed",
    ),
    ("primary-replicas", "raft-backed-replay-journal"),
];

fn frontmatter() -> &'static str {
    TD.strip_prefix("---\n")
        .expect("TD frontmatter start")
        .split("\n---\n")
        .next()
        .expect("TD frontmatter end")
}

#[test]
fn exact_primary_full_linkage_inventory_is_preserved() {
    let frontmatter = frontmatter();

    for (capability, claim) in EXPECTED_REFS {
        let expected = format!(
            "  - id: \"{capability}\"\n    role: primary\n    gap: \"{claim}\"\n    claim: \"{claim}\"\n    coverage: full"
        );
        assert!(
            frontmatter.contains(&expected),
            "missing exact primary/full linkage for {capability}/{claim}"
        );
    }

    assert_eq!(frontmatter.matches("  - id: \"").count(), EXPECTED_REFS.len());
    assert_eq!(frontmatter.matches("    role: primary").count(), EXPECTED_REFS.len());
    assert_eq!(frontmatter.matches("    coverage: full").count(), EXPECTED_REFS.len());
}

#[test]
fn reconciliation_scope_is_metadata_only() {
    let changes = TD
        .split("## Changes")
        .nth(1)
        .expect("changes section")
        .split("## Unit Test")
        .next()
        .expect("changes section boundary");

    assert!(changes.contains("path: apps/tape/tests/capability_primary_linkage.rs"));
    assert_eq!(changes.matches("  - path:").count(), 1);
    assert!(!changes.contains("path: apps/tape/src/"));
    assert!(!changes.contains("path: apps/tape/k8s/"));
    assert!(!changes.contains("path: apps/tape/scripts/"));
}

// HANDWRITE-END
