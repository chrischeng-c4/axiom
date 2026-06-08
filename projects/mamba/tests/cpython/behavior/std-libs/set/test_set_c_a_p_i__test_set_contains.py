# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "set"
# dimension = "behavior"
# case = "test_set_c_a_p_i__test_set_contains"
# subject = "cpython.test_set.TestSetCAPI.test_set_contains"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_set.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_set
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSetCAPI.test_set_contains", test_set)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSetCAPI.test_set_contains did not pass"
print("TestSetCAPI::test_set_contains: ok")
