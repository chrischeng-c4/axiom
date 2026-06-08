# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dynamic"
# dimension = "behavior"
# case = "rebind_builtins_tests__test_modify_builtins_from_leaf_function"
# subject = "cpython.test_dynamic.RebindBuiltinsTests.test_modify_builtins_from_leaf_function"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dynamic.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dynamic.py::RebindBuiltinsTests::test_modify_builtins_from_leaf_function
"""Auto-ported test: RebindBuiltinsTests::test_modify_builtins_from_leaf_function (CPython 3.12 oracle)."""


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
with swap_attr(builtins, 'len', len):

    def bar():
        builtins.len = lambda x: 4

    def foo(modifier):
        l = []
        l.append(len(range(7)))
        modifier()
        l.append(len(range(7)))
        return l
    configure_func(foo, lambda: None)

    assert foo(bar) == [7, 4]
print("RebindBuiltinsTests::test_modify_builtins_from_leaf_function: ok")
