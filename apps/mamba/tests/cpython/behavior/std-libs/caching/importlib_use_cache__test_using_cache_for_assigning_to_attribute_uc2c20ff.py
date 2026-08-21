# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "caching"
# dimension = "behavior"
# case = "importlib_use_cache__test_using_cache_for_assigning_to_attribute_uc2c20ff"
# subject = "cpython.test_caching.ImportlibUseCache.test_using_cache_for_assigning_to_attribute"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/import_/test_caching.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib.import_ import test_caching
_suite = unittest.defaultTestLoader.loadTestsFromName("ImportlibUseCache.test_using_cache_for_assigning_to_attribute", test_caching)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ImportlibUseCache.test_using_cache_for_assigning_to_attribute did not pass"
print("ImportlibUseCache::test_using_cache_for_assigning_to_attribute: ok")
