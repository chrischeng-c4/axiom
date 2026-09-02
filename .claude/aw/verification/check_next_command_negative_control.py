#!/usr/bin/env python3
"""Prove `check_next_command.py` can be seen to fail, once per defect shape.

The gate it controls went in red -- six real dead ends across four scripts --
so its ability to fail was demonstrated before it ever passed. This exists for
what happens next: it is now green, and a green cross-check is indistinguishable
from a cross-check that stopped finding the commands it is supposed to compare.

Four mutations, each restoring one of the shapes the gate was written for, plus
two vacuity probes for the ways it could pass having read nothing.

A mutation is a *list* of edits rather than one, because the rows below are not
all single-file and a two-file mutation restored to one file is a mutant left in
the tree.

The shipped defect is deliberately split across the first and last rows. It was
one edit -- the engine's ladder tuple naming a deleted lifecycle -- and `leg.py`
now refuses that at import, before a command is printed at all. So the last row
measures the refusal, and the first measures what the refusal cannot see: the
same disagreement declared at the other end, in a phase script's own `PHASE`.

Isolation is asserted, not just the red. Each mutation must produce exactly its
own (emitter, refuser) pairs -- a gate that reported every printed command on
any defect at all would go red on all three below while being unable to tell a
renamed leg from a missing flag. The pairs are named by script rather than by
`file:line`, because a control keyed to line numbers goes red when someone adds
a comment, which is a red about the control.

Restore is verified by sha256 and wrapped in `finally`: these mutations are to
product scripts that every other flow gate in this suite imports, so a mutant
left behind by a crash would not sit quietly in a diff -- it would fail the rest
of the suite as though the plugin were broken.
"""
import contextlib
import hashlib
import io
import pathlib
import re
import subprocess
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import SCRIPTS, WI_TYPES_SCRIPT, load_script_module, pinned_interpreter  # noqa: E402

HERE = pathlib.Path(__file__).resolve().parent
CHECK = HERE / "check_next_command.py"
LAUNCH = pinned_interpreter()

IMPL = SCRIPTS / "impl.py"
E2E = SCRIPTS / "e2e.py"
METADOC = SCRIPTS / "metadoc.py"

