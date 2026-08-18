from __future__ import annotations

import re

from openapi_codegen.domain.errors import OutputPathEscape


def check_output_path(rel_path: str) -> tuple[str, ...] | OutputPathEscape:
    if rel_path.startswith(("/", "\\")) or re.match(r"^[A-Za-z]:", rel_path):
        return OutputPathEscape(rel_path, "absolute")
    segments: list[str] = []
    for seg in re.split(r"[\\/]+", rel_path):
        if seg == "" or seg == ".":
            continue
        if seg == "..":
            return OutputPathEscape(rel_path, "parent-component")
        segments.append(seg)
    return tuple(segments)


def joined_output_path(out_dir: str, rel_path: str) -> str | OutputPathEscape:
    res = check_output_path(rel_path)
    if isinstance(res, OutputPathEscape):
        return res
    base = out_dir.rstrip("/")
    if not res:
        return base
    return base + "/" + "/".join(res)
