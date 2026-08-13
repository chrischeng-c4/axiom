#!/usr/bin/env python3
"""External-contract (EC) case surface, prototyped in Python over `uv` and `git`.

This is the surface the four `/aw:wi-ec-*` skills drive. It exists so the EC
leg of the change lifecycle can be proven -- red-first, reviewed, committed --
before any of it is spent on a Rust verb axis. Like `epic.py` beside it, it
deliberately does not call `aw`: the cases are run by `uv` and the commit is
made by `git`, and both are reachable directly.

The one thing this script exists to prevent is a green that means nothing. Every
question it answers is mechanical and closed; every question it cannot answer
mechanically it routes to a reviewer and refuses to guess at. It never decides
whether a contract is *meaningful* -- see `review-prompt` and `verdict` for
where that decision comes from and what binds it.

What "this change" means
------------------------
A work item's EC change is **the diff against HEAD**, and that definition holds
only because `start` refuses to open the leg while the working tree is dirty.
From a clean start, whatever `git status` reports is what this leg wrote: the
population is derived from git rather than remembered in a side table, a list
of case ids, or a constant inside each case. A side table can point at a case
that was deleted and nothing would notice; `git status` cannot.

The same clean start is what lets `C0` refuse a `src/**` or `tech-design/**`
path. `CLAUDE.md` fixes the order -- external-contracts, then tech-design, then
src -- and until now that order was prose in a skill body, which is a request
rather than a gate.

Interpreter
-----------
This script reads TOML, so it needs 3.11+. The skills invoke it as

    uv run --python 3.13 --no-project "${CLAUDE_PLUGIN_ROOT}/scripts/ec.py" ...

because a bare `python3` is 3.9 on at least one developer machine, and the
failure mode there is a `ModuleNotFoundError` traceback rather than a sentence.

Verbs
-----
Work-item scoped -- the four legs, each re-running every earlier one's checks:

  start <iid>          demand a clean tree, print the work item to author against
  verify <iid>         the closed mechanical list over this change; no writes
  review-prompt <iid>  emit the customised prompt the reviewer is fed
  verdict <iid>        bind a reviewer transcript to the exact bytes it reviewed
  commit <iid>         re-run everything, require a live verdict, commit the diff

Case scoped -- primitives for looking at one case while authoring it:

  red --case      run one case and classify *how* it fails
  check --case    the same mechanical list, narrowed to a single case
"""
from __future__ import annotations

import argparse
import ast
import hashlib
import importlib.util
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

if sys.version_info < (3, 11):
    sys.stderr.write(
        f"ec.py needs Python 3.11+ for tomllib; this is {sys.version.split()[0]}.\n"
        "Invoke it as: uv run --python 3.13 --no-project <path>/ec.py ...\n"
    )
    raise SystemExit(2)

import tomllib

DEFAULT_PROJECT = "agentic-workflow"

# `git` in this checkout is run with fsmonitor disabled: a stalled fsmonitor
# daemon blocks every command that reads the index, indefinitely and silently.
GIT = ("git", "-c", "core.fsmonitor=false")

# The reviewer's output contract. `ec.py verdict` parses these out of the raw
# transcript rather than trusting whoever ran the reviewer to report them.
VERDICT_LINE = re.compile(r"^VERDICT:\s*(accepted|rejected)\s*$", re.M)
FINDING_LINE = re.compile(r"^FINDING:\s*(\S.*)$", re.M)

# Case-source shapes that satisfy a red-first gate without observing anything.
# These are the near misses for the "fails where it declared it would" check:
# each one produces a real AssertionError carrying the declared message.
LITERAL_NODES = (ast.Constant,)


# --------------------------------------------------------------------------
# locating things
# --------------------------------------------------------------------------
def repo_root(start: Path | None = None) -> Path:
    """The outermost directory carrying an `aw.toml`.

    Outermost, not nearest: `apps/<project>/aw.toml` exists too, and stopping
    at the first one found would silently scope every path to one project.
    """
    here = (start or Path.cwd()).resolve()
    found: Path | None = None
    for candidate in [here, *here.parents]:
        if (candidate / "aw.toml").is_file():
            found = candidate
    if found is None:
        raise SystemExit(
            f"not inside a checkout: no aw.toml above {here}\n"
            "Run this from inside the repository you mean to write against."
        )
    return found


def ec_root(repo: Path, project: str) -> Path:
    root = repo / "apps" / project / "external-contracts"
    if not (root / "pyproject.toml").is_file():
        raise SystemExit(f"no EC inventory at {root / 'pyproject.toml'}")
    return root


def inventory(ec: Path) -> dict[str, dict[str, Any]]:
    data = tomllib.loads((ec / "pyproject.toml").read_text(encoding="utf-8"))
    cases = data.get("tool", {}).get("aw", {}).get("python-ec", {}).get("cases", [])
    return {c["id"]: c for c in cases}


def review_route(repo: Path, project: str, stage: str = "ec") -> str:
    """Who produces the semantic verdict, per `apps/<project>/aw.toml`.

    The value names a *skill*, not a mode: `skill:codex-review`. Every value is
    a skill so that "does this route resolve to something real?" is one total
    check with no special cases.
    """
    path = repo / "apps" / project / "aw.toml"
    if not path.is_file():
        return ""
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    return str(data.get("review", {}).get(stage, ""))


def case_path(ec: Path, case_id: str) -> Path:
    return ec / "src" / "cases" / f"{case_id}.py"


