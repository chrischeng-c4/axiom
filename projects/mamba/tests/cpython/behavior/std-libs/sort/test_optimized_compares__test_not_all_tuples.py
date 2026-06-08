# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sort"
# dimension = "behavior"
# case = "test_optimized_compares__test_not_all_tuples"
# subject = "cpython.test_sort.TestOptimizedCompares.test_not_all_tuples"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sort.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sort.py::TestOptimizedCompares::test_not_all_tuples
"""Auto-ported test: TestOptimizedCompares::test_not_all_tuples (CPython 3.12 oracle)."""


from test import support
import random
import unittest
from functools import cmp_to_key


verbose = support.verbose

nerrors = 0

def check(tag, expected, raw, compare=None):
    global nerrors
    if verbose:
        print('    checking', tag)
    orig = raw[:]
    if compare:
        raw.sort(key=cmp_to_key(compare))
    else:
        raw.sort()
    if len(expected) != len(raw):
        print('error in', tag)
        print('length mismatch;', len(expected), len(raw))
        print(expected)
        print(orig)
        print(raw)
        nerrors += 1
        return
    for i, good in enumerate(expected):
        maybe = raw[i]
        if good is not maybe:
            print('error in', tag)
            print('out of order at index', i, good, maybe)
            print(expected)
            print(orig)
            print(raw)
            nerrors += 1
            return

def check_against_PyObject_RichCompareBool(self, L):
    random.seed(0)
    random.shuffle(L)
    L_1 = L[:]
    L_2 = [(x,) for x in L]
    L_3 = [((x,),) for x in L]
    for L in [L_1, L_2, L_3]:
        optimized = sorted(L)
        reference = [y[1] for y in sorted([(0, x) for x in L])]
        for opt, ref in zip(optimized, reference):
            self.assertIs(opt, ref)


# --- test body ---

try:
    [(1.0, 1.0), (False, 'A'), 6].sort()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    [('a', 1), (1, 'a')].sort()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    [(1, 'a'), ('a', 1)].sort()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestOptimizedCompares::test_not_all_tuples: ok")
