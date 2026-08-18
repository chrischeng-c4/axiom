from __future__ import annotations

from service_auth.application.reload_registry import (
    ReloadableRegistry,
    reload_documents,
)
from service_auth.domain.audit import ReloadFailure
from service_auth.infrastructure.ports import AuthEventSink, RegistrySource


class TextSource:
    def __init__(self, name: str, text: str) -> None:
        self.name = name
        self._text = text

    def read(self) -> str:
        return self._text


class BrokenSource:
    def __init__(self, name: str) -> None:
        self.name = name

    def read(self) -> str:
        raise OSError("unreadable")


class ListSink:
    def __init__(self) -> None:
        self.events: list[object] = []

    def record(self, event: object) -> None:
        self.events.append(event)


GOOD_A = '{"tokens": {"secret-a": {"subject": "alice", "roles": {"docs": "read"}}}}'
GOOD_B = '{"tokens": {"secret-b": {"subject": "bob", "roles": {"docs": "write"}}}}'
DUP_A = '{"tokens": {"secret-a": {"subject": "carol", "roles": {"docs": "read"}}}}'
NOT_JSON = 'this is not json'
UNKNOWN_ROLE = (
    '{"tokens": {"secret-c": {"subject": "dan", "roles": {"docs": "wizard"}}}}'
)
BLANK_SUBJECT = (
    '{"tokens": {"secret-d": {"subject": "   ", "roles": {"docs": "read"}}}}'
)
RESERVED = '{"tokens": {"secret-e": {"subject": "system:masters", "roles": {"docs": "read"}}}}'

MINIMUM_CHECKS = 15

CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX = (
    ("a_reload_of_two_good_sources_publishes_both_entries", ("applied", 1, 2)),
    ("a_successful_reload_records_one_applied_event", (1, True, 1, 2, "none")),
    ("an_unreadable_source_classifies_as_a_read_failure", "read"),
    ("a_document_that_is_not_json_classifies_as_a_parse_failure", "parse"),
    (
        "a_document_naming_an_unknown_role_classifies_as_a_parse_failure",
        "parse",
    ),
    (
        "the_same_key_from_two_sources_classifies_as_an_invalid_failure",
        "invalid",
    ),
    (
        "a_registry_failing_validation_classifies_as_an_invalid_failure",
        "invalid",
    ),
    ("a_reserved_subject_classifies_as_an_invalid_failure", "invalid"),
    (
        "a_validation_failure_leaves_the_published_registry_and_revision_untouched",
        (1, 2),
    ),
    (
        "a_read_failure_publishes_nothing_from_the_prefix_it_had_merged",
        (0, 0),
    ),
    (
        "a_failed_reload_records_one_event_that_is_not_applied",
        (1, False, 0, "read"),
    ),
    (
        "the_event_of_a_failed_reload_reports_the_revision_still_in_force",
        1,
    ),
    ("a_reload_of_no_sources_publishes_an_empty_registry", ("applied", 1, 0)),
    ("four_attempts_record_four_events", (4, 2, 2)),
    (
        "the_reload_failure_classification_has_exactly_three_values",
        ("read", "parse", "invalid"),
    ),
)


