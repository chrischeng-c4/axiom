from __future__ import annotations

from collections.abc import Sequence
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from cli_std.domain.tool_identity import ToolInfo


def percent_encode_query(text: str) -> str:
    unreserved = set(
        b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_.~"
    )
    res: list[str] = []
    for b in text.encode("utf-8"):
        if b in unreserved:
            res.append(chr(b))
        else:
            res.append(f"%{b:02X}")
    return "".join(res)


def render_diagnostics(
    tool: ToolInfo, os_name: str, arch: str, node: str | None
) -> str:
    lines = [
        "## Diagnostics",
        f"- {tool.project} version: {tool.version}",
        f"- target: {tool.target}",
        f"- git sha: {tool.git_sha}",
        f"- built at: {tool.built_at}",
        f"- os/arch: {os_name}/{arch}",
    ]
    if node is not None:
        lines.append(f"- node: {node}")
    return "\n".join(lines) + "\n"


def assemble_body(message: str | None, diagnostics: str) -> str:
    if message is not None and message.strip() != "":
        return f"{message.strip()}\n\n---\n{diagnostics}"
    return diagnostics


def resolve_repo(tool: ToolInfo, repo: str | None) -> str:
    if repo is None:
        return tool.repo
    return repo


def issue_payload(
    title: str, body: str, labels: Sequence[str]
) -> dict[str, object]:
    res: dict[str, object] = {
        "title": title,
        "body": body,
    }
    if labels:
        res["labels"] = list(labels)
    return res


def report_labels(
    tool: ToolInfo, labels: Sequence[str]
) -> tuple[str, ...]:
    res: list[str] = list(labels)
    t_label = tool.issue_label()
    if t_label not in res:
        res.append(t_label)
    if "type:report" not in res:
        res.append("type:report")
    return tuple(res)


def comment_payload(body: str) -> dict[str, object]:
    return {"body": body}


def followup_comment_body(
    tool: ToolInfo, message: str | None, os_name: str, arch: str
) -> str:
    chosen = (
        message
        if message is not None and message.strip() != ""
        else "User-side verification failed after closure; reopening for follow-up."
    )
    diag = render_diagnostics(tool, os_name, arch, None)
    return assemble_body(chosen, diag)


def prefilled_url(
    repo: str, title: str, body: str, labels: Sequence[str]
) -> str:
    url = f"https://github.com/{repo}/issues/new?title={percent_encode_query(title)}&body={percent_encode_query(body)}"
    if labels:
        lbl_str = ",".join(labels)
        url += f"&labels={percent_encode_query(lbl_str)}"
    return url
