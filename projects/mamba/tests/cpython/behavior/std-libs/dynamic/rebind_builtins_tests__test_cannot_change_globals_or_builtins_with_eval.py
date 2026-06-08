# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dynamic"
# dimension = "behavior"
# case = "rebind_builtins_tests__test_cannot_change_globals_or_builtins_with_eval"
# subject = "cpython.test_dynamic.RebindBuiltinsTests.test_cannot_change_globals_or_builtins_with_eval"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dynamic.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dynamic.py::RebindBuiltinsTests::test_cannot_change_globals_or_builtins_with_eval
"""Auto-ported test: RebindBuiltinsTests::test_cannot_change_globals_or_builtins_with_eval (CPython 3.12 oracle)."""


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
builtins_dict = {'len': lambda x: 7}
globals_dict = {'foo': foo, '__builtins__': builtins_dict, 'len': lambda x: 8}

assert eval('foo()', globals_dict) == 3

assert eval('foo()', {'foo': foo}) == 3
print("RebindBuiltinsTests::test_cannot_change_globals_or_builtins_with_eval: ok")
