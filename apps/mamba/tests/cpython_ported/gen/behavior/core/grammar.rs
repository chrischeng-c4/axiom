use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/core/grammar/bigint_literal_parsing.py`.
#[test]
fn test_gen_behavior_core_grammar_bigint_literal_parsing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "bigint_literal_parsing"
# subject = "integer literals"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Integer literals larger than i64 parse as Python arbitrary-precision ints."""

decimal_value = 123456789012345678901234567890
assert decimal_value == int("123456789012345678901234567890")
assert repr(decimal_value) == "123456789012345678901234567890"

hex_value = 0x1_0000_0000_0000_0000
assert hex_value == int("10000000000000000", 16)

binary_value = 0b1_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000
assert binary_value == int("10000000000000000000000000000000000000000000000000000000000000000", 2)

octal_value = 0o1_0000_0000_0000_0000_0000_0000
assert octal_value == int("1000000000000000000000000", 8)

assert -9223372036854775808 == -int("9223372036854775808")

print("bigint_literal_parsing OK")
"###);
    assert_output(&out, r###"bigint_literal_parsing OK
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_additive_ops.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_additive_ops() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_additive_ops"
# subject = "cpython.test_grammar.GrammarTests.test_additive_ops"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_additive_ops
"""Auto-ported test: GrammarTests::test_additive_ops (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
x = 1
x = 1 + 1
x = 1 - 1 - 1
x = 1 - 1 + 1 - 1 + 1
print("GrammarTests::test_additive_ops: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_additive_ops: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_assert.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_assert() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_assert"
# subject = "cpython.test_grammar.GrammarTests.test_assert"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_assert
"""Auto-ported test: GrammarTests::test_assert (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
assert 1
assert 1, 1
assert lambda x: x
assert 1, lambda x: x + 1
try:
    assert True
except AssertionError as e:

    raise AssertionError("'assert True' should not have raised an AssertionError")
try:
    assert True, 'this should always pass'
except AssertionError as e:

    raise AssertionError("'assert True, msg' should not have raised an AssertionError")
print("GrammarTests::test_assert: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_assert: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_atoms.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_atoms() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_atoms"
# subject = "cpython.test_grammar.GrammarTests.test_atoms"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_atoms
"""Auto-ported test: GrammarTests::test_atoms (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
x = 1
x = 1 or 2 or 3
x = (1 or 2 or 3, 2, 3)
x = []
x = [1]
x = [1 or 2 or 3]
x = [1 or 2 or 3, 2, 3]
x = []
x = {}
x = {'one': 1}
x = {'one': 1}
x = {'one' or 'two': 1 or 2}
x = {'one': 1, 'two': 2}
x = {'one': 1, 'two': 2}
x = {'one': 1, 'two': 2, 'three': 3, 'four': 4, 'five': 5, 'six': 6}
x = {'one'}
x = {'one', 1}
x = {'one', 'two', 'three'}
x = {2, 3, 4}
x = x
x = 'x'
x = 123
print("GrammarTests::test_atoms: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_atoms: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_binary_mask_ops.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_binary_mask_ops() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_binary_mask_ops"
# subject = "cpython.test_grammar.GrammarTests.test_binary_mask_ops"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_binary_mask_ops
"""Auto-ported test: GrammarTests::test_binary_mask_ops (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
x = 1 & 1
x = 1 ^ 1
x = 1 | 1
print("GrammarTests::test_binary_mask_ops: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_binary_mask_ops: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_break_continue_loop.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_break_continue_loop() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_break_continue_loop"
# subject = "cpython.test_grammar.GrammarTests.test_break_continue_loop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_break_continue_loop
"""Auto-ported test: GrammarTests::test_break_continue_loop (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
def test_inner(extra_burning_oil=1, count=0):
    big_hippo = 2
    while big_hippo:
        count += 1
        try:
            if extra_burning_oil and big_hippo == 1:
                extra_burning_oil -= 1
                break
            big_hippo -= 1
            continue
        except:
            raise
    if count > 2 or big_hippo != 1:
        self.fail('continue then break in try/except in loop broken!')
test_inner()
print("GrammarTests::test_break_continue_loop: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_break_continue_loop: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_break_in_finally.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_break_in_finally() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_break_in_finally"
# subject = "cpython.test_grammar.GrammarTests.test_break_in_finally"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_break_in_finally
"""Auto-ported test: GrammarTests::test_break_in_finally (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
count = 0
while count < 2:
    count += 1
    try:
        pass
    finally:
        break

assert count == 1
count = 0
while count < 2:
    count += 1
    try:
        continue
    finally:
        break

assert count == 1
count = 0
while count < 2:
    count += 1
    try:
        1 / 0
    finally:
        break

assert count == 1
for count in [0, 1]:

    assert count == 0
    try:
        pass
    finally:
        break

assert count == 0
for count in [0, 1]:

    assert count == 0
    try:
        continue
    finally:
        break

assert count == 0
for count in [0, 1]:

    assert count == 0
    try:
        1 / 0
    finally:
        break

assert count == 0
print("GrammarTests::test_break_in_finally: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_break_in_finally: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_break_stmt.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_break_stmt() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_break_stmt"
# subject = "cpython.test_grammar.GrammarTests.test_break_stmt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_break_stmt
"""Auto-ported test: GrammarTests::test_break_stmt (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
while 1:
    break
print("GrammarTests::test_break_stmt: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_break_stmt: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_comparison.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_comparison() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_comparison"
# subject = "cpython.test_grammar.GrammarTests.test_comparison"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_comparison
"""Auto-ported test: GrammarTests::test_comparison (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
if 1:
    pass
x = 1 == 1
if 1 == 1:
    pass
if 1 != 1:
    pass
if 1 < 1:
    pass
if 1 > 1:
    pass
if 1 <= 1:
    pass
if 1 >= 1:
    pass
if x is x:
    pass
if x is not x:
    pass
if 1 in ():
    pass
if 1 not in ():
    pass
if 1 < 1 > 1 == 1 >= 1 <= 1 != 1 in 1 not in x is x is not x:
    pass
print("GrammarTests::test_comparison: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_comparison: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_complex_lambda.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_complex_lambda() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_complex_lambda"
# subject = "cpython.test_grammar.GrammarTests.test_complex_lambda"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_complex_lambda
"""Auto-ported test: GrammarTests::test_complex_lambda (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
def test1(foo, bar):
    return ''

def test2():
    return f"{test1(foo=lambda: '、、、、、、、、、、、、、、、、、', bar=lambda: 'abcdefghijklmnopqrstuvwxyz 123456789 123456789')}"

assert test2() == ''
print("GrammarTests::test_complex_lambda: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_complex_lambda: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_dictcomps.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_dictcomps() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_dictcomps"
# subject = "cpython.test_grammar.GrammarTests.test_dictcomps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_dictcomps
"""Auto-ported test: GrammarTests::test_dictcomps (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
nums = [1, 2, 3]

assert {i: i + 1 for i in nums} == {1: 2, 2: 3, 3: 4}
print("GrammarTests::test_dictcomps: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_dictcomps: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_eval_input.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_eval_input() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_eval_input"
# subject = "cpython.test_grammar.GrammarTests.test_eval_input"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_eval_input
"""Auto-ported test: GrammarTests::test_eval_input (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
x = eval('1, 0 or 1')
print("GrammarTests::test_eval_input: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_eval_input: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_global.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_global() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_global"
# subject = "cpython.test_grammar.GrammarTests.test_global"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_global
"""Auto-ported test: GrammarTests::test_global (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
global a
global a, b
global one, two, three, four, five, six, seven, eight, nine, ten
print("GrammarTests::test_global: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_global: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_if.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_if() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_if"
# subject = "cpython.test_grammar.GrammarTests.test_if"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_if
"""Auto-ported test: GrammarTests::test_if (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
if 1:
    pass
if 1:
    pass
else:
    pass
if 0:
    pass
elif 0:
    pass
if 0:
    pass
elif 0:
    pass
elif 0:
    pass
elif 0:
    pass
else:
    pass
print("GrammarTests::test_if: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_if: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_import.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_import() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_import"
# subject = "cpython.test_grammar.GrammarTests.test_import"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_import
"""Auto-ported test: GrammarTests::test_import (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
import sys
import time, sys
from time import time
from time import time
from sys import path, argv
from sys import path, argv
from sys import path, argv
print("GrammarTests::test_import: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_import: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_multiplicative_ops.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_multiplicative_ops() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_multiplicative_ops"
# subject = "cpython.test_grammar.GrammarTests.test_multiplicative_ops"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_multiplicative_ops
"""Auto-ported test: GrammarTests::test_multiplicative_ops (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
x = 1 * 1
x = 1 / 1
x = 1 % 1
x = 1 / 1 * 1 % 1
print("GrammarTests::test_multiplicative_ops: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_multiplicative_ops: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_paren_evaluation.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_paren_evaluation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_paren_evaluation"
# subject = "cpython.test_grammar.GrammarTests.test_paren_evaluation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_paren_evaluation
"""Auto-ported test: GrammarTests::test_paren_evaluation (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---

assert 16 // (4 // 2) == 8

assert 16 // 4 // 2 == 2

assert 16 // 4 // 2 == 2
x = 2
y = 3

assert False is (x is y)

assert not (False is x) is y

assert not False is x is y
print("GrammarTests::test_paren_evaluation: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_paren_evaluation: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_pass_stmt.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_pass_stmt() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_pass_stmt"
# subject = "cpython.test_grammar.GrammarTests.test_pass_stmt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_pass_stmt
"""Auto-ported test: GrammarTests::test_pass_stmt (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
pass
print("GrammarTests::test_pass_stmt: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_pass_stmt: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_raise.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_raise"
# subject = "cpython.test_grammar.GrammarTests.test_raise"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_raise
"""Auto-ported test: GrammarTests::test_raise (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
try:
    raise RuntimeError('just testing')
except RuntimeError:
    pass
try:
    raise KeyboardInterrupt
except KeyboardInterrupt:
    pass
print("GrammarTests::test_raise: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_raise: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_return_in_finally.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_return_in_finally() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_return_in_finally"
# subject = "cpython.test_grammar.GrammarTests.test_return_in_finally"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_return_in_finally
"""Auto-ported test: GrammarTests::test_return_in_finally (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
def g1():
    try:
        pass
    finally:
        return 1

assert g1() == 1

def g2():
    try:
        return 2
    finally:
        return 3

assert g2() == 3

def g3():
    try:
        1 / 0
    finally:
        return 4

assert g3() == 4
print("GrammarTests::test_return_in_finally: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_return_in_finally: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_shift_ops.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_shift_ops() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_shift_ops"
# subject = "cpython.test_grammar.GrammarTests.test_shift_ops"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_shift_ops
"""Auto-ported test: GrammarTests::test_shift_ops (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
x = 1 << 1
x = 1 >> 1
x = 1 << 1 >> 1
print("GrammarTests::test_shift_ops: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_shift_ops: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_simple_stmt.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_simple_stmt() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_simple_stmt"
# subject = "cpython.test_grammar.GrammarTests.test_simple_stmt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_simple_stmt
"""Auto-ported test: GrammarTests::test_simple_stmt (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
x = 1
pass
del x

def foo():
    x = 1
    pass
    del x
foo()
print("GrammarTests::test_simple_stmt: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_simple_stmt: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_suite.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_suite() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_suite"
# subject = "cpython.test_grammar.GrammarTests.test_suite"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_suite
"""Auto-ported test: GrammarTests::test_suite (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
if 1:
    pass
if 1:
    pass
if 1:
    pass
    pass
    pass
print("GrammarTests::test_suite: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_suite: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_test.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_test() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_test"
# subject = "cpython.test_grammar.GrammarTests.test_test"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_test
"""Auto-ported test: GrammarTests::test_test (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
if not 1:
    pass
if 1 and 1:
    pass
if 1 or 1:
    pass
if not not not 1:
    pass
if not 1 and 1 and 1:
    pass
if 1 and 1 or (1 and 1 and 1) or (not 1 and 1):
    pass
print("GrammarTests::test_test: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_test: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_try.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_try() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_try"
# subject = "cpython.test_grammar.GrammarTests.test_try"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_try
"""Auto-ported test: GrammarTests::test_try (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
try:
    1 / 0
except ZeroDivisionError:
    pass
else:
    pass
try:
    1 / 0
except EOFError:
    pass
except TypeError as msg:
    pass
except:
    pass
else:
    pass
try:
    1 / 0
except (EOFError, TypeError, ZeroDivisionError):
    pass
try:
    1 / 0
except (EOFError, TypeError, ZeroDivisionError) as msg:
    pass
try:
    pass
finally:
    pass
try:
    compile('try:\n    pass\nexcept Exception as a.b:\n    pass', '?', 'exec')
    compile('try:\n    pass\nexcept Exception as a[b]:\n    pass', '?', 'exec')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass
print("GrammarTests::test_try: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_try: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_try_star.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_try_star() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_try_star"
# subject = "cpython.test_grammar.GrammarTests.test_try_star"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_try_star
"""Auto-ported test: GrammarTests::test_try_star (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
try:
    1 / 0
except* ZeroDivisionError:
    pass
else:
    pass
try:
    1 / 0
except* EOFError:
    pass
except* ZeroDivisionError as msg:
    pass
else:
    pass
try:
    1 / 0
except* (EOFError, TypeError, ZeroDivisionError):
    pass
try:
    1 / 0
except* (EOFError, TypeError, ZeroDivisionError) as msg:
    pass
try:
    pass
finally:
    pass
try:
    compile('try:\n    pass\nexcept* Exception as a.b:\n    pass', '?', 'exec')
    compile('try:\n    pass\nexcept* Exception as a[b]:\n    pass', '?', 'exec')
    compile('try:\n    pass\nexcept*:\n    pass', '?', 'exec')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass
print("GrammarTests::test_try_star: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_try_star: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_unary_ops.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_unary_ops() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_unary_ops"
# subject = "cpython.test_grammar.GrammarTests.test_unary_ops"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_unary_ops
"""Auto-ported test: GrammarTests::test_unary_ops (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
x = +1
x = -1
x = ~1
x = ~1 ^ 1 & 1 | 1 & 1 ^ -1
x = -1 * 1 / 1 + 1 * 1 - ---1 * 1
print("GrammarTests::test_unary_ops: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_unary_ops: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_var_annot_basics.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_var_annot_basics() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_var_annot_basics"
# subject = "cpython.test_grammar.GrammarTests.test_var_annot_basics"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_var_annot_basics
"""Auto-ported test: GrammarTests::test_var_annot_basics (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
var1: int = 5
var2: [int, str]
my_lst = [42]

def one():
    return 1
int.new_attr: int
[list][0]: type
my_lst[one() - 1]: int = 5

assert my_lst == [5]
print("GrammarTests::test_var_annot_basics: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_var_annot_basics: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_while.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_while() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_while"
# subject = "cpython.test_grammar.GrammarTests.test_while"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_while
"""Auto-ported test: GrammarTests::test_while (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
while 0:
    pass
while 0:
    pass
else:
    pass
x = 0
while 0:
    x = 1
else:
    x = 2

assert x == 2
print("GrammarTests::test_while: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_while: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/grammar_tests__test_with_statement.py`.
#[test]
fn test_gen_behavior_core_grammar_grammar_tests__test_with_statement() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_with_statement"
# subject = "cpython.test_grammar.GrammarTests.test_with_statement"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_with_statement
"""Auto-ported test: GrammarTests::test_with_statement (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
class manager(object):

    def __enter__(self):
        return (1, 2)

    def __exit__(self, *args):
        pass
with manager():
    pass
with manager() as x:
    pass
with manager() as (x, y):
    pass
with manager(), manager():
    pass
with manager() as x, manager() as y:
    pass
with manager() as x, manager():
    pass
with manager():
    pass
with manager() as x:
    pass
with manager() as (x, y), manager() as z:
    pass
with manager(), manager():
    pass
with manager() as x, manager() as y:
    pass
with manager() as x, manager():
    pass
with manager() as x, manager() as y, manager() as z:
    pass
with manager() as x, manager() as y, manager():
    pass
print("GrammarTests::test_with_statement: ok")
"###);
    assert_output(&out, r###"GrammarTests::test_with_statement: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/token_tests__test_backslash.py`.
#[test]
fn test_gen_behavior_core_grammar_token_tests__test_backslash() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "token_tests__test_backslash"
# subject = "cpython.test_grammar.TokenTests.test_backslash"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::TokenTests::test_backslash
"""Auto-ported test: TokenTests::test_backslash (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
x = 1 + 1

assert x == 2
x = 0

assert x == 0
print("TokenTests::test_backslash: ok")
"###);
    assert_output(&out, r###"TokenTests::test_backslash: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/token_tests__test_floats.py`.
#[test]
fn test_gen_behavior_core_grammar_token_tests__test_floats() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "token_tests__test_floats"
# subject = "cpython.test_grammar.TokenTests.test_floats"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::TokenTests::test_floats
"""Auto-ported test: TokenTests::test_floats (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
x = 3.14
x = 314.0
x = 0.314
x = 0.314
x = 0.314
x = 300000000000000.0
x = 300000000000000.0
x = 3e-14
x = 300000000000000.0
x = 300000000000000.0
x = 30000000000000.0
x = 31000.0
print("TokenTests::test_floats: ok")
"###);
    assert_output(&out, r###"TokenTests::test_floats: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/grammar/token_tests__test_string_literals.py`.
#[test]
fn test_gen_behavior_core_grammar_token_tests__test_string_literals() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "token_tests__test_string_literals"
# subject = "cpython.test_grammar.TokenTests.test_string_literals"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_grammar.py::TokenTests::test_string_literals
"""Auto-ported test: TokenTests::test_string_literals (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
x = ''
y = ''

assert len(x) == 0 and x == y
x = "'"
y = "'"

assert len(x) == 1 and x == y and (ord(x) == 39)
x = '"'
y = '"'

assert len(x) == 1 and x == y and (ord(x) == 34)
x = 'doesn\'t "shrink" does it'
y = 'doesn\'t "shrink" does it'

assert len(x) == 24 and x == y
x = 'does "shrink" doesn\'t it'
y = 'does "shrink" doesn\'t it'

assert len(x) == 24 and x == y
x = '\nThe "quick"\nbrown fox\njumps over\nthe \'lazy\' dog.\n'
y = '\nThe "quick"\nbrown fox\njumps over\nthe \'lazy\' dog.\n'

assert x == y
y = '\nThe "quick"\nbrown fox\njumps over\nthe \'lazy\' dog.\n'

assert x == y
y = '\nThe "quick"\nbrown fox\njumps over\nthe \'lazy\' dog.\n'

assert x == y
y = '\nThe "quick"\nbrown fox\njumps over\nthe \'lazy\' dog.\n'

assert x == y
print("TokenTests::test_string_literals: ok")
"###);
    assert_output(&out, r###"TokenTests::test_string_literals: ok
"###);
}