def case_constants(path: Path) -> dict[str, Any]:
    """Module-level literal assignments, read without importing the module.

    Importing would run the case. Reading is the point: these constants are the
    case's own declaration of what it is, and the declaration has to be
    readable before the case is trusted to run.
    """
    out: dict[str, Any] = {}
    for node in ast.parse(path.read_text(encoding="utf-8")).body:
        if not isinstance(node, ast.Assign):
            continue
        for target in node.targets:
            if isinstance(target, ast.Name):
                try:
                    out[target.id] = ast.literal_eval(node.value)
                except (ValueError, TypeError, SyntaxError):
                    pass
    return out


def wi_body_path(repo: Path, iid: int) -> Path:
    """Where `change.py fetch <iid>` stages the work item's body."""
    return repo / ".aw" / "workitems" / "changes" / f"{iid}.md"


def change_module() -> Any:
    """`change.py` beside this file, imported rather than shelled out to.

    Shelling out would mean reading a human-facing report to find out whether
    the body is admissible. Importing returns the same list of errors the
    `validate` verb prints, before it has been turned into prose.
    """
    path = Path(__file__).resolve().parent / "change.py"
    spec = importlib.util.spec_from_file_location("changemod", path)
    module = importlib.util.module_from_spec(spec)
    sys.modules["changemod"] = module
    spec.loader.exec_module(module)
    return module


def dirty_set(repo: Path) -> list[str]:
    """Every path git reports as differing from HEAD, tracked or not.

    `-uall` rather than the default: an untracked *directory* is reported
    collapsed to its own name, and a collapsed directory cannot be digested,
    scoped, or handed to `git commit` as a pathspec with any precision. A new
    case arrives untracked, so that is the common shape, not the exotic one.
    """
    proc = subprocess.run(
        [*GIT, "status", "--porcelain", "-uall"], cwd=repo, capture_output=True, text=True
    )
    if proc.returncode != 0:
        raise SystemExit(proc.stderr.strip() or "git status failed")
    paths: list[str] = []
    for line in proc.stdout.splitlines():
        if not line.strip():
            continue
        entry = line[3:]
        # A rename reads `old -> new`; the new name is the one on disk and the
        # one a pathspec has to carry.
        if " -> " in entry:
            entry = entry.split(" -> ", 1)[1]
        paths.append(entry.strip().strip('"'))
    return sorted(paths)


def change_digest(repo: Path, iid: int, paths: list[str]) -> str:
    """One digest over the work item and every byte of the change.

    The body is in here on purpose. The question the reviewer answers is "does
    this change satisfy this work item", so an edit to *either* side can flip
    the answer -- and a verdict that survives an edit to the work item is an
    approval of a requirement nobody reviewed.
    """
    h = hashlib.sha256()
    h.update(wi_body_path(repo, iid).read_bytes())
    for rel in sorted(paths):
        h.update(b"\n--" + rel.encode() + b"--\n")
        target = repo / rel
        h.update(target.read_bytes() if target.is_file() else b"(absent)")
    return h.hexdigest()


def verdict_path(repo: Path, key: str) -> Path:
    return repo / ".aw" / "ec-review" / f"{key}.json"


# --------------------------------------------------------------------------
# running a case
# --------------------------------------------------------------------------
ANSI = re.compile(r"\x1b\[[0-9;]*m")
EXC_LINE = re.compile(r"^(?P<kind>[A-Za-z_][\w.]*(?:Error|Exception)):\s?(?P<msg>.*)", re.M)


def run_case(repo: Path, entry: dict[str, Any]) -> dict[str, Any]:
    proc = subprocess.run(
        entry["command"].split(), cwd=repo, capture_output=True, text=True, timeout=900
    )
    clean = ANSI.sub("", proc.stderr)
    matches = list(EXC_LINE.finditer(clean))
    last = matches[-1] if matches else None
    return {
        "exit": proc.returncode,
        "exception": last.group("kind") if last else "",
        "message": last.group("msg").strip() if last else "",
        "stderr": clean,
        "stdout": proc.stdout,
    }


# --------------------------------------------------------------------------
# the closed mechanical list
# --------------------------------------------------------------------------
class Check:
    def __init__(self) -> None:
        self.rows: list[tuple[str, str, str]] = []

    def add(self, status: str, name: str, detail: str = "") -> None:
        self.rows.append((status, name, detail))

    @property
    def failed(self) -> list[tuple[str, str, str]]:
        return [r for r in self.rows if r[0] == "FAIL"]

    @property
    def pending(self) -> list[tuple[str, str, str]]:
        return [r for r in self.rows if r[0] == "PENDING"]

    def report(self) -> None:
        for status, name, detail in self.rows:
            print(f"  {status:8s} {name}")
            if detail:
                for line in detail.splitlines():
                    print(f"           {line}")


def p1_work_item(chk: Check, repo: Path, iid: int) -> None:
    """The work item exists locally and is an admissible change body.

    `ec.py` never reads the tracker. The skill runs `change.py fetch <iid>`
    first, which overwrites the local copy unconditionally -- and that
    overwrite is what makes the file on disk the tracker's body rather than a
    draft an earlier session left behind.
    """
    path = wi_body_path(repo, iid)
    if not path.is_file():
        chk.add("FAIL", "P1 work item",
                f"no staged body at {path}\nrun: change.py fetch {iid}")
        return
    errors = [e for e in change_module().validate_body(path.read_text(encoding="utf-8"))
              if not e.startswith("note:")]
    if errors:
        chk.add("FAIL", "P1 work item",
                f"#{iid} is not a valid change body:\n" + "\n".join(errors[:6]))
        return
    chk.add("PASS", "P1 work item", f"#{iid} is a valid change body")


