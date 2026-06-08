# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_inheritance__test_multiple_inheritance_mapping"
# subject = "cpython.test_patma.TestInheritance.test_multiple_inheritance_mapping"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestInheritance::test_multiple_inheritance_mapping
"""Auto-ported test: TestInheritance::test_multiple_inheritance_mapping (CPython 3.12 oracle)."""


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

class M1(collections.UserDict, collections.abc.Sequence):
    pass

class M2(C, collections.UserDict, collections.abc.Sequence):
    pass

class M3(collections.UserDict, C, list):
    pass

class M4(dict, collections.abc.Sequence, C):
    pass

assert check_sequence_then_mapping(M1()) == 'map'

assert check_sequence_then_mapping(M2()) == 'map'

assert check_sequence_then_mapping(M3()) == 'map'

assert check_sequence_then_mapping(M4()) == 'map'

assert check_mapping_then_sequence(M1()) == 'map'

assert check_mapping_then_sequence(M2()) == 'map'

assert check_mapping_then_sequence(M3()) == 'map'

assert check_mapping_then_sequence(M4()) == 'map'
print("TestInheritance::test_multiple_inheritance_mapping: ok")
