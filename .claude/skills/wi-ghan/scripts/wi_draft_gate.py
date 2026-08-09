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

# A rewrite round is handed exactly one runnable command -- this script -- so
# it is the only command whose baseline the round can observe, and three rounds
# out of three (#3387, #3392, #3460) answered `## Acceptance` with it. The
# resulting table is circular: it asserts that the body validates, which is what
# the gate running it already established, and says nothing about the change the
# work item asks for. The injection now forbids it in prose; this is the
# consumer that refuses it, because prose alone is what already failed.
SELF_REFERENCE = "wi_draft_gate.py"


def change_points(body: str) -> str:
    """The `### Change points` block, as text, for a containment test.

    A rewrite round may write only `.aw-wi/<n>.md`, so it can never list this
    script as something it changes. A work item *about* this script can and
    does -- #3506 is one -- and its acceptance rows name it for the honest
    reason. That difference is the whole test.
    """
    out: list[str] = []
    inside = False
    for line in body.splitlines():
        if line.startswith("#"):
            inside = line.strip() == "### Change points"
            continue
        if inside:
            out.append(line)
    return "\n".join(out)


def acceptance_rows(body: str) -> list[str]:
    """The `## Acceptance` table's data rows, header and separator dropped."""
    rows: list[str] = []
    inside = False
    for line in body.splitlines():
        if line.startswith("## "):
            inside = line.strip() == "## Acceptance"
            continue
        if not inside:
            continue
        if line.startswith("### "):
            break
        stripped = line.strip()
        if not stripped.startswith("|"):
            continue
        cells = [cell.strip() for cell in stripped.strip("|").split("|")]
        if not cells or cells[0].lower() in ("#", "") or set(cells[0]) <= set("-: "):
            continue
        rows.append(stripped)
    return rows


def extract_command(row: str) -> str:
    """Extract raw command string from an acceptance table row."""
    cells = [cell.strip() for cell in row.strip().strip("|").split("|")]
    if len(cells) < 2:
        return ""
    return re.sub(r"^`+|`+$", "", cells[1]).strip()



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


def predates_ghan_flip(aw: str, project: str) -> bool:
    """Ask the binary what it scaffolds for a change with no body.

    A binary from before the GHAN flip merges any body into the legacy
    six-section template, and the only symptom the caller sees is that the
    stored body is not the one it handed in -- which reads as the normalizer
    editing the round's work rather than as the wrong binary being on `PATH`.
    The default body is the same decision made in the open: post-flip it is a
    blank GHAN form, pre-flip it opens with `## Problem`.
    """
    code, out, _ = run(
        [aw, "wi", "draft", "init",
         "--title", "ghan flip probe",
         "--type", "change",
         "--project", project,
         "--json"]
    )
    if code != 0:
        return False
    try:
        draft = Path(json.loads(out)["path"])
    except (json.JSONDecodeError, KeyError, OSError):
        return False
    try:
        body = FRONTMATTER.sub("", draft.read_text())
    except OSError:
        return False
    headings = [line for line in body.splitlines() if line.startswith("## ")]
    return "## Goal" not in headings


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
    ap.add_argument(
        "--structure-only",
        action="store_true",
        help="judge only the document structure and GHAN rules; do not execute "
        "or measure the acceptance table's commands against the checkout",
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

    rows = acceptance_rows(body)
    owns_gate = SELF_REFERENCE in change_points(body)
    circular = [] if owns_gate else [row for row in rows if SELF_REFERENCE in row]
    if circular:
        print(
            f"FAIL  {path}: {len(circular)} acceptance row(s) name this gate "
            "itself, so the table asserts that the body validates rather than "
            "that the change happened"
        )
        for row in circular:
            print(f"  - {row}")
        print(
            "       The row's command belongs to the work item's implementer "
            "and has to exercise\n"
            "       the product -- a test, a CLI invocation, an EC case. This "
            "script is the gate on\n"
            "       the round that writes the body, and a round cannot be its "
            "own acceptance."
        )
        return 1
    if not rows:
        print(f"FAIL  {path}: `## Acceptance` carries no table rows to judge")
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
        if predates_ghan_flip(args.aw, args.project):
            print(
                f"FAIL  `{args.aw}` predates the GHAN flip: it still answers a "
                "change with the legacy six-section template, so it merged this "
                "body into that scaffold. Rebuild and reinstall it, or pass "
                "`--aw <path>`; the round was not judged."
            )
            return 1
        print(
            "FAIL  `draft init` did not store the body verbatim, so what was "
            "judged is not what would be published"
        )
        print(f"       stored {len(stored.splitlines())} line(s) for a body of "
              f"{len(body.strip().splitlines())}; draft at {draft}")
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

    if not args.structure_only:
        repo_root = Path(__file__).resolve().parents[4]
        measured_rows: list[tuple[str, int, str]] = []
        for row in rows:
            cmd = extract_command(row)
            if not cmd:
                continue
            done = subprocess.run(
                cmd, shell=True, cwd=repo_root, capture_output=True, text=True
            )
            out_err = (done.stdout + done.stderr).strip()
            measured_rows.append((cmd, done.returncode, out_err))

        if measured_rows and all(rc == 0 for _, rc, _ in measured_rows):
            print(
                f"FAIL  {path}: all {len(measured_rows)} acceptance row(s) already succeed against the checkout; "
                "a work item's gate baseline must be measured red before the change"
            )
            for cmd, rc, out_err in measured_rows:
                print(f"  - command: {cmd}")
                print(f"    exit: {rc}")
                if out_err:
                    for line in out_err.splitlines():
                        print(f"    {line}")
            return 1

    print(f"PASS  {path}")
    print(f"title : {title}")
    print(f"draft : {draft}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
