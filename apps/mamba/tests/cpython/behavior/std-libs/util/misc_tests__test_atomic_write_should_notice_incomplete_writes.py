# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "util"
# dimension = "behavior"
# case = "misc_tests__test_atomic_write_should_notice_incomplete_writes"
# subject = "cpython.test_util.MiscTests.test_atomic_write_should_notice_incomplete_writes"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_util.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_util
_suite = unittest.defaultTestLoader.loadTestsFromName("MiscTests.test_atomic_write_should_notice_incomplete_writes", test_util)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MiscTests.test_atomic_write_should_notice_incomplete_writes did not pass"
print("MiscTests::test_atomic_write_should_notice_incomplete_writes: ok")
