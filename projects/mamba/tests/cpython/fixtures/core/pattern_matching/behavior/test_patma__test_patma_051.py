# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_051"
# subject = "cpython.test_patma.TestPatma.test_patma_051"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_051
"""Auto-ported test: TestPatma::test_patma_051 (CPython 3.12 oracle)."""


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
w = None
x = [1, 0]
match x:
    case [0 as w]:
        y = 0
    case [z] | [1, 0 | 1 as z] | [z]:
        y = 1

assert w is None

assert x == [1, 0]

assert y == 1

assert z == 0
print("TestPatma::test_patma_051: ok")
