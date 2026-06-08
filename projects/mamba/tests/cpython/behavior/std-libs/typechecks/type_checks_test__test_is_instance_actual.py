# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typechecks"
# dimension = "behavior"
# case = "type_checks_test__test_is_instance_actual"
# subject = "cpython.test_typechecks.TypeChecksTest.testIsInstanceActual"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typechecks.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_typechecks.py::TypeChecksTest::testIsInstanceActual
"""Auto-ported test: TypeChecksTest::testIsInstanceActual (CPython 3.12 oracle)."""


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

assert isinstance(Integer(), Integer) == True

assert isinstance(Integer(), (Integer,)) == True
print("TypeChecksTest::testIsInstanceActual: ok")
