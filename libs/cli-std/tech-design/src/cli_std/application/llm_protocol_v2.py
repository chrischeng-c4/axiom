from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from enum import Enum

from cli_std.application.errors import ProtocolInvalid, StepInvalid
from cli_std.application.llm import Format
from cli_std.domain.errors import UnknownTopic

PROTOCOL: str = "cclab.llm.v2"


class Risk(Enum):
    INSPECT = "inspect"
    LOCAL_WRITE = "local_write"
    REMOTE_WRITE = "remote_write"


@dataclass(frozen=True)
class Task:
    id: str
    use_when: str
    requires: tuple[str, ...]
    reads: tuple[str, ...]
    produces: tuple[str, ...]
    risk: Risk
    topic: str
    contract_refs: tuple[str, ...]


@dataclass(frozen=True)
class Input:
    name: str
    value_type: str
    description: str
    required: bool


@dataclass(frozen=True)
class Step:
    id: str
    instruction: str
    command: str | None
    command_template: str | None
    inputs: tuple[Input, ...]


@dataclass(frozen=True)
class Runbook:
    purpose: str
    preconditions: tuple[str, ...]
    inputs: tuple[Input, ...]
    constraints: tuple[str, ...]
    steps: tuple[Step, ...]
    verification: tuple[str, ...]
    references: tuple[str, ...]


@dataclass(frozen=True)
class ProtocolTopic:
    task: Task
    runbook: Runbook


@dataclass(frozen=True)
class ProtocolDocument:
    project: str
    topics: tuple[ProtocolTopic, ...]


def template_placeholders(template: str) -> frozenset[str] | StepInvalid:
    placeholders: set[str] = set()
    remaining = template
    allowed_chars = set(
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-"
    )
    while "{" in remaining:
        open_at = remaining.index("{")
        after = remaining[open_at + 1 :]
        if "}" not in after:
            return StepInvalid(step_id="", reason="unclosed placeholder")
        close_at = after.index("}")
        name = after[:close_at]
        if not name or not all(c in allowed_chars for c in name):
            return StepInvalid(step_id="", reason="invalid placeholder")
        placeholders.add(name)
        remaining = after[close_at + 1 :]

    if "}" in remaining:
        return StepInvalid(step_id="", reason="closing brace")

    return frozenset(placeholders)


def validate_step(step: Step) -> StepInvalid | None:
    if step.command is not None and step.command_template is not None:
        return StepInvalid(
            step.id, "cannot contain both command and command_template"
        )

    if step.command is not None:
        if step.inputs or "<" in step.command or "{" in step.command:
            return StepInvalid(
                step.id,
                "command must be fully bound; use command_template with typed inputs",
            )

    if step.command_template is not None and not step.inputs:
        return StepInvalid(
            step.id, "command_template requires typed inputs"
        )

    if step.command_template is not None:
        declared = {i.name for i in step.inputs}
        referenced = template_placeholders(step.command_template)
        if isinstance(referenced, StepInvalid):
            return StepInvalid(step.id, referenced.reason)
        if declared != referenced:
            return StepInvalid(
                step.id,
                "command_template placeholders must exactly match its typed inputs",
            )

    return None


def validate_document(
    project: str, topics: Sequence[ProtocolTopic]
) -> ProtocolDocument | ProtocolInvalid | StepInvalid:
    if project.strip() == "":
        return ProtocolInvalid("project cannot be empty")

    seen_ids: set[str] = set()
    seen_topics: set[str] = set()

    for entry in topics:
        if entry.task.id.strip() == "" or entry.task.topic.strip() == "":
            return ProtocolInvalid("task id and topic are both required")
        if entry.task.id in seen_ids:
            return ProtocolInvalid(f"duplicate task id '{entry.task.id}'")
        if entry.task.topic in seen_topics:
            return ProtocolInvalid(f"duplicate topic '{entry.task.topic}'")

        seen_ids.add(entry.task.id)
        seen_topics.add(entry.task.topic)

        for step in entry.runbook.steps:
            problem = validate_step(step)
            if problem is not None:
                return problem

    return ProtocolDocument(project, tuple(topics))


