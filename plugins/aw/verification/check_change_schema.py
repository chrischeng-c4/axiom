#!/usr/bin/env python3
"""Refuse a change facade that has drifted from the crate that owns its schema.

The epic schema is this plugin's own invention, so `epic.py` holds it as data
and the gates check that data against live epics. The change schema is not:
`apps/agentic-workflow/src/issues/ghan.rs` owns it, `aw wi validate` enforces
it, and 640 work items are already judged by it. A hand-written second reading
of those rules is not a schema -- it is a fork with a delay fuse, and the fork
is invisible precisely while both sides agree.

So `change.py` is a *port*, and this gate is what keeps a port a port. It reads
the crate as the oracle in three ways, none of which this repository's plugin
side gets to author:

  constants   the four H2s, the six H3s, the hedge vocabulary and the failure
              assertions are extracted from `ghan.rs` and compared to the
              port's. Re-typing a word list is how a rule silently narrows.
  template    the empty change body is extracted from `issues.rs` and compared
              to `change.py skeleton`. Two surfaces handing out two different
              forms is the same fork one layer up.
  corpus      every `#[test]` in `ghan.rs` is replayed against the port. The
              crate's own acceptance suite, applied to the second reading.

The corpus replay carries the assertion that makes it non-optional: the number
of replayed cases must equal the number of `#[test]` functions in `ghan.rs`. A
rule added to the crate turns this gate red until the port learns it, which is
the only moment a drift is cheap to fix.

Deliberately not ported: `looks_too_large_for_atomic_wi`, which
`validate_ghan_body` also runs. It is a boundedness heuristic over prose rather
than a section rule, it lives in `planner.rs`, and its false-positive behaviour
on ordinary sentences is documented elsewhere. The live differential in
`measure_change_agreement.py` calls `validate_ghan_sections` on both sides, so
it excludes that rule symmetrically rather than tolerating it on one side.

That differential and this corpus cover different halves of the schema, and
neither one subsumes the other. Measured over all 640 live change bodies: 619
are refused on the structural tier -- a missing or unexpected H2 -- which
short-circuits before any section rule runs, and the 21 that do reach
`validate_goal` and its three siblings pass all of them. So the live population
exercises the per-section rules only on their *non-firing* path, and every case
where those rules actually fire comes from the crate's own tests, replayed
here. A ported rule that is too strict shows up live as a spurious error; one
that is too lax is invisible there and catchable only in this corpus.
"""
import json
import pathlib
import re
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import (CHANGE_SCRIPT, GHAN_RS, ISSUES_RS,  # noqa: E402
                    load_change_module, load_epic_module)

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
# The crate, read as the oracle
# --------------------------------------------------------------------------

GHAN_SRC = GHAN_RS.read_text(encoding="utf-8")
ISSUES_SRC = ISSUES_RS.read_text(encoding="utf-8")


def rust_str_list(src, name):
    """Members of a `const NAME: &[&str] = &[...]` declaration."""
    found = re.search(rf"const {name}: &\[&str\] = &\[(.*?)\];", src, re.S)
    return tuple(re.findall(r'"((?:[^"\\]|\\.)*)"', found.group(1))) if found else ()


def rust_str_const(src, name):
    """The value of a `const NAME: &str = "..."` declaration."""
    found = re.search(rf'const {name}: &str = "((?:[^"\\]|\\.)*)";', src)
    return found.group(1) if found else None


def rust_raw_const(src, name):
    """The value of a `const NAME: &str = r#"..."#` declaration."""
    found = re.search(rf'{name}: &str = r#"(.*?)"#;', src, re.S)
    return found.group(1) if found else None


def crate_change_template(src):
    """The empty change body `aw wi` hands out, from its one `return` site."""
    found = re.search(r'return "(## Goal.*?)"\.to_string\(\);', src)
    return json.loads('"' + found.group(1) + '"') if found else None


CRATE_SECTIONS = rust_str_list(GHAN_SRC, "GHAN_SECTIONS")
CRATE_HEDGES = rust_str_list(GHAN_SRC, "HEDGE_WORDS")
CRATE_FAILURES = rust_str_list(GHAN_SRC, "FAILURE_ASSERTIONS")
CRATE_H3 = {
    name: rust_str_const(GHAN_SRC, name)
    for name in ("HOW_PREMISES", "HOW_CHANGE_POINTS", "HOW_FROZEN",
                 "ACCEPTANCE_NEGATIVE_CONTROL", "NEVER_MUST_NOT_TOUCH", "NEVER_MUST_NOT_DO")
}
GOOD_BODY = rust_raw_const(GHAN_SRC, "SAMPLE_GHAN_BODY")
CRATE_TEMPLATE = crate_change_template(ISSUES_SRC)

# Control: every extractor above must have found something. A regex that
# silently returns `()` would make each comparison below a tautology -- the
# port would be measured against nothing and reported green.
check("positive control: the crate constants extract",
      all([CRATE_SECTIONS, CRATE_HEDGES, CRATE_FAILURES, GOOD_BODY, CRATE_TEMPLATE])
      and all(CRATE_H3.values()),
      f"{len(CRATE_SECTIONS)} sections, {len(CRATE_HEDGES)} hedges, "
      f"{len(CRATE_FAILURES)} failure phrases, {len(CRATE_H3)} H3s, "
      f"body={len(GOOD_BODY or '')}B template={len(CRATE_TEMPLATE or '')}B")

