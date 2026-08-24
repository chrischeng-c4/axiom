use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/core/dictcomps/dict_comprehension_test__test_basics.py`.
#[test]
fn test_gen_behavior_core_dictcomps_dict_comprehension_test__test_basics() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "dictcomps"
# dimension = "behavior"
# case = "dict_comprehension_test__test_basics"
# subject = "cpython.test_dictcomps.DictComprehensionTest.test_basics"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dictcomps.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dictcomps.py::DictComprehensionTest::test_basics
"""Auto-ported test: DictComprehensionTest::test_basics (CPython 3.12 oracle)."""


import traceback
import unittest
from test.support import BrokenIter


g = 'Global variable'


# --- test body ---
expected = {0: 10, 1: 11, 2: 12, 3: 13, 4: 14, 5: 15, 6: 16, 7: 17, 8: 18, 9: 19}
actual = {k: k + 10 for k in range(10)}

assert actual == expected
expected = {0: 0, 1: 1, 2: 2, 3: 3, 4: 4, 5: 5, 6: 6, 7: 7, 8: 8, 9: 9}
actual = {k: v for k in range(10) for v in range(10) if k == v}

assert actual == expected
print("DictComprehensionTest::test_basics: ok")
"###);
    assert_output(&out, r###"DictComprehensionTest::test_basics: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/dictcomps/dict_comprehension_test__test_evaluation_order.py`.
#[test]
fn test_gen_behavior_core_dictcomps_dict_comprehension_test__test_evaluation_order() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "dictcomps"
# dimension = "behavior"
# case = "dict_comprehension_test__test_evaluation_order"
# subject = "cpython.test_dictcomps.DictComprehensionTest.test_evaluation_order"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dictcomps.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dictcomps.py::DictComprehensionTest::test_evaluation_order
"""Auto-ported test: DictComprehensionTest::test_evaluation_order (CPython 3.12 oracle)."""


import traceback
import unittest
from test.support import BrokenIter


g = 'Global variable'


# --- test body ---
expected = {'H': 'W', 'e': 'o', 'l': 'l', 'o': 'd'}
expected_calls = [('key', 'H'), ('value', 'W'), ('key', 'e'), ('value', 'o'), ('key', 'l'), ('value', 'r'), ('key', 'l'), ('value', 'l'), ('key', 'o'), ('value', 'd')]
actual_calls = []

def add_call(pos, value):
    actual_calls.append((pos, value))
    return value
actual = {add_call('key', k): add_call('value', v) for k, v in zip('Hello', 'World')}

assert actual == expected

