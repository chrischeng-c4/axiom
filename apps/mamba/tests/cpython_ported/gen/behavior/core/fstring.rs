use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/core/fstring/test_case__test_call.py`.
#[test]
fn test_gen_behavior_core_fstring_test_case__test_call() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_call"
# subject = "cpython.test_fstring.TestCase.test_call"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_call
"""Auto-ported test: TestCase::test_call (CPython 3.12 oracle)."""


import ast
import datetime
import os
import re
import types
import decimal
import unittest
import warnings
from test import support
from test.support.os_helper import temp_cwd
from test.support.script_helper import assert_python_failure, assert_python_ok


a_global = 'global variable'


# --- test body ---
def foo(x):
    return 'x=' + str(x)

assert f'{foo(10)}' == 'x=10'
print("TestCase::test_call: ok")
"###);
    assert_output(&out, r###"TestCase::test_call: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/fstring/test_case__test_closure.py`.
#[test]
fn test_gen_behavior_core_fstring_test_case__test_closure() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_closure"
# subject = "cpython.test_fstring.TestCase.test_closure"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_closure
"""Auto-ported test: TestCase::test_closure (CPython 3.12 oracle)."""


import ast
import datetime
import os
import re
import types
import decimal
import unittest
import warnings
from test import support
from test.support.os_helper import temp_cwd
from test.support.script_helper import assert_python_failure, assert_python_ok


a_global = 'global variable'


# --- test body ---
def outer(x):

    def inner():
        return f'x:{x}'
    return inner

assert outer('987')() == 'x:987'

