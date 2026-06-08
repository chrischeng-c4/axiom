# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "mapping_test_case__test_make_weak_keyed_dict_repr_uc615230"
# subject = "cpython.test_weakref.MappingTestCase.test_make_weak_keyed_dict_repr"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_weakref
_suite = unittest.defaultTestLoader.loadTestsFromName("MappingTestCase.test_make_weak_keyed_dict_repr", test_weakref)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MappingTestCase.test_make_weak_keyed_dict_repr did not pass"
print("MappingTestCase::test_make_weak_keyed_dict_repr: ok")
