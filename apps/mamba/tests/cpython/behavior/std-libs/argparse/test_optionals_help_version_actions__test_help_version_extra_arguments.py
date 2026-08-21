# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "test_optionals_help_version_actions__test_help_version_extra_arguments"
# subject = "cpython.test_argparse.TestOptionalsHelpVersionActions.test_help_version_extra_arguments"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_argparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_argparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestOptionalsHelpVersionActions.test_help_version_extra_arguments", test_argparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestOptionalsHelpVersionActions.test_help_version_extra_arguments did not pass"
print("TestOptionalsHelpVersionActions::test_help_version_extra_arguments: ok")