def p2_clean_tree(chk: Check, dirty: list[str]) -> None:
    """Nothing is modified yet, which is what makes the later diff readable.

    This is the only check that has to hold *before* any work. With a clean
    start, everything `git status` reports from here on was written by this
    leg -- so the EC change never has to be remembered, listed, or bound to
    its cases by hand. Every later check derives its population from it.
    """
    if dirty:
        chk.add("FAIL", "P2 clean tree",
                "the working tree already carries changes, so an EC change made "
                "now could not be told apart from them; commit or stash first:\n"
                + "\n".join(f"  {p}" for p in dirty[:20])
                + (f"\n  ... and {len(dirty) - 20} more" if len(dirty) > 20 else ""))
        return
    chk.add("PASS", "P2 clean tree", "nothing differs from HEAD")


def p3_leg_is_open(chk: Check, repo: Path, iid: int) -> None:
    """This work item's EC leg has not already landed.

    Without this, re-running the leg over a work item whose contract is already
    committed finds an empty diff and reports it as a change with no cases,
    which names the wrong defect.
    """
    proc = subprocess.run(
        [*GIT, "log", "--format=%h %s", "--extended-regexp", f"--grep=^Refs #{iid}$"],
        cwd=repo, capture_output=True, text=True,
    )
    landed = [line for line in proc.stdout.splitlines()
              if line.split(" ", 1)[1:] and line.split(" ", 1)[1].startswith("ec(")]
    if landed:
        chk.add("FAIL", "P3 leg is open",
                "the EC leg for this work item is already committed:\n"
                + "\n".join(f"  {line}" for line in landed))
        return
    chk.add("PASS", "P3 leg is open", f"no ec(...) commit carries `Refs #{iid}`")


def c0_scope(chk: Check, repo: Path, ec: Path, dirty: list[str]) -> None:
    """Every changed path is inside this project's external-contracts tree.

    `CLAUDE.md` fixes the order -- external-contracts, then tech-design, then
    src -- and `/aw:wi-ec-start` has always carried "never write
    `tech-design/**` or `src/**` here" as prose. Prose in a skill body is a
    request. This is the same sentence with a consumer that can refuse it, and
    it works only because `start` demanded a clean tree: from a clean start, a
    `src/**` path showing up here was written by this leg.
    """
    root = f"{ec.relative_to(repo)}/"
    if not dirty:
        chk.add("FAIL", "C0 scope",
                "nothing differs from HEAD; there is no EC change to verify")
        return
    outside = [p for p in dirty if not p.startswith(root)]
    if outside:
        chk.add("FAIL", "C0 scope",
                f"changed outside {root}:\n" + "\n".join(f"  {p}" for p in outside))
        return
    chk.add("PASS", "C0 scope", f"all {len(dirty)} changed paths are under {root}")


def c0b_contract_present(chk: Check, repo: Path, ec: Path, dirty: list[str]) -> list[str]:
    """The change carries at least one case.

    A change that touches only the inventory has registered a promise and
    written nothing able to refuse it. Letting that through -- on the grounds
    that every per-case check below is green over an empty set -- is the exact
    vacuous green this script exists to prevent, and it is worse than the usual
    kind because there is no red anywhere for anyone to notice.
    """
    prefix = f"{(ec / 'src' / 'cases').relative_to(repo)}/"
    cases = sorted(Path(p).stem for p in dirty
                   if p.startswith(prefix) and p.endswith(".py"))
    if not cases:
        chk.add("FAIL", "C0b contract present",
                f"the change touches no case under {prefix}; an inventory entry "
                "with no case is a promise with no verifier")
        return []
    chk.add("PASS", "C0b contract present", ", ".join(cases))
    return cases


def c1_registered(chk: Check, ec: Path, case_id: str, entry: dict[str, Any] | None,
                  tag: str = "") -> None:
    """The case source and the inventory agree, in all four places."""
    path = case_path(ec, case_id)
    if not path.is_file():
        chk.add("FAIL", f"C1 registered{tag}", f"no case source at {path}")
        return
    if entry is None:
        chk.add("FAIL", f"C1 registered{tag}",
                f"`{case_id}` has no [[tool.aw.python-ec.cases]] entry")
        return
    consts = case_constants(path)
    for const, field in (("CASE_ID", "id"), ("DIMENSION", "dimension"), ("TARGET_COMMAND", "command")):
        declared, recorded = consts.get(const), entry.get(field)
        if declared is None:
            chk.add("FAIL", f"C1 registered{tag}", f"case source declares no {const}")
            return
        if re.sub(r"\s+", " ", str(declared)).strip() != re.sub(r"\s+", " ", str(recorded)).strip():
            chk.add(
                "FAIL",
                f"C1 registered{tag}",
                f"{const} disagrees with inventory `{field}`\n"
                f"source   : {declared}\ninventory: {recorded}",
            )
            return
    if consts.get("CASE_ID") != path.stem:
        chk.add("FAIL", f"C1 registered{tag}",
                f"CASE_ID {consts.get('CASE_ID')!r} != filename {path.stem!r}")
        return
    chk.add("PASS", f"C1 registered{tag}", "CASE_ID == filename == inventory id; command matches")


