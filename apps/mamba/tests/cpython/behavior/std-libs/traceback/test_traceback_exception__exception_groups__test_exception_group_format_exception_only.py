# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "test_traceback_exception__exception_groups__test_exception_group_format_exception_only"
# subject = "cpython.test_traceback.TestTracebackException_ExceptionGroups.test_exception_group_format_exception_only"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_traceback
_suite = unittest.defaultTestLoader.loadTestsFromName("TestTracebackException_ExceptionGroups.test_exception_group_format_exception_only", test_traceback)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestTracebackException_ExceptionGroups.test_exception_group_format_exception_only did not pass"
print("TestTracebackException_ExceptionGroups::test_exception_group_format_exception_only: ok")
