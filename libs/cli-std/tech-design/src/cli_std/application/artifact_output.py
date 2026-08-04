from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class StdoutOutput:
    body: str


@dataclass(frozen=True)
class FileOutput:
    path: str
    body: str


OutputPlan = StdoutOutput | FileOutput


def has_extension(path: str) -> bool:
    segment = path.rsplit("/", 1)[-1]
    if segment in ("", ".", ".."):
        return False
    if segment.startswith("."):
        segment = segment[1:]
    return "." in segment


def join_path(directory: str, name: str) -> str:
    if not directory:
        return name
    if directory.endswith("/"):
        return f"{directory}{name}"
    return f"{directory}/{name}"


def plan_output(
    out: str | None, default_file: str, body: str
) -> OutputPlan:
    if out is None:
        return StdoutOutput(body)
    if has_extension(out):
        return FileOutput(out, body)
    return FileOutput(join_path(out, default_file), body)
