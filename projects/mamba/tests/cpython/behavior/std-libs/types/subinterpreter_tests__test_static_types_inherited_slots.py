# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "subinterpreter_tests__test_static_types_inherited_slots"
# subject = "cpython.test_types.SubinterpreterTests.test_static_types_inherited_slots"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_types
_suite = unittest.defaultTestLoader.loadTestsFromName("SubinterpreterTests.test_static_types_inherited_slots", test_types)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SubinterpreterTests.test_static_types_inherited_slots did not pass"
print("SubinterpreterTests::test_static_types_inherited_slots: ok")
