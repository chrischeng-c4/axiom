# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "behavior"
# case = "find_test_case__test_find_with_env_vars"
# subject = "cpython.test_gettext.FindTestCase.test_find_with_env_vars"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gettext.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_gettext
_suite = unittest.defaultTestLoader.loadTestsFromName("FindTestCase.test_find_with_env_vars", test_gettext)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FindTestCase.test_find_with_env_vars did not pass"
print("FindTestCase::test_find_with_env_vars: ok")
