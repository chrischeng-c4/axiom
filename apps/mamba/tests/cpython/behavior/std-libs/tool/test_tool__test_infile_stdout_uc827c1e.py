# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tool"
# dimension = "behavior"
# case = "test_tool__test_infile_stdout_uc827c1e"
# subject = "cpython.test_tool.TestTool.test_infile_stdout"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_json/test_tool.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_json import test_tool
_suite = unittest.defaultTestLoader.loadTestsFromName("TestTool.test_infile_stdout", test_tool)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestTool.test_infile_stdout did not pass"
print("TestTool::test_infile_stdout: ok")
