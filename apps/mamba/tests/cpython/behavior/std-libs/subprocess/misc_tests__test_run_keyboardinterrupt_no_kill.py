# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "misc_tests__test_run_keyboardinterrupt_no_kill"
# subject = "cpython.test_subprocess.MiscTests.test_run_keyboardinterrupt_no_kill"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_subprocess
_suite = unittest.defaultTestLoader.loadTestsFromName("MiscTests.test_run_keyboardinterrupt_no_kill", test_subprocess)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MiscTests.test_run_keyboardinterrupt_no_kill did not pass"
print("MiscTests::test_run_keyboardinterrupt_no_kill: ok")
