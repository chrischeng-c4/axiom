use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_bool.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_bool() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_bool"
# subject = "cpython.test_dict.DictTest.test_bool"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_bool
"""Auto-ported test: DictTest::test_bool (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---

assert (not {}) is True

assert {1: 2}

assert bool({}) is False

assert bool({1: 2}) is True
print("DictTest::test_bool: ok")
"###);
    assert_output(&out, r###"DictTest::test_bool: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_constructor.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_constructor() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_constructor"
# subject = "cpython.test_dict.DictTest.test_constructor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_constructor
"""Auto-ported test: DictTest::test_constructor (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---

assert dict() == {}

assert dict() is not {}
print("DictTest::test_constructor: ok")
"###);
    assert_output(&out, r###"DictTest::test_constructor: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_copy_fuzz.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_copy_fuzz() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_copy_fuzz"
# subject = "cpython.test_dict.DictTest.test_copy_fuzz"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_copy_fuzz
"""Auto-ported test: DictTest::test_copy_fuzz (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---
def _not_tracked(t):
    gc.collect()
    gc.collect()

    assert not gc.is_tracked(t)

def _tracked(t):

    assert gc.is_tracked(t)
    gc.collect()
    gc.collect()

    assert gc.is_tracked(t)

def check_reentrant_insertion(mutate):

    class Mutating:

        def __del__(self):
            mutate(d)
    d = {k: Mutating() for k in 'abcdefghijklmnopqr'}
    for k in list(d):
        d[k] = k

def helper_keys_contained(fn):
    empty = fn(dict())
    empty2 = fn(dict())
    smaller = fn({1: 1, 2: 2})
    larger = fn({1: 1, 2: 2, 3: 3})
    larger2 = fn({1: 1, 2: 2, 3: 3})
    larger3 = fn({4: 1, 2: 2, 3: 3})

    assert smaller < larger

    assert smaller <= larger

    assert larger > smaller

    assert larger >= smaller

    assert not smaller >= larger

    assert not smaller > larger

    assert not larger <= smaller

    assert not larger < smaller

    assert not smaller < larger3

    assert not smaller <= larger3

    assert not larger3 > smaller

    assert not larger3 >= smaller

    assert larger2 >= larger

    assert larger2 <= larger

    assert not larger2 > larger

    assert not larger2 < larger

    assert larger == larger2

    assert smaller != larger

    assert empty == empty2

    assert not empty != empty2

    assert not empty == smaller

    assert empty != smaller

    assert larger != larger3

    assert not larger == larger3

def make_shared_key_dict(n):

    class C:
        pass
    dicts = []
    for i in range(n):
        a = C()
        a.x, a.y, a.z = (1, 2, 3)
        dicts.append(a.__dict__)
    return dicts
for dict_size in [10, 100, 1000, 10000, 100000]:
    dict_size = random.randrange(dict_size // 2, dict_size + dict_size // 2)
    d = {}
    for i in range(dict_size):
        d[i] = i
    d2 = d.copy()

    assert d2 is not d

    assert d == d2
    d2['key'] = 'value'

    assert d != d2

    assert len(d2) == len(d) + 1
print("DictTest::test_copy_fuzz: ok")
"###);
    assert_output(&out, r###"DictTest::test_copy_fuzz: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_copy_maintains_tracking.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_copy_maintains_tracking() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_copy_maintains_tracking"
# subject = "cpython.test_dict.DictTest.test_copy_maintains_tracking"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_copy_maintains_tracking
"""Auto-ported test: DictTest::test_copy_maintains_tracking (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---
class A:
    pass
key = A()
for d in ({}, {'a': 1}, {key: 'val'}):
    d2 = d.copy()

    assert gc.is_tracked(d) == gc.is_tracked(d2)
print("DictTest::test_copy_maintains_tracking: ok")
"###);
    assert_output(&out, r###"DictTest::test_copy_maintains_tracking: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_copy_noncompact.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_copy_noncompact() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_copy_noncompact"
# subject = "cpython.test_dict.DictTest.test_copy_noncompact"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_copy_noncompact
"""Auto-ported test: DictTest::test_copy_noncompact (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---
d = {k: k for k in range(1000)}
for k in range(950):
    del d[k]
d2 = d.copy()

assert d2 == d
print("DictTest::test_copy_noncompact: ok")
"###);
    assert_output(&out, r###"DictTest::test_copy_noncompact: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_dict_contain_use_after_free.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_dict_contain_use_after_free() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_dict_contain_use_after_free"
# subject = "cpython.test_dict.DictTest.test_dict_contain_use_after_free"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_dict_contain_use_after_free
"""Auto-ported test: DictTest::test_dict_contain_use_after_free (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---
class S(str):

    def __eq__(self, other):
        d.clear()
        return NotImplemented

    def __hash__(self):
        return hash('test')
d = {S(): 'value'}

assert not 'test' in d
print("DictTest::test_dict_contain_use_after_free: ok")
"###);
    assert_output(&out, r###"DictTest::test_dict_contain_use_after_free: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_dict_items_result_gc.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_dict_items_result_gc() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_dict_items_result_gc"
# subject = "cpython.test_dict.DictTest.test_dict_items_result_gc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_dict_items_result_gc
"""Auto-ported test: DictTest::test_dict_items_result_gc (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---
it = iter({None: []}.items())
gc.collect()

assert gc.is_tracked(next(it))
print("DictTest::test_dict_items_result_gc: ok")
"###);
    assert_output(&out, r###"DictTest::test_dict_items_result_gc: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_dict_items_result_gc_reversed.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_dict_items_result_gc_reversed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_dict_items_result_gc_reversed"
# subject = "cpython.test_dict.DictTest.test_dict_items_result_gc_reversed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_dict_items_result_gc_reversed
"""Auto-ported test: DictTest::test_dict_items_result_gc_reversed (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---
it = reversed({None: []}.items())
gc.collect()

assert gc.is_tracked(next(it))
print("DictTest::test_dict_items_result_gc_reversed: ok")
"###);
    assert_output(&out, r###"DictTest::test_dict_items_result_gc_reversed: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_dictitems_contains_use_after_free.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_dictitems_contains_use_after_free() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_dictitems_contains_use_after_free"
# subject = "cpython.test_dict.DictTest.test_dictitems_contains_use_after_free"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_dictitems_contains_use_after_free
"""Auto-ported test: DictTest::test_dictitems_contains_use_after_free (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---
class X:

    def __eq__(self, other):
        d.clear()
        return NotImplemented
d = {0: set()}
(0, X()) in d.items()
print("DictTest::test_dictitems_contains_use_after_free: ok")
"###);
    assert_output(&out, r###"DictTest::test_dictitems_contains_use_after_free: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_empty_presized_dict_in_freelist.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_empty_presized_dict_in_freelist() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_empty_presized_dict_in_freelist"
# subject = "cpython.test_dict.DictTest.test_empty_presized_dict_in_freelist"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_empty_presized_dict_in_freelist
"""Auto-ported test: DictTest::test_empty_presized_dict_in_freelist (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---
try:
    d = {'a': 1 // 0, 'b': None, 'c': None, 'd': None, 'e': None, 'f': None, 'g': None, 'h': None}
    raise AssertionError('expected ZeroDivisionError')
except ZeroDivisionError:
    pass
d = {}
print("DictTest::test_empty_presized_dict_in_freelist: ok")
"###);
    assert_output(&out, r###"DictTest::test_empty_presized_dict_in_freelist: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_free_after_iterating.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_free_after_iterating() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_free_after_iterating"
# subject = "cpython.test_dict.DictTest.test_free_after_iterating"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
"""Auto-ported test: DictTest::test_free_after_iterating (CPython 3.12 oracle)."""

import unittest
from test import support


case = unittest.TestCase()
support.check_free_after_iterating(case, iter, dict)
support.check_free_after_iterating(case, lambda d: iter(d.keys()), dict)
support.check_free_after_iterating(case, lambda d: iter(d.values()), dict)
support.check_free_after_iterating(case, lambda d: iter(d.items()), dict)

print("DictTest::test_free_after_iterating: ok")
"###);
    assert_output(&out, r###"DictTest::test_free_after_iterating: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_fromkeys_operator_modifying_dict_operand.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_fromkeys_operator_modifying_dict_operand() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_fromkeys_operator_modifying_dict_operand"
# subject = "cpython.test_dict.DictTest.test_fromkeys_operator_modifying_dict_operand"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_fromkeys_operator_modifying_dict_operand
"""Auto-ported test: DictTest::test_fromkeys_operator_modifying_dict_operand (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---
class X(int):

    def __hash__(self):
        return 13

    def __eq__(self, other):
        if len(d) > 1:
            d.clear()
        return False
d = {}
d = {X(1): 1, X(2): 2}
try:
    dict.fromkeys(d)
except RuntimeError:
    pass
print("DictTest::test_fromkeys_operator_modifying_dict_operand: ok")
"###);
    assert_output(&out, r###"DictTest::test_fromkeys_operator_modifying_dict_operand: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_fromkeys_operator_modifying_set_operand.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_fromkeys_operator_modifying_set_operand() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_fromkeys_operator_modifying_set_operand"
# subject = "cpython.test_dict.DictTest.test_fromkeys_operator_modifying_set_operand"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_fromkeys_operator_modifying_set_operand
"""Auto-ported test: DictTest::test_fromkeys_operator_modifying_set_operand (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---
class X(int):

    def __hash__(self):
        return 13

    def __eq__(self, other):
        if len(d) > 1:
            d.clear()
        return False
d = {}
d = {X(1), X(2)}
try:
    dict.fromkeys(d)
except RuntimeError:
    pass
print("DictTest::test_fromkeys_operator_modifying_set_operand: ok")
"###);
    assert_output(&out, r###"DictTest::test_fromkeys_operator_modifying_set_operand: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_init_use_after_free.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_init_use_after_free() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_init_use_after_free"
# subject = "cpython.test_dict.DictTest.test_init_use_after_free"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_init_use_after_free
"""Auto-ported test: DictTest::test_init_use_after_free (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---
class X:

    def __hash__(self):
        pair[:] = []
        return 13
pair = [X(), 123]
dict([pair])
print("DictTest::test_init_use_after_free: ok")
"###);
    assert_output(&out, r###"DictTest::test_init_use_after_free: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_iterator_pickling.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_iterator_pickling() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_iterator_pickling"
# subject = "cpython.test_dict.DictTest.test_iterator_pickling"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_iterator_pickling
"""Auto-ported test: DictTest::test_iterator_pickling (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    data = {1: 'a', 2: 'b', 3: 'c'}
    it = iter(data)
    d = pickle.dumps(it, proto)
    it = pickle.loads(d)

    assert list(it) == list(data)
    it = pickle.loads(d)
    try:
        drop = next(it)
    except StopIteration:
        continue
    d = pickle.dumps(it, proto)
    it = pickle.loads(d)
    del data[drop]

    assert list(it) == list(data)
print("DictTest::test_iterator_pickling: ok")
"###);
    assert_output(&out, r###"DictTest::test_iterator_pickling: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_len.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_len() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_len"
# subject = "cpython.test_dict.DictTest.test_len"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_len
"""Auto-ported test: DictTest::test_len (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---
d = {}

assert len(d) == 0
d = {'a': 1, 'b': 2}

assert len(d) == 2
print("DictTest::test_len: ok")
"###);
    assert_output(&out, r###"DictTest::test_len: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_popitem.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_popitem() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_popitem"
# subject = "cpython.test_dict.DictTest.test_popitem"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_popitem
"""Auto-ported test: DictTest::test_popitem (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---
for copymode in (-1, +1):
    for log2size in range(12):
        size = 2 ** log2size
        a = {}
        b = {}
        for i in range(size):
            a[repr(i)] = i
            if copymode < 0:
                b[repr(i)] = i
        if copymode > 0:
            b = a.copy()
        for i in range(size):
            ka, va = ta = a.popitem()

            assert va == int(ka)
            kb, vb = tb = b.popitem()

            assert vb == int(kb)

            assert not (copymode < 0 and ta != tb)

        assert not a

        assert not b
d = {}

try:
    d.popitem()
    raise AssertionError('expected KeyError')
except KeyError:
    pass
print("DictTest::test_popitem: ok")
"###);
    assert_output(&out, r###"DictTest::test_popitem: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_reentrant_insertion.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_reentrant_insertion() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_reentrant_insertion"
# subject = "cpython.test_dict.DictTest.test_reentrant_insertion"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_reentrant_insertion
"""Auto-ported test: DictTest::test_reentrant_insertion (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---
def _not_tracked(t):
    gc.collect()
    gc.collect()

    assert not gc.is_tracked(t)

def _tracked(t):

    assert gc.is_tracked(t)
    gc.collect()
    gc.collect()

    assert gc.is_tracked(t)

def check_reentrant_insertion(mutate):

    class Mutating:

        def __del__(self):
            mutate(d)
    d = {k: Mutating() for k in 'abcdefghijklmnopqr'}
    for k in list(d):
        d[k] = k

def helper_keys_contained(fn):
    empty = fn(dict())
    empty2 = fn(dict())
    smaller = fn({1: 1, 2: 2})
    larger = fn({1: 1, 2: 2, 3: 3})
    larger2 = fn({1: 1, 2: 2, 3: 3})
    larger3 = fn({4: 1, 2: 2, 3: 3})

    assert smaller < larger

    assert smaller <= larger

    assert larger > smaller

    assert larger >= smaller

    assert not smaller >= larger

    assert not smaller > larger

    assert not larger <= smaller

    assert not larger < smaller

    assert not smaller < larger3

    assert not smaller <= larger3

    assert not larger3 > smaller

    assert not larger3 >= smaller

    assert larger2 >= larger

    assert larger2 <= larger

    assert not larger2 > larger

    assert not larger2 < larger

    assert larger == larger2

    assert smaller != larger

    assert empty == empty2

    assert not empty != empty2

    assert not empty == smaller

    assert empty != smaller

    assert larger != larger3

    assert not larger == larger3

def make_shared_key_dict(n):

    class C:
        pass
    dicts = []
    for i in range(n):
        a = C()
        a.x, a.y, a.z = (1, 2, 3)
        dicts.append(a.__dict__)
    return dicts

def mutate(d):
    d['b'] = 5
check_reentrant_insertion(mutate)

def mutate(d):
    d.update(self.__dict__)
    d.clear()
check_reentrant_insertion(mutate)

def mutate(d):
    while d:
        d.popitem()
check_reentrant_insertion(mutate)
print("DictTest::test_reentrant_insertion: ok")
"###);
    assert_output(&out, r###"DictTest::test_reentrant_insertion: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_resize1.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_resize1() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_resize1"
# subject = "cpython.test_dict.DictTest.test_resize1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_resize1
"""Auto-ported test: DictTest::test_resize1 (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---
d = {}
for i in range(5):
    d[i] = i
for i in range(5):
    del d[i]
for i in range(5, 9):
    d[i] = i
print("DictTest::test_resize1: ok")
"###);
    assert_output(&out, r###"DictTest::test_resize1: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_resize2.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_resize2() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_resize2"
# subject = "cpython.test_dict.DictTest.test_resize2"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_resize2
"""Auto-ported test: DictTest::test_resize2 (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---
class X(object):

    def __hash__(self):
        return 5

    def __eq__(self, other):
        if resizing:
            d.clear()
        return False
d = {}
resizing = False
d[X()] = 1
d[X()] = 2
d[X()] = 3
d[X()] = 4
d[X()] = 5
resizing = True
d[9] = 6
print("DictTest::test_resize2: ok")
"###);
    assert_output(&out, r###"DictTest::test_resize2: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_reverse_iterator_for_empty_dict.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_reverse_iterator_for_empty_dict() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_reverse_iterator_for_empty_dict"
# subject = "cpython.test_dict.DictTest.test_reverse_iterator_for_empty_dict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_reverse_iterator_for_empty_dict
"""Auto-ported test: DictTest::test_reverse_iterator_for_empty_dict (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---

assert list(reversed({})) == []

assert list(reversed({}.items())) == []

assert list(reversed({}.values())) == []

assert list(reversed({}.keys())) == []

assert list(reversed(dict())) == []

assert list(reversed(dict().items())) == []

assert list(reversed(dict().values())) == []

assert list(reversed(dict().keys())) == []
print("DictTest::test_reverse_iterator_for_empty_dict: ok")
"###);
    assert_output(&out, r###"DictTest::test_reverse_iterator_for_empty_dict: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_str_nonstr.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_str_nonstr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_str_nonstr"
# subject = "cpython.test_dict.DictTest.test_str_nonstr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
"""Auto-ported test: DictTest::test_str_nonstr (CPython 3.12 oracle)."""

import sys


def run():
    class StrSub(str):
        pass

    eq_count = 0

    class Key3:
        def __hash__(self):
            return hash("key3")

        def __eq__(self, other):
            nonlocal eq_count
            if isinstance(other, Key3) or isinstance(other, str) and other == "key3":
                eq_count += 1
                return True
            return False

    key3_1 = StrSub("key3")
    key3_2 = Key3()
    key3_3 = Key3()

    dicts = []

    for key3 in (key3_1, key3_2):
        dicts.append({"key1": 42, "key2": 43, key3: 44})

        d = {"key1": 42, "key2": 43}
        d[key3] = 44
        dicts.append(d)

        d = {"key1": 42, "key2": 43}
        assert d.setdefault(key3, 44) == 44
        dicts.append(d)

        d = {"key1": 42, "key2": 43}
        d.update({key3: 44})
        dicts.append(d)

        d = {"key1": 42, "key2": 43}
        d |= {key3: 44}
        dicts.append(d)

        def make_pairs():
            yield ("key1", 42)
            yield ("key2", 43)
            yield (key3, 44)

        d = dict(make_pairs())
        dicts.append(d)

        d = d.copy()
        dicts.append(d)

        d = {key: 42 + i for i, key in enumerate(["key1", "key2", key3])}
        dicts.append(d)

    for d in dicts:
        assert d.get("key1") == 42

        noninterned_key1 = "ke"
        noninterned_key1 += "y1"
        if sys.implementation.name == "cpython":
            interned_key1 = "key1"
            assert noninterned_key1 is not interned_key1
        assert d.get(noninterned_key1) == 42

        assert d.get("key3") == 44
        assert d.get(key3_1) == 44
        assert d.get(key3_2) == 44

        eq_count = 0
        assert d.get(key3_3) == 44
        assert eq_count >= 1


run()
print("DictTest::test_str_nonstr: ok")
"###);
    assert_output(&out, r###"DictTest::test_str_nonstr: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_string_keys_can_track_values.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_string_keys_can_track_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_string_keys_can_track_values"
# subject = "cpython.test_dict.DictTest.test_string_keys_can_track_values"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_string_keys_can_track_values
"""Auto-ported test: DictTest::test_string_keys_can_track_values (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---
for i in range(10):
    d = {}
    for j in range(10):
        d[str(j)] = j
    d['foo'] = d
print("DictTest::test_string_keys_can_track_values: ok")
"###);
    assert_output(&out, r###"DictTest::test_string_keys_can_track_values: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/dict_methods/dict_test__test_track_subtypes.py`.
#[test]
fn test_gen_behavior_builtin_libs_dict_methods_dict_test__test_track_subtypes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_track_subtypes"
# subject = "cpython.test_dict.DictTest.test_track_subtypes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_track_subtypes
"""Auto-ported test: DictTest::test_track_subtypes (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---
def _not_tracked(t):
    gc.collect()
    gc.collect()

    assert not gc.is_tracked(t)

def _tracked(t):

    assert gc.is_tracked(t)
    gc.collect()
    gc.collect()

    assert gc.is_tracked(t)

def check_reentrant_insertion(mutate):

    class Mutating:

        def __del__(self):
            mutate(d)
    d = {k: Mutating() for k in 'abcdefghijklmnopqr'}
    for k in list(d):
        d[k] = k

def helper_keys_contained(fn):
    empty = fn(dict())
    empty2 = fn(dict())
    smaller = fn({1: 1, 2: 2})
    larger = fn({1: 1, 2: 2, 3: 3})
    larger2 = fn({1: 1, 2: 2, 3: 3})
    larger3 = fn({4: 1, 2: 2, 3: 3})

    assert smaller < larger

    assert smaller <= larger

    assert larger > smaller

    assert larger >= smaller

    assert not smaller >= larger

    assert not smaller > larger

    assert not larger <= smaller

    assert not larger < smaller

    assert not smaller < larger3

    assert not smaller <= larger3

    assert not larger3 > smaller

    assert not larger3 >= smaller

    assert larger2 >= larger

    assert larger2 <= larger

    assert not larger2 > larger

    assert not larger2 < larger

    assert larger == larger2

    assert smaller != larger

    assert empty == empty2

    assert not empty != empty2

    assert not empty == smaller

    assert empty != smaller

    assert larger != larger3

    assert not larger == larger3

def make_shared_key_dict(n):

    class C:
        pass
    dicts = []
    for i in range(n):
        a = C()
        a.x, a.y, a.z = (1, 2, 3)
        dicts.append(a.__dict__)
    return dicts

class MyDict(dict):
    pass
_tracked(MyDict())
print("DictTest::test_track_subtypes: ok")
"###);
    assert_output(&out, r###"DictTest::test_track_subtypes: ok
"###);
}
