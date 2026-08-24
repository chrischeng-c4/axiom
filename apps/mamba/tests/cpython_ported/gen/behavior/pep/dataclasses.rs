use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/pep/dataclasses/eq_false_uses_identity.py`.
#[test]
fn test_gen_behavior_pep_dataclasses_eq_false_uses_identity() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "dataclasses"
# dimension = "behavior"
# case = "eq_false_uses_identity"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba does not honor eq=False (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "behavior.py"
# status = "filled"
# ///
"""dataclasses.dataclass: @dataclass(eq=False) suppresses value __eq__ so two equal-field instances compare by identity (are not ==)"""
from dataclasses import dataclass


@dataclass(eq=False)
class NoEq:
    val: int


a = NoEq(1)
b = NoEq(1)
assert a is not b, "distinct instances"
assert a != b, "no eq -> identity comparison, equal fields still compare !="
assert a == a, "identity comparison: an instance equals itself"
print("eq_false_uses_identity OK")
"###);
    assert_output(&out, r###"eq_false_uses_identity OK
"###);
}
