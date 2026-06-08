# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keywordonlyarg"
# dimension = "behavior"
# case = "keyword_only_arg_test_case__test_function_call"
# subject = "cpython.test_keywordonlyarg.KeywordOnlyArgTestCase.testFunctionCall"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_keywordonlyarg.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_keywordonlyarg.py::KeywordOnlyArgTestCase::testFunctionCall
"""Auto-ported test: KeywordOnlyArgTestCase::testFunctionCall (CPython 3.12 oracle)."""


import unittest


'Unit tests for the keyword only argument specified in PEP 3102.'

__author__ = 'Jiwon Seo'

__email__ = 'seojiwon at gmail dot com'

def posonly_sum(pos_arg1, *arg, **kwarg):
    return pos_arg1 + sum(arg) + sum(kwarg.values())

def keywordonly_sum(*, k1=0, k2):
    return k1 + k2

def keywordonly_nodefaults_sum(*, k1, k2):
    return k1 + k2

def keywordonly_and_kwarg_sum(*, k1, k2, **kwarg):
    return k1 + k2 + sum(kwarg.values())

def mixedargs_sum(a, b=0, *arg, k1, k2=0):
    return a + b + k1 + k2 + sum(arg)

def mixedargs_sum2(a, b=0, *arg, k1, k2=0, **kwargs):
    return a + b + k1 + k2 + sum(arg) + sum(kwargs.values())

def sortnum(*nums, reverse=False):
    return sorted(list(nums), reverse=reverse)

def sortwords(*words, reverse=False, **kwargs):
    return sorted(list(words), reverse=reverse)

class Foo:

    def __init__(self, *, k1, k2=0):
        self.k1 = k1
        self.k2 = k2

    def set(self, p1, *, k1, k2):
        self.k1 = k1
        self.k2 = k2

    def sum(self):
        return self.k1 + self.k2


# --- test body ---
def assertRaisesSyntaxError(codestr):

    def shouldRaiseSyntaxError(s):
        compile(s, '<test>', 'single')

    try:
        shouldRaiseSyntaxError(codestr)
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass

assert 1 == posonly_sum(1)

assert 1 + 2 == posonly_sum(1, **{'2': 2})

assert 1 + 2 + 3 == posonly_sum(1, *(2, 3))

assert 1 + 2 + 3 + 4 == posonly_sum(1, *(2, 3), **{'4': 4})

assert 1 == keywordonly_sum(k2=1)

assert 1 + 2 == keywordonly_sum(k1=1, k2=2)

assert 1 + 2 == keywordonly_and_kwarg_sum(k1=1, k2=2)

assert 1 + 2 + 3 == keywordonly_and_kwarg_sum(k1=1, k2=2, k3=3)

assert 1 + 2 + 3 + 4 == keywordonly_and_kwarg_sum(k1=1, k2=2, **{'a': 3, 'b': 4})

assert 1 + 2 == mixedargs_sum(1, k1=2)

assert 1 + 2 + 3 == mixedargs_sum(1, 2, k1=3)

assert 1 + 2 + 3 + 4 == mixedargs_sum(1, 2, k1=3, k2=4)

assert 1 + 2 + 3 + 4 + 5 == mixedargs_sum(1, 2, 3, k1=4, k2=5)

assert 1 + 2 == mixedargs_sum2(1, k1=2)

assert 1 + 2 + 3 == mixedargs_sum2(1, 2, k1=3)

assert 1 + 2 + 3 + 4 == mixedargs_sum2(1, 2, k1=3, k2=4)

assert 1 + 2 + 3 + 4 + 5 == mixedargs_sum2(1, 2, 3, k1=4, k2=5)

assert 1 + 2 + 3 + 4 + 5 + 6 == mixedargs_sum2(1, 2, 3, k1=4, k2=5, k3=6)

assert 1 + 2 + 3 + 4 + 5 + 6 == mixedargs_sum2(1, 2, 3, k1=4, **{'k2': 5, 'k3': 6})

assert 1 == Foo(k1=1).sum()

assert 1 + 2 == Foo(k1=1, k2=2).sum()

assert [1, 2, 3] == sortnum(3, 2, 1)

assert [3, 2, 1] == sortnum(1, 2, 3, reverse=True)

assert ['a', 'b', 'c'] == sortwords('a', 'c', 'b')

assert ['c', 'b', 'a'] == sortwords('a', 'c', 'b', reverse=True)

assert ['c', 'b', 'a'] == sortwords('a', 'c', 'b', reverse=True, ignore='ignore')
print("KeywordOnlyArgTestCase::testFunctionCall: ok")
