#!/usr/bin/env python3
"""Turn one legacy work item into the round that rewrites it into GHAN.

`type=change` work items are now validated as Goal / How / Acceptance / Never,
and several hundred open ones still carry the six-section body that shape
replaced. Rewriting them by hand is the same paragraph re-derived a few hundred
times, and the thing that makes it expensive is not the prose -- it is that
every rewrite has to re-read the checkout to turn "the CLI is wrong" into a
premise carrying a `file:line`.

That is a worker's job, not a controller's. So this projects the work item into
a bounded round: one write target, one gate, one oracle, one injection. The
worker reads the legacy body out of the injection -- it cannot reach the
tracker, and it should not be able to -- and writes the rewritten body into
`.aw-wi/<issue>.md`.

The gate is `wi_draft_gate.py`, which runs the product's own validator against
the proposed body with no tracker side effect. It is not a proxy for the rule:
it *is* the rule, the same code `aw wi validate` runs. What it cannot check is
whether the rewrite still asks for what the work item asked for, which is why
the oracle carries that as a row the controller reads at `review` and why the
fabrication tells name the shapes a body can take while passing.

Nothing here decides *what* a work item should say. A round this produces is
refused, revised, or accepted on the same evidence as any other.
"""
from __future__ import annotations

import argparse
import json
import re
import shlex
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "codex-dispatch" / "scripts"))
import codex_dispatch  # noqa: E402

SKILL_SCRIPTS = ".claude/skills/wi-ghan/scripts"
GATE_SCRIPT = f"{SKILL_SCRIPTS}/wi_draft_gate.py"
DRAFT_DIR = ".aw-wi"
GHAN_RULE = ".agents/rules/authoring/agent-instruction-ghan.md"
GHAN_VALIDATOR = "apps/agentic-workflow/src/issues/ghan.rs"

GHAN_HEADINGS = ("## Goal", "## How", "## Acceptance", "## Never")
# The tracker body's own path-looking tokens. Used only to warn: a legacy body
# citing a path that has since moved is a fact about the work item the worker
# needs to know, not a reason to refuse the round.
PATH_TOKEN = re.compile(r"`([A-Za-z0-9_./-]+/[A-Za-z0-9_./-]+)`")


def gh(argv: list[str], repo: str | None) -> str:
    full = ["gh", *argv]
    if repo:
        full += ["--repo", repo]
    done = subprocess.run(full, capture_output=True, text=True)
    if done.returncode != 0:
        raise SystemExit(f"`{' '.join(full)}` failed: {done.stderr.strip()}")
    return done.stdout


def read_work_item(issue: str, repo: str | None) -> dict:
    raw = gh(["issue", "view", issue, "--json", "title,body,labels,state"], repo)
    item = json.loads(raw)
    item["labels"] = [label["name"] for label in item.get("labels", [])]
    return item


def project_of(labels: list[str], override: str | None) -> str:
    if override:
        return override
    for label in labels:
        if label.startswith("app:"):
            return label.split(":", 1)[1]
    raise SystemExit(
        "no `app:<project>` label on this work item, so the draft has no "
        "project to validate under; pass --project"
    )


def refuse_unless_rewritable(item: dict, issue: str) -> None:
    if item.get("state") != "OPEN":
        raise SystemExit(f"work item {issue} is {item.get('state')}, not open")
    if "type:change" not in item["labels"]:
        raise SystemExit(
            f"work item {issue} is not `type:change`; Goal / How / Acceptance / "
            "Never is the change shape and the other types keep the six-section "
            "body, so there is nothing here to rewrite"
        )
    body = item["body"] or ""
    present = [head for head in GHAN_HEADINGS if head in body]
    if len(present) == len(GHAN_HEADINGS):
        raise SystemExit(
            f"work item {issue} already carries all four GHAN sections; run the "
            "gate against it rather than dispatching a rewrite"
        )


def fence(text: str) -> str:
    """Quote a body that may itself contain fences.

    A legacy work item routinely holds a ``` block, so the quote around it has
    to be longer than anything inside it or the injection's own section ends
    early and lint reads the remainder as prose.
    """
    longest = max((len(run) for run in re.findall(r"`+", text)), default=0)
    bar = "`" * max(4, longest + 1)
    return f"{bar}\n{text.rstrip()}\n{bar}"


