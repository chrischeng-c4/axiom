from __future__ import annotations

from service_http.application.admission import AdmissionLedger
from service_http.application.admission_config import AdmissionConfig, controller_policies, from_lookup, is_enabled, policies
from service_http.domain.admission import DEFAULT_MAX_KEYS, DEFAULT_REFILL_SECS
from service_http.infrastructure.env import all_keys, capacity_keys, common_keys, env_key

MINIMUM_CHECKS = 12

OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX = (
    ("the_key_grammar_is_one_prefix_and_a_fixed_infix",
     ('LUMEN_ADMISSION_READ_CAPACITY', ('LUMEN_ADMISSION_READ_CAPACITY', 'LUMEN_ADMISSION_WRITE_CAPACITY', 'LUMEN_ADMISSION_ADMIN_CAPACITY'), ('LUMEN_ADMISSION_REFILL_SECS', 'LUMEN_ADMISSION_MAX_KEYS'))),
    ("the_full_key_set_is_the_capacities_then_the_common_settings",
     (('LUMEN_ADMISSION_READ_CAPACITY', 'LUMEN_ADMISSION_WRITE_CAPACITY', 'LUMEN_ADMISSION_ADMIN_CAPACITY', 'LUMEN_ADMISSION_REFILL_SECS', 'LUMEN_ADMISSION_MAX_KEYS'), 5, True)),
    ("an_environment_with_no_keys_is_a_disabled_default_config",
     ('AdmissionConfig', None, None, None, 60, 1024, False)),
    ("one_capacity_enables_the_whole_feature",
     ('AdmissionConfig', 5, None, True, 60, 1024)),
    ("the_common_settings_override_their_defaults",
     (10, 5, 16, True)),
    ("only_the_configured_classes_get_a_policy",
     ({'w': (5, 60000000000, 1024)}, ('w',), 0)),
    ("a_built_policy_carries_the_window_in_nanoseconds",
     (10, 5000000000, 16)),
    ("a_hand_built_config_that_skips_the_parser_is_still_refused",
     ('InvalidPolicy', 'capacity must be positive', 'refill window must be positive', 'max keys must be positive', 'r')),
    ("a_disabled_config_builds_no_controller_at_all",
     (None, 'dict', ('w',))),
    ("three_capacities_build_three_named_classes",
     (('r', 'w', 'a'), 1, 2, 3)),
    ("the_parsed_config_reaches_a_real_bucket",
     ('allow', 'deny', 'allow', 1)),
    ("the_defaults_are_the_documented_constants",
     (60, 1024, 60, 1024, True)),
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


def verify_opt_in_admission_configuration_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. the key grammar is one prefix and a fixed infix
    exp1 = OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[0][1]
    obs1 = plain((env_key("LUMEN", "READ_CAPACITY"), capacity_keys("LUMEN"),
        common_keys("LUMEN")))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. the full key set is the capacities then the common settings
    exp2 = OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[1][1]
    obs2 = plain((all_keys("LUMEN"), len(all_keys("LUMEN")),
        all_keys("LUMEN") == capacity_keys("LUMEN") + common_keys("LUMEN")))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. an environment with no keys is a disabled default config
    exp3 = OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[2][1]
    empty = from_lookup("LUMEN", lambda k: None)
    obs3 = plain((variant(empty), empty.read_capacity, empty.write_capacity,
        empty.admin_capacity, empty.refill_secs, empty.max_keys,
        is_enabled(empty)))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. one capacity enables the whole feature
    exp4 = OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[3][1]
    one = from_lookup("LUMEN", {"LUMEN_ADMISSION_WRITE_CAPACITY": "5"}.get)
    obs4 = plain((variant(one), one.write_capacity, one.read_capacity,
        is_enabled(one), one.refill_secs, one.max_keys))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the common settings override their defaults
    exp5 = OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[4][1]
    tuned = from_lookup("LUMEN", {
        "LUMEN_ADMISSION_READ_CAPACITY": "10",
        "LUMEN_ADMISSION_REFILL_SECS": "5",
        "LUMEN_ADMISSION_MAX_KEYS": "16",
        }.get)
    obs5 = plain((tuned.read_capacity, tuned.refill_secs, tuned.max_keys,
        is_enabled(tuned)))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. only the configured classes get a policy
    exp6 = OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[5][1]
    empty = from_lookup("LUMEN", lambda k: None)
    one = from_lookup("LUMEN", {"LUMEN_ADMISSION_WRITE_CAPACITY": "5"}.get)
    obs6 = plain((policies(one, "r", "w", "a"),
        tuple(policies(one, "r", "w", "a").keys()),
        len(policies(empty, "r", "w", "a"))))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. a built policy carries the window in nanoseconds
    exp7 = OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[6][1]
    tuned = from_lookup("LUMEN", {
        "LUMEN_ADMISSION_READ_CAPACITY": "10",
        "LUMEN_ADMISSION_REFILL_SECS": "5",
        "LUMEN_ADMISSION_MAX_KEYS": "16",
        }.get)
    obs7 = plain((policies(tuned, "r", "w", "a")["r"].capacity,
        policies(tuned, "r", "w", "a")["r"].refill_window_ns,
        policies(tuned, "r", "w", "a")["r"].max_keys))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a hand built config that skips the parser is still refused
    exp8 = OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[7][1]
    obs8 = plain((variant(policies(AdmissionConfig(0, None, None, 5, 16), "r", "w", "a")),
        policies(AdmissionConfig(0, None, None, 5, 16), "r", "w", "a").reason,
        policies(AdmissionConfig(1, None, None, 0, 16), "r", "w", "a").reason,
        policies(AdmissionConfig(1, None, None, 5, 0), "r", "w", "a").reason,
        policies(AdmissionConfig(0, None, None, 5, 16),
        "r", "w", "a").route_class))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. a disabled config builds no controller at all
    exp9 = OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[8][1]
    empty = from_lookup("LUMEN", lambda k: None)
    one = from_lookup("LUMEN", {"LUMEN_ADMISSION_WRITE_CAPACITY": "5"}.get)
    obs9 = plain((controller_policies(empty, "r", "w", "a"),
        variant(controller_policies(one, "r", "w", "a")),
        tuple(controller_policies(one, "r", "w", "a").keys())))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. three capacities build three named classes
    exp10 = OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[9][1]
    all_three = from_lookup("LUMEN", {
        "LUMEN_ADMISSION_READ_CAPACITY": "1",
        "LUMEN_ADMISSION_WRITE_CAPACITY": "2",
        "LUMEN_ADMISSION_ADMIN_CAPACITY": "3",
        }.get)
    obs10 = plain((tuple(controller_policies(all_three, "r", "w", "a").keys()),
        controller_policies(all_three, "r", "w", "a")["r"].capacity,
        controller_policies(all_three, "r", "w", "a")["w"].capacity,
        controller_policies(all_three, "r", "w", "a")["a"].capacity))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. the parsed config reaches a real bucket
    exp11 = OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[10][1]
    all_three = from_lookup("LUMEN", {
        "LUMEN_ADMISSION_READ_CAPACITY": "1",
        "LUMEN_ADMISSION_WRITE_CAPACITY": "2",
        "LUMEN_ADMISSION_ADMIN_CAPACITY": "3",
        }.get)
    built = AdmissionLedger(controller_policies(all_three, "r", "w", "a"))
    obs11 = plain((built.admit_at("r", "k", 0).outcome,
        built.admit_at("r", "k", 0).outcome,
        built.admit_at("w", "k", 0).outcome,
        built.tracked_keys("r")))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. the defaults are the documented constants
    exp12 = OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[11][1]
    empty = from_lookup("LUMEN", lambda k: None)
    obs12 = plain((DEFAULT_REFILL_SECS, DEFAULT_MAX_KEYS, empty.refill_secs,
        empty.max_keys, AdmissionConfig(None, None, None, 60, 1024) == empty))
    checks.append({"name": OPT_IN_ADMISSION_CONFIGURATION_BEHAVIOR_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    return {
        "case_id": "opt-in-admission-configuration-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
