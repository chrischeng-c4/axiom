from __future__ import annotations

from service_backup.application.parse import parse_destination
from service_backup.domain.destination import FILE_SCHEME, GCS_SCHEME, S3_SCHEME
from service_backup.infrastructure.schemes import BuildFeatures, find_scheme, scheme_names, supported_schemes, topic_destination_section

MINIMUM_CHECKS = 10

NON_DRIFTING_SCHEME_DOCUMENTATION_BEHAVIOR_MATRIX = (
    ("the_section_renders_one_line_per_inventory_entry",
     (3, 3, 3)),
    ("no_scheme_is_rendered_twice",
     (3, ('file://', 's3://', 'gs://'))),
    ("the_leading_tokens_appear_in_the_inventorys_own_order",
     (('file://', 's3://', 'gs://'), ('file://', 's3://', 'gs://'))),
    ("the_unconditional_lines_carry_their_own_descriptions",
     ('file://  local filesystem path - dev/tests and PVC-backed local runs', 'gs://  Google Cloud Storage - workload identity in production')),
    ("the_object_store_line_is_whole_in_a_linked_build",
     's3://  Amazon S3-compatible object store'),
    ("the_inventory_publishes_a_scheme_a_description_and_an_availability",
     (('file://', 'local filesystem path - dev/tests and PVC-backed local runs', True), ('s3://', 'Amazon S3-compatible object store', True), ('gs://', 'Google Cloud Storage - workload identity in production', True))),
    ("the_section_is_generated_from_the_constants_the_parser_uses",
     (True, True, True, 'S3')),
    ("only_the_object_store_line_differs_between_builds",
     ((1,), 's3://  Amazon S3-compatible object store', 's3://  Amazon S3-compatible object store (not linked in this build)')),
    ("the_section_ends_without_a_trailing_separator",
     (False, 2, True)),
    ("the_lookup_answers_the_same_entry_the_section_rendered",
     ('Amazon S3-compatible object store', 'file://', True, None)),
)


def variant(value: object) -> str:
    """The name of the returned variant — the shape of an error-as-value."""
    return type(value).__name__


LINKED = BuildFeatures(s3=True)


UNLINKED = BuildFeatures(s3=False)


def verify_non_drifting_scheme_documentation_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. the section renders one line per inventory entry
    exp1 = NON_DRIFTING_SCHEME_DOCUMENTATION_BEHAVIOR_MATRIX[0][1]
    obs1 = (len(topic_destination_section(LINKED).split("\n")),
        len(supported_schemes(LINKED)),
        len(topic_destination_section(UNLINKED).split("\n")))
    checks.append({"name": NON_DRIFTING_SCHEME_DOCUMENTATION_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. no scheme is rendered twice
    exp2 = NON_DRIFTING_SCHEME_DOCUMENTATION_BEHAVIOR_MATRIX[1][1]
    obs2 = (len({line.split("  ")[0]
        for line in topic_destination_section(LINKED).split("\n")}),
        tuple(line.split("  ")[0]
        for line in topic_destination_section(LINKED).split("\n")))
    checks.append({"name": NON_DRIFTING_SCHEME_DOCUMENTATION_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the leading tokens appear in the inventorys own order
    exp3 = NON_DRIFTING_SCHEME_DOCUMENTATION_BEHAVIOR_MATRIX[2][1]
    obs3 = (tuple(line.split(" ")[0]
        for line in topic_destination_section(LINKED).split("\n")),
        scheme_names(LINKED))
    checks.append({"name": NON_DRIFTING_SCHEME_DOCUMENTATION_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. the unconditional lines carry their own descriptions
    exp4 = NON_DRIFTING_SCHEME_DOCUMENTATION_BEHAVIOR_MATRIX[3][1]
    obs4 = (topic_destination_section(LINKED).split("\n")[0],
        topic_destination_section(LINKED).split("\n")[2])
    checks.append({"name": NON_DRIFTING_SCHEME_DOCUMENTATION_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the object store line is whole in a linked build
    exp5 = NON_DRIFTING_SCHEME_DOCUMENTATION_BEHAVIOR_MATRIX[4][1]
    obs5 = topic_destination_section(LINKED).split("\n")[1]
    checks.append({"name": NON_DRIFTING_SCHEME_DOCUMENTATION_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. the inventory publishes a scheme a description and an availability
    exp6 = NON_DRIFTING_SCHEME_DOCUMENTATION_BEHAVIOR_MATRIX[5][1]
    obs6 = tuple((s.scheme, s.description, s.sink_available)
        for s in supported_schemes(LINKED))
    checks.append({"name": NON_DRIFTING_SCHEME_DOCUMENTATION_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. the section is generated from the constants the parser uses
    exp7 = NON_DRIFTING_SCHEME_DOCUMENTATION_BEHAVIOR_MATRIX[6][1]
    obs7 = (scheme_names(LINKED)[0] == FILE_SCHEME, scheme_names(LINKED)[1] == S3_SCHEME,
        scheme_names(LINKED)[2] == GCS_SCHEME,
        variant(parse_destination(scheme_names(LINKED)[1] + "b/p")))
    checks.append({"name": NON_DRIFTING_SCHEME_DOCUMENTATION_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. only the object store line differs between builds
    exp8 = NON_DRIFTING_SCHEME_DOCUMENTATION_BEHAVIOR_MATRIX[7][1]
    obs8 = (tuple(i for i in range(len(topic_destination_section(LINKED).split("\n")))
        if topic_destination_section(LINKED).split("\n")[i]
        != topic_destination_section(UNLINKED).split("\n")[i]),
        topic_destination_section(LINKED).split("\n")[1],
        topic_destination_section(UNLINKED).split("\n")[1])
    checks.append({"name": NON_DRIFTING_SCHEME_DOCUMENTATION_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the section ends without a trailing separator
    exp9 = NON_DRIFTING_SCHEME_DOCUMENTATION_BEHAVIOR_MATRIX[8][1]
    obs9 = (topic_destination_section(LINKED).endswith("\n"),
        topic_destination_section(LINKED).count("\n"),
        topic_destination_section(LINKED).startswith("file://"))
    checks.append({"name": NON_DRIFTING_SCHEME_DOCUMENTATION_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the lookup answers the same entry the section rendered
    exp10 = NON_DRIFTING_SCHEME_DOCUMENTATION_BEHAVIOR_MATRIX[9][1]
    obs10 = (find_scheme("s3://", LINKED).description, find_scheme("file://", LINKED).scheme,
        find_scheme("gs://", UNLINKED).sink_available, find_scheme("ftp://", LINKED))
    checks.append({"name": NON_DRIFTING_SCHEME_DOCUMENTATION_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    return {
        "case_id": "non-drifting-scheme-documentation-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
