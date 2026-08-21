# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_135"
# subject = "cpython.test_patma.TestPatma.test_patma_135"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_135
"""Auto-ported test: TestPatma::test_patma_135 (CPython 3.12 oracle)."""


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
x = collections.defaultdict(int, {0: 1})
match x:
    case {1: 0}:
        y = 0
    case {0: 0}:
        y = 1
    case {0: _, **z}:
        y = 2

assert x == {0: 1}

assert y == 2

assert z == {}
print("TestPatma::test_patma_135: ok")
