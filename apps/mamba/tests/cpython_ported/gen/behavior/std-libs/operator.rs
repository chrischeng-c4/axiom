use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/operator/arithmetic_matches_builtin_ops.py`.
#[test]
fn test_gen_behavior_std_libs_operator_arithmetic_matches_builtin_ops() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "arithmetic_matches_builtin_ops"
# subject = "operator.add"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.add: binary arithmetic functions (add/sub/mul/truediv/floordiv/mod/pow) match the built-in operators exactly over representative integer inputs"""
import operator

assert operator.add(100, 200) == 300, "add"
assert operator.sub(100, 37) == 63, "sub"
assert operator.mul(12, 13) == 156, "mul"
assert operator.truediv(22, 7) == 22 / 7, "truediv"
assert operator.floordiv(22, 7) == 3, f"floordiv = {operator.floordiv(22, 7)!r}"
assert operator.mod(22, 7) == 1, "mod"
assert operator.pow(3, 4) == 81, "pow"

print("arithmetic_matches_builtin_ops OK")
"###);
    assert_output(&out, r###"arithmetic_matches_builtin_ops OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/attrgetter_dotted_and_multi.py`.
#[test]
fn test_gen_behavior_std_libs_operator_attrgetter_dotted_and_multi() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "attrgetter_dotted_and_multi"
# subject = "operator.attrgetter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.attrgetter: attrgetter reads a single attribute, walks a dotted path through nested objects, and with multiple names returns a tuple in argument order"""
import operator

class Node:
    pass


root = Node()
root.name = "arthur"
root.child = Node()
root.child.name = "thomas"
root.child.child = Node()
root.child.child.name = "johnson"

assert operator.attrgetter("name")(root) == "arthur", "single attr"
assert operator.attrgetter("child.name")(root) == "thomas", "one-level dotted"
assert operator.attrgetter("child.child.name")(root) == "johnson", "two-level dotted"

get_multi = operator.attrgetter("name", "child.name", "child.child.name")
assert get_multi(root) == ("arthur", "thomas", "johnson"), "multi dotted tuple"

