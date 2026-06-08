# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subclassinit"
# dimension = "behavior"
# case = "test__test_init_subclass_diamond"
# subject = "cpython.test_subclassinit.Test.test_init_subclass_diamond"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subclassinit.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_subclassinit.py::Test::test_init_subclass_diamond
"""Auto-ported test: Test::test_init_subclass_diamond (CPython 3.12 oracle)."""


import types
import unittest


# --- test body ---
class Base:

    def __init_subclass__(cls, **kwargs):
        super().__init_subclass__(**kwargs)
        cls.calls = []

class Left(Base):
    pass

class Middle:

    def __init_subclass__(cls, middle, **kwargs):
        super().__init_subclass__(**kwargs)
        cls.calls += [middle]

class Right(Base):

    def __init_subclass__(cls, right='right', **kwargs):
        super().__init_subclass__(**kwargs)
        cls.calls += [right]

class A(Left, Middle, Right, middle='middle'):
    pass

assert A.calls == ['right', 'middle']

assert Left.calls == []

assert Right.calls == []
print("Test::test_init_subclass_diamond: ok")
