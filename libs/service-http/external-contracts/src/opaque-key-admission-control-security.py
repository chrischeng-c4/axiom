from __future__ import annotations

from service_http.application.admission import AdmissionLedger, decision_event
from service_http.domain.admission import AdmissionPolicy, Decision, Event, Outcome, is_valid_policy, observed_fields, policy_problem
from service_http.infrastructure.headers import RETRY_AFTER_HEADER, retry_after_seconds, retry_after_value

MINIMUM_CHECKS = 15

OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX = (
    ("the_observed_event_carries_no_key_material",
     ({'class': 'write', 'outcome': 'deny', 'retryAfterMs': 5000}, ('class', 'outcome', 'retryAfterMs'), False)),
    ("the_event_type_has_no_field_that_could_hold_a_key",
     ('write', False, ('class', 'outcome'))),
    ("the_key_ledger_is_bounded_by_eviction",
     ('allow', 'allow', 'allow', 2, 2)),
    ("eviction_drops_the_least_recently_seen_key_and_spares_the_rest",
     ('allow', 'allow', 'deny', 'allow', 'deny', 'allow', 2)),
    ("a_key_flood_saturates_the_ledger_instead_of_growing_it",
     ('allow', 1, 'allow', 2, 'allow', 2, 'allow', 2)),
    ("an_unconfigured_class_allocates_nothing_beside_a_configured_one",
     (('bypass', None), 1, 1, 0, 1)),
    ("the_named_wait_counts_down_as_the_bucket_refills",
     ('allow', 5000000000, 2500000000, 1)),
    ("the_named_wait_rounds_up_when_the_deficit_does_not_divide_evenly",
     ('allow', 'allow', 'allow', 1666666667, 2, True)),
    ("eviction_takes_its_victim_from_the_crowded_class_alone",
     ('allow', 2, 2, 4, 'deny')),
    ("the_retry_header_rounds_up_and_never_falls_below_one_second",
     (5, 2, 2, 1, 1, 1)),
    ("the_header_the_wait_is_advertised_in_is_lower_case",
     ('retry-after', True, '2')),
    ("the_retry_header_renders_as_a_decimal_string",
     ('5', '1', '1', '2')),
    ("a_policy_with_a_non_positive_field_is_refused",
     ('capacity must be positive', 'refill window must be positive', 'max keys must be positive', None)),
    ("the_validity_test_reads_the_same_way",
     (True, False, False)),
    ("admission_is_total_and_still_answers",
     ('accepted', 'accepted', 'accepted', 1, 'accepted', {'class': 'w', 'outcome': 'deny'})),
)


def plain(value: object) -> object:
    """A literal-shaped view: records by their fields, enum members by value.

    An expected value has to be a plain literal, and `repr` of a dataclass or
    an enum member is not one. Reading a record as the tuple of its fields
    keeps every field observable while staying transcribable.
    """
    fields = getattr(type(value), "__dataclass_fields__", None)
    if fields is not None:
        return tuple(plain(getattr(value, n)) for n in fields)
    if getattr(type(value), "__members__", None) is not None:
        return plain(value.value)
    if isinstance(value, tuple):
        return tuple(plain(v) for v in value)
    if isinstance(value, list):
        return [plain(v) for v in value]
    if isinstance(value, dict):
        return {k: plain(v) for k, v in value.items()}
    return value


def refusal(fn, *args, **kwargs) -> str:
    """The exception type a refusing call raises, or 'accepted' if it returns."""
    try:
        fn(*args, **kwargs)
    except Exception as exc:  # noqa: BLE001 — the type is the observation
        return type(exc).__name__
    return "accepted"


WINDOW = 5_000_000_000


