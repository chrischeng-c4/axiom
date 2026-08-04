from __future__ import annotations

from transport_h2c.application.request import Admitted, AdmissionOutcome, admit
from transport_h2c.domain.endpoint import authority_of
from transport_h2c.infrastructure.config import (
    ManagerConfig,
    admission_permits,
    default_config,
    for_concurrency,
)

MINIMUM_CHECKS = 12

BOUNDED_ADMISSION_SECURITY_MATRIX = (
    ("a_drained_manager_and_a_busy_one_are_distinguishable", (("refused", "Shutdown"), ("refused", "Timeout"))),
    ("no_wait_however_long_is_ever_admitted_past_the_deadline", (("refused", "Timeout"), ("refused", "Timeout"), ("refused", "Timeout"), ("refused", "Timeout"))),
    ("a_shut_down_manager_refuses_at_every_wait_length", (("refused", "Shutdown"), ("refused", "Shutdown"), ("refused", "Shutdown"))),
    ("closed_admission_cannot_be_bypassed_by_arriving_early", ("refused", "Shutdown")),
    ("the_permit_count_is_never_zero_at_any_configured_cap", (1, 1, 1, 2)),
    ("a_target_concurrency_reaches_the_admission_cap", 4),
    ("a_target_concurrency_also_moves_the_connection_ceiling", 2),
    ("the_default_manager_must_hold_at_least_one_connection", 1),
    ("a_zero_target_concurrency_still_admits_one_request", 1),
    ("two_spellings_of_one_peer_share_one_admission_key", ("keep:7117", "keep:7117")),
    ("a_path_is_not_folded_away_into_the_bare_authority", "keep:7117/v1"),
    ("a_doubled_scheme_is_not_unwrapped_twice", "http://keep:7117"),
)


def _cfg(
    max_in_flight_per_origin: int = 128,
    pool_timeout_seconds: float = 5.0,
) -> ManagerConfig:
    return ManagerConfig(
        min_connections=1,
        max_connections=4,
        max_keepalive_connections=16,
        max_in_flight_per_origin=max_in_flight_per_origin,
        grow_threshold=32,
        pool_timeout_seconds=pool_timeout_seconds,
        connect_timeout_seconds=5.0,
        request_timeout_seconds=30.0,
        ping_interval_seconds=15.0,
        idle_timeout_seconds=5.0,
        stream_window_bytes=1048576,
        conn_window_bytes=4194304,
        max_frame_bytes=16384,
    )


def _adm(outcome: AdmissionOutcome) -> tuple:
    if isinstance(outcome, Admitted):
        return ("admitted",)
    return ("refused", type(outcome.error).__name__)


def verify_bounded_admission_security() -> dict:
    checks = []

    # 1. a_drained_manager_and_a_busy_one_are_distinguishable
    exp1 = BOUNDED_ADMISSION_SECURITY_MATRIX[0][1]
    drained1 = _adm(
        admit(_cfg(), shut_down=True, admission_closed=False, waited_seconds=0.0)
    )
    busy1 = _adm(
        admit(_cfg(), shut_down=False, admission_closed=False, waited_seconds=99.0)
    )
    obs1 = (drained1, busy1)
    checks.append(
        {
            "name": BOUNDED_ADMISSION_SECURITY_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. no_wait_however_long_is_ever_admitted_past_the_deadline
    exp2 = BOUNDED_ADMISSION_SECURITY_MATRIX[1][1]
    waits2 = (5.5, 6.0, 60.0, 3600.0)
    res2 = []
    for waited in waits2:
        res2.append(
            _adm(
                admit(
                    _cfg(pool_timeout_seconds=5.0),
                    shut_down=False,
                    admission_closed=False,
                    waited_seconds=waited,
                )
            )
        )
    obs2 = tuple(res2)
    checks.append(
        {
            "name": BOUNDED_ADMISSION_SECURITY_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. a_shut_down_manager_refuses_at_every_wait_length
    exp3 = BOUNDED_ADMISSION_SECURITY_MATRIX[2][1]
    waits3 = (0.0, 1.0, 99.0)
    res3 = []
    for waited in waits3:
        res3.append(
            _adm(
                admit(
                    _cfg(),
                    shut_down=True,
                    admission_closed=False,
                    waited_seconds=waited,
                )
            )
        )
    obs3 = tuple(res3)
    checks.append(
        {
            "name": BOUNDED_ADMISSION_SECURITY_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. closed_admission_cannot_be_bypassed_by_arriving_early
    exp4 = BOUNDED_ADMISSION_SECURITY_MATRIX[3][1]
    obs4 = _adm(
        admit(_cfg(), shut_down=False, admission_closed=True, waited_seconds=0.0)
    )
    checks.append(
        {
            "name": BOUNDED_ADMISSION_SECURITY_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. the_permit_count_is_never_zero_at_any_configured_cap
    exp5 = BOUNDED_ADMISSION_SECURITY_MATRIX[4][1]
    caps5 = (-1, 0, 1, 2)
    res5 = []
    for cap in caps5:
        res5.append(admission_permits(_cfg(max_in_flight_per_origin=cap)))
    obs5 = tuple(res5)
    checks.append(
        {
            "name": BOUNDED_ADMISSION_SECURITY_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. a_target_concurrency_reaches_the_admission_cap
    exp6 = BOUNDED_ADMISSION_SECURITY_MATRIX[5][1]
    obs6 = for_concurrency(4, 64).max_in_flight_per_origin
    checks.append(
        {
            "name": BOUNDED_ADMISSION_SECURITY_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. a_target_concurrency_also_moves_the_connection_ceiling
    exp7 = BOUNDED_ADMISSION_SECURITY_MATRIX[6][1]
    obs7 = for_concurrency(4, 64).max_connections
    checks.append(
        {
            "name": BOUNDED_ADMISSION_SECURITY_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. the_default_manager_must_hold_at_least_one_connection
    exp8 = BOUNDED_ADMISSION_SECURITY_MATRIX[7][1]
    obs8 = default_config(64).min_connections
    checks.append(
        {
            "name": BOUNDED_ADMISSION_SECURITY_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. a_zero_target_concurrency_still_admits_one_request
    exp9 = BOUNDED_ADMISSION_SECURITY_MATRIX[8][1]
    obs9 = for_concurrency(0, 64).max_in_flight_per_origin
    checks.append(
        {
            "name": BOUNDED_ADMISSION_SECURITY_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. two_spellings_of_one_peer_share_one_admission_key
    exp10 = BOUNDED_ADMISSION_SECURITY_MATRIX[9][1]
    obs10 = (authority_of("http://keep:7117"), authority_of("http://keep:7117/"))
    checks.append(
        {
            "name": BOUNDED_ADMISSION_SECURITY_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. a_path_is_not_folded_away_into_the_bare_authority
    exp11 = BOUNDED_ADMISSION_SECURITY_MATRIX[10][1]
    obs11 = authority_of("http://keep:7117/v1")
    checks.append(
        {
            "name": BOUNDED_ADMISSION_SECURITY_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. a_doubled_scheme_is_not_unwrapped_twice
    exp12 = BOUNDED_ADMISSION_SECURITY_MATRIX[11][1]
    obs12 = authority_of("http://http://keep:7117")
    checks.append(
        {
            "name": BOUNDED_ADMISSION_SECURITY_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    return {
        "case_id": "bounded-admission-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
