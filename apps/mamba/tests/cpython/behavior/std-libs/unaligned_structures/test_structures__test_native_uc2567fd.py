# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unaligned_structures"
# dimension = "behavior"
# case = "test_structures__test_native_uc2567fd"
# subject = "cpython.test_unaligned_structures.TestStructures.test_native"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_unaligned_structures.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_unaligned_structures
_suite = unittest.defaultTestLoader.loadTestsFromName("TestStructures.test_native", test_unaligned_structures)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestStructures.test_native did not pass"
print("TestStructures::test_native: ok")
