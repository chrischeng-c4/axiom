# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_180"
# subject = "cpython.test_patma.TestPatma.test_patma_180"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_180
"""Auto-ported test: TestPatma::test_patma_180 (CPython 3.12 oracle)."""


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
def whereis(point):
    match point:
        case Point(x=1, y=var):
            return var

assert whereis(Point(1, 0)) == 0

assert whereis(Point(0, 0)) is None
print("TestPatma::test_patma_180: ok")
