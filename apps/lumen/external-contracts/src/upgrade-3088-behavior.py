"""EC behavior case for #3088 -- immutable-digest target staging.

Every expected value below is an EC-owned literal transcribed from #3088:
R1/AC1 retains the authoritative generation while staging a changed image; R2
records the CR-keyed operation identity; R3 creates one target generation from
stable members; R4/AC2 admits only complete target evidence; R6/AC3 resumes
the same target identity; and R8 projects the complete upgrade status surface.
"""

from __future__ import annotations

from lumen.upgrade.admission import decide_authority, decide_upgrade, plan_target
from lumen.upgrade.operation import claim_upgrade, resume_upgrade
from lumen.upgrade.spec import (
    CurrentGeneration,
    RequestedImage,
    StableMemberIdentity,
    TargetObservation,
    TemporaryCapacity,
)
from lumen.upgrade.status import project_upgrade_status
from lumen.upgrade.verdict import Rejection

MINIMUM_CHECKS = 10

UPGRADE_3088_BEHAVIOR_MATRIX = (
    ("changed_image_stages_a_target_without_replacing_current_authority", ("staged_target", "current_authoritative_unchanged")),
    ("unchanged_image_has_no_staged_target", None),
    ("operation_identity_records_every_issue_required_value", (42, "lumen:v1", "sha256:current", "release-7", "lumen:v2", "sha256:target")),
    ("admitted_plan_has_exactly_one_target_generation", 1),
    ("stable_members_map_deterministically_to_target_members", (("stable-a", "upgrade-42-target-a"), ("stable-b", "upgrade-42-target-b"))),
    ("complete_target_evidence_is_admitted_for_authority", "admitted"),
    ("repeated_resume_derives_the_same_target_generation", ("upgrade-42-target", "upgrade-42-target")),
    ("repeated_resume_derives_the_same_target_ownership", ((("stable-a", "upgrade-42-target-a"), ("stable-b", "upgrade-42-target-b")), (("stable-a", "upgrade-42-target-a"), ("stable-b", "upgrade-42-target-b")))),
    ("status_projects_all_required_upgrade_fields", ("current_generation", "target_generation", "requested_image", "resolved_digest", "release", "phase", "blocker", "temporary_capacity")),
    ("status_retains_the_staged_digest_and_capacity_state", ("sha256:target", "temporary_capacity_reserved")),
)


def _outcome(value) -> str:
    return value.reason.value if isinstance(value, Rejection) else "admitted"


def verify_upgrade_3088_behavior() -> dict:
    checks = []
    current = CurrentGeneration(
        generation="generation-7", image="lumen:v1", digest="sha256:current", release="release-7"
    )
    requested = RequestedImage(image="lumen:v2", resolved_digest="sha256:target")

    # 1. R1/AC1 -- a changed desired image stages a target; it does not turn
    # the still-serving generation into a native replacement.
    changed = decide_upgrade(current, requested)
    obs1 = (changed.disposition, changed.authority_state)
    exp1 = UPGRADE_3088_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": UPGRADE_3088_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1/AC1 -- naming the existing image explicitly proves the no-change
    # neighbour does not manufacture a temporary generation.
    unchanged = decide_upgrade(current, RequestedImage(image="lumen:v1", resolved_digest="sha256:current"))
    obs2 = unchanged.target_generation
    exp2 = UPGRADE_3088_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": UPGRADE_3088_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    operation = claim_upgrade(42, current, requested)

    # 3. R2 -- the operation identity is the CR generation plus every image,
    # digest, and release fact the later reconciliation must not guess.
    obs3 = (operation.cr_generation, operation.current_image, operation.current_digest, operation.current_release, operation.requested_image, operation.target_digest)
    exp3 = UPGRADE_3088_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": UPGRADE_3088_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    members = (StableMemberIdentity("stable-a"), StableMemberIdentity("stable-b"))
    plan = plan_target(operation, members)

    # 4. R3 -- one admitted operation is allowed precisely one target generation.
    obs4 = plan.target_generation_count
    exp4 = UPGRADE_3088_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": UPGRADE_3088_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R3 -- the temporary identities are derived from, rather than replacing,
    # the supplied stable member identities.
    obs5 = tuple((item.stable_member, item.target_member) for item in plan.member_identities)
    exp5 = UPGRADE_3088_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": UPGRADE_3088_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R4/AC2 -- only the complete positive evidence tuple admits target
    # authority. The security case pins each missing component independently.
    authority = decide_authority(TargetObservation(digest_matches=True, compatible=True, ready=True))
    obs6 = _outcome(authority)
    exp6 = UPGRADE_3088_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": UPGRADE_3088_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    first_resume = resume_upgrade(operation, members)
    second_resume = resume_upgrade(operation, members)

    # 7. R6/AC3 -- repeated reconciliation over the persisted operation derives
    # the same target generation, rather than a newly minted generation.
    obs7 = (first_resume.target_generation, second_resume.target_generation)
    exp7 = UPGRADE_3088_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": UPGRADE_3088_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R6/AC3 -- it also preserves the stable-to-target ownership relation.
    obs8 = (first_resume.target_ownership, second_resume.target_ownership)
    exp8 = UPGRADE_3088_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": UPGRADE_3088_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    status = project_upgrade_status(operation, changed, TemporaryCapacity(state="temporary_capacity_reserved"))

    # 9. R8 -- the projection has all required operator-visible compartments.
    obs9 = tuple(status.__dataclass_fields__)
    exp9 = UPGRADE_3088_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": UPGRADE_3088_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R8 -- it carries the immutable staged digest and temporary capacity,
    # rather than only generic phase text.
    obs10 = (status.resolved_digest, status.temporary_capacity)
    exp10 = UPGRADE_3088_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": UPGRADE_3088_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    return {"case_id": "upgrade-3088-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
