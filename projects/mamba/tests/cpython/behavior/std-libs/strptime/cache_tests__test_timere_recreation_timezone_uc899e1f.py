# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strptime"
# dimension = "behavior"
# case = "cache_tests__test_timere_recreation_timezone_uc899e1f"
# subject = "cpython.test_strptime.CacheTests.test_TimeRE_recreation_timezone"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strptime.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_strptime
_suite = unittest.defaultTestLoader.loadTestsFromName("CacheTests.test_TimeRE_recreation_timezone", test_strptime)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CacheTests.test_TimeRE_recreation_timezone did not pass"
print("CacheTests::test_TimeRE_recreation_timezone: ok")
