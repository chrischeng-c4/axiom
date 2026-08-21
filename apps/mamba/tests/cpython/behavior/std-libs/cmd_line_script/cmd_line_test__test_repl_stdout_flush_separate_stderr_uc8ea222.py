# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd_line_script"
# dimension = "behavior"
# case = "cmd_line_test__test_repl_stdout_flush_separate_stderr_uc8ea222"
# subject = "cpython.test_cmd_line_script.CmdLineTest.test_repl_stdout_flush_separate_stderr"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmd_line_script.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_cmd_line_script
_suite = unittest.defaultTestLoader.loadTestsFromName("CmdLineTest.test_repl_stdout_flush_separate_stderr", test_cmd_line_script)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CmdLineTest.test_repl_stdout_flush_separate_stderr did not pass"
print("CmdLineTest::test_repl_stdout_flush_separate_stderr: ok")
