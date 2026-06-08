# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_dumb"
# dimension = "behavior"
# case = "dumb_d_b_m_test_case__test_readonly_files_ucf66511"
# subject = "cpython.test_dbm_dumb.DumbDBMTestCase.test_readonly_files"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_dumb.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_dbm_dumb
_suite = unittest.defaultTestLoader.loadTestsFromName("DumbDBMTestCase.test_readonly_files", test_dbm_dumb)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DumbDBMTestCase.test_readonly_files did not pass"
print("DumbDBMTestCase::test_readonly_files: ok")
