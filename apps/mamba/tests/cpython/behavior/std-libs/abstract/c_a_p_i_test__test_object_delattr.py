# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abstract"
# dimension = "behavior"
# case = "c_a_p_i_test__test_object_delattr"
# subject = "cpython.test_abstract.CAPITest.test_object_delattr"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_abstract.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_abstract
_suite = unittest.defaultTestLoader.loadTestsFromName("CAPITest.test_object_delattr", test_abstract)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CAPITest.test_object_delattr did not pass"
print("CAPITest::test_object_delattr: ok")
