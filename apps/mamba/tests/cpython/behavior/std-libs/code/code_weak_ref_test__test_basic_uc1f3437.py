# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "code_weak_ref_test__test_basic_uc1f3437"
# subject = "cpython.test_code.CodeWeakRefTest.test_basic"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_code
_suite = unittest.defaultTestLoader.loadTestsFromName("CodeWeakRefTest.test_basic", test_code)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CodeWeakRefTest.test_basic did not pass"
print("CodeWeakRefTest::test_basic: ok")
