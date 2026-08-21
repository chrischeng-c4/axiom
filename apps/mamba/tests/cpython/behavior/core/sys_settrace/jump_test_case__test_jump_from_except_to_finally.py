# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "sys_settrace"
# dimension = "behavior"
# case = "jump_test_case__test_jump_from_except_to_finally"
# subject = "cpython.test_sys_settrace.JumpTestCase.test_jump_from_except_to_finally"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys_settrace.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_sys_settrace
_suite = unittest.defaultTestLoader.loadTestsFromName("JumpTestCase.test_jump_from_except_to_finally", test_sys_settrace)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython JumpTestCase.test_jump_from_except_to_finally did not pass"
print("JumpTestCase::test_jump_from_except_to_finally: ok")
