from __future__ import annotations

from service_k8s.infrastructure.crd import (
    CelRuleError,
    add_spec_validation_rule,
    normalize_unsigned_integer_formats,
    quote_yaml_1_1_boolean_like_strings,
)
from service_k8s.infrastructure.resize import (
    SHRINK_DETAIL,
    PvcFacts,
    PvcResizeOutcome,
    QuantityError,
    decide,
    parse_storage_bytes,
    plan_resize,
)

MINIMUM_CHECKS = 16

CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX = (
    (
        "an_empty_quantity_is_an_error_rather_than_zero_bytes",
        ("empty storage quantity", "empty storage quantity", "empty storage quantity"),
    ),
    (
        "a_negative_quantity_is_an_error_rather_than_a_byte_count",
        (
            "negative storage quantity '-1Gi'",
            "unrecognized storage quantity '-1'",
            "negative storage quantity '-0.5Mi'",
        ),
    ),
    (
        "a_non_numeric_mantissa_is_refused_and_the_whole_quantity_is_named",
        (
            "invalid numeric part in storage quantity 'abcGi'",
            "invalid numeric part in storage quantity 'Gi'",
            "invalid numeric part in storage quantity '1.2.3Mi'",
        ),
    ),
    (
        "a_bare_count_must_be_ascii_digits_so_no_python_integer_spelling_leaks_in",
        (
            "unrecognized storage quantity '5_0'",
            "unrecognized storage quantity '+5'",
            "unrecognized storage quantity '\u0665'",
            "unrecognized storage quantity '0x10'",
        ),
    ),
    (
        "a_bare_fractional_count_is_refused_rather_than_truncated",
        ("unrecognized storage quantity '1.5'", "unrecognized storage quantity '1e3'"),
    ),
    (
        "an_unparseable_current_size_is_reported_per_pvc_and_never_patched",
        (
            "data-lumen-5",
            False,
            "current quantity 'bogus': unrecognized storage quantity 'bogus'",
        ),
    ),
    (
        "a_shrink_is_classified_and_never_patched",
        ("shrink-unsupported", "data-lumen-4", False, True),
    ),
    (
        "a_pvc_with_no_storage_class_is_never_patched",
        (
            "data-lumen-2",
            False,
            "StorageClass '<none>' does not allow volume expansion; recreate the PVC/StatefulSet manually",
        ),
    ),
    (
        "a_class_absent_from_the_expansion_map_is_never_patched",
        (
            "data-lumen-1",
            False,
            "StorageClass 'slow' does not allow volume expansion; recreate the PVC/StatefulSet manually",
            "data-lumen-6",
            False,
            "StorageClass 'unknown' does not allow volume expansion; recreate the PVC/StatefulSet manually",
        ),
    ),
    (
        "a_dry_run_patches_nothing_at_all",
        (False, 1),
    ),
    (
        "a_pvc_the_filter_rejects_produces_no_outcome_at_all",
        (True, False, 0),
    ),
    (
        "a_cel_rule_comparing_against_null_is_refused_under_any_whitespace",
        (
            "CEL rules must not compare against null directly; use has(self.field)",
            "CEL rules must not compare against null directly; use has(self.field)",
            "CEL rules must not compare against null directly; use has(self.field)",
            "CEL rules must not compare against null directly; use has(self.field)",
            "CEL rules must not compare against null directly; use has(self.field)",
        ),
    ),
    (
        "a_crd_without_a_spec_schema_attaches_nothing_rather_than_raising",
        (0, 0, 0),
    ),
    (
        "an_existing_tighter_minimum_survives_normalization",
        (1, False, 0),
    ),
    (
        "a_real_boolean_is_left_unquoted_because_json_never_spells_it_that_way",
        ("default: false", "default: true", "nullable: true"),
    ),
    (
        "the_quoter_preserves_prose_and_the_documents_trailing_newline_shape",
        ("desc: say no to this", 'a: "on"\n', 'a: "on"', ""),
    ),
)

PVCS = (
    PvcFacts("data-lumen-0", "1Gi", "fast"),
    PvcFacts("data-lumen-1", "1Gi", "slow"),
    PvcFacts("logs-lumen-0", "1Gi", "fast"),
    PvcFacts("data-lumen-2", "1Gi", None),
    PvcFacts("data-lumen-3", "2Gi", "fast"),
    PvcFacts("data-lumen-4", "4Gi", "fast"),
    PvcFacts("data-lumen-5", "bogus", "fast"),
    PvcFacts("data-lumen-6", "1Gi", "unknown"),
)

ALLOW = {"fast": True, "slow": False}


def only_data(name: str) -> bool:
    return name.startswith("data-")


def reject_all(name: str) -> bool:
    return False


def two_gib(name: str) -> str:
    return "2Gi"


def plan(dry_run: bool) -> tuple[PvcResizeOutcome, ...]:
    return plan_resize(PVCS, only_data, two_gib, ALLOW, dry_run)


def quantity_error(text: str) -> str:
    try:
        parse_storage_bytes(text)
    except QuantityError as exc:
        return str(exc)
    return "NO ERROR"


def cel_error(rule: str) -> str:
    try:
        add_spec_validation_rule({}, rule, "m")
    except CelRuleError as exc:
        return str(exc)
    return "NO ERROR"


