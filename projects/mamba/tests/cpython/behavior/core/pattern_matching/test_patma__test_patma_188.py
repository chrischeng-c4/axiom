# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_188"
# subject = "cpython.test_patma.TestPatma.test_patma_188"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_188
"""Auto-ported test: TestPatma::test_patma_188 (CPython 3.12 oracle)."""


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
w = range(100)
match w:
    case [x, y, *rest]:
        z = 0

assert w == range(100)

assert x == 0

assert y == 1

assert z == 0

assert rest == list(range(2, 100))
print("TestPatma::test_patma_188: ok")
