# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "py_compile"
# dimension = "behavior"
# case = "py_compile_c_l_i_test_case__test_bad_syntax_uca113d4"
# subject = "cpython.test_py_compile.PyCompileCLITestCase.test_bad_syntax"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_py_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_py_compile
_suite = unittest.defaultTestLoader.loadTestsFromName("PyCompileCLITestCase.test_bad_syntax", test_py_compile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PyCompileCLITestCase.test_bad_syntax did not pass"
print("PyCompileCLITestCase::test_bad_syntax: ok")
