# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test__sizeof___uc945aa9"
# subject = "cpython.test_struct.StructTest.test__sizeof__"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_struct
_suite = unittest.defaultTestLoader.loadTestsFromName("StructTest.test__sizeof__", test_struct)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StructTest.test__sizeof__ did not pass"
print("StructTest::test__sizeof__: ok")
