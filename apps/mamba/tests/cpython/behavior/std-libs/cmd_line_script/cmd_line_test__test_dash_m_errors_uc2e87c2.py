# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd_line_script"
# dimension = "behavior"
# case = "cmd_line_test__test_dash_m_errors_uc2e87c2"
# subject = "cpython.test_cmd_line_script.CmdLineTest.test_dash_m_errors"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmd_line_script.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_cmd_line_script
_suite = unittest.defaultTestLoader.loadTestsFromName("CmdLineTest.test_dash_m_errors", test_cmd_line_script)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CmdLineTest.test_dash_m_errors did not pass"
print("CmdLineTest::test_dash_m_errors: ok")
