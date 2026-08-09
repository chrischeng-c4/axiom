#!/usr/bin/env python3
"""Judge one rewritten work-item body without touching the tracker.

`aw wi validate` needs an issue that already exists, so validating a proposed
body used to mean opening one -- which is how a rewrite round would leave a
trail of garbage tickets behind every attempt. `aw wi draft` validates the same
body against the same rules with no tracker side effect, and this is the single
command a round can be judged by: `draft init` writes the body under a local
workspace, `draft validate` reports `passed` and the section errors, and this
exits non-zero unless `passed` is true.

The body file carries its own title on the first line as `# <title>`, because a
title with an apostrophe inside a gate command string is the kind of quoting
accident that reads afterwards as the round failing. The rest of the file is the
work-item body verbatim, and this refuses if `draft init` did not store it
verbatim: a normalization that silently edits the body would mean the thing
judged is not the thing the controller is about to publish.
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

# The unfilled-slot spellings the product's own generated form uses, plus the
# two the GHAN validator refuses by name. A body still holding one is a form,
# not an answer, and the round is not done.
UNFILLED = ("<!-- fill", "(fill)", "(replace-this)", "TODO", "TBD")
FRONTMATTER = re.compile(r"\A---\n.*?\n---\n", re.DOTALL)


def split_title(text: str, path: Path) -> tuple[str, str]:
    lines = text.splitlines()
    for index, line in enumerate(lines):
        if not line.strip():
            continue
        if not line.startswith("# ") or line.startswith("## "):
            raise SystemExit(
                f"{path}: first non-empty line must be the work-item title as "
                f"`# <title>`, not {line!r}. The title is part of the artifact "
                "because the boundedness gate reads it together with `## Goal`"
            )
        title = line[2:].strip()
        if not title:
            raise SystemExit(f"{path}: the title line is empty")
        return title, "\n".join(lines[index + 1 :]).lstrip("\n")
    raise SystemExit(f"{path}: file is empty")


def run(argv: list[str]) -> tuple[int, str, str]:
    done = subprocess.run(argv, capture_output=True, text=True)
    return done.returncode, done.stdout, done.stderr


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Validate a proposed work-item body as a local draft."
    )
    ap.add_argument("body_file", help="`# <title>` on line one, then the body")
    ap.add_argument("--project", required=True)
    ap.add_argument("--type", default="change", dest="issue_type")
    ap.add_argument(
        "--aw",
        default="aw",
        help="the binary to judge with; the default is whatever is installed, "
        "which is the one `aw wi create` will apply to this body later",
    )
    args = ap.parse_args()

    path = Path(args.body_file)
    if not path.is_file():
        raise SystemExit(f"no body to judge: {path} does not exist")
    text = path.read_text()
    title, body = split_title(text, path)

    held = [marker for marker in UNFILLED if marker in text]
    if held:
        print(f"FAIL  {path} still holds an unfilled slot: {', '.join(held)}")
        return 1

    with tempfile.TemporaryDirectory() as tmp:
        staged = Path(tmp) / "body.md"
        staged.write_text(body)
        code, out, err = run(
            [
                args.aw, "wi", "draft", "init",
                "--title", title,
                "--type", args.issue_type,
                "--project", args.project,
                "--body-file", str(staged),
                "--json",
            ]
        )
        if code != 0:
            print(f"FAIL  `aw wi draft init` exited {code}")
            print((out + err).strip())
            return 1
        try:
            draft = json.loads(out)["path"]
        except (json.JSONDecodeError, KeyError) as exc:
            print(f"FAIL  could not read the draft path from `draft init`: {exc}")
            print(out.strip())
            return 1

    stored = FRONTMATTER.sub("", Path(draft).read_text()).strip()
    if stored != body.strip():
        print(
            "FAIL  `draft init` did not store the body verbatim, so what was "
            "judged is not what would be published"
        )
        return 1

    code, out, err = run([args.aw, "wi", "draft", "validate", draft, "--json"])
    try:
        verdict = json.loads(out)
    except json.JSONDecodeError:
        print(f"FAIL  `aw wi draft validate` printed no JSON (exit {code})")
        print((out + err).strip())
        return 1

    errors = verdict.get("errors") or []
    if not verdict.get("passed"):
        print(f"FAIL  {path}: {len(errors)} error(s)")
        for message in errors:
            print(f"  - {message}")
        return 1
    if errors:
        print(f"FAIL  {path}: passed=true with {len(errors)} error(s) reported")
        for message in errors:
            print(f"  - {message}")
        return 1

    print(f"PASS  {path}")
    print(f"title : {title}")
    print(f"draft : {draft}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
