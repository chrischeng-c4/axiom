# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "eintr"
# dimension = "behavior"
# case = "eintr_tests__test_all"
# subject = "cpython.test_eintr.EINTRTests.test_all"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_eintr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""CPython EINTR suite handles interrupted syscalls transparently."""

import os
import subprocess
import sys
import tempfile
from test import support


script = support.findfile("_test_eintr.py")

with tempfile.TemporaryDirectory(prefix="mamba-cpython-eintr-") as tmpdir:
    env = dict(os.environ)
    env["TMPDIR"] = tmpdir
    env["TEMP"] = tmpdir
    env["TMP"] = tmpdir
    result = subprocess.run(
        [sys.executable, script],
        cwd=tmpdir,
        text=True,
        capture_output=True,
        timeout=90,
        env=env,
    )

combined = result.stdout + result.stderr

if result.returncode == 0:
    print("eintr_tests__test_all OK")
elif (
    "SocketEINTRTest.test_accept" in combined
    and "PermissionError: [Errno 1] Operation not permitted" in combined
):
    print("eintr_tests__test_all skipped: loopback bind denied by sandbox")
else:
    raise AssertionError(
        "CPython _test_eintr.py failed with exit "
        f"{result.returncode}\n{combined[-4000:]}"
    )
