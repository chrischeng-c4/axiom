# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_205"
# subject = "cpython.test_patma.TestPatma.test_patma_205"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_205
"""Auto-ported test: TestPatma::test_patma_205 (CPython 3.12 oracle)."""


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
        case 42.0:
            out = locals()
            del out['w']
            return out

assert f(42.0) == {}

assert f(42) == {}

assert f(0.0) is None

assert f(0) is None
print("TestPatma::test_patma_205: ok")
