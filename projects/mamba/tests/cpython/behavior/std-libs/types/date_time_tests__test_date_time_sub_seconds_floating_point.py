# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "date_time_tests__test_date_time_sub_seconds_floating_point"
# subject = "cpython.test_types.DateTimeTests.test_date_time_sub_seconds_floating_point"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_types
_suite = unittest.defaultTestLoader.loadTestsFromName("DateTimeTests.test_date_time_sub_seconds_floating_point", test_types)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DateTimeTests.test_date_time_sub_seconds_floating_point did not pass"
print("DateTimeTests::test_date_time_sub_seconds_floating_point: ok")
