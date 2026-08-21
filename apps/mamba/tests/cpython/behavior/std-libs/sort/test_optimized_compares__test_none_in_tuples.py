# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sort"
# dimension = "behavior"
# case = "test_optimized_compares__test_none_in_tuples"
# subject = "cpython.test_sort.TestOptimizedCompares.test_none_in_tuples"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sort.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_sort.py::TestOptimizedCompares::test_none_in_tuples
"""Auto-ported test: TestOptimizedCompares::test_none_in_tuples (CPython 3.12 oracle)."""


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
expected = [(None, 1), (None, 2)]
actual = sorted([(None, 2), (None, 1)])

assert actual == expected
print("TestOptimizedCompares::test_none_in_tuples: ok")
