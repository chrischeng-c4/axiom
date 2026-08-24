use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/pep/604/isinstance_matches_member.py`.
#[test]
fn test_gen_behavior_pep_604_isinstance_matches_member() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "604"
# dimension = "behavior"
# case = "isinstance_matches_member"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "isinstance(x, int | str) raises TypeError on mamba (None second arg; project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: isinstance(x, int | str) is True for an int or a str member and False for a non-member (float)"""
import types

u = int | str
assert isinstance(1, u) is True
assert isinstance("a", u) is True
assert isinstance(1.5, u) is False

print("isinstance_matches_member OK")
"###);
    assert_output(&out, r###"isinstance_matches_member OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/604/issubclass_matches_member.py`.
#[test]
fn test_gen_behavior_pep_604_issubclass_matches_member() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "604"
# dimension = "behavior"
# case = "issubclass_matches_member"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | str` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: issubclass(int, int | str) is True; the union acts as the second arg to issubclass"""
import types

u = int | str
assert issubclass(int, u) is True
assert issubclass(str, u) is True
assert issubclass(float, u) is False
# A subclass of a member still matches (bool is a subclass of int).
assert issubclass(bool, u) is True

print("issubclass_matches_member OK")
"###);
    assert_output(&out, r###"issubclass_matches_member OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/604/none_pipe_int_is_optional.py`.
#[test]
fn test_gen_behavior_pep_604_none_pipe_int_is_optional() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "604"
# dimension = "behavior"
# case = "none_pipe_int_is_optional"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`None | int` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: None | int yields a UnionType whose __args__ is (NoneType, int) and whose repr is 'None | int'"""
import types

opt = None | int
assert isinstance(opt, types.UnionType)
assert opt.__args__ == (type(None), int)
assert repr(opt) == "None | int"
# It still works as an isinstance check over both members.
assert isinstance(None, opt) is True
assert isinstance(5, opt) is True
assert isinstance("a", opt) is False

print("none_pipe_int_is_optional OK")
"###);
    assert_output(&out, r###"none_pipe_int_is_optional OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/604/union_args_tuple.py`.
#[test]
fn test_gen_behavior_pep_604_union_args_tuple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "604"
# dimension = "behavior"
# case = "union_args_tuple"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | str` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: (int | str).__args__ is the ordered tuple (int, str) of the union members"""
import types

u = int | str
assert u.__args__ == (int, str)
# Order follows the operands, and a three-member union keeps all three.
assert (int | str | bytes).__args__ == (int, str, bytes)

print("union_args_tuple OK")
"###);
    assert_output(&out, r###"union_args_tuple OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/604/union_creates_uniontype.py`.
#[test]
fn test_gen_behavior_pep_604_union_creates_uniontype() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "604"
# dimension = "behavior"
# case = "union_creates_uniontype"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | str` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: int | str produces an instance of types.UnionType"""
import types

u = int | str
assert isinstance(u, types.UnionType), type(u).__name__

print("union_creates_uniontype OK")
"###);
    assert_output(&out, r###"union_creates_uniontype OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/604/union_dedupes_repeated_member.py`.
#[test]
fn test_gen_behavior_pep_604_union_dedupes_repeated_member() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "604"
# dimension = "behavior"
# case = "union_dedupes_repeated_member"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | int` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: a union of one type with itself collapses to that bare type: int | int is int"""
import types

# A union of a single repeated member collapses back to the bare type.
assert (int | int) is int
# A repeated member inside a larger union is deduplicated, preserving order.
assert (int | str | int).__args__ == (int, str)

print("union_dedupes_repeated_member OK")
"###);
    assert_output(&out, r###"union_dedupes_repeated_member OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/604/union_repr_uses_pipe.py`.
#[test]
fn test_gen_behavior_pep_604_union_repr_uses_pipe() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "604"
# dimension = "behavior"
# case = "union_repr_uses_pipe"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | str` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: the repr of int | str renders the members joined by ' | ': 'int | str'"""
import types

assert repr(int | str) == "int | str"
assert str(int | str) == "int | str"
# A three-member union joins every member with ' | '.
assert repr(int | str | bytes) == "int | str | bytes"

print("union_repr_uses_pipe OK")
"###);
    assert_output(&out, r###"union_repr_uses_pipe OK
"###);
}
