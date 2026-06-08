# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "float"
# dimension = "behavior"
# case = "c_a_p_i_float_test__test_pack_unpack_roundtrip_uc8910a5"
# subject = "cpython.test_float.CAPIFloatTest.test_pack_unpack_roundtrip"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_float.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_float
_suite = unittest.defaultTestLoader.loadTestsFromName("CAPIFloatTest.test_pack_unpack_roundtrip", test_float)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CAPIFloatTest.test_pack_unpack_roundtrip did not pass"
print("CAPIFloatTest::test_pack_unpack_roundtrip: ok")
