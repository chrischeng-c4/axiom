#!/usr/bin/env python3.12
"""Shared oracle/SUT execution core for the CPython conformance harness tools.

ONE canonical fixture runner so the per-tool copies can no longer drift. Before
this, keep_status / gen_goldens / gate_status / coverage_matrix / golden_capstone /
verify_cpython_oracle each reimplemented the spawn loop — and they HAD already
drifted: gate_status & golden_capstone leaked CPython test.support TESTFN files
(no isolated CWD), and verify_cpython_oracle dropped PYTHONBREAKPOINT=0
(breakpoint() hang risk). Every leak/bug fix only patched one of N copies, so the
meters silently disagreed.

`run_fixture()` spawns argv with, uniformly:
  - PYTHONBREAKPOINT=0               — no interactive breakpoint hang
  - an isolated PER-CALL scratch CWD + TMPDIR/TEMP/TMP pointed at it — so
    test.support TESTFN (@test_<pid>_tmp), stray .db/.pyc, and tempfile output
    land in throwaway scratch, never the repo tree; per-fixture so two fixtures
    can't collide on a fixed-name temp file
  - text OR bytes capture            — goldens freeze raw bytes, meters compare text
  - one exception policy             — timeout/OSError -> (None, empty, empty)

The scratch root is removed at process exit; cleanup chmod-walks first so
os_helper read-only-dir fixtures don't block removal.
"""
from __future__ import annotations

import atexit
import os
import shutil
import subprocess
import tempfile

# ---------------------------------------------------------------------------
# Verdict vocabulary — ONE canonical set of classification verdicts so the
# per-tool meters (gate_status / keep_status / coverage_matrix / …) can no
# longer drift on the bare-string spelling. These are the EXACT string values
# the tools have always emitted; they are used as Counter keys and as the
# return value of every classify().
#
#   PASS         oracle exit 0, mamba exit 0, axis-specific pass holds
#   MAMBA_RED    mamba non-zero / crash / timeout — not yet implemented
#   DIVERGE      both exit 0 but behavior differs — wrong behavior
#   ORACLE_SKIP  oracle itself can't run here — leaves the graded denominator
PASS = "PASS"
MAMBA_RED = "MAMBA_RED"
DIVERGE = "DIVERGE"
ORACLE_SKIP = "ORACLE_SKIP"


def compute_pass_rate(counter):
    """Return ``(pass, graded, rate)`` for a verdict ``Counter``.

    ``graded = total - ORACLE_SKIP`` (oracle-skips leave the denominator) and
    ``rate = 100 * PASS / graded`` when ``graded > 0`` else ``0.0``. This is the
    EXACT arithmetic the meters open-coded; centralising it kills the copies."""
    passed = counter[PASS]
    graded = sum(counter.values()) - counter[ORACLE_SKIP]
    rate = 100 * passed / graded if graded else 0.0
    return passed, graded, rate


_SCRATCH_ROOT = tempfile.mkdtemp(prefix="mamba_harness_")


def _rmtree_force(path: str) -> None:
    for root, dirs, _files in os.walk(path):
        for d in dirs:
            try:
                os.chmod(os.path.join(root, d), 0o700)
            except OSError:
                pass
    shutil.rmtree(path, ignore_errors=True)


atexit.register(_rmtree_force, _SCRATCH_ROOT)


def run_fixture(argv, timeout, *, text: bool = True):
    """Spawn ``argv`` in an isolated scratch CWD with PYTHONBREAKPOINT=0.

    Returns ``(returncode | None, stdout, stderr)``. stdout/stderr are ``str``
    when ``text=True`` else ``bytes``. Any exception (timeout / OSError) yields
    ``(None, empty, empty)`` so callers branch on ``rc is None``.
    """
    empty = "" if text else b""
    cwd = tempfile.mkdtemp(prefix="run_", dir=_SCRATCH_ROOT)
    env = dict(os.environ, PYTHONBREAKPOINT="0", TMPDIR=cwd, TEMP=cwd, TMP=cwd)
    try:
        r = subprocess.run(
            argv, capture_output=True, text=text, timeout=timeout, env=env, cwd=cwd,
        )
        return r.returncode, r.stdout, r.stderr
    except Exception:  # noqa: BLE001  (timeout / OSError / decode -> not gradable)
        return None, empty, empty
    finally:
        _rmtree_force(cwd)
