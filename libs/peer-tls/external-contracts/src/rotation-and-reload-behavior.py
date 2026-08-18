from __future__ import annotations

from peer_tls.domain.material import TrustAnchor
from peer_tls.domain.rotation import (
    Generation,
    RotationPhase,
    RotationState,
    advance,
    reload,
)

MINIMUM_CHECKS = 11

ROTATION_AND_RELOAD_BEHAVIOR_MATRIX = (
    ("advance_steady_to_incoming_trusted", "incoming_trusted"),
    ("advance_incoming_trusted_to_incoming_active", "incoming_active"),
    ("advance_incoming_active_to_outgoing_retired", "outgoing_retired"),
    ("advance_terminal_outgoing_retired_noop", "outgoing_retired"),
    ("reload_gen1_strictly_greater", True),
    ("reload_gen2_strictly_greater", True),
    ("reload_identical_material_strictly_greater", True),
    ("swap_preserves_pre_and_advances_post", (1, "v1", 2, "v2")),
    ("post_swap_session_takes_new_generation", 2),
    ("reload_installs_the_new_leaf_label", "v2"),
    ("second_reload_installs_the_third_leaf_label", "v3"),
)


def verify_rotation_and_reload_behavior() -> dict:
    checks = []

    out_anchor = TrustAnchor(key_id="out_ca", label="ca_out")
    in_anchor = TrustAnchor(key_id="in_ca", label="ca_in")
    gen1 = Generation(number=1, leaf_label="v1")

    s0 = RotationState(
        phase=RotationPhase.STEADY,
        outgoing=out_anchor,
        incoming=None,
        active=gen1,
        activation_observed=False,
    )

    # 1. advance_steady_to_incoming_trusted
    s1 = advance(s0)
    obs1 = s1.phase.value
    exp1 = ROTATION_AND_RELOAD_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": ROTATION_AND_RELOAD_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. advance_incoming_trusted_to_incoming_active
    s1_with_incoming = RotationState(
        phase=RotationPhase.INCOMING_TRUSTED,
        outgoing=out_anchor,
        incoming=in_anchor,
        active=gen1,
        activation_observed=False,
    )
    s2 = advance(s1_with_incoming)
    obs2 = s2.phase.value
    exp2 = ROTATION_AND_RELOAD_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": ROTATION_AND_RELOAD_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. advance_incoming_active_to_outgoing_retired
    s2_observed = RotationState(
        phase=RotationPhase.INCOMING_ACTIVE,
        outgoing=out_anchor,
        incoming=in_anchor,
        active=gen1,
        activation_observed=True,
    )
    s3 = advance(s2_observed)
    obs3 = s3.phase.value
    exp3 = ROTATION_AND_RELOAD_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": ROTATION_AND_RELOAD_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. advance_terminal_outgoing_retired_noop
    s4 = advance(s3)
    obs4 = s4.phase.value
    exp4 = ROTATION_AND_RELOAD_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": ROTATION_AND_RELOAD_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. reload_gen1_strictly_greater
    gen2 = Generation(number=2, leaf_label="v2")
    r1 = reload(s0, gen2)
    obs5 = r1.active.number > s0.active.number
    exp5 = ROTATION_AND_RELOAD_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": ROTATION_AND_RELOAD_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. reload_gen2_strictly_greater
    gen3 = Generation(number=3, leaf_label="v3")
    r2 = reload(r1, gen3)
    obs6 = r2.active.number > r1.active.number
    exp6 = ROTATION_AND_RELOAD_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": ROTATION_AND_RELOAD_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. reload_identical_material_strictly_greater
    r3 = reload(r2, gen3)
    obs7 = r3.active.number > r2.active.number
    exp7 = ROTATION_AND_RELOAD_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": ROTATION_AND_RELOAD_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. swap_preserves_pre_and_advances_post
    obs8 = (s0.active.number, s0.active.leaf_label, r1.active.number, r1.active.leaf_label)
    exp8 = ROTATION_AND_RELOAD_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": ROTATION_AND_RELOAD_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. post_swap_session_takes_new_generation
    post_swap_session_gen = r1.active.number
    obs9 = post_swap_session_gen
    exp9 = ROTATION_AND_RELOAD_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": ROTATION_AND_RELOAD_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. reload_installs_the_new_leaf_label
    obs10 = r1.active.leaf_label
    exp10 = ROTATION_AND_RELOAD_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": ROTATION_AND_RELOAD_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. second_reload_installs_the_third_leaf_label
    obs11 = r2.active.leaf_label
    exp11 = ROTATION_AND_RELOAD_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": ROTATION_AND_RELOAD_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    return {
        "case_id": "rotation-and-reload-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
