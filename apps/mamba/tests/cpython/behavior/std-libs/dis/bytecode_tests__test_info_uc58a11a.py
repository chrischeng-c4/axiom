# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "behavior"
# case = "bytecode_tests__test_info_uc58a11a"
# subject = "cpython.test_dis.BytecodeTests.test_info"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dis.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_dis
_suite = unittest.defaultTestLoader.loadTestsFromName("BytecodeTests.test_info", test_dis)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BytecodeTests.test_info did not pass"
print("BytecodeTests::test_info: ok")
