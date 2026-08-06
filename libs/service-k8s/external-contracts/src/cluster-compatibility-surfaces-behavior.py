from __future__ import annotations

from service_k8s.infrastructure.crd import (
    add_spec_validation_rule,
    normalize_unsigned_integer_formats,
    quote_yaml_1_1_boolean_like_strings,
)
from service_k8s.infrastructure.resize import (
    PvcFacts,
    PvcResizeOutcome,
    decide,
    parse_storage_bytes,
    plan_resize,
    storage_patch,
)

MINIMUM_CHECKS = 19

CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX = (
    (
        "binary_suffixes_are_powers_of_1024",
        (1024, 1048576, 1073741824, 1099511627776, 1125899906842624, 1152921504606846976),
    ),
    (
        "decimal_suffixes_are_powers_of_1000",
        (1000, 1000000, 1000000000, 1000000000000, 1000000000000000, 1000000000000000000),
    ),
    (
        "a_bare_integer_is_already_a_byte_count",
        (0, 512, 1099511627776),
    ),
    (
        "a_fractional_quantity_is_scaled_and_rounded_to_whole_bytes",
        (1610612736, 524288, 2500, 1280, 629146),
    ),
    (
        "surrounding_whitespace_is_tolerated_on_both_sides_of_the_suffix",
        (1073741824, 512, 2097152),
    ),
    (
        "the_comparison_falls_into_exactly_four_kinds",
        ("grow", "noop", "shrink-unsupported", "unparseable"),
    ),
    (
        "equality_is_measured_in_bytes_not_in_spelling",
        ("noop", 1073741824, 1073741824, "already at desired size"),
    ),
    (
        "a_grow_carries_both_byte_counts_and_needs_no_explanation",
        ("grow", 1073741824, 2147483648, ""),
    ),
    (
        "a_shrink_carries_the_reason_it_will_not_be_attempted",
        (
            2147483648,
            1073741824,
            "desired size is smaller than current; Kubernetes cannot shrink a bound PVC, recreate it instead",
        ),
    ),
    (
        "an_unparseable_side_is_named_and_carries_no_byte_counts",
        (
            "current quantity 'bogus': unrecognized storage quantity 'bogus'",
            "desired quantity 'bogus': unrecognized storage quantity 'bogus'",
            True,
            True,
        ),
    ),
    (
        "the_patch_touches_exactly_the_one_field_a_template_cannot_change",
        (("spec",), ("resources",), ("requests",), ("storage",), "2Gi"),
    ),
    (
        "planning_considers_only_the_pvcs_the_filter_admits",
        (
            8,
            7,
            (
                "data-lumen-0",
                "data-lumen-1",
                "data-lumen-2",
                "data-lumen-3",
                "data-lumen-4",
                "data-lumen-5",
                "data-lumen-6",
            ),
        ),
    ),
    (
        "an_expandable_class_outside_a_dry_run_is_the_only_thing_patched",
        (
            True,
            "patched spec.resources.requests.storage",
            "1Gi",
            "2Gi",
            (True, False, False, False, False, False, False),
        ),
    ),
    (
        "a_dry_run_says_what_it_would_do_and_patches_nothing",
        (
            False,
            "dry run: would patch spec.resources.requests.storage",
            (False, False, False, False, False, False, False),
        ),
    ),
    (
        "every_non_growing_outcome_carries_its_own_reason",
        (
            "StorageClass 'slow' does not allow volume expansion; recreate the PVC/StatefulSet manually",
            "already at desired size",
            "desired size is smaller than current; Kubernetes cannot shrink a bound PVC, recreate it instead",
            "current quantity 'bogus': unrecognized storage quantity 'bogus'",
        ),
    ),
    (
        "unsigned_formats_are_removed_at_every_depth_including_inside_arrays",
        (("minimum", "type"), 0, ("minimum", "type"), 0, ("minimum", "type"), 0),
    ),
    (
        "a_format_kubernetes_understands_is_left_alone",
        ("int64", -5),
    ),
    (
        "a_cel_rule_reaches_every_version_and_a_second_does_not_replace_the_first",
        (2, 2, 2, "has(self.a)", "b required"),
    ),
    (
        "yaml_1_1_boolean_spellings_are_quoted_in_both_mapping_and_sequence_slots",
        (
            "spec:",
            '  enabled: "on"',
            '  mode: "off"',
            '  answer: "Yes"',
            "  flags:",
            '    - "no"',
            '    - "y"',
            "  note: turn it on: no",
        ),
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


def two_gib(name: str) -> str:
    return "2Gi"


def plan(dry_run: bool) -> tuple[PvcResizeOutcome, ...]:
    return plan_resize(PVCS, only_data, two_gib, ALLOW, dry_run)


def crd_with_versions(count: int) -> dict[str, object]:
    return {
        "spec": {
            "versions": [
                {
                    "schema": {
                        "openAPIV3Schema": {
                            "properties": {"spec": {"type": "object"}}
                        }
                    }
                }
                for _ in range(count)
            ]
        }
    }


def rules_of(crd: dict[str, object], index: int) -> list[dict[str, str]]:
    versions = crd["spec"]["versions"]
    schema = versions[index]["schema"]["openAPIV3Schema"]["properties"]["spec"]
    return schema.get("x-kubernetes-validations", [])


def verify_cluster_compatibility_surfaces_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. binary_suffixes_are_powers_of_1024
    exp1 = CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[0][1]
    obs1 = (
        parse_storage_bytes("1Ki"),
        parse_storage_bytes("1Mi"),
        parse_storage_bytes("1Gi"),
        parse_storage_bytes("1Ti"),
        parse_storage_bytes("1Pi"),
        parse_storage_bytes("1Ei"),
    )
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. decimal_suffixes_are_powers_of_1000
    exp2 = CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[1][1]
    obs2 = (
        parse_storage_bytes("1k"),
        parse_storage_bytes("1M"),
        parse_storage_bytes("1G"),
        parse_storage_bytes("1T"),
        parse_storage_bytes("1P"),
        parse_storage_bytes("1E"),
    )
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. a_bare_integer_is_already_a_byte_count
    exp3 = CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[2][1]
    obs3 = (
        parse_storage_bytes("0"),
        parse_storage_bytes("512"),
        parse_storage_bytes("1099511627776"),
    )
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. a_fractional_quantity_is_scaled_and_rounded_to_whole_bytes
    exp4 = CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[3][1]
    obs4 = (
        parse_storage_bytes("1.5Gi"),
        parse_storage_bytes("0.5Mi"),
        parse_storage_bytes("2.5k"),
        parse_storage_bytes("1.25Ki"),
        parse_storage_bytes("0.6Mi"),
    )
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. surrounding_whitespace_is_tolerated_on_both_sides_of_the_suffix
    exp5 = CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[4][1]
    obs5 = (
        parse_storage_bytes(" 1 Gi "),
        parse_storage_bytes("  512  "),
        parse_storage_bytes("\t2Mi\n"),
    )
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. the_comparison_falls_into_exactly_four_kinds
    exp6 = CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[5][1]
    obs6 = (
        decide("1Gi", "2Gi").kind.value,
        decide("1Gi", "1024Mi").kind.value,
        decide("2Gi", "1Gi").kind.value,
        decide("bogus", "1Gi").kind.value,
    )
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. equality_is_measured_in_bytes_not_in_spelling
    exp7 = CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[6][1]
    same7 = decide("1Gi", "1024Mi")
    obs7 = (same7.kind.value, same7.current_bytes, same7.desired_bytes, same7.detail)
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. a_grow_carries_both_byte_counts_and_needs_no_explanation
    exp8 = CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[7][1]
    grow8 = decide("1Gi", "2Gi")
    obs8 = (grow8.kind.value, grow8.current_bytes, grow8.desired_bytes, grow8.detail)
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. a_shrink_carries_the_reason_it_will_not_be_attempted
    exp9 = CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[8][1]
    shrink9 = decide("2Gi", "1Gi")
    obs9 = (shrink9.current_bytes, shrink9.desired_bytes, shrink9.detail)
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. an_unparseable_side_is_named_and_carries_no_byte_counts
    exp10 = CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[9][1]
    bad_current10 = decide("bogus", "1Gi")
    bad_desired10 = decide("1Gi", "bogus")
    obs10 = (
        bad_current10.detail,
        bad_desired10.detail,
        bad_current10.current_bytes is None,
        bad_current10.desired_bytes is None,
    )
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. the_patch_touches_exactly_the_one_field_a_template_cannot_change
    exp11 = CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[10][1]
    patch11 = storage_patch("2Gi")
    obs11 = (
        tuple(patch11),
        tuple(patch11["spec"]),
        tuple(patch11["spec"]["resources"]),
        tuple(patch11["spec"]["resources"]["requests"]),
        patch11["spec"]["resources"]["requests"]["storage"],
    )
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. planning_considers_only_the_pvcs_the_filter_admits
    exp12 = CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[11][1]
    live12 = plan(False)
    obs12 = (len(PVCS), len(live12), tuple(o.pvc_name for o in live12))
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. an_expandable_class_outside_a_dry_run_is_the_only_thing_patched
    exp13 = CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[12][1]
    live13 = plan(False)
    obs13 = (
        live13[0].patched,
        live13[0].detail,
        live13[0].current,
        live13[0].desired,
        tuple(o.patched for o in live13),
    )
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    # 14. a_dry_run_says_what_it_would_do_and_patches_nothing
    exp14 = CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[13][1]
    dry14 = plan(True)
    obs14 = (dry14[0].patched, dry14[0].detail, tuple(o.patched for o in dry14))
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[13][0],
            "expected": exp14,
            "observed": obs14,
            "passed": obs14 == exp14,
        }
    )

    # 15. every_non_growing_outcome_carries_its_own_reason
    exp15 = CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[14][1]
    live15 = plan(False)
    obs15 = (live15[1].detail, live15[3].detail, live15[4].detail, live15[5].detail)
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[14][0],
            "expected": exp15,
            "observed": obs15,
            "passed": obs15 == exp15,
        }
    )

    # 16. unsigned_formats_are_removed_at_every_depth_including_inside_arrays
    exp16 = CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[15][1]
    nested16 = {
        "properties": {
            "shards": {"type": "integer", "format": "uint32"},
            "members": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "port": {"type": "integer", "format": "uint64"},
                    },
                },
            },
            "budget": {
                "anyOf": [
                    {"type": "string"},
                    {"type": "integer", "format": "uint64"},
                ]
            },
        }
    }
    normalize_unsigned_integer_formats(nested16)
    inner16 = nested16["properties"]["members"]["items"]["properties"]["port"]
    branch16 = nested16["properties"]["budget"]["anyOf"][1]
    obs16 = (
        tuple(sorted(nested16["properties"]["shards"])),
        nested16["properties"]["shards"]["minimum"],
        tuple(sorted(inner16)),
        inner16["minimum"],
        tuple(sorted(branch16)),
        branch16["minimum"],
    )
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[15][0],
            "expected": exp16,
            "observed": obs16,
            "passed": obs16 == exp16,
        }
    )

    # 17. a_format_kubernetes_understands_is_left_alone
    exp17 = CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[16][1]
    untouched17 = {"type": "integer", "format": "int64", "minimum": -5}
    normalize_unsigned_integer_formats(untouched17)
    obs17 = (untouched17["format"], untouched17["minimum"])
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[16][0],
            "expected": exp17,
            "observed": obs17,
            "passed": obs17 == exp17,
        }
    )

    # 18. a_cel_rule_reaches_every_version_and_a_second_does_not_replace_the_first
    exp18 = CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[17][1]
    crd18 = crd_with_versions(2)
    first18 = add_spec_validation_rule(crd18, "has(self.a)", "a required")
    second18 = add_spec_validation_rule(crd18, "has(self.b)", "b required")
    obs18 = (
        first18,
        second18,
        len(rules_of(crd18, 0)),
        rules_of(crd18, 0)[0]["rule"],
        rules_of(crd18, 1)[1]["message"],
    )
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[17][0],
            "expected": exp18,
            "observed": obs18,
            "passed": obs18 == exp18,
        }
    )

    # 19. yaml_1_1_boolean_spellings_are_quoted_in_both_mapping_and_sequence_slots
    exp19 = CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[18][1]
    yaml19 = "\n".join(
        (
            "spec:",
            "  enabled: on",
            "  mode: off",
            "  answer: Yes",
            "  flags:",
            "    - no",
            "    - y",
            "  note: turn it on: no",
        )
    )
    obs19 = tuple(quote_yaml_1_1_boolean_like_strings(yaml19).splitlines())
    checks.append(
        {
            "name": CLUSTER_COMPATIBILITY_SURFACES_BEHAVIOR_MATRIX[18][0],
            "expected": exp19,
            "observed": obs19,
            "passed": obs19 == exp19,
        }
    )

    return {
        "case_id": "cluster-compatibility-surfaces-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
