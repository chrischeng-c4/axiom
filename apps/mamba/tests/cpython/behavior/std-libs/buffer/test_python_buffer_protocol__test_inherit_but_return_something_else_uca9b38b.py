# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "buffer"
# dimension = "behavior"
# case = "test_python_buffer_protocol__test_inherit_but_return_something_else_uca9b38b"
# subject = "cpython.test_buffer.TestPythonBufferProtocol.test_inherit_but_return_something_else"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_buffer.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_buffer
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPythonBufferProtocol.test_inherit_but_return_something_else", test_buffer)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPythonBufferProtocol.test_inherit_but_return_something_else did not pass"
print("TestPythonBufferProtocol::test_inherit_but_return_something_else: ok")
