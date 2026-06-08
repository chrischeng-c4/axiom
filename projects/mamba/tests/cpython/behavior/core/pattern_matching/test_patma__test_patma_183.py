# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_183"
# subject = "cpython.test_patma.TestPatma.test_patma_183"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_183
"""Auto-ported test: TestPatma::test_patma_183 (CPython 3.12 oracle)."""


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
        case Point(x, y) if x == y:
            return f'Y=X at {x}'
        case Point(x, y):
            return 'Not on the diagonal'

assert whereis(Point(0, 0)) == 'Y=X at 0'

assert whereis(Point(0, False)) == 'Y=X at 0'

assert whereis(Point(False, 0)) == 'Y=X at False'

assert whereis(Point(-1 - 1j, -1 - 1j)) == 'Y=X at (-1-1j)'

assert whereis(Point('X', 'X')) == 'Y=X at X'

assert whereis(Point('X', 'x')) == 'Not on the diagonal'
print("TestPatma::test_patma_183: ok")
