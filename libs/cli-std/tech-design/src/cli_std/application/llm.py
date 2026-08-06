from __future__ import annotations

from collections.abc import Callable, Sequence
from dataclasses import dataclass
from enum import Enum

from cli_std.application.errors import ProtocolInvalid
from cli_std.domain.errors import UnknownTopic


@dataclass(frozen=True)
class Topic:
    id: str
    summary: str
    body: str


@dataclass(frozen=True)
class ProseSection:
    text: str


@dataclass(frozen=True)
class GeneratedSection:
    id: str
    render: Callable[[], str]


TopicSection = ProseSection | GeneratedSection


@dataclass(frozen=True)
class SectionedTopic:
    id: str
    summary: str
    sections: tuple[TopicSection, ...]


@dataclass(frozen=True)
class RenderedSection:
    id: str
    kind: str
    content: str


class Format(Enum):
    MD = "md"
    JSON = "json"

    @staticmethod
    def parse(text: str) -> Format:
        if text.lower() == "json":
            return Format.JSON
        return Format.MD


def render_sections(
    topic: Topic | SectionedTopic,
) -> tuple[RenderedSection, ...]:
    if isinstance(topic, Topic):
        return (RenderedSection(id="body", kind="prose", content=topic.body),)

    res: list[RenderedSection] = []
    for i, sec in enumerate(topic.sections):
        if isinstance(sec, ProseSection):
            res.append(
                RenderedSection(id=f"prose-{i}", kind="prose", content=sec.text)
            )
        else:
            res.append(
                RenderedSection(
                    id=sec.id, kind="generated", content=sec.render()
                )
            )
    return tuple(res)


def join_sections(sections: Sequence[RenderedSection]) -> str:
    return "\n\n".join(sec.content for sec in sections)


def outline_markdown(
    project: str, topics: Sequence[Topic | SectionedTopic]
) -> str:
    lines = [
        f"# {project} — agent topic outline",
        "",
        f"Run `{project} llm --topic <topic>` for detail (add `--format json` for a machine-readable form).",
        "",
        "## Topics",
        "",
    ]
    for t in topics:
        lines.append(f"- `{t.id}` — {t.summary}")
    lines.extend(
        [
            "",
            "## Standard agent commands",
            "",
            f"- `{project} llm [--topic <t>] [--format md|json]` — this self-documentation (offline)",
            f"- `{project} upgrade [--version <tag>] [--check]` — self-update from GitHub releases",
        ]
    )
    return "\n".join(lines)


def render(
    project: str,
    version: str,
    topics: Sequence[Topic],
    topic: str,
    format: Format,
) -> str | dict[str, object] | UnknownTopic:
    if topic == "outline":
        if format == Format.MD:
            return outline_markdown(project, topics)
        return {
            "project": project,
            "version": version,
            "topics": [
                {"id": t.id, "summary": t.summary} for t in topics
            ],
        }

    found: Topic | None = None
    for t in topics:
        if t.id == topic:
            found = t
            break

    if found is None:
        return UnknownTopic(topic, tuple(t.id for t in topics))

    if format == Format.MD:
        return found.body
    return {
        "project": project,
        "topic": found.id,
        "summary": found.summary,
        "body": found.body,
    }


def render_sectioned(
    project: str,
    version: str,
    topics: Sequence[SectionedTopic],
    topic: str,
    format: Format,
) -> str | dict[str, object] | UnknownTopic:
    if topic == "outline":
        if format == Format.MD:
            return outline_markdown(project, topics)
        return {
            "project": project,
            "version": version,
            "topics": [
                {"id": t.id, "summary": t.summary} for t in topics
            ],
        }

    found: SectionedTopic | None = None
    for t in topics:
        if t.id == topic:
            found = t
            break

    if found is None:
        return UnknownTopic(topic, tuple(t.id for t in topics))

    rendered_secs = render_sections(found)
    detail_body = join_sections(rendered_secs)
    if format == Format.MD:
        return detail_body

    sec_dicts = [
        {"id": r.id, "kind": r.kind, "content": r.content}
        for r in rendered_secs
    ]
    return {
        "project": project,
        "topic": found.id,
        "summary": found.summary,
        "body": detail_body,
        "sections": sec_dicts,
    }


def assert_topics_render(
    topics: Sequence[Topic | SectionedTopic],
) -> ProtocolInvalid | None:
    seen_gen_ids: set[str] = set()

    for t in topics:
        rendered = render_sections(t)
        joined = join_sections(rendered)
        if joined.strip() == "":
            return ProtocolInvalid(
                f"topic '{t.id}' rendered empty output"
            )

        if isinstance(t, SectionedTopic):
            for sec in t.sections:
                if isinstance(sec, GeneratedSection):
                    sec_content = sec.render()
                    if sec_content.strip() == "":
                        return ProtocolInvalid(
                            f"generated section '{sec.id}' in topic '{t.id}' produced empty output"
                        )
                    if sec.id in seen_gen_ids:
                        return ProtocolInvalid(
                            f"generated section id '{sec.id}' in topic '{t.id}' is not unique"
                        )
                    seen_gen_ids.add(sec.id)

    return None
