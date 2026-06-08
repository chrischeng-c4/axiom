# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "c_whitebox__test_maxcontext_exact_arith"
# subject = "cpython.test_decimal.CWhitebox.test_maxcontext_exact_arith"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_decimal
_suite = unittest.defaultTestLoader.loadTestsFromName("CWhitebox.test_maxcontext_exact_arith", test_decimal)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CWhitebox.test_maxcontext_exact_arith did not pass"
print("CWhitebox::test_maxcontext_exact_arith: ok")
