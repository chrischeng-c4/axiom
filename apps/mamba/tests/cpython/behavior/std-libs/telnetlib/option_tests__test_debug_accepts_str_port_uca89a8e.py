# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "telnetlib"
# dimension = "behavior"
# case = "option_tests__test_debug_accepts_str_port_uca89a8e"
# subject = "cpython.test_telnetlib.OptionTests.test_debug_accepts_str_port"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_telnetlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_telnetlib
_suite = unittest.defaultTestLoader.loadTestsFromName("OptionTests.test_debug_accepts_str_port", test_telnetlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython OptionTests.test_debug_accepts_str_port did not pass"
print("OptionTests::test_debug_accepts_str_port: ok")
