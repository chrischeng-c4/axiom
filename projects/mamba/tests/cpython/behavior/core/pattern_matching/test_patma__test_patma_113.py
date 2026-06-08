# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_113"
# subject = "cpython.test_patma.TestPatma.test_patma_113"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_113
"""Auto-ported test: TestPatma::test_patma_113 (CPython 3.12 oracle)."""


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

    class B:
        C = 0
        D = 1
x = 1
match x:
    case A.B.C:
        y = 0
    case A.B.D:
        y = 1

assert A.B.C == 0

assert A.B.D == 1

assert x == 1

assert y == 1
print("TestPatma::test_patma_113: ok")