def c2_engineering(chk: Check, ec: Path) -> None:
    """The EC project's own engineering baselines have not regressed.

    The slot is named even while it is empty. An unimplemented check and a
    check that passes vacuously read identically in a report; only one of them
    is honest about it.
    """
    gate = ec / "verification" / "run_all.py"
    if not gate.is_file():
        chk.add(
            "PENDING",
            "C2 engineering baselines",
            f"not wired: {gate.relative_to(ec)} does not exist yet "
            "(mypy / ruff / forbidden-pattern / layering baselines)",
        )
        return
    proc = subprocess.run(
        [sys.executable, str(gate)], cwd=ec, capture_output=True, text=True, timeout=1800
    )
    if proc.returncode != 0:
        chk.add("FAIL", "C2 engineering baselines", proc.stdout.strip()[-1500:])
    else:
        chk.add("PASS", "C2 engineering baselines", "no regression against the recorded baselines")


def c3_green_set(chk: Check, repo: Path, ec: Path, baseline: Path | None) -> None:
    """The cases that were green stay green.

    Not "all cases pass": this suite is not green and has not been for a while,
    so a whole-suite assertion would either be permanently red or quietly
    weakened until it was. A named roster is the only version of this claim
    that can hold.
    """
    if baseline is None or not baseline.is_file():
        chk.add(
            "PENDING",
            "C3 green roster",
            "no --baseline given; the roster of cases that must stay green is unknown",
        )
        return
    roster = json.loads(baseline.read_text(encoding="utf-8")).get("green", [])
    if not roster:
        chk.add("FAIL", "C3 green roster", "baseline names zero green cases; that asserts nothing")
        return
    inv = inventory(ec)
    broke: list[str] = []
    for case_id in roster:
        entry = inv.get(case_id)
        if entry is None:
            broke.append(f"{case_id}: dropped out of the inventory")
            continue
        if run_case(repo, entry)["exit"] != 0:
            broke.append(f"{case_id}: was green, now red")
    if broke:
        chk.add("FAIL", "C3 green roster", "\n".join(broke))
    else:
        chk.add("PASS", "C3 green roster", f"all {len(roster)} previously-green cases still green")


def c4_c5_red_where_declared(
    chk: Check, repo: Path, ec: Path, case_id: str, entry: dict[str, Any], tag: str = ""
) -> None:
    """The case is red, and red on a line it declared in advance.

    C4 alone is satisfied by a typo: a case that cannot import is exactly as
    non-zero as a case whose subject does not exist yet. C5 is what separates
    them -- the terminating exception must be an AssertionError, and its
    message must match one of the assertions the committed source declares.
    """
    result = run_case(repo, entry)
    if result["exit"] == 0:
        chk.add(
            "FAIL",
            f"C4 red{tag}",
            "the case passes against the current tree; a contract that was green "
            "before the change proves nothing about the change",
        )
        return
    chk.add("PASS", f"C4 red{tag}", f"exit {result['exit']}")

    if result["exception"] != "AssertionError":
        chk.add(
            "FAIL",
            f"C5 red where declared{tag}",
            f"terminating exception is {result['exception'] or 'unrecognised'}, not AssertionError\n"
            f"{result['message'][:300]}",
        )
        return
    declared = case_constants(case_path(ec, case_id)).get("ASSERTIONS") or []
    if not isinstance(declared, (list, tuple)) or not declared:
        chk.add("FAIL", f"C5 red where declared{tag}",
                "case declares no ASSERTIONS to match the failure against")
        return

    def norm(s: str) -> str:
        return re.sub(r"[^a-z0-9 ]+", " ", re.sub(r"\s+", " ", str(s).lower())).strip()

    actual = norm(result["message"])
    hit = next((d for d in declared if actual and norm(d)[:40] and norm(d)[:40] in actual), None)
    if hit is None:
        chk.add(
            "FAIL",
            f"C5 red where declared{tag}",
            "the AssertionError matches none of the declared ASSERTIONS\n"
            f"actual  : {result['message'][:200]}\n"
            + "\n".join(f"declared: {d[:120]}" for d in declared[:4]),
        )
        return
    chk.add("PASS", f"C5 red where declared{tag}", f"fails on: {hit[:120]}")


def c6_observes_product(chk: Check, ec: Path, case_id: str, tag: str = "") -> None:
    """The assertions look at something, rather than at themselves.

    C5's escape hatch is `assert False, "<the declared sentence>"`. So is any
    comparison whose two sides are both literals -- the `0 == 0 + 0` shape,
    which reports a passing count while measuring nothing. Both are refused
    here by name, because a rule whose evasion is left unnamed is a rule that
    selects for the evasion.
    """
    tree = ast.parse(case_path(ec, case_id).read_text(encoding="utf-8"))
    offences: list[str] = []
    for node in ast.walk(tree):
        if not isinstance(node, ast.Assert):
            continue
        test = node.test
        if isinstance(test, ast.Constant):
            offences.append(f"line {node.lineno}: `assert <literal>` observes nothing")
        elif isinstance(test, ast.Compare):
            operands = [test.left, *test.comparators]
            if all(_is_literal_expr(o) for o in operands):
                offences.append(f"line {node.lineno}: both sides of the comparison are literals")
    if offences:
        chk.add("FAIL", f"C6 observes the product{tag}", "\n".join(offences))
    else:
        chk.add("PASS", f"C6 observes the product{tag}", "no literal-only assertion")


def _is_literal_expr(node: ast.expr) -> bool:
    """True when the expression is built entirely out of constants."""
    if isinstance(node, LITERAL_NODES):
        return True
    if isinstance(node, ast.BinOp):
        return _is_literal_expr(node.left) and _is_literal_expr(node.right)
    if isinstance(node, ast.UnaryOp):
        return _is_literal_expr(node.operand)
    if isinstance(node, (ast.Tuple, ast.List, ast.Set)):
        return all(_is_literal_expr(e) for e in node.elts)
    return False


