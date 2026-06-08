# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_aliases"
# dimension = "behavior"
# case = "type_alias_pickle_test__test_pickling_local_ucd99b35"
# subject = "cpython.test_type_aliases.TypeAliasPickleTest.test_pickling_local"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_aliases.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_type_aliases
_suite = unittest.defaultTestLoader.loadTestsFromName("TypeAliasPickleTest.test_pickling_local", test_type_aliases)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TypeAliasPickleTest.test_pickling_local did not pass"
print("TypeAliasPickleTest::test_pickling_local: ok")
