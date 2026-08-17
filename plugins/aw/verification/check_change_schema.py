#!/usr/bin/env python3
"""Refuse a change facade whose schema has quietly changed under it.

`change.py` used to be a *port*. `apps/agentic-workflow/src/issues/ghan.rs`
owned the change schema, and this gate read the crate as an oracle in three
ways -- constants, template, and a replay of every `#[test]` in `ghan.rs`. That
crate is gone, so the port is now the original and there is nothing external
left to compare it against.

Deleting an oracle costs discrimination unless something replaces it, and the
thing the crate actually caught was narrowing: a word list re-typed one entry
shorter, invisible because both sides were edited together. Three replacements
carry that load, and each catches something the crate channel did not.

  declared inventory   the four H2s, the six H3s, the fifteen hedges and the
                       seven failure assertions are written out below as
                       literals and compared to `change.py`. This is the gate
                       declaring the schema, not a copy of a foreign file: the
                       plugin owns these words now, and owning them means
                       stating them somewhere a diff has to go through.

  liveness             every entry in those lists is then *used* -- each hedge
                       must get a premise refused, each failure assertion must
                       get a negative control accepted, each heading must be
                       load-bearing. A declared inventory alone would let a
                       word sit in both lists doing nothing; the crate oracle
                       never checked this, because two identical dead lists
                       agree perfectly.

  refusal coverage     every one of the 24 `errors.append` sites in the five
                       `validate_*` functions must be reached by some case
                       below, measured by tracing rather than counted by hand.
                       This is what replaces "every crate test has a replay":
                       a claim about coverage of the rules that exist, instead
                       of coverage of a test list in another language.

The inventory and the liveness probe have to travel together, and neither is
sufficient. A probe loop derived from the list cannot see the list shrink --
delete a hedge and its probe deletes itself -- which is exactly the failure
being defended against. The declared literals are what make the loop honest.

The fixtures under `_fixtures/` were lifted out of the crate before it was
removed, at a point where crate and port were measured to agree on all five
channels (4 sections, 15 hedges, 7 failure phrases, 987B template, and a
sample body validating clean). They are the plugin's own now.

Also gone with the crate: `measure_change_agreement.py`, which built the crate
binary and ran it against all 640 live change bodies. Its finding is worth
keeping even though the tool is not -- 619 of those bodies are refused on the
structural tier, a missing or unexpected H2, which short-circuits before any
section rule runs. Only 21 reach `validate_goal` and its siblings, and they
pass. So the live population exercises the per-section rules almost entirely on
their non-firing path, and the cases below are the only place those rules are
observed to fire at all. That was true while the crate existed too; the crate
was never what made it true.

Deliberately not modelled: `looks_too_large_for_atomic_wi`. It is a boundedness
heuristic over prose rather than a section rule, and its false-positive
behaviour on ordinary sentences is documented elsewhere.
"""
import ast
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import (CHANGE_SCRIPT, HERE,  # noqa: E402
                    load_change_module, load_epic_module)

FIXTURES = HERE / "_fixtures"

fails = []


def check(label, ok, detail=""):
    print(f"{'PASS' if ok else 'FAIL'} {label}{(' -- ' + detail) if detail else ''}")
    if not ok:
        fails.append(label)


check("the change facade is beside the engine", CHANGE_SCRIPT.is_file(), str(CHANGE_SCRIPT))
if not CHANGE_SCRIPT.is_file():
    print("\n=> RED (1 failure(s))")
    sys.exit(1)

mod = load_change_module()

# --------------------------------------------------------------------------
# The fixtures the plugin owns
# --------------------------------------------------------------------------

SAMPLE = FIXTURES / "sample_change_body.md"
SKELETON = FIXTURES / "change_skeleton.md"

check("positive control: the fixtures are on disk",
      SAMPLE.is_file() and SKELETON.is_file(), str(FIXTURES))
if fails:
    print(f"\n=> RED ({len(fails)} failure(s))")
    sys.exit(1)

