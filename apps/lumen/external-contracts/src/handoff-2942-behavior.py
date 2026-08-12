"""EC behavior case for #2942 -- safe member-handoff decisions.

Every expected value is an EC-owned literal from #2942: R1 orders the persisted
handoff phases, R2 exposes the temporary workload-generation surge and its
cost, R3 protects every committed voter while excluding learners from quorum,
R4 resumes an already-persisted handoff idempotently, R5 keeps the old member
authoritative on target failure, R6 makes planned-handoff availability explicit,
and AC4 distinguishes convergence from rollback authority.
"""

from __future__ import annotations

from lumen.topology.availability import planned_handoff_availability
from lumen.topology.handoff_admission import (
    decide_handoff_transition,
    decide_resume,
    decide_target_failure,
    decide_voter_protection,
)
from lumen.topology.handoff_spec import (
    HandoffFacts,
    HandoffPhase,
    ResumeFacts,
    TargetFailureFacts,
    VoterProtectionFacts,
)
from lumen.topology.handoff_status import derive_handoff_status
from lumen.topology.handoff_verdict import Rejection

MINIMUM_CHECKS = 11

HANDOFF_2942_BEHAVIOR_MATRIX = (
    ("provision_target_advances_only_to_add_learner", "add_learner"),
    ("add_learner_advances_only_to_catch_up", "catch_up"),
    ("catch_up_advances_only_to_protect_new_voter_set", "protect_new_voter_set"),
    ("surge_status_exposes_old_and_target_generation_roles", (("generation-7", "old"), ("generation-8", "target"))),
    ("surge_status_exposes_one_extra_generation_cost", 1),
    ("converged_status_has_exactly_one_steady_generation", 1),
    ("committed_voters_are_all_protected", ("old-a", "old-b", "new-c")),
    ("learners_are_excluded_from_the_quorum", ("old-a", "old-b", "new-c")),
    ("persisted_catch_up_resumes_with_one_catch_up_action", "catch_up"),
    ("failed_target_rolls_back_to_old_authority", ("rollback", "old-a")),
    ("planned_handoff_has_an_explicit_availability_promise", "planned_handoff_available"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else verdict.action


def verify_handoff_2942_behavior() -> dict:
    checks = []

    transition_facts = HandoffFacts(
        old_member="old-a", target_member="new-c", old_generation="generation-7",
        target_generation="generation-8", old_voters=("old-a", "old-b"),
        new_voters=("old-a", "old-b", "new-c"), committed_voters=("old-a", "old-b"),
        learners=(), protected_voters=("old-a", "old-b"), target_healthy=True,
    )
    # 1-3. R1 -- the persisted phase machine permits only the stated order.
    first = decide_handoff_transition(HandoffPhase.PROVISION_TARGET, HandoffPhase.ADD_LEARNER, transition_facts)
    obs1 = _outcome(first); exp1 = HANDOFF_2942_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": HANDOFF_2942_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    second = decide_handoff_transition(HandoffPhase.ADD_LEARNER, HandoffPhase.CATCH_UP, transition_facts)
    obs2 = _outcome(second); exp2 = HANDOFF_2942_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": HANDOFF_2942_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    third = decide_handoff_transition(HandoffPhase.CATCH_UP, HandoffPhase.PROTECT_NEW_VOTER_SET, transition_facts)
    obs3 = _outcome(third); exp3 = HANDOFF_2942_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": HANDOFF_2942_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    surge = derive_handoff_status(HandoffFacts(
        old_member="old-a", target_member="new-c", old_generation="generation-7", target_generation="generation-8",
        active_generation_roles=(("generation-7", "old"), ("generation-8", "target")),
        old_voters=("old-a",), new_voters=("old-a",), committed_voters=("old-a",), learners=("new-c",),
        protected_voters=("old-a",), target_healthy=True, phase=HandoffPhase.CATCH_UP,
    ))
    # 4-6. R2/AC4 -- a surge is visible and convergence is exactly one steady generation.
    obs4 = surge.active_generation_roles; exp4 = HANDOFF_2942_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": HANDOFF_2942_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = surge.temporary_generation_cost; exp5 = HANDOFF_2942_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": HANDOFF_2942_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    converged = derive_handoff_status(HandoffFacts(
        old_member="old-a", target_member="new-c", old_generation="generation-7", target_generation="generation-8",
        active_generation_roles=(("generation-8", "steady"),), old_voters=("old-a",), new_voters=("new-c",),
        committed_voters=("new-c",), learners=(), protected_voters=("new-c",), target_healthy=True,
        phase=HandoffPhase.CLEAN_UP,
    ))
    obs6 = converged.steady_generation_count; exp6 = HANDOFF_2942_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": HANDOFF_2942_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    voter = decide_voter_protection(VoterProtectionFacts(
        old_voters=("old-a", "old-b"), new_voters=("old-a", "old-b", "new-c"),
        committed_voters=("old-a", "old-b", "new-c"), learners=("new-d",),
        protected_voters=("old-a", "old-b", "new-c"), phase=HandoffPhase.PROTECT_NEW_VOTER_SET,
    ))
    # 7-8. R3 -- a learner is explicit input, but never changes the protected quorum.
    obs7 = voter.protected_committed_voters if not isinstance(voter, Rejection) else ()
    exp7 = HANDOFF_2942_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": HANDOFF_2942_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = voter.quorum_voters if not isinstance(voter, Rejection) else ()
    exp8 = HANDOFF_2942_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": HANDOFF_2942_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R4/AC2 -- replaying a persisted phase yields one action, not a second member add.
    resume = decide_resume(ResumeFacts(phase=HandoffPhase.CATCH_UP, old_member="old-a", target_member="new-c", target_present=True, target_caught_up=False, old_member_removed=False))
    obs9 = _outcome(resume); exp9 = HANDOFF_2942_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": HANDOFF_2942_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R5/AC4 -- an explicit unschedulable target leaves the known-good old member authoritative.
    failed = decide_target_failure(TargetFailureFacts(old_member="old-a", target_member="new-c", target_healthy=False, target_schedulable=False, old_member_healthy=True))
    obs10 = (failed.action, failed.authoritative_member) if not isinstance(failed, Rejection) else (failed.reason.value, "")
    exp10 = HANDOFF_2942_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": HANDOFF_2942_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R6 -- planned movement is distinct from the no-promise unexpected-loss policy.
    obs11 = planned_handoff_availability(voters=1, target_healthy=True, routing_transferred=False)
    exp11 = HANDOFF_2942_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": HANDOFF_2942_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    return {"case_id": "handoff-2942-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
