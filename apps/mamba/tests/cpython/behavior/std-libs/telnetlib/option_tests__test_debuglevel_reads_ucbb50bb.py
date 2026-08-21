# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "telnetlib"
# dimension = "behavior"
# case = "option_tests__test_debuglevel_reads_ucbb50bb"
# subject = "cpython.test_telnetlib.OptionTests.test_debuglevel_reads"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_telnetlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_telnetlib
_suite = unittest.defaultTestLoader.loadTestsFromName("OptionTests.test_debuglevel_reads", test_telnetlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython OptionTests.test_debuglevel_reads did not pass"
print("OptionTests::test_debuglevel_reads: ok")
