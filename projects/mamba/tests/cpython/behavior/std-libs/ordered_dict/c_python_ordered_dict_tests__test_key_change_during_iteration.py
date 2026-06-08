# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ordered_dict"
# dimension = "behavior"
# case = "c_python_ordered_dict_tests__test_key_change_during_iteration"
# subject = "cpython.test_ordered_dict.CPythonOrderedDictTests.test_key_change_during_iteration"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ordered_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ordered_dict
_suite = unittest.defaultTestLoader.loadTestsFromName("CPythonOrderedDictTests.test_key_change_during_iteration", test_ordered_dict)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CPythonOrderedDictTests.test_key_change_during_iteration did not pass"
print("CPythonOrderedDictTests::test_key_change_during_iteration: ok")
