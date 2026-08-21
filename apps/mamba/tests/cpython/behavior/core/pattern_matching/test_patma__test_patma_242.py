# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_242"
# subject = "cpython.test_patma.TestPatma.test_patma_242"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_242
"""Auto-ported test: TestPatma::test_patma_242 (CPython 3.12 oracle)."""


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
x = range(3)
match x:
    case [y, *_, z]:
        w = 0

assert w == 0

assert x == range(3)

assert y == 0

assert z == 2
print("TestPatma::test_patma_242: ok")
