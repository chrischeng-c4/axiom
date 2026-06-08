# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_182"
# subject = "cpython.test_patma.TestPatma.test_patma_182"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_182
"""Auto-ported test: TestPatma::test_patma_182 (CPython 3.12 oracle)."""


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
def whereis(points):
    match points:
        case []:
            return 'No points'
        case [Point(0, 0)]:
            return 'The origin'
        case [Point(x, y)]:
            return f'Single point {x}, {y}'
        case [Point(0, y1), Point(0, y2)]:
            return f'Two on the Y axis at {y1}, {y2}'
        case _:
            return 'Something else'

assert whereis([]) == 'No points'

assert whereis([Point(0, 0)]) == 'The origin'

assert whereis([Point(0, 1)]) == 'Single point 0, 1'

assert whereis([Point(0, 0), Point(0, 0)]) == 'Two on the Y axis at 0, 0'

assert whereis([Point(0, 1), Point(0, 1)]) == 'Two on the Y axis at 1, 1'

assert whereis([Point(0, 0), Point(1, 0)]) == 'Something else'

assert whereis([Point(0, 0), Point(0, 0), Point(0, 0)]) == 'Something else'

assert whereis([Point(0, 1), Point(0, 1), Point(0, 1)]) == 'Something else'
print("TestPatma::test_patma_182: ok")