def c7_verdict(chk: Check, repo: Path, iid: int, want: str, route: str) -> None:
    """A semantic verdict exists and is bound to these exact bytes."""
    if not route:
        chk.add("FAIL", "C7 verdict", "apps/<project>/aw.toml names no [review] ec route")
        return
    path = verdict_path(repo, f"wi-{iid}")
    if not path.is_file():
        chk.add(
            "FAIL",
            "C7 verdict",
            f"no verdict at {path}\nrun the reviewer named by [review] ec = {route!r}",
        )
        return
    record = json.loads(path.read_text(encoding="utf-8"))
    if record.get("change_digest") != want:
        chk.add(
            "FAIL",
            "C7 verdict",
            "the verdict is bound to different bytes than the ones being committed\n"
            f"reviewed: {record.get('change_digest', '')[:16]}\ncommitting: {want[:16]}",
        )
        return
    if record.get("result") != "accepted":
        chk.add("FAIL", "C7 verdict", f"result is {record.get('result')!r}")
        return
    if record.get("reviewer") != route:
        chk.add(
            "FAIL",
            "C7 verdict",
            f"produced by {record.get('reviewer')!r}, but the route names {route!r}",
        )
        return
    chk.add("PASS", "C7 verdict", f"{route} accepted {want[:16]}")


# --------------------------------------------------------------------------
# the review prompt
# --------------------------------------------------------------------------
RUBRIC = """\
You are reviewing the external-contract (EC) change for ONE work item in the
axiom repository. Below you are given: the work item's body, the list of every
path the change touches, and the full source of every case it adds or edits.

An EC case is a black-box verifier. Its whole job is to pin externally
observable product behaviour so that a wrong implementation cannot pass. These
cases are expected to be RED right now: the behaviour they pin does not exist
yet. Do not report "the test fails" as a finding -- that is the design.

Q0 is the question this review exists to answer. Q1-Q7 are the ways an answer
of "yes" to Q0 can still be worthless.

  Q0 DOES THIS CHANGE SATISFY THE WORK ITEM?
     The work item's `## Goal` names a trigger, an observation point, a current
     value, and a target value; its `## Acceptance` names the gates. Read them
     against the cases actually written here.
     - Is every observable the work item promises pinned by some case?
     - Does any case pin something the work item did not ask for?
     - Where the work item names a specific command, value, or file, does the
       case assert on THAT one, or on something adjacent and easier to hit?
     Name any requirement in the work item that no case here would refuse.

Answer each of the following explicitly too. Each names a way a case can be
green while measuring nothing; if you cannot rule one out from the source
below, say so. Apply them per case, naming the case each finding is about.

  Q1 DISCRIMINATION
     Describe a concrete WRONG implementation that would still make this case
     pass. If you can name one, the case has a hole. If you genuinely cannot,
     say what forces every wrong implementation to fail.

  Q2 ORACLE INDEPENDENCE
     Is the expected value re-derived from the same code path being tested?
     A case that computes its expectation by calling the subject, then asserts
     the subject equals it, is an identity transform and always passes.

  Q3 BLACK BOX
     Does it assert on externally observable behaviour -- stdout, exit code,
     files, tracker state -- or does it reach into implementation internals?
     Internals make the case break on refactors and pass on wrong behaviour.

  Q4 PROMISE vs ASSERTION
     The inventory entry below carries `promise` and `oracle` prose. Does the
     code assert THAT, or something adjacent and easier?

  Q5 POSITIVE CONTROLS
     Does the case prove its own detectors fire? A search that matches nothing,
     a filter that selects zero rows, a regex that never hits -- each is
     compatible with every hypothesis and reports success.

  Q6 VACUITY
     Any assertion that cannot fail: comparisons over empty collections,
     `>= 0` on a length, a loop body that never executes, a try/except that
     swallows the failure.

  Q7 DECLARED FAILURE
     Each case declares ASSERTIONS and currently fails on the one quoted below
     it. Is that failure caused by observing the product, or by the case's own
     setup -- a missing fixture, a wrong path, an import that cannot resolve?

OUTPUT CONTRACT -- parsed mechanically, not read by a human first.

Emit zero or more finding lines, then exactly one verdict line, last:

  FINDING: <one line, name the question number, the case, and what is wrong>
  VERDICT: accepted

Rules: `VERDICT:` must be the final non-empty line. `rejected` requires at
least one `FINDING:` line. Do not emit `VERDICT:` anywhere else in the output.
"""


