# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "sys_settrace"
# dimension = "behavior"
# case = "jump_test_case__test_jump_with_null_on_stack_load_attr"
# subject = "cpython.test_sys_settrace.JumpTestCase.test_jump_with_null_on_stack_load_attr"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys_settrace.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_sys_settrace
_suite = unittest.defaultTestLoader.loadTestsFromName("JumpTestCase.test_jump_with_null_on_stack_load_attr", test_sys_settrace)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython JumpTestCase.test_jump_with_null_on_stack_load_attr did not pass"
print("JumpTestCase::test_jump_with_null_on_stack_load_attr: ok")
