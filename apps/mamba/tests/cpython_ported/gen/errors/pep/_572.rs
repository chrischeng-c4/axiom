use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/pep/572/attribute_target_rejected.py`.
#[test]
fn test_gen_errors_pep_572_attribute_target_rejected() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "errors"
# case = "attribute_target_rejected"
# subject = ":="
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: attribute_target_rejected (errors)."""
pass

_raised = False
try:
    compile('(o.x := 5)', '<t>', 'exec')
except SyntaxError:
    _raised = True
assert _raised, "attribute_target_rejected: expected SyntaxError"
print("attribute_target_rejected OK")
"###);
    assert_output(&out, r###"attribute_target_rejected OK
"###);
}

/// Ported from `tests/cpython/errors/pep/572/bare_walrus_statement_rejected.py`.
#[test]
fn test_gen_errors_pep_572_bare_walrus_statement_rejected() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "errors"
# case = "bare_walrus_statement_rejected"
# subject = ":="
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: bare_walrus_statement_rejected (errors)."""
pass

_raised = False
try:
    compile('a := 5', '<t>', 'exec')
except SyntaxError:
    _raised = True
assert _raised, "bare_walrus_statement_rejected: expected SyntaxError"
print("bare_walrus_statement_rejected OK")
"###);
    assert_output(&out, r###"bare_walrus_statement_rejected OK
"###);
}

/// Ported from `tests/cpython/errors/pep/572/class_body_comprehension_rejected.py`.
#[test]
fn test_gen_errors_pep_572_class_body_comprehension_rejected() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "errors"
# case = "class_body_comprehension_rejected"
# subject = ":="
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: class_body_comprehension_rejected (errors)."""
pass

_raised = False
try:
    compile('class Foo:\n    [(42, j := i) for i in range(5)]\n', '<t>', 'exec')
except SyntaxError:
    _raised = True
assert _raised, "class_body_comprehension_rejected: expected SyntaxError"
print("class_body_comprehension_rejected OK")
"###);
    assert_output(&out, r###"class_body_comprehension_rejected OK
"###);
}

/// Ported from `tests/cpython/errors/pep/572/comprehension_iterable_target_rejected.py`.
#[test]
fn test_gen_errors_pep_572_comprehension_iterable_target_rejected() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "errors"
# case = "comprehension_iterable_target_rejected"
# subject = ":="
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: comprehension_iterable_target_rejected (errors)."""
pass

_raised = False
try:
    compile('[i + 1 for i in i := [1, 2]]', '<t>', 'exec')
except SyntaxError:
    _raised = True
assert _raised, "comprehension_iterable_target_rejected: expected SyntaxError"
print("comprehension_iterable_target_rejected OK")
"###);
    assert_output(&out, r###"comprehension_iterable_target_rejected OK
"###);
}

/// Ported from `tests/cpython/errors/pep/572/lambda_target_rejected.py`.
#[test]
fn test_gen_errors_pep_572_lambda_target_rejected() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "errors"
# case = "lambda_target_rejected"
# subject = ":="
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: lambda_target_rejected (errors)."""
pass

_raised = False
try:
    compile('(lambda: x := 1)', '<t>', 'exec')
except SyntaxError:
    _raised = True
assert _raised, "lambda_target_rejected: expected SyntaxError"
print("lambda_target_rejected OK")
"###);
    assert_output(&out, r###"lambda_target_rejected OK
"###);
}

/// Ported from `tests/cpython/errors/pep/572/rebind_iteration_variable_rejected.py`.
#[test]
fn test_gen_errors_pep_572_rebind_iteration_variable_rejected() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "errors"
# case = "rebind_iteration_variable_rejected"
# subject = ":="
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: rebind_iteration_variable_rejected (errors)."""
pass

_raised = False
try:
    compile('[[(__x := 2) for _ in range(2)] for __x in range(2)]', '<t>', 'exec')
except SyntaxError:
    _raised = True
assert _raised, "rebind_iteration_variable_rejected: expected SyntaxError"
print("rebind_iteration_variable_rejected OK")
"###);
    assert_output(&out, r###"rebind_iteration_variable_rejected OK
"###);
}

/// Ported from `tests/cpython/errors/pep/572/subscript_target_rejected.py`.
#[test]
fn test_gen_errors_pep_572_subscript_target_rejected() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "errors"
# case = "subscript_target_rejected"
# subject = ":="
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: subscript_target_rejected (errors)."""
pass

_raised = False
try:
    compile("(d := {}); (d['k'] := 1)", '<t>', 'exec')
except SyntaxError:
    _raised = True
assert _raised, "subscript_target_rejected: expected SyntaxError"
print("subscript_target_rejected OK")
"###);
    assert_output(&out, r###"subscript_target_rejected OK
"###);
}

/// Ported from `tests/cpython/errors/pep/572/tuple_target_rejected.py`.
#[test]
fn test_gen_errors_pep_572_tuple_target_rejected() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "errors"
# case = "tuple_target_rejected"
# subject = ":="
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: tuple_target_rejected (errors)."""
pass

_raised = False
try:
    compile('((a, b) := (1, 2))', '<t>', 'exec')
except SyntaxError:
    _raised = True
assert _raised, "tuple_target_rejected: expected SyntaxError"
print("tuple_target_rejected OK")
"###);
    assert_output(&out, r###"tuple_target_rejected OK
"###);
}
