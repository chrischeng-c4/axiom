# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_with_ascii__test_one_insert_uce76b99"
# subject = "cpython.test_difflib.TestWithAscii.test_one_insert"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_difflib
_suite = unittest.defaultTestLoader.loadTestsFromName("TestWithAscii.test_one_insert", test_difflib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestWithAscii.test_one_insert did not pass"
print("TestWithAscii::test_one_insert: ok")
