#!/usr/bin/env python3
"""Apply the retirement: clear tree files and the headers that point at them.

usage: td-retire-apply.py [--whole-tree] --prefix <tree-prefix> [--prefix ...]
       td-retire-apply.py [--whole-tree] <tree-prefix> [<tree-prefix> ...]

Two modes, matching the two the gate judges.  Without `--whole-tree` this takes
the Markdown corpus and its `td.lock` and leaves everything else -- `.py`,
`pyproject.toml`, `uv.lock` -- where the Markdown children of #3694 need it.
With `--whole-tree` it takes every file under the prefix and the directories
themselves, which is the decision for the fifteen lumen-scope projects.

The header rule is `probe.selects` either way, so it covers `external-contracts`
targets as well as `tech-design` ones and matches, line for line, what
`td-retire-gate.py`'s `deletions` row will accept.  Nothing here decides what a
header is; a second copy of that rule is the copy that drifts.
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


def delete_corpus_files(prefixes: list[str], whole_tree: bool = False) -> int:
    """Clears the retired files under the given tree prefixes.

    In whole-tree mode that is every file and then every directory, the prefix
    root included: a tree that survives as one `.gitkeep` still resolves as a
    prefix and still appears in the census, which is #3737.
    """
    deleted_count = 0
    for prefix in prefixes:
        prefix_dir = os.path.join(ROOT, prefix.rstrip("/"))
        if not os.path.exists(prefix_dir):
            continue
        for dp, dns, fns in os.walk(prefix_dir, topdown=False):
            for fn in fns:
                if whole_tree or fn.endswith(".md") or fn == "td.lock":
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
        in_tree = "tech-design" in parts or "external-contracts" in parts
        if in_tree and under(probe._tree(parts), prefixes):
            continue

        for fn in fns:
            fp = os.path.join(dp, fn)
            text = probe.read_text(fp)
            if text is None or not (
                "tech-design" in text or "external-contracts" in text
            ):
                continue

            masked = probe.literal_lines(text) if fn.endswith(".rs") else set()
            lines = text.splitlines(keepends=True)
            new_lines = []
            modified = False

            for n, line in enumerate(lines, 1):
                line_stripped = line.rstrip("\r\n")
                if (
                    n not in masked
                    and probe.selects(line_stripped.strip(), rd, want)
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
    whole_tree = False
    it = iter(argv)
    for arg in it:
        if arg == "--whole-tree":
            whole_tree = True
        elif arg == "--prefix":
            val = next(it, None)
            if val:
                prefixes.append(val)
        elif not arg.startswith("-"):
            prefixes.append(arg)
        else:
            sys.exit(f"error: unknown argument: {arg}")

    if not prefixes:
        sys.exit("error: at least one --prefix is required")

    deleted_files = delete_corpus_files(prefixes, whole_tree)
    touched_files, removed_lines = strip_headers(prefixes)

    print(
        f"applied: whole_tree={whole_tree} deleted_files={deleted_files} "
        f"touched_files={touched_files} removed_lines={removed_lines}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
