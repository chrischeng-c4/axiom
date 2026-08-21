# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "python_api"
# dimension = "behavior"
# case = "python_a_p_i_test_case__test_pybytes_fromstringandsize_uceb7fa2"
# subject = "cpython.test_python_api.PythonAPITestCase.test_PyBytes_FromStringAndSize"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_python_api.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
from _ctypes import PyObj_FromPtr
from sys import getrefcount as grc
PyBytes_FromStringAndSize = pythonapi.PyBytes_FromStringAndSize
PyBytes_FromStringAndSize.restype = py_object
PyBytes_FromStringAndSize.argtypes = (c_char_p, c_size_t)
assert PyBytes_FromStringAndSize(b'abcdefghi', 3) == b'abc'

print("PythonAPITestCase::test_PyBytes_FromStringAndSize: ok")