GOOD_BODY = SAMPLE.read_text(encoding="utf-8")
GOOD_SKELETON = SKELETON.read_text(encoding="utf-8")

# Control: a truncated or empty fixture would make every mutation below a
# mutation of nothing, and the negative cases would pass by vacuity.
check("positive control: the sample body is whole",
      len(GOOD_BODY) > 1000 and GOOD_BODY.startswith("## Goal") and "## Never" in GOOD_BODY,
      f"{len(GOOD_BODY)}B")

# --------------------------------------------------------------------------
# Declared inventory -- the schema, stated by the gate
# --------------------------------------------------------------------------

DECLARED_H2 = ("## Goal", "## How", "## Acceptance", "## Never")
DECLARED_H3 = {
    "HOW_PREMISES": "### Verified premises",
    "HOW_CHANGE_POINTS": "### Change points",
    "HOW_FROZEN": "### Frozen decisions",
    "ACCEPTANCE_NEGATIVE_CONTROL": "### Negative control",
    "NEVER_MUST_NOT_TOUCH": "### Must not touch",
    "NEVER_MUST_NOT_DO": "### Must not do",
}
DECLARED_HEDGES = (
    "should", "might", "probably", "seems", "appears", "likely", "presumably",
    "supposedly", "應該", "可能", "推測", "看起來", "似乎", "大概", "或許",
)
DECLARED_FAILURES = (
    "must fail", "must go red", "must be red", "must turn red",
    "必須紅", "必須失敗", "必须红",
)

check("the four H2 sections are the declared ones",
      tuple(mod.GHAN_SECTIONS) == DECLARED_H2, str(DECLARED_H2))
check("the hedge vocabulary is the declared one",
      tuple(mod.HEDGE_WORDS) == DECLARED_HEDGES,
      f"{len(DECLARED_HEDGES)} words; "
      f"differs={sorted(set(mod.HEDGE_WORDS) ^ set(DECLARED_HEDGES))}")
check("the failure assertions are the declared ones",
      tuple(mod.FAILURE_ASSERTIONS) == DECLARED_FAILURES,
      f"{len(DECLARED_FAILURES)} phrases; "
      f"differs={sorted(set(mod.FAILURE_ASSERTIONS) ^ set(DECLARED_FAILURES))}")
check("the six H3 sub-sections are the declared ones",
      {name: getattr(mod, name) for name in DECLARED_H3} == DECLARED_H3,
      ", ".join(sorted(DECLARED_H3.values())))
check("the skeleton is the declared empty change body",
      mod.skeleton() == GOOD_SKELETON,
      f"{len(mod.skeleton())}B vs the fixture's {len(GOOD_SKELETON)}B")

# --------------------------------------------------------------------------
# Case machinery
# --------------------------------------------------------------------------


def body_without(section):
    """Drop one H2 section and everything under it."""
    out, skipping = [], False
    for line in GOOD_BODY.splitlines():
        if line.startswith("## "):
            skipping = line.strip() == section
        if not skipping:
            out.append(line + "\n")
    return "".join(out)


def errors(body):
    return mod.validate_body(body)


def has(body, needle):
    return any(needle in error for error in errors(body))


def swap(find, replace):
    return GOOD_BODY.replace(find, replace)


PREMISE = "- `apps/agentic-workflow/src/cli/issues.rs:2176` pushes the unstructured error and early-returns."
GOAL_LINE = ("Running `aw wi validate` on a GHAN body reports section errors instead of "
             "`body must contain structured work-item sections`.")
GATE_ROW = ("| 1 | `cargo test -p agentic-workflow --lib -- --test-threads=1` | "
            "3755 passed / 0 failed | 3770 passed / 0 failed | the new cases assert "
            "refusal strings that do not exist before the change |")

