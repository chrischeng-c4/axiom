# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "behavior"
# case = "test_standard__test_short_option_joined_and_separator"
# subject = "cpython.test_optparse.TestStandard.test_short_option_joined_and_separator"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_optparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_optparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestStandard.test_short_option_joined_and_separator", test_optparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestStandard.test_short_option_joined_and_separator did not pass"
print("TestStandard::test_short_option_joined_and_separator: ok")
