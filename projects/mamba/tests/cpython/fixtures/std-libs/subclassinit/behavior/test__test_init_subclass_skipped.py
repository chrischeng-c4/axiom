# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subclassinit"
# dimension = "behavior"
# case = "test__test_init_subclass_skipped"
# subject = "cpython.test_subclassinit.Test.test_init_subclass_skipped"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subclassinit.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_subclassinit.py::Test::test_init_subclass_skipped
"""Auto-ported test: Test::test_init_subclass_skipped (CPython 3.12 oracle)."""


import types
import unittest


# --- test body ---
class BaseWithInit:

    def __init_subclass__(cls, **kwargs):
        super().__init_subclass__(**kwargs)
        cls.initialized = cls

class BaseWithoutInit(BaseWithInit):
    pass

class A(BaseWithoutInit):
    pass

assert A.initialized is A

assert BaseWithoutInit.initialized is BaseWithoutInit
print("Test::test_init_subclass_skipped: ok")
