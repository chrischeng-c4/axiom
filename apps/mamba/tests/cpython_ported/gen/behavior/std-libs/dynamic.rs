use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/dynamic/rebind_builtins_tests__test_cannot_replace_builtins_dict_between_calls.py`.
#[test]
fn test_gen_behavior_std_libs_dynamic_rebind_builtins_tests__test_cannot_replace_builtins_dict_between_calls() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dynamic"
# dimension = "behavior"
# case = "rebind_builtins_tests__test_cannot_replace_builtins_dict_between_calls"
# subject = "cpython.test_dynamic.RebindBuiltinsTests.test_cannot_replace_builtins_dict_between_calls"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dynamic.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dynamic.py::RebindBuiltinsTests::test_cannot_replace_builtins_dict_between_calls
"""Auto-ported test: RebindBuiltinsTests::test_cannot_replace_builtins_dict_between_calls (CPython 3.12 oracle)."""


import builtins
import sys
import unittest
from test.support import swap_item, swap_attr


# --- test body ---
def configure_func(func, *args):
    """Perform TestCase-specific configuration on a function before testing.

        By default, this does nothing. Example usage: spinning a function so
        that a JIT will optimize it. Subclasses should override this as needed.

        Args:
            func: function to configure.
            *args: any arguments that should be passed to func, if calling it.

        Returns:
            Nothing. Work will be performed on func in-place.
        """
    pass

def foo():
    return len([1, 2, 3])
configure_func(foo)

assert foo() == 3
with swap_item(globals(), '__builtins__', {'len': lambda x: 7}):

    assert foo() == 3
print("RebindBuiltinsTests::test_cannot_replace_builtins_dict_between_calls: ok")
"###);
    assert_output(&out, r###"RebindBuiltinsTests::test_cannot_replace_builtins_dict_between_calls: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/dynamic/rebind_builtins_tests__test_cannot_replace_builtins_dict_while_active.py`.
#[test]
fn test_gen_behavior_std_libs_dynamic_rebind_builtins_tests__test_cannot_replace_builtins_dict_while_active() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dynamic"
# dimension = "behavior"
# case = "rebind_builtins_tests__test_cannot_replace_builtins_dict_while_active"
# subject = "cpython.test_dynamic.RebindBuiltinsTests.test_cannot_replace_builtins_dict_while_active"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dynamic.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dynamic.py::RebindBuiltinsTests::test_cannot_replace_builtins_dict_while_active
"""Auto-ported test: RebindBuiltinsTests::test_cannot_replace_builtins_dict_while_active (CPython 3.12 oracle)."""


import builtins
import sys
import unittest
from test.support import swap_item, swap_attr


# --- test body ---
def configure_func(func, *args):
    """Perform TestCase-specific configuration on a function before testing.

        By default, this does nothing. Example usage: spinning a function so
        that a JIT will optimize it. Subclasses should override this as needed.

        Args:
            func: function to configure.
            *args: any arguments that should be passed to func, if calling it.

        Returns:
            Nothing. Work will be performed on func in-place.
        """
    pass

def foo():
    x = range(3)
    yield len(x)
    yield len(x)
configure_func(foo)
g = foo()

assert next(g) == 3
with swap_item(globals(), '__builtins__', {'len': lambda x: 7}):

    assert next(g) == 3
print("RebindBuiltinsTests::test_cannot_replace_builtins_dict_while_active: ok")
"###);
    assert_output(&out, r###"RebindBuiltinsTests::test_cannot_replace_builtins_dict_while_active: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/dynamic/test_tracing__test_after_specialization.py`.
#[test]
fn test_gen_behavior_std_libs_dynamic_test_tracing__test_after_specialization() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dynamic"
# dimension = "behavior"
# case = "test_tracing__test_after_specialization"
# subject = "cpython.test_dynamic.TestTracing.test_after_specialization"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dynamic.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dynamic.py::TestTracing::test_after_specialization
"""Auto-ported test: TestTracing::test_after_specialization (CPython 3.12 oracle)."""


import builtins
import sys
import unittest
from test.support import swap_item, swap_attr


# --- test body ---
pass
sys.settrace(None)

def trace(frame, event, arg):
    return trace
turn_on_trace = False

class C:

    def __init__(self, x):
        self.x = x

    def __del__(self):
        if turn_on_trace:
            sys.settrace(trace)

def f():
    (C(0).x, len)

def g():
    [0][C(0).x]

def h():
    0 + C(0).x
for func in (f, g, h):
    for _ in range(58):
        func()
    turn_on_trace = True
    func()
    sys.settrace(None)
    turn_on_trace = False
print("TestTracing::test_after_specialization: ok")
"###);
    assert_output(&out, r###"TestTracing::test_after_specialization: ok
"###);
}
