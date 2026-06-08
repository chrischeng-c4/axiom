# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "python_api"
# dimension = "behavior"
# case = "python_a_p_i_test_case__test_pyobj_fromptr_ucda2bdb"
# subject = "cpython.test_python_api.PythonAPITestCase.test_PyObj_FromPtr"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_python_api.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_python_api
_suite = unittest.defaultTestLoader.loadTestsFromName("PythonAPITestCase.test_PyObj_FromPtr", test_python_api)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PythonAPITestCase.test_PyObj_FromPtr did not pass"
print("PythonAPITestCase::test_PyObj_FromPtr: ok")
