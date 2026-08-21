# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_177"
# subject = "cpython.test_patma.TestPatma.test_patma_177"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_177
"""Auto-ported test: TestPatma::test_patma_177 (CPython 3.12 oracle)."""


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
def whereis(point):
    match point:
        case Point(0, 0):
            return 'Origin'
        case Point(0, y):
            return f'Y={y}'
        case Point(x, 0):
            return f'X={x}'
        case Point():
            return 'Somewhere else'
        case _:
            return 'Not a point'

assert whereis(Point(1, 0)) == 'X=1'

assert whereis(Point(0, 0)) == 'Origin'

assert whereis(10) == 'Not a point'

assert whereis(Point(False, False)) == 'Origin'

assert whereis(Point(0, -1.0)) == 'Y=-1.0'

assert whereis(Point('X', 0)) == 'X=X'

assert whereis(Point(None, 1j)) == 'Somewhere else'

assert whereis(Point) == 'Not a point'

assert whereis(42) == 'Not a point'
print("TestPatma::test_patma_177: ok")
