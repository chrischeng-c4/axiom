# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getargs"
# dimension = "behavior"
# case = "unsigned__test_case__test_b_uc51f6bc"
# subject = "cpython.test_getargs.Unsigned_TestCase.test_B"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_getargs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_getargs
_suite = unittest.defaultTestLoader.loadTestsFromName("Unsigned_TestCase.test_B", test_getargs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Unsigned_TestCase.test_B did not pass"
print("Unsigned_TestCase::test_B: ok")
