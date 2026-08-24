use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/positional_only_arg/positional_only_test_case__test_pos_only_call_via_unpacking.py`.
#[test]
fn test_gen_behavior_std_libs_positional_only_arg_positional_only_test_case__test_pos_only_call_via_unpacking() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "positional_only_arg"
# dimension = "behavior"
# case = "positional_only_test_case__test_pos_only_call_via_unpacking"
# subject = "cpython.test_positional_only_arg.PositionalOnlyTestCase.test_pos_only_call_via_unpacking"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_positional_only_arg.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_positional_only_arg.py::PositionalOnlyTestCase::test_pos_only_call_via_unpacking
"""Auto-ported test: PositionalOnlyTestCase::test_pos_only_call_via_unpacking (CPython 3.12 oracle)."""


import dis
import pickle
import unittest
from test.support import check_syntax_error


'Unit tests for the positional only argument syntax specified in PEP 570.'

def global_pos_only_f(a, b, /):
    return (a, b)

def global_pos_only_and_normal(a, /, b):
    return (a, b)

def global_pos_only_defaults(a=1, /, b=2):
    return (a, b)


# --- test body ---
def f(a, b, /):
    return a + b

assert f(*[1, 2]) == 3
print("PositionalOnlyTestCase::test_pos_only_call_via_unpacking: ok")
"###);
    assert_output(&out, r###"PositionalOnlyTestCase::test_pos_only_call_via_unpacking: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/positional_only_arg/positional_only_test_case__test_super.py`.
#[test]
fn test_gen_behavior_std_libs_positional_only_arg_positional_only_test_case__test_super() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "positional_only_arg"
# dimension = "behavior"
# case = "positional_only_test_case__test_super"
# subject = "cpython.test_positional_only_arg.PositionalOnlyTestCase.test_super"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_positional_only_arg.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_positional_only_arg.py::PositionalOnlyTestCase::test_super
"""Auto-ported test: PositionalOnlyTestCase::test_super (CPython 3.12 oracle)."""


import dis
import pickle
import unittest
from test.support import check_syntax_error


'Unit tests for the positional only argument syntax specified in PEP 570.'

def global_pos_only_f(a, b, /):
    return (a, b)

def global_pos_only_and_normal(a, /, b):
    return (a, b)

def global_pos_only_defaults(a=1, /, b=2):
    return (a, b)


# --- test body ---
sentinel = object()

class A:

    def method(self):
        return sentinel

class C(A):

    def method(self, /):
        return super().method()

assert C().method() == sentinel
print("PositionalOnlyTestCase::test_super: ok")
"###);
    assert_output(&out, r###"PositionalOnlyTestCase::test_super: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/positional_only_arg/positional_only_test_case__test_syntax_for_many_positional_only.py`.
#[test]
fn test_gen_behavior_std_libs_positional_only_arg_positional_only_test_case__test_syntax_for_many_positional_only() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "positional_only_arg"
# dimension = "behavior"
# case = "positional_only_test_case__test_syntax_for_many_positional_only"
# subject = "cpython.test_positional_only_arg.PositionalOnlyTestCase.test_syntax_for_many_positional_only"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_positional_only_arg.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_positional_only_arg.py::PositionalOnlyTestCase::test_syntax_for_many_positional_only
"""Auto-ported test: PositionalOnlyTestCase::test_syntax_for_many_positional_only (CPython 3.12 oracle)."""


import dis
import pickle
import unittest
from test.support import check_syntax_error


'Unit tests for the positional only argument syntax specified in PEP 570.'

def global_pos_only_f(a, b, /):
    return (a, b)

def global_pos_only_and_normal(a, /, b):
    return (a, b)

def global_pos_only_defaults(a=1, /, b=2):
    return (a, b)


# --- test body ---
fundef = 'def f(%s, /):\n  pass\n' % ', '.join(('i%d' % i for i in range(300)))
compile(fundef, '<test>', 'single')
print("PositionalOnlyTestCase::test_syntax_for_many_positional_only: ok")
"###);
    assert_output(&out, r###"PositionalOnlyTestCase::test_syntax_for_many_positional_only: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/positional_only_arg/positional_only_test_case__test_too_many_arguments.py`.
#[test]
fn test_gen_behavior_std_libs_positional_only_arg_positional_only_test_case__test_too_many_arguments() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "positional_only_arg"
# dimension = "behavior"
# case = "positional_only_test_case__test_too_many_arguments"
# subject = "cpython.test_positional_only_arg.PositionalOnlyTestCase.test_too_many_arguments"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_positional_only_arg.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_positional_only_arg.py::PositionalOnlyTestCase::test_too_many_arguments
"""Auto-ported test: PositionalOnlyTestCase::test_too_many_arguments (CPython 3.12 oracle)."""


import dis
import pickle
import unittest
from test.support import check_syntax_error


'Unit tests for the positional only argument syntax specified in PEP 570.'

def global_pos_only_f(a, b, /):
    return (a, b)

def global_pos_only_and_normal(a, /, b):
    return (a, b)

def global_pos_only_defaults(a=1, /, b=2):
    return (a, b)


# --- test body ---
fundef = 'def f(%s, /):\n  pass\n' % ', '.join(('i%d' % i for i in range(300)))
compile(fundef, '<test>', 'single')
print("PositionalOnlyTestCase::test_too_many_arguments: ok")
"###);
    assert_output(&out, r###"PositionalOnlyTestCase::test_too_many_arguments: ok
"###);
}
