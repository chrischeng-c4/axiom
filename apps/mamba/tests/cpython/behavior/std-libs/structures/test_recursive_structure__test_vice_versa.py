# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structures"
# dimension = "behavior"
# case = "test_recursive_structure__test_vice_versa"
# subject = "cpython.test_structures.TestRecursiveStructure.test_vice_versa"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_structures.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_structures
_suite = unittest.defaultTestLoader.loadTestsFromName("TestRecursiveStructure.test_vice_versa", test_structures)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestRecursiveStructure.test_vice_versa did not pass"
print("TestRecursiveStructure::test_vice_versa: ok")
