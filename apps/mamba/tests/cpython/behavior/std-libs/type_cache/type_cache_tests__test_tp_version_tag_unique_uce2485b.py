# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_cache"
# dimension = "behavior"
# case = "type_cache_tests__test_tp_version_tag_unique_uce2485b"
# subject = "cpython.test_type_cache.TypeCacheTests.test_tp_version_tag_unique"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_cache.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_type_cache
_suite = unittest.defaultTestLoader.loadTestsFromName("TypeCacheTests.test_tp_version_tag_unique", test_type_cache)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TypeCacheTests.test_tp_version_tag_unique did not pass"
print("TypeCacheTests::test_tp_version_tag_unique: ok")
