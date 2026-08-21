# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "callbacks"
# dimension = "behavior"
# case = "callbacks__test_ushort_uc01faab"
# subject = "cpython.test_callbacks.Callbacks.test_ushort"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_callbacks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_callbacks
_suite = unittest.defaultTestLoader.loadTestsFromName("Callbacks.test_ushort", test_callbacks)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Callbacks.test_ushort did not pass"
print("Callbacks::test_ushort: ok")
