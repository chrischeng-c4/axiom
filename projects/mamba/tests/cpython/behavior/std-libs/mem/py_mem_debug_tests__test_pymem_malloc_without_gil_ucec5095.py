# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mem"
# dimension = "behavior"
# case = "py_mem_debug_tests__test_pymem_malloc_without_gil_ucec5095"
# subject = "cpython.test_mem.PyMemDebugTests.test_pymem_malloc_without_gil"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_mem.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_mem
_suite = unittest.defaultTestLoader.loadTestsFromName("PyMemDebugTests.test_pymem_malloc_without_gil", test_mem)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PyMemDebugTests.test_pymem_malloc_without_gil did not pass"
print("PyMemDebugTests::test_pymem_malloc_without_gil: ok")