assert outer(7)() == 'x:7'
print("TestCase::test_closure: ok")
"###);
    assert_output(&out, r###"TestCase::test_closure: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/fstring/test_case__test_docstring.py`.
#[test]
fn test_gen_behavior_core_fstring_test_case__test_docstring() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_docstring"
# subject = "cpython.test_fstring.TestCase.test_docstring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_docstring
"""Auto-ported test: TestCase::test_docstring (CPython 3.12 oracle)."""


import ast
import datetime
import os
import re
import types
import decimal
import unittest
import warnings
from test import support
from test.support.os_helper import temp_cwd
from test.support.script_helper import assert_python_failure, assert_python_ok


a_global = 'global variable'


# --- test body ---
def f():
    f'Not a docstring'

assert f.__doc__ is None

def g():
    f'Not a docstring'

assert g.__doc__ is None
print("TestCase::test_docstring: ok")
"###);
    assert_output(&out, r###"TestCase::test_docstring: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/fstring/test_case__test_empty_format_specifier.py`.
#[test]
fn test_gen_behavior_core_fstring_test_case__test_empty_format_specifier() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_empty_format_specifier"
# subject = "cpython.test_fstring.TestCase.test_empty_format_specifier"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_empty_format_specifier
"""Auto-ported test: TestCase::test_empty_format_specifier (CPython 3.12 oracle)."""


import ast
import datetime
import os
import re
import types
import decimal
import unittest
import warnings
from test import support
from test.support.os_helper import temp_cwd
from test.support.script_helper import assert_python_failure, assert_python_ok


a_global = 'global variable'


# --- test body ---
x = 'test'

assert f'{x}' == 'test'

assert f'{x:}' == 'test'

assert f'{x!s:}' == 'test'

assert f'{x!r:}' == "'test'"
print("TestCase::test_empty_format_specifier: ok")
"###);
    assert_output(&out, r###"TestCase::test_empty_format_specifier: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/fstring/test_case__test_equal_equal.py`.
#[test]
fn test_gen_behavior_core_fstring_test_case__test_equal_equal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_equal_equal"
# subject = "cpython.test_fstring.TestCase.test_equal_equal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_equal_equal
"""Auto-ported test: TestCase::test_equal_equal (CPython 3.12 oracle)."""


import ast
import datetime
import os
import re
import types
import decimal
import unittest
import warnings
from test import support
from test.support.os_helper import temp_cwd
from test.support.script_helper import assert_python_failure, assert_python_ok


a_global = 'global variable'


# --- test body ---

assert f'{0 == 1}' == 'False'
print("TestCase::test_equal_equal: ok")
"###);
    assert_output(&out, r###"TestCase::test_equal_equal: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/fstring/test_case__test_fstring_backslash_prefix_raw.py`.
#[test]
fn test_gen_behavior_core_fstring_test_case__test_fstring_backslash_prefix_raw() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_fstring_backslash_prefix_raw"
# subject = "cpython.test_fstring.TestCase.test_fstring_backslash_prefix_raw"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_fstring_backslash_prefix_raw
"""Auto-ported test: TestCase::test_fstring_backslash_prefix_raw (CPython 3.12 oracle)."""


import ast
import datetime
import os
import re
import types
import decimal
import unittest
import warnings
from test import support
from test.support.os_helper import temp_cwd
from test.support.script_helper import assert_python_failure, assert_python_ok


a_global = 'global variable'


# --- test body ---

assert f'\\' == '\\'

assert f'\\\\' == '\\\\'

assert f'\\\\' == '\\\\'

assert f'\\\\\\\\' == '\\\\\\\\'

assert f'\\\\' == '\\\\'

assert f'\\\\\\\\' == '\\\\\\\\'

assert f'\\\\' == '\\\\'

assert f'\\\\\\\\' == '\\\\\\\\'

assert f'\\\\' == '\\\\'

assert f'\\\\\\\\' == '\\\\\\\\'

assert f'\\\\' == '\\\\'

assert f'\\\\\\\\' == '\\\\\\\\'
print("TestCase::test_fstring_backslash_prefix_raw: ok")
"###);
    assert_output(&out, r###"TestCase::test_fstring_backslash_prefix_raw: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/fstring/test_case__test_gh129093.py`.
#[test]
fn test_gen_behavior_core_fstring_test_case__test_gh129093() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_gh129093"
# subject = "cpython.test_fstring.TestCase.test_gh129093"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_gh129093
"""Auto-ported test: TestCase::test_gh129093 (CPython 3.12 oracle)."""


import ast
import datetime
import os
import re
import types
import decimal
import unittest
import warnings
from test import support
from test.support.os_helper import temp_cwd
from test.support.script_helper import assert_python_failure, assert_python_ok


a_global = 'global variable'


# --- test body ---

assert f'1==2={1 == 2!r}' == '1==2=False'

assert f'1 == 2={1 == 2!r}' == '1 == 2=False'

assert f'1!=2={1 != 2!r}' == '1!=2=True'

assert f'1 != 2={1 != 2!r}' == '1 != 2=True'

assert f'(1) != 2={1 != 2!r}' == '(1) != 2=True'

assert f'(1*2) != (3)={1 * 2 != 3!r}' == '(1*2) != (3)=True'

assert f'1 != 2 == 3 != 4={1 != 2 == 3 != 4!r}' == '1 != 2 == 3 != 4=False'

assert f'1 == 2 != 3 == 4={1 == 2 != 3 == 4!r}' == '1 == 2 != 3 == 4=False'

assert f"f'{{1==2=}}'={f'1==2={1 == 2!r}'!r}" == "f'{1==2=}'='1==2=False'"

assert f"f'{{1 == 2=}}'={f'1 == 2={1 == 2!r}'!r}" == "f'{1 == 2=}'='1 == 2=False'"

assert f"f'{{1!=2=}}'={f'1!=2={1 != 2!r}'!r}" == "f'{1!=2=}'='1!=2=True'"

assert f"f'{{1 != 2=}}'={f'1 != 2={1 != 2!r}'!r}" == "f'{1 != 2=}'='1 != 2=True'"
print("TestCase::test_gh129093: ok")
"###);
    assert_output(&out, r###"TestCase::test_gh129093: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/fstring/test_case__test_literal.py`.
#[test]
fn test_gen_behavior_core_fstring_test_case__test_literal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_literal"
# subject = "cpython.test_fstring.TestCase.test_literal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_literal
"""Auto-ported test: TestCase::test_literal (CPython 3.12 oracle)."""


import ast
import datetime
import os
import re
import types
import decimal
import unittest
import warnings
from test import support
from test.support.os_helper import temp_cwd
from test.support.script_helper import assert_python_failure, assert_python_ok


a_global = 'global variable'


# --- test body ---

assert f'' == ''

assert f'a' == 'a'

assert f' ' == ' '
print("TestCase::test_literal: ok")
"###);
    assert_output(&out, r###"TestCase::test_literal: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/fstring/test_case__test_locals.py`.
#[test]
fn test_gen_behavior_core_fstring_test_case__test_locals() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_locals"
# subject = "cpython.test_fstring.TestCase.test_locals"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_locals
"""Auto-ported test: TestCase::test_locals (CPython 3.12 oracle)."""


import ast
import datetime
import os
import re
import types
import decimal
import unittest
import warnings
from test import support
from test.support.os_helper import temp_cwd
from test.support.script_helper import assert_python_failure, assert_python_ok


a_global = 'global variable'


# --- test body ---
value = 123

assert f'v:{value}' == 'v:123'
print("TestCase::test_locals: ok")
"###);
    assert_output(&out, r###"TestCase::test_locals: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/fstring/test_case__test_loop.py`.
#[test]
fn test_gen_behavior_core_fstring_test_case__test_loop() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_loop"
# subject = "cpython.test_fstring.TestCase.test_loop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_loop
"""Auto-ported test: TestCase::test_loop (CPython 3.12 oracle)."""


import ast
import datetime
import os
import re
import types
import decimal
import unittest
import warnings
from test import support
from test.support.os_helper import temp_cwd
from test.support.script_helper import assert_python_failure, assert_python_ok


a_global = 'global variable'


# --- test body ---
for i in range(1000):

    assert f'i:{i}' == 'i:' + str(i)
print("TestCase::test_loop: ok")
"###);
    assert_output(&out, r###"TestCase::test_loop: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/fstring/test_case__test_multiple_vars.py`.
#[test]
fn test_gen_behavior_core_fstring_test_case__test_multiple_vars() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_multiple_vars"
# subject = "cpython.test_fstring.TestCase.test_multiple_vars"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_multiple_vars
"""Auto-ported test: TestCase::test_multiple_vars (CPython 3.12 oracle)."""


import ast
import datetime
import os
import re
import types
import decimal
import unittest
import warnings
from test import support
from test.support.os_helper import temp_cwd
from test.support.script_helper import assert_python_failure, assert_python_ok


a_global = 'global variable'


# --- test body ---
x = 98
y = 'abc'

assert f'{x}{y}' == '98abc'

assert f'X{x}{y}' == 'X98abc'

assert f'{x}X{y}' == '98Xabc'

assert f'{x}{y}X' == '98abcX'

assert f'X{x}Y{y}' == 'X98Yabc'

assert f'X{x}{y}Y' == 'X98abcY'

assert f'{x}X{y}Y' == '98XabcY'

assert f'X{x}Y{y}Z' == 'X98YabcZ'
print("TestCase::test_multiple_vars: ok")
"###);
    assert_output(&out, r###"TestCase::test_multiple_vars: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/fstring/test_case__test_nested_fstrings.py`.
#[test]
fn test_gen_behavior_core_fstring_test_case__test_nested_fstrings() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_nested_fstrings"
# subject = "cpython.test_fstring.TestCase.test_nested_fstrings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_nested_fstrings
"""Auto-ported test: TestCase::test_nested_fstrings (CPython 3.12 oracle)."""


import ast
import datetime
import os
import re
import types
import decimal
import unittest
import warnings
from test import support
from test.support.os_helper import temp_cwd
from test.support.script_helper import assert_python_failure, assert_python_ok


a_global = 'global variable'


# --- test body ---
y = 5

assert f"{f'{0}' * 3}" == '000'

assert f"{f'{y}' * 3}" == '555'
print("TestCase::test_nested_fstrings: ok")
"###);
    assert_output(&out, r###"TestCase::test_nested_fstrings: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/fstring/test_case__test_newlines_in_expressions.py`.
#[test]
fn test_gen_behavior_core_fstring_test_case__test_newlines_in_expressions() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_newlines_in_expressions"
# subject = "cpython.test_fstring.TestCase.test_newlines_in_expressions"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_newlines_in_expressions
"""Auto-ported test: TestCase::test_newlines_in_expressions (CPython 3.12 oracle)."""


import ast
import datetime
import os
import re
import types
import decimal
import unittest
import warnings
from test import support
from test.support.os_helper import temp_cwd
from test.support.script_helper import assert_python_failure, assert_python_ok


a_global = 'global variable'


# --- test body ---

assert f'{0}' == '0'

assert f'{3 + 4}' == '7'
print("TestCase::test_newlines_in_expressions: ok")
"###);
    assert_output(&out, r###"TestCase::test_newlines_in_expressions: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/fstring/test_case__test_no_escapes_for_braces.py`.
#[test]
fn test_gen_behavior_core_fstring_test_case__test_no_escapes_for_braces() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_no_escapes_for_braces"
# subject = "cpython.test_fstring.TestCase.test_no_escapes_for_braces"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_no_escapes_for_braces
"""Auto-ported test: TestCase::test_no_escapes_for_braces (CPython 3.12 oracle)."""


import ast
import datetime
import os
import re
import types
import decimal
import unittest
import warnings
from test import support
from test.support.os_helper import temp_cwd
from test.support.script_helper import assert_python_failure, assert_python_ok


a_global = 'global variable'


# --- test body ---
"""
        Only literal curly braces begin an expression.
        """

assert f'{{1+1}}' == '{1+1}'

assert f'{{1+1' == '{1+1'

assert f'{{1+1' == '{1+1'

assert f'{{1+1}}' == '{1+1}'
print("TestCase::test_no_escapes_for_braces: ok")
"###);
    assert_output(&out, r###"TestCase::test_no_escapes_for_braces: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/fstring/test_case__test_not_equal.py`.
#[test]
fn test_gen_behavior_core_fstring_test_case__test_not_equal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_not_equal"
# subject = "cpython.test_fstring.TestCase.test_not_equal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_not_equal
"""Auto-ported test: TestCase::test_not_equal (CPython 3.12 oracle)."""


import ast
import datetime
import os
import re
import types
import decimal
import unittest
import warnings
from test import support
from test.support.os_helper import temp_cwd
from test.support.script_helper import assert_python_failure, assert_python_ok


a_global = 'global variable'


# --- test body ---

assert f'{3 != 4}' == 'True'

assert f'{3 != 4:}' == 'True'

assert f'{3 != 4!s}' == 'True'

assert f'{3 != 4!s:.3}' == 'Tru'
print("TestCase::test_not_equal: ok")
"###);
    assert_output(&out, r###"TestCase::test_not_equal: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/fstring/test_case__test_shadowed_global.py`.
#[test]
fn test_gen_behavior_core_fstring_test_case__test_shadowed_global() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_shadowed_global"
# subject = "cpython.test_fstring.TestCase.test_shadowed_global"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_shadowed_global
"""Auto-ported test: TestCase::test_shadowed_global (CPython 3.12 oracle)."""


import ast
import datetime
import os
import re
import types
import decimal
import unittest
import warnings
from test import support
from test.support.os_helper import temp_cwd
from test.support.script_helper import assert_python_failure, assert_python_ok


a_global = 'global variable'


# --- test body ---
a_global = 'really a local'

assert f'g:{a_global}' == 'g:really a local'

assert f'g:{a_global!r}' == "g:'really a local'"
a_local = 'local variable'

assert f'g:{a_global} l:{a_local}' == 'g:really a local l:local variable'

assert f'g:{a_global!r}' == "g:'really a local'"

assert f'g:{a_global} l:{a_local!r}' == "g:really a local l:'local variable'"
print("TestCase::test_shadowed_global: ok")
"###);
    assert_output(&out, r###"TestCase::test_shadowed_global: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/fstring/test_case__test_valid_prefixes.py`.
#[test]
fn test_gen_behavior_core_fstring_test_case__test_valid_prefixes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_valid_prefixes"
# subject = "cpython.test_fstring.TestCase.test_valid_prefixes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_valid_prefixes
"""Auto-ported test: TestCase::test_valid_prefixes (CPython 3.12 oracle)."""


import ast
import datetime
import os
import re
import types
import decimal
import unittest
import warnings
from test import support
from test.support.os_helper import temp_cwd
from test.support.script_helper import assert_python_failure, assert_python_ok


a_global = 'global variable'


# --- test body ---

assert f'{1}' == '1'

assert f'{2}' == '2'

assert f'{3}' == '3'
print("TestCase::test_valid_prefixes: ok")
"###);
    assert_output(&out, r###"TestCase::test_valid_prefixes: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/fstring/test_case__test_yield.py`.
#[test]
fn test_gen_behavior_core_fstring_test_case__test_yield() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_yield"
# subject = "cpython.test_fstring.TestCase.test_yield"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_yield
"""Auto-ported test: TestCase::test_yield (CPython 3.12 oracle)."""


import ast
import datetime
import os
import re
import types
import decimal
import unittest
import warnings
from test import support
from test.support.os_helper import temp_cwd
from test.support.script_helper import assert_python_failure, assert_python_ok


a_global = 'global variable'


# --- test body ---
def fn(y):
    f'y:{(yield (y * 2))}'
    f'{(yield)}'
g = fn(4)

assert next(g) == 8

assert next(g) == None
print("TestCase::test_yield: ok")
"###);
    assert_output(&out, r###"TestCase::test_yield: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/fstring/test_case__test_yield_send.py`.
#[test]
fn test_gen_behavior_core_fstring_test_case__test_yield_send() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_yield_send"
# subject = "cpython.test_fstring.TestCase.test_yield_send"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_yield_send
"""Auto-ported test: TestCase::test_yield_send (CPython 3.12 oracle)."""


import ast
import datetime
import os
import re
import types
import decimal
import unittest
import warnings
from test import support
from test.support.os_helper import temp_cwd
from test.support.script_helper import assert_python_failure, assert_python_ok


a_global = 'global variable'


# --- test body ---
def fn(x):
    yield f'x:{(yield (lambda i: x * i))}'
g = fn(10)
the_lambda = next(g)

assert the_lambda(4) == 40

assert g.send('string') == 'x:string'
print("TestCase::test_yield_send: ok")
"###);
    assert_output(&out, r###"TestCase::test_yield_send: ok
"###);
}
