# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_196"
# subject = "cpython.test_patma.TestPatma.test_patma_196"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_196
"""Auto-ported test: TestPatma::test_patma_196 (CPython 3.12 oracle)."""


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
x = {'bandwidth': 0, 'latency': 1}
match x:
    case {'latency': l, 'bandwidth': b, **rest}:
        y = 0

assert x == {'bandwidth': 0, 'latency': 1}

assert l is x['latency']

assert b is x['bandwidth']

assert rest == {}

assert y == 0
print("TestPatma::test_patma_196: ok")
