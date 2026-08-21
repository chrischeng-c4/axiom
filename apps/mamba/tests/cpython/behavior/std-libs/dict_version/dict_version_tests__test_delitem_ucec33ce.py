# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dict_version"
# dimension = "behavior"
# case = "dict_version_tests__test_delitem_ucec33ce"
# subject = "cpython.test_dict_version.DictVersionTests.test_delitem"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict_version.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_dict_version
_suite = unittest.defaultTestLoader.loadTestsFromName("DictVersionTests.test_delitem", test_dict_version)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DictVersionTests.test_delitem did not pass"
print("DictVersionTests::test_delitem: ok")
