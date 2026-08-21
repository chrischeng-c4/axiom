# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_type_errors__test_got_multiple_subpatterns_for_attribute_0"
# subject = "cpython.test_patma.TestTypeErrors.test_got_multiple_subpatterns_for_attribute_0"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestTypeErrors::test_got_multiple_subpatterns_for_attribute_0
"""Auto-ported test: TestTypeErrors::test_got_multiple_subpatterns_for_attribute_0 (CPython 3.12 oracle)."""


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
    __match_args__ = ('a', 'a')
    a = None
x = Class()
w = y = z = None
try:
    match x:
        case Class(y, z):
            w = 0
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert w is None

assert y is None

assert z is None
print("TestTypeErrors::test_got_multiple_subpatterns_for_attribute_0: ok")
