# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contains"
# dimension = "behavior"
# case = "test_contains__test_common_tests"
# subject = "cpython.test_contains.TestContains.test_common_tests"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contains.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_contains.py::TestContains::test_common_tests
"""Auto-ported test: TestContains::test_common_tests (CPython 3.12 oracle)."""


from collections import deque
import unittest
from test.support import NEVER_EQ


class base_set:

    def __init__(self, el):
        self.el = el

class myset(base_set):

    def __contains__(self, el):
        return self.el == el

class seq(base_set):

    def __getitem__(self, n):
        return [self.el][n]


# --- test body ---
a = base_set(1)
b = myset(1)
c = seq(1)

assert 1 in b

assert 0 not in b

assert 1 in c

assert 0 not in c

try:
    (lambda: 1 in a)()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    (lambda: 1 not in a)()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert 'c' in 'abc'

assert 'd' not in 'abc'

assert '' in ''

assert '' in 'abc'

try:
    (lambda: None in 'abc')()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestContains::test_common_tests: ok")
