# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "finalize_test_case__test_finalize_uc0f3150"
# subject = "cpython.test_weakref.FinalizeTestCase.test_finalize"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_weakref
_suite = unittest.defaultTestLoader.loadTestsFromName("FinalizeTestCase.test_finalize", test_weakref)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FinalizeTestCase.test_finalize did not pass"
print("FinalizeTestCase::test_finalize: ok")
