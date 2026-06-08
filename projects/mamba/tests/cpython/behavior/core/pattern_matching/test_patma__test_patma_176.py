# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_176"
# subject = "cpython.test_patma.TestPatma.test_patma_176"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_176
"""Auto-ported test: TestPatma::test_patma_176 (CPython 3.12 oracle)."""


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
        case [0, 0]:
            return 'Origin'
        case [0, y]:
            return f'Y={y}'
        case [x, 0]:
            return f'X={x}'
        case [x, y]:
            return f'X={x}, Y={y}'
        case _:
            return 'Not a point'

assert whereis((0, 0)) == 'Origin'

assert whereis((0, -1.0)) == 'Y=-1.0'

assert whereis(('X', 0)) == 'X=X'

assert whereis((None, 1j)) == 'X=None, Y=1j'

assert whereis(42) == 'Not a point'
print("TestPatma::test_patma_176: ok")
