# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_value_errors__test_mapping_pattern_checks_duplicate_key_1"
# subject = "cpython.test_patma.TestValueErrors.test_mapping_pattern_checks_duplicate_key_1"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestValueErrors::test_mapping_pattern_checks_duplicate_key_1
"""Auto-ported test: TestValueErrors::test_mapping_pattern_checks_duplicate_key_1 (CPython 3.12 oracle)."""


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
class Keys:
    KEY = 'a'
x = {'a': 0, 'b': 1}
w = y = z = None
try:
    match x:
        case {Keys.KEY: y, 'a': z}:
            w = 0
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert w is None

assert y is None

assert z is None
print("TestValueErrors::test_mapping_pattern_checks_duplicate_key_1: ok")
