# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "p_o_s_i_x_process_test_case__test_exception_bad_args_0"
# subject = "cpython.test_subprocess.POSIXProcessTestCase.test_exception_bad_args_0"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_subprocess
_suite = unittest.defaultTestLoader.loadTestsFromName("POSIXProcessTestCase.test_exception_bad_args_0", test_subprocess)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython POSIXProcessTestCase.test_exception_bad_args_0 did not pass"
print("POSIXProcessTestCase::test_exception_bad_args_0: ok")
