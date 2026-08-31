#!/usr/bin/env python3
"""Run the gates in this directory and report one verdict.

Two modes, because the two halves answer different questions and cost two
orders of magnitude apart.

The **checkers** ask whether this tree is admissible. That is the question a
working session asks, and it is answered in about half a minute -- almost all
of it `check_tdd_flow.py`, whose fixture is a real cargo crate.

The **negative controls** ask whether a checker can be seen to fail at all.
That question is about the gate rather than the tree, so its answer only
changes when a gate changes -- and answering it is expensive by construction:
each control mutates the thing under test once per declared defect and re-runs
the checker. `check_plugin_negative_control.py` plants five isolated defects.

So the controls are opt-in, and the thing that makes the split safe is that the
default mode is not allowed to sound like the full one. A run that skipped
every discrimination proof must never print the string a full run prints,
because that string is what gets pasted as evidence. It names the controls it
did not run instead, so the gap is in the output rather than in someone's
memory of which flag they used.

Order matters twice over, and the pairing below is what preserves it. Each
checker runs immediately before the control that mutates the file it reads. And
the manifest pair runs first, so `check_plugin.py` -- which reads the same
`plugin.json` -- lands *after* the control that mutates it, and a restore that
silently failed is caught by the next checker rather than by the next session.
The engine-split pair sits after the coverage pair for the same reason: both
read `epic.py`, and the coverage control mutates it. Dropping the controls
cannot disturb any of this, because it removes the second element of each pair
and never reorders the first.

Measurement scripts are not run in either mode. They hit the tracker over the
network and produce evidence for a decision, not a pass/fail;
`check_coverage_rule.py` names the snapshot it needs when it is missing.
"""
import pathlib
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent

FLAG = "--with-negative-controls"

# (checker, its negative control). `None` where a gate has no control: the two
# probes stage their own throwaway trees, and the flow gates carry their
# controls inside themselves -- each row already a declared mutation.
SUITE = [
    # `check_manifests_cli.py` and its control stood first here until
    # 2026-08-21. They asked `claude plugin validate` about `plugin.json` and
    # `marketplace.json`; the plugin was deleted that day and both manifests
    # went with it, so the gate had no subject left. Deleted rather than
    # skipped -- a gate that passes because its subject is absent is the exact
    # false green this directory exists to prevent.
    #
    # First, and early on purpose. Its control is the only one here that
    # mutates product scripts the rest of the suite depends on -- `workitem.py`,
    # `leg.py` and `e2e.py` -- and `check_plugin.py`, `check_engine_split.py`
    # and `check_tdd_flow.py` all read at least one of them. Running it here
    # means a restore that silently failed is caught by three later checkers
    # rather than by the next session.
    ("check_next_command.py", "check_next_command_negative_control.py"),
    ("check_plugin.py", "check_plugin_negative_control.py"),
    ("check_type_registry.py", None),
    ("check_milestone.py", None),
    ("check_type_migration.py", None),
    ("check_coverage_rule.py", "check_coverage_rule_negative_control.py"),
    ("check_engine_split.py", "check_engine_split_negative_control.py"),
    ("check_change_schema.py", "check_change_schema_negative_control.py"),
    # Reads the epic snapshot and calls `order_children` as a pure function;
    # nothing is spawned and nothing is written, so it costs about as much as
    # the probes and sits with them rather than with the flow gates below.
    #
    # It carried no control until 2026-08-27, when its two headline corpus rows
    # stopped being declared counts and became relational -- and a relational
    # row states an absence, which an instrument that reads nothing satisfies
    # too. Its control mutates `epic.py`, the same product script
    # `check_coverage_rule_negative_control.py` mutates, and restores by
    # captured bytes with an sha256 check.
    ("check_epic_order.py", "check_epic_order_negative_control.py"),
    ("probe_offtree_root.py", None),
    ("probe_local_verbs.py", None),
    # The META-doc validator. Exempt from the ordering rule above for the same
    # reason the flow gates are: its fixture is a `tempfile` git repository of
    # its own, and the two tables it mutates are mutated in-process on a module
    # it loaded, so it writes nothing any gate here reads.
    #
    # What it asserts is that each rule fires on its own defect and on nothing
    # else -- the detector, not the tree.
    ("check_meta_flow.py", None),
    # The tree, which is the pair above's answer applied to this checkout: no
    # findings, over a population that is checked for having been read at all.
    # It is a separate gate rather than four more rows on `check_meta_flow.py`
    # because the two reds mean opposite things -- one says the detector broke
    # and nothing it reported can be trusted, the other says the detector is
    # fine and a document rotted -- and the suite's output is a filename.
    #
    # It is also the one ratchet here that could not be written until its
    # subject was already green: `meta.py check` reported 103 findings when the
    # validator landed, and a gate that lands red is a gate that joins the pile
    # of pre-existing failures nobody reads. So it went in one commit later,
    # with no tolerated-failure list, and the tolerated set stays empty.
    #
    # Exempt from the ordering rule for a different reason than the flow gates:
    # its control does mutate real tracked files, but they are two `apps/cube`
    # documents that nothing else in this suite reads or writes.
    ("check_meta_clean.py", "check_meta_clean_negative_control.py"),
    # The META-doc run's allowlist. Exempt from the ordering rule for the same
    # reason the flow gate below is: every case builds its own `tempfile` git
    # repository with its own `aw.toml`, plants one violation in it, and spawns
    # `metadoc.py` against that -- nothing in this checkout is read or written,
    # so it can neither be disturbed by a control above nor leave residue below.
    #
    # It carries no separate negative control, and that is not an exemption:
    # the gate *is* the control. Each refusal case starts from a fixture that
    # produces zero findings and plants exactly one defect, so a rule that
    # stopped firing shows up as that case reporting `[]`, and a rule that fires
    # on everything shows up as the baseline case reporting it. Since the
    # allowlist widened from `docs/product/` alone to four paths on 2026-08-27
    # the admissions carry their own cases too -- three of those four paths were
    # previously the *near misses* the refusal case planted writes into, and an
    # inverted control nobody rewrote would go on passing while measuring the
    # opposite of its own name.
    ("check_metadoc_scope.py", None),
    # Exempt from the ordering rule above, and last because they are the
    # slowest. They mutate nothing in this checkout: each fixture is a
    # `tempfile` tree with its own `aw.toml` and its own git repository, so it
    # can neither be disturbed by a control above nor leave residue for one
    # below.
    #
    # `check_ec_flow.py`, `check_td_flow.py` and `check_cb_flow.py` stood here
    # until the ladder they measured was deleted. A gate outlives the thing it
    # gates only as a source of false confidence, so they went with it -- but
    # not before what they covered had somewhere else to be measured, which is
    # what the row below is. (`check_review_flow.py` stood beside it until the
    # semantic review left the ladder on 2026-08-26, and went the same way.)
    #
    # The `e2e -> unit -> logic` ladder, and the slowest of the lot: its
    # fixture is a real cargo crate, so every row that runs `test` pays a
    # compile.
    ("check_maint_flow.py", "check_maint_flow_negative_control.py"),
    ("check_tdd_flow.py", None),
]

