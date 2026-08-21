# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_inheritance__test_late_registration_mapping"
# subject = "cpython.test_patma.TestInheritance.test_late_registration_mapping"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestInheritance::test_late_registration_mapping
"""Auto-ported test: TestInheritance::test_late_registration_mapping (CPython 3.12 oracle)."""


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

class Parent:
    pass

class ChildPre(Parent):
    pass

class GrandchildPre(ChildPre):
    pass
collections.abc.Mapping.register(Parent)

class ChildPost(Parent):
    pass

class GrandchildPost(ChildPost):
    pass

assert check_sequence_then_mapping(Parent()) == 'map'

assert check_sequence_then_mapping(ChildPre()) == 'map'

assert check_sequence_then_mapping(GrandchildPre()) == 'map'

assert check_sequence_then_mapping(ChildPost()) == 'map'

assert check_sequence_then_mapping(GrandchildPost()) == 'map'

assert check_mapping_then_sequence(Parent()) == 'map'

assert check_mapping_then_sequence(ChildPre()) == 'map'

assert check_mapping_then_sequence(GrandchildPre()) == 'map'

assert check_mapping_then_sequence(ChildPost()) == 'map'

assert check_mapping_then_sequence(GrandchildPost()) == 'map'
print("TestInheritance::test_late_registration_mapping: ok")
