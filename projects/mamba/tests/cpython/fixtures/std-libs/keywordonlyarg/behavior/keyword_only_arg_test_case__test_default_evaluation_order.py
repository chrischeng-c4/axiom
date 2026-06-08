# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keywordonlyarg"
# dimension = "behavior"
# case = "keyword_only_arg_test_case__test_default_evaluation_order"
# subject = "cpython.test_keywordonlyarg.KeywordOnlyArgTestCase.test_default_evaluation_order"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_keywordonlyarg.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_keywordonlyarg.py::KeywordOnlyArgTestCase::test_default_evaluation_order
"""Auto-ported test: KeywordOnlyArgTestCase::test_default_evaluation_order (CPython 3.12 oracle)."""


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
a = 42
try:

    def f(v=a, x=b, *, y=c, z=d):
        pass
    raise AssertionError('expected NameError')
except NameError as _aR_e:
    import types as _types_aR
    err = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(err.exception) == "name 'b' is not defined"
try:
    g = lambda v=a, x=b, *, y=c, z=d: None
    raise AssertionError('expected NameError')
except NameError as _aR_e:
    import types as _types_aR
    err = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(err.exception) == "name 'b' is not defined"
print("KeywordOnlyArgTestCase::test_default_evaluation_order: ok")
