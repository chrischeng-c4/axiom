# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo_property"
# dimension = "behavior"
# case = "python_c_consistency_test__test_same_str_uc959439"
# subject = "cpython.test_zoneinfo_property.PythonCConsistencyTest.test_same_str"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo_property.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_zoneinfo import test_zoneinfo_property
_suite = unittest.defaultTestLoader.loadTestsFromName("PythonCConsistencyTest.test_same_str", test_zoneinfo_property)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PythonCConsistencyTest.test_same_str did not pass"
print("PythonCConsistencyTest::test_same_str: ok")
