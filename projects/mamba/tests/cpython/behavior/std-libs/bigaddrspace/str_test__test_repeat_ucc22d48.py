# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bigaddrspace"
# dimension = "behavior"
# case = "str_test__test_repeat_ucc22d48"
# subject = "cpython.test_bigaddrspace.StrTest.test_repeat"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bigaddrspace.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_bigaddrspace
_suite = unittest.defaultTestLoader.loadTestsFromName("StrTest.test_repeat", test_bigaddrspace)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StrTest.test_repeat did not pass"
print("StrTest::test_repeat: ok")
