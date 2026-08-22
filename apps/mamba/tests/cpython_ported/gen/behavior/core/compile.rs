use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/core/compile/test_expression_stack_size__test_stack_3050.py`.
#[test]
fn test_gen_behavior_core_compile_test_expression_stack_size__test_stack_3050() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_expression_stack_size__test_stack_3050"
# subject = "cpython.test_compile.TestExpressionStackSize.test_stack_3050"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestExpressionStackSize::test_stack_3050
"""Auto-ported test: TestExpressionStackSize::test_stack_3050 (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
N = 100
M = 3050
code = 'x,' * M + '=t'
compile(code, '<foo>', 'single')
print("TestExpressionStackSize::test_stack_3050: ok")
"###);
    assert_output(&out, r###"TestExpressionStackSize::test_stack_3050: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_expression_stack_size__test_stack_3050_2.py`.
#[test]
fn test_gen_behavior_core_compile_test_expression_stack_size__test_stack_3050_2() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_expression_stack_size__test_stack_3050_2"
# subject = "cpython.test_compile.TestExpressionStackSize.test_stack_3050_2"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestExpressionStackSize::test_stack_3050_2
"""Auto-ported test: TestExpressionStackSize::test_stack_3050_2 (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
N = 100
M = 3050
args = ', '.join((f'arg{i}:type{i}' for i in range(M)))
code = f'def f({args}):\n  pass'
compile(code, '<foo>', 'single')
print("TestExpressionStackSize::test_stack_3050_2: ok")
"###);
    assert_output(&out, r###"TestExpressionStackSize::test_stack_3050_2: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_annotation_limit.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_annotation_limit() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_annotation_limit"
# subject = "cpython.test_compile.TestSpecifics.test_annotation_limit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_annotation_limit
"""Auto-ported test: TestSpecifics::test_annotation_limit (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
s = 'def f(%s): pass'
s %= ', '.join(('a%d:%d' % (i, i) for i in range(300)))
compile(s, '?', 'exec')
print("TestSpecifics::test_annotation_limit: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_annotation_limit: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_apply_static_swaps.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_apply_static_swaps() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_apply_static_swaps"
# subject = "cpython.test_compile.TestSpecifics.test_apply_static_swaps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_apply_static_swaps
"""Auto-ported test: TestSpecifics::test_apply_static_swaps (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
def f(x, y):
    a, a = (x, y)
    return a

assert f('x', 'y') == 'y'
print("TestSpecifics::test_apply_static_swaps: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_apply_static_swaps: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_apply_static_swaps_2.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_apply_static_swaps_2() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_apply_static_swaps_2"
# subject = "cpython.test_compile.TestSpecifics.test_apply_static_swaps_2"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_apply_static_swaps_2
"""Auto-ported test: TestSpecifics::test_apply_static_swaps_2 (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
def f(x, y, z):
    a, b, a = (x, y, z)
    return a

assert f('x', 'y', 'z') == 'z'
print("TestSpecifics::test_apply_static_swaps_2: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_apply_static_swaps_2: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_apply_static_swaps_3.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_apply_static_swaps_3() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_apply_static_swaps_3"
# subject = "cpython.test_compile.TestSpecifics.test_apply_static_swaps_3"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_apply_static_swaps_3
"""Auto-ported test: TestSpecifics::test_apply_static_swaps_3 (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
def f(x, y, z):
    a, a, b = (x, y, z)
    return a

assert f('x', 'y', 'z') == 'y'
print("TestSpecifics::test_apply_static_swaps_3: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_apply_static_swaps_3: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_big_dict_literal.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_big_dict_literal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_big_dict_literal"
# subject = "cpython.test_compile.TestSpecifics.test_big_dict_literal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_big_dict_literal
"""Auto-ported test: TestSpecifics::test_big_dict_literal (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
dict_size = 65535 + 1
the_dict = '{' + ','.join((f'{x}:{x}' for x in range(dict_size))) + '}'

assert len(eval(the_dict)) == dict_size
print("TestSpecifics::test_big_dict_literal: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_big_dict_literal: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_cold_block_moved_to_end.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_cold_block_moved_to_end() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_cold_block_moved_to_end"
# subject = "cpython.test_compile.TestSpecifics.test_cold_block_moved_to_end"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_cold_block_moved_to_end
"""Auto-ported test: TestSpecifics::test_cold_block_moved_to_end (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
def f():
    while name:
        try:
            break
        except:
            pass
    else:
        1 if 1 else 1
print("TestSpecifics::test_cold_block_moved_to_end: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_cold_block_moved_to_end: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_consts_in_conditionals.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_consts_in_conditionals() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_consts_in_conditionals"
# subject = "cpython.test_compile.TestSpecifics.test_consts_in_conditionals"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_consts_in_conditionals
"""Auto-ported test: TestSpecifics::test_consts_in_conditionals (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
def assertInvalidSingle(source):

    try:
        compile_single(source)
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass

def check_constant(func, expected):
    for const in func.__code__.co_consts:
        if repr(const) == repr(expected):
            break
    else:

        raise AssertionError('unable to find constant %r in %r' % (expected, func.__code__.co_consts))

def compile_single(source):
    compile(source, '<single>', 'single')

def get_code_lines(code):
    last_line = -2
    res = []
    for _, _, line in code.co_lines():
        if line is not None and line != last_line:
            res.append(line - code.co_firstlineno)
            last_line = line
    return res

def and_true(x):
    return True and x

def and_false(x):
    return False and x

def or_true(x):
    return True or x

def or_false(x):
    return False or x
funcs = [and_true, and_false, or_true, or_false]
for func in funcs:
    opcodes = list(dis.get_instructions(func))

    assert len(opcodes) <= 3

    assert 'LOAD_' in opcodes[-2].opname

    assert 'RETURN_VALUE' == opcodes[-1].opname
print("TestSpecifics::test_consts_in_conditionals: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_consts_in_conditionals: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_duplicated_small_exit_block.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_duplicated_small_exit_block() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_duplicated_small_exit_block"
# subject = "cpython.test_compile.TestSpecifics.test_duplicated_small_exit_block"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_duplicated_small_exit_block
"""Auto-ported test: TestSpecifics::test_duplicated_small_exit_block (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
def f():
    while element and something:
        try:
            return something
        except:
            pass
print("TestSpecifics::test_duplicated_small_exit_block: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_duplicated_small_exit_block: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_empty.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_empty"
# subject = "cpython.test_compile.TestSpecifics.test_empty"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_empty
"""Auto-ported test: TestSpecifics::test_empty (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
compile('', '<test>', 'exec')
print("TestSpecifics::test_empty: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_empty: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_if_expression_expression_empty_block.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_if_expression_expression_empty_block() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_if_expression_expression_empty_block"
# subject = "cpython.test_compile.TestSpecifics.test_if_expression_expression_empty_block"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_if_expression_expression_empty_block
"""Auto-ported test: TestSpecifics::test_if_expression_expression_empty_block (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
def assertInvalidSingle(source):

    try:
        compile_single(source)
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass

def check_constant(func, expected):
    for const in func.__code__.co_consts:
        if repr(const) == repr(expected):
            break
    else:

        raise AssertionError('unable to find constant %r in %r' % (expected, func.__code__.co_consts))

def compile_single(source):
    compile(source, '<single>', 'single')

def get_code_lines(code):
    last_line = -2
    res = []
    for _, _, line in code.co_lines():
        if line is not None and line != last_line:
            res.append(line - code.co_firstlineno)
            last_line = line
    return res
exprs = ['assert (False if 1 else True)', 'def f():\n\tif not (False if 1 else True): raise AssertionError', 'def f():\n\tif not (False if 1 else True): return 12']
for expr in exprs:
    compile(expr, '<single>', 'exec')
print("TestSpecifics::test_if_expression_expression_empty_block: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_if_expression_expression_empty_block: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_indentation.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_indentation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_indentation"
# subject = "cpython.test_compile.TestSpecifics.test_indentation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_indentation
"""Auto-ported test: TestSpecifics::test_indentation (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
s = textwrap.dedent('\n            if 1:\n                if 2:\n                    pass\n            ')
compile(s, '<string>', 'exec')
print("TestSpecifics::test_indentation: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_indentation: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_lambda_doc.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_lambda_doc() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_lambda_doc"
# subject = "cpython.test_compile.TestSpecifics.test_lambda_doc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_lambda_doc
"""Auto-ported test: TestSpecifics::test_lambda_doc (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
l = lambda: 'foo'

assert l.__doc__ is None
print("TestSpecifics::test_lambda_doc: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_lambda_doc: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_merge_code_attrs.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_merge_code_attrs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_merge_code_attrs"
# subject = "cpython.test_compile.TestSpecifics.test_merge_code_attrs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_merge_code_attrs
"""Auto-ported test: TestSpecifics::test_merge_code_attrs (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
f1 = lambda x: x.y.z
f2 = lambda a: a.b.c

assert f1.__code__.co_linetable is f2.__code__.co_linetable
print("TestSpecifics::test_merge_code_attrs: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_merge_code_attrs: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_multi_line_lambda_as_argument.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_multi_line_lambda_as_argument() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_multi_line_lambda_as_argument"
# subject = "cpython.test_compile.TestSpecifics.test_multi_line_lambda_as_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_multi_line_lambda_as_argument
"""Auto-ported test: TestSpecifics::test_multi_line_lambda_as_argument (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
code = textwrap.dedent('\n            def foo(param, lambda_exp):\n                pass\n\n            foo(param=0,\n                lambda_exp=lambda:\n                1)\n        ')
compile(code, '<test>', 'exec')
print("TestSpecifics::test_multi_line_lambda_as_argument: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_multi_line_lambda_as_argument: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_no_ending_newline.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_no_ending_newline() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_no_ending_newline"
# subject = "cpython.test_compile.TestSpecifics.test_no_ending_newline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_no_ending_newline
"""Auto-ported test: TestSpecifics::test_no_ending_newline (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
compile('hi', '<test>', 'exec')
compile('hi\r', '<test>', 'exec')
print("TestSpecifics::test_no_ending_newline: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_no_ending_newline: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_none_keyword_arg.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_none_keyword_arg() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_none_keyword_arg"
# subject = "cpython.test_compile.TestSpecifics.test_none_keyword_arg"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_none_keyword_arg
"""Auto-ported test: TestSpecifics::test_none_keyword_arg (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---

try:
    compile('f(None=1)', '<string>', 'exec')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass
print("TestSpecifics::test_none_keyword_arg: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_none_keyword_arg: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_other_newlines.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_other_newlines() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_other_newlines"
# subject = "cpython.test_compile.TestSpecifics.test_other_newlines"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_other_newlines
"""Auto-ported test: TestSpecifics::test_other_newlines (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
compile('\r\n', '<test>', 'exec')
compile('\r', '<test>', 'exec')
compile('hi\r\nstuff\r\ndef f():\n    pass\r', '<test>', 'exec')
compile('this_is\rreally_old_mac\rdef f():\n    pass', '<test>', 'exec')
print("TestSpecifics::test_other_newlines: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_other_newlines: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_path_like_objects.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_path_like_objects() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_path_like_objects"
# subject = "cpython.test_compile.TestSpecifics.test_path_like_objects"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_path_like_objects
"""Auto-ported test: TestSpecifics::test_path_like_objects (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
compile('42', FakePath('test_compile_pathlike'), 'single')
print("TestSpecifics::test_path_like_objects: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_path_like_objects: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_remove_empty_basic_block_with_jump_target_label.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_remove_empty_basic_block_with_jump_target_label() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_remove_empty_basic_block_with_jump_target_label"
# subject = "cpython.test_compile.TestSpecifics.test_remove_empty_basic_block_with_jump_target_label"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_remove_empty_basic_block_with_jump_target_label
"""Auto-ported test: TestSpecifics::test_remove_empty_basic_block_with_jump_target_label (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
def f(x):
    while x:
        0 if 1 else 0
print("TestSpecifics::test_remove_empty_basic_block_with_jump_target_label: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_remove_empty_basic_block_with_jump_target_label: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_remove_redundant_nop_edge_case.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_remove_redundant_nop_edge_case() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_remove_redundant_nop_edge_case"
# subject = "cpython.test_compile.TestSpecifics.test_remove_redundant_nop_edge_case"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_remove_redundant_nop_edge_case
"""Auto-ported test: TestSpecifics::test_remove_redundant_nop_edge_case (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
def f():
    a if (1 if b else c) else d
print("TestSpecifics::test_remove_redundant_nop_edge_case: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_remove_redundant_nop_edge_case: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_sequence_unpacking_error.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_sequence_unpacking_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_sequence_unpacking_error"
# subject = "cpython.test_compile.TestSpecifics.test_sequence_unpacking_error"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_sequence_unpacking_error
"""Auto-ported test: TestSpecifics::test_sequence_unpacking_error (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
i, j = (1, -1) or (-1, 1)

assert i == 1

assert j == -1
print("TestSpecifics::test_sequence_unpacking_error: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_sequence_unpacking_error: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_stack_overflow.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_stack_overflow() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_stack_overflow"
# subject = "cpython.test_compile.TestSpecifics.test_stack_overflow"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_stack_overflow
"""Auto-ported test: TestSpecifics::test_stack_overflow (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
compile('if a: b\n' * 200000, '<dummy>', 'exec')
print("TestSpecifics::test_stack_overflow: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_stack_overflow: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compile/test_specifics__test_syntax_error.py`.
#[test]
fn test_gen_behavior_core_compile_test_specifics__test_syntax_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_syntax_error"
# subject = "cpython.test_compile.TestSpecifics.test_syntax_error"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_syntax_error
"""Auto-ported test: TestSpecifics::test_syntax_error (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---

try:
    compile('1+*3', 'filename', 'exec')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass
print("TestSpecifics::test_syntax_error: ok")
"###);
    assert_output(&out, r###"TestSpecifics::test_syntax_error: ok
"###);
}
