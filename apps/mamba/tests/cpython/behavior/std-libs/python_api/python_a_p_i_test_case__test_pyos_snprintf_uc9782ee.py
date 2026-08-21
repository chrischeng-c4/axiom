# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "python_api"
# dimension = "behavior"
# case = "python_a_p_i_test_case__test_pyos_snprintf_uc9782ee"
# subject = "cpython.test_python_api.PythonAPITestCase.test_PyOS_snprintf"
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
PyOS_snprintf = pythonapi.PyOS_snprintf
PyOS_snprintf.argtypes = (POINTER(c_char), c_size_t, c_char_p)
buf = c_buffer(256)
PyOS_snprintf(buf, sizeof(buf), b'Hello from %s', b'ctypes')
assert buf.value == b'Hello from ctypes'
PyOS_snprintf(buf, sizeof(buf), b'Hello from %s (%d, %d, %d)', b'ctypes', 1, 2, 3)
assert buf.value == b'Hello from ctypes (1, 2, 3)'
try:
    PyOS_snprintf(buf)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("PythonAPITestCase::test_PyOS_snprintf: ok")
