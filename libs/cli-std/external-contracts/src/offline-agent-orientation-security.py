from __future__ import annotations

from cli_std.application.errors import ProtocolInvalid
from cli_std.application.llm import (
    Format,
    GeneratedSection,
    ProseSection,
    SectionedTopic,
    Topic,
    assert_topics_render,
    render,
    render_sectioned,
)
from cli_std.domain.errors import UnknownTopic

MINIMUM_CHECKS = 11

OFFLINE_AGENT_ORIENTATION_SECURITY_MATRIX = [
    ("format_parse_unknown_defaults_to_markdown", "md"),
    ("format_parse_uppercase_md", "md"),
    ("unknown_topic_returns_unknown_topic_error", ("unknown", ("alpha",))),
    ("prefix_topic_name_does_not_match_exact", ("alp", ("alpha",))),
    ("sectioned_unknown_topic_returns_error", ("alp", ("alpha",))),
    ("empty_topic_refused_by_assert_topics_render", "topic 't1' rendered empty output"),
    ("empty_generated_section_refused", "generated section 'g1' in topic 't2' produced empty output"),
    ("duplicate_generated_section_id_across_topics_refused", "generated section id 'g1' in topic 't2' is not unique"),
    ("valid_topics_pass_assert_topics_render", None),
    ("json_detail_contains_body", {"project": "app", "topic": "t1", "summary": "sum", "body": "body_content"}),
    ("unknown_topic_lists_all_known_ids", ("t3", ("t1", "t2"))),
]


def verify_offline_agent_orientation_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    fmt0 = Format.parse("unknown")
    c0 = fmt0.value if isinstance(fmt0, Format) else None
    checks.append({"name": "format_parse_unknown_defaults_to_markdown", "passed": c0 == "md"})

    fmt1 = Format.parse("MD")
    c1 = fmt1.value if isinstance(fmt1, Format) else None
    checks.append({"name": "format_parse_uppercase_md", "passed": c1 == "md"})

    res2 = render("app", "1.0.0", [Topic("alpha", "a", "body")], "unknown", Format.MD)
    c2 = (res2.topic, res2.known) if isinstance(res2, UnknownTopic) else None
    checks.append({"name": "unknown_topic_returns_unknown_topic_error", "passed": c2 == ("unknown", ("alpha",))})

    res3 = render("app", "1.0.0", [Topic("alpha", "a", "body")], "alp", Format.MD)
    c3 = (res3.topic, res3.known) if isinstance(res3, UnknownTopic) else None
    checks.append({"name": "prefix_topic_name_does_not_match_exact", "passed": c3 == ("alp", ("alpha",))})

    res4 = render_sectioned("app", "1.0.0", [SectionedTopic("alpha", "a", (ProseSection("p"),))], "alp", Format.MD)
    c4 = (res4.topic, res4.known) if isinstance(res4, UnknownTopic) else None
    checks.append({"name": "sectioned_unknown_topic_returns_error", "passed": c4 == ("alp", ("alpha",))})

    res5 = assert_topics_render([Topic("t1", "s1", "  ")])
    c5 = res5.reason if isinstance(res5, ProtocolInvalid) else None
    checks.append({"name": "empty_topic_refused_by_assert_topics_render", "passed": c5 == "topic 't1' rendered empty output"})

    res6 = assert_topics_render([SectionedTopic("t2", "s2", (ProseSection("Non-empty prose"), GeneratedSection("g1", lambda: " ")))])
    c6 = res6.reason if isinstance(res6, ProtocolInvalid) else None
    checks.append({"name": "empty_generated_section_refused", "passed": c6 == "generated section 'g1' in topic 't2' produced empty output"})

    res7 = assert_topics_render([
        SectionedTopic("t1", "s1", (GeneratedSection("g1", lambda: "A"),)),
        SectionedTopic("t2", "s2", (GeneratedSection("g1", lambda: "B"),)),
    ])
    c7 = res7.reason if isinstance(res7, ProtocolInvalid) else None
    checks.append({"name": "duplicate_generated_section_id_across_topics_refused", "passed": c7 == "generated section id 'g1' in topic 't2' is not unique"})

    res8 = assert_topics_render([Topic("t1", "s1", "body")])
    c8 = res8.reason if isinstance(res8, ProtocolInvalid) else None
    checks.append({"name": "valid_topics_pass_assert_topics_render", "passed": c8 is None})

    c9 = render("app", "1.0.0", [Topic("t1", "sum", "body_content")], "t1", Format.JSON)
    checks.append({"name": "json_detail_contains_body", "passed": c9 == {"project": "app", "topic": "t1", "summary": "sum", "body": "body_content"}})

    res10 = render("app", "1.0.0", [Topic("t1", "s1", "b1"), Topic("t2", "s2", "b2")], "t3", Format.MD)
    c10 = (res10.topic, res10.known) if isinstance(res10, UnknownTopic) else None
    checks.append({"name": "unknown_topic_lists_all_known_ids", "passed": c10 == ("t3", ("t1", "t2"))})

    return {
        "case_id": "offline-agent-orientation-security",
        "minimum_checks": MINIMUM_CHECKS,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
        "checks": checks,
    }
