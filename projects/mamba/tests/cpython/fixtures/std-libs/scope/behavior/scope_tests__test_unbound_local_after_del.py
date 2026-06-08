# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__test_unbound_local_after_del"
# subject = "cpython.test_scope.ScopeTests.testUnboundLocal_AfterDel"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_scope.py::ScopeTests::testUnboundLocal_AfterDel
"""Auto-ported test: ScopeTests::testUnboundLocal_AfterDel (CPython 3.12 oracle)."""


import unittest
import weakref
from test.support import check_syntax_error, cpython_only
from test.support import gc_collect


# --- test body ---
def errorInOuter():
    y = 1
    del y
    print(y)

    def inner():
        return y

def errorInInner():

    def inner():
        return y
    y = 1
    del y
    inner()

try:
    errorInOuter()
    raise AssertionError('expected UnboundLocalError')
except UnboundLocalError:
    pass

try:
    errorInInner()
    raise AssertionError('expected NameError')
except NameError:
    pass
print("ScopeTests::testUnboundLocal_AfterDel: ok")
