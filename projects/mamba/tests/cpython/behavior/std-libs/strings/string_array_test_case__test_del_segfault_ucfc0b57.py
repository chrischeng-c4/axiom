# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strings"
# dimension = "behavior"
# case = "string_array_test_case__test_del_segfault_ucfc0b57"
# subject = "cpython.test_strings.StringArrayTestCase.test_del_segfault"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_strings.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_strings
_suite = unittest.defaultTestLoader.loadTestsFromName("StringArrayTestCase.test_del_segfault", test_strings)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StringArrayTestCase.test_del_segfault did not pass"
print("StringArrayTestCase::test_del_segfault: ok")
