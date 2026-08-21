# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "c_simple_type_meta"
# dimension = "behavior"
# case = "py_c_simple_type_as_metaclass_test__test_creating_pointer_in_dunder_init_2_uc5e2cad"
# subject = "cpython.test_c_simple_type_meta.PyCSimpleTypeAsMetaclassTest.test_creating_pointer_in_dunder_init_2"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_c_simple_type_meta.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_c_simple_type_meta
_suite = unittest.defaultTestLoader.loadTestsFromName("PyCSimpleTypeAsMetaclassTest.test_creating_pointer_in_dunder_init_2", test_c_simple_type_meta)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PyCSimpleTypeAsMetaclassTest.test_creating_pointer_in_dunder_init_2 did not pass"
print("PyCSimpleTypeAsMetaclassTest::test_creating_pointer_in_dunder_init_2: ok")
