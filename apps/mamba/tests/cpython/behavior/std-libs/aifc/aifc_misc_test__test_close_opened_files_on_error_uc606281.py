# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "behavior"
# case = "aifc_misc_test__test_close_opened_files_on_error_uc606281"
# subject = "cpython.test_aifc.AifcMiscTest.test_close_opened_files_on_error"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_aifc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_aifc
_suite = unittest.defaultTestLoader.loadTestsFromName("AifcMiscTest.test_close_opened_files_on_error", test_aifc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AifcMiscTest.test_close_opened_files_on_error did not pass"
print("AifcMiscTest::test_close_opened_files_on_error: ok")
