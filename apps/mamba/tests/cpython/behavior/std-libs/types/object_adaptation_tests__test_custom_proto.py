# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "object_adaptation_tests__test_custom_proto"
# subject = "cpython.test_types.ObjectAdaptationTests.test_custom_proto"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_types
_suite = unittest.defaultTestLoader.loadTestsFromName("ObjectAdaptationTests.test_custom_proto", test_types)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ObjectAdaptationTests.test_custom_proto did not pass"
print("ObjectAdaptationTests::test_custom_proto: ok")
