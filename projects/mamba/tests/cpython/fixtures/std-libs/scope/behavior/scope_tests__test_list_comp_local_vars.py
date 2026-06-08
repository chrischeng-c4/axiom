# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_list_comp_local_vars"
# subject = "cpython.test_scope.ScopeTests.testListCompLocalVars"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testListCompLocalVars
"""Auto-ported test: ScopeTests::testListCompLocalVars (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
try:
    print(bad)
except NameError:
    pass
else:
    print('bad should not be defined')

def x():
    [bad for s in 'a b' for bad in s.split()]
x()
try:
    print(bad)
except NameError:
    pass
print("ScopeTests::testListCompLocalVars: ok")
