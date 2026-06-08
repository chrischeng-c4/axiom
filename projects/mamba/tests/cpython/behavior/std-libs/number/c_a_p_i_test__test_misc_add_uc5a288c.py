# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "number"
# dimension = "behavior"
# case = "c_a_p_i_test__test_misc_add_uc5a288c"
# subject = "cpython.test_number.CAPITest.test_misc_add"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_number.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_number
_suite = unittest.defaultTestLoader.loadTestsFromName("CAPITest.test_misc_add", test_number)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CAPITest.test_misc_add did not pass"
print("CAPITest::test_misc_add: ok")
