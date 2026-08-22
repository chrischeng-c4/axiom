use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/future_stmt/future_test__test_attributes.py`.
#[test]
fn test_gen_behavior_std_libs_future_stmt_future_test__test_attributes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "future_test__test_attributes"
# subject = "cpython.test.test_future_stmt.test_future_flags.FutureTest.test_attributes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future_flags.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_future_flags.py::FutureTest::test_attributes
"""Auto-ported test: FutureTest::test_attributes (CPython 3.12 oracle)."""


import __future__


GOOD_SERIALS = ("alpha", "beta", "candidate", "final")

features = __future__.all_feature_names


def check_release_tuple(value, name):
    assert isinstance(value, tuple), f"{name} isn't tuple"
    assert len(value) == 5, f"{name} isn't 5-tuple"
    major, minor, micro, level, serial = value
    assert isinstance(major, int), f"{name} major isn't int"
    assert isinstance(minor, int), f"{name} minor isn't int"
    assert isinstance(micro, int), f"{name} micro isn't int"
    assert isinstance(level, str), f"{name} level isn't string"
    assert level in GOOD_SERIALS, f"{name} level string has unknown value"
    assert isinstance(serial, int), f"{name} serial isn't int"


for feature in features:
    value = getattr(__future__, feature)

    optional = value.getOptionalRelease()
    mandatory = value.getMandatoryRelease()

    check_release_tuple(optional, "optional")
    if mandatory is not None:
        check_release_tuple(mandatory, "mandatory")
        assert optional < mandatory, "optional not less than mandatory, and mandatory not None"

    assert hasattr(value, "compiler_flag"), "feature is missing a .compiler_flag attr"
    compile("", "<test>", "exec", value.compiler_flag)
    assert isinstance(getattr(value, "compiler_flag"), int), ".compiler_flag isn't int"

print("FutureTest::test_attributes: ok")
"###);
    assert_output(&out, r###"FutureTest::test_attributes: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/future_stmt/future_test__test_badfuture10.py`.
#[test]
fn test_gen_behavior_std_libs_future_stmt_future_test__test_badfuture10() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "future_test__test_badfuture10"
# subject = "cpython.test.test_future_stmt.test_future.FutureTest.test_badfuture10"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_future.py::FutureTest::test_badfuture10
"""Auto-ported test: FutureTest::test_badfuture10 (CPython 3.12 oracle)."""


import __future__
import ast
import unittest
from test.support import import_helper
from test.support.script_helper import spawn_python, kill_python
from textwrap import dedent
import os
import re
import sys


rx = re.compile('\\((\\S+).py, line (\\d+)')

def get_error_location(msg):
    mo = rx.search(str(msg))
    return mo.group(1, 2)


# --- test body ---
def check_syntax_error(err, basename, lineno, offset=1):

    assert '%s.py, line %d' % (basename, lineno) in str(err)

    assert os.path.basename(err.filename) == basename + '.py'

    assert err.lineno == lineno

    assert err.offset == offset
