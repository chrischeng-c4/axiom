# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_nn_code_uce55b78"
# subject = "cpython.test_struct.StructTest.test_nN_code"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_struct
_suite = unittest.defaultTestLoader.loadTestsFromName("StructTest.test_nN_code", test_struct)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StructTest.test_nN_code did not pass"
print("StructTest::test_nN_code: ok")
