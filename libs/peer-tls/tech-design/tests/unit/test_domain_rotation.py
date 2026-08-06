"""Unit tests for domain rotation state machine and generation tracking."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

SRC_ROOT = Path(__file__).resolve().parents[2] / "src"
sys.path.insert(0, str(SRC_ROOT))

from peer_tls.domain.material import TrustAnchor  # noqa: E402
from peer_tls.domain.rotation import Generation, RotationPhase, RotationState, advance, reload  # noqa: E402


class TestDomainRotation(unittest.TestCase):
    def setUp(self) -> None:
        self.anchor_old = TrustAnchor(key_id="ca_old", label="old")
        self.anchor_new = TrustAnchor(key_id="ca_new", label="new")
        self.gen1 = Generation(number=1, leaf_label="gen1")
        self.initial_state = RotationState(
            phase=RotationPhase.STEADY,
            outgoing=self.anchor_old,
            incoming=self.anchor_new,
            active=self.gen1,
            activation_observed=False,
        )

    def test_phase_walk_and_retirement_guard(self) -> None:
        s1 = advance(self.initial_state)
        self.assertEqual(s1.phase, RotationPhase.INCOMING_TRUSTED)

        s2 = advance(s1)
        self.assertEqual(s2.phase, RotationPhase.INCOMING_ACTIVE)

        # Transition to OUTGOING_RETIRED is refused if activation_observed is False
        s3_unobserved = advance(s2)
        self.assertEqual(s3_unobserved.phase, RotationPhase.INCOMING_ACTIVE)

        # Set activation_observed True
        s2_observed = RotationState(
            phase=s2.phase,
            outgoing=s2.outgoing,
            incoming=s2.incoming,
            active=s2.active,
            activation_observed=True,
        )
        s3 = advance(s2_observed)
        self.assertEqual(s3.phase, RotationPhase.OUTGOING_RETIRED)

        s4 = advance(s3)
        self.assertEqual(s4.phase, RotationPhase.OUTGOING_RETIRED)

    def test_anchor_admission_matrix(self) -> None:
        # STEADY
        self.assertTrue(self.initial_state.admits("ca_old"))
        self.assertTrue(self.initial_state.admits("ca_new"))
        self.assertFalse(self.initial_state.admits("ca_unknown"))

        # OUTGOING_RETIRED
        retired_state = RotationState(
            phase=RotationPhase.OUTGOING_RETIRED,
            outgoing=self.anchor_old,
            incoming=self.anchor_new,
            active=self.gen1,
            activation_observed=True,
        )
        self.assertFalse(retired_state.admits("ca_old"))
        self.assertTrue(retired_state.admits("ca_new"))

    def test_monotonic_generation_on_reload(self) -> None:
        gen_same = Generation(number=1, leaf_label="gen1")
        reloaded1 = reload(self.initial_state, gen_same)
        self.assertEqual(reloaded1.active.number, 2)

        reloaded2 = reload(reloaded1, gen_same)
        self.assertEqual(reloaded2.active.number, 3)

    def test_mutual_authentication_invariant(self) -> None:
        self.assertTrue(self.initial_state.requires_mutual_authentication())


if __name__ == "__main__":
    unittest.main()
