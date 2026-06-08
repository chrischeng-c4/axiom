# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "set"
# dimension = "behavior"
# case = "test_set_c_a_p_i__test_frozenset_new"
# subject = "cpython.test_set.TestSetCAPI.test_frozenset_new"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_set.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_set
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSetCAPI.test_frozenset_new", test_set)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSetCAPI.test_frozenset_new did not pass"
print("TestSetCAPI::test_frozenset_new: ok")
