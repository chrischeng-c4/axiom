# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pytree"
# dimension = "behavior"
# case = "test_patterns__test_basic_patterns_uc13fa69"
# subject = "cpython.test_pytree.TestPatterns.test_basic_patterns"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lib2to3/test_pytree.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_lib2to3 import test_pytree
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPatterns.test_basic_patterns", test_pytree)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPatterns.test_basic_patterns did not pass"
print("TestPatterns::test_basic_patterns: ok")
