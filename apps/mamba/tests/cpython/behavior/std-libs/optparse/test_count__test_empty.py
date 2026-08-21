# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "behavior"
# case = "test_count__test_empty"
# subject = "cpython.test_optparse.TestCount.test_empty"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_optparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_optparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCount.test_empty", test_optparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCount.test_empty did not pass"
print("TestCount::test_empty: ok")
