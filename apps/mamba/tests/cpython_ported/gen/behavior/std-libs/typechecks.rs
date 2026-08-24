use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/typechecks/type_checks_test__test_is_instance_actual.py`.
#[test]
fn test_gen_behavior_std_libs_typechecks_type_checks_test__test_is_instance_actual() {
    let out = jit_capture(r###"# /// script
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
"###);
    assert_output(&out, r###"TypeChecksTest::testIsInstanceActual: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/typechecks/type_checks_test__test_is_subclass_actual.py`.
#[test]
fn test_gen_behavior_std_libs_typechecks_type_checks_test__test_is_subclass_actual() {
    let out = jit_capture(r###"# /// script
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
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typechecks.py"
# status = "filled"
# ///
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
"###);
    assert_output(&out, r###"TypeChecksTest::testIsSubclassActual: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/typechecks/type_checks_test__test_subclass_behavior.py`.
#[test]
fn test_gen_behavior_std_libs_typechecks_type_checks_test__test_subclass_behavior() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typechecks"
# dimension = "behavior"
# case = "type_checks_test__test_subclass_behavior"
# subject = "cpython.test_typechecks.TypeChecksTest.testSubclassBehavior"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typechecks.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_typechecks.py::TypeChecksTest::testSubclassBehavior
"""Auto-ported test: TypeChecksTest::testSubclassBehavior (CPython 3.12 oracle)."""


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

assert issubclass(SubInt, Integer) == True

assert issubclass(SubInt, (Integer,)) == True

assert issubclass(SubInt, SubInt) == True

assert issubclass(SubInt, (SubInt,)) == True

assert issubclass(Integer, SubInt) == False

assert issubclass(Integer, (SubInt,)) == False

assert issubclass(int, SubInt) == False

assert issubclass(int, (SubInt,)) == False

assert isinstance(SubInt(), Integer) == True

assert isinstance(SubInt(), (Integer,)) == True

assert isinstance(SubInt(), SubInt) == True

assert isinstance(SubInt(), (SubInt,)) == True

assert isinstance(42, SubInt) == False

assert isinstance(42, (SubInt,)) == False
print("TypeChecksTest::testSubclassBehavior: ok")
"###);
    assert_output(&out, r###"TypeChecksTest::testSubclassBehavior: ok
"###);
}
