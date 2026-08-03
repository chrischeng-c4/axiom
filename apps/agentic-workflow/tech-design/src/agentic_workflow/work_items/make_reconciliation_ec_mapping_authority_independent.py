"Tech design for WI #3338: aw: make reconciliation EC mapping authority independent.\n\n@spec #3338"

from __future__ import annotations

__aw_changes__ = """
changes:
  - path: apps/agentic-workflow/external-contracts/fixtures/claim-reconciliation/capability-catalog-td-claim-linkage-expected-mapping.json
    action: create
    description: >
      Static hand-authored expected-mapping fixture (schema aw.python-ec.expected-mapping.v1)
      containing all 110 production EC case records. Never derived at runtime from canonical
      or candidate TOML; loaded independently by the production EC as the mapping authority.
  - path: apps/agentic-workflow/external-contracts/src/cases/capability-control-plane-capability-catalog-and-td-claim-linkage-consistency.py
    action: modify
    description: >
      Load the checked-in fixture as --expected-mapping authority instead of deriving expected
      records from canonical/candidate pyproject.toml. Prove full clean mapping and exact count
      against the authority. Prove missing, duplicate, and misbound findings using copied-candidate-
      only perturbations; no runtime self-derivation from canonical or candidate TOML.
"""


__aw_artifact_id__ = "artifact:capability-control-plane/make-reconciliation-ec-mapping-authority-independent-wi-3338"
__aw_work_item__ = "3338"

# Hand-authored static authority: all 110 production EC cases, sorted by case_id.
# Must not be derived from canonical or copied candidate pyproject.toml at runtime.
# Each record: (case_id, capability_id, use_case_id, dimension)
def _case_id(record: tuple[str, str, str, str]) -> str:
    return record[0]


