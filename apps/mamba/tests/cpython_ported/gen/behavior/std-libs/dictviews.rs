use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/dictviews/dict_set_test__test_dict_mixed_keys_items.py`.
#[test]
fn test_gen_behavior_std_libs_dictviews_dict_set_test__test_dict_mixed_keys_items() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dictviews"
# dimension = "behavior"
# case = "dict_set_test__test_dict_mixed_keys_items"
# subject = "cpython.test_dictviews.DictSetTest.test_dict_mixed_keys_items"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dictviews.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dictviews.py::DictSetTest::test_dict_mixed_keys_items
"""Auto-ported test: DictSetTest::test_dict_mixed_keys_items (CPython 3.12 oracle)."""


import collections.abc
import copy
import pickle
import sys
import unittest
from test.support import C_RECURSION_LIMIT


# --- test body ---
d = {(1, 1): 11, (2, 2): 22}
e = {1: 1, 2: 2}

assert d.keys() == e.items()

assert d.items() != e.keys()
print("DictSetTest::test_dict_mixed_keys_items: ok")
"###);
    assert_output(&out, r###"DictSetTest::test_dict_mixed_keys_items: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/dictviews/dict_set_test__test_dict_values.py`.
#[test]
fn test_gen_behavior_std_libs_dictviews_dict_set_test__test_dict_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dictviews"
# dimension = "behavior"
# case = "dict_set_test__test_dict_values"
# subject = "cpython.test_dictviews.DictSetTest.test_dict_values"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dictviews.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dictviews.py::DictSetTest::test_dict_values
"""Auto-ported test: DictSetTest::test_dict_values (CPython 3.12 oracle)."""


import collections.abc
import copy
import pickle
import sys
import unittest
from test.support import C_RECURSION_LIMIT


# --- test body ---
d = {1: 10, 'a': 'ABC'}
values = d.values()

assert set(values) == {10, 'ABC'}

assert len(values) == 2
print("DictSetTest::test_dict_values: ok")
"###);
    assert_output(&out, r###"DictSetTest::test_dict_values: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/dictviews/dict_set_test__test_recursive_repr.py`.
#[test]
fn test_gen_behavior_std_libs_dictviews_dict_set_test__test_recursive_repr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dictviews"
# dimension = "behavior"
# case = "dict_set_test__test_recursive_repr"
# subject = "cpython.test_dictviews.DictSetTest.test_recursive_repr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dictviews.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dictviews.py::DictSetTest::test_recursive_repr
"""Auto-ported test: DictSetTest::test_recursive_repr (CPython 3.12 oracle)."""


import collections.abc
import copy
import pickle
import sys
import unittest
from test.support import C_RECURSION_LIMIT


# --- test body ---
d = {}
d[42] = d.values()
r = repr(d)

assert isinstance(r, str)
d[42] = d.items()
r = repr(d)

assert isinstance(r, str)
print("DictSetTest::test_recursive_repr: ok")
"###);
    assert_output(&out, r###"DictSetTest::test_recursive_repr: ok
"###);
}
