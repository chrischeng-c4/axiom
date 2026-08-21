# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_lambdef_uc5d17a0"
# subject = "cpython.test_grammar.GrammarTests.test_lambdef"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_grammar
_suite = unittest.defaultTestLoader.loadTestsFromName("GrammarTests.test_lambdef", test_grammar)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GrammarTests.test_lambdef did not pass"
print("GrammarTests::test_lambdef: ok")
