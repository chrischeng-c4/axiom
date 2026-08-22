use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/pep/walrus/comp_filter_walrus_valid_form.py`.
#[test]
fn test_gen_behavior_pep_walrus_comp_filter_walrus_valid_form() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "walrus"
# dimension = "behavior"
# case = "comp_filter_walrus_valid_form"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus bound in a comprehension filter is usable as the element expression: [d for v in range(5) if (d := v*2) > 2] yields [4, 6, 8]"""
# A filter-bound walrus is usable as the element expression.
valid = [doubled for v in range(5) if (doubled := v * 2) > 2]
assert valid == [4, 6, 8], f"valid = {valid!r}"

print("comp_filter_walrus_valid_form OK")
"###);
    assert_output(&out, r###"comp_filter_walrus_valid_form OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/walrus/comp_walrus_leaks_to_enclosing.py`.
#[test]
fn test_gen_behavior_pep_walrus_comp_walrus_leaks_to_enclosing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "walrus"
# dimension = "behavior"
# case = "comp_walrus_leaks_to_enclosing"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus target inside a comprehension leaks to the enclosing scope, holding the last value assigned during iteration"""
# A comprehension walrus target leaks out, holding the last assigned value.
leak = None
lst = [leak := v for v in range(5)]
assert lst == [0, 1, 2, 3, 4], f"lst = {lst!r}"
assert leak == 4, f"leak = {leak!r}"

print("comp_walrus_leaks_to_enclosing OK")
"###);
    assert_output(&out, r###"comp_walrus_leaks_to_enclosing OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/walrus/lower_precedence_than_arithmetic.py`.
#[test]
fn test_gen_behavior_pep_walrus_lower_precedence_than_arithmetic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "walrus"
# dimension = "behavior"
# case = "lower_precedence_than_arithmetic"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: the walrus has lower precedence than arithmetic, so y = (x := 1) + 10 binds x to 1 and the whole expression to 11"""
# := binds 1, then the addition runs: the whole expression is 11.
x = 0
y = (x := 1) + 10
assert x == 1, f"x = {x!r}"
assert y == 11, f"y = {y!r}"

print("lower_precedence_than_arithmetic OK")
"###);
    assert_output(&out, r###"lower_precedence_than_arithmetic OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/walrus/nested_comp_walrus_accumulator.py`.
#[test]
fn test_gen_behavior_pep_walrus_nested_comp_walrus_accumulator() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "walrus"
# dimension = "behavior"
# case = "nested_comp_walrus_accumulator"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus that reads and writes the same enclosing name accumulates a running total across a comprehension: [(o := o + v) for v in range(5)] yields [0,1,3,6,10] and o == 10"""
# A walrus reading + writing the same enclosing name accumulates a running total.
outer2 = 0
inner_list = [(outer2 := outer2 + v) for v in range(5)]
# accumulates: 0, 0+1=1, 1+2=3, 3+3=6, 6+4=10
assert inner_list == [0, 1, 3, 6, 10], f"accumulate = {inner_list!r}"
assert outer2 == 10, f"outer2 = {outer2!r}"

print("nested_comp_walrus_accumulator OK")
"###);
    assert_output(&out, r###"nested_comp_walrus_accumulator OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/walrus/walrus_does_not_create_scope.py`.
#[test]
fn test_gen_behavior_pep_walrus_walrus_does_not_create_scope() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "walrus"
# dimension = "behavior"
# case = "walrus_does_not_create_scope"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus does not create a new scope: a walrus in an if-condition overwrites the existing module-level binding"""
# A walrus does not introduce a new scope; it overwrites the current binding.
outer = 99
if (outer := 42) > 0:
    pass
assert outer == 42, f"outer overwritten = {outer!r}"

print("walrus_does_not_create_scope OK")
"###);
    assert_output(&out, r###"walrus_does_not_create_scope OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/walrus/walrus_in_boolean_short_circuit.py`.
#[test]
fn test_gen_behavior_pep_walrus_walrus_in_boolean_short_circuit() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "walrus"
# dimension = "behavior"
# case = "walrus_in_boolean_short_circuit"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus on the right of `or` binds and short-circuits correctly: False or (z := fn(7)) is 7, z == 7, and fn was called exactly once"""
# A walrus on the right of `or` binds and short-circuits correctly.
calls: list = []
def fn(v: int) -> int:
    calls.append(v)
    return v

result = False or (z := fn(7))
assert result == 7, f"or walrus = {result!r}"
assert z == 7, f"z = {z!r}"
assert calls == [7], f"calls = {calls!r}"

print("walrus_in_boolean_short_circuit OK")
"###);
    assert_output(&out, r###"walrus_in_boolean_short_circuit OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/walrus/while_walrus_holds_last_value_after_loop.py`.
#[test]
fn test_gen_behavior_pep_walrus_while_walrus_holds_last_value_after_loop() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "walrus"
# dimension = "behavior"
# case = "while_walrus_holds_last_value_after_loop"
# subject = ":="
# kind = "semantic"
# xfail = "mamba walrus-in-while target reads 0 after loop exit; matches the legacy behavior.py mamba-xfail (val2 after loop = 0)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus in a while-condition reads from the enclosing scope and, after the loop exits on a falsy value, the target still holds that last assigned (falsy) value"""
# := in a while-condition binds each element; after the loop exits on the
# falsy 0, the target still holds that last assigned (falsy) value.
data2 = [10, 20, 0, 30]  # 0 is falsy -> loop stops
idx2 = 0
acc2 = 0
while idx2 < len(data2) and (val2 := data2[idx2]):
    acc2 += val2
    idx2 += 1
assert acc2 == 30, f"acc = {acc2!r}"  # 10 + 20, stops at 0
assert val2 == 0, f"val2 after loop = {val2!r}"  # last assigned value

print("while_walrus_holds_last_value_after_loop OK")
"###);
    assert_output(&out, r###"while_walrus_holds_last_value_after_loop OK
"###);
}
