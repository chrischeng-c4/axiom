# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tool"
# dimension = "behavior"
# case = "test_tool__test_ensure_ascii_default_uc416948"
# subject = "cpython.test_tool.TestTool.test_ensure_ascii_default"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_json/test_tool.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_json import test_tool
_suite = unittest.defaultTestLoader.loadTestsFromName("TestTool.test_ensure_ascii_default", test_tool)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestTool.test_ensure_ascii_default did not pass"
print("TestTool::test_ensure_ascii_default: ok")
