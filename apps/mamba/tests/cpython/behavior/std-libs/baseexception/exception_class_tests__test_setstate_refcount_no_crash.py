# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "baseexception"
# dimension = "behavior"
# case = "exception_class_tests__test_setstate_refcount_no_crash"
# subject = "cpython.test_baseexception.ExceptionClassTests.test_setstate_refcount_no_crash"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_baseexception.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_baseexception.py::ExceptionClassTests::test_setstate_refcount_no_crash
"""Auto-ported test: ExceptionClassTests::test_setstate_refcount_no_crash (CPython 3.12 oracle)."""


import unittest
import builtins
import os
from platform import system as platform_system


# --- test body ---
interface_tests = ('length', 'args', 'str', 'repr')
import gc
d = {}

class HashThisKeyWillClearTheDict(str):

    def __hash__(self) -> int:
        d.clear()
        return super().__hash__()

class Value(str):
    pass
exc = Exception()
d[HashThisKeyWillClearTheDict()] = Value()
exc.__setstate__(d)
gc.collect()
print("ExceptionClassTests::test_setstate_refcount_no_crash: ok")
