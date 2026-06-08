# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "line_cache_tests__test_invalid_names_uc71f8ae"
# subject = "cpython.test_linecache.LineCacheTests.test_invalid_names"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_linecache
_suite = unittest.defaultTestLoader.loadTestsFromName("LineCacheTests.test_invalid_names", test_linecache)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LineCacheTests.test_invalid_names did not pass"
print("LineCacheTests::test_invalid_names: ok")
