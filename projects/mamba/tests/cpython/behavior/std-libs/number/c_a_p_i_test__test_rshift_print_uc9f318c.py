# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "number"
# dimension = "behavior"
# case = "c_a_p_i_test__test_rshift_print_uc9f318c"
# subject = "cpython.test_number.CAPITest.test_rshift_print"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_number.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_number
_suite = unittest.defaultTestLoader.loadTestsFromName("CAPITest.test_rshift_print", test_number)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CAPITest.test_rshift_print did not pass"
print("CAPITest::test_rshift_print: ok")
