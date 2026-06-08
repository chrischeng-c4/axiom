# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exception_group"
# dimension = "behavior"
# case = "nested_exception_group_subclass_split_test__test_split_exceptiongroup_subclass_no_derive_no_new_override_uc8931aa"
# subject = "cpython.test_exception_group.NestedExceptionGroupSubclassSplitTest.test_split_ExceptionGroup_subclass_no_derive_no_new_override"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exception_group.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_exception_group
_suite = unittest.defaultTestLoader.loadTestsFromName("NestedExceptionGroupSubclassSplitTest.test_split_ExceptionGroup_subclass_no_derive_no_new_override", test_exception_group)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NestedExceptionGroupSubclassSplitTest.test_split_ExceptionGroup_subclass_no_derive_no_new_override did not pass"
print("NestedExceptionGroupSubclassSplitTest::test_split_ExceptionGroup_subclass_no_derive_no_new_override: ok")
