use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/pep/557/make_dataclass_bases_and_options.py`.
#[test]
fn test_gen_behavior_pep_557_make_dataclass_bases_and_options() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "make_dataclass_bases_and_options"
# subject = "dataclasses.make_dataclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "make_dataclass.py"
# status = "filled"
# ///
"""dataclasses.make_dataclass: make_dataclass bases= makes the generated class inherit, and decorator options like frozen=True pass through"""
from dataclasses import make_dataclass, FrozenInstanceError


class Base1:
    pass


class Base2:
    pass


E = make_dataclass("E", [("x", int)], bases=(Base1, Base2))
e = E(5)
assert isinstance(e, E)
assert isinstance(e, Base1)
assert isinstance(e, Base2)
assert e.x == 5

F = make_dataclass("F", [("x", int)], frozen=True)
f = F(3)
_raised = False
try:
    f.x = 4
except FrozenInstanceError:
    _raised = True
assert _raised, "expected FrozenInstanceError"
print("make_dataclass_bases_and_options OK")
"###);
    assert_output(&out, r###"make_dataclass_bases_and_options OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/557/make_dataclass_field_spec_forms.py`.
#[test]
fn test_gen_behavior_pep_557_make_dataclass_field_spec_forms() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "make_dataclass_field_spec_forms"
# subject = "dataclasses.make_dataclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "make_dataclass.py"
# status = "filled"
# ///
"""dataclasses.make_dataclass: make_dataclass field specs may be a bare name, (name, type), or (name, type, field(...))"""
from dataclasses import make_dataclass, field

D = make_dataclass(
    "D",
    [
        "a",  # bare name
        ("b", int),  # name + type
        ("c", int, field(default=9)),  # name + type + field
    ],
)
d = D(1, 2)
assert (d.a, d.b, d.c) == (1, 2, 9)
print("make_dataclass_field_spec_forms OK")
"###);
    assert_output(&out, r###"make_dataclass_field_spec_forms OK
"###);
}
