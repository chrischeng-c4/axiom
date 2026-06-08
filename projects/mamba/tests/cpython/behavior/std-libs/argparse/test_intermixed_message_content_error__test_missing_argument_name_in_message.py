# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "test_intermixed_message_content_error__test_missing_argument_name_in_message"
# subject = "cpython.test_argparse.TestIntermixedMessageContentError.test_missing_argument_name_in_message"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_argparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_argparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestIntermixedMessageContentError.test_missing_argument_name_in_message", test_argparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestIntermixedMessageContentError.test_missing_argument_name_in_message did not pass"
print("TestIntermixedMessageContentError::test_missing_argument_name_in_message: ok")
