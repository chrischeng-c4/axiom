# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "process_test_case__test_bufsize_equal_one_binary_mode"
# subject = "cpython.test_subprocess.ProcessTestCase.test_bufsize_equal_one_binary_mode"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_subprocess
_suite = unittest.defaultTestLoader.loadTestsFromName("ProcessTestCase.test_bufsize_equal_one_binary_mode", test_subprocess)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ProcessTestCase.test_bufsize_equal_one_binary_mode did not pass"
print("ProcessTestCase::test_bufsize_equal_one_binary_mode: ok")