if fails:
    print(f"\n=> RED ({len(fails)} failure(s))")
    sys.exit(1)

# --------------------------------------------------------------------------
# Constant agreement
# --------------------------------------------------------------------------

check("the four H2 sections agree with the crate",
      tuple(mod.GHAN_SECTIONS) == CRATE_SECTIONS, str(CRATE_SECTIONS))
check("the hedge vocabulary agrees with the crate",
      tuple(mod.HEDGE_WORDS) == CRATE_HEDGES, f"{len(CRATE_HEDGES)} words")
check("the failure assertions agree with the crate",
      tuple(mod.FAILURE_ASSERTIONS) == CRATE_FAILURES, f"{len(CRATE_FAILURES)} phrases")

PORT_H3 = {
    "HOW_PREMISES": mod.HOW_PREMISES,
    "HOW_CHANGE_POINTS": mod.HOW_CHANGE_POINTS,
    "HOW_FROZEN": mod.HOW_FROZEN,
    "ACCEPTANCE_NEGATIVE_CONTROL": mod.ACCEPTANCE_NEGATIVE_CONTROL,
    "NEVER_MUST_NOT_TOUCH": mod.NEVER_MUST_NOT_TOUCH,
    "NEVER_MUST_NOT_DO": mod.NEVER_MUST_NOT_DO,
}
check("the six H3 sub-sections agree with the crate", PORT_H3 == CRATE_H3,
      ", ".join(sorted(CRATE_H3.values())))

# --------------------------------------------------------------------------
# Template agreement
# --------------------------------------------------------------------------

check("the skeleton is the crate's own empty change body",
      mod.skeleton() == CRATE_TEMPLATE,
      f"{len(mod.skeleton())}B vs the crate's {len(CRATE_TEMPLATE)}B")

# --------------------------------------------------------------------------
# Corpus replay: every `#[test]` in `ghan.rs`, against the port
# --------------------------------------------------------------------------


def body_without(section):
    """The crate test helper: drop one H2 section and everything under it."""
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


def case_missing_named():
    missed = [s for s in CRATE_SECTIONS
              if f"ghan: missing required {s} section" not in errors(body_without(s))]
    return not missed, f"unnamed={missed}"


def case_goal_shape():
    listed = swap(GOAL_LINE, "- one goal\n- another goal")
    two = swap(GOAL_LINE, "First observation point moves.\n\nSecond observation point also moves.")
    return has(listed, "not a list") and has(two, "single paragraph"), ""


def case_premise_coordinate():
    return has(swap(PREMISE, "- the validator rejects unstructured bodies."),
               "no `file:line` evidence coordinate"), ""


def case_premise_hedge():
    body = swap(PREMISE,
                "- `apps/agentic-workflow/src/cli/issues.rs:2176` should push the unstructured error.")
    return has(body, "hedges with 'should'"), ""


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


# Keyed by the crate test each case replays, so the count assertion below is a
# claim about coverage rather than about how many functions live in this file.
CASES = {
    "good_ghan_body_passes_every_section_rule": case_good,
    "shape_detection_separates_ghan_from_legacy_and_mixed": case_shape,
    "acceptance_criteria_heading_is_not_read_as_acceptance": case_acceptance_criteria_prefix,
    "each_missing_section_is_named": case_missing_named,
    "goal_rejects_a_list_and_a_second_paragraph": case_goal_shape,
    "premise_without_file_line_is_refused": case_premise_coordinate,
    "hedged_premise_is_refused": case_premise_hedge,
    "empty_change_point_list_routes_to_spike": case_change_points_empty,
    "gate_with_identical_current_and_target_cannot_discriminate": case_gate_cannot_discriminate,
    "missing_negative_control_is_refused": case_missing_negative_control,
    "negative_control_needs_a_restore_digest_and_a_red_assertion": case_negative_control_content,
    "never_needs_an_addressee_line_before_its_lists": case_never_addressee,
    "a_change_point_cannot_also_be_must_not_touch": case_change_point_vs_must_not_touch,
    "foreign_h2_is_refused_so_the_four_sections_stay_the_contract": case_foreign_h2,
    "fenced_headings_do_not_split_sections": case_fenced_headings,
    "table_rows_skips_header_and_separator": case_table_rows,
    "path_and_file_line_helpers_reject_non_coordinates": case_path_helpers,
    "html_comments_are_stripped_so_comment_only_slots_fail_validation": case_comments_stripped,
}

crate_tests = set(re.findall(r"#\[test\]\s*\n\s*fn (\w+)\(", GHAN_SRC))
check("every crate test has a replay here",
      crate_tests == set(CASES),
      f"crate={len(crate_tests)} replayed={len(CASES)} "
      f"unreplayed={sorted(crate_tests - set(CASES))} stale={sorted(set(CASES) - crate_tests)}")

for name, case in CASES.items():
    ok, detail = case()
    check(f"replay: {name}", ok, detail)

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

# Control: the replay machinery must be able to report a failure. Every case
# above runs through `has`/`errors`, so a port that returned `[]` for
# everything would satisfy the negative cases only if those cases are actually
# read -- this proves an impossible expectation is seen as failing.
impossible = has(GOOD_BODY, "ghan: this error string cannot exist")
check("positive control: the replayer reports a false expectation as failing",
      not impossible, "an unsatisfiable expectation is not silently true")

print("\n=> " + ("GREEN" if not fails else f"RED ({len(fails)} failure(s))"))
sys.exit(1 if fails else 0)
