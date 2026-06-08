# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "numeric_owner_test__test_extract_without_numeric_owner"
# subject = "cpython.test_tarfile.NumericOwnerTest.test_extract_without_numeric_owner"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tarfile
_suite = unittest.defaultTestLoader.loadTestsFromName("NumericOwnerTest.test_extract_without_numeric_owner", test_tarfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NumericOwnerTest.test_extract_without_numeric_owner did not pass"
print("NumericOwnerTest::test_extract_without_numeric_owner: ok")
