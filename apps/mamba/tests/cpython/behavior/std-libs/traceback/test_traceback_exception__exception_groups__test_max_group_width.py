# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "test_traceback_exception__exception_groups__test_max_group_width"
# subject = "cpython.test_traceback.TestTracebackException_ExceptionGroups.test_max_group_width"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_traceback
_suite = unittest.defaultTestLoader.loadTestsFromName("TestTracebackException_ExceptionGroups.test_max_group_width", test_traceback)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestTracebackException_ExceptionGroups.test_max_group_width did not pass"
print("TestTracebackException_ExceptionGroups::test_max_group_width: ok")
