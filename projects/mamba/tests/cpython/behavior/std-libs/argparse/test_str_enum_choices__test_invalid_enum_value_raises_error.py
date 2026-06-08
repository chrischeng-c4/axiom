# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "test_str_enum_choices__test_invalid_enum_value_raises_error"
# subject = "cpython.test_argparse.TestStrEnumChoices.test_invalid_enum_value_raises_error"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_argparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_argparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestStrEnumChoices.test_invalid_enum_value_raises_error", test_argparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestStrEnumChoices.test_invalid_enum_value_raises_error did not pass"
print("TestStrEnumChoices::test_invalid_enum_value_raises_error: ok")
