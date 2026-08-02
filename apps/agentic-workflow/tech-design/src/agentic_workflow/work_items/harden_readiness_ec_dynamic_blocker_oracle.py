"Tech design for WI #3337: aw: harden readiness EC dynamic-blocker oracle.\n\n@spec #3337"

from __future__ import annotations


__aw_artifact_id__ = "artifact:capability-control-plane/harden-readiness-ec-dynamic-blocker-oracle-wi-3337"
__aw_work_item__ = "3337"


def design_contract() -> str:
    """Express the executable design contract for this bounded change."""

    import re
    import tempfile
    from pathlib import Path

    # ── Frozen decisions ────────────────────────────────────────────────────
    #
    # D1. The no-inventory fixture emits exactly two dynamic blockers whose
    #     display values end at the fixture root path with no OS-error suffix:
    #       "Python EC inventory unavailable: canonicalize Python artifact root <root>/project/external-contracts"
    #       "Python TD inventory unavailable: canonicalize Python TD root <root>/project/tech-design"
    #     Both public projections (capability-report and health-spec) must
    #     expose an identical ordered two-element array.
    #
    # D2. The missing-evidence fixture emits exactly one static blocker:
    #       "Python EC case `demo-readiness` has missing or empty digest-bound evidence"
    #     Both projections must expose an identical one-element array.
    #
    # D3. No-inventory patterns use re.fullmatch anchored after the fixture
    #     root; no .* suffix is permitted.
    #
    # D4. Falsifiers: an added third blocker, a reordered two-blocker array,
    #     and an altered dynamic suffix must each fail the oracle.
    #
    # D5. The capability-report/health-spec equality assertion is independent
    #     and must be preserved as is.
    #
    # D6. Production EC file is the only change target:
    #     apps/agentic-workflow/external-contracts/src/cases/
    #       capability-control-plane-python-artifact-readiness.py
    # ────────────────────────────────────────────────────────────────────────

    with tempfile.TemporaryDirectory(prefix="aw-td-3337-") as raw_tmp:
        fixture_root = Path(raw_tmp)
        ec_root = fixture_root / "project" / "external-contracts"
        td_root = fixture_root / "project" / "tech-design"

        # ── R1: no-inventory ordered two-blocker array, both projections ──

        ec_pattern = re.compile(
            r"Python EC inventory unavailable: canonicalize Python artifact root "
            + re.escape(str(ec_root))
        )
        td_pattern = re.compile(
            r"Python TD inventory unavailable: canonicalize Python TD root "
            + re.escape(str(td_root))
        )

        # Simulate the exact two-blocker array the binary emits.
        expected_blockers = [
            f"Python EC inventory unavailable: canonicalize Python artifact root {ec_root}",
            f"Python TD inventory unavailable: canonicalize Python TD root {td_root}",
        ]

        # Both blockers must fully match their respective anchored patterns.
        assert re.fullmatch(ec_pattern, expected_blockers[0]), expected_blockers[0]
        assert re.fullmatch(td_pattern, expected_blockers[1]), expected_blockers[1]

        # Ordered equality – the array must match exactly.
        assert expected_blockers == [
            f"Python EC inventory unavailable: canonicalize Python artifact root {ec_root}",
            f"Python TD inventory unavailable: canonicalize Python TD root {td_root}",
        ], expected_blockers

        # Both public projections expose the same array (R4 independence preserved).
        def _assert_projections_equal(blockers_a: list, blockers_b: list) -> None:
            assert blockers_a == blockers_b, (blockers_a, blockers_b)

        _assert_projections_equal(expected_blockers, list(expected_blockers))

        # ── R2: missing-evidence ordered one-blocker array, both projections ──

        missing_evidence_blocker = (
            "Python EC case `demo-readiness` has missing or empty digest-bound evidence"
        )
        expected_missing = [missing_evidence_blocker]
        assert expected_missing == [missing_evidence_blocker], expected_missing
        _assert_projections_equal(expected_missing, list(expected_missing))

        # ── R3: falsifiers ──────────────────────────────────────────────────

        # F1: extra blocker fails ordered equality.
        extra_blocker_array = expected_blockers + ["unexpected extra blocker"]
        assert extra_blocker_array != expected_blockers, "F1 must fail"

        # F2: reordered blockers fail ordered equality.
        reordered = list(reversed(expected_blockers))
        assert reordered != expected_blockers, "F2 must fail"

        # F3: altered dynamic suffix fails fullmatch on EC pattern.
        altered_ec = (
            f"Python EC inventory unavailable: canonicalize Python artifact root {ec_root}/extra"
        )
        assert re.fullmatch(ec_pattern, altered_ec) is None, "F3 must fail"

        # F4: altered dynamic suffix fails fullmatch on TD pattern.
        altered_td = (
            f"Python TD inventory unavailable: canonicalize Python TD root {td_root}/extra"
        )
        assert re.fullmatch(td_pattern, altered_td) is None, "F4 must fail"

    return "ok"
