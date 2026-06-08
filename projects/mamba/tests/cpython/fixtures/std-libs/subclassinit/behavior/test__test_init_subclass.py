# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subclassinit"
# dimension = "behavior"
# case = "test__test_init_subclass"
# subject = "cpython.test_subclassinit.Test.test_init_subclass"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subclassinit.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_subclassinit.py::Test::test_init_subclass
"""Auto-ported test: Test::test_init_subclass (CPython 3.12 oracle)."""


import types
import unittest


# --- test body ---
class A:
    initialized = False

    def __init_subclass__(cls):
        super().__init_subclass__()
        cls.initialized = True

class B(A):
    pass

assert not A.initialized

assert B.initialized
print("Test::test_init_subclass: ok")
