# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "libc"
# dimension = "behavior"
# case = "lib_test__test_qsort_ucd0847d"
# subject = "cpython.test_libc.LibTest.test_qsort"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_libc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_libc
_suite = unittest.defaultTestLoader.loadTestsFromName("LibTest.test_qsort", test_libc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LibTest.test_qsort did not pass"
print("LibTest::test_qsort: ok")
