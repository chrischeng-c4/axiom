# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_207"
# subject = "cpython.test_patma.TestPatma.test_patma_207"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_207
"""Auto-ported test: TestPatma::test_patma_207 (CPython 3.12 oracle)."""


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
        case [1, 2] | [3, 4]:
            out = locals()
            del out['w']
            return out

assert f([1, 2]) == {}

assert f([3, 4]) == {}

assert f(42) is None

assert f([2, 3]) is None

assert f([1, 2, 3]) is None

assert f([1, 2.0]) == {}
print("TestPatma::test_patma_207: ok")
