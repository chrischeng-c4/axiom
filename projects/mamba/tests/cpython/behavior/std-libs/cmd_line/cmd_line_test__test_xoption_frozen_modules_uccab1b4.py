# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd_line"
# dimension = "behavior"
# case = "cmd_line_test__test_xoption_frozen_modules_uccab1b4"
# subject = "cpython.test_cmd_line.CmdLineTest.test_xoption_frozen_modules"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmd_line.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_cmd_line
_suite = unittest.defaultTestLoader.loadTestsFromName("CmdLineTest.test_xoption_frozen_modules", test_cmd_line)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CmdLineTest.test_xoption_frozen_modules did not pass"
print("CmdLineTest::test_xoption_frozen_modules: ok")
