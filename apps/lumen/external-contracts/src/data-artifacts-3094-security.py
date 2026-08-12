"""EC security case for #3094 -- fail-closed data-artifact selection.

Expected values are EC-owned literals from #3094: R4 rejects mutable-label-only
authority, R5 blocks legacy reconciliation unless all three independent facts
agree, R6 refuses incomplete selectors, R7 retains exact selection, and AC3
excludes incomplete or contradictory artifacts from automatic deletion.  Live
Kubernetes reconciliation and deletion are runtime-only and not modeled here.
"""

from __future__ import annotations

from lumen.data_artifacts.identity import DataArtifactIdentity, derive_identity
from lumen.data_artifacts.inventory import (
    automatic_deletion_candidates,
    decide_legacy_reconciliation,
    project_inventory,
)
from lumen.data_artifacts.selector import build_strict_selector
from lumen.data_artifacts.spec import ArtifactFacts, LegacyReconciliationFacts
from lumen.data_artifacts.verdict import DeletionBlocked, Refused

MINIMUM_CHECKS = 14

DATA_ARTIFACTS_3094_SECURITY_MATRIX = (
    ("mutable_label_only_identity_proof_is_refused", "mutable_label_not_authority"),
    ("mutable_label_refusal_names_authority_proof", "observed_cr_uid"),
    ("observed_uid_and_catalog_generation_neighbour_is_admitted", "uid-orders-v2"),
    ("owner_reference_mismatch_blocks_legacy_reconciliation", "owner_reference_mismatch"),
    ("owner_reference_block_names_owner_reference", "owner_reference_uid"),
    ("workload_identity_mismatch_blocks_legacy_reconciliation", "workload_identity_mismatch"),
    ("workload_block_names_workload_identity", "workload_identity"),
    ("catalog_state_mismatch_blocks_legacy_reconciliation", "catalog_state_mismatch"),
    ("catalog_block_names_catalog_generation", "catalog_generation"),
    ("three_matching_legacy_facts_are_adoptable", "adoptable"),
    ("incomplete_identity_is_refused_by_strict_selector", "identity_incomplete"),
    ("selector_refusal_names_missing_identity_field", "cr_uid"),
    ("missing_or_contradictory_artifacts_are_not_deletion_candidates", ()),
    ("inventory_omits_same_namespace_nonmatching_uid", ("raft-0",)),
)


def _identity(*, uid: str = "uid-orders-v2") -> DataArtifactIdentity:
    return DataArtifactIdentity(namespace="payments", instance_name="orders", cr_uid=uid, role="raft", member_identity="shard-0-member-1", topology_generation=17, authority_class="authoritative_voter")


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, (Refused, DeletionBlocked)) else verdict.action


def _legacy(**overrides) -> LegacyReconciliationFacts:
    values = {"identity": _identity(), "owner_reference_uid": "uid-orders-v2", "workload_identity": "orders", "catalog_generation": 17}
    values.update(overrides)
    return LegacyReconciliationFacts(**values)