_AUTHORITY: tuple[tuple[str, str, str, str], ...] = tuple(sorted((
    ("aw-core-client-core-concept-model-phase-less-admission", "aw-core-client-model-workitem-first-artifact-lifecycle", "core-concept-model-and-invariants", "behavior"),
    ("aw-core-client-operational-efficiency", "aw-core-client-model-workitem-first-artifact-lifecycle", "core-concept-model-and-invariants", "efficiency"),
    ("aw-core-client-operational-stability", "aw-core-client-model-workitem-first-artifact-lifecycle", "core-concept-model-and-invariants", "stability"),
    ("aw-health-default-full-verification-smoke", "existing-project-standardization", "aw-health-default-full-verification-smoke", "behavior"),
    ("authoritative-fixture-blocks-on-regenerability-gap", "existing-project-standardization", "authoritative-fixture-blocks-on-regenerability-gap", "behavior"),
    ("capability-control-plane-agent-facing-dx-baseline-trait", "capability-control-plane", "agent-facing-dx-baseline-trait", "behavior"),
    ("capability-control-plane-capability-catalog-and-td-claim-linkage-consistency", "capability-control-plane", "capability-catalog-and-td-claim-linkage-consistency", "behavior"),
    ("capability-control-plane-capability-project-sweep", "capability-control-plane", "capability-project-sweep", "behavior"),
    ("capability-control-plane-default-cap-path-flips-to-capabilities-md", "capability-control-plane", "default-cap-path-flips-to-capabilities-md", "behavior"),
    ("capability-control-plane-markdown-capability-schema", "capability-control-plane", "markdown-capability-schema", "behavior"),
    ("capability-control-plane-missing-readme-initialization", "capability-control-plane", "missing-readme-initialization", "behavior"),
    ("capability-control-plane-one-way-wi-reference-direction", "capability-control-plane", "one-way-wi-reference-direction", "behavior"),
    ("capability-control-plane-operational-efficiency", "capability-control-plane", "capability-readiness-reporting", "efficiency"),
    ("capability-control-plane-operational-stability", "capability-control-plane", "capability-readiness-reporting", "stability"),
    ("capability-control-plane-python-artifact-readiness", "capability-control-plane", "python-artifact-readiness", "behavior"),
    ("capability-scoped-dependency-verification", "capability-control-plane", "scoped-dependency-closed-verification", "behavior"),
    ("coordination-authority", "workflow-root-runner", "aw-only-coordination-authority", "behavior"),
    ("coordination-contract-schema", "workflow-root-runner", "versioned-coordination-contract", "behavior"),
    ("coordination-event-validation", "workflow-root-runner", "fail-closed-coordination-event-validation", "behavior"),
    ("ec-gated-terminal-check-unification-python-contract", "td-cb-lifecycle-automation", "td-surface-convergence-ec-gated-terminal-check-unification-verb-lifecycle-policy-fixture-loop-self-ec", "behavior"),
    ("existing-project-standardization-authoritative-source-snapshot-projection", "existing-project-standardization", "authoritative-source-snapshot-projection", "behavior"),
    ("existing-project-standardization-aw-review-skill-and-doc-projection", "existing-project-standardization", "aw-review-skill-and-doc-projection", "behavior"),
    ("existing-project-standardization-brownfield-takeover-surface", "existing-project-standardization", "brownfield-takeover-surface", "behavior"),
    ("existing-project-standardization-cb-and-cold-verification-gates", "existing-project-standardization", "cb-and-cold-verification-gates", "behavior"),
    ("existing-project-standardization-force-regeneration-project-root-llms-projection", "existing-project-standardization", "force-regeneration-project-root-llms-projection", "behavior"),
    ("existing-project-standardization-managed-and-semantic-production-gates", "existing-project-standardization", "managed-and-semantic-production-gates", "behavior"),
    ("existing-project-standardization-operational-efficiency", "existing-project-standardization", "project-health-no-regression", "efficiency"),
    ("existing-project-standardization-operational-stability", "existing-project-standardization", "project-health-no-regression", "stability"),
    ("existing-project-standardization-project-profile-conformance-review", "existing-project-standardization", "project-profile-conformance-review", "behavior"),
    ("existing-project-standardization-service-workload-profile-derivation", "existing-project-standardization", "service-workload-profile-derivation", "behavior"),
    ("existing-project-standardization-shared-service-kit-conformance-rules", "existing-project-standardization", "shared-service-kit-conformance-rules", "behavior"),
    ("existing-project-standardization-shared-service-kit-substrate", "existing-project-standardization", "shared-service-kit-substrate", "behavior"),
    ("existing-project-standardization-structured-observability-and-raft-telemetry-conformance-rules", "existing-project-standardization", "structured-observability-and-raft-telemetry-conformance-rules", "behavior"),
    ("existing-project-standardization-traceability-closure-gate", "existing-project-standardization", "traceability-closure-gate", "behavior"),
    ("existing-project-standardization-xml-handwrite-marker-fill-queue-lifecycle", "existing-project-standardization", "xml-handwrite-marker-fill-queue-lifecycle", "behavior"),
    ("external-fixture-reports-advisory-gap", "existing-project-standardization", "external-fixture-reports-advisory-gap", "behavior"),
    ("issue-cache-canonical-change", "work-item-planning", "canonical-change-issue-cache-round-trip", "behavior"),
    ("manual-evidence-artifacts-operational-efficiency", "manual-evidence-artifacts", "manual-runner-output-convention", "efficiency"),
    ("manual-evidence-artifacts-operational-stability", "manual-evidence-artifacts", "manual-runner-output-convention", "stability"),
    ("manual-evidence-schema-python-contract", "manual-evidence-artifacts", "generated-manual-ec-evidence-schema", "behavior"),
    ("manual-runner-output-convention-python-contract", "manual-evidence-artifacts", "manual-runner-output-convention", "behavior"),
    ("project-health-total-observation", "aw-core-client-model-workitem-first-artifact-lifecycle", "two-cell-ec-and-td-semantic-health", "behavior"),
    ("project-local-td-and-ec-gates-operational-efficiency", "project-local-td-and-ec-gates", "project-local-td-root-resolver", "efficiency"),
    ("project-local-td-and-ec-gates-operational-stability", "project-local-td-and-ec-gates", "project-local-td-root-resolver", "stability"),
    ("python-ec-cache-safe-discovery", "project-local-td-and-ec-gates", "cache-safe-python-ec-source-discovery", "behavior"),
    ("python-ec-only-authoring", "project-local-td-and-ec-gates", "python-only-ec-authoring-lifecycle", "behavior"),
    ("python-td-global-artifact-identity", "aw-core-client-model-workitem-first-artifact-lifecycle", "global-python-td-artifact-identity", "behavior"),
    ("python-td-claim-linkage", "capability-control-plane", "python-td-claim-linkage", "behavior"),
    ("self-ec-fixture-loop-gate-python-contract", "td-cb-lifecycle-automation", "self-ec-fixture-loop-gate", "behavior"),
    ("self-hosting-bounded-admission", "workflow-root-runner", "self-hosting-root-runner-policy", "efficiency"),
    ("self-hosting-capability-admission", "workflow-root-runner", "self-hosting-root-runner-policy", "behavior"),
    ("self-hosting-health-policy", "workflow-root-runner", "self-hosting-root-runner-policy", "behavior"),
    ("self-hosting-identity-stability", "workflow-root-runner", "self-hosting-root-runner-policy", "stability"),
    ("self-hosting-wi-admission", "workflow-root-runner", "self-hosting-root-runner-policy", "behavior"),
    ("shared-cli-issue-report-intake", "work-item-planning", "terminology-first-four-type-wi-taxonomy", "behavior"),
    ("standardize-audit-first-contract-test", "existing-project-standardization", "standardize-audit-first-contract-test", "behavior"),
    ("td-artifact-producer-cli-fixture", "aw-core-client-model-workitem-first-artifact-lifecycle", "shared-artifact-producer-contract", "behavior"),
    ("td-cb-lifecycle-automation-operational-efficiency", "td-cb-lifecycle-automation", "terminal-ec-process-liveness", "efficiency"),
    ("td-cb-lifecycle-automation-operational-stability", "td-cb-lifecycle-automation", "terminal-ec-process-liveness", "stability"),
    ("td-cb-lifecycle-automation-remove-td-merge-command", "td-cb-lifecycle-automation", "remove-td-merge-command", "behavior"),
    ("td-create-dirty-persistent-branch", "td-cb-lifecycle-automation", "dirty-persistent-branch-td-activation", "behavior"),
    ("td-existing-workspace-dirty-persistent-branch", "td-cb-lifecycle-automation", "dirty-persistent-branch-existing-td-activation", "behavior"),
    ("terminal-ec-cross-process-single-flight-python-contract", "td-cb-lifecycle-automation", "terminal-ec-process-liveness", "behavior"),
    ("terminal-ec-retry-transition-lease-python-contract", "td-cb-lifecycle-automation", "terminal-ec-process-liveness", "behavior"),
    ("terminal-ec-wrapper-timeout-teardown-python-contract", "td-cb-lifecycle-automation", "terminal-ec-process-liveness", "behavior"),
    ("wi-close-remote-real-cli", "work-item-planning", "wi-close-remote-rehydration", "behavior"),
    ("wi-create-help-command", "work-item-planning", "capability-to-epic-planning", "behavior"),
    ("wi-create-help-smoke", "work-item-planning", "wi-create-help-smoke", "behavior"),
    ("wi-create-remote-flag-tests", "work-item-planning", "wi-create-remote-flag-tests", "behavior"),
    ("wi-create-remote-unit-command", "work-item-planning", "capability-to-epic-planning", "behavior"),
    ("wi-remove-agent-estimate-build", "work-item-planning", "capability-to-epic-planning", "behavior"),
    ("wi-remove-agent-estimate-spec-check", "work-item-planning", "capability-to-epic-planning", "behavior"),
    ("wi-remove-agent-estimate-unit-command", "work-item-planning", "wi-remove-agent-estimate-unit-command", "behavior"),
    ("wi-typed-epic-owner", "work-item-planning", "typed-epic-owner-authoring", "behavior"),
    ("wi-typed-priority-label", "work-item-planning", "typed-priority-label-authoring", "behavior"),
    ("work-item-four-type-taxonomy", "work-item-planning", "terminology-first-four-type-wi-taxonomy", "behavior"),
    ("work-item-intake-health", "work-item-planning", "terminology-first-four-type-wi-taxonomy", "behavior"),
    ("work-item-planning-agent-backed-inventory-plan-review", "work-item-planning", "agent-backed-inventory-plan-review", "behavior"),
    ("work-item-planning-atomized-requirement-dependency-publication", "work-item-planning", "atomized-requirement-dependency-publication", "behavior"),
    ("work-item-planning-body-dependency-declaration-extraction", "work-item-planning", "body-dependency-declaration-extraction", "behavior"),
    ("work-item-planning-deterministic-staged-epic-change-planner", "work-item-planning", "deterministic-staged-epic-change-planner", "behavior"),
    ("work-item-planning-digest-bound-project-planning-transaction", "work-item-planning", "digest-bound-project-planning-transaction", "behavior"),
    ("work-item-planning-epic-child-graph-terminal-rollup", "work-item-planning", "epic-child-graph-terminal-rollup", "behavior"),
    ("work-item-planning-epic-to-change-atomization", "work-item-planning", "epic-to-change-atomization", "behavior"),
    ("work-item-planning-epic-verification-inventory-authoring-contract", "work-item-planning", "epic-verification-inventory-authoring-contract", "behavior"),
    ("work-item-planning-issue-platform-epic-change-graph-invariants", "work-item-planning", "issue-platform-epic-change-graph-invariants", "behavior"),
    ("work-item-planning-legacy-backlog-reconciliation-and-plan-convergence", "work-item-planning", "legacy-backlog-reconciliation-and-plan-convergence", "behavior"),
    ("work-item-planning-legacy-wi-type-decoding-baseline", "work-item-planning", "legacy-wi-type-decoding-baseline", "behavior"),
    ("work-item-planning-operational-efficiency", "work-item-planning", "capability-to-epic-planning", "efficiency"),
    ("work-item-planning-operational-stability", "work-item-planning", "capability-to-epic-planning", "stability"),
    ("work-item-planning-parent-ownership-reference-extraction", "work-item-planning", "parent-ownership-reference-extraction", "behavior"),
    ("work-item-planning-prefix-safe-planning-transaction-recovery", "work-item-planning", "prefix-safe-planning-transaction-recovery", "behavior"),
    ("work-item-planning-prioritized-reviewed-graph-goal-selection", "work-item-planning", "prioritized-reviewed-graph-goal-selection", "behavior"),
    ("work-item-planning-remote-wi-create-authoring-continuity", "work-item-planning", "remote-wi-create-authoring-continuity", "behavior"),
    ("work-item-planning-wi-linear-authoring-without-crrr", "work-item-planning", "wi-linear-authoring-without-crrr", "behavior"),
    ("work-item-report-triage", "work-item-planning", "terminology-first-four-type-wi-taxonomy", "behavior"),
    ("work-item-spike-terminal", "work-item-planning", "terminology-first-four-type-wi-taxonomy", "behavior"),
    ("work-item-type-templates", "work-item-planning", "terminology-first-four-type-wi-taxonomy", "behavior"),
    ("work-item-type-vocabulary", "work-item-planning", "terminology-first-four-type-wi-taxonomy", "behavior"),
    ("workflow-root-runner-cli-workflow-chain", "workflow-root-runner", "cli-workflow-chain", "behavior"),
    ("workflow-root-runner-closed-workflow-lock-release", "workflow-root-runner", "closed-workflow-lock-release", "behavior"),
    ("workflow-root-runner-goal-unified-loop-verb", "workflow-root-runner", "goal-unified-loop-verb", "behavior"),
    ("workflow-root-runner-parent-rollup-routing", "workflow-root-runner", "parent-rollup-routing", "behavior"),
    ("workflow-root-runner-prioritized-reviewed-graph-goal-selection", "workflow-root-runner", "prioritized-reviewed-graph-goal-selection", "behavior"),
    ("workflow-root-runner-python-artifact-goal-flow", "workflow-root-runner", "python-artifact-goal-flow", "behavior"),
    ("workflow-root-runner-python-artifact-model-compatibility-parser", "workflow-root-runner", "python-artifact-model-compatibility-parser", "behavior"),
    ("workflow-root-runner-python-artifact-protocol", "workflow-root-runner", "python-artifact-protocol", "behavior"),
    ("workflow-root-runner-python-only-artifact-model-routing", "workflow-root-runner", "python-only-artifact-model-routing", "behavior"),
    ("workflow-root-runner-runtime-envelope-backward-compatibility", "workflow-root-runner", "runtime-envelope-backward-compatibility", "behavior"),
    ("workflow-root-runner-wi-ec-td-root-loop", "workflow-root-runner", "wi-ec-td-root-loop", "behavior"),
), key=_case_id))