COMMENT_ONLY_BODY = """## Goal

<!-- Goal must be one observable-difference sentence, not a list. -->

## How

### Verified premises

<!-- Premise needs at least one observed premise with a file:line evidence coordinate. -->

### Change points

<!-- Change point must name at least one write target path. -->

### Frozen decisions

<!-- Must record decisions. -->

## Acceptance

<!-- Table with columns -->

### Negative control

<!-- Negative control -->

## Never

This addresses the worker implementing this work item, not the controller reviewing it.

### Must not touch

<!-- Must not touch -->

### Must not do

<!-- Must not do -->
"""


def case_good():
    return errors(GOOD_BODY) == [], f"errors={errors(GOOD_BODY)}"


def case_shape():
    return (
        mod.body_shape(GOOD_BODY) == "ghan"
        and mod.body_shape("## Problem\n\nx\n\n## Requirements\n\n- R1: y\n") == "legacy"
        and mod.body_shape("## Goal\n\nx\n\n## Problem\n\ny\n") == "mixed"
        and mod.body_shape("just prose\n") == "unstructured"
    ), "ghan/legacy/mixed/unstructured"


def case_acceptance_criteria_prefix():
    body = "## Problem\n\nx\n\n## Acceptance Criteria\n\n- AC1: y\n"
    return mod.body_shape(body) == "legacy", f"got {mod.body_shape(body)}"


def case_goal_shape():
    listed = swap(GOAL_LINE, "- one goal\n- another goal")
    two = swap(GOAL_LINE, "First observation point moves.\n\nSecond observation point also moves.")
    return has(listed, "not a list") and has(two, "single paragraph"), ""


def case_premise_coordinate():
    return has(swap(PREMISE, "- the validator rejects unstructured bodies."),
               "no `file:line` evidence coordinate"), ""


def case_change_points_empty():
    body = (GOOD_BODY
            .replace("- `apps/agentic-workflow/src/cli/issues.rs` — route by body shape.\n", "")
            .replace("- `apps/agentic-workflow/src/issues/ghan.rs` — the validator itself.\n", ""))
    return has(body, "--type spike"), ""


def case_gate_cannot_discriminate():
    return has(swap("| 3755 passed / 0 failed | 3770 passed / 0 failed |",
                    "| 3755 passed / 0 failed | 3755 passed / 0 failed |"),
               "cannot discriminate"), ""


def case_missing_negative_control():
    return has(swap("### Negative control", "### Notes"),
               "a gate nobody has seen fail proves nothing"), ""


def case_negative_control_content():
    no_digest = swap("59d66dea106b9bd7c8c319d9096f1e5fe1c82957faa4837a8fa8c7cd6528a32b",
                     "the original content")
    no_red = swap("the new cases must fail.", "check the result.")
    return has(no_digest, "must name the sha256") and has(no_red, "must require the gate to go red"), ""


def case_never_addressee():
    body = swap("The addressee of these limits is the agent executing this work item, "
                "not the dispatcher.", "- no addressee, straight to a list")
    return has(body, "fixing the addressee"), ""


def case_change_point_vs_must_not_touch():
    body = swap("- `apps/agentic-workflow/external-contracts/src/wi_contract_fixture.py`",
                "- `apps/agentic-workflow/src/issues/ghan.rs`")
    return has(body, "both a change point and must-not-touch"), ""


def case_foreign_h2():
    return has(f"{GOOD_BODY}\n## Notes\n\nanything\n", "unexpected H2 `## Notes`"), ""


def case_fenced_headings():
    body = swap("### Negative control\n", "### Negative control\n\n```sh\n## not a heading\n```\n")
    return mod.body_shape(body) == "ghan" and errors(body) == [], f"errors={errors(body)}"


def case_table_rows():
    return mod.table_rows("| a | b |\n|---|---|\n| 1 | 2 |\n") == [["1", "2"]], ""


def case_path_helpers():
    return (
        mod.file_line_ref("see `src/cli/issues.rs:2176` there") is not None
        and mod.file_line_ref("see issue #3358") is None
        and mod.path_ref("touch `src/issues/ghan.rs`") is not None
        and mod.path_ref("touch the validator") is None
        and mod.normalize_path("`src/cli/issues.rs:2176`") == "src/cli/issues.rs"
    ), ""


