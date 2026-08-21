# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_155"
# subject = "cpython.test_patma.TestPatma.test_patma_155"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_155
"""Auto-ported test: TestPatma::test_patma_155 (CPython 3.12 oracle)."""


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
x = 0
y = None
match x:
    case 1e309:
        y = 0

assert x == 0

assert y is None
print("TestPatma::test_patma_155: ok")