def render_oracle(issue: str, title: str, gate: str, target: str) -> str:
    return f"""\
## Claim

Work item #{issue} is restated as Goal / How / Acceptance / Never that the \
product's own validator accepts, asking for the same change the six-section \
body asked for and stating no premise, coordinate, or digest that was not read \
in this checkout.

## Measurements

| # | input | expected observation | why it cannot hold by accident |
|---|---|---|---|
| 1 | `{gate}` | exits 0 and reports `PASS` | it runs `aw wi draft init` and `aw wi draft validate`, the same GHAN validator `aw wi validate` applies to a published body; a body that merely looks like the shape fails on a per-section rule -- a premise without a `file:line`, an acceptance table without its five columns, a negative control without a sha256 |
| 2 | every `file:line` under `### Verified premises`, read at that line in this checkout | the line says what the premise says it says | the validator checks that a coordinate is *shaped* like one, never that it resolves; a premise pointing at a line that does not exist, or at one saying something else, passes row 1 and fails here |
| 3 | the requirements the tracker body states, compared against the rewritten one | each survives, and the rewrite adds none the tracker body does not ask for | row 1 is indifferent to content: a well-formed body about a different change passes it |
| 4 | remove the `### Negative control` sub-section from `{target}` and rerun row 1 (negative control) | must FAIL, naming the missing sub-section | a gate that stayed green here would be reporting on the file's existence rather than on its contents |

## Gate

```
{gate}
```

## Fabrication tells

- A `## Goal` that restates the title. The title says what the work is called; the Goal has to name a trigger, an observation point, a current value and a target value, and a Goal that could be written without opening the checkout has none of them.
- A premise citing `:1`, or a line range. `file_line_ref` refuses a range and accepts `:1`, so `:1` is where an unmeasured coordinate lands.
- A negative control naming an all-zero sha256, or any digest that is not the current digest of the file it says it restores. The validator accepts any 64-character hex.
- An acceptance row whose "why it cannot hold by accident" cell restates the target cell. That column exists to say what *other* change would also make the row green.
- A gate command in `## Acceptance` that no one ran. The row's "current" column is an observation, so a row whose current column describes the target rather than today's output was never measured.
- Requirements dropped on the way across, especially ones the six-section body kept in `## Scope` or `## Reference Context` rather than in `## Requirements`.
- A body that fills `### Frozen decisions` with a restatement of the change instead of a decision or an exclusion, or with `none` where the tracker body plainly froze something.
"""


