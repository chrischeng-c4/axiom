# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_211"
# subject = "cpython.test_patma.TestPatma.test_patma_211"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_211
"""Auto-ported test: TestPatma::test_patma_211 (CPython 3.12 oracle)."""


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
        case {'x': x, 'y': 'y', 'z': z}:
            out = locals()
            del out['w']
            return out

assert f({'x': 'x', 'y': 'y', 'z': 'z'}) == {'x': 'x', 'z': 'z'}

assert f({'x': 'x', 'y': 'y', 'z': 'z', 'a': 'a'}) == {'x': 'x', 'z': 'z'}

assert f({'x': 'x', 'y': 'yy', 'z': 'z', 'a': 'a'}) is None

assert f({'x': 'x', 'y': 'y'}) is None
print("TestPatma::test_patma_211: ok")
