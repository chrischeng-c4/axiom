# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subclassinit"
# dimension = "behavior"
# case = "test__test_init_subclass_kwargs"
# subject = "cpython.test_subclassinit.Test.test_init_subclass_kwargs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subclassinit.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_subclassinit.py::Test::test_init_subclass_kwargs
"""Auto-ported test: Test::test_init_subclass_kwargs (CPython 3.12 oracle)."""


import types
import unittest


# --- test body ---
class A:

    def __init_subclass__(cls, **kwargs):
        cls.kwargs = kwargs

class B(A, x=3):
    pass

assert B.kwargs == dict(x=3)
print("Test::test_init_subclass_kwargs: ok")
