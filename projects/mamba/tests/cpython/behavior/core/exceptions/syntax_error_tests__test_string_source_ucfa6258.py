# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "syntax_error_tests__test_string_source_ucfa6258"
# subject = "cpython.test_exceptions.SyntaxErrorTests.test_string_source"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_exceptions
_suite = unittest.defaultTestLoader.loadTestsFromName("SyntaxErrorTests.test_string_source", test_exceptions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SyntaxErrorTests.test_string_source did not pass"
print("SyntaxErrorTests::test_string_source: ok")
