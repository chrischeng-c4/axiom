# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "list"
# dimension = "behavior"
# case = "c_a_p_i_test__test_list_insert_ucd7f030"
# subject = "cpython.test_list.CAPITest.test_list_insert"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_list.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_list
_suite = unittest.defaultTestLoader.loadTestsFromName("CAPITest.test_list_insert", test_list)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CAPITest.test_list_insert did not pass"
print("CAPITest::test_list_insert: ok")
