# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structures"
# dimension = "behavior"
# case = "structure_test_case__test_packed_c_limits"
# subject = "cpython.test_structures.StructureTestCase.test_packed_c_limits"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_structures.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_structures
_suite = unittest.defaultTestLoader.loadTestsFromName("StructureTestCase.test_packed_c_limits", test_structures)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StructureTestCase.test_packed_c_limits did not pass"
print("StructureTestCase::test_packed_c_limits: ok")
