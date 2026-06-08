# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_main_handling"
# dimension = "behavior"
# case = "forkserver_cmd_line_test__test_basic_script"
# subject = "cpython.test_multiprocessing_main_handling.ForkServerCmdLineTest.test_basic_script"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_multiprocessing_main_handling.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Multiprocessing forkserver can run a normal script when the host permits it."""

import io
import os
import tempfile
import unittest
from test import test_multiprocessing_main_handling as tmh


original_cwd = os.getcwd()
stream = io.StringIO()

with tempfile.TemporaryDirectory(prefix="mp-", dir="/tmp") as tmpdir:
    old_env = {name: os.environ.get(name) for name in ("TMPDIR", "TEMP", "TMP")}
    os.environ["TMPDIR"] = tmpdir
    os.environ["TEMP"] = tmpdir
    os.environ["TMP"] = tmpdir
    os.chdir(tmpdir)
    try:
        suite = unittest.TestSuite([tmh.ForkServerCmdLineTest("test_basic_script")])
        result = unittest.TextTestRunner(stream=stream, verbosity=0).run(suite)
    finally:
        tmh.tearDownModule()
        os.chdir(original_cwd)
        for name, value in old_env.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value

output = stream.getvalue()
environment_blocked = (
    "forkserver.py" in output
    and "PermissionError: [Errno 1] Operation not permitted" in output
)

assert result.testsRun == 1, result.testsRun
assert not result.errors, output

if result.failures and environment_blocked:
    print("forkserver_cmd_line_test__test_basic_script skipped: forkserver bind denied")
else:
    assert not result.failures, output
    if result.skipped:
        print("forkserver_cmd_line_test__test_basic_script skipped:", result.skipped[0][1])
    else:
        print("forkserver_cmd_line_test__test_basic_script OK")
