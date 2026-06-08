# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "errors"
# case = "bare_testcase_run_missing_method_raises"
# subject = "unittest.TestCase.run"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase.run: running a bare unittest.TestCase() with no selected test method raises AttributeError (no _testMethodName-resolvable method)"""
import unittest

bare = unittest.TestCase()
_raised = False
try:
    bare.run()
except AttributeError:
    _raised = True
assert _raised, "bare_testcase_run_missing_method_raises: expected AttributeError"
print("bare_testcase_run_missing_method_raises OK")
