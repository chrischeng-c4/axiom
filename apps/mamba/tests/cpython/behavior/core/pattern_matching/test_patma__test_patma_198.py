# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_198"
# subject = "cpython.test_patma.TestPatma.test_patma_198"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_198
"""Auto-ported test: TestPatma::test_patma_198 (CPython 3.12 oracle)."""


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
class Color(enum.Enum):
    RED = 0
    GREEN = 1
    BLUE = 2

def f(color):
    match color:
        case Color.RED:
            return 'I see red!'
        case Color.GREEN:
            return 'Grass is green'
        case Color.BLUE:
            return "I'm feeling the blues :("

assert f(Color.RED) == 'I see red!'

assert f(Color.GREEN) == 'Grass is green'

assert f(Color.BLUE) == "I'm feeling the blues :("

assert f(Color) is None

assert f(0) is None

assert f(1) is None

assert f(2) is None

assert f(3) is None

assert f(False) is None

assert f(True) is None

assert f(2 + 0j) is None

assert f(3.0) is None
print("TestPatma::test_patma_198: ok")