# (label, [(target, anchor, mutant), ...], the (emitter, refuser) pairs it must
# produce, and a substring the output must carry -- `None` where the pairs are
# the whole assertion)
MUTATIONS = [
    # The defect as it actually shipped: the changeover deleted `ec -> td -> cb`
    # and left the engine's lifecycle vocabulary naming it, so all three phases
    # printed a `--leg` their own tracker script exits 2 on. Both edits, because
    # `LEG_ROOTS` carried the retired keys then and does not now -- see the
    # fourth row for what happens when only the first edit is applied.
    #
    # Six pairs, not the three the shipped defect produced. The fix bound
    # `leg.PHASES` to this same tuple instead of restating it, so the drift now
    # reaches `leg.py`'s printed commands as well -- it names `ec.py`, `td.py`
    # and `cb.py`, none of which exist. Those three extra reds are the alias
    # being load-bearing: before it, `leg.py` held its own correct copy and
    # stayed silent while the engine underneath it was wrong.
    # The shipped defect's reachable half. `workitem.LEGS` read `("ec", "td",
    # "cb")` one commit past the changeover, and `change.py` takes its `--leg`
    # choices from there, so all three phases printed a line it exits 2 on.
    #
    # Renaming the engine tuple no longer reproduces that -- `leg.py` refuses at
    # import first, which is the fourth row. What survives is the same drift from
    # the other end: a phase script's own `PHASE` literal disagreeing with the
    # ladder. Nothing asserts that at import, because `PHASE` is read at the call
    # site rather than declared to anything.
    #
    # Four pairs from one edit, because `PHASE` is both the `--leg` value and the
    # script name `phase_command` builds: the phase names a leg `change.py` does
    # not have, and three more lines name a script that is not on disk.
    ("phase-literal-drifts-from-the-ladder",
     [(E2E, 'PHASE = "e2e"', 'PHASE = "ec"')],
     {("e2e.py", "change.py"), ("e2e.py", "ec.py")},
     None),
    # The second one, from the other direction: a command spelled by hand rather
    # than built by `phase_command`, omitting a flag the receiver requires.
    # Re-pointed from `leg.py` to `unit.py` on 2026-08-26, and from `unit.py` to
    # `impl.py` on 2026-08-27 when `unit.py` and `logic.py` were deleted and
    # merged into it. Every phase's `test` still prints its own `commit` line
    # by calling `leg.phase_command`, so the shape under test is what happens
    # when a future edit spells that call out by hand instead and drops
    # `--project` in the process; `impl.py` is the target because it is the
    # phase this round's refactor touched most.
    ("printed-command-drops-required-flag",
     [(IMPL, "{leg.phase_command(PHASE, args.project, 'commit', args.wi)}",
       "{PHASE}.py commit {args.wi}")],
     {("impl.py", "impl.py")},
     None),
    # The branch neither real defect reached: a command naming a script that is
    # not there at all. Without this the `is_file` arm is never measured, and a
    # gate that silently classified an unresolvable name as prose would pass.
    # Since the 2026-09-02 move the printed line is `{AW_CLI} change lifecycle
    # ...`, so the typo lands on the group token and the gate resolves it to a
    # `chagne.py` that is not on disk -- the same red, reached through the
    # CLI-prefix arm instead of the bare-token one.
    ("printed-command-names-no-script",
     [(E2E, "change lifecycle {args.wi}", "chagne lifecycle {args.wi}")],
     {("e2e.py", "chagne.py")},
     None),
    # The shape the pre-move gate could not see at all: the two clean-run
    # commands are passed as a parameter into `report()`, and the renderer
    # only reaches them through the call-site hop added on 2026-09-02. This
    # plants the exact dead end that hop caught when it first ran -- `meta
    # check <project>` where `meta.py` has no positional and the scoped form
    # is `--path` -- so a regression that quietly drops the hop turns this row
    # from red back to prose and the control refuses it.
    ("parameter-hop-reaches-the-call-site",
     [(METADOC, "meta check --path {project}", "meta check {project}")],
     {("metadoc.py", "meta.py")},
     None),
    # The first row's first edit on its own. `leg.py` now asserts at import that
    # its two phase-keyed tables name the phases the engine declares, so this
    # never reaches a printed command: nothing is compared, and the red is the
    # import refusing.
    #
    # It is here rather than left implicit because the assertion is what makes
    # the first row need two edits, and a control that did not measure it would
    # not notice the assertion being deleted -- the first row would keep passing,
    # since it repairs the table it would otherwise trip.
    ("phase-table-drift-refused-at-import",
     [(WI_TYPES_SCRIPT, '    "behavior": ("e2e", "impl"),',
       '    "behavior": ("ec", "td", "cb"),')],
     set(),
     "a phase table names something the ladder does not"),
]

PAIR = re.compile(r"^FAIL (?P<emitter>[a-z0-9_]+\.py):\d+: "
                  r"(?:prints `(?P<target>[a-z0-9_]+\.py)|names `(?P<missing>[a-z0-9_.]+))")


def gate():
    """Exit code and output. Both streams: a mutation can now stop the gate at
    import, and that refusal is written to stderr."""
    r = subprocess.run([*LAUNCH, str(CHECK)], capture_output=True, text=True)
    return r.returncode, r.stdout + r.stderr


def pairs_of(out):
    """`{(emitter, refuser): why}` -- the pair is the assertion, `why` is for
    reading. The gate has two red shapes and they are not the same finding: a
    parser that rejected the argv, and a name that is not a script at all."""
    found = {}
    for line in out.splitlines():
        m = PAIR.match(line)
        if m:
            found[(m["emitter"], m["target"] or m["missing"])] = (
                "refuses it" if m["target"] else "is not a script here")
    return found


