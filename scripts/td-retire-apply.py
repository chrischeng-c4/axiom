#!/usr/bin/env python3
"""Apply tech-design retirement: delete tree files and referencing headers.

usage: td-retire-apply.py --prefix <tree-prefix> [--prefix <tree-prefix> ...]
       td-retire-apply.py <tree-prefix> [<tree-prefix> ...]
"""
from __future__ import annotations

import importlib.util
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)

_spec = importlib.util.spec_from_file_location(
    "td_retire_probe", os.path.join(HERE, "td-retire-probe.py")
)
probe = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(probe)


def under(path: str, prefixes: list[str]) -> bool:
    clean = [p.rstrip("/") for p in prefixes]
    return any(path == p or path.startswith(p + "/") for p in clean)


def delete_corpus_files(prefixes: list[str]) -> int:
    """Deletes all *.md and td.lock files under the given tree prefixes."""
    deleted_count = 0
    for prefix in prefixes:
        prefix_dir = os.path.join(ROOT, prefix.rstrip("/"))
        if not os.path.exists(prefix_dir):
            continue
        for dp, dns, fns in os.walk(prefix_dir, topdown=False):
            for fn in fns:
                if fn.endswith(".md") or fn == "td.lock":
                    fp = os.path.join(dp, fn)
                    os.remove(fp)
                    deleted_count += 1
            if not os.listdir(dp):
                try:
                    os.rmdir(dp)
                except OSError:
                    pass
    return deleted_count


def strip_headers(prefixes: list[str]) -> tuple[int, int]:
    """Removes rule-selected header lines pointing into the given prefixes."""
    want = lambda t: under(t, prefixes)
    touched_files = 0
    removed_lines_total = 0

    for dp, dns, fns in os.walk(ROOT):
        dns[:] = [d for d in dns if d not in probe.SKIP]
        rd = os.path.relpath(dp, ROOT)
        parts = rd.split("/")
        if "tech-design" in parts and under(probe._tree(parts), prefixes):
            continue

        for fn in fns:
            fp = os.path.join(dp, fn)
            text = probe.read_text(fp)
            if text is None or "tech-design" not in text:
                continue

            masked = probe.literal_lines(text) if fn.endswith(".rs") else set()
            lines = text.splitlines(keepends=True)
            new_lines = []
            modified = False

            for n, line in enumerate(lines, 1):
                line_stripped = line.rstrip("\r\n")
                m = probe.REF.search(line_stripped)
                if (
                    m
                    and probe._hits(m, rd, want)
                    and probe.HEADER.match(line_stripped.strip())
                    and n not in masked
                ):
                    modified = True
                    removed_lines_total += 1
                else:
                    new_lines.append(line)

            if modified:
                with open(fp, "w", encoding="utf-8") as f:
                    f.write("".join(new_lines))
                touched_files += 1

    return touched_files, removed_lines_total


def main(argv: list[str]) -> int:
    prefixes = []
    it = iter(argv)
    for arg in it:
        if arg == "--prefix":
            val = next(it, None)
            if val:
                prefixes.append(val)
        elif not arg.startswith("-"):
            prefixes.append(arg)
        else:
            sys.exit(f"error: unknown argument: {arg}")

    if not prefixes:
        sys.exit("error: at least one --prefix is required")

    deleted_files = delete_corpus_files(prefixes)
    touched_files, removed_lines = strip_headers(prefixes)

    print(
        f"applied: deleted_files={deleted_files} touched_files={touched_files} removed_lines={removed_lines}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
