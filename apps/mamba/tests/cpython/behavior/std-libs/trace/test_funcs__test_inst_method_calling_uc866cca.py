# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "behavior"
# case = "test_funcs__test_inst_method_calling_uc866cca"
# subject = "cpython.test_trace.TestFuncs.test_inst_method_calling"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_trace.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_trace
_suite = unittest.defaultTestLoader.loadTestsFromName("TestFuncs.test_inst_method_calling", test_trace)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestFuncs.test_inst_method_calling did not pass"
print("TestFuncs::test_inst_method_calling: ok")
