# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "misc"
# dimension = "behavior"
# case = "builtin_static_types_tests__test_tp_mro_is_set"
# subject = "cpython.test_misc.BuiltinStaticTypesTests.test_tp_mro_is_set"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_misc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_misc
_suite = unittest.defaultTestLoader.loadTestsFromName("BuiltinStaticTypesTests.test_tp_mro_is_set", test_misc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BuiltinStaticTypesTests.test_tp_mro_is_set did not pass"
print("BuiltinStaticTypesTests::test_tp_mro_is_set: ok")
