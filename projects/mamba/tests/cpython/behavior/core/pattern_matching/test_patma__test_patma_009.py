# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_009"
# subject = "cpython.test_patma.TestPatma.test_patma_009"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_009
"""Auto-ported test: TestPatma::test_patma_009 (CPython 3.12 oracle)."""


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
class A:
    B = 0
match 0:
    case x if x:
        z = 0
    case _ as y if y == x and y:
        z = 1
    case A.B:
        z = 2

assert A.B == 0

assert x == 0

assert y == 0

assert z == 2
print("TestPatma::test_patma_009: ok")
