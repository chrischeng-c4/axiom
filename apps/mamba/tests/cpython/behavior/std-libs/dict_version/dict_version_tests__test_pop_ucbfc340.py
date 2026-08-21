# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dict_version"
# dimension = "behavior"
# case = "dict_version_tests__test_pop_ucbfc340"
# subject = "cpython.test_dict_version.DictVersionTests.test_pop"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict_version.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_dict_version
_suite = unittest.defaultTestLoader.loadTestsFromName("DictVersionTests.test_pop", test_dict_version)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DictVersionTests.test_pop did not pass"
print("DictVersionTests::test_pop: ok")
