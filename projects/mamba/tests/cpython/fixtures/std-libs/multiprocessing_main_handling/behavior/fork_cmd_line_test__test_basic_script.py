# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_main_handling"
# dimension = "behavior"
# case = "fork_cmd_line_test__test_basic_script"
# subject = "cpython.test_multiprocessing_main_handling.ForkCmdLineTest.test_basic_script"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_multiprocessing_main_handling.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Multiprocessing fork can run a normal script and expose __main__ functions."""

import io
import os
import tempfile
import unittest
from test import test_multiprocessing_main_handling as tmh


original_cwd = os.getcwd()
stream = io.StringIO()

with tempfile.TemporaryDirectory(prefix="mamba-cpython-mp-main-") as tmpdir:
    old_env = {name: os.environ.get(name) for name in ("TMPDIR", "TEMP", "TMP")}
    os.environ["TMPDIR"] = tmpdir
    os.environ["TEMP"] = tmpdir
    os.environ["TMP"] = tmpdir
    os.chdir(tmpdir)
    try:
        suite = unittest.TestSuite([tmh.ForkCmdLineTest("test_basic_script")])
        result = unittest.TextTestRunner(stream=stream, verbosity=0).run(suite)
    finally:
        tmh.tearDownModule()
        os.chdir(original_cwd)
        for name, value in old_env.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value

assert result.testsRun == 1, result.testsRun
assert not result.failures, stream.getvalue()
assert not result.errors, stream.getvalue()

if result.skipped:
    print("fork_cmd_line_test__test_basic_script skipped:", result.skipped[0][1])
else:
    print("fork_cmd_line_test__test_basic_script OK")
