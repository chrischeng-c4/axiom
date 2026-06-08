# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "pid_tests__test_waitstatus_to_exitcode_kill"
# subject = "cpython.test_os.PidTests.test_waitstatus_to_exitcode_kill"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_os
_suite = unittest.defaultTestLoader.loadTestsFromName("PidTests.test_waitstatus_to_exitcode_kill", test_os)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PidTests.test_waitstatus_to_exitcode_kill did not pass"
print("PidTests::test_waitstatus_to_exitcode_kill: ok")
