use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t__tests__test_AST_fields_NULL_check.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t__tests__test_AST_fields_NULL_check() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_AST_fields_NULL_check"
# subject = "cpython.test_ast.AST_Tests.test_AST_fields_NULL_check"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import unittest, io
from test.test_ast import test_ast
_suite = unittest.defaultTestLoader.loadTestsFromName("AST_Tests.test_AST_fields_NULL_check", test_ast)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AST_Tests.test_AST_fields_NULL_check did not pass"
print("AST_Tests::test_AST_fields_NULL_check: ok")
"###);
    assert_output(&out, r###"AST_Tests::test_AST_fields_NULL_check: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t__tests__test_AST_objects.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t__tests__test_AST_objects() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_AST_objects"
# subject = "cpython.test_ast.AST_Tests.test_AST_objects"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

x = ast.AST()
assert x._fields == ()
x.foobar = 42
assert x.foobar == 42
assert x.__dict__["foobar"] == 42

try:
    x.vararg
except AttributeError:
    pass
else:
    raise AssertionError("ast.AST missing attribute did not raise AttributeError")

try:
    ast.AST(2)
except TypeError:
    pass
else:
    raise AssertionError("ast.AST positional argument did not raise TypeError")

print("AST_Tests::test_AST_objects: ok")
"###);
    assert_output(&out, r###"AST_Tests::test_AST_objects: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t__tests__test_alias.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t__tests__test_alias() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_alias"
# subject = "cpython.test_ast.AST_Tests.test_alias"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

im = ast.parse("from bar import y").body[0]
assert len(im.names) == 1
alias = im.names[0]
assert alias.name == "y"
assert alias.asname is None
assert alias.lineno == 1
assert alias.end_lineno == 1
assert alias.col_offset == 16
assert alias.end_col_offset == 17

im = ast.parse("from bar import *").body[0]
alias = im.names[0]
assert alias.name == "*"
assert alias.asname is None
assert alias.lineno == 1
assert alias.end_lineno == 1
assert alias.col_offset == 16
assert alias.end_col_offset == 17

im = ast.parse("from bar import y as z").body[0]
alias = im.names[0]
assert alias.name == "y"
assert alias.asname == "z"
assert alias.lineno == 1
assert alias.end_lineno == 1
assert alias.col_offset == 16
assert alias.end_col_offset == 22

im = ast.parse("import bar as foo").body[0]
alias = im.names[0]
assert alias.name == "bar"
assert alias.asname == "foo"
assert alias.lineno == 1
assert alias.end_lineno == 1
assert alias.col_offset == 7
assert alias.end_col_offset == 17

print("AST_Tests::test_alias: ok")
"###);
    assert_output(&out, r###"AST_Tests::test_alias: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t__tests__test_assignment_expression_feature_version.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t__tests__test_assignment_expression_feature_version() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_assignment_expression_feature_version"
# subject = "cpython.test_ast.AST_Tests.test_assignment_expression_feature_version"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import unittest, io
from test.test_ast import test_ast
_suite = unittest.defaultTestLoader.loadTestsFromName("AST_Tests.test_assignment_expression_feature_version", test_ast)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AST_Tests.test_assignment_expression_feature_version did not pass"
print("AST_Tests::test_assignment_expression_feature_version: ok")
"###);
    assert_output(&out, r###"AST_Tests::test_assignment_expression_feature_version: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t__tests__test_ast_asdl_signature.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t__tests__test_ast_asdl_signature() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_ast_asdl_signature"
# subject = "cpython.test_ast.AST_Tests.test_ast_asdl_signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import unittest, io
from test.test_ast import test_ast
_suite = unittest.defaultTestLoader.loadTestsFromName("AST_Tests.test_ast_asdl_signature", test_ast)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AST_Tests.test_ast_asdl_signature did not pass"
print("AST_Tests::test_ast_asdl_signature: ok")
"###);
    assert_output(&out, r###"AST_Tests::test_ast_asdl_signature: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t__tests__test_base_classes.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t__tests__test_base_classes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_base_classes"
# subject = "cpython.test_ast.AST_Tests.test_base_classes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

checks = [
    ("For_stmt", ast.For, ast.stmt),
    ("Name_expr", ast.Name, ast.expr),
    ("stmt_AST", ast.stmt, ast.AST),
    ("expr_AST", ast.expr, ast.AST),
    ("comprehension_AST", ast.comprehension, ast.AST),
    ("Gt_AST", ast.Gt, ast.AST),
]

for label, child, parent in checks:
    result = issubclass(child, parent)
    if not result:
        raise AssertionError(label)
print("AST_Tests::test_base_classes: ok")
"###);
    assert_output(&out, r###"AST_Tests::test_base_classes: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t__tests__test_classattrs_deprecated.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t__tests__test_classattrs_deprecated() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_classattrs_deprecated"
# subject = "cpython.test_ast.AST_Tests.test_classattrs_deprecated"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import unittest, io
from test.test_ast import test_ast
_suite = unittest.defaultTestLoader.loadTestsFromName("AST_Tests.test_classattrs_deprecated", test_ast)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AST_Tests.test_classattrs_deprecated did not pass"
print("AST_Tests::test_classattrs_deprecated: ok")
"###);
    assert_output(&out, r###"AST_Tests::test_classattrs_deprecated: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t__tests__test_conditional_context_managers_parse_with_low_feature_version.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t__tests__test_conditional_context_managers_parse_with_low_feature_version() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_conditional_context_managers_parse_with_low_feature_version"
# subject = "cpython.test_ast.AST_Tests.test_conditional_context_managers_parse_with_low_feature_version"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import unittest, io
from test.test_ast import test_ast
_suite = unittest.defaultTestLoader.loadTestsFromName("AST_Tests.test_conditional_context_managers_parse_with_low_feature_version", test_ast)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AST_Tests.test_conditional_context_managers_parse_with_low_feature_version did not pass"
print("AST_Tests::test_conditional_context_managers_parse_with_low_feature_version: ok")
"###);
    assert_output(&out, r###"AST_Tests::test_conditional_context_managers_parse_with_low_feature_version: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t__tests__test_constant_as_name.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t__tests__test_constant_as_name() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_constant_as_name"
# subject = "cpython.test_ast.AST_Tests.test_constant_as_name"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

for constant in ("True", "False", "None"):
    expr = ast.Expression(ast.Name(constant, ast.Load()))
    ast.fix_missing_locations(expr)
    try:
        compile(expr, "<test>", "eval")
    except ValueError as exc:
        expected = f"identifier field can't represent '{constant}' constant"
        if expected not in str(exc):
            raise AssertionError((constant, str(exc), expected))
    else:
        raise AssertionError(f"expected ValueError for {constant}")

print("AST_Tests::test_constant_as_name: ok")
"###);
    assert_output(&out, r###"AST_Tests::test_constant_as_name: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t__tests__test_constant_subclasses.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t__tests__test_constant_subclasses() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_constant_subclasses"
# subject = "cpython.test_ast.AST_Tests.test_constant_subclasses"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import unittest, io
from test.test_ast import test_ast
_suite = unittest.defaultTestLoader.loadTestsFromName("AST_Tests.test_constant_subclasses", test_ast)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AST_Tests.test_constant_subclasses did not pass"
print("AST_Tests::test_constant_subclasses: ok")
"###);
    assert_output(&out, r###"AST_Tests::test_constant_subclasses: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t__tests__test_constant_subclasses_deprecated.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t__tests__test_constant_subclasses_deprecated() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_constant_subclasses_deprecated"
# subject = "cpython.test_ast.AST_Tests.test_constant_subclasses_deprecated"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import unittest, io
from test.test_ast import test_ast
_suite = unittest.defaultTestLoader.loadTestsFromName("AST_Tests.test_constant_subclasses_deprecated", test_ast)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AST_Tests.test_constant_subclasses_deprecated did not pass"
print("AST_Tests::test_constant_subclasses_deprecated: ok")
"###);
    assert_output(&out, r###"AST_Tests::test_constant_subclasses_deprecated: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t__tests__test_field_attr_existence.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t__tests__test_field_attr_existence() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_field_attr_existence"
# subject = "cpython.test_ast.AST_Tests.test_field_attr_existence"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast


def is_ast_node(name, node):
    if not isinstance(node, type):
        return False
    if "ast" not in node.__module__:
        return False
    return name != "AST" and name[0].isupper()


for name, item in ast.__dict__.items():
    if name in {"Num", "Str", "Bytes", "NameConstant", "Ellipsis"}:
        continue
    if name == "Index":
        continue
    if is_ast_node(name, item):
        node = item()
        if isinstance(node, ast.AST) and type(node._fields) is not tuple:
            raise AssertionError((name, node._fields))

print("AST_Tests::test_field_attr_existence: ok")
"###);
    assert_output(&out, r###"AST_Tests::test_field_attr_existence: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t__tests__test_field_attr_existence_deprecated.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t__tests__test_field_attr_existence_deprecated() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_field_attr_existence_deprecated"
# subject = "cpython.test_ast.AST_Tests.test_field_attr_existence_deprecated"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import unittest, io
from test.test_ast import test_ast
_suite = unittest.defaultTestLoader.loadTestsFromName("AST_Tests.test_field_attr_existence_deprecated", test_ast)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AST_Tests.test_field_attr_existence_deprecated did not pass"
print("AST_Tests::test_field_attr_existence_deprecated: ok")
"###);
    assert_output(&out, r###"AST_Tests::test_field_attr_existence_deprecated: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t__tests__test_from_import.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t__tests__test_from_import() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_from_import"
# subject = "cpython.test_ast.AST_Tests.test_from_import"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

im = ast.parse("from . import y").body[0]
assert im.module is None

print("AST_Tests::test_from_import: ok")
"###);
    assert_output(&out, r###"AST_Tests::test_from_import: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t__tests__test_invalid_constant.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t__tests__test_invalid_constant() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_invalid_constant"
# subject = "cpython.test_ast.AST_Tests.test_invalid_constant"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import unittest, io
from test.test_ast import test_ast
_suite = unittest.defaultTestLoader.loadTestsFromName("AST_Tests.test_invalid_constant", test_ast)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AST_Tests.test_invalid_constant did not pass"
print("AST_Tests::test_invalid_constant: ok")
"###);
    assert_output(&out, r###"AST_Tests::test_invalid_constant: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t__tests__test_invalid_major_feature_version.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t__tests__test_invalid_major_feature_version() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_invalid_major_feature_version"
# subject = "cpython.test_ast.AST_Tests.test_invalid_major_feature_version"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import unittest, io
from test.test_ast import test_ast
_suite = unittest.defaultTestLoader.loadTestsFromName("AST_Tests.test_invalid_major_feature_version", test_ast)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AST_Tests.test_invalid_major_feature_version did not pass"
print("AST_Tests::test_invalid_major_feature_version: ok")
"###);
    assert_output(&out, r###"AST_Tests::test_invalid_major_feature_version: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t__tests__test_isinstance.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t__tests__test_isinstance() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_isinstance"
# subject = "cpython.test_ast.AST_Tests.test_isinstance"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import unittest, io
from test.test_ast import test_ast
_suite = unittest.defaultTestLoader.loadTestsFromName("AST_Tests.test_isinstance", test_ast)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AST_Tests.test_isinstance did not pass"
print("AST_Tests::test_isinstance: ok")
"###);
    assert_output(&out, r###"AST_Tests::test_isinstance: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t__tests__test_no_fields.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t__tests__test_no_fields() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_no_fields"
# subject = "cpython.test_ast.AST_Tests.test_no_fields"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

x = ast.Sub()
assert x._fields == ()

print("AST_Tests::test_no_fields: ok")
"###);
    assert_output(&out, r###"AST_Tests::test_no_fields: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t__tests__test_null_bytes.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t__tests__test_null_bytes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_null_bytes"
# subject = "cpython.test_ast.AST_Tests.test_null_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import unittest, io
from test.test_ast import test_ast
_suite = unittest.defaultTestLoader.loadTestsFromName("AST_Tests.test_null_bytes", test_ast)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AST_Tests.test_null_bytes did not pass"
print("AST_Tests::test_null_bytes: ok")
"###);
    assert_output(&out, r###"AST_Tests::test_null_bytes: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t__tests__test_slice.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t__tests__test_slice() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_slice"
# subject = "cpython.test_ast.AST_Tests.test_slice"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

slc = ast.parse("x[::]").body[0].value.slice
assert slc.upper is None
assert slc.lower is None
assert slc.step is None

print("AST_Tests::test_slice: ok")
"###);
    assert_output(&out, r###"AST_Tests::test_slice: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t_helpers__test__test_dump.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t_helpers__test__test_dump() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_dump"
# subject = "cpython.test_ast.ASTHelpers_Test.test_dump"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent
maxDiff = None
node = ast.parse('spam(eggs, "and cheese")')
assert ast.dump(node) == "Module(body=[Expr(value=Call(func=Name(id='spam', ctx=Load()), args=[Name(id='eggs', ctx=Load()), Constant(value='and cheese')], keywords=[]))], type_ignores=[])"
assert ast.dump(node, annotate_fields=False) == "Module([Expr(Call(Name('spam', Load()), [Name('eggs', Load()), Constant('and cheese')], []))], [])"
assert ast.dump(node, include_attributes=True) == "Module(body=[Expr(value=Call(func=Name(id='spam', ctx=Load(), lineno=1, col_offset=0, end_lineno=1, end_col_offset=4), args=[Name(id='eggs', ctx=Load(), lineno=1, col_offset=5, end_lineno=1, end_col_offset=9), Constant(value='and cheese', lineno=1, col_offset=11, end_lineno=1, end_col_offset=23)], keywords=[], lineno=1, col_offset=0, end_lineno=1, end_col_offset=24), lineno=1, col_offset=0, end_lineno=1, end_col_offset=24)], type_ignores=[])"

print("ASTHelpers_Test::test_dump: ok")
"###);
    assert_output(&out, r###"ASTHelpers_Test::test_dump: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t_helpers__test__test_dump_incomplete.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t_helpers__test__test_dump_incomplete() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_dump_incomplete"
# subject = "cpython.test_ast.ASTHelpers_Test.test_dump_incomplete"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent
maxDiff = None
node = ast.Raise(lineno=3, col_offset=4)
assert ast.dump(node) == 'Raise()'
assert ast.dump(node, include_attributes=True) == 'Raise(lineno=3, col_offset=4)'
node = ast.Raise(exc=ast.Name(id='e', ctx=ast.Load()), lineno=3, col_offset=4)
assert ast.dump(node) == "Raise(exc=Name(id='e', ctx=Load()))"
assert ast.dump(node, annotate_fields=False) == "Raise(Name('e', Load()))"
assert ast.dump(node, include_attributes=True) == "Raise(exc=Name(id='e', ctx=Load()), lineno=3, col_offset=4)"
assert ast.dump(node, annotate_fields=False, include_attributes=True) == "Raise(Name('e', Load()), lineno=3, col_offset=4)"
node = ast.Raise(cause=ast.Name(id='e', ctx=ast.Load()))
assert ast.dump(node) == "Raise(cause=Name(id='e', ctx=Load()))"
assert ast.dump(node, annotate_fields=False) == "Raise(cause=Name('e', Load()))"

print("ASTHelpers_Test::test_dump_incomplete: ok")
"###);
    assert_output(&out, r###"ASTHelpers_Test::test_dump_incomplete: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t_helpers__test__test_dump_indent.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t_helpers__test__test_dump_indent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_dump_indent"
# subject = "cpython.test_ast.ASTHelpers_Test.test_dump_indent"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent
maxDiff = None
node = ast.parse('spam(eggs, "and cheese")')
assert ast.dump(node, indent=3) == "Module(\n   body=[\n      Expr(\n         value=Call(\n            func=Name(id='spam', ctx=Load()),\n            args=[\n               Name(id='eggs', ctx=Load()),\n               Constant(value='and cheese')],\n            keywords=[]))],\n   type_ignores=[])"
assert ast.dump(node, annotate_fields=False, indent='\t') == "Module(\n\t[\n\t\tExpr(\n\t\t\tCall(\n\t\t\t\tName('spam', Load()),\n\t\t\t\t[\n\t\t\t\t\tName('eggs', Load()),\n\t\t\t\t\tConstant('and cheese')],\n\t\t\t\t[]))],\n\t[])"
assert ast.dump(node, include_attributes=True, indent=3) == "Module(\n   body=[\n      Expr(\n         value=Call(\n            func=Name(\n               id='spam',\n               ctx=Load(),\n               lineno=1,\n               col_offset=0,\n               end_lineno=1,\n               end_col_offset=4),\n            args=[\n               Name(\n                  id='eggs',\n                  ctx=Load(),\n                  lineno=1,\n                  col_offset=5,\n                  end_lineno=1,\n                  end_col_offset=9),\n               Constant(\n                  value='and cheese',\n                  lineno=1,\n                  col_offset=11,\n                  end_lineno=1,\n                  end_col_offset=23)],\n            keywords=[],\n            lineno=1,\n            col_offset=0,\n            end_lineno=1,\n            end_col_offset=24),\n         lineno=1,\n         col_offset=0,\n         end_lineno=1,\n         end_col_offset=24)],\n   type_ignores=[])"

print("ASTHelpers_Test::test_dump_indent: ok")
"###);
    assert_output(&out, r###"ASTHelpers_Test::test_dump_indent: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t_helpers__test__test_elif_stmt_start_position.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t_helpers__test__test_elif_stmt_start_position() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_elif_stmt_start_position"
# subject = "cpython.test_ast.ASTHelpers_Test.test_elif_stmt_start_position"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent
maxDiff = None
node = ast.parse('if a:\n    pass\nelif b:\n    pass\n')
elif_stmt = node.body[0].orelse[0]
assert elif_stmt.lineno == 3
assert elif_stmt.col_offset == 0

print("ASTHelpers_Test::test_elif_stmt_start_position: ok")
"###);
    assert_output(&out, r###"ASTHelpers_Test::test_elif_stmt_start_position: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t_helpers__test__test_elif_stmt_start_position_with_else.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t_helpers__test__test_elif_stmt_start_position_with_else() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_elif_stmt_start_position_with_else"
# subject = "cpython.test_ast.ASTHelpers_Test.test_elif_stmt_start_position_with_else"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent
maxDiff = None
node = ast.parse('if a:\n    pass\nelif b:\n    pass\nelse:\n    pass\n')
elif_stmt = node.body[0].orelse[0]
assert elif_stmt.lineno == 3
assert elif_stmt.col_offset == 0

print("ASTHelpers_Test::test_elif_stmt_start_position_with_else: ok")
"###);
    assert_output(&out, r###"ASTHelpers_Test::test_elif_stmt_start_position_with_else: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t_helpers__test__test_get_docstring.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t_helpers__test__test_get_docstring() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_get_docstring"
# subject = "cpython.test_ast.ASTHelpers_Test.test_get_docstring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast


node = ast.parse('"""line one\n  line two"""')
assert ast.get_docstring(node) == 'line one\nline two'
node = ast.parse('class foo:\n  """line one\n  line two"""')
assert ast.get_docstring(node.body[0]) == 'line one\nline two'
node = ast.parse('def foo():\n  """line one\n  line two"""')
assert ast.get_docstring(node.body[0]) == 'line one\nline two'
node = ast.parse('async def foo():\n  """spam\n  ham"""')
assert ast.get_docstring(node.body[0]) == 'spam\nham'
node = ast.parse('async def foo():\n  """spam\n  ham"""')
assert ast.get_docstring(node.body[0], clean=False) == 'spam\n  ham'
node = ast.parse('x')
try:
    ast.get_docstring(node.body[0])
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("ASTHelpers_Test::test_get_docstring: ok")
"###);
    assert_output(&out, r###"ASTHelpers_Test::test_get_docstring: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t_helpers__test__test_get_docstring_none.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t_helpers__test__test_get_docstring_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_get_docstring_none"
# subject = "cpython.test_ast.ASTHelpers_Test.test_get_docstring_none"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent
maxDiff = None
assert ast.get_docstring(ast.parse('')) is None
node = ast.parse('x = "not docstring"')
assert ast.get_docstring(node) is None
node = ast.parse('def foo():\n  pass')
assert ast.get_docstring(node) is None
node = ast.parse('class foo:\n  pass')
assert ast.get_docstring(node.body[0]) is None
node = ast.parse('class foo:\n  x = "not docstring"')
assert ast.get_docstring(node.body[0]) is None
node = ast.parse('class foo:\n  def bar(self): pass')
assert ast.get_docstring(node.body[0]) is None
node = ast.parse('def foo():\n  pass')
assert ast.get_docstring(node.body[0]) is None
node = ast.parse('def foo():\n  x = "not docstring"')
assert ast.get_docstring(node.body[0]) is None
node = ast.parse('async def foo():\n  pass')
assert ast.get_docstring(node.body[0]) is None
node = ast.parse('async def foo():\n  x = "not docstring"')
assert ast.get_docstring(node.body[0]) is None
node = ast.parse('async def foo():\n  42')
assert ast.get_docstring(node.body[0]) is None

print("ASTHelpers_Test::test_get_docstring_none: ok")
"###);
    assert_output(&out, r###"ASTHelpers_Test::test_get_docstring_none: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t_helpers__test__test_increment_lineno.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t_helpers__test__test_increment_lineno() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_increment_lineno"
# subject = "cpython.test_ast.ASTHelpers_Test.test_increment_lineno"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent
maxDiff = None
src = ast.parse('1 + 1', mode='eval')
assert ast.increment_lineno(src, n=3) == src
assert ast.dump(src, include_attributes=True) == 'Expression(body=BinOp(left=Constant(value=1, lineno=4, col_offset=0, end_lineno=4, end_col_offset=1), op=Add(), right=Constant(value=1, lineno=4, col_offset=4, end_lineno=4, end_col_offset=5), lineno=4, col_offset=0, end_lineno=4, end_col_offset=5))'
src = ast.parse('1 + 1', mode='eval')
assert ast.increment_lineno(src.body, n=3) == src.body
assert ast.dump(src, include_attributes=True) == 'Expression(body=BinOp(left=Constant(value=1, lineno=4, col_offset=0, end_lineno=4, end_col_offset=1), op=Add(), right=Constant(value=1, lineno=4, col_offset=4, end_lineno=4, end_col_offset=5), lineno=4, col_offset=0, end_lineno=4, end_col_offset=5))'
src = ast.Call(func=ast.Name('test', ast.Load()), args=[], keywords=[], lineno=1)
assert ast.increment_lineno(src).lineno == 2
assert ast.increment_lineno(src).end_lineno is None

print("ASTHelpers_Test::test_increment_lineno: ok")
"###);
    assert_output(&out, r###"ASTHelpers_Test::test_increment_lineno: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t_helpers__test__test_increment_lineno_on_module.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t_helpers__test__test_increment_lineno_on_module() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_increment_lineno_on_module"
# subject = "cpython.test_ast.ASTHelpers_Test.test_increment_lineno_on_module"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent
maxDiff = None
src = ast.parse(dedent('        a = 1\n        b = 2 # type: ignore\n        c = 3\n        d = 4 # type: ignore@tag\n        '), type_comments=True)
ast.increment_lineno(src, n=5)
assert src.type_ignores[0].lineno == 7
assert src.type_ignores[1].lineno == 9
assert src.type_ignores[1].tag == '@tag'

print("ASTHelpers_Test::test_increment_lineno_on_module: ok")
"###);
    assert_output(&out, r###"ASTHelpers_Test::test_increment_lineno_on_module: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t_helpers__test__test_iter_child_nodes.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t_helpers__test__test_iter_child_nodes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_iter_child_nodes"
# subject = "cpython.test_ast.ASTHelpers_Test.test_iter_child_nodes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast


node = ast.parse("spam(23, 42, eggs='leek')", mode='eval')
assert len(list(ast.iter_child_nodes(node.body))) == 4
iterator = ast.iter_child_nodes(node.body)
assert next(iterator).id == 'spam'
assert next(iterator).value == 23
assert next(iterator).value == 42
assert ast.dump(next(iterator)) == "keyword(arg='eggs', value=Constant(value='leek'))"

print("ASTHelpers_Test::test_iter_child_nodes: ok")
"###);
    assert_output(&out, r###"ASTHelpers_Test::test_iter_child_nodes: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t_helpers__test__test_iter_fields.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t_helpers__test__test_iter_fields() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_iter_fields"
# subject = "cpython.test_ast.ASTHelpers_Test.test_iter_fields"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast


node = ast.parse('foo()', mode='eval')
d = dict(ast.iter_fields(node.body))
assert d.pop('func').id == 'foo'
assert d == {'keywords': [], 'args': []}

print("ASTHelpers_Test::test_iter_fields: ok")
"###);
    assert_output(&out, r###"ASTHelpers_Test::test_iter_fields: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t_helpers__test__test_literal_eval.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t_helpers__test__test_literal_eval() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_literal_eval"
# subject = "cpython.test_ast.ASTHelpers_Test.test_literal_eval"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent
maxDiff = None
assert ast.literal_eval('[1, 2, 3]') == [1, 2, 3]
assert ast.literal_eval('{"foo": 42}') == {'foo': 42}
assert ast.literal_eval('(True, False, None)') == (True, False, None)
assert ast.literal_eval('{1, 2, 3}') == {1, 2, 3}
assert ast.literal_eval('b"hi"') == b'hi'
assert ast.literal_eval('set()') == set()
try:
    ast.literal_eval('foo()')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
assert ast.literal_eval('6') == 6
assert ast.literal_eval('+6') == 6
assert ast.literal_eval('-6') == -6
assert ast.literal_eval('3.25') == 3.25
assert ast.literal_eval('+3.25') == 3.25
assert ast.literal_eval('-3.25') == -3.25
assert repr(ast.literal_eval('-0.0')) == '-0.0'
try:
    ast.literal_eval('++6')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    ast.literal_eval('+True')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    ast.literal_eval('2+3')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass

print("ASTHelpers_Test::test_literal_eval: ok")
"###);
    assert_output(&out, r###"ASTHelpers_Test::test_literal_eval: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t_helpers__test__test_literal_eval_complex.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t_helpers__test__test_literal_eval_complex() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_literal_eval_complex"
# subject = "cpython.test_ast.ASTHelpers_Test.test_literal_eval_complex"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent
maxDiff = None
assert ast.literal_eval('6j') == 6j
assert ast.literal_eval('-6j') == -6j
assert ast.literal_eval('6.75j') == 6.75j
assert ast.literal_eval('-6.75j') == -6.75j
assert ast.literal_eval('3+6j') == 3 + 6j
assert ast.literal_eval('-3+6j') == -3 + 6j
assert ast.literal_eval('3-6j') == 3 - 6j
assert ast.literal_eval('-3-6j') == -3 - 6j
assert ast.literal_eval('3.25+6.75j') == 3.25 + 6.75j
assert ast.literal_eval('-3.25+6.75j') == -3.25 + 6.75j
assert ast.literal_eval('3.25-6.75j') == 3.25 - 6.75j
assert ast.literal_eval('-3.25-6.75j') == -3.25 - 6.75j
assert ast.literal_eval('(3+6j)') == 3 + 6j
try:
    ast.literal_eval('-6j+3')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    ast.literal_eval('-6j+3j')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    ast.literal_eval('3+-6j')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    ast.literal_eval('3+(0+6j)')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    ast.literal_eval('-(3+6j)')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass

print("ASTHelpers_Test::test_literal_eval_complex: ok")
"###);
    assert_output(&out, r###"ASTHelpers_Test::test_literal_eval_complex: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t_helpers__test__test_literal_eval_syntax_errors.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t_helpers__test__test_literal_eval_syntax_errors() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_literal_eval_syntax_errors"
# subject = "cpython.test_ast.ASTHelpers_Test.test_literal_eval_syntax_errors"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import unittest, io
from test.test_ast import test_ast
_suite = unittest.defaultTestLoader.loadTestsFromName("ASTHelpers_Test.test_literal_eval_syntax_errors", test_ast)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ASTHelpers_Test.test_literal_eval_syntax_errors did not pass"
print("ASTHelpers_Test::test_literal_eval_syntax_errors: ok")
"###);
    assert_output(&out, r###"ASTHelpers_Test::test_literal_eval_syntax_errors: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t_helpers__test__test_literal_eval_trailing_ws.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t_helpers__test__test_literal_eval_trailing_ws() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_literal_eval_trailing_ws"
# subject = "cpython.test_ast.ASTHelpers_Test.test_literal_eval_trailing_ws"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent
maxDiff = None
assert ast.literal_eval('    -1') == -1
assert ast.literal_eval('\t\t-1') == -1
assert ast.literal_eval(' \t -1') == -1
try:
    ast.literal_eval('\n -1')
    raise AssertionError('assertRaises: no raise')
except IndentationError:
    pass

print("ASTHelpers_Test::test_literal_eval_trailing_ws: ok")
"###);
    assert_output(&out, r###"ASTHelpers_Test::test_literal_eval_trailing_ws: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t_helpers__test__test_multi_line_docstring_col_offset_and_lineno_issue16806.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t_helpers__test__test_multi_line_docstring_col_offset_and_lineno_issue16806() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_multi_line_docstring_col_offset_and_lineno_issue16806"
# subject = "cpython.test_ast.ASTHelpers_Test.test_multi_line_docstring_col_offset_and_lineno_issue16806"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent
maxDiff = None
node = ast.parse('"""line one\nline two"""\n\ndef foo():\n  """line one\n  line two"""\n\n  def bar():\n    """line one\n    line two"""\n  """line one\n  line two"""\n"""line one\nline two"""\n\n')
assert node.body[0].col_offset == 0
assert node.body[0].lineno == 1
assert node.body[1].body[0].col_offset == 2
assert node.body[1].body[0].lineno == 5
assert node.body[1].body[1].body[0].col_offset == 4
assert node.body[1].body[1].body[0].lineno == 9
assert node.body[1].body[2].col_offset == 2
assert node.body[1].body[2].lineno == 11
assert node.body[2].col_offset == 0
assert node.body[2].lineno == 13

print("ASTHelpers_Test::test_multi_line_docstring_col_offset_and_lineno_issue16806: ok")
"###);
    assert_output(&out, r###"ASTHelpers_Test::test_multi_line_docstring_col_offset_and_lineno_issue16806: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/a_s_t_helpers__test__test_starred_expr_end_position_within_call.py`.
#[test]
fn test_gen_behavior_std_libs_ast_a_s_t_helpers__test__test_starred_expr_end_position_within_call() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_starred_expr_end_position_within_call"
# subject = "cpython.test_ast.ASTHelpers_Test.test_starred_expr_end_position_within_call"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent
maxDiff = None
node = ast.parse('f(*[0, 1])')
starred_expr = node.body[0].value.args[0]
assert starred_expr.end_lineno == 1
assert starred_expr.end_col_offset == 9

print("ASTHelpers_Test::test_starred_expr_end_position_within_call: ok")
"###);
    assert_output(&out, r###"ASTHelpers_Test::test_starred_expr_end_position_within_call: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/constant_tests__test_get_docstring.py`.
#[test]
fn test_gen_behavior_std_libs_ast_constant_tests__test_get_docstring() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "constant_tests__test_get_docstring"
# subject = "cpython.test_ast.ConstantTests.test_get_docstring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast


tree = ast.parse("'docstring'\nx = 1")
assert ast.get_docstring(tree) == 'docstring'

print("ConstantTests::test_get_docstring: ok")
"###);
    assert_output(&out, r###"ConstantTests::test_get_docstring: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/constant_tests__test_literal_eval.py`.
#[test]
fn test_gen_behavior_std_libs_ast_constant_tests__test_literal_eval() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "constant_tests__test_literal_eval"
# subject = "cpython.test_ast.ConstantTests.test_literal_eval"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent

def compile_constant(value):
    tree = ast.parse('x = 123')
    node = tree.body[0].value
    new_node = ast.Constant(value=value)
    ast.copy_location(new_node, node)
    tree.body[0].value = new_node
    code = compile(tree, '<string>', 'exec')
    ns = {}
    exec(code, ns)
    return ns['x']

def get_load_const(tree):
    co = compile(tree, '<string>', 'exec')
    consts = []
    for instr in dis.get_instructions(co):
        if instr.opname == 'LOAD_CONST' or instr.opname == 'RETURN_CONST':
            consts.append(instr.argval)
    return consts
tree = ast.parse('1 + 2')
binop = tree.body[0].value
new_left = ast.Constant(value=10)
ast.copy_location(new_left, binop.left)
binop.left = new_left
new_right = ast.Constant(value=20j)
ast.copy_location(new_right, binop.right)
binop.right = new_right
assert ast.literal_eval(binop) == 10 + 20j

print("ConstantTests::test_literal_eval: ok")
"###);
    assert_output(&out, r###"ConstantTests::test_literal_eval: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/constant_tests__test_string_kind.py`.
#[test]
fn test_gen_behavior_std_libs_ast_constant_tests__test_string_kind() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "constant_tests__test_string_kind"
# subject = "cpython.test_ast.ConstantTests.test_string_kind"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent

def compile_constant(value):
    tree = ast.parse('x = 123')
    node = tree.body[0].value
    new_node = ast.Constant(value=value)
    ast.copy_location(new_node, node)
    tree.body[0].value = new_node
    code = compile(tree, '<string>', 'exec')
    ns = {}
    exec(code, ns)
    return ns['x']

def get_load_const(tree):
    co = compile(tree, '<string>', 'exec')
    consts = []
    for instr in dis.get_instructions(co):
        if instr.opname == 'LOAD_CONST' or instr.opname == 'RETURN_CONST':
            consts.append(instr.argval)
    return consts
c = ast.parse('"x"', mode='eval').body
assert c.value == 'x'
assert c.kind == None
c = ast.parse('u"x"', mode='eval').body
assert c.value == 'x'
assert c.kind == 'u'
c = ast.parse('r"x"', mode='eval').body
assert c.value == 'x'
assert c.kind == None
c = ast.parse('b"x"', mode='eval').body
assert c.value == b'x'
assert c.kind == None

print("ConstantTests::test_string_kind: ok")
"###);
    assert_output(&out, r###"ConstantTests::test_string_kind: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/copy_location_copies_location_attrs.py`.
#[test]
fn test_gen_behavior_std_libs_ast_copy_location_copies_location_attrs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "copy_location_copies_location_attrs"
# subject = "ast.copy_location"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
"""ast.copy_location: copy CPython location attributes without clobbering None starts."""
import ast

old = ast.Constant(1)
old.lineno = 7
old.col_offset = 3
old.end_lineno = None
old.end_col_offset = None

new = ast.Constant(2)
copied = ast.copy_location(new, old)

assert copied is new
assert copied.lineno == 7
assert copied.col_offset == 3
assert copied.end_lineno is None
assert copied.end_col_offset is None
assert copied.value == 2

old_without_start = ast.Constant(1)
old_without_start.lineno = None
old_without_start.col_offset = None
old_without_start.end_lineno = None
old_without_start.end_col_offset = None

preserved_start = ast.Constant(3)
preserved_start.lineno = 11
preserved_start.col_offset = 5
preserved = ast.copy_location(preserved_start, old_without_start)

assert preserved.lineno == 11
assert preserved.col_offset == 5
assert preserved.end_lineno is None
assert preserved.end_col_offset is None
print("copy_location_copies_location_attrs OK")
"###);
    assert_output(&out, r###"copy_location_copies_location_attrs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_attribute_spaces.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_attribute_spaces() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_attribute_spaces"
# subject = "cpython.test_ast.EndPositionTests.test_attribute_spaces"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent

def _check_end_pos(ast_node, end_lineno, end_col_offset):
    assert ast_node.end_lineno == end_lineno
    assert ast_node.end_col_offset == end_col_offset

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content

def _parse_value(s):
    return ast.parse(s).body[0].value
s = 'func(x. y .z)'
call = _parse_value(s)
_check_content(s, call, s)
_check_content(s, call.args[0], 'x. y .z')

print("EndPositionTests::test_attribute_spaces: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_attribute_spaces: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_binop.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_binop() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_binop"
# subject = "cpython.test_ast.EndPositionTests.test_binop"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent

def _check_end_pos(ast_node, end_lineno, end_col_offset):
    assert ast_node.end_lineno == end_lineno
    assert ast_node.end_col_offset == end_col_offset

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content

def _parse_value(s):
    return ast.parse(s).body[0].value
s = dedent('\n            (1 * 2 + (3 ) +\n                 4\n            )\n        ').strip()
binop = _parse_value(s)
_check_end_pos(binop, 2, 6)
_check_content(s, binop.right, '4')
_check_content(s, binop.left, '1 * 2 + (3 )')
_check_content(s, binop.left.right, '3')

print("EndPositionTests::test_binop: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_binop: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_boolop.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_boolop() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_boolop"
# subject = "cpython.test_ast.EndPositionTests.test_boolop"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent

def _check_end_pos(ast_node, end_lineno, end_col_offset):
    assert ast_node.end_lineno == end_lineno
    assert ast_node.end_col_offset == end_col_offset

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content

def _parse_value(s):
    return ast.parse(s).body[0].value
s = dedent('\n            if (one_condition and\n                    (other_condition or yet_another_one)):\n                pass\n        ').strip()
bop = ast.parse(s).body[0].test
_check_end_pos(bop, 2, 44)
_check_content(s, bop.values[1], 'other_condition or yet_another_one')

print("EndPositionTests::test_boolop: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_boolop: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_call.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_call() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_call"
# subject = "cpython.test_ast.EndPositionTests.test_call"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

def _check_end_pos(ast_node, end_lineno, end_col_offset):
    assert ast_node.end_lineno == end_lineno
    assert ast_node.end_col_offset == end_col_offset

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content

def _parse_value(s):
    return ast.parse(s).body[0].value
s = 'func(x, y=2, **kw)'
call = _parse_value(s)
_check_content(s, call.func, 'func')
_check_content(s, call.keywords[0].value, '2')
_check_content(s, call.keywords[1].value, 'kw')

print("EndPositionTests::test_call: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_call: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_call_noargs.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_call_noargs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_call_noargs"
# subject = "cpython.test_ast.EndPositionTests.test_call_noargs"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

def _check_end_pos(ast_node, end_lineno, end_col_offset):
    assert ast_node.end_lineno == end_lineno
    assert ast_node.end_col_offset == end_col_offset

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content

def _parse_value(s):
    return ast.parse(s).body[0].value
s = 'x[0]()'
call = _parse_value(s)
_check_content(s, call.func, 'x[0]')
_check_end_pos(call, 1, 6)

print("EndPositionTests::test_call_noargs: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_call_noargs: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_class_def.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_class_def() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_class_def"
# subject = "cpython.test_ast.EndPositionTests.test_class_def"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
from textwrap import dedent

def _check_end_pos(ast_node, end_lineno, end_col_offset):
    assert ast_node.end_lineno == end_lineno
    assert ast_node.end_col_offset == end_col_offset

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content
s = dedent('\n            class C(A, B):\n                x: int = 0\n        ').strip()
cdef = ast.parse(s).body[0]
_check_end_pos(cdef, 2, 14)
_check_content(s, cdef.bases[1], 'B')
_check_content(s, cdef.body[0], 'x: int = 0')

print("EndPositionTests::test_class_def: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_class_def: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_class_kw.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_class_kw() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_class_kw"
# subject = "cpython.test_ast.EndPositionTests.test_class_kw"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content
s = 'class S(metaclass=abc.ABCMeta): pass'
cdef = ast.parse(s).body[0]
_check_content(s, cdef.keywords[0].value, 'abc.ABCMeta')

print("EndPositionTests::test_class_kw: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_class_kw: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_continued_str.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_continued_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_continued_str"
# subject = "cpython.test_ast.EndPositionTests.test_continued_str"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
from textwrap import dedent

def _check_end_pos(ast_node, end_lineno, end_col_offset):
    assert ast_node.end_lineno == end_lineno
    assert ast_node.end_col_offset == end_col_offset

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content

def _parse_value(s):
    return ast.parse(s).body[0].value
s = dedent('\n            x = "first part" \\\n            "second part"\n        ').strip()
assign = ast.parse(s).body[0]
_check_end_pos(assign, 2, 13)
_check_end_pos(assign.value, 2, 13)

print("EndPositionTests::test_continued_str: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_continued_str: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_displays.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_displays() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_displays"
# subject = "cpython.test_ast.EndPositionTests.test_displays"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent

def _check_end_pos(ast_node, end_lineno, end_col_offset):
    assert ast_node.end_lineno == end_lineno
    assert ast_node.end_col_offset == end_col_offset

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content

def _parse_value(s):
    return ast.parse(s).body[0].value
s1 = '[{}, {1, }, {1, 2,} ]'
s2 = '{a: b, f (): g () ,}'
c1 = _parse_value(s1)
c2 = _parse_value(s2)
_check_content(s1, c1.elts[0], '{}')
_check_content(s1, c1.elts[1], '{1, }')
_check_content(s1, c1.elts[2], '{1, 2,}')
_check_content(s2, c2.keys[1], 'f ()')
_check_content(s2, c2.values[1], 'g ()')

print("EndPositionTests::test_displays: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_displays: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_func_def.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_func_def() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_func_def"
# subject = "cpython.test_ast.EndPositionTests.test_func_def"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent

def _check_end_pos(ast_node, end_lineno, end_col_offset):
    assert ast_node.end_lineno == end_lineno
    assert ast_node.end_col_offset == end_col_offset

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content

def _parse_value(s):
    return ast.parse(s).body[0].value
s = dedent('\n            def func(x: int,\n                     *args: str,\n                     z: float = 0,\n                     **kwargs: Any) -> bool:\n                return True\n            ').strip()
fdef = ast.parse(s).body[0]
_check_end_pos(fdef, 5, 15)
_check_content(s, fdef.body[0], 'return True')
_check_content(s, fdef.args.args[0], 'x: int')
_check_content(s, fdef.args.args[0].annotation, 'int')
_check_content(s, fdef.args.kwarg, 'kwargs: Any')
_check_content(s, fdef.args.kwarg.annotation, 'Any')

print("EndPositionTests::test_func_def: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_func_def: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_import_from_multi_line.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_import_from_multi_line() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_import_from_multi_line"
# subject = "cpython.test_ast.EndPositionTests.test_import_from_multi_line"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent

def _check_end_pos(ast_node, end_lineno, end_col_offset):
    assert ast_node.end_lineno == end_lineno
    assert ast_node.end_col_offset == end_col_offset

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content

def _parse_value(s):
    return ast.parse(s).body[0].value
s = dedent('\n            from x.y.z import (\n                a, b, c as c\n            )\n        ').strip()
imp = ast.parse(s).body[0]
_check_end_pos(imp, 3, 1)
_check_end_pos(imp.names[2], 2, 16)

print("EndPositionTests::test_import_from_multi_line: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_import_from_multi_line: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_lambda.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_lambda() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_lambda"
# subject = "cpython.test_ast.EndPositionTests.test_lambda"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

def _check_end_pos(ast_node, end_lineno, end_col_offset):
    assert ast_node.end_lineno == end_lineno
    assert ast_node.end_col_offset == end_col_offset

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content

def _parse_value(s):
    return ast.parse(s).body[0].value
s = 'lambda x, *y: None'
lam = _parse_value(s)
_check_content(s, lam.body, 'None')
_check_content(s, lam.args.args[0], 'x')
_check_content(s, lam.args.vararg, 'y')

print("EndPositionTests::test_lambda: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_lambda: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_multi_line_str.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_multi_line_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_multi_line_str"
# subject = "cpython.test_ast.EndPositionTests.test_multi_line_str"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent

def _check_end_pos(ast_node, end_lineno, end_col_offset):
    assert ast_node.end_lineno == end_lineno
    assert ast_node.end_col_offset == end_col_offset

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content

def _parse_value(s):
    return ast.parse(s).body[0].value
s = dedent('\n            x = """Some multi-line text.\n\n            It goes on starting from same indent."""\n        ').strip()
assign = ast.parse(s).body[0]
_check_end_pos(assign, 3, 40)
_check_end_pos(assign.value, 3, 40)

print("EndPositionTests::test_multi_line_str: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_multi_line_str: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_redundant_parenthesis.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_redundant_parenthesis() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_redundant_parenthesis"
# subject = "cpython.test_ast.EndPositionTests.test_redundant_parenthesis"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

def _check_end_pos(ast_node, end_lineno, end_col_offset):
    assert ast_node.end_lineno == end_lineno
    assert ast_node.end_col_offset == end_col_offset

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content

def _parse_value(s):
    return ast.parse(s).body[0].value
s = '( ( ( a + b ) ) )'
v = ast.parse(s).body[0].value
assert type(v).__name__ == 'BinOp'
_check_content(s, v, 'a + b')
s2 = 'await ' + s
v = ast.parse(s2).body[0].value.value
assert type(v).__name__ == 'BinOp'
_check_content(s2, v, 'a + b')

print("EndPositionTests::test_redundant_parenthesis: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_redundant_parenthesis: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_slices.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_slices() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_slices"
# subject = "cpython.test_ast.EndPositionTests.test_slices"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent

def _check_end_pos(ast_node, end_lineno, end_col_offset):
    assert ast_node.end_lineno == end_lineno
    assert ast_node.end_col_offset == end_col_offset

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content

def _parse_value(s):
    return ast.parse(s).body[0].value
s1 = 'f()[1, 2] [0]'
s2 = 'x[ a.b: c.d]'
sm = dedent('\n            x[ a.b: f () ,\n               g () : c.d\n              ]\n        ').strip()
i1, i2, im = map(_parse_value, (s1, s2, sm))
_check_content(s1, i1.value, 'f()[1, 2]')
_check_content(s1, i1.value.slice, '1, 2')
_check_content(s2, i2.slice.lower, 'a.b')
_check_content(s2, i2.slice.upper, 'c.d')
_check_content(sm, im.slice.elts[0].upper, 'f ()')
_check_content(sm, im.slice.elts[1].lower, 'g ()')
_check_end_pos(im, 3, 3)

print("EndPositionTests::test_slices: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_slices: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_source_segment_endings.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_source_segment_endings() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_source_segment_endings"
# subject = "cpython.test_ast.EndPositionTests.test_source_segment_endings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content
s = 'v = 1\r\nw = 1\nx = 1\n\ry = 1\rz = 1\r\n'
v, w, x, y, z = ast.parse(s).body
_check_content(s, v, 'v = 1')
_check_content(s, w, 'w = 1')
_check_content(s, x, 'x = 1')
_check_content(s, y, 'y = 1')
_check_content(s, z, 'z = 1')

print("EndPositionTests::test_source_segment_endings: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_source_segment_endings: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_source_segment_missing_info.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_source_segment_missing_info() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_source_segment_missing_info"
# subject = "cpython.test_ast.EndPositionTests.test_source_segment_missing_info"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

def _check_end_pos(ast_node, end_lineno, end_col_offset):
    assert ast_node.end_lineno == end_lineno
    assert ast_node.end_col_offset == end_col_offset

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content

def _parse_value(s):
    return ast.parse(s).body[0].value
s = 'v = 1\r\nw = 1\nx = 1\n\ry = 1\r\n'
v, w, x, y = ast.parse(s).body
del v.lineno
del w.end_lineno
del x.col_offset
del y.end_col_offset
assert ast.get_source_segment(s, v) is None
assert ast.get_source_segment(s, w) is None
assert ast.get_source_segment(s, x) is None
assert ast.get_source_segment(s, y) is None

print("EndPositionTests::test_source_segment_missing_info: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_source_segment_missing_info: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_source_segment_multi.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_source_segment_multi() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_source_segment_multi"
# subject = "cpython.test_ast.EndPositionTests.test_source_segment_multi"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
from textwrap import dedent

def _parse_value(s):
    return ast.parse(s).body[0].value
s_orig = dedent('\n            x = (\n                a, b,\n            ) + ()\n        ').strip()
s_tuple = dedent('\n            (\n                a, b,\n            )\n        ').strip()
binop = _parse_value(s_orig)
assert ast.get_source_segment(s_orig, binop.left) == s_tuple

print("EndPositionTests::test_source_segment_multi: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_source_segment_multi: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_source_segment_newlines.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_source_segment_newlines() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_source_segment_newlines"
# subject = "cpython.test_ast.EndPositionTests.test_source_segment_newlines"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content
s = 'def f():\n  pass\ndef g():\r  pass\r\ndef h():\r\n  pass\r\n'
f, g, h = ast.parse(s).body
_check_content(s, f, 'def f():\n  pass')
_check_content(s, g, 'def g():\r  pass')
_check_content(s, h, 'def h():\r\n  pass')
s = 'def f():\n  a = 1\r  b = 2\r\n  c = 3\n'
f = ast.parse(s).body[0]
_check_content(s, f, s.rstrip())

print("EndPositionTests::test_source_segment_newlines: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_source_segment_newlines: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_source_segment_padded.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_source_segment_padded() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_source_segment_padded"
# subject = "cpython.test_ast.EndPositionTests.test_source_segment_padded"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
from textwrap import dedent
s_orig = dedent('\n            class C:\n                def fun(self) -> None:\n                    "ЖЖЖЖЖ"\n        ').strip()
s_method = '    def fun(self) -> None:\n        "ЖЖЖЖЖ"'
cdef = ast.parse(s_orig).body[0]
assert ast.get_source_segment(s_orig, cdef.body[0], padded=True) == s_method

print("EndPositionTests::test_source_segment_padded: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_source_segment_padded: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_source_segment_tabs.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_source_segment_tabs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_source_segment_tabs"
# subject = "cpython.test_ast.EndPositionTests.test_source_segment_tabs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
from textwrap import dedent
s = dedent('\n            class C:\n              \t\x0c  def fun(self) -> None:\n              \t\x0c      pass\n        ').strip()
s_method = '  \t\x0c  def fun(self) -> None:\n  \t\x0c      pass'
cdef = ast.parse(s).body[0]
assert ast.get_source_segment(s, cdef.body[0], padded=True) == s_method

print("EndPositionTests::test_source_segment_tabs: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_source_segment_tabs: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_suites.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_suites() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_suites"
# subject = "cpython.test_ast.EndPositionTests.test_suites"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent

def _check_end_pos(ast_node, end_lineno, end_col_offset):
    assert ast_node.end_lineno == end_lineno
    assert ast_node.end_col_offset == end_col_offset

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content

def _parse_value(s):
    return ast.parse(s).body[0].value
s = dedent('\n            while True:\n                pass\n\n            if one():\n                x = None\n            elif other():\n                y = None\n            else:\n                z = None\n\n            for x, y in stuff:\n                assert True\n\n            try:\n                raise RuntimeError\n            except TypeError as e:\n                pass\n\n            pass\n        ').strip()
mod = ast.parse(s)
while_loop = mod.body[0]
if_stmt = mod.body[1]
for_loop = mod.body[2]
try_stmt = mod.body[3]
pass_stmt = mod.body[4]
_check_end_pos(while_loop, 2, 8)
_check_end_pos(if_stmt, 9, 12)
_check_end_pos(for_loop, 12, 15)
_check_end_pos(try_stmt, 17, 8)
_check_end_pos(pass_stmt, 19, 4)
_check_content(s, while_loop.test, 'True')
_check_content(s, if_stmt.body[0], 'x = None')
_check_content(s, if_stmt.orelse[0].test, 'other()')
_check_content(s, for_loop.target, 'x, y')
_check_content(s, try_stmt.body[0], 'raise RuntimeError')
_check_content(s, try_stmt.handlers[0].type, 'TypeError')

print("EndPositionTests::test_suites: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_suites: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_tuples.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_tuples() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_tuples"
# subject = "cpython.test_ast.EndPositionTests.test_tuples"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
from textwrap import dedent

def _check_end_pos(ast_node, end_lineno, end_col_offset):
    assert ast_node.end_lineno == end_lineno
    assert ast_node.end_col_offset == end_col_offset

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content

def _parse_value(s):
    return ast.parse(s).body[0].value
s1 = 'x = () ;'
s2 = 'x = 1 , ;'
s3 = 'x = (1 , 2 ) ;'
sm = dedent('\n            x = (\n                a, b,\n            )\n        ').strip()
t1, t2, t3, tm = map(_parse_value, (s1, s2, s3, sm))
_check_content(s1, t1, '()')
_check_content(s2, t2, '1 ,')
_check_content(s3, t3, '(1 , 2 )')
_check_end_pos(tm, 3, 1)

print("EndPositionTests::test_tuples: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_tuples: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/end_position_tests__test_yield_await.py`.
#[test]
fn test_gen_behavior_std_libs_ast_end_position_tests__test_yield_await() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_yield_await"
# subject = "cpython.test_ast.EndPositionTests.test_yield_await"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent

def _check_end_pos(ast_node, end_lineno, end_col_offset):
    assert ast_node.end_lineno == end_lineno
    assert ast_node.end_col_offset == end_col_offset

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content

def _parse_value(s):
    return ast.parse(s).body[0].value
s = dedent('\n            async def f():\n                yield x\n                await y\n        ').strip()
fdef = ast.parse(s).body[0]
_check_content(s, fdef.body[0].value, 'yield x')
_check_content(s, fdef.body[1].value, 'await y')

print("EndPositionTests::test_yield_await: ok")
"###);
    assert_output(&out, r###"EndPositionTests::test_yield_await: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ast/module_state_tests__test_reload_module.py`.
#[test]
fn test_gen_behavior_std_libs_ast_module_state_tests__test_reload_module() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "module_state_tests__test_reload_module"
# subject = "cpython.test_ast.ModuleStateTests.test_reload_module"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import gc
import sys

saved = sys.modules.get("_ast")
try:
    sys.modules.pop("_ast", None)
    import _ast as ast1

    sys.modules.pop("_ast", None)
    import _ast as ast2
finally:
    if saved is not None:
        sys.modules["_ast"] = saved

del ast1
del ast2
gc.collect()

print("ModuleStateTests::test_reload_module: ok")
"###);
    assert_output(&out, r###"ModuleStateTests::test_reload_module: ok
"###);
}
