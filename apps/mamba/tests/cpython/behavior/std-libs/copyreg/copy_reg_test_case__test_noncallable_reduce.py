# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copyreg"
# dimension = "behavior"
# case = "copy_reg_test_case__test_noncallable_reduce"
# subject = "cpython.test_copyreg.CopyRegTestCase.test_noncallable_reduce"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_copyreg.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_copyreg.py::CopyRegTestCase::test_noncallable_reduce
"""Auto-ported test: CopyRegTestCase::test_noncallable_reduce (CPython 3.12 oracle)."""


import copyreg
import unittest
from test.pickletester import ExtensionSaver


class C:
    pass

def pickle_C(c):
    return (C, ())

class WithoutSlots(object):
    pass

class WithWeakref(object):
    __slots__ = ('__weakref__',)

class WithPrivate(object):
    __slots__ = ('__spam',)

class _WithLeadingUnderscoreAndPrivate(object):
    __slots__ = ('__spam',)

class ___(object):
    __slots__ = ('__spam',)

class WithSingleString(object):
    __slots__ = 'spam'

class WithInherited(WithSingleString):
    __slots__ = ('eggs',)


# --- test body ---

try:
    copyreg.pickle(C, 'not a callable')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("CopyRegTestCase::test_noncallable_reduce: ok")
