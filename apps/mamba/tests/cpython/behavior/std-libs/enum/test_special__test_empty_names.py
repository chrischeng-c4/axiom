# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "test_special__test_empty_names"
# subject = "cpython.test_enum.TestSpecial.test_empty_names"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_enum
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSpecial.test_empty_names", test_enum)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSpecial.test_empty_names did not pass"
print("TestSpecial::test_empty_names: ok")