def cmd_review_prompt(args: argparse.Namespace) -> int:
    """The prompt the reviewer is fed: the work item, then the change.

    Both halves are here because the question is a comparison. A prompt
    carrying only the cases can be answered "these are well-built verifiers"
    by a reviewer who never learned what was asked for, and that answer is
    indistinguishable from the one worth having.
    """
    chk, repo, ec, dirty, cases = _wi_checks(args, require_clean=False, include_verdict=False)
    if chk.failed:
        chk.report()
        raise SystemExit(
            f"#{args.wi} is not mechanically admissible yet, and a semantic review "
            f"of an inadmissible change spends a reviewer on a question the checks "
            f"already answered. Run: ec.py verify {args.wi}"
        )

    print(RUBRIC)
    print("=" * 78)
    print(f"WORK ITEM : #{args.wi}")
    print(f"DIGEST    : {change_digest(repo, args.wi, dirty)}")
    print()
    print("-- the work item this change must satisfy -------------------------")
    print(wi_body_path(repo, args.wi).read_text(encoding="utf-8"))
    print("-- every path this change touches ---------------------------------")
    for path in dirty:
        print(f"  {path}")
    print()

    inv = inventory(ec)
    for case_id in cases:
        entry = inv[case_id]
        result = run_case(repo, entry)
        source = case_path(ec, case_id)
        print("=" * 78)
        print(f"CASE      : {case_id}")
        print(f"DIMENSION : {entry.get('dimension')}")
        print(f"CAPABILITY: {entry.get('capability_id')}")
        print()
        print("-- inventory entry ------------------------------------------------")
        for field in ("promise", "oracle", "target", "applicability", "command"):
            if entry.get(field):
                print(f"{field}: {entry[field]}")
        print()
        print("-- currently fails with -------------------------------------------")
        print(f"exit={result['exit']} {result['exception']}: {result['message'][:600]}")
        print()
        print(f"-- case source: {source.relative_to(repo)} ------------------------")
        print(source.read_text(encoding="utf-8"))

    rest = [p for p in dirty if p not in {str(case_path(ec, c).relative_to(repo)) for c in cases}]
    if rest:
        print("=" * 78)
        print("-- the rest of the change -----------------------------------------")
        tracked = subprocess.run(
            [*GIT, "diff", "HEAD", "--", *rest], cwd=repo, capture_output=True, text=True
        ).stdout
        if tracked.strip():
            print(tracked)
        for path in rest:
            target = repo / path
            if target.is_file() and not tracked_by_git(repo, path):
                print(f"-- new file: {path} --")
                print(target.read_text(encoding="utf-8", errors="replace"))
    return 0


def tracked_by_git(repo: Path, rel: str) -> bool:
    proc = subprocess.run(
        [*GIT, "ls-files", "--error-unmatch", "--", rel],
        cwd=repo, capture_output=True, text=True,
    )
    return proc.returncode == 0


# --------------------------------------------------------------------------
# verbs
# --------------------------------------------------------------------------
def cmd_red(args: argparse.Namespace) -> int:
    repo = repo_root()
    ec = ec_root(repo, args.project)
    entry = inventory(ec).get(args.case)
    if entry is None:
        raise SystemExit(f"`{args.case}` has no inventory entry")
    result = run_case(repo, entry)
    print(f"exit      : {result['exit']}")
    print(f"exception : {result['exception'] or '(none recognised)'}")
    print(f"message   : {result['message'][:400]}")
    return 0 if result["exit"] != 0 else 1


def _run_checks(args: argparse.Namespace) -> Check:
    """The mechanical list narrowed to one case, for looking at it in isolation.

    This is the debugging primitive, not a gate: it cannot see the work item,
    so it cannot tell whether the case belongs to the change being made. The
    gate is `verify`, which derives its population from the diff.
    """
    repo = repo_root()
    ec = ec_root(repo, args.project)
    entry = inventory(ec).get(args.case)
    chk = Check()

    c1_registered(chk, ec, args.case, entry)
    c2_engineering(chk, ec)
    if entry is not None and not chk.failed:
        c3_green_set(chk, repo, ec, Path(args.baseline) if args.baseline else None)
        c4_c5_red_where_declared(chk, repo, ec, args.case, entry)
        c6_observes_product(chk, ec, args.case)
    return chk


def _wi_checks(
    args: argparse.Namespace, *, require_clean: bool, include_verdict: bool
) -> tuple[Check, Path, Path, list[str], list[str]]:
    """Every check this work item's EC leg has to satisfy, recomputed.

    Nothing is read from a previous stage. Each verb runs the whole ladder
    below it, because a stage that trusted a recorded "verify passed" would be
    trusting a boolean that goes stale the moment a file is touched -- and the
    files are touched between every pair of stages here.
    """
    repo = repo_root()
    ec = ec_root(repo, args.project)
    chk = Check()

    p1_work_item(chk, repo, args.wi)
    if chk.failed:
        # Everything below is measured *against* the work item; with no body
        # there is nothing to measure against, and running on would produce
        # rows whose greens mean nothing.
        return chk, repo, ec, [], []
    p3_leg_is_open(chk, repo, args.wi)

    dirty = dirty_set(repo)
    if require_clean:
        p2_clean_tree(chk, dirty)
        return chk, repo, ec, dirty, []

    c0_scope(chk, repo, ec, dirty)
    cases = [] if chk.failed else c0b_contract_present(chk, repo, ec, dirty)
    if cases:
        inv = inventory(ec)
        c2_engineering(chk, ec)
        c3_green_set(chk, repo, ec, Path(args.baseline) if args.baseline else None)
        for case_id in cases:
            tag = f" [{case_id}]"
            entry = inv.get(case_id)
            c1_registered(chk, ec, case_id, entry, tag)
            if entry is None:
                continue
            c4_c5_red_where_declared(chk, repo, ec, case_id, entry, tag)
            c6_observes_product(chk, ec, case_id, tag)
    if include_verdict:
        c7_verdict(chk, repo, args.wi, change_digest(repo, args.wi, dirty),
                   review_route(repo, args.project))
    return chk, repo, ec, dirty, cases


def cmd_check(args: argparse.Namespace) -> int:
    chk = _run_checks(args)
    print(f"mechanical admissibility: {args.case}")
    chk.report()
    print()
    print("These checks say the case is ADMISSIBLE. They do not say the contract")
    print("is meaningful -- nothing here reads what it means. That is the")
    print(f"reviewer's job; see [review] ec in apps/{args.project}/aw.toml.")
    return 1 if chk.failed else 0


