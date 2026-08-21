# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "python_api"
# dimension = "behavior"
# case = "python_a_p_i_test_case__test_pyobject_repr_uc1dc6fc"
# subject = "cpython.test_python_api.PythonAPITestCase.test_pyobject_repr"
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
assert repr(py_object()) == 'py_object(<NULL>)'
assert repr(py_object(42)) == 'py_object(42)'
assert repr(py_object(object)) == 'py_object(%r)' % object

print("PythonAPITestCase::test_pyobject_repr: ok")
