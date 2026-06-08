# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_lineno_after_implicit_return_ucd70dc9"
# subject = "cpython.test_compile.TestSpecifics.test_lineno_after_implicit_return"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_compile
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSpecifics.test_lineno_after_implicit_return", test_compile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSpecifics.test_lineno_after_implicit_return did not pass"
print("TestSpecifics::test_lineno_after_implicit_return: ok")
