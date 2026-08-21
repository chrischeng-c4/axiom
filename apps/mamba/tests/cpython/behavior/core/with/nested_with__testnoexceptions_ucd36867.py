# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "with"
# dimension = "behavior"
# case = "nested_with__testnoexceptions_ucd36867"
# subject = "cpython.test_with.NestedWith.testNoExceptions"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_with.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_with
_suite = unittest.defaultTestLoader.loadTestsFromName("NestedWith.testNoExceptions", test_with)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NestedWith.testNoExceptions did not pass"
print("NestedWith::testNoExceptions: ok")
