# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "macholib"
# dimension = "behavior"
# case = "mach_o_test__test_info_uce2d18e"
# subject = "cpython.test_macholib.MachOTest.test_info"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_macholib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_macholib
_suite = unittest.defaultTestLoader.loadTestsFromName("MachOTest.test_info", test_macholib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MachOTest.test_info did not pass"
print("MachOTest::test_info: ok")
