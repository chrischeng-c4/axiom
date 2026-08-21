# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "code_test__test_co_lnotab_is_deprecated_uccf1192"
# subject = "cpython.test_code.CodeTest.test_co_lnotab_is_deprecated"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_code
_suite = unittest.defaultTestLoader.loadTestsFromName("CodeTest.test_co_lnotab_is_deprecated", test_code)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CodeTest.test_co_lnotab_is_deprecated did not pass"
print("CodeTest::test_co_lnotab_is_deprecated: ok")
