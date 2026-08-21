# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structures"
# dimension = "behavior"
# case = "structure_test_case__test_array_in_struct"
# subject = "cpython.test_structures.StructureTestCase.test_array_in_struct"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_structures.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_structures
_suite = unittest.defaultTestLoader.loadTestsFromName("StructureTestCase.test_array_in_struct", test_structures)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StructureTestCase.test_array_in_struct did not pass"
print("StructureTestCase::test_array_in_struct: ok")
