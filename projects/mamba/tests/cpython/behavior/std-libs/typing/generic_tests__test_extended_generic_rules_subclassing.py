# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "generic_tests__test_extended_generic_rules_subclassing"
# subject = "cpython.test_typing.GenericTests.test_extended_generic_rules_subclassing"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_typing
_suite = unittest.defaultTestLoader.loadTestsFromName("GenericTests.test_extended_generic_rules_subclassing", test_typing)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GenericTests.test_extended_generic_rules_subclassing did not pass"
print("GenericTests::test_extended_generic_rules_subclassing: ok")