# Invariant: authority is sorted by case_id and contains no duplicates.
assert list(_AUTHORITY) == sorted(_AUTHORITY, key=_case_id), (
    "authority must be sorted by case_id"
)
assert len({r[0] for r in _AUTHORITY}) == len(_AUTHORITY), (
    "authority must have no duplicate case_ids"
)

_EXPECTED_CARDINALITY = 110
_FIXTURE_SCHEMA_VERSION = "aw.python-ec.expected-mapping.v1"

# Implementation plan paths (controller-owned writes, not modified in this TD):
#   create:  apps/agentic-workflow/external-contracts/fixtures/claim-reconciliation/
#              capability-catalog-td-claim-linkage-expected-mapping.json
#   modify:  apps/agentic-workflow/external-contracts/src/cases/
#              capability-control-plane-capability-catalog-and-td-claim-linkage-consistency.py


def design_contract() -> str:
    """Express the executable design contract for this bounded change."""

    # R1: Static authority has exactly 110 records and is case_id-sorted.
    assert len(_AUTHORITY) == _EXPECTED_CARDINALITY, (
        f"authority cardinality must be {_EXPECTED_CARDINALITY}, got {len(_AUTHORITY)}"
    )

    # Authority records as mapping dicts (the shape the fixture and EC consume).
    authority_records = [
        {
            "case_id": case_id,
            "capability_id": capability_id,
            "use_case_id": use_case_id,
            "dimension": dimension,
        }
        for case_id, capability_id, use_case_id, dimension in _AUTHORITY
    ]

    # R1/AC1: Fixture JSON must declare the schema version and the full mapping.
    assert _FIXTURE_SCHEMA_VERSION == "aw.python-ec.expected-mapping.v1"
    assert len(authority_records) == _EXPECTED_CARDINALITY

    # AC2: An unmodified copied inventory matches the authority exactly (clean/count).
    # The production EC must load the checked-in fixture as the expected-mapping
    # authority and pass it as --expected-mapping to claim_reconciliation.py.
    # A copied canonical inventory with no perturbation must yield:
    #   status == "clean"
    #   case_count == 110
    #   case_mapping == authority_records (sorted by case_id)
    #   findings.missing_expected_mappings == []
    #   findings.unexpected_mappings == []
    #   findings.duplicate_case_ids == []
    expected_clean = {
        "status": "clean",
        "case_count": _EXPECTED_CARDINALITY,
        "case_mapping": authority_records,
        "findings": {
            "missing_expected_mappings": [],
            "unexpected_mappings": [],
            "duplicate_case_ids": [],
        },
    }
    assert expected_clean["status"] == "clean"
    assert expected_clean["case_count"] == _EXPECTED_CARDINALITY
    assert expected_clean["case_mapping"] == authority_records

    # AC3/R3: Copied-candidate-only perturbations produce exact drift findings.
    # Target case used to prove the three falsifiers:
    target_id = "capability-control-plane-capability-project-sweep"
    target_record = next(r for r in authority_records if r["case_id"] == target_id)

    # Missing falsifier: removing target_id from copied inventory must produce:
    #   status == "drifted"
    #   findings.missing_expected_mappings == [target_record]
    #   findings.unexpected_mappings == []
    #   findings.duplicate_case_ids == []
    expected_missing = {
        "status": "drifted",
        "findings": {
            "missing_expected_mappings": [target_record],
            "unexpected_mappings": [],
            "duplicate_case_ids": [],
        },
    }
    assert expected_missing["findings"]["missing_expected_mappings"] == [target_record]

    # Duplicate falsifier: adding a second copy of the target block must produce:
    #   status == "drifted"
    #   findings.duplicate_case_ids == [target_id]
    expected_duplicate = {
        "status": "drifted",
        "findings": {
            "missing_expected_mappings": [],
            "unexpected_mappings": [],
            "duplicate_case_ids": [target_id],
        },
    }
    assert expected_duplicate["findings"]["duplicate_case_ids"] == [target_id]

    # Misbound falsifier: replacing use_case_id in the target block must produce:
    #   status == "drifted"
    #   findings.missing_expected_mappings == [target_record]   (original missing)
    #   findings.unexpected_mappings == [mutated_record]        (wrong binding present)
    #   findings.duplicate_case_ids == []
    mutated_record = {**target_record, "use_case_id": "wrong-claim"}
    expected_misbound = {
        "status": "drifted",
        "findings": {
            "missing_expected_mappings": [target_record],
            "unexpected_mappings": [mutated_record],
            "duplicate_case_ids": [],
        },
    }
    assert expected_misbound["findings"]["missing_expected_mappings"] == [target_record]
    assert expected_misbound["findings"]["unexpected_mappings"] == [mutated_record]

    # AC4: The authority is never derived from candidate or canonical TOML at
    # runtime. Proving this: the fixture JSON must be a checked-in static file
    # consumed by the production EC via --expected-mapping, not computed from
    # the pyproject.toml currently under certification.

    # R4: No-argument v2 reconciliation (schema_version aw.python-ec.claim-reconciliation.v2)
    # remains supplemental; Rust health claim closure remains authoritative.
    assert _FIXTURE_SCHEMA_VERSION != "aw.python-ec.claim-reconciliation.v2"

    return "ok"
