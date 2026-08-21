# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "behavior"
# case = "untokenize_test__test_backslash_continuation_uc7d4757"
# subject = "cpython.test_tokenize.UntokenizeTest.test_backslash_continuation"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tokenize.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tokenize
_suite = unittest.defaultTestLoader.loadTestsFromName("UntokenizeTest.test_backslash_continuation", test_tokenize)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UntokenizeTest.test_backslash_continuation did not pass"
print("UntokenizeTest::test_backslash_continuation: ok")
