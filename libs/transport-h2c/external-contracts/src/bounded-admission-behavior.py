from __future__ import annotations

from transport_h2c.application.request import Admitted, AdmissionOutcome, admit
from transport_h2c.domain.endpoint import authority_of
from transport_h2c.infrastructure.config import ManagerConfig, admission_permits

MINIMUM_CHECKS = 15

BOUNDED_ADMISSION_BEHAVIOR_MATRIX = (
    ("an_ordinary_caller_is_admitted", ("admitted",)),
    ("a_shut_down_manager_refuses_immediately", ("refused", "Shutdown")),
    ("a_wait_past_the_deadline_is_a_timeout", ("refused", "Timeout")),
    ("a_wait_exactly_at_the_deadline_is_still_admitted", ("admitted",)),
    ("closed_admission_refuses_with_shutdown", ("refused", "Shutdown")),
    ("shutdown_outranks_an_expired_wait", ("refused", "Shutdown")),
    ("the_deadline_comes_from_the_configuration_not_a_constant", (("refused", "Timeout"), ("admitted",))),
    ("the_permit_count_is_the_configured_cap", 7),
    ("a_zero_cap_still_yields_one_permit", 1),
    ("a_negative_cap_still_yields_one_permit", 1),
    ("the_scheme_is_not_part_of_the_origin", "keep:7117"),
    ("a_trailing_slash_is_not_part_of_the_origin", "keep:7117"),
    ("three_spellings_of_one_peer_collapse_to_one_origin", ("keep:7117", "keep:7117", "keep:7117")),
    ("a_bare_authority_is_left_alone", "keep:7117"),
    ("an_https_endpoint_is_not_a_cleartext_origin", "https://keep:7117"),
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


def verify_bounded_admission_behavior() -> dict:
    checks = []

    # 1. an_ordinary_caller_is_admitted
    exp1 = BOUNDED_ADMISSION_BEHAVIOR_MATRIX[0][1]
    obs1 = _adm(
        admit(_cfg(), shut_down=False, admission_closed=False, waited_seconds=0.0)
    )
    checks.append(
        {
            "name": BOUNDED_ADMISSION_BEHAVIOR_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. a_shut_down_manager_refuses_immediately
    exp2 = BOUNDED_ADMISSION_BEHAVIOR_MATRIX[1][1]
    obs2 = _adm(
        admit(_cfg(), shut_down=True, admission_closed=False, waited_seconds=0.0)
    )
    checks.append(
        {
            "name": BOUNDED_ADMISSION_BEHAVIOR_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. a_wait_past_the_deadline_is_a_timeout
    exp3 = BOUNDED_ADMISSION_BEHAVIOR_MATRIX[2][1]
    obs3 = _adm(
        admit(
            _cfg(pool_timeout_seconds=5.0),
            shut_down=False,
            admission_closed=False,
            waited_seconds=5.5,
        )
    )
    checks.append(
        {
            "name": BOUNDED_ADMISSION_BEHAVIOR_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. a_wait_exactly_at_the_deadline_is_still_admitted
    exp4 = BOUNDED_ADMISSION_BEHAVIOR_MATRIX[3][1]
    obs4 = _adm(
        admit(
            _cfg(pool_timeout_seconds=5.0),
            shut_down=False,
            admission_closed=False,
            waited_seconds=5.0,
        )
    )
    checks.append(
        {
            "name": BOUNDED_ADMISSION_BEHAVIOR_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. closed_admission_refuses_with_shutdown
    exp5 = BOUNDED_ADMISSION_BEHAVIOR_MATRIX[4][1]
    obs5 = _adm(
        admit(_cfg(), shut_down=False, admission_closed=True, waited_seconds=0.0)
    )
    checks.append(
        {
            "name": BOUNDED_ADMISSION_BEHAVIOR_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. shutdown_outranks_an_expired_wait
    exp6 = BOUNDED_ADMISSION_BEHAVIOR_MATRIX[5][1]
    obs6 = _adm(
        admit(_cfg(), shut_down=True, admission_closed=False, waited_seconds=99.0)
    )
    checks.append(
        {
            "name": BOUNDED_ADMISSION_BEHAVIOR_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. the_deadline_comes_from_the_configuration_not_a_constant
    exp7 = BOUNDED_ADMISSION_BEHAVIOR_MATRIX[6][1]
    a7 = _adm(
        admit(
            _cfg(pool_timeout_seconds=1.0),
            shut_down=False,
            admission_closed=False,
            waited_seconds=2.0,
        )
    )
    b7 = _adm(
        admit(
            _cfg(pool_timeout_seconds=10.0),
            shut_down=False,
            admission_closed=False,
            waited_seconds=2.0,
        )
    )
    obs7 = (a7, b7)
    checks.append(
        {
            "name": BOUNDED_ADMISSION_BEHAVIOR_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. the_permit_count_is_the_configured_cap
    exp8 = BOUNDED_ADMISSION_BEHAVIOR_MATRIX[7][1]
    obs8 = admission_permits(_cfg(max_in_flight_per_origin=7))
    checks.append(
        {
            "name": BOUNDED_ADMISSION_BEHAVIOR_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. a_zero_cap_still_yields_one_permit
    exp9 = BOUNDED_ADMISSION_BEHAVIOR_MATRIX[8][1]
    obs9 = admission_permits(_cfg(max_in_flight_per_origin=0))
    checks.append(
        {
            "name": BOUNDED_ADMISSION_BEHAVIOR_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. a_negative_cap_still_yields_one_permit
    exp10 = BOUNDED_ADMISSION_BEHAVIOR_MATRIX[9][1]
    obs10 = admission_permits(_cfg(max_in_flight_per_origin=-5))
    checks.append(
        {
            "name": BOUNDED_ADMISSION_BEHAVIOR_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. the_scheme_is_not_part_of_the_origin
    exp11 = BOUNDED_ADMISSION_BEHAVIOR_MATRIX[10][1]
    obs11 = authority_of("http://keep:7117")
    checks.append(
        {
            "name": BOUNDED_ADMISSION_BEHAVIOR_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. a_trailing_slash_is_not_part_of_the_origin
    exp12 = BOUNDED_ADMISSION_BEHAVIOR_MATRIX[11][1]
    obs12 = authority_of("http://keep:7117/")
    checks.append(
        {
            "name": BOUNDED_ADMISSION_BEHAVIOR_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. three_spellings_of_one_peer_collapse_to_one_origin
    exp13 = BOUNDED_ADMISSION_BEHAVIOR_MATRIX[12][1]
    spellings13 = ("http://keep:7117", "http://keep:7117/", "keep:7117//")
    res13 = []
    for endpoint in spellings13:
        res13.append(authority_of(endpoint))
    obs13 = tuple(res13)
    checks.append(
        {
            "name": BOUNDED_ADMISSION_BEHAVIOR_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    # 14. a_bare_authority_is_left_alone
    exp14 = BOUNDED_ADMISSION_BEHAVIOR_MATRIX[13][1]
    obs14 = authority_of("keep:7117")
    checks.append(
        {
            "name": BOUNDED_ADMISSION_BEHAVIOR_MATRIX[13][0],
            "expected": exp14,
            "observed": obs14,
            "passed": obs14 == exp14,
        }
    )

    # 15. an_https_endpoint_is_not_a_cleartext_origin
    exp15 = BOUNDED_ADMISSION_BEHAVIOR_MATRIX[14][1]
    obs15 = authority_of("https://keep:7117")
    checks.append(
        {
            "name": BOUNDED_ADMISSION_BEHAVIOR_MATRIX[14][0],
            "expected": exp15,
            "observed": obs15,
            "passed": obs15 == exp15,
        }
    )

    return {
        "case_id": "bounded-admission-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
