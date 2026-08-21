# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter"
# dimension = "behavior"
# case = "tkinter_regrtest_success"
# subject = "cpython.test_tkinter.regrtest"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tkinter/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""CPython tkinter package regrtest suite passes or skips cleanly."""

import os
import subprocess
import sys
import tempfile


with tempfile.TemporaryDirectory(prefix="mamba-cpython-regrtest-") as tmpdir:
    env = dict(os.environ)
    env["TMPDIR"] = tmpdir
    env["TEMP"] = tmpdir
    env["TMP"] = tmpdir
    result = subprocess.run(
        [sys.executable, "-m", "test", "-q", "test_tkinter"],
        cwd=tmpdir,
        text=True,
        capture_output=True,
        timeout=180,
        env=env,
    )

combined = result.stdout + result.stderr
assert result.returncode == 0, combined[-4000:]
assert "Result: SUCCESS" in combined, combined[-4000:]
print("tkinter_regrtest_success OK")
