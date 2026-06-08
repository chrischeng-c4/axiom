# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "parser"
# dimension = "behavior"
# case = "test_matrix_multiplication__test_matrix_multiplication_operator"
# subject = "cpython.test_parser.TestMatrixMultiplication.test_matrix_multiplication_operator"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lib2to3/test_parser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_lib2to3 import test_parser
_suite = unittest.defaultTestLoader.loadTestsFromName("TestMatrixMultiplication.test_matrix_multiplication_operator", test_parser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestMatrixMultiplication.test_matrix_multiplication_operator did not pass"
print("TestMatrixMultiplication::test_matrix_multiplication_operator: ok")