def verify_credential_reload_audit_behavior() -> dict:
    checks = []

    # 1. a_reload_of_two_good_sources_publishes_both_entries
    exp1 = CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[0][1]
    st1 = ReloadableRegistry()
    snk1 = ListSink()
    res1 = reload_documents(
        st1, [TextSource("a", GOOD_A), TextSource("b", GOOD_B)], snk1
    )
    obs1 = (
        res1.value if res1 is not None else "applied",
        st1.revision,
        st1.registry.len(),
    )
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. a_successful_reload_records_one_applied_event
    exp2 = CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[1][1]
    e2 = snk1.events[0]
    obs2 = (
        len(snk1.events),
        getattr(e2, "applied", False),
        getattr(e2, "revision", 0),
        getattr(e2, "entries", 0),
        "none" if getattr(e2, "failure", None) is None else getattr(e2, "failure").value,
    )
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. an_unreadable_source_classifies_as_a_read_failure
    exp3 = CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[2][1]
    st3 = ReloadableRegistry()
    snk3 = ListSink()
    res3 = reload_documents(st3, [BrokenSource("x")], snk3)
    obs3 = res3.value if res3 is not None else "applied"
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. a_document_that_is_not_json_classifies_as_a_parse_failure
    exp4 = CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[3][1]
    st4 = ReloadableRegistry()
    snk4 = ListSink()
    res4 = reload_documents(st4, [TextSource("x", NOT_JSON)], snk4)
    obs4 = res4.value if res4 is not None else "applied"
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. a_document_naming_an_unknown_role_classifies_as_a_parse_failure
    exp5 = CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[4][1]
    st5 = ReloadableRegistry()
    snk5 = ListSink()
    res5 = reload_documents(st5, [TextSource("x", UNKNOWN_ROLE)], snk5)
    obs5 = res5.value if res5 is not None else "applied"
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. the_same_key_from_two_sources_classifies_as_an_invalid_failure
    exp6 = CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[5][1]
    st6 = ReloadableRegistry()
    snk6 = ListSink()
    res6 = reload_documents(
        st6, [TextSource("a", GOOD_A), TextSource("b", DUP_A)], snk6
    )
    obs6 = res6.value if res6 is not None else "applied"
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. a_registry_failing_validation_classifies_as_an_invalid_failure
    exp7 = CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[6][1]
    st7 = ReloadableRegistry()
    snk7 = ListSink()
    res7 = reload_documents(st7, [TextSource("x", BLANK_SUBJECT)], snk7)
    obs7 = res7.value if res7 is not None else "applied"
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. a_reserved_subject_classifies_as_an_invalid_failure
    exp8 = CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[7][1]
    st8 = ReloadableRegistry(reserved=("system:masters",))
    snk8 = ListSink()
    res8 = reload_documents(st8, [TextSource("x", RESERVED)], snk8)
    obs8 = res8.value if res8 is not None else "applied"
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. a_validation_failure_leaves_the_published_registry_and_revision_untouched
    exp9 = CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[8][1]
    st9 = ReloadableRegistry()
    snk9 = ListSink()
    reload_documents(
        st9, [TextSource("a", GOOD_A), TextSource("b", GOOD_B)], snk9
    )
    reload_documents(st9, [TextSource("x", BLANK_SUBJECT)], snk9)
    obs9 = (st9.revision, st9.registry.len())
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. a_read_failure_publishes_nothing_from_the_prefix_it_had_merged
    exp10 = CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[9][1]
    st10 = ReloadableRegistry()
    snk10 = ListSink()
    reload_documents(
        st10, [TextSource("a", GOOD_A), BrokenSource("b")], snk10
    )
    obs10 = (st10.revision, st10.registry.len())
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. a_failed_reload_records_one_event_that_is_not_applied
    exp11 = CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[10][1]
    e11 = snk10.events[0]
    obs11 = (
        len(snk10.events),
        getattr(e11, "applied", True),
        getattr(e11, "entries", -1),
        getattr(e11, "failure").value if getattr(e11, "failure", None) is not None else "none",
    )
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. the_event_of_a_failed_reload_reports_the_revision_still_in_force
    exp12 = CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[11][1]
    e12 = snk9.events[-1]
    obs12 = getattr(e12, "revision", -1)
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. a_reload_of_no_sources_publishes_an_empty_registry
    exp13 = CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[12][1]
    st13 = ReloadableRegistry()
    snk13 = ListSink()
    res13 = reload_documents(st13, [], snk13)
    obs13 = (
        res13.value if res13 is not None else "applied",
        st13.revision,
        st13.registry.len(),
    )
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    # 14. four_attempts_record_four_events
    exp14 = CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[13][1]
    snk14 = ListSink()
    reload_documents(ReloadableRegistry(), [TextSource("a", GOOD_A)], snk14)
    reload_documents(ReloadableRegistry(), [TextSource("b", GOOD_B)], snk14)
    reload_documents(ReloadableRegistry(), [BrokenSource("x")], snk14)
    reload_documents(ReloadableRegistry(), [TextSource("x", NOT_JSON)], snk14)
    obs14 = (
        len(snk14.events),
        sum(1 for e in snk14.events if getattr(e, "applied", False)),
        sum(1 for e in snk14.events if not getattr(e, "applied", False)),
    )
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[13][0],
            "expected": exp14,
            "observed": obs14,
            "passed": obs14 == exp14,
        }
    )

    # 15. the_reload_failure_classification_has_exactly_three_values
    exp15 = CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[14][1]
    obs15 = tuple(f.value for f in ReloadFailure)
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_BEHAVIOR_MATRIX[14][0],
            "expected": exp15,
            "observed": obs15,
            "passed": obs15 == exp15,
        }
    )

    return {
        "case_id": "credential-reload-audit-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