def verify_cluster_compatibility_surfaces_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. an_empty_quantity_is_an_error_rather_than_zero_bytes
    exp1 = CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[0][1]
    obs1 = (quantity_error(""), quantity_error("   "), quantity_error("\t\n"))
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. a_negative_quantity_is_an_error_rather_than_a_byte_count
    exp2 = CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[1][1]
    obs2 = (quantity_error("-1Gi"), quantity_error("-1"), quantity_error("-0.5Mi"))
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. a_non_numeric_mantissa_is_refused_and_the_whole_quantity_is_named
    exp3 = CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[2][1]
    obs3 = (quantity_error("abcGi"), quantity_error("Gi"), quantity_error("1.2.3Mi"))
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. a_bare_count_must_be_ascii_digits_so_no_python_integer_spelling_leaks_in
    exp4 = CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[3][1]
    obs4 = (
        quantity_error("5_0"),
        quantity_error("+5"),
        quantity_error("\u0665"),
        quantity_error("0x10"),
    )
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. a_bare_fractional_count_is_refused_rather_than_truncated
    exp5 = CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[4][1]
    obs5 = (quantity_error("1.5"), quantity_error("1e3"))
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. an_unparseable_current_size_is_reported_per_pvc_and_never_patched
    exp6 = CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[5][1]
    live6 = plan(False)
    obs6 = (live6[5].pvc_name, live6[5].patched, live6[5].detail)
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. a_shrink_is_classified_and_never_patched
    exp7 = CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[6][1]
    live7 = plan(False)
    obs7 = (
        decide("4Gi", "2Gi").kind.value,
        live7[4].pvc_name,
        live7[4].patched,
        live7[4].detail == SHRINK_DETAIL,
    )
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. a_pvc_with_no_storage_class_is_never_patched
    exp8 = CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[7][1]
    live8 = plan(False)
    obs8 = (live8[2].pvc_name, live8[2].patched, live8[2].detail)
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. a_class_absent_from_the_expansion_map_is_never_patched
    exp9 = CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[8][1]
    live9 = plan(False)
    obs9 = (
        live9[1].pvc_name,
        live9[1].patched,
        live9[1].detail,
        live9[6].pvc_name,
        live9[6].patched,
        live9[6].detail,
    )
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. a_dry_run_patches_nothing_at_all
    exp10 = CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[9][1]
    dry10 = plan(True)
    live10 = plan(False)
    obs10 = (
        any(o.patched for o in dry10),
        sum(1 for o in live10 if o.patched),
    )
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. a_pvc_the_filter_rejects_produces_no_outcome_at_all
    exp11 = CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[10][1]
    live11 = plan(False)
    obs11 = (
        "logs-lumen-0" in tuple(p.name for p in PVCS),
        "logs-lumen-0" in tuple(o.pvc_name for o in live11),
        len(plan_resize(PVCS, reject_all, two_gib, ALLOW, False)),
    )
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. a_cel_rule_comparing_against_null_is_refused_under_any_whitespace
    exp12 = CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[11][1]
    obs12 = (
        cel_error("self.x != null"),
        cel_error("self.x!=null"),
        cel_error("self . x  ==  null"),
        cel_error("self.x\t!=\tnull"),
        cel_error("self.x !=\n  null"),
    )
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. a_crd_without_a_spec_schema_attaches_nothing_rather_than_raising
    exp13 = CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[12][1]
    no_schema13 = {"spec": {"versions": [{"schema": {}}]}}
    empty13: dict[str, object] = {}
    not_a_list13 = {"spec": {"versions": "v1"}}
    obs13 = (
        add_spec_validation_rule(no_schema13, "has(self.a)", "m"),
        add_spec_validation_rule(empty13, "has(self.a)", "m"),
        add_spec_validation_rule(not_a_list13, "has(self.a)", "m"),
    )
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    # 14. an_existing_tighter_minimum_survives_normalization
    exp14 = CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[13][1]
    tighter14 = {"type": "integer", "format": "uint32", "minimum": 1}
    normalize_unsigned_integer_formats(tighter14)
    zeroed14 = {"type": "integer", "format": "uint64", "minimum": 0}
    normalize_unsigned_integer_formats(zeroed14)
    obs14 = (tighter14["minimum"], "format" in tighter14, zeroed14["minimum"])
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[13][0],
            "expected": exp14,
            "observed": obs14,
            "passed": obs14 == exp14,
        }
    )

    # 15. a_real_boolean_is_left_unquoted_because_json_never_spells_it_that_way
    exp15 = CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[14][1]
    obs15 = (
        quote_yaml_1_1_boolean_like_strings("default: false"),
        quote_yaml_1_1_boolean_like_strings("default: true"),
        quote_yaml_1_1_boolean_like_strings("nullable: true"),
    )
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[14][0],
            "expected": exp15,
            "observed": obs15,
            "passed": obs15 == exp15,
        }
    )

    # 16. the_quoter_preserves_prose_and_the_documents_trailing_newline_shape
    exp16 = CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[15][1]
    obs16 = (
        quote_yaml_1_1_boolean_like_strings("desc: say no to this"),
        quote_yaml_1_1_boolean_like_strings("a: on\n"),
        quote_yaml_1_1_boolean_like_strings("a: on"),
        quote_yaml_1_1_boolean_like_strings(""),
    )
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_SECURITY_MATRIX[15][0],
            "expected": exp16,
            "observed": obs16,
            "passed": obs16 == exp16,
        }
    )

    return {
        "case_id": "cluster-compatibility-surfaces-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
