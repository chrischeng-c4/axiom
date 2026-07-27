use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/pep/union_types/isinstance_matches_member.py`.
#[test]
fn test_gen_behavior_pep_union_types_isinstance_matches_member() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
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

/// Ported from `tests/cpython/behavior/pep/union_types/issubclass_matches_member.py`.
#[test]
fn test_gen_behavior_pep_union_types_issubclass_matches_member() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
# dimension = "behavior"
# case = "issubclass_matches_member"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | str` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: issubclass(x, int | str) is True for a member or a member's subclass (bool <: int) and False for a non-member (list)"""
import types

u = int | str
assert issubclass(int, u) is True
assert issubclass(str, u) is True
# A subclass of a member still matches (bool is a subclass of int).
assert issubclass(bool, u) is True
assert issubclass(list, u) is False

print("issubclass_matches_member OK")
"###);
    assert_output(&out, r###"issubclass_matches_member OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/union_types/none_pipe_int_is_optional.py`.
#[test]
fn test_gen_behavior_pep_union_types_none_pipe_int_is_optional() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
# dimension = "behavior"
# case = "none_pipe_int_is_optional"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | None` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: int | None is the Optional idiom: None and an int member both isinstance-match, a str does not"""
import types

nullable = int | None
assert isinstance(None, nullable) is True
assert isinstance(42, nullable) is True
assert isinstance("hi", nullable) is False

print("none_pipe_int_is_optional OK")
"###);
    assert_output(&out, r###"none_pipe_int_is_optional OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/union_types/type_of_union_is_uniontype.py`.
#[test]
fn test_gen_behavior_pep_union_types_type_of_union_is_uniontype() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
# dimension = "behavior"
# case = "type_of_union_is_uniontype"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | float` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: type(int | float) is types.UnionType"""
import types

ut = int | float
assert type(ut) is types.UnionType, repr(type(ut))

print("type_of_union_is_uniontype OK")
"###);
    assert_output(&out, r###"type_of_union_is_uniontype OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/union_types/union_args_tuple.py`.
#[test]
fn test_gen_behavior_pep_union_types_union_args_tuple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
# dimension = "behavior"
# case = "union_args_tuple"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | str` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: (int | str | bytes).__args__ holds exactly the member types {int, str, bytes}"""
import types

args = (int | str | bytes).__args__
assert isinstance(args, tuple)
assert set(args) == {int, str, bytes}, args

print("union_args_tuple OK")
"###);
    assert_output(&out, r###"union_args_tuple OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/union_types/union_creates_uniontype.py`.
#[test]
fn test_gen_behavior_pep_union_types_union_creates_uniontype() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
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

/// Ported from `tests/cpython/behavior/pep/union_types/union_dedupes_repeated_member.py`.
#[test]
fn test_gen_behavior_pep_union_types_union_dedupes_repeated_member() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
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

dup = int | int
assert dup is int, repr(dup)

print("union_dedupes_repeated_member OK")
"###);
    assert_output(&out, r###"union_dedupes_repeated_member OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/union_types/union_flattens_nested.py`.
#[test]
fn test_gen_behavior_pep_union_types_union_flattens_nested() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
# dimension = "behavior"
# case = "union_flattens_nested"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | str` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: nested unions flatten: (int | str) | float and int | (str | float) both contain float and share __args__"""
import types

u1 = (int | str) | float
u2 = int | (str | float)
assert isinstance(3.14, u1) is True
assert isinstance(3.14, u2) is True
# Both nestings flatten to the same set of members.
assert set(u1.__args__) == {int, str, float}
assert set(u2.__args__) == {int, str, float}

print("union_flattens_nested OK")
"###);
    assert_output(&out, r###"union_flattens_nested OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/union_types/union_in_function_annotation.py`.
#[test]
fn test_gen_behavior_pep_union_types_union_in_function_annotation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
# dimension = "behavior"
# case = "union_in_function_annotation"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | str | None` annotation evaluates to None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: a `X | Y | None` parameter annotation does not change runtime call behavior; the function dispatches on isinstance over the union"""
import types


def process(val: int | str | None) -> str:
    if val is None:
        return "none"
    return str(val)


assert process(42) == "42"
assert process("hi") == "hi"
assert process(None) == "none"

print("union_in_function_annotation OK")
"###);
    assert_output(&out, r###"union_in_function_annotation OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/union_types/union_interops_with_typing_union.py`.
#[test]
fn test_gen_behavior_pep_union_types_union_interops_with_typing_union() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
# dimension = "behavior"
# case = "union_interops_with_typing_union"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | str` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: isinstance over an X | Y union and over typing.Union[X, Y] agree on the same members"""
import types
from typing import Union

pipe = int | str
classic = Union[int, str]
for value in (42, "hi", 3.14):
    assert isinstance(value, pipe) == isinstance(value, classic), value

print("union_interops_with_typing_union OK")
"###);
    assert_output(&out, r###"union_interops_with_typing_union OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/union_types/union_is_commutative.py`.
#[test]
fn test_gen_behavior_pep_union_types_union_is_commutative() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
# dimension = "behavior"
# case = "union_is_commutative"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | str` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: operand order does not matter: int | str equals str | int"""
import types

assert (int | str) == (str | int)

print("union_is_commutative OK")
"###);
    assert_output(&out, r###"union_is_commutative OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/union_types/union_multi_member_isinstance.py`.
#[test]
fn test_gen_behavior_pep_union_types_union_multi_member_isinstance() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "union_types"
# dimension = "behavior"
# case = "union_multi_member_isinstance"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`int | str | float | bytes` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: a four-member union int | str | float | bytes isinstance-matches each member type and rejects a non-member (list)"""
import types

multi = int | str | float | bytes
assert isinstance(42, multi) is True
assert isinstance("hi", multi) is True
assert isinstance(3.14, multi) is True
assert isinstance(b"x", multi) is True
assert isinstance([1], multi) is False

print("union_multi_member_isinstance OK")
"###);
    assert_output(&out, r###"union_multi_member_isinstance OK
"###);
}
