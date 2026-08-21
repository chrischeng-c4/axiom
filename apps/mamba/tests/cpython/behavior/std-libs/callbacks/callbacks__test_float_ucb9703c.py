# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "callbacks"
# dimension = "behavior"
# case = "callbacks__test_float_ucb9703c"
# subject = "cpython.test_callbacks.Callbacks.test_float"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_callbacks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_callbacks
_suite = unittest.defaultTestLoader.loadTestsFromName("Callbacks.test_float", test_callbacks)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Callbacks.test_float did not pass"
print("Callbacks::test_float: ok")
