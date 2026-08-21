# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "behavior"
# case = "instruction_tests__test_nested_uc268c61"
# subject = "cpython.test_dis.InstructionTests.test_nested"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dis.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_dis
_suite = unittest.defaultTestLoader.loadTestsFromName("InstructionTests.test_nested", test_dis)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython InstructionTests.test_nested did not pass"
print("InstructionTests::test_nested: ok")
