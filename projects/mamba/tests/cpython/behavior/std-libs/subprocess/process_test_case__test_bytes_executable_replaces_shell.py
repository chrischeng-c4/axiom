# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "process_test_case__test_bytes_executable_replaces_shell"
# subject = "cpython.test_subprocess.ProcessTestCase.test_bytes_executable_replaces_shell"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_subprocess
_suite = unittest.defaultTestLoader.loadTestsFromName("ProcessTestCase.test_bytes_executable_replaces_shell", test_subprocess)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ProcessTestCase.test_bytes_executable_replaces_shell did not pass"
print("ProcessTestCase::test_bytes_executable_replaces_shell: ok")
