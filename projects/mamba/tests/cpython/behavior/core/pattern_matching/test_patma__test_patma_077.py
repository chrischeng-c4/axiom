# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_077"
# subject = "cpython.test_patma.TestPatma.test_patma_077"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_077
"""Auto-ported test: TestPatma::test_patma_077 (CPython 3.12 oracle)."""


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
x = bytearray(b'x')
y = None
match x:
    case [120]:
        y = 0
    case 120:
        y = 1

assert x == b'x'

assert y is None
print("TestPatma::test_patma_077: ok")
