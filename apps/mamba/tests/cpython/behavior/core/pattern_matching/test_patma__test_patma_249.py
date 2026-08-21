# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_249"
# subject = "cpython.test_patma.TestPatma.test_patma_249"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_249
"""Auto-ported test: TestPatma::test_patma_249 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
class C:
    __attr = 'eggs'
    _Outer__attr = 'bacon'

class Outer:

    def f(self, x):
        match x:
            case C(__attr=y):
                return y
c = C()
setattr(c, '__attr', 'spam')

assert Outer().f(c) == 'spam'
print("TestPatma::test_patma_249: ok")
