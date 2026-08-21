# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_197"
# subject = "cpython.test_patma.TestPatma.test_patma_197"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_197
"""Auto-ported test: TestPatma::test_patma_197 (CPython 3.12 oracle)."""


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
w = [Point(-1, 0), Point(1, 2)]
match w:
    case [Point(x1, y1), Point(x2, y2) as p2]:
        z = 0

assert w == [Point(-1, 0), Point(1, 2)]

assert x1 is w[0].x

assert y1 is w[0].y

assert p2 is w[1]

assert x2 is w[1].x

assert y2 is w[1].y

assert z is 0
print("TestPatma::test_patma_197: ok")
