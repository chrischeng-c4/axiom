from __future__ import annotations

from cli_std.application.llm import (
    Format,
    GeneratedSection,
    ProseSection,
    SectionedTopic,
    Topic,
    join_sections,
    outline_markdown,
    render,
    render_sectioned,
    render_sections,
)

MINIMUM_CHECKS = 12

OFFLINE_AGENT_ORIENTATION_BEHAVIOR_MATRIX = [
    ("format_parse_case_insensitive", "json"),
    ("topic_render_sections_kind_and_id", (("body", "prose", "Body 1"),)),
    ("sectioned_render_sections_ids_and_kinds", (("prose-0", "prose", "P1"), ("g1", "generated", "G1"))),
    ("join_sections_double_newline", "A\n\nB"),
    (
        "outline_markdown_exact_string",
        "# app — agent topic outline\n\nRun `app llm --topic <topic>` for detail (add `--format json` for a machine-readable form).\n\n## Topics\n\n- `t1` — Sum 1\n\n## Standard agent commands\n\n- `app llm [--topic <t>] [--format md|json]` — this self-documentation (offline)\n- `app upgrade [--version <tag>] [--check]` — self-update from GitHub releases",
    ),
    (
        "render_outline_json",
        {
            "project": "app",
            "version": "1.0.0",
            "topics": [{"id": "t1", "summary": "Sum 1"}],
        },
    ),
    ("render_topic_markdown", "Body 1"),
    (
        "render_topic_json",
        {
            "project": "app",
            "topic": "t1",
            "summary": "Sum 1",
            "body": "Body 1",
        },
    ),
    ("render_sectioned_markdown", "P1\n\nG1"),
    (
        "render_sectioned_json",
        {
            "project": "app",
            "topic": "t2",
            "summary": "Sum 2",
            "body": "P1\n\nG1",
            "sections": [
                {"id": "prose-0", "kind": "prose", "content": "P1"},
                {"id": "g1", "kind": "generated", "content": "G1"},
            ],
        },
    ),
    (
        "render_sectioned_outline_json",
        {
            "project": "app",
            "version": "1.0.0",
            "topics": [{"id": "t2", "summary": "Sum 2"}],
        },
    ),
    (
        "outline_markdown_sectioned_exact",
        "# app — agent topic outline\n\nRun `app llm --topic <topic>` for detail (add `--format json` for a machine-readable form).\n\n## Topics\n\n- `t2` — Sum 2\n\n## Standard agent commands\n\n- `app llm [--topic <t>] [--format md|json]` — this self-documentation (offline)\n- `app upgrade [--version <tag>] [--check]` — self-update from GitHub releases",
    ),
]


def verify_offline_agent_orientation_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    t1 = Topic(id="t1", summary="Sum 1", body="Body 1")
    st2 = SectionedTopic(
        id="t2",
        summary="Sum 2",
        sections=(ProseSection("P1"), GeneratedSection("g1", lambda: "G1")),
    )

    fmt0 = Format.parse("JSON")
    c0 = fmt0.value if isinstance(fmt0, Format) else None
    checks.append({"name": "format_parse_case_insensitive", "passed": c0 == "json"})

    secs1 = render_sections(t1)
    c1 = tuple((s.id, s.kind, s.content) for s in secs1)
    checks.append({"name": "topic_render_sections_kind_and_id", "passed": c1 == (("body", "prose", "Body 1"),)})

    secs2 = render_sections(st2)
    c2 = tuple((s.id, s.kind, s.content) for s in secs2)
    checks.append({"name": "sectioned_render_sections_ids_and_kinds", "passed": c2 == (("prose-0", "prose", "P1"), ("g1", "generated", "G1"))})

    c3 = join_sections(render_sections(SectionedTopic("t", "s", (ProseSection("A"), ProseSection("B")))))
    checks.append({"name": "join_sections_double_newline", "passed": c3 == "A\n\nB"})

    c4 = outline_markdown("app", [t1])
    expected_outline = (
        "# app — agent topic outline\n\n"
        "Run `app llm --topic <topic>` for detail (add `--format json` for a machine-readable form).\n\n"
        "## Topics\n\n"
        "- `t1` — Sum 1\n\n"
        "## Standard agent commands\n\n"
        "- `app llm [--topic <t>] [--format md|json]` — this self-documentation (offline)\n"
        "- `app upgrade [--version <tag>] [--check]` — self-update from GitHub releases"
    )
    checks.append({"name": "outline_markdown_exact_string", "passed": c4 == expected_outline})

    c5 = render("app", "1.0.0", [t1], "outline", Format.JSON)
    checks.append({"name": "render_outline_json", "passed": c5 == {"project": "app", "version": "1.0.0", "topics": [{"id": "t1", "summary": "Sum 1"}]}})

    c6 = render("app", "1.0.0", [t1], "t1", Format.MD)
    checks.append({"name": "render_topic_markdown", "passed": c6 == "Body 1"})

    c7 = render("app", "1.0.0", [t1], "t1", Format.JSON)
    checks.append({"name": "render_topic_json", "passed": c7 == {"project": "app", "topic": "t1", "summary": "Sum 1", "body": "Body 1"}})

    c8 = render_sectioned("app", "1.0.0", [st2], "t2", Format.MD)
    checks.append({"name": "render_sectioned_markdown", "passed": c8 == "P1\n\nG1"})

    c9 = render_sectioned("app", "1.0.0", [st2], "t2", Format.JSON)
    checks.append({"name": "render_sectioned_json", "passed": c9 == {"project": "app", "topic": "t2", "summary": "Sum 2", "body": "P1\n\nG1", "sections": [{"id": "prose-0", "kind": "prose", "content": "P1"}, {"id": "g1", "kind": "generated", "content": "G1"}]}})

    c10 = render_sectioned("app", "1.0.0", [st2], "outline", Format.JSON)
    checks.append({"name": "render_sectioned_outline_json", "passed": c10 == {"project": "app", "version": "1.0.0", "topics": [{"id": "t2", "summary": "Sum 2"}]}})

    c11 = render_sectioned("app", "1.0.0", [st2], "outline", Format.MD)
    expected_st_outline = (
        "# app — agent topic outline\n\n"
        "Run `app llm --topic <topic>` for detail (add `--format json` for a machine-readable form).\n\n"
        "## Topics\n\n"
        "- `t2` — Sum 2\n\n"
        "## Standard agent commands\n\n"
        "- `app llm [--topic <t>] [--format md|json]` — this self-documentation (offline)\n"
        "- `app upgrade [--version <tag>] [--check]` — self-update from GitHub releases"
    )
    checks.append({"name": "outline_markdown_sectioned_exact", "passed": c11 == expected_st_outline})

    return {
        "case_id": "offline-agent-orientation-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
        "checks": checks,
    }