baseline_code, baseline_out = gate()
print(f"== baseline == {baseline_out.strip().splitlines()[-1]} (exit={baseline_code})")

failures = []
for label, edits, expected, expect_text in MUTATIONS:
    originals = {target: target.read_bytes() for target, _, _ in edits}
    before = {t_: hashlib.sha256(b).hexdigest() for t_, b in originals.items()}

    counts = {target: originals[target].decode("utf-8").count(anchor)
              for target, anchor, _ in edits}
    if any(n != 1 for n in counts.values()):
        failures.append(f"{label}: anchor counts {counts}, declared 1x each")
        print(f"\n== {label} == ANCHOR COUNT WRONG "
              f"({ {t_.name: n for t_, n in counts.items()} })")
        continue

    try:
        for target, anchor, mutant in edits:
            text = originals[target].decode("utf-8")
            target.write_text(text.replace(anchor, mutant), encoding="utf-8")
        code, out = gate()
    finally:
        for target, blob in originals.items():
            target.write_bytes(blob)
    after = {t_: hashlib.sha256(t_.read_bytes()).hexdigest() for t_ in originals}

    why = pairs_of(out)
    found = set(why)
    isolated = found == expected
    said = expect_text is None or expect_text in out
    restored = before == after
    edited = ", ".join(f"{t_.name}: {a[:34]!r}" for t_, a, _ in edits)
    print(f"\n== {label} == exit={code}  ({edited})")
    for pair in sorted(found):
        print(f"   RED  {pair[0]} prints a command {pair[1]} {why[pair]}")
    if not isolated:
        print(f"   isolation: UNEXPECTED -- missing={sorted(expected - found)} "
              f"extra={sorted(found - expected)}")
    else:
        print("   isolation: exactly the declared pairs"
              if expected else "   isolation: no command compared, as declared")
    if expect_text is not None:
        print(f"   said:      {'yes' if said else 'NO'} -- {expect_text!r}")
    print(f"   restore:   {'byte-identical' if restored else 'FAILED'} "
          f"({', '.join(sorted(h[:12] for h in before.values()))})")

    if not isolated or not said or not restored or code == 0:
        failures.append(label)

# The two ways this gate could pass having compared nothing. Neither is
# reachable by editing a script -- both are about the population itself -- so
# they are driven in-process against a scripts directory built for the purpose.
print("\n== vacuity ==")
mod = load_script_module(CHECK, "gate_under_control")
for label, body in [
    ("no script prints the marker", None),
    # The print sits inside a function so that importing the fixture does not
    # run it. The gate executes every module it analyses -- that is how it reads
    # their constants -- and a fixture printing at import would put a stray line
    # in this control's own output.
    ("every printed line is prose",
     'def emit():\n    print("next.command: fix the rows above")\n'),
]:
    with tempfile.TemporaryDirectory() as tmp:
        empty = pathlib.Path(tmp)
        if body:
            (empty / "fake.py").write_text(body, encoding="utf-8")
        mod.SCRIPTS = empty
        noise = io.StringIO()
        try:
            with contextlib.redirect_stdout(noise):
                code = mod.main()
            refused = code != 0
            why = next((ln for ln in noise.getvalue().splitlines()
                        if ln.startswith("FAIL")), f"exit {code}")
        except SystemExit as exit_:
            refused = bool(exit_.code)
            why = str(exit_)
    print(f"   {label}: {'REFUSED -- ' + why if refused else 'PASSED, which is the defect'}")
    if not refused:
        failures.append(f"vacuity: {label}")
mod.SCRIPTS = SCRIPTS

restored_code, restored_out = gate()
print(f"\n== restored == {restored_out.strip().splitlines()[-1]} (exit={restored_code})")

ok = not failures and restored_code == 0 and baseline_code == 0
print("=> " + ("GREEN" if ok else f"RED ({failures or 'gate not green after restore'})"))
sys.exit(0 if ok else 1)
