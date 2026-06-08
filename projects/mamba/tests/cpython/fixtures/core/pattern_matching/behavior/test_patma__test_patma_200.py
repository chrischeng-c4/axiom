# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_200"
# subject = "cpython.test_patma.TestPatma.test_patma_200"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_200
"""Auto-ported test: TestPatma::test_patma_200 (CPython 3.12 oracle)."""


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
class Class:
    __match_args__ = ('a', 'b')
c = Class()
c.a = 0
c.b = 1
match c:
    case Class(x, y):
        z = 0

assert x is c.a

assert y is c.b

assert z == 0
print("TestPatma::test_patma_200: ok")
