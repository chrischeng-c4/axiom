# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "frame"
# dimension = "behavior"
# case = "test_c_a_p_i__test_getvar_uc3774c7"
# subject = "cpython.test_frame.TestCAPI.test_getvar"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_frame.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_frame
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCAPI.test_getvar", test_frame)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCAPI.test_getvar did not pass"
print("TestCAPI::test_getvar: ok")
