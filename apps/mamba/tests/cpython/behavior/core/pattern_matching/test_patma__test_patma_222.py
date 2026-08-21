# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_222"
# subject = "cpython.test_patma.TestPatma.test_patma_222"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_222
"""Auto-ported test: TestPatma::test_patma_222 (CPython 3.12 oracle)."""


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
def f(x):
    match x:
        case _:
            return 0

assert f(0) == 0

assert f(1) == 0

assert f(2) == 0

assert f(3) == 0
print("TestPatma::test_patma_222: ok")
