# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_assert_failures_ucdb0497"
# subject = "cpython.test_grammar.GrammarTests.test_assert_failures"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_grammar
_suite = unittest.defaultTestLoader.loadTestsFromName("GrammarTests.test_assert_failures", test_grammar)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GrammarTests.test_assert_failures did not pass"
print("GrammarTests::test_assert_failures: ok")