try:
    from test.test_future_stmt import badsyntax_future10
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)
check_syntax_error(cm.exception, 'badsyntax_future10', 3)
print("FutureTest::test_badfuture10: ok")
"###);
    assert_output(&out, r###"FutureTest::test_badfuture10: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/future_stmt/future_test__test_badfuture3.py`.
#[test]
fn test_gen_behavior_std_libs_future_stmt_future_test__test_badfuture3() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "future_test__test_badfuture3"
# subject = "cpython.test.test_future_stmt.test_future.FutureTest.test_badfuture3"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_future.py::FutureTest::test_badfuture3
"""Auto-ported test: FutureTest::test_badfuture3 (CPython 3.12 oracle)."""


import __future__
import ast
import unittest
from test.support import import_helper
from test.support.script_helper import spawn_python, kill_python
from textwrap import dedent
import os
import re
import sys


rx = re.compile('\\((\\S+).py, line (\\d+)')

def get_error_location(msg):
    mo = rx.search(str(msg))
    return mo.group(1, 2)


# --- test body ---
def check_syntax_error(err, basename, lineno, offset=1):

    assert '%s.py, line %d' % (basename, lineno) in str(err)

    assert os.path.basename(err.filename) == basename + '.py'

    assert err.lineno == lineno

    assert err.offset == offset
try:
    from test.test_future_stmt import badsyntax_future3
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)
check_syntax_error(cm.exception, 'badsyntax_future3', 3, 24)
print("FutureTest::test_badfuture3: ok")
"###);
    assert_output(&out, r###"FutureTest::test_badfuture3: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/future_stmt/future_test__test_badfuture4.py`.
#[test]
fn test_gen_behavior_std_libs_future_stmt_future_test__test_badfuture4() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "future_test__test_badfuture4"
# subject = "cpython.test.test_future_stmt.test_future.FutureTest.test_badfuture4"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_future.py::FutureTest::test_badfuture4
"""Auto-ported test: FutureTest::test_badfuture4 (CPython 3.12 oracle)."""


import __future__
import ast
import unittest
from test.support import import_helper
from test.support.script_helper import spawn_python, kill_python
from textwrap import dedent
import os
import re
import sys


rx = re.compile('\\((\\S+).py, line (\\d+)')

def get_error_location(msg):
    mo = rx.search(str(msg))
    return mo.group(1, 2)


# --- test body ---
def check_syntax_error(err, basename, lineno, offset=1):

    assert '%s.py, line %d' % (basename, lineno) in str(err)

    assert os.path.basename(err.filename) == basename + '.py'

    assert err.lineno == lineno

    assert err.offset == offset
try:
    from test.test_future_stmt import badsyntax_future4
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)
check_syntax_error(cm.exception, 'badsyntax_future4', 3)
print("FutureTest::test_badfuture4: ok")
"###);
    assert_output(&out, r###"FutureTest::test_badfuture4: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/future_stmt/future_test__test_badfuture5.py`.
#[test]
fn test_gen_behavior_std_libs_future_stmt_future_test__test_badfuture5() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "future_test__test_badfuture5"
# subject = "cpython.test.test_future_stmt.test_future.FutureTest.test_badfuture5"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_future.py::FutureTest::test_badfuture5
"""Auto-ported test: FutureTest::test_badfuture5 (CPython 3.12 oracle)."""


import __future__
import ast
import unittest
from test.support import import_helper
from test.support.script_helper import spawn_python, kill_python
from textwrap import dedent
import os
import re
import sys


rx = re.compile('\\((\\S+).py, line (\\d+)')

def get_error_location(msg):
    mo = rx.search(str(msg))
    return mo.group(1, 2)


# --- test body ---
def check_syntax_error(err, basename, lineno, offset=1):

    assert '%s.py, line %d' % (basename, lineno) in str(err)

    assert os.path.basename(err.filename) == basename + '.py'

    assert err.lineno == lineno

    assert err.offset == offset
try:
    from test.test_future_stmt import badsyntax_future5
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)
check_syntax_error(cm.exception, 'badsyntax_future5', 4)
print("FutureTest::test_badfuture5: ok")
"###);
    assert_output(&out, r###"FutureTest::test_badfuture5: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/future_stmt/future_test__test_badfuture6.py`.
#[test]
fn test_gen_behavior_std_libs_future_stmt_future_test__test_badfuture6() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "future_test__test_badfuture6"
# subject = "cpython.test.test_future_stmt.test_future.FutureTest.test_badfuture6"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_future.py::FutureTest::test_badfuture6
"""Auto-ported test: FutureTest::test_badfuture6 (CPython 3.12 oracle)."""


import __future__
import ast
import unittest
from test.support import import_helper
from test.support.script_helper import spawn_python, kill_python
from textwrap import dedent
import os
import re
import sys


rx = re.compile('\\((\\S+).py, line (\\d+)')

def get_error_location(msg):
    mo = rx.search(str(msg))
    return mo.group(1, 2)


# --- test body ---
def check_syntax_error(err, basename, lineno, offset=1):

    assert '%s.py, line %d' % (basename, lineno) in str(err)

    assert os.path.basename(err.filename) == basename + '.py'

    assert err.lineno == lineno

    assert err.offset == offset
try:
    from test.test_future_stmt import badsyntax_future6
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)
check_syntax_error(cm.exception, 'badsyntax_future6', 3)
print("FutureTest::test_badfuture6: ok")
"###);
    assert_output(&out, r###"FutureTest::test_badfuture6: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/future_stmt/future_test__test_badfuture7.py`.
#[test]
fn test_gen_behavior_std_libs_future_stmt_future_test__test_badfuture7() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "future_test__test_badfuture7"
# subject = "cpython.test.test_future_stmt.test_future.FutureTest.test_badfuture7"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_future.py::FutureTest::test_badfuture7
"""Auto-ported test: FutureTest::test_badfuture7 (CPython 3.12 oracle)."""


import __future__
import ast
import unittest
from test.support import import_helper
from test.support.script_helper import spawn_python, kill_python
from textwrap import dedent
import os
import re
import sys


rx = re.compile('\\((\\S+).py, line (\\d+)')

def get_error_location(msg):
    mo = rx.search(str(msg))
    return mo.group(1, 2)


# --- test body ---
def check_syntax_error(err, basename, lineno, offset=1):

    assert '%s.py, line %d' % (basename, lineno) in str(err)

    assert os.path.basename(err.filename) == basename + '.py'

    assert err.lineno == lineno

    assert err.offset == offset
try:
    from test.test_future_stmt import badsyntax_future7
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)
check_syntax_error(cm.exception, 'badsyntax_future7', 3, 54)
print("FutureTest::test_badfuture7: ok")
"###);
    assert_output(&out, r###"FutureTest::test_badfuture7: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/future_stmt/future_test__test_badfuture8.py`.
#[test]
fn test_gen_behavior_std_libs_future_stmt_future_test__test_badfuture8() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "future_test__test_badfuture8"
# subject = "cpython.test.test_future_stmt.test_future.FutureTest.test_badfuture8"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_future.py::FutureTest::test_badfuture8
"""Auto-ported test: FutureTest::test_badfuture8 (CPython 3.12 oracle)."""


import __future__
import ast
import unittest
from test.support import import_helper
from test.support.script_helper import spawn_python, kill_python
from textwrap import dedent
import os
import re
import sys


rx = re.compile('\\((\\S+).py, line (\\d+)')

def get_error_location(msg):
    mo = rx.search(str(msg))
    return mo.group(1, 2)


# --- test body ---
def check_syntax_error(err, basename, lineno, offset=1):

    assert '%s.py, line %d' % (basename, lineno) in str(err)

    assert os.path.basename(err.filename) == basename + '.py'

    assert err.lineno == lineno

    assert err.offset == offset
try:
    from test.test_future_stmt import badsyntax_future8
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)
check_syntax_error(cm.exception, 'badsyntax_future8', 3, 24)
print("FutureTest::test_badfuture8: ok")
"###);
    assert_output(&out, r###"FutureTest::test_badfuture8: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/future_stmt/future_test__test_badfuture9.py`.
#[test]
fn test_gen_behavior_std_libs_future_stmt_future_test__test_badfuture9() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "future_test__test_badfuture9"
# subject = "cpython.test.test_future_stmt.test_future.FutureTest.test_badfuture9"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_future.py::FutureTest::test_badfuture9
"""Auto-ported test: FutureTest::test_badfuture9 (CPython 3.12 oracle)."""


import __future__
import ast
import unittest
from test.support import import_helper
from test.support.script_helper import spawn_python, kill_python
from textwrap import dedent
import os
import re
import sys


rx = re.compile('\\((\\S+).py, line (\\d+)')

def get_error_location(msg):
    mo = rx.search(str(msg))
    return mo.group(1, 2)


# --- test body ---
def check_syntax_error(err, basename, lineno, offset=1):

    assert '%s.py, line %d' % (basename, lineno) in str(err)

    assert os.path.basename(err.filename) == basename + '.py'

    assert err.lineno == lineno

    assert err.offset == offset
try:
    from test.test_future_stmt import badsyntax_future9
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)
check_syntax_error(cm.exception, 'badsyntax_future9', 3, 39)
print("FutureTest::test_badfuture9: ok")
"###);
    assert_output(&out, r###"FutureTest::test_badfuture9: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/future_stmt/future_test__test_ensure_flags_dont_clash.py`.
#[test]
fn test_gen_behavior_std_libs_future_stmt_future_test__test_ensure_flags_dont_clash() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "future_test__test_ensure_flags_dont_clash"
# subject = "cpython.test.test_future_stmt.test_future.FutureTest.test_ensure_flags_dont_clash"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_future.py::FutureTest::test_ensure_flags_dont_clash
"""Auto-ported test: FutureTest::test_ensure_flags_dont_clash (CPython 3.12 oracle)."""


import __future__
import ast


flags = {
    f"CO_FUTURE_{future.upper()}": getattr(__future__, future).compiler_flag
    for future in __future__.all_feature_names
}
flags |= {
    flag: getattr(ast, flag)
    for flag in dir(ast)
    if flag.startswith("PyCF_")
}

values = list(flags.values())
assert len(set(values)) == len(values), flags

print("FutureTest::test_ensure_flags_dont_clash: ok")
"###);
    assert_output(&out, r###"FutureTest::test_ensure_flags_dont_clash: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/future_stmt/future_test__test_names.py`.
#[test]
fn test_gen_behavior_std_libs_future_stmt_future_test__test_names() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "future_test__test_names"
# subject = "cpython.test.test_future_stmt.test_future_flags.FutureTest.test_names"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future_flags.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_future_flags.py::FutureTest::test_names
"""Auto-ported test: FutureTest::test_names (CPython 3.12 oracle)."""


import unittest
import __future__


GOOD_SERIALS = ('alpha', 'beta', 'candidate', 'final')

features = __future__.all_feature_names


# --- test body ---
given_feature_names = features[:]
for name in dir(__future__):
    obj = getattr(__future__, name, None)
    if obj is not None and isinstance(obj, __future__._Feature):

        assert name in given_feature_names
        given_feature_names.remove(name)

assert len(given_feature_names) == 0
print("FutureTest::test_names: ok")
"###);
    assert_output(&out, r###"FutureTest::test_names: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/future_stmt/test_future__test_floor_div_operator.py`.
#[test]
fn test_gen_behavior_std_libs_future_stmt_test_future__test_floor_div_operator() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "test_future__test_floor_div_operator"
# subject = "cpython.test.test_future_stmt.test_future_single_import.TestFuture.test_floor_div_operator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future_single_import.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_future_single_import.py::TestFuture::test_floor_div_operator
"""Auto-ported test: TestFuture::test_floor_div_operator (CPython 3.12 oracle)."""


from __future__ import nested_scopes
from __future__ import division
import unittest


x = 2

def nester():
    x = 3

    def inner():
        return x
    return inner()


# --- test body ---

assert 7 // 2 == 3
print("TestFuture::test_floor_div_operator: ok")
"###);
    assert_output(&out, r###"TestFuture::test_floor_div_operator: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/future_stmt/test_future__test_true_div_as_default.py`.
#[test]
fn test_gen_behavior_std_libs_future_stmt_test_future__test_true_div_as_default() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "test_future__test_true_div_as_default"
# subject = "cpython.test.test_future_stmt.test_future_single_import.TestFuture.test_true_div_as_default"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future_single_import.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_future_single_import.py::TestFuture::test_true_div_as_default
"""Auto-ported test: TestFuture::test_true_div_as_default (CPython 3.12 oracle)."""


from __future__ import nested_scopes
from __future__ import division
import unittest


x = 2

def nester():
    x = 3

    def inner():
        return x
    return inner()


# --- test body ---

assert abs(7 / 2 - 3.5) < 1e-07
print("TestFuture::test_true_div_as_default: ok")
"###);
    assert_output(&out, r###"TestFuture::test_true_div_as_default: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/future_stmt/test_multiple_features__test_unicode_literals.py`.
#[test]
fn test_gen_behavior_std_libs_future_stmt_test_multiple_features__test_unicode_literals() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "test_multiple_features__test_unicode_literals"
# subject = "cpython.test.test_future_stmt.test_future_multiple_features.TestMultipleFeatures.test_unicode_literals"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future_multiple_features.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_future_multiple_features.py::TestMultipleFeatures::test_unicode_literals
"""Auto-ported test: TestMultipleFeatures::test_unicode_literals (CPython 3.12 oracle)."""


from __future__ import unicode_literals, print_function
import sys
import unittest
from test import support


# --- test body ---

assert isinstance('', str)
print("TestMultipleFeatures::test_unicode_literals: ok")
"###);
    assert_output(&out, r###"TestMultipleFeatures::test_unicode_literals: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/future_stmt/tests__test_unicode_literals.py`.
#[test]
fn test_gen_behavior_std_libs_future_stmt_tests__test_unicode_literals() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "tests__test_unicode_literals"
# subject = "cpython.test.test_future_stmt.test_future_multiple_imports.Tests.test_unicode_literals"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future_multiple_imports.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_future_multiple_imports.py::Tests::test_unicode_literals
"""Auto-ported test: Tests::test_unicode_literals (CPython 3.12 oracle)."""


from __future__ import unicode_literals
import unittest


# --- test body ---

assert isinstance('literal', str)
print("Tests::test_unicode_literals: ok")
"###);
    assert_output(&out, r###"Tests::test_unicode_literals: ok
"###);
}