# The list above is hand-maintained and nothing else reads it, so a gate that
# was written and never registered runs in the session that wrote it and never
# again -- which is worse than never writing it, because the directory listing
# reads as coverage that exists. The listing is therefore the assertion, the
# same way `check_plugin.py` asserts the skills on disk are exactly the ones
# under test: every `check_*` and `probe_*` here must appear in exactly one slot
# above. `measure_*` is excluded by name because it is deliberately not run --
# those hit the network and produce evidence, not a verdict.
on_disk = {p.name for pattern in ("check_*.py", "probe_*.py")
           for p in HERE.glob(pattern)}
registered = {name for pair in SUITE for name in pair if name}
if on_disk != registered:
    raise SystemExit(
        "error: the suite and the directory disagree about what this is\n"
        f"  on disk but never run:  {sorted(on_disk - registered) or 'none'}\n"
        f"  run but not on disk:    {sorted(registered - on_disk) or 'none'}")

unknown = [a for a in sys.argv[1:] if a != FLAG]
if unknown:
    raise SystemExit(f"usage: {pathlib.Path(sys.argv[0]).name} [{FLAG}]\n"
                     f"error: unrecognized argument(s): {' '.join(unknown)}")

controls = FLAG in sys.argv[1:]
gates = [name
         for checker, control in SUITE
         for name in ((checker, control) if controls else (checker,))
         if name]
skipped = [] if controls else [c for _checker, c in SUITE if c]

results = []
for name in gates:
    r = subprocess.run([sys.executable, str(HERE / name)], capture_output=True, text=True)
    verdict = "GREEN" if r.returncode == 0 else f"RED (exit {r.returncode})"
    results.append((name, r.returncode, r.stdout))
    print(f"{verdict:16s} {name}")

failed = [name for name, code, _ in results if code != 0]
if failed:
    print("\n" + "=" * 70)
    for name, code, out in results:
        if code == 0:
            continue
        print(f"\n--- {name} ---")
        for line in out.splitlines():
            if line.startswith("FAIL") or line.startswith("=>") or "RED" in line:
                print("  " + line)

if skipped:
    # Named rather than counted. A count reads as bookkeeping; the names say
    # which specific claims -- "this gate can be seen to fail" -- went
    # unmeasured, and they are the only claims this mode cannot make.
    print("\nnot run (no gate here was proven able to fail):")
    for name in skipped:
        print(f"  {name}")

if failed:
    print(f"\n=> RED: {', '.join(failed)}")
elif skipped:
    print(f"\n=> CHECKERS GREEN -- negative controls not run; `{FLAG}` for the full suite")
else:
    print("\n=> ALL GREEN")

sys.exit(1 if failed else 0)
