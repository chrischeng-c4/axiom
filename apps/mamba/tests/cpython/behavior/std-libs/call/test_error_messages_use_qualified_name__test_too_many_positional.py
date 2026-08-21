# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "call"
# dimension = "behavior"
# case = "test_error_messages_use_qualified_name__test_too_many_positional"
# subject = "cpython.test_call.TestErrorMessagesUseQualifiedName.test_too_many_positional"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_call.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_call
_suite = unittest.defaultTestLoader.loadTestsFromName("TestErrorMessagesUseQualifiedName.test_too_many_positional", test_call)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestErrorMessagesUseQualifiedName.test_too_many_positional did not pass"
print("TestErrorMessagesUseQualifiedName::test_too_many_positional: ok")
