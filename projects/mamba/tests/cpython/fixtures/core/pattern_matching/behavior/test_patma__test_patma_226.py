# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_226"
# subject = "cpython.test_patma.TestPatma.test_patma_226"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_226
"""Auto-ported test: TestPatma::test_patma_226 (CPython 3.12 oracle)."""


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
        case 0:
            return 0
        case 1:
            return 1
        case _:
            return 2

assert f(0) == 0

assert f(1) == 1

assert f(2) == 2

assert f(3) == 2
print("TestPatma::test_patma_226: ok")
