# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "except_star"
# dimension = "behavior"
# case = "test_except_star_exception_group_subclass__test_except_star_eg_subclass"
# subject = "cpython.test_except_star.TestExceptStarExceptionGroupSubclass.test_except_star_EG_subclass"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_except_star.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_except_star.py::TestExceptStarExceptionGroupSubclass::test_except_star_EG_subclass
"""Auto-ported test: TestExceptStarExceptionGroupSubclass::test_except_star_EG_subclass (CPython 3.12 oracle)."""


import sys
import unittest
import textwrap
from test.support.testcase import ExceptionIsLikeMixin


# --- test body ---
def assertMetadataEqual(e1, e2):
    if e1 is None or e2 is None:

        assert e1 is None and e2 is None
    else:

        assert e1.__context__ == e2.__context__

        assert e1.__cause__ == e2.__cause__

        assert e1.__traceback__ == e2.__traceback__

def assertMetadataNotEqual(e1, e2):
    if e1 is None or e2 is None:

        assert e1 != e2
    else:
        return not (e1.__context__ == e2.__context__ and e1.__cause__ == e2.__cause__ and (e1.__traceback__ == e2.__traceback__))

class EG(ExceptionGroup):

    def __new__(cls, message, excs, code):
        obj = super().__new__(cls, message, excs)
        obj.code = code
        return obj

    def derive(self, excs):
        return EG(self.message, excs, self.code)
try:
    try:
        try:
            try:
                raise TypeError(2)
            except TypeError as te:
                raise EG('nested', [te], 101) from None
        except EG as nested:
            try:
                raise ValueError(1)
            except ValueError as ve:
                raise EG('eg', [ve, nested], 42)
    except* ValueError as eg:
        veg = eg
except EG as eg:
    teg = eg

assert isinstance(veg, EG)

assert isinstance(teg, EG)

assert isinstance(teg.exceptions[0], EG)
assertMetadataEqual(veg, teg)

assert veg.code == 42

assert teg.code == 42

assert teg.exceptions[0].code == 101
print("TestExceptStarExceptionGroupSubclass::test_except_star_EG_subclass: ok")
