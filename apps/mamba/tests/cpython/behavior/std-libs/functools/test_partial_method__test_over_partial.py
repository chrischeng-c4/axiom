# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "test_partial_method__test_over_partial"
# subject = "cpython.test_functools.TestPartialMethod.test_over_partial"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_functools
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPartialMethod.test_over_partial", test_functools)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPartialMethod.test_over_partial did not pass"
print("TestPartialMethod::test_over_partial: ok")
