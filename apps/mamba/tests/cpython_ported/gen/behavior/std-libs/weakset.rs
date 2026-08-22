use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/weakset/test_weak_set__test_constructor_identity.py`.
#[test]
fn test_gen_behavior_std_libs_weakset_test_weak_set__test_constructor_identity() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakset"
# dimension = "behavior"
# case = "test_weak_set__test_constructor_identity"
# subject = "cpython.test_weakset.TestWeakSet.test_constructor_identity"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakset.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakset.py::TestWeakSet::test_constructor_identity
"""Auto-ported test: TestWeakSet::test_constructor_identity (CPython 3.12 oracle)."""


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
s = WeakSet(self_items)
t = WeakSet(s)

assert id(s) != id(t)
print("TestWeakSet::test_constructor_identity: ok")
"###);
    assert_output(&out, r###"TestWeakSet::test_constructor_identity: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakset/test_weak_set__test_iand.py`.
#[test]
fn test_gen_behavior_std_libs_weakset_test_weak_set__test_iand() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakset"
# dimension = "behavior"
# case = "test_weak_set__test_iand"
# subject = "cpython.test_weakset.TestWeakSet.test_iand"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakset.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakset.py::TestWeakSet::test_iand
"""Auto-ported test: TestWeakSet::test_iand (CPython 3.12 oracle)."""


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
self_s &= set(self_items2)
for c in self_items + self_items2:
    if c in self_items2 and c in self_items:

        assert c in self_s
    else:

        assert c not in self_s
print("TestWeakSet::test_iand: ok")
"###);
    assert_output(&out, r###"TestWeakSet::test_iand: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakset/test_weak_set__test_methods.py`.
#[test]
fn test_gen_behavior_std_libs_weakset_test_weak_set__test_methods() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakset"
# dimension = "behavior"
# case = "test_weak_set__test_methods"
# subject = "cpython.test_weakset.TestWeakSet.test_methods"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakset.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakset.py::TestWeakSet::test_methods
"""Auto-ported test: TestWeakSet::test_methods (CPython 3.12 oracle)."""


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
weaksetmethods = dir(WeakSet)
for method in dir(set):
    if method == 'test_c_api' or method.startswith('_'):
        continue

    assert method in weaksetmethods
print("TestWeakSet::test_methods: ok")
"###);
    assert_output(&out, r###"TestWeakSet::test_methods: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/weakset/test_weak_set__test_subclass_with_custom_hash.py`.
#[test]
fn test_gen_behavior_std_libs_weakset_test_weak_set__test_subclass_with_custom_hash() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakset"
# dimension = "behavior"
# case = "test_weak_set__test_subclass_with_custom_hash"
# subject = "cpython.test_weakset.TestWeakSet.test_subclass_with_custom_hash"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakset.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_weakset.py::TestWeakSet::test_subclass_with_custom_hash
"""Auto-ported test: TestWeakSet::test_subclass_with_custom_hash (CPython 3.12 oracle)."""


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

class H(WeakSet):

    def __hash__(self):
        return int(id(self) & 2147483647)
s = H()
f = set()
f.add(s)

assert s in f
f.remove(s)
f.add(s)
f.discard(s)
print("TestWeakSet::test_subclass_with_custom_hash: ok")
"###);
    assert_output(&out, r###"TestWeakSet::test_subclass_with_custom_hash: ok
"###);
}
