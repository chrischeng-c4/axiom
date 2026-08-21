# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "test_int_enum_class__test_shadowed_attr"
# subject = "cpython.test_enum.TestIntEnumClass.test_shadowed_attr"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_enum
_suite = unittest.defaultTestLoader.loadTestsFromName("TestIntEnumClass.test_shadowed_attr", test_enum)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestIntEnumClass.test_shadowed_attr did not pass"
print("TestIntEnumClass::test_shadowed_attr: ok")
