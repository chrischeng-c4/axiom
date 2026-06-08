# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "set"
# dimension = "behavior"
# case = "test_basic_ops_mixed_string_bytes__test_repr"
# subject = "cpython.test_set.TestBasicOpsMixedStringBytes.test_repr"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_set.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_set
_suite = unittest.defaultTestLoader.loadTestsFromName("TestBasicOpsMixedStringBytes.test_repr", test_set)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestBasicOpsMixedStringBytes.test_repr did not pass"
print("TestBasicOpsMixedStringBytes::test_repr: ok")
