# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "basic_filter_test__test_empty_filter"
# subject = "cpython.test_logging.BasicFilterTest.test_empty_filter"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_logging
_suite = unittest.defaultTestLoader.loadTestsFromName("BasicFilterTest.test_empty_filter", test_logging)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BasicFilterTest.test_empty_filter did not pass"
print("BasicFilterTest::test_empty_filter: ok")
