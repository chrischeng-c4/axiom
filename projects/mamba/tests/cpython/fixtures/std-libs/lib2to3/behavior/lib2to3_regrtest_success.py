# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3"
# dimension = "behavior"
# case = "lib2to3_regrtest_success"
# subject = "cpython.test_lib2to3.regrtest"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lib2to3/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""CPython lib2to3 package regrtest suite passes under regrtest."""

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
        [sys.executable, "-m", "test", "-q", "test_lib2to3"],
        cwd=tmpdir,
        text=True,
        capture_output=True,
        timeout=180,
        env=env,
    )

combined = result.stdout + result.stderr
assert result.returncode == 0, combined[-4000:]
assert "Result: SUCCESS" in combined, combined[-4000:]
print("lib2to3_regrtest_success OK")