def render_injection(
    issue: str, title: str, body: str, gate: str, target: str, project: str
) -> str:
    return f"""\
## Task

Rewrite work item #{issue} as Goal / How / Acceptance / Never without changing \
what it asks for

## Current behavior

`{target}` does not exist, and the tracker body for #{issue} is the six-section \
shape the validator now refuses for a `type=change` work item:

{fence(body)}

## Required change

- `{target}` holds the work-item title on its first line as `# {title}`, then the rewritten body.
- The body's H2 sections are `## Goal`, `## How`, `## Acceptance` and `## Never`, in that order and with no other H2.
- Every item under `### Verified premises` carries a `file:line` coordinate read at that line in this checkout, and states what is observable there rather than what it implies.
- `### Change points` names the paths this work item would write, and `### Must not touch` names none of them.
- `## Acceptance` carries a table with the columns `#`, command, current, target, and why it cannot hold by accident, and a `### Negative control` naming a mutation, asserting failure, and naming the sha256 the mutated file restores to.
- Every requirement the tracker body states is present in the rewritten body, and the rewritten body asks for nothing the tracker body does not.

## Shape to follow

Follow `{GHAN_RULE}`, which states what each section refuses, and read the \
predicates in `{GHAN_VALIDATOR}` rather than inferring them from the rule's \
prose; do not invent a second vocabulary for a section that already has one.

## Reference

| path | why the worker must read it |
|---|---|
| `{GHAN_RULE}` | states, per section, the refusal condition the section exists to carry -- the reason a section that no consumer refuses degenerates into a title echo |
| `{GHAN_VALIDATOR}` | the exact predicates the gate applies: what counts as a `file:line`, which hedge words a premise may not contain, why a path listed as both a change point and must-not-touch is refused, and what shape a negative control has to have |
| `{GATE_SCRIPT}` | the gate itself, including the unfilled-slot spellings it refuses and the title line it expects on line one |

## Out of scope

- Do not implement the work item. This round produces a work-item body; the change it describes is a later round.
- Do not decide the work item is wrong and rewrite what it asks for. A requirement you believe is mistaken stays, stated as the tracker states it.
- Do not write outside `{target}`.
- Do not state a coordinate, a digest, a count, or a command output you did not read in this checkout. An unmeasured value is the one thing this round cannot recover from, because the gate accepts it.

## Definition of done

`{target}` exists and

```
{gate}
```
"""


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Project a legacy work item into the round that rewrites it into GHAN."
    )
    ap.add_argument("issue", help="work-item number; also the round's task key")
    ap.add_argument("--repo", help="owner/name; defaults to gh's own default")
    ap.add_argument(
        "--root",
        default=str(Path.cwd()),
        help="controller repository root; the round's worktree is derived from it",
    )
    ap.add_argument("--project", help="overrides the `app:<project>` label")
    ap.add_argument("--out", help="profile path; defaults under /tmp/codex-dispatch")
    ap.add_argument(
        "--print-only",
        action="store_true",
        help="show the profile invocation and both documents without writing anything",
    )
    args = ap.parse_args()

    root = Path(args.root).resolve()
    found = subprocess.run(
        ["git", "-c", "core.fsmonitor=false", "-C", str(root), "rev-parse", "--show-toplevel"],
        capture_output=True,
        text=True,
    )
    if found.returncode == 0:
        root = Path(found.stdout.strip())

    repo = args.repo
    if not repo:
        repo = json.loads(
            gh(["repo", "view", "--json", "nameWithOwner"], None)
        )["nameWithOwner"]

    item = read_work_item(args.issue, repo)
    refuse_unless_rewritable(item, args.issue)
    project = project_of(item["labels"], args.project)
    body = (item["body"] or "").replace("\r\n", "\n").rstrip()
    title = item["title"].strip()

    target = f"{DRAFT_DIR}/{args.issue}.md"
    gate = (
        f"python3 {GATE_SCRIPT} {target} --project {project}"
    )

    for required in (GHAN_RULE, GHAN_VALIDATOR, GATE_SCRIPT, f"{DRAFT_DIR}/.gitkeep"):
        if not (root / required).exists():
            raise SystemExit(
                f"{required} is missing from {root}; the round cites it, and a "
                "worktree cut from HEAD will not have it until it is committed"
            )

    missing = sorted(
        {
            token
            for token in PATH_TOKEN.findall(body)
            if not (root / token.split(":")[0]).exists()
        }
    )

    out = args.out or f"/tmp/codex-dispatch/{args.issue}-ghan.json"
    argv = [
        sys.executable,
        str(Path(__file__).resolve().parents[2] / "codex-dispatch" / "scripts" / "make_profile.py"),
        "--root", str(root),
        "--repo", repo,
        "--scope", DRAFT_DIR,
        "--issue", args.issue,
        "--design-input", GHAN_RULE,
        "--design-input", GHAN_VALIDATOR,
        "--design-input", GATE_SCRIPT,
        "--write", target,
        "--gate", gate,
        "--read-commands",
        "--out", out,
    ]

    print(f"work item : #{args.issue} {title}")
    print(f"project   : {project}")
    print(f"write     : {target}")
    print(f"gate      : {gate}")
    print(f"body      : {len(body.splitlines())} lines from the tracker")
    if missing:
        print(
            "note      : the tracker body cites "
            f"{len(missing)} path(s) absent from this checkout, which the "
            "rewrite must not carry forward as premises:"
        )
        for path in missing:
            print(f"            {path}")
    print()
    print("profile:")
    print("  " + " ".join(shlex.quote(part) for part in argv))

    oracle_text = render_oracle(args.issue, title, gate, target)
    injection_text = render_injection(args.issue, title, body, gate, target, project)

    if args.print_only:
        print()
        print(oracle_text)
        print(injection_text)
        return 0

    made = subprocess.run(argv, capture_output=True, text=True)
    sys.stdout.write(made.stdout)
    if made.returncode != 0:
        sys.stderr.write(made.stderr)
        raise SystemExit(f"make_profile.py exited {made.returncode}")

    profile = json.loads(Path(out).read_text())
    for path, text in (
        (codex_dispatch.oracle_path(profile, args.issue), oracle_text),
        (codex_dispatch.injection_path(profile, args.issue), injection_text),
    ):
        if path.exists():
            print(f"kept   {path} (already authored; projection never overwrites)")
            continue
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
        print(f"wrote  {path}")

    scripts = ".claude/skills/codex-dispatch/scripts"
    print(
        f"""
next, in order:

  P={out}
  python3 {scripts}/codex_dispatch.py worktree $P {args.issue}
  python3 {scripts}/codex_dispatch.py rules    $P
  python3 {scripts}/codex_dispatch.py doctor   $P
  python3 {scripts}/codex_dispatch.py capture  $P {args.issue} \\
      'gh issue view {args.issue} --repo {repo} --json body --jq .body'
  python3 {scripts}/codex_dispatch.py lint     $P {args.issue}
  python3 {scripts}/codex_dispatch.py snapshot $P {args.issue}
  python3 {scripts}/codex_dispatch.py dispatch $P {args.issue}

the `capture` step is not optional: lint accepts a quoted line only when it is
in a cited file or in a recorded capture, and the tracker body is in neither
until that command runs."""
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
