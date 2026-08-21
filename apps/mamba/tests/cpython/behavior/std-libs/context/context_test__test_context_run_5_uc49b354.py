# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "context"
# dimension = "behavior"
# case = "context_test__test_context_run_5_uc49b354"
# subject = "cpython.test_context.ContextTest.test_context_run_5"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_context.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_context
_suite = unittest.defaultTestLoader.loadTestsFromName("ContextTest.test_context_run_5", test_context)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ContextTest.test_context_run_5 did not pass"
print("ContextTest::test_context_run_5: ok")
