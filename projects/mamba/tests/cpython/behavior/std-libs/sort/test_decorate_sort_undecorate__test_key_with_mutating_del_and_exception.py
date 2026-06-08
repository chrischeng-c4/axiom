# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sort"
# dimension = "behavior"
# case = "test_decorate_sort_undecorate__test_key_with_mutating_del_and_exception"
# subject = "cpython.test_sort.TestDecorateSortUndecorate.test_key_with_mutating_del_and_exception"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sort.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sort.py::TestDecorateSortUndecorate::test_key_with_mutating_del_and_exception
"""Auto-ported test: TestDecorateSortUndecorate::test_key_with_mutating_del_and_exception (CPython 3.12 oracle)."""


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
data = list(range(10))

class SortKiller(object):

    def __init__(self, x):
        if x > 2:
            raise RuntimeError

    def __del__(self):
        del data[:]
        data[:] = list(range(20))

try:
    data.sort(key=SortKiller)
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass
print("TestDecorateSortUndecorate::test_key_with_mutating_del_and_exception: ok")
