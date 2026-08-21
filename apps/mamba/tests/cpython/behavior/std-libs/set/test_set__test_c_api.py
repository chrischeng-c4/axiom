# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "set"
# dimension = "behavior"
# case = "test_set__test_c_api"
# subject = "cpython.test_set.TestSet.test_c_api"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_set.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_set
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSet.test_c_api", test_set)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSet.test_c_api did not pass"
print("TestSet::test_c_api: ok")