def task_json(task: Task) -> dict[str, object]:
    return {
        "id": task.id,
        "use_when": task.use_when,
        "requires": list(task.requires),
        "reads": list(task.reads),
        "produces": list(task.produces),
        "risk": task.risk.value,
        "topic": task.topic,
        "contract_refs": list(task.contract_refs),
    }


def input_json(value: Input) -> dict[str, object]:
    return {
        "name": value.name,
        "type": value.value_type,
        "description": value.description,
        "required": value.required,
    }


def step_json(step: Step) -> dict[str, object]:
    res: dict[str, object] = {
        "id": step.id,
        "instruction": step.instruction,
    }
    if step.command is not None:
        res["command"] = step.command
    if step.command_template is not None:
        res["command_template"] = step.command_template
    if step.inputs:
        res["inputs"] = [input_json(i) for i in step.inputs]
    return res


def runbook_json(runbook: Runbook) -> dict[str, object]:
    return {
        "purpose": runbook.purpose,
        "preconditions": list(runbook.preconditions),
        "inputs": [input_json(i) for i in runbook.inputs],
        "constraints": list(runbook.constraints),
        "steps": [step_json(s) for s in runbook.steps],
        "verification": list(runbook.verification),
        "references": list(runbook.references),
    }


def outline_markdown_v2(document: ProtocolDocument) -> str:
    lines = [
        f"# {document.project} task navigation",
        "",
        "Select the smallest task, then read its typed runbook:",
        "",
    ]
    for entry in document.topics:
        t = entry.task
        lines.append(
            f"- `{t.id}` — {t.use_when} (`{document.project} llm --topic {t.topic}`)"
        )
    lines.extend(
        [
            "",
            f"Use `{document.project} llm --topic <topic> --format json` for cclab.llm.v2 data.",
        ]
    )
    return "\n".join(lines)


def topic_markdown_v2(document: ProtocolDocument, entry: ProtocolTopic) -> str:
    t = entry.task
    r = entry.runbook
    lines = [
        f"# {document.project} — {t.id}",
        "",
        r.purpose,
    ]

    if r.preconditions:
        lines.extend(["", "## Preconditions", ""])
        for item in r.preconditions:
            lines.append(f"- {item}")

    if r.inputs:
        lines.extend(["", "## Inputs", ""])
        for inp in r.inputs:
            req_str = "required" if inp.required else "optional"
            lines.append(
                f"- `{inp.name}` ({inp.value_type}, {req_str}) — {inp.description}"
            )

    if r.constraints:
        lines.extend(["", "## Constraints", ""])
        for item in r.constraints:
            lines.append(f"- {item}")

    if r.steps:
        lines.extend(["", "## Steps", ""])
        for i, s in enumerate(r.steps, 1):
            lines.append(f"{i}. {s.instruction}")
            if s.command is not None:
                lines.append(f"   `{s.command}`")
            elif s.command_template is not None:
                lines.append(f"   Template: `{s.command_template}`")

    if r.verification:
        lines.extend(["", "## Verification", ""])
        for item in r.verification:
            lines.append(f"- {item}")

    if r.references:
        lines.extend(["", "## References", ""])
        for item in r.references:
            lines.append(f"- {item}")

    return "\n".join(lines)


def render_document(
    document: ProtocolDocument, topic: str, format: Format
) -> str | dict[str, object] | UnknownTopic:
    if topic == "outline":
        markdown = outline_markdown_v2(document)
        if format == Format.MD:
            return markdown
        return {
            "topic": "outline",
            "markdown": markdown,
            "protocol": PROTOCOL,
            "tasks": [task_json(e.task) for e in document.topics],
        }

    found: ProtocolTopic | None = None
    for entry in document.topics:
        if entry.task.topic == topic:
            found = entry
            break

    if found is None:
        return UnknownTopic(
            topic, tuple(e.task.topic for e in document.topics)
        )

    markdown = topic_markdown_v2(document, found)
    if format == Format.MD:
        return markdown

    return {
        "topic": topic,
        "markdown": markdown,
        "protocol": PROTOCOL,
        "task": task_json(found.task),
        "runbook": runbook_json(found.runbook),
    }
