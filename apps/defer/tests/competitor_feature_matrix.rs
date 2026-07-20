// HANDWRITE-BEGIN gap="missing-generator:unit-test:defer-competitor-feature-matrix" tracker="#2216" reason="Machine-check the deliberately scoped Google Cloud Tasks comparison and explicit non-goals without treating prose as runtime behavior proof."
use std::collections::BTreeMap;

const MATRIX: &str = include_str!("../benchmarks/competitor-feature-matrix.md");

fn rows() -> BTreeMap<&'static str, Vec<&'static str>> {
    let mut rows = BTreeMap::new();
    for line in MATRIX.lines().filter(|line| line.starts_with('|')).skip(2) {
        let cells: Vec<_> = line.trim_matches('|').split('|').map(str::trim).collect();
        assert_eq!(cells.len(), 4, "matrix rows have exactly four fields");
        let prior = rows.insert(cells[0], cells);
        assert!(prior.is_none(), "duplicate matrix row `{line}`");
    }
    rows
}

// <HANDWRITE gap="missing-generator:unit-test" tracker="pending-tracker" reason="Own the parsed, duplicate-free 15-row Google Cloud Tasks-shaped comparison, explicit category exclusions, accurate retry-exhaustion semantics, and bounded performance-claim boundary.">
#[test]
fn cloud_tasks_shaped_contract_and_exclusions_are_explicit() {
    let normalized = MATRIX.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(normalized.contains(
        "Google Cloud Tasks is the semantic competitor; language worker frameworks such as Celery and Sidekiq are explicitly outside the comparison because they do not own the same HTTP push service contract."
    ));
    assert_eq!(MATRIX.matches("Celery").count(), 1);
    assert_eq!(MATRIX.matches("Sidekiq").count(), 1);

    let contract_yaml = MATRIX
        .split_once("```yaml\n")
        .and_then(|(_, rest)| rest.split_once("\n```").map(|(yaml, _)| yaml))
        .expect("machine-readable comparison contract exists");
    let contract: serde_yaml::Value = serde_yaml::from_str(contract_yaml).unwrap();
    let contract = &contract["comparison_contract"];
    assert_eq!(contract["semantic_competitor"], "google-cloud-tasks");
    assert_eq!(contract["category"], "managed-http-push-queue");
    assert_eq!(
        contract["worker_frameworks"]["celery"],
        "excluded-category-mismatch"
    );
    assert_eq!(
        contract["worker_frameworks"]["sidekiq"],
        "excluded-category-mismatch"
    );
    assert_eq!(contract["cloud_tasks_performance"], "unproven");
    assert_eq!(contract["vat_role"], "protocol-emulator-only");
    assert_eq!(
        contract["relay_role"],
        "local-implementation-overhead-ceiling-only"
    );

    let rows = rows();
    assert_eq!(rows.len(), 15, "matrix capability inventory is exact");

    for (capability, cloud_tasks, defer, decision) in [
        ("Future schedule / ETA", "yes", "yes", "core"),
        ("HTTP target dispatch", "native", "native", "core"),
        ("Success deletes/completes task", "2xx", "2xx", "core"),
        (
            "At-least-once retry",
            "yes",
            "yes",
            "core; stable idempotency key is sent on every attempt",
        ),
        (
            "Queue rate / burst / max in-flight",
            "yes",
            "yes",
            "committed globally across replicas",
        ),
        ("Pause / resume / disable", "yes", "yes", "core operations"),
        ("Per-task target headers and method", "yes", "yes", "core"),
        (
            "Target authentication",
            "OAuth/OIDC",
            "HMAC signing",
            "intentionally cloud-neutral",
        ),
        (
            "Durable HA scheduler state",
            "managed",
            "Raft + per-replica durable state",
            "core",
        ),
        (
            "DLQ terminal state",
            "no; exhausted tasks are deleted",
            "explicit replicated terminal state",
            "intentional Defer extension; Cloud Tasks is not a DLQ oracle",
        ),
        ("Task cancellation / inspection", "yes", "yes", "core"),
    ] {
        assert_eq!(
            rows[capability],
            [capability, cloud_tasks, defer, decision],
            "matrix drifted for {capability}"
        );
    }

    assert_eq!(
        rows["Force-run bypass"],
        [
            "Force-run bypass",
            "yes",
            "no",
            "excluded: bypassing committed rate/permit policy weakens the service contract",
        ]
    );
    assert_eq!(
        rows["Arbitrary language worker execution"],
        [
            "Arbitrary language worker execution",
            "no",
            "no",
            "excluded: Relay owns pull workers; Defer owns HTTP push",
        ]
    );
    assert_eq!(
        rows["Periodic/cron schedules"],
        [
            "Periodic/cron schedules",
            "separate Scheduler product",
            "no",
            "excluded: scheduler composition belongs outside Defer",
        ]
    );
    assert_eq!(
        rows["Workflow/batches"],
        [
            "Workflow/batches",
            "no",
            "no",
            "excluded: Loom owns orchestration",
        ]
    );

    assert!(normalized.contains(
        "Until a real Cloud Tasks queue and publicly reachable target are available under the same declared region/hardware/network conditions, that claim remains unproven."
    ));
    assert!(normalized.contains(
        "The local vat emulator may verify protocol behavior, but must not be reported as Google Cloud Tasks performance."
    ));
    assert!(normalized.contains(
        "Defer's local implementation-efficiency ceiling is its sibling Relay, not a substitute competitor."
    ));
    assert!(normalized.contains("Defer may trail Relay by at most 20%"));
    assert!(normalized.contains(
        "Passing this gate proves bounded scheduler overhead only; it does not make a Cloud Tasks performance claim."
    ));
    assert!(
        normalized.contains("Google Cloud Tasks RetryConfig deletes tasks after retry exhaustion:")
    );
    assert!(normalized
        .contains("Google Cloud Tasks versus Pub/Sub assigns dead-letter topics to Pub/Sub:"));
}
// </HANDWRITE>
// HANDWRITE-END
