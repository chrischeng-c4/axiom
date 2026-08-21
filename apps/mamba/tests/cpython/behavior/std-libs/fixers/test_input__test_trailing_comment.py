# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fixers"
# dimension = "behavior"
# case = "test_input__test_trailing_comment"
# subject = "cpython.test_fixers.Test_input.test_trailing_comment"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lib2to3/test_fixers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_lib2to3 import test_fixers
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_input.test_trailing_comment", test_fixers)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_input.test_trailing_comment did not pass"
print("Test_input::test_trailing_comment: ok")
