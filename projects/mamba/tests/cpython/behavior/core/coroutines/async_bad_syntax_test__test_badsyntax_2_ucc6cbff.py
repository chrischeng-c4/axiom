# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "coroutines"
# dimension = "behavior"
# case = "async_bad_syntax_test__test_badsyntax_2_ucc6cbff"
# subject = "cpython.test_coroutines.AsyncBadSyntaxTest.test_badsyntax_2"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_coroutines.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_coroutines
_suite = unittest.defaultTestLoader.loadTestsFromName("AsyncBadSyntaxTest.test_badsyntax_2", test_coroutines)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AsyncBadSyntaxTest.test_badsyntax_2 did not pass"
print("AsyncBadSyntaxTest::test_badsyntax_2: ok")
