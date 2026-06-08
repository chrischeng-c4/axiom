# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subclassinit"
# dimension = "behavior"
# case = "test__test_init_subclass_error"
# subject = "cpython.test_subclassinit.Test.test_init_subclass_error"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subclassinit.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_subclassinit.py::Test::test_init_subclass_error
"""Auto-ported test: Test::test_init_subclass_error (CPython 3.12 oracle)."""


import types
import unittest


# --- test body ---
class A:

    def __init_subclass__(cls):
        raise RuntimeError
try:

    class B(A):
        pass
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass
print("Test::test_init_subclass_error: ok")
