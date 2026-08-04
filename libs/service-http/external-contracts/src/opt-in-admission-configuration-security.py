from __future__ import annotations

from service_http.application.admission_config import from_lookup
from service_http.domain.admission import AdmissionPolicy
from service_http.domain.errors import InvalidPolicy, InvalidValue, OrphanedCommonSetting, describe
from service_http.infrastructure.env import all_keys, env_key
from service_http.infrastructure.numbers import parse_ascii_unsigned, parse_positive

MINIMUM_CHECKS = 12

OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX = (
    ("an_unreadable_value_is_refused_and_names_its_own_key",
     ('InvalidValue', 'LUMEN_ADMISSION_READ_CAPACITY', 'abc', 'LUMEN_ADMISSION_READ_CAPACITY must be a positive integer, got `abc`')),
    ("zero_is_not_a_positive_capacity",
     ('InvalidValue', None, 1, 0)),
    ("a_signed_spaced_or_non_ascii_value_is_refused",
     (None, None, None, None, None, 1)),
    ("a_tuning_knob_with_no_capacity_behind_it_is_refused",
     ('OrphanedCommonSetting', 'LUMEN_ADMISSION_REFILL_SECS', 'LUMEN_ADMISSION_REFILL_SECS is set but no admission capacity is configured; set at least one capacity key or remove LUMEN_ADMISSION_REFILL_SECS')),
    ("the_second_tuning_knob_is_refused_the_same_way",
     ('OrphanedCommonSetting', 'LUMEN_ADMISSION_MAX_KEYS', 'LUMEN_ADMISSION_MAX_KEYS is set but no admission capacity is configured; set at least one capacity key or remove LUMEN_ADMISSION_MAX_KEYS')),
    ("a_tuning_knob_beside_a_capacity_is_accepted",
     ('AdmissionConfig', 'AdmissionConfig')),
    ("the_value_check_covers_every_key_not_only_the_capacities",
     ('InvalidValue', 'LUMEN_ADMISSION_MAX_KEYS')),
    ("an_invalid_policy_names_the_class_and_the_reason",
     ('admission policy for class `write` is invalid: capacity must be positive', 'InvalidPolicy', 'K must be a positive integer, got `v`', 'K is set but no admission capacity is configured; set at least one capacity key or remove K')),
    ("a_refusal_is_returned_rather_than_raised",
     ('accepted', 'accepted', 'accepted', 'AdmissionConfig')),
    ("an_unknown_error_value_has_no_sentence",
     ('TypeError', 'TypeError', 'accepted')),
    ("the_prefix_is_carried_verbatim_into_every_key",
     ('_ADMISSION_READ_CAPACITY', 'A_B_ADMISSION_MAX_KEYS', '_ADMISSION_READ_CAPACITY', 5)),
    ("a_config_is_immutable",
     ('FrozenInstanceError', 'FrozenInstanceError', 'FrozenInstanceError')),
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


def variant(value: object) -> str:
    """The name of the returned variant — the shape of an error-as-value."""
    return type(value).__name__


def refusal(fn, *args, **kwargs) -> str:
    """The exception type a refusing call raises, or 'accepted' if it returns."""
    try:
        fn(*args, **kwargs)
    except Exception as exc:  # noqa: BLE001 — the type is the observation
        return type(exc).__name__
    return "accepted"


WINDOW = 5_000_000_000


def verify_opt_in_admission_configuration_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. an unreadable value is refused and names its own key
    exp1 = OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[0][1]
    bad = from_lookup("LUMEN", {"LUMEN_ADMISSION_READ_CAPACITY": "abc"}.get)
    obs1 = plain((variant(bad), bad.key, bad.value, describe(bad)))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. zero is not a positive capacity
    exp2 = OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[1][1]
    obs2 = plain((variant(from_lookup("LUMEN",
        {"LUMEN_ADMISSION_READ_CAPACITY": "0"}.get)),
        parse_positive("0"), parse_positive("1"), parse_ascii_unsigned("0")))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. a signed spaced or non ascii value is refused
    exp3 = OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[2][1]
    obs3 = plain((parse_positive("-1"), parse_positive(" 1"), parse_positive("1 "),
        parse_positive("１"), parse_positive("1.0"), parse_positive("1")))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. a tuning knob with no capacity behind it is refused
    exp4 = OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[3][1]
    orphan = from_lookup("LUMEN", {"LUMEN_ADMISSION_REFILL_SECS": "5"}.get)
    obs4 = plain((variant(orphan), orphan.key, describe(orphan)))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the second tuning knob is refused the same way
    exp5 = OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[4][1]
    orphan_keys = from_lookup("LUMEN", {"LUMEN_ADMISSION_MAX_KEYS": "8"}.get)
    obs5 = plain((variant(orphan_keys), orphan_keys.key, describe(orphan_keys)))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a tuning knob beside a capacity is accepted
    exp6 = OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[5][1]
    obs6 = plain((variant(from_lookup("LUMEN", {
        "LUMEN_ADMISSION_READ_CAPACITY": "1",
        "LUMEN_ADMISSION_REFILL_SECS": "5",
        }.get)),
        variant(from_lookup("LUMEN", {
        "LUMEN_ADMISSION_MAX_KEYS": "8",
        "LUMEN_ADMISSION_ADMIN_CAPACITY": "1",
        }.get))))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. the value check covers every key not only the capacities
    exp7 = OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[6][1]
    obs7 = plain((variant(from_lookup("LUMEN", {
        "LUMEN_ADMISSION_READ_CAPACITY": "1",
        "LUMEN_ADMISSION_REFILL_SECS": "x",
        }.get)),
        from_lookup("LUMEN", {
        "LUMEN_ADMISSION_READ_CAPACITY": "1",
        "LUMEN_ADMISSION_MAX_KEYS": "0",
        }.get).key))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. an invalid policy names the class and the reason
    exp8 = OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[7][1]
    obs8 = plain((describe(InvalidPolicy("write", "capacity must be positive")),
        variant(InvalidPolicy("write", "x")),
        describe(InvalidValue("K", "v")),
        describe(OrphanedCommonSetting("K"))))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. a refusal is returned rather than raised
    exp9 = OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[8][1]
    obs9 = plain((refusal(from_lookup, "LUMEN",
        {"LUMEN_ADMISSION_READ_CAPACITY": "abc"}.get),
        refusal(from_lookup, "LUMEN", {"LUMEN_ADMISSION_REFILL_SECS": "5"}.get),
        refusal(from_lookup, "LUMEN", lambda k: None),
        variant(from_lookup("LUMEN", lambda k: None))))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. an unknown error value has no sentence
    exp10 = OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[9][1]
    obs10 = plain((refusal(describe, "boom"), refusal(describe, None),
        refusal(describe, InvalidValue("K", "v"))))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. the prefix is carried verbatim into every key
    exp11 = OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[10][1]
    obs11 = plain((env_key("", "READ_CAPACITY"), env_key("A_B", "MAX_KEYS"),
        all_keys("")[0], len(set(all_keys("LUMEN")))))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. a config is immutable
    exp12 = OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[11][1]
    obs12 = plain((refusal(setattr, from_lookup("LUMEN", lambda k: None),
        "max_keys", 1),
        refusal(setattr, InvalidValue("K", "v"), "key", "X"),
        refusal(setattr, AdmissionPolicy(1, WINDOW, 8), "capacity", 2)))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_SECURITY_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    return {
        "case_id": "opt-in-admission-configuration-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
