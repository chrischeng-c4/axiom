# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "refcounts"
# dimension = "behavior"
# case = "another_leak__test_callback_py_object_none_return_uce9df09"
# subject = "cpython.test_refcounts.AnotherLeak.test_callback_py_object_none_return"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_refcounts.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_refcounts
_suite = unittest.defaultTestLoader.loadTestsFromName("AnotherLeak.test_callback_py_object_none_return", test_refcounts)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AnotherLeak.test_callback_py_object_none_return did not pass"
print("AnotherLeak::test_callback_py_object_none_return: ok")
