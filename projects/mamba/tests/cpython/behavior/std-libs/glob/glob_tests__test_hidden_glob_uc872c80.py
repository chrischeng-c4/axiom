# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "glob_tests__test_hidden_glob_uc872c80"
# subject = "cpython.test_glob.GlobTests.test_hidden_glob"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_glob
_suite = unittest.defaultTestLoader.loadTestsFromName("GlobTests.test_hidden_glob", test_glob)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GlobTests.test_hidden_glob did not pass"
print("GlobTests::test_hidden_glob: ok")
