# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cfuncs"
# dimension = "behavior"
# case = "c_functions__test_longlong_plus_uc7b6fe0"
# subject = "cpython.test_cfuncs.CFunctions.test_longlong_plus"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_cfuncs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_cfuncs
_suite = unittest.defaultTestLoader.loadTestsFromName("CFunctions.test_longlong_plus", test_cfuncs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CFunctions.test_longlong_plus did not pass"
print("CFunctions::test_longlong_plus: ok")
