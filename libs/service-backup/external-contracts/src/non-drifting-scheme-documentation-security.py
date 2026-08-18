from __future__ import annotations

from service_backup.infrastructure.schemes import BuildFeatures, find_scheme, supported_schemes, topic_destination_section, unavailable_schemes

MINIMUM_CHECKS = 11

NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX = (
    ("every_inventory_scheme_and_description_appears_in_the_section",
     (True, True, True, 3)),
    ("the_section_names_no_scheme_the_inventory_does_not",
     (3, 3, False)),
    ("the_unlinked_marking_falls_on_exactly_the_unavailable_schemes",
     (('s3://',), ('s3://',), (), ())),
    ("the_marking_is_appended_to_the_line_rather_than_replacing_it",
     (True, True, 67)),
    ("the_same_scheme_reads_differently_in_the_two_builds",
     ('s3://  Amazon S3-compatible object store', 's3://  Amazon S3-compatible object store (not linked in this build)', False)),
    ("an_available_scheme_never_carries_the_marking",
     (False, False, False, False)),
    ("no_rendered_line_is_blank",
     (40, 61, (False, False, False))),
    ("the_lookup_and_the_section_agree_on_availability",
     ((True, False, True), (True, False, True), (True, True, True), (True, True, True))),
    ("the_description_text_is_carried_verbatim_from_the_inventory",
     (True, True, True, 'Amazon S3-compatible object store')),
    ("the_default_build_marks_the_object_store_as_unlinked",
     (True, ('s3://',), False)),
    ("the_lookup_refuses_a_scheme_outside_the_inventory",
     (None, None, None, 's3://')),
)


LINKED = BuildFeatures(s3=True)


UNLINKED = BuildFeatures(s3=False)


MARK = "(not linked in this build)"


def verify_non_drifting_scheme_documentation_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. every inventory scheme and description appears in the section
    exp1 = NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX[0][1]
    obs1 = (all(s.scheme in topic_destination_section(LINKED)
        for s in supported_schemes(LINKED)),
        all(s.scheme in topic_destination_section(UNLINKED)
        for s in supported_schemes(UNLINKED)),
        all(s.description in topic_destination_section(UNLINKED)
        for s in supported_schemes(UNLINKED)),
        len(supported_schemes(LINKED)))
    checks.append({"name": NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. the section names no scheme the inventory does not
    exp2 = NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX[1][1]
    obs2 = (topic_destination_section(LINKED).count("://"),
        topic_destination_section(UNLINKED).count("://"),
        "ftp://" in topic_destination_section(LINKED))
    checks.append({"name": NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the unlinked marking falls on exactly the unavailable schemes
    exp3 = NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX[2][1]
    obs3 = (tuple(line.split("  ")[0]
        for line in topic_destination_section(UNLINKED).split("\n")
        if MARK in line),
        unavailable_schemes(UNLINKED),
        tuple(line.split("  ")[0]
        for line in topic_destination_section(LINKED).split("\n")
        if MARK in line),
        unavailable_schemes(LINKED))
    checks.append({"name": NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. the marking is appended to the line rather than replacing it
    exp4 = NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX[3][1]
    obs4 = (topic_destination_section(UNLINKED).split("\n")[1].startswith(
        "s3://  Amazon S3-compatible object store"),
        topic_destination_section(UNLINKED).split("\n")[1].endswith(MARK),
        len(topic_destination_section(UNLINKED).split("\n")[1]))
    checks.append({"name": NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the same scheme reads differently in the two builds
    exp5 = NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX[4][1]
    obs5 = (topic_destination_section(LINKED).split("\n")[1],
        topic_destination_section(UNLINKED).split("\n")[1],
        topic_destination_section(LINKED).split("\n")[1]
        == topic_destination_section(UNLINKED).split("\n")[1])
    checks.append({"name": NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. an available scheme never carries the marking
    exp6 = NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX[5][1]
    obs6 = (MARK in topic_destination_section(LINKED).split("\n")[0],
        MARK in topic_destination_section(UNLINKED).split("\n")[0],
        MARK in topic_destination_section(UNLINKED).split("\n")[2],
        MARK in topic_destination_section(LINKED).split("\n")[1])
    checks.append({"name": NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. no rendered line is blank
    exp7 = NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX[6][1]
    obs7 = (min(len(line) for line in topic_destination_section(LINKED).split("\n")),
        min(len(line) for line in topic_destination_section(UNLINKED).split("\n")),
        tuple(line == "" for line in topic_destination_section(UNLINKED).split("\n")))
    checks.append({"name": NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. the lookup and the section agree on availability
    exp8 = NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX[7][1]
    obs8 = (tuple(s.sink_available for s in supported_schemes(UNLINKED)),
        tuple(MARK not in line
        for line in topic_destination_section(UNLINKED).split("\n")),
        tuple(s.sink_available for s in supported_schemes(LINKED)),
        tuple(MARK not in line
        for line in topic_destination_section(LINKED).split("\n")))
    checks.append({"name": NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the description text is carried verbatim from the inventory
    exp9 = NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX[8][1]
    obs9 = (supported_schemes(LINKED)[0].description in topic_destination_section(LINKED),
        supported_schemes(LINKED)[1].description in topic_destination_section(UNLINKED),
        supported_schemes(LINKED)[2].description in topic_destination_section(UNLINKED),
        supported_schemes(UNLINKED)[1].description)
    checks.append({"name": NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the default build marks the object store as unlinked
    exp10 = NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX[9][1]
    obs10 = (MARK in topic_destination_section(BuildFeatures()),
        unavailable_schemes(BuildFeatures()), BuildFeatures().s3)
    checks.append({"name": NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. the lookup refuses a scheme outside the inventory
    exp11 = NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX[10][1]
    obs11 = (find_scheme("ftp://", LINKED), find_scheme("", LINKED),
        find_scheme("s3", LINKED), find_scheme("s3://", LINKED).scheme)
    checks.append({"name": NON_DRIFTING_SCHEME_DOCUMENTATION_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    return {
        "case_id": "non-drifting-scheme-documentation-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
