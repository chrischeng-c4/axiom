# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_inheritance__test_late_registration_sequence"
# subject = "cpython.test_patma.TestInheritance.test_late_registration_sequence"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestInheritance::test_late_registration_sequence
"""Auto-ported test: TestInheritance::test_late_registration_sequence (CPython 3.12 oracle)."""


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
collections.abc.Sequence.register(Parent)

class ChildPost(Parent):
    pass

class GrandchildPost(ChildPost):
    pass

assert check_sequence_then_mapping(Parent()) == 'seq'

assert check_sequence_then_mapping(ChildPre()) == 'seq'

assert check_sequence_then_mapping(GrandchildPre()) == 'seq'

assert check_sequence_then_mapping(ChildPost()) == 'seq'

assert check_sequence_then_mapping(GrandchildPost()) == 'seq'

assert check_mapping_then_sequence(Parent()) == 'seq'

assert check_mapping_then_sequence(ChildPre()) == 'seq'

assert check_mapping_then_sequence(GrandchildPre()) == 'seq'

assert check_mapping_then_sequence(ChildPost()) == 'seq'

assert check_mapping_then_sequence(GrandchildPost()) == 'seq'
print("TestInheritance::test_late_registration_sequence: ok")
