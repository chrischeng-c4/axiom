# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tuple"
# dimension = "behavior"
# case = "c_a_p_i_test__test_tuple_pack_uc940635"
# subject = "cpython.test_tuple.CAPITest.test_tuple_pack"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_tuple.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_tuple
_suite = unittest.defaultTestLoader.loadTestsFromName("CAPITest.test_tuple_pack", test_tuple)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CAPITest.test_tuple_pack did not pass"
print("CAPITest::test_tuple_pack: ok")
