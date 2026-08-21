# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structures"
# dimension = "behavior"
# case = "structure_test_case__test_issue18060_c"
# subject = "cpython.test_structures.StructureTestCase.test_issue18060_c"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_structures.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_structures
_suite = unittest.defaultTestLoader.loadTestsFromName("StructureTestCase.test_issue18060_c", test_structures)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StructureTestCase.test_issue18060_c did not pass"
print("StructureTestCase::test_issue18060_c: ok")
