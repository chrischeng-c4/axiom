use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/genericclass/test_class_getitem__test_class_getitem_format.py`.
#[test]
fn test_gen_behavior_std_libs_genericclass_test_class_getitem__test_class_getitem_format() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "test_class_getitem__test_class_getitem_format"
# subject = "cpython.test_genericclass.TestClassGetitem.test_class_getitem_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_genericclass.py::TestClassGetitem::test_class_getitem_format
"""Auto-ported test: TestClassGetitem::test_class_getitem_format (CPython 3.12 oracle)."""


import unittest
from test import support


# --- test body ---
class C:

    def __class_getitem__(cls, item):
        return f'C[{item.__name__}]'

assert C[int] == 'C[int]'

assert C[C] == 'C[C]'
print("TestClassGetitem::test_class_getitem_format: ok")
"###);
    assert_output(&out, r###"TestClassGetitem::test_class_getitem_format: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/genericclass/test_class_getitem__test_class_getitem_inheritance.py`.
#[test]
fn test_gen_behavior_std_libs_genericclass_test_class_getitem__test_class_getitem_inheritance() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "test_class_getitem__test_class_getitem_inheritance"
# subject = "cpython.test_genericclass.TestClassGetitem.test_class_getitem_inheritance"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_genericclass.py::TestClassGetitem::test_class_getitem_inheritance
"""Auto-ported test: TestClassGetitem::test_class_getitem_inheritance (CPython 3.12 oracle)."""


import unittest
from test import support


# --- test body ---
class C:

    def __class_getitem__(cls, item):
        return f'{cls.__name__}[{item.__name__}]'

class D(C):
    ...

assert D[int] == 'D[int]'

assert D[D] == 'D[D]'
print("TestClassGetitem::test_class_getitem_inheritance: ok")
"###);
    assert_output(&out, r###"TestClassGetitem::test_class_getitem_inheritance: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/genericclass/test_class_getitem__test_class_getitem_inheritance_2.py`.
#[test]
fn test_gen_behavior_std_libs_genericclass_test_class_getitem__test_class_getitem_inheritance_2() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "test_class_getitem__test_class_getitem_inheritance_2"
# subject = "cpython.test_genericclass.TestClassGetitem.test_class_getitem_inheritance_2"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_genericclass.py::TestClassGetitem::test_class_getitem_inheritance_2
"""Auto-ported test: TestClassGetitem::test_class_getitem_inheritance_2 (CPython 3.12 oracle)."""


import unittest
from test import support


# --- test body ---
class C:

    def __class_getitem__(cls, item):
        return 'Should not see this'

class D(C):

    def __class_getitem__(cls, item):
        return f'{cls.__name__}[{item.__name__}]'

assert D[int] == 'D[int]'

assert D[D] == 'D[D]'
print("TestClassGetitem::test_class_getitem_inheritance_2: ok")
"###);
    assert_output(&out, r###"TestClassGetitem::test_class_getitem_inheritance_2: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/genericclass/test_class_getitem__test_class_getitem_metaclass.py`.
#[test]
fn test_gen_behavior_std_libs_genericclass_test_class_getitem__test_class_getitem_metaclass() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "test_class_getitem__test_class_getitem_metaclass"
# subject = "cpython.test_genericclass.TestClassGetitem.test_class_getitem_metaclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_genericclass.py::TestClassGetitem::test_class_getitem_metaclass
"""Auto-ported test: TestClassGetitem::test_class_getitem_metaclass (CPython 3.12 oracle)."""


import unittest
from test import support


# --- test body ---
class Meta(type):

    def __class_getitem__(cls, item):
        return f'{cls.__name__}[{item.__name__}]'

assert Meta[int] == 'Meta[int]'
print("TestClassGetitem::test_class_getitem_metaclass: ok")
"###);
    assert_output(&out, r###"TestClassGetitem::test_class_getitem_metaclass: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/genericclass/test_class_getitem__test_class_getitem_with_builtins.py`.
#[test]
fn test_gen_behavior_std_libs_genericclass_test_class_getitem__test_class_getitem_with_builtins() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "test_class_getitem__test_class_getitem_with_builtins"
# subject = "cpython.test_genericclass.TestClassGetitem.test_class_getitem_with_builtins"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_genericclass.py::TestClassGetitem::test_class_getitem_with_builtins
"""Auto-ported test: TestClassGetitem::test_class_getitem_with_builtins (CPython 3.12 oracle)."""


import unittest
from test import support


# --- test body ---
class A(dict):
    called_with = None

    def __class_getitem__(cls, item):
        cls.called_with = item

class B(A):
    pass

assert B.called_with is None
B[int]

assert B.called_with is int
print("TestClassGetitem::test_class_getitem_with_builtins: ok")
"###);
    assert_output(&out, r###"TestClassGetitem::test_class_getitem_with_builtins: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/genericclass/test_class_getitem__test_class_getitem_with_metaclass.py`.
#[test]
fn test_gen_behavior_std_libs_genericclass_test_class_getitem__test_class_getitem_with_metaclass() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "test_class_getitem__test_class_getitem_with_metaclass"
# subject = "cpython.test_genericclass.TestClassGetitem.test_class_getitem_with_metaclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_genericclass.py::TestClassGetitem::test_class_getitem_with_metaclass
"""Auto-ported test: TestClassGetitem::test_class_getitem_with_metaclass (CPython 3.12 oracle)."""


import unittest
from test import support


# --- test body ---
class Meta(type):
    pass

class C(metaclass=Meta):

    def __class_getitem__(cls, item):
        return f'{cls.__name__}[{item.__name__}]'

assert C[int] == 'C[int]'
print("TestClassGetitem::test_class_getitem_with_metaclass: ok")
"###);
    assert_output(&out, r###"TestClassGetitem::test_class_getitem_with_metaclass: ok
"###);
}
