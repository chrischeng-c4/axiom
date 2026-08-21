# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_208"
# subject = "cpython.test_patma.TestPatma.test_patma_208"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_208
"""Auto-ported test: TestPatma::test_patma_208 (CPython 3.12 oracle)."""


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
        case x:
            out = locals()
            del out['w']
            return out

assert f(42) == {'x': 42}

assert f((1, 2)) == {'x': (1, 2)}

assert f(None) == {'x': None}
print("TestPatma::test_patma_208: ok")