def verify_data_artifacts_3094_security() -> dict:
    checks = []

    # 1-3. R4 -- a mutable name label alone cannot establish data authority.
    label_only = derive_identity(namespace="payments", instance_name="orders", observed_cr_uid=None, role="raft", member_identity="shard-0-member-1", catalog_generation=None, authority_class="authoritative_voter", mutable_label="orders")
    obs1 = _outcome(label_only)
    exp1 = DATA_ARTIFACTS_3094_SECURITY_MATRIX[0][1]
    checks.append({"name": DATA_ARTIFACTS_3094_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = label_only.field_path if isinstance(label_only, Refused) else "admitted"
    exp2 = DATA_ARTIFACTS_3094_SECURITY_MATRIX[1][1]
    checks.append({"name": DATA_ARTIFACTS_3094_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    observed = derive_identity(namespace="payments", instance_name="orders", observed_cr_uid="uid-orders-v2", role="raft", member_identity="shard-0-member-1", catalog_generation=17, authority_class="authoritative_voter", mutable_label="renamed-by-user")
    obs3 = observed.cr_uid if not isinstance(observed, Refused) else "refused"
    exp3 = DATA_ARTIFACTS_3094_SECURITY_MATRIX[2][1]
    checks.append({"name": DATA_ARTIFACTS_3094_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4-10. R5 -- all three proofs are independently necessary and jointly sufficient.
    owner_bad = decide_legacy_reconciliation(_legacy(owner_reference_uid="uid-other"))
    obs4 = _outcome(owner_bad)
    exp4 = DATA_ARTIFACTS_3094_SECURITY_MATRIX[3][1]
    checks.append({"name": DATA_ARTIFACTS_3094_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = owner_bad.field_path if isinstance(owner_bad, DeletionBlocked) else "adoptable"
    exp5 = DATA_ARTIFACTS_3094_SECURITY_MATRIX[4][1]
    checks.append({"name": DATA_ARTIFACTS_3094_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    workload_bad = decide_legacy_reconciliation(_legacy(workload_identity="other"))
    obs6 = _outcome(workload_bad)
    exp6 = DATA_ARTIFACTS_3094_SECURITY_MATRIX[5][1]
    checks.append({"name": DATA_ARTIFACTS_3094_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = workload_bad.field_path if isinstance(workload_bad, DeletionBlocked) else "adoptable"
    exp7 = DATA_ARTIFACTS_3094_SECURITY_MATRIX[6][1]
    checks.append({"name": DATA_ARTIFACTS_3094_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    catalog_bad = decide_legacy_reconciliation(_legacy(catalog_generation=16))
    obs8 = _outcome(catalog_bad)
    exp8 = DATA_ARTIFACTS_3094_SECURITY_MATRIX[7][1]
    checks.append({"name": DATA_ARTIFACTS_3094_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    obs9 = catalog_bad.field_path if isinstance(catalog_bad, DeletionBlocked) else "adoptable"
    exp9 = DATA_ARTIFACTS_3094_SECURITY_MATRIX[8][1]
    checks.append({"name": DATA_ARTIFACTS_3094_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    exact = decide_legacy_reconciliation(_legacy())
    obs10 = _outcome(exact)
    exp10 = DATA_ARTIFACTS_3094_SECURITY_MATRIX[9][1]
    checks.append({"name": DATA_ARTIFACTS_3094_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11-12. R6 -- selector construction rejects an explicitly missing UID.
    incomplete = build_strict_selector(DataArtifactIdentity(namespace="payments", instance_name="orders", cr_uid="", role="raft", member_identity="shard-0-member-1", topology_generation=17, authority_class="authoritative_voter"))
    obs11 = _outcome(incomplete)
    exp11 = DATA_ARTIFACTS_3094_SECURITY_MATRIX[10][1]
    checks.append({"name": DATA_ARTIFACTS_3094_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    obs12 = incomplete.field_path if isinstance(incomplete, Refused) else "selected"
    exp12 = DATA_ARTIFACTS_3094_SECURITY_MATRIX[11][1]
    checks.append({"name": DATA_ARTIFACTS_3094_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. AC3 -- neither missing metadata nor contradictory facts enter an automatic deletion set.
    candidates = automatic_deletion_candidates(_identity(), (ArtifactFacts(name="missing-uid", identity=DataArtifactIdentity(namespace="payments", instance_name="orders", cr_uid="", role="raft", member_identity="shard-0-member-1", topology_generation=17, authority_class="authoritative_voter"), backup_complete=None, failed_target=False), ArtifactFacts(name="contradictory", identity=_identity(), observed_cr_uid="uid-other", backup_complete=None, failed_target=False)))
    obs13 = tuple(candidate.name for candidate in candidates)
    exp13 = DATA_ARTIFACTS_3094_SECURITY_MATRIX[12][1]
    checks.append({"name": DATA_ARTIFACTS_3094_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R7/AC4 -- a same-namespace artifact from an older CR incarnation is not inventory.
    inventory = project_inventory(_identity(), (ArtifactFacts(name="raft-0", identity=_identity(), backup_complete=None, failed_target=False), ArtifactFacts(name="raft-old", identity=_identity(uid="uid-orders-v1"), backup_complete=None, failed_target=False)))
    obs14 = tuple(row.name for row in inventory)
    exp14 = DATA_ARTIFACTS_3094_SECURITY_MATRIX[13][1]
    checks.append({"name": DATA_ARTIFACTS_3094_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    return {"case_id": "data-artifacts-3094-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
