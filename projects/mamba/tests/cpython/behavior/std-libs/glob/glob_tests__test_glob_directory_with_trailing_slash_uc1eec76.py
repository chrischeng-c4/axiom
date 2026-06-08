# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "glob_tests__test_glob_directory_with_trailing_slash_uc1eec76"
# subject = "cpython.test_glob.GlobTests.test_glob_directory_with_trailing_slash"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_glob
_suite = unittest.defaultTestLoader.loadTestsFromName("GlobTests.test_glob_directory_with_trailing_slash", test_glob)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GlobTests.test_glob_directory_with_trailing_slash did not pass"
print("GlobTests::test_glob_directory_with_trailing_slash: ok")
