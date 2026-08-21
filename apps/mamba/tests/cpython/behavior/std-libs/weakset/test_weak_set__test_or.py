# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakset"
# dimension = "behavior"
# case = "test_weak_set__test_or"
# subject = "cpython.test_weakset.TestWeakSet.test_or"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_weakset.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_weakset.py::TestWeakSet::test_or
"""Auto-ported test: TestWeakSet::test_or (CPython 3.12 oracle)."""


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
i = self_s.union(self_items2)

assert self_s | set(self_items2) == i

assert self_s | frozenset(self_items2) == i
print("TestWeakSet::test_or: ok")