def cmd_start(args: argparse.Namespace) -> int:
    """Open the leg: refuse a dirty tree, then print what has to be satisfied."""
    chk, repo, _ec, _dirty, _cases = _wi_checks(
        args, require_clean=True, include_verdict=False
    )
    print(f"opening the EC leg of #{args.wi}")
    chk.report()
    if chk.failed:
        print("\nthe leg was not opened; nothing on disk changed.")
        print("next.command: clear the FAIL rows above, then re-run this verb")
        return 1

    body = wi_body_path(repo, args.wi).read_text(encoding="utf-8")
    mod = change_module()
    print()
    print("=" * 78)
    for heading in ("Goal", "Acceptance"):
        section = mod.section_at(body, 2, heading)
        print(f"## {heading}\n")
        print((section or "(this work item has no such section)").strip())
        print()
    print("=" * 78)
    print("Write the case at")
    print(f"  apps/{args.project}/external-contracts/src/cases/<case-id>.py")
    print("and register it in that project's external-contracts/pyproject.toml.")
    print()
    print("Nothing else may change. The tree was clean when this verb passed, so")
    print("every later stage reads `git status` as the change itself -- and any")
    print("path outside external-contracts/ is refused by name rather than")
    print("committed alongside the contract.")
    print(f"\nnext.command: ec.py verify {args.wi}")
    return 0


def cmd_verify(args: argparse.Namespace) -> int:
    chk, _repo, _ec, _dirty, cases = _wi_checks(
        args, require_clean=False, include_verdict=False
    )
    print(f"mechanical admissibility: #{args.wi}")
    chk.report()
    print()
    print(f"These checks say the change is ADMISSIBLE. They do not say it satisfies")
    print(f"#{args.wi} -- nothing here read the work item's requirements against what")
    print(f"was written. That is the reviewer's job; see [review] ec in")
    print(f"apps/{args.project}/aw.toml.")
    if chk.failed:
        print("\nnext.command: fix the FAIL rows above, then re-run this verb")
        return 1
    print(f"\n{len(cases)} case(s) admissible: {', '.join(cases)}")
    print(f"next.command: /aw:wi-ec-review {args.wi}")
    return 0


def cmd_verdict(args: argparse.Namespace) -> int:
    """Bind a reviewer transcript to the exact bytes it reviewed.

    The transcript is parsed here, not summarised by whoever ran the reviewer.
    An agent that pipes a reviewer's output into this verb has no discretion
    over what the verdict says; an agent that reports the verdict in prose has
    all of it.
    """
    chk, repo, _ec, dirty, cases = _wi_checks(
        args, require_clean=False, include_verdict=False
    )
    if chk.failed:
        print(f"verdict gate: #{args.wi}")
        chk.report()
        print("\nno verdict was recorded. A verdict binds to a change that passed")
        print("the mechanical list; recording one over a change that did not would")
        print("produce an approval nobody could act on, since `commit` re-runs")
        print("these same rows and would refuse it anyway.")
        print(f"next.command: fix the FAIL rows above, then re-run the reviewer")
        return 1

    transcript = Path(args.transcript)
    if not transcript.is_file():
        raise SystemExit(f"no transcript at {transcript}")
    raw = transcript.read_text(encoding="utf-8", errors="replace")

    verdicts = VERDICT_LINE.findall(raw)
    if not verdicts:
        raise SystemExit("transcript carries no `VERDICT: accepted|rejected` line")

    # Measured, not assumed: `codex exec` prints its final answer twice -- once
    # in the streamed body and once as the closing message -- so "exactly one
    # VERDICT line" refused every real transcript. What has to stay refused is
    # the tampering shape, which is *disagreement*: a `rejected` in the body
    # with an `accepted` appended, or the reverse. Requiring unanimity plus a
    # VERDICT as the final non-empty line keeps both halves of that, and neither
    # is satisfied by echoing the same answer n times.
    distinct = sorted(set(verdicts))
    if len(distinct) > 1:
        raise SystemExit(
            f"transcript carries disagreeing verdicts {distinct}; a reviewer that "
            "says both has not decided, and picking one here would be this verb "
            "choosing the answer it was supposed to read"
        )
    tail = [line for line in raw.strip().splitlines() if line.strip()]
    if not VERDICT_LINE.match(tail[-1].strip()):
        raise SystemExit(f"VERDICT must be the final non-empty line; found {tail[-1][:80]!r}")
    result = distinct[0]

    # Same duplication reaches the findings. `dict.fromkeys` dedupes while
    # keeping the order they were raised in, so the record carries each finding
    # once rather than once per echo.
    findings = list(dict.fromkeys(FINDING_LINE.findall(raw)))
    if result == "rejected" and not findings:
        raise SystemExit("a rejected verdict with no FINDING line cannot be acted on")

    route = review_route(repo, args.project)
    if not route:
        raise SystemExit(f"apps/{args.project}/aw.toml names no [review] ec route")

    record = {
        "work_item": args.wi,
        "result": result,
        "reviewer": route,
        "change_digest": change_digest(repo, args.wi, dirty),
        "cases": cases,
        "paths": dirty,
        "transcript_digest": hashlib.sha256(raw.encode()).hexdigest(),
        "findings": findings,
    }
    out = verdict_path(repo, f"wi-{args.wi}")
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    shutil.copyfile(transcript, out.with_suffix(".transcript.txt"))

    print(f"recorded {result} by {route}")
    print(f"  change    {record['change_digest'][:16]} over {len(dirty)} path(s)")
    print(f"  transcript {record['transcript_digest'][:16]} -> {out.with_suffix('.transcript.txt')}")
    for f in findings:
        print(f"  FINDING: {f}")
    if result != "accepted":
        print("\nnext.command: address the findings, then re-run the reviewer")
        return 1
    print(f"\nnext.command: /aw:wi-ec-commit {args.wi}")
    return 0


