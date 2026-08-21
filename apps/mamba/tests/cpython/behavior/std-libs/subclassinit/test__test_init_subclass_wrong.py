# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subclassinit"
# dimension = "behavior"
# case = "test__test_init_subclass_wrong"
# subject = "cpython.test_subclassinit.Test.test_init_subclass_wrong"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subclassinit.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_subclassinit.py::Test::test_init_subclass_wrong
"""Auto-ported test: Test::test_init_subclass_wrong (CPython 3.12 oracle)."""


import types
import unittest


# --- test body ---
class A:

    def __init_subclass__(cls, whatever):
        pass
try:

    class B(A):
        pass
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("Test::test_init_subclass_wrong: ok")
