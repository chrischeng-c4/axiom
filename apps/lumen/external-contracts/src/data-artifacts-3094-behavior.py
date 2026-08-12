"""EC behavior case for #3094 -- UID- and authority-stamped data artifacts.

Every expected value is an EC-owned literal from #3094: R1 requires the
complete identity, R3 distinguishes authority, reconstruction, and backup
states, R4 derives identity from observed UID and durable generation, R6 builds
strict selectors, R7 projects exact inventory, and AC2 fences CR incarnations.
Runtime producer coverage, Kubernetes selection, and crate placement are
intentionally absent because this is a pure design-model contract.
"""

from __future__ import annotations

from lumen.data_artifacts.identity import DataArtifactIdentity, derive_identity
from lumen.data_artifacts.inventory import classify_artifact, project_inventory
from lumen.data_artifacts.selector import build_strict_selector
from lumen.data_artifacts.spec import ArtifactFacts

MINIMUM_CHECKS = 18

DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX = (
    ("identity_value_is_frozen", True),
    ("identity_retains_namespace", "payments"),
    ("identity_retains_instance_name", "orders"),
    ("identity_retains_observed_cr_uid", "uid-orders-v2"),
    ("identity_retains_artifact_role", "raft"),
    ("identity_retains_member_identity", "shard-0-member-1"),
    ("identity_retains_topology_generation", 17),
    ("identity_retains_authority_class", "authoritative_voter"),
    ("classification_names_authoritative_voter", "authoritative_voter"),
    ("classification_names_reconstructible_read_replica", "reconstructible_read_replica"),
    ("classification_names_complete_backup_set", "complete_backup_set"),
    ("classification_names_retained_former_authority", "retained_former_authority"),
    ("classification_names_failed_target", "failed_target"),
    ("classification_names_incomplete_backup_set", "incomplete_backup_set"),
    ("derived_identity_uses_observed_uid_not_label", "uid-orders-v2"),
    ("derived_identity_uses_durable_catalog_generation", 17),
    ("same_name_different_uid_has_a_disjoint_selector", True),
    ("inventory_names_only_the_exact_selected_artifact", ("raft-0",)),
)


def _identity(*, uid: str = "uid-orders-v2") -> DataArtifactIdentity:
    return DataArtifactIdentity(
        namespace="payments",
        instance_name="orders",
        cr_uid=uid,
        role="raft",
        member_identity="shard-0-member-1",
        topology_generation=17,
        authority_class="authoritative_voter",
    )


def verify_data_artifacts_3094_behavior() -> dict:
    checks = []
    identity = _identity()

    # 1-8. R1 -- a stable artifact identity contains every named dimension.
    obs1 = getattr(identity, "__dataclass_params__").frozen
    exp1 = DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = identity.namespace
    exp2 = DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    obs3 = identity.instance_name
    exp3 = DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    obs4 = identity.cr_uid
    exp4 = DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = identity.role
    exp5 = DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = identity.member_identity
    exp6 = DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = identity.topology_generation
    exp7 = DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = identity.authority_class
    exp8 = DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9-11. R3 -- authority, reconstruction, and backup completeness stay disjoint.
    voter = classify_artifact(ArtifactFacts(name="raft-0", identity=identity, backup_complete=None, failed_target=False))
    obs9 = voter.classification
    exp9 = DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    replica = classify_artifact(ArtifactFacts(name="read-0", identity=DataArtifactIdentity(namespace="payments", instance_name="orders", cr_uid="uid-orders-v2", role="read-replica", member_identity="shard-0-replica-0", topology_generation=17, authority_class="reconstructible_read_replica"), backup_complete=None, failed_target=False))
    obs10 = replica.classification
    exp10 = DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    backup = classify_artifact(ArtifactFacts(name="backup-17", identity=DataArtifactIdentity(namespace="payments", instance_name="orders", cr_uid="uid-orders-v2", role="backup", member_identity="backup-set-17", topology_generation=17, authority_class="backup"), backup_complete=True, failed_target=False))
    obs11 = backup.classification
    exp11 = DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12-14. R3 -- retained authority, failed targets, and incomplete backups
    # retain their own vocabulary rather than collapsing into the safe classes.
    retained = classify_artifact(ArtifactFacts(name="former-raft-0", identity=DataArtifactIdentity(namespace="payments", instance_name="orders", cr_uid="uid-orders-v2", role="raft", member_identity="shard-0-member-0", topology_generation=16, authority_class="retained_former_authority"), backup_complete=None, failed_target=False))
    obs12 = retained.classification
    exp12 = DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    failed = classify_artifact(ArtifactFacts(name="target-0", identity=identity, backup_complete=None, failed_target=True))
    obs13 = failed.classification
    exp13 = DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    incomplete_backup = classify_artifact(ArtifactFacts(name="backup-partial-17", identity=DataArtifactIdentity(namespace="payments", instance_name="orders", cr_uid="uid-orders-v2", role="backup", member_identity="backup-set-17", topology_generation=17, authority_class="backup"), backup_complete=False, failed_target=False))
    obs14 = incomplete_backup.classification
    exp14 = DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15-16. R4 -- UID and catalog generation are supplied observations, not a label claim.
    derived = derive_identity(namespace="payments", instance_name="orders", observed_cr_uid="uid-orders-v2", role="raft", member_identity="shard-0-member-1", catalog_generation=17, authority_class="authoritative_voter", mutable_label="orders")
    obs15 = derived.cr_uid
    exp15 = DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})
    obs16 = derived.topology_generation
    exp16 = DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[15][1]
    checks.append({"name": DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    # 14. R6/AC2 -- name reuse must not select artifacts from a prior CR UID.
    obs17 = build_strict_selector(_identity()) != build_strict_selector(_identity(uid="uid-orders-v1"))
    exp17 = DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[16][1]
    checks.append({"name": DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    # 18. R7/AC4 -- projection retains the exact matching supplied artifact,
    # never the same-name fact from a different CR incarnation.
    inventory = project_inventory(identity, (ArtifactFacts(name="raft-0", identity=identity, backup_complete=None, failed_target=False), ArtifactFacts(name="raft-old", identity=_identity(uid="uid-orders-v1"), backup_complete=None, failed_target=False)))
    obs18 = tuple(row.name for row in inventory)
    exp18 = DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[17][1]
    checks.append({"name": DATA_ARTIFACTS_3094_BEHAVIOR_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    return {"case_id": "data-artifacts-3094-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
