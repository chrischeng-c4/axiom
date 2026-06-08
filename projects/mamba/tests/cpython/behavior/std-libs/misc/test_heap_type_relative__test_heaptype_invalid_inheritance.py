# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "misc"
# dimension = "behavior"
# case = "test_heap_type_relative__test_heaptype_invalid_inheritance"
# subject = "cpython.test_misc.TestHeapTypeRelative.test_heaptype_invalid_inheritance"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_misc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_misc
_suite = unittest.defaultTestLoader.loadTestsFromName("TestHeapTypeRelative.test_heaptype_invalid_inheritance", test_misc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestHeapTypeRelative.test_heaptype_invalid_inheritance did not pass"
print("TestHeapTypeRelative::test_heaptype_invalid_inheritance: ok")
