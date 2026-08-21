# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "test_partial_c__test_keystr_replaces_value"
# subject = "cpython.test_functools.TestPartialC.test_keystr_replaces_value"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_functools
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPartialC.test_keystr_replaces_value", test_functools)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPartialC.test_keystr_replaces_value did not pass"
print("TestPartialC::test_keystr_replaces_value: ok")
