# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "behavior"
# case = "test_callback_meddle_args__test_callback_meddle_args_separator"
# subject = "cpython.test_optparse.TestCallbackMeddleArgs.test_callback_meddle_args_separator"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_optparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_optparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCallbackMeddleArgs.test_callback_meddle_args_separator", test_optparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCallbackMeddleArgs.test_callback_meddle_args_separator did not pass"
print("TestCallbackMeddleArgs::test_callback_meddle_args_separator: ok")
