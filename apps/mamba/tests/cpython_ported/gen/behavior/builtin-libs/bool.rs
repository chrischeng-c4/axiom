use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/builtin-libs/bool/bool_test__test_bool_called_at_least_once.py`.
#[test]
fn test_gen_behavior_builtin_libs_bool_bool_test__test_bool_called_at_least_once() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_bool_called_at_least_once"
# subject = "cpython.test.test_bool.BoolTest.test_bool_called_at_least_once"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_bool_called_at_least_once
"""Auto-ported test: BoolTest::test_bool_called_at_least_once (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---
class X:

    def __init__(self):
        self.count = 0

    def __bool__(self):
        self.count += 1
        return True

def f(x):
    if x or True:
        pass
x = X()
f(x)

assert x.count >= 1
print("BoolTest::test_bool_called_at_least_once: ok")
"###);
    assert_output(&out, r###"BoolTest::test_bool_called_at_least_once: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bool/bool_test__test_boolean.py`.
#[test]
fn test_gen_behavior_builtin_libs_bool_bool_test__test_boolean() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_boolean"
# subject = "cpython.test.test_bool.BoolTest.test_boolean"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_boolean
"""Auto-ported test: BoolTest::test_boolean (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert True & 1 == 1

assert not isinstance(True & 1, bool)

assert True & True is True

assert True | 1 == 1

assert not isinstance(True | 1, bool)

assert True | True is True

assert True ^ 1 == 0

assert not isinstance(True ^ 1, bool)

assert True ^ True is False
print("BoolTest::test_boolean: ok")
"###);
    assert_output(&out, r###"BoolTest::test_boolean: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bool/bool_test__test_callable.py`.
#[test]
fn test_gen_behavior_builtin_libs_bool_bool_test__test_callable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_callable"
# subject = "cpython.test.test_bool.BoolTest.test_callable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_callable
"""Auto-ported test: BoolTest::test_callable (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert callable(len) is True

assert callable(1) is False
print("BoolTest::test_callable: ok")
"###);
    assert_output(&out, r###"BoolTest::test_callable: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bool/bool_test__test_contains.py`.
#[test]
fn test_gen_behavior_builtin_libs_bool_bool_test__test_contains() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_contains"
# subject = "cpython.test.test_bool.BoolTest.test_contains"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_contains
"""Auto-ported test: BoolTest::test_contains (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert (1 in {}) is False

assert (1 in {1: 1}) is True
print("BoolTest::test_contains: ok")
"###);
    assert_output(&out, r###"BoolTest::test_contains: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bool/bool_test__test_fileclosed.py`.
#[test]
fn test_gen_behavior_builtin_libs_bool_bool_test__test_fileclosed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_fileclosed"
# subject = "cpython.test.test_bool.BoolTest.test_fileclosed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_fileclosed
"""Auto-ported test: BoolTest::test_fileclosed (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---
try:
    with open(os_helper.TESTFN, 'w', encoding='utf-8') as f:

        assert f.closed is False

    assert f.closed is True
finally:
    os.remove(os_helper.TESTFN)
print("BoolTest::test_fileclosed: ok")
"###);
    assert_output(&out, r###"BoolTest::test_fileclosed: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bool/bool_test__test_float.py`.
#[test]
fn test_gen_behavior_builtin_libs_bool_bool_test__test_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_float"
# subject = "cpython.test.test_bool.BoolTest.test_float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_float
"""Auto-ported test: BoolTest::test_float (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert float(False) == 0.0

assert float(False) is not False

assert float(True) == 1.0

assert float(True) is not True
print("BoolTest::test_float: ok")
"###);
    assert_output(&out, r###"BoolTest::test_float: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bool/bool_test__test_hasattr.py`.
#[test]
fn test_gen_behavior_builtin_libs_bool_bool_test__test_hasattr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_hasattr"
# subject = "cpython.test.test_bool.BoolTest.test_hasattr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_hasattr
"""Auto-ported test: BoolTest::test_hasattr (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert hasattr([], 'append') is True

assert hasattr([], 'wobble') is False
print("BoolTest::test_hasattr: ok")
"###);
    assert_output(&out, r###"BoolTest::test_hasattr: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bool/bool_test__test_int.py`.
#[test]
fn test_gen_behavior_builtin_libs_bool_bool_test__test_int() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_int"
# subject = "cpython.test.test_bool.BoolTest.test_int"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_int
"""Auto-ported test: BoolTest::test_int (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert int(False) == 0

assert int(False) is not False

assert int(True) == 1

assert int(True) is not True
print("BoolTest::test_int: ok")
"###);
    assert_output(&out, r###"BoolTest::test_int: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bool/bool_test__test_interpreter_convert_to_bool_raises.py`.
#[test]
fn test_gen_behavior_builtin_libs_bool_bool_test__test_interpreter_convert_to_bool_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_interpreter_convert_to_bool_raises"
# subject = "cpython.test.test_bool.BoolTest.test_interpreter_convert_to_bool_raises"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_interpreter_convert_to_bool_raises
"""Auto-ported test: BoolTest::test_interpreter_convert_to_bool_raises (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---
class SymbolicBool:

    def __bool__(self):
        raise TypeError

class Symbol:

    def __gt__(self, other):
        return SymbolicBool()
x = Symbol()
try:
    if x > 0:
        msg = 'x > 0 was true'
    else:
        msg = 'x > 0 was false'
    raise AssertionError('expected TypeError')
except TypeError:
    pass
del x
print("BoolTest::test_interpreter_convert_to_bool_raises: ok")
"###);
    assert_output(&out, r###"BoolTest::test_interpreter_convert_to_bool_raises: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bool/bool_test__test_isinstance.py`.
#[test]
fn test_gen_behavior_builtin_libs_bool_bool_test__test_isinstance() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_isinstance"
# subject = "cpython.test.test_bool.BoolTest.test_isinstance"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_isinstance
"""Auto-ported test: BoolTest::test_isinstance (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert isinstance(True, bool) is True

assert isinstance(False, bool) is True

assert isinstance(True, int) is True

assert isinstance(False, int) is True

assert isinstance(1, bool) is False

assert isinstance(0, bool) is False
print("BoolTest::test_isinstance: ok")
"###);
    assert_output(&out, r###"BoolTest::test_isinstance: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bool/bool_test__test_issubclass.py`.
#[test]
fn test_gen_behavior_builtin_libs_bool_bool_test__test_issubclass() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_issubclass"
# subject = "cpython.test.test_bool.BoolTest.test_issubclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_issubclass
"""Auto-ported test: BoolTest::test_issubclass (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert issubclass(bool, int) is True

assert issubclass(int, bool) is False
print("BoolTest::test_issubclass: ok")
"###);
    assert_output(&out, r###"BoolTest::test_issubclass: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bool/bool_test__test_operator.py`.
#[test]
fn test_gen_behavior_builtin_libs_bool_bool_test__test_operator() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_operator"
# subject = "cpython.test.test_bool.BoolTest.test_operator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_operator
"""Auto-ported test: BoolTest::test_operator (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---
import operator

assert operator.truth(0) is False

assert operator.truth(1) is True

assert operator.not_(1) is False

assert operator.not_(0) is True

assert operator.contains([], 1) is False

assert operator.contains([1], 1) is True

assert operator.lt(0, 0) is False

assert operator.lt(0, 1) is True

assert operator.is_(True, True) is True

assert operator.is_(True, False) is False

assert operator.is_not(True, True) is False

assert operator.is_not(True, False) is True
print("BoolTest::test_operator: ok")
"###);
    assert_output(&out, r###"BoolTest::test_operator: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bool/bool_test__test_pickle.py`.
#[test]
fn test_gen_behavior_builtin_libs_bool_bool_test__test_pickle() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_pickle"
# subject = "cpython.test.test_bool.BoolTest.test_pickle"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_pickle
"""Auto-ported test: BoolTest::test_pickle (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---
import pickle
for proto in range(pickle.HIGHEST_PROTOCOL + 1):

    assert pickle.loads(pickle.dumps(True, proto)) is True

    assert pickle.loads(pickle.dumps(False, proto)) is False
print("BoolTest::test_pickle: ok")
"###);
    assert_output(&out, r###"BoolTest::test_pickle: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bool/bool_test__test_repr.py`.
#[test]
fn test_gen_behavior_builtin_libs_bool_bool_test__test_repr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_repr"
# subject = "cpython.test.test_bool.BoolTest.test_repr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_repr
"""Auto-ported test: BoolTest::test_repr (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert repr(False) == 'False'

assert repr(True) == 'True'

assert eval(repr(False)) is False

assert eval(repr(True)) is True
print("BoolTest::test_repr: ok")
"###);
    assert_output(&out, r###"BoolTest::test_repr: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bool/bool_test__test_str.py`.
#[test]
fn test_gen_behavior_builtin_libs_bool_bool_test__test_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_str"
# subject = "cpython.test.test_bool.BoolTest.test_str"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_str
"""Auto-ported test: BoolTest::test_str (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert str(False) == 'False'

assert str(True) == 'True'
print("BoolTest::test_str: ok")
"###);
    assert_output(&out, r###"BoolTest::test_str: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bool/bool_test__test_string.py`.
#[test]
fn test_gen_behavior_builtin_libs_bool_bool_test__test_string() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_string"
# subject = "cpython.test.test_bool.BoolTest.test_string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_string
"""Auto-ported test: BoolTest::test_string (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert 'xyz'.endswith('z') is True

assert 'xyz'.endswith('x') is False

assert 'xyz0123'.isalnum() is True

assert '@#$%'.isalnum() is False

assert 'xyz'.isalpha() is True

assert '@#$%'.isalpha() is False

assert '0123'.isdigit() is True

assert 'xyz'.isdigit() is False

assert 'xyz'.islower() is True

assert 'XYZ'.islower() is False

assert '0123'.isdecimal() is True

assert 'xyz'.isdecimal() is False

assert '0123'.isnumeric() is True

assert 'xyz'.isnumeric() is False

assert ' '.isspace() is True

assert '\xa0'.isspace() is True

assert '\u3000'.isspace() is True

assert 'XYZ'.isspace() is False

assert 'X'.istitle() is True

assert 'x'.istitle() is False

assert 'XYZ'.isupper() is True

assert 'xyz'.isupper() is False

assert 'xyz'.startswith('x') is True

assert 'xyz'.startswith('z') is False
print("BoolTest::test_string: ok")
"###);
    assert_output(&out, r###"BoolTest::test_string: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bool/bool_test__test_types.py`.
#[test]
fn test_gen_behavior_builtin_libs_bool_bool_test__test_types() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_types"
# subject = "cpython.test.test_bool.BoolTest.test_types"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_types
"""Auto-ported test: BoolTest::test_types (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---
for t in [bool, complex, dict, float, int, list, object, set, str, tuple, type]:

    assert bool(t) is True
print("BoolTest::test_types: ok")
"###);
    assert_output(&out, r###"BoolTest::test_types: ok
"###);
}