def cmd_commit(args: argparse.Namespace) -> int:
    chk, repo, _ec, dirty, _cases = _wi_checks(
        args, require_clean=False, include_verdict=True
    )
    print(f"commit gate: #{args.wi}")
    chk.report()
    if chk.failed:
        print("\nnothing was committed; the tree is unchanged and the work is still here.")
        print(f"next.command: fix the FAIL rows above, then re-run this verb")
        return 1

    # The allowlist *is* the dirty set, which is what makes the commit and the
    # thing that was reviewed the same object. A hand-written allowlist would
    # let a path be reviewed and then not committed, or committed without ever
    # having been in the digest -- and neither would show up as a red row.
    allowlist = dirty
    digest = change_digest(repo, args.wi, dirty)
    trailers = [
        f"EC-Review: {review_route(repo, args.project)}",
        f"EC-Change-Digest: {digest}",
    ]
    for _status, name, detail in chk.pending:
        trailers.append(f"EC-Pending: {name} ({detail.splitlines()[0]})")
    message = f"ec(wi-{args.wi}): pin the contract before the implementation\n"
    message += f"\nRefs #{args.wi}\n"
    message += "\n" + "\n".join(trailers) + "\n"

    if args.dry_run:
        print("\n-- would commit, exactly these paths ------------------------")
        for p in allowlist:
            print(f"  {p}")
        print("-- message -------------------------------------------------")
        print(message)
        return 0

    # A brand-new case is untracked, and `git commit -- <pathspec>` refuses a
    # path git has never seen. Staging the allowlist first is what makes the
    # common shape -- the one every new case has -- work at all. The pathspec
    # stays on the commit so anything else already in the index is left there.
    add = subprocess.run([*GIT, "add", "--", *allowlist], cwd=repo, capture_output=True, text=True)
    if add.returncode != 0:
        print(add.stderr)
        return add.returncode
    proc = subprocess.run(
        [*GIT, "commit", "-m", message, "--", *allowlist],
        cwd=repo, capture_output=True, text=True,
    )
    print(proc.stdout or proc.stderr)
    if proc.returncode != 0:
        return proc.returncode

    # The link the commit makes is one-way: it carries `Refs #<iid>`, and
    # nothing on the work item points back. Recovering the commit from the
    # tracker means running `git log --grep` in a checkout, which is not
    # available to anyone reading the issue.
    #
    # Closing that is a tracker write, and `ec.py` does not make tracker
    # writes -- the same boundary that makes the skill run `change.py fetch`
    # rather than reading the issue here. So this resolves the sha, prints it,
    # and names the verb that records it. Resolved rather than parsed out of
    # git's own output: that line carries an abbreviated sha whose length is a
    # local config, and half a link is worse than none.
    head = subprocess.run([*GIT, "rev-parse", "HEAD"],
                          cwd=repo, capture_output=True, text=True)
    if head.returncode != 0:
        print(head.stderr)
        return head.returncode
    sha = head.stdout.strip()
    print(f"EC-Commit: {sha}")
    print(f"\nnext.command: change.py lifecycle {args.wi} --leg ec "
          f"--commit {sha} --digest {digest}")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="ec.py", description=__doc__.splitlines()[0])
    parser.add_argument("--project", default=DEFAULT_PROJECT, help="project under apps/")
    sub = parser.add_subparsers(dest="verb", required=True)

    # Shared shapes, declared once. `--baseline` in particular is not optional
    # in practice -- the suite as a whole is not green, so "every case passes"
    # is not a claim anyone can gate on, and the roster is the version of it
    # that can hold. Four copies of that flag is four places to forget it.
    wi = argparse.ArgumentParser(add_help=False)
    wi.add_argument("wi", type=int, help="work item iid")
    baseline = argparse.ArgumentParser(add_help=False)
    baseline.add_argument("--baseline", help="JSON naming the cases that must stay green")

    p = sub.add_parser("red", help="run one case and classify how it fails")
    p.add_argument("--case", required=True)
    p.set_defaults(func=cmd_red)

    p = sub.add_parser("check", parents=[baseline],
                       help="the mechanical list narrowed to one case")
    p.add_argument("--case", required=True)
    p.set_defaults(func=cmd_check)

    p = sub.add_parser("start", parents=[wi],
                       help="open a work item's EC leg; refuses a dirty tree")
    p.set_defaults(func=cmd_start)

    p = sub.add_parser("verify", parents=[wi, baseline],
                       help="the closed mechanical list over the whole change")
    p.set_defaults(func=cmd_verify)

    p = sub.add_parser("review-prompt", parents=[wi, baseline],
                       help="emit the prompt the reviewer is fed")
    p.set_defaults(func=cmd_review_prompt)

    p = sub.add_parser("verdict", parents=[wi, baseline],
                       help="bind a reviewer transcript to the bytes it reviewed")
    p.add_argument("--transcript", required=True)
    p.set_defaults(func=cmd_verdict)

    p = sub.add_parser("commit", parents=[wi, baseline],
                       help="re-check, require a live verdict, commit the change")
    p.add_argument("--dry-run", action="store_true")
    p.set_defaults(func=cmd_commit)

    args = parser.parse_args(argv)
    if not hasattr(args, "baseline"):
        args.baseline = None
    return int(args.func(args))


if __name__ == "__main__":
    raise SystemExit(main())
