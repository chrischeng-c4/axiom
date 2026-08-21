# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "prototypes"
# dimension = "behavior"
# case = "char_pointers_test_case__test_c_void_p_arg_ucca33bb"
# subject = "cpython.test_prototypes.CharPointersTestCase.test_c_void_p_arg"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_prototypes.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_prototypes
_suite = unittest.defaultTestLoader.loadTestsFromName("CharPointersTestCase.test_c_void_p_arg", test_prototypes)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CharPointersTestCase.test_c_void_p_arg did not pass"
print("CharPointersTestCase::test_c_void_p_arg: ok")
