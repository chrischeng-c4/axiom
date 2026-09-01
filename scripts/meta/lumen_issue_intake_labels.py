#!/usr/bin/env python3
"""Classify canonical Lumen CLI issue reports from an issue body."""

from __future__ import annotations

import json
import sys


CANONICAL_LABELS = ["app:lumen", "type:report"]
_DIAGNOSTIC_PREFIXES = (
    "- lumen version: ",
    "- target: ",
    "- git sha: ",
    "- built at: ",
    "- os/arch: ",
)
_OPTIONAL_NODE_PREFIX = "- node: "


def _outside_fences(lines: list[str]) -> list[bool]:
    outside: list[bool] = []
    fence: str | None = None
    for line in lines:
        stripped = line.lstrip()
        marker = stripped[:3]
        if fence is None and marker in {"```", "~~~"}:
            outside.append(False)
            fence = marker
            continue
        if fence is not None:
            outside.append(False)
            if stripped.startswith(fence):
                fence = None
            continue
        outside.append(True)
    return outside


def labels_for_body(body: str) -> list[str]:
    """Return canonical labels only for a complete, real diagnostics section."""

    lines = body.splitlines()
    outside = _outside_fences(lines)
    for heading_index, line in enumerate(lines):
        if not outside[heading_index] or line != "## Diagnostics":
            continue

        values: list[str] = []
        contains_fence = False
        for index in range(heading_index + 1, len(lines)):
            if outside[index] and lines[index].startswith("#"):
                break
            if not outside[index]:
                contains_fence = True
            elif lines[index].strip():
                values.append(lines[index])

        if contains_fence or len(values) not in {
            len(_DIAGNOSTIC_PREFIXES),
            len(_DIAGNOSTIC_PREFIXES) + 1,
        }:
            continue
        required_fields_match = all(
            value.startswith(prefix) and bool(value.removeprefix(prefix).strip())
            for value, prefix in zip(
                values[: len(_DIAGNOSTIC_PREFIXES)],
                _DIAGNOSTIC_PREFIXES,
                strict=True,
            )
        )
        optional_node_matches = len(values) == len(_DIAGNOSTIC_PREFIXES) or (
            values[-1].startswith(_OPTIONAL_NODE_PREFIX)
            and bool(values[-1].removeprefix(_OPTIONAL_NODE_PREFIX).strip())
        )
        if required_fields_match and optional_node_matches:
            return CANONICAL_LABELS.copy()
    return []


def main() -> int:
    labels = labels_for_body(sys.stdin.read())
    print(json.dumps(labels, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
