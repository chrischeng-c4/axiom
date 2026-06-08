# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "types_tests__test_dunder_get_signature"
# subject = "cpython.test_types.TypesTests.test_dunder_get_signature"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_types
_suite = unittest.defaultTestLoader.loadTestsFromName("TypesTests.test_dunder_get_signature", test_types)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TypesTests.test_dunder_get_signature did not pass"
print("TypesTests::test_dunder_get_signature: ok")
