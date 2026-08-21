# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "type_var_tuple_tests__test_variadic_class_alias_repr_is_correct"
# subject = "cpython.test_typing.TypeVarTupleTests.test_variadic_class_alias_repr_is_correct"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_typing
_suite = unittest.defaultTestLoader.loadTestsFromName("TypeVarTupleTests.test_variadic_class_alias_repr_is_correct", test_typing)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TypeVarTupleTests.test_variadic_class_alias_repr_is_correct did not pass"
print("TypeVarTupleTests::test_variadic_class_alias_repr_is_correct: ok")
