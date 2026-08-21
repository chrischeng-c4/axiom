# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getargs"
# dimension = "behavior"
# case = "string__test_case__test_z_hash_uc5a1cf3"
# subject = "cpython.test_getargs.String_TestCase.test_Z_hash"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_getargs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_getargs
_suite = unittest.defaultTestLoader.loadTestsFromName("String_TestCase.test_Z_hash", test_getargs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython String_TestCase.test_Z_hash did not pass"
print("String_TestCase::test_Z_hash: ok")
