# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_aliases"
# dimension = "behavior"
# case = "type_params_alias_value_test__test_raising_ucb5c425"
# subject = "cpython.test_type_aliases.TypeParamsAliasValueTest.test_raising"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_aliases.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_type_aliases
_suite = unittest.defaultTestLoader.loadTestsFromName("TypeParamsAliasValueTest.test_raising", test_type_aliases)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TypeParamsAliasValueTest.test_raising did not pass"
print("TypeParamsAliasValueTest::test_raising: ok")
