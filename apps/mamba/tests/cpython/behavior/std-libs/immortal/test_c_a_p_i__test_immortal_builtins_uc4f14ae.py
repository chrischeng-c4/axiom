# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "immortal"
# dimension = "behavior"
# case = "test_c_a_p_i__test_immortal_builtins_uc4f14ae"
# subject = "cpython.test_immortal.TestCAPI.test_immortal_builtins"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_immortal.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_immortal
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCAPI.test_immortal_builtins", test_immortal)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCAPI.test_immortal_builtins did not pass"
print("TestCAPI::test_immortal_builtins: ok")
