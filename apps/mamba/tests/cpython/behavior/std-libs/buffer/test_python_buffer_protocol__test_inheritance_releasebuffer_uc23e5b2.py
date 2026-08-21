# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "buffer"
# dimension = "behavior"
# case = "test_python_buffer_protocol__test_inheritance_releasebuffer_uc23e5b2"
# subject = "cpython.test_buffer.TestPythonBufferProtocol.test_inheritance_releasebuffer"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_buffer.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_buffer
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPythonBufferProtocol.test_inheritance_releasebuffer", test_buffer)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPythonBufferProtocol.test_inheritance_releasebuffer did not pass"
print("TestPythonBufferProtocol::test_inheritance_releasebuffer: ok")
