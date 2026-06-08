# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_inheritance__test_multiple_inheritance_sequence"
# subject = "cpython.test_patma.TestInheritance.test_multiple_inheritance_sequence"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestInheritance::test_multiple_inheritance_sequence
"""Auto-ported test: TestInheritance::test_multiple_inheritance_sequence (CPython 3.12 oracle)."""


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
def check_mapping_then_sequence(x):
    match x:
        case {}:
            return 'map'
        case [*_]:
            return 'seq'

def check_sequence_then_mapping(x):
    match x:
        case [*_]:
            return 'seq'
        case {}:
            return 'map'

class C:
    pass

class S1(collections.UserList, collections.abc.Mapping):
    pass

class S2(C, collections.UserList, collections.abc.Mapping):
    pass

class S3(list, C, collections.abc.Mapping):
    pass

class S4(collections.UserList, dict, C):
    pass

assert check_sequence_then_mapping(S1()) == 'seq'

assert check_sequence_then_mapping(S2()) == 'seq'

assert check_sequence_then_mapping(S3()) == 'seq'

assert check_sequence_then_mapping(S4()) == 'seq'

assert check_mapping_then_sequence(S1()) == 'seq'

assert check_mapping_then_sequence(S2()) == 'seq'

assert check_mapping_then_sequence(S3()) == 'seq'

assert check_mapping_then_sequence(S4()) == 'seq'
print("TestInheritance::test_multiple_inheritance_sequence: ok")
