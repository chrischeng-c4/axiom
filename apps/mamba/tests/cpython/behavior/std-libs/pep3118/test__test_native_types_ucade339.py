# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pep3118"
# dimension = "behavior"
# case = "test__test_native_types_ucade339"
# subject = "cpython.test_pep3118.Test.test_native_types"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_pep3118.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_pep3118
_suite = unittest.defaultTestLoader.loadTestsFromName("Test.test_native_types", test_pep3118)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test.test_native_types did not pass"
print("Test::test_native_types: ok")
