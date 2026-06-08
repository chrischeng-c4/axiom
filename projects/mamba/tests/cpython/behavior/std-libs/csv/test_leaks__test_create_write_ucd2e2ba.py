# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_leaks__test_create_write_ucd2e2ba"
# subject = "cpython.test_csv.TestLeaks.test_create_write"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_csv
_suite = unittest.defaultTestLoader.loadTestsFromName("TestLeaks.test_create_write", test_csv)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestLeaks.test_create_write did not pass"
print("TestLeaks::test_create_write: ok")
