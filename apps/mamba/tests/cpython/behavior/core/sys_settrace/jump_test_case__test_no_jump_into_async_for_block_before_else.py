# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "sys_settrace"
# dimension = "behavior"
# case = "jump_test_case__test_no_jump_into_async_for_block_before_else"
# subject = "cpython.test_sys_settrace.JumpTestCase.test_no_jump_into_async_for_block_before_else"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys_settrace.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_sys_settrace
_suite = unittest.defaultTestLoader.loadTestsFromName("JumpTestCase.test_no_jump_into_async_for_block_before_else", test_sys_settrace)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython JumpTestCase.test_no_jump_into_async_for_block_before_else did not pass"
print("JumpTestCase::test_no_jump_into_async_for_block_before_else: ok")
