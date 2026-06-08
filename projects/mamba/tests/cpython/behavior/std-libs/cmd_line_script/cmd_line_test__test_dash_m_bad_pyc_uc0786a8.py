# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd_line_script"
# dimension = "behavior"
# case = "cmd_line_test__test_dash_m_bad_pyc_uc0786a8"
# subject = "cpython.test_cmd_line_script.CmdLineTest.test_dash_m_bad_pyc"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmd_line_script.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_cmd_line_script
_suite = unittest.defaultTestLoader.loadTestsFromName("CmdLineTest.test_dash_m_bad_pyc", test_cmd_line_script)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CmdLineTest.test_dash_m_bad_pyc did not pass"
print("CmdLineTest::test_dash_m_bad_pyc: ok")
