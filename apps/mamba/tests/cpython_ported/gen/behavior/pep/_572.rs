use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/pep/572/bare_named_expr_binds.py`.
#[test]
fn test_gen_behavior_pep_572_bare_named_expr_binds() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "bare_named_expr_binds"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a bare parenthesized named expression (a := 10) binds the name and the statement's value is the assigned value"""
# A bare named expression binds the name and the statement is the value.
(a := 10)
assert a == 10

print("bare_named_expr_binds OK")
"###);
    assert_output(&out, r###"bare_named_expr_binds OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/572/call_arg_walrus_leaks.py`.
#[test]
fn test_gen_behavior_pep_572_call_arg_walrus_leaks() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "call_arg_walrus_leaks"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus passed as a positional call argument binds in the surrounding scope after the call"""
# A walrus passed as a call argument binds in the surrounding scope.
def identity(value):
    return value

out = identity((arg := 2))
assert out == 2
assert arg == 2

print("call_arg_walrus_leaks OK")
"###);
    assert_output(&out, r###"call_arg_walrus_leaks OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/572/chained_walrus_same_value.py`.
#[test]
fn test_gen_behavior_pep_572_chained_walrus_same_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "chained_walrus_same_value"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: chained walrus (z := (y := (x := 0))) binds every name to the same innermost value 0"""
# Chained walrus binds every name to the same innermost value.
(z := (y := (x := 0)))
assert x == 0 and y == 0 and z == 0

