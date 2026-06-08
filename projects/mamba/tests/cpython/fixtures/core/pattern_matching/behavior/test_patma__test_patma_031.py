# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_031"
# subject = "cpython.test_patma.TestPatma.test_patma_031"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_031
"""Auto-ported test: TestPatma::test_patma_031 (CPython 3.12 oracle)."""


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
x = {False: (True, 2.0, {}), 1: [[]], 2: 0}
match x:
    case {0: [1, 2, {}]}:
        y = 0
    case {0: [1, 2, {}], 1: [[]]}:
        y = 1
    case []:
        y = 2

assert x == {False: (True, 2.0, {}), 1: [[]], 2: 0}

assert y == 0
print("TestPatma::test_patma_031: ok")
