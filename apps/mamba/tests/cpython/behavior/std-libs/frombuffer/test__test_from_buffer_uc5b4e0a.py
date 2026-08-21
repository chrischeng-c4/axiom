# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "frombuffer"
# dimension = "behavior"
# case = "test__test_from_buffer_uc5b4e0a"
# subject = "cpython.test_frombuffer.Test.test_from_buffer"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_frombuffer.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_frombuffer
_suite = unittest.defaultTestLoader.loadTestsFromName("Test.test_from_buffer", test_frombuffer)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test.test_from_buffer did not pass"
print("Test::test_from_buffer: ok")
