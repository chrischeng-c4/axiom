use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/scope/scope_tests__test_cell_is_kwonly_arg.py`.
#[test]
fn test_gen_behavior_std_libs_scope_scope_tests__test_cell_is_kwonly_arg() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_cell_is_kwonly_arg"
# subject = "cpython.test_scope.ScopeTests.testCellIsKwonlyArg"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testCellIsKwonlyArg
"""Auto-ported test: ScopeTests::testCellIsKwonlyArg (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def foo(*, a=17):

    def bar():
        return a + 5
    return bar() + 3

assert foo(a=42) == 50

assert foo() == 25
print("ScopeTests::testCellIsKwonlyArg: ok")
"###);
    assert_output(&out, r###"ScopeTests::testCellIsKwonlyArg: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/scope/scope_tests__test_freeing_cell.py`.
#[test]
fn test_gen_behavior_std_libs_scope_scope_tests__test_freeing_cell() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_freeing_cell"
# subject = "cpython.test_scope.ScopeTests.testFreeingCell"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testFreeingCell
"""Auto-ported test: ScopeTests::testFreeingCell (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
class Special:

    def __del__(self):
        nestedcell_get()
print("ScopeTests::testFreeingCell: ok")
"###);
    assert_output(&out, r###"ScopeTests::testFreeingCell: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/scope/scope_tests__test_locals_class_with_trace.py`.
#[test]
fn test_gen_behavior_std_libs_scope_scope_tests__test_locals_class_with_trace() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_locals_class_with_trace"
# subject = "cpython.test_scope.ScopeTests.testLocalsClass_WithTrace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testLocalsClass_WithTrace
"""Auto-ported test: ScopeTests::testLocalsClass_WithTrace (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
import sys
pass
sys.settrace(lambda a, b, c: None)
x = 12

class C:

    def f(self):
        return x

assert x == 12
print("ScopeTests::testLocalsClass_WithTrace: ok")
"###);
    assert_output(&out, r###"ScopeTests::testLocalsClass_WithTrace: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/scope/scope_tests__test_nearest_enclosing_scope.py`.
#[test]
fn test_gen_behavior_std_libs_scope_scope_tests__test_nearest_enclosing_scope() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_nearest_enclosing_scope"
# subject = "cpython.test_scope.ScopeTests.testNearestEnclosingScope"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testNearestEnclosingScope
"""Auto-ported test: ScopeTests::testNearestEnclosingScope (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def f(x):

    def g(y):
        x = 42

        def h(z):
            return x + z
        return h
    return g(2)
test_func = f(10)

assert test_func(5) == 47
print("ScopeTests::testNearestEnclosingScope: ok")
"###);
    assert_output(&out, r###"ScopeTests::testNearestEnclosingScope: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/scope/scope_tests__test_nested_non_local.py`.
#[test]
fn test_gen_behavior_std_libs_scope_scope_tests__test_nested_non_local() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_nested_non_local"
# subject = "cpython.test_scope.ScopeTests.testNestedNonLocal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testNestedNonLocal
"""Auto-ported test: ScopeTests::testNestedNonLocal (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def f(x):

    def g():
        nonlocal x
        x -= 2

        def h():
            nonlocal x
            x += 4
            return x
        return h
    return g
g = f(1)
h = g()

assert h() == 3
print("ScopeTests::testNestedNonLocal: ok")
"###);
    assert_output(&out, r###"ScopeTests::testNestedNonLocal: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/scope/scope_tests__test_nesting_global_no_free.py`.
#[test]
fn test_gen_behavior_std_libs_scope_scope_tests__test_nesting_global_no_free() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_nesting_global_no_free"
# subject = "cpython.test_scope.ScopeTests.testNestingGlobalNoFree"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testNestingGlobalNoFree
"""Auto-ported test: ScopeTests::testNestingGlobalNoFree (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def make_adder4():

    def nest():

        def nest():

            def adder(y):
                return global_x + y
            return adder
        return nest()
    return nest()
global_x = 1
adder = make_adder4()

assert adder(1) == 2
global_x = 10

assert adder(-2) == 8
print("ScopeTests::testNestingGlobalNoFree: ok")
"###);
    assert_output(&out, r###"ScopeTests::testNestingGlobalNoFree: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/scope/scope_tests__test_non_local_function.py`.
#[test]
fn test_gen_behavior_std_libs_scope_scope_tests__test_non_local_function() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_non_local_function"
# subject = "cpython.test_scope.ScopeTests.testNonLocalFunction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testNonLocalFunction
"""Auto-ported test: ScopeTests::testNonLocalFunction (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def f(x):

    def inc():
        nonlocal x
        x += 1
        return x

    def dec():
        nonlocal x
        x -= 1
        return x
    return (inc, dec)
inc, dec = f(0)

assert inc() == 1

assert inc() == 2

assert dec() == 1

assert dec() == 0
print("ScopeTests::testNonLocalFunction: ok")
"###);
    assert_output(&out, r###"ScopeTests::testNonLocalFunction: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/scope/scope_tests__test_top_is_not_significant.py`.
#[test]
fn test_gen_behavior_std_libs_scope_scope_tests__test_top_is_not_significant() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_top_is_not_significant"
# subject = "cpython.test_scope.ScopeTests.testTopIsNotSignificant"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testTopIsNotSignificant
"""Auto-ported test: ScopeTests::testTopIsNotSignificant (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def top(a):
    pass

def b():
    global a
print("ScopeTests::testTopIsNotSignificant: ok")
"###);
    assert_output(&out, r###"ScopeTests::testTopIsNotSignificant: ok
"###);
}
