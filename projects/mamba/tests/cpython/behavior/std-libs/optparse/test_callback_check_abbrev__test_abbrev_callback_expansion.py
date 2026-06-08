# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "behavior"
# case = "test_callback_check_abbrev__test_abbrev_callback_expansion"
# subject = "cpython.test_optparse.TestCallbackCheckAbbrev.test_abbrev_callback_expansion"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_optparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_optparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCallbackCheckAbbrev.test_abbrev_callback_expansion", test_optparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCallbackCheckAbbrev.test_abbrev_callback_expansion did not pass"
print("TestCallbackCheckAbbrev::test_abbrev_callback_expansion: ok")
