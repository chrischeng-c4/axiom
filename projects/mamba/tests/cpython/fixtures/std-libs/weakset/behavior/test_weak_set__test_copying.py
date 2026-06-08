# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakset"
# dimension = "behavior"
# case = "test_weak_set__test_copying"
# subject = "cpython.test_weakset.TestWeakSet.test_copying"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_weakset.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_weakset.py::TestWeakSet::test_copying
"""Auto-ported test: TestWeakSet::test_copying (CPython 3.12 oracle)."""


import unittest
from weakref import WeakSet
import copy
import string
from collections import UserString as ustr
from collections.abc import Set, MutableSet
import gc
import contextlib
from test import support


class Foo:
    pass

class RefCycle:

    def __init__(self):
        self.cycle = self

class WeakSetSubclass(WeakSet):
    pass

class WeakSetWithSlots(WeakSet):
    __slots__ = ('x', 'y')


# --- test body ---
self_items = [ustr(c) for c in ('a', 'b', 'c')]
self_items2 = [ustr(c) for c in ('x', 'y', 'z')]
self_ab_items = [ustr(c) for c in 'ab']
self_abcde_items = [ustr(c) for c in 'abcde']
self_def_items = [ustr(c) for c in 'def']
self_ab_weakset = WeakSet(self_ab_items)
self_abcde_weakset = WeakSet(self_abcde_items)
self_def_weakset = WeakSet(self_def_items)
self_letters = [ustr(c) for c in string.ascii_letters]
self_s = WeakSet(self_items)
self_d = dict.fromkeys(self_items)
self_obj = ustr('F')
self_fs = WeakSet([self_obj])
for cls in (WeakSet, WeakSetWithSlots):
    s = cls(self_items)
    s.x = ['x']
    s.z = ['z']
    dup = copy.copy(s)

    assert isinstance(dup, cls)

    assert dup == s

    assert dup is not s

    assert dup.x is s.x

    assert dup.z is s.z

    assert not hasattr(dup, 'y')
    dup = copy.deepcopy(s)

    assert isinstance(dup, cls)

    assert dup == s

    assert dup is not s

    assert dup.x == s.x

    assert dup.x is not s.x

    assert dup.z == s.z

    assert dup.z is not s.z

    assert not hasattr(dup, 'y')
print("TestWeakSet::test_copying: ok")
