# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sort"
# dimension = "behavior"
# case = "test_base__test_stressfully"
# subject = "cpython.test_sort.TestBase.testStressfully"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sort.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sort.py::TestBase::testStressfully
"""Auto-ported test: TestBase::testStressfully (CPython 3.12 oracle)."""


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
sizes = [0]
for power in range(1, 10):
    n = 2 ** power
    sizes.extend(range(n - 1, n + 2))
sizes.extend([10, 100, 1000])

class Complains(object):
    maybe_complain = True

    def __init__(self, i):
        self.i = i

    def __lt__(self, other):
        if Complains.maybe_complain and random.random() < 0.001:
            if verbose:
                print('        complaining at', self, other)
            raise RuntimeError
        return self.i < other.i

    def __repr__(self):
        return 'Complains(%d)' % self.i

class Stable(object):

    def __init__(self, key, i):
        self.key = key
        self.index = i

    def __lt__(self, other):
        return self.key < other.key

    def __repr__(self):
        return 'Stable(%d, %d)' % (self.key, self.index)
for n in sizes:
    x = list(range(n))
    if verbose:
        print('Testing size', n)
    s = x[:]
    check('identity', x, s)
    s = x[:]
    s.reverse()
    check('reversed', x, s)
    s = x[:]
    random.shuffle(s)
    check('random permutation', x, s)
    y = x[:]
    y.reverse()
    s = x[:]
    check('reversed via function', y, s, lambda a, b: (b > a) - (b < a))
    if verbose:
        print('    Checking against an insane comparison function.')
        print("        If the implementation isn't careful, this may segfault.")
    s = x[:]
    s.sort(key=cmp_to_key(lambda a, b: int(random.random() * 3) - 1))
    check('an insane function left some permutation', x, s)
    if len(x) >= 2:

        def bad_key(x):
            raise RuntimeError
        s = x[:]

        try:
            s.sort(key=bad_key)
            raise AssertionError('expected RuntimeError')
        except RuntimeError:
            pass
    x = [Complains(i) for i in x]
    s = x[:]
    random.shuffle(s)
    Complains.maybe_complain = True
    it_complained = False
    try:
        s.sort()
    except RuntimeError:
        it_complained = True
    if it_complained:
        Complains.maybe_complain = False
        check('exception during sort left some permutation', x, s)
    s = [Stable(random.randrange(10), i) for i in range(n)]
    augmented = [(e, e.index) for e in s]
    augmented.sort()
    x = [e for e, i in augmented]
    check('stability', x, s)
print("TestBase::testStressfully: ok")
