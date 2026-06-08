# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "line_cache_invalidation_tests__test_checkcache_with_no_parameter_uc0ea3a2"
# subject = "cpython.test_linecache.LineCacheInvalidationTests.test_checkcache_with_no_parameter"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_linecache
_suite = unittest.defaultTestLoader.loadTestsFromName("LineCacheInvalidationTests.test_checkcache_with_no_parameter", test_linecache)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LineCacheInvalidationTests.test_checkcache_with_no_parameter did not pass"
print("LineCacheInvalidationTests::test_checkcache_with_no_parameter: ok")