print("chained_walrus_same_value OK")
"###);
    assert_output(&out, r###"chained_walrus_same_value OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/572/comp_filter_binding_reused_and_leaks.py`.
#[test]
fn test_gen_behavior_pep_572_comp_filter_binding_reused_and_leaks() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "comp_filter_binding_reused_and_leaks"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus in a comprehension filter binds per-element, is reusable in that element's value expression, and the last filter binding leaks to the enclosing scope"""
# A walrus in the comprehension's filter binds per-element and is reusable
# in the value expression of the same element.
def positive(value):
    return value

rows = [(x, y, x / y) for x in [1, 2, 3] if (y := positive(x)) > 0]
assert rows == [(1, 1, 1.0), (2, 2, 1.0), (3, 3, 1.0)]
# The last filter binding leaks to the enclosing scope.
assert y == 3

print("comp_filter_binding_reused_and_leaks OK")
"###);
    assert_output(&out, r###"comp_filter_binding_reused_and_leaks OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/572/comp_preseeded_accumulator.py`.
#[test]
fn test_gen_behavior_pep_572_comp_preseeded_accumulator() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "comp_preseeded_accumulator"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a pre-seeded accumulator outside the comprehension is honored by the walrus accumulation inside it"""
# Pre-seeding the accumulator outside the comprehension is honored.
acc = 0
sums = [(acc := i + acc) for i in range(5)]
assert sums == [0, 1, 3, 6, 10]
assert acc == 10

print("comp_preseeded_accumulator OK")
"###);
    assert_output(&out, r###"comp_preseeded_accumulator OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/572/comp_running_total_accumulator.py`.
#[test]
fn test_gen_behavior_pep_572_comp_running_total_accumulator() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "comp_running_total_accumulator"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus that reads and writes the same enclosing name accumulates a running total across a comprehension: [(total := total + v) for v in range(5)] yields [0,1,3,6,10] and total == 10"""
# A walrus that reads and writes the same enclosing name accumulates a
# running total across the comprehension.
total = 0
partial = [(total := total + v) for v in range(5)]
assert partial == [0, 1, 3, 6, 10]
assert total == 10

print("comp_running_total_accumulator OK")
"###);
    assert_output(&out, r###"comp_running_total_accumulator OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/572/comp_value_binding_reused_in_element.py`.
#[test]
fn test_gen_behavior_pep_572_comp_value_binding_reused_in_element() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "comp_value_binding_reused_in_element"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus bound in a comprehension's value expression can be reused later in the same element, and the final binding leaks to the enclosing scope"""
# The walrus value can be reused later in the element expression.
def positive(value):
    return value

ratios = [[(z := positive(n)), n / z] for n in range(1, 5)]
assert ratios == [[1, 1.0], [2, 1.0], [3, 1.0], [4, 1.0]]
# The final binding leaks to the enclosing scope.
assert z == 4

print("comp_value_binding_reused_in_element OK")
"###);
    assert_output(&out, r###"comp_value_binding_reused_in_element OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/572/default_arg_walrus_allowed.py`.
#[test]
fn test_gen_behavior_pep_572_default_arg_walrus_allowed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "default_arg_walrus_allowed"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus in a parameter default is allowed (the def does not raise): def f(x=(n := 5)) compiles and f() returns 5"""
# A walrus in a parameter default is allowed; the def does not raise.
def f(x=(n := 5)):
    return x

# The default is evaluated once at def time, binding n in the enclosing scope.
assert n == 5
assert f() == 5
assert f(99) == 99

print("default_arg_walrus_allowed OK")
"###);
    assert_output(&out, r###"default_arg_walrus_allowed OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/572/falsy_while_guard_binds_no_body.py`.
#[test]
fn test_gen_behavior_pep_572_falsy_while_guard_binds_no_body() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "falsy_while_guard_binds_no_body"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus while-guard that is falsy never runs the body but still binds the name: while (flag := False) leaves flag False and the body unexecuted"""
# Walrus loop guard that is falsy never runs the body but still binds.
ran = False
while (flag := False):
    ran = True
assert ran is False
assert flag is False

print("falsy_while_guard_binds_no_body OK")
"###);
    assert_output(&out, r###"falsy_while_guard_binds_no_body OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/572/function_local_does_not_escape.py`.
#[test]
fn test_gen_behavior_pep_572_function_local_does_not_escape() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "function_local_does_not_escape"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus target inside a function is local and does not escape to the module after the function returns"""
# A walrus target inside a function is local; it does not escape to the
# module after the function returns.
def assign_inside():
    (secret := 5)
    return secret

assert assign_inside() == 5
assert "secret" not in globals()

print("function_local_does_not_escape OK")
"###);
    assert_output(&out, r###"function_local_does_not_escape OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/572/genexp_arg_walrus_leaks_after_consume.py`.
#[test]
fn test_gen_behavior_pep_572_genexp_arg_walrus_leaks_after_consume() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "genexp_arg_walrus_leaks_after_consume"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus inside a generator-expression argument to a builtin (any) leaks to the surrounding scope once the generator has been consumed"""
# A walrus inside a generator expression argument to a builtin also
# leaks once the generator has been consumed.
contains_one = any((last := num) == 1 for num in [3, 2, 1])
assert contains_one is True
assert last == 1

print("genexp_arg_walrus_leaks_after_consume OK")
"###);
    assert_output(&out, r###"genexp_arg_walrus_leaks_after_consume OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/572/genexp_walrus_no_preleak_until_consumed.py`.
#[test]
fn test_gen_behavior_pep_572_genexp_walrus_no_preleak_until_consumed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "genexp_walrus_no_preleak_until_consumed"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a generator expression does NOT pre-leak its walrus target before consumption; the bound name only appears after the generator is iterated"""
# A generator expression does NOT pre-leak its walrus target before it is
# consumed; the name only appears after iteration.
seed = 1
genexp = ((c := i + seed) for i in [1, 2, 3, 4])
assert "c" not in locals()
produced = list(genexp)
assert produced == [2, 3, 4, 5]
assert c == 5

print("genexp_walrus_no_preleak_until_consumed OK")
"###);
    assert_output(&out, r###"genexp_walrus_no_preleak_until_consumed OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/572/global_walrus_writes_module_binding.py`.
#[test]
fn test_gen_behavior_pep_572_global_walrus_writes_module_binding() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "global_walrus_writes_module_binding"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus inside a function declared global writes the module-level binding"""
# A walrus inside a function with `global` writes the module binding.
g = 1

def bump():
    global g
    (g := 20)

bump()
assert g == 20

print("global_walrus_writes_module_binding OK")
"###);
    assert_output(&out, r###"global_walrus_writes_module_binding OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/572/keyword_arg_walrus_leaks.py`.
#[test]
fn test_gen_behavior_pep_572_keyword_arg_walrus_leaks() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "keyword_arg_walrus_leaks"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus passed as a keyword call argument binds in the surrounding scope after the call"""
# A walrus passed as a keyword argument binds in the surrounding scope.
def identity(value):
    return value

out2 = identity(value=(kw := 7))
assert out2 == 7
assert kw == 7

print("keyword_arg_walrus_leaks OK")
"###);
    assert_output(&out, r###"keyword_arg_walrus_leaks OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/572/list_comp_walrus_binds_enclosing_scope.py`.
#[test]
fn test_gen_behavior_pep_572_list_comp_walrus_binds_enclosing_scope() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "list_comp_walrus_binds_enclosing_scope"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus target in a list comprehension binds in the ENCLOSING scope, not the comprehension's own scope (the deliberate PEP 572 exception): [(j := i) for i in range(5)] leaves j == 4"""
# A walrus target in a list comprehension binds in the ENCLOSING scope,
# not the comprehension's own scope (a deliberate PEP 572 exception).
res = [(j := i) for i in range(5)]
assert res == [0, 1, 2, 3, 4]
assert j == 4

print("list_comp_walrus_binds_enclosing_scope OK")
"###);
    assert_output(&out, r###"list_comp_walrus_binds_enclosing_scope OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/572/named_expr_value_usable_inline.py`.
#[test]
fn test_gen_behavior_pep_572_named_expr_value_usable_inline() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "named_expr_value_usable_inline"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a named expression evaluates to the assigned value usable inline: len((lines := [1,2,3])) is 3 and lines is bound to the list"""
# A named expression evaluates to the assigned value, usable inline.
total = len((lines := [1, 2, 3]))
assert total == 3
assert lines == [1, 2, 3]

print("named_expr_value_usable_inline OK")
"###);
    assert_output(&out, r###"named_expr_value_usable_inline OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/572/nested_comp_walrus_leaks_enclosing.py`.
#[test]
fn test_gen_behavior_pep_572_nested_comp_walrus_leaks_enclosing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "nested_comp_walrus_leaks_enclosing"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: the enclosing-scope walrus binding survives nesting; the inner walrus of a nested comprehension leaks out to the surrounding scope"""
# The enclosing binding survives nesting; the inner walrus leaks out.
nested = [[(spam := i) for i in range(3)] for _ in range(2)]
assert nested == [[0, 1, 2], [0, 1, 2]]
assert spam == 2

print("nested_comp_walrus_leaks_enclosing OK")
"###);
    assert_output(&out, r###"nested_comp_walrus_leaks_enclosing OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/572/slice_with_walrus_index_allowed.py`.
#[test]
fn test_gen_behavior_pep_572_slice_with_walrus_index_allowed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "slice_with_walrus_index_allowed"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a parenthesized walrus inside a slice bound is allowed: s[(i := 0):] compiles, runs, and binds i == 0"""
# A parenthesized walrus inside a slice bound is allowed.
s = "abc"
sliced = s[(i := 0):]
assert i == 0
assert sliced == "abc"

print("slice_with_walrus_index_allowed OK")
"###);
    assert_output(&out, r###"slice_with_walrus_index_allowed OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/572/walrus_in_if_guard_binds.py`.
#[test]
fn test_gen_behavior_pep_572_walrus_in_if_guard_binds() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "walrus_in_if_guard_binds"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus in an if-condition binds the name and the bound value is visible in the body: if (n := 10) > 5 sees n == 10"""
# Walrus binds + returns the value; the binding is visible in the body.
taken = False
if (n := 10) > 5:
    taken = True
    assert n == 10
assert taken is True
assert n == 10

print("walrus_in_if_guard_binds OK")
"###);
    assert_output(&out, r###"walrus_in_if_guard_binds OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/572/walrus_in_subscript_index.py`.
#[test]
fn test_gen_behavior_pep_572_walrus_in_subscript_index() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "walrus_in_subscript_index"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus inside a subscript index binds and indexes in one expression: data[(pos := 2)] binds pos == 2 and picks data[2]"""
# Walrus inside a subscript index binds and indexes in one expression.
data = [10, 20, 30]
picked = data[(pos := 2)]
assert pos == 2
assert picked == 30

print("walrus_in_subscript_index OK")
"###);
    assert_output(&out, r###"walrus_in_subscript_index OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/572/walrus_in_while_condition_reevaluated.py`.
#[test]
fn test_gen_behavior_pep_572_walrus_in_while_condition_reevaluated() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "walrus_in_while_condition_reevaluated"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus in a while-condition is re-evaluated each iteration; an integer floor-sqrt loop over 9 converges to base == 1"""
# Walrus in a while-condition is re-evaluated each iteration; here it
# computes the integer floor square root of 9.
base, root, target = 9, 2, 3
while base > (step := (target // base ** (root - 1))):
    base = ((root - 1) * base + step) // root
assert base == 1

print("walrus_in_while_condition_reevaluated OK")
"###);
    assert_output(&out, r###"walrus_in_while_condition_reevaluated OK
"###);
}
