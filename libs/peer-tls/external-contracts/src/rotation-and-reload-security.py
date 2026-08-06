from __future__ import annotations

from peer_tls.domain.material import TrustAnchor
from peer_tls.domain.rotation import (
    Generation,
    RotationPhase,
    RotationState,
    advance,
)

MINIMUM_CHECKS = 18

ROTATION_AND_RELOAD_SECURITY_MATRIX = (
    ("steady_admits_outgoing", True),
    ("steady_admits_incoming", False),
    ("steady_admits_unrelated", False),
    ("incoming_trusted_admits_outgoing", True),
    ("incoming_trusted_admits_incoming", True),
    ("incoming_trusted_admits_unrelated", False),
    ("incoming_active_admits_outgoing", True),
    ("incoming_active_admits_incoming", True),
    ("incoming_active_admits_unrelated", False),
    ("outgoing_retired_admits_outgoing", False),
    ("outgoing_retired_admits_incoming", True),
    ("outgoing_retired_admits_unrelated", False),
    ("retirement_guard_blocked_when_unobserved", "incoming_active"),
    ("retirement_guard_allowed_when_observed", "outgoing_retired"),
    ("steady_requires_mutual_auth", True),
    ("incoming_trusted_requires_mutual_auth", True),
    ("incoming_active_requires_mutual_auth", True),
    ("outgoing_retired_requires_mutual_auth", True),
)


def verify_rotation_and_reload_security() -> dict:
    checks = []

    out_anchor = TrustAnchor(key_id="out_ca_key", label="ca_out")
    in_anchor = TrustAnchor(key_id="in_ca_key", label="ca_in")
    gen1 = Generation(number=1, leaf_label="v1")
    gen2 = Generation(number=2, leaf_label="v2")

    state_steady = RotationState(
        phase=RotationPhase.STEADY,
        outgoing=out_anchor,
        incoming=None,
        active=gen1,
        activation_observed=False,
    )

    state_trusted = RotationState(
        phase=RotationPhase.INCOMING_TRUSTED,
        outgoing=out_anchor,
        incoming=in_anchor,
        active=gen1,
        activation_observed=False,
    )

    state_active_unobserved = RotationState(
        phase=RotationPhase.INCOMING_ACTIVE,
        outgoing=out_anchor,
        incoming=in_anchor,
        active=gen2,
        activation_observed=False,
    )

    state_active_observed = RotationState(
        phase=RotationPhase.INCOMING_ACTIVE,
        outgoing=out_anchor,
        incoming=in_anchor,
        active=gen2,
        activation_observed=True,
    )

    state_retired = RotationState(
        phase=RotationPhase.OUTGOING_RETIRED,
        outgoing=out_anchor,
        incoming=in_anchor,
        active=gen2,
        activation_observed=True,
    )

    # 1-3. STEADY admission
    obs1 = state_steady.admits("out_ca_key")
    exp1 = ROTATION_AND_RELOAD_SECURITY_MATRIX[0][1]
    checks.append({"name": ROTATION_AND_RELOAD_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    obs2 = state_steady.admits("in_ca_key")
    exp2 = ROTATION_AND_RELOAD_SECURITY_MATRIX[1][1]
    checks.append({"name": ROTATION_AND_RELOAD_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    obs3 = state_steady.admits("unrelated_key")
    exp3 = ROTATION_AND_RELOAD_SECURITY_MATRIX[2][1]
    checks.append({"name": ROTATION_AND_RELOAD_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4-6. INCOMING_TRUSTED admission
    obs4 = state_trusted.admits("out_ca_key")
    exp4 = ROTATION_AND_RELOAD_SECURITY_MATRIX[3][1]
    checks.append({"name": ROTATION_AND_RELOAD_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    obs5 = state_trusted.admits("in_ca_key")
    exp5 = ROTATION_AND_RELOAD_SECURITY_MATRIX[4][1]
    checks.append({"name": ROTATION_AND_RELOAD_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    obs6 = state_trusted.admits("unrelated_key")
    exp6 = ROTATION_AND_RELOAD_SECURITY_MATRIX[5][1]
    checks.append({"name": ROTATION_AND_RELOAD_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7-9. INCOMING_ACTIVE admission
    obs7 = state_active_unobserved.admits("out_ca_key")
    exp7 = ROTATION_AND_RELOAD_SECURITY_MATRIX[6][1]
    checks.append({"name": ROTATION_AND_RELOAD_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    obs8 = state_active_unobserved.admits("in_ca_key")
    exp8 = ROTATION_AND_RELOAD_SECURITY_MATRIX[7][1]
    checks.append({"name": ROTATION_AND_RELOAD_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    obs9 = state_active_unobserved.admits("unrelated_key")
    exp9 = ROTATION_AND_RELOAD_SECURITY_MATRIX[8][1]
    checks.append({"name": ROTATION_AND_RELOAD_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10-12. OUTGOING_RETIRED admission
    obs10 = state_retired.admits("out_ca_key")
    exp10 = ROTATION_AND_RELOAD_SECURITY_MATRIX[9][1]
    checks.append({"name": ROTATION_AND_RELOAD_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    obs11 = state_retired.admits("in_ca_key")
    exp11 = ROTATION_AND_RELOAD_SECURITY_MATRIX[10][1]
    checks.append({"name": ROTATION_AND_RELOAD_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    obs12 = state_retired.admits("unrelated_key")
    exp12 = ROTATION_AND_RELOAD_SECURITY_MATRIX[11][1]
    checks.append({"name": ROTATION_AND_RELOAD_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13-14. Retirement guard
    next_unobserved = advance(state_active_unobserved)
    obs13 = next_unobserved.phase.value
    exp13 = ROTATION_AND_RELOAD_SECURITY_MATRIX[12][1]
    checks.append({"name": ROTATION_AND_RELOAD_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    next_observed = advance(state_active_observed)
    obs14 = next_observed.phase.value
    exp14 = ROTATION_AND_RELOAD_SECURITY_MATRIX[13][1]
    checks.append({"name": ROTATION_AND_RELOAD_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15-18. Mutual authentication required across all 4 phases
    obs15 = state_steady.requires_mutual_authentication()
    exp15 = ROTATION_AND_RELOAD_SECURITY_MATRIX[14][1]
    checks.append({"name": ROTATION_AND_RELOAD_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    obs16 = state_trusted.requires_mutual_authentication()
    exp16 = ROTATION_AND_RELOAD_SECURITY_MATRIX[15][1]
    checks.append({"name": ROTATION_AND_RELOAD_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    obs17 = state_active_unobserved.requires_mutual_authentication()
    exp17 = ROTATION_AND_RELOAD_SECURITY_MATRIX[16][1]
    checks.append({"name": ROTATION_AND_RELOAD_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    obs18 = state_retired.requires_mutual_authentication()
    exp18 = ROTATION_AND_RELOAD_SECURITY_MATRIX[17][1]
    checks.append({"name": ROTATION_AND_RELOAD_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    return {
        "case_id": "rotation-and-reload-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
