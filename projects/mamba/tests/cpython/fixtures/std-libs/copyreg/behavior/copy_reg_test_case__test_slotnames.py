# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copyreg"
# dimension = "behavior"
# case = "copy_reg_test_case__test_slotnames"
# subject = "cpython.test_copyreg.CopyRegTestCase.test_slotnames"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_copyreg.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_copyreg.py::CopyRegTestCase::test_slotnames
"""Auto-ported test: CopyRegTestCase::test_slotnames (CPython 3.12 oracle)."""


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

assert copyreg._slotnames(WithoutSlots) == []

assert copyreg._slotnames(WithWeakref) == []
expected = ['_WithPrivate__spam']

assert copyreg._slotnames(WithPrivate) == expected
expected = ['_WithLeadingUnderscoreAndPrivate__spam']

assert copyreg._slotnames(_WithLeadingUnderscoreAndPrivate) == expected

assert copyreg._slotnames(___) == ['__spam']

assert copyreg._slotnames(WithSingleString) == ['spam']
expected = ['eggs', 'spam']
expected.sort()
result = copyreg._slotnames(WithInherited)
result.sort()

assert result == expected
print("CopyRegTestCase::test_slotnames: ok")