assert actual_calls == expected_calls
print("DictComprehensionTest::test_evaluation_order: ok")
"###);
    assert_output(&out, r###"DictComprehensionTest::test_evaluation_order: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/dictcomps/dict_comprehension_test__test_global_visibility.py`.
#[test]
fn test_gen_behavior_core_dictcomps_dict_comprehension_test__test_global_visibility() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "dictcomps"
# dimension = "behavior"
# case = "dict_comprehension_test__test_global_visibility"
# subject = "cpython.test_dictcomps.DictComprehensionTest.test_global_visibility"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dictcomps.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dictcomps.py::DictComprehensionTest::test_global_visibility
"""Auto-ported test: DictComprehensionTest::test_global_visibility (CPython 3.12 oracle)."""


import traceback
import unittest
from test.support import BrokenIter


g = 'Global variable'


# --- test body ---
expected = {0: 'Global variable', 1: 'Global variable', 2: 'Global variable', 3: 'Global variable', 4: 'Global variable', 5: 'Global variable', 6: 'Global variable', 7: 'Global variable', 8: 'Global variable', 9: 'Global variable'}
actual = {k: g for k in range(10)}

assert actual == expected
print("DictComprehensionTest::test_global_visibility: ok")
"###);
    assert_output(&out, r###"DictComprehensionTest::test_global_visibility: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/dictcomps/dict_comprehension_test__test_local_visibility.py`.
#[test]
fn test_gen_behavior_core_dictcomps_dict_comprehension_test__test_local_visibility() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "dictcomps"
# dimension = "behavior"
# case = "dict_comprehension_test__test_local_visibility"
# subject = "cpython.test_dictcomps.DictComprehensionTest.test_local_visibility"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dictcomps.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dictcomps.py::DictComprehensionTest::test_local_visibility
"""Auto-ported test: DictComprehensionTest::test_local_visibility (CPython 3.12 oracle)."""


import traceback
import unittest
from test.support import BrokenIter


g = 'Global variable'


# --- test body ---
v = 'Local variable'
expected = {0: 'Local variable', 1: 'Local variable', 2: 'Local variable', 3: 'Local variable', 4: 'Local variable', 5: 'Local variable', 6: 'Local variable', 7: 'Local variable', 8: 'Local variable', 9: 'Local variable'}
actual = {k: v for k in range(10)}

assert actual == expected

assert v == 'Local variable'
print("DictComprehensionTest::test_local_visibility: ok")
"###);
    assert_output(&out, r###"DictComprehensionTest::test_local_visibility: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/dictcomps/dict_comprehension_test__test_scope_isolation.py`.
#[test]
fn test_gen_behavior_core_dictcomps_dict_comprehension_test__test_scope_isolation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "dictcomps"
# dimension = "behavior"
# case = "dict_comprehension_test__test_scope_isolation"
# subject = "cpython.test_dictcomps.DictComprehensionTest.test_scope_isolation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dictcomps.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dictcomps.py::DictComprehensionTest::test_scope_isolation
"""Auto-ported test: DictComprehensionTest::test_scope_isolation (CPython 3.12 oracle)."""


import traceback
import unittest
from test.support import BrokenIter


g = 'Global variable'


# --- test body ---
k = 'Local Variable'
expected = {0: None, 1: None, 2: None, 3: None, 4: None, 5: None, 6: None, 7: None, 8: None, 9: None}
actual = {k: None for k in range(10)}

assert actual == expected

assert k == 'Local Variable'
expected = {9: 1, 18: 2, 19: 2, 27: 3, 28: 3, 29: 3, 36: 4, 37: 4, 38: 4, 39: 4, 45: 5, 46: 5, 47: 5, 48: 5, 49: 5, 54: 6, 55: 6, 56: 6, 57: 6, 58: 6, 59: 6, 63: 7, 64: 7, 65: 7, 66: 7, 67: 7, 68: 7, 69: 7, 72: 8, 73: 8, 74: 8, 75: 8, 76: 8, 77: 8, 78: 8, 79: 8, 81: 9, 82: 9, 83: 9, 84: 9, 85: 9, 86: 9, 87: 9, 88: 9, 89: 9}
actual = {k: v for v in range(10) for k in range(v * 9, v * 10)}

assert k == 'Local Variable'

assert actual == expected
print("DictComprehensionTest::test_scope_isolation: ok")
"###);
    assert_output(&out, r###"DictComprehensionTest::test_scope_isolation: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/dictcomps/dict_comprehension_test__test_scope_isolation_from_global.py`.
#[test]
fn test_gen_behavior_core_dictcomps_dict_comprehension_test__test_scope_isolation_from_global() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "dictcomps"
# dimension = "behavior"
# case = "dict_comprehension_test__test_scope_isolation_from_global"
# subject = "cpython.test_dictcomps.DictComprehensionTest.test_scope_isolation_from_global"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dictcomps.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dictcomps.py::DictComprehensionTest::test_scope_isolation_from_global
"""Auto-ported test: DictComprehensionTest::test_scope_isolation_from_global (CPython 3.12 oracle)."""


import traceback
import unittest
from test.support import BrokenIter


g = 'Global variable'


# --- test body ---
expected = {0: None, 1: None, 2: None, 3: None, 4: None, 5: None, 6: None, 7: None, 8: None, 9: None}
actual = {g: None for g in range(10)}

assert actual == expected

assert g == 'Global variable'
expected = {9: 1, 18: 2, 19: 2, 27: 3, 28: 3, 29: 3, 36: 4, 37: 4, 38: 4, 39: 4, 45: 5, 46: 5, 47: 5, 48: 5, 49: 5, 54: 6, 55: 6, 56: 6, 57: 6, 58: 6, 59: 6, 63: 7, 64: 7, 65: 7, 66: 7, 67: 7, 68: 7, 69: 7, 72: 8, 73: 8, 74: 8, 75: 8, 76: 8, 77: 8, 78: 8, 79: 8, 81: 9, 82: 9, 83: 9, 84: 9, 85: 9, 86: 9, 87: 9, 88: 9, 89: 9}
actual = {g: v for v in range(10) for g in range(v * 9, v * 10)}

assert g == 'Global variable'

assert actual == expected
print("DictComprehensionTest::test_scope_isolation_from_global: ok")
"###);
    assert_output(&out, r###"DictComprehensionTest::test_scope_isolation_from_global: ok
"###);
}
