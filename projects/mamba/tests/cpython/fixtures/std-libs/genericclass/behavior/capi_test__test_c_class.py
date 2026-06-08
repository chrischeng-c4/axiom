# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "capi_test__test_c_class"
# subject = "cpython.test_genericclass.CAPITest.test_c_class"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_genericclass.py::CAPITest::test_c_class
"""Auto-ported test: CAPITest::test_c_class (CPython 3.12 oracle)."""


import unittest
from test import support


# --- test body ---
from _testcapi import Generic, GenericAlias

assert isinstance(Generic.__class_getitem__(int), GenericAlias)
IntGeneric = Generic[int]

assert type(IntGeneric) is GenericAlias

assert IntGeneric.__mro_entries__(()) == (int,)

class C(IntGeneric):
    pass

assert C.__bases__ == (int,)

assert C.__orig_bases__ == (IntGeneric,)

assert C.__mro__ == (C, int, object)
print("CAPITest::test_c_class: ok")
