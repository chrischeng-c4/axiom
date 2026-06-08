# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "process_test_case__test_failed_child_execute_fd_leak"
# subject = "cpython.test_subprocess.ProcessTestCase.test_failed_child_execute_fd_leak"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_subprocess
_suite = unittest.defaultTestLoader.loadTestsFromName("ProcessTestCase.test_failed_child_execute_fd_leak", test_subprocess)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ProcessTestCase.test_failed_child_execute_fd_leak did not pass"
print("ProcessTestCase::test_failed_child_execute_fd_leak: ok")
