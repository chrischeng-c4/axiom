# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_type_errors__test_accepts_positional_subpatterns_1"
# subject = "cpython.test_patma.TestTypeErrors.test_accepts_positional_subpatterns_1"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestTypeErrors::test_accepts_positional_subpatterns_1
"""Auto-ported test: TestTypeErrors::test_accepts_positional_subpatterns_1 (CPython 3.12 oracle)."""


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
x = range(10)
y = None
try:
    match x:
        case range(10):
            y = 0
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert x == range(10)

assert y is None
print("TestTypeErrors::test_accepts_positional_subpatterns_1: ok")
