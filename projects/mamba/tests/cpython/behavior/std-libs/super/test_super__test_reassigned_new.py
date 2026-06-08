# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "super"
# dimension = "behavior"
# case = "test_super__test_reassigned_new"
# subject = "cpython.test_super.TestSuper.test_reassigned_new"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_super.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_super.py::TestSuper::test_reassigned_new
"""Auto-ported test: TestSuper::test_reassigned_new (CPython 3.12 oracle)."""


import textwrap
import threading
import unittest
from unittest.mock import patch
from test.support import import_helper, threading_helper


'Unit tests for zero-argument super() & related machinery.'

ADAPTIVE_WARMUP_DELAY = 2

class A:

    def f(self):
        return 'A'

    @classmethod
    def cm(cls):
        return (cls, 'A')

class B(A):

    def f(self):
        return super().f() + 'B'

    @classmethod
    def cm(cls):
        return (cls, super().cm(), 'B')

class C(A):

    def f(self):
        return super().f() + 'C'

    @classmethod
    def cm(cls):
        return (cls, super().cm(), 'C')

class D(C, B):

    def f(self):
        return super().f() + 'D'

    def cm(cls):
        return (cls, super().cm(), 'D')

class E(D):
    pass

class F(E):
    f = E.f

class G(A):
    pass


# --- test body ---
class A:

    def __new__(cls):
        pass

    def __init_subclass__(cls):
        if '__new__' not in cls.__dict__:
            cls.__new__ = cls.__new__

class B(A):
    pass

class C(B):

    def __new__(cls):
        return super().__new__(cls)
for _ in range(ADAPTIVE_WARMUP_DELAY):
    C()
print("TestSuper::test_reassigned_new: ok")
