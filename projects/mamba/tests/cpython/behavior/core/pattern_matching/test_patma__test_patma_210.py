# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_210"
# subject = "cpython.test_patma.TestPatma.test_patma_210"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_210
"""Auto-ported test: TestPatma::test_patma_210 (CPython 3.12 oracle)."""


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
def f(w):
    match w:
        case [x, y, z]:
            out = locals()
            del out['w']
            return out

assert f((1, 2, 3)) == {'x': 1, 'y': 2, 'z': 3}

assert f((1, 2)) is None

assert f((1, 2, 3, 4)) is None

assert f(123) is None

assert f('abc') is None

assert f(b'abc') is None

assert f(array.array('b', b'abc')) == {'x': 97, 'y': 98, 'z': 99}

assert f(memoryview(b'abc')) == {'x': 97, 'y': 98, 'z': 99}

assert f(bytearray(b'abc')) is None
print("TestPatma::test_patma_210: ok")
