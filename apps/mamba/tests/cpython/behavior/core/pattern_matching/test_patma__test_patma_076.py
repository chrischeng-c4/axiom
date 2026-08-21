# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_076"
# subject = "cpython.test_patma.TestPatma.test_patma_076"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_076
"""Auto-ported test: TestPatma::test_patma_076 (CPython 3.12 oracle)."""


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
x = b'x'
match x:
    case [b'x']:
        y = 0
    case ['x']:
        y = 1
    case [120]:
        y = 2
    case b'x':
        y = 4

assert x == b'x'

assert y == 4
print("TestPatma::test_patma_076: ok")