print("attrgetter_dotted_and_multi OK")
"###);
    assert_output(&out, r###"attrgetter_dotted_and_multi OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/bitwise_on_ints.py`.
#[test]
fn test_gen_behavior_std_libs_operator_bitwise_on_ints() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "bitwise_on_ints"
# subject = "operator.and_"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.and_: the bitwise functions and_/or_/xor/lshift/rshift compute the same integer results as &, |, ^, <<, >>"""
import operator

assert operator.and_(0xFF, 0x0F) == 0x0F, "and_"
assert operator.or_(0xF0, 0x0F) == 0xFF, "or_"
assert operator.xor(0xFF, 0x0F) == 0xF0, "xor"
assert operator.lshift(1, 10) == 1024, "lshift"
assert operator.rshift(1024, 3) == 128, "rshift"

print("bitwise_on_ints OK")
"###);
    assert_output(&out, r###"bitwise_on_ints OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_abs.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_abs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_abs"
# subject = "cpython.test_operator.COperatorTestCase.test_abs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_abs
"""Auto-ported test: COperatorTestCase::test_abs (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.abs()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.abs(None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.abs(-1) == 1

assert operator.abs(1) == 1
print("COperatorTestCase::test_abs: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_abs: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_add.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_add() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_add"
# subject = "cpython.test_operator.COperatorTestCase.test_add"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_add
"""Auto-ported test: COperatorTestCase::test_add (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.add()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.add(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.add(3, 4) == 7
print("COperatorTestCase::test_add: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_add: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_attrgetter.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_attrgetter() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_attrgetter"
# subject = "cpython.test_operator.COperatorTestCase.test_attrgetter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_attrgetter
"""Auto-ported test: COperatorTestCase::test_attrgetter (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

class A:
    pass
a = A()
a.name = 'arthur'
f = operator.attrgetter('name')

assert f(a) == 'arthur'

try:
    f()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    f(a, 'dent')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    f(a, surname='dent')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
f = operator.attrgetter('rank')

try:
    f(a)
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass

try:
    operator.attrgetter(2)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.attrgetter()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
record = A()
record.x = 'X'
record.y = 'Y'
record.z = 'Z'

assert operator.attrgetter('x', 'z', 'y')(record) == ('X', 'Z', 'Y')

try:
    operator.attrgetter(('x', (), 'y'))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class C(object):

    def __getattr__(self, name):
        raise SyntaxError

try:
    operator.attrgetter('foo')(C())
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass
a = A()
a.name = 'arthur'
a.child = A()
a.child.name = 'thomas'
f = operator.attrgetter('child.name')

assert f(a) == 'thomas'

try:
    f(a.child)
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
f = operator.attrgetter('name', 'child.name')

assert f(a) == ('arthur', 'thomas')
f = operator.attrgetter('name', 'child.name', 'child.child.name')

try:
    f(a)
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
f = operator.attrgetter('child.')

try:
    f(a)
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
f = operator.attrgetter('.child')

try:
    f(a)
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
a.child.child = A()
a.child.child.name = 'johnson'
f = operator.attrgetter('child.child.name')

assert f(a) == 'johnson'
f = operator.attrgetter('name', 'child.name', 'child.child.name')

assert f(a) == ('arthur', 'thomas', 'johnson')
print("COperatorTestCase::test_attrgetter: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_attrgetter: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_bitwise_and.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_bitwise_and() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_bitwise_and"
# subject = "cpython.test_operator.COperatorTestCase.test_bitwise_and"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_bitwise_and
"""Auto-ported test: COperatorTestCase::test_bitwise_and (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.and_()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.and_(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.and_(15, 10) == 10
print("COperatorTestCase::test_bitwise_and: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_bitwise_and: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_bitwise_or.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_bitwise_or() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_bitwise_or"
# subject = "cpython.test_operator.COperatorTestCase.test_bitwise_or"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_bitwise_or
"""Auto-ported test: COperatorTestCase::test_bitwise_or (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.or_()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.or_(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.or_(10, 5) == 15
print("COperatorTestCase::test_bitwise_or: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_bitwise_or: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_bitwise_xor.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_bitwise_xor() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_bitwise_xor"
# subject = "cpython.test_operator.COperatorTestCase.test_bitwise_xor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_bitwise_xor
"""Auto-ported test: COperatorTestCase::test_bitwise_xor (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.xor()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.xor(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.xor(11, 12) == 7
print("COperatorTestCase::test_bitwise_xor: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_bitwise_xor: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_concat.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_concat() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_concat"
# subject = "cpython.test_operator.COperatorTestCase.test_concat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_concat
"""Auto-ported test: COperatorTestCase::test_concat (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.concat()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.concat(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.concat('py', 'thon') == 'python'

assert operator.concat([1, 2], [3, 4]) == [1, 2, 3, 4]

assert operator.concat(Seq1([5, 6]), Seq1([7])) == [5, 6, 7]

assert operator.concat(Seq2([5, 6]), Seq2([7])) == [5, 6, 7]

try:
    operator.concat(13, 29)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("COperatorTestCase::test_concat: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_concat: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_contains.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_contains() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_contains"
# subject = "cpython.test_operator.COperatorTestCase.test_contains"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_contains
"""Auto-ported test: COperatorTestCase::test_contains (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.contains()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.contains(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.contains(BadIterable(), 1)
    raise AssertionError('expected ZeroDivisionError')
except ZeroDivisionError:
    pass

assert operator.contains(range(4), 2)

assert not operator.contains(range(4), 5)
print("COperatorTestCase::test_contains: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_contains: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_count_of.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_count_of() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_count_of"
# subject = "cpython.test_operator.COperatorTestCase.test_countOf"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_countOf
"""Auto-ported test: COperatorTestCase::test_countOf (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.countOf()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.countOf(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.countOf(BadIterable(), 1)
    raise AssertionError('expected ZeroDivisionError')
except ZeroDivisionError:
    pass

assert operator.countOf([1, 2, 1, 3, 1, 4], 3) == 1

assert operator.countOf([1, 2, 1, 3, 1, 4], 5) == 0
nan = float('nan')

assert operator.countOf([nan, nan, 21], nan) == 2

assert operator.countOf([{}, 1, {}, 2], {}) == 2
print("COperatorTestCase::test_countOf: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_countOf: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_delitem.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_delitem() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_delitem"
# subject = "cpython.test_operator.COperatorTestCase.test_delitem"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_delitem
"""Auto-ported test: COperatorTestCase::test_delitem (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module
a = [4, 3, 2, 1]

try:
    operator.delitem(a)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.delitem(a, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.delitem(a, 1) is None

assert a == [4, 2, 1]
print("COperatorTestCase::test_delitem: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_delitem: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_dunder_is_original.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_dunder_is_original() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_dunder_is_original"
# subject = "cpython.test_operator.COperatorTestCase.test_dunder_is_original"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_dunder_is_original
"""Auto-ported test: COperatorTestCase::test_dunder_is_original (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module
names = [name for name in dir(operator) if not name.startswith('_')]
for name in names:
    orig = getattr(operator, name)
    dunder = getattr(operator, '__' + name.strip('_') + '__', None)
    if dunder:

        assert dunder is orig
print("COperatorTestCase::test_dunder_is_original: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_dunder_is_original: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_eq.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_eq() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_eq"
# subject = "cpython.test_operator.COperatorTestCase.test_eq"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_eq
"""Auto-ported test: COperatorTestCase::test_eq (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

class C(object):

    def __eq__(self, other):
        raise SyntaxError

try:
    operator.eq()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.eq(C(), C())
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass

assert not operator.eq(1, 0)

assert not operator.eq(1, 0.0)

assert operator.eq(1, 1)

assert operator.eq(1, 1.0)

assert not operator.eq(1, 2)

assert not operator.eq(1, 2.0)
print("COperatorTestCase::test_eq: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_eq: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_floordiv.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_floordiv() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_floordiv"
# subject = "cpython.test_operator.COperatorTestCase.test_floordiv"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_floordiv
"""Auto-ported test: COperatorTestCase::test_floordiv (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.floordiv(5)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.floordiv(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.floordiv(5, 2) == 2
print("COperatorTestCase::test_floordiv: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_floordiv: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_ge.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_ge() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_ge"
# subject = "cpython.test_operator.COperatorTestCase.test_ge"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_ge
"""Auto-ported test: COperatorTestCase::test_ge (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.ge()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.ge(1j, 2j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.ge(1, 0)

assert operator.ge(1, 0.0)

assert operator.ge(1, 1)

assert operator.ge(1, 1.0)

assert not operator.ge(1, 2)

assert not operator.ge(1, 2.0)
print("COperatorTestCase::test_ge: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_ge: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_getitem.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_getitem() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_getitem"
# subject = "cpython.test_operator.COperatorTestCase.test_getitem"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_getitem
"""Auto-ported test: COperatorTestCase::test_getitem (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module
a = range(10)

try:
    operator.getitem()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.getitem(a, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.getitem(a, 2) == 2
print("COperatorTestCase::test_getitem: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_getitem: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_gt.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_gt() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_gt"
# subject = "cpython.test_operator.COperatorTestCase.test_gt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_gt
"""Auto-ported test: COperatorTestCase::test_gt (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.gt()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.gt(1j, 2j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.gt(1, 0)

assert operator.gt(1, 0.0)

assert not operator.gt(1, 1)

assert not operator.gt(1, 1.0)

assert not operator.gt(1, 2)

assert not operator.gt(1, 2.0)
print("COperatorTestCase::test_gt: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_gt: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_iconcat_without_getitem.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_iconcat_without_getitem() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_iconcat_without_getitem"
# subject = "cpython.test_operator.COperatorTestCase.test_iconcat_without_getitem"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_iconcat_without_getitem
"""Auto-ported test: COperatorTestCase::test_iconcat_without_getitem (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module
msg = "'int' object can't be concatenated"
try:
    operator.iconcat(1, 0.5)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(msg, str(_aR_e))
print("COperatorTestCase::test_iconcat_without_getitem: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_iconcat_without_getitem: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_index_of.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_index_of() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_index_of"
# subject = "cpython.test_operator.COperatorTestCase.test_indexOf"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_indexOf
"""Auto-ported test: COperatorTestCase::test_indexOf (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.indexOf()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.indexOf(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.indexOf(BadIterable(), 1)
    raise AssertionError('expected ZeroDivisionError')
except ZeroDivisionError:
    pass

assert operator.indexOf([4, 3, 2, 1], 3) == 1

try:
    operator.indexOf([4, 3, 2, 1], 0)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
nan = float('nan')

assert operator.indexOf([nan, nan, 21], nan) == 0

assert operator.indexOf([{}, 1, {}, 2], {}) == 0
it = iter('leave the iterator at exactly the position after the match')

assert operator.indexOf(it, 'a') == 2

assert next(it) == 'v'
print("COperatorTestCase::test_indexOf: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_indexOf: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_inplace.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_inplace() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_inplace"
# subject = "cpython.test_operator.COperatorTestCase.test_inplace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_inplace
"""Auto-ported test: COperatorTestCase::test_inplace (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

class C(object):

    def __iadd__(self, other):
        return 'iadd'

    def __iand__(self, other):
        return 'iand'

    def __ifloordiv__(self, other):
        return 'ifloordiv'

    def __ilshift__(self, other):
        return 'ilshift'

    def __imod__(self, other):
        return 'imod'

    def __imul__(self, other):
        return 'imul'

    def __imatmul__(self, other):
        return 'imatmul'

    def __ior__(self, other):
        return 'ior'

    def __ipow__(self, other):
        return 'ipow'

    def __irshift__(self, other):
        return 'irshift'

    def __isub__(self, other):
        return 'isub'

    def __itruediv__(self, other):
        return 'itruediv'

    def __ixor__(self, other):
        return 'ixor'

    def __getitem__(self, other):
        return 5
c = C()

assert operator.iadd(c, 5) == 'iadd'

assert operator.iand(c, 5) == 'iand'

assert operator.ifloordiv(c, 5) == 'ifloordiv'

assert operator.ilshift(c, 5) == 'ilshift'

assert operator.imod(c, 5) == 'imod'

assert operator.imul(c, 5) == 'imul'

assert operator.imatmul(c, 5) == 'imatmul'

assert operator.ior(c, 5) == 'ior'

assert operator.ipow(c, 5) == 'ipow'

assert operator.irshift(c, 5) == 'irshift'

assert operator.isub(c, 5) == 'isub'

assert operator.itruediv(c, 5) == 'itruediv'

assert operator.ixor(c, 5) == 'ixor'

assert operator.iconcat(c, c) == 'iadd'
print("COperatorTestCase::test_inplace: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_inplace: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_invert.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_invert() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_invert"
# subject = "cpython.test_operator.COperatorTestCase.test_invert"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_invert
"""Auto-ported test: COperatorTestCase::test_invert (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.invert()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.invert(None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.inv(4) == -5
print("COperatorTestCase::test_invert: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_invert: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_is.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_is() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_is"
# subject = "cpython.test_operator.COperatorTestCase.test_is"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_is
"""Auto-ported test: COperatorTestCase::test_is (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module
a = b = 'xyzpdq'
c = a[:3] + b[3:]

try:
    operator.is_()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.is_(a, b)

assert not operator.is_(a, c)
print("COperatorTestCase::test_is: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_is: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_is_not.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_is_not() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_is_not"
# subject = "cpython.test_operator.COperatorTestCase.test_is_not"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_is_not
"""Auto-ported test: COperatorTestCase::test_is_not (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module
a = b = 'xyzpdq'
c = a[:3] + b[3:]

try:
    operator.is_not()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert not operator.is_not(a, b)

assert operator.is_not(a, c)
print("COperatorTestCase::test_is_not: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_is_not: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_le.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_le() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_le"
# subject = "cpython.test_operator.COperatorTestCase.test_le"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_le
"""Auto-ported test: COperatorTestCase::test_le (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.le()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.le(1j, 2j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert not operator.le(1, 0)

assert not operator.le(1, 0.0)

assert operator.le(1, 1)

assert operator.le(1, 1.0)

assert operator.le(1, 2)

assert operator.le(1, 2.0)
print("COperatorTestCase::test_le: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_le: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_lshift.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_lshift() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_lshift"
# subject = "cpython.test_operator.COperatorTestCase.test_lshift"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_lshift
"""Auto-ported test: COperatorTestCase::test_lshift (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.lshift()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.lshift(None, 42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.lshift(5, 1) == 10

assert operator.lshift(5, 0) == 5

try:
    operator.lshift(2, -1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("COperatorTestCase::test_lshift: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_lshift: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_lt.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_lt() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_lt"
# subject = "cpython.test_operator.COperatorTestCase.test_lt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_lt
"""Auto-ported test: COperatorTestCase::test_lt (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.lt()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.lt(1j, 2j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert not operator.lt(1, 0)

assert not operator.lt(1, 0.0)

assert not operator.lt(1, 1)

assert not operator.lt(1, 1.0)

assert operator.lt(1, 2)

assert operator.lt(1, 2.0)
print("COperatorTestCase::test_lt: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_lt: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_matmul.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_matmul() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_matmul"
# subject = "cpython.test_operator.COperatorTestCase.test_matmul"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_matmul
"""Auto-ported test: COperatorTestCase::test_matmul (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.matmul()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.matmul(42, 42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class M:

    def __matmul__(self, other):
        return other - 1

assert M() @ 42 == 41
print("COperatorTestCase::test_matmul: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_matmul: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_mod.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_mod() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_mod"
# subject = "cpython.test_operator.COperatorTestCase.test_mod"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_mod
"""Auto-ported test: COperatorTestCase::test_mod (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.mod()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.mod(None, 42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.mod(5, 2) == 1
print("COperatorTestCase::test_mod: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_mod: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_mul.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_mul() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_mul"
# subject = "cpython.test_operator.COperatorTestCase.test_mul"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_mul
"""Auto-ported test: COperatorTestCase::test_mul (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.mul()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.mul(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.mul(5, 2) == 10
print("COperatorTestCase::test_mul: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_mul: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_ne.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_ne() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_ne"
# subject = "cpython.test_operator.COperatorTestCase.test_ne"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_ne
"""Auto-ported test: COperatorTestCase::test_ne (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

class C(object):

    def __ne__(self, other):
        raise SyntaxError

try:
    operator.ne()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.ne(C(), C())
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass

assert operator.ne(1, 0)

assert operator.ne(1, 0.0)

assert not operator.ne(1, 1)

assert not operator.ne(1, 1.0)

assert operator.ne(1, 2)

assert operator.ne(1, 2.0)
print("COperatorTestCase::test_ne: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_ne: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_neg.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_neg() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_neg"
# subject = "cpython.test_operator.COperatorTestCase.test_neg"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_neg
"""Auto-ported test: COperatorTestCase::test_neg (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.neg()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.neg(None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.neg(5) == -5

assert operator.neg(-5) == 5

assert operator.neg(0) == 0

assert operator.neg(-0) == 0
print("COperatorTestCase::test_neg: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_neg: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_not.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_not() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_not"
# subject = "cpython.test_operator.COperatorTestCase.test_not_"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_not_
"""Auto-ported test: COperatorTestCase::test_not_ (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

class C:

    def __bool__(self):
        raise SyntaxError

try:
    operator.not_()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.not_(C())
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass

assert not operator.not_(5)

assert not operator.not_([0])

assert operator.not_(0)

assert operator.not_([])
print("COperatorTestCase::test_not_: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_not_: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_pos.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_pos() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_pos"
# subject = "cpython.test_operator.COperatorTestCase.test_pos"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_pos
"""Auto-ported test: COperatorTestCase::test_pos (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.pos()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.pos(None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.pos(5) == 5

assert operator.pos(-5) == -5

assert operator.pos(0) == 0

assert operator.pos(-0) == 0
print("COperatorTestCase::test_pos: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_pos: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_pow.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_pow() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_pow"
# subject = "cpython.test_operator.COperatorTestCase.test_pow"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_pow
"""Auto-ported test: COperatorTestCase::test_pow (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.pow()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.pow(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.pow(3, 5) == 3 ** 5

try:
    operator.pow(1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.pow(1, 2, 3)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("COperatorTestCase::test_pow: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_pow: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_rshift.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_rshift() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_rshift"
# subject = "cpython.test_operator.COperatorTestCase.test_rshift"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_rshift
"""Auto-ported test: COperatorTestCase::test_rshift (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.rshift()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.rshift(None, 42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.rshift(5, 1) == 2

assert operator.rshift(5, 0) == 5

try:
    operator.rshift(2, -1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("COperatorTestCase::test_rshift: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_rshift: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_setitem.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_setitem() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_setitem"
# subject = "cpython.test_operator.COperatorTestCase.test_setitem"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_setitem
"""Auto-ported test: COperatorTestCase::test_setitem (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module
a = list(range(3))

try:
    operator.setitem(a)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.setitem(a, None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.setitem(a, 0, 2) is None

assert a == [2, 1, 2]

try:
    operator.setitem(a, 4, 2)
    raise AssertionError('expected IndexError')
except IndexError:
    pass
print("COperatorTestCase::test_setitem: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_setitem: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_sub.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_sub() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_sub"
# subject = "cpython.test_operator.COperatorTestCase.test_sub"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_sub
"""Auto-ported test: COperatorTestCase::test_sub (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.sub()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.sub(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.sub(5, 2) == 3
print("COperatorTestCase::test_sub: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_sub: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_truediv.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_truediv() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_truediv"
# subject = "cpython.test_operator.COperatorTestCase.test_truediv"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_truediv
"""Auto-ported test: COperatorTestCase::test_truediv (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

try:
    operator.truediv(5)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.truediv(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.truediv(5, 2) == 2.5
print("COperatorTestCase::test_truediv: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_truediv: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_operator_test_case__test_truth.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_operator_test_case__test_truth() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_operator_test_case__test_truth"
# subject = "cpython.test_operator.COperatorTestCase.test_truth"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::COperatorTestCase::test_truth
"""Auto-ported test: COperatorTestCase::test_truth (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
operator = module

class C(object):

    def __bool__(self):
        raise SyntaxError

try:
    operator.truth()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.truth(C())
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass

assert operator.truth(5)

assert operator.truth([0])

assert not operator.truth(0)

assert not operator.truth([])
print("COperatorTestCase::test_truth: ok")
"###);
    assert_output(&out, r###"COperatorTestCase::test_truth: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_py_operator_pickle_test_case__test_attrgetter.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_py_operator_pickle_test_case__test_attrgetter() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_py_operator_pickle_test_case__test_attrgetter"
# subject = "cpython.test_operator.CPyOperatorPickleTestCase.test_attrgetter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::CPyOperatorPickleTestCase::test_attrgetter
"""Auto-ported test: CPyOperatorPickleTestCase::test_attrgetter (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
module2 = py_operator

def copy(obj, proto):
    with support.swap_item(sys.modules, 'operator', module):
        pickled = pickle.dumps(obj, proto)
    with support.swap_item(sys.modules, 'operator', module2):
        return pickle.loads(pickled)
attrgetter = module.attrgetter

class A:
    pass
a = A()
a.x = 'X'
a.y = 'Y'
a.z = 'Z'
a.t = A()
a.t.u = A()
a.t.u.v = 'V'
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    f = attrgetter('x')
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
    f = attrgetter('x', 'y', 'z')
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
    f = attrgetter('t.u.v')
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
print("CPyOperatorPickleTestCase::test_attrgetter: ok")
"###);
    assert_output(&out, r###"CPyOperatorPickleTestCase::test_attrgetter: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/c_py_operator_pickle_test_case__test_itemgetter.py`.
#[test]
fn test_gen_behavior_std_libs_operator_c_py_operator_pickle_test_case__test_itemgetter() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "c_py_operator_pickle_test_case__test_itemgetter"
# subject = "cpython.test_operator.CPyOperatorPickleTestCase.test_itemgetter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::CPyOperatorPickleTestCase::test_itemgetter
"""Auto-ported test: CPyOperatorPickleTestCase::test_itemgetter (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
module2 = py_operator

def copy(obj, proto):
    with support.swap_item(sys.modules, 'operator', module):
        pickled = pickle.dumps(obj, proto)
    with support.swap_item(sys.modules, 'operator', module2):
        return pickle.loads(pickled)
itemgetter = module.itemgetter
a = 'ABCDE'
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    f = itemgetter(2)
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
    f = itemgetter(2, 0, 4)
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
print("CPyOperatorPickleTestCase::test_itemgetter: ok")
"###);
    assert_output(&out, r###"CPyOperatorPickleTestCase::test_itemgetter: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/call_invokes_callable.py`.
#[test]
fn test_gen_behavior_std_libs_operator_call_invokes_callable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "call_invokes_callable"
# subject = "operator.call"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.call: call(obj, *args, **kwargs) invokes the callable forwarding positional and keyword arguments exactly"""
import operator

def collect(*args, **kwargs):
    return (args, kwargs)


assert operator.call(collect) == ((), {}), "call no args"
assert operator.call(collect, 0, 1) == ((0, 1), {}), "call positional"
assert operator.call(collect, a=2, b=3) == ((), {"a": 2, "b": 3}), "call kwargs"
assert operator.call(collect, 0, a=2) == ((0,), {"a": 2}), "call mixed"

print("call_invokes_callable OK")
"###);
    assert_output(&out, r###"call_invokes_callable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/cc_operator_pickle_test_case__test_attrgetter.py`.
#[test]
fn test_gen_behavior_std_libs_operator_cc_operator_pickle_test_case__test_attrgetter() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "cc_operator_pickle_test_case__test_attrgetter"
# subject = "cpython.test_operator.CCOperatorPickleTestCase.test_attrgetter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::CCOperatorPickleTestCase::test_attrgetter
"""Auto-ported test: CCOperatorPickleTestCase::test_attrgetter (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
module2 = c_operator

def copy(obj, proto):
    with support.swap_item(sys.modules, 'operator', module):
        pickled = pickle.dumps(obj, proto)
    with support.swap_item(sys.modules, 'operator', module2):
        return pickle.loads(pickled)
attrgetter = module.attrgetter

class A:
    pass
a = A()
a.x = 'X'
a.y = 'Y'
a.z = 'Z'
a.t = A()
a.t.u = A()
a.t.u.v = 'V'
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    f = attrgetter('x')
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
    f = attrgetter('x', 'y', 'z')
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
    f = attrgetter('t.u.v')
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
print("CCOperatorPickleTestCase::test_attrgetter: ok")
"###);
    assert_output(&out, r###"CCOperatorPickleTestCase::test_attrgetter: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/cc_operator_pickle_test_case__test_itemgetter.py`.
#[test]
fn test_gen_behavior_std_libs_operator_cc_operator_pickle_test_case__test_itemgetter() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "cc_operator_pickle_test_case__test_itemgetter"
# subject = "cpython.test_operator.CCOperatorPickleTestCase.test_itemgetter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::CCOperatorPickleTestCase::test_itemgetter
"""Auto-ported test: CCOperatorPickleTestCase::test_itemgetter (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = c_operator
module2 = c_operator

def copy(obj, proto):
    with support.swap_item(sys.modules, 'operator', module):
        pickled = pickle.dumps(obj, proto)
    with support.swap_item(sys.modules, 'operator', module2):
        return pickle.loads(pickled)
itemgetter = module.itemgetter
a = 'ABCDE'
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    f = itemgetter(2)
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
    f = itemgetter(2, 0, 4)
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
print("CCOperatorPickleTestCase::test_itemgetter: ok")
"###);
    assert_output(&out, r###"CCOperatorPickleTestCase::test_itemgetter: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/comparison_returns_bool.py`.
#[test]
fn test_gen_behavior_std_libs_operator_comparison_returns_bool() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "comparison_returns_bool"
# subject = "operator.eq"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.eq: the comparison functions eq/ne/lt/le/gt/ge each return the bool singleton (is True / is False) matching the corresponding operator"""
import operator

_cases = [
    (operator.eq, 5, 5, True),
    (operator.ne, 5, 6, True),
    (operator.lt, 3, 5, True),
    (operator.le, 5, 5, True),
    (operator.gt, 7, 3, True),
    (operator.ge, 5, 4, True),
    (operator.eq, 5, 6, False),
    (operator.lt, 5, 3, False),
]
for _op, _a, _b, _expected in _cases:
    _result = _op(_a, _b)
    assert _result is _expected, f"{_op.__name__}({_a},{_b}) = {_result!r}"

print("comparison_returns_bool OK")
"###);
    assert_output(&out, r###"comparison_returns_bool OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/concat_and_length_hint.py`.
#[test]
fn test_gen_behavior_std_libs_operator_concat_and_length_hint() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "concat_and_length_hint"
# subject = "operator.concat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.concat: concat joins two sequences of the same type (str/list/tuple) and length_hint reports the size of sized objects and ranges, with 0 for empty"""
import operator

assert operator.concat("hello ", "world") == "hello world", "concat str"
assert operator.concat([1, 2], [3, 4]) == [1, 2, 3, 4], "concat list"
assert operator.concat((1,), (2, 3)) == (1, 2, 3), "concat tuple"
assert operator.length_hint([]) == 0, "length_hint empty"
assert operator.length_hint([1, 2, 3]) == 3, "length_hint list"
assert operator.length_hint(range(10)) == 10, "length_hint range"

print("concat_and_length_hint OK")
"###);
    assert_output(&out, r###"concat_and_length_hint OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/contains_not_truth_semantics.py`.
#[test]
fn test_gen_behavior_std_libs_operator_contains_not_truth_semantics() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "contains_not_truth_semantics"
# subject = "operator.contains"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.contains: contains tests membership (b in a), not_ returns the boolean negation, and truth maps any object to its bool — all returning the bool singletons"""
import operator

assert operator.contains("hello", "ell") is True, "contains substring"
assert operator.contains([1, 2, 3], 2) is True, "contains element"
assert operator.contains([1, 2, 3], 4) is False, "not contains"
assert operator.not_(0) is True, "not_(0)"
assert operator.not_("non-empty") is False, "not_(non-empty)"
assert operator.not_(False) is True, "not_(False)"
assert operator.truth(0) is False, "truth(0)"
assert operator.truth(1) is True, "truth(1)"
assert operator.truth("") is False, "truth empty str"
assert operator.truth([1]) is True, "truth non-empty"

print("contains_not_truth_semantics OK")
"###);
    assert_output(&out, r###"contains_not_truth_semantics OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/index_uses_dunder_index.py`.
#[test]
fn test_gen_behavior_std_libs_operator_index_uses_dunder_index() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "index_uses_dunder_index"
# subject = "operator.index"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.index: index returns the integer from a type's __index__ and passes true ints through unchanged"""
import operator

class HasIndex:
    def __index__(self):
        return 7


assert operator.index(HasIndex()) == 7, "index via __index__"
assert operator.index(0) == 0, "index of int 0"
assert operator.index(42) == 42, "index of int"

print("index_uses_dunder_index OK")
"###);
    assert_output(&out, r###"index_uses_dunder_index OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/inplace_dispatches_to_dunders.py`.
#[test]
fn test_gen_behavior_std_libs_operator_inplace_dispatches_to_dunders() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "inplace_dispatches_to_dunders"
# subject = "operator.iadd"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.iadd: each in-place function dispatches to its __i*__ dunder; iconcat falls back to __iadd__ when no __iconcat__ exists; in-place numeric ops on built-in ints behave like the binary form"""
import operator

class Recorder:
    """Each in-place dunder returns its own name so dispatch is observable."""

    def __iadd__(self, other):
        return "iadd"

    def __iand__(self, other):
        return "iand"

    def __ifloordiv__(self, other):
        return "ifloordiv"

    def __ilshift__(self, other):
        return "ilshift"

    def __imod__(self, other):
        return "imod"

    def __imul__(self, other):
        return "imul"

    def __imatmul__(self, other):
        return "imatmul"

    def __ior__(self, other):
        return "ior"

    def __ipow__(self, other):
        return "ipow"

    def __irshift__(self, other):
        return "irshift"

    def __isub__(self, other):
        return "isub"

    def __itruediv__(self, other):
        return "itruediv"

    def __ixor__(self, other):
        return "ixor"

    def __getitem__(self, key):
        return 0


r = Recorder()
cases = [
    (operator.iadd, "iadd"),
    (operator.iand, "iand"),
    (operator.ifloordiv, "ifloordiv"),
    (operator.ilshift, "ilshift"),
    (operator.imod, "imod"),
    (operator.imul, "imul"),
    (operator.imatmul, "imatmul"),
    (operator.ior, "ior"),
    (operator.ipow, "ipow"),
    (operator.irshift, "irshift"),
    (operator.isub, "isub"),
    (operator.itruediv, "itruediv"),
    (operator.ixor, "ixor"),
]
for func, expected in cases:
    result = func(r, 5)
    assert result == expected, f"{func.__name__} -> {result!r}"

# iconcat falls back to __iadd__ when no __iconcat__ exists.
assert operator.iconcat(r, r) == "iadd", "iconcat falls back to iadd"

# In-place numeric ops on built-in ints behave like the binary form.
assert operator.iadd(3, 4) == 7, "iadd on int"
assert operator.imul(6, 7) == 42, "imul on int"

print("inplace_dispatches_to_dunders OK")
"###);
    assert_output(&out, r###"inplace_dispatches_to_dunders OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/itemgetter_containers.py`.
#[test]
fn test_gen_behavior_std_libs_operator_itemgetter_containers() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "itemgetter_containers"
# subject = "operator.itemgetter"
# kind = "semantic"
# xfail = "operator.itemgetter(i)(row) returns 0 instead of row[i] under mamba (repo-memory project_mamba_operator_itemgetter_returns_zero)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.itemgetter: a single-key itemgetter indexes lists, tuples and dicts by that key, returning the element at that position/key"""
import operator

_row = [10, 20, 30, 40]
assert operator.itemgetter(2)(_row) == 30, f"list -> {operator.itemgetter(2)(_row)!r}"
assert operator.itemgetter(1)((100, 200, 300)) == 200, "tuple"
assert operator.itemgetter("key")({"key": "val"}) == "val", "dict"

print("itemgetter_containers OK")
"###);
    assert_output(&out, r###"itemgetter_containers OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/itemgetter_multi_index_returns_tuple.py`.
#[test]
fn test_gen_behavior_std_libs_operator_itemgetter_multi_index_returns_tuple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "itemgetter_multi_index_returns_tuple"
# subject = "operator.itemgetter"
# kind = "semantic"
# xfail = "operator.itemgetter(i)(row) returns 0 instead of row[i] under mamba (repo-memory project_mamba_operator_itemgetter_returns_zero)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.itemgetter: a multi-key itemgetter returns a tuple of the indexed elements in argument order"""
import operator

_result = operator.itemgetter(0, 2, 4)([10, 20, 30, 40, 50])
assert _result == (10, 30, 50), f"multi-itemgetter = {_result!r}"

print("itemgetter_multi_index_returns_tuple OK")
"###);
    assert_output(&out, r###"itemgetter_multi_index_returns_tuple OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/itemgetter_slice_negative_and_sortkey.py`.
#[test]
fn test_gen_behavior_std_libs_operator_itemgetter_slice_negative_and_sortkey() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "itemgetter_slice_negative_and_sortkey"
# subject = "operator.itemgetter"
# kind = "semantic"
# xfail = "operator.itemgetter(i)(row) returns 0 instead of row[i] under mamba (repo-memory project_mamba_operator_itemgetter_returns_zero)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.itemgetter: itemgetter accepts negative indices and slice objects as keys and works as a map/sort key over a list of records"""
import operator

text = "ABCDE"
assert operator.itemgetter(-1)(tuple(text)) == "E", "negative index"
assert operator.itemgetter(slice(2, 4))(tuple(text)) == ("C", "D"), "slice key"
assert operator.itemgetter(0)(range(100, 200)) == 100, "range"

inventory = [("apple", 3), ("banana", 2), ("pear", 5), ("orange", 1)]
by_count = operator.itemgetter(1)
assert list(map(by_count, inventory)) == [3, 2, 5, 1], "map key"
assert sorted(inventory, key=by_count) == [
    ("orange", 1), ("banana", 2), ("apple", 3), ("pear", 5)
], "sort key"

print("itemgetter_slice_negative_and_sortkey OK")
"###);
    assert_output(&out, r###"itemgetter_slice_negative_and_sortkey OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/length_hint_len_hint_and_default.py`.
#[test]
fn test_gen_behavior_std_libs_operator_length_hint_len_hint_and_default() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "length_hint_len_hint_and_default"
# subject = "operator.length_hint"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.length_hint: length_hint prefers __len__, falls back to __length_hint__, and returns the supplied default only when no length information is available"""
import operator

class Hinted:
    def __init__(self, value):
        self.value = value

    def __length_hint__(self):
        return self.value


assert operator.length_hint([], 2) == 0, "len wins over default"
assert operator.length_hint(iter([1, 2, 3])) == 3, "iterator length hint"
assert operator.length_hint(Hinted(5)) == 5, "explicit __length_hint__"
assert operator.length_hint(object(), 10) == 10, "fallback default"

print("length_hint_len_hint_and_default OK")
"###);
    assert_output(&out, r###"length_hint_len_hint_and_default OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/matmul_dispatches_to_dunder.py`.
#[test]
fn test_gen_behavior_std_libs_operator_matmul_dispatches_to_dunder() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "matmul_dispatches_to_dunder"
# subject = "operator.matmul"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.matmul: matmul dispatches to a type's __matmul__ dunder"""
import operator

class Mat:
    def __matmul__(self, other):
        return other - 1


assert operator.matmul(Mat(), 42) == 41, "matmul via __matmul__"

print("matmul_dispatches_to_dunder OK")
"###);
    assert_output(&out, r###"matmul_dispatches_to_dunder OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/public_func_aliases_its_dunder.py`.
#[test]
fn test_gen_behavior_std_libs_operator_public_func_aliases_its_dunder() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "public_func_aliases_its_dunder"
# subject = "operator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator: each public function whose name has a dunder twin IS that twin object (operator.add is operator.__add__, etc.) across the whole module surface"""
import operator

# Each public function whose name has a dunder twin IS that twin.
# e.g. operator.add is operator.__add__.
_checked = 0
for _name in (n for n in dir(operator) if not n.startswith("_")):
    _dunder = getattr(operator, "__" + _name.strip("_") + "__", None)
    if _dunder is not None:
        assert _dunder is getattr(operator, _name), f"{_name} not aliased to its dunder"
        _checked += 1
assert _checked > 0, "expected at least one dunder-aliased function"

print("public_func_aliases_its_dunder OK")
"###);
    assert_output(&out, r###"public_func_aliases_its_dunder OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_c_operator_pickle_test_case__test_attrgetter.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_c_operator_pickle_test_case__test_attrgetter() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_c_operator_pickle_test_case__test_attrgetter"
# subject = "cpython.test_operator.PyCOperatorPickleTestCase.test_attrgetter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyCOperatorPickleTestCase::test_attrgetter
"""Auto-ported test: PyCOperatorPickleTestCase::test_attrgetter (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
module2 = c_operator

def copy(obj, proto):
    with support.swap_item(sys.modules, 'operator', module):
        pickled = pickle.dumps(obj, proto)
    with support.swap_item(sys.modules, 'operator', module2):
        return pickle.loads(pickled)
attrgetter = module.attrgetter

class A:
    pass
a = A()
a.x = 'X'
a.y = 'Y'
a.z = 'Z'
a.t = A()
a.t.u = A()
a.t.u.v = 'V'
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    f = attrgetter('x')
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
    f = attrgetter('x', 'y', 'z')
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
    f = attrgetter('t.u.v')
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
print("PyCOperatorPickleTestCase::test_attrgetter: ok")
"###);
    assert_output(&out, r###"PyCOperatorPickleTestCase::test_attrgetter: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_c_operator_pickle_test_case__test_itemgetter.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_c_operator_pickle_test_case__test_itemgetter() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_c_operator_pickle_test_case__test_itemgetter"
# subject = "cpython.test_operator.PyCOperatorPickleTestCase.test_itemgetter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyCOperatorPickleTestCase::test_itemgetter
"""Auto-ported test: PyCOperatorPickleTestCase::test_itemgetter (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
module2 = c_operator

def copy(obj, proto):
    with support.swap_item(sys.modules, 'operator', module):
        pickled = pickle.dumps(obj, proto)
    with support.swap_item(sys.modules, 'operator', module2):
        return pickle.loads(pickled)
itemgetter = module.itemgetter
a = 'ABCDE'
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    f = itemgetter(2)
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
    f = itemgetter(2, 0, 4)
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
print("PyCOperatorPickleTestCase::test_itemgetter: ok")
"###);
    assert_output(&out, r###"PyCOperatorPickleTestCase::test_itemgetter: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_abs.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_abs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_abs"
# subject = "cpython.test_operator.PyOperatorTestCase.test_abs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_abs
"""Auto-ported test: PyOperatorTestCase::test_abs (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.abs()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.abs(None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.abs(-1) == 1

assert operator.abs(1) == 1
print("PyOperatorTestCase::test_abs: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_abs: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_add.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_add() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_add"
# subject = "cpython.test_operator.PyOperatorTestCase.test_add"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_add
"""Auto-ported test: PyOperatorTestCase::test_add (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.add()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.add(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.add(3, 4) == 7
print("PyOperatorTestCase::test_add: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_add: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_attrgetter.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_attrgetter() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_attrgetter"
# subject = "cpython.test_operator.PyOperatorTestCase.test_attrgetter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_attrgetter
"""Auto-ported test: PyOperatorTestCase::test_attrgetter (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

class A:
    pass
a = A()
a.name = 'arthur'
f = operator.attrgetter('name')

assert f(a) == 'arthur'

try:
    f()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    f(a, 'dent')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    f(a, surname='dent')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
f = operator.attrgetter('rank')

try:
    f(a)
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass

try:
    operator.attrgetter(2)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.attrgetter()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
record = A()
record.x = 'X'
record.y = 'Y'
record.z = 'Z'

assert operator.attrgetter('x', 'z', 'y')(record) == ('X', 'Z', 'Y')

try:
    operator.attrgetter(('x', (), 'y'))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class C(object):

    def __getattr__(self, name):
        raise SyntaxError

try:
    operator.attrgetter('foo')(C())
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass
a = A()
a.name = 'arthur'
a.child = A()
a.child.name = 'thomas'
f = operator.attrgetter('child.name')

assert f(a) == 'thomas'

try:
    f(a.child)
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
f = operator.attrgetter('name', 'child.name')

assert f(a) == ('arthur', 'thomas')
f = operator.attrgetter('name', 'child.name', 'child.child.name')

try:
    f(a)
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
f = operator.attrgetter('child.')

try:
    f(a)
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
f = operator.attrgetter('.child')

try:
    f(a)
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
a.child.child = A()
a.child.child.name = 'johnson'
f = operator.attrgetter('child.child.name')

assert f(a) == 'johnson'
f = operator.attrgetter('name', 'child.name', 'child.child.name')

assert f(a) == ('arthur', 'thomas', 'johnson')
print("PyOperatorTestCase::test_attrgetter: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_attrgetter: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_bitwise_and.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_bitwise_and() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_bitwise_and"
# subject = "cpython.test_operator.PyOperatorTestCase.test_bitwise_and"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_bitwise_and
"""Auto-ported test: PyOperatorTestCase::test_bitwise_and (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.and_()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.and_(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.and_(15, 10) == 10
print("PyOperatorTestCase::test_bitwise_and: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_bitwise_and: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_bitwise_or.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_bitwise_or() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_bitwise_or"
# subject = "cpython.test_operator.PyOperatorTestCase.test_bitwise_or"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_bitwise_or
"""Auto-ported test: PyOperatorTestCase::test_bitwise_or (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.or_()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.or_(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.or_(10, 5) == 15
print("PyOperatorTestCase::test_bitwise_or: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_bitwise_or: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_bitwise_xor.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_bitwise_xor() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_bitwise_xor"
# subject = "cpython.test_operator.PyOperatorTestCase.test_bitwise_xor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_bitwise_xor
"""Auto-ported test: PyOperatorTestCase::test_bitwise_xor (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.xor()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.xor(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.xor(11, 12) == 7
print("PyOperatorTestCase::test_bitwise_xor: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_bitwise_xor: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_concat.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_concat() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_concat"
# subject = "cpython.test_operator.PyOperatorTestCase.test_concat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_concat
"""Auto-ported test: PyOperatorTestCase::test_concat (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.concat()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.concat(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.concat('py', 'thon') == 'python'

assert operator.concat([1, 2], [3, 4]) == [1, 2, 3, 4]

assert operator.concat(Seq1([5, 6]), Seq1([7])) == [5, 6, 7]

assert operator.concat(Seq2([5, 6]), Seq2([7])) == [5, 6, 7]

try:
    operator.concat(13, 29)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("PyOperatorTestCase::test_concat: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_concat: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_contains.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_contains() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_contains"
# subject = "cpython.test_operator.PyOperatorTestCase.test_contains"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_contains
"""Auto-ported test: PyOperatorTestCase::test_contains (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.contains()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.contains(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.contains(BadIterable(), 1)
    raise AssertionError('expected ZeroDivisionError')
except ZeroDivisionError:
    pass

assert operator.contains(range(4), 2)

assert not operator.contains(range(4), 5)
print("PyOperatorTestCase::test_contains: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_contains: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_count_of.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_count_of() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_count_of"
# subject = "cpython.test_operator.PyOperatorTestCase.test_countOf"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_countOf
"""Auto-ported test: PyOperatorTestCase::test_countOf (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.countOf()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.countOf(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.countOf(BadIterable(), 1)
    raise AssertionError('expected ZeroDivisionError')
except ZeroDivisionError:
    pass

assert operator.countOf([1, 2, 1, 3, 1, 4], 3) == 1

assert operator.countOf([1, 2, 1, 3, 1, 4], 5) == 0
nan = float('nan')

assert operator.countOf([nan, nan, 21], nan) == 2

assert operator.countOf([{}, 1, {}, 2], {}) == 2
print("PyOperatorTestCase::test_countOf: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_countOf: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_delitem.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_delitem() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_delitem"
# subject = "cpython.test_operator.PyOperatorTestCase.test_delitem"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_delitem
"""Auto-ported test: PyOperatorTestCase::test_delitem (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module
a = [4, 3, 2, 1]

try:
    operator.delitem(a)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.delitem(a, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.delitem(a, 1) is None

assert a == [4, 2, 1]
print("PyOperatorTestCase::test_delitem: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_delitem: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_dunder_is_original.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_dunder_is_original() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_dunder_is_original"
# subject = "cpython.test_operator.PyOperatorTestCase.test_dunder_is_original"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_dunder_is_original
"""Auto-ported test: PyOperatorTestCase::test_dunder_is_original (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module
names = [name for name in dir(operator) if not name.startswith('_')]
for name in names:
    orig = getattr(operator, name)
    dunder = getattr(operator, '__' + name.strip('_') + '__', None)
    if dunder:

        assert dunder is orig
print("PyOperatorTestCase::test_dunder_is_original: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_dunder_is_original: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_eq.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_eq() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_eq"
# subject = "cpython.test_operator.PyOperatorTestCase.test_eq"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_eq
"""Auto-ported test: PyOperatorTestCase::test_eq (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

class C(object):

    def __eq__(self, other):
        raise SyntaxError

try:
    operator.eq()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.eq(C(), C())
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass

assert not operator.eq(1, 0)

assert not operator.eq(1, 0.0)

assert operator.eq(1, 1)

assert operator.eq(1, 1.0)

assert not operator.eq(1, 2)

assert not operator.eq(1, 2.0)
print("PyOperatorTestCase::test_eq: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_eq: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_floordiv.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_floordiv() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_floordiv"
# subject = "cpython.test_operator.PyOperatorTestCase.test_floordiv"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_floordiv
"""Auto-ported test: PyOperatorTestCase::test_floordiv (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.floordiv(5)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.floordiv(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.floordiv(5, 2) == 2
print("PyOperatorTestCase::test_floordiv: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_floordiv: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_ge.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_ge() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_ge"
# subject = "cpython.test_operator.PyOperatorTestCase.test_ge"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_ge
"""Auto-ported test: PyOperatorTestCase::test_ge (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.ge()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.ge(1j, 2j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.ge(1, 0)

assert operator.ge(1, 0.0)

assert operator.ge(1, 1)

assert operator.ge(1, 1.0)

assert not operator.ge(1, 2)

assert not operator.ge(1, 2.0)
print("PyOperatorTestCase::test_ge: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_ge: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_getitem.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_getitem() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_getitem"
# subject = "cpython.test_operator.PyOperatorTestCase.test_getitem"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_getitem
"""Auto-ported test: PyOperatorTestCase::test_getitem (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module
a = range(10)

try:
    operator.getitem()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.getitem(a, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.getitem(a, 2) == 2
print("PyOperatorTestCase::test_getitem: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_getitem: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_gt.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_gt() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_gt"
# subject = "cpython.test_operator.PyOperatorTestCase.test_gt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_gt
"""Auto-ported test: PyOperatorTestCase::test_gt (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.gt()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.gt(1j, 2j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.gt(1, 0)

assert operator.gt(1, 0.0)

assert not operator.gt(1, 1)

assert not operator.gt(1, 1.0)

assert not operator.gt(1, 2)

assert not operator.gt(1, 2.0)
print("PyOperatorTestCase::test_gt: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_gt: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_iconcat_without_getitem.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_iconcat_without_getitem() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_iconcat_without_getitem"
# subject = "cpython.test_operator.PyOperatorTestCase.test_iconcat_without_getitem"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_iconcat_without_getitem
"""Auto-ported test: PyOperatorTestCase::test_iconcat_without_getitem (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module
msg = "'int' object can't be concatenated"
try:
    operator.iconcat(1, 0.5)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(msg, str(_aR_e))
print("PyOperatorTestCase::test_iconcat_without_getitem: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_iconcat_without_getitem: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_index_of.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_index_of() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_index_of"
# subject = "cpython.test_operator.PyOperatorTestCase.test_indexOf"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_indexOf
"""Auto-ported test: PyOperatorTestCase::test_indexOf (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.indexOf()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.indexOf(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.indexOf(BadIterable(), 1)
    raise AssertionError('expected ZeroDivisionError')
except ZeroDivisionError:
    pass

assert operator.indexOf([4, 3, 2, 1], 3) == 1

try:
    operator.indexOf([4, 3, 2, 1], 0)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
nan = float('nan')

assert operator.indexOf([nan, nan, 21], nan) == 0

assert operator.indexOf([{}, 1, {}, 2], {}) == 0
it = iter('leave the iterator at exactly the position after the match')

assert operator.indexOf(it, 'a') == 2

assert next(it) == 'v'
print("PyOperatorTestCase::test_indexOf: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_indexOf: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_inplace.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_inplace() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_inplace"
# subject = "cpython.test_operator.PyOperatorTestCase.test_inplace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_inplace
"""Auto-ported test: PyOperatorTestCase::test_inplace (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

class C(object):

    def __iadd__(self, other):
        return 'iadd'

    def __iand__(self, other):
        return 'iand'

    def __ifloordiv__(self, other):
        return 'ifloordiv'

    def __ilshift__(self, other):
        return 'ilshift'

    def __imod__(self, other):
        return 'imod'

    def __imul__(self, other):
        return 'imul'

    def __imatmul__(self, other):
        return 'imatmul'

    def __ior__(self, other):
        return 'ior'

    def __ipow__(self, other):
        return 'ipow'

    def __irshift__(self, other):
        return 'irshift'

    def __isub__(self, other):
        return 'isub'

    def __itruediv__(self, other):
        return 'itruediv'

    def __ixor__(self, other):
        return 'ixor'

    def __getitem__(self, other):
        return 5
c = C()

assert operator.iadd(c, 5) == 'iadd'

assert operator.iand(c, 5) == 'iand'

assert operator.ifloordiv(c, 5) == 'ifloordiv'

assert operator.ilshift(c, 5) == 'ilshift'

assert operator.imod(c, 5) == 'imod'

assert operator.imul(c, 5) == 'imul'

assert operator.imatmul(c, 5) == 'imatmul'

assert operator.ior(c, 5) == 'ior'

assert operator.ipow(c, 5) == 'ipow'

assert operator.irshift(c, 5) == 'irshift'

assert operator.isub(c, 5) == 'isub'

assert operator.itruediv(c, 5) == 'itruediv'

assert operator.ixor(c, 5) == 'ixor'

assert operator.iconcat(c, c) == 'iadd'
print("PyOperatorTestCase::test_inplace: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_inplace: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_invert.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_invert() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_invert"
# subject = "cpython.test_operator.PyOperatorTestCase.test_invert"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_invert
"""Auto-ported test: PyOperatorTestCase::test_invert (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.invert()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.invert(None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.inv(4) == -5
print("PyOperatorTestCase::test_invert: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_invert: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_is.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_is() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_is"
# subject = "cpython.test_operator.PyOperatorTestCase.test_is"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_is
"""Auto-ported test: PyOperatorTestCase::test_is (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module
a = b = 'xyzpdq'
c = a[:3] + b[3:]

try:
    operator.is_()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.is_(a, b)

assert not operator.is_(a, c)
print("PyOperatorTestCase::test_is: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_is: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_is_not.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_is_not() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_is_not"
# subject = "cpython.test_operator.PyOperatorTestCase.test_is_not"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_is_not
"""Auto-ported test: PyOperatorTestCase::test_is_not (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module
a = b = 'xyzpdq'
c = a[:3] + b[3:]

try:
    operator.is_not()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert not operator.is_not(a, b)

assert operator.is_not(a, c)
print("PyOperatorTestCase::test_is_not: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_is_not: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_le.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_le() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_le"
# subject = "cpython.test_operator.PyOperatorTestCase.test_le"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_le
"""Auto-ported test: PyOperatorTestCase::test_le (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.le()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.le(1j, 2j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert not operator.le(1, 0)

assert not operator.le(1, 0.0)

assert operator.le(1, 1)

assert operator.le(1, 1.0)

assert operator.le(1, 2)

assert operator.le(1, 2.0)
print("PyOperatorTestCase::test_le: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_le: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_lshift.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_lshift() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_lshift"
# subject = "cpython.test_operator.PyOperatorTestCase.test_lshift"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_lshift
"""Auto-ported test: PyOperatorTestCase::test_lshift (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.lshift()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.lshift(None, 42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.lshift(5, 1) == 10

assert operator.lshift(5, 0) == 5

try:
    operator.lshift(2, -1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("PyOperatorTestCase::test_lshift: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_lshift: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_lt.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_lt() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_lt"
# subject = "cpython.test_operator.PyOperatorTestCase.test_lt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_lt
"""Auto-ported test: PyOperatorTestCase::test_lt (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.lt()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.lt(1j, 2j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert not operator.lt(1, 0)

assert not operator.lt(1, 0.0)

assert not operator.lt(1, 1)

assert not operator.lt(1, 1.0)

assert operator.lt(1, 2)

assert operator.lt(1, 2.0)
print("PyOperatorTestCase::test_lt: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_lt: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_matmul.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_matmul() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_matmul"
# subject = "cpython.test_operator.PyOperatorTestCase.test_matmul"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_matmul
"""Auto-ported test: PyOperatorTestCase::test_matmul (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.matmul()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.matmul(42, 42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class M:

    def __matmul__(self, other):
        return other - 1

assert M() @ 42 == 41
print("PyOperatorTestCase::test_matmul: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_matmul: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_mod.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_mod() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_mod"
# subject = "cpython.test_operator.PyOperatorTestCase.test_mod"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_mod
"""Auto-ported test: PyOperatorTestCase::test_mod (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.mod()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.mod(None, 42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.mod(5, 2) == 1
print("PyOperatorTestCase::test_mod: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_mod: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_mul.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_mul() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_mul"
# subject = "cpython.test_operator.PyOperatorTestCase.test_mul"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_mul
"""Auto-ported test: PyOperatorTestCase::test_mul (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.mul()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.mul(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.mul(5, 2) == 10
print("PyOperatorTestCase::test_mul: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_mul: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_ne.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_ne() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_ne"
# subject = "cpython.test_operator.PyOperatorTestCase.test_ne"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_ne
"""Auto-ported test: PyOperatorTestCase::test_ne (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

class C(object):

    def __ne__(self, other):
        raise SyntaxError

try:
    operator.ne()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.ne(C(), C())
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass

assert operator.ne(1, 0)

assert operator.ne(1, 0.0)

assert not operator.ne(1, 1)

assert not operator.ne(1, 1.0)

assert operator.ne(1, 2)

assert operator.ne(1, 2.0)
print("PyOperatorTestCase::test_ne: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_ne: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_neg.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_neg() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_neg"
# subject = "cpython.test_operator.PyOperatorTestCase.test_neg"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_neg
"""Auto-ported test: PyOperatorTestCase::test_neg (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.neg()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.neg(None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.neg(5) == -5

assert operator.neg(-5) == 5

assert operator.neg(0) == 0

assert operator.neg(-0) == 0
print("PyOperatorTestCase::test_neg: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_neg: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_not.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_not() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_not"
# subject = "cpython.test_operator.PyOperatorTestCase.test_not_"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_not_
"""Auto-ported test: PyOperatorTestCase::test_not_ (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

class C:

    def __bool__(self):
        raise SyntaxError

try:
    operator.not_()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.not_(C())
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass

assert not operator.not_(5)

assert not operator.not_([0])

assert operator.not_(0)

assert operator.not_([])
print("PyOperatorTestCase::test_not_: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_not_: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_pos.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_pos() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_pos"
# subject = "cpython.test_operator.PyOperatorTestCase.test_pos"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_pos
"""Auto-ported test: PyOperatorTestCase::test_pos (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.pos()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.pos(None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.pos(5) == 5

assert operator.pos(-5) == -5

assert operator.pos(0) == 0

assert operator.pos(-0) == 0
print("PyOperatorTestCase::test_pos: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_pos: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_pow.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_pow() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_pow"
# subject = "cpython.test_operator.PyOperatorTestCase.test_pow"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_pow
"""Auto-ported test: PyOperatorTestCase::test_pow (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.pow()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.pow(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.pow(3, 5) == 3 ** 5

try:
    operator.pow(1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.pow(1, 2, 3)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("PyOperatorTestCase::test_pow: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_pow: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_rshift.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_rshift() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_rshift"
# subject = "cpython.test_operator.PyOperatorTestCase.test_rshift"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_rshift
"""Auto-ported test: PyOperatorTestCase::test_rshift (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.rshift()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.rshift(None, 42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.rshift(5, 1) == 2

assert operator.rshift(5, 0) == 5

try:
    operator.rshift(2, -1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("PyOperatorTestCase::test_rshift: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_rshift: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_setitem.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_setitem() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_setitem"
# subject = "cpython.test_operator.PyOperatorTestCase.test_setitem"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_setitem
"""Auto-ported test: PyOperatorTestCase::test_setitem (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module
a = list(range(3))

try:
    operator.setitem(a)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.setitem(a, None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.setitem(a, 0, 2) is None

assert a == [2, 1, 2]

try:
    operator.setitem(a, 4, 2)
    raise AssertionError('expected IndexError')
except IndexError:
    pass
print("PyOperatorTestCase::test_setitem: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_setitem: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_sub.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_sub() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_sub"
# subject = "cpython.test_operator.PyOperatorTestCase.test_sub"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_sub
"""Auto-ported test: PyOperatorTestCase::test_sub (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.sub()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.sub(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.sub(5, 2) == 3
print("PyOperatorTestCase::test_sub: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_sub: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_truediv.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_truediv() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_truediv"
# subject = "cpython.test_operator.PyOperatorTestCase.test_truediv"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_truediv
"""Auto-ported test: PyOperatorTestCase::test_truediv (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

try:
    operator.truediv(5)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.truediv(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert operator.truediv(5, 2) == 2.5
print("PyOperatorTestCase::test_truediv: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_truediv: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_operator_test_case__test_truth.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_operator_test_case__test_truth() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_operator_test_case__test_truth"
# subject = "cpython.test_operator.PyOperatorTestCase.test_truth"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyOperatorTestCase::test_truth
"""Auto-ported test: PyOperatorTestCase::test_truth (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
operator = module

class C(object):

    def __bool__(self):
        raise SyntaxError

try:
    operator.truth()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.truth(C())
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass

assert operator.truth(5)

assert operator.truth([0])

assert not operator.truth(0)

assert not operator.truth([])
print("PyOperatorTestCase::test_truth: ok")
"###);
    assert_output(&out, r###"PyOperatorTestCase::test_truth: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_py_operator_pickle_test_case__test_attrgetter.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_py_operator_pickle_test_case__test_attrgetter() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_py_operator_pickle_test_case__test_attrgetter"
# subject = "cpython.test_operator.PyPyOperatorPickleTestCase.test_attrgetter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyPyOperatorPickleTestCase::test_attrgetter
"""Auto-ported test: PyPyOperatorPickleTestCase::test_attrgetter (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
module2 = py_operator

def copy(obj, proto):
    with support.swap_item(sys.modules, 'operator', module):
        pickled = pickle.dumps(obj, proto)
    with support.swap_item(sys.modules, 'operator', module2):
        return pickle.loads(pickled)
attrgetter = module.attrgetter

class A:
    pass
a = A()
a.x = 'X'
a.y = 'Y'
a.z = 'Z'
a.t = A()
a.t.u = A()
a.t.u.v = 'V'
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    f = attrgetter('x')
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
    f = attrgetter('x', 'y', 'z')
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
    f = attrgetter('t.u.v')
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
print("PyPyOperatorPickleTestCase::test_attrgetter: ok")
"###);
    assert_output(&out, r###"PyPyOperatorPickleTestCase::test_attrgetter: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/py_py_operator_pickle_test_case__test_itemgetter.py`.
#[test]
fn test_gen_behavior_std_libs_operator_py_py_operator_pickle_test_case__test_itemgetter() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "py_py_operator_pickle_test_case__test_itemgetter"
# subject = "cpython.test_operator.PyPyOperatorPickleTestCase.test_itemgetter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_operator.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_operator.py::PyPyOperatorPickleTestCase::test_itemgetter
"""Auto-ported test: PyPyOperatorPickleTestCase::test_itemgetter (CPython 3.12 oracle)."""


import unittest
import pickle
import sys
from decimal import Decimal
from fractions import Fraction
from test import support
from test.support import import_helper


py_operator = import_helper.import_fresh_module('operator', blocked=['_operator'])

c_operator = import_helper.import_fresh_module('operator', fresh=['_operator'])

class Seq1:

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):

    def __init__(self, lst):
        self.lst = lst

    def __len__(self):
        return len(self.lst)

    def __getitem__(self, i):
        return self.lst[i]

    def __add__(self, other):
        return self.lst + other.lst

    def __mul__(self, other):
        return self.lst * other

    def __rmul__(self, other):
        return other * self.lst

class BadIterable:

    def __iter__(self):
        raise ZeroDivisionError


# --- test body ---
module = py_operator
module2 = py_operator

def copy(obj, proto):
    with support.swap_item(sys.modules, 'operator', module):
        pickled = pickle.dumps(obj, proto)
    with support.swap_item(sys.modules, 'operator', module2):
        return pickle.loads(pickled)
itemgetter = module.itemgetter
a = 'ABCDE'
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    f = itemgetter(2)
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
    f = itemgetter(2, 0, 4)
    f2 = copy(f, proto)

    assert repr(f2) == repr(f)

    assert f2(a) == f(a)
print("PyPyOperatorPickleTestCase::test_itemgetter: ok")
"###);
    assert_output(&out, r###"PyPyOperatorPickleTestCase::test_itemgetter: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/setitem_delitem_mutate_in_place.py`.
#[test]
fn test_gen_behavior_std_libs_operator_setitem_delitem_mutate_in_place() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "setitem_delitem_mutate_in_place"
# subject = "operator.setitem"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.setitem: setitem and delitem mutate lists and dicts in place and both return None"""
import operator

a = list(range(3))
result = operator.setitem(a, 0, 99)
assert result is None, "setitem returns None"
assert a == [99, 1, 2], f"setitem mutated list -> {a!r}"

d = {}
operator.setitem(d, "k", "v")
assert d == {"k": "v"}, "setitem on dict"

b = [4, 3, 2, 1]
result = operator.delitem(b, 1)
assert result is None, "delitem returns None"
assert b == [4, 2, 1], f"delitem mutated list -> {b!r}"

m = {"x": 1, "y": 2}
operator.delitem(m, "x")
assert m == {"y": 2}, "delitem on dict"

print("setitem_delitem_mutate_in_place OK")
"###);
    assert_output(&out, r###"setitem_delitem_mutate_in_place OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/truth_not_propagate_bool_error.py`.
#[test]
fn test_gen_behavior_std_libs_operator_truth_not_propagate_bool_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "truth_not_propagate_bool_error"
# subject = "operator.truth"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.truth: truth and not_ delegate to __bool__; an exception raised inside __bool__ propagates unchanged out of both functions"""
import operator

class BoolRaises:
    def __bool__(self):
        raise SyntaxError("boom")


for _func in (operator.truth, operator.not_):
    _raised = False
    try:
        _func(BoolRaises())
    except SyntaxError:
        _raised = True
    assert _raised, f"{_func.__name__} should propagate __bool__ error"

print("truth_not_propagate_bool_error OK")
"###);
    assert_output(&out, r###"truth_not_propagate_bool_error OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/operator/unary_neg_abs_pos_inv.py`.
#[test]
fn test_gen_behavior_std_libs_operator_unary_neg_abs_pos_inv() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "unary_neg_abs_pos_inv"
# subject = "operator.neg"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.neg: the unary functions neg/abs/pos/inv compute -x, |x|, +x, ~x respectively over positive, negative and boundary integers"""
import operator

assert operator.neg(42) == -42, "neg positive"
assert operator.neg(-42) == 42, "neg negative"
assert operator.abs(-7) == 7, "abs negative"
assert operator.abs(7) == 7, "abs positive"
assert operator.pos(-3) == -3, "pos"
assert operator.inv(0) == -1, "inv 0"
assert operator.inv(-1) == 0, "inv -1"
assert operator.inv(5) == -6, "inv 5"

print("unary_neg_abs_pos_inv OK")
"###);
    assert_output(&out, r###"unary_neg_abs_pos_inv OK
"###);
}
