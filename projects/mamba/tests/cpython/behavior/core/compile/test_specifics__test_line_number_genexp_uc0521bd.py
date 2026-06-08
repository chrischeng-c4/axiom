# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_line_number_genexp_uc0521bd"
# subject = "cpython.test_compile.TestSpecifics.test_line_number_genexp"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_compile
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSpecifics.test_line_number_genexp", test_compile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSpecifics.test_line_number_genexp did not pass"
print("TestSpecifics::test_line_number_genexp: ok")
