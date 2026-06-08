# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "behavior"
# case = "a_i_f_c_low_level_test__test_read_wrong_marks_ucc5e1d5"
# subject = "cpython.test_aifc.AIFCLowLevelTest.test_read_wrong_marks"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_aifc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_aifc
_suite = unittest.defaultTestLoader.loadTestsFromName("AIFCLowLevelTest.test_read_wrong_marks", test_aifc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AIFCLowLevelTest.test_read_wrong_marks did not pass"
print("AIFCLowLevelTest::test_read_wrong_marks: ok")
