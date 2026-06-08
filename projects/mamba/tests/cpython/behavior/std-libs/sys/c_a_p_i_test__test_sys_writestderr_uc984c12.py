# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "c_a_p_i_test__test_sys_writestderr_uc984c12"
# subject = "cpython.test_sys.CAPITest.test_sys_writestderr"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_sys
_suite = unittest.defaultTestLoader.loadTestsFromName("CAPITest.test_sys_writestderr", test_sys)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CAPITest.test_sys_writestderr did not pass"
print("CAPITest::test_sys_writestderr: ok")
