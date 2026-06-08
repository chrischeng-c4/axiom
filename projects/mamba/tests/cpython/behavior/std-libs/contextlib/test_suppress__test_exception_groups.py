# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "test_suppress__test_exception_groups"
# subject = "cpython.test_contextlib.TestSuppress.test_exception_groups"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_contextlib
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSuppress.test_exception_groups", test_contextlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSuppress.test_exception_groups did not pass"
print("TestSuppress::test_exception_groups: ok")