def case_comments_stripped():
    found = errors(COMMENT_ONLY_BODY)
    return (
        "ghan: ## Goal is empty" in found
        and any("`### Verified premises` needs at least one observed premise" in e for e in found)
    ), f"errors={len(found)}"


# The five cases below were added because the refusal-coverage assertion at the
# bottom reported their sites unreached on its first run. Each names a rule
# `validate_*` has always enforced and nothing had ever observed firing.

def case_goal_placeholder():
    return has(swap(GOAL_LINE, "Running the validator (fill) reports section errors."),
               "still carries the `(fill)` placeholder"), ""


def case_change_point_without_path():
    return has(swap("- `apps/agentic-workflow/src/cli/issues.rs` — route by body shape.",
                    "- route by body shape, without naming where."),
               "change point names no path"), ""


def case_acceptance_needs_a_row():
    return has(swap(GATE_ROW + "\n", ""),
               "needs a gate table with at least one row"), ""


def case_gate_row_column_count():
    return has(swap(GATE_ROW, "| 1 | `cargo test -p agentic-workflow` | 0 |"),
               "needs 5 columns"), ""


def case_gate_row_needs_a_why():
    return has(swap(GATE_ROW,
                    "| 1 | `cargo test -p agentic-workflow` | 3755 passed | 3770 passed | (fill) |"),
               "must say why it cannot hold by accident"), ""


def case_gate_command_backticked():
    return has(swap(GATE_ROW,
                    "| 1 | cargo test -p agentic-workflow | 3755 passed | 3770 passed | "
                    "the new cases assert refusal strings that do not exist before |"),
               "must be a verbatim backticked command"), ""


CASES = {
    "a good body passes every section rule": case_good,
    "shape detection separates ghan from legacy and mixed": case_shape,
    "an `## Acceptance Criteria` heading is not read as `## Acceptance`":
        case_acceptance_criteria_prefix,
    "a goal that is a list, or two paragraphs, is refused": case_goal_shape,
    "a premise without a file:line is refused": case_premise_coordinate,
    "an empty change-point list routes to a spike": case_change_points_empty,
    "a gate whose current equals its target cannot discriminate":
        case_gate_cannot_discriminate,
    "a missing negative control is refused": case_missing_negative_control,
    "a negative control needs a restore digest and a red assertion":
        case_negative_control_content,
    "`## Never` needs an addressee line before its lists": case_never_addressee,
    "a change point cannot also be must-not-touch": case_change_point_vs_must_not_touch,
    "a foreign H2 is refused so the four sections stay the contract": case_foreign_h2,
    "fenced headings do not split sections": case_fenced_headings,
    "table rows skip the header and separator": case_table_rows,
    "the path and file:line helpers reject non-coordinates": case_path_helpers,
    "html comments are stripped so comment-only slots fail": case_comments_stripped,
    "a `(fill)` placeholder left in `## Goal` is refused": case_goal_placeholder,
    "a change point that names no path is refused": case_change_point_without_path,
    "`## Acceptance` with no table row is refused": case_acceptance_needs_a_row,
    "a gate row with fewer than 5 columns is refused": case_gate_row_column_count,
    "a gate row whose why-column is a placeholder is refused": case_gate_row_needs_a_why,
    "a gate command that is not backticked is refused": case_gate_command_backticked,
}

# --------------------------------------------------------------------------
# Liveness: every declared entry does work
#
# Written as generated cases rather than a bare loop so each entry shows up in
# the output by name -- a silent loop reports one line whether it covered
# fifteen words or one.
# --------------------------------------------------------------------------

for _w in DECLARED_HEDGES:
    def _hedge_case(w=_w):
        body = swap(PREMISE, f"- `apps/agentic-workflow/src/cli/issues.rs:2176` {w} push the error.")
        return has(body, f"hedges with '{w}'"), ""
    CASES[f"the hedge `{_w}` refuses a premise"] = _hedge_case

