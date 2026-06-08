# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "sys_settrace"
# dimension = "behavior"
# case = "trace_test_case__test_class_creation_with_docstrings"
# subject = "cpython.test_sys_settrace.TraceTestCase.test_class_creation_with_docstrings"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys_settrace.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_sys_settrace
_suite = unittest.defaultTestLoader.loadTestsFromName("TraceTestCase.test_class_creation_with_docstrings", test_sys_settrace)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TraceTestCase.test_class_creation_with_docstrings did not pass"
print("TraceTestCase::test_class_creation_with_docstrings: ok")
