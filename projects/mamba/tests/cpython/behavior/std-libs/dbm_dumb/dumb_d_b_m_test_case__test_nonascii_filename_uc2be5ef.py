# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_dumb"
# dimension = "behavior"
# case = "dumb_d_b_m_test_case__test_nonascii_filename_uc2be5ef"
# subject = "cpython.test_dbm_dumb.DumbDBMTestCase.test_nonascii_filename"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_dumb.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_dbm_dumb
_suite = unittest.defaultTestLoader.loadTestsFromName("DumbDBMTestCase.test_nonascii_filename", test_dbm_dumb)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DumbDBMTestCase.test_nonascii_filename did not pass"
print("DumbDBMTestCase::test_nonascii_filename: ok")
