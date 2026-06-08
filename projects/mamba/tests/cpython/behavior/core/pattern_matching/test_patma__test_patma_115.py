# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_115"
# subject = "cpython.test_patma.TestPatma.test_patma_115"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_115
"""Auto-ported test: TestPatma::test_patma_115 (CPython 3.12 oracle)."""


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

        class C:
            D = 0
            E = 1
x = 1
match x:
    case A.B.C.D:
        y = 0
    case A.B.C.E:
        y = 1

assert A.B.C.D == 0

assert A.B.C.E == 1

assert x == 1

assert y == 1
print("TestPatma::test_patma_115: ok")
