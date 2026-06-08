# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "test_special__test_tuple_subclass_with_auto_1"
# subject = "cpython.test_enum.TestSpecial.test_tuple_subclass_with_auto_1"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_enum
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSpecial.test_tuple_subclass_with_auto_1", test_enum)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSpecial.test_tuple_subclass_with_auto_1 did not pass"
print("TestSpecial::test_tuple_subclass_with_auto_1: ok")
