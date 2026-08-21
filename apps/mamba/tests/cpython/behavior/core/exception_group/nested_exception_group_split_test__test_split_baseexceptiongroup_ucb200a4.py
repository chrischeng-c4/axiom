# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exception_group"
# dimension = "behavior"
# case = "nested_exception_group_split_test__test_split_baseexceptiongroup_ucb200a4"
# subject = "cpython.test_exception_group.NestedExceptionGroupSplitTest.test_split_BaseExceptionGroup"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exception_group.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_exception_group
_suite = unittest.defaultTestLoader.loadTestsFromName("NestedExceptionGroupSplitTest.test_split_BaseExceptionGroup", test_exception_group)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NestedExceptionGroupSplitTest.test_split_BaseExceptionGroup did not pass"
print("NestedExceptionGroupSplitTest::test_split_BaseExceptionGroup: ok")
