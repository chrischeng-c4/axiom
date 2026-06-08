# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_class_namespace_overrides_closure"
# subject = "cpython.test_scope.ScopeTests.testClassNamespaceOverridesClosure"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testClassNamespaceOverridesClosure
"""Auto-ported test: ScopeTests::testClassNamespaceOverridesClosure (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
x = 42

class X:
    locals()['x'] = 43
    y = x

assert X.y == 43

class X:
    locals()['x'] = 43
    del x

assert not hasattr(X, 'x')

assert x == 42
print("ScopeTests::testClassNamespaceOverridesClosure: ok")