def verify_opaque_key_admission_control_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. the observed event carries no key material
    exp1 = OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[0][1]
    obs1 = plain((observed_fields(Event("write", Outcome.DENY, 5000)),
        tuple(observed_fields(Event("write", Outcome.DENY, 5000)).keys()),
        "fingerprint" in observed_fields(Event("write", Outcome.DENY, 5000))))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. the event type has no field that could hold a key
    exp2 = OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[1][1]
    obs2 = plain((decision_event("write", Decision(Outcome.DENY, 1)).route_class,
        "secret" in str(decision_event(
        "write", Decision(Outcome.DENY, 1))),
        tuple(observed_fields(decision_event(
        "write", Decision(Outcome.ALLOW, None))).keys())))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the key ledger is bounded by eviction
    exp3 = OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[2][1]
    evicting = AdmissionLedger({"write": AdmissionPolicy(1, WINDOW, 2)})
    obs3 = plain((evicting.admit_at("write", "a", 0).outcome,
        evicting.admit_at("write", "b", 0).outcome,
        evicting.admit_at("write", "c", 0).outcome,
        evicting.tracked_keys("write"), evicting.total_keys()))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. eviction drops the least recently seen key and spares the rest
    exp4 = OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[3][1]
    ordered = AdmissionLedger({"write": AdmissionPolicy(1, WINDOW, 2)})
    obs4 = plain((ordered.admit_at("write", "a", 0).outcome,
        ordered.admit_at("write", "b", 0).outcome,
        ordered.admit_at("write", "a", 1).outcome,
        ordered.admit_at("write", "c", 1).outcome,
        ordered.admit_at("write", "a", 1).outcome,
        ordered.admit_at("write", "b", 1).outcome,
        ordered.tracked_keys("write")))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. a key flood saturates the ledger instead of growing it
    exp5 = OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[4][1]
    spread = AdmissionLedger({"write": AdmissionPolicy(1, WINDOW, 2)})
    obs5 = plain((spread.admit_at("write", "a", 0).outcome, spread.tracked_keys("write"),
        spread.admit_at("write", "b", 0).outcome, spread.tracked_keys("write"),
        spread.admit_at("write", "c", 0).outcome, spread.tracked_keys("write"),
        spread.admit_at("write", "d", 0).outcome, spread.tracked_keys("write")))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. an unconfigured class allocates nothing beside a configured one
    exp6 = OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[5][1]
    unconfigured = AdmissionLedger({"write": AdmissionPolicy(1, WINDOW, 2)})
    unconfigured.admit_at("write", "k", 0)
    obs6 = plain((unconfigured.admit_at("read", "k", 0),
        unconfigured.total_keys(), unconfigured.sequence(),
        unconfigured.tracked_keys("read"), unconfigured.tracked_keys("write")))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. the named wait counts down as the bucket refills
    exp7 = OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[6][1]
    countdown = AdmissionLedger({"write": AdmissionPolicy(1, WINDOW, 8)})
    obs7 = plain((countdown.admit_at("write", "k", 0).outcome,
        countdown.admit_at("write", "k", 0).retry_after_ns,
        countdown.admit_at("write", "k", WINDOW // 2).retry_after_ns,
        countdown.admit_at("write", "k", WINDOW - 1).retry_after_ns))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. the named wait rounds up when the deficit does not divide evenly
    exp8 = OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[7][1]
    uneven = AdmissionLedger({"write": AdmissionPolicy(3, WINDOW, 8)})
    obs8 = plain((uneven.admit_at("write", "k", 0).outcome,
        uneven.admit_at("write", "k", 0).outcome,
        uneven.admit_at("write", "k", 0).outcome,
        uneven.admit_at("write", "k", 0).retry_after_ns,
        WINDOW % 3, uneven.admit_at("write", "k", 0).retry_after_ns * 3
        >= WINDOW))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. eviction takes its victim from the crowded class alone
    exp9 = OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[8][1]
    crossing = AdmissionLedger({"read": AdmissionPolicy(1, WINDOW, 2),
        "write": AdmissionPolicy(1, WINDOW, 2)})
    crossing.admit_at("read", "a", 0)
    crossing.admit_at("read", "b", 0)
    crossing.admit_at("write", "a", 0)
    crossing.admit_at("write", "b", 0)
    obs9 = plain((crossing.admit_at("write", "c", 0).outcome,
        crossing.tracked_keys("read"), crossing.tracked_keys("write"),
        crossing.total_keys(),
        crossing.admit_at("read", "a", 0).outcome))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the retry header rounds up and never falls below one second
    exp10 = OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[9][1]
    obs10 = plain((retry_after_seconds(5_000_000_000), retry_after_seconds(1_500_000_000),
        retry_after_seconds(1_000_000_001), retry_after_seconds(1),
        retry_after_seconds(0), retry_after_seconds(None)))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. the header the wait is advertised in is lower case
    exp11 = OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[10][1]
    obs11 = plain((RETRY_AFTER_HEADER, RETRY_AFTER_HEADER == RETRY_AFTER_HEADER.lower(),
        retry_after_value(1_000_000_001)))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. the retry header renders as a decimal string
    exp12 = OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[11][1]
    obs12 = plain((retry_after_value(5_000_000_000), retry_after_value(None),
        retry_after_value(0), retry_after_value(2_000_000_000)))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. a policy with a non positive field is refused
    exp13 = OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[12][1]
    obs13 = plain((policy_problem(AdmissionPolicy(0, WINDOW, 8)),
        policy_problem(AdmissionPolicy(1, 0, 8)),
        policy_problem(AdmissionPolicy(1, WINDOW, 0)),
        policy_problem(AdmissionPolicy(1, WINDOW, 8))))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    # 14. the validity test reads the same way
    exp14 = OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[13][1]
    obs14 = plain((is_valid_policy(AdmissionPolicy(1, WINDOW, 8)),
        is_valid_policy(AdmissionPolicy(0, WINDOW, 8)),
        is_valid_policy(AdmissionPolicy(-1, -1, -1))))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[13][0], "expected": exp14,
                   "observed": obs14, "passed": obs14 == exp14})

    # 15. admission is total and still answers
    exp15 = OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[14][1]
    admitting = AdmissionLedger({"write": AdmissionPolicy(1, WINDOW, 8)})
    obs15 = plain((refusal(admitting.admit_at, "write", "", 0),
        refusal(admitting.admit_at, "read", "k", -1),
        refusal(retry_after_seconds, None), retry_after_seconds(None),
        refusal(observed_fields, Event("w", Outcome.DENY, None)),
        observed_fields(Event("w", Outcome.DENY, None))))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_SECURITY_MATRIX[14][0], "expected": exp15,
                   "observed": obs15, "passed": obs15 == exp15})

    return {
        "case_id": "opaque-key-admission-control-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
