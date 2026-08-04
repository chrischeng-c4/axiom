from __future__ import annotations

from openapi_codegen.domain.errors import OutputPathEscape


def check_output_path(rel_path: str) -> tuple[str, ...] | OutputPathEscape:
    if rel_path.startswith("/"):
        return OutputPathEscape(rel_path, "absolute")
    segments: list[str] = []
    for seg in rel_path.split("/"):
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
