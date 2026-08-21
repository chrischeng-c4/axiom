# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_cache"
# dimension = "behavior"
# case = "type_cache_with_specialization_tests__test_store_attr_specialization_user_type_uc4096a4"
# subject = "cpython.test_type_cache.TypeCacheWithSpecializationTests.test_store_attr_specialization_user_type"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_cache.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_type_cache
_suite = unittest.defaultTestLoader.loadTestsFromName("TypeCacheWithSpecializationTests.test_store_attr_specialization_user_type", test_type_cache)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TypeCacheWithSpecializationTests.test_store_attr_specialization_user_type did not pass"
print("TypeCacheWithSpecializationTests::test_store_attr_specialization_user_type: ok")
