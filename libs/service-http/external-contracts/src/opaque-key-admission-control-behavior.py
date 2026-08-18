from __future__ import annotations

from service_http.application.admission import AdmissionLedger, decision_event
from service_http.domain.admission import AdmissionPolicy, DEFAULT_MAX_KEYS, DEFAULT_REFILL_SECS, Decision, Event, Outcome, default_refill_window_ns, max_credits, observed_fields, request_cost

MINIMUM_CHECKS = 15

OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX = (
    ("an_unconfigured_class_bypasses_and_allocates_nothing",
     (('bypass', None), 0, 0, 0)),
    ("one_unit_of_capacity_admits_once_then_denies",
     (('allow', None), ('deny', 5000000000))),
    ("the_deficit_names_the_instant_the_next_request_is_admitted",
     (('allow', None), 5000000000, ('allow', None))),
    ("capacity_two_admits_twice_before_denying",
     ('allow', 'allow', 'deny', 2500000000)),
    ("distinct_fingerprints_hold_distinct_buckets",
     ('allow', 'allow', 2, 2)),
    ("each_class_counts_its_own_keys",
     ('allow', 'allow', 1, 1, 2)),
    ("a_clock_that_runs_backwards_grants_no_credit",
     ('allow', 'deny', 5000000000)),
    ("a_key_idle_for_many_windows_banks_no_more_than_its_ceiling",
     ('allow', 'allow', 'deny', 'allow', 'allow', 'deny')),
    ("credit_comes_back_at_the_configured_rate_not_a_flat_one",
     ('allow', 'allow', 'allow', 'allow', 'deny')),
    ("the_bucket_arithmetic_is_window_scaled",
     (5000000000, 5000000000, 15000000000, 5000000000)),
    ("the_default_window_is_the_documented_one",
     (60, 1024, 60000000000)),
    ("a_decision_becomes_an_observable_event_in_milliseconds",
     (('write', 'deny', 5000), ('write', 'allow', None), ('read', 'bypass', None))),
    ("the_event_body_omits_the_retry_field_when_there_is_none",
     ({'class': 'write', 'outcome': 'deny', 'retryAfterMs': 5000}, {'class': 'write', 'outcome': 'allow'}, {'class': 'read', 'outcome': 'bypass'})),
    ("the_three_outcomes_are_distinct_named_values",
     ('allow', 'deny', 'bypass', 3)),
    ("the_published_event_fields_are_wire_primitives_not_enum_members",
     ('str', 'str', 'int')),
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


WINDOW = 5_000_000_000


def verify_opaque_key_admission_control_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. an unconfigured class bypasses and allocates nothing
    exp1 = OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[0][1]
    bypass = AdmissionLedger({})
    obs1 = plain((bypass.admit_at("read", "k", 0), bypass.total_keys(),
        bypass.tracked_keys("read"), bypass.sequence()))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. one unit of capacity admits once then denies
    exp2 = OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[1][1]
    single = AdmissionLedger({"write": AdmissionPolicy(1, WINDOW, 8)})
    obs2 = plain((single.admit_at("write", "k", 0), single.admit_at("write", "k", 0)))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the deficit names the instant the next request is admitted
    exp3 = OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[2][1]
    refilled = AdmissionLedger({"write": AdmissionPolicy(1, WINDOW, 8)})
    obs3 = plain((refilled.admit_at("write", "k", 0),
        refilled.admit_at("write", "k", 0).retry_after_ns,
        refilled.admit_at("write", "k", WINDOW)))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. capacity two admits twice before denying
    exp4 = OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[3][1]
    paced = AdmissionLedger({"write": AdmissionPolicy(2, WINDOW, 8)})
    obs4 = plain((paced.admit_at("write", "k", 0).outcome,
        paced.admit_at("write", "k", 0).outcome,
        paced.admit_at("write", "k", 0).outcome,
        paced.admit_at("write", "k", 0).retry_after_ns))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. distinct fingerprints hold distinct buckets
    exp5 = OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[4][1]
    keyed = AdmissionLedger({"write": AdmissionPolicy(1, WINDOW, 8)})
    obs5 = plain((keyed.admit_at("write", "a", 0).outcome,
        keyed.admit_at("write", "b", 0).outcome,
        keyed.tracked_keys("write"), keyed.total_keys()))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. each class counts its own keys
    exp6 = OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[5][1]
    classed = AdmissionLedger({"read": AdmissionPolicy(1, WINDOW, 8),
        "write": AdmissionPolicy(1, WINDOW, 8)})
    obs6 = plain((classed.admit_at("read", "k", 0).outcome,
        classed.admit_at("write", "k", 0).outcome,
        classed.tracked_keys("read"), classed.tracked_keys("write"),
        classed.total_keys()))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. a clock that runs backwards grants no credit
    exp7 = OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[6][1]
    backwards = AdmissionLedger({"write": AdmissionPolicy(1, WINDOW, 8)})
    obs7 = plain((backwards.admit_at("write", "k", WINDOW).outcome,
        backwards.admit_at("write", "k", 0).outcome,
        backwards.admit_at("write", "k", 0).retry_after_ns))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a key idle for many windows banks no more than its ceiling
    exp8 = OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[7][1]
    banked = AdmissionLedger({"write": AdmissionPolicy(2, WINDOW, 8)})
    obs8 = plain((banked.admit_at("write", "k", 0).outcome,
        banked.admit_at("write", "k", 0).outcome,
        banked.admit_at("write", "k", 0).outcome,
        banked.admit_at("write", "k", 10 * WINDOW).outcome,
        banked.admit_at("write", "k", 10 * WINDOW).outcome,
        banked.admit_at("write", "k", 10 * WINDOW).outcome))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. credit comes back at the configured rate not a flat one
    exp9 = OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[8][1]
    accruing = AdmissionLedger({"write": AdmissionPolicy(2, WINDOW, 8)})
    obs9 = plain((accruing.admit_at("write", "k", 0).outcome,
        accruing.admit_at("write", "k", 0).outcome,
        accruing.admit_at("write", "k", WINDOW).outcome,
        accruing.admit_at("write", "k", WINDOW).outcome,
        accruing.admit_at("write", "k", WINDOW).outcome))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the bucket arithmetic is window scaled
    exp10 = OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[9][1]
    obs10 = plain((max_credits(AdmissionPolicy(1, WINDOW, 8)),
        request_cost(AdmissionPolicy(1, WINDOW, 8)),
        max_credits(AdmissionPolicy(3, WINDOW, 8)),
        request_cost(AdmissionPolicy(3, WINDOW, 8))))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. the default window is the documented one
    exp11 = OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[10][1]
    obs11 = plain((DEFAULT_REFILL_SECS, DEFAULT_MAX_KEYS, default_refill_window_ns()))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. a decision becomes an observable event in milliseconds
    exp12 = OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[11][1]
    obs12 = plain((decision_event("write", Decision(Outcome.DENY, 5_000_000_000)),
        decision_event("write", Decision(Outcome.ALLOW, None)),
        decision_event("read", Decision(Outcome.BYPASS, None))))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. the event body omits the retry field when there is none
    exp13 = OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[12][1]
    obs13 = plain((observed_fields(Event("write", Outcome.DENY, 5000)),
        observed_fields(Event("write", Outcome.ALLOW, None)),
        observed_fields(Event("read", Outcome.BYPASS, None))))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    # 14. the three outcomes are distinct named values
    exp14 = OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[13][1]
    obs14 = plain((Outcome.ALLOW.value, Outcome.DENY.value, Outcome.BYPASS.value,
        len({Outcome.ALLOW, Outcome.DENY, Outcome.BYPASS})))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[13][0], "expected": exp14,
                   "observed": obs14, "passed": obs14 == exp14})

    # 15. the published event fields are wire primitives not enum members
    exp15 = OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[14][1]
    obs15 = plain((type(observed_fields(
        Event("write", Outcome.DENY, 5000)).get("outcome")).__name__,
        type(observed_fields(
        Event("write", Outcome.DENY, 5000)).get("class")).__name__,
        type(observed_fields(
        Event("write", Outcome.DENY, 5000)).get("retryAfterMs")).__name__))
    checks.append({"name": OPAQUE_KEY_ADMISSION_CONTROL_BEHAVIOR_MATRIX[14][0], "expected": exp15,
                   "observed": obs15, "passed": obs15 == exp15})

    return {
        "case_id": "opaque-key-admission-control-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
