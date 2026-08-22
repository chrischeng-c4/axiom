use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/core/generators/method_dispatch_in_generator_body.py`.
#[test]
fn test_gen_behavior_core_generators_method_dispatch_in_generator_body() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generators"
# dimension = "behavior"
# case = "method_dispatch_in_generator_body"
# subject = "generator method dispatch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""generator method dispatch: generator bodies can call bound/native methods."""


def direct_method_call():
    for c in "abc":
        yield c.upper()


def stored_bound_method_call():
    for c in "abc":
        method = c.upper
        yield method()


assert list(direct_method_call()) == ["A", "B", "C"]
assert list(stored_bound_method_call()) == ["A", "B", "C"]
assert list(c.upper() for c in "abc") == ["A", "B", "C"]

print("method_dispatch_in_generator_body OK")
"###);
    assert_output(&out, r###"method_dispatch_in_generator_body OK
"###);
}

/// Ported from `tests/cpython/behavior/core/generators/top_level_generator_expression_contract.py`.
#[test]
fn test_gen_behavior_core_generators_top_level_generator_expression_contract() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generators"
# dimension = "behavior"
# case = "top_level_generator_expression_contract"
# subject = "generator expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""generator expression: module-level expressions have generator object semantics."""

import copy


g = (i for i in [1])
assert type(g).__name__ == "generator"
assert iter(g) is g
first_value = next(g)
assert first_value == 1

g = (i for i in [1])
try:
    copy.copy(g)
    raise AssertionError("copy.copy must reject generator expressions")
except TypeError:
    pass

g = (i for i in [1])
try:
    copy.deepcopy(g)
    raise AssertionError("copy.deepcopy must reject generator expressions")
except TypeError:
    pass

seen = 0
first = next((seen := i) for i in [7])
assert first == 7
assert seen == 7

print("top_level_generator_expression_contract OK")
"###);
    assert_output(&out, r###"top_level_generator_expression_contract OK
"###);
}
