# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_lambdas"
# subject = "cpython.test_scope.ScopeTests.testLambdas"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testLambdas
"""Auto-ported test: ScopeTests::testLambdas (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
f1 = lambda x: lambda y: x + y
inc = f1(1)
plus10 = f1(10)

assert inc(1) == 2

assert plus10(5) == 15
f2 = lambda x: (lambda: lambda y: x + y)()
inc = f2(1)
plus10 = f2(10)

assert inc(1) == 2

assert plus10(5) == 15
f3 = lambda x: lambda y: global_x + y
global_x = 1
inc = f3(None)

assert inc(2) == 3
f8 = lambda x, y, z: lambda a, b, c: lambda: z * (b + y)
g = f8(1, 2, 3)
h = g(2, 4, 6)

assert h() == 18
print("ScopeTests::testLambdas: ok")
