use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/opcodes/opcode_test__test_do_not_recreate_annotations.py`.
#[test]
fn test_gen_behavior_std_libs_opcodes_opcode_test__test_do_not_recreate_annotations() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcodes"
# dimension = "behavior"
# case = "opcode_test__test_do_not_recreate_annotations"
# subject = "cpython.test_opcodes.OpcodeTest.test_do_not_recreate_annotations"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_opcodes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_opcodes.py::OpcodeTest::test_do_not_recreate_annotations
"""Auto-ported test: OpcodeTest::test_do_not_recreate_annotations."""


from test import support


with support.swap_item(globals(), "__annotations__", {}):
    del globals()["__annotations__"]

    class C:
        del __annotations__
        try:
            x: int
        except NameError:
            pass
        else:
            raise AssertionError("expected NameError")


print("OpcodeTest::test_do_not_recreate_annotations: ok")
"###);
    assert_output(&out, r###"OpcodeTest::test_do_not_recreate_annotations: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/opcodes/opcode_test__test_modulo_of_string_subclasses.py`.
#[test]
fn test_gen_behavior_std_libs_opcodes_opcode_test__test_modulo_of_string_subclasses() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcodes"
# dimension = "behavior"
# case = "opcode_test__test_modulo_of_string_subclasses"
# subject = "cpython.test_opcodes.OpcodeTest.test_modulo_of_string_subclasses"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_opcodes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_opcodes.py::OpcodeTest::test_modulo_of_string_subclasses
"""Auto-ported test: OpcodeTest::test_modulo_of_string_subclasses (CPython 3.12 oracle)."""


import unittest
from test import support
from test.typinganndata import ann_module


# --- test body ---
class MyString(str):

    def __mod__(self, value):
        return 42

assert MyString() % 3 == 42
print("OpcodeTest::test_modulo_of_string_subclasses: ok")
"###);
    assert_output(&out, r###"OpcodeTest::test_modulo_of_string_subclasses: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/opcodes/opcode_test__test_raise_class_exceptions.py`.
#[test]
fn test_gen_behavior_std_libs_opcodes_opcode_test__test_raise_class_exceptions() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcodes"
# dimension = "behavior"
# case = "opcode_test__test_raise_class_exceptions"
# subject = "cpython.test_opcodes.OpcodeTest.test_raise_class_exceptions"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_opcodes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_opcodes.py::OpcodeTest::test_raise_class_exceptions
"""Auto-ported test: OpcodeTest::test_raise_class_exceptions (CPython 3.12 oracle)."""


import unittest
from test import support
from test.typinganndata import ann_module


# --- test body ---
class AClass(Exception):
    pass

class BClass(AClass):
    pass

class CClass(Exception):
    pass

class DClass(AClass):

    def __init__(self, ignore):
        pass
try:
    raise AClass()
except:
    pass
try:
    raise AClass()
except AClass:
    pass
try:
    raise BClass()
except AClass:
    pass
try:
    raise BClass()
except CClass:

    raise AssertionError('fail')
except:
    pass
a = AClass()
b = BClass()
try:
    raise b
except AClass as v:

    assert v == b
else:

    raise AssertionError('no exception')
try:
    raise DClass(a)
except DClass as v:

    assert isinstance(v, DClass)
else:

    raise AssertionError('no exception')
print("OpcodeTest::test_raise_class_exceptions: ok")
"###);
    assert_output(&out, r###"OpcodeTest::test_raise_class_exceptions: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/opcodes/opcode_test__test_try_inside_for_loop.py`.
#[test]
fn test_gen_behavior_std_libs_opcodes_opcode_test__test_try_inside_for_loop() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcodes"
# dimension = "behavior"
# case = "opcode_test__test_try_inside_for_loop"
# subject = "cpython.test_opcodes.OpcodeTest.test_try_inside_for_loop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_opcodes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_opcodes.py::OpcodeTest::test_try_inside_for_loop
"""Auto-ported test: OpcodeTest::test_try_inside_for_loop (CPython 3.12 oracle)."""


import unittest
from test import support
from test.typinganndata import ann_module


# --- test body ---
n = 0
for i in range(10):
    n = n + i
    try:
        1 / 0
    except NameError:
        pass
    except ZeroDivisionError:
        pass
    except TypeError:
        pass
    try:
        pass
    except:
        pass
    try:
        pass
    finally:
        pass
    n = n + i
if n != 90:

    raise AssertionError('try inside for')
print("OpcodeTest::test_try_inside_for_loop: ok")
"###);
    assert_output(&out, r###"OpcodeTest::test_try_inside_for_loop: ok
"###);
}
