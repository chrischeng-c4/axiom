# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_232"
# subject = "cpython.test_patma.TestPatma.test_patma_232"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_232
"""Auto-ported test: TestPatma::test_patma_232 (CPython 3.12 oracle)."""


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
class Eq:

    def __eq__(self, other):
        return True
x = eq = Eq()
y = None
match x:
    case None:
        y = 0

assert x is eq

assert y == None
print("TestPatma::test_patma_232: ok")