for _p in DECLARED_FAILURES:
    def _failure_case(p=_p):
        body = swap("the new cases must fail.", f"the new cases {p}.")
        return errors(body) == [], f"errors={errors(body)}"
    CASES[f"the failure assertion `{_p}` is accepted"] = _failure_case

for _s in DECLARED_H2:
    def _h2_case(s=_s):
        return f"ghan: missing required {s} section" in errors(body_without(s)), ""
    CASES[f"a missing `{_s}` is named"] = _h2_case

for _name, _h3 in DECLARED_H3.items():
    def _h3_case(h3=_h3):
        return errors(swap(f"{h3}\n", "### Renamed\n")) != [], ""
    CASES[f"the sub-section `{_h3}` is load-bearing"] = _h3_case

# --------------------------------------------------------------------------
# Run every case, tracing which refusal sites they reach
# --------------------------------------------------------------------------

SOURCE = CHANGE_SCRIPT.read_text(encoding="utf-8")
TREE = ast.parse(SOURCE)

REFUSAL_SITES = {}
for node in ast.walk(TREE):
    if isinstance(node, ast.FunctionDef) and node.name.startswith("validate_"):
        for inner in ast.walk(node):
            if (isinstance(inner, ast.Call)
                    and isinstance(inner.func, ast.Attribute)
                    and inner.func.attr == "append"):
                REFUSAL_SITES[inner.lineno] = node.name

check("positive control: the refusal sites were found",
      len(REFUSAL_SITES) >= 20,
      f"{len(REFUSAL_SITES)} sites across "
      f"{len(set(REFUSAL_SITES.values()))} validate_* functions")

reached = set()
CHANGE_FILE = str(CHANGE_SCRIPT)


def tracer(frame, event, arg):
    if event == "call":
        return tracer if frame.f_code.co_filename == CHANGE_FILE else None
    if event == "line" and frame.f_lineno in REFUSAL_SITES:
        reached.add(frame.f_lineno)
    return tracer


sys.settrace(tracer)
try:
    results = {name: case() for name, case in CASES.items()}
finally:
    sys.settrace(None)

for name, (ok, detail) in results.items():
    check(name, ok, detail)

unreached = sorted(set(REFUSAL_SITES) - reached)
check("every refusal site in the validators is reached by some case",
      not unreached,
      f"{len(reached)}/{len(REFUSAL_SITES)} reached; "
      f"unreached={[f'{ln} in {REFUSAL_SITES[ln]}' for ln in unreached]}")

# --------------------------------------------------------------------------
# The one thing the two facades must agree about
#
# A change names its epic with `epic:<iid>`; an epic reads exactly that label
# to find its children. The two spellings are written out independently -- the
# alternative is one facade importing the other, which would make the epic
# surface a dependency of a type that exists with or without any epic. So the
# duplication is deliberate, and this is what keeps it from being a divergence
# waiting to happen: rename either side alone and the link goes silently dead,
# with `create --epic` writing a label `children` does not look for.
# --------------------------------------------------------------------------

epic_mod = load_epic_module()
check("the ownership label prefix agrees with the epic facade",
      mod.PARENT_LABEL_PREFIX == epic_mod.CHILD_LABEL_PREFIX,
      f"change={mod.PARENT_LABEL_PREFIX!r} epic={epic_mod.CHILD_LABEL_PREFIX!r}")

# Control: the case machinery must be able to report a failure. Every case
# above runs through `has`/`errors`, so a validator that returned `[]` for
# everything would satisfy the negative cases only if those cases are actually
# read -- this proves an impossible expectation is seen as failing.
impossible = has(GOOD_BODY, "ghan: this error string cannot exist")
check("positive control: an unsatisfiable expectation is reported as failing",
      not impossible, "an impossible needle is not silently found")

print("\n=> " + ("GREEN" if not fails else f"RED ({len(fails)} failure(s))"))
sys.exit(1 if fails else 0)
