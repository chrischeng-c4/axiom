# #1558 — TESTFN droppings: scratch-CWD for sweep.py and direct runs

Status: OPEN (p2). Design for implementation. First TD of the harness/ context.

## Mechanism

`test.support.os_helper.TESTFN` = CWD-relative `@mamba_test_<pid>`
(support_mod.rs:~479 — CPython-faithful, must stay relative). The cargo
conformance runner already isolates via `ScratchCwd::enter()`
(conformance/mod.rs — regrtest-style throwaway CWD, restores+removes on every
exit path). sweep.py and human/agent direct `mamba run` invocations have NO
such isolation → droppings accumulate at whatever CWD (71 collected at repo
root on 2026-07-13; tracked trio already `git rm`'d in 90252aa09 — that AC is
done).

## Invariant

TESTFN stays CWD-relative (fixtures assert relative-path I/O). Isolation moves
the CWD, never the filename. Default `mamba run` behavior unchanged (real
programs must run in the user's CWD).

## Fix direction

1. sweep.py: run each fixture subprocess with `cwd=<mkdtemp()>`, cleaned per
   fixture (mirror ScratchCwd semantics).
2. `mamba run`: opt-in env `MAMBA_SCRATCH_CWD=1` wraps execution in the
   existing ScratchCwd (reuse conformance/mod.rs's guard — do not reimplement).
3. Doc note in the fixture-authoring conventions pointing agents at the env.

## Verification contract

Run a TESTFN-heavy fixture (e.g. the shutil/tempfile family) via sweep.py and
via `MAMBA_SCRATCH_CWD=1 mamba run` — `find . -maxdepth 1 -name '@*'` diff
before/after is empty; without the env the behavior is unchanged; ScratchCwd
unit/conformance paths still pass.
