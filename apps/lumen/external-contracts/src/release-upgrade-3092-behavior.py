"""EC behavior case for #3092 -- N-1-to-N release-journey admission.

Every expected value is an EC-owned literal transcribed from #3092 R1-R11 and
AC3.  This pure contract deliberately excludes the GKE, traffic, time,
storage, and cloud-cleanup observations assigned to the runtime stage.
"""

from __future__ import annotations

from lumen.release_upgrade.admission import (
    decide_image_observation,
    decide_lifecycle_operation,
    decide_membership_transition,
    decide_release_metadata,
    decide_rollout,
)
from lumen.release_upgrade.compatibility import decide_api_visibility
from lumen.release_upgrade.compatibility import decide_restore
from lumen.release_upgrade.evidence import validate_run_evidence
from lumen.release_upgrade.spec import (
    ApiVisibilityRequest,
    CompatibilityDescriptor,
    ImageObservationRequest,
    LifecycleOperationRequest,
    MembershipTransitionRequest,
    BackupManifest,
    ReleaseIdentity,
    ReleaseMetadataRequest,
    RolloutRequest,
    RunEvidenceManifest,
)
from lumen.release_upgrade.verdict import Rejected

MINIMUM_CHECKS = 14

RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX = (
    ("admitted_metadata_preserves_distinct_generations_and_digests", ("N-1", "sha256:previous", "N", "sha256:target")),
    ("admitted_metadata_preserves_binary_and_build_identities", ("lumen-1.9.0", "build-previous", "lumen-2.0.0", "build-target")),
    ("admitted_metadata_preserves_public_peer_and_durable_compatibility", (("public-v1", "peer-v1", ("wal-v1", "segment-v1")), ("public-v1", "peer-v1", ("wal-v1", "segment-v1")))),
    ("crd_first_rollout_orders_crd_before_operator", ("crd-N", "operator-N-1-to-N")),
    ("crd_first_rollout_excludes_unchanged_n_minus_one_instances_from_target", ("orders-0",)),
    ("one_voter_transition_inserts_learner_before_voter_replacement", ("add-learner", "replace-voter")),
    ("three_voter_transition_upgrades_followers_before_leader", ("follower-0", "follower-1", "leader-2")),
    ("shared_api_is_available_before_activation", "available"),
    ("n_only_api_is_held_before_activation", "held"),
    ("n_only_api_is_available_after_explicit_activation", "available"),
    ("shared_api_remains_available_after_explicit_activation", "available"),
    ("matching_digest_observation_retains_pinned_authoritative_target", ("sha256:target", "authoritative")),
    ("forward_compatible_backup_manifest_is_admitted_for_restore", "admitted"),
    ("complete_run_evidence_is_admitted", "admitted"),
)


def _outcome(value) -> str:
    return value.reason.value if isinstance(value, Rejected) else "admitted"


def _identity(generation: str, digest: str, binary: str, build: str) -> ReleaseIdentity:
    return ReleaseIdentity(
        generation=generation,
        image_digest=digest,
        binary_identity=binary,
        build_identity=build,
        compatibility=CompatibilityDescriptor(
            public_contract="public-v1",
            peer_contract="peer-v1",
            durable_formats=("wal-v1", "segment-v1"),
        ),
    )


def verify_release_upgrade_3092_behavior() -> dict:
    checks = []
    previous = _identity("N-1", "sha256:previous", "lumen-1.9.0", "build-previous")
    target = _identity("N", "sha256:target", "lumen-2.0.0", "build-target")

    # 1-3. R1 -- release identity is a complete pair of immutable facts, not
    # a mutable tag plus a single version string.
    metadata = decide_release_metadata(ReleaseMetadataRequest(previous=previous, target=target))
    admitted = metadata.metadata if not isinstance(metadata, Rejected) else None
    obs1 = (admitted.previous.generation, admitted.previous.image_digest, admitted.target.generation, admitted.target.image_digest) if admitted else ()
    exp1 = RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = (admitted.previous.binary_identity, admitted.previous.build_identity, admitted.target.binary_identity, admitted.target.build_identity) if admitted else ()
    exp2 = RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    obs3 = tuple((item.compatibility.public_contract, item.compatibility.peer_contract, item.compatibility.durable_formats) for item in (admitted.previous, admitted.target)) if admitted else ()
    exp3 = RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4-5. R2 -- the abstract plan pins CRD-first ordering and does not claim
    # that an unchanged N-1 instance is a rollout target.
    rollout = decide_rollout(RolloutRequest(crd_generation="N", operator_from="N-1", operator_to="N", operator_replicas=2, unchanged_instances=("orders-0",)))
    obs4 = rollout.phases if not isinstance(rollout, Rejected) else ()
    exp4 = RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = rollout.outside_target_instances if not isinstance(rollout, Rejected) else ()
    exp5 = RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6-7. R3/R4 -- one-voter and three-voter journeys have independently
    # observable safe transition orders.
    single = decide_membership_transition(MembershipTransitionRequest(voters=1, phases=("add-learner", "replace-voter"), learner_node="node-b", voter_node="node-a"))
    obs6 = single.phases if not isinstance(single, Rejected) else ()
    exp6 = RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    three = decide_membership_transition(MembershipTransitionRequest(voters=3, phases=("follower-0", "follower-1", "leader-2"), learner_node="node-d", voter_node="node-a"))
    obs7 = three.phases if not isinstance(three, Rejected) else ()
    exp7 = RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8-10. R5/AC3 -- activation gates only N-only calls; the shared contract
    # remains available on both sides of the activation boundary.
    before = decide_api_visibility(ApiVisibilityRequest(shared_call="search", n_only_call="ranking-v2", activated=False, compatibility=target.compatibility))
    obs8 = before.shared_call if not isinstance(before, Rejected) else "rejected"
    exp8 = RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    obs9 = before.n_only_call if not isinstance(before, Rejected) else "rejected"
    exp9 = RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    after = decide_api_visibility(ApiVisibilityRequest(shared_call="search", n_only_call="ranking-v2", activated=True, compatibility=target.compatibility))
    obs10 = after.n_only_call if not isinstance(after, Rejected) else "rejected"
    exp10 = RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    obs11 = after.shared_call if not isinstance(after, Rejected) else "rejected"
    exp11 = RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 11. R6 -- the only observation that can remain authoritative matches the
    # pinned digest exactly.
    image = decide_image_observation(ImageObservationRequest(requested_generation="N", pinned_digest="sha256:target", observed_digest="sha256:target"))
    obs12 = (image.pinned_digest, image.authority) if not isinstance(image, Rejected) else ()
    exp12 = RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 12. R9 -- this model admits only the abstract forward-compatible backup
    # manifest. Reproducing real catalog/data/query state is a runtime oracle.
    restore = decide_restore(BackupManifest(public_epoch="public-v1", durable_epoch="durable-v1", runtime_public_epoch="public-v1", runtime_durable_epoch="durable-v1"))
    obs13 = _outcome(restore)
    exp13 = RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 13. R11 -- only the closed complete evidence vocabulary is admitted by
    # the pure validator; actual retained cloud evidence is runtime proof.
    evidence = validate_run_evidence(RunEvidenceManifest(keys=("release", "traffic", "api_format", "kubernetes", "backup", "cleanup")))
    obs12 = _outcome(evidence)
    exp14 = RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": RELEASE_UPGRADE_3092_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs12, "passed": obs12 == exp14})

    return {"case_id": "release-upgrade-3092-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
