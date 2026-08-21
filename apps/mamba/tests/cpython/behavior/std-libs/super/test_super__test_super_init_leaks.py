# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "super"
# dimension = "behavior"
# case = "test_super__test_super_init_leaks"
# subject = "cpython.test_super.TestSuper.test_super_init_leaks"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_super.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_super.py::TestSuper::test_super_init_leaks
"""Auto-ported test: TestSuper::test_super_init_leaks (CPython 3.12 oracle)."""


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
sp = super(float, 1.0)
for i in range(1000):
    super.__init__(sp, int, i)
print("TestSuper::test_super_init_leaks: ok")
