# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_global_in_parallel_nested_functions"
# subject = "cpython.test_scope.ScopeTests.testGlobalInParallelNestedFunctions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testGlobalInParallelNestedFunctions
"""Auto-ported test: ScopeTests::testGlobalInParallelNestedFunctions (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
local_ns = {}
global_ns = {}
exec('if 1:\n            def f():\n                y = 1\n                def g():\n                    global y\n                    return y\n                def h():\n                    return y + 1\n                return g, h\n            y = 9\n            g, h = f()\n            result9 = g()\n            result2 = h()\n            ', local_ns, global_ns)

assert 2 == global_ns['result2']

assert 9 == global_ns['result9']
print("ScopeTests::testGlobalInParallelNestedFunctions: ok")
