# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_238"
# subject = "cpython.test_patma.TestPatma.test_patma_238"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_238
"""Auto-ported test: TestPatma::test_patma_238 (CPython 3.12 oracle)."""


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
x = ((0, 1), (2, 3))
match x:
    case [([a as b, c as d] as e) as w, ([f as g, h] as i) as z]:
        y = 0

assert a == 0

assert b == 0

assert c == 1

assert d == 1

assert e == (0, 1)

assert f == 2

assert g == 2

assert h == 3

assert i == (2, 3)

assert w == (0, 1)

assert x == ((0, 1), (2, 3))

assert y == 0

assert z == (2, 3)
print("TestPatma::test_patma_238: ok")
