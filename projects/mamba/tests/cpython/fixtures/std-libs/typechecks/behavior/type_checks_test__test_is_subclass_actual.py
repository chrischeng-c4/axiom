# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typechecks"
# dimension = "behavior"
# case = "type_checks_test__test_is_subclass_actual"
# subject = "cpython.test_typechecks.TypeChecksTest.testIsSubclassActual"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_typechecks.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_typechecks.py::TypeChecksTest::testIsSubclassActual
"""Auto-ported test: TypeChecksTest::testIsSubclassActual (CPython 3.12 oracle)."""


import unittest


'Unit tests for __instancecheck__ and __subclasscheck__.'

class ABC(type):

    def __instancecheck__(cls, inst):
        """Implement isinstance(inst, cls)."""
        return any((cls.__subclasscheck__(c) for c in {type(inst), inst.__class__}))

    def __subclasscheck__(cls, sub):
        """Implement issubclass(sub, cls)."""
        candidates = cls.__dict__.get('__subclass__', set()) | {cls}
        return any((c in candidates for c in sub.mro()))

class Integer(metaclass=ABC):
    __subclass__ = {int}

class SubInt(Integer):
    pass


# --- test body ---

assert issubclass(Integer, Integer) == True

assert issubclass(Integer, (Integer,)) == True
print("TypeChecksTest::testIsSubclassActual: ok")
